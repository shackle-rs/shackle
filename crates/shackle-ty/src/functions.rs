/// Function overloading and instantiation
use rustc_hash::FxHashMap;
use shackle_utils::{InternedString, maybe_grow_stack};

use super::{OptType, Ty, TyData, TyVarRef, VarType};
use crate::{Db, registry::TypeRegistry};

/// Represents failure to resolve overloading
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionResolutionError<'db, T> {
	/// No matching function
	NoMatchingFunction(Vec<(T, InstantiationError<'db>)>),
	/// Ambiguous call
	AmbiguousOverloading(Vec<T>),
}

impl<'db, T: Overload<'db>> FunctionResolutionError<'db, T> {
	/// Get the pretty error message
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		match self {
			Self::NoMatchingFunction(fs) => ["No matching function:".to_owned()]
				.into_iter()
				.chain(fs.iter().map(|(f, e)| {
					format!(
						"  {}: {}",
						f.overload().pretty_print(db),
						e.pretty_print(db)
					)
				}))
				.collect::<Vec<_>>()
				.join("\n"),
			Self::AmbiguousOverloading(fs) => ["Ambiguous overloading".to_owned()]
				.into_iter()
				.chain(
					fs.iter()
						.map(|f| format!("  {}", f.overload().pretty_print(db))),
				)
				.collect::<Vec<_>>()
				.join("\n"),
		}
	}
}

/// Represent failure to instantiate a function
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstantiationError<'db> {
	/// Attempted to instantiate a type-inst var with two incompatible types.
	IncompatibleTypeInstVariable {
		/// The type-inst variable
		ty_var: TyVarRef<'db>,
		/// The types which the variable was instantiated with
		types: Vec<Ty<'db>>,
	},
	/// Mismatch in type of argument
	ArgumentMismatch {
		/// The argument index
		index: usize,
		/// Expected Type
		expected: Ty<'db>,
		/// Actual type
		actual: Ty<'db>,
	},
	/// Mismatch in number of arguments
	ArgumentCountMismatch {
		/// Expected number of arguments
		expected: usize,
		/// Actual number of arguments
		actual: usize,
	},
}

impl<'db> InstantiationError<'db> {
	/// Get the pretty error message
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		match self {
			Self::IncompatibleTypeInstVariable { ty_var, types } => {
				format!(
					"type-inst var {} instantiated with incompatible types [{}]",
					ty_var.pretty_print(db),
					types
						.iter()
						.map(|ty| ty.pretty_print(db))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			Self::ArgumentMismatch {
				index,
				expected,
				actual,
			} => {
				format!(
					"argument {} expected {} but got {}",
					*index + 1,
					expected.pretty_print(db),
					actual.pretty_print(db)
				)
			}
			Self::ArgumentCountMismatch { expected, actual } => {
				format!("expected {} arguments but got {}", *expected, *actual)
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate<'db, T> {
	is_candidate: bool,
	has_error: bool,
	entry: T,
	ty_params: TyParamInstantiations<'db>,
	function_type: FunctionType<'db>,
}

/// An overloaded function
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub enum OverloadedFunction<'db> {
	/// A non-generic function
	Function(FunctionType<'db>),
	/// A generic function
	PolymorphicFunction(PolymorphicFunctionType<'db>),
}

impl<'db> OverloadedFunction<'db> {
	/// Get the inner non-polymorphic function
	pub fn into_function(self) -> Option<FunctionType<'db>> {
		match self {
			OverloadedFunction::Function(f) => Some(f),
			OverloadedFunction::PolymorphicFunction(_) => None,
		}
	}

	/// Get the return type of the function
	pub fn return_type(&self) -> Ty<'db> {
		match self {
			OverloadedFunction::Function(f) => f.return_type,
			OverloadedFunction::PolymorphicFunction(p) => p.return_type,
		}
	}

	/// Get the parameters of the function
	pub fn params(&self) -> &[Ty<'db>] {
		match self {
			OverloadedFunction::Function(f) => &f.params,
			OverloadedFunction::PolymorphicFunction(p) => &p.params,
		}
	}

	/// Get mutable reference to the parameters of the function
	pub fn params_mut(&mut self) -> &mut [Ty<'db>] {
		match self {
			OverloadedFunction::Function(f) => &mut f.params,
			OverloadedFunction::PolymorphicFunction(p) => &mut p.params,
		}
	}

	/// Set the parameters of the function
	pub fn set_params(&mut self, params: Box<[Ty<'db>]>) {
		match self {
			OverloadedFunction::Function(f) => f.params = params,
			OverloadedFunction::PolymorphicFunction(p) => p.params = params,
		}
	}

	/// Whether this function is polymorphic
	pub fn is_polymorphic(&self) -> bool {
		match self {
			OverloadedFunction::Function(_) => false,
			OverloadedFunction::PolymorphicFunction(_) => true,
		}
	}

	/// Return whether this function contains an error type
	pub fn contains_error(&self, db: &'db dyn Db) -> bool {
		match self {
			OverloadedFunction::Function(f) => f.contains_error(db),
			OverloadedFunction::PolymorphicFunction(p) => p.contains_error(db),
		}
	}

	/// Instantiate this function's type parameters with the given argument types
	pub fn instantiate_ty_params(
		&self,
		db: &'db dyn Db,
		args: &[Ty<'db>],
	) -> Result<(TyParamInstantiations<'db>, FunctionType<'db>), InstantiationError<'db>> {
		match self {
			OverloadedFunction::Function(f) => {
				f.matches(db, args)?;
				Ok((TyParamInstantiations::default(), f.clone()))
			}
			OverloadedFunction::PolymorphicFunction(p) => p.instantiate_ty_params(db, args),
		}
	}

	/// Instantiate this function using the given type parameter types
	pub fn instantiate(
		&self,
		db: &'db dyn Db,
		instantiations: &TyParamInstantiations<'db>,
	) -> FunctionType<'db> {
		match self {
			OverloadedFunction::Function(f) => f.clone(),
			OverloadedFunction::PolymorphicFunction(p) => p.instantiate(db, instantiations),
		}
	}

	/// Get human readable representation of this signature
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print(db),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print(db),
		}
	}

	/// Get human readable representation of this signature in item form
	pub fn pretty_print_item(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print_item(db, name),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print_item(db, name),
		}
	}

	/// Get human readable representation of this signature in item form without the return type
	pub fn pretty_print_call_signature(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print_call_signature(db, name),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print_call_signature(db, name),
		}
	}
}

/// Trait for overloaded functions
pub trait Overload<'db> {
	/// Get the overload
	fn overload(&self) -> &OverloadedFunction<'db>;

	/// Called when two overloads are identical, to determine which one to prefer.
	///
	/// Return `Some(true)` to prefer `self`, `Some(false)` to prefer `other`, or `None` to indicate that neither is preferred.
	fn tie_break(&self, _other: &Self) -> Option<bool> {
		None
	}
}

impl<'db> Overload<'db> for OverloadedFunction<'db> {
	fn overload(&self) -> &OverloadedFunction<'db> {
		self
	}
}

impl<'db, T, U: Overload<'db>> Overload<'db> for (T, U) {
	fn overload(&self) -> &OverloadedFunction<'db> {
		self.1.overload()
	}

	fn tie_break(&self, other: &Self) -> Option<bool> {
		self.1.tie_break(&other.1)
	}
}

/// A matched function
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct ResolvedFunction<'db, T> {
	/// The function
	pub function: T,
	/// The type parameter instantiations for the function
	pub ty_params: TyParamInstantiations<'db>,
}

/// Return the most specific function overload which matches the given argument types.
///
/// If the function to dispatch to is polymorphic then also instantiate the polymorphic function.
/// If there is no one specific function, this is an error.
pub fn match_fn<'db, T: Overload<'db>>(
	db: &'db dyn Db,
	overloads: impl IntoIterator<Item = T>,
	args: &[Ty<'db>],
) -> Result<ResolvedFunction<'db, T>, FunctionResolutionError<'db, T>> {
	let (matches, mismatches) = overloads
		.into_iter()
		.map(|entry| {
			let ty_params = entry.overload().instantiate_ty_params(db, args);
			(entry, ty_params)
		})
		.partition::<Vec<_>, _>(|(_, ty_params)| ty_params.is_ok());

	if matches.is_empty() {
		return Err(FunctionResolutionError::NoMatchingFunction(
			mismatches
				.into_iter()
				.map(|(entry, ty_params)| (entry, ty_params.unwrap_err()))
				.collect(),
		));
	}

	let mut candidates = matches
		.into_iter()
		.map(|(entry, instantiation)| {
			let (ty_params, function_type) = instantiation.unwrap();
			Candidate {
				is_candidate: true,
				has_error: entry.overload().contains_error(db),
				entry,
				ty_params,
				function_type,
			}
		})
		.collect::<Vec<_>>();

	log::debug!(
		"Overload resolution found {} matching candidates",
		candidates.len()
	);
	for i in 1..candidates.len() {
		// For each pair, eliminate the less specific function (based on instantiated signature if there were candidate polymorphic functions)
		// e.g. prefer 'bool' over 'int', prefer 'int' over 'var int'
		//      for an 'int' argument, prefer '$T' over 'float' (prefer the instantiated polymorphic function over the concrete function which requires a coercion)
		//      prefer concrete function over polymorphic instantiation if equivalent
		//      for two polymorphic candidates, prefer '$$E' over '$T' if they both instantiate to the same type
		let (left, right) = candidates.split_at_mut(i);
		let c1 = left.last_mut().unwrap();
		if !c1.is_candidate {
			continue;
		}
		for (j, c2) in right.iter_mut().enumerate() {
			if !c2.is_candidate {
				continue;
			}
			if c1.has_error && !c2.has_error {
				c1.is_candidate = false;
				continue;
			} else if c2.has_error && !c1.has_error {
				c2.is_candidate = false;
				continue;
			}
			let f1 = &c1.function_type;
			let f2 = &c2.function_type;
			let m1 = f1.matches(db, &f2.params).is_ok();
			let m2 = f2.matches(db, &f1.params).is_ok();

			log::debug!(
				"Candidate {}: {} instantiates to {}",
				i,
				c1.entry.overload().pretty_print(db),
				f1.pretty_print(db)
			);
			log::debug!(
				"Candidate {}: {} instantiates to {}",
				i + j + 1,
				c2.entry.overload().pretty_print(db),
				f2.pretty_print(db)
			);
			log::debug!("Candidate {} accepts candidate 2's parameters? {:?}", i, m1);
			log::debug!(
				"Candidate {} accepts candidate 1's parameters? {:?}",
				i + j + 1,
				m2
			);
			if m1 && !m2 {
				// We accept their args, but they don't accept ours, so they're more specific
				c1.is_candidate = false;
			} else if m2 && !m1 {
				// They accept our args, but we don't accept theirs, so we're more specific
				c2.is_candidate = false;
			} else if m1 && m2 {
				// Equivalent instantiation
				match (&c1.entry.overload(), &c2.entry.overload()) {
					// Prefer concrete function over polymorphic instance
					(
						OverloadedFunction::PolymorphicFunction(_),
						OverloadedFunction::Function(_),
					) => {
						c1.is_candidate = false;
					}
					(
						OverloadedFunction::Function(_),
						OverloadedFunction::PolymorphicFunction(_),
					) => {
						c2.is_candidate = false;
					}
					// Prefer more specific polymorphic function
					(
						OverloadedFunction::PolymorphicFunction(p1),
						OverloadedFunction::PolymorphicFunction(p2),
					) => {
						let m1 = p1.instantiate_ty_params(db, &p2.params).is_ok();
						let m2 = p2.instantiate_ty_params(db, &p1.params).is_ok();
						log::debug!(
							"Polymorphic candidate {} accepts candidate 2's polymorphic parameters? {:?}",
							i,
							m1
						);
						log::debug!(
							"Polymorphic candidate {} accepts candidate 1's polymorphic parameters? {:?}",
							i + j + 1,
							m2
						);
						if m1 && !m2 {
							// We accept their args, but they don't accept ours, so they're more specific
							c1.is_candidate = false;
						} else if m2 && !m1 {
							// They accept our args, but we don't accept theirs, so we're more specific
							c2.is_candidate = false;
						} else {
							// Prefer earlier candidate if both equivalent
							c2.is_candidate = false;
						}
					}
					_ => {
						if let Some(prefer_c1) = c1.entry.tie_break(&c2.entry) {
							if prefer_c1 {
								c2.is_candidate = false;
							} else {
								c1.is_candidate = false;
							}
						}
					}
				}
			}
			if !c1.is_candidate {
				log::debug!("Eliminated candidate {}", i);
			}
			if !c2.is_candidate {
				log::debug!("Eliminated candidate {}", i + j + 1);
			}
		}
	}
	candidates.retain(|c| c.is_candidate);
	assert!(
		!candidates.is_empty(),
		"Overload matches found, but all candidates eliminated!"
	);
	if candidates.len() > 1 {
		return Err(FunctionResolutionError::AmbiguousOverloading(
			candidates.into_iter().map(|c| c.entry).collect(),
		));
	}
	let c = candidates.pop().unwrap();
	Ok(ResolvedFunction {
		function: c.entry,
		ty_params: c.ty_params,
	})
}

/// Type of a function expression.
///
/// Function expressions can't be generic, so there are no unbound type-inst variables.
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct FunctionType<'db> {
	/// Return type
	pub return_type: Ty<'db>,
	/// Parameter types
	pub params: Box<[Ty<'db>]>,
}

impl<'db> FunctionType<'db> {
	/// Return whether this function is a subtype of another
	pub fn is_subtype_of(&self, db: &'db dyn Db, other: &FunctionType<'db>) -> bool {
		// op(bool: (int, float)) is a subtype of op(int: (bool, int))
		self.return_type.is_subtype_of(db, other.return_type)
			&& self.params.len() == other.params.len()
			&& self
				.params
				.iter()
				.zip(other.params.iter())
				.all(|(a, b)| b.is_subtype_of(db, *a))
	}

	/// Return whether this function contains an error type in its parameters
	pub fn contains_error(&self, db: &'db dyn Db) -> bool {
		self.params.iter().any(|f| f.contains_error(db))
	}

	/// Whether or not the given parameter types are compatible with this function
	pub fn matches(
		&self,
		db: &'db dyn Db,
		args: &[Ty<'db>],
	) -> Result<(), InstantiationError<'db>> {
		if args.len() != self.params.len() {
			return Err(InstantiationError::ArgumentCountMismatch {
				expected: self.params.len(),
				actual: args.len(),
			});
		}
		for (i, (arg, param)) in args.iter().zip(self.params.iter()).enumerate() {
			if !arg.is_subtype_of(db, *param) {
				return Err(InstantiationError::ArgumentMismatch {
					index: i,
					expected: *param,
					actual: *arg,
				});
			}
		}
		Ok(())
	}

	/// Get human readable representation of type
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		format!(
			"op({}: ({}))",
			self.return_type.pretty_print(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}

	/// Get human readable representation of type as an item
	pub fn pretty_print_item(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		let tys = TypeRegistry::new(db);
		let prefix = if self.return_type == tys.par_bool {
			"test".to_owned()
		} else if self.return_type == tys.var_bool {
			"predicate".to_owned()
		} else {
			format!("function {}:", self.return_type.pretty_print(db))
		};
		format!("{} {}", prefix, self.pretty_print_call_signature(db, name))
	}

	/// Get human readable representation of type as an item without the return type
	pub fn pretty_print_call_signature(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		format!(
			"{}({})",
			name.into().lookup(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}

/// Mapping from type parameters to the concrete type used to instantiate them
pub type TyParamInstantiations<'db> = FxHashMap<TyVarRef<'db>, Ty<'db>>;

/// Type of a generic function with type-inst parameters
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct PolymorphicFunctionType<'db> {
	/// Return type
	pub return_type: Ty<'db>,
	/// Type-inst parameters
	pub ty_params: Box<[TyVarRef<'db>]>,
	/// Parameter types
	pub params: Box<[Ty<'db>]>,
}

impl<'db> PolymorphicFunctionType<'db> {
	/// Return whether this function contains an error type in its parameters
	pub fn contains_error(&self, db: &'db dyn Db) -> bool {
		self.params.iter().any(|f| f.contains_error(db))
	}

	/// Instantiates this polymorphic function using the given parameter types if possible.
	pub fn instantiate(
		&self,
		db: &'db dyn Db,
		ty_vars: &TyParamInstantiations<'db>,
	) -> FunctionType<'db> {
		FunctionType {
			return_type: self.return_type.instantiate_ty_vars(db, ty_vars),
			params: self
				.params
				.iter()
				.map(|p| p.instantiate_ty_vars(db, ty_vars))
				.collect(),
		}
	}

	/// Instantiates this polymorphic function using the given parameter types if possible, returning
	/// the type-parameter instantiations.
	pub fn instantiate_ty_params(
		&self,
		db: &'db dyn Db,
		args: &[Ty<'db>],
	) -> Result<(TyParamInstantiations<'db>, FunctionType<'db>), InstantiationError<'db>> {
		if args.len() != self.params.len() {
			return Err(InstantiationError::ArgumentCountMismatch {
				expected: self.params.len(),
				actual: args.len(),
			});
		}
		let mut instantiations = FxHashMap::default();
		for t in self.ty_params.iter() {
			let _ = instantiations.insert(*t, Vec::new());
		}
		for (i, (arg, param)) in args.iter().zip(self.params.iter()).enumerate() {
			if !PolymorphicFunctionType::collect_instantiations(
				db,
				&mut |tv, ty| {
					if let Some(is) = instantiations.get_mut(&tv) {
						is.push(ty);
						true
					} else {
						false
					}
				},
				*arg,
				*param,
			) {
				return Err(InstantiationError::ArgumentMismatch {
					index: i,
					expected: *param,
					actual: *arg,
				});
			}
		}
		let mut resolved = FxHashMap::default();
		for (tv, ts) in instantiations {
			match Ty::most_specific_supertype(db, ts.iter().copied()) {
				Some(t) => {
					let _ = resolved.insert(tv, t);
				}
				None => {
					return Err(InstantiationError::IncompatibleTypeInstVariable {
						ty_var: tv,
						types: ts,
					});
				}
			}
		}
		resolved.shrink_to_fit();
		let ft = self.instantiate(db, &resolved);
		ft.matches(db, args)?;
		Ok((resolved, ft))
	}

	/// Collects the types to instantiate unbound type-inst variables with.
	pub fn collect_instantiations(
		db: &'db dyn Db,
		add_instantiation: &mut impl FnMut(TyVarRef<'db>, Ty<'db>) -> bool,
		arg: Ty<'db>,
		param: Ty<'db>,
	) -> bool {
		maybe_grow_stack(|| Self::collect_instantiations_inner(db, add_instantiation, arg, param))
	}

	fn collect_instantiations_inner(
		db: &'db dyn Db,
		add_instantiation: &mut impl FnMut(TyVarRef<'db>, Ty<'db>) -> bool,
		arg: Ty<'db>,
		param: Ty<'db>,
	) -> bool {
		match (arg.lookup(db), param.lookup(db)) {
			(
				TyData::Array {
					opt: o1,
					dim: d1,
					element: e1,
				},
				TyData::Array {
					opt: o2,
					dim: d2,
					element: e2,
				},
			) => {
				(*o1 == *o2 || *o1 == OptType::NonOpt)
					&& PolymorphicFunctionType::collect_instantiations(
						db,
						add_instantiation,
						*d1,
						*d2,
					) && PolymorphicFunctionType::collect_instantiations(
					db,
					add_instantiation,
					*e1,
					*e2,
				)
			}
			(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => {
				(*i1 == *i2 || *i1 == VarType::Par)
					&& (*o1 == *o2 || *o1 == OptType::NonOpt)
					&& PolymorphicFunctionType::collect_instantiations(
						db,
						add_instantiation,
						*e1,
						*e2,
					)
			}
			(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
				(*o1 == *o2 || *o1 == OptType::NonOpt)
					&& f1.len() == f2.len()
					&& f1.iter().zip(f2.iter()).all(|(t1, t2)| {
						PolymorphicFunctionType::collect_instantiations(
							db,
							add_instantiation,
							*t1,
							*t2,
						)
					})
			}
			(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
				(*o1 == *o2 || *o1 == OptType::NonOpt)
					&& f2.iter().all(|(i2, t2)| {
						f1.iter().any(|(i1, t1)| {
							i1 == i2
								&& PolymorphicFunctionType::collect_instantiations(
									db,
									add_instantiation,
									*t1,
									*t2,
								)
						})
					})
			}
			(TyData::Function(o1, f1), TyData::Function(o2, f2)) => {
				(*o1 == OptType::NonOpt || *o1 == *o2)
					&& PolymorphicFunctionType::collect_instantiations(
						db,
						add_instantiation,
						f1.return_type,
						f2.return_type,
					) && f1.params.len() == f2.params.len()
					&& f1.params.iter().zip(f2.params.iter()).all(|(t1, t2)| {
						PolymorphicFunctionType::collect_instantiations(
							db,
							add_instantiation,
							*t2,
							*t1,
						)
					})
			}
			// Type-inst vars don't accept functions currently
			(TyData::Function(_, _), TyData::TyVar(_, _, _)) => false,
			(_, TyData::TyVar(inst, opt, t)) => {
				if arg.contains_function(db) {
					// $T doesn't accept functions
					return false;
				}
				let mut arg_ty = arg;
				if inst.is_some() {
					arg_ty = arg_ty.make_par(db);
				}
				if opt.is_some() {
					arg_ty = arg_ty.make_occurs(db);
				}
				if !arg_ty.known_varifiable(db) && t.varifiable
					|| !arg_ty.known_enumerable(db) && t.enumerable
					|| !arg_ty.known_indexable(db) && t.indexable
				{
					return false;
				}
				add_instantiation(t.ty_var, arg_ty)
			}
			_ => arg.is_subtype_of(db, param),
		}
	}

	/// Get human readable representation of type
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		format!(
			"op<{}>({}: ({}))",
			self.ty_params
				.iter()
				.map(|p| p.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", "),
			self.return_type.pretty_print(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}

	/// Get human readable representation of type as an item
	pub fn pretty_print_item(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		// TODO: output the type-inst-var definitions as well when we have syntax for this
		let prefix = if self.return_type == Ty::par_bool(db) {
			"test".to_owned()
		} else if self.return_type == Ty::par_bool(db).with_inst(db, VarType::Var).unwrap() {
			"predicate".to_owned()
		} else {
			format!("function {}:", self.return_type.pretty_print(db))
		};
		format!("{} {}", prefix, self.pretty_print_call_signature(db, name))
	}

	/// Get human readable representation of type as an item without the return type
	pub fn pretty_print_call_signature(
		&self,
		db: &'db dyn Db,
		name: impl Into<InternedString<'db>>,
	) -> String {
		format!(
			"{}({})",
			name.into().lookup(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}
