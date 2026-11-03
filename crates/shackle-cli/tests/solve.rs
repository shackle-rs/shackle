//! End-to-end tests for `shackle solve`: the model is compiled and handed to
//! the MiniZinc interpreter, and the standard output of the whole run is
//! compared against a checked-in expected file.

use std::{
	io::Read,
	path::{Path, PathBuf},
	process::{Command, Stdio},
	sync::mpsc,
	thread,
	time::Duration,
};

use expect_test::expect_file;

fn fixture(name: &str, extension: &str) -> PathBuf {
	std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/solve")
		.join(name)
		.with_extension(extension)
}

/// Solve the given model fixture and compare the output with its expected file
fn check_solve(name: &str, all_solutions: bool, target_minizinc: bool) {
	check_solve_with_data(name, &[], all_solutions, target_minizinc);
}

/// Solve the given model fixture with the given data files and compare the
/// output with the model's expected file
fn check_solve_with_data(name: &str, data: &[PathBuf], all_solutions: bool, target_minizinc: bool) {
	let output = solve(&fixture(name, "mzn"), data, all_solutions, target_minizinc);
	expect_file![fixture(name, "expected")].assert_eq(&output);
}

/// Solve a model with the given data files and return the standard output
fn solve(model: &Path, data: &[PathBuf], all_solutions: bool, target_minizinc: bool) -> String {
	let mut cmd = Command::new(env!("CARGO_BIN_EXE_shackle"));
	cmd.arg("solve").arg(model).args(data);
	if target_minizinc {
		cmd.args(["--target", "minizinc"]);
	}
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
	String::from_utf8(output.stdout).unwrap()
}

/// Solve a model with the given data files, which must fail, and return the
/// error report with its line wrapping undone
fn solve_error(model: &Path, data: &[PathBuf]) -> String {
	let output = Command::new(env!("CARGO_BIN_EXE_shackle"))
		.arg("solve")
		.arg(model)
		.args(data)
		.output()
		.unwrap();
	assert!(
		!output.status.success(),
		"solve succeeded\nstdout:\n{}",
		String::from_utf8_lossy(&output.stdout),
	);
	String::from_utf8_lossy(&output.stderr)
		.lines()
		.filter(|line| !line.starts_with("[WARN ]"))
		.flat_map(str::split_whitespace)
		.filter(|word| *word != "│")
		.collect::<Vec<_>>()
		.join(" ")
}

/// Solve the `18_data_errors` model with the data file `data` (a fixture
/// name), and check that it fails with an error report containing `expected`
/// that does not blame Shackle itself
fn check_object_data_error(data: &str, extension: &str, expected: &str) {
	let error = solve_error(
		&fixture("objects/18_data_errors", "mzn"),
		&[fixture(data, extension)],
	);
	assert!(
		error.contains(expected) && !error.contains("Internal Error"),
		"expected an error containing\n{expected}\nbut got\n{error}"
	);
}

/// A fresh directory for the files a test writes
fn scratch_dir(name: &str) -> PathBuf {
	let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
	let _ = std::fs::remove_dir_all(&dir);
	std::fs::create_dir_all(&dir).unwrap();
	dir
}

#[test]
fn solve_basic_all_solutions() {
	check_solve("basic", true, false);
}

#[test]
fn solve_output_item() {
	check_solve("output_item", false, false);
}

/// The MiniZinc target leaves the output item to MiniZinc
#[test]
fn solve_output_item_target_minizinc() {
	check_solve("output_item", false, true);
}

#[test]
fn solve_output_item_absent() {
	check_solve("output_item_absent", false, false);
}

#[test]
fn solve_enum_members() {
	check_solve("enum_members", true, false);
}

/// Values of a type whose constructors take arguments cannot be named yet, so
/// they are shown by position. The expected output records that gap: it becomes
/// MiniZinc's `Bar(1)` when the constructor arguments can be resolved.
#[test]
fn solve_enum_constructor() {
	check_solve("enum_constructor", true, false);
}

#[test]
fn solve_enum_constructor_index() {
	check_solve("enum_constructor_index", true, false);
}

/// Solve `basic` with a stand-in for the MiniZinc interpreter, a shell script
/// running `script`, and return the error report with its line wrapping undone
#[cfg(unix)]
fn solve_with_failing_interpreter(name: &str, script: &str) -> String {
	use std::os::unix::fs::PermissionsExt;

	let dir = scratch_dir(name);
	let interpreter = dir.join("minizinc");
	std::fs::write(&interpreter, format!("#!/bin/sh\n{script}\n")).unwrap();
	std::fs::set_permissions(&interpreter, std::fs::Permissions::from_mode(0o755)).unwrap();
	let path = std::env::join_paths(
		std::iter::once(dir).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
	)
	.unwrap();
	let output = Command::new(env!("CARGO_BIN_EXE_shackle"))
		.arg("solve")
		.arg(fixture("basic", "mzn"))
		.env("PATH", path)
		.output()
		.unwrap();
	assert!(
		!output.status.success(),
		"solve succeeded\nstdout:\n{}",
		String::from_utf8_lossy(&output.stdout),
	);
	String::from_utf8_lossy(&output.stderr)
		.split_whitespace()
		.collect::<Vec<_>>()
		.join(" ")
}

/// An interpreter that fails without reporting an error fails the solve
#[cfg(unix)]
#[test]
fn solve_interpreter_exit_code() {
	let error = solve_with_failing_interpreter("solve_interpreter_exit_code", "exit 3");
	assert!(error.contains("exit status: 3"), "{error}");
}

/// An interpreter that crashes fails the solve, rather than the crash being
/// reported as a successful run, or making shackle panic
#[cfg(unix)]
#[test]
fn solve_interpreter_crash() {
	let error = solve_with_failing_interpreter("solve_interpreter_crash", "kill -SEGV $$");
	assert!(
		error.contains("signal: 11") && !error.contains("panicked"),
		"{error}"
	);
}

#[test]
fn solve_enum_equality() {
	check_solve("enum_equality", true, false);
}

#[test]
fn solve_opt_domain_absent() {
	check_solve("opt_domain_absent", true, false);
}

#[test]
fn solve_opt_array_output() {
	check_solve("opt_array_output", true, false);
}

#[test]
fn solve_opt_bool_array() {
	check_solve("opt_bool_array", false, false);
}

/// The interpreter's output has to be read as it is produced, so a solution
/// bigger than the pipe's buffer must not deadlock the run. The expected output
/// is too big to keep in a file, and a regression makes the solve hang rather
/// than produce something to compare, so this checks the shape under a deadline.
#[test]
#[ignore = "takes too long and needs too much memory"]
fn solve_large_output() {
	let model = fixture("large_output", "mzn");
	let mut child = Command::new(env!("CARGO_BIN_EXE_shackle"))
		.args(["solve", model.to_str().unwrap()])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.unwrap();
	// The output is bigger than the pipe holds, so it has to be read while the
	// solve is still running whether or not the deadlock is back. Reading the
	// (small) error output afterwards is safe: by then the solve has finished.
	let mut stdout = child.stdout.take().unwrap();
	let mut stderr = child.stderr.take().unwrap();
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || {
		let mut out = String::new();
		let mut err = String::new();
		let _ = stdout.read_to_string(&mut out);
		let _ = stderr.read_to_string(&mut err);
		let _ = sender.send((out, err));
	});

	let (stdout, stderr) = match receiver.recv_timeout(Duration::from_secs(120)) {
		Ok(output) => output,
		Err(_) => {
			// A deadlocked solve never exits, and takes the interpreter with it
			let _ = child.kill();
			let _ = child.wait();
			panic!("solve did not return a large solution within 120s");
		}
	};
	let status = child.wait().unwrap();

	assert!(
		status.success(),
		"solve failed\nstatus: {status}\nstderr:\n{stderr}"
	);
	// The 20000 lines of the output item, followed by the solution separator
	assert_eq!(stdout.lines().count(), 20001);
	assert!(stdout.starts_with("line 1 of a large output, x=2\n"));
	assert!(stdout.ends_with("line 20000 of a large output, x=2\n----------\n"));
}

#[test]
fn solve_object_var_singular() {
	check_solve("objects/01_var_singular", true, false);
}

#[test]
fn solve_object_par_set_literal() {
	check_solve("objects/02_par_set_literal", true, false);
}

#[test]
fn solve_object_bounded_set_new_small() {
	check_solve("objects/03_bounded_set_new_small", true, false);
}

#[test]
fn solve_object_nested_singular_object_par_only() {
	check_solve("objects/04_nested_singular_object_par_only", true, false);
}

#[test]
fn solve_object_nested_singular_object_var_reach() {
	check_solve("objects/05_nested_singular_object_var_reach", true, false);
}

#[test]
fn solve_object_inheritance_bounded() {
	check_solve("objects/06_inheritance_bounded", true, false);
}

#[test]
fn solve_object_deep_nested_depth3() {
	check_solve("objects/07_deep_nested_depth3", true, false);
}

#[test]
fn solve_object_mixed_par_var_children() {
	check_solve("objects/08_mixed_par_var_children", true, false);
}

#[test]
fn solve_object_computed_cardset_attr() {
	check_solve("objects/09_computed_cardset_attr", true, false);
}

#[test]
fn solve_object_optimize_set_new() {
	check_solve("objects/10_optimize_set_new", false, false);
}

#[test]
fn solve_include_globals() {
	check_solve("include_globals", false, true);
}

#[test]
fn solve_target_data_output() {
	let name = "target_data_output";
	check_solve_with_data(name, &[fixture(name, "dzn")], false, false);
	check_solve_with_data(name, &[fixture(name, "dzn")], false, true);
}

/// Only with the MiniZinc target: the MicroZinc target fails on an enum
/// defined in data
#[test]
fn solve_target_minizinc_data_enum() {
	let name = "target_minizinc_data_enum";
	check_solve_with_data(name, &[fixture(name, "dzn")], false, true);
}

#[test]
fn solve_object_output_ids() {
	check_solve("objects/11_output_ids", false, false);
}

#[test]
fn solve_object_show_ids() {
	check_solve("objects/12_show_ids", false, false);
}

#[test]
fn solve_object_reference_existence_unsat() {
	check_solve("objects/13_reference_existence_unsat", false, false);
}

#[test]
fn solve_object_reference_existence() {
	check_solve("objects/14_reference_existence", false, false);
}

#[test]
fn solve_object_class_indexed_output() {
	check_solve("objects/20_class_indexed_output", false, false);
}

#[test]
fn solve_object_class_indexed_show() {
	check_solve("objects/21_class_indexed_show", false, false);
}

#[test]
fn solve_object_class_name_attribute() {
	check_solve("objects/22_class_name_attribute", false, false);
}

#[test]
fn solve_object_data_dzn() {
	let name = "objects/15_data_objects";
	check_solve_with_data(name, &[fixture(name, "dzn")], false, false);
}

#[test]
fn solve_object_data_json() {
	let name = "objects/15_data_objects";
	check_solve_with_data(name, &[fixture(name, "json")], false, false);
}

#[test]
fn solve_object_data_mutual_references() {
	let name = "objects/16_data_mutual_references";
	check_solve_with_data(name, &[fixture(name, "dzn")], false, false);
}

#[test]
fn solve_object_data_without_ids() {
	let name = "objects/19_data_without_ids";
	check_solve_with_data(name, &[fixture(name, "dzn")], false, false);
	check_solve_with_data(name, &[fixture(name, "json")], false, false);
}

#[test]
fn solve_object_data_unknown_id_in_attribute() {
	let expected = "Invalid object data: no Leaf object with id \"zz\" in the data, \
	                referenced by attribute 'pick' of the Holder object \"h1\"";
	let data = "objects/18_data_errors_unknown_attribute";
	check_object_data_error(data, "dzn", expected);
	check_object_data_error(data, "json", expected);
}

#[test]
fn solve_object_data_unknown_id_in_attribute_without_id() {
	check_object_data_error(
		"objects/18_data_errors_unknown_attribute_without_id",
		"dzn",
		"Invalid object data: no Leaf object with id \"zz\" in the data, referenced by attribute \
		 'pick' of a Holder object without an id",
	);
}

#[test]
fn solve_object_data_unknown_id_in_reference() {
	check_object_data_error(
		"objects/18_data_errors_unknown_reference",
		"dzn",
		"Invalid object data: no Leaf object with id \"zz\" in the data, referenced by 'top'",
	);
}

#[test]
fn solve_object_data_duplicate_id() {
	check_object_data_error(
		"objects/18_data_errors_duplicate",
		"dzn",
		"Invalid object data: more than one Leaf object has the id \"l2\"",
	);
}

/// A data id may not repeat the generated id of an object the model creates
#[test]
fn solve_object_data_duplicate_generated_id() {
	check_object_data_error(
		"objects/18_data_errors_generated_duplicate",
		"dzn",
		"Invalid object data: more than one Leaf object has the id \"g\"",
	);
}

#[test]
fn solve_object_data_empty_id() {
	check_object_data_error(
		"objects/18_data_errors_empty_id",
		"dzn",
		"Invalid object data: object ids must not be empty, but a Leaf object in the data \
		 has the id \"\"",
	);
}

/// Every declaration left without a value is reported, under its own name
#[test]
fn solve_object_data_missing() {
	let data = "objects/18_data_errors_missing";
	check_object_data_error(data, "dzn", "No value given for 'holders'");
	check_object_data_error(data, "dzn", "No value given for 'top'");
}

/// The compiled model itself reads data under the user's names, so plain
/// `minizinc` runs it. The object helpers are passed along until they are part
/// of MiniZinc's standard library.
#[test]
fn compile_object_data_then_minizinc() {
	let name = "objects/15_data_objects";
	let dir = scratch_dir("compile_object_data_then_minizinc");
	let model = dir.join("model.mzn");
	std::fs::copy(fixture(name, "mzn"), &model).unwrap();

	let output = Command::new(env!("CARGO_BIN_EXE_shackle"))
		.args(["compile", "--target", "minizinc"])
		.arg(&model)
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"compile failed\nstderr:\n{}",
		String::from_utf8_lossy(&output.stderr),
	);

	let helpers =
		Path::new(env!("CARGO_MANIFEST_DIR")).join("../../share/minizinc/std/internal/object.mzn");
	let output = Command::new("minizinc")
		.args(["--solver", "gecode"])
		.arg(dir.join("model.shackle.mzn"))
		.arg(helpers)
		.arg(fixture(name, "dzn"))
		.output()
		.unwrap();
	assert!(
		output.status.success(),
		"minizinc failed\nstdout:\n{}\nstderr:\n{}",
		String::from_utf8_lossy(&output.stdout),
		String::from_utf8_lossy(&output.stderr),
	);
	expect_file![fixture("objects/15_data_objects_minizinc", "expected")]
		.assert_eq(&String::from_utf8(output.stdout).unwrap());
}

/// A solution is printed in the data form, so it can be read back as data.
#[test]
fn solve_object_round_trip() {
	let solution = solve(
		&fixture("objects/17_round_trip_solution", "mzn"),
		&[],
		false,
		false,
	);
	expect_file![fixture("objects/17_round_trip_solution", "expected")].assert_eq(&solution);

	let dir = scratch_dir("solve_object_round_trip");
	let data = dir.join("solution.dzn");
	let assignments = solution
		.lines()
		.filter(|line| !line.starts_with("---") && !line.starts_with("==="))
		.collect::<Vec<_>>()
		.join("\n");
	std::fs::write(&data, assignments).unwrap();
	check_solve_with_data("objects/17_round_trip_data", &[data], false, false);
}

#[test]
fn solve_object_var_reference_equality() {
	check_solve("objects/23_var_reference_equality", true, false);
}

#[test]
fn solve_object_inline_references() {
	check_solve("objects/24_inline_references", false, false);
}

#[test]
fn solve_object_var_opt_new_child_in_var_set() {
	check_solve("objects/25_var_opt_new_child_in_var_set", true, false);
}

#[test]
fn solve_object_var_opt_class_indexed_output() {
	check_solve("objects/26_var_opt_class_indexed_output", true, false);
}

#[test]
fn solve_object_names_of_internal_locals() {
	check_solve("objects/27_names_of_internal_locals", false, false);
}

#[test]
fn solve_object_two_par_roots_nested() {
	check_solve("objects/28_two_par_roots_nested", false, false);
}

#[test]
fn solve_object_var_and_par_roots_nested() {
	check_solve("objects/29_var_and_par_roots_nested", false, false);
}

#[test]
fn solve_object_reference_into_same_data() {
	check_solve("objects/30_reference_into_same_data", false, false);
}

#[test]
fn solve_object_reference_into_same_data_children() {
	check_solve("objects/31_reference_into_same_data_children", false, false);
}

#[test]
fn solve_object_empty_record_data() {
	check_solve("objects/32_empty_record_data", false, false);
}

#[test]
fn solve_object_var_existence_in_par_child() {
	check_solve("objects/33_var_existence_in_par_child", false, false);
}

#[test]
fn solve_object_var_subclass_root_par_superclass_root() {
	check_solve(
		"objects/34_var_subclass_root_par_superclass_root",
		true,
		false,
	);
}

#[test]
fn solve_object_var_collection_root_first() {
	check_solve("objects/35_var_collection_root_first", true, false);
}

#[test]
fn solve_object_two_subclass_collection_roots() {
	check_solve("objects/36_two_subclass_collection_roots", false, false);
}

#[test]
fn solve_object_reference_in_nested_object() {
	check_solve("objects/37_reference_in_nested_object", false, false);
}

#[test]
fn solve_object_var_opt_root() {
	check_solve("objects/38_var_opt_root", true, false);
}

#[test]
fn solve_opt_tuple_record() {
	let name = "opt_tuple_record";
	for data in ["dzn", "json"] {
		check_solve_with_data(name, &[fixture(name, data)], false, false);
		check_solve_with_data(name, &[fixture(name, data)], false, true);
	}
}
