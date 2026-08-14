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

// An output item is typed with references to outside declarations made par,
// because output is evaluated against the solved model. The declaration still
// lowers var, so the reference is fixed — otherwise HIR and THIR disagree on
// the inst and every call over the reference has to be silently re-dispatched
// to a var overload, which only works when one exists.
#[test]
fn test_lower_output_fixes_var_reference() {
	check_no_stdlib(
		r#"
		function $T: fix(var $T: x);
		function string: show(int: x);
		var int: x;
		output [show(x)];
		"#,
		expect![[r#"
    function $T: fix(var $T: x);
    function string: show(int: x);
    var int: x;
    output [show(fix(x))];
    solve satisfy;
"#]],
	);
}

// The same par-ification applies to the definition of an `::output_only`
// declaration, which the HIR typer routes through `collect_output_declaration`.
#[test]
fn test_lower_output_only_declaration_fixes_var_reference() {
	check_no_stdlib(
		r#"
		function $T: fix(var $T: x);
		annotation output_only;
		var int: x;
		int: y :: output_only = x;
		"#,
		expect![[r#"
    function $T: fix(var $T: x);
    annotation output_only;
    var int: x;
    int: y :: (output_only) = fix(x);
    solve satisfy;
"#]],
	);
}

// An assignment generator's value is lowered by a nested collector, which has
// to carry the output context across — otherwise the reference inside it loses
// the par-ification the typer applied and the two disagree.
#[test]
fn test_lower_output_comprehension_generator_fixes_var_reference() {
	check_no_stdlib(
		r#"
		function $T: fix(var $T: x);
		function string: show(int: x);
		var int: x;
		output [show(v) | i in {1}, v = x];
		"#,
		expect![[r#"
    function $T: fix(var $T: x);
    function string: show(int: x);
    var int: x;
    output [show(v) | i in {1}, v = fix(x)];
    solve satisfy;
"#]],
	);
}

// A `let` inside an output expression is collected through the ItemCollector,
// which builds a fresh ExpressionCollector — so the output context has to be
// handed across explicitly. Without that, a reference to an OUTSIDE declaration
// made from inside the let keeps the typer's par type but lowers var, and hits
// `collect_identifier`'s unreachable arm. Note the let-local `z` is NOT fixed:
// it is declared in this item, so the typer never par-ified it.
#[test]
fn test_lower_output_let_item_fixes_outer_var_reference() {
	check_no_stdlib(
		r#"
		function $T: fix(var $T: x);
		function string: show(var int: x);
		var int: y;
		output [show(let { var int: z = y } in z)];
		"#,
		expect![[r#"
    function $T: fix(var $T: x);
    function string: show(var int: x);
    var int: y;
    output [show(let {
      var int: z = fix(y);
    } in z)];
    solve satisfy;
"#]],
	);
}

// Outside an output context the reference keeps its var-ness: nothing is
// fixed, and the var overload is what the typer resolved against.
#[test]
fn test_lower_non_output_reference_is_not_fixed() {
	check_no_stdlib(
		r#"
		function $T: fix(var $T: x);
		predicate p(var int: x);
		var int: x;
		constraint p(x);
		"#,
		expect![[r#"
    function $T: fix(var $T: x);
    predicate p(var int: x);
    var int: x;
    constraint p(x);
    solve satisfy;
"#]],
	);
}
