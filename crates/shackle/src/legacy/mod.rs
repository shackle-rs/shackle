use std::{
	collections::BTreeMap,
	io::{BufRead, BufReader, Write},
	path::PathBuf,
	process::{Command, Stdio},
};

use itertools::Itertools;
use rustc_hash::FxHashMap;
use serde_json::Map;
use tempfile::Builder;

use crate::{
	data::json::deserialize_legacy_value,
	diagnostics::{FileError, InternalError, ShackleError},
	hir::Identifier,
	ty::{self},
	value::{Polarity, Set, Value},
	Message, Program, Status, Type,
};

impl Program {
	/// Run the program in the current state
	/// Solutions are emitted to the callback, and the resulting status is returned.
	pub fn run<F: Fn(&Message) -> bool>(
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

		// let ty_map = self.db.variable_type_map();
		let ty_map: FxHashMap<Identifier, ty::Ty> = FxHashMap::default(); // TODO!!
		let mut status = Status::Unknown;
		for line in BufReader::new(stdout).lines() {
			match line {
				Err(err) => {
					return Err(
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
								return Err(InternalError::new(format!(
									"minizinc invalid statistics message: {}",
									obj["statistics"]
								))
								.into());
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
										Err(e) => return Err(e.into()),
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
								return Err(InternalError::new(
									"Error occurred, but no message was provided",
								)
								.into())
							}
							s => {
								return Err(InternalError::new(format!(
									"minizinc unknown status type: {}",
									s
								))
								.into());
							}
						},
						"error" => {
							return Err(InternalError::new(format!(
								"minizinc error: {}",
								obj["message"]
							))
							.into());
						}
						"warning" => {
							msg_callback(&Message::Warning(
								obj["message"]
									.as_str()
									.expect("invalid message field in mzn json object"),
							));
						}
						s => {
							return Err(InternalError::new(format!(
								"minizinc unknown message type: {}",
								s
							))
							.into());
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
			let Type::Array { opt: _, dim: _, element } = ty else {unreachable!()};
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
			Set::SetList(s) => write!(out, "{{{}}}", s.iter().format(", "))?,
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
			let Type::Tuple(_, tys) = ty else { unreachable!() };
			write!(out, "(")?;
			for (ty, val) in tys.iter().zip_eq(v) {
				write_legacy_value(out, ty, val)?;
				write!(out, ",")?
			}
			write!(out, ")")?;
		}
		Value::Record(v) => {
			let Type::Record(_, tys) = ty else { unreachable!() };
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
