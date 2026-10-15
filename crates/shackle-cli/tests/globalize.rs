//! End-to-end tests for `shackle globalize`. They need `MZN_STDLIB_DIR` and
//! `minizinc` with Gecode on `PATH`. Each test uses two worker threads. The
//! fixtures run from `tests/globalize` with `--json`, so locations use
//! relative file names. The expected files hold the JSON output without the
//! number of MiniZinc runs.

use std::{
	path::Path,
	process::{Command, Output},
};

use expect_test::expect_file;
use serde_json::Value;

fn dir() -> &'static Path {
	Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/globalize"))
}

fn globalize(args: &[&str]) -> Output {
	Command::new(env!("CARGO_BIN_EXE_shackle"))
		.current_dir(dir())
		.args(["globalize", "--seed", "1", "--jobs", "2"])
		.args(args)
		.output()
		.unwrap()
}

/// Run on the fixture `name` (plus data files) and compare with `name.expected`
fn check(name: &str, extra: &[&str]) {
	let model = format!("{name}.mzn");
	let output = globalize(&[&["--json"], extra, &[model.as_str()]].concat());
	assert!(
		output.status.success(),
		"globalize failed\nstdout:\n{}\nstderr:\n{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr),
	);
	let mut json: Value = serde_json::from_slice(&output.stdout).unwrap();
	// The number of MiniZinc runs changes with the timing of the workers
	let calls = json.as_object_mut().unwrap().remove("calls");
	assert!(calls.is_some_and(|c| c.is_u64()), "{json}");
	let expected = serde_json::to_string_pretty(&json).unwrap() + "\n";
	expect_file![dir().join(format!("{name}.expected"))].assert_eq(&expected);
}

#[test]
fn globalize_scalars() {
	check("scalars", &["-M", "3"]);
}

#[test]
fn globalize_pairwise_with_data() {
	check("pairwise", &["pairwise.dzn"]);
}

/// Car sequencing, with a `global_cardinality` on the class counts and a
/// `sliding_sum` for each option
#[test]
fn globalize_cars() {
	check("cars", &["-M", "1", "cars.dzn"]);
}

#[test]
fn globalize_bin_packing_with_data() {
	check("bin_packing", &["-M", "1", "bin_packing.dzn"]);
}

#[test]
fn globalize_latin_square_viewpoints() {
	check("latin01", &["-M", "1"]);
}

#[test]
fn globalize_queens() {
	check("queens", &[]);
}

#[test]
fn globalize_sudoku() {
	check("sudoku", &[]);
}

#[test]
fn globalize_increasing() {
	check("increasing", &[]);
}

#[test]
fn globalize_no_global() {
	check("no_global", &[]);
}

#[test]
fn globalize_prints_diagnostics() {
	let output = Command::new(env!("CARGO_BIN_EXE_shackle"))
		.current_dir(dir())
		.env("NO_COLOR", "1")
		.args(["globalize", "--seed", "1", "--jobs", "2", "-M", "3"])
		.arg("scalars.mzn")
		.output()
		.unwrap();
	assert!(output.status.success());
	let stdout = String::from_utf8(output.stdout).unwrap();
	assert!(
		stdout.contains("replace with `all_different([a, b, c])`"),
		"{stdout}"
	);
	assert!(stdout.contains("scalars.mzn:"), "{stdout}");
	assert!(stdout.contains("Suggestions: 1, viewpoints: 0"), "{stdout}");
}

#[test]
fn globalize_filters_globals_by_name() {
	let excluded = globalize(&["-e", "all_diff", "pairwise.mzn", "pairwise.dzn"]);
	assert!(excluded.status.success());
	assert!(!String::from_utf8_lossy(&excluded.stdout).contains("all_different"));
	let included = globalize(&["-f", "different", "pairwise.mzn", "pairwise.dzn"]);
	assert!(String::from_utf8_lossy(&included.stdout).contains("all_different(x)"));
}

#[test]
fn globalize_rejects_unsupported_file_type() {
	let output = globalize(&["pairwise.mzn", "notes.txt"]);
	assert!(!output.status.success());
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("unsupported file type"), "{stderr}");
}

#[test]
fn globalize_reports_type_errors() {
	let dir = tempfile::tempdir().unwrap();
	let model = dir.path().join("bad.mzn");
	std::fs::write(&model, "var 1..3: x;\nconstraint x = \"one\";\n").unwrap();
	let output = globalize(&[model.to_str().unwrap()]);
	assert!(!output.status.success());
	assert!(output.stdout.is_empty());
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(
		stderr.contains("No function '='(var int, string) could be found"),
		"{stderr}"
	);
}
