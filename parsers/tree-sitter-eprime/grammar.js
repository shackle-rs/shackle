/*
Important Notes:
- The integer domain may need to be modified to differentiate between specific integer domains and ones that require a file.

*/

const PREC = {
	not: 20,
	absolute: 20,
	power: 18,
	negation: 15,
	multiplicative: 10,
	intersect: 2,
	additive: 1,
	union: 1,
	range: 0, // Not in the language specification
	set_in: 0,
	comparative: -1,
	conjunction: -2,
	disjunction: -3,
	implication: -4,
	equivalence: -5,
	quantifiers: -10,
}

const MAX_PREC = Math.max(...Object.values(PREC))

const ADDITIVE_OPERATORS = ["+", "-"]
// prettier-ignore
const COMPARISON_OPERATORS = [
  "=", "!=","<", "<=", ">", ">=",
  "<lex" ,"<=lex", ">=lex", ">lex"
];
const MULTIPLICATIVE_OPERATORS = ["*", "/", "%"]

module.exports = grammar({
	name: "eprime",

	extras: ($) => [/\s/, $.line_comment],

	word: ($) => $.identifier,

	conflicts: ($) => [[$._expression, $.generator]],

	rules: {
		source_file: ($) =>
			seq(
				// Note: this is not optional in the language specification, but this
				// makes the parser a bit more flexible
				optional(field("lang_version", $.lang_version)),
				// Note: some of these items must be in a particular order in the
				// language specification, but this makes the parser a bit more
				// flexible
				repeat(
					field(
						"item",
						choice(
							$.param_decl,
							$.const_def,
							$.domain_alias,
							$.decision_decl,
							$.objective,
							$.branching,
							$.heuristic,
							$.constraint
						)
					)
				)
			),

		lang_version: ($) =>
			seq(
				"language",
				"ESSENCE",
				token.immediate(choice("’", "'")),
				field("version", /\d+.\d+/)
			),

		param_decl: ($) =>
			seq(
				"given",
				sepBy(",", field("name", $.identifier)),
				":",
				field("domain", $._domain),
				optional(seq("where", field("where", $._expression)))
			),

		const_def: ($) =>
			seq(
				"letting",
				field("name", $.identifier),
				optional(seq(":", field("domain", $._domain))),
				choice("=", "be"),
				field("definition", $._expression)
			),

		domain_alias: ($) =>
			seq(
				"letting",
				field("name", $.identifier),
				"be",
				"domain",
				field("definition", $._domain)
			),

		decision_decl: ($) =>
			seq(
				"find",
				sepBy(",", field("name", $.identifier)),
				":",
				field("domain", $._domain)
			),

		objective: ($) =>
			seq(
				field("strategy", choice("maximising", "minimising")),
				field("expression", $._expression)
			),

		branching: ($) =>
			seq(
				"branching",
				"on",
				"[",
				sepBy(",", field("expression", $._expression)),
				"]"
			),

		constraint: ($) =>
			seq("such", "that", sepBy1(",", field("expression", $._expression))),

		heuristic: ($) =>
			seq(
				"heuristic",
				optional(field("heuristic", choice("static", "sdf", "srf", "conflict")))
			),

		_expression: ($) =>
			choice(
				$.boolean_literal,
				$.call,
				$.identifier,
				$.indexed_access,
				$.infix_operator,
				$.integer_literal,
				$.matrix_literal,
				$.prefix_operator,
				$.postfix_operator,
				$.quantification,
				$.matrix_comprehension,
				$.absolute_operator,
				seq("(", $._expression, ")")
			),

		call: ($) =>
			prec(
				MAX_PREC + 1,
				seq(
					field("function", $.identifier),
					"(",
					sepBy(",", field("argument", $._expression)),
					")"
				)
			),

		quantification: ($) =>
			prec(
				PREC.quantifiers,
				seq(
					field("function", $.identifier),
					field("generator", $.generator),
					".",
					field("template", $._expression)
				)
			),

		matrix_comprehension: ($) =>
			seq(
				"[",
				field("template", $._expression),
				"|",
				field("generator", $.generator),
				repeat(seq(",", field("generator", $.generator))),
				repeat(seq(",", field("condition", $._expression))),
				optional(seq(";", field("index", $._base_domain))),
				"]"
			),
		generator: ($) =>
			seq(
				sepBy1(",", field("name", $.identifier)),
				":",
				field("collection", $._domain)
			),

		indexed_access: ($) =>
			prec(
				MAX_PREC + 1,
				seq(
					field("collection", $._expression),
					"[",
					sepBy1(",", field("index", choice("..", $._expression))),
					"]"
				)
			),

		infix_operator: ($) => {
			const table = [
				[prec.right, PREC.power, "**"],
				[prec.left, PREC.multiplicative, choice(...MULTIPLICATIVE_OPERATORS)],
				[prec.left, PREC.additive, choice(...ADDITIVE_OPERATORS)],
				[prec.left, PREC.comparative, choice(...COMPARISON_OPERATORS)],
				[prec.left, PREC.conjunction, "/\\"],
				[prec.left, PREC.disjunction, "\\/"],
				[prec.left, PREC.implication, "->"],
				[prec.left, PREC.equivalence, "<->"],
				[prec.left, PREC.implication, "=>"],
				[prec.left, PREC.equivalence, "<=>"],
				[prec.left, PREC.set_in, "in"],
				[prec.left, PREC.range, ".."],
			]

			return choice(
				...table.map(([assoc, precedence, operator]) =>
					assoc(
						precedence,
						seq(
							field("left", $._expression),
							field("operator", operator),
							field("right", $._expression)
						)
					)
				)
			)
		},

		absolute_operator: ($) =>
			prec(PREC.absolute, seq("|", field("operand", $._expression), "|")),

		prefix_operator: ($) => {
			const table = [
				[PREC.not, "!"],
				[PREC.negation, "-"],
			]

			return choice(
				...table.map(([precedence, operator]) =>
					prec.left(
						precedence,
						seq(field("operator", operator), field("operand", $._expression))
					)
				),
				prec.left(
					PREC.range,
					seq(field("operator", ".."), field("operand", $._expression))
				)
			)
		},

		postfix_operator: ($) =>
			prec.right(
				PREC.range,
				seq(field("operand", $._expression), field("operator", ".."))
			),

		_domain: ($) => choice($._base_domain, $.matrix_domain),

		matrix_domain: ($) =>
			seq(
				"matrix",
				"indexed",
				"by",
				"[",
				sepBy1(",", field("index", $._base_domain)),
				"]",
				"of",
				field("base", $._base_domain)
			),

		_base_domain: ($) =>
			choice(
				$.boolean_domain,
				$.integer_domain,
				$.domain_operation,
				$.identifier
			),

		domain_operation: ($) => {
			const table = [
				[prec.left, PREC.intersect, "intersect"],
				[prec.left, PREC.union, "union"],
				[prec.left, PREC.additive, "-"],
			]

			return choice(
				...table.map(([assoc, precedence, operator]) =>
					assoc(
						precedence,
						seq(
							field("left", $._base_domain),
							field("operator", operator),
							field("right", $._base_domain)
						)
					)
				)
			)
		},

		boolean_domain: ($) => "bool",
		integer_domain: ($) =>
			seq(
				"int",
				optional(seq("(", sepBy(",", field("member", $._expression)), ")"))
			),

		matrix_literal: ($) =>
			seq(
				"[",
				sepBy(",", field("member", $._expression)),
				optional(seq(";", field("index", $._base_domain))),
				"]"
			),

		boolean_literal: ($) => choice("true", "false"),
		integer_literal: ($) => /\d+/,

		identifier: ($) => /[A-Za-z][A-Za-z0-9_]*/,

		line_comment: ($) => token(seq("$", /.*/)),
	},
})

function sepBy(sep, rule) {
	return seq(repeat(seq(rule, sep)), optional(rule))
}

function sepBy1(sep, rule) {
	return seq(rule, repeat(seq(sep, rule)), optional(sep))
}
