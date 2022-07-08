//! Destructuring/case matching patterns
//!

use crate::{arena::ArenaIndex, utils::impl_enum_from};

use super::{
	db::{Hir, HirString, HirStringData},
	BooleanLiteral, Expression, FloatLiteral, IntegerLiteral, StringLiteral,
};

/// A pattern for destructuring
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Pattern {
	/// A single identifier
	Identifier(Identifier),
	/// Don't care wildcard
	Anonymous,
	/// Absent literal
	Absent,
	/// Boolean literal
	Boolean(BooleanLiteral),
	/// Float literal
	Float {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: FloatLiteral,
	},
	/// Integer literal
	Integer {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: IntegerLiteral,
	},
	/// Infinity
	Infinity {
		/// Whether this has been negated
		negated: bool,
	},
	/// String literal
	String(StringLiteral),
	/// Enum constructor pattern
	Call {
		/// Callee identifier
		function: ArenaIndex<Expression>,
		/// Call arguments
		arguments: Box<[ArenaIndex<Pattern>]>,
	},
	/// Tuple pattern
	Tuple {
		/// Tuple fields
		fields: Box<[ArenaIndex<Pattern>]>,
	},
	/// Record pattern
	Record {
		/// Record fields (pairs of field name, field value pattern)
		fields: Box<[(ArenaIndex<Expression>, ArenaIndex<Pattern>)]>,
	},
	/// Indicates an error
	Missing,
}

impl_enum_from!(Pattern::Identifier);
impl_enum_from!(Pattern::Boolean(BooleanLiteral));
impl_enum_from!(Pattern::String(StringLiteral));

/// Identifier
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Identifier(pub HirString);

impl Identifier {
	/// Create a new identifier with the given value
	pub fn new<T: Into<HirStringData>>(v: T, db: &dyn Hir) -> Self {
		Self(db.intern_string(v.into()))
	}

	/// Get the name of this identifier
	pub fn lookup(&self, db: &dyn Hir) -> String {
		db.lookup_intern_string(self.0).0
	}
}
