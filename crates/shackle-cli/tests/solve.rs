//! End-to-end tests for `shackle solve`: the model is compiled and handed to
//! the MiniZinc interpreter, and the standard output of the whole run is
//! compared against a checked-in expected file.

use std::{
	io::Read,
	path::PathBuf,
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
