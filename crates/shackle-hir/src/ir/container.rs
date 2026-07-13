//! HIR representation of containers

use crate::{ExpressionId, PatternId};

/// Set literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct SetLiteral<'db> {
	/// Set values
	pub members: Box<[ExpressionId<'db>]>,
}

/// Array literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct ArrayLiteral<'db> {
	/// Array values
	pub members: Box<[ExpressionId<'db>]>,
}

/// 2D array literal row/column index set
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum MaybeIndexSet<'db> {
	/// Index set not specified
	NonIndexed(usize),
	/// Index set specified
	Indexed(Box<[ExpressionId<'db>]>),
}

impl<'db> MaybeIndexSet<'db> {
	/// Get the number of index sets
	#[allow(clippy::len_without_is_empty, reason = "Always at least one index set")]
	pub fn len(&self) -> usize {
		match self {
			Self::NonIndexed(count) => *count,
			Self::Indexed(indices) => indices.len(),
		}
	}
}

/// 2D array literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct ArrayLiteral2D<'db> {
	/// Row indices
	pub rows: MaybeIndexSet<'db>,
	/// Column indices
	pub columns: MaybeIndexSet<'db>,
	/// Array values
	pub members: Box<[ExpressionId<'db>]>,
}

/// Indexed array literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct IndexedArrayLiteral<'db> {
	/// Indices
	pub indices: Box<[ExpressionId<'db>]>,
	/// Array values
	pub members: Box<[ExpressionId<'db>]>,
}

/// Array access
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct ArrayAccess<'db> {
	/// The array being indexed into
	pub collection: ExpressionId<'db>,
	/// The indices
	pub indices: ExpressionId<'db>,
}

/// Array comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct ArrayComprehension<'db> {
	/// Value of the comprehension
	pub template: ExpressionId<'db>,
	/// The indices to generate
	pub indices: Option<ExpressionId<'db>>,
	/// Generators of the comprehension
	pub generators: Box<[Generator<'db>]>,
}

/// Set comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct SetComprehension<'db> {
	/// Value of the comprehension
	pub template: ExpressionId<'db>,
	/// Generators of the comprehension
	pub generators: Box<[Generator<'db>]>,
}

/// Comprehension generator
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum Generator<'db> {
	/// Iterator generator
	Iterator {
		/// Patterns (usually variable names)
		patterns: Box<[PatternId<'db>]>,
		/// Expression being iterated over
		collection: ExpressionId<'db>,
		/// Where clause
		where_clause: Option<ExpressionId<'db>>,
	},
	/// Assignment generator
	Assignment {
		/// Pattern (usually variable name)
		pattern: PatternId<'db>,
		/// Assigned value
		value: ExpressionId<'db>,
		/// Where clause
		where_clause: Option<ExpressionId<'db>>,
	},
}

/// Tuple literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct TupleLiteral<'db> {
	/// Tuple fields
	pub fields: Box<[ExpressionId<'db>]>,
}

/// Record literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct RecordLiteral<'db> {
	/// Record fields (pairs of identifier and expressions)
	pub fields: Box<[(PatternId<'db>, ExpressionId<'db>)]>,
}
