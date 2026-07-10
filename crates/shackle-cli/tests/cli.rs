use std::{
	fs,
	path::Path,
	process::{Command, Output},
};

use tempfile::{TempDir, tempdir};

fn shackle() -> Command {
	Command::new(env!("CARGO_BIN_EXE_shackle"))
}

fn write(path: impl AsRef<Path>, contents: impl AsRef<str>) {
	fs::write(path, contents.as_ref()).unwrap();
}

fn assert_success(output: Output) -> Output {
	if !output.status.success() {
		panic!(
			"expected command to succeed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
			output.status,
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr)
		);
	}
	output
}

fn assert_failure(output: Output) -> Output {
	if output.status.success() {
		panic!(
			"expected command to fail\nstdout:\n{}\nstderr:\n{}",
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr)
		);
	}
	output
}

fn unformatted_model(dir: &TempDir) -> std::path::PathBuf {
	let model = dir.path().join("format_me.mzn");
	write(
		&model,
		"array [int] of int: xs=[1,2,3,4,5,6,7,8];\nint: y=(1+(2*3));\n",
	);
	model
}

#[test]
fn format_rewrites_files_using_config_and_cli_overrides() {
	let dir = tempdir().unwrap();
	let model = unformatted_model(&dir);
	let config = dir.path().join("shackle-format.json");
	write(
		&config,
		r#"{"lineWidth":20,"useTabs":true,"indentSize":2,"keepParentheses":true}"#,
	);

	let output = shackle()
		.args([
			"format",
			"--config",
			config.to_str().unwrap(),
			"--use-tabs",
			"0",
			"--keep-parentheses",
			"false",
			model.to_str().unwrap(),
		])
		.output()
		.unwrap();

	assert_success(output);
	let formatted = fs::read_to_string(model).unwrap();
	assert!(formatted.contains("  1,"));
	assert!(!formatted.contains('\t'));
	assert!(formatted.contains("int: y = 1 + 2 * 3;"));
}

#[test]
fn format_check_reports_diff_without_changing_file() {
	let dir = tempdir().unwrap();
	let model = unformatted_model(&dir);
	let original = fs::read_to_string(&model).unwrap();

	let output = shackle()
		.args(["format", "--check", model.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stdout = String::from_utf8_lossy(&output.stdout);
	assert!(stdout.contains("--- "));
	assert!(stdout.contains("+++ "));
	assert_eq!(fs::read_to_string(model).unwrap(), original);
}

#[test]
fn format_check_accepts_verbose_flags() {
	let dir = tempdir().unwrap();
	let model = dir.path().join("already_formatted.mzn");
	write(&model, "int: x = 1;\n");

	for verbose in ["-v", "-vv", "-vvv"] {
		let output = shackle()
			.args([verbose, "format", "--check", model.to_str().unwrap()])
			.output()
			.unwrap();
		assert_success(output);
	}
}

#[test]
fn check_reports_missing_model_file() {
	let dir = tempdir().unwrap();
	let data = dir.path().join("instance.dzn");
	write(&data, "n = 3;\n");

	let output = shackle()
		.args(["check", "--solver", "test-solver", data.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("no model file detected"));
}

#[test]
fn check_accepts_model_and_data_files() {
	let dir = tempdir().unwrap();
	let model = dir.path().join("instance.mzn");
	let data = dir.path().join("instance.dzn");
	write(&model, "int: n;\n");
	write(&data, "n = 3;\n");

	let output = shackle()
		.args([
			"check",
			"--solver",
			"test-solver",
			model.to_str().unwrap(),
			data.to_str().unwrap(),
		])
		.output()
		.unwrap();

	assert_success(output);
}

#[test]
fn compile_writes_shackle_model() {
	let dir = tempdir().unwrap();
	let model = dir.path().join("model.mzn");
	write(&model, "var int: x;\nsolve satisfy;\n");

	let output = shackle()
		.args([
			"compile",
			"--solver",
			"test-solver",
			model.to_str().unwrap(),
		])
		.output()
		.unwrap();

	assert_success(output);
	let compiled = model.with_extension("shackle.mzn");
	let compiled_source = fs::read_to_string(compiled).unwrap();
	assert!(!compiled_source.is_empty());
	assert!(compiled_source.contains("solve satisfy;"));
}

#[test]
fn compile_reports_missing_or_invalid_minizinc_stdlib() {
	let dir = tempdir().unwrap();
	let model = dir.path().join("model.mzn");
	write(&model, "var int: x;\nsolve satisfy;\n");

	let output = shackle()
		.env_remove("MZN_STDLIB_DIR")
		.args(["compile", model.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("Failed to locate the MiniZinc standard library"));
	assert!(stderr.contains("MZN_STDLIB_DIR"));

	let output = shackle()
		.env("MZN_STDLIB_DIR", dir.path().join("not-minizinc"))
		.args(["compile", model.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("Failed to locate the MiniZinc standard library"));
	assert!(stderr.contains("MZN_STDLIB_DIR"));
}

#[test]
fn compile_rejects_unsupported_file_types() {
	let dir = tempdir().unwrap();
	let model = dir.path().join("model.mzn");
	let readme = dir.path().join("notes.txt");
	write(&model, "var 1..3: x;\nsolve satisfy;\n");
	write(&readme, "not a MiniZinc data file\n");

	let output = shackle()
		.args(["compile", model.to_str().unwrap(), readme.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("unsupported file type"));
	assert!(stderr.contains("notes.txt"));
}

#[test]
fn compile_rejects_multiple_model_files() {
	let dir = tempdir().unwrap();
	let first = dir.path().join("first.mzn");
	let second = dir.path().join("second.mzn");
	write(&first, "var 1..3: x;\nsolve satisfy;\n");
	write(&second, "var 1..3: y;\nsolve satisfy;\n");

	let output = shackle()
		.args(["compile", first.to_str().unwrap(), second.to_str().unwrap()])
		.output()
		.unwrap();

	let output = assert_failure(output);
	let stderr = String::from_utf8_lossy(&output.stderr);
	assert!(stderr.contains("detected multiple model files"));
}
