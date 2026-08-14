//! End-to-end tests for `shackle solve`: the model is compiled and handed to
//! the MiniZinc interpreter, and the standard output of the whole run is
//! compared against a checked-in expected file.

use std::{path::PathBuf, process::Command};

use expect_test::expect_file;

fn fixture(name: &str, extension: &str) -> PathBuf {
	std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/solve")
		.join(name)
		.with_extension(extension)
}

/// Solve the given model fixture and compare the output with its expected file
fn check_solve(name: &str, all_solutions: bool) {
	let model = fixture(name, "mzn");
	let expected_path = fixture(name, "expected");

	let mut cmd = Command::new(env!("CARGO_BIN_EXE_shackle"));
	cmd.args(["solve", model.to_str().unwrap()]);
	if all_solutions {
		cmd.args(["--", "--all-solutions"]);
	}
	let output = cmd.output().unwrap();

	assert!(
		output.status.success(),
		"solve failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
		output.status,
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr),
	);
	expect_file![expected_path].assert_eq(&String::from_utf8(output.stdout).unwrap());
}

#[test]
fn solve_basic_all_solutions() {
	check_solve("basic", true);
}

#[test]
fn solve_output_item() {
	check_solve("output_item", false);
}

#[test]
fn solve_output_item_absent() {
	check_solve("output_item_absent", false);
}

#[test]
fn solve_object_var_singular() {
	check_solve("objects/01_var_singular", true);
}

#[test]
fn solve_object_par_set_literal() {
	check_solve("objects/02_par_set_literal", true);
}

#[test]
fn solve_object_bounded_set_new_small() {
	check_solve("objects/03_bounded_set_new_small", true);
}

#[test]
fn solve_object_nested_singular_object_par_only() {
	check_solve("objects/04_nested_singular_object_par_only", true);
}

#[test]
fn solve_object_inheritance_bounded() {
	check_solve("objects/06_inheritance_bounded", true);
}

#[test]
fn solve_object_deep_nested_depth3() {
	check_solve("objects/07_deep_nested_depth3", true);
}

#[test]
fn solve_object_computed_cardset_attr() {
	check_solve("objects/09_computed_cardset_attr", true);
}

#[test]
fn solve_object_optimize_set_new() {
	check_solve("objects/10_optimize_set_new", false);
}
