//! HIR representation of primitive values
//!
use crate::db::{InternedString, InternedStringData};

use super::db::Hir;
use std::fmt;

/// An integer literal
#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct IntegerLiteral(pub i64);

/// A boolean literal
#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct BooleanLiteral(pub bool);

/// A float literal
///
/// Uses u64 for storage so that Eq and Hash can be defined
/// (since float literals in MiniZinc are always finite)
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
pub struct FloatLiteral(u64);

impl FloatLiteral {
	/// Create a new float literal
	pub fn new(v: f64) -> Self {
		Self(v.to_bits())
	}

	/// Get the value of this float literal
	pub fn value(&self) -> f64 {
		f64::from_bits(self.0)
	}
}

impl fmt::Debug for FloatLiteral {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("FloatLiteral").field(&self.value()).finish()
	}
}

/// A string literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StringLiteral(InternedString);

impl StringLiteral {
	/// Create a new string literal
	pub fn new<T: Into<InternedStringData>>(v: T, db: &dyn Hir) -> Self {
		Self(db.intern_string(v.into()))
	}

	/// Get the value of this string literal
	pub fn value(&self, db: &dyn Hir) -> String {
		db.lookup_intern_string(self.0).into()
	}
}

impl From<InternedString> for StringLiteral {
	fn from(s: InternedString) -> Self {
		Self(s)
	}
}
