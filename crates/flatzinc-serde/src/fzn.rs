//! Parse the original `.fzn` file format.

use winnow::{
	ascii::{digit1, hex_digit1, oct_digit1},
	combinator::{alt, opt, trace},
	stream::Stream,
	token::one_of,
	Parser, Result,
};

use crate::Literal;

/// Parses a basic literal expression.
///
/// ```bnf
/// <basic-literal-expr> ::= <bool-literal>
///                        | <int-literal>
///                        | <float-literal>
///                        | <set-literal>
/// ```
fn literal<'s>(input: &mut &'s str) -> Result<Literal> {
	// This can be optimized if it turns out to be a bottleneck. At the moment, to parse a literal,
	// it will first attempt to parse a float and, if that fails, parse an integer. We can be more
	// clever about that by peeking at the next character to determine what is being parsed.

	alt((float.map(Literal::Float), int.map(Literal::Int))).parse_next(input)
}

/// Parses a float literal from the input.
///
/// ```bnf
/// <float-literal> ::= [-]?[0-9]+.[0-9]+
///                   | [-]?[0-9]+.[0-9]+[Ee][-+]?[0-9]+
///                   | [-]?[0-9]+[Ee][-+]?[0-9]+
/// ```
fn float(input: &mut &str) -> Result<f64> {
	trace("float", move |input: &mut &str| {
		(
			opt('-'),
			digit1,
			alt((
				(
					'.',
					digit1,
					one_of(['e', 'E']),
					opt(one_of(['-', '+'])),
					digit1,
				)
					.take(),
				(one_of(['e', 'E']), opt(one_of(['-', '+'])), digit1).take(),
				('.', digit1).take(),
			)),
		)
			.take()
			.try_map(|parsed: &str| parsed.parse::<f64>())
			.parse_next(input)
	})
	.parse_next(input)
}

/// Parses an integer literal from the input.
///
/// ```bnf
/// <int-literal> ::= [-]?[0-9]+
///                 | [-]?0x[0-9A-Fa-f]+
///                 | [-]?0o[0-7]+
/// ```
fn int(input: &mut &str) -> Result<i64> {
	trace("int", move |input: &mut &str| {
		let is_negative = opt('-').parse_next(input)?.is_some();

		let unsigned_integer = alt((
			("0x", hex_digit1).try_map(|(_, hex)| i64::from_str_radix(hex, 16)),
			("0o", oct_digit1).try_map(|(_, octal)| i64::from_str_radix(octal, 8)),
			digit1.try_map(|base_ten: &str| base_ten.parse::<i64>()),
		))
		.parse_next(input)?;

		if is_negative {
			Ok(-unsigned_integer)
		} else {
			Ok(unsigned_integer)
		}
	})
	.parse_next(input)
}

#[cfg(test)]
mod tests {
	use std::fmt::Debug;

	use winnow::{combinator::eof, error::ParserError, Parser};

	use super::*;

	fn check_parser<'s, O, E>(
		mut parser: impl Parser<&'s str, O, E>,
		expected: O,
		mut input: &'s str,
	) where
		O: Debug + PartialEq,
		E: ParserError<&'s str> + Debug + PartialEq,
		E::Inner: ParserError<&'s str> + PartialEq + Debug,
	{
		let parsed = parser.parse(&mut input);
		assert_eq!(Ok(expected), parsed);
	}

	#[test]
	fn int_literal() {
		check_parser(literal, Literal::Int(0), "0");
		check_parser(literal, Literal::Int(420), "420");
		check_parser(literal, Literal::Int(-38), "-38");
		check_parser(literal, Literal::Int(0xff32a), "0xff32a");
		check_parser(literal, Literal::Int(-0xadc20), "-0xadc20");
		check_parser(literal, Literal::Int(0o12356), "0o12356");
		check_parser(literal, Literal::Int(-0o230), "-0o230");
	}

	#[test]
	fn float_literal() {
		check_parser(literal, Literal::Float(3.02), "3.02");
		check_parser(literal, Literal::Float(-34.85), "-34.85");
		check_parser(literal, Literal::Float(5e-1), "5e-1");
		check_parser(literal, Literal::Float(5e12), "5e12");
		check_parser(literal, Literal::Float(-11e3), "-11e3");
		check_parser(literal, Literal::Float(5e-1), "5E-1");
		check_parser(literal, Literal::Float(5e12), "5E12");
		check_parser(literal, Literal::Float(-11e3), "-11E3");
		check_parser(literal, Literal::Float(5.2e-1), "5.2E-1");
		check_parser(literal, Literal::Float(5.54e12), "5.54E12");
		check_parser(literal, Literal::Float(-11e3), "-11E+3");
	}
}
