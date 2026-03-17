//! Parse the original `.fzn` file format.

mod annotations;
mod error;
mod primitives;

use std::{
	collections::HashMap,
	fmt::{Debug, Display},
	io::BufRead,
	str::FromStr,
};

use annotations::*;
pub use error::FznParseError;
use primitives::*;
use winnow::{
	combinator::{alt, delimited, opt, preceded, separated, separated_pair},
	Parser, Result, Stateful,
};

use crate::{
	Argument, Array, Constraint, FlatZinc, Literal, Method, SolveObjective, Type, Variable,
};

/// A declaration item in a FlatZinc model.
#[derive(Debug, PartialEq)]
enum Declaration<Identifier> {
	/// A parameter declaration.
	Parameter((String, Literal<Identifier>)),
	/// A variable declaration.
	Variable((Identifier, Variable<Identifier>, bool)),
	/// An array declaration.
	Array((Identifier, Array<Identifier>, bool)),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ParsePhase {
	Predicates,
	Declarations,
	Constraints,
	Solve,
}

#[derive(Debug, PartialEq)]
struct ParseState<'s, Identifier> {
	parameters: &'s mut HashMap<String, Literal<Identifier>>,
}

type Stream<'source, 'state, Identifier> = Stateful<&'source str, ParseState<'state, Identifier>>;

/// Parses a constraint argument.
///
/// ```bnf
/// <expr> ::= <basic-expr>
///          | <array-literal>
/// ```
fn argument<Identifier>(input: &mut Stream<'_, '_, Identifier>) -> Result<Argument<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	alt((
		literal.map(Argument::Literal),
		delimited(
			token("["),
			separated(0.., token(literal), token(",")),
			token("]"),
		)
		.map(Argument::Array),
	))
	.parse_next(input)
}

/// Parse an array declaration.
fn array_item<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<(Identifier, Array<Identifier>, bool)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	let _ = token("array").parse_next(input)?;
	let _ = delimited(token("["), interval_set(int), token("]")).parse_next(input)?;
	let _ = token("of").parse_next(input)?;
	let _ = opt(token("var")).parse_next(input)?;
	let _ = basic_variable_type.parse_next(input)?;

	let _ = token(":").parse_next(input)?;
	let id = token(identifier).parse_next(input)?;
	let (flags, ann) = variable_annotations.parse_next(input)?;
	let contents = preceded(token("="), delimited_list("[", literal, "]")).parse_next(input)?;
	let _ = token(";").parse_next(input)?;

	Ok((
		id,
		Array {
			contents,
			ann,
			defined: flags.defined,
			introduced: flags.introduced,
		},
		flags.output,
	))
}

/// Parse a basic parameter type.
///
/// ```bnf
/// <basic-par-type> ::= "bool"
///                    | "int"
///                    | "float"
///                    | "set of int"
/// ```
fn basic_parameter_type<I>(input: &mut Stream<'_, '_, I>) -> Result<Type>
where
	I: Debug,
{
	alt((
		"bool".map(|_| Type::Bool),
		"int".map(|_| Type::Int(None)),
		"float".map(|_| Type::Float(None)),
		(token("set"), token("of"), token("int")).map(|_| Type::IntSet(None)),
	))
	.parse_next(input)
}

/// Parses the domain in a variable declaration.
///
/// Has no direct analogue in the grammar. However, it is essentially the `<basic-var-type>`
/// without the "var" token preceding it:
///
/// ```bnf
/// <basic-var-type> ::= "var" <basic-par-type>
///                    | "var" <int-literal> ".." <int-literal>
///                    | "var" "{" <int-literal> "," ... "}"
///                    | "var" <float-literal> ".." <float-literal>
///                    | "var" "set" "of" <int-literal> ".." <int-literal>
///                    | "var" "set" "of" "{" [ <int-literal> "," ... ] "}"
/// ```
fn basic_variable_type<I>(input: &mut Stream<'_, '_, I>) -> Result<Type>
where
	I: Debug,
{
	alt((
		basic_parameter_type,
		preceded((token("set"), token("of")), set(int)).map(|values| Type::IntSet(Some(values))),
		set(int).map(|values| Type::Int(Some(values))),
		interval_set(float).map(|values| Type::Float(Some(values))),
	))
	.parse_next(input)
}

/// Parse a constraint item.
///
/// ```bnf
/// <constraint-item> ::= "constraint" <identifier> "(" [ <expr> "," ... ] ")" <annotations> ";"
/// ```
fn constraint<Identifier>(input: &mut Stream<'_, '_, Identifier>) -> Result<Constraint<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	(
		token("constraint"),
		token(identifier),
		delimited(
			token("("),
			separated(0.., token(argument), token(",")),
			token(")"),
		),
		constraint_annotations,
		token(";"),
	)
		.map(|(_, id, args, (defines, ann), _)| Constraint {
			id,
			args,
			ann,
			defines,
		})
		.parse_next(input)
}

/// Parse a declaration item.
fn declaration<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<Declaration<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	alt((
		array_item.map(Declaration::Array),
		variable.map(Declaration::Variable),
		parameter_item.map(Declaration::Parameter),
	))
	.parse_next(input)
}

fn parameter_item<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<(String, Literal<Identifier>)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	delimited(
		(basic_parameter_type, token(":")),
		separated_pair(
			token(identifier_raw.map(str::to_owned)),
			token("="),
			token(literal::<Identifier>),
		),
		token(";"),
	)
	.parse_next(input)
}

/// Parse the `.fzn` source to a [`FlatZinc`] instance.
///
/// This is used by [`crate::FlatZinc::from_fzn`], which is the public entry
/// point for `.fzn` parsing.
pub(crate) fn parse<Identifier, VarMap, ArrayMap>(
	mut source: impl BufRead,
) -> Result<FlatZinc<Identifier, VarMap, ArrayMap>, FznParseError>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
	VarMap: FromIterator<(Identifier, Variable<Identifier>)>,
	ArrayMap: FromIterator<(Identifier, Array<Identifier>)>,
{
	let mut variables = Vec::new();
	let mut arrays = Vec::new();
	let mut constraints = Vec::new();
	let mut output = Vec::new();
	let mut solve = None;

	let mut buffer = Vec::new();
	let mut parameters = HashMap::default();
	let mut phase = ParsePhase::Predicates;

	loop {
		read_statement(&mut source, &mut buffer)?;
		if buffer.is_empty() {
			break;
		}

		let mut stream = Stateful {
			input: std::str::from_utf8(&buffer)?,
			state: ParseState::<Identifier> {
				parameters: &mut parameters,
			},
		};

		// Check whether the statement only contains whitespace and comments
		token(())
			.parse_next(&mut stream)
			.map_err(|error| FznParseError::SyntaxError(error.to_string()))?;
		if stream.input.is_empty() {
			continue;
		}

		if stream.input.starts_with("predicate") {
			if phase > ParsePhase::Predicates {
				return Err(FznParseError::SyntaxError(
					"predicate items must appear before declarations, constraints, and solve"
						.into(),
				));
			}
			token(predicate_item)
				.parse_next(&mut stream)
				.map_err(|error| FznParseError::SyntaxError(error.to_string()))?;
			continue;
		}

		if stream.input.starts_with("constraint") {
			if phase > ParsePhase::Constraints {
				return Err(FznParseError::SyntaxError(
					"constraint items must appear before the solve item".into(),
				));
			}
			phase = ParsePhase::Constraints;
			let constraint = token(constraint)
				.parse_next(&mut stream)
				.map_err(|error| FznParseError::SyntaxError(error.to_string()))?;
			constraints.push(constraint);
			continue;
		}

		if stream.input.starts_with("solve") {
			if phase > ParsePhase::Solve || solve.is_some() {
				return Err(FznParseError::MultipleSolveItems);
			}
			phase = ParsePhase::Solve;
			let solve_objective = token(solve_objective)
				.parse_next(&mut stream)
				.map_err(|error| FznParseError::SyntaxError(error.to_string()))?;
			solve = Some(solve_objective);
			continue;
		}

		if phase > ParsePhase::Declarations {
			return Err(FznParseError::SyntaxError(
				"declarations must appear before constraints and the solve item".into(),
			));
		}
		phase = ParsePhase::Declarations;

		let declaration = token(declaration)
			.parse_next(&mut stream)
			.map_err(|error| FznParseError::SyntaxError(error.to_string()))?;
		match declaration {
			Declaration::Parameter((name, literal)) => {
				let _ = parameters.insert(name, literal);
			}
			Declaration::Variable((name, variable, is_output)) => {
				if is_output {
					output.push(name.clone());
				}
				variables.push((name, variable));
			}
			Declaration::Array((name, array, is_output)) => {
				if is_output {
					output.push(name.clone());
				}
				arrays.push((name, array));
			}
		}
	}

	Ok(FlatZinc {
		variables: variables.into_iter().collect(),
		arrays: arrays.into_iter().collect(),
		constraints,
		output,
		solve: solve.ok_or(FznParseError::MissingSolveItem)?,
		version: "1.0".to_owned(),
	})
}

/// Parses a predicate item.
///
/// ```bnf
/// <predicate-item> ::= "predicate" <identifier> "(" [ <pred-param-type> : <identifier> "," ... ] ")" ";"
/// ```
fn predicate_item<Identifier>(input: &mut Stream<'_, '_, Identifier>) -> Result<()>
where
	Identifier: Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	(
		token("predicate"),
		token(identifier::<Identifier>),
		delimited_list("(", predicate_parameter::<Identifier>, ")"),
		token(";"),
	)
		.map(|_| ())
		.parse_next(input)
}

/// Parse a predicate parameter.
///
/// Has no named equivalent in the FlatZinc grammar.
///
/// ```bnf
/// <pred-param-type> ":" <identifier>
/// ```
fn predicate_parameter<Identifier>(input: &mut Stream<'_, '_, Identifier>) -> Result<()>
where
	Identifier: Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	separated_pair(
		token(predicate_parameter_type),
		token(":"),
		token(identifier::<Identifier>),
	)
	.map(|_| ())
	.parse_next(input)
}

/// Parse a predicate parameter type.
///
/// ```bnf
/// <pred-param-type> ::= <basic-pred-param-type>
///                     | "array" "[" <pred-index-set> "]" "of" <basic-pred-param-type>
///
/// <basic-pred-param-type> ::= <basic-par-type>
///                           | <basic-var-type>
///                           | <int-literal> ".." <int-literal>
///                           | <float-literal> ".." <float-literal>
///                           | "{" <int-literal> "," ... "}"
///                           | "set" "of" "float"
///                           | "set" "of" <set-float-literal>
///                           | "set" "of" <set-int-literal>
/// ```
fn predicate_parameter_type<I>(input: &mut Stream<'_, '_, I>) -> Result<()>
where
	I: Debug,
{
	fn basic_predicate_parameter_type<I>(input: &mut Stream<'_, '_, I>) -> Result<()>
	where
		I: Debug,
	{
		alt((
			basic_parameter_type.map(|_| ()),
			(token("set"), token("of"), token("float")).map(|_| ()),
			preceded(token("var"), basic_variable_type).map(|_| ()),
			set(int).map(|_| ()),
			interval_set(float).map(|_| ()),
			preceded((token("set"), token("of"), token("int")), set(int)).map(|_| ()),
			preceded((token("set"), token("of"), token("float")), set(float)).map(|_| ()),
		))
		.parse_next(input)
	}

	alt((
		basic_predicate_parameter_type,
		(
			token("array"),
			delimited(
				token("["),
				alt((
					token("int").map(|_| ()),
					token(interval_set(int)).map(|_| ()),
				)),
				token("]"),
			),
			token("of"),
			basic_predicate_parameter_type,
		)
			.map(|_| ()),
	))
	.parse_next(input)
}

/// Read a single FlatZinc statement, stopping at a `;` that is outside of
/// comments.
fn read_statement(source: &mut impl BufRead, buffer: &mut Vec<u8>) -> Result<(), FznParseError> {
	buffer.clear();

	enum CommentState {
		Normal,
		Line,
		Block,
		BlockStar,
	}
	let mut state = CommentState::Normal;
	let mut index = 0;

	loop {
		let read = source.read_until(b';', buffer)?;
		if read == 0 {
			return Ok(());
		}

		while index < buffer.len() {
			let byte = buffer[index];

			match state {
				CommentState::Normal => match byte {
					b'%' => {
						state = CommentState::Line;
						index += 1;
					}
					b'/' if index + 1 < buffer.len() && buffer[index + 1] == b'*' => {
						state = CommentState::Block;
						index += 2;
					}
					b';' => return Ok(()),
					_ => index += 1,
				},
				CommentState::Line => {
					if byte == b'\n' {
						state = CommentState::Normal;
					}
					index += 1;
				}
				CommentState::Block => {
					if byte == b'*' {
						state = CommentState::BlockStar;
					}
					index += 1;
				}
				CommentState::BlockStar => {
					state = if byte == b'/' {
						CommentState::Normal
					} else if byte == b'*' {
						CommentState::BlockStar
					} else {
						CommentState::Block
					};
					index += 1;
				}
			}
		}

		if buffer.last() != Some(&b';') {
			return Ok(());
		}
	}
}

/// Parse a solve item.
///
/// ```bnf
/// <solve-item> ::= "solve" <annotations> "satisfy" ";"
///                | "solve" <annotations> "minimize" <basic-expr> ";"
///                | "solve" <annotations> "maximize" <basic-expr> ";"
/// ```
fn solve_objective<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<SolveObjective<Identifier>>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	(
		token("solve"),
		general_annotations,
		alt((
			token("satisfy").map(|_| Method::Satisfy),
			preceded(
				token("minimize"),
				token(identifier.map(Literal::Identifier)),
			)
			.map(Method::Minimize),
			preceded(
				token("maximize"),
				token(identifier.map(Literal::Identifier)),
			)
			.map(Method::Maximize),
		)),
		token(";"),
	)
		.map(|(_, ann, method, _)| SolveObjective { method, ann })
		.parse_next(input)
}

/// Parse a variable model item.
fn variable<Identifier>(
	input: &mut Stream<'_, '_, Identifier>,
) -> Result<(Identifier, Variable<Identifier>, bool)>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
{
	(
		token("var"),
		token(basic_variable_type),
		token(":"),
		token(identifier),
		variable_annotations,
		opt(preceded(token("="), token(literal))),
		token(";"),
	)
		.map(|(_, ty, _, name, (flags, ann), value, _)| {
			(
				name,
				Variable {
					ty,
					value,
					ann,
					defined: flags.defined,
					introduced: flags.introduced,
				},
				flags.output,
			)
		})
		.parse_next(input)
}

#[cfg(test)]
mod tests {
	use std::{
		collections::HashMap,
		fmt::Debug,
		fs::File,
		io::{BufReader, Cursor},
		path::PathBuf,
		str::FromStr,
	};

	use rangelist::RangeList;
	use ustr::Ustr;
	use winnow::{error::ParserError, Parser, Stateful};

	use crate::{
		fzn::{
			array_item, constraint, parameter_item, predicate_item, solve_objective, variable,
			ParseState, Stream,
		},
		Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, Argument, Array,
		Constraint, FlatZinc, FznParseError, Literal, Method, SolveObjective, Type, Variable,
	};

	#[test]
	fn basic_constraint_with_array_argument() {
		check_parser(
			constraint,
			Constraint {
				id: "all_different".into(),
				args: vec![Argument::Array(vec![
					Literal::Identifier("x".to_owned()),
					Literal::Identifier("y".to_owned()),
				])],
				defines: None,
				ann: vec![],
			},
			"constraint all_different([x, y]);",
		);
	}

	#[test]
	fn basic_constraint_with_identifier_arguments() {
		check_parser(
			constraint,
			Constraint {
				id: "int_lt".into(),
				args: vec![
					Argument::Literal(Literal::Identifier("x".to_owned())),
					Argument::Literal(Literal::Identifier("y".to_owned())),
				],
				defines: None,
				ann: vec![],
			},
			"constraint int_lt(x, y);",
		);
	}

	#[test]
	fn basic_constraint_with_identifier_arguments_and_annotation() {
		check_parser(
			constraint,
			Constraint {
				id: "int_lt".into(),
				args: vec![
					Argument::Literal(Literal::Identifier("x".to_owned())),
					Argument::Literal(Literal::Identifier("y".to_owned())),
				],
				defines: None,
				ann: vec![Annotation::Atom("domain_consistent".to_owned())],
			},
			"constraint int_lt(x, y) :: domain_consistent;",
		);
	}

	pub(super) fn check_parser<'s, P, O, E>(mut parser: P, expected: O, input: &'s str)
	where
		P: for<'a> Parser<Stream<'s, 'a, String>, O, E>,
		O: Debug + PartialEq,
		E: for<'a> ParserError<Stream<'s, 'a, String>> + Debug + PartialEq,
		for<'a> <E as ParserError<Stream<'s, 'a, String>>>::Inner:
			ParserError<Stream<'s, 'a, String>> + PartialEq + Debug,
	{
		let mut parameters = HashMap::default();

		let stream = Stateful {
			input,
			state: ParseState {
				parameters: &mut parameters,
			},
		};

		let parsed = parser.parse(stream);
		assert_eq!(Ok(expected), parsed);
	}

	#[test]
	fn constraint_defines_var_annotation_is_promoted() {
		check_parser(
			constraint,
			Constraint {
				id: "bool2int".into(),
				args: vec![
					Argument::Literal(Literal::Identifier("b".to_owned())),
					Argument::Literal(Literal::Identifier("x".to_owned())),
				],
				defines: Some("x".to_owned()),
				ann: vec![],
			},
			"constraint bool2int(b, x) :: defines_var(x);",
		);
	}

	#[test]
	fn constraint_keeps_non_semantic_annotations_after_promotion() {
		check_parser(
			constraint,
			Constraint {
				id: "int_lin_eq".into(),
				args: vec![
					Argument::Array(vec![Literal::Int(400), Literal::Int(450), Literal::Int(-1)]),
					Argument::Array(vec![
						Literal::Identifier("b".to_owned()),
						Literal::Identifier("c".to_owned()),
						Literal::Identifier("obj".to_owned()),
					]),
					Argument::Literal(Literal::Int(0)),
				],
				defines: Some("obj".to_owned()),
				ann: vec![Annotation::Atom("ctx_pos".to_owned())],
			},
			"constraint int_lin_eq([400, 450, -1], [b, c, obj], 0) :: defines_var(obj) :: ctx_pos;",
		);
	}

	#[test]
	fn introduced_array_of_variables() {
		check_parser(
			array_item,
			(
				"X_INTRODUCED_1_".to_owned(),
				Array {
					contents: vec![
						Literal::Identifier("x".to_owned()),
						Literal::Identifier("y".to_owned()),
						Literal::Identifier("z".to_owned()),
					],
					ann: vec![],
					defined: false,
					introduced: true,
				},
				false,
			),
			"array [1..3] of var int: X_INTRODUCED_1_ ::var_is_introduced  = [x,y,z];",
		);
	}

	#[test]
	fn output_array_annotation_is_promoted() {
		check_parser(
			array_item,
			(
				"xs".to_owned(),
				Array {
					contents: vec![
						Literal::Identifier("x".to_owned()),
						Literal::Identifier("y".to_owned()),
					],
					ann: vec![],
					defined: false,
					introduced: true,
				},
				true,
			),
			"array [1..2] of var int: xs :: output_array([1..2]) :: var_is_introduced = [x, y];",
		);

		let fzn = FlatZinc::<String>::from_fzn(Cursor::new(
			"array [1..2] of var int: xs :: output_array([1..2]) = [x, y];\nsolve satisfy;",
		))
		.expect("failed to parse output array model");
		assert_eq!(fzn.output, vec!["xs".to_owned()]);
	}

	#[test]
	fn output_variable_annotation_is_promoted() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: None,
					ann: vec![],
					defined: false,
					introduced: true,
				},
				true,
			),
			"var int: x :: output_var :: var_is_introduced;",
		);

		let fzn =
			FlatZinc::<String>::from_fzn(Cursor::new("var int: x :: output_var;\nsolve satisfy;"))
				.expect("failed to parse output variable model");
		assert_eq!(fzn.output, vec!["x".to_owned()]);
	}

	#[test]
	fn parse_allows_block_comments() {
		let fzn = FlatZinc::<String>::from_fzn(Cursor::new(
			"/* leading block comment with ; inside */\nvar int: x;\nconstraint int_eq(x, x) /* inline block ; comment */;\nsolve satisfy;",
		))
		.expect("failed to parse model with block comments");

		assert!(fzn.variables.contains_key("x"));
		assert_eq!(fzn.constraints.len(), 1);
		assert_eq!(fzn.solve.method, Method::Satisfy);
	}

	#[test]
	fn parse_allows_percent_line_comments() {
		let fzn = FlatZinc::<String>::from_fzn(Cursor::new(
			"% model header\nvar int: x; % trailing declaration comment\n% before solve\nsolve satisfy;",
		))
		.expect("failed to parse model with line comments");

		assert!(fzn.variables.contains_key("x"));
		assert_eq!(fzn.solve.method, Method::Satisfy);
	}

	#[test]
	fn parse_rejects_constraints_after_solve() {
		let error =
			FlatZinc::<String>::from_fzn(Cursor::new("solve satisfy;\nconstraint int_eq(x, x);"))
				.expect_err("expected parse to reject constraints after solve");
		assert!(matches!(error, FznParseError::SyntaxError(_)));
	}

	#[test]
	fn parse_rejects_declarations_after_constraints() {
		let error = FlatZinc::<String>::from_fzn(Cursor::new(
			"constraint int_eq(x, x);\nvar int: x;\nsolve satisfy;",
		))
		.expect_err("expected parse to reject declarations after constraints");
		assert!(matches!(error, FznParseError::SyntaxError(_)));
	}

	#[test]
	fn parse_rejects_multiple_solve_items() {
		let error = FlatZinc::<String>::from_fzn(Cursor::new("solve satisfy;\nsolve minimize x;"))
			.expect_err("expected parse to reject multiple solve items");
		assert!(matches!(error, FznParseError::MultipleSolveItems));
	}

	#[test]
	fn parse_rejects_predicates_after_declarations() {
		let error = FlatZinc::<String>::from_fzn(Cursor::new(
			"var int: x;\npredicate p(var int: y);\nsolve satisfy;",
		))
		.expect_err("expected parse to reject predicates after declarations");
		assert!(matches!(error, FznParseError::SyntaxError(_)));
	}

	#[test]
	fn parse_reports_identifier_parse_errors() {
		#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
		struct XIdentifier(String);

		impl FromStr for XIdentifier {
			type Err = &'static str;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				if s.starts_with('x') {
					Ok(Self(s.to_owned()))
				} else {
					Err("identifier must start with x")
				}
			}
		}

		let error = FlatZinc::<XIdentifier>::from_fzn(Cursor::new("var int: y;\nsolve satisfy;"))
			.expect_err("expected parse to reject unsupported identifiers");

		assert!(matches!(error, FznParseError::SyntaxError(_)));
	}

	#[test]
	fn parse_supports_custom_identifier_types() {
		let fzn = FlatZinc::<Ustr>::from_fzn(Cursor::new(
			"var int: x :: output_var;\nconstraint int_eq(x, x);\nsolve satisfy;",
		))
		.expect("failed to parse model with Ustr identifiers");

		assert!(fzn.variables.contains_key(&Ustr::from("x")));
		assert_eq!(fzn.output, vec![Ustr::from("x")]);
		assert_eq!(fzn.constraints[0].id, Ustr::from("int_eq"));
	}

	#[test]
	fn parse_supports_custom_map_types() {
		type HashMapFzn =
			FlatZinc<String, HashMap<String, Variable<String>>, HashMap<String, Array<String>>>;

		let fzn = HashMapFzn::from_fzn(Cursor::new(
			"var int: x;\narray [1..2] of var int: xs = [x, x];\nsolve satisfy;",
		))
		.expect("failed to parse model into HashMap-backed maps");

		assert!(fzn.variables.contains_key("x"));
		assert!(fzn.arrays.contains_key("xs"));
	}

	#[test]
	fn predicate_items_are_parsed_but_ignored() {
		check_parser(
			predicate_item::<String>,
			(),
			"predicate array_int_minimum(var int: m,array [int] of var int: x);",
		);
		check_parser(
			predicate_item::<String>,
			(),
			"predicate my_float_set_in(var float: x,set of float: y);",
		);
	}

	#[test]
	fn run_integration_tests() {
		let flatzinc_file_prefix =
			PathBuf::from(format!("{}/corpus/fzn/", env!("CARGO_MANIFEST_DIR")));

		let dir_iterator = flatzinc_file_prefix
			.read_dir()
			.expect("failed to iterate corpus");

		for file in dir_iterator {
			let file = file.expect("failed to read path from corpus iterator");

			let fzn_file_path = file.path();
			if fzn_file_path.extension().is_none_or(|ext| ext != "fzn") {
				// Only read fzn files.
				continue;
			}

			let fzn_file = File::open(file.path()).expect("failed to open FZN file");
			let fzn_reader = BufReader::new(fzn_file);
			let actual = match FlatZinc::<String>::from_fzn(fzn_reader) {
				Ok(fzn) => fzn,
				Err(error) => panic!(
					"failed to parse file '{}': {}",
					file.path().file_name().unwrap().display(),
					error
				),
			};

			let expected_path = file.path().with_extension("expected");
			let expected = expect_test::expect_file![expected_path];

			expected.assert_eq(&actual.to_string());
		}
	}

	#[test]
	fn solve_optimize() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Minimize(Literal::Identifier("w".to_owned())),
				ann: vec![],
			},
			"solve minimize w;",
		);

		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Maximize(Literal::Identifier("w".to_owned())),
				ann: vec![],
			},
			"solve maximize w;",
		);
	}

	#[test]
	fn solve_satisfy() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Satisfy,
				ann: vec![],
			},
			"solve satisfy;",
		);
	}

	#[test]
	fn solve_with_annotations() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Satisfy,
				ann: vec![Annotation::Call(AnnotationCall {
					id: "int_search".to_owned(),
					args: vec![
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("xs".to_owned()),
						)),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("input_order".to_owned()),
						)),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("indomain_min".to_owned()),
						)),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("complete".to_owned()),
						)),
					],
				})],
			},
			"solve :: int_search(xs, input_order, indomain_min, complete) satisfy;",
		);

		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Maximize(Literal::Identifier("x".to_owned())),
				ann: vec![Annotation::Call(AnnotationCall {
					id: "int_search".to_owned(),
					args: vec![
						AnnotationArgument::Array(vec![
							AnnotationLiteral::BaseLiteral(Literal::Identifier("x".to_owned())),
							AnnotationLiteral::BaseLiteral(Literal::Identifier("y".to_owned())),
							AnnotationLiteral::BaseLiteral(Literal::Identifier("z".to_owned())),
						]),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("first_fail".to_owned()),
						)),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("indomain_split".to_owned()),
						)),
						AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(
							Literal::Identifier("complete".to_owned()),
						)),
					],
				})],
			},
			"solve :: int_search([x, y, z], first_fail, indomain_split, complete) maximize x;",
		);
	}

	#[test]
	fn some_parameter_array_items() {
		check_parser(
			array_item,
			(
				"some_param".to_owned(),
				Array {
					contents: vec![Literal::Int(5), Literal::Int(3), Literal::Int(10)],
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"array [1..3] of int: some_param = [5, 3, 10];",
		);
		check_parser(
			array_item,
			(
				"X_INTRODUCED_4_".to_owned(),
				Array {
					contents: vec![Literal::Int(-1), Literal::Int(1)],
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"array [1..2] of int: X_INTRODUCED_4_ = [-1,1];",
		);
	}

	#[test]
	fn some_parameter_items() {
		check_parser(
			parameter_item,
			("some_param".to_owned(), Literal::Int(5)),
			"int: some_param = 5;",
		);
		check_parser(
			parameter_item,
			("some_param".to_owned(), Literal::Bool(true)),
			"bool: some_param = true;",
		);
		check_parser(
			parameter_item,
			("some_param".to_owned(), Literal::Float(35.3)),
			"float: some_param = 35.3;",
		);
	}

	#[test]
	fn variable_introduced_and_or_defined() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: None,
					ann: vec![],
					defined: false,
					introduced: true,
				},
				false,
			),
			"var int: x :: var_is_introduced;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: None,
					ann: vec![],
					defined: true,
					introduced: false,
				},
				false,
			),
			"var int: x :: is_defined_var;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Bool,
					value: None,
					ann: vec![],
					defined: true,
					introduced: true,
				},
				false,
			),
			"var bool: x :: is_defined_var :: var_is_introduced;",
		);
	}

	#[test]
	fn variable_with_annotation() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: Some(Literal::Int(5)),
					ann: vec![Annotation::Atom("mip".to_owned())],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var int: x :: mip = 5;",
		);
	}

	#[test]
	fn variable_with_assignment() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: Some(Literal::Int(5)),
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var int: x = 5;",
		);
	}

	#[test]
	fn variable_with_bounded_float_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Float(Some(RangeList::from(1.0..=5.5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var 1.0..5.5: x;",
		);
	}

	#[test]
	fn variable_with_bounded_int_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(Some(RangeList::from(1..=5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var 1..5: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(Some(RangeList::from_iter([1..=1, 4..=4, 6..=6]))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var {1, 4, 6}: x;",
		);
	}

	#[test]
	fn variable_with_int_set_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::IntSet(Some(RangeList::from(1..=5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var set of 1..5: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::IntSet(Some(RangeList::from_iter([1..=1, 3..=3]))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var set of {1, 3}: x;",
		);
	}

	#[test]
	fn variable_with_named_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int(None),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var int: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Float(None),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var float: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Bool,
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
				false,
			),
			"var bool: x;",
		);
	}
}
