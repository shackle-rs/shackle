//! AST representation of Eprime Expressions

use super::{BooleanLiteral, IntegerLiteral};
use crate::syntax::ast::{ast_enum, ast_node, AstNode};

ast_enum!(
	/// Expression
	Expression,
	"boolean_literal" => BooleanLiteral,
	// "call" => Call,
	"identifier" => Identifier,
	// "indexed_access" => IndexedAccess,
	// "infix_operator" => InfixOperator,
	// "set_in" => SetIn,
	"integer_literal" => IntegerLiteral,
	// "matrix_literal" => MatrixLiteral,
	// "prefix_operator" => PrefixOperator,
	// "quantification" => Quantification,
	// "matrix_comprehension" => MatrixComprehension,
	// "absolute_operator" => AbsoluteOperator,
);

ast_node!(
	// Identifier
	Identifier, name
);

impl Identifier {
	/// Get the name of this identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::*;

	#[test]
	fn test_identifier() {
		check_ast_eprime(
			"letting x = a",
			expect![[r#"
    EPrimeModel(
        Model {
            items: [
                ConstDefinition(
                    ConstDefinition {
                        cst_kind: "const_def",
                        name: Identifier {
                            cst_kind: "identifier",
                            name: "x",
                        },
                        definition: Identifier(
                            Identifier {
                                cst_kind: "identifier",
                                name: "a",
                            },
                        ),
                        domain: None,
                    },
                ),
            ],
        },
    )
"#]],
		);
	}
}
