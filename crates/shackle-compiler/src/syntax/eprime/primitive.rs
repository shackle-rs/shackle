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
		check_ast_eprime(
			"letting one be 1",
			expect!([r#"
        EPrimeModel(
            Model {
                items: [
                    ConstDefinition(
                        ConstDefinition {
                            cst_kind: "const_def",
                            name: Identifier {
                                cst_kind: "identifier",
                                name: "one",
                            },
                            definition: IntegerLiteral(
                                IntegerLiteral {
                                    cst_kind: "integer_literal",
                                    value: 1,
                                },
                            ),
                            domain: None,
                        },
                    ),
                ],
            },
        )
"#]),
		);
	}

	#[test]
	fn test_boolean_literal() {
		check_ast_eprime(
			"letting T = true",
			expect!([r#"
            EPrimeModel(
                Model {
                    items: [
                        ConstDefinition(
                            ConstDefinition {
                                cst_kind: "const_def",
                                name: Identifier {
                                    cst_kind: "identifier",
                                    name: "T",
                                },
                                definition: BooleanLiteral(
                                    BooleanLiteral {
                                        cst_kind: "boolean_literal",
                                        value: true,
                                    },
                                ),
                                domain: None,
                            },
                        ),
                    ],
                },
            )
"#]),
		);
	}
}
