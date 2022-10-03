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
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde_json::Map;
use tempfile::{Builder, NamedTempFile};

use std::{
	collections::BTreeMap,
	env,
	fmt::{self, Display},
	io::{BufRead, BufReader, Write},
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::Arc,
	time::Instant,
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
		if let Some(path) = &self.stdlib {
			search_dirs.push(path.clone())
		} else if let Ok(pathstr) = env::var("MZN_STDLIB_DIR") {
			search_dirs.push(PathBuf::from(pathstr).join("std"))
		}
		db.set_search_directories(Arc::new(search_dirs));
		let mut errors = (*db.all_diagnostics()).clone();
		if errors.len() == 1 {
			return Err(errors.pop().unwrap());
		}
		if errors.len() > 1 {
			return Err(MultipleErrors { errors }.into());
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
							Some(db.intern_file_ref(FileRefData::InputFile(idx)).into())
						} else {
							None
						}
					}
					None => None,
				},
				InputFile::ModelString(_) => {
					Some(db.intern_file_ref(FileRefData::InputFile(idx)).into())
				}
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
	Array(Vec<Value>),
	/// A set of values
	/// All values are of the same type and only occur once
	Set(Vec<Value>),
	/// A tuple of values
	Tuple(Vec<Value>),
	/// A record of values
	Record(BTreeMap<String, Value>),
}

impl<'a> Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Absent => write!(f, "<>"),
			Value::Boolean(v) => write!(f, "{}", v),
			Value::Integer(v) => write!(f, "{}", v),
			Value::Float(v) => write!(f, "{}", v),
			Value::String(v) => write!(f, "{:?}", v),
			Value::Enum(v) => write!(f, "{}", v),
			Value::Array(v) => {
				let mut first = true;
				write!(f, "[")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}", x)?;
					first = false;
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
		let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or(0));
		while let Ok(Some(el)) = seq.next_element() {
			vec.push(el)
		}
		// TODO Detect array
		Ok(Value::Tuple(vec))
	}

	// fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
	// where
	// 	V: MapAccess<'de>,
	// {
	// 	let mut secs = None;
	// 	let mut nanos = None;
	// 	while let Some(key) = map.next_key()? {
	// 		match key {
	// 			Field::Secs => {
	// 				if secs.is_some() {
	// 					return Err(de::Error::duplicate_field("secs"));
	// 				}
	// 				secs = Some(map.next_value()?);
	// 			}
	// 			Field::Nanos => {
	// 				if nanos.is_some() {
	// 					return Err(de::Error::duplicate_field("nanos"));
	// 				}
	// 				nanos = Some(map.next_value()?);
	// 			}
	// 		}
	// 	}
	// 	let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
	// 	let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;
	// 	Ok(Duration::new(secs, nanos))
	// }
}

impl<'de> Deserialize<'de> for Value {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_option(ValueVisitor)
	}
}

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> bool>(&mut self, msg_callback: F) -> Status {
		let mut status = Status::Unknown;
		let mut child = Command::new("minizinc")
			.args([
				"--output-mode",
				"json",
				"--json-stream",
				"--output-time",
				"--output-objective",
				"--output-output-item",
				"--intermediate-solutions",
				"--statistics",
				"--solver",
				self.slv.ident.as_str(),
				self.code.path().to_str().unwrap(), // TODO: fix unwrap
			])
			.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()
			.unwrap(); // TODO: fix unwrap
		let stdout = child.stdout.take().unwrap();

		for line in BufReader::new(stdout).lines() {
			match line {
				Err(err) => {
					return Status::Err(
						InternalError::new(format!("minizinc output error: {}", err)).into(),
					);
				}
				Ok(line) => match serde_json::from_str::<Map<String, serde_json::Value>>(&line) {
					Ok(obj) => {
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
								if let serde_json::Value::Object(map) = &obj["output"]["json"] {
									let mut sol = BTreeMap::new();
									for (k, v) in map {
										let val = v.deserialize_any(ValueVisitor);
										match val {
											Ok(val) => {
												sol.insert(k.clone(), val);
											}
											Err(err) => {
												return Status::Err(
													InternalError::new(format!(
														"invalid minizinc json value: {}",
														err
													))
													.into(),
												);
											}
										}
									}
									msg_callback(&Message::Solution(sol));
									if let Status::Unknown = status {
										status = Status::Satisfied;
									}
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
									InternalError::new(format!(
										"minizinc error: {}",
										obj["message"]
									))
									.into(),
								);
							}
							s @ _ => {
								return Status::Err(
									InternalError::new(format!(
										"minizinc unknown message type: {}",
										s
									))
									.into(),
								);
							}
						}
					}
					Err(err) => {
						return Status::Err(
							InternalError::new(format!("minizinc invalid json: {}", err)).into(),
						);
					}
				},
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
