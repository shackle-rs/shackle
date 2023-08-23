//! AST representation of primitive values

use std::num::ParseFloatError;
use std::num::ParseIntError;

use super::AstNode;

use super::helpers::*;

ast_node!(
	/// Integer literal
	IntegerLiteral,
	value
);

impl IntegerLiteral {
	/// Get the value of this integer literal
	pub fn value(&self) -> Result<i64, ParseIntError> {
		parse_integer_literal(self.cst_text())
	}
}

ast_node!(
	/// Float literal
	FloatLiteral,
	value
);

impl FloatLiteral {
	/// Get the value of this float literal
	pub fn value(&self) -> Result<f64, ParseFloatError> {
		parse_float_literal(self.cst_text())
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

/// Parse a MiniZinc integer literal
pub fn parse_integer_literal(text: &str) -> Result<i64, ParseIntError> {
	if text.starts_with("0x") {
		i64::from_str_radix(&text[2..], 16)
	} else if text.starts_with("0b") {
		i64::from_str_radix(&text[2..], 2)
	} else if text.starts_with("0o") {
		i64::from_str_radix(&text[2..], 8)
	} else {
		text.parse::<i64>()
	}
}

/// Parse a MiniZinc float literal
pub fn parse_float_literal(text: &str) -> Result<f64, ParseFloatError> {
	if text.starts_with("0x") {
		todo!("Hexadecimal floats not yet implemented")
	}
	text.parse()
}

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
                            value: Ok(
                                1,
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
                            value: Ok(
                                1.2,
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

	#[test]
	fn test_non_decimal() {
		check_ast(
			r#"x = 0xFF;"#,
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
                            value: Ok(
                                255,
                            ),
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);

		check_ast(
			r#"x = 0b11;"#,
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
                            value: Ok(
                                3,
                            ),
                        },
                    ),
                },
            ),
        ],
    }
"#]),
		);

		check_ast(
			r#"x = 0o77;"#,
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
                            value: Ok(
                                63,
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
}
