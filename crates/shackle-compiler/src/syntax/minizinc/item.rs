//! AST representation of items

use super::{Anonymous, Children, Expression, Identifier, Pattern, StringLiteral, Type};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name, AstNode,
};

ast_enum!(
	/// Item
	Item,
	"include" => Include,
	"declaration" => Declaration,
	"enumeration" => Enumeration,
	"assignment" => Assignment,
	"constraint" => Constraint,
	"goal" => Solve,
	"output" => Output,
	"function_item" => Function,
	"predicate" => Predicate,
	"annotation" => Annotation,
	"type_alias" => TypeAlias,
);

ast_node!(
	/// Include item
	Include,
	file
);

impl Include {
	/// Get the included file
	pub fn file(&self) -> StringLiteral {
		child_with_field_name(self, "file")
	}
}

ast_node!(
	/// Variable declaration item
	Declaration,
	pattern,
	declared_type,
	definition,
	annotations
);

impl Declaration {
	/// Get the pattern of the declaration
	pub fn pattern(&self) -> Pattern {
		child_with_field_name(self, "name")
	}

	/// The type of the declaration
	pub fn declared_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the right hand side of this declaration if there is one
	pub fn definition(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "definition")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Enum declaration item
	Enumeration,
	id,
	cases,
	annotations
);

impl Enumeration {
	/// Get the variable being declared
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the definition of this enumeration
	pub fn cases(&self) -> Children<'_, EnumerationCase> {
		children_with_field_name(self, "case")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_enum!(
	/// Enum definition cases
	EnumerationCase,
	"enumeration_members" => Members(EnumerationMembers),
	"anonymous_enumeration" => Anonymous(AnonymousEnumeration),
	"enumeration_constructor" => Constructor(EnumerationConstructor)
);

ast_node!(
	/// Enum definition using set of identifiers
	EnumerationMembers,
	members
);

impl EnumerationMembers {
	/// Get the members of this enum case
	pub fn members(&self) -> Children<'_, Identifier> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Enum definition using anonymous enum
	AnonymousEnumeration,
	parameters
);

impl AnonymousEnumeration {
	/// Get the callee (will be _)
	pub fn anonymous(&self) -> Anonymous {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'_, Type> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Enum definition using enum constructor call
	EnumerationConstructor,
	id,
	parameters
);

impl EnumerationConstructor {
	/// Get the id of the call
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'_, Type> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Assignment item
	Assignment,
	assignee,
	definition
);

impl Assignment {
	/// Get the variable being assigned to
	pub fn assignee(&self) -> Expression {
		child_with_field_name(self, "name")
	}

	/// Get the right hand side of this assignment
	pub fn definition(&self) -> Expression {
		child_with_field_name(self, "definition")
	}
}

ast_node!(
	/// Constraint item
	Constraint,
	expression,
	annotations
);

impl Constraint {
	/// Get the value of the constraint
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Solve item
	Solve,
	goal,
	annotations
);

impl Solve {
	/// Get the goal of the solve item
	pub fn goal(&self) -> Goal {
		let tree = self.cst_node().cst();
		let node = self.cst_node().as_ref();
		match node.child_by_field_name("strategy").unwrap().kind() {
			"satisfy" => Goal::Satisfy,
			"maximize" => Goal::Maximize(Expression::new(
				tree.node(node.child_by_field_name("objective").unwrap()),
			)),
			"minimize" => Goal::Minimize(Expression::new(
				tree.node(node.child_by_field_name("objective").unwrap()),
			)),
			_ => unreachable!(),
		}
	}
	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

/// Solve goal
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Goal {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize(Expression),
	/// Minimize the given objective
	Minimize(Expression),
}

impl Goal {
	/// Return whether the solve goal is satisfaction
	pub fn is_satisfy(&self) -> bool {
		matches!(*self, Goal::Satisfy)
	}

	/// Return whether the solve goal is maximization
	pub fn is_maximize(&self) -> bool {
		matches!(*self, Goal::Maximize(_))
	}

	/// Return whether the solve goal is minimization
	pub fn is_minimize(&self) -> bool {
		matches!(*self, Goal::Minimize(_))
	}

	/// Get the objective value if there is one
	pub fn objective(&self) -> Option<&Expression> {
		match *self {
			Goal::Maximize(ref obj) => Some(obj),
			Goal::Minimize(ref obj) => Some(obj),
			_ => None,
		}
	}
}

ast_node!(
	/// Output item
	Output,
	expression,
	section
);

impl Output {
	/// Get the value of the output item
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}
	/// The output section (from the annotation)
	pub fn section(&self) -> Option<StringLiteral> {
		optional_child_with_field_name(self, "section")
	}
}

ast_node!(
	/// Function item
	Function,
	return_type,
	id,
	parameters,
	body,
	annotations
);

impl Function {
	/// Get the declared return type of this function
	pub fn return_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the name of this function
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this function
	pub fn parameters(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this function if there is one
	pub fn body(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Predicate item
	Predicate,
	declared_type,
	id,
	parameters,
	body,
	annotations
);

impl Predicate {
	/// Get the type of this predicate
	pub fn declared_type(&self) -> PredicateType {
		match self
			.cst_node()
			.as_ref()
			.child_by_field_name("type")
			.unwrap()
			.kind()
		{
			"predicate" => PredicateType::Predicate,
			"test" => PredicateType::Test,
			_ => unreachable!(),
		}
	}

	/// Get the name of this predicate
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this predicate
	pub fn parameters(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this predicate if there is one
	pub fn body(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

/// Return type of predicate
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredicateType {
	/// `var bool` function
	Predicate,
	/// `par bool` function
	Test,
}

impl PredicateType {
	/// Return whether this is a predicate
	pub fn is_predicate(&self) -> bool {
		matches!(*self, PredicateType::Predicate)
	}

	/// Return whether this is a test
	pub fn is_test(&self) -> bool {
		matches!(*self, PredicateType::Test)
	}
}

ast_node!(
	/// Annotation item
	Annotation,
	id,
	parameters
);

impl Annotation {
	/// Get the name of this annotation
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters if this is an annotation constructor, or return `None`
	/// if this is an atomic annotation.
	pub fn parameters(&self) -> Option<AnnotationParameters> {
		optional_child_with_field_name(self, "parameters")
	}
}

ast_node!(
	/// Annotation constructor function parameters
	AnnotationParameters,
	iter
);

impl AnnotationParameters {
	/// Get the parameters
	pub fn iter(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// A function parameter
	Parameter,
	declared_type,
	pattern,
	annotations
);

impl Parameter {
	/// Get the type of this parameter
	pub fn declared_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the pattern of this parameter if there is one
	pub fn pattern(&self) -> Option<Pattern> {
		optional_child_with_field_name(self, "name")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Type alias item
	TypeAlias,
	name,
	aliased_type,
	annotations
);

impl TypeAlias {
	/// The name of this type alias
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// The type this is an alias for
	pub fn aliased_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::*;

	#[test]
	fn test_include() {
		check_ast(
			r#"include "foo.mzn";"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Include(
                    Include {
                        cst_kind: "include",
                        file: StringLiteral {
                            cst_kind: "string_literal",
                            value: "foo.mzn",
                        },
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_declaration() {
		check_ast(
			"int: x = 3;",
			expect!([r#"
    MznModel(
        Model {
            items: [
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: None,
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: Int,
                                    },
                                ),
                            },
                        ),
                        definition: Some(
                            IntegerLiteral(
                                IntegerLiteral {
                                    cst_kind: "integer_literal",
                                    value: Ok(
                                        3,
                                    ),
                                },
                            ),
                        ),
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_enumeration() {
		check_ast(
			"enum Foo = {A, B, C};",
			expect!([r#"
    MznModel(
        Model {
            items: [
                Enumeration(
                    Enumeration {
                        cst_kind: "enumeration",
                        id: UnquotedIdentifier(
                            UnquotedIdentifier {
                                cst_kind: "identifier",
                                name: "Foo",
                            },
                        ),
                        cases: [
                            Members(
                                EnumerationMembers {
                                    cst_kind: "enumeration_members",
                                    members: [
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "A",
                                            },
                                        ),
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "B",
                                            },
                                        ),
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "C",
                                            },
                                        ),
                                    ],
                                },
                            ),
                        ],
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_assignment() {
		check_ast(
			"x = 1;",
			expect!([r#"
    MznModel(
        Model {
            items: [
                Assignment(
                    Assignment {
                        cst_kind: "assignment",
                        assignee: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        definition: IntegerLiteral(
                            IntegerLiteral {
                                cst_kind: "integer_literal",
                                value: Ok(
                                    1,
                                ),
                            },
                        ),
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_constraint() {
		check_ast(
			"constraint x > 1;",
			expect!([r#"
    MznModel(
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
        },
    )
"#]),
		);
	}

	#[test]
	fn test_solve() {
		check_ast(
			"solve minimize x;",
			expect!([r#"
    MznModel(
        Model {
            items: [
                Solve(
                    Solve {
                        cst_kind: "goal",
                        goal: Minimize(
                            Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "x",
                                    },
                                ),
                            ),
                        ),
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_output() {
		check_ast(
			r#"output ["foo"];"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Output(
                    Output {
                        cst_kind: "output",
                        expression: ArrayLiteral(
                            ArrayLiteral {
                                cst_kind: "array_literal",
                                members: [
                                    ArrayLiteralMember {
                                        cst_kind: "array_literal_member",
                                        indices: None,
                                        value: StringLiteral(
                                            StringLiteral {
                                                cst_kind: "string_literal",
                                                value: "foo",
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                        section: None,
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_function() {
		check_ast(
			"function int: foo(int: x) = x + 1;",
			expect!([r#"
    MznModel(
        Model {
            items: [
                Function(
                    Function {
                        cst_kind: "function_item",
                        return_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: None,
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: Int,
                                    },
                                ),
                            },
                        ),
                        id: UnquotedIdentifier(
                            UnquotedIdentifier {
                                cst_kind: "identifier",
                                name: "foo",
                            },
                        ),
                        parameters: [
                            Parameter {
                                cst_kind: "parameter",
                                declared_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                                pattern: Some(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "x",
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                            },
                        ],
                        body: Some(
                            InfixOperator(
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
                                        cst_kind: "+",
                                        name: "+",
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
                        ),
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}

	#[test]
	fn test_type_alias() {
		check_ast(
			"type Foo = set of int",
			expect!([r#"
    MznModel(
        Model {
            items: [
                TypeAlias(
                    TypeAlias {
                        cst_kind: "type_alias",
                        name: UnquotedIdentifier(
                            UnquotedIdentifier {
                                cst_kind: "identifier",
                                name: "Foo",
                            },
                        ),
                        aliased_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Par,
                                opt_type: NonOpt,
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                        annotations: [],
                    },
                ),
            ],
        },
    )
"#]),
		);
	}
}
