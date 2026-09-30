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
fn check_solve(name: &str, all_solutions: bool) {
	check_solve_with_data(name, &[], all_solutions);
}

/// Solve the given model fixture with the given data files and compare the
/// output with the model's expected file
fn check_solve_with_data(name: &str, data: &[PathBuf], all_solutions: bool) {
	let output = solve(&fixture(name, "mzn"), data, all_solutions);
	expect_file![fixture(name, "expected")].assert_eq(&output);
}

/// Solve a model with the given data files and return the standard output
fn solve(model: &Path, data: &[PathBuf], all_solutions: bool) -> String {
	let mut cmd = Command::new(env!("CARGO_BIN_EXE_shackle"));
	cmd.arg("solve").arg(model).args(data);
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

/// A fresh directory for the files a test writes
fn scratch_dir(name: &str) -> PathBuf {
	let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
	let _ = std::fs::remove_dir_all(&dir);
	std::fs::create_dir_all(&dir).unwrap();
	dir
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
fn solve_enum_members() {
	check_solve("enum_members", true);
}

/// Values of a type whose constructors take arguments cannot be named yet, so
/// they are shown by position. The expected output records that gap: it becomes
/// MiniZinc's `Bar(1)` when the constructor arguments can be resolved.
#[test]
fn solve_enum_constructor() {
	check_solve("enum_constructor", true);
}

#[test]
fn solve_enum_constructor_index() {
	check_solve("enum_constructor_index", true);
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
fn solve_object_nested_singular_object_var_reach() {
	check_solve("objects/05_nested_singular_object_var_reach", true);
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
fn solve_object_mixed_par_var_children() {
	check_solve("objects/08_mixed_par_var_children", true);
}

#[test]
fn solve_object_computed_cardset_attr() {
	check_solve("objects/09_computed_cardset_attr", true);
}

#[test]
fn solve_object_optimize_set_new() {
	check_solve("objects/10_optimize_set_new", false);
}

#[test]
fn solve_object_output_ids() {
	check_solve("objects/11_output_ids", false);
}

#[test]
fn solve_object_show_ids() {
	check_solve("objects/12_show_ids", false);
}

#[test]
fn solve_object_reference_existence_unsat() {
	check_solve("objects/13_reference_existence_unsat", false);
}

#[test]
fn solve_object_reference_existence() {
	check_solve("objects/14_reference_existence", false);
}

#[test]
fn solve_object_data_dzn() {
	let name = "objects/15_data_objects";
	check_solve_with_data(name, &[fixture(name, "dzn")], false);
}

#[test]
fn solve_object_data_json() {
	let name = "objects/15_data_objects";
	check_solve_with_data(name, &[fixture(name, "json")], false);
}

#[test]
fn solve_object_data_mutual_references() {
	let name = "objects/16_data_mutual_references";
	check_solve_with_data(name, &[fixture(name, "dzn")], false);
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
	check_solve_with_data("objects/17_round_trip_data", &[data], false);
}
