//! AST representation of expressions

use std::borrow::Cow;

use super::{
	Absent, ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, BooleanLiteral,
	Children, Constraint, Declaration, FloatLiteral, Generator, Infinity, IntegerLiteral,
	Parameter, Pattern, RecordLiteral, SetComprehension, SetLiteral, StringLiteral, TupleLiteral,
	Type,
};
use crate::syntax::{
	ast::{
		ast_enum, ast_node, child_with_field_name, children_with_field_name, decode_string,
		optional_child_with_field_name, AstNode,
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

ast_node!(
	/// An annotated expression
	AnnotatedExpression,
	annotations,
	expression
);

impl AnnotatedExpression {
	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
	/// The expression which was annotated
	pub fn expression(&self) -> Expression {
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

impl Identifier {
	/// Get the name of this identifier
	pub fn name(&self) -> Cow<str> {
		match *self {
			Identifier::QuotedIdentifier(ref i) => Cow::from(i.name()),
			Identifier::UnquotedIdentifier(ref i) => Cow::from(i.name()),
			Identifier::InversedIdentifier(ref i) => Cow::from(i.name()),
		}
	}
}

ast_node!(
	/// Identifier
	UnquotedIdentifier,
	name
);

impl UnquotedIdentifier {
	/// Get the name of this identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Quoted identifier
	QuotedIdentifier,
	name
);

impl QuotedIdentifier {
	/// Get the name of this identifier without the enclosing quotes
	pub fn name(&self) -> &str {
		let text = self.cst_text();
		&text[1..text.len() - 1]
	}
}

ast_node!(
	/// Inversed identifier Foo^-1
	InversedIdentifier,
	identifier,
	name
);

impl InversedIdentifier {
	/// Get the identifier (without the ^-1)
	pub fn identifier(&self) -> Identifier {
		child_with_field_name(self, "identifier")
	}
	/// Get the name of this identifier ending with ⁻¹ without any enclosing quotes
	pub fn name(&self) -> String {
		format!("{}⁻¹", self.identifier().name())
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

impl IfThenElse {
	/// If-then and elseif-then pairs
	pub fn branches(&self) -> Branches<'_> {
		Branches {
			conditions: children_with_field_name(self, "condition"),
			results: children_with_field_name(self, "result"),
		}
	}

	/// Else expression
	pub fn else_result(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "else")
	}
}

/// Iterator over the branches of an `IfThenElse`

#[derive(Clone, Debug)]
pub struct Branches<'a> {
	conditions: Children<'a, Expression>,
	results: Children<'a, Expression>,
}

impl Iterator for Branches<'_> {
	type Item = Branch;
	fn next(&mut self) -> Option<Branch> {
		match (self.conditions.next(), self.results.next()) {
			(Some(condition), Some(result)) => Some(Branch { condition, result }),
			(None, None) => None,
			_ => unreachable!("Mismatch in size of conditions and results for if-then-else"),
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Branch {
	/// The boolean condition
	pub condition: Expression,
	/// The result if the condition holds
	pub result: Expression,
}

ast_node!(
	/// Function call
	Call,
	function,
	arguments
);

impl Call {
	/// Get the expression being called
	/// Will usually be an identifier
	pub fn function(&self) -> Expression {
		child_with_field_name(self, "function")
	}

	/// Get the call arguments.
	pub fn arguments(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "argument")
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
	/// Prefix (unary) operator
	PrefixOperator,
	operator,
	operand
);

impl PrefixOperator {
	/// Get the operator
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the operand
	pub fn operand(&self) -> Expression {
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

impl InfixOperator {
	/// Get the left hand side
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the left hand side
	pub fn left(&self) -> Expression {
		child_with_field_name(self, "left")
	}

	/// Get the left hand side
	pub fn right(&self) -> Expression {
		child_with_field_name(self, "right")
	}
}

ast_node!(
	/// Postfix operator
	PostfixOperator,
	operand,
	operator,
);

impl PostfixOperator {
	/// Get the operator
	pub fn operator(&self) -> Operator {
		child_with_field_name(self, "operator")
	}

	/// Get the operand
	pub fn operand(&self) -> Expression {
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

impl GeneratorCall {
	/// Get the expression being called
	/// Should always be an `Identifier` for now but for lambdas would be something else
	pub fn function(&self) -> Expression {
		child_with_field_name(self, "function")
	}

	/// The generators for this call
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}

	/// The body of this call
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}
}

ast_node!(
	/// String interpolation
	StringInterpolation,
	contents
);

impl StringInterpolation {
	/// Get the contents of this string interpolation
	pub fn contents(&self) -> Children<'_, InterpolationItem> {
		children_with_field_name(self, "item")
	}
}

/// An element in a string interpolation
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum InterpolationItem {
	/// String content
	String(String),
	/// An expression
	Expression(Expression),
}

impl InterpolationItem {
	/// Return whether this interpolation item is a string
	pub fn is_string(&self) -> bool {
		matches!(*self, InterpolationItem::String(_))
	}
	/// Return whether this interpolation item is an expression
	pub fn is_expression(&self) -> bool {
		matches!(*self, InterpolationItem::Expression(_))
	}

	/// Get the string if this is one
	pub fn string(&self) -> Option<&str> {
		match *self {
			InterpolationItem::String(ref s) => Some(s),
			_ => None,
		}
	}

	/// Get the expression if this is one
	pub fn expression(&self) -> Option<&Expression> {
		match *self {
			InterpolationItem::Expression(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<CstNode> for InterpolationItem {
	fn from(syntax: CstNode) -> Self {
		let tree = syntax.cst();
		let c = syntax.as_ref();
		match c.kind() {
			"string" => InterpolationItem::String(decode_string(&tree.node(*c))),
			"expression" => {
				InterpolationItem::Expression(Expression::new(tree.node(c.child(0).unwrap())))
			}
			_ => unreachable!(),
		}
	}
}

ast_node!(
	/// Let expression
	Let,
	items,
	in_expression
);

impl Let {
	/// The items of the let expression
	pub fn items(&self) -> Children<'_, LetItem> {
		children_with_field_name(self, "item")
	}

	/// The value of the let expression
	pub fn in_expression(&self) -> Expression {
		child_with_field_name(self, "in")
	}
}

ast_node!(
	/// Case pattern match
	Case,
	expression,
	cases,
);

impl Case {
	/// The expression being matched
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}

	/// The cases
	pub fn cases(&self) -> Children<'_, CaseItem> {
		children_with_field_name(self, "case")
	}
}

ast_node!(
	/// Case pattern case
	CaseItem,
	pattern,
	value
);

impl CaseItem {
	/// The pattern to match
	pub fn pattern(&self) -> Pattern {
		child_with_field_name(self, "pattern")
	}

	/// The value if this case holds
	pub fn value(&self) -> Expression {
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

impl TupleAccess {
	/// The tuple being accessed
	pub fn tuple(&self) -> Expression {
		child_with_field_name(self, "tuple")
	}

	/// The field being accessed
	pub fn field(&self) -> IntegerLiteral {
		child_with_field_name(self, "field")
	}
}

ast_node!(
	/// Record access
	RecordAccess,
	record,
	field
);

impl RecordAccess {
	/// The record being accessed
	pub fn record(&self) -> Expression {
		child_with_field_name(self, "record")
	}

	/// The field being accessed
	pub fn field(&self) -> Identifier {
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

impl Lambda {
	/// The ascribed return type if there is one
	pub fn return_type(&self) -> Option<Type> {
		optional_child_with_field_name(self, "return_type")
	}

	/// The parameters of the function
	pub fn parameters(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}

	/// The body of the function
	pub fn body(&self) -> Expression {
		child_with_field_name(self, "body")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::*;

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
                                    name: "x",
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
                                                name: "bar",
                                            },
                                        ),
                                    ),
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "qux",
                                            },
                                        ),
                                    ),
                                ],
                                expression: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "foo",
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
                                    name: "y",
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
                                                    value: Ok(
                                                        1,
                                                    ),
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
                                                        name: "n",
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
		bool: ✔️;
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
                                name: "hello world",
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
                                name: "✔\u{fe0f}",
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
                                name: "x",
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
                                                name: "a",
                                            },
                                        ),
                                    ),
                                ],
                                results: [
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
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
                                            name: "c",
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
                                name: "y",
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
                                                name: "a",
                                            },
                                        ),
                                    ),
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "c",
                                            },
                                        ),
                                    ),
                                ],
                                results: [
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
                                            },
                                        ),
                                    ),
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "d",
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
                                            name: "e",
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
                                name: "z",
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
                                                name: "a",
                                            },
                                        ),
                                    ),
                                ],
                                results: [
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
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
                                name: "x",
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
                                        name: "foo",
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
                                name: "y",
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
                                        name: "foo",
                                    },
                                ),
                            ),
                            arguments: [
                                Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "one",
                                        },
                                    ),
                                ),
                                Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "two",
                                        },
                                    ),
                                ),
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
                                name: "z",
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
                                                name: "foo",
                                            },
                                        ),
                                    ),
                                    arguments: [
                                        Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "bar",
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
                            arguments: [
                                Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "qux",
                                        },
                                    ),
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
                                name: "x",
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
                                        name: "a",
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
                                name: "x",
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
                                        name: "a",
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
                                        name: "b",
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
                                name: "y",
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
                                        name: "a",
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
                                                name: "b",
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
                                                name: "c",
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
                                name: "x",
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
                                        name: "a",
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
                                        name: "forall",
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
                                                        name: "i",
                                                    },
                                                ),
                                            ),
                                        ],
                                        collection: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "s",
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
                                        name: "exists",
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
                                                        name: "i",
                                                    },
                                                ),
                                            ),
                                            Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "j",
                                                    },
                                                ),
                                            ),
                                        ],
                                        collection: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "s",
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
                                                        name: "k",
                                                    },
                                                ),
                                            ),
                                        ],
                                        collection: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "t",
                                                },
                                            ),
                                        ),
                                        where_clause: Some(
                                            Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "p",
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
                                name: "x",
                            },
                        ),
                    ),
                    definition: StringInterpolation(
                        StringInterpolation {
                            cst_kind: "string_interpolation",
                            contents: [
                                String(
                                    "foo",
                                ),
                                Expression(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "y",
                                            },
                                        ),
                                    ),
                                ),
                                String(
                                    "bar",
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
                                                    name: "x",
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
                                name: "x",
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
                                        name: "a",
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
                                                    name: "Foo",
                                                },
                                            ),
                                            arguments: [
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "b",
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
                                name: "x",
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
                                        name: "foo",
                                    },
                                ),
                            ),
                            field: IntegerLiteral {
                                cst_kind: "integer_literal",
                                value: Ok(
                                    1,
                                ),
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
                                name: "x",
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
                                        name: "foo",
                                    },
                                ),
                            ),
                            field: UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "bar",
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
                                name: "x",
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
                                                    name: "x",
                                                },
                                            ),
                                        ),
                                    ),
                                    annotations: [],
                                },
                            ],
                            body: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "x",
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
}
