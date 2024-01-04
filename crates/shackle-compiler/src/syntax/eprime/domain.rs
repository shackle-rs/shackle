//! Eprime Domain Expressions

use super::{Children, Expression, Identifier, Operator};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name, AstNode,
};

ast_enum!(
	/// Domain
	Domain,
	"boolean_domain" => BooleanDomain,
	"integer_domain" => IntegerDomain,
	"any_domain" => AnyDomain,
	"matrix_domain" => MatrixDomain,
	"domain_operation" => DomainOperation,
	_ => Identifier,
);

ast_node!(
	/// Boolean domain
	BooleanDomain,
);

impl BooleanDomain {}

ast_node!(
	/// Integer domain
	IntegerDomain,
	domain,
);

impl IntegerDomain {
	/// Get the range expressions of domain
	pub fn domain(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Type is inferred for RHS
	AnyDomain,
);

ast_node!(
	/// Matrix domain
	MatrixDomain,
	indexes,
	base,
);

impl MatrixDomain {
	/// Get the indexes of this matrix domain
	pub fn indexes(&self) -> Children<'_, Domain> {
		children_with_field_name(self, "index")
	}

	/// Get the base domain of this matrix domain
	pub fn base(&self) -> Domain {
		child_with_field_name(self, "base")
	}
}

ast_node!(
	/// Domain operation
	DomainOperation,
	operator,
	left,
	right,
);

impl DomainOperation {
	/// Get the operator of this domain operation
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the left operand of this domain operation
	pub fn left(&self) -> Domain {
		child_with_field_name(self, "left")
	}

	/// Get the right operand of this domain operation
	pub fn right(&self) -> Domain {
		child_with_field_name(self, "right")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::check_ast_eprime;

	#[test]
	fn test_domain_operation() {
		check_ast_eprime(
			"given a: int(1..2) union int(3..4)",
			expect![
				r#"
                EPrimeModel(
                    Model {
                        items: [
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "a",
                                        },
                                    ],
                                    domain: DomainOperation(
                                        DomainOperation {
                                            cst_kind: "domain_operation",
                                            operator: Operator {
                                                cst_kind: "union",
                                                name: "union",
                                            },
                                            left: IntegerDomain(
                                                IntegerDomain {
                                                    cst_kind: "integer_domain",
                                                    domain: [
                                                        SetConstructor(
                                                            SetConstructor {
                                                                cst_kind: "set_constructor",
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
                                                                        value: 2,
                                                                    },
                                                                ),
                                                            },
                                                        ),
                                                    ],
                                                },
                                            ),
                                            right: IntegerDomain(
                                                IntegerDomain {
                                                    cst_kind: "integer_domain",
                                                    domain: [
                                                        SetConstructor(
                                                            SetConstructor {
                                                                cst_kind: "set_constructor",
                                                                operator: Operator {
                                                                    cst_kind: "..",
                                                                    name: "..",
                                                                },
                                                                left: IntegerLiteral(
                                                                    IntegerLiteral {
                                                                        cst_kind: "integer_literal",
                                                                        value: 3,
                                                                    },
                                                                ),
                                                                right: IntegerLiteral(
                                                                    IntegerLiteral {
                                                                        cst_kind: "integer_literal",
                                                                        value: 4,
                                                                    },
                                                                ),
                                                            },
                                                        ),
                                                    ],
                                                },
                                            ),
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                        ],
                    },
                )
        "#
			],
		)
	}

	#[test]
	fn test_integer_domain() {
		check_ast_eprime(
			r#"
            given a: int(1..10)
            given i: int(1,3,5..10,15..20)
            given j: int
            "#,
			expect!([r#"
                EPrimeModel(
                    Model {
                        items: [
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "a",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            domain: [
                                                SetConstructor(
                                                    SetConstructor {
                                                        cst_kind: "set_constructor",
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
                                                                value: 10,
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "i",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            domain: [
                                                IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: 1,
                                                    },
                                                ),
                                                IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: 3,
                                                    },
                                                ),
                                                SetConstructor(
                                                    SetConstructor {
                                                        cst_kind: "set_constructor",
                                                        operator: Operator {
                                                            cst_kind: "..",
                                                            name: "..",
                                                        },
                                                        left: IntegerLiteral(
                                                            IntegerLiteral {
                                                                cst_kind: "integer_literal",
                                                                value: 5,
                                                            },
                                                        ),
                                                        right: IntegerLiteral(
                                                            IntegerLiteral {
                                                                cst_kind: "integer_literal",
                                                                value: 10,
                                                            },
                                                        ),
                                                    },
                                                ),
                                                SetConstructor(
                                                    SetConstructor {
                                                        cst_kind: "set_constructor",
                                                        operator: Operator {
                                                            cst_kind: "..",
                                                            name: "..",
                                                        },
                                                        left: IntegerLiteral(
                                                            IntegerLiteral {
                                                                cst_kind: "integer_literal",
                                                                value: 15,
                                                            },
                                                        ),
                                                        right: IntegerLiteral(
                                                            IntegerLiteral {
                                                                cst_kind: "integer_literal",
                                                                value: 20,
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "j",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            domain: [],
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                        ],
                    },
                )
            "#]),
		);
	}

	#[test]
	fn test_boolean_domain() {
		check_ast_eprime(
			r#"
                given x: bool
                given x, y: bool
            "#,
			expect!([r#"
                EPrimeModel(
                    Model {
                        items: [
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                    ],
                                    domain: BooleanDomain(
                                        BooleanDomain {
                                            cst_kind: "boolean_domain",
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "y",
                                        },
                                    ],
                                    domain: BooleanDomain(
                                        BooleanDomain {
                                            cst_kind: "boolean_domain",
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                        ],
                    },
                )
            "#]),
		);
	}

	#[test]
	fn test_matrix_domain() {
		check_ast_eprime(
			"given simple: matrix indexed by [int(1..4)] of bool",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            ParamDeclaration(
                                ParamDeclaration {
                                    cst_kind: "param_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "simple",
                                        },
                                    ],
                                    domain: MatrixDomain(
                                        MatrixDomain {
                                            cst_kind: "matrix_domain",
                                            indexes: [
                                                IntegerDomain(
                                                    IntegerDomain {
                                                        cst_kind: "integer_domain",
                                                        domain: [
                                                            SetConstructor(
                                                                SetConstructor {
                                                                    cst_kind: "set_constructor",
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
                                                                            value: 4,
                                                                        },
                                                                    ),
                                                                },
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            ],
                                            base: BooleanDomain(
                                                BooleanDomain {
                                                    cst_kind: "boolean_domain",
                                                },
                                            ),
                                        },
                                    ),
                                    wheres: [],
                                },
                            ),
                        ],
                    },
                )
            "#]],
		)
	}
}
