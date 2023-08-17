use std::{
	io::{BufRead, BufReader, Write},
	path::PathBuf,
	process::{Command, Stdio},
	sync::Arc,
};

use itertools::Itertools;
use rustc_hash::FxHashMap;
use serde::{
	de::{DeserializeSeed, Error, IgnoredAny, Visitor},
	Deserializer,
};
use tempfile::Builder;

use crate::{
	data::serde::SerdeValueVisitor,
	diagnostics::{FileError, InternalError, ShackleError},
	value::{Polarity, Set, Value},
	Message, Program, Status, Type,
};

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> Result<(), ShackleError>>(
		&mut self,
		msg_callback: F,
	) -> Result<Status, ShackleError> {
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
			ShackleError::from(FileError {
				file: tmp_path.clone(),
				message: format!("unable to write model to temporary file: {}", err),
				other: vec![],
			})
		};
		let file_mut = tmpfile.as_file_mut();
		self.write(file_mut).map_err(write_err)?;
		for (name, ty) in &self._input_types {
			let val = if let Some(val) = self._input_data.get(name) {
				val
			} else if ty.is_opt() {
				&Value::Absent
			} else {
				// TODO: throw error that non-opt thing is not defined
				todo!()
			};
			write!(file_mut, "{name} = ").map_err(write_err)?;
			write_legacy_value(file_mut, ty, val).map_err(write_err)?;
			writeln!(file_mut, ";").map_err(write_err)?;
		}

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
						.deserialize_map(SerdeMessageVisitor(&self._output_types))
						.map_err(|e| {
							ShackleError::from_serde_json(e, &Arc::new(line.clone()).into())
						})? {
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

fn write_legacy_value<W: Write>(out: &mut W, ty: &Type, val: &Value) -> Result<(), std::io::Error> {
	let is_opt = ty.is_opt();
	if is_opt {
		if val == &Value::Absent {
			write!(out, "(false, ")?;
			write_legacy_dummy_value(out, ty)?;
			return write!(out, ")");
		} else {
			write!(out, "(true, ")?;
		}
	}
	match val {
		Value::Absent => unreachable!("found absent assigned to non-opt parameter"),
		Value::Infinity(p) => write!(
			out,
			"{}infinity",
			if p == &Polarity::Neg { "-" } else { "" }
		)?,
		Value::Boolean(v) => write!(out, "{}", v)?,
		Value::Integer(v) => write!(out, "{}", v)?,
		Value::Float(v) => write!(out, "{}", v)?,
		Value::String(v) => write!(out, "\"{}\"", v)?,
		Value::Enum(v) => write!(out, "{}", v.int_val())?,
		Value::Ann(name, args) => {
			if args.is_empty() {
				write!(out, "{name}")?
			} else {
				write!(out, "{name}({})", args.iter().format(", "))?
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
				write!(out, "[]")?;
			} else if v.dim() == 1 {
				let first = extract_idx(&(v.iter().next().unwrap().0[0]));
				write!(out, "[{first}:")?;
				for el in v.iter().map(|(_, el)| el) {
					write_legacy_value(out, element, el)?;
					write!(out, ", ")?;
				}
				write!(out, "]")?;
			} else {
				write!(out, "[")?;
				for (ii, el) in v.iter() {
					write!(out, "({}): ", ii.iter().map(extract_idx).format(","))?;
					write_legacy_value(out, element, el)?;
					write!(out, ", ")?;
				}
				write!(out, "]")?;
			}
		}
		Value::Set(s) => match s {
			Set::EnumRangeList(s) => write!(
				out,
				"{}",
				s.iter().format_with(" union ", |elt, f| f(&format_args!(
					"{}..{}",
					elt.start().int_val(),
					elt.end().int_val()
				)))
			)?,
			Set::IntRangeList(s) => write!(
				out,
				"{}",
				s.iter().format_with(" union ", |elt, f| f(&format_args!(
					"{}..{}",
					elt.start(),
					elt.end()
				)))
			)?,
			Set::FloatRangeList(s) => write!(
				out,
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
			write!(out, "(")?;
			for (ty, val) in tys.iter().zip_eq(v) {
				write_legacy_value(out, ty, val)?;
				write!(out, ",")?
			}
			write!(out, ")")?;
		}
		Value::Record(v) => {
			let Type::Record(_, tys) = ty else {
				unreachable!()
			};
			write!(out, "(")?;
			for (ty, val) in tys.iter().map(|(_, t)| t).zip_eq(v.iter().map(|(_, v)| v)) {
				write_legacy_value(out, ty, val)?;
				write!(out, ",")?
			}
			write!(out, ")")?;
		}
	}
	if is_opt {
		write!(out, ")")?
	}
	Ok(())
}

fn write_legacy_dummy_value<W: Write>(out: &mut W, ty: &Type) -> Result<(), std::io::Error> {
	match ty {
		Type::Boolean(_) => write!(out, "true"),
		Type::Integer(_) => write!(out, "0"),
		Type::Float(_) => write!(out, "0.0"),
		Type::Enum(_, _) => write!(out, "1"),
		Type::String(_) => write!(out, "\"\""),
		Type::Annotation(_) => write!(out, "empty_annotation"),
		Type::Array {
			opt: _,
			dim: _,
			element: _,
		} => write!(out, "[]"),
		Type::Set(_, _) => write!(out, "{{}}"),
		Type::Tuple(_, tys) => {
			write!(out, "(")?;
			for ty in tys.iter() {
				write_legacy_dummy_value(out, ty)?;
				write!(out, ",")?;
			}
			write!(out, ")")
		}
		Type::Record(_, tys) => {
			write!(out, "(")?;
			for (_, ty) in tys.iter() {
				write_legacy_dummy_value(out, ty)?;
				write!(out, ",")?;
			}
			write!(out, ")")
		}
	}
}

struct SerdeMessageVisitor<'a>(pub &'a FxHashMap<String, Type>);

enum LegacyOutput<'a> {
	Status(Status),
	Msg(Message<'a>),
	Error(ShackleError),
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
						return Err(Error::duplicate_field("type"));
					}
					msg_type = Some(map.next_value()?);
				}
				"message" => {
					if message.is_some() {
						return Err(Error::duplicate_field("message"));
					}
					message = Some(map.next_value::<&str>()?);
				}
				"output" => {
					if solution.is_some() {
						return Err(Error::duplicate_field("output"));
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
						return Err(Error::duplicate_field("statistics"));
					}
					statistics = Some(map.next_value()?);
				}
				"status" => {
					if status.is_some() {
						return Err(Error::duplicate_field("status"));
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
							return Err(Error::unknown_variant(
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
				_ => return Err(Error::unknown_field(k, FIELDS)),
			}
		}

		match msg_type {
			Some("solution") => match solution {
				None => Err(Error::missing_field("output")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Solution(x))),
			},
			Some("statistics") => match statistics {
				None => Err(Error::missing_field("statistics")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Statistic(x))),
			},
			Some("error") => match message {
				None => Err(Error::missing_field("message")),
				Some(msg) => Ok(LegacyOutput::Error(
					InternalError::new(format!("minizinc error: {msg}")).into(),
				)),
			},
			Some("warning") => match message {
				None => Err(Error::missing_field("message")),
				Some(msg) => Ok(LegacyOutput::Msg(Message::Warning(msg))),
			},
			Some("status") => match status {
				None => Err(Error::missing_field("status")),
				Some(s) => Ok(LegacyOutput::Status(s)),
			},
			None => Err(Error::missing_field("type")),
			Some(ty) => Err(Error::unknown_variant(ty, &["statistics", "status", ""])),
		}
	}
}

#[derive(Clone)]
struct SerdeOutputVisitor<'a>(pub &'a FxHashMap<String, Type>);

impl<'de, 'a> Visitor<'de> for SerdeOutputVisitor<'a> {
	type Value = Result<FxHashMap<&'de str, Value>, ShackleError>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "minizinc output assignment")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let type_map = self.0;

		let mut sol = FxHashMap::default();
		sol.reserve(type_map.len());
		while let Some(k) = map.next_key()? {
			if let Some(ty) = type_map.get(k) {
				let v = map.next_value_seed(SerdeValueVisitor(ty))?;
				match v.resolve_value(ty) {
					Ok(v) => {
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
	type Value = Result<FxHashMap<&'de str, Value>, ShackleError>;

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
			None => Err(Error::missing_field(self.name)),
			Some(x) => Ok(x),
		}
	}
}
