use std::path::PathBuf;

use common::check_format_file;
use expect_test::expect_file;
use shackle_fmt::MiniZincFormatOptions;
mod common;

#[test]
fn format_1() {
	let options = MiniZincFormatOptions::default();
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests/formatting_1.mzn");
	let actual = check_format_file(&path, &options);
	let expected = expect_file![path];
	expected.assert_eq(&actual);
}

#[test]
fn format_2() {
	let options = MiniZincFormatOptions::default();
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests/formatting_2.mzn");
	let actual = check_format_file(&path, &options);
	let expected = expect_file![path];
	expected.assert_eq(&actual);
}

#[test]
fn format_3() {
	let options = MiniZincFormatOptions::default();
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests/formatting_3.mzn");
	let actual = check_format_file(&path, &options);
	let expected = expect_file![path];
	expected.assert_eq(&actual);
}

#[test]
fn format_4() {
	let options = MiniZincFormatOptions::default();
	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("tests/formatting_4.mzn");
	let actual = check_format_file(&path, &options);
	let expected = expect_file![path];
	expected.assert_eq(&actual);
}
