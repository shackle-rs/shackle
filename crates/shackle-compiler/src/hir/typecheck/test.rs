use std::sync::Arc;

use expect_test::{expect, Expect};

use crate::{
	db::{CompilerDatabase, FileReader, Inputs},
	file::InputFile,
	hir::{db::Hir, ids::LocalItemRef},
	ty::Ty,
};

#[derive(Default)]
struct TypeTester {
	db: CompilerDatabase,
}

impl TypeTester {
	fn check_expression(&mut self, expr: &str, expected: Expect) {
		let ty = self.type_expression("", expr);
		let pretty = ty.pretty_print(&self.db);
		expected.assert_eq(&pretty);
	}

	fn check_expression_preamble(&mut self, preamble: &str, expr: &str, expected: Expect) {
		let ty = self.type_expression(preamble, expr);
		let pretty = ty.pretty_print(&self.db);
		expected.assert_eq(&pretty);
	}

	fn check_error(&mut self, model: &str, expected: Expect) {
		self.db
			.set_input_files(Arc::new(vec![InputFile::ModelString(model.to_owned())]));
		let mut errors = Vec::new();
		for m in self.db.resolve_includes().unwrap().iter() {
			for i in self.db.lookup_items(*m).iter() {
				for e in self.db.lookup_item_type_errors(*i).outer_iter() {
					errors.extend(e.iter().cloned());
				}
			}
		}
		let result = errors
			.iter()
			.map(|e| e.to_string())
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&result);
	}

	fn type_expression(&mut self, preamble: &str, expr: &str) -> Ty {
		self.db.set_input_files(Arc::new(vec![
			InputFile::ModelString(format!("any: _TEST_EXPR = {};", expr)),
			InputFile::ModelString(preamble.to_owned()),
		]));
		let model = self.db.input_models();
		let items = self.db.lookup_items(model[0]);
		let item = *items.last().unwrap();
		let types = self.db.lookup_item_types(item);
		let local = item.local_item_ref(&self.db);
		match local {
			LocalItemRef::Declaration(d) => {
				let e = item.model(&self.db)[d].definition.unwrap();
				types[e]
			}
			_ => unreachable!(),
		}
	}
}

#[test]
fn test_type_expressions() {
	let mut tester = TypeTester::default();
	tester.check_expression("true", expect!("bool"));
	tester.check_expression("false", expect!("bool"));
	tester.check_expression("1", expect!("int"));
	tester.check_expression("infinity", expect!("int"));
	tester.check_expression("1.5", expect!("float"));
	tester.check_expression(r#""foo""#, expect!("string"));
	tester.check_expression(r#"empty_annotation"#, expect!("ann"));
	tester.check_expression("[]", expect!("array [..] of .."));
	tester.check_expression("[1, 2, 3]", expect!("array [int] of int"));
	tester.check_expression("[1, 2.5]", expect!("array [int] of float"));
	tester.check_expression("[|1, 2|3, 4|]", expect!("array [int, int] of int"));
	tester.check_expression(r#"["foo", 1]"#, expect!("array [int] of error"));
	tester.check_expression("{1, 3}", expect!("set of int"));
	tester.check_expression("{1.5, 3}", expect!("set of float"));
	tester.check_expression("{}", expect!("set of .."));
	tester.check_expression(r#"{"foo", 1}"#, expect!("set of error"));
	tester.check_expression("(1, 2.5)", expect!("tuple(int, float)"));
	tester.check_expression("(1, (2, 3.5))", expect!("tuple(int, tuple(int, float))"));
	tester.check_expression(
		r#"(a: 1, b: (c: 2.5, d: "foo"))"#,
		expect!("record(int: a, record(float: c, string: d): b)"),
	);
	tester.check_expression("[i | i in 1..3]", expect!("array [int] of int"));
	tester.check_expression(
		"let { var 1..3: x } in [i | i in [x, 2 * x]]",
		expect!("array [int] of var int"),
	);
	tester.check_expression(
		"let { var bool: p } in [i | i in 1..3 where p]",
		expect!("array [int] of var opt int"),
	);
	tester.check_expression("{i | i in 1..3}", expect!("set of int"));
	tester.check_expression(
		"let { var bool: p } in {i | i in 1..3 where p}",
		expect!("var set of int"),
	);
	tester.check_expression(
		"let { var set of 1..3: s } in {i | i in s}",
		expect!("var set of int"),
	);
	tester.check_expression("let { any: x = (1, 2) } in x.1", expect!("int"));
	tester.check_expression(
		"let { any: x = (1, (1.5, 2)) } in x.2",
		expect!("tuple(float, int)"),
	);
	tester.check_expression("let { any: x = (a: 1, b: 2) } in x.a", expect!("int"));
	tester.check_expression(
		"let { any: x = (a: 1, b: (c: 1.5, d: 2)) } in x.b",
		expect!("record(float: c, int: d)"),
	);
	tester.check_expression("if true then 1 else 2 endif", expect!("int"));
	tester.check_expression(
		"if true then [1] else [2] endif",
		expect!("array [int] of int"),
	);
	tester.check_expression(
		r#"
        let {
            var bool: p;
        } in if p then 1 else 2 endif
        "#,
		expect!("var int"),
	);
	tester.check_expression(
		r#"
        let {
            var bool: p;
        } in if p then [1] else [2] endif
        "#,
		expect!("error"),
	);
	tester.check_expression("[1, 2, 3][1]", expect!("int"));
	tester.check_expression(
		r#"
        let {
            var 1..3: i;
        } in [1, 2, 3][i]
        "#,
		expect!("var int"),
	);
	tester.check_expression("[1, 2, 3][..]", expect!("array [int] of int"));
	tester.check_expression("[|1, 2|3, 4|][.., 2]", expect!("array [int] of int"));
	tester.check_expression("[|1, 2|3, 4|][1, 2..]", expect!("array [int] of int"));
	tester.check_expression(
		r#"
        case 1 of
            1 => 1,
            2 => 1.5,
            _ => 3
        "#,
		expect!("float"),
	);
	tester.check_expression("lambda (int: x) => x", expect!("op(int: (int))"));
	tester.check_expression(
		"lambda var int: (var bool: x) => x",
		expect!("op(var int: (var bool))"),
	);
	tester.check_expression("let { var int: x; } in true", expect!("var bool"));
	tester.check_expression(
		"let { constraint let { var bool: p } in p } in true",
		expect!("var bool"),
	);
	tester.check_expression("(lambda int: (int: x) => x)(1)", expect!("int"));
}

#[test]
fn test_function_resolution() {
	let mut tester = TypeTester::default();
	tester.check_expression_preamble(
		r#"
        function bool: foo(bool);
        function int: foo(int);
        function var int: foo(var int);
        function bool: foo(int);
        "#,
		"foo(1)",
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function bool: foo(bool);
        function int: foo(int);
        function var int: foo(var int);
        function bool: foo(int);
        var bool: p;
        "#,
		"foo(p)",
		expect!("var int"),
	);
	tester.check_expression_preamble(
		r#"
        function any $T: foo(any $T);
        var 1..3: x;
        "#,
		"foo(x)",
		expect!("var int"),
	);
	tester.check_expression_preamble(
		r#"
        function any $T: foo(any $T);
        function bool: foo(var bool);
        var bool: x;
        "#,
		"foo(x)",
		expect!("bool"),
	);
	tester.check_expression_preamble(
		r#"
        function var $$E: foo($$E);
        "#,
		"foo(123)",
		expect!("var int"),
	);
	tester.check_expression_preamble(
		r#"
        function var $$E: foo($$E);
        enum Foo = {A};
        "#,
		"foo(A)",
		expect!("var Foo"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int, float);
        function int: foo(float, int);
        "#,
		"foo(1, 1)",
		expect!("error"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int, float);
        function int: foo(float, int);
        function int: foo(float, float);
        "#,
		"foo(1, 1)",
		expect!("error"),
	);
}

#[test]
fn test_type_errors() {
	let mut tester = TypeTester::default();
	tester.check_error(
		r#"
		int: x = 1.5;
		"#,
		expect!("Type mismatch"),
	);
	tester.check_error(
		r#"
		array [float] of int: x;
		"#,
		expect!("Illegal type"),
	);
	tester.check_error(
		r#"
		[1]: x;
		"#,
		expect!("Type mismatch"),
	);
	tester.check_error(
		r#"
		any: x = [1, "two"];
		"#,
		expect!("Invalid array literal"),
	);
	tester.check_error(
		r#"
		any: x = nope;
		"#,
		expect!("Undefined identifier"),
	);
}
