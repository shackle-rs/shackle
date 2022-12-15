//! THIR representation
//!

use std::ops::Index;

use crate::arena::Arena;
use crate::hir::Identifier;
use crate::ty::FunctionEntry;
use crate::ty::FunctionResolutionError;
use crate::ty::FunctionType;
use crate::ty::Ty;
use crate::ty::TyVar;
use crate::utils::impl_index;

mod domain;
mod expression;
mod item;

pub use self::domain::*;
pub use self::expression::*;
pub use self::item::*;

use super::db::Thir;

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
		self.all_items()
			.filter(|idx| match idx {
				ItemId::Constraint(c) => self[*c].top_level(),
				ItemId::Declaration(d) => self[*d].top_level(),
				_ => true,
			})
			.chain(self.solve().map(|_| ItemId::Solve))
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
		self.all_constraints().filter(|(_, c)| c.top_level())
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId, &mut ConstraintItem)> {
		self.all_constraints_mut().filter(|(_, c)| c.top_level())
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
		self.all_declarations().filter(|(_, d)| d.top_level())
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId, &mut DeclarationItem)> {
		self.all_declarations_mut().filter(|(_, d)| d.top_level())
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
	pub fn set_solve(&mut self, solve: SolveItem) -> ItemId {
		self.solve = Some(solve);
		ItemId::Solve
	}

	/// Get the expressions storage for an item
	pub fn expressions(&self, item: ItemId) -> &ExpressionAllocator {
		match item {
			ItemId::Annotation(idx) => self[idx].expressions(),
			ItemId::Constraint(idx) => self[idx].expressions(),
			ItemId::Declaration(idx) => self[idx].expressions(),
			ItemId::Enumeration(idx) => self[idx].expressions(),
			ItemId::Function(idx) => self[idx].expressions(),
			ItemId::Output(idx) => self[idx].expressions(),
			ItemId::Solve => self.solve().expect("No solve item").expressions(),
		}
	}

	/// Get the expressions storage for an item
	pub fn expressions_mut(&mut self, item: ItemId) -> &mut ExpressionAllocator {
		match item {
			ItemId::Annotation(idx) => self[idx].expressions_mut(),
			ItemId::Constraint(idx) => self[idx].expressions_mut(),
			ItemId::Declaration(idx) => self[idx].expressions_mut(),
			ItemId::Enumeration(idx) => self[idx].expressions_mut(),
			ItemId::Function(idx) => self[idx].expressions_mut(),
			ItemId::Output(idx) => self[idx].expressions_mut(),
			ItemId::Solve => self.solve_mut().expect("No solve item").expressions_mut(),
		}
	}

	/// Lookup a function by its signature
	///
	/// Prefer using `ExpressionAllocator::lookup_call` to create an call expression.
	pub fn lookup_function(
		&self,
		db: &dyn Thir,
		name: Identifier,
		args: &[Ty],
	) -> Result<FunctionLookup, FunctionLookupError> {
		let overloads = self.functions().filter_map(|(i, f)| {
			if f.name() == name {
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

	/// Lookup a top-level top-level variable or atom
	///
	/// Prefer using `ExpressionAllocator::lookup_identifier` to create an identifier expression.
	pub fn lookup_identifier(&self, db: &dyn Thir, name: Identifier) -> Option<ResolvedIdentifier> {
		self.top_level_declarations()
			.find_map(|(idx, decl)| {
				if decl.name() == Some(name) {
					Some(ResolvedIdentifier::Declaration(idx))
				} else {
					None
				}
			})
			.or_else(|| {
				for (idx, e) in self.enumerations() {
					if e.enum_type().name(db.upcast()) == name.0 {
						return Some(ResolvedIdentifier::Enumeration(idx));
					}
					if let Some(cs) = e.definition() {
						for (j, c) in cs.iter().enumerate() {
							if let Some(n) = c.name {
								if n == name {
									return Some(ResolvedIdentifier::EnumerationMember(
										EnumMemberId::new(idx, j as u32),
									));
								}
							}
						}
					}
				}
				None
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

impl Index<EnumMemberId> for Model {
	type Output = Constructor;
	fn index(&self, index: EnumMemberId) -> &Self::Output {
		&self.enumerations[index.enumeration_id()]
			.definition()
			.expect("No definition for enum")[index.member_index() as usize]
	}
}

impl Index<TyVarId> for Model {
	type Output = TyVar;
	fn index(&self, index: TyVarId) -> &Self::Output {
		&self.functions[index.function_id()].type_inst_vars()[index.ty_var_index() as usize]
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

/// Error representing failure to lookup a function
pub type FunctionLookupError = FunctionResolutionError<FunctionId>;
