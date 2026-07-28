//! AST representation of expressions

use std::borrow::Cow;

use super::{
	Absent, ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, BooleanLiteral,
	Children, Constraint, Declaration, FloatLiteral, Generator, Infinity, IntegerLiteral,
	Parameter, Pattern, RecordLiteral, SetComprehension, SetLiteral, StringLiteral, TupleLiteral,
	Type,
};
use crate::{
	ast::{
		AstNode, ast_enum, ast_node, child_with_field_name, children_with_field_name,
		decode_string_literal, optional_child_with_field_name,
	},
	cst::CstNode,
};

ast_enum!(
	/// Expression
	Expression,
	"integer_literal" => IntegerLiteral,
	"float_literal" => FloatLiteral,
	"tuple_literal" => TupleLiteral,
	"record_literal" => RecordLiteral,
	"set_literal" => SetLiteral,
	"boolean_literal" => BooleanLiteral,
	"string_literal" => StringLiteral,
	"identifier" | "quoted_identifier" | "inversed_identifier" => Identifier,
	"absent" => Absent,
	"infinity" => Infinity,
	"anonymous" => Anonymous,
	"array_literal" => ArrayLiteral,
	"array_literal_2d" => ArrayLiteral2D,
	"indexed_access" => ArrayAccess,
	"array_comprehension" => ArrayComprehension,
	"set_comprehension" => SetComprehension,
	"if_then_else" => IfThenElse,
	"call" => Call,
	"prefix_operator" => PrefixOperator,
	"infix_operator" => InfixOperator,
	"postfix_operator" => PostfixOperator,
	"generator_call" => GeneratorCall,
	"string_interpolation" => StringInterpolation,
	"case_expression" => Case,
	"let_expression" => Let,
	"tuple_access" => TupleAccess,
	"record_access" => RecordAccess,
	"lambda" => Lambda,
	"annotated_expression" => AnnotatedExpression,
	"parenthesised_expression" => "expression" // Turn parenthesised_expression into Expression node
);

impl<'tree> Expression<'tree> {
	/// Whether or not this expression is parenthesised
	pub fn is_parenthesised(&self) -> bool {
		self.cst_node()
			.parent()
			.map(|p| p.kind() == "parenthesised_expression")
			.unwrap_or_default()
	}
}

ast_node!(
	/// An annotated expression
	AnnotatedExpression,
	annotations,
	expression
);

impl<'tree> AnnotatedExpression<'tree> {
	/// The annotations
	pub fn annotations(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "annotation")
	}

	/// The expression which was annotated
	pub fn expression(&self) -> Expression<'tree> {
		child_with_field_name(self, "expression")
	}
}

ast_enum!(
	/// An identifier (quoted or normal)
	Identifier,
	"identifier" => UnquotedIdentifier,
	"quoted_identifier" => QuotedIdentifier,
	"inversed_identifier" => InversedIdentifier
);

impl<'tree> Identifier<'tree> {
	/// Get the name of this identifier
	pub fn name<'a>(&self, source: &'a str) -> Cow<'a, str> {
		match *self {
			Identifier::QuotedIdentifier(ref i) => Cow::from(i.name(source)),
			Identifier::UnquotedIdentifier(ref i) => Cow::from(i.name(source)),
			Identifier::InversedIdentifier(ref i) => Cow::from(i.name(source)),
		}
	}
}

ast_node!(
	/// Identifier
	UnquotedIdentifier
);

impl<'tree> UnquotedIdentifier<'tree> {
	/// Get the name of this identifier
	pub fn name<'a>(&self, source: &'a str) -> &'a str {
		self.cst_text(source)
	}
}

ast_node!(
	/// Quoted identifier
	QuotedIdentifier
);

impl<'tree> QuotedIdentifier<'tree> {
	/// Get the name of this identifier without the enclosing quotes
	pub fn name<'a>(&self, source: &'a str) -> &'a str {
		let text = self.cst_text(source);
		&text[1..text.len() - 1]
	}
}

ast_node!(
	/// Inversed identifier Foo^-1
	InversedIdentifier,
	identifier
);

impl<'tree> InversedIdentifier<'tree> {
	/// Get the identifier (without the ^-1)
	pub fn identifier(&self) -> Identifier<'tree> {
		child_with_field_name(self, "identifier")
	}

	/// Get the name of this identifier ending with ⁻¹ without any enclosing quotes
	pub fn name(&self, source: &str) -> String {
		format!("{}⁻¹", self.identifier().name(source))
	}
}

ast_node!(
	/// Anonymous variable `_`
	Anonymous,
);

ast_node!(
	/// If-then-else
	IfThenElse,
	branches,
	else_result
);

impl<'tree> IfThenElse<'tree> {
	/// If-then and elseif-then pairs
	pub fn branches(&self) -> Branches<'tree> {
		Branches {
			conditions: children_with_field_name(self, "condition"),
			results: children_with_field_name(self, "result"),
		}
	}

	/// Else expression
	pub fn else_result(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "else")
	}
}

/// Iterator over the branches of an `IfThenElse`

#[derive(Clone, Debug)]
pub struct Branches<'tree> {
	/// Conditions for each if/elseif branch.
	conditions: Children<'tree, Expression<'tree>>,
	/// Result expressions paired with each condition.
	results: Children<'tree, Expression<'tree>>,
}

impl<'tree> Iterator for Branches<'tree> {
	type Item = Branch<'tree>;

	fn next(&mut self) -> Option<Self::Item> {
		match (self.conditions.next(), self.results.next()) {
			(Some(condition), Some(result)) => Some(Branch { condition, result }),
			(None, None) => None,
			_ => unreachable!("Mismatch in size of conditions and results for if-then-else"),
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Branch<'tree> {
	/// The boolean condition
	pub condition: Expression<'tree>,
	/// The result if the condition holds
	pub result: Expression<'tree>,
}

ast_node!(
	/// Function call
	Call,
	function,
	arguments
);

impl<'tree> Call<'tree> {
	/// Get the expression being called
	/// Will usually be an identifier
	pub fn function(&self) -> Expression<'tree> {
		child_with_field_name(self, "function")
	}

	/// Get the call arguments.
	pub fn arguments(&self) -> Children<'tree, ArgOrParam<'tree>> {
		children_with_field_name(self, "argument")
	}

	/// Get the call arguments as expressions (used for dzn)
	pub fn dzn_arguments(&self) -> Children<'tree, Expression<'tree>> {
		children_with_field_name(self, "argument")
	}
}

ast_node!(
	/// Either a call argument (possibly named) or a parameter if this is a actually from an enum assignment
	ArgOrParam,
	left,
	right,
	default,
);

impl<'tree> ArgOrParam<'tree> {
	/// The left hand side of the colon as a type (or the sole type if there is no colon)
	pub fn left(&self) -> Type<'tree> {
		child_with_field_name(self, "type")
	}

	/// The right hand side of the colon as an expression
	pub fn right(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "expression")
	}

	/// The right hand side of the equals sign
	pub fn default(&self) -> Option<Expression<'tree>> {
		optional_child_with_field_name(self, "default")
	}
}

ast_node!(
	/// An operator node
	Operator,
	name,
);

impl<'tree> Operator<'tree> {
	/// The name of the operator
	pub fn name(&self) -> &str {
		self.cst_kind()
	}
}
ast_node!(
	/// Prefix (unary) operator
	PrefixOperator,
	operator,
	operand
);

impl<'tree> PrefixOperator<'tree> {
	/// Get the operator
	pub fn operator(&self) -> Operator<'tree> {
		child_with_field_name(self, "operator")
	}

	/// Get the operand
	pub fn operand(&self) -> Expression<'tree> {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// Infix (binary) operator
	InfixOperator,
	left,
	operator,
	right
);

impl<'tree> InfixOperator<'tree> {
	/// Get the left hand side
	pub fn operator(&self) -> Operator<'tree> {
		child_with_field_name(self, "operator")
	}

	/// Get the left hand side
	pub fn left(&self) -> Expression<'tree> {
		child_with_field_name(self, "left")
	}

	/// Get the left hand side
	pub fn right(&self) -> Expression<'tree> {
		child_with_field_name(self, "right")
	}
}

ast_node!(
	/// Postfix operator
	PostfixOperator,
	operand,
	operator,
);

impl<'tree> PostfixOperator<'tree> {
	/// Get the operator
	pub fn operator(&self) -> Operator<'tree> {
		child_with_field_name(self, "operator")
	}

	/// Get the operand
	pub fn operand(&self) -> Expression<'tree> {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// Call using generator syntax
	GeneratorCall,
	function,
	generators,
	template
);

impl<'tree> GeneratorCall<'tree> {
	/// Get the expression being called
	/// Should always be an `Identifier` for now but for lambdas would be something else
	pub fn function(&self) -> Expression<'tree> {
		child_with_field_name(self, "function")
	}

	/// The generators for this call
	pub fn generators(&self) -> Children<'tree, Generator<'tree>> {
		children_with_field_name(self, "generator")
	}

	/// The body of this call
	pub fn template(&self) -> Expression<'tree> {
		child_with_field_name(self, "template")
	}
}

ast_node!(
	/// String interpolation
	StringInterpolation,
	contents
);

impl<'tree> StringInterpolation<'tree> {
	/// Get the contents of this string interpolation
	pub fn contents(&self) -> Children<'tree, InterpolationItem<'tree>> {
		children_with_field_name(self, "item")
	}
}

#[derive(Clone, Eq, PartialEq, Hash)]
/// Internal representation of string interpolation contents.
enum InterpolationPart<'tree> {
	/// Literal string content.
	String(CstNode<'tree>),
	/// Embedded MiniZinc expression.
	Expression(Expression<'tree>),
}

impl std::fmt::Debug for InterpolationPart<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InterpolationPart::String(_) => write!(f, "StringCharacters"),
			InterpolationPart::Expression(e) => e.fmt(f),
		}
	}
}

/// An element in a string interpolation
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct InterpolationItem<'tree>(InterpolationPart<'tree>);

impl<'tree> InterpolationItem<'tree> {
	/// Get the string or expresssion
	pub fn value(&'tree self, source: &str) -> InterpolationValue<'tree> {
		match self.0 {
			InterpolationPart::String(ref cst_node) => {
				InterpolationValue::String(decode_string_literal(cst_node, source))
			}
			InterpolationPart::Expression(ref e) => InterpolationValue::Expression(e),
		}
	}
}

impl<'tree> AstNode<'tree> for InterpolationItem<'tree> {
	fn cst_node(&self) -> &CstNode<'tree> {
		match self.0 {
			InterpolationPart::String(ref n) => n,
			InterpolationPart::Expression(ref e) => e.cst_node(),
		}
	}
}

impl<'tree> From<CstNode<'tree>> for InterpolationItem<'tree> {
	fn from(syntax: CstNode<'tree>) -> Self {
		match syntax.kind() {
			"string" => InterpolationItem(InterpolationPart::String(syntax)),
			"expression" => InterpolationItem(InterpolationPart::Expression(Expression::new(
				syntax.child(0).unwrap(),
			))),
			_ => unreachable!(),
		}
	}
}

/// A value in a string interpolation, either a string or an expression
///
/// Returned by `InterpolationItem::value()`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum InterpolationValue<'tree> {
	/// String value
	String(String),
	/// Expression
	Expression(&'tree Expression<'tree>),
}

ast_node!(
	/// Let expression
	Let,
	items,
	in_expression
);

impl<'tree> Let<'tree> {
	/// The items of the let expression
	pub fn items(&self) -> Children<'tree, LetItem<'tree>> {
		children_with_field_name(self, "item")
	}

	/// The value of the let expression
	pub fn in_expression(&self) -> Expression<'tree> {
		child_with_field_name(self, "in")
	}
}

ast_node!(
	/// Case pattern match
	Case,
	expression,
	cases,
);

impl<'tree> Case<'tree> {
	/// The expression being matched
	pub fn expression(&self) -> Expression<'tree> {
		child_with_field_name(self, "expression")
	}

	/// The cases
	pub fn cases(&self) -> Children<'tree, CaseItem<'tree>> {
		children_with_field_name(self, "case")
	}
}

ast_node!(
	/// Case pattern case
	CaseItem,
	pattern,
	value
);

impl<'tree> CaseItem<'tree> {
	/// The pattern to match
	pub fn pattern(&self) -> Pattern<'tree> {
		child_with_field_name(self, "pattern")
	}

	/// The value if this case holds
	pub fn value(&self) -> Expression<'tree> {
		child_with_field_name(self, "value")
	}
}

ast_enum!(
	/// Item in a let expression
	LetItem,
	"declaration" => Declaration,
	"constraint" => Constraint
);

ast_node!(
	/// Tuple access
	TupleAccess,
	tuple,
	field
);

impl<'tree> TupleAccess<'tree> {
	/// The tuple being accessed
	pub fn tuple(&self) -> Expression<'tree> {
		child_with_field_name(self, "tuple")
	}

	/// The field being accessed
	pub fn field(&self) -> IntegerLiteral<'tree> {
		child_with_field_name(self, "field")
	}
}

ast_node!(
	/// Record access
	RecordAccess,
	record,
	field
);

impl<'tree> RecordAccess<'tree> {
	/// The record being accessed
	pub fn record(&self) -> Expression<'tree> {
		child_with_field_name(self, "record")
	}

	/// The field being accessed
	pub fn field(&self) -> Identifier<'tree> {
		child_with_field_name(self, "field")
	}
}

ast_node!(
	/// Lambda expression
	Lambda,
	return_type,
	parameters,
	body
);

impl<'tree> Lambda<'tree> {
	/// The ascribed return type if there is one
	pub fn return_type(&self) -> Option<Type<'tree>> {
		optional_child_with_field_name(self, "return_type")
	}

	/// The parameters of the function
	pub fn parameters(&self) -> Children<'tree, Parameter<'tree>> {
		children_with_field_name(self, "parameter")
	}

	/// The body of the function
	pub fn body(&self) -> Expression<'tree> {
		child_with_field_name(self, "body")
	}
}

/// Pretty print an identifier.
///
/// Either returns the string as is, if it is already a valid identifier,
/// otherwise, encloses it in quotes.
///
/// Panics if the given name contains a quote.
pub fn pretty_print_identifier(name: &str) -> String {
	assert!(
		!name.contains('\''),
		"Identifier {} is invalid because it contains a single quote",
		name
	);
	if matches!(
		name,
		"ann"
			| "annotation"
			| "any" | "array"
			| "bool" | "case"
			| "constraint"
			| "default"
			| "diff" | "div"
			| "else" | "elseif"
			| "endif" | "enum"
			| "false" | "float"
			| "function"
			| "if" | "in"
			| "include"
			| "int" | "intersect"
			| "let" | "list"
			| "maximize"
			| "minimize"
			| "mod" | "not"
			| "of" | "op"
			| "opt" | "output"
			| "par" | "predicate"
			| "record"
			| "satisfy"
			| "set" | "solve"
			| "string"
			| "subset"
			| "superset"
			| "symdiff"
			| "test" | "then"
			| "true" | "tuple"
			| "type" | "union"
			| "var" | "where"
			| "xor"
	) {
		// Identifiers which are keywords need quoting
		return format!("'{}'", name);
	}

	let mut chars = name.chars();
	let first_char = chars.next().expect("Identifier cannot be empty");

	if !first_char.is_alphabetic() && first_char != '_' {
		// Identifiers which don't start with a letter or underscore need quoting
		return format!("'{}'", name);
	}

	for c in chars {
		if !c.is_alphanumeric() && c != '_' {
			// Non alphanumeric identifiers need quoting
			return format!("'{}'", name);
		}
	}

	name.to_owned()
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::{ast::tests::*, minizinc::pretty_print_identifier};

	#[test]
	fn test_annotated_expression() {
		check_ast(
			r#"
		x = foo :: bar :: qux;
        var 1..n: y;
		"#,
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
                        definition: AnnotatedExpression(
                            AnnotatedExpression {
                                cst_kind: "annotated_expression",
                                annotations: [
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
                                    ),
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
                                    ),
                                ],
                                expression: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                            },
                        ),
                    },
                ),
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
                                var_type: Some(
                                    Var,
                                ),
                                opt_type: None,
                                any_type: false,
                                domain: Bounded(
                                    InfixOperator(
                                        InfixOperator {
                                            cst_kind: "infix_operator",
                                            left: IntegerLiteral(
                                                IntegerLiteral {
                                                    cst_kind: "integer_literal",
                                                },
                                            ),
                                            operator: Operator {
                                                cst_kind: "..",
                                                name: "..",
                                            },
                                            right: Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                    },
                                                ),
                                            ),
                                        },
                                    ),
                                ),
                            },
                        ),
                        definition: None,
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
	fn test_identifier() {
		check_ast(
			r#"
		bool: x;
		bool: 'hello world';
		bool: Δ;
        bool: inversed = Foo^-1;
		"#,
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
                                        primitive_type: Bool,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            QuotedIdentifier(
                                QuotedIdentifier {
                                    cst_kind: "quoted_identifier",
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
                                        primitive_type: Bool,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
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
                                        primitive_type: Bool,
                                    },
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
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
                                        primitive_type: Bool,
                                    },
                                ),
                            },
                        ),
                        definition: Some(
                            Identifier(
                                InversedIdentifier(
                                    InversedIdentifier {
                                        cst_kind: "inversed_identifier",
                                        identifier: UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
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
	fn test_if_then_else() {
		check_ast(
			r#"
		x = if a then b else c endif;
		y = if a then b elseif c then d else e endif;
		z = if a then b endif;
		"#,
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
                        definition: IfThenElse(
                            IfThenElse {
                                cst_kind: "if_then_else",
                                branches: Branches {
                                    conditions: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                    results: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                },
                                else_result: Some(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
                                    ),
                                ),
                            },
                        ),
                    },
                ),
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
                        definition: IfThenElse(
                            IfThenElse {
                                cst_kind: "if_then_else",
                                branches: Branches {
                                    conditions: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                    results: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                },
                                else_result: Some(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                            },
                                        ),
                                    ),
                                ),
                            },
                        ),
                    },
                ),
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
                        definition: IfThenElse(
                            IfThenElse {
                                cst_kind: "if_then_else",
                                branches: Branches {
                                    conditions: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                    results: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ],
                                },
                                else_result: None,
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
	fn test_call() {
		check_ast(
			r#"
		x = foo();
		y = foo(one, two);
		z = foo(bar)(qux);
		"#,
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
                        definition: Call(
                            Call {
                                cst_kind: "call",
                                function: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                arguments: [],
                            },
                        ),
                    },
                ),
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
                        definition: Call(
                            Call {
                                cst_kind: "call",
                                function: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                arguments: [
                                    ArgOrParam {
                                        cst_kind: "arg_or_param",
                                        left: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Bounded(
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                            },
                                                        ),
                                                    ),
                                                ),
                                            },
                                        ),
                                        right: None,
                                        default: None,
                                    },
                                    ArgOrParam {
                                        cst_kind: "arg_or_param",
                                        left: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Bounded(
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                            },
                                                        ),
                                                    ),
                                                ),
                                            },
                                        ),
                                        right: None,
                                        default: None,
                                    },
                                ],
                            },
                        ),
                    },
                ),
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
                        definition: Call(
                            Call {
                                cst_kind: "call",
                                function: Call(
                                    Call {
                                        cst_kind: "call",
                                        function: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                        arguments: [
                                            ArgOrParam {
                                                cst_kind: "arg_or_param",
                                                left: TypeBase(
                                                    TypeBase {
                                                        cst_kind: "type_base",
                                                        var_type: None,
                                                        opt_type: None,
                                                        any_type: false,
                                                        domain: Bounded(
                                                            Identifier(
                                                                UnquotedIdentifier(
                                                                    UnquotedIdentifier {
                                                                        cst_kind: "identifier",
                                                                    },
                                                                ),
                                                            ),
                                                        ),
                                                    },
                                                ),
                                                right: None,
                                                default: None,
                                            },
                                        ],
                                    },
                                ),
                                arguments: [
                                    ArgOrParam {
                                        cst_kind: "arg_or_param",
                                        left: TypeBase(
                                            TypeBase {
                                                cst_kind: "type_base",
                                                var_type: None,
                                                opt_type: None,
                                                any_type: false,
                                                domain: Bounded(
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                            },
                                                        ),
                                                    ),
                                                ),
                                            },
                                        ),
                                        right: None,
                                        default: None,
                                    },
                                ],
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
	fn test_prefix_operator() {
		check_ast(
			"x = -a;",
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
                        definition: PrefixOperator(
                            PrefixOperator {
                                cst_kind: "prefix_operator",
                                operator: Operator {
                                    cst_kind: "-",
                                    name: "-",
                                },
                                operand: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
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
	fn test_infix_operator() {
		check_ast(
			r#"
		x = a + b;
		y = a + b * c;
		"#,
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
                        definition: InfixOperator(
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
                                right: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                            },
                        ),
                    },
                ),
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
                        definition: InfixOperator(
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
                                right: InfixOperator(
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
                                            cst_kind: "*",
                                            name: "*",
                                        },
                                        right: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    },
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
	fn test_postfix_operator() {
		check_ast(
			"x = a..;",
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
                        definition: PostfixOperator(
                            PostfixOperator {
                                cst_kind: "postfix_operator",
                                operand: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                operator: Operator {
                                    cst_kind: "..",
                                    name: "..",
                                },
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
	fn test_generator_call() {
		check_ast(
			r#"
			constraint forall (i in s) (true);
			constraint exists (i, j in s, k in t where p) (true);
			"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Constraint(
                    Constraint {
                        cst_kind: "constraint",
                        expression: GeneratorCall(
                            GeneratorCall {
                                cst_kind: "generator_call",
                                function: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                generators: [
                                    IteratorGenerator(
                                        IteratorGenerator {
                                            cst_kind: "generator",
                                            patterns: [
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                        },
                                                    ),
                                                ),
                                            ],
                                            collection: Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                    },
                                                ),
                                            ),
                                            where_clause: None,
                                        },
                                    ),
                                ],
                                template: BooleanLiteral(
                                    BooleanLiteral {
                                        cst_kind: "boolean_literal",
                                        value: true,
                                    },
                                ),
                            },
                        ),
                        annotations: [],
                    },
                ),
                Constraint(
                    Constraint {
                        cst_kind: "constraint",
                        expression: GeneratorCall(
                            GeneratorCall {
                                cst_kind: "generator_call",
                                function: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                generators: [
                                    IteratorGenerator(
                                        IteratorGenerator {
                                            cst_kind: "generator",
                                            patterns: [
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                        },
                                                    ),
                                                ),
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                        },
                                                    ),
                                                ),
                                            ],
                                            collection: Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                    },
                                                ),
                                            ),
                                            where_clause: None,
                                        },
                                    ),
                                    IteratorGenerator(
                                        IteratorGenerator {
                                            cst_kind: "generator",
                                            patterns: [
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                        },
                                                    ),
                                                ),
                                            ],
                                            collection: Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                    },
                                                ),
                                            ),
                                            where_clause: Some(
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                        },
                                                    ),
                                                ),
                                            ),
                                        },
                                    ),
                                ],
                                template: BooleanLiteral(
                                    BooleanLiteral {
                                        cst_kind: "boolean_literal",
                                        value: true,
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
	fn test_string_interpolation() {
		check_ast(
			r#"x = "foo\(y)bar";"#,
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
                        definition: StringInterpolation(
                            StringInterpolation {
                                cst_kind: "string_interpolation",
                                contents: [
                                    InterpolationItem(
                                        StringCharacters,
                                    ),
                                    InterpolationItem(
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                },
                                            ),
                                        ),
                                    ),
                                    InterpolationItem(
                                        StringCharacters,
                                    ),
                                ],
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
	fn test_let() {
		check_ast(
			r#"
			constraint let {
				var int: x;
				constraint false;
			} in true;
			"#,
			expect!([r#"
    MznModel(
        Model {
            items: [
                Constraint(
                    Constraint {
                        cst_kind: "constraint",
                        expression: Let(
                            Let {
                                cst_kind: "let_expression",
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
                                                    var_type: Some(
                                                        Var,
                                                    ),
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
                                            definition: None,
                                            annotations: [],
                                        },
                                    ),
                                    Constraint(
                                        Constraint {
                                            cst_kind: "constraint",
                                            expression: BooleanLiteral(
                                                BooleanLiteral {
                                                    cst_kind: "boolean_literal",
                                                    value: false,
                                                },
                                            ),
                                            annotations: [],
                                        },
                                    ),
                                ],
                                in_expression: BooleanLiteral(
                                    BooleanLiteral {
                                        cst_kind: "boolean_literal",
                                        value: true,
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
	fn test_case() {
		check_ast(
			r#"
			x = case a of
					Foo(b) => true,
					_ => false
				endcase;
			"#,
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
                        definition: Case(
                            Case {
                                cst_kind: "case_expression",
                                expression: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                cases: [
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Call(
                                            PatternCall {
                                                cst_kind: "pattern_call",
                                                identifier: UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                    },
                                                ),
                                                arguments: [
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                            },
                                                        ),
                                                    ),
                                                ],
                                            },
                                        ),
                                        value: BooleanLiteral(
                                            BooleanLiteral {
                                                cst_kind: "boolean_literal",
                                                value: true,
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Anonymous(
                                            Anonymous {
                                                cst_kind: "anonymous",
                                            },
                                        ),
                                        value: BooleanLiteral(
                                            BooleanLiteral {
                                                cst_kind: "boolean_literal",
                                                value: false,
                                            },
                                        ),
                                    },
                                ],
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
	fn test_tuple_access() {
		check_ast(
			"x = foo.1;",
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
                        definition: TupleAccess(
                            TupleAccess {
                                cst_kind: "tuple_access",
                                tuple: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                field: IntegerLiteral {
                                    cst_kind: "integer_literal",
                                },
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
	fn test_record_access() {
		check_ast(
			"x = foo.bar;",
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
                        definition: RecordAccess(
                            RecordAccess {
                                cst_kind: "record_access",
                                record: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
                                ),
                                field: UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                    },
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

	fn test_lambda() {
		check_ast(
			"x = lambda int: (int: x) => x;",
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
                        definition: Lambda(
                            Lambda {
                                cst_kind: "lambda",
                                return_type: Some(
                                    TypeBase(
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
                                body: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                        },
                                    ),
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
	fn pretty_print_ident() {
		assert_eq!(pretty_print_identifier("x"), "x");
		assert_eq!(pretty_print_identifier("-"), "'-'");
		assert_eq!(pretty_print_identifier("a b"), "'a b'");
		assert_eq!(pretty_print_identifier("😃"), "'😃'");
		assert_eq!(pretty_print_identifier("Δ"), "Δ");
		assert_eq!(pretty_print_identifier("123"), "'123'");
		assert_eq!(pretty_print_identifier("1E24"), "'1E24'");
	}
}
