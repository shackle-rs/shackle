//! Syntax representations.
//!
//! A concrete syntax tree is created by the parser.
//! This completely represents the source text, including comments and whitespace.
//!
//! Since this is not convenient for most stages of compilation, an abstract syntax tree is
//! generated which provides type-safe access to children. The AST includes all language constructs
//! (i.e. no desugaring is performed, just removal of non-semantic nodes).
//!
//! The AST is then lowered into HIR, which is the main representation used by the compiler.
//!

pub mod ast;
pub mod cst;
pub mod db;

// AST representations for different modelling languages
pub mod eprime;
pub mod minizinc;

use self::{ast::ConstraintModel, cst::Cst, eprime::EPrimeModel, minizinc::MznModel};

/// Enum 'Or Type' for
#[derive(PartialEq, Eq, Default)]
pub enum SyntaxModel {
	#[default]
	/// Generate New MiniZinc Model for EPrime
	MznModel,
	/// Generate New Constraint Model for EPrime
	EPrimeModel,
}
impl SyntaxModel {
	/// Generate New Constraint Model for different modelling languages
	fn new(&self, cst: Cst) -> ConstraintModel {
		match self {
			Self::MznModel => ConstraintModel::MznModel(MznModel::new(cst)),
			Self::EPrimeModel => ConstraintModel::EPrimeModel(EPrimeModel::new(cst)),
		}
	}
}
