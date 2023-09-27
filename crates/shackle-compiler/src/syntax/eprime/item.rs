//! AST representation of Eprime items

use super::{Domain, Expression, Identifier};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name, AstNode, Children,
};

ast_enum!(
	/// Item
	Item,
	"param_decl" => ParamDeclaration,
	"const_def" => ConstDefinition,
	"domain_alias" => DomainAlias,
	"decision_decl" => DecisionDeclaration,
	"objective" => Objective,
	"branching" => Branching,
	"heuristic" => Heuristic,
	"constraint" => Constraint,
);

ast_node!(
	/// Parameter Declaration
	ParamDeclaration,
	names,
	domain,
	wheres,
);

impl ParamDeclaration {
	/// Get variable being declared
	pub fn names(&self) -> Children<'_, Identifier> {
		children_with_field_name(self, "name")
	}

	/// Domain of variable
	pub fn domain(&self) -> Domain {
		child_with_field_name(self, "domain")
	}

	/// Where clauses
	pub fn wheres(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "where")
	}
}

ast_node!(
	/// Constant Definition
	ConstDefinition,
	name,
	definition,
	domain,
);

impl ConstDefinition {
	/// Get constant being declared
	pub fn name(&self) -> Expression {
		child_with_field_name(self, "name")
	}

	/// Definition of constant
	pub fn definition(&self) -> Expression {
		child_with_field_name(self, "definition")
	}

	/// Optional domain of constant
	pub fn domain(&self) -> Option<Domain> {
		optional_child_with_field_name(self, "domain")
	}
}

ast_node!(
	/// Domain Alias
	DomainAlias,
	name,
	definition,
);

impl DomainAlias {
	/// Get alias being declared
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Definition of alias
	pub fn definition(&self) -> Domain {
		child_with_field_name(self, "definition")
	}
}

ast_node!(
	/// Decision Declaration
	DecisionDeclaration,
	names,
	domain,
);

impl DecisionDeclaration {
	/// Get variables being declared
	pub fn names(&self) -> Children<'_, Identifier> {
		children_with_field_name(self, "name")
	}

	/// Domain of decision
	pub fn domain(&self) -> Domain {
		child_with_field_name(self, "domain")
	}
}

ast_node!(
	/// Objective
	Objective,
	strategy,
	expression,
);

impl Objective {
	/// Get objective strategy
	pub fn strategy(&self) -> ObjectiveStrategy {
		let node = self.cst_node().as_ref();
		match node.child_by_field_name("strategy").unwrap().kind() {
			"minimising" => ObjectiveStrategy::Minimising,
			"maximising" => ObjectiveStrategy::Maximising,
			_ => unreachable!(),
		}
	}

	/// Get objective expression
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ObjectiveStrategy {
	Minimising,
	Maximising,
}

ast_node!(
	/// Branching
	Branching,
	expressions,
);

impl Branching {
	/// Get branching expressions
	pub fn expressions(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "expression")
	}
}

ast_node!(
	/// Heuristic
	Heuristic,
	heuristic,
);

impl Heuristic {
	/// Get heuristic expression
	pub fn heuristic(&self) -> Option<HeuristicType> {
		optional_child_with_field_name(self, "heuristic")
	}
}

ast_node!(
	/// Heuristic Type
	HeuristicType,
	name,
);

impl HeuristicType {
	/// Get heuristic name
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Constraint
	Constraint,
	expressions,
);

impl Constraint {
	/// Get constraint expressions
	pub fn expressions(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "expression")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::check_ast_eprime;

	#[test]
	fn test_const_definition() {
		check_ast_eprime(
			r#"
                letting x = 10
                letting x be 10
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
                                            name: "x",
                                        },
                                    ),
                                    definition: IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: 10,
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
                                            name: "x",
                                        },
                                    ),
                                    definition: IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: 10,
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
	fn test_param_declaration() {
		check_ast_eprime(
			r#"
                given x: int(1..10)
                given y: int(1..10)
                    where y < x
            "#,
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
                                            name: "x",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            range_members: [
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
                                            name: "y",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            range_members: [
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
                                                                value: 10,
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                    wheres: [
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
                                                        name: "y",
                                                    },
                                                ),
                                                right: Identifier(
                                                    Identifier {
                                                        cst_kind: "identifier",
                                                        name: "x",
                                                    },
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                )
            "#]],
		);
	}

	#[test]
	fn test_domain_alias() {
		check_ast_eprime(
			"letting INDEX be domain int(1..c*n)",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            DomainAlias(
                                DomainAlias {
                                    cst_kind: "domain_alias",
                                    name: Identifier {
                                        cst_kind: "identifier",
                                        name: "INDEX",
                                    },
                                    definition: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            range_members: [
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
                                                        right: InfixOperator(
                                                            InfixOperator {
                                                                cst_kind: "infix_operator",
                                                                operator: Operator {
                                                                    cst_kind: "*",
                                                                    name: "*",
                                                                },
                                                                left: Identifier(
                                                                    Identifier {
                                                                        cst_kind: "identifier",
                                                                        name: "c",
                                                                    },
                                                                ),
                                                                right: Identifier(
                                                                    Identifier {
                                                                        cst_kind: "identifier",
                                                                        name: "n",
                                                                    },
                                                                ),
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                },
                            ),
                        ],
                    },
                )
            "#]],
		);
	}

	#[test]
	fn test_decision_declaration() {
		check_ast_eprime(
			"find x : int(1..10)",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            DecisionDeclaration(
                                DecisionDeclaration {
                                    cst_kind: "decision_decl",
                                    names: [
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                    ],
                                    domain: IntegerDomain(
                                        IntegerDomain {
                                            cst_kind: "integer_domain",
                                            range_members: [
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
                                                                value: 10,
                                                            },
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                },
                            ),
                        ],
                    },
                )
            "#]],
		);
	}

	#[test]
	fn test_objective() {
		check_ast_eprime(
			"minimising x",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            Objective(
                                Objective {
                                    cst_kind: "objective",
                                    strategy: ObjectiveStrategy {
                                        cst_kind: "minimising",
                                        name: "minimising",
                                    },
                                    expression: Identifier(
                                        Identifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                    ),
                                },
                            ),
                        ],
                    },
                )
            "#]],
		);
	}

	#[test]
	fn test_heuristic() {
		check_ast_eprime(
			"heuristic static",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            Heuristic(
                                Heuristic {
                                    cst_kind: "heuristic",
                                    heuristic: Some(
                                        HeuristicType {
                                            cst_kind: "static",
                                            name: "static",
                                        },
                                    ),
                                },
                            ),
                        ],
                    },
                )
            "#]],
		)
	}

	#[test]
	fn test_branching() {
		check_ast_eprime(
			"branching on [x]",
			expect![
				r#"
                EPrimeModel(
                    Model {
                        items: [
                            Branching(
                                Branching {
                                    cst_kind: "branching",
                                    expressions: [
                                        Identifier(
                                            Identifier {
                                                cst_kind: "identifier",
                                                name: "x",
                                            },
                                        ),
                                    ],
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
	fn test_constraint() {
		check_ast_eprime(
			"such that x, y",
			expect![[r#"
                EPrimeModel(
                    Model {
                        items: [
                            Constraint(
                                Constraint {
                                    cst_kind: "constraint",
                                    expressions: [
                                        Identifier(
                                            Identifier {
                                                cst_kind: "identifier",
                                                name: "x",
                                            },
                                        ),
                                        Identifier(
                                            Identifier {
                                                cst_kind: "identifier",
                                                name: "y",
                                            },
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                )
            "#]],
		)
	}
}
