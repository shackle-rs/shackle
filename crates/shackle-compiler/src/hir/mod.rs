//! High-level intermediate representation.
//!
//! This representation is used for name resolution and type checking.
//!
//! This is also the final representation used by the language server, and as
//! such is the final representation which needs to be continue as far as
//! possible in the presence of errors.
//!
//! The steps which occur using this representation are
//! - Lowering of AST to HIR (see the `lower` module)
//! - Scope collection (see the `scope` module)
//! - Computing types of expressions and declarations, identifier resolution
//!   (see the `ty` and `typecheck` modules)
//! - Checking case expressions for exhaustiveness (see the `pattern_matching`)
//!   module
//! - Validation of whole program (see the `validate` module)

pub mod container;
pub mod db;
pub mod expression;
pub mod ids;
pub mod item;
pub mod lower;
pub mod pattern;
pub mod pattern_matching;
pub mod primitive;
pub mod scope;
pub mod source;
pub mod typecheck;
pub mod types;
pub mod validate;

pub use container::*;
pub use expression::*;
pub use item::*;
pub use pattern::*;
pub use primitive::*;
pub use scope::*;
pub use typecheck::*;
pub use types::*;

use self::ids::LocalItemRef;
use crate::utils::{
	arena::{Arena, ArenaIndex},
	impl_index,
};

/// A model (a single `.mzn` file)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
	/// Items in original order
	pub items: Vec<LocalItemRef>,

	/// Annotation items
	pub annotations: Arena<Item<Annotation>>,
	/// Assignment items
	pub assignments: Arena<Item<Assignment>>,
	/// Enum assignments
	pub enum_assignments: Arena<Item<EnumAssignment>>,
	/// Constraint items
	pub constraints: Arena<Item<Constraint>>,
	/// Declaration items
	pub declarations: Arena<Item<Declaration>>,
	/// Enumeration items
	pub enumerations: Arena<Item<Enumeration>>,
	/// Function items
	pub functions: Arena<Item<Function>>,
	/// Output items
	pub outputs: Arena<Item<Output>>,
	/// Solve items (only one should be present across entire program, but we
	/// allow for many to allow typechecking to occur)
	pub solves: Arena<Item<Solve>>,
	/// Type alias items
	pub type_aliases: Arena<Item<TypeAlias>>,
}

impl_index!(Model[self, index: ArenaIndex<Item<Annotation>>] -> Item<Annotation> { self.annotations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Assignment>>] -> Item<Assignment> { self.assignments[index] });
impl_index!(Model[self, index: ArenaIndex<Item<EnumAssignment>>] -> Item<EnumAssignment> { self.enum_assignments[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Constraint>>] -> Item<Constraint> { self.constraints[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Declaration>>] -> Item<Declaration> { self.declarations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Enumeration>>] -> Item<Enumeration> { self.enumerations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Function>>] -> Item<Function> { self.functions[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Output>>] -> Item<Output> { self.outputs[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Solve>>] -> Item<Solve> { self.solves[index] });
impl_index!(Model[self, index: ArenaIndex<Item<TypeAlias>>] -> Item<TypeAlias> { self.type_aliases[index] });
