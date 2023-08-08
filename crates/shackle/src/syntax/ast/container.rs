//! AST Representation for containers

use super::{helpers::*, Identifier};
use super::{AstNode, Children, Expression, Pattern};

ast_node!(
	/// Tuple literal
	TupleLiteral,
	members,
);

impl TupleLiteral {
	/// Get the values in this tuple literal
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Record literal
	RecordLiteral,
	members,
);

impl RecordLiteral {
	/// Get the values in this record literal
	pub fn members(&self) -> Children<'_, RecordLiteralMember> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Record literal key-value pair
	RecordLiteralMember,
	name,
	value
);

impl RecordLiteralMember {
	/// Get the name of this member
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the value of this member
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}
}

ast_node!(
	/// Set literal
	SetLiteral,
	members
);

impl SetLiteral {
	/// Get the values in this set literal
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array literal
	ArrayLiteral,
	members
);

impl ArrayLiteral {
	/// Get the members of this array literal
	pub fn members(&self) -> Children<'_, ArrayLiteralMember> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array literal member (indices if present and value)
	ArrayLiteralMember,
	indices,
	value
);

impl ArrayLiteralMember {
	/// Get the indices for this member
	pub fn indices(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// Get the value of this member
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}
}

ast_node!(
	/// 2D array literal
	ArrayLiteral2D,
	column_indices,
	rows
);

impl ArrayLiteral2D {
	/// Get the column indices if any
	pub fn column_indices(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "column_index")
	}

	/// Get the rows in this 2D array literal
	pub fn rows(&self) -> Children<'_, ArrayLiteral2DRow> {
		children_with_field_name(self, "row")
	}
}

ast_node!(
	/// 2D array literal row
	ArrayLiteral2DRow,
	index,
	members
);

impl ArrayLiteral2DRow {
	/// Get the row index if present
	pub fn index(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// Get the values in this 2D array literal row
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array access
	ArrayAccess,
	collection,
	indices
);

impl ArrayAccess {
	/// The array being indexed
	pub fn collection(&self) -> Expression {
		child_with_field_name(self, "collection")
	}

	/// Get the indices
	pub fn indices(&self) -> Children<'_, ArrayIndex> {
		children_with_field_name(self, "index")
	}
}

ast_enum!(
	/// Array index (could be `..` or an expression)
	ArrayIndex,
	".." | "<.." | "<..<" | "..<" => IndexSlice,
	_ => Expression
);

ast_node!(
	/// Array index slice
	IndexSlice,
	operator,
);

impl IndexSlice {
	/// Get the operator
	pub fn operator(&self) -> &str {
		let node = self.cst_node().as_ref();
		node.kind()
	}
}

ast_node!(
	/// Array comprehension
	ArrayComprehension,
	indices,
	template,
	generators
);

impl ArrayComprehension {
	/// The indices for the body of this comprehension
	pub fn indices(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// The body of this comprehension
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}

	/// The generators for this comprehension
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}
}

ast_node!(
	/// Set comprehension
	SetComprehension,
	template,
	generators
);

impl SetComprehension {
	/// The body of this comprehension
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}
	/// The generators for this comprehension
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}
}

ast_enum!(
	/// Generator for a comprehension
	Generator,
	"generator" => IteratorGenerator,
	"assignment_generator" => AssignmentGenerator
);

ast_node!(
	/// Generator for a comprehension
	IteratorGenerator,
	patterns,
	collection,
	where_clause
);

impl IteratorGenerator {
	/// Patterns (variable names)
	pub fn patterns(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "name")
	}

	/// Expression being iterated over
	pub fn collection(&self) -> Expression {
		child_with_field_name(self, "collection")
	}

	/// Where clause constraining iteration
	pub fn where_clause(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "where")
	}
}

ast_node!(
	/// Assignment generator for a comprehension
	AssignmentGenerator,
	pattern,
	value,
	where_clause
);

impl AssignmentGenerator {
	/// Pattern (variable name)
	pub fn pattern(&self) -> Pattern {
		child_with_field_name(self, "name")
	}

	/// Expression being iterated over
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}

	/// Where clause constraining iteration
	pub fn where_clause(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "where")
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use expect_test::expect;

	#[test]
	fn test_tuple_literal() {
		check_ast(
			r#"
		x = (1, 2);
		y = (1, (2, 3));
		"#,
			expect!([r#"
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
                    definition: TupleLiteral(
                        TupleLiteral {
                            cst_kind: "tuple_literal",
                            members: [
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            1,
                                        ),
                                    },
                                ),
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            2,
                                        ),
                                    },
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
                                name: "y",
                            },
                        ),
                    ),
                    definition: TupleLiteral(
                        TupleLiteral {
                            cst_kind: "tuple_literal",
                            members: [
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            1,
                                        ),
                                    },
                                ),
                                TupleLiteral(
                                    TupleLiteral {
                                        cst_kind: "tuple_literal",
                                        members: [
                                            IntegerLiteral(
                                                IntegerLiteral {
                                                    cst_kind: "integer_literal",
                                                    value: Ok(
                                                        2,
                                                    ),
                                                },
                                            ),
                                            IntegerLiteral(
                                                IntegerLiteral {
                                                    cst_kind: "integer_literal",
                                                    value: Ok(
                                                        3,
                                                    ),
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_record_literal() {
		check_ast(
			r#"
		x = (a: 1, b: 2);
		y = (a: 1, b: (c: 2, d: 3));
		"#,
			expect!([r#"
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
                    definition: RecordLiteral(
                        RecordLiteral {
                            cst_kind: "record_literal",
                            members: [
                                RecordLiteralMember {
                                    cst_kind: "record_member",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "a",
                                        },
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
                                RecordLiteralMember {
                                    cst_kind: "record_member",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "b",
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
                                name: "y",
                            },
                        ),
                    ),
                    definition: RecordLiteral(
                        RecordLiteral {
                            cst_kind: "record_literal",
                            members: [
                                RecordLiteralMember {
                                    cst_kind: "record_member",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "a",
                                        },
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
                                RecordLiteralMember {
                                    cst_kind: "record_member",
                                    name: UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "b",
                                        },
                                    ),
                                    value: RecordLiteral(
                                        RecordLiteral {
                                            cst_kind: "record_literal",
                                            members: [
                                                RecordLiteralMember {
                                                    cst_kind: "record_member",
                                                    name: UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "c",
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
                                                RecordLiteralMember {
                                                    cst_kind: "record_member",
                                                    name: UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "d",
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
                                            ],
                                        },
                                    ),
                                },
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_set_literal() {
		check_ast(
			"x = {1, 2};",
			expect!([r#"
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
                    definition: SetLiteral(
                        SetLiteral {
                            cst_kind: "set_literal",
                            members: [
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            1,
                                        ),
                                    },
                                ),
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            2,
                                        ),
                                    },
                                ),
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		)
	}

	#[test]
	fn test_array_literal() {
		check_ast(
			r#"
		x = [1, 3];
		y = [2: 1, 3];
		z = [0: 1, 1: 3];
		w = [(1, 1): 1, (1, 2): 3];
		"#,
			expect!([r#"
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
                    definition: ArrayLiteral(
                        ArrayLiteral {
                            cst_kind: "array_literal",
                            members: [
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: None,
                                    value: IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                1,
                                            ),
                                        },
                                    ),
                                },
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: None,
                                    value: IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                3,
                                            ),
                                        },
                                    ),
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
                                name: "y",
                            },
                        ),
                    ),
                    definition: ArrayLiteral(
                        ArrayLiteral {
                            cst_kind: "array_literal",
                            members: [
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: Some(
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
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
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: None,
                                    value: IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                3,
                                            ),
                                        },
                                    ),
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
                                name: "z",
                            },
                        ),
                    ),
                    definition: ArrayLiteral(
                        ArrayLiteral {
                            cst_kind: "array_literal",
                            members: [
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: Some(
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    0,
                                                ),
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
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: Some(
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
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
                                name: "w",
                            },
                        ),
                    ),
                    definition: ArrayLiteral(
                        ArrayLiteral {
                            cst_kind: "array_literal",
                            members: [
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: Some(
                                        TupleLiteral(
                                            TupleLiteral {
                                                cst_kind: "tuple_literal",
                                                members: [
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                1,
                                                            ),
                                                        },
                                                    ),
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                1,
                                                            ),
                                                        },
                                                    ),
                                                ],
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
                                ArrayLiteralMember {
                                    cst_kind: "array_literal_member",
                                    indices: Some(
                                        TupleLiteral(
                                            TupleLiteral {
                                                cst_kind: "tuple_literal",
                                                members: [
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                1,
                                                            ),
                                                        },
                                                    ),
                                                    IntegerLiteral(
                                                        IntegerLiteral {
                                                            cst_kind: "integer_literal",
                                                            value: Ok(
                                                                2,
                                                            ),
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
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
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_2d_array_literal() {
		check_ast(
			r#"
		x = [| 1, 2
		     | 3, 4 |];
		y = [| 1: 2:
		     | 1, 2 |];
		z = [|    1: 2: |
		     | 1: 1, 2 |];
		w = [| 1: 1, 2 |];
		"#,
			expect!([r#"
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
                    definition: ArrayLiteral2D(
                        ArrayLiteral2D {
                            cst_kind: "array_literal_2d",
                            column_indices: [],
                            rows: [
                                ArrayLiteral2DRow {
                                    cst_kind: "array_literal_2d_row",
                                    index: None,
                                    members: [
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
                                            },
                                        ),
                                    ],
                                },
                                ArrayLiteral2DRow {
                                    cst_kind: "array_literal_2d_row",
                                    index: None,
                                    members: [
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    3,
                                                ),
                                            },
                                        ),
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    4,
                                                ),
                                            },
                                        ),
                                    ],
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
                                name: "y",
                            },
                        ),
                    ),
                    definition: ArrayLiteral2D(
                        ArrayLiteral2D {
                            cst_kind: "array_literal_2d",
                            column_indices: [
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            1,
                                        ),
                                    },
                                ),
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            2,
                                        ),
                                    },
                                ),
                            ],
                            rows: [
                                ArrayLiteral2DRow {
                                    cst_kind: "array_literal_2d_row",
                                    index: None,
                                    members: [
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
                                            },
                                        ),
                                    ],
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
                                name: "z",
                            },
                        ),
                    ),
                    definition: ArrayLiteral2D(
                        ArrayLiteral2D {
                            cst_kind: "array_literal_2d",
                            column_indices: [
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            1,
                                        ),
                                    },
                                ),
                                IntegerLiteral(
                                    IntegerLiteral {
                                        cst_kind: "integer_literal",
                                        value: Ok(
                                            2,
                                        ),
                                    },
                                ),
                            ],
                            rows: [
                                ArrayLiteral2DRow {
                                    cst_kind: "array_literal_2d_row",
                                    index: Some(
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                    ),
                                    members: [
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
                                            },
                                        ),
                                    ],
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
                                name: "w",
                            },
                        ),
                    ),
                    definition: ArrayLiteral2D(
                        ArrayLiteral2D {
                            cst_kind: "array_literal_2d",
                            column_indices: [],
                            rows: [
                                ArrayLiteral2DRow {
                                    cst_kind: "array_literal_2d_row",
                                    index: Some(
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                    ),
                                    members: [
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    1,
                                                ),
                                            },
                                        ),
                                        IntegerLiteral(
                                            IntegerLiteral {
                                                cst_kind: "integer_literal",
                                                value: Ok(
                                                    2,
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_array_access() {
		check_ast(
			r#"
		x = foo[1];
		y = foo[1, 2];
		z = foo[1, .., 3..];
		"#,
			expect!([r#"
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
                    definition: ArrayAccess(
                        ArrayAccess {
                            cst_kind: "indexed_access",
                            collection: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "foo",
                                    },
                                ),
                            ),
                            indices: [
                                Expression(
                                    IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                1,
                                            ),
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
                                name: "y",
                            },
                        ),
                    ),
                    definition: ArrayAccess(
                        ArrayAccess {
                            cst_kind: "indexed_access",
                            collection: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "foo",
                                    },
                                ),
                            ),
                            indices: [
                                Expression(
                                    IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                1,
                                            ),
                                        },
                                    ),
                                ),
                                Expression(
                                    IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                2,
                                            ),
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
                    definition: ArrayAccess(
                        ArrayAccess {
                            cst_kind: "indexed_access",
                            collection: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "foo",
                                    },
                                ),
                            ),
                            indices: [
                                Expression(
                                    IntegerLiteral(
                                        IntegerLiteral {
                                            cst_kind: "integer_literal",
                                            value: Ok(
                                                1,
                                            ),
                                        },
                                    ),
                                ),
                                IndexSlice(
                                    IndexSlice {
                                        cst_kind: "..",
                                        operator: "..",
                                    },
                                ),
                                Expression(
                                    PostfixOperator(
                                        PostfixOperator {
                                            cst_kind: "postfix_operator",
                                            operand: IntegerLiteral(
                                                IntegerLiteral {
                                                    cst_kind: "integer_literal",
                                                    value: Ok(
                                                        3,
                                                    ),
                                                },
                                            ),
                                            operator: Operator {
                                                cst_kind: "..",
                                                name: "..",
                                            },
                                        },
                                    ),
                                ),
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_array_comprehension() {
		check_ast(
			r#"
		x = [1 | i in s];
		y = [i: v | i in 1..3, j in s where i < j];
		z = [(i, j): v | i, j in s]
		a = [j | i in s, j = i + 1];
		"#,
			expect!([r#"
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
                    definition: ArrayComprehension(
                        ArrayComprehension {
                            cst_kind: "array_comprehension",
                            indices: None,
                            template: IntegerLiteral(
                                IntegerLiteral {
                                    cst_kind: "integer_literal",
                                    value: Ok(
                                        1,
                                    ),
                                },
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
                    definition: ArrayComprehension(
                        ArrayComprehension {
                            cst_kind: "array_comprehension",
                            indices: Some(
                                Identifier(
                                    UnquotedIdentifier(
                                        UnquotedIdentifier {
                                            cst_kind: "identifier",
                                            name: "i",
                                        },
                                    ),
                                ),
                            ),
                            template: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "v",
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
                                        collection: InfixOperator(
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
                                                right: IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: Ok(
                                                            3,
                                                        ),
                                                    },
                                                ),
                                            },
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
                                        where_clause: Some(
                                            InfixOperator(
                                                InfixOperator {
                                                    cst_kind: "infix_operator",
                                                    left: Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "i",
                                                            },
                                                        ),
                                                    ),
                                                    operator: Operator {
                                                        cst_kind: "<",
                                                        name: "<",
                                                    },
                                                    right: Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "j",
                                                            },
                                                        ),
                                                    ),
                                                },
                                            ),
                                        ),
                                    },
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
                    definition: InfixOperator(
                        InfixOperator {
                            cst_kind: "infix_operator",
                            left: ArrayComprehension(
                                ArrayComprehension {
                                    cst_kind: "array_comprehension",
                                    indices: Some(
                                        TupleLiteral(
                                            TupleLiteral {
                                                cst_kind: "tuple_literal",
                                                members: [
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
                                            },
                                        ),
                                    ),
                                    template: Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "v",
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
                                    ],
                                },
                            ),
                            operator: Operator {
                                cst_kind: "=",
                                name: "=",
                            },
                            right: ArrayComprehension(
                                ArrayComprehension {
                                    cst_kind: "array_comprehension",
                                    indices: None,
                                    template: Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "j",
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
                                        AssignmentGenerator(
                                            AssignmentGenerator {
                                                cst_kind: "assignment_generator",
                                                pattern: Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "j",
                                                        },
                                                    ),
                                                ),
                                                value: InfixOperator(
                                                    InfixOperator {
                                                        cst_kind: "infix_operator",
                                                        left: Identifier(
                                                            UnquotedIdentifier(
                                                                UnquotedIdentifier {
                                                                    cst_kind: "identifier",
                                                                    name: "i",
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
                                                where_clause: None,
                                            },
                                        ),
                                    ],
                                },
                            ),
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}

	#[test]
	fn test_set_comprehension() {
		check_ast(
			r#"
		x = {v | i in s};
		y = {v | i in 1..3, j in s where i < j};
		z = {v | i, j in s};
		a = {j | i in s, j = i + 1};
		"#,
			expect!([r#"
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
                    definition: SetComprehension(
                        SetComprehension {
                            cst_kind: "set_comprehension",
                            template: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "v",
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
                    definition: SetComprehension(
                        SetComprehension {
                            cst_kind: "set_comprehension",
                            template: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "v",
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
                                        collection: InfixOperator(
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
                                                right: IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: Ok(
                                                            3,
                                                        ),
                                                    },
                                                ),
                                            },
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
                                        where_clause: Some(
                                            InfixOperator(
                                                InfixOperator {
                                                    cst_kind: "infix_operator",
                                                    left: Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "i",
                                                            },
                                                        ),
                                                    ),
                                                    operator: Operator {
                                                        cst_kind: "<",
                                                        name: "<",
                                                    },
                                                    right: Identifier(
                                                        UnquotedIdentifier(
                                                            UnquotedIdentifier {
                                                                cst_kind: "identifier",
                                                                name: "j",
                                                            },
                                                        ),
                                                    ),
                                                },
                                            ),
                                        ),
                                    },
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
                    definition: SetComprehension(
                        SetComprehension {
                            cst_kind: "set_comprehension",
                            template: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "v",
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
                                name: "a",
                            },
                        ),
                    ),
                    definition: SetComprehension(
                        SetComprehension {
                            cst_kind: "set_comprehension",
                            template: Identifier(
                                UnquotedIdentifier(
                                    UnquotedIdentifier {
                                        cst_kind: "identifier",
                                        name: "j",
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
                                AssignmentGenerator(
                                    AssignmentGenerator {
                                        cst_kind: "assignment_generator",
                                        pattern: Identifier(
                                            UnquotedIdentifier(
                                                UnquotedIdentifier {
                                                    cst_kind: "identifier",
                                                    name: "j",
                                                },
                                            ),
                                        ),
                                        value: InfixOperator(
                                            InfixOperator {
                                                cst_kind: "infix_operator",
                                                left: Identifier(
                                                    UnquotedIdentifier(
                                                        UnquotedIdentifier {
                                                            cst_kind: "identifier",
                                                            name: "i",
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
                                        where_clause: None,
                                    },
                                ),
                            ],
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);
	}
}
