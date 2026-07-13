//! Typing and name resolution of MiniZinc programs.
//!
//! Performs
//! - Name resolution
//! - Overloading resolution
//! - Field access resolution
//! - Computation of types of items and expressions
//! - Type correctness check
//!
//! There are two entities which are typed: `signatures`, and `bodies`.
//!
//! Signatures are for any top-level items which can be referred to by an
//! identifier (e.g. functions, variable declarations). They only contain the
//! types relevant for computing the type of an identifier which refers to them.
//! That is, we don't care about the type of the RHS (unless it's an `any`
//! declaration).
//!
//! Bodies are for expressions in top-level items which need to be type-checked,
//! but cannot be referred to by an identifier. So computing types for bodies
//! may require signatures to be typed, and these in turn may require other
//! signatures to be typed, but never other bodies.
//!
//! The `SignatureTypeContext` and `BodyTypeContext` structs implement the
//! `TypeContext` trait, which allows them to both use the `Typer` struct to
//! perform type-checking of expressions.

use std::ops::{Deref, Index};

use shackle_diagnostics::{Error, Warning};
use shackle_ty::{FunctionEntry, Ty, TyData, TyVar};
use shackle_utils::arena::ArenaMap;

mod body;
mod signature;
mod typer;

pub use self::{body::*, signature::*, typer::*};
use crate::{
	Db, Expression, ExpressionId, Identifier, Item, ItemData, Model, Pattern, PatternId, TypeId,
	ids::{EntityId, PatternRef},
	input::resolve_includes,
};

/// Typecheck the entire program
#[salsa::tracked]
pub fn typecheck(db: &dyn Db) {
	log::info!("Type-checking program");
	for model in resolve_includes(db) {
		model.hir(db).typecheck(db);
	}
	log::info!("Finished type-checking program");
}

/// Accumulate typechecking diagnostics for all items.
#[salsa::tracked]
pub fn accumulate_typecheck_diagnostics(db: &dyn Db) {
	for model in resolve_includes(db) {
		accumulate_typecheck_model_diagnostics(db, model.hir(db));
	}
}

#[salsa::tracked]
fn typecheck_model<'db>(db: &'db dyn Db, model: Model<'db>) {
	log::info!("Type-checking model {}", model.file(db));
	for i in model.items(db) {
		// Avoid accumulator dependencies while running cyclic body/signature queries.
		// Diagnostics are accumulated by later non-cyclic passes.
		match i {
			Item::Assignment(_) | Item::Constraint(_) | Item::Output(_) => {
				let _ = i.body_types(db);
			}
			_ => {
				let _ = i.body_types(db);
				let _ = i.signature(db);
			}
		}
	}
}

#[salsa::tracked]
fn accumulate_typecheck_model_diagnostics<'db>(db: &'db dyn Db, model: Model<'db>) {
	for item in model.items(db) {
		accumulate_item_body_diagnostics(db, *item);
		match item {
			Item::Assignment(_) | Item::Constraint(_) | Item::Output(_) => {}
			_ => {
				accumulate_item_signature_diagnostics(db, *item);
			}
		}
	}
}

impl<'db> Model<'db> {
	/// Typecheck this model
	pub fn typecheck(&self, db: &'db dyn Db) {
		typecheck_model(db, *self);
	}
}

impl<'db> Item<'db> {
	/// Get the types for this item
	pub fn types(&self, db: &'db dyn Db) -> TypeResult<'db> {
		TypeResult::new(db, *self)
	}
}

/// Collected types for an item
///
/// This allows us to get the results of type computation in a particular item
/// by combining the computed types for the body along with its signature (if
/// it has one).
#[derive(Clone)]
pub struct TypeResult<'db> {
	db: &'db dyn Db,
	body: &'db BodyTypes<'db>,
	signature: Option<&'db SignatureTypes<'db>>,
}

impl<'db> TypeResult<'db> {
	/// Get the computed types for this item
	pub fn new(db: &'db dyn Db, item: Item<'db>) -> Self {
		match item {
			Item::Assignment(_) | Item::Constraint(_) | Item::Output(_) => TypeResult {
				db,
				body: item.body_types(db),
				signature: None,
			},
			_ => TypeResult {
				db,
				body: item.body_types(db),
				signature: Some(item.signature(db)),
			},
		}
	}

	/// Get the pattern this identifier expression resolves to
	pub fn name_resolution(&self, index: ExpressionId<'db>) -> Option<PatternRef<'db>> {
		if let Some(t) = self.body.identifier_resolution.get(&index) {
			return Some(*t);
		}
		if let Some(b) = &self.signature
			&& let Some(t) = b.identifier_resolution.get(&index)
		{
			return Some(*t);
		}
		None
	}

	/// Get the pattern this pattern (e.g. enum atom/constructor) resolves to
	pub fn pattern_resolution(&self, index: PatternId<'db>) -> Option<PatternRef<'db>> {
		if let Some(t) = self.body.pattern_resolution.get(&index) {
			return Some(*t);
		}
		if let Some(b) = &self.signature
			&& let Some(t) = b.pattern_resolution.get(&index)
		{
			return Some(*t);
		}
		None
	}

	/// Get the entities from this item which resolve to the given patter
	pub fn reverse_resolutions(
		&self,
		pattern: PatternRef<'db>,
	) -> impl Iterator<Item = EntityId<'db>> {
		self.body
			.identifier_resolution
			.iter()
			.filter_map(move |(src, dst)| {
				if *dst == pattern {
					Some(EntityId::from(*src))
				} else {
					None
				}
			})
			.chain(
				self.body
					.pattern_resolution
					.iter()
					.filter_map(move |(src, dst)| {
						if *dst == pattern {
							Some(EntityId::from(*src))
						} else {
							None
						}
					}),
			)
			.chain(self.signature.iter().flat_map(move |signature| {
				signature
					.identifier_resolution
					.iter()
					.filter_map(move |(src, dst)| {
						if *dst == pattern {
							Some(EntityId::from(*src))
						} else {
							None
						}
					})
					.chain(
						signature
							.pattern_resolution
							.iter()
							.filter_map(move |(src, dst)| {
								if *dst == pattern {
									Some(EntityId::from(*src))
								} else {
									None
								}
							}),
					)
			}))
	}

	/// Get the declaration for a pattern
	pub fn get_pattern(&self, pattern: PatternId<'db>) -> Option<&PatternTy<'db>> {
		if let Some(d) = self.body.patterns.get(pattern) {
			return Some(d);
		}
		if let Some(b) = &self.signature
			&& let Some(d) = b.patterns.get(&pattern)
		{
			return Some(d);
		}
		None
	}

	/// Get the type of an expression
	pub fn get_expression(&self, expression: ExpressionId<'db>) -> Option<Ty<'db>> {
		if let Some(t) = self.body.expressions.get(expression) {
			return Some(*t);
		}
		if let Some(b) = &self.signature
			&& let Some(t) = b.expressions.get(&expression)
		{
			return Some(*t);
		}
		None
	}

	/// Pretty print the type of an expression
	pub fn pretty_print_expression_ty(
		&self,
		data: &'db ItemData<'db>,
		expression: ExpressionId<'db>,
	) -> Option<String> {
		let ty = self.get_expression(expression)?;
		if let Expression::Identifier(i) = data[expression]
			&& let TyData::Function(opt, function) = ty.lookup(self.db)
		{
			// Pretty print functions using item-like syntax if possible
			return Some(
				opt.pretty_print()
					.into_iter()
					.chain([function.pretty_print_item(self.db, i)])
					.collect::<Vec<_>>()
					.join(" "),
			);
		}
		Some(ty.pretty_print(self.db))
	}

	/// Pretty print the type of a pattern
	pub fn pretty_print_pattern_ty(
		&self,
		data: &ItemData,
		pattern: PatternId<'db>,
	) -> Option<String> {
		let decl = self.get_pattern(pattern)?;
		match decl {
			PatternTy::Variable(ty)
			| PatternTy::Argument(ty)
			| PatternTy::Enum(ty)
			| PatternTy::Destructuring(ty)
			| PatternTy::DestructuringFn {
				constructor: ty, ..
			} => {
				if let Pattern::Identifier(i) = data[pattern] {
					if let TyData::Function(opt, function) = ty.lookup(self.db) {
						// Pretty print functions using item-like syntax if possible
						return Some(
							opt.pretty_print()
								.into_iter()
								.chain([function.pretty_print_item(self.db, i)])
								.collect::<Vec<_>>()
								.join(" "),
						);
					}
					return Some(format!(
						"{}: {}",
						ty.pretty_print(self.db),
						i.pretty_print(self.db)
					));
				}
				Some(ty.pretty_print(self.db))
			}
			PatternTy::EnumAtom(ty) => Some(format!(
				"{}: {}",
				ty.pretty_print(self.db),
				data[pattern].identifier()?.pretty_print(self.db)
			)),
			PatternTy::Function(f) => Some(
				f.overload
					.pretty_print_item(self.db, data[pattern].identifier()?),
			),
			PatternTy::EnumConstructor(ec) => Some(
				ec.first()?
					.overload
					.pretty_print_item(self.db, data[pattern].identifier()?),
			),
			PatternTy::TyVar(t) => Some(t.ty_var.pretty_print(self.db).to_owned()),
			PatternTy::TypeAlias { ty, .. } => Some(format!(
				"type {} = {}",
				data[pattern].identifier()?.pretty_print(self.db),
				ty.pretty_print(self.db)
			)),
			PatternTy::TupleField(ty) => {
				Some(format!("(tuple field) {}", ty.pretty_print(self.db)))
			}
			PatternTy::RecordField(ty) => Some(format!(
				"(record field) {}: {}",
				ty.pretty_print(self.db),
				data[pattern].identifier()?.pretty_print(self.db),
			)),
			_ => None,
		}
	}
}

impl<'db> Index<PatternId<'db>> for TypeResult<'db> {
	type Output = PatternTy<'db>;

	fn index(&self, index: PatternId<'db>) -> &Self::Output {
		self.get_pattern(index).expect("No declaration for pattern")
	}
}

impl<'db> Index<ExpressionId<'db>> for TypeResult<'db> {
	type Output = Ty<'db>;

	fn index(&self, index: ExpressionId<'db>) -> &Self::Output {
		if let Some(t) = self.body.expressions.get(index) {
			return t;
		}
		if let Some(b) = &self.signature
			&& let Some(t) = b.expressions.get(&index)
		{
			return t;
		}
		unreachable!("No type for expression {:?}", index)
	}
}

impl<'db> std::fmt::Debug for TypeResult<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let patterns = self
			.body
			.patterns
			.iter()
			.chain(
				self.signature
					.iter()
					.flat_map(|ts| ts.patterns.iter().map(|(p, d)| (*p, d))),
			)
			.collect::<ArenaMap<_, _>>();
		let expressions = self
			.body
			.expressions
			.iter()
			.chain(
				self.signature
					.iter()
					.flat_map(|ts| ts.expressions.iter().map(|(e, t)| (*e, t))),
			)
			.collect::<ArenaMap<_, _>>();

		let identifier_resolutions = self
			.body
			.identifier_resolution
			.iter()
			.map(|(e, t)| (*e, t))
			.chain(
				self.signature
					.iter()
					.flat_map(|ts| ts.identifier_resolution.iter().map(|(k, v)| (*k, v))),
			)
			.collect::<ArenaMap<_, _>>();

		let pattern_resolutions = self
			.body
			.pattern_resolution
			.iter()
			.map(|(e, t)| (*e, t))
			.chain(
				self.signature
					.iter()
					.flat_map(|ts| ts.pattern_resolution.iter().map(|(k, v)| (*k, v))),
			)
			.collect::<ArenaMap<_, _>>();

		f.debug_struct("TypeResult")
			.field("patterns", &patterns)
			.field("expressions", &expressions)
			.field("identifier_resolutions", &identifier_resolutions)
			.field("pattern_resolutions", &pattern_resolutions)
			.finish()
	}
}

/// Context for computation of types
///
/// The `Typer` calls these functions when computing types for expressions.
pub trait TypeContext<'db> {
	/// Add a declaration for a pattern
	fn add_declaration(
		&mut self,
		db: &'db dyn Db,
		pattern: PatternId<'db>,
		declaration: PatternTy<'db>,
	);
	/// Add a type for an expression
	fn add_expression(&mut self, db: &'db dyn Db, expression: ExpressionId<'db>, ty: Ty<'db>);
	/// Add identifier resolution
	fn add_identifier_resolution(
		&mut self,
		db: &'db dyn Db,
		expression: ExpressionId<'db>,
		resolution: PatternRef<'db>,
	);
	/// Add pattern resolution
	fn add_pattern_resolution(
		&mut self,
		db: &'db dyn Db,
		pattern: PatternId<'db>,
		resolution: PatternRef<'db>,
	);
	/// Add an error
	fn add_diagnostic(&mut self, db: &'db dyn Db, item: Item<'db>, e: impl Into<Error>);

	/// Add a warning
	fn add_warning(&mut self, db: &'db dyn Db, item: Item<'db>, e: impl Into<Warning>);

	/// Record the type computed for a declared type
	fn add_type(&mut self, db: &'db dyn Db, declared_type: TypeId<'db>, ty: Ty<'db>);

	/// Get the type computed for a declared type
	fn get_type(&self, db: &'db dyn Db, declared_type: TypeId<'db>) -> Ty<'db>;

	/// Type a pattern (or lookup the type if already known)
	fn type_pattern(&mut self, db: &'db dyn Db, pattern: PatternRef<'db>) -> PatternTy<'db>;
}

/// Type of a pattern (usually a declaration)
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub enum PatternTy<'db> {
	/// Pattern is a variable declaration.
	Variable(Ty<'db>),
	/// Pattern is a function declaration.
	Function(Box<FunctionEntry<'db>>),
	/// Pattern is a function parameter.
	Argument(Ty<'db>),
	/// Pattern is a type-inst variable declaration.
	TyVar(TyVar<'db>),
	/// Pattern is a type-inst alias declaration.
	TypeAlias {
		/// The type which is aliased
		ty: Ty<'db>,
		/// True if this type alias contains a bounded type
		has_bounded: bool,
		/// True if this type alias contains a primitive type
		has_unbounded: bool,
	},
	/// An enum declaration (type is of the defining set of the enum).
	Enum(Ty<'db>),
	/// Enum constructor.
	///
	/// Defines the Foo(x) function.
	EnumConstructor(Box<[EnumConstructorEntry<'db>]>),
	/// Anonymous enum constructor.
	///
	/// While the constructor cannot actually be called,
	/// we still keep track of it for convenience.
	AnonymousEnumConstructor(Box<FunctionEntry<'db>>),
	/// Enum destructor.
	///
	/// Defines the Foo^-1(x) function.
	EnumDestructure(Box<[FunctionEntry<'db>]>),
	/// Enum atom
	EnumAtom(Ty<'db>),
	/// Annotation constructor.
	///
	/// Defines the Foo(x) function.
	AnnotationConstructor(Box<FunctionEntry<'db>>),
	/// Annotation destructor.
	///
	/// Defines the Foo^-1(x) function.
	AnnotationDestructure(Box<FunctionEntry<'db>>),
	/// Annotation atom
	AnnotationAtom,
	/// Destructuring pattern
	Destructuring(Ty<'db>),
	/// Destructuring function call identifier
	///
	/// Used for the constructor identifier pattern
	/// (the call will have the `Destructuring` type)
	DestructuringFn {
		/// The type of the constructor function
		constructor: Ty<'db>,
		/// The type of the destructor function
		destructor: Ty<'db>,
	},
	/// Tuple field (e.g. x in t.1, or (x: 1))
	///
	TupleField(Ty<'db>),
	/// Record field (e.g. x in r.x, or (x: 1))
	///
	RecordField(Ty<'db>),
	/// Class declaration
	ClassDecl {
		/// The attributes declared by this class itself, in declaration order.
		///
		/// Inherited attributes are not included; walk the superclass chain for
		/// those. Class types carry only the name and superclass, so this is
		/// where a class's attribute list lives.
		attributes: Vec<(Identifier<'db>, Ty<'db>)>,
		/// The set of objects of this class, which is the type the class name
		/// denotes in a domain position
		defining_set_ty: Ty<'db>,
		/// The record type an object of this class is constructed from
		input_record_ty: Ty<'db>,
		/// The record type an object of this class is stored as
		storage_record_ty: Ty<'db>,
		/// The storage record with every attribute varified, used for classes
		/// reached via a `var new` introduction path
		var_storage_record_ty: Ty<'db>,
	},
	/// Currently computing - if encountered, indicates a cycle
	Computing,
}

/// Constructor for an enum
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct EnumConstructorEntry<'db> {
	/// If true, this constructor is lifted and is not used for pattern matching
	pub is_lifted: bool,
	/// The function entry
	pub constructor: FunctionEntry<'db>,
}

impl<'db> Deref for EnumConstructorEntry<'db> {
	type Target = FunctionEntry<'db>;

	fn deref(&self) -> &Self::Target {
		&self.constructor
	}
}

#[cfg(test)]
mod tests;
