//! HIR representation of items
//!
//! A top-level item `T` is represented as an `Item<T>` which holds
//! the item-specific data `T` as well as the `ItemData` storage for
//! expressions, types, and patterns.
//!
//! Non-top-level items (i.e. let items) currently do not have their own
//! `ItemData` storage, and refer to their top-level item's `ItemData` instead.
//!
//! Since each top-level item contains its own storage, these can be lowered
//! from AST independently (i.e. modifying an item does not need to cause
//! other items to be processed again). Note that currently, this is not fully
//! utilised, as the AST for an entire file is always considered to have
//! changed when modified, so always causes all items in that file to be lowered
//! again (but not ones in other files).

use std::fmt::Write;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::arena::{Arena, ArenaIndex, ArenaMap};
use crate::utils::{debug_print_strings, DebugPrint};

use super::db::Hir;
use super::{source::Origin, Expression, Pattern, Type};

/// An item with its data
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T> {
	inner: T,
	/// The data for this item
	pub data: Box<ItemData>,
}

impl<T> Item<T> {
	/// Create a new item
	pub fn new(item: T, data: ItemData) -> Self {
		Item {
			inner: item,
			data: Box::new(data),
		}
	}
}

impl<'a, T> DebugPrint<'a> for Item<T>
where
	T: std::fmt::Debug,
{
	type Database = dyn Hir + 'a;
	fn debug_print(&self, db: &Self::Database) -> String {
		let mut w = String::new();
		writeln!(&mut w, "Item: {:?}", self.inner).unwrap();
		writeln!(&mut w, "  Expressions:").unwrap();
		for (i, e) in self.data.expressions.iter() {
			writeln!(&mut w, "    {:?}: {:?}", i, e).unwrap();
		}
		writeln!(&mut w, "  Types:").unwrap();
		for (i, t) in self.data.types.iter() {
			writeln!(&mut w, "    {:?}: {:?}", i, t).unwrap();
		}
		writeln!(&mut w, "  Patterns:").unwrap();
		for (i, p) in self.data.patterns.iter() {
			writeln!(&mut w, "    {:?}: {:?}", i, p).unwrap();
		}
		writeln!(&mut w, "  Annotations:").unwrap();
		for (i, e) in self.data.annotations.iter() {
			writeln!(&mut w, "    {:?}: {:?}", i, e).unwrap();
		}
		debug_print_strings(db, &w)
	}
}

impl<T> Deref for Item<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T> DerefMut for Item<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

/// Storage for expressions, types and sub-items owned by an item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemData {
	/// Allocation for expressions
	pub expressions: Arena<Expression>,
	/// Allocation for types
	pub types: Arena<Type>,
	/// Allocation for patterns
	pub patterns: Arena<Pattern>,
	/// Annotations for a given expression
	pub annotations: ArenaMap<Expression, Box<[ArenaIndex<Expression>]>>,
}

impl ItemData {
	/// Create new item data
	pub fn new() -> Self {
		Self::default()
	}

	/// Get the annotations for for the given expression
	pub fn annotations(
		&self,
		index: ArenaIndex<Expression>,
	) -> impl Iterator<Item = ArenaIndex<Expression>> + '_ {
		self.annotations
			.get(index)
			.into_iter()
			.flat_map(|v| v.iter())
			.copied()
	}

	/// Resize arenas to be as small as possible
	pub fn shrink_to_fit(&mut self) {
		self.expressions.shrink_to_fit();
		self.types.shrink_to_fit();
		self.patterns.shrink_to_fit();
		self.annotations.shrink_to_fit();
	}
}

impl Index<ArenaIndex<Expression>> for ItemData {
	type Output = Expression;
	fn index(&self, index: ArenaIndex<Expression>) -> &Self::Output {
		&self.expressions[index]
	}
}

impl IndexMut<ArenaIndex<Expression>> for ItemData {
	fn index_mut(&mut self, index: ArenaIndex<Expression>) -> &mut Self::Output {
		&mut self.expressions[index]
	}
}

impl Index<ArenaIndex<Type>> for ItemData {
	type Output = Type;
	fn index(&self, index: ArenaIndex<Type>) -> &Self::Output {
		&self.types[index]
	}
}

impl IndexMut<ArenaIndex<Type>> for ItemData {
	fn index_mut(&mut self, index: ArenaIndex<Type>) -> &mut Self::Output {
		&mut self.types[index]
	}
}

impl Index<ArenaIndex<Pattern>> for ItemData {
	type Output = Pattern;
	fn index(&self, index: ArenaIndex<Pattern>) -> &Self::Output {
		&self.patterns[index]
	}
}

impl IndexMut<ArenaIndex<Pattern>> for ItemData {
	fn index_mut(&mut self, index: ArenaIndex<Pattern>) -> &mut Self::Output {
		&mut self.patterns[index]
	}
}

/// Maps expressions, types and sub-items back to AST nodes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemDataSourceMap {
	/// Source map for expressions
	pub expression_source: ArenaMap<Expression, Origin>,
	/// Source map for types
	pub type_source: ArenaMap<Type, Origin>,
	/// Source map for patterns
	pub pattern_source: ArenaMap<Pattern, Origin>,
}

impl ItemDataSourceMap {
	/// Create expressions source map
	pub fn new() -> Self {
		Self::default()
	}
}

impl Index<ArenaIndex<Expression>> for ItemDataSourceMap {
	type Output = Origin;
	fn index(&self, index: ArenaIndex<Expression>) -> &Self::Output {
		&self.expression_source[index]
	}
}

impl IndexMut<ArenaIndex<Expression>> for ItemDataSourceMap {
	fn index_mut(&mut self, index: ArenaIndex<Expression>) -> &mut Self::Output {
		&mut self.expression_source[index]
	}
}

impl Index<ArenaIndex<Type>> for ItemDataSourceMap {
	type Output = Origin;
	fn index(&self, index: ArenaIndex<Type>) -> &Self::Output {
		&self.type_source[index]
	}
}

impl IndexMut<ArenaIndex<Type>> for ItemDataSourceMap {
	fn index_mut(&mut self, index: ArenaIndex<Type>) -> &mut Self::Output {
		&mut self.type_source[index]
	}
}

impl Index<ArenaIndex<Pattern>> for ItemDataSourceMap {
	type Output = Origin;
	fn index(&self, index: ArenaIndex<Pattern>) -> &Self::Output {
		&self.pattern_source[index]
	}
}

impl IndexMut<ArenaIndex<Pattern>> for ItemDataSourceMap {
	fn index_mut(&mut self, index: ArenaIndex<Pattern>) -> &mut Self::Output {
		&mut self.pattern_source[index]
	}
}

/// An assignment item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Assignment {
	/// Expression being assigned (usually just an identifier)
	pub assignee: ArenaIndex<Expression>,
	/// Right-hand-side definition
	pub definition: ArenaIndex<Expression>,
}

/// Constraint item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Constraint {
	/// Constraint value
	pub expression: ArenaIndex<Expression>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// A declaration item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Declaration {
	/// Type of declaration
	pub declared_type: ArenaIndex<Type>,
	/// Pattern being declared (usually just an identifier)
	pub pattern: ArenaIndex<Pattern>,
	/// Right-hand-side definition
	pub definition: Option<ArenaIndex<Expression>>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration {
	/// Pattern being declared (an identifier)
	pub pattern: ArenaIndex<Pattern>,
	/// Right-hand-side definition
	pub definition: Option<Box<[EnumerationCase]>>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// An assignment item for an enum
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumAssignment {
	/// Expression being assigned (an identifier)
	pub assignee: ArenaIndex<Expression>,
	/// Enum definition
	pub definition: Box<[EnumerationCase]>,
}

/// An enumeration case definition
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumerationCase {
	/// The name of this case (an identifier)
	pub pattern: ArenaIndex<Pattern>,
	/// The types this case contains
	pub parameters: Box<[ArenaIndex<Type>]>,
}

/// Function item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Function {
	/// Return type of function
	pub return_type: ArenaIndex<Type>,
	/// Pattern (always an identifier)
	pub pattern: ArenaIndex<Pattern>,
	/// Type-inst vars
	pub type_inst_vars: Box<[TypeInstIdentifierDeclaration]>,
	/// Function parameters
	pub parameters: Box<[Parameter]>,
	/// The body of this function
	pub body: Option<ArenaIndex<Expression>>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// Declaration of a type-inst identifier
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeInstIdentifierDeclaration {
	/// The name of this identifier
	pub name: ArenaIndex<Pattern>,
	/// Whether this is an enum ID
	pub is_enum: bool,
	/// Whether this is varifiable
	pub is_varifiable: bool,
	/// Whether this is indexable
	pub is_indexable: bool,
}

/// Function parameter
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Parameter {
	/// Type of declaration
	pub declared_type: ArenaIndex<Type>,
	/// Pattern of the parameter (usually just an identifier)
	pub pattern: Option<ArenaIndex<Pattern>>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// Output item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Output {
	/// Section (always a `StringLiteral` or `None`)
	pub section: Option<ArenaIndex<Expression>>,
	/// Output value
	pub expression: ArenaIndex<Expression>,
}

/// Solve item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Solve {
	/// Solve goal
	pub goal: Goal,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// Solve method and objective
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Goal {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize {
		/// Accessor for `_objective`
		pattern: ArenaIndex<Pattern>,
		/// Objective value
		objective: ArenaIndex<Expression>,
	},
	/// Minimize the given objective
	Minimize {
		/// Accessor for `_objective`
		pattern: ArenaIndex<Pattern>,
		/// Objective value
		objective: ArenaIndex<Expression>,
	},
}

/// Type alias item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeAlias {
	/// Name of this type alias
	pub name: ArenaIndex<Pattern>,
	/// The aliased type
	pub aliased_type: ArenaIndex<Type>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}
