//! Shackle library

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

pub mod arena;
pub mod db;
pub mod error;
pub mod file;
pub mod hir;
pub mod syntax;
pub mod thir;
pub mod ty;
pub mod utils;

use db::{FileReader, Inputs};
use error::{FileError, InternalError, MultipleErrors, ShackleError};
use file::{FileRefData, InputFile};
use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Map;
use tempfile::{Builder, NamedTempFile};

use std::{
	collections::BTreeMap,
	env,
	fmt::{self, Display},
	io::{BufRead, BufReader, Write},
	ops::Range,
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::Arc,
	time::{Duration, Instant},
};

use crate::{
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

/// Shackle error type
pub type Error = ShackleError;
/// Result type for Shackle operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Parses a list of MiniZinc files given located using the Paths in the vector
pub fn parse_files(paths: Vec<&Path>) -> Result<()> {
	let now = Instant::now();
	let mut db = db::CompilerDatabase::new();
	db.set_input_files(Arc::new(
		paths
			.into_iter()
			.map(|p| InputFile::Path(p.to_owned()))
			.collect(),
	));
	let mut errors = (*db.all_diagnostics()).clone();
	eprintln!("Done in {}ms", now.elapsed().as_millis());
	if errors.is_empty() {
		// Can print THIR if there were no errors
		println!(
			"{}",
			PrettyPrinter::new(&db, &db.model_thir()).pretty_print()
		);
		Ok(())
	} else if errors.len() == 1 {
		Err(errors.pop().unwrap())
	} else {
		Err(MultipleErrors { errors }.into())
	}
}

/// Structure used to build a shackle model
#[derive(Default)]
pub struct Model {
	files: Vec<InputFile>,
	stdlib: Option<PathBuf>,
}

impl Model {
	/// Create a Model from the file at the given path
	pub fn from_file(path: PathBuf) -> Model {
		Model {
			files: vec![InputFile::Path(path)],
			stdlib: None,
		}
	}

	/// Create a Model from the given string
	pub fn from_string(m: String) -> Model {
		Model {
			files: vec![InputFile::ModelString(m)],
			stdlib: None,
		}
	}

	/// Compile current model into a Program that can be used by the Shackle interpreter
	pub fn compile(&self, slv: &Solver) -> Result<Program> {
		let mut db = db::CompilerDatabase::new();
		db.set_input_files(Arc::new(self.files.clone()));

		let mut search_dirs = Vec::new();
		// TODO: New STDLIB behaviour
		if let Some(path) = &self.stdlib {
			search_dirs.push(path.clone())
		} else if let Ok(pathstr) = env::var("MZN_STDLIB_DIR") {
			search_dirs.push(PathBuf::from(pathstr).join("std"))
		}
		db.set_search_directories(Arc::new(search_dirs));
		let errors = db.all_diagnostics();
		if !errors.is_empty() {
			if errors.len() == 1 {
				return Err(errors.last().unwrap().clone());
			}
			if errors.len() > 1 {
				return Err(MultipleErrors {
					errors: (*errors).clone(),
				}
				.into());
			}
		}
		let output = Builder::new()
			.suffix(".mzn")
			.tempfile()
			.map_err(|err| FileError {
				file: PathBuf::from("tempfile"),
				message: err.to_string(),
				other: Vec::new(),
			})?;
		for file in db
			.input_files()
			.iter()
			.enumerate()
			.filter_map(|(idx, f)| match f {
				InputFile::Path(p) => match p.extension() {
					Some(e) => {
						if e.to_str() == Some("mzn") {
							Some(db.intern_file_ref(FileRefData::InputFile(idx)))
						} else {
							None
						}
					}
					None => None,
				},
				InputFile::ModelString(_) => Some(db.intern_file_ref(FileRefData::InputFile(idx))),
				_ => None,
			}) {
			let contents = db.file_contents(file)?;
			output
				.as_file()
				.write_all(contents.as_bytes())
				.map_err(|e| FileError {
					file: PathBuf::from(output.path()),
					message: format!("{}", e),
					other: vec![],
				})?;
		}
		Ok(Program {
			slv: slv.clone(),
			code: output,
			enable_stats: false,
			time_limit: None,
		})
	}
}

/// Solver specification to compile and solve Model instances.
#[derive(Clone)]
pub struct Solver {
	// TODO: actual information (Load from solver configurations)
	/// Identifier of the solver
	ident: String,
}

impl Solver {
	/// Lookup a solver specification in default locations that best matches the given identifier
	pub fn lookup(ident: &str) -> Option<Solver> {
		Some(Solver {
			ident: ident.into(),
		})
	}
}

/// Structure to capture the result of succesful compilation of a Model object
pub struct Program {
	slv: Solver,
	code: NamedTempFile,
	enable_stats: bool,
	time_limit: Option<Duration>,
}

/// Status of running and solving a Program
pub enum Status {
	/// No solutions exist
	Infeasible,
	/// A solution has been found
	Satisfied,
	/// A solution with the best possible objective value has been found
	Optimal,
	/// All possible solutions have been found
	AllSolutions,
	/// No result reached within the given limits
	Unknown,
	/// An error occurred
	Err(ShackleError),
}

/// Intermediate messages emitted by shackle in processing and solving a program
pub enum Message<'a> {
	/// (Intermediate) solution emitted in the process
	Solution(BTreeMap<String, Value>),
	/// Statistical information of the shackle or solving process
	Statistic(&'a Map<String, serde_json::Value>),
	/// Trace messages emitted during the shackle process
	Trace(&'a str),
	/// Warning messages emitted by shackle or the solver
	Warning(&'a str),
}

impl<'a> Display for Message<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Message::Solution(sol) => {
				for (name, val) in sol {
					writeln!(f, "{} = {};", name, val)?;
				}
				writeln!(f, "----------")
			}
			Message::Statistic(map) => {
				for (name, val) in *map {
					writeln!(f, "%%%mzn-stat: {}={}", name, val)?;
				}
				writeln!(f, "%%%mzn-stat-end")
			}
			Message::Trace(msg) => write!(f, "% mzn-trace: {}", msg),
			Message::Warning(msg) => write!(f, "% WARNING: {}", msg),
		}
	}
}

/// Value types that can be part of a Solution
pub enum Value {
	/// Absence of an optional value
	Absent,
	/// Boolean
	Boolean(bool),
	/// Signed integer
	Integer(i64),
	/// Floating point
	Float(f64),
	/// String
	String(String),
	/// Identifier of a value of an enumerated type
	Enum(String),
	/// An array of values
	/// All values are of the same type
	Array(Vec<Range<i64>>, Vec<Value>),
	/// A set of values
	/// All values are of the same type and only occur once
	Set(Vec<Value>),
	/// A tuple of values
	Tuple(Vec<Value>),
	/// A record of values
	Record(BTreeMap<String, Value>),
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Absent => write!(f, "<>"),
			Value::Boolean(v) => write!(f, "{}", v),
			Value::Integer(v) => write!(f, "{}", v),
			Value::Float(v) => write!(f, "{}", v),
			Value::String(v) => write!(f, "{:?}", v),
			Value::Enum(v) => write!(f, "{}", v),
			Value::Array(idx, v) => {
				let mut ii: Vec<i64> = idx.iter().map(|r| r.start).collect();
				let incr = |ii: &mut Vec<i64>| {
					let mut i = ii.len() - 1;
					ii[i] += 1;
					while !idx[i].contains(&ii[i]) && i > 0 {
						ii[i] = idx[i].start;
						i = i.wrapping_sub(1);
						ii[i] += 1;
					}
				};
				let mut first = true;
				write!(f, "[")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					match &ii[..] {
						[i] => write!(f, "{i}: "),
						ii => {
							write!(f, "(")?;
							let mut tup_first = true;
							for i in ii {
								if !tup_first {
									write!(f, ",")?;
								}
								write!(f, "{i}")?;
								tup_first = false;
							}
							write!(f, "): ")
						}
					}?;
					write!(f, "{x}")?;
					first = false;
					incr(&mut ii);
				}
				write!(f, "]")
			}
			Value::Set(v) => {
				let mut first = true;
				write!(f, "{{")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}", x)?;
					first = false;
				}
				write!(f, "}}")
			}
			Value::Tuple(v) => {
				let mut first = true;
				write!(f, "(")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}", x)?;
					first = false;
				}
				write!(f, ")")
			}
			Value::Record(rec) => {
				let mut first = true;
				write!(f, "(")?;
				for (k, v) in rec {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}: {}", k, v)?;
					first = false;
				}
				write!(f, ")")
			}
		}
	}
}

struct ValueVisitor;
impl<'de> Visitor<'de> for ValueVisitor {
	type Value = Value;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("MiniZinc Value")
	}
	fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
		Ok(Value::Absent)
	}
	fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_any(Self)
	}
	fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
		Ok(Value::Boolean(v))
	}
	fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
		Ok(Value::Integer(v))
	}
	fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
		Ok(Value::Integer(v.try_into().unwrap()))
	}
	fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
		Ok(Value::Float(v))
	}
	fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok(Value::String(String::from(v)))
	}
	fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Self::Value, V::Error> {
		let mut vec: Vec<Value> = Vec::with_capacity(seq.size_hint().unwrap_or(0));
		while let Some(el) = seq.next_element()? {
			vec.push(el);
		}
		Ok(Value::Tuple(vec))
	}

	fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
		let ret = match map.next_key()? {
			Some("c") => {
				let func: &str = map.next_value()?;
				match map.next_key()? {
					Some("e") => {
						let val: Value = map.next_value()?;
						Ok(Value::Enum(format!("{func}({val})")))
					}
					Some(key) => Err(serde::de::Error::unknown_field(key, &["e"])),
					None => Err(serde::de::Error::missing_field("e")),
				}
			}
			Some("i") => {
				let val: i64 = map.next_value()?;
				match map.next_key()? {
					Some("e") => {
						let name: &str = map.next_value()?;
						Ok(Value::Enum(format!("to_enum({name}, {val})")))
					}
					Some(key) => Err(serde::de::Error::unknown_field(key, &["e"])),
					None => Err(serde::de::Error::missing_field("e")),
				}
			}
			Some("e") => {
				let val = map.next_value()?;
				match map.next_key()? {
					Some("i") => {
						if let Value::String(val) = val {
							let ival: i64 = map.next_value()?;
							Ok(Value::Enum(format!("to_enum({val}, {ival})")))
						} else {
							Err(serde::de::Error::custom("non-string enum identifier"))
						}
					}
					Some("c") => {
						let func: &str = map.next_value()?;
						Ok(Value::Enum(format!("{func}({val})")))
					}
					Some(field) => Err(serde::de::Error::unknown_field(field, &["i", "c"])),
					None => {
						if let Value::String(e) = val {
							Ok(Value::Enum(e))
						} else {
							Err(serde::de::Error::custom("non-string enum identifier"))
						}
					}
				}
			}
			Some("indices") => {
				let idx: Vec<(i64, i64)> = map.next_value()?;
				match map.next_key()? {
					Some("members") => {
						let members = map.next_value()?;
						Ok(Value::Array(
							idx.iter().map(|r| r.0..(r.1 + 1)).collect(),
							members,
						))
					}
					Some(field) => Err(serde::de::Error::unknown_field(field, &["members"])),
					None => Err(serde::de::Error::missing_field("members")),
				}
			}
			Some("members") => {
				let members = map.next_value()?;
				match map.next_key()? {
					Some("indices") => {
						let idx = map.next_value()?;
						Ok(Value::Array(idx, members))
					}
					Some(field) => Err(serde::de::Error::unknown_field(field, &["indices"])),
					None => Err(serde::de::Error::missing_field("indices")),
				}
			}
			Some("set") => {
				// FIXME: Only works for integer sets
				let ranges: Vec<(i64, i64)> = map.next_value()?;
				let members = ranges
					.iter()
					.flat_map(|range| (range.0..=range.1))
					.map(Value::Integer)
					.collect();
				Ok(Value::Set(members))
			}
			Some("record") => Ok(Value::Record(map.next_value()?)),
			_ => Err(serde::de::Error::invalid_length(
				0,
				&"object of size 1 or 2",
			)),
		};
		if let Some(key) = map.next_key()? {
			Err(serde::de::Error::unknown_field(key, &[]))
		} else {
			ret
		}
	}
}

impl<'de> Deserialize<'de> for Value {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_option(ValueVisitor)
	}
}

impl Program {
	/// Set whether messages containing statistical information regarding running the program should be sent
	pub fn with_statistics(mut self, stats: bool) -> Self {
		self.enable_stats = stats;
		self
	}
	/// Add the maximum duration that the run method is allowed to take before it will be canceled
	pub fn with_time_limit(mut self, dur: Duration) -> Self {
		self.time_limit = Some(dur);
		self
	}

	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> bool>(&mut self, msg_callback: F) -> Status {
		let mut cmd = Command::new("minizinc");
		cmd.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.arg(self.code.path())
			.args([
				"--output-mode",
				"typed-json",
				"--json-stream",
				"--output-time",
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
				Err(err) => {
					return Status::Err(
						InternalError::new(format!("minizinc output error: {}", err)).into(),
					);
				}
				Ok(line) => {
					let obj = serde_json::from_str::<Map<String, serde_json::Value>>(&line)
						.expect("bad message in mzn json");
					let ty = obj["type"].as_str().expect("bad type field in mzn json");
					match ty {
						"statistics" => {
							if let serde_json::Value::Object(map) = &obj["statistics"] {
								msg_callback(&Message::Statistic(map));
							} else {
								return Status::Err(
									InternalError::new(format!(
										"minizinc invalid statistics message: {}",
										obj["statistics"]
									))
									.into(),
								);
							}
						}
						"solution" => {
							let mut sol = BTreeMap::new();
							for (k, v) in obj["output"]["json"]
								.as_object()
								.expect("invalid output.json field in mzn json")
							{
								let val = v.deserialize_any(ValueVisitor).unwrap_or_else(|_| {
									panic!("invalid minizinc json value: {:?}", v)
								});
								sol.insert(k.clone(), val);
							}
							msg_callback(&Message::Solution(sol));
							if let Status::Unknown = status {
								status = Status::Satisfied;
							}
						}
						"status" => match obj["status"]
							.as_str()
							.expect("bad status field in mzn json")
						{
							"ALL_SOLUTIONS" => status = Status::AllSolutions,
							"OPTIMAL_SOLUTION" => status = Status::Optimal,
							"UNSATISFIABLE" => status = Status::Infeasible,
							"UNBOUNDED" => todo!(),
							"UNSAT_OR_UNBOUNDED" => todo!(),
							"UNKNOWN" => status = Status::Unknown,
							"ERROR" => {
								status = Status::Err(
									InternalError::new(
										"Error occurred, but no message was provided",
									)
									.into(),
								)
							}
							s => {
								return Status::Err(
									InternalError::new(format!(
										"minizinc unknown status type: {}",
										s
									))
									.into(),
								);
							}
						},
						"error" => {
							return Status::Err(
								InternalError::new(format!("minizinc error: {}", obj["message"]))
									.into(),
							);
						}
						"warning" => {
							msg_callback(&Message::Warning(
								obj["message"]
									.as_str()
									.expect("invalid message field in mzn json object"),
							));
						}
						s => {
							return Status::Err(
								InternalError::new(format!("minizinc unknown message type: {}", s))
									.into(),
							);
						}
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
				status
			}
			Err(e) => Status::Err(InternalError::new(format!("process error: {}", e)).into()),
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
