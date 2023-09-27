//! Eprime Domain Expressions

use super::{Children, Expression, Identifier, Operator};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name, AstNode,
};

ast_enum!(
	Domain,
	"boolean_domain" => BooleanDomain,
	"integer_domain" => IntegerDomain,
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
	range_members,
);

impl IntegerDomain {
	/// Get the range expressions of domain
	pub fn range_members(&self) -> Children<'_, RangeMember> {
		children_with_field_name(self, "member")
	}
}

ast_enum!(
	RangeMember,
	"range_literal" => RangeLiteral,
	_ => Expression,
);

ast_node!(
	/// Range literal
	RangeLiteral,
	min,
	max,
);

impl RangeLiteral {
	/// Get the minimum value of this range
	pub fn min(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "min")
	}

	/// Get the maximum value of this range
	pub fn max(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "max")
	}
}

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
                                            range_members: [
                                                RangeLiteral(
                                                    RangeLiteral {
                                                        cst_kind: "range_literal",
                                                        min: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 1,
                                                                },
                                                            ),
                                                        ),
                                                        max: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 10,
                                                                },
                                                            ),
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
                                            range_members: [
                                                Expression(
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: 1,
                                                        },
                                                    ),
                                                ),
                                                Expression(
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: 3,
                                                        },
                                                    ),
                                                ),
                                                RangeLiteral(
                                                    RangeLiteral {
                                                        cst_kind: "range_literal",
                                                        min: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 5,
                                                                },
                                                            ),
                                                        ),
                                                        max: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 10,
                                                                },
                                                            ),
                                                        ),
                                                    },
                                                ),
                                                RangeLiteral(
                                                    RangeLiteral {
                                                        cst_kind: "range_literal",
                                                        min: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 15,
                                                                },
                                                            ),
                                                        ),
                                                        max: Some(
                                                            IntegerLiteral(
                                                                IntegerLiteral {
                                                                    cst_kind: "integer_literal",
                                                                    value: 20,
                                                                },
                                                            ),
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
                                            range_members: [],
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
                                                        range_members: [
                                                            RangeLiteral(
                                                                RangeLiteral {
                                                                    cst_kind: "range_literal",
                                                                    min: Some(
                                                                        IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 1,
                                                                            },
                                                                        ),
                                                                    ),
                                                                    max: Some(
                                                                        IntegerLiteral(
                                                                            IntegerLiteral {
                                                                                cst_kind: "integer_literal",
                                                                                value: 4,
                                                                            },
                                                                        ),
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
