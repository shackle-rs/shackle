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
//! - Validation of whole program (see the `validate` module)

pub mod container;
pub mod db;
pub mod expression;
pub mod ids;
pub mod item;
pub mod lower;
pub mod pattern;
pub mod primitive;
pub mod scope;
pub mod source;
pub mod typecheck;
pub mod types;
pub mod validate;

use std::ops::Index;

pub use container::*;
pub use expression::*;
pub use item::*;
pub use pattern::*;
pub use primitive::*;
pub use scope::*;
pub use typecheck::*;
pub use types::*;

use crate::arena::{Arena, ArenaIndex};

use self::ids::LocalItemRef;

/// A model (a single `.mzn` file)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
	/// Items in original order
	pub items: Vec<LocalItemRef>,

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

impl Index<ArenaIndex<Item<Assignment>>> for Model {
	type Output = Item<Assignment>;
	fn index(&self, index: ArenaIndex<Item<Assignment>>) -> &Self::Output {
		&self.assignments[index]
	}
}

impl Index<ArenaIndex<Item<EnumAssignment>>> for Model {
	type Output = Item<EnumAssignment>;
	fn index(&self, index: ArenaIndex<Item<EnumAssignment>>) -> &Self::Output {
		&self.enum_assignments[index]
	}
}

impl Index<ArenaIndex<Item<Constraint>>> for Model {
	type Output = Item<Constraint>;
	fn index(&self, index: ArenaIndex<Item<Constraint>>) -> &Self::Output {
		&self.constraints[index]
	}
}

impl Index<ArenaIndex<Item<Declaration>>> for Model {
	type Output = Item<Declaration>;
	fn index(&self, index: ArenaIndex<Item<Declaration>>) -> &Self::Output {
		&self.declarations[index]
	}
}

impl Index<ArenaIndex<Item<Enumeration>>> for Model {
	type Output = Item<Enumeration>;
	fn index(&self, index: ArenaIndex<Item<Enumeration>>) -> &Self::Output {
		&self.enumerations[index]
	}
}

impl Index<ArenaIndex<Item<Function>>> for Model {
	type Output = Item<Function>;
	fn index(&self, index: ArenaIndex<Item<Function>>) -> &Self::Output {
		&self.functions[index]
	}
}

impl Index<ArenaIndex<Item<Output>>> for Model {
	type Output = Item<Output>;
	fn index(&self, index: ArenaIndex<Item<Output>>) -> &Self::Output {
		&self.outputs[index]
	}
}

impl Index<ArenaIndex<Item<Solve>>> for Model {
	type Output = Item<Solve>;
	fn index(&self, index: ArenaIndex<Item<Solve>>) -> &Self::Output {
		&self.solves[index]
	}
}

impl Index<ArenaIndex<Item<TypeAlias>>> for Model {
	type Output = Item<TypeAlias>;
	fn index(&self, index: ArenaIndex<Item<TypeAlias>>) -> &Self::Output {
		&self.type_aliases[index]
	}
}
