//! Parsers for tokens used throughout the FlatZinc grammar.

use rangelist::RangeList;
use winnow::{
	ascii::{digit1, hex_digit1, multispace0, oct_digit1},
	combinator::{alt, delimited, opt, separated, separated_pair, trace},
	error::ContextError,
	stream::AsChar,
	token::{one_of, take_while},
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
pub(super) fn literal(input: &mut &str) -> Result<Literal> {
	// This can be optimized if it turns out to be a bottleneck. At the moment, to parse a literal,
	// it will first attempt to parse a float and, if that fails, parse an integer. We can be more
	// clever about that by peeking at the next character to determine what is being parsed.

	alt((
		set(int).map(Literal::IntSet),
		set(float).map(Literal::FloatSet),
		boolean.map(Literal::Bool),
		float.map(Literal::Float),
		int.map(Literal::Int),
		identifier.map(Literal::Identifier),
	))
	.parse_next(input)
}

/// Parses a boolean literal.
///
/// ```bnf
/// <bool-literal> ::= "false"
///                  | "true"
/// ```
pub(super) fn boolean(input: &mut &str) -> Result<bool> {
	alt(("true".map(|_| true), "false".map(|_| false))).parse_next(input)
}

/// Parses a float literal from the input.
///
/// ```bnf
/// <float-literal> ::= [-]?[0-9]+.[0-9]+
///                   | [-]?[0-9]+.[0-9]+[Ee][-+]?[0-9]+
///                   | [-]?[0-9]+[Ee][-+]?[0-9]+
/// ```
pub(super) fn float(input: &mut &str) -> Result<f64> {
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
pub(super) fn int(input: &mut &str) -> Result<i64> {
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

/// Parses an identifier.
///
/// ```bnf
/// <var-par-identifier> ::= [A-Za-z_][A-Za-z0-9_]*
/// ```
pub(super) fn identifier(input: &mut &str) -> Result<String> {
	trace(
		"identifier",
		(
			one_of(|c: char| c.is_alpha() || c == '_'),
			take_while(0.., |c: char| c.is_alphanum() || c == '_'),
		),
	)
	.take()
	.map(Into::into)
	.parse_next(input)
}

/// Parses a set literal.
///
/// Works with either interval sets or sparse sets.
///
/// The grammar is modified from the documentation. Here we abstract the element type.
/// ```bnf
/// <set-literal> ::= "{" [ <elem> "," ... ] "}"
///                 | <elem> ".." <elem>
/// ```
pub(super) fn set<'s, T>(
	elem_parser: impl Parser<&'s str, T, ContextError> + Copy,
) -> impl Parser<&'s str, RangeList<T>, ContextError>
where
	T: PartialOrd + Copy + 'static,
{
	move |input: &mut &'s str| -> Result<RangeList<T>> {
		let sparse_set = delimited(
			token('{'),
			separated(0.., token(elem_parser), token(',')),
			token('}'),
		)
		.map(|elems: Vec<T>| RangeList::from_iter(elems.into_iter().map(|elem| elem..=elem)));

		alt((sparse_set, interval_set(elem_parser))).parse_next(input)
	}
}

/// Higher-order parser for `<token> .. <token>`.
pub(super) fn interval_set<'s, T>(
	elem_parser: impl Parser<&'s str, T, ContextError> + Copy,
) -> impl Parser<&'s str, RangeList<T>, ContextError>
where
	T: PartialOrd + Copy + 'static,
{
	move |input: &mut &'s str| {
		separated_pair(token(elem_parser), token(".."), token(elem_parser))
			.map(|(start, end)| RangeList::from_iter([start..=end]))
			.parse_next(input)
	}
}

/// Parses a token from the input.
///
/// Wraps the given parser with optional preceding and succeeding whitespace.
pub(super) fn token<'s, T>(
	parser: impl Parser<&'s str, T, ContextError>,
) -> impl Parser<&'s str, T, ContextError> {
	delimited(multispace0, parser, multispace0)
}

#[cfg(test)]
mod tests {
	use super::*;

	use super::super::tests::check_parser;

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

	#[test]
	fn identifier_literal() {
		check_parser(
			literal,
			Literal::Identifier("some_name".to_owned()),
			"some_name",
		);
		check_parser(
			literal,
			Literal::Identifier("_some_name".to_owned()),
			"_some_name",
		);
		check_parser(
			literal,
			Literal::Identifier("_SomeName283".to_owned()),
			"_SomeName283",
		);
	}

	#[test]
	fn boolean_literal() {
		check_parser(literal, Literal::Bool(true), "true");
		check_parser(literal, Literal::Bool(false), "false");
	}

	#[test]
	fn int_set_literal() {
		check_parser(literal, Literal::IntSet(RangeList::from(1..=5)), "1..5");
		check_parser(
			literal,
			Literal::IntSet(RangeList::from_iter([1..=1, 4..=4, 6..=6])),
			"{1, 4, 6}",
		);
	}

	#[test]
	fn float_set_literal() {
		check_parser(literal, Literal::IntSet(RangeList::from(1..=5)), "1..5");
		check_parser(
			literal,
			Literal::FloatSet(RangeList::from_iter([1.3..=1.3, 4e3..=4e3, -4.8..=-4.8])),
			"{1.3, 4e3, -4.8}",
		);
	}
}
