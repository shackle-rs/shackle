use expect_test::{Expect, expect};
use salsa::Setter;
use shackle_hir::{
	CompilerDatabase,
	input::{CompilerSettings, InlineModelFile, InputFiles},
};
use shackle_syntax::InputLang;

use crate::{lower::lower_model, pretty_print::PrettyPrinter};

/// Perform a transform on the THIR, and verify the result matches an expected value.
///
/// Turns off stdlib inclusion.
pub(crate) fn check_no_stdlib(source: &str, expected: Expect) {
	let mut db = CompilerDatabase::default();
	let _ = CompilerSettings::get(&db)
		.set_ignore_stdlib(&mut db)
		.to(true);
	let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
	let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
	let model = lower_model(&db).take();
	let pretty = PrettyPrinter::new(&db, &model).pretty_print();
	expected.assert_eq(&pretty);
}

#[test]
fn test_lower_named_args() {
	check_no_stdlib(
		r#"
		test foo(int: hello, int: world, int: bar, int: qux);
		any: x = foo(1, 2, qux: 4, bar: 3);
		"#,
		expect![[r#"
    function bool: foo(int: hello, int: world, int: bar, int: qux);
    bool: x = foo(1, 2, 3, 4);
    solve satisfy;
"#]],
	);
}
#[test]
fn test_lower_named_and_default_args() {
	check_no_stdlib(
		r#"
		test foo(int: hello, int: world, int: bar = 3, int: qux = 4);
		any: x = foo(1, world: 2, qux: 10);
		"#,
		expect![[r#"
    function bool: foo(int: hello, int: world, int: bar, int: qux);
    bool: x = foo(1, 2, 3, 10);
    solve satisfy;
"#]],
	);
}
