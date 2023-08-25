//! AST representation
//!
//! AST nodes are thin wrappers around CST nodes and provide type-safe access
//! methods. No desugaring is performed at this stage, so all language constructs
//! are available other than parentheses which are implicit in the tree structure.

use std::{fmt::Debug, marker::PhantomData};

pub mod container;
pub mod expression;
pub mod item;
pub mod pattern;
pub mod primitive;
pub mod types;

pub use container::*;
pub use expression::*;
pub use item::*;
pub use pattern::*;
pub use primitive::*;
pub use types::*;

use super::{ast::Children, cst::Cst};

/// MznModel (wrapper for a CST).
///
/// A model is a single `.mzn` file.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct MznModel {
	cst: Cst,
}

impl MznModel {
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

impl Debug for MznModel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Model")
			.field("items", &self.items())
			.finish()
	}
}

#[cfg(test)]
mod test {
	use expect_test::{expect, expect_file};

	use crate::syntax::ast::test::*;

	#[test]
	fn test_model() {
		check_ast(
			r#"% Line comment"#,
			expect!([r#"
    Model {
        items: [],
    }
"#]),
		);
	}

	#[test]
	fn test_doc_simple_model() {
		check_ast_file(
			include_str!("../../../../../docs/src/examples/simple-model.mzn"),
			expect_file!("../../../../../docs/src/examples/simple-model-ast.txt"),
		);
	}
}
