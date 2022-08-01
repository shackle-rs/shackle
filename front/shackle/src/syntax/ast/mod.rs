//! AST representation
//!
//! AST nodes are thin wrappers around CST nodes and provide type-safe access
//! methods. No desugaring is performed at this stage, so all language constructs
//! are available other than parentheses which are implicit in the tree structure.

use std::{fmt::Debug, marker::PhantomData, ops::Deref};

use crate::{syntax::cst::CstNode, utils::impl_enum_from};

pub mod container;
pub mod expression;
pub(crate) mod helpers;
pub mod item;
pub mod pattern;
pub mod primitive;
pub mod types;

pub use container::*;
pub use expression::*;
pub use item::*;
pub use pattern::*;
pub use primitive::*;
use tree_sitter::TreeCursor;
pub use types::*;

use helpers::*;

use super::cst::Cst;

/// Base trait for AST nodes
pub trait AstNode: Debug {
	/// Create a new node
	fn new(node: CstNode) -> Self
	where
		Self: Sized + From<CstNode>,
	{
		Self::from(node)
	}

	/// Get the underlying CST node
	fn cst_node(&self) -> &CstNode;

	/// Get the (concrete) text content of this node
	fn cst_text(&self) -> &str {
		self.cst_node().text()
	}

	/// Get the kind of the CST node
	fn cst_kind(&self) -> &str {
		self.cst_node().as_ref().kind()
	}

	/// Whether this node is missing
	fn is_missing(&self) -> bool {
		self.cst_node().as_ref().is_missing()
	}

	/// Convert to T if possible
	fn cast_ref<'a, T: TryCastFrom<Self>>(&'a self) -> Option<&'a T>
	where
		Self: Sized,
	{
		T::from_ref(self)
	}

	/// Convert to T if possible
	fn cast<T: TryCastFrom<Self>>(self) -> Option<T>
	where
		Self: Sized,
	{
		T::from(self)
	}
}

/// Iterator over child nodes with a particular field name
#[derive(Clone)]
pub struct Children<'a, T> {
	field: u16,
	tree: &'a Cst,
	cursor: TreeCursor<'a>,
	done: bool,
	phantom: PhantomData<T>,
}

impl<'a, T: From<CstNode>> Iterator for Children<'a, T> {
	type Item = T;
	fn next(&mut self) -> Option<T> {
		if self.done {
			return None;
		}
		while self.cursor.field_id() != Some(self.field) {
			if !self.cursor.goto_next_sibling() {
				return None;
			}
		}
		let result = self.tree.node(self.cursor.node());
		self.done = !self.cursor.goto_next_sibling();
		Some(T::from(result))
	}
}

impl<'a, T: Debug + From<CstNode>> std::fmt::Debug for Children<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut cursor = self.cursor.clone();
		cursor.goto_parent();
		let done = !cursor.goto_first_child();

		let iter: Children<'a, T> = Children {
			field: self.field,
			tree: self.tree,
			cursor,
			done,
			phantom: PhantomData,
		};
		f.debug_list().entries(iter).finish()
	}
}

/// Helper trait to aid in unwrapping enum nodes into their underlying type.
pub trait TryCastFrom<T>: Sized {
	/// Create from &T
	fn from_ref(value: &T) -> Option<&Self>;
	/// Create from T
	fn from(value: T) -> Option<Self>;
}

/// Model (wrapper for a CST).
///
/// A model is a single `.mzn` file.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Model {
	cst: Cst,
}

impl Model {
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

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;

	#[test]
	fn test_model() {
		let model = parse_model(r#"% Line comment"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 0);
	}
}
