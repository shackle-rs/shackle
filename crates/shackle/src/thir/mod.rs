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
pub mod sanity_check;
pub mod source;

use std::ops::Index;
use std::ops::IndexMut;

use self::db::Thir;
pub use self::domain::*;
pub use self::expression::*;
pub use self::item::*;
use self::source::Origin;

use crate::arena::Arena;
use crate::arena::ArenaIndex;
pub use crate::hir::Identifier;
use crate::ty::FunctionEntry;
use crate::ty::FunctionResolutionError;
use crate::ty::FunctionType;
use crate::ty::Ty;
use crate::utils::impl_index;

/// A model
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Model {
	annotations: Arena<AnnotationItem>,
	constraints: Arena<ConstraintItem>,
	declarations: Arena<DeclarationItem>,
	enumerations: Arena<EnumerationItem>,
	functions: Arena<FunctionItem>,
	outputs: Arena<OutputItem>,
	solve: Option<SolveItem>,
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

	/// Get the top-level annotation items
	pub fn annotations(&self) -> impl Iterator<Item = (AnnotationId, &AnnotationItem)> {
		self.annotations.iter()
	}

	/// Get the top-level annotation items
	pub fn annotations_mut(&mut self) -> impl Iterator<Item = (AnnotationId, &mut AnnotationItem)> {
		self.annotations.iter_mut()
	}

	/// Add an annotation item
	pub fn add_annotation(&mut self, item: AnnotationItem) -> AnnotationId {
		let idx = self.annotations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints(&self) -> impl Iterator<Item = (ConstraintId, &ConstraintItem)> {
		self.all_constraints().filter(|(_, c)| c.top_level)
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId, &mut ConstraintItem)> {
		self.all_constraints_mut().filter(|(_, c)| c.top_level)
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints(&self) -> impl Iterator<Item = (ConstraintId, &ConstraintItem)> {
		self.constraints.iter()
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId, &mut ConstraintItem)> {
		self.constraints.iter_mut()
	}

	/// Add a constraint item
	pub fn add_constraint(&mut self, item: ConstraintItem) -> ConstraintId {
		let idx = self.constraints.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations(
		&self,
	) -> impl Iterator<Item = (DeclarationId, &DeclarationItem)> {
		self.all_declarations().filter(|(_, d)| d.top_level)
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId, &mut DeclarationItem)> {
		self.all_declarations_mut().filter(|(_, d)| d.top_level)
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations(&self) -> impl Iterator<Item = (DeclarationId, &DeclarationItem)> {
		self.declarations.iter()
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId, &mut DeclarationItem)> {
		self.declarations.iter_mut()
	}

	/// Add a declaration item
	pub fn add_declaration(&mut self, item: DeclarationItem) -> DeclarationId {
		let idx = self.declarations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the enumeration items
	pub fn enumerations(&self) -> impl Iterator<Item = (EnumerationId, &EnumerationItem)> {
		self.enumerations.iter()
	}

	/// Get the enumeration items
	pub fn enumerations_mut(
		&mut self,
	) -> impl Iterator<Item = (EnumerationId, &mut EnumerationItem)> {
		self.enumerations.iter_mut()
	}

	/// Add an enumeration item
	pub fn add_enumeration(&mut self, item: EnumerationItem) -> EnumerationId {
		let idx = self.enumerations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the function items
	pub fn functions(&self) -> impl Iterator<Item = (FunctionId, &FunctionItem)> {
		self.functions.iter()
	}

	/// Get the function items
	pub fn functions_mut(&mut self) -> impl Iterator<Item = (FunctionId, &mut FunctionItem)> {
		self.functions.iter_mut()
	}

	/// Add a function item
	pub fn add_function(&mut self, item: FunctionItem) -> FunctionId {
		let idx = self.functions.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the output items
	pub fn outputs(&self) -> impl Iterator<Item = (OutputId, &OutputItem)> {
		self.outputs.iter()
	}

	/// Get the output item
	pub fn output_mut(&mut self) -> impl Iterator<Item = (OutputId, &mut OutputItem)> {
		self.outputs.iter_mut()
	}

	/// Add an output item
	pub fn add_output(&mut self, item: OutputItem) -> OutputId {
		let idx = self.outputs.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Get the solve item
	pub fn solve(&self) -> Option<&SolveItem> {
		self.solve.as_ref()
	}

	/// Get the solve item
	pub fn solve_mut(&mut self) -> Option<&mut SolveItem> {
		self.solve.as_mut()
	}

	/// Set the solve item
	pub fn set_solve(&mut self, solve: SolveItem) {
		self.solve = Some(solve);
	}

	/// Lookup a function by its signature
	pub fn lookup_function(
		&self,
		db: &dyn Thir,
		name: Identifier,
		args: &[Ty],
	) -> Result<FunctionLookup, FunctionLookupError> {
		let overloads = self.functions().filter_map(|(i, f)| {
			if f.name == name {
				Some((i, f.function_entry(self)))
			} else {
				None
			}
		});
		let (i, fe, ft) = FunctionEntry::match_fn(db.upcast(), overloads, args)?;
		Ok(FunctionLookup {
			function: i,
			fn_entry: fe,
			fn_type: ft,
		})
	}

	/// Lookup a top-level top-level variable or annotation atom
	pub fn lookup_identifier(&self, name: Identifier) -> Option<ResolvedIdentifier> {
		self.top_level_declarations()
			.find_map(|(idx, decl)| {
				if decl.name == Some(name) {
					Some(ResolvedIdentifier::Declaration(idx))
				} else {
					None
				}
			})
			.or_else(|| {
				self.annotations().find_map(|(idx, ann)| {
					if ann.name == Some(name) && ann.parameters.is_none() {
						Some(ResolvedIdentifier::Annotation(idx))
					} else {
						None
					}
				})
			})
	}
}

impl_index!(Model[self, index: AnnotationId] -> AnnotationItem { self.annotations[index] });
impl_index!(Model[self, index: ConstraintId] -> ConstraintItem { self.constraints[index] });
impl_index!(Model[self, index: DeclarationId] -> DeclarationItem { self.declarations[index] });
impl_index!(Model[self, index: EnumerationId] -> EnumerationItem { self.enumerations[index] });
impl_index!(Model[self, index: FunctionId] -> FunctionItem { self.functions[index] });
impl_index!(Model[self, index: OutputId] -> OutputItem { self.outputs[index] });

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

/// Result of looking up a function by its signature
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionLookup {
	/// Id of the resolved function
	pub function: FunctionId,
	/// The function entry (i.e. not instantiated with the call arguments)
	pub fn_entry: FunctionEntry,
	/// The type of the resolved function (i.e. instantiated with the call arguments)
	pub fn_type: FunctionType,
}

impl FunctionLookup {
	/// Create a call to this function (does not have arguments set).
	pub fn into_call(self, db: &dyn Thir, origin: impl Into<Origin>) -> Box<CallBuilder> {
		let origin = origin.into();
		CallBuilder::new(
			self.fn_type.return_type,
			IdentifierBuilder::new(
				Ty::function(db.upcast(), self.fn_type),
				ResolvedIdentifier::Function(self.function),
				origin,
			),
			origin,
		)
	}
}

/// Error representing failure to lookup a function
pub type FunctionLookupError = FunctionResolutionError<FunctionId>;
