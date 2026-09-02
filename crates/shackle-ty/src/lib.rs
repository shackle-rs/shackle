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

use std::{collections::hash_map::Entry, sync::atomic::AtomicU32};

use rustc_hash::FxHashMap;
use salsa::Database as Db;
use shackle_syntax::minizinc::pretty_print_identifier;
use shackle_utils::{InternedString, maybe_grow_stack};

mod functions;
pub use shackle_syntax::minizinc::{OptType, VarType};

pub use self::functions::*;

pub mod registry;
#[salsa::interned(debug)]
struct InternedTy<'db> {
	inner: TyData<'db>,
}

/// A type used in the type-system (as opposed to the type that is declared by the user and used in the `hir` module).
#[derive(Copy, Clone, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Ty<'db>(InternedTy<'db>);

impl<'db> Ty<'db> {
	fn new(db: &'db dyn Db, data: TyData<'db>) -> Self {
		Ty(InternedTy::new(db, data))
	}

	/// Get the underlying type data
	pub fn lookup(&self, db: &'db dyn Db) -> &'db TyData<'db> {
		let Ty(interned_ty) = self;
		interned_ty.inner(db)
	}

	/// Create an error type
	pub fn error(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Error)
	}

	/// Create a par bool
	pub fn par_bool(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Boolean(VarType::Par, OptType::NonOpt))
	}

	/// Create a par int
	pub fn par_int(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Integer(VarType::Par, OptType::NonOpt))
	}

	/// Create a par float
	pub fn par_float(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Float(VarType::Par, OptType::NonOpt))
	}

	/// Create a par enum
	pub fn par_enum(db: &'db dyn Db, e: EnumRef<'db>) -> Self {
		Ty::new(db, TyData::Enum(VarType::Par, OptType::NonOpt, e))
	}

	/// Create a par class object type
	pub fn class(db: &'db dyn Db, c: ClassRef<'db>) -> Self {
		Ty::new(db, TyData::Class(VarType::Par, OptType::NonOpt, c))
	}

	/// Create a par string
	pub fn string(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::String(OptType::NonOpt))
	}

	/// Create a par annotation type
	pub fn ann(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Annotation(OptType::NonOpt))
	}

	/// Create a par bottom type
	pub fn bottom(db: &'db dyn Db) -> Self {
		Ty::new(db, TyData::Bottom(OptType::NonOpt))
	}

	/// Create an array type if possible
	pub fn array(db: &'db dyn Db, dim: Ty<'db>, element: Ty<'db>) -> Option<Self> {
		if !dim.known_par(db) || !dim.known_occurs(db) || !dim.known_indexable(db) {
			// Invalid index type
			return None;
		}
		Some(Ty::new(
			db,
			TyData::Array {
				opt: OptType::NonOpt,
				dim,
				element,
			},
		))
	}

	/// Create a par set type if possible
	pub fn par_set(db: &'db dyn Db, element: Ty<'db>) -> Option<Self> {
		if element.known_par(db) && element.known_occurs(db) {
			Some(Ty::new(
				db,
				TyData::Set(VarType::Par, OptType::NonOpt, element),
			))
		} else {
			None
		}
	}

	/// Create a tuple type
	pub fn tuple(db: &'db dyn Db, fields: impl IntoIterator<Item = Ty<'db>>) -> Self {
		Ty::new(
			db,
			TyData::Tuple(OptType::NonOpt, fields.into_iter().collect()),
		)
	}

	/// Create a record type
	pub fn record(
		db: &'db dyn Db,
		fields: impl IntoIterator<Item = (impl Into<InternedString<'db>>, Ty<'db>)>,
	) -> Self {
		let mut fields = fields
			.into_iter()
			.map(|(i, t)| (i.into(), t))
			.collect::<Vec<_>>();
		fields.sort_by_key(|(i, _)| i.lookup(db));
		Ty::new(
			db,
			TyData::Record(OptType::NonOpt, fields.into_boxed_slice()),
		)
	}

	/// Create a function type
	pub fn function(db: &'db dyn Db, f: FunctionType<'db>) -> Self {
		Ty::new(db, TyData::Function(OptType::NonOpt, f))
	}

	/// Create a type-inst variable type
	pub fn type_inst_var(db: &'db dyn Db, t: TyVar<'db>) -> Self {
		Ty::new(db, TyData::TyVar(None, None, t))
	}

	/// Sets the inst of this type if possible.
	///
	/// Some types e.g. var strings are not possible.
	pub fn with_inst(&self, db: &'db dyn Db, inst: VarType) -> Option<Ty<'db>> {
		maybe_grow_stack(|| self.with_inst_inner(db, inst))
	}

	fn with_inst_inner(&self, db: &'db dyn Db, inst: VarType) -> Option<Ty<'db>> {
		if inst == VarType::Par {
			return Some(self.make_par(db));
		}
		let result = match self.lookup(db) {
			TyData::Boolean(_, o) => TyData::Boolean(inst, *o),
			TyData::Integer(_, o) => TyData::Integer(inst, *o),
			TyData::Float(_, o) => TyData::Float(inst, *o),
			TyData::Enum(_, o, e) => TyData::Enum(inst, *o, *e),
			TyData::Class(_, o, c) => TyData::Class(inst, *o, *c),
			TyData::Set(_, o, e) if e.known_enumerable(db) => TyData::Set(inst, *o, *e),
			TyData::Array { opt, dim, element } => TyData::Array {
				opt: *opt,
				dim: *dim,
				element: element.with_inst(db, inst)?,
			},
			TyData::Tuple(o, fs) => TyData::Tuple(
				*o,
				fs.iter()
					.map(|f| f.with_inst(db, inst))
					.collect::<Option<_>>()?,
			),
			TyData::Record(o, fs) => TyData::Record(
				*o,
				fs.iter()
					.map(|(i, f)| Some((*i, f.with_inst(db, inst)?)))
					.collect::<Option<_>>()?,
			),
			TyData::Function(_, _) if inst == VarType::Par => return Some(*self),
			TyData::TyVar(_, o, t) if t.varifiable => TyData::TyVar(Some(inst), *o, t.clone()),
			TyData::Bottom(_) | TyData::Error => return Some(*self),
			_ => return None,
		};
		Some(Ty::new(db, result))
	}

	/// Make this type var if possible.
	pub fn make_var(&self, db: &'db dyn Db) -> Option<Ty<'db>> {
		self.with_inst(db, VarType::Var)
	}

	/// Returns the fixed version of this type
	pub fn make_par(&self, db: &'db dyn Db) -> Ty<'db> {
		maybe_grow_stack(|| self.make_par_inner(db))
	}

	fn make_par_inner(&self, db: &'db dyn Db) -> Ty<'db> {
		Ty::new(
			db,
			match self.lookup(db) {
				TyData::Boolean(_, o) => TyData::Boolean(VarType::Par, *o),
				TyData::Integer(_, o) => TyData::Integer(VarType::Par, *o),
				TyData::Float(_, o) => TyData::Float(VarType::Par, *o),
				TyData::Enum(_, o, e) => TyData::Enum(VarType::Par, *o, *e),
				TyData::Class(_, o, c) => TyData::Class(VarType::Par, *o, *c),
				TyData::Array { opt, dim, element } => TyData::Array {
					opt: *opt,
					dim: *dim,
					element: element.make_par(db),
				},
				TyData::Set(_, o, e) => TyData::Set(VarType::Par, *o, *e),
				TyData::Tuple(o, fs) => {
					TyData::Tuple(*o, fs.iter().map(|f| f.make_par(db)).collect())
				}
				TyData::Record(o, fs) => {
					TyData::Record(*o, fs.iter().map(|(i, f)| (*i, f.make_par(db))).collect())
				}
				TyData::TyVar(_, o, t) => TyData::TyVar(Some(VarType::Par), *o, t.clone()),
				_ => return *self,
			},
		)
	}

	/// Sets the optionality of this type if possible.
	pub fn with_opt(&self, db: &'db dyn Db, opt: OptType) -> Ty<'db> {
		Ty::new(
			db,
			match self.lookup(db) {
				TyData::Boolean(i, _) => TyData::Boolean(*i, opt),
				TyData::Integer(i, _) => TyData::Integer(*i, opt),
				TyData::Float(i, _) => TyData::Float(*i, opt),
				TyData::Enum(i, _, e) => TyData::Enum(*i, opt, *e),
				TyData::Class(i, _, c) => TyData::Class(*i, opt, *c),
				TyData::String(_) => TyData::String(opt),
				TyData::Annotation(_) => TyData::Annotation(opt),
				TyData::Bottom(_) => TyData::Bottom(opt),
				TyData::Array { dim, element, .. } => TyData::Array {
					opt,
					dim: *dim,
					element: *element,
				},
				TyData::Set(i, _, e) => TyData::Set(*i, opt, *e),
				TyData::Tuple(_, fs) => TyData::Tuple(opt, fs.clone()),
				TyData::Record(_, fs) => TyData::Record(opt, fs.clone()),
				TyData::Function(_, f) => TyData::Function(opt, f.clone()),
				TyData::TyVar(i, _, t) => TyData::TyVar(*i, Some(opt), t.clone()),
				TyData::Error => return *self,
			},
		)
	}

	/// Make this type optional
	pub fn make_opt(&self, db: &'db dyn Db) -> Ty<'db> {
		self.with_opt(db, OptType::Opt)
	}

	/// Make this type non-optional
	pub fn make_occurs(&self, db: &'db dyn Db) -> Ty<'db> {
		self.with_opt(db, OptType::NonOpt)
	}

	/// Whether this type-inst is known to be completely par.
	pub fn known_par(&self, db: &'db dyn Db) -> bool {
		let mut todo = vec![*self];
		while let Some(ty) = todo.pop() {
			match ty.lookup(db) {
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
				| TyData::Class(VarType::Par, _, _)
				| TyData::TyVar(Some(VarType::Par), _, _) => (),
				TyData::Array { dim, element, .. } => {
					todo.push(*dim);
					todo.push(*element);
				}
				TyData::Tuple(_, fs) => todo.extend(fs.iter().copied()),
				TyData::Record(_, fs) => todo.extend(fs.iter().map(|(_, t)| *t)),
				_ => return false,
			}
		}
		true
	}

	/// Whether this type-inst is known to be non-optional.
	pub fn known_occurs(&self, db: &'db dyn Db) -> bool {
		matches!(
			self.lookup(db),
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
				| TyData::Class(_, OptType::NonOpt, _)
				| TyData::TyVar(_, Some(OptType::NonOpt), _)
				| TyData::Array {
					opt: OptType::NonOpt,
					..
				} | TyData::Tuple(OptType::NonOpt, _)
				| TyData::Record(OptType::NonOpt, _)
		)
	}

	/// Whether this type-inst can definitely be made var
	pub fn known_varifiable(&self, db: &'db dyn Db) -> bool {
		self.with_inst(db, VarType::Var).is_some()
	}

	/// Whether this type-inst is enumerable ($$E)
	pub fn known_enumerable(&self, db: &'db dyn Db) -> bool {
		match self.lookup(db) {
			TyData::Error
			| TyData::Bottom(OptType::NonOpt)
			| TyData::Boolean(_, OptType::NonOpt)
			| TyData::Integer(_, OptType::NonOpt)
			| TyData::Class(_, OptType::NonOpt, _)
			| TyData::Enum(_, OptType::NonOpt, _) => true,
			TyData::TyVar(_, Some(OptType::NonOpt), t) => t.enumerable,
			_ => false,
		}
	}

	/// Whether this type-inst is a class object type
	pub fn is_class(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Class(_, _, _))
	}

	/// Get the class this type refers to, looking through sets
	pub fn class_type(&self, db: &'db dyn Db) -> Option<ClassRef<'db>> {
		match self.lookup(db) {
			TyData::Class(_, _, c) => Some(*c),
			TyData::Set(_, _, t) => t.class_type(db),
			_ => None,
		}
	}

	/// Whether this type-inst is indexable ($T used in an array dimension)
	pub fn known_indexable(&self, db: &'db dyn Db) -> bool {
		self.known_enumerable(db)
			|| match self.lookup(db) {
				TyData::Tuple(OptType::NonOpt, fs) => fs.iter().all(|f| f.known_enumerable(db)),
				TyData::TyVar(_, Some(OptType::NonOpt), t) => t.indexable,
				_ => false,
			}
	}

	/// Whether this type-inst has a default value (allowing omission of else branch in ITE)
	pub fn has_default_value(&self, db: &'db dyn Db) -> bool {
		let mut todo = vec![*self];
		while let Some(ty) = todo.pop() {
			match ty.lookup(db) {
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
				| TyData::Error => (),
				TyData::Tuple(_, fs) => todo.extend(fs.iter().copied()),
				TyData::Record(_, fs) => todo.extend(fs.iter().map(|(_, t)| *t)),
				_ => return false,
			}
		}
		true
	}

	/// Whether this type-inst contains a type-inst variable
	pub fn contains_type_inst_var(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db)
			.any(|(_, td)| matches!(td, TyData::TyVar(_, _, _)))
	}

	/// Whether this type-inst contains an operation type
	pub fn contains_function(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db)
			.any(|(_, td)| matches!(td, TyData::Function(_, _)))
	}

	/// Whether this type-inst contains an optional type
	pub fn contains_opt(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db).any(|(_, td)| {
			matches!(
				td,
				TyData::Boolean(_, OptType::Opt)
					| TyData::Integer(_, OptType::Opt)
					| TyData::Float(_, OptType::Opt)
					| TyData::Enum(_, OptType::Opt, _)
					| TyData::String(OptType::Opt)
					| TyData::Annotation(OptType::Opt)
					| TyData::Bottom(OptType::Opt)
					| TyData::Function(OptType::Opt, _)
					| TyData::Set(_, OptType::Opt, _)
					| TyData::TyVar(_, Some(OptType::Opt), _)
					| TyData::Array {
						opt: OptType::Opt,
						..
					} | TyData::Tuple(OptType::Opt, _)
					| TyData::Record(OptType::Opt, _)
			)
		})
	}

	/// Whether this type inst contains something that is par
	pub fn contains_par(&self, db: &'db dyn Db) -> bool {
		let mut todo = vec![*self];
		while let Some(ty) = todo.pop() {
			match ty.lookup(db) {
				TyData::Array { element, .. } => {
					todo.push(*element);
				}
				TyData::Tuple(_, fs) => todo.extend(fs.iter().copied()),
				TyData::Record(_, fs) => todo.extend(fs.iter().map(|(_, t)| *t)),
				_ => {
					if ty.known_par(db) {
						return true;
					}
				}
			}
		}
		false
	}

	/// Whether this type-inst contains a var type
	pub fn contains_var(&self, db: &'db dyn Db) -> bool {
		self.walk(db).any(|ty| ty.inst(db) == Some(VarType::Var))
	}

	/// Whether this type-inst contains an error.
	pub fn contains_error(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db)
			.any(|(_, td)| matches!(td, TyData::Error))
	}

	/// Whether this type-inst contains bottom.
	pub fn contains_bottom(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db)
			.any(|(_, td)| matches!(td, TyData::Bottom(_)))
	}

	/// Whether this type-inst contains an enum.
	pub fn contains_enum(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db)
			.any(|(_, td)| matches!(td, TyData::Enum(_, _, _)))
	}

	/// Whether this type contains a type that will be erased
	pub fn contains_erased_type(&self, db: &'db dyn Db) -> bool {
		self.walk_data(db).any(|(_, td)| {
			matches!(
				td,
				TyData::Annotation(OptType::Opt)
					| TyData::Array {
						opt: OptType::Opt,
						..
					} | TyData::Boolean(_, OptType::Opt)
					| TyData::Bottom(OptType::Opt)
					| TyData::Enum(_, _, _)
					| TyData::Float(_, OptType::Opt)
					| TyData::Function(OptType::Opt, _)
					| TyData::Integer(_, OptType::Opt)
					| TyData::Record(_, _)
					| TyData::Set(_, OptType::Opt, _)
					| TyData::String(OptType::Opt)
					| TyData::Tuple(OptType::Opt, _)
					| TyData::TyVar(_, Some(OptType::Opt), _)
			)
		})
	}

	/// Convert any enum types in this type to ints (for erasure)
	pub fn enum2int(&self, db: &'db dyn Db) -> Ty<'db> {
		Ty::new(
			db,
			match self.lookup(db) {
				TyData::Enum(i, o, _) => TyData::Integer(*i, *o),
				TyData::Set(i, o, ty) => TyData::Set(*i, *o, ty.enum2int(db)),
				TyData::Array { opt, dim, element } => TyData::Array {
					opt: *opt,
					dim: dim.enum2int(db),
					element: element.enum2int(db),
				},
				TyData::Tuple(o, fs) => {
					TyData::Tuple(*o, fs.iter().map(|f| f.enum2int(db)).collect())
				}
				TyData::Record(o, fs) => {
					TyData::Record(*o, fs.iter().map(|(i, f)| (*i, f.enum2int(db))).collect())
				}
				TyData::Function(o, f) => TyData::Function(
					*o,
					FunctionType {
						return_type: f.return_type.enum2int(db),
						params: f.params.iter().map(|p| p.enum2int(db)).collect(),
					},
				),
				_ => return *self,
			},
		)
	}

	/// Walk over the `Ty`s in this `Ty`
	pub fn walk(&'db self, db: &'db dyn Db) -> impl 'db + Iterator<Item = Ty<'db>> {
		self.walk_data(db).map(|(ty, _)| ty)
	}

	/// Walk the `Ty`s and their data in this `Ty`
	pub fn walk_data(
		&'db self,
		db: &'db dyn Db,
	) -> impl 'db + Iterator<Item = (Ty<'db>, TyData<'db>)> {
		let mut todo = vec![*self];
		std::iter::from_fn(move || {
			let ty = todo.pop()?;
			let td = ty.lookup(db).clone();
			match &td {
				TyData::Array { dim, element, .. } => {
					todo.push(*dim);
					todo.push(*element);
				}
				TyData::Set(_, _, e) => todo.push(*e),
				TyData::Tuple(_, fs) => todo.extend(fs.iter().copied()),
				TyData::Record(_, fs) => todo.extend(fs.iter().map(|(_, t)| *t)),
				TyData::Function(_, ft) => {
					todo.extend(ft.params.iter().copied());
					todo.push(ft.return_type);
				}
				_ => (),
			}
			Some((ty, td))
		})
	}

	/// The inst of this type (or `None` if the inst cannot be determined)
	pub fn inst(&self, db: &'db dyn Db) -> Option<VarType> {
		match self.lookup(db) {
			TyData::Boolean(inst, _)
			| TyData::Integer(inst, _)
			| TyData::Float(inst, _)
			| TyData::Enum(inst, _, _)
			| TyData::Set(inst, _, _)
			| TyData::Class(inst, _, _)
			| TyData::TyVar(Some(inst), _, _) => Some(*inst),

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
	pub fn opt(&self, db: &'db dyn Db) -> Option<OptType> {
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
			| TyData::Class(_, opt, _)
			| TyData::TyVar(_, Some(opt), _)
			| TyData::Array { opt, .. }
			| TyData::Tuple(opt, _)
			| TyData::Record(opt, _) => Some(*opt),
			TyData::Error => Some(OptType::NonOpt),
			_ => None,
		}
	}

	/// Return whether or not this is an array
	pub fn is_array(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Array { .. })
	}

	/// Whether or not this type is a set
	pub fn is_set(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Set(_, _, _))
	}

	/// Whether or not this type is a tuple
	pub fn is_tuple(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Tuple(_, _))
	}

	/// Whether or not this type is a record
	pub fn is_record(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Record(_, _))
	}

	/// Whether or not this type is a var set
	pub fn is_var_set(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Set(VarType::Var, _, _))
	}

	/// Whether or not this type is a function
	pub fn is_function(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Function(_, _))
	}

	/// Whether or not this type is a boolean type
	pub fn is_bool(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Boolean(_, _))
	}

	/// Whether or not this type is an integer type
	pub fn is_int(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Integer(_, _))
	}

	/// Whether or not this type is an enum type
	pub fn is_enum(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Enum(_, _, _))
	}

	/// Whether or not this type is a float type
	pub fn is_float(&self, db: &'db dyn Db) -> bool {
		matches!(self.lookup(db), TyData::Float(_, _))
	}

	/// Returns the number of array dimensions if known
	pub fn dims(&self, db: &'db dyn Db) -> Option<usize> {
		match self.lookup(db) {
			TyData::Array { dim, .. } => match dim.lookup(db) {
				TyData::Tuple(_, fs) => Some(fs.len()),
				TyData::TyVar(_, _, tv) if !tv.enumerable => None,
				_ => Some(1),
			},
			_ => None,
		}
	}

	/// Get the type of the dimensions if this is an array
	pub fn dim_ty(&self, db: &'db dyn Db) -> Option<Ty<'db>> {
		match self.lookup(db) {
			TyData::Array { dim, .. } => Some(*dim),
			_ => None,
		}
	}

	/// Returns the number of fields if this is a tuple/record type
	pub fn field_len(&self, db: &'db dyn Db) -> Option<usize> {
		match self.lookup(db) {
			TyData::Tuple(_, fs) => Some(fs.len()),
			TyData::Record(_, fs) => Some(fs.len()),
			_ => None,
		}
	}

	/// Get the field types if this is a tuple/record type
	pub fn fields(&self, db: &'db dyn Db) -> Option<Vec<Ty<'db>>> {
		match self.lookup(db) {
			TyData::Tuple(_, fs) => Some(fs.to_vec()),
			TyData::Record(_, fs) => Some(fs.iter().map(|(_, f)| *f).collect::<Vec<_>>()),
			_ => None,
		}
	}

	/// Get the field types if this is a record type
	pub fn record_fields(&self, db: &'db dyn Db) -> Option<Vec<(InternedString<'db>, Ty<'db>)>> {
		match self.lookup(db) {
			TyData::Record(_, fs) => Some(fs.to_vec()),
			_ => None,
		}
	}

	/// Get the element type for array/set types (will be par for var sets)
	pub fn elem_ty(&self, db: &'db dyn Db) -> Option<Ty<'db>> {
		match self.lookup(db) {
			TyData::Array { element, .. } => Some(*element),
			TyData::Set(_, _, e) => Some(*e),
			_ => None,
		}
	}

	/// Get the enum ref for this type if it is an enum
	pub fn enum_ty(&self, db: &'db dyn Db) -> Option<EnumRef<'db>> {
		match self.lookup(db) {
			TyData::Enum(_, _, e) => Some(*e),
			TyData::Array { element, .. } => element.enum_ty(db),
			TyData::Set(_, _, e) => e.enum_ty(db),
			_ => None,
		}
	}

	/// Get the type-inst var for this type if it is one
	pub fn ty_var(&self, db: &'db dyn Db) -> Option<TyVar<'db>> {
		if let TyData::TyVar(_, _, tv) = self.lookup(db) {
			Some(tv.clone())
		} else {
			None
		}
	}

	/// Get the types of the parameters if this is a function type
	pub fn function_params(&self, db: &'db dyn Db) -> Option<Vec<Ty<'db>>> {
		match self.lookup(db) {
			TyData::Function(_, ft) => Some(ft.params.to_vec()),
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
		db: &'db dyn Db,
		ts: impl IntoIterator<Item = Ty<'db>>,
	) -> Option<Ty<'db>> {
		maybe_grow_stack(|| Self::most_specific_supertype_inner(db, ts))
	}

	fn most_specific_supertype_inner(
		db: &'db dyn Db,
		ts: impl IntoIterator<Item = Ty<'db>>,
	) -> Option<Ty<'db>> {
		ts.into_iter()
			.map(Some)
			.reduce(|a, b| {
				// E.g. t1 = var opt bool, t2 = var int, then super type is var opt int
				let result = match (a?.lookup(db), b?.lookup(db)) {
					(TyData::Error, _) | (_, TyData::Error) => TyData::Error,
					(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2)) => {
						TyData::Boolean((*i1).max(*i2), (*o1).max(*o2))
					}
					(TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Integer(i1, o1), TyData::Integer(i2, o2)) => {
						TyData::Integer((*i1).max(*i2), (*o1).max(*o2))
					}
					(TyData::Boolean(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Integer(i1, o1))
					| (TyData::Float(i2, o2), TyData::Float(i1, o1)) => {
						TyData::Float((*i1).max(*i2), (*o1).max(*o2))
					}
					(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) if e1 == e2 => {
						TyData::Enum((*i1).max(*i2), (*o1).max(*o2), *e1)
					}
					(TyData::Class(i1, o1, c1), TyData::Class(i2, o2, c2))
						if c1.is_subclass_of(db, c2) =>
					{
						TyData::Class((*i1).max(*i2), (*o1).max(*o2), *c2)
					}
					(TyData::Class(i1, o1, c1), TyData::Class(i2, o2, c2))
						if c2.is_subclass_of(db, c1) =>
					{
						TyData::Class((*i1).max(*i2), (*o1).max(*o2), *c1)
					}
					(TyData::String(o1), TyData::String(o2)) => TyData::String((*o1).max(*o2)),
					(TyData::Annotation(o1), TyData::Annotation(o2)) => {
						TyData::Annotation((*o1).max(*o2))
					}
					(TyData::Bottom(o1), TyData::Bottom(o2)) => TyData::Bottom((*o1).max(*o2)),
					(TyData::Bottom(o1), TyData::Boolean(i, o2))
					| (TyData::Boolean(i, o2), TyData::Bottom(o1)) => TyData::Boolean(*i, (*o1).max(*o2)),
					(TyData::Bottom(o1), TyData::Integer(i, o2))
					| (TyData::Integer(i, o2), TyData::Bottom(o1)) => TyData::Integer(*i, (*o1).max(*o2)),
					(TyData::Bottom(o1), TyData::Float(i, o2))
					| (TyData::Float(i, o2), TyData::Bottom(o1)) => TyData::Float(*i, (*o1).max(*o2)),
					(TyData::Bottom(o1), TyData::Enum(i, o2, e))
					| (TyData::Enum(i, o2, e), TyData::Bottom(o1)) => TyData::Enum(*i, (*o1).max(*o2), *e),
					(TyData::Bottom(o1), TyData::Class(i, o2, c))
					| (TyData::Class(i, o2, c), TyData::Bottom(o1)) => TyData::Class(*i, (*o1).max(*o2), *c),
					(TyData::Bottom(o1), TyData::String(o2))
					| (TyData::String(o2), TyData::Bottom(o1)) => TyData::String((*o1).max(*o2)),
					(TyData::Bottom(o1), TyData::Annotation(o2))
					| (TyData::Annotation(o2), TyData::Bottom(o1)) => TyData::Annotation((*o1).max(*o2)),
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
						opt: (*o1).max(*o2),
						dim: *dim,
						element: *element,
					},
					(TyData::Bottom(o1), TyData::Set(inst, o2, element))
					| (TyData::Set(inst, o2, element), TyData::Bottom(o1)) => {
						TyData::Set(*inst, (*o1).max(*o2), *element)
					}
					(TyData::Bottom(o1), TyData::Tuple(o2, fields))
					| (TyData::Tuple(o2, fields), TyData::Bottom(o1)) => {
						TyData::Tuple((*o1).max(*o2), fields.clone())
					}
					(TyData::Bottom(o1), TyData::Record(o2, fields))
					| (TyData::Record(o2, fields), TyData::Bottom(o1)) => {
						TyData::Record((*o1).max(*o2), fields.clone())
					}
					(TyData::Bottom(o1), TyData::Function(o2, f))
					| (TyData::Function(o2, f), TyData::Bottom(o1)) => TyData::Function((*o1).max(*o2), f.clone()),
					(TyData::Bottom(o1), TyData::TyVar(inst, o2, tv))
					| (TyData::TyVar(inst, o2, tv), TyData::Bottom(o1)) => TyData::TyVar(
						*inst,
						if *o1 == OptType::Opt {
							Some(OptType::Opt)
						} else {
							(*o2).map(|o2| (*o1).max(o2))
						},
						tv.clone(),
					),
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
						opt: (*o1).max(*o2),
						dim: Ty::most_specific_supertype(db, [*d1, *d2])?,
						element: Ty::most_specific_supertype(db, [*e1, *e2])?,
					},
					(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => TyData::Set(
						(*i1).max(*i2),
						(*o1).max(*o2),
						Ty::most_specific_supertype(db, [*e1, *e2])?,
					),
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
						TyData::Tuple((*o1).max(*o2), {
							if f1.len() != f2.len() {
								return None;
							}
							f1.iter()
								.zip(f2.iter())
								.map(|(t1, t2)| Ty::most_specific_supertype(db, [*t1, *t2]))
								.collect::<Option<_>>()?
						})
					}
					(TyData::Record(o1, f1), TyData::Record(o2, f2)) => TyData::Record(
						(*o1).max(*o2),
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
						(*o1).max(*o2),
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
							match (*i1, *i2) {
								(Some(VarType::Var), _) | (_, Some(VarType::Var)) => {
									Some(VarType::Var)
								}
								(Some(VarType::Par), Some(VarType::Par)) => Some(VarType::Par),
								_ => None,
							},
							match (*o1, *o2) {
								(Some(OptType::Opt), _) | (_, Some(OptType::Opt)) => {
									Some(OptType::Opt)
								}
								(Some(OptType::NonOpt), Some(OptType::NonOpt)) => {
									Some(OptType::NonOpt)
								}
								_ => None,
							},
							t1.clone(),
						)
					}
					_ => {
						return None;
					}
				};
				Some(Ty::new(db, result))
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
	pub fn most_general_subtype(
		db: &'db dyn Db,
		ts: impl IntoIterator<Item = Ty<'db>>,
	) -> Option<Ty<'db>> {
		maybe_grow_stack(|| Self::most_general_subtype_inner(db, ts))
	}

	fn most_general_subtype_inner(
		db: &'db dyn Db,
		ts: impl IntoIterator<Item = Ty<'db>>,
	) -> Option<Ty<'db>> {
		ts.into_iter()
			.map(Some)
			.reduce(|a, b| {
				let result = match (a?.lookup(db), b?.lookup(db)) {
					(TyData::Error, _) | (_, TyData::Error) => TyData::Error,
					(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2))
					| (TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i2, o2), TyData::Boolean(i1, o1))
					| (TyData::Boolean(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Boolean(i1, o1)) => {
						TyData::Boolean((*i1).min(*i2), (*o1).min(*o2))
					}
					(TyData::Integer(i1, o1), TyData::Integer(i2, o2))
					| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
					| (TyData::Float(i2, o2), TyData::Integer(i1, o1)) => {
						TyData::Integer((*i1).min(*i2), (*o1).min(*o2))
					}
					(TyData::Float(i1, o1), TyData::Float(i2, o2)) => {
						TyData::Float((*i1).min(*i2), (*o1).min(*o2))
					}
					(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) if e1 == e2 => {
						TyData::Enum((*i1).min(*i2), (*o1).min(*o2), *e1)
					}
					(TyData::Class(i1, o1, c1), TyData::Class(i2, o2, c2))
						if c1.is_subclass_of(db, c2) =>
					{
						TyData::Class((*i1).min(*i2), (*o1).min(*o2), *c1)
					}
					(TyData::Class(i1, o1, c1), TyData::Class(i2, o2, c2))
						if c2.is_subclass_of(db, c1) =>
					{
						TyData::Class((*i1).min(*i2), (*o1).min(*o2), *c2)
					}
					(TyData::String(o1), TyData::String(o2)) => TyData::String((*o1).min(*o2)),
					(TyData::Annotation(o1), TyData::Annotation(o2)) => {
						TyData::Annotation((*o1).min(*o2))
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
					| (TyData::Bottom(o1), TyData::Class(_, o2, _))
					| (TyData::Class(_, o2, _), TyData::Bottom(o1))
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
					| (TyData::Function(o2, _), TyData::Bottom(o1)) => TyData::Bottom((*o1).min(*o2)),
					(TyData::Bottom(o1), TyData::TyVar(_, o2, _))
					| (TyData::TyVar(_, o2, _), TyData::Bottom(o1)) => {
						TyData::Bottom((*o1).min((*o2).unwrap_or(OptType::NonOpt)))
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
						opt: (*o1).min(*o2),
						dim: Ty::most_general_subtype(db, [*d1, *d2])?,
						element: Ty::most_general_subtype(db, [*e1, *e2])?,
					},
					(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => TyData::Set(
						(*i1).min(*i2),
						(*o1).min(*o2),
						Ty::most_general_subtype(db, [*e1, *e2])?,
					),
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
						TyData::Tuple((*o1).max(*o2), {
							if f1.len() != f2.len() {
								return None;
							}
							f1.iter()
								.zip(f2.iter())
								.map(|(t1, t2)| Ty::most_general_subtype(db, [*t1, *t2]))
								.collect::<Option<_>>()?
						})
					}
					(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
						// Union and subtype fields
						// e.g. subtype of `record(int: a, bool: b)` and `record(bool: a)` is `record(bool: a, bool b)`
						let mut fields: FxHashMap<InternedString, (usize, Ty)> =
							FxHashMap::default();
						for (idx, (name, t)) in f1.iter().chain(f2.iter()).enumerate() {
							match fields.entry(*name) {
								Entry::Occupied(mut e) => {
									let (_, current_ty) = e.get_mut();
									*current_ty = Ty::most_general_subtype(db, [*current_ty, *t])?
								}
								Entry::Vacant(e) => {
									let _ = e.insert((idx, *t));
								}
							}
						}
						let mut result = fields.into_iter().collect::<Vec<_>>();
						result.sort_by_key(|(_, (i, _))| *i);
						TyData::Record(
							(*o1).max(*o2),
							result
								.into_iter()
								.map(|(name, (_, ty))| (name, ty))
								.collect(),
						)
					}
					(TyData::Function(o1, f1), TyData::Function(o2, f2)) => TyData::Function(
						(*o1).max(*o2),
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
							match (*i1, *i2) {
								(Some(VarType::Par), _) | (_, Some(VarType::Par)) => {
									Some(VarType::Par)
								}
								(Some(VarType::Var), Some(VarType::Var)) => Some(VarType::Var),
								_ => None,
							},
							match (*o1, *o2) {
								(Some(OptType::NonOpt), _) | (_, Some(OptType::NonOpt)) => {
									Some(OptType::NonOpt)
								}
								(Some(OptType::Opt), Some(OptType::Opt)) => Some(OptType::Opt),
								_ => None,
							},
							t1.clone(),
						)
					}
					_ => {
						return None;
					}
				};
				Some(Ty::new(db, result))
			})
			.flatten()
	}

	/// Whether this type is a subtype of `other`.
	///
	/// Note that e.g. `int` is not a subtype of `any $T` since $T may be bound to an incompatible
	/// type-inst (e.g. a set type).
	pub fn is_subtype_of(&self, db: &'db dyn Db, other: Ty<'db>) -> bool {
		if *self == other {
			return true;
		}

		let mut todo = vec![(*self, other)];
		while let Some((a, b)) = todo.pop() {
			match (a.lookup(db).clone(), b.lookup(db).clone()) {
				(TyData::Error, _) | (_, TyData::Error) => (),
				// Scalar coercions
				(TyData::Boolean(i1, o1), TyData::Boolean(i2, o2))
				| (TyData::Boolean(i1, o1), TyData::Integer(i2, o2))
				| (TyData::Boolean(i1, o1), TyData::Float(i2, o2))
				| (TyData::Integer(i1, o1), TyData::Integer(i2, o2))
				| (TyData::Integer(i1, o1), TyData::Float(i2, o2))
				| (TyData::Float(i1, o1), TyData::Float(i2, o2)) => {
					if i1 != VarType::Par && i1 != i2 || o1 != OptType::NonOpt && o1 != o2 {
						return false;
					}
				}
				(TyData::Enum(i1, o1, e1), TyData::Enum(i2, o2, e2)) => {
					if i1 != VarType::Par && i1 != i2
						|| o1 != OptType::NonOpt && o1 != o2
						|| e1 != e2
					{
						return false;
					}
				}
				(TyData::Class(i1, o1, c1), TyData::Class(i2, o2, c2)) => {
					if i1 != VarType::Par && i1 != i2 || o1 != OptType::NonOpt && o1 != o2 {
						return false;
					}
					return c1.is_subclass_of(db, &c2);
				}
				(TyData::Bottom(o1), TyData::Bottom(o2))
				| (TyData::Bottom(o1), TyData::Boolean(_, o2))
				| (TyData::Bottom(o1), TyData::Integer(_, o2))
				| (TyData::Bottom(o1), TyData::Float(_, o2))
				| (TyData::Bottom(o1), TyData::Enum(_, o2, _))
				| (TyData::Bottom(o1), TyData::Class(_, o2, _))
				| (TyData::Bottom(o1), TyData::String(o2))
				| (TyData::Bottom(o1), TyData::Annotation(o2))
				| (TyData::Bottom(o1), TyData::Array { opt: o2, .. })
				| (TyData::Bottom(o1), TyData::Set(_, o2, _))
				| (TyData::Bottom(o1), TyData::Tuple(o2, _))
				| (TyData::Bottom(o1), TyData::Record(o2, _))
				| (TyData::Bottom(o1), TyData::Function(o2, _))
				| (TyData::String(o1), TyData::String(o2))
				| (TyData::Annotation(o1), TyData::Annotation(o2)) => {
					if o1 != OptType::NonOpt && o1 != o2 {
						return false;
					}
				}
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
					if o1 != OptType::NonOpt && o1 != o2 {
						return false;
					}
					todo.push((d1, d2));
					todo.push((e1, e2));
				}
				(TyData::Set(i1, o1, e1), TyData::Set(i2, o2, e2)) => {
					if o1 != OptType::NonOpt && o1 != o2 || i1 != VarType::Par && i1 != i2 {
						return false;
					}
					todo.push((e1, e2));
				}
				(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
					if o1 != OptType::NonOpt && o1 != o2 || f1.len() != f2.len() {
						return false;
					}
					todo.extend(f1.iter().copied().zip(f2.iter().copied()));
				}
				(TyData::Record(o1, f1), TyData::Record(o2, f2)) => {
					if o1 != OptType::NonOpt && o1 != o2
						|| f1.len() != f2.len()
						|| !f1.iter().zip(f2.iter()).all(|((i1, _), (i2, _))| i1 == i2)
					{
						return false;
					}
					todo.extend(
						f1.iter()
							.zip(f2.iter())
							.map(|((_, t1), (_, t2))| (*t1, *t2)),
					);
				}
				// Function coercion
				(TyData::Function(o1, f1), TyData::Function(o2, f2)) => {
					if o1 != OptType::NonOpt && o1 != o2 || !f1.is_subtype_of(db, &f2) {
						return false;
					}
				}
				// Type-inst var coercion (par T -> var T, par T -> T, T -> var T)
				(TyData::TyVar(i1, o1, t1), TyData::TyVar(i2, o2, t2)) => {
					if (i1 != i2 && i1 != Some(VarType::Par) && i2 != Some(VarType::Var))
						|| (o1 != o2 && o1 != Some(OptType::NonOpt) && o2 != Some(OptType::Opt))
						|| t1 != t2
					{
						return false;
					}
				}
				(TyData::Bottom(o1), TyData::TyVar(_, o2, _)) => {
					if o1 != OptType::NonOpt && Some(o1) != o2 {
						return false;
					}
				}
				_ => return false,
			}
		}
		true
	}

	/// Instantiate the given type-inst variables with the given types from `instantiations` in this type.
	///
	/// Panics if this is not possible.
	pub fn instantiate_ty_vars(
		&self,
		db: &'db dyn Db,
		ty_vars: &TyParamInstantiations<'db>,
	) -> Ty<'db> {
		maybe_grow_stack(|| self.instantiate_ty_vars_inner(db, ty_vars))
	}

	fn instantiate_ty_vars_inner(
		&self,
		db: &'db dyn Db,
		ty_vars: &TyParamInstantiations<'db>,
	) -> Ty<'db> {
		match self.lookup(db) {
			TyData::TyVar(i, o, t) if ty_vars.contains_key(&t.ty_var) => {
				let mut ty = ty_vars[&t.ty_var];
				if let Some(inst) = *i {
					ty = ty
						.with_inst(db, inst)
						.expect("Type-inst is incompatible with type-inst var");
				}
				if let Some(opt) = *o {
					ty = ty.with_opt(db, opt);
				}
				ty
			}
			TyData::Array { opt, dim, element } => Ty::new(
				db,
				TyData::Array {
					opt: *opt,
					dim: dim.instantiate_ty_vars(db, ty_vars),
					element: element.instantiate_ty_vars(db, ty_vars),
				},
			),
			TyData::Set(i, o, t) => {
				Ty::new(db, TyData::Set(*i, *o, t.instantiate_ty_vars(db, ty_vars)))
			}
			TyData::Tuple(o, fs) => Ty::new(
				db,
				TyData::Tuple(
					*o,
					fs.iter()
						.map(|f| f.instantiate_ty_vars(db, ty_vars))
						.collect(),
				),
			),
			TyData::Record(o, fs) => Ty::new(
				db,
				TyData::Record(
					*o,
					fs.iter()
						.map(|(i, f)| (*i, f.instantiate_ty_vars(db, ty_vars)))
						.collect(),
				),
			),
			_ => *self,
		}
	}

	/// Get human readable type name
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		maybe_grow_stack(|| self.pretty_print_inner(db))
	}

	fn pretty_print_inner(&self, db: &'db dyn Db) -> String {
		match self.lookup(db) {
			TyData::Boolean(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print())
				.chain(["bool".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Integer(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print())
				.chain(["int".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Float(i, o) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print())
				.chain(["float".to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Enum(i, o, e) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print())
				.chain([e.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Class(i, o, c) => i
				.pretty_print()
				.into_iter()
				.chain(o.pretty_print())
				.chain([c.pretty_print(db)])
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
				.chain(o.pretty_print())
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
			TyData::Record(o, fs) => {
				let mut fields = fs.to_vec();
				fields.sort_by_key(|(i, _)| i.lookup(db));
				o.pretty_print()
					.into_iter()
					.chain([format!(
						"record({})",
						fields
							.iter()
							.map(|(i, f)| format!("{}: {}", f.pretty_print(db), i.lookup(db)))
							.collect::<Vec<_>>()
							.join(", ")
					)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			TyData::Function(o, f) => o
				.pretty_print()
				.into_iter()
				.chain([f.pretty_print(db)])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::TyVar(None, None, t) => format!("any {}", t.ty_var.pretty_print(db)),
			TyData::TyVar(i, o, t) => i
				.map(|i| i.pretty_print())
				.unwrap_or_else(|| Some("anyvar".to_owned()))
				.into_iter()
				.chain(
					o.map(|o| o.pretty_print())
						.unwrap_or_else(|| Some("anyopt".to_owned())),
				)
				.chain([t.ty_var.pretty_print(db).to_owned()])
				.collect::<Vec<_>>()
				.join(" "),
			TyData::Error => "error".to_owned(),
		}
	}

	/// Pretty print as if type was in the dimension position in an array type
	pub fn pretty_print_as_dims(&self, db: &'db dyn Db) -> String {
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

impl<'db> std::fmt::Debug for Ty<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		salsa::with_attached_database(|db| write!(f, "{}", self.pretty_print(db)))
			.unwrap_or_else(|| write!(f, "<could not read type>"))
	}
}

impl<'db> std::fmt::Display for Ty<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		salsa::with_attached_database(|db| write!(f, "{}", self.pretty_print(db)))
			.unwrap_or_else(|| write!(f, "<could not read type>"))
	}
}

/// A type used in the type-system (as opposed to the type that is declared by the user and used in the `hir` module).
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum TyData<'db> {
	/// Boolean scalar
	Boolean(VarType, OptType),
	/// Integer scalar
	Integer(VarType, OptType),
	/// Float scalar
	Float(VarType, OptType),
	/// Enumerated type scalar
	Enum(VarType, OptType, EnumRef<'db>),
	/// Object of a class type
	Class(VarType, OptType, ClassRef<'db>),

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
		dim: Ty<'db>,
		/// Type of the element
		element: Ty<'db>,
	},
	/// Set type
	Set(VarType, OptType, Ty<'db>),
	/// Tuple type
	Tuple(OptType, Box<[Ty<'db>]>),
	/// Record type
	Record(OptType, Box<[(InternedString<'db>, Ty<'db>)]>),

	/// Type of a function
	Function(OptType, FunctionType<'db>),

	/// Type inst variable and modifiers to apply to the substituted type-inst
	TyVar(Option<VarType>, Option<OptType>, TyVar<'db>),

	/// Indicates a type error but allows typing to continue
	Error,
}

/// A new type (e.g. enums, type-inst vars)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct NewType<'db>(InternedString<'db>);

impl<'db> NewType<'db> {
	fn introduce(db: &'db dyn Db) -> Self {
		static COUNTER: AtomicU32 = AtomicU32::new(0);
		NewType(InternedString::new(
			db,
			format!(
				"_TYPE_{}",
				COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
			),
		))
	}
}

/// The type of an enum value
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct EnumRef<'db>(NewType<'db>);

impl<'db> EnumRef<'db> {
	/// Create a new enum ref
	pub fn new(name: impl Into<InternedString<'db>>) -> Self {
		Self(NewType(name.into()))
	}

	// /// Create a new enum
	// pub fn new(db: &'db dyn Db, pattern: PatternRef<'db>) -> Self {
	// 	Self(NewType::from_pattern(db, pattern))
	// }

	/// Introduce a new enum type.
	///
	/// Calling this always creates a fresh new enum type.
	/// The name is only used for pretty printing.
	pub fn introduce(db: &'db dyn Db) -> Self {
		Self(NewType::introduce(db))
	}

	/// Get the name of this enum
	pub fn name(&self) -> InternedString<'db> {
		let Self(NewType(inner)) = self;
		*inner
	}

	/// Get the human readable name of this enum
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		pretty_print_identifier(self.name().lookup(db))
	}
}

/// The type of a class object value
///
/// A class is identified by its name. The superclass is carried inline because
/// subtyping between class types is resolved here, without access to the HIR;
/// the attribute list lives in the HIR signature instead, since only the
/// typechecker needs it.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct ClassRef<'db> {
	newtype: NewType<'db>,
	superclass: Option<Ty<'db>>,
}

impl<'db> ClassRef<'db> {
	/// Create a class ref for the class with the given name
	pub fn new(name: impl Into<InternedString<'db>>, superclass: Option<Ty<'db>>) -> Self {
		Self {
			newtype: NewType(name.into()),
			superclass,
		}
	}

	/// Get the name of this class
	pub fn name(&self) -> InternedString<'db> {
		let NewType(inner) = self.newtype;
		inner
	}

	/// Get the type of this class's superclass, if it has one
	pub fn superclass(&self) -> Option<Ty<'db>> {
		self.superclass
	}

	/// Get the human readable name of this class
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		pretty_print_identifier(self.name().lookup(db))
	}

	/// Get an iterator over this class and all of its superclasses
	pub fn superclasses(&self, db: &'db dyn Db) -> impl Iterator<Item = ClassRef<'db>> {
		let mut current = Some(*self);
		std::iter::from_fn(move || {
			let class = current.take()?;
			current = class.superclass.and_then(|s| s.class_type(db));
			Some(class)
		})
	}

	/// Test if this is a subclass of `other` (a class is a subclass of itself)
	pub fn is_subclass_of(&self, db: &'db dyn Db, other: &ClassRef<'db>) -> bool {
		self.superclasses(db).any(|c| c.newtype == other.newtype)
	}
}

/// The type of a reference to a type-inst var
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct TyVarRef<'db>(NewType<'db>);

impl<'db> TyVarRef<'db> {
	/// Create a new type-inst var reference
	pub fn new(name: impl Into<InternedString<'db>>) -> Self {
		Self(NewType(name.into()))
	}

	/// Introduce a new type-inst variable.
	///
	/// Calling this always creates a fresh new type-inst variable.
	/// The name is only used for pretty printing.
	pub fn introduce(db: &'db dyn Db) -> Self {
		Self(NewType::introduce(db))
	}

	/// Get the human readable name of this type-inst variable
	pub fn pretty_print(&self, db: &'db dyn Db) -> &'db str {
		let Self(NewType(name)) = self;
		name.lookup(db)
	}
}

/// The type of a type-inst variable
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub struct TyVar<'db> {
	/// The newtype for this type-inst var
	pub ty_var: TyVarRef<'db>,
	/// Whether this type-inst var is varifiable
	pub varifiable: bool,
	/// Whether this type-inst var is enumerable
	pub enumerable: bool,
	/// Whether this type-inst var is an index type (enumerable or tuple of enumerable)
	pub indexable: bool,
}

#[cfg(test)]
mod tests;
