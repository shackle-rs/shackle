//! HIR representation of containers

use super::{Expression, Pattern};
use crate::arena::ArenaIndex;

/// Set literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SetLiteral {
	/// Set values
	pub members: Box<[ArenaIndex<Expression>]>,
}

/// Array literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayLiteral {
	/// Array values
	pub members: Box<[ArenaIndex<Expression>]>,
}

/// Array access
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayAccess {
	/// The array being indexed into
	pub collection: ArenaIndex<Expression>,
	/// The indices
	pub indices: ArenaIndex<Expression>,
}

/// Array comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayComprehension {
	/// Value of the comprehension
	pub template: ArenaIndex<Expression>,
	/// The indices to generate
	pub indices: Option<ArenaIndex<Expression>>,
	/// Generators of the comprehension
	pub generators: Box<[Generator]>,
}

/// Set comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SetComprehension {
	/// Value of the comprehension
	pub template: ArenaIndex<Expression>,
	/// Generators of the comprehension
	pub generators: Box<[Generator]>,
}

/// Comprehension generator
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Generator {
	/// Iterator generator
	Iterator {
		/// Patterns (usually variable names)
		patterns: Box<[ArenaIndex<Pattern>]>,
		/// Expression being iterated over
		collection: ArenaIndex<Expression>,
		/// Where clause
		where_clause: Option<ArenaIndex<Expression>>,
	},
	/// Assignment generator
	Assignment {
		/// Pattern (usually variable name)
		pattern: ArenaIndex<Pattern>,
		/// Assigned value
		value: ArenaIndex<Expression>,
		/// Where clause
		where_clause: Option<ArenaIndex<Expression>>,
	},
}

/// Tuple literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TupleLiteral {
	/// Tuple fields
	pub fields: Box<[ArenaIndex<Expression>]>,
}

/// Record literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecordLiteral {
	/// Record fields (pairs of identifier and expressions)
	pub fields: Box<[(ArenaIndex<Pattern>, ArenaIndex<Expression>)]>,
}
