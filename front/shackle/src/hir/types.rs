//! HIR representation of types written in a model (not computed types)
//!

use crate::arena::ArenaIndex;

use super::{
	db::{Hir, HirString, HirStringData},
	Expression, Identifier,
};

/// Type of an expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
	/// Array type
	Array {
		/// Type of dimensions
		dimensions: Box<[ArrayDimension]>,
		/// Type of element
		element: ArenaIndex<Type>,
	},
	/// Basic type
	Base(TypeBase),
	/// Tuple type
	Tuple(Box<[ArenaIndex<Type>]>),
	/// Record type
	Record(Box<[(Identifier, ArenaIndex<Type>)]>),
	/// Operation (function) type
	Operation {
		/// Return type
		return_type: ArenaIndex<Type>,
		/// Parameter types
		parameter_types: Box<[ArenaIndex<Type>]>,
	},

	/// Sentinel indicating an error during lowering
	Missing,
}

/// Basic type of a value
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TypeBase {
	/// Type is normal
	NonAny {
		/// Whether the type is var
		is_var: bool,
		/// Whether the type is opt
		is_opt: bool,
		/// Whether the type is a set
		set_type: bool,
		/// The domain
		domain: Domain,
	},
	/// Type is any
	Any,
	/// Type is any $T
	AnyTi(TypeInstIdentifier),
}

/// Array dimension type
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ArrayDimension {
	/// An expression for the index set (could also be `_`)
	Expression(ArenaIndex<Expression>),
	/// Type-inst identifier `$T`
	TypeInstIdentifier(TypeInstIdentifier),
	/// Type-inst enum identifier `$$E`
	TypeInstEnumIdentifier(TypeInstEnumIdentifier),
	/// `int` index set (1..n)
	Integer,

	/// Sentinel indicating an error during lowering
	Missing,
}

/// Declaration domain
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Domain {
	/// Bounded domain
	Bounded(ArenaIndex<Expression>),
	/// Type-inst identifier `$T`
	TypeInstIdentifier(TypeInstIdentifier),
	/// Type-inst enum identifier `$$E`
	TypeInstEnumIdentifier(TypeInstEnumIdentifier),
	/// Unbounded domain
	Unbounded(PrimitiveType),
}

/// Primitive base type
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
	/// `ann` type
	Ann,
	/// `bool` type
	Bool,
	/// `float` type
	Float,
	/// `int` type
	Int,
	/// `string` type
	String,
}

/// Type-inst identifier `$T`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeInstIdentifier(pub HirString);

impl TypeInstIdentifier {
	/// Create a new identifier with the given name (without $)
	pub fn new<T: Into<HirStringData>>(v: T, db: &dyn Hir) -> Self {
		Self(db.intern_string(v.into()))
	}
}

/// Type-inst enum identifier `$$E`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeInstEnumIdentifier(pub HirString);

impl TypeInstEnumIdentifier {
	/// Create a new type-inst enum identifier with the given name (without $$)
	pub fn new<T: Into<HirStringData>>(v: T, db: &dyn Hir) -> Self {
		Self(db.intern_string(v.into()))
	}
}
