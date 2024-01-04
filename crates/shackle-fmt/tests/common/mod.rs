use std::path::Path;

use pretty_assertions::assert_str_eq;
use shackle_compiler::syntax::{ast::Model, cst::Cst};
use shackle_fmt::{format_model, MiniZincFormatOptions};
use tree_sitter::Parser;

pub fn check_format_file(path: &Path, options: &MiniZincFormatOptions) -> String {
	let source = std::fs::read_to_string(path)
		.unwrap_or_else(|err| panic!("Failed to read {} ({})", path.to_string_lossy(), err));
	let mut parser = Parser::new();
	parser
		.set_language(tree_sitter_minizinc::language())
		.unwrap();
	let tree = parser.parse(source.as_bytes(), None).unwrap();
	let model = Model::new(Cst::from_str(tree, &source));
	let formatted = format_model(&model, options).unwrap_or_else(|| {
		panic!("Failed to format {}", path.to_string_lossy());
	});
	let formatted_tree = parser.parse(formatted.as_bytes(), None).unwrap();
	let formatted_model = Model::new(Cst::from_str(formatted_tree, &formatted));
	assert_str_eq!(
		format!("{:#?}", model),
		format!("{:#?}", formatted_model),
		"Formatting {} changed AST",
		path.to_string_lossy(),
	);
	let reformatted = format_model(&formatted_model, options).unwrap_or_else(|| {
		panic!("Failed to reformat {}", path.to_string_lossy());
	});
	assert_eq!(
		reformatted,
		formatted,
		"Second format of {} didn't match",
		path.to_string_lossy()
	);
	formatted
}
