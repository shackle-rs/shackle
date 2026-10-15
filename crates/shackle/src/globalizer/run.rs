//! Runs the `minizinc` executable on a printed submodel and collects the
//! sampled solutions from its JSON stream.

use std::{
	hash::{DefaultHasher, Hash, Hasher},
	io::Write,
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::{
		Arc, Mutex,
		atomic::{AtomicUsize, Ordering},
	},
};

use rustc_hash::FxHashMap;
use serde_json::{Map, Value};
use shackle_diagnostics::{FileError, InternalError};
use tempfile::Builder;

use super::GlobalizeOptions;
use crate::{Error, Result};

/// A solution: the JSON object of all output variables
pub(super) type Solution = Map<String, Value>;

/// Result of one `minizinc` run
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Outcome {
	/// Solutions in the order they were found
	pub(super) solutions: Vec<Solution>,
	/// Whether the solver proved there are no more solutions
	pub(super) complete: bool,
	/// Error output if `minizinc` failed (e.g. a type error in a candidate)
	pub(super) error: Option<String>,
}

/// Runs `minizinc` for one globalizer invocation (shared by all workers)
#[derive(Debug)]
pub(super) struct Runner<'a> {
	pub(super) options: &'a GlobalizeOptions,
	/// Number of `minizinc` invocations (reported as `NUMCALLS`)
	pub(super) calls: AtomicUsize,
	/// Outcomes of earlier runs, by a 128-bit hash of the submodel text, data
	/// file and number of solutions: variants and contexts often give the
	/// same submodel
	pub(super) cache: Mutex<FxHashMap<(u64, u64), Arc<Outcome>>>,
}

impl<'a> Runner<'a> {
	pub(super) fn new(options: &'a GlobalizeOptions) -> Self {
		Runner {
			options,
			calls: AtomicUsize::new(0),
			cache: Mutex::default(),
		}
	}

	/// Ask for `n` solutions of `model` (with the optional data file). The
	/// outcome is cached if `cache` is true.
	pub(super) fn run(
		&self,
		model: &str,
		data: Option<&Path>,
		n: usize,
		cache: bool,
	) -> Result<Arc<Outcome>> {
		if !cache {
			return Ok(Arc::new(self.run_uncached(model, data, n)?));
		}
		let hash = |salt: u8| {
			let mut hasher = DefaultHasher::new();
			(salt, model, data, n).hash(&mut hasher);
			hasher.finish()
		};
		let key = (hash(0), hash(1));
		if let Some(outcome) = self.cache.lock().unwrap().get(&key) {
			return Ok(outcome.clone());
		}
		let outcome = Arc::new(self.run_uncached(model, data, n)?);
		let _ = self.cache.lock().unwrap().insert(key, outcome.clone());
		Ok(outcome)
	}

	fn run_uncached(&self, model: &str, data: Option<&Path>, n: usize) -> Result<Outcome> {
		self.calls.fetch_add(1, Ordering::Relaxed);
		let mut file = Builder::new()
			.prefix("globalize")
			.suffix(".mzn")
			.tempfile()
			.and_then(|mut f| f.write_all(model.as_bytes()).map(|_| f))
			.map_err(|err| file_error(PathBuf::from("tempfile"), err))?;
		file.flush()
			.map_err(|err| file_error(file.path().to_owned(), err))?;
		log::debug!("globalizer submodel:\n{model}");

		let mut cmd = Command::new("minizinc");
		cmd.stdin(Stdio::null())
			.arg("--solver")
			.arg(&self.options.solver)
			.args(["--output-mode", "json", "--json-stream"])
			.arg("-n")
			.arg(n.to_string())
			.arg("-r")
			.arg(self.options.seed.to_string())
			.arg("--time-limit")
			.arg(self.options.time_limit.as_millis().to_string())
			.arg(file.path());
		if let Some(data) = data {
			cmd.arg(data);
		}
		let output = cmd.output().map_err(|err| {
			Error::from(InternalError::new(format!(
				"unable to run `minizinc` (is it on PATH?): {err}"
			)))
		})?;
		let mut outcome = parse(&String::from_utf8_lossy(&output.stdout));
		log::debug!(
			"globalizer run: {} solutions, complete: {}, error: {:?}",
			outcome.solutions.len(),
			outcome.complete,
			outcome.error
		);
		if !output.status.success() && outcome.error.is_none() {
			outcome.error = Some(String::from_utf8_lossy(&output.stderr).into_owned());
		}
		Ok(outcome)
	}
}

fn file_error(file: PathBuf, err: std::io::Error) -> Error {
	FileError {
		file,
		message: format!("unable to write globalizer submodel: {err}"),
		other: Vec::new(),
	}
	.into()
}

/// Parse the `--json-stream` output of `minizinc`
pub(super) fn parse(stdout: &str) -> Outcome {
	let mut outcome = Outcome::default();
	for line in stdout.lines() {
		let Ok(msg) = serde_json::from_str::<Value>(line) else {
			continue;
		};
		match msg.get("type").and_then(Value::as_str) {
			Some("solution") => {
				if let Some(Value::Object(sol)) = msg.pointer("/output/json") {
					outcome.solutions.push(sol.clone());
				}
			}
			Some("status") => match msg.get("status").and_then(Value::as_str) {
				// Unsatisfiable is complete with zero solutions
				Some("ALL_SOLUTIONS" | "OPTIMAL_SOLUTION" | "UNSATISFIABLE") => {
					outcome.complete = true
				}
				Some("ERROR") => outcome.error = Some("minizinc reported ERROR".to_owned()),
				_ => {}
			},
			Some("error") => {
				outcome.error = Some(
					msg.get("message")
						.and_then(Value::as_str)
						.unwrap_or("unknown minizinc error")
						.to_owned(),
				)
			}
			_ => {}
		}
	}
	outcome
}

#[cfg(test)]
mod tests {
	use super::parse;

	const SOL: &str = r#"{"type": "solution", "output": {"json": {  "x" : [1, 2],  "GZ_R" : [true, false]}}, "sections": ["json"]}"#;

	#[test]
	fn parses_solutions_without_status() {
		let out = parse(&format!("{SOL}\n{SOL}\n"));
		assert_eq!(out.solutions.len(), 2);
		assert_eq!(out.solutions[0]["GZ_R"][1], false);
		assert!(!out.complete);
		assert!(out.error.is_none());
	}

	#[test]
	fn all_solutions_is_complete() {
		let out = parse(&format!(
			"{SOL}\n{{\"type\": \"status\", \"status\": \"ALL_SOLUTIONS\"}}\n"
		));
		assert_eq!(out.solutions.len(), 1);
		assert!(out.complete);
	}

	#[test]
	fn unsatisfiable_is_complete_and_empty() {
		let out = parse("{\"type\": \"status\", \"status\": \"UNSATISFIABLE\"}\n");
		assert!(out.solutions.is_empty());
		assert!(out.complete);
	}

	#[test]
	fn error_message_is_kept() {
		let out = parse(
			"{\"type\": \"error\", \"what\": \"type error\", \"message\": \"no function all_different(int)\"}\n",
		);
		assert_eq!(out.error.as_deref(), Some("no function all_different(int)"));
	}

	#[test]
	fn garbage_lines_are_ignored() {
		let out = parse("% comment\nnot json\n");
		assert_eq!(out, Default::default());
	}
}
