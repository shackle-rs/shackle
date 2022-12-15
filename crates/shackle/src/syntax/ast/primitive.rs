//! AST representation of primitive values

use super::AstNode;

use super::helpers::*;

ast_node!(
	/// Integer literal
	IntegerLiteral,
	value
);

impl IntegerLiteral {
	/// Get the value of this integer literal
	pub fn value(&self) -> i64 {
		self.cst_text().parse().unwrap()
	}
}

ast_node!(
	/// Float literal
	FloatLiteral,
	value
);

impl FloatLiteral {
	/// Get the value of this float literal
	pub fn value(&self) -> f64 {
		self.cst_text().parse().unwrap()
	}
}

ast_node!(
	/// Boolean literal
	BooleanLiteral,
	value
);

impl BooleanLiteral {
	/// Get the value of this boolean literal
	pub fn value(&self) -> bool {
		match self.cst_text() {
			"true" => true,
			"false" => false,
			_ => unreachable!(),
		}
	}
}

ast_node!(
	/// String literal (without interpolation)
	StringLiteral,
	value
);

impl StringLiteral {
	/// Get the value of this string literal
	pub fn value(&self) -> String {
		decode_string(self.cst_node())
	}
}

ast_node!(
	/// Absent literal `<>`
	Absent,
);

ast_node!(
	/// Infinity literal
	Infinity,
);

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use expect_test::expect;

	#[test]
	fn test_integer_literal() {
		check_ast(
			"x = 1;",
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
                    definition: IntegerLiteral(
                        IntegerLiteral {
                            cst_kind: "integer_literal",
                            value: 1,
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
	fn test_float_literal() {
		check_ast(
			"x = 1.2;",
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
                    definition: FloatLiteral(
                        FloatLiteral {
                            cst_kind: "float_literal",
                            value: 1.2,
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
	fn test_string_literal() {
		check_ast(
			r#"x = "foo";"#,
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
                    definition: StringLiteral(
                        StringLiteral {
                            cst_kind: "string_literal",
                            value: "foo",
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
	fn test_absent() {
		check_ast(
			"x = <>;",
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
                    definition: Absent(
                        Absent {
                            cst_kind: "absent",
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
	fn test_infinity() {
		check_ast(
			r#"x = infinity;"#,
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
                    definition: Infinity(
                        Infinity {
                            cst_kind: "infinity",
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
