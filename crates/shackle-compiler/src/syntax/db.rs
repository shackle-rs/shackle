#![allow(missing_docs)]
//! Database queries for syntax parsing

use tree_sitter::Parser;

use super::{ast::Model, cst::Cst};
use crate::{
	db::{FileReader, Upcast},
	file::FileRef,
	Result,
};

/// Syntax parsing queries
#[salsa::query_group(SourceParserStorage)]
pub trait SourceParser: FileReader + Upcast<dyn FileReader> {
	/// Produce a CST for the given file.
	///
	/// Only gives an `Err` result if getting the file contents failed.
	/// Otherwise, the error is contained in the CST.
	fn cst(&self, file: FileRef) -> Result<Cst>;

	/// Produce an AST for the given file.
	///
	/// Only gives an `Err` result if getting the file contents failed.
	/// Otherwise, the error is contained in the CST.
	fn ast(&self, file: FileRef) -> Result<Model>;
}

fn cst(db: &dyn SourceParser, file: FileRef) -> Result<Cst> {
	let contents = file.contents(db.upcast())?;

	// TODO: Don't create new parser for every file (hard since parsing requires mutable reference to Parser)
	let mut parser = Parser::new();
	parser
		.set_language(tree_sitter_minizinc::language())
		.expect("Failed to set Tree Sitter parser language");
	let tree = parser
		.parse(contents.as_bytes(), None)
		.expect("MiniZinc Tree Sitter parser did not return tree object");

	Ok(Cst::new(tree, file, contents))
}

fn ast(db: &dyn SourceParser, file: FileRef) -> Result<Model> {
	let cst = db.cst(file)?;
	Ok(Model::new(cst))
}
