//! THIR representation
//!

use std::ops::{Deref, Index};

use rustc_hash::FxHashMap;

use crate::{
	hir::Identifier,
	ty::{FunctionEntry, FunctionResolutionError, OverloadedFunction, Ty, TyParamInstantiations},
	utils::{arena::Arena, impl_index},
};

mod annotations;
mod domain;
mod expression;
mod item;
pub mod traverse;

pub use self::{annotations::*, domain::*, expression::*, item::*};
use super::db::Thir;

/// Entity counts
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityCounts {
	/// Number of annotation items
	pub annotations: u32,
	/// Number of constraint items
	pub constraints: u32,
	/// Number of declaration items
	pub declarations: u32,
	/// Number of enumeration items
	pub enumerations: u32,
	/// Number of function items
	pub functions: u32,
	/// Number of output items
	pub outputs: u32,
}

impl EntityCounts {
	/// Total number of items (will not include solve item)
	pub fn items(&self) -> u32 {
		self.annotations
			+ self.constraints
			+ self.declarations
			+ self.enumerations
			+ self.functions
			+ self.outputs
	}
}

impl From<crate::hir::db::EntityCounts> for EntityCounts {
	fn from(value: crate::hir::db::EntityCounts) -> Self {
		Self {
			annotations: value.annotations,
			constraints: value.constraints,
			declarations: value.declarations,
			enumerations: value.enumerations,
			functions: value.functions,
			outputs: value.outputs,
		}
	}
}

/// A model
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Model<T: Marker = ()> {
	annotations: Arena<AnnotationItem<T>>,
	constraints: Arena<ConstraintItem<T>>,
	declarations: Arena<DeclarationItem<T>>,
	enumerations: Arena<EnumerationItem<T>>,
	functions: Arena<FunctionItem<T>>,
	outputs: Arena<OutputItem<T>>,
	solve: Option<SolveItem<T>>,
	items: Vec<ItemId<T>>,
}

impl<T: Marker> Model<T> {
	/// Create a model able to store the given numbers entities without reallocating
	pub fn with_capacities(capacities: &EntityCounts) -> Self {
		Self {
			annotations: Arena::with_capacity(capacities.annotations),
			constraints: Arena::with_capacity(capacities.constraints),
			declarations: Arena::with_capacity(capacities.declarations),
			enumerations: Arena::with_capacity(capacities.enumerations),
			functions: Arena::with_capacity(capacities.functions),
			outputs: Arena::with_capacity(capacities.outputs),
			items: Vec::with_capacity(capacities.items() as usize),
			..Default::default()
		}
	}

	/// Get the entity counts
	pub fn entity_counts(&self) -> EntityCounts {
		EntityCounts {
			annotations: self.annotations_len(),
			constraints: self.constraints_len(),
			declarations: self.declarations_len(),
			enumerations: self.enumerations_len(),
			functions: self.functions_len(),
			outputs: self.outputs_len(),
		}
	}

	/// Get the top-level items
	pub fn top_level_items(&self) -> impl '_ + Iterator<Item = ItemId<T>> {
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
	pub fn all_items(&self) -> impl '_ + Iterator<Item = ItemId<T>> {
		self.items.iter().copied()
	}

	/// Get the top-level annotation items
	pub fn annotations(&self) -> impl Iterator<Item = (AnnotationId<T>, &AnnotationItem<T>)> {
		self.annotations.iter()
	}

	/// Get the top-level annotation items
	pub fn annotations_mut(
		&mut self,
	) -> impl Iterator<Item = (AnnotationId<T>, &mut AnnotationItem<T>)> {
		self.annotations.iter_mut()
	}

	/// Add an annotation item
	pub fn add_annotation(&mut self, item: AnnotationItem<T>) -> AnnotationId<T> {
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
	) -> impl Iterator<Item = (ConstraintId<T>, &ConstraintItem<T>)> {
		self.all_constraints().filter(|(_, c)| c.top_level())
	}

	/// Get the top-level constraint items
	pub fn top_level_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId<T>, &mut ConstraintItem<T>)> {
		self.all_constraints_mut().filter(|(_, c)| c.top_level())
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints(&self) -> impl Iterator<Item = (ConstraintId<T>, &ConstraintItem<T>)> {
		self.constraints.iter()
	}

	/// Get all constraint items (including constraints inside let expressions)
	pub fn all_constraints_mut(
		&mut self,
	) -> impl Iterator<Item = (ConstraintId<T>, &mut ConstraintItem<T>)> {
		self.constraints.iter_mut()
	}

	/// Add a constraint item
	pub fn add_constraint(&mut self, item: ConstraintItem<T>) -> ConstraintId<T> {
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
	) -> impl Iterator<Item = (DeclarationId<T>, &DeclarationItem<T>)> {
		self.all_declarations().filter(|(_, d)| d.top_level())
	}

	/// Get the top-level declaration items
	pub fn top_level_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId<T>, &mut DeclarationItem<T>)> {
		self.all_declarations_mut().filter(|(_, d)| d.top_level())
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations(
		&self,
	) -> impl Iterator<Item = (DeclarationId<T>, &DeclarationItem<T>)> {
		self.declarations.iter()
	}

	/// Get all declaration items (including declarations inside let expressions)
	pub fn all_declarations_mut(
		&mut self,
	) -> impl Iterator<Item = (DeclarationId<T>, &mut DeclarationItem<T>)> {
		self.declarations.iter_mut()
	}

	/// Add a declaration item
	pub fn add_declaration(&mut self, item: DeclarationItem<T>) -> DeclarationId<T> {
		let idx = self.declarations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of declaration items (including non-toplevel)
	pub fn declarations_len(&self) -> u32 {
		self.declarations.len()
	}

	/// Get the enumeration items
	pub fn enumerations(&self) -> impl Iterator<Item = (EnumerationId<T>, &EnumerationItem<T>)> {
		self.enumerations.iter()
	}

	/// Get the enumeration items
	pub fn enumerations_mut(
		&mut self,
	) -> impl Iterator<Item = (EnumerationId<T>, &mut EnumerationItem<T>)> {
		self.enumerations.iter_mut()
	}

	/// Add an enumeration item
	pub fn add_enumeration(&mut self, item: EnumerationItem<T>) -> EnumerationId<T> {
		let idx = self.enumerations.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of enumeration items
	pub fn enumerations_len(&self) -> u32 {
		self.enumerations.len()
	}

	/// Get the function items
	pub fn all_functions(&self) -> impl Iterator<Item = (FunctionId<T>, &FunctionItem<T>)> {
		self.functions.iter()
	}

	/// Get the function items
	pub fn all_functions_mut(
		&mut self,
	) -> impl Iterator<Item = (FunctionId<T>, &mut FunctionItem<T>)> {
		self.functions.iter_mut()
	}

	/// Add a function item
	pub fn add_function(&mut self, item: FunctionItem<T>) -> FunctionId<T> {
		let idx = self.functions.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Add a function item after the given item
	pub fn add_function_after(&mut self, item: FunctionItem<T>, after: ItemId<T>) -> FunctionId<T> {
		let idx: crate::utils::arena::ArenaIndex<Item<Function<T>>> = self.functions.insert(item);
		self.items.insert(
			self.items.iter().position(|it| *it == after).unwrap() + 1,
			idx.into(),
		);
		idx
	}

	/// Add a function item at the start of the model
	pub fn prepend_function(&mut self, item: FunctionItem<T>) -> FunctionId<T> {
		let idx: crate::utils::arena::ArenaIndex<Item<Function<T>>> = self.functions.insert(item);
		self.items.insert(0, idx.into());
		idx
	}

	/// Get the top-level function items
	pub fn top_level_functions(&self) -> impl Iterator<Item = (FunctionId<T>, &FunctionItem<T>)> {
		self.all_functions().filter(|(_, f)| f.top_level())
	}

	/// Get the top-level function items
	pub fn top_level_functions_mut(
		&mut self,
	) -> impl Iterator<Item = (FunctionId<T>, &mut FunctionItem<T>)> {
		self.all_functions_mut().filter(|(_, f)| f.top_level())
	}

	/// Number of function items
	pub fn functions_len(&self) -> u32 {
		self.functions.len()
	}

	/// Get the output items
	pub fn outputs(&self) -> impl Iterator<Item = (OutputId<T>, &OutputItem<T>)> {
		self.outputs.iter()
	}

	/// Get the output item
	pub fn output_mut(&mut self) -> impl Iterator<Item = (OutputId<T>, &mut OutputItem<T>)> {
		self.outputs.iter_mut()
	}

	/// Add an output item
	pub fn add_output(&mut self, item: OutputItem<T>) -> OutputId<T> {
		let idx = self.outputs.insert(item);
		self.items.push(idx.into());
		idx
	}

	/// Number of output items
	pub fn outputs_len(&self) -> u32 {
		self.outputs.len()
	}

	/// Remove the output items and return them
	pub fn take_outputs(&mut self) -> Vec<OutputItem<T>> {
		let outputs = std::mem::take(&mut self.outputs);
		self.items.retain(|it| !matches!(it, ItemId::Output(_)));
		outputs.into_vec()
	}

	/// Get the solve item
	pub fn solve(&self) -> Option<&SolveItem<T>> {
		self.solve.as_ref()
	}

	/// Get the solve item
	pub fn solve_mut(&mut self) -> Option<&mut SolveItem<T>> {
		self.solve.as_mut()
	}

	/// Set the solve item
	pub fn set_solve(&mut self, solve: SolveItem<T>) -> ItemId<T> {
		self.solve = Some(solve);
		ItemId::Solve
	}

	/// Produce a map for looking up function calls
	pub fn overload_map(&self) -> OverloadMap<'_, T> {
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
		db: &dyn Thir,
		name: FunctionName,
		args: &[Ty],
	) -> Result<FunctionLookup<T>, FunctionLookupError<T>> {
		let (specialised, overloads) = self
			.top_level_functions()
			.filter_map(|(i, f)| {
				if f.name() == name {
					Some((i, f.function_entry(self)))
				} else {
					None
				}
			})
			.partition::<Vec<_>, _>(|(i, _)| self[*i].specialised_from().is_some());

		let (function, fn_entry, ty_vars) = FunctionEntry::match_fn(db.upcast(), overloads, args)?;

		if fn_entry.overload.is_polymorphic() {
			let overload =
				OverloadedFunction::Function(fn_entry.overload.instantiate(db.upcast(), &ty_vars));
			let concrete = specialised
				.into_iter()
				.find(|(_, fe)| fe.overload == overload);
			if let Some((f, fe)) = concrete {
				return Ok(FunctionLookup {
					function: f,
					fn_entry: fe,
					ty_vars: TyParamInstantiations::default(),
				});
			}
		}

		Ok(FunctionLookup {
			function,
			fn_entry,
			ty_vars,
		})
	}

	/// Lookup a top-level top-level variable or atom
	///
	/// Prefer using `LookupIdentifier` to create an identifier expression.
	pub fn lookup_identifier(
		&self,
		db: &dyn Thir,
		name: Identifier,
	) -> Option<ResolvedIdentifier<T>> {
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

impl_index!(Model<T: Marker>[self, index: AnnotationId<T>] -> AnnotationItem<T> { self.annotations[index] });
impl_index!(Model<T: Marker>[self, index: ConstraintId<T>] -> ConstraintItem<T> { self.constraints[index] });
impl_index!(Model<T: Marker>[self, index: DeclarationId<T>] -> DeclarationItem<T> { self.declarations[index] });
impl_index!(Model<T: Marker>[self, index: EnumerationId<T>] -> EnumerationItem<T> { self.enumerations[index] });
impl_index!(Model<T: Marker>[self, index: FunctionId<T>] -> FunctionItem<T> { self.functions[index] });
impl_index!(Model<T: Marker>[self, index: OutputId<T>] -> OutputItem<T> { self.outputs[index] });

impl<T: Marker> Index<EnumMemberId<T>> for Model<T> {
	type Output = Constructor<T>;
	fn index(&self, index: EnumMemberId<T>) -> &Self::Output {
		&self.enumerations[index.enumeration_id()]
			.definition()
			.expect("No definition for enum")[index.member_index() as usize]
	}
}

/// Map which is built once to perform multiple function lookups.
pub struct OverloadMap<'a, T: Marker = ()> {
	model: &'a Model<T>,
	overloads: FxHashMap<FunctionName, Vec<FunctionId<T>>>,
}

impl<'a, T: Marker> OverloadMap<'a, T> {
	/// Filter the overloads in this map
	pub fn filter(&mut self, mut p: impl FnMut(&FunctionItem<T>) -> bool) {
		for overloads in self.overloads.values_mut() {
			overloads.retain(|f| p(&self.model[*f]));
		}
	}

	/// Lookup a function
	pub fn lookup_function(
		&self,
		db: &dyn Thir,
		name: FunctionName,
		args: &[Ty],
	) -> Result<FunctionLookup<T>, FunctionLookupError<T>> {
		let (specialised, overloads) = self
			.overloads
			.get(&name)
			.ok_or_else(|| FunctionLookupError::NoMatchingFunction(Vec::new()))?
			.iter()
			.map(|f| (*f, self.model[*f].function_entry(self.model)))
			.partition::<Vec<_>, _>(|(i, _)| self.model[*i].specialised_from().is_some());

		let (function, fn_entry, ty_vars) = FunctionEntry::match_fn(db.upcast(), overloads, args)?;

		if fn_entry.overload.is_polymorphic() {
			let overload =
				OverloadedFunction::Function(fn_entry.overload.instantiate(db.upcast(), &ty_vars));
			let concrete = specialised
				.into_iter()
				.find(|(_, fe)| fe.overload == overload);
			if let Some((f, fe)) = concrete {
				return Ok(FunctionLookup {
					function: f,
					fn_entry: fe,
					ty_vars: TyParamInstantiations::default(),
				});
			}
		}

		Ok(FunctionLookup {
			function,
			fn_entry,
			ty_vars,
		})
	}
}

impl<'a, T: Marker> Deref for OverloadMap<'a, T> {
	type Target = FxHashMap<FunctionName, Vec<FunctionId<T>>>;
	fn deref(&self) -> &Self::Target {
		&self.overloads
	}
}

/// Result of looking up a function by its signature
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionLookup<T: Marker> {
	/// Id of the resolved function
	pub function: FunctionId<T>,
	/// The function entry (i.e. not instantiated with the call arguments)
	pub fn_entry: FunctionEntry,
	/// The instantiated types of the type inst vars (if any)
	pub ty_vars: TyParamInstantiations,
}

/// Error representing failure to lookup a function
pub type FunctionLookupError<T> = FunctionResolutionError<FunctionId<T>>;

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
	+ std::default::Default
{
}

impl Marker for () {}
