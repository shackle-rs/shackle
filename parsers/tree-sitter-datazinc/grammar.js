const mzn = require("tree-sitter-minizinc/grammar")

module.exports = grammar(mzn, {
	name: "datazinc",

	rules: {
		source_file: ($) => sepBy(";", field("item", $.assignment)),

		_expression: ($) =>
			choice(
				$.absent,
				$.array_literal_2d,
				$.array_literal,
				$.boolean_literal,
				$.float_literal,
				$.identifier,
				$.infinity,
				$.integer_literal,
				$.set_literal,
				$.string_literal,
				$.tuple_literal,
				$.record_literal
			),
	},
})

function sepBy(sep, rule) {
	return seq(repeat(seq(rule, sep)), optional(rule))
}
