//! AST representation of items

use super::{Anonymous, Children, Expression, Identifier, Pattern, StringLiteral, Type};
use crate::ast::{
	AstNode, ast_enum, ast_node, child_with_field_name, children_with_field_name,
	optional_child_with_field_name,
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
	"class_decl" => ClassDecl,
);

ast_node!(
	/// Include item
	Include,
	file
);

impl<'tree> Include<'tree> {
	/// Get the included file
	pub fn file(&self) -> StringLiteral<'tree> {
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

impl<'tree> Declaration<'tree> {
	/// Get the pattern of the declaration
	pub fn pattern(&self) -> Pattern<'tree> {
		child_with_field_name(self, "name")
	}

	/// The type of the declaration
	pub fn declared_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}

	/// Get the right hand side of this declaration if there is one
	pub fn definition(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "definition")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
	}
}

ast_node!(
	/// Enum declaration item
	Enumeration,
	id,
	cases,
	annotations
);

impl<'tree> Enumeration<'tree> {
	/// Get the variable being declared
	pub fn id(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the definition of this enumeration
	pub fn cases(&self) -> Children<'tree, EnumerationCase<'tree>> {
		children_with_field_name(self, "case")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
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

impl<'tree> EnumerationMembers<'tree> {
	/// Get the members of this enum case
	pub fn members(&self) -> Children<'tree, Identifier<'tree>> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Enum definition using anonymous enum
	AnonymousEnumeration,
	parameters
);

impl<'tree> AnonymousEnumeration<'tree> {
	/// Get the callee (will be _)
	pub fn anonymous(&self) -> Anonymous<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'tree, Type<'tree>> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Enum definition using enum constructor call
	EnumerationConstructor,
	id,
	parameters
);

impl<'tree> EnumerationConstructor<'tree> {
	/// Get the id of the call
	pub fn id(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'tree, Parameter<'tree>> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Assignment item
	Assignment,
	assignee,
	definition
);

impl<'tree> Assignment<'tree> {
	/// Get the variable being assigned to
	pub fn assignee(&self) -> Expression<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the right hand side of this assignment
	pub fn definition(&self) -> Expression<'tree> {
		child_with_field_name(self, "definition")
	}
}

ast_node!(
	/// Constraint item
	Constraint,
	expression,
	annotations
);

impl<'tree> Constraint<'tree> {
	/// Get the value of the constraint
	pub fn expression(&self) -> Expression<'tree> {
		child_with_field_name(self, "expression")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Solve item
	Solve,
	goal,
	annotations
);

impl<'tree> Solve<'tree> {
	/// Get the goal of the solve item
	pub fn goal(&self) -> Goal<'tree> {
		match self.cst_node().child_with_field_name("strategy").kind() {
			"satisfy" => Goal::Satisfy,
			"maximize" => Goal::Maximize(Expression::new(
				self.cst_node().child_with_field_name("objective"),
			)),
			"minimize" => Goal::Minimize(Expression::new(
				self.cst_node().child_with_field_name("objective"),
			)),
			_ => unreachable!(),
		}
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}
}

/// Solve goal
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Goal<'tree> {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize(Expression<'tree>),
	/// Minimize the given objective
	Minimize(Expression<'tree>),
}

impl<'tree> Goal<'tree> {
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
	pub fn objective(&self) -> Option<&Expression<'tree>> {
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

impl<'tree> Output<'tree> {
	/// Get the value of the output item
	pub fn expression(&self) -> Expression<'tree> {
		child_with_field_name(self, "expression")
	}

	/// The output section (from the annotation)
	pub fn section(&self) -> Option<StringLiteral<'tree>> {
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

impl<'tree> Function<'tree> {
	/// Get the declared return type of this function
	pub fn return_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}

	/// Get the name of this function
	pub fn id(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this function
	pub fn parameters(&self) -> Children<'tree, Parameter<'tree>> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this function if there is one
	pub fn body(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
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

impl<'tree> Predicate<'tree> {
	/// Get the type of this predicate
	pub fn declared_type(&self) -> PredicateType {
		match self.cst_node().child_with_field_name("type").kind() {
			"predicate" => PredicateType::Predicate,
			"test" => PredicateType::Test,
			_ => unreachable!(),
		}
	}

	/// Get the name of this predicate
	pub fn id(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this predicate
	pub fn parameters(&self) -> Children<'tree, Parameter<'tree>> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this predicate if there is one
	pub fn body(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
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

impl<'tree> Annotation<'tree> {
	/// Get the name of this annotation
	pub fn id(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// Get the parameters if this is an annotation constructor, or return `None`
	/// if this is an atomic annotation.
	pub fn parameters(&self) -> Option<AnnotationParameters<'tree>> {
		optional_child_with_field_name(self, "parameters")
	}

	/// Body of annotation item (not supported, rejected during lowering)
	pub fn body(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "body")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
	}
}

ast_node!(
	/// Annotation constructor function parameters
	AnnotationParameters,
	iter
);

impl<'tree> AnnotationParameters<'tree> {
	/// Get the parameters
	pub fn iter(&self) -> Children<'tree, Parameter<'tree>> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// A function parameter
	Parameter,
	declared_type,
	pattern,
	annotations,
	default,
);

impl<'tree> Parameter<'tree> {
	/// Get the type of this parameter
	pub fn declared_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}

	/// Get the pattern of this parameter if there is one
	pub fn pattern(&self) -> Option<Pattern<'tree>> {
		optional_child_with_field_name(self, "name")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the default value of this parameter if there is one
	pub fn default(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "default")
	}
}

ast_node!(
	/// Type alias item
	TypeAlias,
	name,
	aliased_type,
	annotations
);

impl<'tree> TypeAlias<'tree> {
	/// The name of this type alias
	pub fn name(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// The type this is an alias for
	pub fn aliased_type(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
	}
}

ast_enum!(
	/// Member of a class declaration
	ClassItem,
	"declaration" => Declaration,
	"constraint" => Constraint
);

ast_node!(
	/// Class declaration item
	ClassDecl,
	name,
	extends,
	items,
	annotations,
);

impl<'tree> ClassDecl<'tree> {
	/// The name of this class
	pub fn name(&self) -> Identifier<'tree> {
		child_with_field_name(self, "name")
	}

	/// The superclass this class extends, if any
	pub fn extends(&self) -> Option<Identifier<'tree>> {
		optional_child_with_field_name(self, "extends")
	}

	/// The attributes and constraints declared in this class
	pub fn items(&self) -> Children<'tree, ClassItem<'tree>> {
		children_with_field_name(self, "item")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// Get the documentation comment
	pub fn doc_comment(&self) -> Option<DocComment<'tree>> {
		self.cst_node()
			.previous_named_sibling()
			.filter(|node| node.kind() == "doc_comment")
			.map(DocComment::from)
	}
}

ast_node!(
	/// Documentation comment
	DocComment,
);

impl<'tree> DocComment<'tree> {
	/// Get the text of this documentation comment
	pub fn text<'a>(&self, source: &'a str) -> &'a str {
		self.cst_text(source)
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::ast::tests::*;

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
                                            },
                                        ),
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
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
                                },
                            ),
                        ),
                        definition: IntegerLiteral(
                            IntegerLiteral {
                                cst_kind: "integer_literal",
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
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                                default: None,
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
                            },
                        ),
                        aliased_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Par,
                                opt_type: NonOpt,
                                cardinality: None,
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
