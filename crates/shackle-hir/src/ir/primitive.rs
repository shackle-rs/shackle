//! HIR representation of primitive values
//!
use shackle_utils::InternedString;

use crate::Db;

/// An integer literal
#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct IntegerLiteral(pub i64);

impl std::fmt::Display for IntegerLiteral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

/// A boolean literal
#[derive(Copy, Clone, Default, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct BooleanLiteral(pub bool);

impl std::fmt::Display for BooleanLiteral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

/// A float literal
///
/// Uses u64 for storage so that Eq and Hash can be defined
/// (since float literals in MiniZinc are always finite)
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, salsa::SalsaValue)]
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

impl std::fmt::Debug for FloatLiteral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("FloatLiteral").field(&self.value()).finish()
	}
}

impl std::fmt::Display for FloatLiteral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value().fmt(f)
	}
}

/// A string literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct StringLiteral<'db>(pub InternedString<'db>);

impl<'db> StringLiteral<'db> {
	/// Create a new string literal
	pub fn new(db: &'db dyn Db, v: impl AsRef<str>) -> Self {
		Self(InternedString::new(db, v.as_ref()))
	}

	/// Get the value of this string literal
	pub fn value(&self, db: &'db dyn Db) -> &'db str {
		self.0.lookup(db)
	}
}

impl<'db, T: Into<InternedString<'db>>> From<T> for StringLiteral<'db> {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}

impl std::fmt::Display for StringLiteral<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}
