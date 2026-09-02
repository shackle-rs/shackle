use expect_test::{Expect, expect};
use salsa::Setter;
use shackle_syntax::InputLang;

use crate::{
	Db, Item,
	db::CompilerDatabase,
	diagnostics::Errors,
	input::{InlineModelFile, InputFiles, ModelFile},
	typecheck::{accumulate_typecheck_diagnostics, typecheck},
};

#[salsa::tracked(returns(copy))]
fn compute_types(db: &dyn Db) {
	typecheck(db);
	accumulate_typecheck_diagnostics(db);
}

struct TypeTester {
	db: CompilerDatabase,
	preamble: ModelFile,
	file: ModelFile,
}

impl TypeTester {
	fn new() -> Self {
		let mut db = CompilerDatabase::default();
		let preamble = InlineModelFile::new(&db, "".to_owned(), InputLang::MiniZinc).into();
		let file = InlineModelFile::new(&db, "".to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![preamble, file]);
		Self { db, preamble, file }
	}

	fn set_model(&mut self, preamble: &str, model: &str) {
		let _ = self
			.preamble
			.unwrap_inline()
			.set_contents(&mut self.db)
			.to(preamble.to_owned());
		let _ = self
			.file
			.unwrap_inline()
			.set_contents(&mut self.db)
			.to(model.to_owned());
	}

	fn check_expression(&mut self, expr: &str, expected: Expect) {
		let ty = self.type_expression("", expr);
		expected.assert_eq(&ty);
	}

	fn check_expression_preamble(&mut self, preamble: &str, expr: &str, expected: Expect) {
		let ty = self.type_expression(preamble, expr);
		expected.assert_eq(&ty);
	}

	fn check_error(&mut self, model: &str, expected: Expect) {
		self.set_model("", model);
		let errors = compute_types::accumulated::<Errors>(&self.db);
		let result = errors
			.iter()
			.map(|e| e.to_string())
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&result);
	}

	fn type_expression(&mut self, preamble: &str, expr: &str) -> String {
		self.set_model(preamble, &format!("any: _TEST_EXPR = {};", expr));
		let lowered = self.file.hir(&self.db);
		let item = lowered.items(&self.db).first().unwrap();
		let types = item.types(&self.db);
		let definition = match item {
			Item::Declaration(d) => d.declaration(&self.db).definition.unwrap(),
			x => unreachable!("{:?}", x),
		};
		types
			.get_expression(definition)
			.unwrap()
			.pretty_print(&self.db)
	}
}

#[test]
fn test_type_expressions() {
	let mut tester = TypeTester::new();
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
		expect!("array [int] of var int"),
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
	tester.check_expression("let { var int: x; } in true", expect!("bool"));
	tester.check_expression(
		"let { constraint let { var bool: p } in p } in true",
		expect!("var bool"),
	);
	tester.check_expression("(lambda int: (int: x) => x)(1)", expect!("int"));
}

#[test]
fn test_function_resolution() {
	let mut tester = TypeTester::new();
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
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x, string: y);
		"#,
		r#"foo(y: "y", x: 1)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x, string: y = "y");
		"#,
		r#"foo(1)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x, string: y = "y");
		"#,
		r#"foo(1, y: "z")"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x = 1, string: y = "y");
		"#,
		r#"foo()"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x = 1, string: y = "y");
		"#,
		r#"foo(y: "z")"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x, string: y = "y", float: z = 1.5);
		"#,
		r#"foo(z: 2.0, x: 4, y: "q")"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x, string: y = "y", float: z = 1.5);
		"#,
		r#"foo(z: 2.0, x: 4)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function any $T: id(any $T: x);
		"#,
		r#"id(x: 1.5)"#,
		expect!("float"),
	);
	tester.check_expression_preamble(
		r#"
        function bool: foo(int: x);
        function int: foo(int: x) = 5;
		"#,
		r#"foo(1)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo(int: x) = 5;
        function bool: foo(int: x);
		"#,
		r#"foo(1)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function bool: foo($T: x);
        function int: foo($T: x) = 5;
		"#,
		r#"foo(1)"#,
		expect!("int"),
	);
	tester.check_expression_preamble(
		r#"
        function int: foo($T: x) = 5;
        function bool: foo($T: x);
		"#,
		r#"foo(1)"#,
		expect!("int"),
	);
}

#[test]
fn test_type_errors() {
	let mut tester = TypeTester::new();
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
	tester.check_error(
		r#"
		function int: foo(int: x, int: y);
		any: z = foo(x: 1, x: 2);
		"#,
		expect!("Invalid function call"),
	);
	tester.check_error(
		r#"
		function int: foo(int: x, int: y = 2);
		any: z = foo(y: 3);
		"#,
		expect!("No matching function"),
	);
	tester.check_error(
		r#"
		function int: foo(int: x, int: y = 2);
		any: z = foo(1, x: 2);
		"#,
		expect!("No matching function"),
	);
	tester.check_error(
		r#"
		function int: foo(int: x, int: y = 2);
		any: z = foo(z: 3);
		"#,
		expect!("No matching function"),
	);
}

#[test]
fn test_class_typecheck() {
	let mut tester = TypeTester::new();
	// A class, an object introduction supplying its input record, and a field read
	tester.check_error(
		r#"
		class A (
			1..10: x;
			constraint x >= 1;
		);
		new A: a = (x: 3);
		constraint a.x = 3;
		"#,
		expect!(""),
	);
	// Inherited attributes are in scope in the subclass body and readable through it
	tester.check_error(
		r#"
		class A (1..10: x);
		class B extends A (1..10: y; constraint y >= x);
		new B: b = (x: 1, y: 2);
		constraint b.x + b.y = 3;
		"#,
		expect!(""),
	);
	// A class name in a value position denotes the set of its objects
	tester.check_error(
		r#"
		class A (1..10: x);
		set of new A: as = [(x: 1), (x: 2)];
		constraint forall (a in A) (a.x >= 1);
		"#,
		expect!(""),
	);
}

#[test]
fn test_class_type_errors() {
	let mut tester = TypeTester::new();
	tester.check_error(
		r#"
		class A (1..10: x);
		new A: a = (x: 1);
		constraint a.nope = 1;
		"#,
		expect!("Invalid field access"),
	);
	// The object's definition is checked against the class's input record
	tester.check_error(
		r#"
		class A (1..10: x);
		new A: a = (x: "not an int");
		"#,
		expect!("Type mismatch"),
	);
	tester.check_error(
		r#"
		int: notAClass;
		new notAClass: a;
		"#,
		expect!("Type mismatch"),
	);
	// A computed attribute is part of storage but must not be supplied by the caller
	tester.check_error(
		r#"
		class A (1..10: x; 1..20: y = x + 1);
		new A: a = (x: 1, y: 2);
		"#,
		expect!("Type mismatch"),
	);
	// Objects may only be introduced at the top level
	tester.check_error(
		r#"
		class A (1..10: x);
		any: a = let { new A: b = (x: 1) } in b.x;
		"#,
		expect!([r#"
    Syntax Error
    Type mismatch"#]),
	);
	// Field access off a class NAME (`A.b`, `B.as`) rather than an instance is a
	// type error, not a lowering gap.
	tester.check_error(
		r#"
		class A (
		  opt B: b;
		);
		class B (
		  set of A: as;
		);
		constraint association(A.b, B.as);
		"#,
		expect!([r#"
    Type mismatch
    Type mismatch"#]),
	);
}

/// A var-reached class has its whole storage record varified, so reads of its
/// attributes are var wherever they appear — not only through a var handle.
/// Typing them par would let the body resolve calls to par overloads that no
/// longer apply once lowering hands them the var projection, which used to
/// surface as a panic in THIR instead of an error here.
#[test]
fn test_var_reached_class_attribute_reads_are_var() {
	let mut tester = TypeTester::new();
	// Bare attribute name in a class body constraint.
	tester.check_error(
		r#"
		predicate foo(int: x);
		class A (1..10: x; constraint foo(x));
		var new A: a;
		"#,
		expect!("No matching function"),
	);
	// Same read written through `this`, which is itself par.
	tester.check_error(
		r#"
		predicate foo(int: x);
		class A (1..10: x; constraint foo(this.x));
		var new A: a;
		"#,
		expect!("No matching function"),
	);
	// A computed attribute's definition is a class-body expression too.
	tester.check_error(
		r#"
		function int: idx(int: x) = x + 1;
		class A (1..10: x; int: y = idx(x));
		var new A: a;
		"#,
		expect!("No matching function"),
	);
	// An attribute inherited from a superclass, read in the subclass body.
	tester.check_error(
		r#"
		predicate foo(int: x);
		class A (1..10: x);
		class B extends A (1..10: y; constraint foo(x));
		var new B: b;
		"#,
		expect!("No matching function"),
	);
	// A par handle still projects out of the one var storage array, so the read
	// is var even though the handle is not.
	tester.check_error(
		r#"
		predicate foo(int: x);
		class A (1..10: x);
		var new A: a1;
		new A: a2 = (x: 3);
		constraint foo(a2.x);
		"#,
		expect!("No matching function"),
	);
	// A class reached only par keeps par attributes, so the same body is fine.
	tester.check_error(
		r#"
		predicate foo(int: x);
		class A (1..10: x; constraint foo(x));
		new A: a = (x: 3);
		"#,
		expect!(""),
	);
}

#[test]
fn test_needs_rhs() {
	let mut tester = TypeTester::new();
	tester.check_error(
		r#"
		tuple(int, var 1..3): x;
		"#,
		expect!("Invalid declaration"),
	);
	tester.check_error(
		r#"
		int: x :: output_only;
		"#,
		expect!("Invalid declaration"),
	);
	tester.check_error(
		r#"
		int: x = let {
			int: y;
		} in 1;
		"#,
		expect!("Invalid declaration"),
	);
	tester.check_error(
		r#"
		int: x = let {
			tuple(int, var 1..3): y;
		} in 1;
		"#,
		expect!("Invalid declaration"),
	);
}
