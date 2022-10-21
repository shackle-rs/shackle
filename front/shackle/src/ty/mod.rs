//! Representation of computed types of expressions.
//!
//! `TyData` contains the actual type information, and are referred to using the interned `Ty` struct.
//! There are a number of differences between the semantic `Ty` representation and the syntactic HIR representation of types:
//!
//! - `Ty` supports optionality for all types, whereas there is no syntax for e.g. opt arrays
//! - `Ty` type-inst IDs are called `TyVar`s and are based on restriction - so a plain `TyVar` can be any type-inst rather than being par, non-opt.
//! - Type-inst aliases are resolved, so `Ty`s do not refer to aliases.
//! - Type-inst domains are not included, only types. So e.g. `var 1..3` and `var int` are the same `Ty`.
//! - Function types are never generic - since function expressions have all type-inst parameters already bound
//!   (although the bound type-inst may itself be a type-inst variable, such as when calling a generic function from another generic function).

use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use std::sync::atomic::AtomicU32;

use crate::db::{InternedString, Interner};
use crate::hir::db::Hir;
use crate::hir::ids::PatternRef;

mod functions;
pub use self::functions::*;
pub use crate::hir::{OptType, VarType};

/// A type used in the type-system (as opposed to the type that is declared by the user and used in the `hir` module).
///
/// Do not use `db.intern_ty` to create types from `TyData` directly.
/// Instead use the constructor functions as these ensure that the produced type is valid.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ty(salsa::InternId);

impl salsa::InternKey for Ty {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

impl Ty {
	/// Get the underlying type data
	pub fn lookup(&self, db: &dyn Interner) -> TyData {
		db.lookup_intern_ty(*self)
	}

	/// Create an error type
	pub fn error(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Error)
	}

	/// Create a par bool
	pub fn par_bool(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Boolean(VarType::Par, OptType::NonOpt))
	}

	/// Create a par int
	pub fn par_int(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Integer(VarType::Par, OptType::NonOpt))
	}

	/// Create a par float
	pub fn par_float(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Float(VarType::Par, OptType::NonOpt))
	}

	/// Create a par enum
	pub fn par_enum(db: &dyn Interner, e: EnumRef) -> Self {
		db.intern_ty(TyData::Enum(VarType::Par, OptType::NonOpt, e))
	}

	/// Create a par string
	pub fn string(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::String(OptType::NonOpt))
	}

	/// Create a par annotation type
	pub fn ann(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Annotation(OptType::NonOpt))
	}

	/// Create a par bottom type
	pub fn bottom(db: &dyn Interner) -> Self {
		db.intern_ty(TyData::Bottom(OptType::NonOpt))
	}

	/// Create an array type if possible
	pub fn array(db: &dyn Interner, dim: Ty, element: Ty) -> Option<Self> {
		if !dim.known_par(db) || !dim.known_occurs(db) || !dim.known_indexable(db) {
			// Invalid index type
			return None;
		}
		Some(db.intern_ty(TyData::Array {
			opt: OptType::NonOpt,
			dim,
			element,
		}))
	}

	/// Create a par set type if possible
	pub fn par_set(db: &dyn Interner, element: Ty) -> Option<Self> {
		if element.known_par(db) && element.known_occurs(db) {
			Some(db.intern_ty(TyData::Set(VarType::Par, OptType::NonOpt, element)))
		} else {
			None
		}
	}

	/// Create a tuple type
	pub fn tuple(db: &dyn Interner, fields: impl IntoIterator<Item = Ty>) -> Self {
		db.intern_ty(TyData::Tuple(OptType::NonOpt, fields.into_iter().collect()))
	}

	/// Create a record type
	pub fn record(
		db: &dyn Interner,
		fields: impl IntoIterator<Item = (impl Into<InternedString>, Ty)>,
	) -> Self {
		db.intern_ty(TyData::Record(
			OptType::NonOpt,
			fields.into_iter().map(|(i, t)| (i.into(), t)).collect(),
		))
	}

	/// Create a function type
	pub fn function(db: &dyn Interner, f: FunctionType) -> Self {
		db.intern_ty(TyData::Function(OptType::NonOpt, f))
	}

	/// Create a type-inst variable type
	pub fn type_inst_var(db: &dyn Interner, t: TyVar) -> Self {
		db.intern_ty(TyData::TyVar(None, None, t))
	}

	/// Sets the inst of this type if possible.
	///
	/// Some types e.g. var arrays are not possible.
	pub fn with_inst(&self, db: &dyn Interner, inst: VarType) -> Option<Ty> {
		let result = match self.lookup(db) {
			TyData::Boolean(_, o) => TyData::Boolean(inst, o),
			TyData::Integer(_, o) => TyData::Integer(inst, o),
			TyData::Float(_, o) => TyData::Float(inst, o),
			TyData::Enum(_, o, e) => TyData::Enum(inst, o, e),
			TyData::String(_) | TyData::Annotation(_) | TyData::Bottom(_)
				if inst == VarType::Par =>
			{
				return Some(*self)
			}
			TyData::Array { .. } => return None,
			TyData::Set(_, o, e) => {
				if inst == VarType::Var && !e.known_enumerable(db) {
					return None;
				}
				TyData::Set(inst, o, e)
			}
			TyData::Tuple(o, fs) => TyData::Tuple(
				o,
				fs.iter()
					.map(|f| f.with_inst(db, inst))
					.collect::<Option<_>>()?,
			),
			TyData::Record(o, fs) => TyData::Record(
				o,
				fs.iter()
					.map(|(i, f)| Some((*i, f.with_inst(db, inst)?)))
					.collect::<Option<_>>()?,
			),
			TyData::Function(_, _) if inst == VarType::Par => return Some(*self),
			TyData::TyVar(_, o, t) => {
				if inst == VarType::Var && !t.varifiable {
					return None;
				}
				TyData::TyVar(Some(inst), o, t)
			}
			TyData::Bottom(_) | TyData::Error => return Some(*self),
			_ => return None,
		};
		Some(db.intern_ty(result))
	}

	/// Returns the fixed version of this type
	pub fn make_fixed(&self, db: &dyn Interner) -> Ty {
		db.intern_ty(match self.lookup(db) {
			TyData::Boolean(_, o) => TyData::Boolean(VarType::Par, o),
			TyData::Integer(_, o) => TyData::Integer(VarType::Par, o),
			TyData::Float(_, o) => TyData::Float(VarType::Par, o),
			TyData::Enum(_, o, e) => TyData::Enum(VarType::Par, o, e),
			TyData::Array { opt, dim, element } => TyData::Array {
				opt,
				dim,
				element: element.make_fixed(db),
			},
			TyData::Set(_, o, e) => TyData::Set(VarType::Par, o, e),
			TyData::Tuple(o, fs) => TyData::Tuple(o, fs.iter().map(|f| f.make_fixed(db)).collect()),
			TyData::Record(o, fs) => {
				TyData::Record(o, fs.iter().map(|(i, f)| (*i, f.make_fixed(db))).collect())
			}
			TyData::TyVar(_, o, t) => TyData::TyVar(Some(VarType::Par), o, t),
			_ => return *self,
		})
	}

	/// Sets the optionality of this type if possible.
	pub fn with_opt(&self, db: &dyn Interner, opt: OptType) -> Ty {
		db.intern_ty(match self.lookup(db) {
			TyData::Boolean(i, _) => TyData::Boolean(i, opt),
			TyData::Integer(i, _) => TyData::Integer(i, opt),
			TyData::Float(i, _) => TyData::Float(i, opt),
			TyData::Enum(i, _, e) => TyData::Enum(i, opt, e),
			TyData::String(_) => TyData::String(opt),
			TyData::Annotation(_) => TyData::Annotation(opt),
			TyData::Bottom(_) => TyData::Bottom(opt),
			TyData::Array { dim, element, .. } => TyData::Array { opt, dim, element },
			TyData::Set(i, _, e) => TyData::Set(i, opt, e),
			TyData::Tuple(_, fs) => TyData::Tuple(opt, fs),
			TyData::Record(_, fs) => TyData::Record(opt, fs),
			TyData::Function(_, f) => TyData::Function(opt, f),
			TyData::TyVar(i, _, t) => TyData::TyVar(i, Some(opt), t),
			TyData::Error => return *self,
		})
	}

	/// Whether this type-inst is known to be completely par.
	pub fn known_par(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::Error
			| TyData::Boolean(VarType::Par, _)
			| TyData::Integer(VarType::Par, _)
			| TyData::Float(VarType::Par, _)
			| TyData::Enum(VarType::Par, _, _)
			| TyData::String(_)
			| TyData::Annotation(_)
			| TyData::Bottom(_)
			| TyData::Function(_, _)
			| TyData::Set(VarType::Par, _, _)
			| TyData::TyVar(Some(VarType::Par), _, _) => true,
			TyData::Array { element, .. } => element.known_par(db),
			TyData::Tuple(_, fs) => fs.iter().all(|f| f.known_par(db)),
			TyData::Record(_, fs) => fs.iter().all(|(_, f)| f.known_par(db)),
			_ => false,
		}
	}

	/// Whether this type-inst is known to be non-optional.
	pub fn known_occurs(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::Error
			| TyData::Boolean(_, OptType::NonOpt)
			| TyData::Integer(_, OptType::NonOpt)
			| TyData::Float(_, OptType::NonOpt)
			| TyData::Enum(_, OptType::NonOpt, _)
			| TyData::String(OptType::NonOpt)
			| TyData::Annotation(OptType::NonOpt)
			| TyData::Bottom(OptType::NonOpt)
			| TyData::Function(OptType::NonOpt, _)
			| TyData::Set(_, OptType::NonOpt, _)
			| TyData::TyVar(_, Some(OptType::NonOpt), _)
			| TyData::Array {
				opt: OptType::NonOpt,
				..
			}
			| TyData::Tuple(OptType::NonOpt, _)
			| TyData::Record(OptType::NonOpt, _) => true,
			_ => false,
		}
	}

	/// Whether this type-inst can definitely be made var
	pub fn known_varifiable(&self, db: &dyn Interner) -> bool {
		self.with_inst(db, VarType::Var).is_some()
	}

	/// Whether this type-inst is enumerable ($$E)
	pub fn known_enumerable(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::Error
			| TyData::Bottom(_)
			| TyData::Boolean(_, _)
			| TyData::Integer(_, _)
			| TyData::Enum(_, _, _) => true,
			TyData::TyVar(_, _, t) => t.enumerable,
			_ => false,
		}
	}

	/// Whether this type-inst is indexable ($T used in an array dimension)
	pub fn known_indexable(&self, db: &dyn Interner) -> bool {
		self.known_enumerable(db)
			|| match self.lookup(db) {
				TyData::Tuple(_, fs) => fs.iter().all(|f| f.known_enumerable(db)),
				TyData::TyVar(_, _, t) => t.indexable,
				_ => false,
			}
	}

	/// Whether this type-inst has a default value (allowing omission of else branch in ITE)
	pub fn has_default_value(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::Boolean(_, _)
			| TyData::Integer(_, OptType::Opt)
			| TyData::Float(_, OptType::Opt)
			| TyData::Enum(_, OptType::Opt, _)
			| TyData::String(_)
			| TyData::Annotation(_)
			| TyData::Bottom(OptType::Opt)
			| TyData::Array { .. }
			| TyData::Set(_, _, _)
			| TyData::Tuple(OptType::Opt, _)
			| TyData::Record(OptType::Opt, _)
			| TyData::Function(OptType::Opt, _)
			| TyData::TyVar(_, Some(OptType::Opt), _)
			| TyData::Error => true,
			TyData::Tuple(_, fs) => fs.iter().all(|f| f.has_default_value(db)),
			TyData::Record(_, fs) => fs.iter().all(|(_, f)| f.has_default_value(db)),
			_ => false,
		}
	}

	/// Whether this type-inst contains a type-inst variable
	pub fn contains_type_inst_var(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::TyVar(_, _, _) => true,
			TyData::Array { dim, element, .. } => {
				dim.contains_type_inst_var(db) || element.contains_type_inst_var(db)
			}
			TyData::Set(_, _, e) => e.contains_type_inst_var(db),
			TyData::Tuple(_, fs) => fs.iter().any(|f| f.contains_type_inst_var(db)),
			TyData::Record(_, fs) => fs.iter().any(|(_, f)| f.contains_type_inst_var(db)),
			TyData::Function(_, f) => {
				f.return_type.contains_type_inst_var(db)
					|| f.params.iter().any(|p| p.contains_type_inst_var(db))
			}
			_ => false,
		}
	}

	/// Whether this type inst contains something that is par
	pub fn contains_par(&self, db: &dyn Interner) -> bool {
		self.known_par(db)
			|| match self.lookup(db) {
				TyData::Array { element, .. } => element.contains_par(db),
				TyData::Tuple(_, fs) => fs.iter().any(|f| f.contains_par(db)),
				TyData::Record(_, fs) => fs.iter().any(|(_, f)| f.contains_par(db)),
				_ => false,
			}
	}

	/// Whether this type-inst contains an error.
	pub fn contains_error(&self, db: &dyn Interner) -> bool {
		match self.lookup(db) {
			TyData::Error => true,
			TyData::Function(_, f) => f.contains_error(db),
			TyData::Set(_, _, e) => e.contains_error(db),
			TyData::Array { dim, element, .. } => {
				dim.contains_error(db) || element.contains_error(db)
			}
			TyData::Tuple(_, fs) => fs.iter().any(|f| f.contains_error(db)),
			TyData::Record(_, fs) => fs.iter().any(|(_, f)| f.contains_error(db)),
			_ => false,
		}
	}

	/// The inst of this type (or `None` if the inst cannot be determined)
	pub fn inst(&self, db: &dyn Interner) -> Option<VarType> {
		match self.lookup(db) {
			TyData::Boolean(inst, _)
			| TyData::Integer(inst, _)
			| TyData::Float(inst, _)
			| TyData::Enum(inst, _, _)
			| TyData::Set(inst, _, _)
			| TyData::TyVar(Some(inst), _, _) => Some(inst),

			TyData::Error
			| TyData::String(_)
			| TyData::Annotation(_)
			| TyData::Bottom(_)
			| TyData::Array { .. }
			| TyData::Tuple(_, _)
			| TyData::Record(_, _)
			| TyData::Function(_, _) => Some(VarType::Par),
			_ => None,
		}
	}

	/// The optionality of this type (or `None` if the optionality cannot be determined)
	pub fn opt(&self, db: &dyn Interner) -> Option<OptType> {
		match self.lookup(db) {
			TyData::Boolean(_, opt)
			| TyData::Integer(_, opt)
			| TyData::Float(_, opt)
			| TyData::Enum(_, opt, _)
			| TyData::String(opt)
			| TyData::Annotation(opt)
			| TyData::Bottom(opt)
			| TyData::Function(opt, _)
			| TyData::Set(_, opt, _)
			| TyData::TyVar(_, Some(opt), _)
			| TyData::Array { opt, .. }
			| TyData::Tuple(opt, _)
			| TyData::Record(opt, _) => Some(opt),
			TyData::Error => Some(OptType::NonOpt),
			_ => None,
		}
	}

	/// Get the most specific supertype of the given types if there is one.
	///
	/// Returns `None` if there is no supertype of the given types.
	/// Returns the error type if any of the given types are error types.
	///
	/// Note that there is no supertype of e.g. `var int` and `any $T` since `$T` may be bound to an
	/// incompatible type-inst (e.g. a set type).
	pub fn most_specific_supertype(
		db: &dyn Interner,
		ts: impl IntoIterator<Item = Ty>,
	) -> Option<Ty> {
		ts.into_iter()
			.map(Some)
			.reduce(|a, b| {
				// E.g. t1 = var opt bool, t2 = var int, then super type is var opt int
				let result = match (a?.lookup(db), b?.lookup(db)) {
					(TyData::Error, _) | (_, TyData::Error) => TyData::Error,
					(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2)) => {
						TyData::Boolean(i1.max(i2), o1.max(o2))
					}
					(TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Integer(i1, o1), TyData::Integer(i2, o2)) => {
						TyData::Integer(i1.max(i2), o1.max(o2))
					}
					(TyData::Boolean(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Integer(i1, o1))
					| (TyData::Float(i2, o2), TyData::Float(i1, o1)) => TyData::Float(i1.max(i2), o1.max(o2)),
					(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) if e1 == e2 => {
						TyData::Enum(i1.max(i2), o1.max(o2), e1)
					}
					(TyData::String(o1), TyData::String(o2)) => TyData::String(o1.max(o2)),
					(TyData::Annotation(o1), TyData::Annotation(o2)) => {
						TyData::Annotation(o1.max(o2))
					}
					(TyData::Bottom(o1), TyData::Bottom(o2)) => TyData::Bottom(o1.max(o2)),
					(TyData::Bottom(o1), TyData::Boolean(i, o2))
					| (TyData::Boolean(i, o2), TyData::Bottom(o1)) => TyData::Boolean(i, o1.max(o2)),
					(TyData::Bottom(o1), TyData::Integer(i, o2))
					| (TyData::Integer(i, o2), TyData::Bottom(o1)) => TyData::Integer(i, o1.max(o2)),
					(TyData::Bottom(o1), TyData::Float(i, o2))
					| (TyData::Float(i, o2), TyData::Bottom(o1)) => TyData::Float(i, o1.max(o2)),
					(TyData::Bottom(o1), TyData::Enum(i, o2, e))
					| (TyData::Enum(i, o2, e), TyData::Bottom(o1)) => TyData::Enum(i, o1.max(o2), e),
					(TyData::Bottom(o1), TyData::String(o2))
					| (TyData::String(o2), TyData::Bottom(o1)) => TyData::String(o1.max(o2)),
					(TyData::Bottom(o1), TyData::Annotation(o2))
					| (TyData::Annotation(o2), TyData::Bottom(o1)) => TyData::Annotation(o1.max(o2)),
					(
						TyData::Bottom(o1),
						TyData::Array {
							opt: o2,
							dim,
							element,
						},
					)
					| (
						TyData::Array {
							opt: o2,
							dim,
							element,
						},
						TyData::Bottom(o1),
					) => TyData::Array {
						opt: o1.max(o2),
						dim,
						element,
					},
					(TyData::Bottom(o1), TyData::Set(inst, o2, element))
					| (TyData::Set(inst, o2, element), TyData::Bottom(o1)) => {
						TyData::Set(inst, o1.max(o2), element)
					}
					(TyData::Bottom(o1), TyData::Tuple(o2, fields))
					| (TyData::Tuple(o2, fields), TyData::Bottom(o1)) => TyData::Tuple(o1.max(o2), fields),
					(TyData::Bottom(o1), TyData::Record(o2, fields))
					| (TyData::Record(o2, fields), TyData::Bottom(o1)) => TyData::Record(o1.max(o2), fields),
					(TyData::Bottom(o1), TyData::Function(o2, f))
					| (TyData::Function(o2, f), TyData::Bottom(o1)) => TyData::Function(o1.max(o2), f),
					(TyData::Bottom(o1), TyData::TyVar(inst, o2, tv))
					| (TyData::TyVar(inst, o2, tv), TyData::Bottom(o1)) => {
						TyData::TyVar(inst, o2.map(|o2| o1.max(o2)), tv)
					}
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
					) => TyData::Array {
						opt: o1.max(o2),
						dim: Ty::most_specific_supertype(db, [d1, d2])?,
						element: Ty::most_specific_supertype(db, [e1, e2])?,
					},
					(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => TyData::Set(
						i1.max(i2),
						o1.max(o2),
						Ty::most_specific_supertype(db, [e1, e2])?,
					),
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => TyData::Tuple(o1.max(o2), {
						if f1.len() != f2.len() {
							return None;
						}
						f1.iter()
							.zip(f2.iter())
							.map(|(t1, t2)| Ty::most_specific_supertype(db, [*t1, *t2]))
							.collect::<Option<_>>()?
					}),
					(TyData::Record(o1, f1), TyData::Record(o2, f2)) => TyData::Record(
						o1.max(o2),
						// Intersection and supertype fields
						// e.g. super type of `record(int: a, bool: b)` and `record(bool: a)` is `record(int: a)`
						f1.iter()
							.filter_map(|(i1, t1)| {
								f2.iter().find(|(i2, _)| i1 == i2).and_then(|(i2, t2)| {
									Some((*i2, Ty::most_specific_supertype(db, [*t1, *t2])?))
								})
							})
							.collect(),
					),
					(TyData::Function(o1, f1), TyData::Function(o2, f2)) => TyData::Function(
						o1.max(o2),
						FunctionType {
							return_type: Ty::most_specific_supertype(
								db,
								[f1.return_type, f2.return_type],
							)?,
							params: {
								if f1.params.len() != f2.params.len() {
									return None;
								}
								f1.params
									.iter()
									.zip(f2.params.iter())
									.map(|(t1, t2)| Ty::most_general_subtype(db, [*t1, *t2]))
									.collect::<Option<_>>()?
							},
						},
					),
					(TyData::TyVar(i1, o1, t1), TyData::TyVar(i2, o2, t2)) if t1 == t2 => {
						TyData::TyVar(
							match (i1, i2) {
								(Some(VarType::Var), _) | (_, Some(VarType::Var)) => {
									Some(VarType::Var)
								}
								(Some(VarType::Par), Some(VarType::Par)) => Some(VarType::Par),
								_ => None,
							},
							match (o1, o2) {
								(Some(OptType::Opt), _) | (_, Some(OptType::Opt)) => {
									Some(OptType::Opt)
								}
								(Some(OptType::NonOpt), Some(OptType::NonOpt)) => {
									Some(OptType::NonOpt)
								}
								_ => None,
							},
							t1,
						)
					}
					_ => {
						return None;
					}
				};
				Some(db.intern_ty(result))
			})
			.flatten()
	}

	/// Get the most general subtype of the given types if there is one.
	///
	/// Returns `None` if there is no subtype of the given types.
	/// Returns the error type if any of the given types are error types.
	///
	/// Note that there is no subtype of e.g. `var int` and `any $T` since `$T` may be bound to an
	/// incompatible type-inst (e.g. a set type).
	pub fn most_general_subtype(db: &dyn Interner, ts: impl IntoIterator<Item = Ty>) -> Option<Ty> {
		ts.into_iter()
			.map(Some)
			.reduce(|a, b| {
				let result = match (a?.lookup(db), b?.lookup(db)) {
					(TyData::Error, _) | (_, TyData::Error) => TyData::Error,
					(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2))
					| (TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Boolean(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Boolean(i1, o1)) => TyData::Boolean(i1.min(i2), o1.min(o2)),
					(TyData::Integer(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Integer(i1, o1)) => TyData::Integer(i1.min(i2), o1.min(o2)),
					(TyData::Float(i1, o1), TyData::Float(i2, o2)) => {
						TyData::Float(i1.min(i2), o1.min(o2))
					}
					(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) if e1 == e2 => {
						TyData::Enum(i1.min(i2), o1.min(o2), e1)
					}
					(TyData::String(o1), TyData::String(o2)) => TyData::String(o1.min(o2)),
					(TyData::Annotation(o1), TyData::Annotation(o2)) => {
						TyData::Annotation(o1.min(o2))
					}
					(TyData::Bottom(o1), TyData::Bottom(o2))
					| (TyData::Bottom(o1), TyData::Boolean(_, o2))
					| (TyData::Boolean(_, o2), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Integer(_, o2))
					| (TyData::Integer(_, o2), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Float(_, o2))
					| (TyData::Float(_, o2), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Enum(_, o2, _))
					| (TyData::Enum(_, o2, _), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::String(o2))
					| (TyData::String(o2), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Annotation(o2))
					| (TyData::Annotation(o2), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Array { opt: o2, .. })
					| (TyData::Array { opt: o2, .. }, TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Set(_, o2, _))
					| (TyData::Set(_, o2, _), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Tuple(o2, _))
					| (TyData::Tuple(o2, _), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Record(o2, _))
					| (TyData::Record(o2, _), TyData::Bottom(o1))
					| (TyData::Bottom(o1), TyData::Function(o2, _))
					| (TyData::Function(o2, _), TyData::Bottom(o1)) => TyData::Bottom(o1.min(o2)),
					(TyData::Bottom(o1), TyData::TyVar(_, o2, _))
					| (TyData::TyVar(_, o2, _), TyData::Bottom(o1)) => {
						TyData::Bottom(Some(o1).min(o2).unwrap_or(OptType::NonOpt))
					}
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
					) => TyData::Array {
						opt: o1.min(o2),
						dim: Ty::most_general_subtype(db, [d1, d2])?,
						element: Ty::most_general_subtype(db, [e1, e2])?,
					},
					(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => TyData::Set(
						i1.min(i2),
						o1.min(o2),
						Ty::most_general_subtype(db, [e1, e2])?,
					),
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => TyData::Tuple(o1.max(o2), {
						if f1.len() != f2.len() {
							return None;
						}
						f1.iter()
							.zip(f2.iter())
							.map(|(t1, t2)| Ty::most_general_subtype(db, [*t1, *t2]))
							.collect::<Option<_>>()?
					}),
					(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
						// Union and subtype fields
						// e.g. subtype of `record(int: a, bool: b)` and `record(bool: a)` is `record(bool: a, bool b)`
						let mut fields = FxHashMap::default();
						for (i, t) in f1.iter().chain(f2.iter()) {
							match fields.entry(*i) {
								Entry::Occupied(mut e) => {
									*e.get_mut() = Ty::most_general_subtype(db, [*e.get(), *t])?
								}
								Entry::Vacant(e) => {
									e.insert(*t);
								}
							}
						}
						TyData::Record(o1.max(o2), fields.into_iter().collect())
					}
					(TyData::Function(o1, f1), TyData::Function(o2, f2)) => TyData::Function(
						o1.max(o2),
						FunctionType {
							return_type: Ty::most_general_subtype(
								db,
								[f1.return_type, f2.return_type],
							)?,
							params: {
								if f1.params.len() != f2.params.len() {
									return None;
								}
								f1.params
									.iter()
									.zip(f2.params.iter())
									.map(|(t1, t2)| Ty::most_specific_supertype(db, [*t1, *t2]))
									.collect::<Option<_>>()?
							},
						},
					),
					(TyData::TyVar(i1, o1, t1), TyData::TyVar(i2, o2, t2)) if t1 == t2 => {
						TyData::TyVar(
							match (i1, i2) {
								(Some(VarType::Par), _) | (_, Some(VarType::Par)) => {
									Some(VarType::Par)
								}
								(Some(VarType::Var), Some(VarType::Var)) => Some(VarType::Var),
								_ => None,
							},
							match (o1, o2) {
								(Some(OptType::NonOpt), _) | (_, Some(OptType::NonOpt)) => {
									Some(OptType::NonOpt)
								}
								(Some(OptType::Opt), Some(OptType::Opt)) => Some(OptType::Opt),
								_ => None,
							},
							t1,
						)
					}
					_ => {
						return None;
					}
				};
				Some(db.intern_ty(result))
			})
			.flatten()
	}

	/// Whether this type is a subtype of `other`.
	///
	/// Note that e.g. `int` is not a subtype of `any $T` since $T may be bound to an incompatible
	/// type-inst (e.g. a set type).
	pub fn is_subtype_of(&self, db: &dyn Interner, other: Ty) -> bool {
		if *self == other {
			return true;
		}
		match (self.lookup(db), other.lookup(db)) {
			(TyData::Error, _) | (_, TyData::Error) => true,
			// Scalar coercions
			(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2))
			| (TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
			| (TyData::Boolean(i1, o1), TyData::Float(i2, o2))
			| (TyData::Integer(i1, o1), TyData::Integer(i2, o2))
			| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
			| (TyData::Float(i1, o1), TyData::Float(i2, o2)) => {
				(i1 == VarType::Par || i1 == i2) && (o1 == OptType::NonOpt || o1 == o2)
			}
			(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) => {
				(i1 == VarType::Par || i1 == i2) && (o1 == OptType::NonOpt || o1 == o2) && e1 == e2
			}
			(TyData::Bottom(o1), TyData::Bottom(o2))
			| (TyData::Bottom(o1), TyData::Boolean(_, o2))
			| (TyData::Bottom(o1), TyData::Integer(_, o2))
			| (TyData::Bottom(o1), TyData::Float(_, o2))
			| (TyData::Bottom(o1), TyData::Enum(_, o2, _))
			| (TyData::Bottom(o1), TyData::String(o2))
			| (TyData::Bottom(o1), TyData::Annotation(o2))
			| (TyData::Bottom(o1), TyData::Array { opt: o2, .. })
			| (TyData::Bottom(o1), TyData::Set(_, o2, _))
			| (TyData::Bottom(o1), TyData::Tuple(o2, _))
			| (TyData::Bottom(o1), TyData::Record(o2, _))
			| (TyData::Bottom(o1), TyData::Function(o2, _))
			| (TyData::String(o1), TyData::String(o2))
			| (TyData::Annotation(o1), TyData::Annotation(o2)) => o1 == OptType::NonOpt || o1 == o2,
			// Compound type coercions
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
				(o1 == OptType::NonOpt || o1 == o2)
					&& d1.is_subtype_of(db, d2)
					&& e1.is_subtype_of(db, e2)
			}
			(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => {
				(o1 == OptType::NonOpt || o1 == o2)
					&& (i1 == VarType::Par || i1 == i2)
					&& e1.is_subtype_of(db, e2)
			}
			(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
				(o1 == OptType::NonOpt || o1 == o2)
					&& f1.len() == f2.len()
					&& f1
						.iter()
						.zip(f2.iter())
						.all(|(t1, t2)| t1.is_subtype_of(db, *t2))
			}
			(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
				(o1 == OptType::NonOpt || o1 == o2)
					&& f2.iter().all(|(i2, t2)| {
						f1.iter()
							.find(|(i1, t1)| i1 == i2 && t1.is_subtype_of(db, *t2))
							.is_some()
					})
			}
			// Function coercion
			(TyData::Function(o1, f1), TyData::Function(o2, f2)) => {
				(o1 == OptType::NonOpt || o1 == o2) && f1.is_subtype_of(db, &f2)
			}
			// Type-inst var coercion (par T -> var T, par T -> T, T -> var T)
			(TyData::TyVar(i1, o1, t1), TyData::TyVar(i2, o2, t2)) => {
				(i1 == i2 || i1 == Some(VarType::Par) || i2.is_none())
					&& (o1 == o2 || o1 == Some(OptType::NonOpt) || o2.is_none())
					&& t1 == t2
			}
			(TyData::Bottom(o1), TyData::TyVar(_, o2, _)) => {
				o1 == OptType::NonOpt || Some(o1) == o2
			}
			_ => false,
		}
	}

	/// Get human readable type name
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
		match self.lookup(db) {
			TyData::Boolean(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print().into_iter())
				.chain(["bool".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Integer(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print().into_iter())
				.chain(["int".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Float(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print().into_iter())
				.chain(["float".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Enum(i, o, e) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print().into_iter())
				.chain([e.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::String(o) => o
				.pretty_print()
				.into_iter()
				.chain(["string".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Annotation(o) => o
				.pretty_print()
				.into_iter()
				.chain(["ann".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Bottom(o) => o
				.pretty_print()
				.into_iter()
				.chain(["..".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Array { opt, dim, element } => opt
				.pretty_print()
				.into_iter()
				.chain([
					"array".to_owned(),
					format!("[{}]", dim.pretty_print_as_dims(db)),
					"of".to_owned(),
					element.pretty_print(db),
				])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Set(i, o, e) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print().into_iter())
				.chain(["set of".to_owned(), e.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Tuple(o, fs) => o
				.pretty_print()
				.into_iter()
				.chain([format!(
					"tuple({})",
					fs.iter()
						.map(|f| f.pretty_print(db))
						.collect::<Vec<_>>()
						.join(", ")
				)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Record(o, fs) => o
				.pretty_print()
				.into_iter()
				.chain([format!(
					"record({})",
					fs.iter()
						.map(|(i, f)| format!("{}: {}", f.pretty_print(db), i.value(db)))
						.collect::<Vec<_>>()
						.join(", ")
				)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Function(o, f) => o
				.pretty_print()
				.into_iter()
				.chain([f.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::TyVar(None, None, t) => format!("any {}", t.ty_var.pretty_print(db)),
			TyData::TyVar(i, o, t) => i
				.map(|i| i.pretty_print())
				.unwrap_or(Some("anyvar".to_owned()))
				.into_iter()
				.chain(
					o.map(|o| o.pretty_print())
						.unwrap_or(Some("anyopt".to_string())),
				)
				.chain([t.ty_var.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Error => "error".to_owned(),
		}
	}

	/// Pretty print as if type was in the dimension position in an array type
	pub fn pretty_print_as_dims(&self, db: &dyn Interner) -> String {
		match self.lookup(db) {
			TyData::Tuple(OptType::NonOpt, fs) => fs
				.iter()
				.map(|f| f.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", "),
			_ => self.pretty_print(db),
		}
	}
}

/// A type used in the type-system (as opposed to the type that is declared by the user and used in the `hir` module).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TyData {
	/// Boolean scalar
	Boolean(VarType, OptType),
	/// Integer scalar
	Integer(VarType, OptType),
	/// Float scalar
	Float(VarType, OptType),
	/// Enumerated type scalar
	Enum(VarType, OptType, EnumRef),

	/// String scalar
	String(OptType),
	/// Annotation scalar
	Annotation(OptType),
	/// Type for `<>`, `{}`, `[]`
	Bottom(OptType),

	/// Array type
	Array {
		/// Whether the array is optional
		opt: OptType,
		/// Type used for indexing
		dim: Ty,
		/// Type of the element
		element: Ty,
	},
	/// Set type
	Set(VarType, OptType, Ty),
	/// Tuple type
	Tuple(OptType, Box<[Ty]>),
	/// Record type
	Record(OptType, Box<[(InternedString, Ty)]>),

	/// Type of a function
	Function(OptType, FunctionType),

	/// Type inst variable and modifiers to apply to the substituted type-inst
	TyVar(Option<VarType>, Option<OptType>, TyVar),

	/// Indicates a type error but allows typing to continue
	Error,
}

/// A new type (e.g. enums, type-inst vars)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct NewType(salsa::InternId);

impl salsa::InternKey for NewType {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

impl NewType {
	fn from_pattern(db: &dyn Hir, pattern: PatternRef) -> Self {
		let item = pattern.item();
		let model = item.model(db);
		let name = item.local_item_ref(db).data(&*model)[pattern.pattern()]
			.identifier()
			.unwrap()
			.0;
		db.intern_newtype(NewTypeData {
			kind: NewTypeKind::Pattern(pattern),
			name,
		})
	}

	fn introduce(db: &dyn Interner, name: impl Into<InternedString>) -> Self {
		static COUNTER: AtomicU32 = AtomicU32::new(0);
		db.intern_newtype(NewTypeData {
			kind: NewTypeKind::Introduced(
				COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
			),
			name: name.into(),
		})
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum NewTypeKind {
	Pattern(PatternRef),
	Introduced(u32),
}

/// A new type (e.g. enums, type-inst vars)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NewTypeData {
	kind: NewTypeKind,
	name: InternedString,
}

/// The type of an enum value
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumRef(NewType);

impl EnumRef {
	/// Create a new enum
	pub fn new(db: &dyn Hir, pattern: PatternRef) -> Self {
		Self(NewType::from_pattern(db, pattern))
	}

	/// Introduce a new enum type.
	///
	/// Calling this always creates a fresh new enum type.
	/// The name is only used for pretty printing.
	pub fn introduce(db: &dyn Interner, name: impl Into<InternedString>) -> Self {
		Self(NewType::introduce(db, name))
	}

	/// Get the human readable name of this enum
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
		db.lookup_intern_newtype(self.0).name.value(db)
	}
}

/// The type of a reference to a type-inst var
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TyVarRef(NewType);

impl TyVarRef {
	/// Create a new type-inst var reference
	pub fn new(db: &dyn Hir, pattern: PatternRef) -> Self {
		Self(NewType::from_pattern(db, pattern))
	}

	/// Introduce a new type-inst variable.
	///
	/// Calling this always creates a fresh new type-inst variable.
	/// The name is only used for pretty printing.
	pub fn introduce(db: &dyn Interner, name: impl Into<InternedString>) -> Self {
		Self(NewType::introduce(db, name))
	}

	/// Get the human readable name of this type-inst variable
	pub fn pretty_print(&self, db: &dyn Interner) -> String {
		db.lookup_intern_newtype(self.0).name.value(db)
	}
}

/// The type of a type-inst variable
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TyVar {
	/// The newtype for this type-inst var
	pub ty_var: TyVarRef,
	/// Whether this type-inst var is varifiable
	pub varifiable: bool,
	/// Whether this type-inst var is enumerable
	pub enumerable: bool,
	/// Whether this type-inst var is an index type (enumerable or tuple of enumerable)
	pub indexable: bool,
}

macro_rules! type_registry {
	($struct:ident, $db:ident, $($name:ident: $value:expr),+$(,)?) => {
		/// Registry for common types
		#[derive(Clone, Debug, PartialEq, Eq)]
		pub struct $struct {
			#[allow(missing_docs)]
			pub all: Vec<Ty>,
			$(
				#[allow(missing_docs)]
				pub $name: Ty
			),+
		}

		impl $struct {
			/// Create a new type registry
			pub fn new(db: &dyn Interner) -> Self {
				let $db = db;
				let mut all = Vec::new();
				$(let $name = $value; all.push($name);)+
				Self {
					all,
					$($name),+
				}
			}
		}
	};
}

type_registry!(
	TypeRegistry,
	db,
	error: Ty::error(db),
	par_bool: Ty::par_bool(db),
	var_bool: par_bool.with_inst(db, VarType::Var).unwrap(),
	par_int: Ty::par_int(db),
	var_int: par_int.with_inst(db, VarType::Var).unwrap(),
	par_float: Ty::par_float(db),
	var_float: par_float.with_inst(db, VarType::Var).unwrap(),
	string: Ty::string(db),
	ann: Ty::ann(db),
	bottom: Ty::bottom(db),
	opt_bottom: bottom.with_opt(db, OptType::Opt),
	set_of_bottom: Ty::par_set(db, bottom).unwrap(),
	array_of_string: Ty::array(db, par_int, string).unwrap(),
	array_of_bottom: Ty::array(db, bottom, bottom).unwrap(),
);

#[cfg(test)]
mod test;
