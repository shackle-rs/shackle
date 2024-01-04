//! Helper utilities for dealing with AST nodes.
use std::{fmt::Debug, marker::PhantomData};

use crate::syntax::cst::CstNode;

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
	fn cast_ref<T: TryCastFrom<Self>>(&self) -> Option<&T>
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
	pub(crate) field: u16,
	pub(crate) tree: &'a Cst,
	pub(crate) cursor: TreeCursor<'a>,
	pub(crate) done: bool,
	pub(crate) phantom: PhantomData<T>,
}

impl<'a, T: From<CstNode>> Children<'a, T> {
	/// Get the children of a `CstNode`
	pub fn from_cst(parent: &'a CstNode, field: &str) -> Self {
		let tree = parent.cst();
		let id = tree.language().field_id_for_name(field).unwrap();
		let mut cursor = parent.as_ref().walk();
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

/// Helper to retrieve a child node by its field name
pub fn child_with_field_name<T: AstNode, U: From<CstNode>>(parent: &T, field: &str) -> U {
	let tree = parent.cst_node().cst();
	let node = parent.cst_node().as_ref();
	let child = node.child_by_field_name(field).unwrap();
	U::from(tree.node(child))
}

/// Helper to retrieve a child node by its field name
pub fn optional_child_with_field_name<T: AstNode, U: From<CstNode>>(
	parent: &T,
	field: &str,
) -> Option<U> {
	let tree = parent.cst_node().cst();
	let node = parent.cst_node().as_ref();
	node.child_by_field_name(field)
		.map(|c| U::from(tree.node(c)))
}

/// Helper to retrieve child nodes by field name
pub fn children_with_field_name<'a, T: AstNode, U: From<CstNode>>(
	parent: &'a T,
	field: &str,
) -> Children<'a, U> {
	let cst_node = parent.cst_node();
	let tree = cst_node.cst();
	let id = tree.language().field_id_for_name(field).unwrap();
	let mut cursor = cst_node.as_ref().walk();
	let done = !cursor.goto_first_child();
	Children {
		field: id,
		tree,
		cursor,
		done,
		phantom: PhantomData,
	}
}

/// Helper to decode the string contained in a CST node
pub fn decode_string(cst_node: &CstNode) -> String {
	let tree = cst_node.cst();
	let node = cst_node.as_ref();
	let mut cursor = node.walk();
	node.children_by_field_name("content", &mut cursor)
		.map(|c| match c.kind() {
			"string_characters" => c.utf8_text(tree.text().as_bytes()).unwrap().to_owned(),
			"escape_sequence" => {
				let e = c.child_by_field_name("escape").unwrap();
				match e.kind() {
					"octal" => char::from_u32(
						u32::from_str_radix(e.utf8_text(tree.text().as_bytes()).unwrap(), 8)
							.unwrap(),
					)
					.unwrap()
					.to_string(),
					"hexadecimal" => char::from_u32(
						u32::from_str_radix(e.utf8_text(tree.text().as_bytes()).unwrap(), 16)
							.unwrap(),
					)
					.unwrap()
					.to_string(),
					_ => e.kind().to_owned(),
				}
			}
			_ => unreachable!(),
		})
		.collect::<Vec<_>>()
		.join("")
}

/// Declare a new type implementing `AstNode` which includes the given methods in its debugging
/// representation.
macro_rules! ast_node {
	(
		$(#[$meta:meta])*
		$name:ident
		$(, $method:ident)*
		$(,)*
	) => (
        $(#[$meta])*
		#[allow(missing_docs)]
		#[derive(Clone, Eq, PartialEq, Hash)]
		pub struct $name {
			syntax: $crate::syntax::cst::CstNode,
		}

		impl ::std::convert::From<$crate::syntax::cst::CstNode> for $name {
			fn from(syntax: $crate::syntax::cst::CstNode) -> Self {
				$name { syntax }
			}
		}

		impl $crate::syntax::ast::AstNode for $name {
			fn cst_node(&self) -> &$crate::syntax::cst::CstNode {
				&self.syntax
			}
		}

		impl ::std::fmt::Debug for $name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				f.debug_struct(stringify!($name))
					.field("cst_kind", &self.cst_kind())
					$(.field(stringify!($method), &self.$method()))*
					.finish()
			}
		}
	);
}

pub(crate) use ast_node;

/// Declare a new enum implementing `AstNode` which uses the given matches for its variants.
/// ```
macro_rules! ast_enum {
	(
		$(#[$meta:meta])*
		$name:ident,
		$($tail:tt)+
	) => {
		ast_enum!(@enum ($($tail)+) ($(#[$meta])* #[derive(Clone, Eq, PartialEq, Hash, Debug)] pub enum $name));
		ast_enum!(@cast $name, $($tail)+);

		impl ::std::convert::From<$crate::syntax::cst::CstNode> for $name {
			ast_enum!(@ast_node $name syntax ($($tail)+));
		}

		impl $crate::syntax::ast::AstNode for $name {
			ast_enum!(@cst_node $name ($($tail)+));
		}
	};

	// Enum declaration
	(@enum ($(,)?) ($($def:tt)*) $($tail:tt)*) => {
		$($def)* {
			$($tail)*
		}
	};
	(@enum ($pattern:pat => $name:ident $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@enum ($($($rest)*)?) $($tail)*
			#[doc="`"]
			#[doc=stringify!($name)]
			#[doc="` node"]
			$name($name),
		);
	};
	(@enum ($pattern:pat => $name:ident($type:ty) $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@enum ($($($rest)*)?) $($tail)*
			#[doc="`"]
			#[doc=stringify!($type)]
			#[doc="` node"]
			$name($type),
		);
	};
	(@enum ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@enum ($($($rest)*)?) $($tail)*);
	};

	// AstNode impl
	(@ast_node $enum:ident $syntax:ident ($(,)?) $($tail:tt)*) => {
		fn from($syntax: $crate::syntax::cst::CstNode) -> Self {
			match $syntax.as_ref().kind() {
				$($tail)*
				#[allow(unreachable_patterns)]
				x => unreachable!("Cannot create {} from {}", stringify!($enum), x)
			}
		}
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $name:ident $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => $enum::$name($name::new($syntax)),);
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $name:ident($type:ty) $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => $enum::$name(<$type>::new($syntax)),);
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => {
			let tree = $syntax.cst();
			let node = $syntax.as_ref();
			let child = tree.node(node.child_by_field_name($expression).unwrap());
			$enum::new(child)
		},);
	};
	(@cst_node $enum:ident ($(,)?) $($tail:tt)*) => {
		fn cst_node(&self) -> &$crate::syntax::cst::CstNode {
			match *self {
				$($tail)*
			}
		}
	};
	(@cst_node $enum:ident ($pattern:pat => $name:ident $(($type:ty))? $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@cst_node $enum ($($($rest)*)?) $($tail)* $enum::$name(ref x) => x.cst_node(),);
	};
	(@cst_node $enum:ident ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@cst_node $enum ($($($rest)*)?) $($tail)*);
	};

	// Conversions impl
	(@cast $enum:ident, $(,)?) => {};
	(@cast $enum:ident, $pattern:pat => $name:ident $(, $($rest:tt)*)?) => {
		impl $crate::syntax::ast::TryCastFrom<$enum> for $name {
			fn from_ref(value: &$enum) -> Option<&Self> {
				match *value {
					$enum::$name(ref x) => Some(x),
					#[allow(unreachable_patterns)]
					_ => None
				}
			}

			fn from(value: $enum) -> Option<Self> {
				match value {
					$enum::$name(x) => Some(x),
					#[allow(unreachable_patterns)]
					_ => None
				}
			}
		}

		impl ::std::convert::From<$name> for $enum {
			fn from(v: $name) -> Self {
				$enum::$name(v)
			}
		}

		ast_enum!(@cast $enum, $($($rest)*)?);
	};
	(@cast $enum:ident, $pattern:pat => $name:ident($type:ty) $(, $($rest:tt)*)?) => {
		impl $crate::syntax::ast::TryCastFrom<$enum> for $type {
			fn from_ref(value: &$enum) -> Option<&Self> {
				match *value {
					$enum::$name(ref x) => Some(x),
					_ => None
				}
			}

			fn from(value: $enum) -> Option<Self> {
				match value {
					$enum::$name(x) => Some(x),
					_ => None
				}
			}
		}

		impl ::std::convert::From<$type> for $enum {
			fn from(v: $type) -> Self {
				$enum::$name(v)
			}
		}

		ast_enum!(@cast $enum, $($($rest)*)?);
	};
	(@cast $enum:ident, $pattern:pat => $expression:expr $(, $($rest:tt)*)?) => {
		ast_enum!(@cast $enum, $($($rest)*)?);
	};
}

pub(crate) use ast_enum;
use tree_sitter::TreeCursor;

use super::{cst::Cst, eprime::EPrimeModel, minizinc::MznModel};

/// ConstraintModel represents an AST of a constraint model
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstraintModel {
	/// MiniZinc model
	MznModel(MznModel),
	/// Essence' model
	EPrimeModel(EPrimeModel),
}

#[cfg(test)]
pub mod test {
	use expect_test::{Expect, ExpectFile};
	use tree_sitter::Parser;

	use super::ConstraintModel;
	use crate::{
		file::InputLang,
		syntax::{cst::Cst, eprime::EPrimeModel, minizinc::MznModel},
	};

	/// Helper to check parsed AST
	pub fn check_ast_with_lang(language: InputLang, source: &str, expected: Expect) {
		let lang = match language {
			InputLang::MiniZinc => tree_sitter_minizinc::language(),
			InputLang::EPrime => tree_sitter_eprime::language(),
			_ => unreachable!("check_ast_with_lang should only be called on model files"),
		};
		let mut parser = Parser::new();
		parser.set_language(lang).unwrap();
		let tree = parser.parse(source.as_bytes(), None).unwrap();
		let cst = Cst::from_str(tree, source);
		let model = match language {
			InputLang::MiniZinc => ConstraintModel::MznModel(MznModel::new(cst)),
			InputLang::EPrime => ConstraintModel::EPrimeModel(EPrimeModel::new(cst)),
			_ => unreachable!("check_ast_with_lang should only be called on model files"),
		};
		expected.assert_debug_eq(&model);
	}

	/// Helper to check parsed AST in MiniZinc
	pub fn check_ast(source: &str, expected: Expect) {
		check_ast_with_lang(InputLang::MiniZinc, source, expected)
	}

	/// Helper to check parsed AST in EPrime
	pub fn check_ast_eprime(source: &str, expected: Expect) {
		check_ast_with_lang(InputLang::EPrime, source, expected)
	}

	/// Helper to check parsed AST storing the expected result in a file
	pub fn check_ast_file(source: &str, expected: ExpectFile) {
		let mut parser = Parser::new();
		parser
			.set_language(tree_sitter_minizinc::language())
			.unwrap();
		let tree = parser.parse(source.as_bytes(), None).unwrap();
		let cst = Cst::from_str(tree, source);
		let model = ConstraintModel::MznModel(MznModel::new(cst));
		expected.assert_debug_eq(&model);
	}
}
