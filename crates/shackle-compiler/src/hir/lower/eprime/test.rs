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
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::4>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("X")
        <Expression::2>: Identifier("Y")
        <Expression::3>: Identifier("toVec")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("simple"))
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
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::6>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(2)
        <Expression::2>: IntegerLiteral(4)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Identifier("M")
        <Expression::6>: ArrayAccess { collection: <Expression::5>, indices: <Expression::4> }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("multi"))
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
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::4>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: Identifier("y")
        <Expression::3>: Identifier("/\\")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("and"))
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
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::3>), annotations: [] }
      Expressions:
        <Expression::1>: BooleanLiteral(true)
        <Expression::2>: Identifier("not")
        <Expression::3>: Call { function: <Expression::2>, arguments: [<Expression::1>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("negated_bool"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_quantification() {
	check_lower_item_eprime(
		"letting simple_sum = sum i : int(1..2) . i",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::8>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Identifier("i")
        <Expression::6>: ArrayComprehension { template: <Expression::5>, indices: None, generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::4>, where_clause: None }] }
        <Expression::7>: Identifier("sum")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::6>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("simple_sum"))
        <Pattern::2>: Identifier(Identifier("i"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_matrix_comprehension() {
	check_lower_item_eprime(
		"letting simple = [ num**2 | num : int(1..5) ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::9>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(5)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Identifier("num")
        <Expression::6>: IntegerLiteral(2)
        <Expression::7>: Identifier("**")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::5>, <Expression::6>] }
        <Expression::9>: ArrayComprehension { template: <Expression::8>, indices: None, generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::4>, where_clause: None }] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("simple"))
        <Pattern::2>: Identifier(Identifier("num"))
      Annotations:
    "#]],
	);
  check_lower_item_eprime(
		"letting multi = [ [i, i+1] | i : int(1..2) ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::11>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Identifier("i")
        <Expression::6>: Identifier("i")
        <Expression::7>: IntegerLiteral(1)
        <Expression::8>: Identifier("+")
        <Expression::9>: Call { function: <Expression::8>, arguments: [<Expression::6>, <Expression::7>] }
        <Expression::10>: TupleLiteral { fields: [<Expression::5>, <Expression::9>] }
        <Expression::11>: ArrayComprehension { template: <Expression::10>, indices: None, generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::4>, where_clause: None }] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("multi"))
        <Pattern::2>: Identifier(Identifier("i"))
      Annotations:
    "#]],
	);
  check_lower_item_eprime(
		"letting multi = [ [i+j | j : int(1..2)] | i : int(1..2) ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::16>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: IntegerLiteral(1)
        <Expression::6>: IntegerLiteral(2)
        <Expression::7>: Identifier("..")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::5>, <Expression::6>] }
        <Expression::9>: Identifier("i")
        <Expression::10>: Identifier("j")
        <Expression::11>: Identifier("+")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::9>, <Expression::10>] }
        <Expression::13>: Identifier("i")
        <Expression::14>: Identifier("j")
        <Expression::15>: TupleLiteral { fields: [<Expression::13>, <Expression::14>] }
        <Expression::16>: ArrayComprehension { template: <Expression::12>, indices: Some(<Expression::15>), generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::4>, where_clause: None }, Iterator { patterns: [<Pattern::3>], collection: <Expression::8>, where_clause: None }] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("multi"))
        <Pattern::2>: Identifier(Identifier("i"))
        <Pattern::3>: Identifier(Identifier("j"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting where = [ i+j | i: int(1..3), j : int(1..3), i<j]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::17>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("i")
        <Expression::2>: Identifier("j")
        <Expression::3>: Identifier("<")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: IntegerLiteral(1)
        <Expression::6>: IntegerLiteral(3)
        <Expression::7>: Identifier("..")
        <Expression::8>: Call { function: <Expression::7>, arguments: [<Expression::5>, <Expression::6>] }
        <Expression::9>: IntegerLiteral(1)
        <Expression::10>: IntegerLiteral(3)
        <Expression::11>: Identifier("..")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::9>, <Expression::10>] }
        <Expression::13>: Identifier("i")
        <Expression::14>: Identifier("j")
        <Expression::15>: Identifier("+")
        <Expression::16>: Call { function: <Expression::15>, arguments: [<Expression::13>, <Expression::14>] }
        <Expression::17>: ArrayComprehension { template: <Expression::16>, indices: None, generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::8>, where_clause: Some(<Expression::4>) }, Iterator { patterns: [<Pattern::3>], collection: <Expression::12>, where_clause: None }] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("where"))
        <Pattern::2>: Identifier(Identifier("i"))
        <Pattern::3>: Identifier(Identifier("j"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting indexed = [ i | i : int(1..5) ; int(1..2) ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::12>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(5)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Identifier("i")
        <Expression::6>: ArrayComprehension { template: <Expression::5>, indices: None, generators: [Iterator { patterns: [<Pattern::2>], collection: <Expression::4>, where_clause: None }] }
        <Expression::7>: IntegerLiteral(1)
        <Expression::8>: IntegerLiteral(2)
        <Expression::9>: Identifier("..")
        <Expression::10>: Call { function: <Expression::9>, arguments: [<Expression::7>, <Expression::8>] }
        <Expression::11>: Identifier("array1d")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::10>, <Expression::6>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("indexed"))
        <Pattern::2>: Identifier(Identifier("i"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_absolute() {
	check_lower_item_eprime(
		"letting absolute = | x |",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::3>), annotations: [] }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: Identifier("abs")
        <Expression::3>: Call { function: <Expression::2>, arguments: [<Expression::1>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("absolute"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_const_definition() {
	check_lower_item_eprime(
		"letting one = 1",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::1>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("one"))
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
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::3>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(3)
        <Expression::2>: IntegerLiteral(4)
        <Expression::3>: ArrayLiteral { members: [<Expression::1>, <Expression::2>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("matrix1d"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting matrix2d = [ [2,8], [3,7] ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::5>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(2)
        <Expression::2>: IntegerLiteral(8)
        <Expression::3>: IntegerLiteral(3)
        <Expression::4>: IntegerLiteral(7)
        <Expression::5>: ArrayLiteral2D { rows: NonIndexed(2), columns: NonIndexed(2), members: [<Expression::1>, <Expression::2>, <Expression::3>, <Expression::4>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("matrix2d"))
      Annotations:
    "#]],
	);
	check_lower_item_eprime(
		"letting matrix3d = [ [[1,2],[3,4]], [[5,6],[7,8]] ]",
		expect![[r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::23>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(2)
        <Expression::3>: IntegerLiteral(3)
        <Expression::4>: IntegerLiteral(4)
        <Expression::5>: IntegerLiteral(5)
        <Expression::6>: IntegerLiteral(6)
        <Expression::7>: IntegerLiteral(7)
        <Expression::8>: IntegerLiteral(8)
        <Expression::9>: IntegerLiteral(1)
        <Expression::10>: IntegerLiteral(2)
        <Expression::11>: Identifier("..")
        <Expression::12>: Call { function: <Expression::11>, arguments: [<Expression::9>, <Expression::10>] }
        <Expression::13>: IntegerLiteral(1)
        <Expression::14>: IntegerLiteral(2)
        <Expression::15>: Identifier("..")
        <Expression::16>: Call { function: <Expression::15>, arguments: [<Expression::13>, <Expression::14>] }
        <Expression::17>: IntegerLiteral(1)
        <Expression::18>: IntegerLiteral(2)
        <Expression::19>: Identifier("..")
        <Expression::20>: Call { function: <Expression::19>, arguments: [<Expression::17>, <Expression::18>] }
        <Expression::21>: ArrayLiteral { members: [<Expression::1>, <Expression::2>, <Expression::3>, <Expression::4>, <Expression::5>, <Expression::6>, <Expression::7>, <Expression::8>] }
        <Expression::22>: Identifier("array3d")
        <Expression::23>: Call { function: <Expression::22>, arguments: [<Expression::12>, <Expression::16>, <Expression::20>, <Expression::21>] }
      Types:
        <Type::1>: Any
      Patterns:
        <Pattern::1>: Identifier(Identifier("matrix3d"))
      Annotations:
    "#]],
	);
}

#[test]
fn test_lower_output() {
	check_lower_item_eprime(
		"output[show(x)]",
		expect![[r#"
    Item: Output { section: None, expression: <Expression::4> }
      Expressions:
        <Expression::1>: Identifier("x")
        <Expression::2>: Identifier("show")
        <Expression::3>: Call { function: <Expression::2>, arguments: [<Expression::1>] }
        <Expression::4>: ArrayLiteral { members: [<Expression::3>] }
      Types:
      Patterns:
      Annotations:
"#]],
	)
}
