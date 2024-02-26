//! Code formatting for MiniZinc

#![warn(missing_docs)]

use format::Format;
pub use ir::FormatOptions;
use shackle_compiler::syntax::{cst, minizinc::MznModel};
use tree_sitter::Parser;

use crate::format::MiniZincFormatter;

pub(crate) mod container;
pub(crate) mod expression;
pub(crate) mod format;
pub(crate) mod ir;
pub(crate) mod item;
pub(crate) mod pattern;
pub(crate) mod types;

/// Formatting options for MiniZinc
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct MiniZincFormatOptions {
	/// Core formatting options
	pub core: FormatOptions,
	/// Keep parentheses (except double parentheses)
	pub keep_parentheses: bool,
}

/// Format the given source code
pub fn format(source: &str, options: &MiniZincFormatOptions) -> Option<String> {
	let mut parser = Parser::new();
	parser
		.set_language(&tree_sitter_minizinc::language())
		.unwrap();
	let tree = parser.parse(source.as_bytes(), None).unwrap();
	let cst = cst::Cst::from_str(tree, source);
	format_model(&MznModel::new(cst), options)
}

/// Format an AST model
pub fn format_model(model: &MznModel, options: &MiniZincFormatOptions) -> Option<String> {
	if model.cst().error_nodes().next().is_some() {
		return None;
	}
	Some(MiniZincFormatter::new(model, options).format())
}

/// Get IR for debugging
pub fn format_debug(source: &str, options: &MiniZincFormatOptions) -> Option<String> {
	let mut parser = Parser::new();
	parser
		.set_language(&tree_sitter_minizinc::language())
		.unwrap();
	let tree = parser.parse(source.as_bytes(), None).unwrap();
	let cst = cst::Cst::from_str(tree, source);
	format_model_debug(&MznModel::new(cst), options)
}

/// Get IR for debugging
pub fn format_model_debug(model: &MznModel, options: &MiniZincFormatOptions) -> Option<String> {
	if model.cst().error_nodes().next().is_some() {
		return None;
	}
	let mut formatter = MiniZincFormatter::new(model, options);
	Some(format!("{:#?}", model.format(&mut formatter)))
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use super::*;

	#[test]
	fn test_format() {
		let actual = format(
			r#"
			a = albatross + bonobo + cassowary;
			c = albatross + bonobo + cassowary + dinosaur + elephant + frog + giraffe + hyena + iguana + jaguar + kangaroo + llama;
			c = [albatross, bonobo, cassowary];
			d = [albatross, bonobo, cassowary, dinosaur, elephant, frog, giraffe, hyena, iguana, jaguar, kangaroo, llama];
			e = {albatross, bonobo, cassowary};
			f = {albatross, bonobo, cassowary, dinosaur, elephant, frog, giraffe, hyena, iguana, jaguar, kangaroo, llama};
			g = [abacus + banana | abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana)];
			h = [(i, j): albatross * (bonobo + cassowary) + dinosaur * (elephant + frog) + giraffe * (hyena + iguana) + jaguar + kangaroo + llama | abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana), abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana), abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana)];
			i = {abacus + banana | abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana)};
			j = {albatross * (bonobo + cassowary) + dinosaur * (elephant + frog) + giraffe * (hyena + iguana) + jaguar + kangaroo + llama | abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana), abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana), abacus in foo(1, 2, 3), banana in bar(1, 2, 3) where qux(abacus, banana)};
			k = [| a, b, c, d |];
			l = [| a, b | c, d |];
			m = [| a: b: | i: c, d | j: e, f |];
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    a = albatross + bonobo + cassowary;
    c =
    	albatross +
    		bonobo +
    		cassowary +
    		dinosaur +
    		elephant +
    		frog +
    		giraffe +
    		hyena +
    		iguana +
    		jaguar +
    		kangaroo +
    		llama;
    c = [albatross, bonobo, cassowary];
    d = [
    	albatross,
    	bonobo,
    	cassowary,
    	dinosaur,
    	elephant,
    	frog,
    	giraffe,
    	hyena,
    	iguana,
    	jaguar,
    	kangaroo,
    	llama,
    ];
    e = {albatross, bonobo, cassowary};
    f = {
    	albatross,
    	bonobo,
    	cassowary,
    	dinosaur,
    	elephant,
    	frog,
    	giraffe,
    	hyena,
    	iguana,
    	jaguar,
    	kangaroo,
    	llama,
    };
    g = [
    	abacus + banana |
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    ];
    h = [
    	(i, j): albatross * (bonobo + cassowary) +
    		dinosaur * (elephant + frog) +
    		giraffe * (hyena + iguana) +
    		jaguar +
    		kangaroo +
    		llama |
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    ];
    i = {
    	abacus + banana |
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    };
    j = {
    	albatross * (bonobo + cassowary) +
    		dinosaur * (elephant + frog) +
    		giraffe * (hyena + iguana) +
    		jaguar +
    		kangaroo +
    		llama |
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    		abacus in foo(1, 2, 3),
    		banana in bar(1, 2, 3) where qux(abacus, banana),
    };
    k = [| a, b, c, d |];
    l = [|
    	a, b |
    	c, d
    |];
    m = [|
    	a: b: |
    	i: c, d |
    	j: e, f
    |];
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_parentheses() {
		let actual = format(
			r#"
			a = (1 + (2 * 3) - 4) + 5;
			b = (1 + 2) * 3 - (4 + 5);
			c = -2 * 3;
			d = -((2 * 3));
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    a = 1 + 2 * 3 - 4 + 5;
    b = (1 + 2) * 3 - (4 + 5);
    c = -2 * 3;
    d = -(2 * 3);
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_keep_parentheses() {
		let actual = format(
			r#"
			a = (1 + (2 * 3) - 4) + 5;
			b = -((2 * 3));
			"#,
			&MiniZincFormatOptions {
				keep_parentheses: true,
				..Default::default()
			},
		);
		let expected = expect![[r#"
    a = (1 + (2 * 3) - 4) + 5;
    b = -(2 * 3);
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_format_types() {
		let actual = format(
			r#"
			int: x;
			var float: y;
			var opt 1..3:z;
			tuple(var 1..3,2..4,int): a;
			record(var 1..3: a, var int:b): b;
			"#,
			&MiniZincFormatOptions {
				keep_parentheses: true,
				..Default::default()
			},
		);
		let expected = expect![[r#"
    int: x;
    var float: y;
    var opt 1..3: z;
    tuple(var 1..3, 2..4, int): a;
    record(var 1..3: a, var int: b): b;
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_format_extras() {
		let actual = format(
			r#"

			
			% Foo
			/* hello */
			int: x = (1 /* foo */ + /* bar */ 2) + 3 % hello
			;


			/* hello */ /* hello */


			% bar
			int /* foo */:y= 3;
			/* world */
			
			/* one */
			% Hello
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    % Foo
    /* hello */
    int: x =
    	1 + /* foo */
    		/* bar */
    		2 +
    		3; % hello

    /* hello */
    /* hello */

    % bar
    int: y = 3; /* foo */
    /* world */

    /* one */
    % Hello
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_format_extras_2() {
		let actual = format(
			r#"
			% Foo
			int: y;
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    % Foo
    int: y;
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_format_comprehension() {
		let actual = format(
			"constraint [a_really_long_word_here_which_overflows_a_really_long_word_here | j in 1..max(country)];",
			&Default::default(),
		);
		let expected = expect![[r#"
    constraint [
    	a_really_long_word_here_which_overflows_a_really_long_word_here |
    		j in 1..max(country),
    ];
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_attach_comments_infix() {
		let actual = format(
			r#"
			any: x = 
				albatross+ % a
				/* b */ bonobo+ % b
				cassowary+ /* c */
				dinosaur+ /* d */ % d
				elephant+
				frog+
				giraffe+
				hyena+
				iguana+
				jaguar+
				kangaroo+
				llama;
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    any: x =
    	albatross + % a
    		/* b */
    		bonobo + % b
    		cassowary + /* c */
    		dinosaur + /* d */ % d
    		elephant +
    		frog +
    		giraffe +
    		hyena +
    		iguana +
    		jaguar +
    		kangaroo +
    		llama;
"#]];
		expected.assert_eq(&actual.unwrap());
	}

	#[test]
	fn test_debug_format() {
		let actual = format_debug(
			r#"
			int: x = 1;
			"#,
			&Default::default(),
		);
		let expected = expect![[r#"
    Element::sequence(
        [
            Element::sequence(
                [
                    Element::sequence(
                        [
                            Element::text(
                                "int",
                            ),
                            Element::text(
                                ": ",
                            ),
                            Element::text(
                                "x",
                            ),
                            Element::sequence(
                                [],
                            ),
                            Element::text(
                                " =",
                            ),
                            Element::group(
                                Element::indent(
                                    Element::sequence(
                                        [
                                            Element::sequence(
                                                [
                                                    Element::if_broken(
                                                        Element::line_break(),
                                                    ),
                                                    Element::if_unbroken(
                                                        Element::text(
                                                            " ",
                                                        ),
                                                    ),
                                                ],
                                            ),
                                            Element::text(
                                                "1",
                                            ),
                                        ],
                                    ),
                                ),
                            ),
                        ],
                    ),
                    Element::text(
                        ";",
                    ),
                ],
            ),
            Element::line_break(),
        ],
    )"#]];
		expected.assert_eq(&actual.unwrap());
	}
}
