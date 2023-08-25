/*
TODO:
A.3 Indexing expressions and subscripts
    Indexing 
    Set Expression List
A.4 Expressions
    Expression
    Logical Expressions
    Set Expressions
    Strings (done) (tested)
    Number (done) (tested)
    Identifier (done) (tested)
    Infix Operator
    Unary Operator
    Function Call
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
	union: 8,
	difference: 8,
	intersection: 9,
	crossjoin: 10,
	set_constructor: 11,
	additive: 12,
	arithmetic_reduction: 13,
	multiplicative: 14,
	unary: 15,
	exponetiation: 16,
}

const COMPARISON_OPERATORS = ["==", "!=", "<", "<=", ">", ">=", "<>"]
const MULTIPLICATIVE_OPERATORS = ["*", "/", "mod", "div"]
const ADDITIVE_OPERATORS = ["+", "-", "less"]
const EXPONENTIAL_OPERATORS = ["^", "**"]
const UNARY_OPERATORS = ["+", "-"]
const LOGICAL_NOT = ["not", "!"]

module.exports = grammar({
	name: "ampl",
	extras: ($) => [/\s/, $.line_comment, $.block_comment],
	word: ($) => $.identifier,

	rules: {
		source_file: ($) => sepBy(";", field("item", $._item)),

		_item: ($) => choice($.indexing, $.let_decl),

		indexing: ($) =>
			choice(
				seq("{", $._sexpr_list, "}"),
				seq("{", $._sexpr_list, ":", $._expr, "}")
			),

		_sexpr_list: ($) =>
			choice(
				sepBy1(",", $._expr)
				// seq(field("name", $.identifier), "in", field("set", $._expr)) // Name could he called dummy member
			),

		_expr: ($) =>
			choice(
				$.boolean_literal,
				$.string_literal,
				$.number_literal,
				$.identifier,
				$.infix_operator,
				$.unary_operator,
				// $.function_call,
				$.if_then_else,
				// $.reduction,
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
				[prec.left, PREC.additive, choice(...ADDITIVE_OPERATORS)],
				[prec.left, PREC.multiplicative, choice(...MULTIPLICATIVE_OPERATORS)],
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

		// _decl: ($) =>
		//     seq(
		//         field("name", $.identifier),
		//         optional(field("alias", $.identifier)),
		//         optional(field("indexing",$.indexing))
		//     ),

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
