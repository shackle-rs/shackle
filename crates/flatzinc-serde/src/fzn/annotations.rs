//! Parser for an FZN annotation.

use winnow::{
	combinator::{alt, delimited, opt, preceded, separated},
	Parser, Result,
};

use crate::{
	fzn::{identifier, literal, token, Stream},
	Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral,
};

/// Parse an annotation.
///
/// ```bnf
/// <annotation> ::= <identifier>
///                | <identifier> "(" <ann-expr> "," ... ")"
/// ```
pub(super) fn annotation(input: &mut Stream<'_, '_>) -> Result<Annotation> {
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
fn annotation_argument(input: &mut Stream<'_, '_>) -> Result<AnnotationArgument> {
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
fn annotation_literal(input: &mut Stream<'_, '_>) -> Result<AnnotationLiteral> {
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
fn annotation_call(input: &mut Stream<'_, '_>) -> Result<AnnotationCall> {
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

#[cfg(test)]
mod tests {
	use rangelist::RangeList;

	use crate::Literal;

	use super::super::tests::check_parser;
	use super::*;

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
}
