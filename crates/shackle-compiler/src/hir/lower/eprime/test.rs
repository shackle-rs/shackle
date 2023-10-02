use expect_test::expect;

use crate::hir::lower::test::check_lower_item_eprime;
// \[(r#"((\n|.)*?)"#|\n)*\]

#[test]
fn test_lower_integer_domain() {
	check_lower_item_eprime("given i: int(1, 3..10)", expect!([r#""#]));
}

#[test]
fn test_lower_boolean_domain() {
	check_lower_item_eprime(
		r#"
            given x, y: bool
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_lower_matrix_domain() {
	check_lower_item_eprime(
		"given simple: matrix indexed by [int(1..4)] of bool",
		expect![[r#""#]],
	)
}

#[test]
fn test_lower_call() {
	check_lower_item_eprime(
		"letting simple = toVec(X,Y)",
		expect![[r#"
        Item: Assignment { assignee: <Expression::1>, definition: <Expression::5> }
          Expressions:
            <Expression::1>: Identifier("simple")
            <Expression::2>: Identifier("X")
            <Expression::3>: Identifier("Y")
            <Expression::4>: Identifier("toVec")
            <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
          Types:
          Patterns:
          Annotations:
        "#]],
	);
}

#[test]
fn test_lower_indexed_access() {
	check_lower_item_eprime(
		r#"
        letting multi = M[2..4]
        "#,
		expect![[r#"
        Item: Assignment { assignee: <Expression::1>, definition: <Expression::7> }
          Expressions:
            <Expression::1>: Identifier("multi")
            <Expression::2>: IntegerLiteral(2)
            <Expression::3>: IntegerLiteral(4)
            <Expression::4>: Identifier("..")
            <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
            <Expression::6>: Identifier("M")
            <Expression::7>: ArrayAccess { collection: <Expression::6>, indices: <Expression::5> }
          Types:
          Patterns:
          Annotations:
        "#]],
	);
}

#[test]
fn test_lower_infix_operator() {
	check_lower_item_eprime(
		r#"
        letting and = x /\ y
        "#,
		expect![[r#"
        Item: Assignment { assignee: <Expression::1>, definition: <Expression::5> }
          Expressions:
            <Expression::1>: Identifier("and")
            <Expression::2>: Identifier("x")
            <Expression::3>: Identifier("y")
            <Expression::4>: Identifier("/\\")
            <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
          Types:
          Patterns:
          Annotations:
        "#]],
	);
}

#[test]
fn test_lower_prefix_operator() {
	check_lower_item_eprime(
		r#"
        letting negated_bool = !true
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_lower_quantification() {
	check_lower_item_eprime(
		"letting expr = exists i,j : int(1..3) . x[i] = i",
		expect![[r#""#]],
	);
}

#[test]
fn test_lower_matrix_comprehension() {
	check_lower_item_eprime(
		"letting indexed = [ i+j | i: int(1..3), j : int(1..3), i<j ; int(7..) ]",
		expect![[r#""#]],
	);
}

#[test]
fn test_lower_absolute() {
	check_lower_item_eprime(
		"letting absolute = | x |",
		expect![[r#"
        Item: Assignment { assignee: <Expression::1>, definition: <Expression::4> }
          Expressions:
            <Expression::1>: Identifier("absolute")
            <Expression::2>: Identifier("x")
            <Expression::3>: Identifier("abs")
            <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::2>] }
          Types:
          Patterns:
          Annotations:
        "#]],
	);
}

#[test]
fn test_lower_const_definition() {
	check_lower_item_eprime(
		"letting one = 1",
		expect![[r#"
        Item: Assignment { assignee: <Expression::1>, definition: <Expression::2> }
          Expressions:
            <Expression::1>: Identifier("one")
            <Expression::2>: IntegerLiteral(1)
          Types:
          Patterns:
          Annotations:
        "#]],
	)
}

#[test]
fn test_lower_param_declaration() {
	check_lower_item_eprime(
		r#"
            given x: int(1..10)
            given y: int(1..10)
                where y < x
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_lower_domain_alias() {
	check_lower_item_eprime(
		"letting x be domain bool",
		expect![[r#"
        Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::1>, annotations: [] }
          Expressions:
          Types:
            <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Bool }
          Patterns:
            <Pattern::1>: Identifier(Identifier("x"))
          Annotations:
        "#]],
	);
	check_lower_item_eprime(
		"letting x be domain int",
		expect![[r#"
        Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::1>, annotations: [] }
          Expressions:
          Types:
            <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
          Patterns:
            <Pattern::1>: Identifier(Identifier("x"))
          Annotations:
        "#]],
	);
	check_lower_item_eprime(
		"letting x be domain int(1, 2..3)",
		expect![[r#"
      Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::1>, annotations: [] }
        Expressions:
          <Expression::1>: Identifier("union")
          <Expression::2>: IntegerLiteral(1)
          <Expression::3>: IntegerLiteral(2)
          <Expression::4>: IntegerLiteral(3)
          <Expression::5>: Identifier("..")
          <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::3>, <Expression::4>] }
          <Expression::7>: ArrayLiteral { members: [<Expression::2>] }
          <Expression::8>: Call { function: <Expression::1>, arguments: [<Expression::7>, <Expression::6>] }
        Types:
          <Type::1>: Bounded { inst: Some(Par), opt: Some(NonOpt), domain: <Expression::8> }
        Patterns:
          <Pattern::1>: Identifier(Identifier("x"))
        Annotations:
      "#]],
	);
}

#[test]
fn test_lower_decision_declaration() {
	check_lower_item_eprime("find x : int(1..10)", expect![[r#""#]]);
}

#[test]
fn test_lower_objective() {
	check_lower_item_eprime(
		"minimising x",
		expect![[r#"
        Item: Solve { goal: Minimize { pattern: <Pattern::1>, objective: <Expression::1> }, annotations: [] }
          Expressions:
            <Expression::1>: Identifier("x")
          Types:
          Patterns:
            <Pattern::1>: Identifier(Identifier("_objective"))
          Annotations:
        "#]],
	);
}

// Possibly remove this test
#[test]
fn test_lower_heuristic() {
	check_lower_item_eprime("heuristic static", expect![[r#""#]])
}

// Possibly remove this test
#[test]
fn test_lower_branching() {
	check_lower_item_eprime(
		r#"
        minimising x
        branching on [x]
        "#,
		expect![[r#"
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
        "#]],
	)
}

// This test is bad as it only tests the last constraint thus multiple constraints
// such as `such that x, y` isn't tested
#[test]
fn test_lower_constraint() {
	check_lower_item_eprime(
		"such that x",
		expect![[r#"
        Item: Constraint { expression: <Expression::1>, annotations: [] }
          Expressions:
            <Expression::1>: Identifier("x")
          Types:
          Patterns:
          Annotations:
        "#]],
	)
}

#[test]
fn test_lower_matrix_literal() {
	check_lower_item_eprime(
        "letting cmatrix: matrix indexed by [ int(1..2), int(1..4) ] of int(1..10) = [ [2,8,5,1], [3,7,9,4] ]",
        expect![[r#""#]]
    )
}
