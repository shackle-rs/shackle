//! Parser for an FZN annotation.

use std::{
	fmt::{Debug, Display},
	str::FromStr,
};

use winnow::{
	combinator::{alt, delimited, opt, preceded, repeat, separated},
	Parser, Result,
};

use crate::{
	fzn::{identifier, identifier_raw, literal, token, Stream},
	Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, FznParseError, Literal,
};

/// Semantic flags projected out of special FlatZinc annotations.
#[derive(Default)]
pub(crate) struct AnnotationFlags {
	pub(crate) defined: bool,
	pub(crate) introduced: bool,
	pub(crate) output: bool,
}

/// Parse an annotation.
///
/// ```bnf
/// <annotation> ::= <identifier>
///                | <identifier> "(" <ann-expr> "," ... ")"
/// ```
pub(super) fn annotation<'a, Identifier>(
	input: &mut Stream<'a, '_, Identifier>,
) -> Result<(&'a str, Option<Vec<AnnotationArgument<Identifier>>>)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	preceded(
		token("::"),
		(
			identifier_raw,
			opt(delimited(
				token('('),
				separated(0.., token(annotation_argument), token(',')),
				token(')'),
			)),
		),
	)
	.parse_next(input)
}

/// Parses an annotation argument (or annotation expression).
///
/// ```bnf
/// <ann-expr> := <basic-ann-expr>
///             | "[" [ <basic-ann-expr> "," ... ] "]"
/// ```
fn annotation_argument<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<AnnotationArgument<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
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

/// Parses an annotation with arguments.
///
/// This does not have an analogue in the FZN grammar. It is only used to parse annotation
/// arguments that are nested annotation calls.
fn annotation_call<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<AnnotationCall<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
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

/// Parses an annotation literal (or basic annotation expression).
///
/// ```bnf
/// <basic-ann-expr> := <basic-literal-expr>
///                   | <var-par-identifier>
///                   | <string-literal>
///                   | <annotation>
/// ```
fn annotation_literal<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<AnnotationLiteral<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	alt((
		annotation_call.map(AnnotationLiteral::Annotation),
		literal.map(AnnotationLiteral::BaseLiteral),
	))
	.parse_next(input)
}

pub(super) fn constraint_annotations<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<(Option<Identifier>, Vec<Annotation<Identifier>>)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	repeat(0.., annotation)
		.try_map(
			|anns: Vec<(&str, Option<Vec<AnnotationArgument<Identifier>>>)>| -> Result<(_, Vec<Annotation<Identifier>>), FznParseError> {
				let mut defines = None;
				let anns: Result<Vec<_>, _> = anns
					.into_iter()
					.filter_map(|(ident, args)| match (ident, args) {
						("defines_var", Some(mut v)) if v.len() == 1 && matches!(v[0], AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(Literal::Identifier(
							_
						))))
						 => {
							let AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(Literal::Identifier(
								identifier,
							))) = v.remove(0) else {unreachable!()};
							defines = Some(identifier);
							None
						}
						(ident, args) => Some(
							ident
								.parse::<Identifier>()
								.map(|ident| {
									if let Some(args) = args {
										Annotation::Call(AnnotationCall { id: ident, args })
									} else {
										Annotation::Atom(ident)
									}
								})
								.map_err(|err| FznParseError::IdentifierError {
									ident: ident.to_owned(),
									err: err.to_string(),
								}),
						),
					})
					.collect();
				Ok((defines, anns?))
			},
		)
		.parse_next(input)
}

pub(super) fn general_annotations<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<Vec<Annotation<Identifier>>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	repeat(0.., annotation)
		.try_map(
			|anns: Vec<(&str, Option<Vec<AnnotationArgument<Identifier>>>)>| -> Result<Vec<Annotation<Identifier>>, FznParseError> {
				anns
					.into_iter()
					.map(|(ident, args)| ident
						.parse::<Identifier>()
						.map(|ident| {
							if let Some(args) = args {
								Annotation::Call(AnnotationCall { id: ident, args })
							} else {
								Annotation::Atom(ident)
							}
						})
						.map_err(|err| FznParseError::IdentifierError {
							ident: ident.to_owned(),
							err: err.to_string(),
						})
					)
					.collect()
			},
		)
		.parse_next(input)
}

pub(super) fn variable_annotations<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<(AnnotationFlags, Vec<Annotation<Identifier>>)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	repeat(0.., annotation)
		.try_map(
			|anns: Vec<(&str, Option<Vec<AnnotationArgument<Identifier>>>)>| -> Result<(AnnotationFlags, Vec<Annotation<Identifier>>), FznParseError> {
				let mut flags = AnnotationFlags::default();
				let anns: Result<Vec<_>, _> = anns
					.into_iter()
					.filter_map(|(ident, args)| match (ident, args) {
						("is_defined_var", None) => {
							flags.defined = true;
							None
						}
						("var_is_introduced", None) => {
							flags.introduced = true;
							None
						}
						("output_var", None) => {
							flags.output = true;
							None
						}
						("output_array", Some(_)) => {
							flags.output = true;
							None
						}
						(ident, args) => Some(
							ident
								.parse::<Identifier>()
								.map(|ident| {
									if let Some(args) = args {
										Annotation::Call(AnnotationCall { id: ident, args })
									} else {
										Annotation::Atom(ident)
									}
								})
								.map_err(|err| FznParseError::IdentifierError {
									ident: ident.to_owned(),
									err: err.to_string(),
								}),
						),
					})
					.collect();
				Ok((flags, anns?))
			},
		)
		.parse_next(input)
}

#[cfg(test)]
mod tests {
	use rangelist::RangeList;

	use crate::{
		fzn::{general_annotations, tests::check_parser},
		Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, Literal,
	};

	#[test]
	fn annotation_call_with_array_argument() {
		check_parser(
			general_annotations,
			vec![Annotation::Call(AnnotationCall {
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
			})],
			":: some_annotation([other_annotation(5), 3.4])",
		);
	}

	#[test]
	fn annotation_call_with_literal_argument() {
		check_parser(
			general_annotations,
			vec![Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
					Literal::Identifier("other_annotation".to_owned()),
				))],
			})],
			":: some_annotation(other_annotation)",
		);
		check_parser(
			general_annotations,
			vec![Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
					Literal::IntSet(RangeList::from(1..=5)),
				))],
			})],
			":: some_annotation(1..5)",
		);
	}

	#[test]
	fn annotation_call_with_nested_annotation_call_argument() {
		check_parser(
			general_annotations,
			vec![Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::Annotation(
					AnnotationCall {
						id: "other_annotation".to_owned(),
						args: vec![AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Int(5),
						))],
					},
				))],
			})],
			":: some_annotation(other_annotation(5))",
		);
		check_parser(
			general_annotations,
			vec![Annotation::Call(AnnotationCall {
				id: "some_annotation".to_owned(),
				args: vec![AnnotationArgument::Literal(AnnotationLiteral::Annotation(
					AnnotationCall {
						id: "another_annotation".to_owned(),
						args: vec![],
					},
				))],
			})],
			":: some_annotation(another_annotation ())",
		);
	}

	#[test]
	fn atom_annotation() {
		check_parser(
			general_annotations,
			vec![Annotation::Atom("output_var".to_owned())],
			":: output_var",
		);
	}
}
