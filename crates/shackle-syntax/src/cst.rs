//! Wrappers around the tree-sitter tree to allow for usage with salsa.

use std::{
	hash::{Hash, Hasher},
	num::NonZeroU16,
};

use derive_more::{AsRef, From};
use miette::SourceSpan;
use shackle_diagnostics::{MultipleErrors, Result, SourceFile, SyntaxError};
pub use tree_sitter::Point;
use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::InputLang;
/// Wrapper for a tree sitter tree.
#[derive(AsRef, Clone, From)]
pub struct Cst(Tree);

impl Cst {
	/// Parse a file
	pub fn new(text: &str, lang: InputLang) -> Self {
		let mut parser = Parser::new();
		let tree_sitter_lang = match lang {
			InputLang::DataZinc => tree_sitter_datazinc::LANGUAGE,
			InputLang::MiniZinc => tree_sitter_minizinc::LANGUAGE,
			InputLang::EPrime => tree_sitter_eprime::LANGUAGE,
			InputLang::Json => unreachable!("Unsupported language"),
		};
		parser
			.set_language(&tree_sitter_lang.into())
			.expect("Failed to set tree sitter language");
		let tree = parser.parse(text, None).expect("Failed to run parser");
		tree.into()
	}

	/// Get the root node
	pub fn root(&self) -> CstNode<'_> {
		self.0.root_node().into()
	}

	/// Find the node at the given position
	pub fn find(&self, start: Point, end: Point) -> Option<CstNode<'_>> {
		let result = self.root().0.descendant_for_point_range(start, end);
		if start == end && start.column > 0 {
			// Find when we're looking just after a node
			let prev_column = Point {
				row: start.row,
				column: start.column - 1,
			};
			let prev = self
				.root()
				.0
				.descendant_for_point_range(prev_column, prev_column);
			match (prev, result) {
				(Some(p), Some(r)) => {
					if r.byte_range().contains(&p.byte_range().start) {
						return Some(p.into());
					}
					return Some(r.into());
				}
				(Some(n), None) | (None, Some(n)) => {
					return Some(n.into());
				}
				_ => return None,
			}
		}
		Some(result?.into())
	}

	/// Whether this CST has any syntax errors
	pub fn has_errors(&self) -> bool {
		self.error_nodes().next().is_some()
	}

	/// Get the error/missing nodes if any
	fn error_nodes(&self) -> impl Iterator<Item = CstNode<'_>> + '_ {
		let mut cursor = self.0.walk();
		let mut done = false;
		std::iter::from_fn(move || {
			while !done {
				let node = cursor.node();
				if !node.has_error() || !cursor.goto_first_child() {
					while !cursor.goto_next_sibling() {
						if !cursor.goto_parent() {
							done = true;
							let node = cursor.node();
							if node.is_error() || node.is_missing() {
								return Some(CstNode::from(node));
							}
							return None;
						}
					}
				}
				if node.is_error() || node.is_missing() {
					return Some(CstNode::from(node));
				}
			}
			None
		})
	}

	/// Get the syntax error(s) if any
	pub fn errors<'a>(&'a self, src: &'a SourceFile) -> impl Iterator<Item = SyntaxError> + 'a {
		self.error_nodes().map(move |cst_node| {
			let mut node = cst_node.0;
			if node.is_error() {
				if node.parent().is_none() {
					// Root node is ERROR
					let mut cursor = node.walk();
					let mut had_semi = true;
					if cursor.goto_first_child() {
						loop {
							if let Some("item") = cursor.field_name() {
								if had_semi {
									had_semi = false;
								}
							} else if cursor.node().kind() == ";" {
								if had_semi {
									// Invalid semicolon
									node = cursor.node();
									break;
								}
								had_semi = true;
							} else if !cursor.node().is_extra() {
								// Unexpected non-item
								node = cursor.node();
								break;
							}
							if !cursor.goto_next_sibling() {
								break;
							}
						}
					}
				} else if let Some(child) = node.child(0) {
					node = child;
				}
			}

			let cst_node = CstNode::from(node);
			let msg = if node.is_missing() {
				format!("Missing {}", node.kind())
			} else if node.is_error() {
				let text = cst_node.text(src.contents());
				format!("Unexpected {}", text.chars().next().unwrap())
			} else {
				format!("Unexpected {}", node.kind())
			};
			SyntaxError {
				src: src.clone(),
				span: cst_node.span(),
				msg,
			}
		})
	}

	/// Check for syntax errors
	pub fn check(&self, src: &SourceFile) -> Result<()> {
		let mut errors = self.errors(src).map(|e| e.into()).collect::<Vec<_>>();
		if errors.is_empty() {
			Ok(())
		} else if errors.len() == 1 {
			Err(errors.pop().unwrap())
		} else {
			Err(MultipleErrors { errors }.into())
		}
	}
}

impl PartialEq for Cst {
	fn eq(&self, other: &Self) -> bool {
		// Fake equality using pointers, instead of actually comparing trees
		// TODO: replace with real comparison
		std::ptr::eq(&self.0, &other.0)
	}
}

impl Eq for Cst {}

impl Hash for Cst {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// Fake hash using pointers, instead of actually hashing tree
		// TODO: replace with real hash
		std::ptr::hash(&self.0, state)
	}
}

impl std::fmt::Debug for Cst {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.root().fmt(f)
	}
}

/// Reference to tree sitter node.
#[derive(AsRef, Clone, From, Eq, PartialEq, Hash)]
pub struct CstNode<'tree>(Node<'tree>);

impl<'tree> CstNode<'tree> {
	/// THe unique node ID
	pub fn id(&self) -> usize {
		self.0.id()
	}

	/// The kind of this CST node
	pub fn kind(&self) -> &'tree str {
		self.0.kind()
	}

	/// Whether this node is missing
	pub fn is_missing(&self) -> bool {
		self.0.is_missing()
	}

	/// Whether this node is an error
	pub fn is_error(&self) -> bool {
		self.0.is_error()
	}

	/// Whether this node has an error
	pub fn has_error(&self) -> bool {
		self.0.has_error()
	}

	/// Get the text of this node.
	pub fn text<'a>(&self, source: &'a str) -> &'a str {
		self.0.utf8_text(source.as_bytes()).unwrap()
	}

	/// Get the parent of this node if there is one
	pub fn parent(&self) -> Option<Self> {
		self.0.parent().map(Self::from)
	}

	/// Get the previous sibling of this node, including anonymous nodes.
	pub fn previous_sibling(&self) -> Option<Self> {
		self.0.prev_sibling().map(Self::from)
	}

	/// Get the next sibling of this node, including anonymous nodes.
	pub fn next_sibling(&self) -> Option<Self> {
		self.0.next_sibling().map(Self::from)
	}

	/// Get the previous named sibling of this node.
	pub fn previous_named_sibling(&self) -> Option<Self> {
		self.0.prev_named_sibling().map(Self::from)
	}

	/// Get the next named sibling of this node.
	pub fn next_named_sibling(&self) -> Option<Self> {
		self.0.next_named_sibling().map(Self::from)
	}

	/// Whether this node is an extra node, such as a comment.
	pub fn is_extra(&self) -> bool {
		self.0.is_extra()
	}

	/// Whether this node is named by the grammar.
	pub fn is_named(&self) -> bool {
		self.0.is_named()
	}

	/// Get the start point of this node.
	pub fn start_point(&self) -> Point {
		self.0.start_position()
	}

	/// Get the end point of this node.
	pub fn end_point(&self) -> Point {
		self.0.end_position()
	}

	/// Get the given child by index (if present)
	pub fn child(&self, idx: u32) -> Option<Self> {
		self.0.child(idx).map(CstNode::from)
	}

	/// Get the children of this node
	pub fn children(&self) -> impl Iterator<Item = Self> {
		let mut cursor = self.0.walk();
		let mut done = !cursor.goto_first_child();
		std::iter::from_fn(move || {
			if done {
				None
			} else {
				let child = CstNode::from(cursor.node());
				done = !cursor.goto_next_sibling();
				Some(child)
			}
		})
	}

	/// Retrieve a child node by its field name
	pub fn child_with_field_name(&self, field: &str) -> Self {
		self.optional_child_with_field_name(field)
			.unwrap_or_else(|| panic!("Expected child node with field name {}", field))
	}

	/// Optionally retrieve a child node by its field name
	pub fn optional_child_with_field_name(&self, field: &str) -> Option<Self> {
		self.0.child_by_field_name(field).map(Self::from)
	}

	/// Retrieve children by field name
	pub fn children_with_field_name(&self, field: &str) -> ChildrenWithFieldName<'tree> {
		let id = self.0.language().field_id_for_name(field).unwrap();
		let mut cursor = self.0.walk();
		let done = !cursor.goto_first_child();
		ChildrenWithFieldName {
			field: id,
			cursor,
			done,
		}
	}

	/// Get the span of this node
	pub fn span(&self) -> SourceSpan {
		self.0.byte_range().into()
	}
}

impl<'a> std::fmt::Debug for CstNode<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let cursor = self.0.walk();
		let field_name = cursor.field_name();
		f.debug_struct("CstNode")
			.field("kind", &self.0.kind())
			.field("start", &self.0.start_position())
			.field("end", &self.0.end_position())
			.field("is_named", &self.0.is_named())
			.field("has_error", &self.0.has_error())
			.field("is_error", &self.0.is_error())
			.field("is_missing", &self.0.is_missing())
			.field("is_extra", &self.0.is_extra())
			.field("field", &field_name)
			.field("children", &self.children().collect::<Vec<_>>())
			.finish()
	}
}

/// Iterator over child nodes with a particular field name
#[derive(Clone)]
pub struct ChildrenWithFieldName<'tree> {
	/// Tree-sitter field ID to match while iterating.
	field: NonZeroU16,
	/// Cursor positioned at the current child node.
	cursor: TreeCursor<'tree>,
	/// Whether the iterator has reached the end of the sibling list.
	done: bool,
}

impl<'tree> ChildrenWithFieldName<'tree> {
	/// Reset the cursor to the first child of the parent node.
	pub(crate) fn reset(&mut self) {
		if self.cursor.goto_parent() {
			let _ = self.cursor.goto_first_child();
		}
		self.done = false;
	}
}

impl<'tree> Iterator for ChildrenWithFieldName<'tree> {
	type Item = CstNode<'tree>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}
		while self.cursor.field_id() != Some(self.field) {
			if !self.cursor.goto_next_sibling() {
				return None;
			}
		}
		let result = CstNode::from(self.cursor.node());
		self.done = !self.cursor.goto_next_sibling();
		Some(result)
	}
}

impl<'tree> std::fmt::Debug for ChildrenWithFieldName<'tree> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut iter = self.clone();
		iter.reset();
		f.debug_list().entries(iter).finish()
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{ExpectFile, expect, expect_file};

	use super::Cst;
	use crate::InputLang;

	fn check_cst_file(source: &str, expected: ExpectFile) {
		let cst = Cst::new(source, InputLang::MiniZinc);
		expected.assert_debug_eq(&cst);
	}

	#[test]
	fn test_doc_simple_model() {
		check_cst_file(
			include_str!("../../../docs/src/examples/simple-model.mzn"),
			expect_file!("../../../docs/src/examples/simple-model-cst.txt"),
		)
	}

	#[test]
	fn test_cst_errors() {
		let source = r#"
			int: x =;
			int: = 1;
			foo
		"#;
		let cst = Cst::new(source, InputLang::MiniZinc);
		let actual = cst
			.error_nodes()
			.map(|n| {
				format!(
					"{} {}",
					if n.is_missing() {
						"missing"
					} else {
						"unexpected"
					},
					if n.is_error() {
						n.text(source)
					} else {
						n.kind()
					}
				)
			})
			.collect::<Vec<_>>()
			.join("\n");
		let expected = expect![[r#"
    unexpected =
    missing identifier
    unexpected foo"#]];
		expected.assert_eq(&actual);
	}
}
