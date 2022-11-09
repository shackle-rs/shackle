//! Typed high-level intermediate representation.
//!
//! This module provides (almost) all constructs available in the HIR, along
//! with type and name resolution information computed during typechecking.
//!
//! Since this phase is post-HIR, it is not designed to be incremental.
//! An API is provided to allow us to perform transformations/modifications.
//!
//! This representation is used to generate the MIR.

pub mod db;
pub mod domain;
pub mod expression;
pub mod item;
pub mod lower;
pub mod pretty_print;

use std::ops::Index;

pub use self::domain::*;
pub use self::expression::*;
pub use self::item::*;

use crate::arena::Arena;
use crate::arena::ArenaIndex;
pub use crate::hir::Identifier;

/// A model
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
	annotations: Arena<Item<Annotation>>,
	constraints: Arena<Item<Constraint>>,
	declarations: Arena<Item<Declaration>>,
	enumerations: Arena<Item<Enumeration>>,
	functions: Arena<Item<Function>>,
	outputs: Arena<Item<Output>>,
	solve: Item<Solve>,

	top_level: Vec<ItemId>,
}

impl Model {
	/// Get the constraint items
	pub fn constraints(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Constraint>>, &Item<Constraint>)> {
		self.constraints.iter()
	}

	/// Add an annotation item
	pub fn add_annotation(&mut self, item: Item<Annotation>) -> ArenaIndex<Item<Annotation>> {
		self.annotations.insert(item)
	}

	/// Add a constraint item
	pub fn add_constraint(&mut self, item: Item<Constraint>) -> ArenaIndex<Item<Constraint>> {
		self.constraints.insert(item)
	}

	/// Get the declaration items
	pub fn declarations(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Declaration>>, &Item<Declaration>)> {
		self.declarations.iter()
	}

	/// Add a declaration item
	pub fn add_declaration(&mut self, item: Item<Declaration>) -> ArenaIndex<Item<Declaration>> {
		self.declarations.insert(item)
	}

	/// Get the enumeration items
	pub fn enumerations(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Enumeration>>, &Item<Enumeration>)> {
		self.enumerations.iter()
	}

	/// Add an enumeration item
	pub fn add_enumeration(&mut self, item: Item<Enumeration>) -> ArenaIndex<Item<Enumeration>> {
		self.enumerations.insert(item)
	}

	/// Get the function items
	pub fn functions(&self) -> impl Iterator<Item = (ArenaIndex<Item<Function>>, &Item<Function>)> {
		self.functions.iter()
	}

	/// Add a function item
	pub fn add_function(&mut self, item: Item<Function>) -> ArenaIndex<Item<Function>> {
		self.functions.insert(item)
	}

	/// Get the output items
	pub fn outputs(&self) -> impl Iterator<Item = (ArenaIndex<Item<Output>>, &Item<Output>)> {
		self.outputs.iter()
	}

	/// Add an output item
	pub fn add_output(&mut self, item: Item<Output>) -> ArenaIndex<Item<Output>> {
		self.outputs.insert(item)
	}

	/// Get the solve item
	pub fn solve(&self) -> &Item<Solve> {
		&self.solve
	}
}

impl Index<ArenaIndex<Item<Annotation>>> for Model {
	type Output = Item<Annotation>;
	fn index(&self, index: ArenaIndex<Item<Annotation>>) -> &Self::Output {
		&self.annotations[index]
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
