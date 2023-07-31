//! THIR representation
//!

use std::ops::Index;

use crate::arena::Arena;
use crate::hir::Identifier;
use crate::ty::FunctionEntry;
use crate::ty::FunctionResolutionError;
use crate::ty::Ty;
use crate::ty::TyParamInstantiations;
use crate::utils::impl_index;

mod annotations;
mod domain;
mod expression;
mod item;
pub mod traverse;

pub use self::annotations::*;
pub use self::domain::*;
pub use self::expression::*;
pub use self::item::*;

use super::db::Thir;

/// A model
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Model<T = ()> {
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

	/// Lookup a function by its signature
	///
	/// Prefer using `ExpressionAllocator::lookup_call` to create an call expression.
	pub fn lookup_function(
		&self,
		db: &dyn Thir,
		name: FunctionName,
		args: &[Ty],
	) -> Result<FunctionLookup<T>, FunctionLookupError<T>> {
		let overloads = self.top_level_functions().filter_map(|(i, f)| {
			if f.name() == name {
				Some((i, f.function_entry(self)))
			} else {
				None
			}
		});
		let (function, fn_entry, ty_vars) = FunctionEntry::match_fn(db.upcast(), overloads, args)?;
		Ok(FunctionLookup {
			function,
			fn_entry,
			ty_vars,
		})
	}

	/// Lookup a top-level top-level variable or atom
	///
	/// Prefer using `ExpressionAllocator::lookup_identifier` to create an identifier expression.
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

impl_index!(Model<T>[self, index: AnnotationId<T>] -> AnnotationItem<T> { self.annotations[index] });
impl_index!(Model<T>[self, index: ConstraintId<T>] -> ConstraintItem<T> { self.constraints[index] });
impl_index!(Model<T>[self, index: DeclarationId<T>] -> DeclarationItem<T> { self.declarations[index] });
impl_index!(Model<T>[self, index: EnumerationId<T>] -> EnumerationItem<T> { self.enumerations[index] });
impl_index!(Model<T>[self, index: FunctionId<T>] -> FunctionItem<T> { self.functions[index] });
impl_index!(Model<T>[self, index: OutputId<T>] -> OutputItem<T> { self.outputs[index] });

impl<T: Marker> Index<EnumMemberId<T>> for Model<T> {
	type Output = Constructor<T>;
	fn index(&self, index: EnumMemberId<T>) -> &Self::Output {
		&self.enumerations[index.enumeration_id()]
			.definition()
			.expect("No definition for enum")[index.member_index() as usize]
	}
}

/// Result of looking up a function by its signature
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionLookup<T> {
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
	Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash + std::fmt::Debug
{
}

impl Marker for () {}
