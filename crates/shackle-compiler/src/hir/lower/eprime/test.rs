use expect_test::expect;

use crate::hir::lower::test::check_lower_item_eprime;

#[test]
fn test_lower_integer_domain() {
	check_lower_item_eprime(
		"find i: int",
		expect!([r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("i"))
      Annotations:
"#]),
	);
	check_lower_item_eprime(
		"find i: int(1, 3..10)",
		expect!([r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(3)
        <Expression::3>: IntegerLiteral(10)
        <Expression::4>: Identifier("..")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: SetLiteral { members: [<Expression::1>] }
        <Expression::7>: Identifier("union")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::6>, <Expression::5>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::8> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("i"))
      Annotations:
"#]),
	)
}

#[test]
fn test_lower_boolean_domain() {
	check_lower_item_eprime(
		r#"
          find x: bool
      "#,
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Bool }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
"#]],
	);
}

#[test]
fn test_domain_expressions() {
	check_lower_item_eprime(
		"letting x be domain int(1) intersect int(1)",
		expect![[r#"
    Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::1>, annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: SetLiteral { members: [<Expression::1>] }
        <Expression::3>: IntegerLiteral(1)
        <Expression::4>: SetLiteral { members: [<Expression::3>] }
        <Expression::5>: Identifier("intersect")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::2>, <Expression::4>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::6> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_matrix_domain() {
	check_lower_item_eprime(
		"given simple: matrix indexed by [int(1..4)] of bool",
		expect![[r#"
    Item: Declaration { declared_type: <Type::3>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(4)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::4> }
        <Type::2>: Primitive { inst: Par, opt: NonOpt, primitive_type: Bool }
        <Type::3>: Array { opt: NonOpt, dimensions: <Type::1>, element: <Type::2> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("simple"))
      Annotations:
"#]],
	);
	check_lower_item_eprime(
		"letting x be domain matrix indexed by [ int(1..10), int(1..10) ] of int(1..5)",
		expect![[r#"
    Item: TypeAlias { name: <Pattern::1>, aliased_type: <Type::5>, annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(10)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: IntegerLiteral(1)
        <Expression::6>: IntegerLiteral(10)
        <Expression::7>: Identifier("..")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::5>, <Expression::6>] }
        <Expression::9>: IntegerLiteral(1)
        <Expression::10>: IntegerLiteral(5)
        <Expression::11>: Identifier("..")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::9>, <Expression::10>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::4> }
        <Type::2>: Bounded { inst: Some(Par), opt: None, domain: <Expression::8> }
        <Type::3>: Tuple { opt: NonOpt, fields: [<Type::1>, <Type::2>] }
        <Type::4>: Bounded { inst: Some(Par), opt: None, domain: <Expression::12> }
        <Type::5>: Array { opt: NonOpt, dimensions: <Type::3>, element: <Type::4> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_call() {
	check_lower_item_eprime(
		"letting simple = toVec(X,Y)",
		expect![[r#"
      Item: Assignment { assignee: <Expression::1>, definition: <Expression::5> }
        Expressions:
          <Expression::1>: Identifier("simple")
          <Expression::2>: Identifier("toVec")
          <Expression::3>: Identifier("X")
          <Expression::4>: Identifier("Y")
          <Expression::5>: Call { function: <Expression::2>, arguments: [<Expression::3>, <Expression::4>] }
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
		expect![[r#"
      Item: Assignment { assignee: <Expression::1>, definition: <Expression::4> }
        Expressions:
          <Expression::1>: Identifier("negated_bool")
          <Expression::2>: BooleanLiteral(true)
          <Expression::3>: Identifier("not")
          <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::2>] }
        Types:
        Patterns:
        Annotations:
      "#]],
	);
}

#[test]
fn test_lower_quantification() {
	check_lower_item_eprime(
		"letting simple_sum = sum i : int(1..2) . i",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::9> }
      Expressions:
        <Expression::1>: Identifier("simple_sum")
        <Expression::2>: IntegerLiteral(1)
        <Expression::3>: IntegerLiteral(2)
        <Expression::4>: Identifier("..")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: Identifier("i")
        <Expression::7>: ArrayComprehension { template: <Expression::6>, indices: None, generators: [Iterator { patterns: [<Pattern::1>], collection: <Expression::5>, where_clause: None }] }
        <Expression::8>: Identifier("sum")
        <Expression::9>: Call { function: <Expression::8>, arguments: [<Expression::7>] }
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("i"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_matrix_comprehension() {
	check_lower_item_eprime(
		"letting simple = [ num**2 | num : int(1..5) ]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::10> }
      Expressions:
        <Expression::1>: Identifier("simple")
        <Expression::2>: Identifier("num")
        <Expression::3>: IntegerLiteral(2)
        <Expression::4>: Identifier("**")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: IntegerLiteral(1)
        <Expression::7>: IntegerLiteral(5)
        <Expression::8>: Identifier("..")
        <Expression::9>: Call { function: <Expression::8>, arguments: [<Expression::6>, <Expression::7>] }
        <Expression::10>: ArrayComprehension { template: <Expression::5>, indices: None, generators: [Iterator { patterns: [<Pattern::1>], collection: <Expression::9>, where_clause: None }] }
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("num"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting where = [ i+j | i: int(1..3), j : int(1..3), i<j]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::18> }
      Expressions:
        <Expression::1>: Identifier("where")
        <Expression::2>: Identifier("i")
        <Expression::3>: Identifier("j")
        <Expression::4>: Identifier("+")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: Identifier("i")
        <Expression::7>: Identifier("j")
        <Expression::8>: Identifier("<")
        <Expression::9>: Call { function: <Expression::8>, arguments: [<Expression::6>, <Expression::7>] }
        <Expression::10>: IntegerLiteral(1)
        <Expression::11>: IntegerLiteral(3)
        <Expression::12>: Identifier("..")
        <Expression::13>: Call { function: <Expression::12>, arguments: [<Expression::10>, <Expression::11>] }
        <Expression::14>: IntegerLiteral(1)
        <Expression::15>: IntegerLiteral(3)
        <Expression::16>: Identifier("..")
        <Expression::17>: Call { function: <Expression::16>, arguments: [<Expression::14>, <Expression::15>] }
        <Expression::18>: ArrayComprehension { template: <Expression::5>, indices: None, generators: [Iterator { patterns: [<Pattern::1>], collection: <Expression::13>, where_clause: Some(<Expression::9>) }, Iterator { patterns: [<Pattern::2>], collection: <Expression::17>, where_clause: None }] }
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("i"))
        <Pattern::2>: Identifier(Identifier("j"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting indexed = [ i | i : int(1..5) ; int(1..2) ]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::11> }
      Expressions:
        <Expression::1>: Identifier("indexed")
        <Expression::2>: Identifier("i")
        <Expression::3>: IntegerLiteral(1)
        <Expression::4>: IntegerLiteral(2)
        <Expression::5>: Identifier("..")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::3>, <Expression::4>] }
        <Expression::7>: IntegerLiteral(1)
        <Expression::8>: IntegerLiteral(5)
        <Expression::9>: Identifier("..")
        <Expression::10>: Call { function: <Expression::9>, arguments: [<Expression::7>, <Expression::8>] }
        <Expression::11>: ArrayComprehension { template: <Expression::2>, indices: Some(<Expression::6>), generators: [Iterator { patterns: [<Pattern::1>], collection: <Expression::10>, where_clause: None }] }
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("i"))
      Annotations:
    "#]],
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
	// This will output the last param declaration
	check_lower_item_eprime(
		r#"
      given y, x: int
    "#,
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
"#]],
	);
	// This test results in a constraint output due to the where clause
	check_lower_item_eprime(
		r#"
      given y: int
        where y < x
    "#,
		expect![[r#"
    Item: Constraint { expression: <Expression::4>, annotations: [] }
      Expressions:
        <Expression::1>: Identifier("y")
        <Expression::2>: Identifier("x")
        <Expression::3>: Identifier("<")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
      Patterns:
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_domain_alias() {
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
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: IntegerLiteral(3)
        <Expression::4>: Identifier("..")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: SetLiteral { members: [<Expression::1>] }
        <Expression::7>: Identifier("union")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::6>, <Expression::5>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::8> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_decision_declaration() {
	check_lower_item_eprime(
		"find x : int",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: None, annotations: [] }
      Expressions:
      Types:
        <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:
"#]],
	);
}

#[test]
fn test_lower_objective() {
	// Will output a satisfy goal if none specified
	check_lower_item_eprime(
		"",
		expect![[r#"
    Item: Solve { goal: Satisfy, annotations: [] }
      Expressions:
      Types:
      Patterns:
      Annotations:
    "#]],
	);
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
		"letting cmatrix: matrix indexed by [ int(1..2), int(1..2) ] of int = [ [2,8], [3,7] ]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::8> }
      Expressions:
        <Expression::1>: Identifier("cmatrix")
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: IntegerLiteral(8)
        <Expression::4>: ArrayLiteral { members: [<Expression::2>, <Expression::3>] }
        <Expression::5>: IntegerLiteral(3)
        <Expression::6>: IntegerLiteral(7)
        <Expression::7>: ArrayLiteral { members: [<Expression::5>, <Expression::6>] }
        <Expression::8>: ArrayLiteral { members: [<Expression::4>, <Expression::7>] }
      Types:
      Patterns:
      Annotations:
    "#]],
	)
}
