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

ast_node!(
	/// A model (the root node of the AST is always a model)
	Model,
	items
);

impl Model {
	/// Get the top level items in the model
	pub fn items(&self) -> Children<'_, Item> {
		children_with_field_name(self, "item")
	}
}

/// Points to an AST node
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AstNodeRef {
	/// Expression
	Expression(Expression),
	/// Item
	Item(Item),
	/// Type
	Type(Type),
	/// Pattern
	Pattern(Pattern),
	/// Let item
	LetItem(LetItem),
	/// Function parameter
	Parameter(Parameter),
	/// Generator
	Generator(Generator),
	/// Infinite index slice
	IndexSlice(IndexSlice),
	/// Enumeration case
	EnumerationCase(EnumerationCase),
}

impl Deref for AstNodeRef {
	type Target = dyn AstNode;
	fn deref(&self) -> &Self::Target {
		match self {
			AstNodeRef::Expression(e) => e,
			AstNodeRef::Item(i) => i,
			AstNodeRef::Type(t) => t,
			AstNodeRef::Pattern(p) => p,
			AstNodeRef::LetItem(l) => l,
			AstNodeRef::Parameter(p) => p,
			AstNodeRef::Generator(g) => g,
			AstNodeRef::IndexSlice(i) => i,
			AstNodeRef::EnumerationCase(e) => e,
		}
	}
}

impl_enum_from!(AstNodeRef::Expression(Expression));
impl_enum_from!(AstNodeRef::Item(Item));
impl_enum_from!(AstNodeRef::Type(Type));
impl_enum_from!(AstNodeRef::Pattern(Pattern));
impl_enum_from!(AstNodeRef::LetItem(LetItem));
impl_enum_from!(AstNodeRef::Parameter(Parameter));
impl_enum_from!(AstNodeRef::Generator(Generator));
impl_enum_from!(AstNodeRef::IndexSlice(IndexSlice));
impl_enum_from!(AstNodeRef::EnumerationCase(EnumerationCase));

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
