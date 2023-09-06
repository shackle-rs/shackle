use std::{
	fmt::Display,
	io::{BufRead, BufReader, Write},
	ops::Deref,
	path::PathBuf,
	process::{Command, Stdio},
	sync::Arc,
};

use itertools::Itertools;
use rustc_hash::FxHashMap;
use serde::{
	de::{DeserializeSeed, Error as SerdeError, IgnoredAny, Visitor},
	Deserializer,
};
use tempfile::Builder;

use crate::{
	data::serde::SerdeValueVisitor,
	error::{FileError, InternalError},
	value::{Array, EnumInner, EnumRangeInclusive, EnumValue, Index, Polarity, Set, Value},
	Enum, Error, Message, OptType, Program, Result, Status, Type,
};

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> Result<()>>(&mut self, msg_callback: F) -> Result<Status> {
		// Create new (temporary) file used as input for the interpreter
		let tmpfile = Builder::new().suffix(".shackle.mzn").tempfile();
		let mut tmpfile = match tmpfile {
			Err(err) => {
				return Err(FileError {
					file: PathBuf::from("tempfile"),
					message: err.to_string(),
					other: Vec::new(),
				}
				.into());
			}
			Ok(file) => file,
		};
		let tmp_path = tmpfile.path().to_owned();
		let write_err = |err| {
			Error::from(FileError {
				file: tmp_path.clone(),
				message: format!("unable to write model to temporary file: {}", err),
				other: vec![],
			})
		};
		// Write content to file
		let file_mut = tmpfile.as_file_mut();
		// Write model to file
		self.write(file_mut).map_err(write_err)?;
		// Write data to file
		for (name, ty) in &self.input_types {
			let val = if let Some(val) = self.input_data.get(name) {
				val
			} else if ty.is_opt() {
				&Value::Absent
			} else {
				todo!("add new error type - {} is not initialized", name)
			};
			writeln!(file_mut, "{name} = {};", LegacyValue { val, ty }).map_err(write_err)?;
		}
		for e in &self.legacy_enums {
			if e.state.lock().unwrap().deref() == &EnumInner::NoDefinition {
				todo!("add new error type - {} is not initialized", e.name())
			}
			writeln!(file_mut, "{};", LegacyEnum(e)).map_err(write_err)?;
		}

		// Construct command for the MiniZinc intepreter
		let mut cmd = Command::new("minizinc");
		cmd.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::inherit())
			.arg(&tmp_path)
			.args([
				"--output-mode",
				"json",
				"--json-stream",
				"--ignore-stdlib",
				"--output-objective",
				"--output-output-item",
				"--intermediate-solutions",
				"--solver",
				self.slv.ident.as_str(),
			]);
		if let Some(time_limit) = self.time_limit {
			cmd.args(["--time-limit", time_limit.as_millis().to_string().as_str()]);
		}
		if self.enable_stats {
			cmd.arg("--statistics");
		}

		let mut child = cmd.spawn().unwrap(); // TODO: fix unwrap
		let stdout = child.stdout.take().unwrap();

		let mut status = Status::Unknown;
		for line in BufReader::new(stdout).lines() {
			match line {
				Err(e) => {
					return Err(InternalError::new(format!(
						"Unable to read interpreter output: “{e}”"
					))
					.into())
				}
				Ok(line) => {
					match serde_json::Deserializer::from_str(&line)
						.deserialize_map(SerdeMessageVisitor(&self.output_types))
						.map_err(|e| Error::from_serde_json(e, &Arc::new(line.clone()).into()))?
					{
						LegacyOutput::Status(s) => status = s,
						LegacyOutput::Msg(msg) => {
							if let Message::Solution(_) = msg {
								if status == Status::Unknown {
									status = Status::Satisfied
								}
							}
							msg_callback(&msg)?
						}
						LegacyOutput::Error(err) => return Err(err),
					}
				}
			}
		}
		match child.wait() {
			Ok(code) => {
				if !code.success() {
					log::warn!(
						"The MiniZinc process terminated with exit code {}",
						code.code().unwrap()
					)
				};
				Ok(status)
			}
			Err(e) => Err(InternalError::new(format!("process error: {}", e)).into()),
		}
	}
}

struct LegacyValue<'a> {
	val: &'a Value,
	ty: &'a Type,
}

impl<'a> Display for LegacyValue<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let val = self.val;
		let ty = self.ty;

		let is_opt = ty.is_opt();
		if is_opt {
			if val == &Value::Absent {
				return write!(f, "(false, {})", DummyValue(ty));
			} else {
				write!(f, "(true, ")?;
			}
		}
		match val {
			Value::Absent => unreachable!("found absent assigned to non-opt parameter"),
			Value::Infinity(p) => {
				write!(f, "{}infinity", if p == &Polarity::Neg { "-" } else { "" })?
			}
			Value::Boolean(v) => write!(f, "{}", v)?,
			Value::Integer(v) => write!(f, "{}", v)?,
			Value::Float(v) => write!(f, "{}", v)?,
			Value::String(v) => write!(f, "\"{}\"", v)?,
			Value::Enum(v) => write!(f, "{}", v.int_val())?,
			Value::Ann(name, args) => {
				if args.is_empty() {
					write!(f, "{name}")?
				} else {
					write!(f, "{name}({})", args.iter().format(", "))?
				}
			}
			Value::Array(v) => {
				let Type::Array {
					opt: _,
					dim: _,
					element,
				} = ty
				else {
					unreachable!()
				};
				let extract_idx = |x: &Value| match x {
					Value::Integer(i) => *i,
					Value::Enum(v) => v.int_val() as i64,
					_ => unreachable!(),
				};
				if v.is_empty() {
					write!(f, "[]")?;
				} else if v.dim() == 1 {
					let first = extract_idx(&(v.iter().next().unwrap().0[0]));
					write!(
						f,
						"[{first}: {}]",
						v.iter()
							.map(|(_, val)| LegacyValue { val, ty: element })
							.format(",")
					)?;
				} else {
					write!(
						f,
						"[{}]",
						v.iter().format_with(",", |(ii, val), f| f(&format_args!(
							"({}): {}",
							ii.iter().map(extract_idx).format(","),
							LegacyValue { val, ty: element }
						)))
					)?;
				}
			}
			Value::Set(s) => match s {
				Set::Enum(s) => write!(
					f,
					"{}",
					s.iter().format_with(" union ", |elt, f| f(&format_args!(
						"{}..{}",
						elt.start().int_val(),
						elt.end().int_val()
					)))
				)?,
				Set::Int(s) => write!(
					f,
					"{}",
					s.iter().format_with(" union ", |elt, f| f(&format_args!(
						"{}..{}",
						elt.start(),
						elt.end()
					)))
				)?,
				Set::Float(s) => write!(
					f,
					"{}",
					s.iter().format_with(" union ", |elt, f| f(&format_args!(
						"{}..{}",
						elt.start(),
						elt.end()
					)))
				)?,
			},
			Value::Tuple(v) => {
				let Type::Tuple(_, tys) = ty else {
					unreachable!()
				};
				write!(
					f,
					"({}{})",
					tys.iter()
						.zip_eq(v)
						.map(|(ty, val)| LegacyValue { val, ty })
						.format(","),
					if tys.len() == 1 { "," } else { "" }
				)?;
			}
			Value::Record(v) => {
				let Type::Record(_, tys) = ty else {
					unreachable!()
				};
				write!(
					f,
					"({}{})",
					tys.iter()
						.map(|(_, t)| t)
						.zip_eq(v.iter().map(|(_, v)| v))
						.map(|(ty, val)| LegacyValue { val, ty })
						.format(","),
					if tys.len() == 1 { "," } else { "" }
				)?;
			}
		}
		if is_opt {
			write!(f, ")")?
		}
		Ok(())
	}
}
struct DummyValue<'a>(&'a Type);
impl<'a> Display for DummyValue<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let ty = self.0;
		match ty {
			Type::Boolean(_) => write!(f, "true"),
			Type::Integer(_) => write!(f, "0"),
			Type::Float(_) => write!(f, "0.0"),
			Type::Enum(_, _) => write!(f, "1"),
			Type::String(_) => write!(f, "\"\""),
			Type::Annotation(_) => write!(f, "empty_annotation"),
			Type::Array {
				opt: _,
				dim: _,
				element: _,
			} => write!(f, "[]"),
			Type::Set(_, _) => write!(f, "{{}}"),
			Type::Tuple(_, tys) => {
				write!(
					f,
					"({}{})",
					tys.iter().map(DummyValue).format(","),
					if tys.len() == 1 { "," } else { "" }
				)
			}
			Type::Record(_, tys) => {
				write!(
					f,
					"({}{})",
					tys.iter().map(|(_, ty)| DummyValue(ty)).format(","),
					if tys.len() == 1 { "," } else { "" }
				)
			}
		}
	}
}

struct LegacyEnum<'a>(&'a Enum);
impl<'a> Display for LegacyEnum<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		const INT: Type = Type::Integer(crate::OptType::NonOpt);
		write!(
			f,
			"mzn_enum_{} = [{}]",
			self.0.name(),
			self.0
				.lock()
				.iter()
				.format_with(",", |(name, idxs, _), f| f(&format_args!(
					"({:?}, [{}])",
					name,
					idxs.iter().format_with(",", |idx, g| g(&format_args!(
						"(0, {}..{})",
						LegacyValue {
							val: &idx.start(),
							ty: &INT
						},
						LegacyValue {
							val: &idx.end(),
							ty: &INT
						}
					)))
				)))
		)
	}
}

struct SerdeMessageVisitor<'a>(pub &'a FxHashMap<Arc<str>, Type>);

enum LegacyOutput<'a> {
	Status(Status),
	Msg(Message<'a>),
	Error(Error),
}

impl<'de, 'a> Visitor<'de> for SerdeMessageVisitor<'a> {
	type Value = LegacyOutput<'de>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "minizinc interpreter message")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		const FIELDS: &[&str] = &[
			"type",
			"statistics",
			"output",
			"status",
			"message",
			"location",
			"stack",
			"sections",
			"what",
		];
		let type_map = self.0;

		let mut msg_type = None;
		let mut statistics = None;
		let mut message = None;
		let mut status = None;
		let mut solution = None;

		while let Some(k) = map.next_key::<&str>()? {
			match k {
				"type" => {
					if msg_type.is_some() {
						return Err(SerdeError::duplicate_field("type"));
					}
					msg_type = Some(map.next_value()?);
				}
				"message" => {
					if message.is_some() {
						return Err(SerdeError::duplicate_field("message"));
					}
					message = Some(map.next_value::<&str>()?);
				}
				"output" => {
					if solution.is_some() {
						return Err(SerdeError::duplicate_field("output"));
					}
					match map.next_value_seed(SerdeWrappedName {
						name: "json",
						seed: SerdeOutputVisitor(type_map),
					})? {
						Ok(sol) => solution = Some(sol),
						Err(e) => return Ok(LegacyOutput::Error(e)),
					}
				}
				"statistics" => {
					if statistics.is_some() {
						return Err(SerdeError::duplicate_field("statistics"));
					}
					statistics = Some(map.next_value()?);
				}
				"status" => {
					if status.is_some() {
						return Err(SerdeError::duplicate_field("status"));
					}
					status = Some(match map.next_value()? {
						"ALL_SOLUTIONS" => Status::AllSolutions,
						"OPTIMAL_SOLUTION" => Status::Optimal,
						"UNSATISFIABLE" => Status::Infeasible,
						"UNBOUNDED" => Status::Infeasible, // TODO: Should this be seperate?
						"UNSAT_OR_UNBOUNDED" => Status::Infeasible,
						"UNKNOWN" => Status::Unknown,
						"ERROR" => {
							return Ok(LegacyOutput::Error(
								InternalError::new("Error occurred, but no message was provided")
									.into(),
							))
						} // TODO: Probably should do something, but we now rely on another error message type
						s => {
							return Err(SerdeError::unknown_variant(
								s,
								&[
									"ALL_SOLUTIONS",
									"OPTIMAL_SOLUTION",
									"UNSATISFIABLE",
									"UNBOUNDED",
									"UNSAT_OR_UNBOUNDED",
									"UNKNOWN",
									"ERROR",
								],
							));
						}
					})
				}
				"location" | "stack" | "sections" | "what" => {
					map.next_value::<IgnoredAny>()?; // TODO: parse additional error/warning information
				}
				_ => return Err(SerdeError::unknown_field(k, FIELDS)),
			}
		}

		match msg_type {
			Some("solution") => match solution {
				None => Err(SerdeError::missing_field("output")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Solution(x))),
			},
			Some("statistics") => match statistics {
				None => Err(SerdeError::missing_field("statistics")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Statistic(x))),
			},
			Some("error") => match message {
				None => Err(SerdeError::missing_field("message")),
				Some(msg) => Ok(LegacyOutput::Error(
					InternalError::new(format!("minizinc error: {msg}")).into(),
				)),
			},
			Some("warning") => match message {
				None => Err(SerdeError::missing_field("message")),
				Some(msg) => Ok(LegacyOutput::Msg(Message::Warning(msg))),
			},
			Some("status") => match status {
				None => Err(SerdeError::missing_field("status")),
				Some(s) => Ok(LegacyOutput::Status(s)),
			},
			None => Err(SerdeError::missing_field("type")),
			Some(ty) => Err(SerdeError::unknown_variant(
				ty,
				&["statistics", "status", ""],
			)),
		}
	}
}

#[derive(Clone)]
struct SerdeOutputVisitor<'a>(pub &'a FxHashMap<Arc<str>, Type>);

impl<'de, 'a> Visitor<'de> for SerdeOutputVisitor<'a> {
	type Value = Result<FxHashMap<&'de str, Value>, Error>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "minizinc output assignment")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let type_map = self.0;

		let mut sol = FxHashMap::default();
		sol.reserve(type_map.len());
		while let Some(k) = map.next_key()? {
			if let Some(ty) = type_map.get(k) {
				let out_type = ty.type_erase();
				let v = map.next_value_seed(SerdeValueVisitor(&out_type))?;
				match v.resolve_value(&out_type) {
					Ok(v) => {
						let v = v.reverse_type_erase(ty);
						sol.insert(k, v);
					}
					Err(e) => return Ok(Err(e)),
				}
			} else {
				map.next_value::<IgnoredAny>()?; // Ignore unknown
			}
		}
		Ok(Ok(sol))
	}
}

impl<'a, 'de> DeserializeSeed<'de> for SerdeOutputVisitor<'a> {
	type Value = Result<FxHashMap<&'de str, Value>, Error>;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_map(self)
	}
}

struct SerdeWrappedName<X: Clone> {
	name: &'static str,
	seed: X,
}

impl<'de, X: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for SerdeWrappedName<X> {
	type Value = X::Value;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_map(self)
	}
}

impl<'de, X: DeserializeSeed<'de> + Clone> Visitor<'de> for SerdeWrappedName<X> {
	type Value = X::Value;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "map with {} identifier", self.name)
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let mut ret = None;

		while let Some(k) = map.next_key::<&str>()? {
			if k == self.name {
				ret = Some(map.next_value_seed(self.seed.clone())?);
			} else {
				map.next_value::<IgnoredAny>()?; // Ignore unknown
			}
		}
		match ret {
			None => Err(SerdeError::missing_field(self.name)),
			Some(x) => Ok(x),
		}
	}
}

impl Type {
	fn type_erase(&self) -> Type {
		let ty = match self {
			Type::Boolean(_) => Type::Boolean(OptType::NonOpt),
			Type::Integer(_) => Type::Integer(OptType::NonOpt),
			Type::Float(_) => Type::Float(OptType::NonOpt),
			Type::Enum(_, _) => Type::Integer(OptType::NonOpt),
			Type::String(_) => Type::String(OptType::NonOpt),
			Type::Annotation(_) => Type::Annotation(OptType::NonOpt),
			Type::Array {
				opt: _,
				dim,
				element,
			} => Type::Array {
				opt: OptType::NonOpt,
				dim: dim.iter().map(|t| t.type_erase()).collect(),
				element: element.type_erase().into(),
			},
			Type::Set(_, elt) => Type::Set(OptType::NonOpt, elt.type_erase().into()),
			Type::Tuple(_, elts) => Type::Tuple(
				OptType::NonOpt,
				elts.iter().map(|t| t.type_erase()).collect(),
			),
			Type::Record(_, elts) => Type::Tuple(
				OptType::NonOpt,
				elts.iter().map(|(_, t)| t.type_erase()).collect(),
			),
		};
		if self.is_opt() {
			Type::Tuple(
				OptType::NonOpt,
				Arc::new([Type::Boolean(OptType::NonOpt), ty]),
			)
		} else {
			ty
		}
	}
}

impl Value {
	fn reverse_type_erase(self, ty: &Type) -> Value {
		let mut v = self;
		if ty.is_opt() {
			let Value::Tuple(mut tup) = v else {
				panic!("expected opt type tuple")
			};
			debug_assert_eq!(tup.len(), 2);
			let Value::Boolean(occurs) = tup[0] else {
				panic!("expected opt type tuple")
			};
			if !occurs {
				return Value::Absent;
			}
			v = tup.pop().unwrap();
		}
		match ty {
			Type::Enum(_, e) => {
				let Value::Integer(pos) = v else {
					panic!("expected integer value")
				};
				EnumValue::from_enum_and_pos(e.clone(), pos as usize).into()
			}
			Type::Array {
				opt: _,
				dim,
				element,
			} => {
				let Value::Array(x) = v else {
					panic!("expected array");
				};
				let indices = x
					.indices
					.iter()
					.cloned()
					.zip(dim.iter())
					.map(|(idx, t)| {
						if let Type::Enum(_, e) = t {
							let Index::Integer(r) = idx else {
								panic!("expected integer index")
							};
							Index::Enum(EnumRangeInclusive::from_enum_and_positions(
								e.clone(),
								*r.start() as usize,
								*r.end() as usize,
							))
						} else {
							idx
						}
					})
					.collect();
				let members = if matches!(
					**element,
					Type::Enum(_, _) | Type::Tuple(_, _) | Type::Record(_, _) | Type::Set(_, _)
				) {
					x.members
						.iter()
						.cloned()
						.map(|elt| elt.reverse_type_erase(element))
						.collect()
				} else {
					x.members
				};
				Array { indices, members }.into()
			}
			Type::Tuple(_, tys) => {
				let Value::Tuple(tup) = v else {
					panic!("expected tuple")
				};
				Value::Tuple(
					tup.into_iter()
						.zip_eq(tys.iter())
						.map(|(elt, t)| elt.reverse_type_erase(t))
						.collect(),
				)
			}
			Type::Record(_, tys) => {
				let Value::Tuple(tup) = v else {
					panic!("expected tuple")
				};
				Value::Record(
					tup.into_iter()
						.zip_eq(tys.iter())
						.map(|(elt, (n, t))| (n.clone(), elt.reverse_type_erase(t)))
						.collect(),
				)
			}
			_ => v,
		}
	}
}
