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
	Deserializer,
	de::{DeserializeSeed, Error as SerdeError, IgnoredAny, Visitor},
};
use shackle_diagnostics::{
	FileError, InternalError, SourceFile,
	error::{InvalidObjectData, MissingData},
};
use tempfile::Builder;

use crate::{
	CompileTarget, Enum, Error, Message, OptType, Program, Result, Status, Type,
	data::serde::SerdeValueVisitor,
	value::{Array, EnumInner, EnumRangeInclusive, EnumValue, Index, Polarity, Set, Value},
};

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> Result<()>>(&mut self, msg_callback: F) -> Result<Status> {
		// Every input needs a value, from the data or (when optional) absent
		let mut missing = self
			.input_types
			.iter()
			.filter(|(name, ty)| !ty.is_opt() && !self.input_data.contains_key(*name))
			.map(|(name, _)| name.to_string())
			.chain(
				self.legacy_enums
					.iter()
					.filter(|e| e.state.lock().unwrap().deref() == &EnumInner::NoDefinition)
					.map(|e| e.name().to_string()),
			)
			.collect::<Vec<_>>();
		if !missing.is_empty() {
			missing.sort();
			let errors = missing
				.into_iter()
				.map(|identifier| MissingData { identifier }.into())
				.collect::<Vec<Error>>();
			return Err(Error::try_from(errors).unwrap());
		}

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
		// The MicroZinc target erases records, optional values and enums, so the
		// data is written and the solutions are read in the erased form
		let erased = self.target == CompileTarget::MicroZinc;
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
				unreachable!("missing inputs are reported above")
			};
			writeln!(file_mut, "{name} = {};", LegacyValue { val, ty, erased })
				.map_err(write_err)?;
		}
		for e in &self.legacy_enums {
			debug_assert!(e.state.lock().unwrap().deref() != &EnumInner::NoDefinition);
			if erased {
				writeln!(file_mut, "{};", LegacyEnum(e)).map_err(write_err)?;
			} else {
				// The definition of an enum is shown as a MiniZinc assignment
				writeln!(file_mut, "{e};").map_err(write_err)?;
			}
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
		cmd.args(&self.minizinc_args);

		let mut child = cmd.spawn().unwrap(); // TODO: fix unwrap
		// The interpreter's output must be consumed as it is produced: waiting
		// for the process to exit first deadlocks as soon as it writes more
		// than fits in the pipe's buffer
		let stdout = child.stdout.take().unwrap();

		let mut status = Status::Unknown;
		// Reading the output is fallible, but the interpreter has to be waited
		// on whether it succeeds or not, or the process is left behind
		let read = (|| -> Result<()> {
			for line in BufReader::new(stdout).lines() {
				let line = line.map_err(|e| {
					Error::from(InternalError::new(format!(
						"Unable to read interpreter output: “{e}”"
					)))
				})?;
				match serde_json::Deserializer::from_str(&line)
					.deserialize_map(SerdeMessageVisitor {
						types: &self.output_types,
						erased,
					})
					.map_err(|e| Error::from_serde_json(e, &SourceFile::unnamed(line.clone())))?
				{
					LegacyOutput::Status(s) => status = s,
					LegacyOutput::Msg(msg) => {
						if let Message::Solution { .. } = msg
							&& status == Status::Unknown
						{
							status = Status::Satisfied
						}
						msg_callback(&msg)?
					}
					LegacyOutput::Error(err) => return Err(err),
				}
			}
			Ok(())
		})();
		let exit = child.wait();
		read?;
		match exit {
			Ok(code) if code.success() => Ok(status),
			// The interpreter failed without reporting an error of its own, such
			// as when it crashes: its standard error has been passed through, and
			// whatever it printed before failing cannot be trusted.
			// `ExitStatus` shows a signal as well as an exit code.
			Ok(code) => {
				Err(InternalError::new(format!("the MiniZinc process failed ({code})")).into())
			}
			Err(e) => Err(InternalError::new(format!("process error: {}", e)).into()),
		}
	}
}

/// A data value written as a MiniZinc expression
struct LegacyValue<'a> {
	val: &'a Value,
	ty: &'a Type,
	/// Whether the value is written in the erased form, where a record is a
	/// tuple, an optional value an `(occurs, value)` pair and an enum value an
	/// integer
	erased: bool,
}

impl Display for LegacyValue<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let val = self.val;
		let ty = self.ty;
		let erased = self.erased;
		let value = |val, ty| LegacyValue { val, ty, erased };
		// An enum value is named by its position, which needs no knowledge of
		// its constructors
		let enum_value = |v: &EnumValue| {
			if erased {
				v.int_val().to_string()
			} else {
				format!("to_enum({}, {})", v.enum_type().name(), v.int_val())
			}
		};

		let is_opt = erased && ty.is_opt();
		if is_opt {
			if val == &Value::Absent {
				return write!(f, "(false, {})", DummyValue(ty));
			} else {
				write!(f, "(true, ")?;
			}
		}
		match val {
			Value::Absent if !erased && ty.is_opt() => write!(f, "<>")?,
			Value::Absent => unreachable!("found absent assigned to non-opt parameter"),
			Value::Infinity(p) => {
				write!(f, "{}infinity", if p == &Polarity::Neg { "-" } else { "" })?
			}
			Value::Boolean(v) => write!(f, "{}", v)?,
			Value::Integer(v) => write!(f, "{}", v)?,
			Value::Float(v) => write!(f, "{}", v)?,
			Value::String(v) => write!(f, "\"{}\"", v)?,
			Value::Enum(v) => write!(f, "{}", enum_value(v))?,
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
					Value::Integer(i) => i.to_string(),
					Value::Enum(v) => enum_value(v),
					_ => unreachable!(),
				};
				if v.is_empty() {
					write!(f, "[]")?;
				} else if v.dim() == 1 {
					let first = extract_idx(&(v.iter().next().unwrap().0[0]));
					write!(
						f,
						"[{first}: {}]",
						v.iter().map(|(_, val)| value(val, element)).format(",")
					)?;
				} else {
					write!(
						f,
						"[{}]",
						v.iter().format_with(",", |(ii, val), f| f(&format_args!(
							"({}): {}",
							ii.iter().map(extract_idx).format(","),
							value(val, element)
						)))
					)?;
				}
			}
			Value::KeyedArray(_) => {
				unreachable!("keyed arrays are only output, never input data")
			}
			Value::Set(s) => match s {
				Set::Enum(s) => write!(
					f,
					"{}",
					s.iter().format_with(" union ", |elt, f| f(&format_args!(
						"{}..{}",
						enum_value(&elt.start()),
						enum_value(&elt.end())
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
						.map(|(ty, val)| value(val, ty))
						.format(","),
					if tys.len() == 1 { "," } else { "" }
				)?;
			}
			Value::Record(v) => {
				let Type::Record(_, tys) = ty else {
					unreachable!()
				};
				if erased {
					write!(
						f,
						"({}{})",
						tys.iter()
							.map(|(_, t)| t)
							.zip_eq(v.iter().map(|(_, v)| v))
							.map(|(ty, val)| value(val, ty))
							.format(","),
						if tys.len() == 1 { "," } else { "" }
					)?;
				} else {
					write!(
						f,
						"({})",
						tys.iter().zip_eq(v.iter()).format_with(
							",",
							|((name, ty), (_, val)), f| f(&format_args!(
								"{name}: {}",
								value(val, ty)
							))
						)
					)?;
				}
			}
		}
		if is_opt {
			write!(f, ")")?
		}
		Ok(())
	}
}
struct DummyValue<'a>(&'a Type);
impl Display for DummyValue<'_> {
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
impl Display for LegacyEnum<'_> {
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
							ty: &INT,
							erased: true,
						},
						LegacyValue {
							val: &idx.end(),
							ty: &INT,
							erased: true,
						}
					)))
				)))
		)
	}
}

struct SerdeMessageVisitor<'a> {
	/// The types of the output variables
	types: &'a FxHashMap<Arc<str>, Type>,
	/// Whether the model's types are erased (see [`Type::output_type`])
	erased: bool,
}

enum LegacyOutput<'a> {
	Status(Status),
	Msg(Message<'a>),
	Error(Error),
}

/// A frame of the evaluation stack of an interpreter error
#[derive(serde::Deserialize)]
struct StackFrame {
	description: String,
}

/// Whether an interpreter error was raised by the check of the object ids given
/// in data (see `std/internal/object.mzn`), rather than by the compiled model
/// itself. Such an error is caused by the data. The MicroZinc target inlines
/// the stdlib under decorated names (`shackle_mzn_object_from_id_root`), so the
/// functions are matched by substring.
fn in_object_id_check(stack: &[StackFrame]) -> bool {
	stack.iter().any(|frame| {
		frame
			.description
			.strip_prefix("call '")
			.and_then(|call| call.strip_suffix('\''))
			.is_some_and(|name| {
				["mzn_object_from_id", "mzn_check_object_ids"]
					.iter()
					.any(|check| name.contains(check))
			})
	})
}

impl<'de> Visitor<'de> for SerdeMessageVisitor<'_> {
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
		let mut error = None;
		let mut msg_type = None;
		let mut statistics = None;
		let mut message = None;
		let mut status = None;
		let mut solution = None;
		let mut stack = None;

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
					// Owned: a message containing a JSON escape (such as a
					// quoted `\"`) cannot be borrowed from the input
					message = Some(map.next_value::<String>()?);
				}
				"output" => {
					if solution.is_some() {
						return Err(SerdeError::duplicate_field("output"));
					}
					match map.next_value_seed(SerdeSolutionVisitor {
						types: self.types,
						erased: self.erased,
					})? {
						Ok(sol) => solution = Some(sol),
						// The rest of the message must still be read
						Err(e) => error = Some(e),
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
							));
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
				"stack" => {
					if stack.is_some() {
						return Err(SerdeError::duplicate_field("stack"));
					}
					stack = Some(map.next_value::<Vec<StackFrame>>()?);
				}
				"location" | "sections" | "what" => {
					map.next_value::<IgnoredAny>()?; // TODO: parse additional error/warning information
				}
				_ => return Err(SerdeError::unknown_field(k, FIELDS)),
			}
		}
		if let Some(e) = error {
			return Ok(LegacyOutput::Error(e));
		}

		match msg_type {
			Some("solution") => match solution {
				None => Err(SerdeError::missing_field("output")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Solution {
					assignments: x.assignments,
					sections: x.sections,
				})),
			},
			Some("statistics") => match statistics {
				None => Err(SerdeError::missing_field("statistics")),
				Some(x) => Ok(LegacyOutput::Msg(Message::Statistic(x))),
			},
			Some("error") => match message {
				None => Err(SerdeError::missing_field("message")),
				Some(msg) if in_object_id_check(stack.as_deref().unwrap_or_default()) => {
					// The MicroZinc target turns the check's `assert` into an
					// `abort`, whose message MiniZinc prefixes
					let msg = msg.strip_prefix("Abort: ").unwrap_or(&msg).to_owned();
					Ok(LegacyOutput::Error(InvalidObjectData { msg }.into()))
				}
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

/// The rendered contents of the model's output sections are held in string
/// declarations named after the section. These are generated by the THIR
/// output transform, so they never appear in the model's output interface and
/// have to be picked out of the solver's assignments by name.
///
/// [`crate::Program::write`] runs the compatibility transform, which prefixes
/// every declaration coming from an auto-include, so both the plain and the
/// prefixed name have to be recognised.
const OUTPUT_SECTION_PREFIXES: [&str; 2] = ["shackle_mzn_output_", "mzn_output_"];
/// The section user output ends up in when no section is given
const DEFAULT_OUTPUT_SECTION: &str = "default";

/// The contents of a single solution reported by the interpreter
struct SerdeSolution<'a> {
	assignments: FxHashMap<&'a str, Value>,
	sections: Vec<(&'a str, String)>,
}

/// The `output` of a solution message, which holds the values of the output
/// variables in its `json` section.
///
/// The model's output item is rendered in one of two ways. When the model's
/// types are erased, the THIR output transform has turned it into string
/// variables, one per section (see [`OUTPUT_SECTION_PREFIXES`]). Otherwise the
/// interpreter renders it, and each section is a key of the `output` itself.
struct SerdeSolutionVisitor<'a> {
	types: &'a FxHashMap<Arc<str>, Type>,
	erased: bool,
}

impl<'de> DeserializeSeed<'de> for SerdeSolutionVisitor<'_> {
	type Value = Result<SerdeSolution<'de>, Error>;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_map(self)
	}
}

impl<'de> Visitor<'de> for SerdeSolutionVisitor<'_> {
	type Value = Result<SerdeSolution<'de>, Error>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "map with json identifier")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let mut sol = None;
		let mut sections = Vec::new();
		while let Some(k) = map.next_key::<&str>()? {
			match k {
				"json" => {
					sol = Some(map.next_value_seed(SerdeOutputVisitor {
						types: self.types,
						erased: self.erased,
					})?)
				}
				// All sections together
				"raw" => {
					map.next_value::<IgnoredAny>()?;
				}
				_ if !self.erased => sections.push((k, map.next_value::<String>()?)),
				_ => {
					map.next_value::<IgnoredAny>()?; // Ignore unknown
				}
			}
		}
		let Some(sol) = sol else {
			return Err(SerdeError::missing_field("json"));
		};
		Ok(sol.map(|mut sol| {
			sol.sections.extend(sections);
			// Show the unnamed section first, then the named ones in a stable order
			sol.sections
				.sort_by_key(|(name, _)| (*name != DEFAULT_OUTPUT_SECTION, *name));
			sol
		}))
	}
}

/// The `json` section of a solution's output
struct SerdeOutputVisitor<'a> {
	types: &'a FxHashMap<Arc<str>, Type>,
	erased: bool,
}

impl<'de> Visitor<'de> for SerdeOutputVisitor<'_> {
	type Value = Result<SerdeSolution<'de>, Error>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "minizinc output assignment")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let type_map = self.types;

		let mut sol = FxHashMap::default();
		sol.reserve(type_map.len());
		let mut sections = Vec::new();
		// The first value that cannot be resolved; the rest of the map must
		// still be read
		let mut error = None;
		while let Some(k) = map.next_key::<&str>()? {
			if let Some(section) = OUTPUT_SECTION_PREFIXES
				.iter()
				.find_map(|prefix| k.strip_prefix(prefix))
			{
				sections.push((section, map.next_value::<String>()?));
			} else if let Some(ty) = type_map.get(k) {
				let out_type = ty.output_type(self.erased);
				let v = map.next_value_seed(SerdeValueVisitor(&out_type))?;
				match v.resolve_value(&out_type) {
					Ok(v) => {
						let v = v.resolve_output(ty, self.erased);
						sol.insert(k, v);
					}
					Err(e) => {
						error.get_or_insert(e);
					}
				}
			} else {
				map.next_value::<IgnoredAny>()?; // Ignore unknown
			}
		}
		if let Some(e) = error {
			return Ok(Err(e));
		}
		Ok(Ok(SerdeSolution {
			assignments: sol,
			sections,
		}))
	}
}

impl<'de> DeserializeSeed<'de> for SerdeOutputVisitor<'_> {
	type Value = Result<SerdeSolution<'de>, Error>;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_map(self)
	}
}

impl Type {
	/// The type in which a value of this type is read from the interpreter's
	/// output.
	///
	/// An array is output as nested lists, so its dimensions are read as
	/// integers, and a keyed array as its (key, value) pairs. When the model's
	/// types are `erased`, an enum value is an integer, a record a tuple and an
	/// optional value an `(occurs, value)` pair, too.
	fn output_type(&self, erased: bool) -> Type {
		let opt = |opt: &OptType| if erased { OptType::NonOpt } else { *opt };
		let ty = match self {
			Type::Boolean(o) => Type::Boolean(opt(o)),
			Type::Integer(o) => Type::Integer(opt(o)),
			Type::Float(o) => Type::Float(opt(o)),
			Type::Enum(_, _) if erased => Type::Integer(OptType::NonOpt),
			Type::Enum(o, e) => Type::Enum(*o, e.clone()),
			Type::String(o) => Type::String(opt(o)),
			Type::Annotation(o) => Type::Annotation(opt(o)),
			Type::Array {
				opt: o,
				dim,
				element,
			} if Type::is_keyed(dim) => Type::Array {
				opt: opt(o),
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Box::new(Type::Tuple(
					OptType::NonOpt,
					Arc::new([
						Type::key(dim).output_type(erased),
						element.output_type(erased),
					]),
				)),
			},
			Type::Array {
				opt: o,
				dim,
				element,
			} => Type::Array {
				opt: opt(o),
				dim: dim.iter().map(|_| Type::Integer(OptType::NonOpt)).collect(),
				element: element.output_type(erased).into(),
			},
			Type::Set(o, elt) => Type::Set(opt(o), elt.output_type(erased).into()),
			Type::Tuple(o, elts) => {
				Type::Tuple(opt(o), elts.iter().map(|t| t.output_type(erased)).collect())
			}
			Type::Record(_, elts) if erased => Type::Tuple(
				OptType::NonOpt,
				elts.iter().map(|(_, t)| t.output_type(erased)).collect(),
			),
			Type::Record(o, elts) => Type::Record(
				*o,
				elts.iter()
					.map(|(n, t)| (n.clone(), t.output_type(erased)))
					.collect(),
			),
		};
		if erased && self.is_opt() {
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
	/// The value of type `ty` for a value read in its [`Type::output_type`]
	fn resolve_output(self, ty: &Type, erased: bool) -> Value {
		let mut v = self;
		if ty.is_opt() && erased {
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
		} else if v == Value::Absent {
			return v;
		}
		match ty {
			Type::Enum(_, e) if erased => {
				let Value::Integer(pos) = v else {
					panic!("expected integer value")
				};
				EnumValue::from_enum_and_pos(e.clone(), pos as usize).into()
			}
			Type::Array {
				opt: _,
				dim,
				element,
			} if Type::is_keyed(dim) => {
				let Value::Array(x) = v else {
					panic!("expected array");
				};
				let key = Type::key(dim);
				Value::KeyedArray(
					x.members
						.into_iter()
						.map(|pair| {
							let Value::Tuple(pair) = pair else {
								panic!("expected (key, value) tuple")
							};
							let [k, v] = <[Value; 2]>::try_from(pair)
								.unwrap_or_else(|_| panic!("expected (key, value) tuple"));
							(
								k.resolve_output(&key, erased),
								v.resolve_output(element, erased),
							)
						})
						.collect(),
				)
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
				// Only elements of a plain type are output as they are: optional
				// elements are (occurs, value) pairs, for instance
				let members = if matches!(
					**element,
					Type::Boolean(OptType::NonOpt)
						| Type::Integer(OptType::NonOpt)
						| Type::Float(OptType::NonOpt)
						| Type::String(OptType::NonOpt)
						| Type::Annotation(OptType::NonOpt)
				) {
					x.members
				} else {
					x.members
						.into_iter()
						.map(|elt| elt.resolve_output(element, erased))
						.collect()
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
						.map(|(elt, t)| elt.resolve_output(t, erased))
						.collect(),
				)
			}
			Type::Record(_, tys) if erased => {
				let Value::Tuple(tup) = v else {
					panic!("expected tuple")
				};
				Value::Record(
					tup.into_iter()
						.zip_eq(tys.iter())
						.map(|(elt, (n, t))| (n.clone(), elt.resolve_output(t, erased)))
						.collect(),
				)
			}
			Type::Record(_, tys) => {
				let Value::Record(rec) = v else {
					panic!("expected record")
				};
				Value::Record(
					rec.iter()
						.zip_eq(tys.iter())
						.map(|((n, elt), (_, t))| (n, elt.clone().resolve_output(t, erased)))
						.collect(),
				)
			}
			_ => v,
		}
	}
}

#[cfg(test)]
mod tests {
	use rustc_hash::FxHashMap;
	use serde::Deserializer;

	use super::{LegacyOutput, SerdeMessageVisitor};
	use crate::Message;

	fn parse(line: &str) -> LegacyOutput<'_> {
		let output_types = FxHashMap::default();
		serde_json::Deserializer::from_str(line)
			.deserialize_map(SerdeMessageVisitor {
				types: &output_types,
				erased: true,
			})
			.unwrap()
	}

	/// A message containing a JSON escape cannot be borrowed from the input,
	/// and used to fail to parse, hiding the interpreter's actual error.
	#[test]
	fn error_message_with_escaped_quotes() {
		let line = r#"{"type": "error", "what": "evaluation error", "message": "Abort: no A object with id \"x\" in the data"}"#;
		let LegacyOutput::Error(err) = parse(line) else {
			panic!("expected an error");
		};
		assert!(
			format!("{err:?}").contains(r#"no A object with id \"x\" in the data"#),
			"{err:?}"
		);
	}

	#[test]
	fn warning_message_with_escaped_quotes() {
		let line = r#"{"type": "warning", "message": "the id \"x\" is unused"}"#;
		let LegacyOutput::Msg(Message::Warning(msg)) = parse(line) else {
			panic!("expected a warning");
		};
		assert_eq!(msg, r#"the id "x" is unused"#);
	}
}
