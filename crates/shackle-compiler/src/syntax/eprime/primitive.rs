//! AST representation of primitive values

use crate::syntax::ast::{ast_node, AstNode};

ast_node!(
	/// Integer literal
	IntegerLiteral,
	value
);

impl IntegerLiteral {
	/// Get the value of this integer literal
	pub fn value(&self) -> i64 {
		self.cst_text().parse().unwrap()
	}
}

ast_node!(
	/// Boolean literal
	BooleanLiteral,
	value
);

impl BooleanLiteral {
	/// Get the value of this boolean literal
	pub fn value(&self) -> bool {
		match self.cst_text() {
			"true" => true,
			"false" => false,
			_ => unreachable!(),
		}
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::*;

	#[test]
	fn test_integer_literal() {
		check_ast(
			"letting one be 1",
			expect!([r#"
    Model {
        items: [],
    }
"#]),
		);
	}

	#[test]
	fn test_boolean_literal() {
		check_ast(
			"constraint x > 1;",
			expect!([r#"
    Model {
        items: [
            Constraint(
                Constraint {
                    cst_kind: "constraint",
                    expression: InfixOperator(
                        InfixOperator {
                            cst_kind: "infix_operator",
                            left: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "x",
                                    },
                                ),
                            ),
                            operator: Operator {
                                cst_kind: ">",
                                name: ">",
                            },
                            right: IntegerLiteral(
                                IntegerLiteral {
                                    cst_kind: "integer_literal",
                                    value: Ok(
                                        1,
                                    ),
                                },
                            ),
                        },
                    ),
                    annotations: [],
                },
            ),
        ],
    }
"#]),
		);
	}
}
