#![allow(missing_docs)]
//! Database queries for syntax parsing

use tree_sitter::Parser;

use super::{ast::ConstraintModel, cst::Cst, eprime::EPrimeModel, minizinc::MznModel};
use crate::{
	db::{FileReader, Upcast},
	file::{FileRef, InputLang},
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
	fn ast(&self, file: FileRef) -> Result<ConstraintModel>;
}

fn cst(db: &dyn SourceParser, file: FileRef) -> Result<Cst> {
	let contents = file.contents(db.upcast())?;

	let tree_sitter_lang = match file.lang(db.upcast()) {
		InputLang::MiniZinc => tree_sitter_minizinc::language(),
		InputLang::EPrime => tree_sitter_eprime::language(),
		_ => unreachable!("cst should only be called on model files"),
	};

	let mut parser = Parser::new();
	parser
		.set_language(&tree_sitter_lang)
		.expect("Failed to set Tree Sitter parser language");
	let tree = parser
		.parse(contents.as_bytes(), None)
		.expect("Tree Sitter parser did not return tree object");

	Ok(Cst::new(tree, file, contents))
}

fn ast(db: &dyn SourceParser, file: FileRef) -> Result<ConstraintModel> {
	let cst = db.cst(file)?;
	match cst.file().lang(db.upcast()) {
		InputLang::MiniZinc => Ok(ConstraintModel::MznModel(MznModel::new(cst))),
		InputLang::EPrime => Ok(ConstraintModel::EPrimeModel(EPrimeModel::new(cst))),
		_ => unreachable!("ast should only be called on ,odel files"),
	}
}
