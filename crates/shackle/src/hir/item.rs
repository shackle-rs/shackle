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
use std::ops::{Deref, DerefMut};

use crate::arena::{Arena, ArenaIndex, ArenaMap};
use crate::utils::{debug_print_strings, impl_enum_from, impl_index, DebugPrint};

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

impl_index!(ItemData[self, index: ArenaIndex<Expression>] -> Expression {self.expressions[index]});
impl_index!(ItemData[self, index: ArenaIndex<Type>] -> Type {self.types[index]});
impl_index!(ItemData[self, index: ArenaIndex<Pattern>] -> Pattern {self.patterns[index]});

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

impl_index!(ItemDataSourceMap[self, index: ArenaIndex<Expression>] -> Origin { self.expression_source[index]});
impl_index!(ItemDataSourceMap[self, index: ArenaIndex<Type>] -> Origin { self.type_source[index]});
impl_index!(ItemDataSourceMap[self, index: ArenaIndex<Pattern>] -> Origin { self.pattern_source[index]});

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

/// A constructor atom or function for an enum or annotations
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Constructor {
	/// Atomic constructor
	Atom {
		/// Pattern being declared (always an identifier)
		pattern: ArenaIndex<Pattern>,
	},
	/// Functional constructor
	Function {
		/// Pattern being declared (always an identifier)
		constructor: ArenaIndex<Pattern>,
		/// Pattern for deconstructor (always an identifier with ^-1)
		deconstructor: ArenaIndex<Pattern>,
		/// Constructor parameters
		parameters: Box<[ConstructorParameter]>,
	},
}

impl Constructor {
	/// Get the pattern for this constructor
	pub fn constructor_pattern(&self) -> ArenaIndex<Pattern> {
		match self {
			Constructor::Atom { pattern } => *pattern,
			Constructor::Function { constructor, .. } => *constructor,
		}
	}

	/// Get the parameters for this constructor
	pub fn parameters(&self) -> impl '_ + Iterator<Item = &ConstructorParameter> {
		let params = match self {
			Constructor::Function { parameters, .. } => Some(parameters),
			_ => None,
		};
		params.into_iter().flat_map(|ps| ps.iter())
	}
}

/// A constructor function parameter
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ConstructorParameter {
	/// Type of declaration
	pub declared_type: ArenaIndex<Type>,
	/// Pattern of the parameter (usually just an identifier)
	pub pattern: Option<ArenaIndex<Pattern>>,
}

/// An annotation item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Annotation {
	/// The constructor this annotation item declares
	pub constructor: Constructor,
}

impl Deref for Annotation {
	type Target = Constructor;
	fn deref(&self) -> &Self::Target {
		&self.constructor
	}
}

impl DerefMut for Annotation {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.constructor
	}
}

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration {
	/// Pattern being declared (an identifier)
	pub pattern: ArenaIndex<Pattern>,
	/// Right-hand-side definition
	pub definition: Option<Box<[EnumConstructor]>>,
	/// Annotations
	pub annotations: Box<[ArenaIndex<Expression>]>,
}

/// An assignment item for an enum
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumAssignment {
	/// Expression being assigned (an identifier)
	pub assignee: ArenaIndex<Expression>,
	/// Enum definition
	pub definition: Box<[EnumConstructor]>,
}

/// An enum constructor (i.e. can be anonymous)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum EnumConstructor {
	/// Anonymous constructor
	Anonymous {
		/// Anonymous pattern
		pattern: ArenaIndex<Pattern>,
		/// Parameters
		parameters: Box<[ConstructorParameter]>,
	},
	/// Named constructor
	Named(Constructor),
}

impl_enum_from!(EnumConstructor::Named(Constructor));

impl EnumConstructor {
	/// Get the pattern for this enum constructor if there is one
	pub fn constructor_pattern(&self) -> ArenaIndex<Pattern> {
		match self {
			EnumConstructor::Anonymous { pattern, .. } => *pattern,
			EnumConstructor::Named(c) => c.constructor_pattern(),
		}
	}

	/// Get the parameters for this constructor
	pub fn parameters(&self) -> impl '_ + Iterator<Item = &ConstructorParameter> {
		let params = match self {
			EnumConstructor::Anonymous { parameters, .. }
			| EnumConstructor::Named(Constructor::Function { parameters, .. }) => Some(parameters),
			_ => None,
		};
		params.into_iter().flat_map(|fs| fs.iter())
	}
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
