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
use std::ops::IndexMut;

pub use self::domain::*;
pub use self::expression::*;
pub use self::item::*;

use crate::arena::Arena;
use crate::arena::ArenaIndex;
pub use crate::hir::Identifier;
use crate::utils::impl_index;

/// A model
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Model {
	annotations: Arena<Item<Annotation>>,
	constraints: Arena<Item<Constraint>>,
	declarations: Arena<Item<Declaration>>,
	enumerations: Arena<Item<Enumeration>>,
	functions: Arena<Item<Function>>,
	outputs: Arena<Item<Output>>,
	solve: Item<Solve>,
	items: Vec<ItemId>,
}

impl Model {
	/// Get the top-level items
	pub fn top_level_items(&self) -> impl '_ + Iterator<Item = ItemId> {
		self.all_items().filter(|idx| match idx {
			ItemId::Constraint(c) => self[*c].top_level,
			ItemId::Declaration(d) => self[*d].top_level,
			_ => true,
		})
	}

	/// Get all items (including local items)
	pub fn all_items(&self) -> impl '_ + Iterator<Item = ItemId> {
		self.items.iter().copied()
	}

	/// Get the top-level constraint items
	pub fn constraints(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Constraint>>, &Item<Constraint>)> {
		self.all_constraints().filter(|(_, c)| c.top_level)
	}

	/// Get the top-level constraint items
	pub fn constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Constraint>>, &mut Item<Constraint>)> {
		self.all_constraints_mut().filter(|(_, c)| c.top_level)
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Constraint>>, &Item<Constraint>)> {
		self.constraints.iter()
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Constraint>>, &mut Item<Constraint>)> {
		self.constraints.iter_mut()
	}

	/// Add an annotation item
	pub fn add_annotation(&mut self, item: Item<Annotation>) -> ArenaIndex<Item<Annotation>> {
		let idx = self.annotations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Add a constraint item
	pub fn add_constraint(&mut self, item: Item<Constraint>) -> ArenaIndex<Item<Constraint>> {
		let idx = self.constraints.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Declaration>>, &Item<Declaration>)> {
		self.all_declarations().filter(|(_, d)| d.top_level)
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Declaration>>, &mut Item<Declaration>)> {
		self.all_declarations_mut().filter(|(_, d)| d.top_level)
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Declaration>>, &Item<Declaration>)> {
		self.declarations.iter()
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Declaration>>, &mut Item<Declaration>)> {
		self.declarations.iter_mut()
	}

	/// Add a declaration item
	pub fn add_declaration(&mut self, item: Item<Declaration>) -> ArenaIndex<Item<Declaration>> {
		let idx = self.declarations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the enumeration items
	pub fn enumerations(
		&self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Enumeration>>, &Item<Enumeration>)> {
		self.enumerations.iter()
	}

	/// Get the enumeration items
	pub fn enumerations_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Enumeration>>, &mut Item<Enumeration>)> {
		self.enumerations.iter_mut()
	}

	/// Add an enumeration item
	pub fn add_enumeration(&mut self, item: Item<Enumeration>) -> ArenaIndex<Item<Enumeration>> {
		let idx = self.enumerations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the function items
	pub fn functions(&self) -> impl Iterator<Item = (ArenaIndex<Item<Function>>, &Item<Function>)> {
		self.functions.iter()
	}

	/// Get the function items
	pub fn functions_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Function>>, &mut Item<Function>)> {
		self.functions.iter_mut()
	}

	/// Add a function item
	pub fn add_function(&mut self, item: Item<Function>) -> ArenaIndex<Item<Function>> {
		let idx = self.functions.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the output items
	pub fn outputs(&self) -> impl Iterator<Item = (ArenaIndex<Item<Output>>, &Item<Output>)> {
		self.outputs.iter()
	}

	/// Get the output item
	pub fn output_mut(
		&mut self,
	) -> impl Iterator<Item = (ArenaIndex<Item<Output>>, &mut Item<Output>)> {
		self.outputs.iter_mut()
	}

	/// Add an output item
	pub fn add_output(&mut self, item: Item<Output>) -> ArenaIndex<Item<Output>> {
		let idx = self.outputs.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the solve item
	pub fn solve(&self) -> &Item<Solve> {
		&self.solve
	}

	/// Get the solve item
	pub fn solve_mut(&mut self) -> &mut Item<Solve> {
		&mut self.solve
	}
}

impl_index!(Model[self, index: ArenaIndex<Item<Annotation>>] -> Item<Annotation> { self.annotations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Constraint>>] -> Item<Constraint> { self.constraints[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Declaration>>] -> Item<Declaration> { self.declarations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Enumeration>>] -> Item<Enumeration> { self.enumerations[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Function>>] -> Item<Function> { self.functions[index] });
impl_index!(Model[self, index: ArenaIndex<Item<Output>>] -> Item<Output> { self.outputs[index] });

impl Index<ItemId> for Model {
	type Output = ItemData;
	fn index(&self, index: ItemId) -> &Self::Output {
		match index {
			ItemId::Annotation(i) => &self[i].data,
			ItemId::Constraint(i) => &self[i].data,
			ItemId::Declaration(i) => &self[i].data,
			ItemId::Enumeration(i) => &self[i].data,
			ItemId::Function(i) => &self[i].data,
			ItemId::Output(i) => &self[i].data,
		}
	}
}

impl IndexMut<ItemId> for Model {
	fn index_mut(&mut self, index: ItemId) -> &mut Self::Output {
		match index {
			ItemId::Annotation(i) => &mut self[i].data,
			ItemId::Constraint(i) => &mut self[i].data,
			ItemId::Declaration(i) => &mut self[i].data,
			ItemId::Enumeration(i) => &mut self[i].data,
			ItemId::Function(i) => &mut self[i].data,
			ItemId::Output(i) => &mut self[i].data,
		}
	}
}
