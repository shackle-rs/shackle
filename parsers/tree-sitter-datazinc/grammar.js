const mzn = require("tree-sitter-minizinc/grammar")

module.exports = grammar(mzn, {
	name: "datazinc",

	rules: {
		source_file: ($) => sepBy(";", field("item", $.assignment)),

		_expression: ($) =>
			choice(
				$._identifier,
				$.absent,
				$.array_literal_2d,
				$.array_literal,
				$.boolean_literal,
				$.call,
				$.float_literal,
				$.infinity,
				$.infix_operator,
				$.integer_literal,
				$.record_literal,
				$.set_literal,
				$.string_literal,
				$.tuple_literal
			),

		call: ($) =>
			seq(
				field("function", $._identifier),
				"(",
				sepBy(",", field("argument", $._call_arg)),
				")"
			),
		_call_arg: ($) =>
			choice(
				$._identifier,
				$.call,
				$.infix_operator,
				$.integer_literal,
				$.set_literal
			),

		infix_operator: ($) => {
			const table = [
				[prec.left, 10, ".."], // PREC.range
				[prec.left, 12, "++"], // PREC.additive
				[prec.left, 7, choice("union", "∪")], // PREC.union
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

		float_literal: ($, orig) => token(seq(optional("-"), orig)),
		integer_literal: ($, orig) => token(seq(optional("-"), orig)),
		infinity: ($, orig) => token(seq(optional("-"), orig)),
	},
})

function sepBy(sep, rule) {
	return seq(repeat(seq(rule, sep)), optional(rule))
}
