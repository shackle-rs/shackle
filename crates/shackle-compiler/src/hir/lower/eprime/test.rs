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
        <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Int }
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
        <Type::1>: Bounded { inst: Some(Var), opt: None, domain: <Expression::8> }
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
          <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Bool }
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
    Item: Declaration { declared_type: <Type::2>, pattern: <Pattern::1>, definition: Some(<Expression::12>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: SetLiteral { members: [<Expression::1>] }
        <Expression::3>: IntegerLiteral(1)
        <Expression::4>: SetLiteral { members: [<Expression::3>] }
        <Expression::5>: Identifier("intersect")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::2>, <Expression::4>] }
        <Expression::7>: IntegerLiteral(1)
        <Expression::8>: SetLiteral { members: [<Expression::7>] }
        <Expression::9>: IntegerLiteral(1)
        <Expression::10>: SetLiteral { members: [<Expression::9>] }
        <Expression::11>: Identifier("intersect")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::8>, <Expression::10>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::6> }
        <Type::2>: Set { inst: Par, opt: NonOpt, element: <Type::1> }
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
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::12> }
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
        <Expression::11>: Identifier("indexing_0")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::10>] }
      Types:
      Patterns:
        <Pattern::1>: Identifier(Identifier("num"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting where = [ i+j | i: int(1..3), j : int(1..3), i<j]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::20> }
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
        <Expression::19>: Identifier("indexing_0")
        <Expression::20>: Call { function: <Expression::19>, arguments: [<Expression::18>] }
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
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::13> }
      Expressions:
        <Expression::1>: Identifier("indexed")
        <Expression::2>: Identifier("i")
        <Expression::3>: IntegerLiteral(1)
        <Expression::4>: IntegerLiteral(5)
        <Expression::5>: Identifier("..")
        <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::3>, <Expression::4>] }
        <Expression::7>: ArrayComprehension { template: <Expression::2>, indices: None, generators: [Iterator { patterns: [<Pattern::1>], collection: <Expression::6>, where_clause: None }] }
        <Expression::8>: IntegerLiteral(1)
        <Expression::9>: IntegerLiteral(2)
        <Expression::10>: Identifier("..")
        <Expression::11>: Call { function: <Expression::10>, arguments: [<Expression::8>, <Expression::9>] }
        <Expression::12>: Identifier("array1d")
        <Expression::13>: Call { function: <Expression::12>, arguments: [<Expression::11>, <Expression::7>] }
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
      Item: Declaration { declared_type: <Type::2>, pattern: <Pattern::1>, definition: Some(<Expression::6>), annotations: [] }
        Expressions:
          <Expression::1>: Infinity
          <Expression::2>: Identifier("-")
          <Expression::3>: Call { function: <Expression::2>, arguments: [<Expression::1>] }
          <Expression::4>: Infinity
          <Expression::5>: Identifier("..")
          <Expression::6>: Call { function: <Expression::5>, arguments: [<Expression::3>, <Expression::4>] }
        Types:
          <Type::1>: Primitive { inst: Par, opt: NonOpt, primitive_type: Int }
          <Type::2>: Set { inst: Par, opt: NonOpt, element: <Type::1> }
        Patterns:
          <Pattern::1>: Identifier(Identifier("x"))
        Annotations:
      "#]],
	);
	check_lower_item_eprime(
		"letting x be domain int(1, 2..3)",
		expect![[r#"
    Item: Declaration { declared_type: <Type::2>, pattern: <Pattern::1>, definition: Some(<Expression::16>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: IntegerLiteral(3)
        <Expression::4>: Identifier("..")
        <Expression::5>: Call { function: <Expression::4>, arguments: [<Expression::2>, <Expression::3>] }
        <Expression::6>: SetLiteral { members: [<Expression::1>] }
        <Expression::7>: Identifier("union")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::6>, <Expression::5>] }
        <Expression::9>: IntegerLiteral(1)
        <Expression::10>: IntegerLiteral(2)
        <Expression::11>: IntegerLiteral(3)
        <Expression::12>: Identifier("..")
        <Expression::13>: Call { function: <Expression::12>, arguments: [<Expression::10>, <Expression::11>] }
        <Expression::14>: SetLiteral { members: [<Expression::9>] }
        <Expression::15>: Identifier("union")
        <Expression::16>: Call { function: <Expression::15>, arguments: [<Expression::14>, <Expression::13>] }
      Types:
        <Type::1>: Bounded { inst: Some(Par), opt: None, domain: <Expression::8> }
        <Type::2>: Set { inst: Par, opt: NonOpt, element: <Type::1> }
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
          <Type::1>: Primitive { inst: Var, opt: NonOpt, primitive_type: Int }
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
		"letting matrix1d = [3,4]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::4> }
      Expressions:
        <Expression::1>: Identifier("matrix1d")
        <Expression::2>: IntegerLiteral(3)
        <Expression::3>: IntegerLiteral(4)
        <Expression::4>: ArrayLiteral { members: [<Expression::2>, <Expression::3>] }
      Types:
      Patterns:
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting matrix2d = [ [2,8], [3,7] ]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::6> }
      Expressions:
        <Expression::1>: Identifier("matrix2d")
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: IntegerLiteral(8)
        <Expression::4>: IntegerLiteral(3)
        <Expression::5>: IntegerLiteral(7)
        <Expression::6>: ArrayLiteral2D { rows: NonIndexed(2), columns: NonIndexed(2), members: [<Expression::2>, <Expression::3>, <Expression::4>, <Expression::5>] }
      Types:
      Patterns:
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting matrix3d = [ [[1,2,3],[4,5,6]], [[7,8,9],[10,11,12]] ]",
		expect![[r#"
    Item: Assignment { assignee: <Expression::1>, definition: <Expression::28> }
      Expressions:
        <Expression::1>: Identifier("matrix3d")
        <Expression::2>: IntegerLiteral(1)
        <Expression::3>: IntegerLiteral(2)
        <Expression::4>: IntegerLiteral(3)
        <Expression::5>: IntegerLiteral(4)
        <Expression::6>: IntegerLiteral(5)
        <Expression::7>: IntegerLiteral(6)
        <Expression::8>: IntegerLiteral(7)
        <Expression::9>: IntegerLiteral(8)
        <Expression::10>: IntegerLiteral(9)
        <Expression::11>: IntegerLiteral(10)
        <Expression::12>: IntegerLiteral(11)
        <Expression::13>: IntegerLiteral(12)
        <Expression::14>: IntegerLiteral(1)
        <Expression::15>: IntegerLiteral(2)
        <Expression::16>: Identifier("..")
        <Expression::17>: Call { function: <Expression::16>, arguments: [<Expression::14>, <Expression::15>] }
        <Expression::18>: IntegerLiteral(1)
        <Expression::19>: IntegerLiteral(2)
        <Expression::20>: Identifier("..")
        <Expression::21>: Call { function: <Expression::20>, arguments: [<Expression::18>, <Expression::19>] }
        <Expression::22>: IntegerLiteral(1)
        <Expression::23>: IntegerLiteral(3)
        <Expression::24>: Identifier("..")
        <Expression::25>: Call { function: <Expression::24>, arguments: [<Expression::22>, <Expression::23>] }
        <Expression::26>: ArrayLiteral { members: [<Expression::2>, <Expression::3>, <Expression::4>, <Expression::5>, <Expression::6>, <Expression::7>, <Expression::8>, <Expression::9>, <Expression::10>, <Expression::11>, <Expression::12>, <Expression::13>] }
        <Expression::27>: Identifier("array3d")
        <Expression::28>: Call { function: <Expression::27>, arguments: [<Expression::17>, <Expression::21>, <Expression::25>, <Expression::26>] }
      Types:
      Patterns:
      Annotations:
    "#]],
	);
}

#[test]
fn test_output() {
	check_lower_item_eprime(
		"output[show(x)]",
		expect![[r#"
    Item: Output { section: None, expression: <Expression::4> }
      Expressions:
        <Expression::1>: Identifier("show")
        <Expression::2>: Identifier("x")
        <Expression::3>: Call { function: <Expression::1>, arguments: [<Expression::2>] }
        <Expression::4>: ArrayLiteral { members: [<Expression::3>] }
      Types:
      Patterns:
      Annotations:
"#]],
	)
}
