//! THIR representation
//!

use std::ops::{Deref, Index};

use rustc_hash::FxHashMap;
use shackle_hir::{Identifier, ids::NodeRef};
use shackle_ty::{
	FunctionResolutionError, OverloadedFunction, Ty, TyParamInstantiations, match_fn,
};
use shackle_utils::{TypedIndex, arena::Arena};

mod annotations;
mod domain;
mod expression;
pub mod follow;
mod item;
pub mod traverse;

pub use self::{annotations::*, domain::*, expression::*, item::*};
use super::source::Origin;
use crate::{Db, counts::ItemCounts};

/// A model
#[derive(Debug, Clone, PartialEq, Eq, Default, TypedIndex, salsa::Update)]
pub struct Model<'db, T: Marker = ()> {
	#[index_mut(AnnotationId<'db, T>)]
	annotations: Arena<AnnotationItem<'db, T>>,
	#[index_mut(ConstraintId<'db, T>)]
	constraints: Arena<ConstraintItem<'db, T>>,
	#[index_mut(DeclarationId<'db, T>)]
	declarations: Arena<DeclarationItem<'db, T>>,
	#[index_mut(EnumerationId<'db, T>)]
	enumerations: Arena<EnumerationItem<'db, T>>,
	#[index_mut(FunctionId<'db, T>)]
	functions: Arena<FunctionItem<'db, T>>,
	#[index_mut(OutputId<'db, T>)]
	outputs: Arena<OutputItem<'db, T>>,
	solve: Option<SolveItem<'db, T>>,
	#[index_mut(usize)]
	items: Vec<ItemId<'db, T>>,
}

impl<'db, T: Marker> Model<'db, T> {
	/// Create a model able to store the given numbers entities without reallocating
	pub fn with_capacities(capacities: &ItemCounts) -> Self {
		Self {
			annotations: Arena::with_capacity(capacities.annotations),
			constraints: Arena::with_capacity(capacities.constraints),
			declarations: Arena::with_capacity(capacities.declarations),
			enumerations: Arena::with_capacity(capacities.enumerations),
			functions: Arena::with_capacity(capacities.functions),
			outputs: Arena::with_capacity(capacities.outputs),
			items: Vec::with_capacity(
				(capacities.annotations
					+ capacities.constraints
					+ capacities.declarations
					+ capacities.enumerations
					+ capacities.functions
					+ capacities.outputs) as usize,
			),
			..Default::default()
		}
	}

	/// Get the item counts
	pub fn item_counts(&self) -> ItemCounts {
		ItemCounts {
			annotations: self.annotations_len(),
			constraints: self.constraints_len(),
			declarations: self.declarations_len(),
			enumerations: self.enumerations_len(),
			functions: self.functions_len(),
			outputs: self.outputs_len(),
		}
	}

	/// Get the top-level items
	pub fn top_level_items(&self) -> impl '_ + Iterator<Item = ItemId<'db, T>> {
		self.all_items()
			.filter(|idx| match idx {
				ItemId::Constraint(c) => self[*c].top_level(),
				ItemId::Declaration(d) => self[*d].top_level(),
				ItemId::Function(f) => self[*f].top_level(),
				_ => true,
			})
			.chain(self.solve().map(|_| ItemId::Solve))
	}

	/// Get all items (including local items)
	pub fn all_items(&self) -> impl '_ + Iterator<Item = ItemId<'db, T>> {
		self.items.iter().copied()
	}

	/// Get the origin for an item
	pub fn item_origin(&self, item: ItemId<'db, T>) -> Origin<'db> {
		match item {
			ItemId::Annotation(idx) => self[idx].origin(),
			ItemId::Constraint(idx) => self[idx].origin(),
			ItemId::Declaration(idx) => self[idx].origin(),
			ItemId::Enumeration(idx) => self[idx].origin(),
			ItemId::Function(idx) => self[idx].origin(),
			ItemId::Output(idx) => self[idx].origin(),
			ItemId::Solve => self.solve().unwrap().origin(),
		}
	}

	fn is_top_level_item_id(&self, item: ItemId<'db, T>) -> bool {
		match item {
			ItemId::Annotation(_) | ItemId::Enumeration(_) | ItemId::Output(_) => true,
			ItemId::Constraint(c) => self[c].top_level(),
			ItemId::Declaration(d) => self[d].top_level(),
			ItemId::Function(f) => self[f].top_level(),
			ItemId::Solve => true,
		}
	}

	fn origin_hir_item(
		&self,
		db: &'db dyn Db,
		item: ItemId<'db, T>,
	) -> Option<shackle_hir::Item<'db>> {
		match self.item_origin(item) {
			Origin::HirNode(NodeRef::Item(it)) => Some(it),
			Origin::HirNode(NodeRef::Entity(entity)) => Some(entity.item(db)),
			_ => None,
		}
	}

	/// Reorder the top-level items to match the order their originating items
	/// appear in the HIR.
	///
	/// Object lowering emits items grouped by class rather than in source
	/// order; this restores a stable, source-ordered item list. Items without
	/// a HIR origin, and local items, keep their relative order at the end.
	pub(crate) fn reorder_top_level_items_by_hir_order(
		&mut self,
		db: &'db dyn Db,
		item_order: &FxHashMap<shackle_hir::Item<'db>, usize>,
	) {
		let mut keyed_items = self
			.items
			.iter()
			.copied()
			.enumerate()
			.map(|(original_index, item)| {
				let key = if self.is_top_level_item_id(item) {
					(
						0_usize,
						self.origin_hir_item(db, item)
							.and_then(|origin_item| item_order.get(&origin_item).copied())
							.unwrap_or(usize::MAX),
						original_index,
					)
				} else {
					(1_usize, usize::MAX, original_index)
				};
				(key, item)
			})
			.collect::<Vec<_>>();
		keyed_items.sort_by_key(|(key, _)| *key);
		self.items = keyed_items.into_iter().map(|(_, item)| item).collect();
	}

	/// Get the top-level annotation items
	pub fn annotations(
		&self,
	) -> impl Iterator<Item = (AnnotationId<'db, T>, &AnnotationItem<'db, T>)> {
		self.annotations.iter()
	}

	/// Get the top-level annotation items
	pub fn annotations_mut(
		&mut self,
	) -> impl Iterator<Item = (AnnotationId<'db, T>, &mut AnnotationItem<'db, T>)> {
		self.annotations.iter_mut()
	}

	/// Add an annotation item
	pub fn add_annotation(&mut self, item: AnnotationItem<'db, T>) -> AnnotationId<'db, T> {
		let idx = self.annotations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of annotation items
	pub fn annotations_len(&self) -> u32 {
		self.annotations.len()
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints(
		&self,
	) -> impl Iterator<Item = (ConstraintId<'db, T>, &ConstraintItem<'db, T>)> {
		self.all_constraints().filter(|(_, c)| c.top_level())
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId<'db, T>, &mut ConstraintItem<'db, T>)> {
		self.all_constraints_mut().filter(|(_, c)| c.top_level())
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints(
		&self,
	) -> impl Iterator<Item = (ConstraintId<'db, T>, &ConstraintItem<'db, T>)> {
		self.constraints.iter()
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId<'db, T>, &mut ConstraintItem<'db, T>)> {
		self.constraints.iter_mut()
	}

	/// Add a constraint item
	pub fn add_constraint(&mut self, item: ConstraintItem<'db, T>) -> ConstraintId<'db, T> {
		let idx = self.constraints.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of constraint items (including non-toplevel)
	pub fn constraints_len(&self) -> u32 {
		self.constraints.len()
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations(
		&self,
	) -> impl Iterator<Item = (DeclarationId<'db, T>, &DeclarationItem<'db, T>)> {
		self.all_declarations().filter(|(_, d)| d.top_level())
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId<'db, T>, &mut DeclarationItem<'db, T>)> {
		self.all_declarations_mut().filter(|(_, d)| d.top_level())
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations(
		&self,
	) -> impl Iterator<Item = (DeclarationId<'db, T>, &DeclarationItem<'db, T>)> {
		self.declarations.iter()
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId<'db, T>, &mut DeclarationItem<'db, T>)> {
		self.declarations.iter_mut()
	}

	/// Add a declaration item
	pub fn add_declaration(&mut self, item: DeclarationItem<'db, T>) -> DeclarationId<'db, T> {
		let idx = self.declarations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of declaration items (including non-toplevel)
	pub fn declarations_len(&self) -> u32 {
		self.declarations.len()
	}

	/// Get the enumeration items
	pub fn enumerations(
		&self,
	) -> impl Iterator<Item = (EnumerationId<'db, T>, &EnumerationItem<'db, T>)> {
		self.enumerations.iter()
	}

	/// Get the enumeration items
	pub fn enumerations_mut(
		&mut self,
	) -> impl Iterator<Item = (EnumerationId<'db, T>, &mut EnumerationItem<'db, T>)> {
		self.enumerations.iter_mut()
	}

	/// Add an enumeration item
	pub fn add_enumeration(&mut self, item: EnumerationItem<'db, T>) -> EnumerationId<'db, T> {
		let idx = self.enumerations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of enumeration items
	pub fn enumerations_len(&self) -> u32 {
		self.enumerations.len()
	}

	/// Get the function items
	pub fn all_functions(
		&self,
	) -> impl Iterator<Item = (FunctionId<'db, T>, &FunctionItem<'db, T>)> {
		self.functions.iter()
	}

	/// Get the function items
	pub fn all_functions_mut(
		&mut self,
	) -> impl Iterator<Item = (FunctionId<'db, T>, &mut FunctionItem<'db, T>)> {
		self.functions.iter_mut()
	}

	/// Add a function item
	pub fn add_function(&mut self, item: FunctionItem<'db, T>) -> FunctionId<'db, T> {
		let idx = self.functions.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Add a function item after the given item
	pub fn add_function_after(
		&mut self,
		item: FunctionItem<'db, T>,
		after: ItemId<'db, T>,
	) -> FunctionId<'db, T> {
		let idx = self.functions.insert(item);
		self.items.insert(
			self.items.iter().position(|it| *it == after).unwrap() + 1,
			idx.into(),
		);
		idx
	}

	/// Add a function item at the start of the model
	pub fn prepend_function(&mut self, item: FunctionItem<'db, T>) -> FunctionId<'db, T> {
		let idx = self.functions.insert(item);
		self.items.insert(0, idx.into());
		idx
	}

	/// Get the top-level function items
	pub fn top_level_functions(
		&self,
	) -> impl Iterator<Item = (FunctionId<'db, T>, &FunctionItem<'db, T>)> {
		self.all_functions().filter(|(_, f)| f.top_level())
	}

	/// Get the top-level function items
	pub fn top_level_functions_mut(
		&mut self,
	) -> impl Iterator<Item = (FunctionId<'db, T>, &mut FunctionItem<'db, T>)> {
		self.all_functions_mut().filter(|(_, f)| f.top_level())
	}

	/// Number of function items
	pub fn functions_len(&self) -> u32 {
		self.functions.len()
	}

	/// Get the output items
	pub fn outputs(&self) -> impl Iterator<Item = (OutputId<'db, T>, &OutputItem<'db, T>)> {
		self.outputs.iter()
	}

	/// Get the output item
	pub fn output_mut(
		&mut self,
	) -> impl Iterator<Item = (OutputId<'db, T>, &mut OutputItem<'db, T>)> {
		self.outputs.iter_mut()
	}

	/// Add an output item
	pub fn add_output(&mut self, item: OutputItem<'db, T>) -> OutputId<'db, T> {
		let idx = self.outputs.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of output items
	pub fn outputs_len(&self) -> u32 {
		self.outputs.len()
	}

	/// Remove the output items and return them
	pub fn take_outputs(&mut self) -> Vec<OutputItem<'db, T>> {
		let outputs = std::mem::take(&mut self.outputs);
		self.items.retain(|it| !matches!(it, ItemId::Output(_)));
		outputs.into_vec()
	}

	/// Get the solve item
	pub fn solve(&self) -> Option<&SolveItem<'db, T>> {
		self.solve.as_ref()
	}

	/// Get the solve item
	pub fn solve_mut(&mut self) -> Option<&mut SolveItem<'db, T>> {
		self.solve.as_mut()
	}

	/// Set the solve item
	pub fn set_solve(&mut self, solve: SolveItem<'db, T>) -> ItemId<'db, T> {
		self.solve = Some(solve);
		ItemId::Solve
	}

	/// Produce a map for looking up function calls
	pub fn overload_map(&self) -> OverloadMap<'_, 'db, T> {
		let mut overloads: FxHashMap<_, Vec<_>> = FxHashMap::default();
		for (idx, function) in self.top_level_functions() {
			overloads.entry(function.name()).or_default().push(idx);
		}
		OverloadMap {
			model: self,
			overloads,
		}
	}

	/// Lookup a function by its signature
	///
	/// Prefer using `LookupCall` to create a call expression.
	/// If looking up many functions, consider producing an [`OverloadMap`].
	pub fn lookup_function(
		&self,
		db: &'db dyn Db,
		name: FunctionName<'db>,
		args: &[Ty<'db>],
	) -> Result<FunctionLookup<'db, T>, FunctionLookupError<'db, T>> {
		let (specialised, overloads) = self
			.top_level_functions()
			.filter_map(|(i, f)| {
				if f.name() == name {
					Some((i, f.function_entry(self)))
				} else {
					None
				}
			})
			.partition::<Vec<_>, _>(|(f, _)| self[*f].specialised_from().is_some());

		let res = match_fn(db, overloads, args)?;
		let (function, fn_entry) = res.function;

		if fn_entry.is_polymorphic() {
			let overload = OverloadedFunction::Function(fn_entry.instantiate(db, &res.ty_params));
			let concrete = specialised.into_iter().find(|(_, fe)| *fe == overload);
			if let Some((function, fn_entry)) = concrete {
				return Ok(FunctionLookup {
					function,
					fn_entry,
					ty_vars: TyParamInstantiations::default(),
				});
			}
		}

		Ok(FunctionLookup {
			function,
			fn_entry,
			ty_vars: res.ty_params,
		})
	}

	/// Lookup a top-level top-level variable or atom
	///
	/// Prefer using `LookupIdentifier` to create an identifier expression.
	pub fn lookup_identifier(
		&self,
		_db: &'db dyn Db,
		name: Identifier,
	) -> Option<ResolvedIdentifier<'db, T>> {
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
					if e.enum_type().name() == name.0 {
						return Some(ResolvedIdentifier::Enumeration(idx));
					}
					if let Some(cs) = e.definition() {
						for (j, c) in cs.iter().enumerate() {
							if let Some(n) = c.name
								&& n == name
							{
								return Some(ResolvedIdentifier::EnumerationMember(
									EnumMemberId::new(idx, j as u32),
								));
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

impl<'db, T: Marker> Index<EnumMemberId<'db, T>> for Model<'db, T> {
	type Output = Constructor<'db, T>;

	fn index(&self, index: EnumMemberId<'db, T>) -> &Self::Output {
		&self.enumerations[index.enumeration_id()]
			.definition()
			.expect("No definition for enum")[index.member_index() as usize]
	}
}

/// Map which is built once to perform multiple function lookups.
#[derive(Debug)]
pub struct OverloadMap<'a, 'db, T: Marker = ()> {
	model: &'a Model<'db, T>,
	overloads: FxHashMap<FunctionName<'db>, Vec<FunctionId<'db, T>>>,
}

impl<'a, 'db, T: Marker> OverloadMap<'a, 'db, T> {
	/// Filter the overloads in this map
	pub fn filter(&mut self, mut p: impl FnMut(&FunctionItem<'db, T>) -> bool) {
		for overloads in self.overloads.values_mut() {
			overloads.retain(|f| p(&self.model[*f]));
		}
	}

	/// Lookup a function
	pub fn lookup_function(
		&self,
		db: &'db dyn Db,
		name: FunctionName<'db>,
		args: &[Ty<'db>],
	) -> Result<FunctionLookup<'db, T>, FunctionLookupError<'db, T>> {
		let (specialised, overloads) = self
			.overloads
			.get(&name)
			.ok_or_else(|| FunctionLookupError::NoMatchingFunction(Vec::new()))?
			.iter()
			.map(|f| (*f, self.model[*f].function_entry(self.model)))
			.partition::<Vec<_>, _>(|(f, _)| self.model[*f].specialised_from().is_some());

		let res = match_fn(db, overloads, args)?;
		let (function, fn_entry) = res.function;

		if fn_entry.is_polymorphic() {
			let overload = OverloadedFunction::Function(fn_entry.instantiate(db, &res.ty_params));
			let concrete = specialised.into_iter().find(|(_, fe)| *fe == overload);
			if let Some((function, fn_entry)) = concrete {
				return Ok(FunctionLookup {
					function,
					fn_entry,
					ty_vars: TyParamInstantiations::default(),
				});
			}
		}

		Ok(FunctionLookup {
			function,
			fn_entry,
			ty_vars: res.ty_params,
		})
	}
}

impl<'a, 'db, T: Marker> Deref for OverloadMap<'a, 'db, T> {
	type Target = FxHashMap<FunctionName<'db>, Vec<FunctionId<'db, T>>>;

	fn deref(&self) -> &Self::Target {
		&self.overloads
	}
}

/// Result of looking up a function by its signature
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionLookup<'db, T: Marker> {
	/// Id of the resolved function
	pub function: FunctionId<'db, T>,
	/// The function entry (i.e. not instantiated with the call arguments)
	pub fn_entry: OverloadedFunction<'db>,
	/// The instantiated types of the type inst vars (if any)
	pub ty_vars: TyParamInstantiations<'db>,
}

/// Error representing failure to lookup a function
pub type FunctionLookupError<'db, T> =
	FunctionResolutionError<'db, (FunctionId<'db, T>, OverloadedFunction<'db>)>;

/// Trait for THIR marker
///
/// Used as a type parameter for THIR nodes, allowing us to have greater
/// type safety when dealing with multiple THIR models by using different
/// type parameters for each, so that the IDs from one model can't be used
/// to access another.
pub trait Marker:
	Copy
	+ Clone
	+ PartialEq
	+ Eq
	+ PartialOrd
	+ Ord
	+ std::hash::Hash
	+ std::fmt::Debug
	+ Default
	+ salsa::Update
	+ 'static
{
}

impl Marker for () {}
