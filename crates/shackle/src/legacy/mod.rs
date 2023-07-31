use std::{
	collections::BTreeMap,
	io::{BufRead, BufReader},
	path::PathBuf,
	process::{Command, Stdio},
};

use rustc_hash::FxHashMap;
use serde_json::Map;
use tempfile::Builder;

use crate::{
	data::json::deserialize_legacy_value,
	diagnostics::{FileError, InternalError},
	hir::Identifier,
	ty::{self},
	Message, Program, Status,
};

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

		// let ty_map = self.db.variable_type_map();
		let ty_map: FxHashMap<Identifier, ty::Ty> = FxHashMap::default(); // TODO!!
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
