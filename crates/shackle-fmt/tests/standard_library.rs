use common::check_format_file;
use expect_test::expect_file;
use shackle_compiler::{db::CompilerSettings, CompilerDatabase};
use shackle_fmt::MiniZincFormatOptions;

mod common;

#[test]
fn format_stdlib() {
	let db = CompilerDatabase::default();
	let share = db.share_directory().unwrap();
	let mut p = share.to_string_lossy().into_owned();
	let options = MiniZincFormatOptions::default();
	p.push_str("/**/*.mzn");
	for entry in glob::glob(&p).unwrap() {
		let path = entry.unwrap();
		let actual = check_format_file(&path, &options);
		let expected = expect_file![path];
		expected.assert_eq(&actual);
	}
}
