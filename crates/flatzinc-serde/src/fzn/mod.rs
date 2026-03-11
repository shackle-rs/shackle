//! Parse the original `.fzn` file format.

mod annotations;
mod error;
mod primitives;

use std::{collections::BTreeMap, io::BufRead};

use winnow::{
	combinator::{alt, delimited, opt, preceded, repeat, separated, separated_pair},
	Parser, Result,
};

use annotations::*;
pub use error::*;
use primitives::*;

use crate::{
	Annotation, Argument, Array, Constraint, Domain, FlatZinc, Literal, Method, SolveObjective,
	Type, Variable,
};

/// Parse the `.fzn` source to a [`FlatZinc`] instance.
///
/// # Example
/// ```
/// # use std::collections::BTreeMap;
/// # use flatzinc_serde::Argument;
/// # use flatzinc_serde::Constraint;
/// # use flatzinc_serde::Domain;
/// # use flatzinc_serde::FlatZinc;
/// # use flatzinc_serde::Literal;
/// # use flatzinc_serde::Method;
/// # use flatzinc_serde::RangeList;
/// # use flatzinc_serde::SolveObjective;
/// # use flatzinc_serde::Type;
/// # use flatzinc_serde::Variable;
///
/// // The FlatZinc model.
/// //
/// // Typically this would be a file, but for this example we will use an inline string.
/// let source = r#"
/// var 1..5: x;
/// var 1..5: y;
///
/// constraint int_le(x, y);
///
/// solve satisfy;
/// "#;
///
/// // Parse the model to the in-memory representation.
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
///    constraints: vec![Constraint {
///        id: "int_le".to_owned(),
///        args: vec![
///            Argument::Literal(Literal::Identifier("x".to_owned())),
///            Argument::Literal(Literal::Identifier("y".to_owned())),
///        ],
///        ann: vec![],
///        defines: None,
///    }],
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

	let mut variables = BTreeMap::default();
	let arrays = BTreeMap::default();
	let mut constraints = vec![];
	let output = vec![];
	let mut solve = None;

	loop {
		buffer.clear();
		let _ = source.read_until(b';', &mut buffer)?;

		let statement_str = std::str::from_utf8(&buffer)?.trim();
		if statement_str.is_empty() {
			break;
		}

		match model_item.parse(statement_str)? {
			ModelItem::Predicate => {
				// Ignored.
			}
			ModelItem::Variable((name, variable)) => {
				let _ = variables.insert(name, variable);
			}
			ModelItem::Constraint(constraint) => {
				constraints.push(constraint);
			}
			ModelItem::SolveObjective(solve_objective) => {
				// TODO: For now we assume there is only one per model. For completeness we should
				// really throw an error if `solve` already has a value.
				solve = Some(solve_objective);
			}
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
	/// A predicate item.
	///
	/// Since we ignore them, no data is attached.
	Predicate,
	/// A variable model item.
	Variable((String, Variable)),
	/// A constraint model item.
	Constraint(Constraint),
	/// A solve item.
	SolveObjective(SolveObjective),
}

/// Parse a model item.
fn model_item(input: &mut &str) -> Result<ModelItem> {
	alt((
		predicate_item.map(|_| ModelItem::Predicate),
		variable.map(ModelItem::Variable),
		constraint.map(ModelItem::Constraint),
		solve_objective.map(ModelItem::SolveObjective),
	))
	.parse_next(input)
}

/// Parse a variable model item.
fn variable(input: &mut &str) -> Result<(String, Variable)> {
	(
		token("var"),
		token(basic_variable_type),
		token(":"),
		token(identifier),
		repeat(0.., annotation),
		opt(preceded(token("="), token(literal))),
		token(";"),
	)
		.map(|(_, (ty, domain), _, name, ann, value, _)| {
			(
				name,
				Variable {
					ty,
					domain,
					value,
					ann,
					defined: false,
					introduced: false,
				},
			)
		})
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
fn basic_variable_type(input: &mut &str) -> Result<(Type, Option<Domain>)> {
	alt((
		basic_parameter_type.map(|ty| (ty, None)),
		preceded((token("set"), token("of")), set(int))
			.map(|values| (Type::IntSet, Some(Domain::Int(values)))),
		set(int).map(|values| (Type::Int, Some(Domain::Int(values)))),
		interval_set(float).map(|values| (Type::Float, Some(Domain::Float(values)))),
	))
	.parse_next(input)
}

/// Parse a constraint item.
///
/// ```bnf
/// <constraint-item> ::= "constraint" <identifier> "(" [ <expr> "," ... ] ")" <annotations> ";"
/// ```
fn constraint(input: &mut &str) -> Result<Constraint> {
	(
		token("constraint"),
		token(identifier),
		delimited(
			token("("),
			separated(0.., token(argument), token(",")),
			token(")"),
		),
		repeat(0.., annotation),
		token(";"),
	)
		.map(|(_, id, args, ann, _)| Constraint {
			id,
			args,
			ann,
			defines: None,
		})
		.parse_next(input)
}

/// Parses a constraint argument.
///
/// ```bnf
/// <expr> ::= <basic-expr>
///          | <array-literal>
/// ```
fn argument(input: &mut &str) -> Result<Argument> {
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

/// Parse a solve item.
///
/// ```bnf
/// <solve-item> ::= "solve" <annotations> "satisfy" ";"
///                | "solve" <annotations> "minimize" <basic-expr> ";"
///                | "solve" <annotations> "maximize" <basic-expr> ";"
/// ```
fn solve_objective(input: &mut &str) -> Result<SolveObjective> {
	(
		token("solve"),
		repeat(0.., annotation),
		alt((
			token("satisfy").map(|_| Method::Satisfy),
			token("minimize").map(|_| Method::Minimize),
			token("maximize").map(|_| Method::Maximize),
		)),
		opt(identifier.map(Literal::Identifier)),
		token(";"),
	)
		.map(|(_, ann, method, objective, _)| SolveObjective {
			method,
			objective,
			ann,
		})
		.parse_next(input)
}

/// Parses a predicate item.
///
/// ```bnf
/// <predicate-item> ::= "predicate" <identifier> "(" [ <pred-param-type> : <identifier> "," ... ] ")" ";"
/// ```
fn predicate_item(input: &mut &str) -> Result<()> {
	(
		token("predicate"),
		token(identifier),
		delimited_list("(", predicate_parameter, ")"),
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
fn predicate_parameter(input: &mut &str) -> Result<()> {
	separated_pair(
		token(predicate_parameter_type),
		token(":"),
		token(identifier),
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
///                           | "set" "of" <int-literal> .. <int-literal>
///                           | "set" "of" "{" [  <int-literal> "," ... ] "}"
/// ```
fn predicate_parameter_type(input: &mut &str) -> Result<()> {
	fn basic_predicate_parameter_type(input: &mut &str) -> Result<()> {
		alt((
			basic_parameter_type.map(|_| ()),
			preceded(token("var"), basic_variable_type).map(|_| ()),
			set(int).map(|_| ()),
			interval_set(float).map(|_| ()),
			preceded((token("set"), token("of"), token("int")), set(int)).map(|_| ()),
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

/// Parse a basic parameter type.
///
/// ```bnf
/// <basic-par-type> ::= "bool"
///                    | "int"
///                    | "float"
///                    | "set of int"
/// ```
fn basic_parameter_type(input: &mut &str) -> Result<Type> {
	alt((
		"bool".map(|_| Type::Bool),
		"int".map(|_| Type::Int),
		"float".map(|_| Type::Float),
		(token("set"), token("of"), token("int")).map(|_| Type::IntSet),
	))
	.parse_next(input)
}

/// Parse a variable array.
fn variable_array(input: &mut &str) -> Result<(String, Array)> {
	(
		token("array"),
		delimited(token("["), interval_set(int), token("]")),
		token("of"),
		preceded(token("var"), basic_variable_type),
		token(":"),
		token(identifier),
		repeat(0.., annotation),
		preceded(token("="), delimited_list("[", literal, "]")),
		token(";"),
	)
		.map(|(_, _, _, _, _, id, ann, contents, _)| {
			let introduced = is_introduced(&ann);

			(
				id,
				Array {
					contents,
					ann,
					defined: false,
					introduced,
				},
			)
		})
		.parse_next(input)
}

/// Determine whether the given list of annotations implies the object is introduced by the
/// MiniZinc compiler.
#[allow(
	clippy::ptr_arg,
	reason = "used in places where the compiler cannot infer the type of `ann`"
)]
fn is_introduced(ann: &Vec<Annotation>) -> bool {
	ann.iter().any(
		|annotation| matches!(annotation, Annotation::Atom(name) if name == "var_is_introduced"),
	)
}

#[cfg(test)]
mod tests {
	use std::{fmt::Debug, fs::File, io::BufReader, path::PathBuf};

	use rangelist::RangeList;
	use winnow::{error::ParserError, Parser};

	use crate::{
		Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, Argument, Array, Domain,
		Method, Type,
	};

	use super::*;

	#[test]
	fn variable_with_named_domain() {
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
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Float,
					domain: None,
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var float: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Bool,
					domain: None,
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var bool: x;",
		);
	}

	#[test]
	fn variable_with_bounded_int_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int,
					domain: Some(Domain::Int(RangeList::from(1..=5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var 1..5: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int,
					domain: Some(Domain::Int(RangeList::from_iter([1..=1, 4..=4, 6..=6]))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var {1, 4, 6}: x;",
		);
	}

	#[test]
	fn variable_with_bounded_float_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Float,
					domain: Some(Domain::Float(RangeList::from(1.0..=5.5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var 1.0..5.5: x;",
		);
	}

	#[test]
	fn variable_with_int_set_domain() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::IntSet,
					domain: Some(Domain::Int(RangeList::from(1..=5))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var set of 1..5: x;",
		);
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::IntSet,
					domain: Some(Domain::Int(RangeList::from_iter([1..=1, 3..=3]))),
					value: None,
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var set of {1, 3}: x;",
		);
	}

	#[test]
	fn variable_with_assignment() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int,
					domain: None,
					value: Some(Literal::Int(5)),
					ann: vec![],
					defined: false,
					introduced: false,
				},
			),
			"var int: x = 5;",
		);
	}

	#[test]
	fn variable_with_annotation() {
		check_parser(
			variable,
			(
				"x".to_owned(),
				Variable {
					ty: Type::Int,
					domain: None,
					value: Some(Literal::Int(5)),
					ann: vec![Annotation::Atom("mip".to_owned())],
					defined: false,
					introduced: false,
				},
			),
			"var int: x :: mip = 5;",
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
	fn solve_satisfy() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Satisfy,
				objective: None,
				ann: vec![],
			},
			"solve satisfy;",
		);
	}

	#[test]
	fn solve_optimize() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Minimize,
				objective: Some(Literal::Identifier("w".to_owned())),
				ann: vec![],
			},
			"solve minimize w;",
		);

		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Maximize,
				objective: Some(Literal::Identifier("w".to_owned())),
				ann: vec![],
			},
			"solve maximize w;",
		);
	}

	#[test]
	fn solve_with_annotations() {
		check_parser(
			solve_objective,
			SolveObjective {
				method: Method::Satisfy,
				objective: None,
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
				method: Method::Maximize,
				objective: Some(Literal::Identifier("x".to_owned())),
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
	fn introduced_array_of_variables() {
		check_parser(
			variable_array,
			(
				"X_INTRODUCED_1_".to_owned(),
				Array {
					contents: vec![
						Literal::Identifier("x".to_owned()),
						Literal::Identifier("y".to_owned()),
						Literal::Identifier("z".to_owned()),
					],
					ann: vec![Annotation::Atom("var_is_introduced".to_owned())],
					defined: false,
					introduced: true,
				},
			),
			"array [1..3] of var int: X_INTRODUCED_1_ ::var_is_introduced  = [x,y,z];",
		);
	}

	#[test]
	fn predicate_items_are_parsed_but_ignored() {
		check_parser(
			predicate_item,
			(),
			"predicate array_int_minimum(var int: m,array [int] of var int: x);",
		);
	}

	pub(super) fn check_parser<'s, O, E>(
		mut parser: impl Parser<&'s str, O, E>,
		expected: O,
		input: &'s str,
	) where
		O: Debug + PartialEq,
		E: ParserError<&'s str> + Debug + PartialEq,
		E::Inner: ParserError<&'s str> + PartialEq + Debug,
	{
		let parsed = parser.parse(input);
		assert_eq!(Ok(expected), parsed);
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
			let actual = match parse(fzn_reader) {
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
}
