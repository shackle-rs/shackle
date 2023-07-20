hljs.registerLanguage("MiniZinc", (hljs) => {
	const KEYWORDS = {
		keyword:
			"ann annotation any array bool case constraint default div diff else elseif endcase endif enum float function if include intersect in int lambda let maximize minimize mod not of op opt output par predicate record satisfy set solve string subset superset symdiff test then tuple type union var variant_record where xor",
		literal: "false true <> _",
	}
	const STRING = {
		scope: "string",
		begin: '"',
		end: '"',
		contains: [hljs.BACKSLASH_ESCAPE],
	}
	const NUMBER = {
		className: "number",
		begin: "\\b0x[\\da-f]+\\b|(?:\\b\\d+\\.?\\d*|\\B\\.\\d+)(?:e[+-]?\\d+)?",
	}
	const SUBST = {
		className: "subst",
		begin: "\\\\\\(",
		end: "\\)",
		keywords: KEYWORDS,
		contains: [STRING, NUMBER],
	}
	STRING.contains.push(SUBST)
	return {
		name: "MiniZinc",
		aliases: ["mzn", "dzn", "fzn"],
		keywords: KEYWORDS,
		contains: [
			STRING,
			NUMBER,
			hljs.COMMENT("/\\*", "\\*/"),
			hljs.COMMENT("%", "$"),
		],
	}
})

hljs.initHighlightingOnLoad()
