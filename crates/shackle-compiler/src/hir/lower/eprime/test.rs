use expect_test::expect;

use crate::hir::lower::test::check_lower_item_eprime;

#[test]
fn test_integer_domain() {
	check_lower_item_eprime(
		r#"
        given a: int(1..10)
        given i: int(1,3,5..10,15..20)
        given j: int
        "#,
		expect!([r#""#]),
	);
}

#[test]
fn test_boolean_domain() {
	check_lower_item_eprime(
		r#"
            given x: bool
            given x, y: bool
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_matrix_domain() {
	check_lower_item_eprime(
		"given simple: matrix indexed by [int(1..4)] of bool",
		expect![[r#""#]],
	)
}

#[test]
fn test_call() {
	check_lower_item_eprime("letting simple = toVec(X,Y)", expect![[r#""#]]);
}

#[test]
fn test_indexed_access() {
	check_lower_item_eprime(
		r#"
        letting single = M[i]
        letting slice = Ms[..]
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_infix_operator() {
	check_lower_item_eprime(
		r#"
        letting different = x != y
        letting smallerlex = x <lex y
        letting and = x /\ y
        letting equiv = x <=> y
        letting exponent = x ** y
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_prefix_operator() {
	check_lower_item_eprime(
		r#"
        letting negative_ident = -x
        letting negated_bool = !true
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_quantification() {
	check_lower_item_eprime(
		"letting expr = exists i,j : int(1..3) . x[i] = i",
		expect![[r#""#]],
	);
}

#[test]
fn test_matrix_comprehension() {
	check_lower_item_eprime(
		"letting indexed = [ i+j | i: int(1..3), j : int(1..3), i<j ; int(7..) ]",
		expect![[r#""#]],
	);
}

#[test]
fn test_absolute() {
	check_lower_item_eprime("letting absolute = | x |", expect![[r#""#]]);
}

#[test]
fn test_const_definition() {
	check_lower_item_eprime(
		r#"
            letting x = 10
            letting x be 10
        "#,
		expect![[r#""#]],
	);
}

#[test]
fn test_param_declaration() {
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
fn test_domain_alias() {
	check_lower_item_eprime("letting INDEX be domain int(1..c*n)", expect![[r#""#]]);
}

#[test]
fn test_decision_declaration() {
	check_lower_item_eprime("find x : int(1..10)", expect![[r#""#]]);
}

#[test]
fn test_objective() {
	check_lower_item_eprime("minimising x", expect![[r#""#]]);
}

#[test]
fn test_heuristic() {
	check_lower_item_eprime("heuristic static", expect![[r#""#]])
}

#[test]
fn test_branching() {
	check_lower_item_eprime("branching on [x]", expect![r#""#])
}

#[test]
fn test_constraint() {
	check_lower_item_eprime("such that x, y", expect![[r#""#]])
}

#[test]
fn test_integer_literal() {
	check_lower_item_eprime("letting one be 1", expect![[r#""#]]);
}

#[test]
fn test_boolean_literal() {
	check_lower_item_eprime("letting T = true", expect![[r#""#]]);
}

#[test]
fn test_matrix_literal() {
	check_lower_item_eprime(
        "letting cmatrix: matrix indexed by [ int(1..2), int(1..4) ] of int(1..10) = [ [2,8,5,1], [3,7,9,4] ]",
        expect![[r#""#]]
    )
}
