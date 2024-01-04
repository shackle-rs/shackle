//! Wrappers around the tree-sitter tree to allow for usage with salsa.

use std::{
	fmt::Debug,
	hash::{Hash, Hasher},
	ops::Deref,
	sync::Arc,
};

use miette::SourceSpan;
use tree_sitter::{Node, Tree};

use super::db::SourceParser;
use crate::{
	db::FileReader,
	diagnostics::SyntaxError,
	file::{FileRef, SourceFile},
};

/// Wrapper for a tree sitter tree.
///
/// The underlying `Tree` can be accessed through dereferencing.
#[derive(Debug, Clone)]
pub struct Cst {
	inner: Arc<CstInner>,
}

#[derive(Debug, Clone)]
struct CstInner {
	tree: Tree,
	file: Option<FileRef>,
	source: Arc<String>,
}

impl Cst {
	/// Create a CST from a tree sitter tree and source buffer.
	pub fn new(tree: Tree, file: FileRef, source: Arc<String>) -> Self {
		Cst {
			inner: Arc::new(CstInner {
				tree,
				file: Some(file),
				source,
			}),
		}
	}

	/// Create from string (without any `FileRef`).
	pub fn from_str(tree: Tree, source: &str) -> Self {
		Cst {
			inner: Arc::new(CstInner {
				tree,
				file: None,
				source: Arc::new(source.to_owned()),
			}),
		}
	}

	/// Get the underlying source file
	pub fn file(&self) -> FileRef {
		self.inner
			.file
			.expect("Called file() on Cst constructed without FileRef")
	}

	/// Get the underlying source text.
	pub fn text(&self) -> &str {
		self.inner.source.as_str()
	}

	/// Get the error/missing nodes if any
	pub fn error_nodes(&self) -> impl Iterator<Item = CstNode> + '_ {
		let mut cursor = self.walk();
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
								return Some(self.node(node));
							}
							return None;
						}
					}
				}
				if node.is_error() || node.is_missing() {
					return Some(self.node(node));
				}
			}
			None
		})
	}

	/// Get the syntax error(s) if any
	pub fn error<F: Fn(Option<FileRef>) -> SourceFile>(
		&self,
		get_source: F,
	) -> Result<(), SyntaxError> {
		let mut errors = self.error_nodes().map(|cst_node| {
			let mut node = *cst_node.as_ref();
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
			let src = get_source(self.inner.file);
			let msg = if node.is_missing() {
				format!("Missing {}", node.kind())
			} else if node.is_error() {
				let c = self.node(node);
				let text = c.text();
				format!("Unexpected {}", text.chars().next().unwrap())
			} else {
				format!("Unexpected {}", node.kind())
			};
			SyntaxError {
				src,
				span: node.byte_range().into(),
				msg,
				other: Vec::new(),
			}
		});
		let Some(mut error) = errors.next() else {
			return Ok(());
		};
		error.other.extend(errors);
		Err(error)
	}

	/// Create a [`CstNode`] from the given raw node from the same tree.
	pub fn node<'a>(&'a self, node: Node<'a>) -> CstNode {
		let tree = self.clone();
		unsafe { CstNode::new(tree, node) }
	}

	/// Print this tree for debugging purposes.
	pub fn debug_print<W: std::fmt::Write>(&self, buf: &mut W) {
		self.node(self.root_node()).debug_print(buf)
	}
}

impl PartialEq for Cst {
	fn eq(&self, other: &Self) -> bool {
		// Fake equality using pointers, instead of actually comparing trees
		// TODO: replace with real comparison
		std::ptr::eq(self.inner.as_ref(), other.inner.as_ref())
	}
}

impl Eq for Cst {}

impl Hash for Cst {
	fn hash<H: Hasher>(&self, state: &mut H) {
		// Fake hash using pointers, instead of actually hashing tree
		// TODO: replace with real hash
		std::ptr::hash(self.inner.as_ref(), state)
	}
}

impl Deref for Cst {
	type Target = Tree;

	fn deref(&self) -> &Self::Target {
		&self.inner.tree
	}
}

/// Reference to tree sitter node.
///
/// Works around the lifetime parameter for `Node` so that it can be used in salsa queries.
/// Access the underlying `Node` through the `as_ref()` and `as_mut()` methods.
/// Raw `Node`s can be converted to `CstNode`s using `Cst::node()`.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CstNode {
	tree: Cst, // Keeps the Tree alive
	node: Node<'static>,
}

impl Debug for CstNode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("CstNode")
			.field("kind", &self.as_ref().kind())
			.field("text", &self.text())
			.finish()
	}
}

impl CstNode {
	unsafe fn new(tree: Cst, node: Node) -> Self {
		// Unsafe because we can't guarantee that `tree` is actually the tree for `node`.
		CstNode {
			tree,
			node: std::mem::transmute(node),
		}
	}

	/// Get the text of this node.
	pub fn text(&self) -> &str {
		self.node.utf8_text(self.tree.text().as_bytes()).unwrap()
	}

	/// Get the source and span for this node (convenience function for producing errors)
	pub fn source_span(&self, db: &dyn SourceParser) -> (SourceFile, SourceSpan) {
		(
			SourceFile::new(self.cst().file(), db.upcast()),
			self.as_ref().byte_range().into(),
		)
	}

	/// Get the concrete syntax tree containing this node.
	pub fn cst(&self) -> &Cst {
		&self.tree
	}

	/// Get the location of this node
	pub fn debug_location(&self, db: &dyn FileReader) -> String {
		let source = SourceFile::new(self.cst().file(), db);
		let file = source.name();
		let start = self.node.start_position();
		let end = self.node.end_position();
		format!(
			"{:?}:{}.{}-{}.{}",
			file,
			start.row + 1,
			start.column + 1,
			end.row + 1,
			end.column + 1
		)
	}

	/// Print this concrete syntax node and its descendants for debugging purposes.
	pub fn debug_print<W: std::fmt::Write>(&self, buf: &mut W) {
		let mut level = 0;
		let mut cursor = self.as_ref().walk();
		loop {
			let node = cursor.node();
			writeln!(
				buf,
				"{:i$}kind={:?}, named={:?}, has_error={:?}, error={:?}, missing={:?}, extra={:?}, field={:?}",
				"",
				node.kind(),
				node.is_named(),
				node.has_error(),
				node.is_error(),
				node.is_missing(),
				node.is_extra(),
				cursor.field_name(),
				i = level * 2
			)
			.unwrap();

			if cursor.goto_first_child() {
				level += 1;
			} else {
				while !cursor.goto_next_sibling() {
					if cursor.goto_parent() {
						level -= 1;
					} else {
						return;
					}
				}
			}
		}
	}
}

impl<'a> AsRef<Node<'a>> for CstNode
where
	Self: 'a,
{
	fn as_ref(&self) -> &Node<'a> {
		unsafe { std::mem::transmute(&self.node) }
	}
}

impl<'a> AsMut<Node<'a>> for CstNode
where
	Self: 'a,
{
	fn as_mut(&mut self) -> &mut Node<'a> {
		unsafe { std::mem::transmute(&mut self.node) }
	}
}

impl<'a> From<CstNode> for Node<'a>
where
	CstNode: 'a,
{
	fn from(x: CstNode) -> Self {
		x.node
	}
}

#[cfg(test)]
mod test {
	use expect_test::{expect, expect_file, ExpectFile};
	use tree_sitter::Parser;

	use super::Cst;

	fn check_cst_file(source: &str, expected: ExpectFile) {
		let mut parser = Parser::new();
		parser
			.set_language(tree_sitter_minizinc::language())
			.unwrap();
		let tree = parser.parse(source.as_bytes(), None).unwrap();
		let cst = Cst::from_str(tree, source);
		let mut buf = String::new();
		cst.debug_print(&mut buf);
		expected.assert_eq(&buf);
	}

	#[test]
	fn test_doc_simple_model() {
		check_cst_file(
			include_str!("../../../../docs/src/examples/simple-model.mzn"),
			expect_file!("../../../../docs/src/examples/simple-model-cst.txt"),
		)
	}

	#[test]
	fn test_cst_errors() {
		let source = r#"
			int: x =;
			int: = 1;
			foo
		"#;
		let mut parser = Parser::new();
		parser
			.set_language(tree_sitter_minizinc::language())
			.unwrap();
		let tree = parser.parse(source.as_bytes(), None).unwrap();
		let cst = Cst::from_str(tree, source);
		let actual = cst
			.error_nodes()
			.map(|n| {
				format!(
					"{} {}",
					if n.as_ref().is_missing() {
						"missing"
					} else {
						"unexpected"
					},
					if n.as_ref().is_error() {
						n.text()
					} else {
						n.as_ref().kind()
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
