//! AST representation of primitive values

use std::num::IntErrorKind;
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
	pub fn value(&self) -> Result<f64, FloatParsingError> {
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
	if let Some(v) = text.strip_prefix("0x") {
		i64::from_str_radix(v, 16)
	} else if let Some(v) = text.strip_prefix("0b") {
		i64::from_str_radix(v, 2)
	} else if let Some(v) = text.strip_prefix("0o") {
		i64::from_str_radix(v, 8)
	} else {
		text.parse::<i64>()
	}
}

/// An error from parsing a float
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatParsingError {
	/// Float parsing error
	ParseFloatError(ParseFloatError),
	/// Missing hex float exponent
	MissingExponent,
	/// Invalid digit
	InvalidDigit,
	/// Value cannot be represented
	InvalidValue,
	/// Hex float is empty
	EmptyHex,
}

impl std::fmt::Display for FloatParsingError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FloatParsingError::ParseFloatError(e) => e.fmt(f),
			FloatParsingError::MissingExponent => {
				write!(f, "Hexadecimal float literals must include an exponent")
			}
			FloatParsingError::InvalidDigit => {
				write!(f, "Invalid digit in float literal")
			}
			FloatParsingError::InvalidValue => {
				write!(
					f,
					"Value cannot be represented as a finite 64-bit floating point value without loss of precision"
				)
			}
			FloatParsingError::EmptyHex => {
				write!(f, "Missing hexadecimal value after 0x")
			}
		}
	}
}

impl std::error::Error for FloatParsingError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			FloatParsingError::ParseFloatError(e) => Some(e),
			_ => None,
		}
	}
}

impl From<ParseFloatError> for FloatParsingError {
	fn from(value: ParseFloatError) -> Self {
		FloatParsingError::ParseFloatError(value)
	}
}

/// Parse a MiniZinc float literal
///
/// Only allows finite values.
/// Hexadecimal float literals must be exactly representable as a double - no rounding is allowed.
pub fn parse_float_literal(text: &str) -> Result<f64, FloatParsingError> {
	if let Some(hex_float) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
		let mut split_p = hex_float.splitn(2, |c| matches!(c, 'p' | 'P'));
		let mut split_dot = split_p.next().unwrap().splitn(2, '.');
		let before_dot = split_dot.next().unwrap();
		let after_dot = split_dot.next().unwrap_or_default();
		if before_dot.is_empty() && after_dot.is_empty() {
			return Err(FloatParsingError::EmptyHex);
		}
		let after_p = split_p.next().ok_or(FloatParsingError::MissingExponent)?;
		if after_p.is_empty() {
			return Err(FloatParsingError::MissingExponent);
		}
		let w_str = before_dot.trim_start_matches('0');
		let f_str = after_dot.trim_end_matches('0');
		if w_str.len() + f_str.len() > 14 {
			return Err(FloatParsingError::InvalidValue);
		}
		let w = if w_str.is_empty() {
			0
		} else {
			u64::from_str_radix(w_str, 16).map_err(|_| FloatParsingError::InvalidDigit)?
		};
		let f = if f_str.is_empty() {
			0
		} else {
			u64::from_str_radix(f_str, 16).map_err(|_| FloatParsingError::InvalidDigit)?
		};
		let places = 4 * f_str.len();
		let mut significand = (w << places) | f;

		if significand == 0 {
			return Ok(0.0);
		}

		let leading_zeros = significand.leading_zeros() as i64;
		let trailing_zeros = significand.trailing_zeros() as i64;

		if leading_zeros + trailing_zeros < 11 {
			return Err(FloatParsingError::InvalidValue);
		}

		let mut exponent = after_p.parse::<i64>().map_err(|e| match e.kind() {
			IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
				FloatParsingError::InvalidValue
			}
			_ => FloatParsingError::InvalidDigit,
		})? - (places as i64)
			+ 52;

		let diff = (leading_zeros - 11).min((exponent + 1022).max(-trailing_zeros));
		if diff > 0 {
			significand <<= diff;
		} else {
			significand >>= -diff;
		}
		exponent -= diff;

		if (significand & (1 << 52)) == 0 {
			if exponent == -1022 {
				exponent -= 1;
			} else {
				return Err(FloatParsingError::InvalidValue);
			}
		}

		if exponent > 1023 {
			return Err(FloatParsingError::InvalidValue);
		}

		let bits = (((exponent + 1023) as u64) << 52) | (0xFFFFFFFFFFFFF & significand);
		return Ok(f64::from_bits(bits));
	}
	let parsed = text.parse::<f64>()?;
	if parsed.is_finite() {
		Ok(parsed)
	} else {
		Err(FloatParsingError::InvalidValue)
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::{helpers::test::*, FloatParsingError};
	use expect_test::expect;

	use super::parse_float_literal;

	#[test]
	fn test_parse_float() {
		assert_eq!(parse_float_literal("123.4"), Ok(123.4));
		assert_eq!(parse_float_literal("1.5E6"), Ok(1.5E6));
		assert_eq!(parse_float_literal("1E2"), Ok(1E2));
		assert_eq!(parse_float_literal("0x0p10"), Ok(0.0));
		assert_eq!(parse_float_literal("0x0.0p10"), Ok(0.0));
		assert_eq!(parse_float_literal("0x.0p10"), Ok(0.0));
		assert_eq!(parse_float_literal("0x0.p10"), Ok(0.0));
		assert_eq!(parse_float_literal("0x1p-2"), Ok(0.25));
		assert_eq!(parse_float_literal("0x1.b7p-1"), Ok(0.857421875));
		assert_eq!(parse_float_literal("0x1.999999999999ap-4"), Ok(0.1));
		assert_eq!(parse_float_literal("0x3.3333333333334p-5"), Ok(0.1));
		assert_eq!(
			parse_float_literal("0x0.0000000000001p-1022").map(|f| f.to_bits()),
			Ok(1)
		);
		assert_eq!(parse_float_literal("0x1p-1074").map(|f| f.to_bits()), Ok(1));
		assert_eq!(parse_float_literal("0x8p-1077").map(|f| f.to_bits()), Ok(1));
		assert_eq!(
			parse_float_literal("0x1p2000").unwrap_err(),
			FloatParsingError::InvalidValue
		);
		assert_eq!(
			parse_float_literal("0x1p-2000").unwrap_err(),
			FloatParsingError::InvalidValue
		);
		assert_eq!(
			parse_float_literal("0x").unwrap_err(),
			FloatParsingError::EmptyHex
		);
		assert_eq!(
			parse_float_literal("0x1").unwrap_err(),
			FloatParsingError::MissingExponent
		);
		assert_eq!(
			parse_float_literal("0x1p").unwrap_err(),
			FloatParsingError::MissingExponent
		);
	}

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
