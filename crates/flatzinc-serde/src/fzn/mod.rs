//! Parse the original `.fzn` file format.

mod error;

use std::{collections::BTreeMap, io::BufRead};

use rangelist::RangeList;
use winnow::{
	ascii::{digit1, hex_digit1, multispace0, oct_digit1},
	combinator::{alt, delimited, opt, preceded, separated, separated_pair, trace},
	error::ContextError,
	stream::AsChar,
	token::{one_of, take_while},
	Parser, Result,
};

pub use error::*;

use crate::{
	Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, FlatZinc, Literal, Type,
	Variable,
};

/// Parse the `.fzn` source to a [`FlatZinc`] instance.
///
/// # Example
/// ```
/// use std::collections::BTreeMap;
/// use flatzinc_serde::Domain;
/// use flatzinc_serde::FlatZinc;
/// use flatzinc_serde::Method;
/// use flatzinc_serde::RangeList;
/// use flatzinc_serde::SolveObjective;
/// use flatzinc_serde::Type;
/// use flatzinc_serde::Variable;
///
/// let source = r#"
/// var 1..5: x;
/// var 1..5: y;
///
/// constraint int_le(x, y);
///
/// solve satisfy;
/// "#;
///
/// let parsed = flatzinc_serde::fzn::parse(source.as_bytes())
///     .expect("valid fzn");
///
/// let expected: FlatZinc<String> =  FlatZinc {
///     variables: BTreeMap::from([
///        ("x".to_owned(), Variable {
///            ty: Type::Int,
///            domain: Some(Domain::Int(RangeList::from(1..=5))),
///            value: None,
///            ann: vec![],
///            defined: false,
///            introduced: false,
///        }),
///        ("y".to_owned(), Variable {
///            ty: Type::Int,
///            domain: Some(Domain::Int(RangeList::from(1..=5))),
///            value: None,
///            ann: vec![],
///            defined: false,
///            introduced: false,
///        }),
///    ]),
///    arrays: BTreeMap::default(),
///    constraints: vec![
///    ],
///    output: vec![],
///    solve: SolveObjective {
///        method: Method::Satisfy,
///        objective: None,
///        ann: vec![],
///    },
///    version: "FZN".to_owned(),
/// };
///
/// assert_eq!(expected, parsed);
/// ```
pub fn parse(mut source: impl BufRead) -> std::result::Result<FlatZinc, FznParseError> {
	let mut buffer = Vec::new();

	let variables = BTreeMap::default();
	let arrays = BTreeMap::default();
	let constraints = vec![];
	let output = vec![];
	let solve = None;

	loop {
		buffer.clear();
		let _ = source.read_until(b';', &mut buffer)?;

		let statement_str = std::str::from_utf8(&buffer)?.trim();
		if statement_str.is_empty() {
			break;
		}

		match model_item.parse(statement_str)? {
			ModelItem::Variable(variable) => todo!(),
		}
	}

	Ok(FlatZinc {
		variables,
		arrays,
		constraints,
		output,
		solve: solve.ok_or(FznParseError::MissingSolveItem)?,
		version: "FZN".to_owned(),
	})
}

/// Any item in a flatzinc model.
enum ModelItem {
	/// A variable model item.
	Variable((String, Variable)),
}

/// Parse a model item.
fn model_item(input: &mut &str) -> Result<ModelItem> {
	alt((variable.map(ModelItem::Variable),)).parse_next(input)
}

/// Parse an annotation.
///
/// ```bnf
/// <annotation> ::= <identifier>
///                | <identifier> "(" <ann-expr> "," ... ")"
/// ```
fn annotation(input: &mut &str) -> Result<Annotation> {
	preceded(
		token("::"),
		(
			identifier,
			opt(delimited(
				token('('),
				separated(0.., token(annotation_argument), token(',')),
				token(')'),
			)),
		),
	)
	.map(|(id, optional_call)| match optional_call {
		Some(args) => Annotation::Call(AnnotationCall { id, args }),
		None => Annotation::Atom(id),
	})
	.parse_next(input)
}

/// Parses an annotation argument (or annotation expression).
///
/// ```bnf
/// <ann-expr> := <basic-ann-expr>
///             | "[" [ <basic-ann-expr> "," ... ] "]"
/// ```
fn annotation_argument(input: &mut &str) -> Result<AnnotationArgument> {
	alt((
		annotation_literal.map(AnnotationArgument::Literal),
		delimited(
			token('['),
			separated(0.., token(annotation_literal), token(',')),
			token(']'),
		)
		.map(AnnotationArgument::Array),
	))
	.parse_next(input)
}

/// Parses an annotation literal (or basic annotation expression).
///
/// ```bnf
/// <basic-ann-expr> := <basic-literal-expr>
///                   | <var-par-identifier>
///                   | <string-literal>
///                   | <annotation>
/// ```
fn annotation_literal(input: &mut &str) -> Result<AnnotationLiteral> {
	alt((
		annotation_call.map(AnnotationLiteral::Annotation),
		literal.map(AnnotationLiteral::BaseLiteral),
	))
	.parse_next(input)
}

/// Parses an annotation with arguments.
///
/// This does not have an analogue in the FZN grammar. It is only used to parse annotation
/// arguments that are nested annotation calls.
fn annotation_call(input: &mut &str) -> Result<AnnotationCall> {
	(
		identifier,
		delimited(
			token('('),
			separated(0.., token(annotation_argument), token(',')),
			token(')'),
		),
	)
		.map(|(id, args)| AnnotationCall { id, args })
		.parse_next(input)
}

/// Parses a basic literal expression.
///
/// ```bnf
/// <basic-literal-expr> ::= <bool-literal>
///                        | <int-literal>
///                        | <float-literal>
///                        | <set-literal>
/// ```
fn literal(input: &mut &str) -> Result<Literal> {
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
fn boolean(input: &mut &str) -> Result<bool> {
	alt(("true".map(|_| true), "false".map(|_| false))).parse_next(input)
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

/// Parses an identifier.
///
/// ```bnf
/// <var-par-identifier> ::= [A-Za-z_][A-Za-z0-9_]*
/// ```
fn identifier(input: &mut &str) -> Result<String> {
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
fn set<'s, T>(
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

		let interval_set = separated_pair(token(elem_parser), token(".."), token(elem_parser))
			.map(|(start, end)| RangeList::from_iter([start..=end]));

		alt((sparse_set, interval_set)).parse_next(input)
	}
}

/// Parses a token from the input.
///
/// Wraps the given parser with optional preceding and succeeding whitespace.
fn token<'s, T>(
	parser: impl Parser<&'s str, T, ContextError>,
) -> impl Parser<&'s str, T, ContextError> {
	delimited(multispace0, parser, multispace0)
}

/// Parse a variable model item.
fn variable(input: &mut &str) -> Result<(String, Variable)> {
	(
		token("var"),
		token("int"),
		token(":"),
		identifier,
		token(";"),
	)
		.map(|(_, _, _, name, _)| {
			(
				name,
				Variable {
					ty: Type::Int,
					domain: None,
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			)
		})
		.parse_next(input)
}

#[cfg(test)]
mod tests {
	use std::fmt::Debug;

	use rangelist::RangeList;
	use winnow::{error::ParserError, Parser};

	use crate::{Annotation, AnnotationArgument, Type};

	use super::*;

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

	#[test]
	fn atom_annotation() {
		check_parser(
			annotation,
			Annotation::Atom("output_var".to_owned()),
			":: output_var",
		);
	}

	#[test]
	fn annotation_call_with_literal_argument() {
		check_parser(
			annotation,
			Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
					Literal::Identifier("other_annotation".to_owned()),
				))],
			}),
			":: some_annotation(other_annotation)",
		);
		check_parser(
			annotation,
			Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
					Literal::IntSet(RangeList::from(1..=5)),
				))],
			}),
			":: some_annotation(1..5)",
		);
	}

	#[test]
	fn annotation_call_with_nested_annotation_call_argument() {
		check_parser(
			annotation,
			Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::Annotation(
					AnnotationCall {
						id: "other_annotation".to_owned(),
						args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Int(5),
						))],
					},
				))],
			}),
			":: some_annotation(other_annotation(5))",
		);
		check_parser(
			annotation,
			Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::Annotation(
					AnnotationCall {
						id: "another_annotation".to_owned(),
						args: vec![],
					},
				))],
			}),
			":: some_annotation(another_annotation ())",
		);
	}

	#[test]
	fn annotation_call_with_array_argument() {
		check_parser(
			annotation,
			Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Array(vec![
					AnnotationLiteral::Annotation(AnnotationCall {
						id: "other_annotation".to_owned(),
						args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Int(5),
						))],
					}),
					AnnotationLiteral::BaseLiteral(Literal::Float(3.4)),
				])],
			}),
			":: some_annotation([other_annotation(5), 3.4])",
		);
	}

	#[test]
	fn simple_variable_item() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int,
					domain: None,
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var int: x;",
		);
	}

	fn check_parser<'s, O, E>(mut parser: impl Parser<&'s str, O, E>, expected: O, input: &'s str)
	where
		O: Debug + PartialEq,
		E: ParserError<&'s str> + Debug + PartialEq,
		E::Inner: ParserError<&'s str> + PartialEq + Debug,
	{
		let parsed = parser.parse(input);
		assert_eq!(Ok(expected), parsed);
	}
}
