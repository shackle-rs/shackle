//! AST representation
//!
//! AST nodes are thin wrappers around CST nodes and provide type-safe access
//! methods. No desugaring is performed at this stage, so all language constructs
//! are available other than parentheses which are implicit in the tree structure.

use std::fmt::Debug;

use super::{ast::Children, cst::Cst};

/// EPrimeModel (wrapper for a CST).
///
/// A model is a single `.eprime` file.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct EPrimeModel {
	cst: Cst,
}

impl EPrimeModel {
	/// Create a model from a CST
	pub fn new(cst: Cst) -> Self {
		Self { cst }
	}

	/// Get the CST
	pub fn cst(&self) -> &Cst {
		&self.cst
	}

	/// Get the top level items in the model
	pub fn items(&self) -> Children<'_, ()> {
		todo!("Nathan: this might be a good start")
	}
}

impl Debug for EPrimeModel {
	fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}
