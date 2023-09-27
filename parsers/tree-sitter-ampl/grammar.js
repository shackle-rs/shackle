/*
TODO:
A.3 Indexing expressions and subscripts | Fields?
A.4 Expressions
    Expression
    Logical Expressions
    Set Expressions
	Infix Operator (done) (tested)
    Strings (done) (tested)
    Number (done) (tested)
    Identifier (done) (tested)
	Set Constructors
    Unary Operator (done) (tested)
    Function Call (done) (tested)
    if-then-else (done)
A.4.2 Builtins
    Concat &
    Printf
    Match, Sub, GSub
A.4.3 Piecewise-linear terms
A.6 Declarations of model entities
    Set declarations
    Set Attribute
A.6.1 Cardinality
    Cardinality
    Ordered/Circular Set
A.6.3 Intervals
    Interval
    Function Prototypes?
A.7 Parameter Declaration
    Parameter Declaration
    Parameter Attribute
    Relop?
A.7.1 Check
A.7.2 Infinity
A.8 Variable Declaration
    Variable Declaration
    Variable Attribute
A.8.1 Defined Variables
A.9 Constraint Declaration
    Costriant Declaration
    Constraint expression
    Constraint Expression
    Suffixe Initialization
A.9.1 Complementarity Constraints
    Complements
A.10 Objective Declaration
    Objective Declaration
    Objective Expression
A.11 Suffix notation for auxiliary values
A.11.1 Suffix declarations
A.11.2 Statuses
A.12.1 Set data
A.12.2 Parameter data
A.18.9 Modiyfing the data
    reset
    update
    let (done)
*/

const PREC = {
	if_then_else: 1,
	logicalOr: 3,
	logical_reduction: 3,
	logicalAnd: 4,
	comparitive: 5,
	set_in: 6,
	set_within: 6,
	negation: 7,
	set_union_diff: 8,
	intersection: 9,
	crossjoin: 10,
	set_constructor: 11,
	additive: 12,
	arithmetic_reduction: 13,
	multiplicative: 14,
	unary: 15,
	exponetiation: 16,
}

const CONJUNCTION_OPERATORS = ["and", "&&"]
const DISJUNCTION_OPERATORS = ["or", "||"]
const SET_OPERATORS = ["union", "diff", "symdiff"]
const COMPARISON_OPERATORS = ["=", "==", "!=", "<", "<=", ">", ">=", "<>"]
const IN_OPERATORS = ["in", "not in"]
const WITHIN_OPERATORS = ["within", "not within"]
const MULTIPLICATIVE_OPERATORS = ["*", "/", "mod", "div"]
const ADDITIVE_OPERATORS = ["+", "-", "less"]
const EXPONENTIAL_OPERATORS = ["^", "**"]
const UNARY_OPERATORS = ["+", "-"]
const LOGICAL_NOT = ["not", "!"]
const ARITHMETIC_REDUCTION_OPERATORS = ["sum", "prod", "min", "max"]
const LOGICAL_REDUCTION_OPERATORS = ["exists", "forall"]
const SET_RANGE_OPERATORS = ["..", "by"]

module.exports = grammar({
	name: "ampl",
	extras: ($) => [/\s/, $.line_comment, $.block_comment],
	word: ($) => $.identifier,

	rules: {
		source_file: ($) => sepBy(";", field("item", $._item)),

		_item: ($) => 
			choice(
				$.indexing, 
				$._declaration
			),

		indexing: ($) =>
			choice(
				seq("{", $._expr_list, "}"),
				seq("{", $._expr_list, ":", $._expr, "}")
			),

		_expr_list: ($) =>
			sepBy1(",", $._expr),

		_expr: ($) =>
			choice(
				$.boolean_literal,
				$.string_literal,
				$.number_literal,
				$.identifier,
				$.infix_operator,
				$.unary_operator,
				$.function_call,
				$.if_then_else,
				$.reduction,
				// $.set_constructor,
				// $.interval,
				// set {3,12,3,etc}
				$.indexing,
				seq("(", $._expr, ")")
			),

		unary_operator: ($) =>
			choice(
				prec(
					PREC.unary,
					seq(
						field("operator", choice(...UNARY_OPERATORS)),
						field("operand", $._expr)
					)
				),
				prec(
					PREC.negation,
					seq(
						field("operator", choice(...LOGICAL_NOT)),
						field("operand", $._expr)
					)
				)
			),

		infix_operator: ($) => {
			const table = [
				[prec.left, PREC.logicalOr, choice(...DISJUNCTION_OPERATORS)],
				[prec.left, PREC.logicalAnd, choice(...CONJUNCTION_OPERATORS)],
				[prec.left, PREC.comparitive, choice(...COMPARISON_OPERATORS)],
				[prec.left, PREC.set_in, choice(...IN_OPERATORS)],
				[prec.left, PREC.set_within, choice(...WITHIN_OPERATORS)],
				[prec.left, PREC.set_union_diff, choice(...SET_OPERATORS)],
				[prec.left, PREC.intersection, "inter"],
				[prec.left, PREC.crossjoin, "cross"],
				[prec.left, PREC.additive, choice(...ADDITIVE_OPERATORS)],
				[prec.left, PREC.additive, choice(...ADDITIVE_OPERATORS)],
				[prec.left, PREC.multiplicative, choice(...MULTIPLICATIVE_OPERATORS)],
				[prec.right, PREC.exponetiation, choice(...EXPONENTIAL_OPERATORS)],
				[prec.left, PREC.set_constructor, choice(...SET_RANGE_OPERATORS)],
				[prec.left, PREC.set_constructor, "&"] // Concat "precedence below arithmetic operators"
			]
			return choice(
				...table.map(([assoc, precedence, operator]) =>
					assoc(
						precedence,
						seq(
							field("left", $._expr),
							field("operator", operator),
							field("right", $._expr)
						)
					)
				)
			)
		},

		function_call: ($) =>
			seq(
				field("function", $.identifier),
				"(",
				sepBy(",", field("argument", $._expr)),
				")"
			),

		_declaration: ($) =>
			choice(
				$.let_decl
			),

		_decl: ($) =>
		    seq(
		        field("name", $.identifier),
		        optional(field("alias", $.identifier)),
		        optional(field("indexing",$.indexing))
		    ),

		let_decl: ($) =>
			choice(
				seq(
					"let",
					optional(field("indexing", $.indexing)),
					field("name", $.identifier),
					":=",
					$._expr
				)
				// Suffix Version
			),

		if_then_else: ($) =>
			prec.right(
				seq(
					"if",
					field("condition", $._expr),
					"then",
					field("result", $._expr),
					optional(seq("else", field("else", $._expr)))
				)
			),

		reduction: ($) => {
			const reducers = [
				[PREC.arithmetic_reduction, choice(...ARITHMETIC_REDUCTION_OPERATORS)],
				[PREC.logical_reduction, choice(...LOGICAL_REDUCTION_OPERATORS)],
				[PREC.intersection, "inter"],
				[PREC.set_union_diff, "union"],
				[PREC.set_constructor, "setof"]
			]
			return choice(
				...reducers.map(([precedence, operator]) =>
					prec.left(precedence, // I think left associativity should be correct
						seq(
							field("operator", operator),
							field("indexing", $.indexing),
							field("expression", $._expr)
						)
					)
				))
			},
		number_literal: ($) =>
			token(choice(/[0-9]+(\.[0-9]+)?((d|D|e|E)-?[0-9]+)?/)),
		boolean_literal: ($) => choice("true", "false"),
		string_literal: ($) =>
			choice(
				seq("'", optional($._string_content), "'"),
				seq('"', optional($._string_content), '"')
			),
		_string_content: ($) =>
			repeat1(field("content", choice($.string_characters, $.escape_sequence))),
		string_characters: ($) => token.immediate(prec(1, /[^"\n\\]+/)),
		escape_sequence: ($) => {
			const simpleEscape = [
				["\\'", "'"],
				['\\"', '"'],
				["\\\\", "\\"],
				["\\r", "\r"],
				["\\n", "\n"],
				["\\t", "\t"],
			]
			return choice(
				field("escape", choice(...simpleEscape.map(([e, v]) => alias(e, v)))),
				seq("\\", field("escape", alias(/[0-7]{1,3}/, "octal"))),
				seq("\\x", field("escape", alias(/[0-9a-fA-F]{2}/, "hexadecimal"))),
				seq("\\u", field("escape", alias(/[0-9a-fA-F]{4}/, "hexadecimal"))),
				seq("\\U", field("escape", alias(/[0-9a-fA-F]{8}/, "hexadecimal")))
			)
		},

		line_comment: ($) => token(seq("#", /.*/)),
		block_comment: ($) => token(seq("/*", /([^*]|\*[^\/]|\n)*?\*?/, "*/")),
		identifier: ($) => /[A-Za-z][A-Za-z0-9_]*/,
	},
})

function sepBy(sep, rule) {
	return seq(repeat(seq(rule, sep)), optional(rule))
}

function sepBy1(sep, rule) {
	return seq(rule, repeat(seq(sep, rule)), optional(sep))
}
