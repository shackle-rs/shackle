//! AST representation of Eprime Expressions

use super::{BooleanLiteral, Domain, IntegerLiteral, MatrixLiteral, StringLiteral};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name, AstNode, Children,
};

ast_enum!(
	/// Expression
	Expression,
	"boolean_literal" => BooleanLiteral,
	"integer_literal" => IntegerLiteral,
	"string_literal" => StringLiteral,
	"matrix_literal" => MatrixLiteral,
	"call" => Call,
	"identifier" => Identifier,
	"indexed_access" => ArrayAccess,
	"infix_operator" => InfixOperator,
	"prefix_operator" => PrefixOperator,
	"postfix_operator" => PostfixOperator,
	"quantification" => Quantification,
	"matrix_comprehension" => MatrixComprehension,
	"absolute_operator" => AbsoluteOperator,
	"parenthesised_expression" => "expression" // Turn parenthesised_expression into Expression node
);

ast_node!(
	/// Call
	Call,
	function,
	arguments
);

impl Call {
	/// Get the name of this call
	pub fn function(&self) -> Identifier {
		child_with_field_name(self, "function")
	}

	/// Get the arguments of this call
	pub fn arguments(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "argument")
	}
}

ast_node!(
	/// Identifier
	Identifier,
	name
);

impl Identifier {
	/// Get the name of this identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Indexed Access
	ArrayAccess,
	collection,
	indices
);

impl ArrayAccess {
	/// Get the collection of this indexed access
	pub fn collection(&self) -> Expression {
		child_with_field_name(self, "collection")
	}

	/// Get the index of this indexed access
	pub fn indices(&self) -> Children<'_, ArrayIndex> {
		children_with_field_name(self, "index")
	}
}

ast_enum!(
	/// Array Index
	ArrayIndex,
	".." => IndexSlice, // This might be bad
	_ => Expression,
);

ast_node!(IndexSlice, operator,);

impl IndexSlice {
	/// Get the name of this array slice
	pub fn operator(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Infix Operator
	InfixOperator,
	operator,
	left,
	right
);

impl InfixOperator {
	/// Get the operator of this infix operator
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the left expression of this infix operator
	pub fn left(&self) -> Expression {
		child_with_field_name(self, "left")
	}

	/// Get the right expression of this infix operator
	pub fn right(&self) -> Expression {
		child_with_field_name(self, "right")
	}
}

ast_node!(
	/// Prefix Operator
	PrefixOperator,
	operator,
	operand
);

impl PrefixOperator {
	/// Get the operator of this prefix operator
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the operand of this prefix operator
	pub fn operand(&self) -> Expression {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// Postfix Operator
	PostfixOperator,
	operator,
	operand
);

impl PostfixOperator {
	/// Get the operator of this postfix operator
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the operand of this postfix operator
	pub fn operand(&self) -> Expression {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// An operator node
	Operator,
	name,
);

impl Operator {
	/// The name of the operator
	pub fn name(&self) -> &str {
		self.cst_kind()
	}
}

ast_node!(
	/// Quantification
	Quantification,
	function,
	generator,
	template,
);

impl Quantification {
	/// Get the function of this quantification
	pub fn function(&self) -> Identifier {
		child_with_field_name(self, "function")
	}

	/// Get the generator of this quantification
	pub fn generator(&self) -> Generator {
		child_with_field_name(self, "generator")
	}

	/// Get the template of this quantification
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}
}

ast_node!(
	/// Generator
	Generator,
	names,
	collection,
);

impl Generator {
	/// Get the name of this generator
	pub fn names(&self) -> Children<'_, Identifier> {
		children_with_field_name(self, "name")
	}

	/// Get the collection of this generator
	pub fn collection(&self) -> Domain {
		child_with_field_name(self, "collection")
	}
}

ast_node!(
	/// Matrix Comprehension
	MatrixComprehension,
	template,
	generators,
	conditions,
	indices
);

impl MatrixComprehension {
	/// Get the template of this matrix comprehension
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}

	/// Get the generators of this matrix comprehension
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}

	/// Get the conditions of this matrix comprehension
	pub fn conditions(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "condition")
	}

	/// Get the index of this matrix comprehension
	pub fn indices(&self) -> Option<Domain> {
		optional_child_with_field_name(self, "index")
	}
}

ast_node!(
	/// Absolute operator
	AbsoluteOperator,
	operand,
);

impl AbsoluteOperator {
	/// Get the operand of this absolute operator
	pub fn operand(&self) -> Expression {
		child_with_field_name(self, "operand")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::check_ast_eprime;

	#[test]
	fn test_call() {
		check_ast_eprime(
			"letting simple = toVec(X,Y)",
			expect![[r#"
    EPrimeModel(
        Model {
            items: [
                ConstDefinition(
                    ConstDefinition {
                        cst_kind: "const_def",
                        name: Identifier(
                            Identifier {
                                cst_kind: "identifier",
                                name: "simple",
                            },
                        ),
                        definition: Call(
                            Call {
                                cst_kind: "call",
                                function: Identifier {
                                    cst_kind: "identifier",
                                    name: "toVec",
                                },
                                arguments: [
                                    Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "X",
                                        },
                                    ),
                                    Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "Y",
                                        },
                                    ),
                                ],
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

	#[test]
	fn test_indexed_access() {
		check_ast_eprime(
			r#"
            letting single = M[i]
            letting slice = Ms[..]
            "#,
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "single",
                                        },
                                    ),
                                    definition: ArrayAccess(
                                        ArrayAccess {
                                            cst_kind: "indexed_access",
                                            collection: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "M",
                                                },
                                            ),
                                            indices: [
                                                Expression(
                                                    Identifier(
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "i",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "slice",
                                        },
                                    ),
                                    definition: ArrayAccess(
                                        ArrayAccess {
                                            cst_kind: "indexed_access",
                                            collection: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "Ms",
                                                },
                                            ),
                                            indices: [
                                                IndexSlice(
                                                    IndexSlice {
                                                        cst_kind: "..",
                                                        operator: "..",
                                                    },
                                                ),
                                            ],
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

	#[test]
	fn test_infix_operator() {
		check_ast_eprime(
			r#"
            letting different = x != y
            letting smallerlex = x <lex y
            letting and = x /\ y
            letting equiv = x <=> y
            letting exponent = x ** y
            "#,
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "different",
                                        },
                                    ),
                                    definition: InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            operator: Operator {
                                                cst_kind: "!=",
                                                name: "!=",
                                            },
                                            left: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                            right: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "y",
                                                },
                                            ),
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "smallerlex",
                                        },
                                    ),
                                    definition: InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            operator: Operator {
                                                cst_kind: "<lex",
                                                name: "<lex",
                                            },
                                            left: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                            right: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "y",
                                                },
                                            ),
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "and",
                                        },
                                    ),
                                    definition: InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            operator: Operator {
                                                cst_kind: "/\\",
                                                name: "/\\",
                                            },
                                            left: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                            right: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "y",
                                                },
                                            ),
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "equiv",
                                        },
                                    ),
                                    definition: InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            operator: Operator {
                                                cst_kind: "<=>",
                                                name: "<=>",
                                            },
                                            left: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                            right: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "y",
                                                },
                                            ),
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "exponent",
                                        },
                                    ),
                                    definition: InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            operator: Operator {
                                                cst_kind: "**",
                                                name: "**",
                                            },
                                            left: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                            right: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "y",
                                                },
                                            ),
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

	#[test]
	fn test_prefix_operator() {
		check_ast_eprime(
			r#"
            letting negative_ident = -x
            letting negated_bool = !true
            "#,
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "negative_ident",
                                        },
                                    ),
                                    definition: PrefixOperator(
                                        PrefixOperator {
                                            cst_kind: "prefix_operator",
                                            operator: Operator {
                                                cst_kind: "-",
                                                name: "-",
                                            },
                                            operand: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "negated_bool",
                                        },
                                    ),
                                    definition: PrefixOperator(
                                        PrefixOperator {
                                            cst_kind: "prefix_operator",
                                            operator: Operator {
                                                cst_kind: "!",
                                                name: "!",
                                            },
                                            operand: BooleanLiteral(
                                                BooleanLiteral {
                                                    cst_kind: "boolean_literal",
                                                    value: true,
                                                },
                                            ),
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

	#[test]
	fn test_quantification() {
		check_ast_eprime(
			"letting expr = exists i,j : int(1..3) . x[i] = i",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "expr",
                                        },
                                    ),
                                    definition: Quantification(
                                        Quantification {
                                            cst_kind: "quantification",
                                            function: Identifier {
                                                cst_kind: "identifier",
                                                name: "exists",
                                            },
                                            generator: Generator {
                                                cst_kind: "generator",
                                                names: [
                                                    Identifier {
                                                        cst_kind: "identifier",
                                                        name: "i",
                                                    },
                                                    Identifier {
                                                        cst_kind: "identifier",
                                                        name: "j",
                                                    },
                                                ],
                                                collection: IntegerDomain(
                                                    IntegerDomain {
                                                        cst_kind: "integer_domain",
                                                        domain: [
                                                            InfixOperator(
                                                                InfixOperator {
                                                                    cst_kind: "infix_operator",
                                                                    operator: Operator {
                                                                        cst_kind: "..",
                                                                        name: "..",
                                                                    },
                                                                    left: IntegerLiteral(
                                                                        IntegerLiteral {
                                                                            cst_kind: "integer_literal",
                                                                            value: 1,
                                                                        },
                                                                    ),
                                                                    right: IntegerLiteral(
                                                                        IntegerLiteral {
                                                                            cst_kind: "integer_literal",
                                                                            value: 3,
                                                                        },
                                                                    ),
                                                                },
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                            template: InfixOperator(
                                                InfixOperator {
                                                    cst_kind: "infix_operator",
                                                    operator: Operator {
                                                        cst_kind: "=",
                                                        name: "=",
                                                    },
                                                    left: ArrayAccess(
                                                        ArrayAccess {
                                                            cst_kind: "indexed_access",
                                                            collection: Identifier(
                                                                Identifier {
                                                                    cst_kind: "identifier",
                                                                    name: "x",
                                                                },
                                                            ),
                                                            indices: [
                                                                Expression(
                                                                    Identifier(
                                                                        Identifier {
                                                                            cst_kind: "identifier",
                                                                            name: "i",
                                                                        },
                                                                    ),
                                                                ),
                                                            ],
                                                        },
                                                    ),
                                                    right: Identifier(
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "i",
                                                        },
                                                    ),
                                                },
                                            ),
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

	#[test]
	fn test_matrix_comprehension() {
		check_ast_eprime(
			"letting indexed = [ i+j | i: int(1..3), j : int(1..3), i<j ; int(7..) ]",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "indexed",
                                        },
                                    ),
                                    definition: MatrixComprehension(
                                        MatrixComprehension {
                                            cst_kind: "matrix_comprehension",
                                            template: InfixOperator(
                                                InfixOperator {
                                                    cst_kind: "infix_operator",
                                                    operator: Operator {
                                                        cst_kind: "+",
                                                        name: "+",
                                                    },
                                                    left: Identifier(
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "i",
                                                        },
                                                    ),
                                                    right: Identifier(
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "j",
                                                        },
                                                    ),
                                                },
                                            ),
                                            generators: [
                                                Generator {
                                                    cst_kind: "generator",
                                                    names: [
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "i",
                                                        },
                                                    ],
                                                    collection: IntegerDomain(
                                                        IntegerDomain {
                                                            cst_kind: "integer_domain",
                                                            domain: [
                                                                InfixOperator(
                                                                    InfixOperator {
                                                                        cst_kind: "infix_operator",
                                                                        operator: Operator {
                                                                            cst_kind: "..",
                                                                            name: "..",
                                                                        },
                                                                        left: IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 1,
                                                                            },
                                                                        ),
                                                                        right: IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 3,
                                                                            },
                                                                        ),
                                                                    },
                                                                ),
                                                            ],
                                                        },
                                                    ),
                                                },
                                                Generator {
                                                    cst_kind: "generator",
                                                    names: [
                                                        Identifier {
                                                            cst_kind: "identifier",
                                                            name: "j",
                                                        },
                                                    ],
                                                    collection: IntegerDomain(
                                                        IntegerDomain {
                                                            cst_kind: "integer_domain",
                                                            domain: [
                                                                InfixOperator(
                                                                    InfixOperator {
                                                                        cst_kind: "infix_operator",
                                                                        operator: Operator {
                                                                            cst_kind: "..",
                                                                            name: "..",
                                                                        },
                                                                        left: IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 1,
                                                                            },
                                                                        ),
                                                                        right: IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 3,
                                                                            },
                                                                        ),
                                                                    },
                                                                ),
                                                            ],
                                                        },
                                                    ),
                                                },
                                            ],
                                            conditions: [
                                                InfixOperator(
                                                    InfixOperator {
                                                        cst_kind: "infix_operator",
                                                        operator: Operator {
                                                            cst_kind: "<",
                                                            name: "<",
                                                        },
                                                        left: Identifier(
                                                            Identifier {
                                                                cst_kind: "identifier",
                                                                name: "i",
                                                            },
                                                        ),
                                                        right: Identifier(
                                                            Identifier {
                                                                cst_kind: "identifier",
                                                                name: "j",
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                            indices: Some(
                                                IntegerDomain(
                                                    IntegerDomain {
                                                        cst_kind: "integer_domain",
                                                        domain: [
                                                            PostfixOperator(
                                                                PostfixOperator {
                                                                    cst_kind: "postfix_operator",
                                                                    operator: Operator {
                                                                        cst_kind: "..",
                                                                        name: "..",
                                                                    },
                                                                    operand: IntegerLiteral(
                                                                        IntegerLiteral {
                                                                            cst_kind: "integer_literal",
                                                                            value: 7,
                                                                        },
                                                                    ),
                                                                },
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            ),
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

	#[test]
	fn test_absolute() {
		check_ast_eprime(
			"letting absolute = | x |",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "absolute",
                                        },
                                    ),
                                    definition: AbsoluteOperator(
                                        AbsoluteOperator {
                                            cst_kind: "absolute_operator",
                                            operand: Identifier(
                                                Identifier {
                                                    cst_kind: "identifier",
                                                    name: "x",
                                                },
                                            ),
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

	#[test]
	fn test_parenthesis() {
		check_ast_eprime(
			"letting x = ( y )",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ConstDefinition(
                                ConstDefinition {
                                    cst_kind: "const_def",
                                    name: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                    ),
                                    definition: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "y",
                                        },
                                    ),
                                    domain: None,
                                },
                            ),
                        ],
                    },
                )
            "#]],
		)
	}
}
