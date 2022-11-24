use std::{
	collections::BTreeMap,
	io::{BufRead, BufReader},
	ops::Range,
	path::PathBuf,
	process::{Command, Stdio},
};

use serde_json::Map;
use tempfile::Builder;

use crate::{
	db::CompilerDatabase,
	error::{FileError, InternalError},
	hir::{db::Hir, Identifier},
	ty::{Ty, TyData},
	Message, Program, Status, Value,
};

fn flatten_array(
	db: &CompilerDatabase,
	content: &mut Vec<Value>,
	arr: serde_json::Value,
	ndim: usize,
	elem_ty: Ty,
) -> Result<(), InternalError> {
	if let serde_json::Value::Array(vec) = arr {
		if ndim > 1 {
			for sub_arr in vec {
				flatten_array(db, content, sub_arr, ndim - 1, elem_ty)?;
			}
		} else {
			for elem in vec {
				content.push(deserialize_legacy_value(db, elem_ty, elem)?);
			}
		}
		Ok(())
	} else {
		Err(InternalError::new(
			"value from legacy interpreter does not have the expected number of dimensions",
		))
	}
}

fn deserialize_legacy_value(
	db: &CompilerDatabase,
	ty: Ty,
	val: serde_json::Value,
) -> Result<Value, InternalError> {
	match val {
		serde_json::Value::Null => {
			if ty.known_occurs(db) {
				Err(InternalError::new(
					format!("legacy interpreter returned an absent value for variable with a type `{}',  known to occur", ty.pretty_print(db)),
				))
			} else {
				Ok(Value::Absent)
			}
		}
		serde_json::Value::Bool(b) => {
			if let TyData::Boolean(_, _) = ty.lookup(db) {
				Ok(Value::Boolean(b))
			} else {
				Err(InternalError::new(format!(
					"legacy interpreter returned a Boolean value for variable of type `{}'",
					ty.pretty_print(db)
				)))
			}
		}
		serde_json::Value::Number(v) => {
			if v.is_f64() {
				if let TyData::Float(_, _) = ty.lookup(db) {
					Ok(Value::Float(v.as_f64().unwrap()))
				} else {
					Err(InternalError::new(format!(
						"legacy interpreter returned a floating point value for variable of type `{}'",
						ty.pretty_print(db)
					)))
				}
			} else {
				assert!(v.is_i64());
				match ty.lookup(db) {
					TyData::Integer(_, _) => Ok(Value::Integer(v.as_i64().unwrap())),
					TyData::Float(_, _) => Ok(Value::Float(v.as_i64().unwrap() as f64)),
					_ => Err(InternalError::new(format!(
						"legacy interpreter returned a integer value for variable of type `{}'",
						ty.pretty_print(db)
					))),
				}
			}
		}
		serde_json::Value::String(v) => {
			if let TyData::String(_) = ty.lookup(db) {
				Ok(Value::String(v))
			} else {
				Err(InternalError::new(format!(
					"legacy interpreter returned a String value for variable of type `{}'",
					ty.pretty_print(db)
				)))
			}
		}
		serde_json::Value::Array(v) => match ty.lookup(db) {
			TyData::Array {
				opt: _,
				dim,
				element,
			} => {
				if let TyData::Tuple(_, tt) = dim.lookup(db) {
					// Determine the index sets of the array
					// FIXME should be returned from the model
					let mut ranges = Vec::with_capacity(tt.len());
					let arr = serde_json::Value::Array(v);
					let mut ii = &arr;
					while let serde_json::Value::Array(v) = ii {
						ranges.push(1..(v.len() + 1) as i64);
						if let Some(fst) = v.last() {
							ii = fst;
						} else {
							ranges.push(1..1);
						}
					}
					// Flatten content
					let mut content = Vec::new();
					flatten_array(db, &mut content, arr, tt.len(), element)?;

					Ok(Value::Array(ranges, content))
				} else {
					let range: Range<i64> = 1..(v.len() + 1) as i64;
					let content = v
						.into_iter()
						.map(|val| deserialize_legacy_value(db, element, val))
						.collect::<Result<Vec<_>, _>>()?;
					Ok(Value::Array(vec![range], content))
				}
			}
			TyData::Tuple(_, types) => {
				assert_eq!(types.len(), v.len());
				v.into_iter()
					.zip(types.iter())
					.map(|(val, ty)| deserialize_legacy_value(db, *ty, val))
					.collect::<Result<Vec<_>, _>>()
					.map(Value::Tuple)
			}
			_ => Err(InternalError::new(format!(
				"legacy interpreter returned a Array value for variable of type `{}'",
				ty.pretty_print(db)
			))),
		},
		serde_json::Value::Object(mut obj) => match ty.lookup(db) {
			TyData::Enum(_, _, _) => {
				let e = if let Some(s) = obj["e"].as_str() {
					String::from(s)
				} else if let Some(x) = obj["e"].as_i64() {
					x.to_string()
				} else if let Value::Enum(s) = deserialize_legacy_value(db, ty, obj["e"].clone())? {
					s
				} else {
					return Err(InternalError::new(format!(
						"lagacy interpreter returned an invalid enum value `{:?}'",
						obj
					)));
				};
				Ok(if obj.contains_key("c") {
					assert_eq!(obj.len(), 2);
					Value::Enum(format!("{}({})", obj["c"].as_str().unwrap(), e))
				} else if obj.contains_key("i") {
					assert_eq!(obj.len(), 2);
					Value::Enum(format!("to_enum({}, {})", e, obj["i"].as_i64().unwrap()))
				} else {
					assert_eq!(obj.len(), 1);
					Value::Enum(e)
				})
			}
			TyData::Set(_, _, elem) => {
				let set = obj.remove("set").unwrap();
				if let serde_json::Value::Array(set) = set {
					match elem.lookup(db) {
						TyData::Integer(_, _) if matches!(set[0], serde_json::Value::Array(_)) => {
							let mut content = Vec::new();
							for mem in set {
								if let serde_json::Value::Array(x) = mem {
									assert_eq!(x.len(), 2);
									for i in x[0].as_i64().unwrap()..=x[1].as_i64().unwrap() {
										content.push(Value::Integer(i))
									}
								} else {
									return Err(InternalError::new(format!(
										"legacy interpreter invalid range in set members `{}'",
										mem
									)));
								}
							}
							Ok(Value::Set(content))
						}
						_ => {
							let content = set
								.into_iter()
								.map(|v| deserialize_legacy_value(db, elem, v))
								.collect::<Result<Vec<_>, _>>()?;
							Ok(Value::Set(content))
						}
					}
				} else {
					Err(InternalError::new(format!(
						"legacy interpreter returned invalid set members `{}'",
						set
					)))
				}
			}
			TyData::Record(_, types) => {
				assert_eq!(types.len(), obj.len());
				let mut rec = BTreeMap::new();
				for (name, tt) in types.iter() {
					let name = name.value(db);
					let val = deserialize_legacy_value(db, *tt, obj.remove(&name).unwrap())?;
					rec.insert(name, val);
				}
				Ok(Value::Record(rec))
			}
			_ => Err(InternalError::new(format!(
				"legacy interpreter returned a Object value for variable of type `{}'",
				ty.pretty_print(db)
			))),
		},
	}
}

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> bool>(&mut self, msg_callback: F) -> Status {
		let tmpfile = Builder::new().suffix(".shackle.mzn").tempfile();
		let mut tmpfile = match tmpfile {
			Err(err) => {
				return Status::Err(
					FileError {
						file: PathBuf::from("tempfile"),
						message: err.to_string(),
						other: Vec::new(),
					}
					.into(),
				);
			}
			Ok(file) => file,
		};
		if let Err(err) = self.write(tmpfile.as_file_mut()) {
			return Status::Err(
				FileError {
					file: PathBuf::from(tmpfile.path()),
					message: format!("unable to write model to temporary file: {}", err),
					other: vec![],
				}
				.into(),
			);
		}

		let mut cmd = Command::new("minizinc");
		cmd.stdin(Stdio::null())
			.stdout(Stdio::piped())
			.stderr(Stdio::inherit())
			.arg(tmpfile.path())
			.args([
				"--output-mode",
				"json",
				"--json-stream",
				"--ignore-stdlib",
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

		let ty_map = self.db.variable_type_map();
		let mut status = Status::Unknown;
		for line in BufReader::new(stdout).lines() {
			match line {
				Err(err) => {
					return Status::Err(
						InternalError::new(format!("minizinc output error: {}", err)).into(),
					);
				}
				Ok(line) => {
					let mut obj = serde_json::from_str::<Map<String, serde_json::Value>>(&line)
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
							if let serde_json::Value::Object(map) = obj["output"]
								.as_object_mut()
								.unwrap()
								.remove("json")
								.unwrap()
							{
								for (k, v) in map {
									let ident = Identifier::new(&k, &self.db);
									let ty = ty_map[&ident];

									let val = match deserialize_legacy_value(&self.db, ty, v) {
										Ok(val) => val,
										Err(e) => return Status::Err(e.into()),
									};
									sol.insert(k, val);
								}
							} else {
								panic!("invalid output.json field in mzn json")
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
