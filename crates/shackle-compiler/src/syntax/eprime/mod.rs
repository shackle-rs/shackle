//! AST representation
//!
//! AST nodes are thin wrappers around CST nodes and provide type-safe access
//! methods. No desugaring is performed at this stage, so all language constructs
//! are available other than parentheses which are implicit in the tree structure.

use std::{fmt::Debug, marker::PhantomData};

use super::{ast::Children, cst::Cst};

pub mod container;
pub mod domain;
pub mod expression;
pub mod item;
pub mod primitive;

pub use container::*;
pub use domain::*;
pub use expression::*;
pub use item::*;
pub use primitive::*;

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
	pub fn items(&self) -> Children<'_, Item> {
		let tree = &self.cst;
		let id = tree.language().field_id_for_name("item").unwrap();
		let mut cursor = tree.root_node().walk();
		let done = !cursor.goto_first_child();
		Children {
			field: id,
			tree,
			cursor,
			done,
			phantom: PhantomData,
		}
	}
}

impl Debug for EPrimeModel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Model")
			.field("items", &self.items())
			.finish()
	}
}

/*
====================
Temporary Check List
====================

source_file
param_decl
const_def
domain_alias
decision_decl
objective
branching
constraint
heuristic
_expression
call
quantification
matrix_comprehension
generator
indexed_access
infix_operator
set_in
absolute_operator
prefix_operator
_domain
matrix_domain
_base_domain
domain_operation
boolean_domain
integer_domain
matrix_literal
range_literal
boolean_literal (done) (tested)
integer_literal (done) (tested)
identifier (done) (tested)
*/
