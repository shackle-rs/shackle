//! Helper utilities for dealing with AST nodes.
use std::{fmt::Debug, marker::PhantomData};

use crate::cst::{ChildrenWithFieldName, Cst, CstNode};

/// Base trait for AST nodes
pub trait AstNode<'tree>: Debug {
	/// Create a new node
	fn new(node: CstNode<'tree>) -> Self
	where
		Self: Sized + From<CstNode<'tree>>,
	{
		Self::from(node)
	}

	/// Get the underlying CST node
	fn cst_node(&self) -> &CstNode<'tree>;

	/// Get the (concrete) text content of this node
	fn cst_text<'a>(&self, source: &'a str) -> &'a str {
		self.cst_node().text(source)
	}

	/// Get the kind of the CST node
	fn cst_kind(&self) -> &'tree str {
		self.cst_node().kind()
	}

	/// Get the span of this node
	fn span(&self) -> SourceSpan {
		self.cst_node().span()
	}

	/// Whether this node is missing
	fn is_missing(&self) -> bool {
		self.cst_node().is_missing()
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
pub struct Children<'tree, T> {
	/// Untyped iterator over the matching CST children.
	inner: ChildrenWithFieldName<'tree>,
	/// Marker for the AST child type yielded by this iterator.
	phantom: PhantomData<T>,
}

impl<'tree, T> From<ChildrenWithFieldName<'tree>> for Children<'tree, T> {
	fn from(value: ChildrenWithFieldName<'tree>) -> Self {
		Children {
			inner: value,
			phantom: PhantomData,
		}
	}
}

impl<'tree, T> Clone for Children<'tree, T> {
	fn clone(&self) -> Self {
		Children {
			inner: self.inner.clone(),
			phantom: PhantomData,
		}
	}
}

impl<'tree, T: From<CstNode<'tree>>> Iterator for Children<'tree, T> {
	type Item = T;

	fn next(&mut self) -> Option<T> {
		self.inner.next().map(T::from)
	}
}

impl<'tree, T: Debug + From<CstNode<'tree>>> Debug for Children<'tree, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut iter = self.clone();
		iter.inner.reset();
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
pub(crate) fn child_with_field_name<'tree, T: AstNode<'tree>, U: From<CstNode<'tree>>>(
	parent: &T,
	field: &str,
) -> U {
	parent.cst_node().child_with_field_name(field).into()
}

/// Helper to retrieve a child node by its field name
pub(crate) fn optional_child_with_field_name<'tree, T: AstNode<'tree>, U: From<CstNode<'tree>>>(
	parent: &T,
	field: &str,
) -> Option<U> {
	parent
		.cst_node()
		.optional_child_with_field_name(field)
		.map(U::from)
}

/// Helper to retrieve child nodes by field name
pub(crate) fn children_with_field_name<'tree, T: AstNode<'tree>, U: From<CstNode<'tree>>>(
	parent: &T,
	field: &str,
) -> Children<'tree, U> {
	Children {
		inner: parent.cst_node().children_with_field_name(field),
		phantom: PhantomData,
	}
}

/// Helper to decode the string contained in a CST node
pub(crate) fn decode_string_literal(cst_node: &CstNode, source: &str) -> String {
	cst_node
		.children_with_field_name("content")
		.map(|c| match c.kind() {
			"string_characters" => c.text(source).to_owned(),
			"escape_sequence" => {
				let e = c.child_with_field_name("escape");
				match e.kind() {
					"octal" => char::from_u32(u32::from_str_radix(e.text(source), 8).unwrap())
						.unwrap()
						.to_string(),
					"hexadecimal" => {
						char::from_u32(u32::from_str_radix(e.text(source), 16).unwrap())
							.unwrap()
							.to_string()
					}
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
		#[derive(Clone, Eq, PartialEq, Hash)]
		pub struct $name<'tree> {
			syntax: $crate::cst::CstNode<'tree>,
		}

		impl<'tree> ::std::convert::From<$crate::cst::CstNode<'tree>> for $name<'tree> {
			fn from(syntax: $crate::cst::CstNode<'tree>) -> Self {
				$name { syntax }
			}
		}

		impl<'tree> $crate::ast::AstNode<'tree> for $name<'tree> {
			fn cst_node(&self) -> &$crate::cst::CstNode<'tree> {
				&self.syntax
			}
		}

		impl<'tree> ::std::fmt::Debug for $name<'tree> {
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
		ast_enum!(@enum ($($tail)+) ($(#[$meta])* #[derive(Clone, Eq, PartialEq, Hash, Debug)] pub enum $name<'tree>));
		ast_enum!(@cast $name, $($tail)+);

		impl<'tree> ::std::convert::From<$crate::cst::CstNode<'tree>> for $name<'tree> {
			ast_enum!(@ast_node $name syntax ($($tail)+));
		}

		impl<'tree> $crate::ast::AstNode<'tree> for $name<'tree> {
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
			$name($name<'tree>),
		);
	};
	(@enum ($pattern:pat => $name:ident($type:ident) $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@enum ($($($rest)*)?) $($tail)*
			#[doc="`"]
			#[doc=stringify!($type)]
			#[doc="` node"]
			$name($type<'tree>),
		);
	};
	(@enum ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@enum ($($($rest)*)?) $($tail)*);
	};

	// AstNode impl
	(@ast_node $enum:ident $syntax:ident ($(,)?) $($tail:tt)*) => {
		fn from($syntax: $crate::cst::CstNode<'tree>) -> Self {
			match $syntax.kind() {
				$($tail)*
				#[allow(unreachable_patterns, reason = "May not be unreachable")]
				x => unreachable!("Cannot create {} from {}", stringify!($enum), x)
			}
		}
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $name:ident $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => $enum::$name($name::new($syntax)),);
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $name:ident($type:ident) $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => $enum::$name(<$type>::new($syntax)),);
	};
	(@ast_node $enum:ident $syntax:ident ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@ast_node $enum $syntax ($($($rest)*)?) $($tail)* $pattern => {
			let child = $syntax.child_with_field_name($expression);
			$enum::new(child)
		},);
	};
	(@cst_node $enum:ident ($(,)?) $($tail:tt)*) => {
		fn cst_node(&self) -> &$crate::cst::CstNode<'tree> {
			match *self {
				$($tail)*
			}
		}
	};
	(@cst_node $enum:ident ($pattern:pat => $name:ident $(($type:ident))? $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@cst_node $enum ($($($rest)*)?) $($tail)* $enum::$name(ref x) => x.cst_node(),);
	};
	(@cst_node $enum:ident ($pattern:pat => $expression:expr $(, $($rest:tt)*)?) $($tail:tt)*) => {
		ast_enum!(@cst_node $enum ($($($rest)*)?) $($tail)*);
	};

	// Conversions impl
	(@cast $enum:ident, $(,)?) => {};
	(@cast $enum:ident, $pattern:pat => $name:ident $(, $($rest:tt)*)?) => {
		impl<'tree> $crate::ast::TryCastFrom<$enum<'tree>> for $name<'tree> {
			fn from_ref<'a>(value: &'a $enum<'tree>) -> Option<&'a Self> {
				match *value {
					$enum::$name(ref x) => Some(x),
					_ => None
				}
			}

			fn from(value: $enum<'tree>) -> Option<Self> {
				match value {
					$enum::$name(x) => Some(x),
					_ => None
				}
			}
		}

		impl<'tree> ::std::convert::From<$name<'tree>> for $enum<'tree> {
			fn from(v: $name<'tree>) -> Self {
				$enum::$name(v)
			}
		}

		ast_enum!(@cast $enum, $($($rest)*)?);
	};
	(@cast $enum:ident, $pattern:pat => $name:ident($type:ident) $(, $($rest:tt)*)?) => {
		impl<'tree> $crate::ast::TryCastFrom<$enum<'tree>> for $type<'tree> {
			fn from_ref<'a>(value: &'a $enum<'tree>) -> Option<&'a Self> {
				match *value {
					$enum::$name(ref x) => Some(x),
					_ => None
				}
			}

			fn from(value: $enum<'tree>) -> Option<Self> {
				match value {
					$enum::$name(x) => Some(x),
					_ => None
				}
			}
		}

		impl<'tree> ::std::convert::From<$type<'tree>> for $enum<'tree> {
			fn from(v: $type<'tree>) -> Self {
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
use derive_more::From;
use miette::SourceSpan;

use super::{eprime::EPrimeModel, minizinc::MznModel};

/// ConstraintModel represents an AST of a constraint model
#[derive(Clone, From, Eq, PartialEq, Hash, Debug)]
pub enum ConstraintModel {
	/// MiniZinc model
	MznModel(MznModel),
	/// Essence' model
	EPrimeModel(EPrimeModel),
}

impl ConstraintModel {
	/// Get the CST
	pub fn cst(&self) -> &Cst {
		match self {
			ConstraintModel::MznModel(m) => m.cst(),
			ConstraintModel::EPrimeModel(m) => m.cst(),
		}
	}
}

/// Module for testing AST
#[cfg(test)]
/// Test utilities for the AST nodes.
pub mod tests {
	use expect_test::{Expect, ExpectFile};

	use super::ConstraintModel;
	use crate::{InputLang, cst::Cst, eprime::EPrimeModel, minizinc::MznModel};

	/// Helper to check parsed AST
	pub fn check_ast_with_lang(language: InputLang, source: &str, expected: Expect) {
		let cst = Cst::new(source, language);
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
		let cst = Cst::new(source, InputLang::MiniZinc);
		let model = ConstraintModel::MznModel(MznModel::new(cst));
		expected.assert_debug_eq(&model);
	}
}
