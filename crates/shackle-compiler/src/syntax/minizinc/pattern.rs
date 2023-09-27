//! AST representation of destructuring patterns

use super::{
	Absent, Anonymous, BooleanLiteral, Children, FloatLiteral, Identifier, Infinity,
	IntegerLiteral, StringLiteral,
};
use crate::syntax::ast::{
	ast_enum, ast_node, child_with_field_name, children_with_field_name, AstNode,
};

ast_enum!(
	/// A pattern for (future) destructuring.
	Pattern,
	"identifier" | "quoted_identifier" => Identifier,
	"anonymous" => Anonymous,
	"absent" => Absent,
	"boolean_literal" => BooleanLiteral,
	"string_literal" => StringLiteral,
	"pattern_numeric_literal" => PatternNumericLiteral,
	"pattern_call" => Call(PatternCall),
	"pattern_tuple" => Tuple(PatternTuple),
	"pattern_record" => Record(PatternRecord)
);

ast_node!(
	/// A pattern that matches a numeric literal
	///
	/// Note that we have to deal with possible negation here because numeric
	/// literals are always positive.
	PatternNumericLiteral,
	negated,
	value
);

impl PatternNumericLiteral {
	/// Whether this literal is negative
	pub fn negated(&self) -> bool {
		self.cst_node()
			.as_ref()
			.child_by_field_name("negative")
			.is_some()
	}

	/// The underlying literal
	pub fn value(&self) -> NumericLiteral {
		child_with_field_name(self, "value")
	}
}

ast_enum!(
	/// A numeric literal
	NumericLiteral,
	"integer_literal" => IntegerLiteral,
	"float_literal" => FloatLiteral,
	"infinity" => Infinity
);

ast_node!(
	/// A pattern that matches a call
	PatternCall,
	identifier,
	arguments
);

impl PatternCall {
	/// Get the name of the function
	pub fn identifier(&self) -> Identifier {
		child_with_field_name(self, "identifier")
	}
	/// Get the arguments to this call pattern
	pub fn arguments(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "argument")
	}
}

ast_node!(
	/// A pattern that matches a tuple
	PatternTuple,
	fields
);

impl PatternTuple {
	/// Get the fields of this tuple pattern
	pub fn fields(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// A pattern that matches a record
	PatternRecord,
	fields
);

impl PatternRecord {
	/// Get the fields of this tuple pattern
	pub fn fields(&self) -> Children<'_, PatternRecordField> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Field in a record pattern
	PatternRecordField,
	name,
	value
);

impl PatternRecordField {
	/// The field name being matched
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// The pattern of the field being matched
	pub fn value(&self) -> Pattern {
		child_with_field_name(self, "value")
	}
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::syntax::ast::test::*;

	#[test]
	fn test_patterns() {
		check_ast(
			r#"
		any: (a: (p, q), b: r) = foo;
		any: v = case x of
			A => 1,
			B(x) => 2,
			C(x, D(y)) => 3,
			true => 4,
			123 => 5,
			-5.5 => 6,
			infinity => 7,
			"foo" => 8,
			<> => 9,
			_ => 10,
		endcase;
		"#,
			expect!([r#"
MznModel(
    Model {
        items: [
            Declaration(
                Declaration {
                    cst_kind: "declaration",
                    pattern: Record(
                        PatternRecord {
                            cst_kind: "pattern_record",
                            fields: [
                                PatternRecordField {
                                    cst_kind: "pattern_record_field",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "a",
                                        },
                                    ),
                                    value: Tuple(
                                        PatternTuple {
                                            cst_kind: "pattern_tuple",
                                            fields: [
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "p",
                                                        },
                                                    ),
                                                ),
                                                Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "q",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                },
                                PatternRecordField {
                                    cst_kind: "pattern_record_field",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "b",
                                        },
                                    ),
                                    value: Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "r",
                                            },
                                        ),
                                    ),
                                },
                            ],
                        },
                    ),
                    declared_type: AnyType(
                        AnyType {
                            cst_kind: "any_type",
                        },
                    ),
                    definition: Some(
                        Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "foo",
                                },
                            ),
                        ),
                    ),
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
                                name: "v",
                            },
                        ),
                    ),
                    declared_type: AnyType(
                        AnyType {
                            cst_kind: "any_type",
                        },
                    ),
                    definition: Some(
                        Case(
                            Case {
                                cst_kind: "case_expression",
                                expression: Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "x",
                                        },
                                    ),
                                ),
                                cases: [
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "A",
                                                },
                                            ),
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Call(
                                            PatternCall {
                                                cst_kind: "pattern_call",
                                                identifier: UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "B",
                                                    },
                                                ),
                                                arguments: [
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "x",
                                                            },
                                                        ),
                                                    ),
                                                ],
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Call(
                                            PatternCall {
                                                cst_kind: "pattern_call",
                                                identifier: UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "C",
                                                    },
                                                ),
                                                arguments: [
                                                    Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "x",
                                                            },
                                                        ),
                                                    ),
                                                    Call(
                                                        PatternCall {
                                                            cst_kind: "pattern_call",
                                                            identifier: UnquotedIdentifier(
                                                                UnquotedIdentifier {
                                                                    cst_kind: "identifier",
                                                                    name: "D",
                                                                },
                                                            ),
                                                            arguments: [
                                                                Identifier(
                                                                    UnquotedIdentifier(
                                                                        UnquotedIdentifier {
                                                                            cst_kind: "identifier",
                                                                            name: "y",
                                                                        },
                                                                    ),
                                                                ),
                                                            ],
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    3,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: BooleanLiteral(
                                            BooleanLiteral {
                                                cst_kind: "boolean_literal",
                                                value: true,
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    4,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: PatternNumericLiteral(
                                            PatternNumericLiteral {
                                                cst_kind: "pattern_numeric_literal",
                                                negated: false,
                                                value: IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: Ok(
                                                            123,
                                                        ),
                                                    },
                                                ),
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    5,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: PatternNumericLiteral(
                                            PatternNumericLiteral {
                                                cst_kind: "pattern_numeric_literal",
                                                negated: true,
                                                value: FloatLiteral(
                                                    FloatLiteral {
                                                        cst_kind: "float_literal",
                                                        value: Ok(
                                                            5.5,
                                                        ),
                                                    },
                                                ),
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    6,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: PatternNumericLiteral(
                                            PatternNumericLiteral {
                                                cst_kind: "pattern_numeric_literal",
                                                negated: false,
                                                value: Infinity(
                                                    Infinity {
                                                        cst_kind: "infinity",
                                                    },
                                                ),
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    7,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: StringLiteral(
                                            StringLiteral {
                                                cst_kind: "string_literal",
                                                value: "foo",
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    8,
                                                ),
                                            },
                                        ),
                                    },
                                    CaseItem {
                                        cst_kind: "case_expression_case",
                                        pattern: Absent(
                                            Absent {
                                                cst_kind: "absent",
                                            },
                                        ),
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    9,
                                                ),
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
                                        value: IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    10,
                                                ),
                                            },
                                        ),
                                    },
                                ],
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
}
