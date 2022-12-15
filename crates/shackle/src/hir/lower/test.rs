use std::sync::Arc;

use expect_test::{expect, Expect};

use crate::{
	db::{CompilerDatabase, FileReader, Inputs},
	file::InputFile,
	hir::db::Hir,
	utils::DebugPrint,
};

fn check_lower_item(item: &str, expected: Expect) {
	let mut db = CompilerDatabase::default();
	db.set_ignore_stdlib(true);
	db.set_input_files(Arc::new(vec![InputFile::ModelString(item.to_owned())]));
	let model = db.input_models();
	let items = db.lookup_items(model[0]);
	let item = *items.last().unwrap();
	let debug_print = item.debug_print(&db);
	expected.assert_eq(&debug_print);
}

#[test]
fn test_lower_assignment() {
	check_lower_item(
		"x = 1;",
		expect!([r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::2> }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: IntegerLiteral(1)
      Types:
      Patterns:
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_constraint() {
	check_lower_item(
		"constraint x = 1;",
		expect!([r#"
    Item: Constraint { expression: <Expression::4>, annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: IntegerLiteral(1)
        <Expression::3>: Identifier("=")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
      Patterns:
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_declaration() {
	check_lower_item(
		"var int: x;",
		expect!([r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
"#]),
	);
	check_lower_item(
		"tuple(int, int): (x, y) = (1, 2);",
		expect!([r#"
    Item: Declaration { declared_type: <Type::3>, pattern: <Pattern::3>, definition: Some(<Expression::3>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: TupleLiteral { fields: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::3>: Tuple { opt: NonOpt, fields: [<Type::1>, <Type::2>] }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
        <Pattern::2>: Identifier(Identifier("y"))
        <Pattern::3>: Tuple { fields: [<Pattern::1>, <Pattern::2>] }
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_annotation() {
	check_lower_item(
		"annotation foo;",
		expect!([r#"
    Item: Annotation { constructor: Atom { pattern: <Pattern::1> } }
      Expressions:
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
      Annotations:
"#]),
	);
	check_lower_item(
		"annotation foo(int, float);",
		expect!([r#"
    Item: Annotation { constructor: Function { constructor: <Pattern::1>, destructor: <Pattern::2>, parameters: [ConstructorParameter { declared_type: <Type::1>, pattern: None }, ConstructorParameter { declared_type: <Type::2>, pattern: None }] } }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Float }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
        <Pattern::2>: Identifier(Identifier("foo⁻¹"))
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_enumeration() {
	check_lower_item(
		"enum Foo;",
		expect!([r#"
    Item: Enumeration { pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("Foo"))
      Annotations:
"#]),
	);
	check_lower_item(
		"enum Foo = {A, B, C};",
		expect!([r#"
    Item: Enumeration { pattern: <Pattern::1>, definition: Some([Named(Atom { pattern: <Pattern::2> }), Named(Atom { pattern: <Pattern::3> }), Named(Atom { pattern: <Pattern::4> })]), annotations: [] }
      Expressions:
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("Foo"))
        <Pattern::2>: Identifier(Identifier("A"))
        <Pattern::3>: Identifier(Identifier("B"))
        <Pattern::4>: Identifier(Identifier("C"))
      Annotations:
"#]),
	);
	check_lower_item(
		"enum Foo = A(B) ++ {C};",
		expect!([r#"
    Item: Enumeration { pattern: <Pattern::1>, definition: Some([Named(Function { constructor: <Pattern::2>, destructor: <Pattern::3>, parameters: [ConstructorParameter { declared_type: <Type::1>, pattern: None }] }), Named(Atom { pattern: <Pattern::4> })]), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("B")
      Types:
        <Type::1>: Bounded { inst: None, opt: None, domain: <Expression::1> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("Foo"))
        <Pattern::2>: Identifier(Identifier("A"))
        <Pattern::3>: Identifier(Identifier("A⁻¹"))
        <Pattern::4>: Identifier(Identifier("C"))
      Annotations:
"#]),
	);
	check_lower_item(
		r#"
            enum Foo;
            Foo = {A, B, C};
        "#,
		expect!([r#"
    Item: EnumAssignment { assignee: <Expression::1>, definition: [Named(Atom { pattern: <Pattern::3> }), Named(Atom { pattern: <Pattern::2> }), Named(Atom { pattern: <Pattern::1> })] }
      Expressions:
        <Expression::1>: Identifier("Foo")
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("C"))
        <Pattern::2>: Identifier(Identifier("B"))
        <Pattern::3>: Identifier(Identifier("A"))
      Annotations:
"#]),
	);
	check_lower_item(
		r#"
            enum Foo;
            Foo = A(B) ++ {C};
        "#,
		expect!([r#"
    Item: EnumAssignment { assignee: <Expression::1>, definition: [Named(Function { constructor: <Pattern::2>, destructor: <Pattern::3>, parameters: [ConstructorParameter { declared_type: <Type::1>, pattern: None }] }), Named(Atom { pattern: <Pattern::1> })] }
      Expressions:
        <Expression::1>: Identifier("Foo")
        <Expression::2>: Identifier("B")
      Types:
        <Type::1>: Bounded { inst: None, opt: None, domain: <Expression::2> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("C"))
        <Pattern::2>: Identifier(Identifier("A"))
        <Pattern::3>: Identifier(Identifier("A⁻¹"))
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_function() {
	check_lower_item(
		"function var int: foo(int: x, var bool: y) = if y then x else 0 endif;",
		expect!([r#"
    Item: Function { return_type: <Type::1>, pattern: <Pattern::1>, type_inst_vars: [], parameters: [Parameter { declared_type: <Type::2>, pattern: Some(<Pattern::2>), annotations: [] }, Parameter { declared_type: <Type::3>, pattern: Some(<Pattern::3>), annotations: [] }], body: Some(<Expression::4>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("y")
        <Expression::2>: Identifier("x")
        <Expression::3>: IntegerLiteral(0)
        <Expression::4>: IfThenElse { branches: [Branch { condition: <Expression::1>, result: <Expression::2> }], else_result: Some(<Expression::3>) }
      Types:
        <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Int }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::3>: Primitive { inst: Var, opt: NonOpt, primitive_type: Bool }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
        <Pattern::2>: Identifier(Identifier("x"))
        <Pattern::3>: Identifier(Identifier("y"))
      Annotations:
"#]),
	);
	check_lower_item(
		"function int: foo(tuple(int, int): (x, y)) = x + y;",
		expect!([r#"
    Item: Function { return_type: <Type::1>, pattern: <Pattern::1>, type_inst_vars: [], parameters: [Parameter { declared_type: <Type::4>, pattern: Some(<Pattern::4>), annotations: [] }], body: Some(<Expression::4>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: Identifier("y")
        <Expression::3>: Identifier("+")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::3>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::4>: Tuple { opt: NonOpt, fields: [<Type::2>, <Type::3>] }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
        <Pattern::2>: Identifier(Identifier("x"))
        <Pattern::3>: Identifier(Identifier("y"))
        <Pattern::4>: Tuple { fields: [<Pattern::2>, <Pattern::3>] }
      Annotations:
"#]),
	);
	check_lower_item(
		"predicate foo(int) = true;",
		expect!([r#"
    Item: Function { return_type: <Type::1>, pattern: <Pattern::1>, type_inst_vars: [], parameters: [Parameter { declared_type: <Type::2>, pattern: None, annotations: [] }], body: Some(<Expression::1>), annotations: [] }
      Expressions:
        <Expression::1>: BooleanLiteral(true)
      Types:
        <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Bool }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
      Annotations:
"#]),
	);
	check_lower_item(
		"test foo(int) = false;",
		expect!([r#"
    Item: Function { return_type: <Type::1>, pattern: <Pattern::1>, type_inst_vars: [], parameters: [Parameter { declared_type: <Type::2>, pattern: None, annotations: [] }], body: Some(<Expression::1>), annotations: [] }
      Expressions:
        <Expression::1>: BooleanLiteral(false)
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Bool }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
      Annotations:
"#]),
	);
	check_lower_item(
		"function var $$E: foo($T: x, $$E: y);",
		expect!([r#"
    Item: Function { return_type: <Type::1>, pattern: <Pattern::1>, type_inst_vars: [TypeInstIdentifierDeclaration { name: <Pattern::2>, anonymous: false, is_enum: true, is_varifiable: true, is_indexable: false }, TypeInstIdentifierDeclaration { name: <Pattern::3>, anonymous: false, is_enum: false, is_varifiable: false, is_indexable: false }], parameters: [Parameter { declared_type: <Type::2>, pattern: Some(<Pattern::4>), annotations: [] }, Parameter { declared_type: <Type::3>, pattern: Some(<Pattern::6>), annotations: [] }], body: None, annotations: [] }
      Expressions:
        <Expression::1>: Identifier("$$E")
        <Expression::2>: Identifier("$T")
        <Expression::3>: Identifier("$$E")
      Types:
        <Type::1>: Bounded { inst: Some(Var), opt: Some(NonOpt), domain: <Expression::1> }
        <Type::2>: Bounded { inst: Some(Par), opt: Some(NonOpt), domain: <Expression::2> }
        <Type::3>: Bounded { inst: Some(Par), opt: Some(NonOpt), domain: <Expression::3> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("foo"))
        <Pattern::2>: Identifier(Identifier("$$E"))
        <Pattern::3>: Identifier(Identifier("$T"))
        <Pattern::4>: Identifier(Identifier("x"))
        <Pattern::5>: Identifier(Identifier("$$E"))
        <Pattern::6>: Identifier(Identifier("y"))
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_output() {
	check_lower_item(
		r#"
        output ["foo"];
    "#,
		expect!([r#"
    Item: Output { section: None, expression: <Expression::2> }
      Expressions:
        <Expression::1>: StringLiteral("foo")
        <Expression::2>: ArrayLiteral { members: [<Expression::1>] }
      Types:
      Patterns:
      Annotations:
"#]),
	);
	check_lower_item(
		r#"
        output :: "foo" [x, y];
    "#,
		expect!([r#"
    Item: Output { section: Some(<Expression::1>), expression: <Expression::4> }
      Expressions:
        <Expression::1>: StringLiteral("foo")
        <Expression::2>: Identifier("x")
        <Expression::3>: Identifier("y")
        <Expression::4>: ArrayLiteral { members: [<Expression::2>, <Expression::3>] }
      Types:
      Patterns:
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_solve() {
	check_lower_item(
		"solve satisfy;",
		expect!([r#"
    Item: Solve { goal: Satisfy, annotations: [] }
      Expressions:
      Types:
      Patterns:
      Annotations:
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) satisfy;",
		expect!([r#"
    Item: Solve { goal: Satisfy, annotations: [<Expression::6>] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: ArrayLiteral { members: [<Expression::1>] }
        <Expression::3>: Identifier("input_order")
        <Expression::4>: Identifier("indomain_min")
        <Expression::5>: Identifier("int_search")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::2>, <Expression::3>, <Expression::4>] }
      Types:
      Patterns:
      Annotations:
"#]),
	);
	check_lower_item(
		"solve minimize x;",
		expect!([r#"
    Item: Solve { goal: Minimize { pattern: <Pattern::1>, objective: <Expression::1> }, annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("_objective"))
      Annotations:
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) minimize x;",
		expect!([r#"
    Item: Solve { goal: Minimize { pattern: <Pattern::1>, objective: <Expression::7> }, annotations: [<Expression::6>] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: ArrayLiteral { members: [<Expression::1>] }
        <Expression::3>: Identifier("input_order")
        <Expression::4>: Identifier("indomain_min")
        <Expression::5>: Identifier("int_search")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::2>, <Expression::3>, <Expression::4>] }
        <Expression::7>: Identifier("x")
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("_objective"))
      Annotations:
"#]),
	);
	check_lower_item(
		"solve maximize x;",
		expect!([r#"
    Item: Solve { goal: Maximize { pattern: <Pattern::1>, objective: <Expression::1> }, annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("_objective"))
      Annotations:
"#]),
	);
	check_lower_item(
		"solve :: int_search([x], input_order, indomain_min) maximize x;",
		expect!([r#"
    Item: Solve { goal: Maximize { pattern: <Pattern::1>, objective: <Expression::7> }, annotations: [<Expression::6>] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: ArrayLiteral { members: [<Expression::1>] }
        <Expression::3>: Identifier("input_order")
        <Expression::4>: Identifier("indomain_min")
        <Expression::5>: Identifier("int_search")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::2>, <Expression::3>, <Expression::4>] }
        <Expression::7>: Identifier("x")
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("_objective"))
      Annotations:
"#]),
	);
}

#[test]
fn test_lower_type_alias() {
	check_lower_item(
		"type Foo = set of int;",
		expect!([r#"
    Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::2>, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
        <Type::2>: Set { inst: Par, opt: NonOpt, element: <Type::1> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("Foo"))
      Annotations:
"#]),
	);
	check_lower_item(
		"type Foo = var 1..3;",
		expect!([r#"
    Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::1>, annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(3)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Bounded { inst: Some(Var), opt: None, domain: <Expression::4> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("Foo"))
      Annotations:
"#]),
	);
}
