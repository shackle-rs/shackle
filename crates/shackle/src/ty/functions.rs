/// Function overloading and instantiation
use rustc_hash::FxHashMap;

use crate::db::{InternedString, Interner};

use super::{OptType, Ty, TyData, TyVarRef, VarType};

/// Represents failure to resolve overloading
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionResolutionError<T> {
	/// No matching function
	NoMatchingFunction(Vec<(T, FunctionEntry, InstantiationError)>),
	/// Ambiguous call
	AmbiguousOverloading(Vec<(T, FunctionEntry)>),
}

/// Represent failure to instantiate a function
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstantiationError {
	/// Attempted to instantiate a type-inst var with two incompatible types.
	IncompatibleTypeInstVariable {
		/// The type-inst variable
		ty_var: TyVarRef,
		/// The types which the variable was instantiated with
		types: Vec<Ty>,
	},
	/// Mismatch in type of argument
	ArgumentMismatch {
		/// The argument index
		index: usize,
		/// Expected Type
		expected: Ty,
		/// Actual type
		actual: Ty,
	},
	/// Mismatch in number of arguments
	ArgumentCountMismatch {
		/// Expected number of arguments
		expected: usize,
		/// Actual number of arguments
		actual: usize,
	},
}

/// Illegal overloading error
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OverloadingError<T> {
	/// Function with the same signature already defined
	FunctionAlreadyDefined {
		/// First function with the signature
		first: (T, FunctionEntry),
		/// Other functions with the same signature
		others: Vec<(T, FunctionEntry)>,
	},
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Candidate<T> {
	is_candidate: bool,
	has_error: bool,
	data: T,
	entry: FunctionEntry,
	instantiation: FunctionType,
}

/// An overloaded function entry
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionEntry {
	/// Whether this function has a body
	pub has_body: bool,
	/// The overloaded function
	pub overload: OverloadedFunction,
}

impl FunctionEntry {
	/// Return the most specific function overload which matches the given argument types.
	///
	/// If the function to dispatch to is polymorphic then also instantiate the polymorphic function.
	/// If there is no one specific function, this is an error.
	pub fn match_fn<T>(
		db: &dyn Interner,
		overloads: impl IntoIterator<Item = (T, FunctionEntry)>,
		args: &[Ty],
	) -> Result<(T, FunctionEntry, FunctionType), FunctionResolutionError<T>> {
		let (matches, mismatches) = overloads
			.into_iter()
			.map(|(data, entry)| {
				let instantiation = entry.overload.instantiate(db, args);
				(data, entry, instantiation)
			})
			.partition::<Vec<_>, _>(|(_, _, instantiation)| instantiation.is_ok());

		if matches.is_empty() {
			return Err(FunctionResolutionError::NoMatchingFunction(
				mismatches
					.into_iter()
					.map(|(data, overload, instantiation)| {
						(data, overload, instantiation.unwrap_err())
					})
					.collect(),
			));
		}

		let mut candidates = matches
			.into_iter()
			.map(|(data, overload, instantiation)| Candidate {
				is_candidate: true,
				has_error: overload.overload.contains_error(db),
				data,
				entry: overload,
				instantiation: instantiation.unwrap(),
			})
			.collect::<Vec<_>>();

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
			for c2 in right {
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
				let m1 = c1
					.instantiation
					.matches(db, &c2.instantiation.params)
					.is_ok();
				let m2 = c2
					.instantiation
					.matches(db, &c1.instantiation.params)
					.is_ok();
				if m1 && !m2 {
					// We accept their args, but they don't accept ours, so they're more specific
					c1.is_candidate = false;
				} else if m2 && !m1 {
					// They accept our args, but we don't accept theirs, so we're more specific
					c2.is_candidate = false;
				} else if m1 && m2 {
					// Equivalent instantiation
					match (&c1.entry.overload, &c2.entry.overload) {
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
							let m1 = p1.instantiate(db, &p2.params).is_ok();
							let m2 = p2.instantiate(db, &p1.params).is_ok();
							if m1 && !m2 {
								// We accept their args, but they don't accept ours, so they're more specific
								c1.is_candidate = false;
							} else if m2 && !m1 {
								// They accept our args, but we don't accept theirs, so we're more specific
								c2.is_candidate = false;
							} else if c1.entry.has_body && !c2.entry.has_body {
								// We have a body but they don't, so use us
								c2.is_candidate = false;
							} else if c2.entry.has_body && !c1.entry.has_body {
								// They have a body but we don't, so use them
								c1.is_candidate = false;
							} else {
								// Both have or don't have a body, so just choose one
								c2.is_candidate = false;
							}
						}
						_ => {
							if c1.entry.has_body && !c2.entry.has_body {
								// We have a body but they don't, so use us
								c2.is_candidate = false;
							} else if c2.entry.has_body && !c1.entry.has_body {
								// They have a body but we don't, so use them
								c1.is_candidate = false;
							} else {
								// Both have or don't have a body, so just choose one
								c2.is_candidate = false;
							}
						}
					}
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
				candidates.into_iter().map(|c| (c.data, c.entry)).collect(),
			));
		}
		let c = candidates.pop().unwrap();
		Ok((c.data, c.entry, c.instantiation))
	}

	/// Validate that the given overloads are legal
	pub fn check_overloading<T>(
		db: &dyn Interner,
		overloads: impl IntoIterator<Item = (T, FunctionEntry)>,
	) -> Vec<OverloadingError<T>> {
		let mut diagnostics = Vec::new();
		let overloads = overloads.into_iter().collect::<Vec<_>>();
		let mut same_fns = overloads.iter().map(|_| None).collect::<Vec<_>>();
		// TODO: Make less horrible
		for (i, (_, a)) in overloads.iter().enumerate() {
			for (j, (_, b)) in overloads[i + 1..].iter().enumerate() {
				if let Ok(fa) = a.overload.instantiate(db, b.overload.params()) {
					if b.overload.instantiate(db, a.overload.params()).is_ok() {
						if a.has_body && b.has_body || fa.return_type != b.overload.return_type() {
							// Same function with multiple definitions
							same_fns[i + j + 1] = Some(i);
						}
					}
				}
			}
		}
		let mut drain = overloads.into_iter().map(|x| Some(x)).collect::<Vec<_>>();
		for i in 0..same_fns.len() {
			let others = same_fns
				.iter()
				.enumerate()
				.filter_map(|(j, dup)| {
					if let Some(x) = dup {
						if *x == i {
							return Some(drain[j].take().unwrap());
						}
					}
					None
				})
				.collect::<Vec<_>>();
			if !others.is_empty() {
				diagnostics.push(OverloadingError::FunctionAlreadyDefined {
					first: drain[i].take().unwrap(),
					others,
				});
			}
		}
		diagnostics
	}
}

/// An overloaded function
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OverloadedFunction {
	/// A non-generic function
	Function(FunctionType),
	/// A generic function
	PolymorphicFunction(PolymorphicFunctionType),
}

impl OverloadedFunction {
	/// Get the inner non-polymorphic function
	pub fn into_function(self) -> Option<FunctionType> {
		match self {
			OverloadedFunction::Function(f) => Some(f),
			OverloadedFunction::PolymorphicFunction(_) => None,
		}
	}

	/// Get the return type of the function
	pub fn return_type(&self) -> Ty {
		match self {
			OverloadedFunction::Function(f) => f.return_type,
			OverloadedFunction::PolymorphicFunction(p) => p.return_type,
		}
	}

	/// Get the parameters of the function
	pub fn params(&self) -> &[Ty] {
		match self {
			OverloadedFunction::Function(f) => &f.params,
			OverloadedFunction::PolymorphicFunction(p) => &p.params,
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
	pub fn contains_error(&self, db: &dyn Interner) -> bool {
		match self {
			OverloadedFunction::Function(f) => f.contains_error(db),
			OverloadedFunction::PolymorphicFunction(p) => p.contains_error(db),
		}
	}

	/// Instantiate this function with the given argument types
	pub fn instantiate(
		&self,
		db: &dyn Interner,
		args: &[Ty],
	) -> Result<FunctionType, InstantiationError> {
		match self {
			OverloadedFunction::Function(f) => {
				f.matches(db, args)?;
				Ok(f.clone())
			}
			OverloadedFunction::PolymorphicFunction(p) => p.instantiate(db, args),
		}
	}

	/// Get human readable representation of this signature
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print(db),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print(db),
		}
	}

	/// Get human readable representation of this signature in item form
	pub fn pretty_print_item(&self, db: &dyn Interner, name: impl Into<InternedString>) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print_item(db, name),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print_item(db, name),
		}
	}

	/// Get human readable representation of this signature in item form without the return type
	pub fn pretty_print_call_signature(
		&self,
		db: &dyn Interner,
		name: impl Into<InternedString>,
	) -> String {
		match self {
			OverloadedFunction::Function(f) => f.pretty_print_call_signature(db, name),
			OverloadedFunction::PolymorphicFunction(p) => p.pretty_print_call_signature(db, name),
		}
	}
}

/// Type of a function expression.
///
/// Function expressions can't be generic, so there are no unbound type-inst variables.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionType {
	/// Return type
	pub return_type: Ty,
	/// Parameter types
	pub params: Box<[Ty]>,
}

impl FunctionType {
	/// Return whether this function is a subtype of another
	pub fn is_subtype_of(&self, db: &dyn Interner, other: &FunctionType) -> bool {
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
	pub fn contains_error(&self, db: &dyn Interner) -> bool {
		self.params.iter().any(|f| f.contains_error(db))
	}

	/// Whether or not the given parameter types are compatible with this function
	pub fn matches(&self, db: &dyn Interner, args: &[Ty]) -> Result<(), InstantiationError> {
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
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
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
	pub fn pretty_print_item(&self, db: &dyn Interner, name: impl Into<InternedString>) -> String {
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
		db: &dyn Interner,
		name: impl Into<InternedString>,
	) -> String {
		format!(
			"{}({})",
			name.into().value(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}

/// Type of a generic function with type-inst parameters
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PolymorphicFunctionType {
	/// Return type
	pub return_type: Ty,
	/// Type-inst parameters
	pub ty_params: Box<[TyVarRef]>,
	/// Parameter types
	pub params: Box<[Ty]>,
}

impl PolymorphicFunctionType {
	/// Return whether this function contains an error type in its parameters
	pub fn contains_error(&self, db: &dyn Interner) -> bool {
		self.params.iter().any(|f| f.contains_error(db))
	}

	/// Instantiates this polymorphic function using the given parameter types if possible.
	pub fn instantiate(
		&self,
		db: &dyn Interner,
		args: &[Ty],
	) -> Result<FunctionType, InstantiationError> {
		if args.len() != self.params.len() {
			return Err(InstantiationError::ArgumentCountMismatch {
				expected: self.params.len(),
				actual: args.len(),
			});
		}
		let mut instantiations = FxHashMap::default();
		for t in self.ty_params.iter() {
			instantiations.insert(*t, Vec::new());
		}
		for (i, (arg, param)) in args.iter().zip(self.params.iter()).enumerate() {
			if !PolymorphicFunctionType::collect_instantiations(
				db,
				&mut instantiations,
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
					resolved.insert(tv, t);
				}
				None => {
					return Err(InstantiationError::IncompatibleTypeInstVariable {
						ty_var: tv,
						types: ts,
					})
				}
			}
		}
		Ok(FunctionType {
			return_type: PolymorphicFunctionType::instantiate_type(db, &resolved, self.return_type),
			params: self
				.params
				.iter()
				.map(|p| PolymorphicFunctionType::instantiate_type(db, &resolved, *p))
				.collect(),
		})
	}

	/// Collects the types to instantiate unbound type-inst variables with.
	fn collect_instantiations(
		db: &dyn Interner,
		instantiations: &mut FxHashMap<TyVarRef, Vec<Ty>>,
		arg: Ty,
		param: Ty,
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
				(o1 == o2 || o1 == OptType::NonOpt)
					&& PolymorphicFunctionType::collect_instantiations(db, instantiations, d1, d2)
					&& PolymorphicFunctionType::collect_instantiations(db, instantiations, e1, e2)
			}
			(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => {
				(i1 == i2 || i1 == VarType::Par)
					&& (o1 == o2 || o1 == OptType::NonOpt)
					&& PolymorphicFunctionType::collect_instantiations(db, instantiations, e1, e2)
			}
			(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
				(o1 == o2 || o1 == OptType::NonOpt)
					&& f1.len() == f2.len()
					&& f1.iter().zip(f2.iter()).all(|(t1, t2)| {
						PolymorphicFunctionType::collect_instantiations(
							db,
							instantiations,
							*t1,
							*t2,
						)
					})
			}
			(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
				(o1 == o2 || o1 == OptType::NonOpt)
					&& f2.iter().all(|(i2, t2)| {
						f1.iter()
							.find(|(i1, t1)| {
								i1 == i2
									&& PolymorphicFunctionType::collect_instantiations(
										db,
										instantiations,
										*t1,
										*t2,
									)
							})
							.is_some()
					})
			}
			(TyData::Function(o1, f1), TyData::Function(o2, f2)) => {
				(o1 == OptType::NonOpt || o1 == o2)
					&& PolymorphicFunctionType::collect_instantiations(
						db,
						instantiations,
						f1.return_type,
						f2.return_type,
					) && f1.params.len() == f2.params.len()
					&& f1.params.iter().zip(f2.params.iter()).all(|(t1, t2)| {
						PolymorphicFunctionType::collect_instantiations(
							db,
							instantiations,
							*t2,
							*t1,
						)
					})
			}
			// Type-inst vars don't accept functions/arrays currently
			(TyData::Function(_, _), TyData::TyVar(_, _, _)) => false,
			(TyData::Array { .. }, TyData::TyVar(_, _, _)) => false,
			(_, TyData::TyVar(i, o, t)) => {
				let mut ty = arg;
				match (ty.inst(db), i) {
					(_, None) | (_, Some(VarType::Var)) | (Some(VarType::Par), _) => (),
					_ => return false,
				}
				match (ty.opt(db), o) {
					(_, None) | (_, Some(OptType::Opt)) | (Some(OptType::NonOpt), _) => (),
					_ => return false,
				}
				if let Some(VarType::Var) = i {
					ty = ty.with_inst(db, VarType::Par).expect("Failed to make par!");
				}
				if let Some(OptType::Opt) = o {
					ty = ty.with_opt(db, OptType::NonOpt);
				}
				if !ty.known_varifiable(db) && t.varifiable
					|| !ty.known_enumerable(db) && t.enumerable
					|| !ty.known_indexable(db) && t.indexable
				{
					return false;
				}
				if let Some(is) = instantiations.get_mut(&t.ty_var) {
					is.push(ty);
					return true;
				}
				false
			}
			_ => arg.is_subtype_of(db, param),
		}
	}

	/// Instantiate the given type-inst variables with the given types from `instantiations` in the type `t`.
	fn instantiate_type(db: &dyn Interner, instantiations: &FxHashMap<TyVarRef, Ty>, t: Ty) -> Ty {
		match t.lookup(db) {
			TyData::TyVar(i, o, t) if instantiations.contains_key(&t.ty_var) => {
				let mut ty = instantiations[&t.ty_var];
				if let Some(inst) = i {
					ty = ty
						.with_inst(db, inst)
						.expect("Type-inst is incompatible with type-inst var");
				}
				if let Some(opt) = o {
					ty = ty.with_opt(db, opt);
				}
				ty
			}
			TyData::Array { opt, dim, element } => db.intern_ty(TyData::Array {
				opt,
				dim: PolymorphicFunctionType::instantiate_type(db, instantiations, dim),
				element: PolymorphicFunctionType::instantiate_type(db, instantiations, element),
			}),
			TyData::Set(i, o, t) => db.intern_ty(TyData::Set(
				i,
				o,
				PolymorphicFunctionType::instantiate_type(db, instantiations, t),
			)),
			TyData::Tuple(o, fs) => db.intern_ty(TyData::Tuple(
				o,
				fs.iter()
					.map(|f| PolymorphicFunctionType::instantiate_type(db, instantiations, *f))
					.collect(),
			)),
			TyData::Record(o, fs) => db.intern_ty(TyData::Record(
				o,
				fs.iter()
					.map(|(i, f)| {
						(
							*i,
							PolymorphicFunctionType::instantiate_type(db, instantiations, *f),
						)
					})
					.collect(),
			)),
			_ => t,
		}
	}

	/// Get human readable representation of type
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
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
	pub fn pretty_print_item(&self, db: &dyn Interner, name: impl Into<InternedString>) -> String {
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
		db: &dyn Interner,
		name: impl Into<InternedString>,
	) -> String {
		format!(
			"{}({})",
			name.into().value(db),
			self.params
				.iter()
				.map(|t| t.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
}
