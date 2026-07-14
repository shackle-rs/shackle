//! THIR representation of expressions

use std::{
	fmt::Debug,
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use derive_more::From;
use rustc_hash::FxHashMap;
use shackle_hir::constants::IdentifierRegistry;
pub use shackle_hir::{BooleanLiteral, FloatLiteral, IntegerLiteral, StringLiteral};
use shackle_ty::{FunctionType, Ty, TyData, TyParamInstantiations, TyVar, registry::TypeRegistry};
use shackle_utils::maybe_grow_stack;

use super::{
	AnnotationId, Annotations, ConstraintId, Declaration, DeclarationId, Domain, EnumerationId,
	FunctionId, FunctionName, Identifier, Item, Marker, Model,
	domain::{OptType, VarType},
};
use crate::{Db, DomainData, source::Origin};

/// Trait for building expressions
pub trait ExpressionBuilder<'db, T: Marker = ()> {
	/// Build the expression
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T>;
}

/// An expression.
///
/// The data inside an expression is immutable (as modifying the data could invalidate the type).
#[derive(Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Expression<'db, T: Marker = ()> {
	ty: Ty<'db>,
	data: ExpressionData<'db, T>,
	annotations: Annotations<'db, T>,
	origin: Origin<'db>,
	phantom: PhantomData<T>,
}

impl<'db, T: Marker> Expression<'db, T> {
	/// Create a new expression
	pub fn new(
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: impl Into<Origin<'db>>,
		value: impl ExpressionBuilder<'db, T>,
	) -> Self {
		value.build(db, model, origin.into())
	}

	/// Create a new expression without checking if the type is correct
	pub fn new_unchecked(
		ty: Ty<'db>,
		data: impl Into<ExpressionData<'db, T>>,
		origin: impl Into<Origin<'db>>,
	) -> Self {
		Self {
			ty,
			data: data.into(),
			annotations: Annotations::default(),
			origin: origin.into(),
			phantom: PhantomData,
		}
	}

	/// Get the type of this expression
	pub fn ty(&self) -> Ty<'db> {
		self.ty
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}

	/// Get the origin of this expression
	pub fn origin(&self) -> Origin<'db> {
		self.origin
	}

	/// Set the origin of this expression
	pub fn set_origin(&mut self, origin: impl Into<Origin<'db>>) {
		self.origin = origin.into()
	}
}

impl<'db, T: Marker> Deref for Expression<'db, T> {
	type Target = ExpressionData<'db, T>;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<'db, T: Marker> Clone for Expression<'db, T> {
	fn clone(&self) -> Self {
		// Default recursive clone can cause stack overflow
		maybe_grow_stack(|| Self {
			ty: self.ty,
			data: self.data.clone(),
			annotations: self.annotations.clone(),
			origin: self.origin,
			phantom: PhantomData,
		})
	}
}

/// An expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, From, salsa::Update)]
pub enum ExpressionData<'db, T: Marker = ()> {
	/// Absent `<>`
	Absent,
	/// Bool literal
	BooleanLiteral(BooleanLiteral),
	/// Integer literal
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	FloatLiteral(FloatLiteral),
	/// String literal
	StringLiteral(StringLiteral<'db>),
	/// Infinity
	Infinity,
	/// Identifier
	Identifier(ResolvedIdentifier<'db, T>),
	/// Array literal
	ArrayLiteral(ArrayLiteral<'db, T>),
	/// Set literal
	SetLiteral(SetLiteral<'db, T>),
	/// Tuple literal
	TupleLiteral(TupleLiteral<'db, T>),
	/// Record literal
	RecordLiteral(RecordLiteral<'db, T>),
	/// Array comprehension
	ArrayComprehension(ArrayComprehension<'db, T>),
	/// Set comprehension
	SetComprehension(SetComprehension<'db, T>),
	/// Tuple access
	TupleAccess(TupleAccess<'db, T>),
	/// Record access
	RecordAccess(RecordAccess<'db, T>),
	/// If-then-else
	IfThenElse(IfThenElse<'db, T>),
	/// Case expression
	Case(Case<'db, T>),
	/// Function call
	Call(Call<'db, T>),
	/// Let expression
	Let(Let<'db, T>),
	/// Lambda function
	Lambda(Lambda<'db, T>),
}

impl<'db, T: Marker> From<Absent> for ExpressionData<'db, T> {
	fn from(_: Absent) -> Self {
		ExpressionData::Absent
	}
}
impl<'db, T: Marker> From<Infinity> for ExpressionData<'db, T> {
	fn from(_: Infinity) -> Self {
		ExpressionData::Infinity
	}
}

/// Creates a dummy value of a given type
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DummyValue<'db>(pub Ty<'db>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for DummyValue<'db> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		if self.0.opt(db) == Some(OptType::Opt) {
			return Absent.build(db, model, origin);
		}
		let ids = IdentifierRegistry::lookup(db);
		match self.0.lookup(db) {
			TyData::Annotation(_) => model
				.lookup_identifier(db, ids.annotations.empty_annotation)
				.unwrap()
				.build(db, model, origin),
			TyData::Array { .. } => ArrayLiteral(Vec::new()).build(db, model, origin),
			TyData::Boolean(_, _) => BooleanLiteral(false).build(db, model, origin),
			TyData::Float(_, _) => FloatLiteral::new(0.0).build(db, model, origin),
			TyData::Integer(_, _) => IntegerLiteral(0).build(db, model, origin),
			TyData::Record(_, fs) => RecordLiteral(
				fs.iter()
					.map(|(i, ty)| ((*i).into(), DummyValue(*ty).build(db, model, origin)))
					.collect(),
			)
			.build(db, model, origin),
			TyData::Set(_, _, _) => SetLiteral(Vec::new()).build(db, model, origin),
			TyData::String(_) => {
				StringLiteral::from(ids.literals.empty_string).build(db, model, origin)
			}
			TyData::Tuple(_, fs) => TupleLiteral(
				fs.iter()
					.map(|ty| DummyValue(*ty).build(db, model, origin))
					.collect(),
			)
			.build(db, model, origin),
			_ => panic!(
				"Cannot create dummy value for type {}",
				self.0.pretty_print(db)
			),
		}
	}
}

/// Absent `<>`
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Absent;

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Absent {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).opt_bottom, self, origin)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for BooleanLiteral {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).par_bool, self, origin)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for IntegerLiteral {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).par_int, self, origin)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for FloatLiteral {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).par_float, self, origin)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for StringLiteral<'db> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).string, self, origin)
	}
}

/// Infinity
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Infinity;

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Infinity {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(TypeRegistry::lookup(db).par_int, self, origin)
	}
}

/// Array literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, salsa::Update)]
pub struct ArrayLiteral<'db, T: Marker = ()>(pub Vec<Expression<'db, T>>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for ArrayLiteral<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let Self(items) = &self;
		let ty = if items.is_empty() {
			TypeRegistry::lookup(db).array_of_bottom
		} else {
			let tys = items.iter().map(|e| e.ty());
			let elem_ty = Ty::most_specific_supertype(db, tys).unwrap_or_else(|| {
				panic!(
					"Non uniform array literal [{}] at {}",
					items
						.iter()
						.map(|e| e.ty().pretty_print(db))
						.collect::<Vec<_>>()
						.join(", "),
					origin.pretty_print(db)
				)
			});
			Ty::array(db, TypeRegistry::lookup(db).par_int, elem_ty).unwrap_or_else(|| {
				panic!(
					"Invalid array type (array [int] of {}) at {}",
					elem_ty.pretty_print(db),
					origin.pretty_print(db)
				)
			})
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<'db, T: Marker> Deref for ArrayLiteral<'db, T> {
	type Target = Vec<Expression<'db, T>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'db, T: Marker> DerefMut for ArrayLiteral<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Set literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, salsa::Update)]
pub struct SetLiteral<'db, T: Marker = ()>(pub Vec<Expression<'db, T>>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for SetLiteral<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let Self(items) = &self;
		if items.is_empty() {
			return Expression::new_unchecked(TypeRegistry::lookup(db).set_of_bottom, self, origin);
		}
		let elem_ty =
			Ty::most_specific_supertype(db, items.iter().map(|e| e.ty())).unwrap_or_else(|| {
				panic!(
					"Non uniform set literal [{}] at {}",
					items
						.iter()
						.map(|e| e.ty().pretty_print(db))
						.collect::<Vec<_>>()
						.join(", "),
					origin.pretty_print(db)
				)
			});
		let ty = if let VarType::Var = elem_ty.inst(db).expect("No inst for set literal") {
			Ty::par_set(db, elem_ty.make_par(db))
				.unwrap()
				.make_var(db)
				.unwrap_or_else(|| {
					panic!(
						"Cannot make set of {} var at {}",
						elem_ty.pretty_print(db),
						origin.pretty_print(db)
					)
				})
		} else {
			Ty::par_set(db, elem_ty).unwrap_or_else(|| {
				panic!(
					"Invalid set type (set of {}) at {}",
					elem_ty.pretty_print(db),
					origin.pretty_print(db)
				)
			})
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<'db, T: Marker> Deref for SetLiteral<'db, T> {
	type Target = Vec<Expression<'db, T>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'db, T: Marker> DerefMut for SetLiteral<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Tuple literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct TupleLiteral<'db, T: Marker = ()>(pub Vec<Expression<'db, T>>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for TupleLiteral<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let TupleLiteral(items) = &self;
		Expression::new_unchecked(Ty::tuple(db, items.iter().map(|e| e.ty())), self, origin)
	}
}

impl<'db, T: Marker> Deref for TupleLiteral<'db, T> {
	type Target = Vec<Expression<'db, T>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'db, T: Marker> DerefMut for TupleLiteral<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Record literal
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct RecordLiteral<'db, T: Marker = ()>(pub Vec<(Identifier<'db>, Expression<'db, T>)>);

impl<'db, T: Marker> RecordLiteral<'db, T> {
	/// Convert to hash map
	pub fn as_hash_map(&self) -> FxHashMap<Identifier<'db>, &Expression<'db, T>> {
		self.0.iter().map(|(k, v)| (*k, v)).collect()
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for RecordLiteral<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let RecordLiteral(items) = &self;
		let ty = Ty::record(db, items.iter().map(|(i, e)| (*i, e.ty())));
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<'db, T: Marker> Deref for RecordLiteral<'db, T> {
	type Target = Vec<(Identifier<'db>, Expression<'db, T>)>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'db, T: Marker> DerefMut for RecordLiteral<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Array comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct ArrayComprehension<'db, T: Marker = ()> {
	/// Value of the comprehension
	pub template: Box<Expression<'db, T>>,
	/// The indices to generate
	pub indices: Option<Box<Expression<'db, T>>>,
	/// Generators of the comprehension
	pub generators: Vec<Generator<'db, T>>,
}

impl<'db, T: Marker> ArrayComprehension<'db, T> {
	/// Create an non-indexed array comprehension
	pub fn new(
		generators: impl IntoIterator<Item = Generator<'db, T>>,
		template: Expression<'db, T>,
	) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: None,
			template: Box::new(template),
		}
	}

	/// Create an indexed array comprehension
	pub fn indexed(
		generators: impl IntoIterator<Item = Generator<'db, T>>,
		indices: Expression<'db, T>,
		template: Expression<'db, T>,
	) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: Some(Box::new(indices)),
			template: Box::new(template),
		}
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for ArrayComprehension<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		assert!(
			!self.generators.is_empty(),
			"Comprehensions must have at least one generator"
		);
		for g in self.generators.iter() {
			g.validate(db, model);
		}
		let lift_to_opt = self
			.generators
			.iter()
			.any(|g| g.var_where(db) || g.var_set(db));
		let ty = Ty::array(
			db,
			self.indices
				.as_ref()
				.map(|i| i.ty())
				.unwrap_or_else(|| TypeRegistry::lookup(db).par_int),
			if lift_to_opt {
				self.template.ty().make_var(db).unwrap().make_opt(db)
			} else {
				self.template.ty()
			},
		)
		.expect("Invalid array type");
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Set comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct SetComprehension<'db, T: Marker = ()> {
	/// Value of the comprehension
	pub template: Box<Expression<'db, T>>,
	/// Generators of the comprehension
	pub generators: Vec<Generator<'db, T>>,
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for SetComprehension<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		assert!(
			!self.generators.is_empty(),
			"Comprehensions must have at least one generator"
		);
		for g in self.generators.iter() {
			g.validate(db, model);
		}
		let is_var = self
			.generators
			.iter()
			.any(|g| g.var_where(db) || g.var_set(db));
		let elem_ty = self.template.ty().make_occurs(db);
		let ty = if let VarType::Var = elem_ty
			.inst(db)
			.expect("Invalid template inst for set comprehension")
		{
			Ty::par_set(db, elem_ty.make_par(db))
				.expect("Invalid set type")
				.make_var(db)
				.expect("Cannot make set var")
		} else {
			let st = Ty::par_set(db, elem_ty).expect("Invalid set type");
			if is_var {
				st.make_var(db).expect("Cannot make set var")
			} else {
				st
			}
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Tuple access
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct TupleAccess<'db, T: Marker = ()> {
	/// Tuple being accessed
	pub tuple: Box<Expression<'db, T>>,
	/// Field being accessed
	pub field: IntegerLiteral,
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for TupleAccess<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let ty = match self.tuple.ty().lookup(db) {
			TyData::Tuple(opt, fields) => {
				let field_ty = fields[self.field.0 as usize - 1];
				if *opt == OptType::Opt {
					field_ty.make_opt(db)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform tuple access on {} at {:?}",
				self.tuple.ty().pretty_print(db),
				origin.pretty_print(db)
			),
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Record access
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct RecordAccess<'db, T: Marker = ()> {
	/// Record being accessed
	pub record: Box<Expression<'db, T>>,
	/// Field being accessed
	pub field: Identifier<'db>,
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for RecordAccess<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let ty = match self.record.ty().lookup(db) {
			TyData::Record(opt, fields) => {
				let field_ty = fields
					.iter()
					.find_map(|(i, f)| if *i == self.field.0 { Some(*f) } else { None })
					.expect("Record field doesn't exist");
				if *opt == OptType::Opt {
					field_ty.make_opt(db)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform record access on {}",
				self.record.ty().pretty_print(db)
			),
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// If-then-else
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct IfThenElse<'db, T: Marker = ()> {
	/// The if-then and elseif-then branches
	pub branches: Vec<Branch<'db, T>>,
	/// The else result
	pub else_result: Box<Expression<'db, T>>,
}

impl<'db, T: Marker> IfThenElse<'db, T> {
	/// Whether or not this if-then-else has a var condition
	pub fn has_var_condition(&self, db: &'db dyn Db) -> bool {
		let tys = TypeRegistry::lookup(db);
		self.branches
			.iter()
			.any(|b| b.condition.ty() == tys.var_bool)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for IfThenElse<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let types = TypeRegistry::lookup(db);
		let result_ty = Ty::most_specific_supertype(
			db,
			self.branches
				.iter()
				.map(|b| b.result.ty())
				.chain([self.else_result.ty()]),
		)
		.unwrap_or_else(|| {
			panic!(
				"Invalid if-then-else branch types {} at {}",
				self.branches
					.iter()
					.map(|b| b.result.ty())
					.chain([self.else_result.ty()])
					.map(|t| t.pretty_print(db))
					.collect::<Vec<_>>()
					.join(", "),
				origin.pretty_print(db),
			)
		});
		let make_var = self
			.branches
			.iter()
			.any(|b| b.condition.ty() == types.var_bool);
		let ty = if make_var {
			result_ty.make_var(db).expect("Cannot make var")
		} else {
			result_ty
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Case expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Case<'db, T: Marker = ()> {
	/// The expression being matched on
	pub scrutinee: Box<Expression<'db, T>>,
	/// The case match arms
	pub branches: Vec<CaseBranch<'db, T>>,
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Case<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		_model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let make_var = self
			.scrutinee
			.ty()
			.inst(db)
			.expect("No inst for case scrutinee")
			== VarType::Var;
		let result_ty =
			Ty::most_specific_supertype(db, self.branches.iter().map(|b| b.result.ty()))
				.expect("Invalid case result type");
		let ty = if make_var {
			result_ty.make_var(db).expect("Cannot make var")
		} else {
			result_ty
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Target of a function call
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum Callable<'db, T: Marker = ()> {
	/// Call to a function item
	Function(FunctionId<'db, T>),
	/// Call to an annotation constructor function
	Annotation(AnnotationId<'db, T>),
	/// Call to an annotation destructor function
	AnnotationDestructure(AnnotationId<'db, T>),
	/// Call to an enum constructor function
	EnumConstructor(EnumMemberId<'db, T>),
	/// Call to an enum destructor function
	EnumDestructor(EnumMemberId<'db, T>),
	/// Call to a lambda expression
	Expression(Box<Expression<'db, T>>),
}

/// A function call
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Call<'db, T: Marker = ()> {
	/// Function being called
	pub function: Callable<'db, T>,
	/// Call arguments
	pub arguments: Vec<Expression<'db, T>>,
}

impl<'db, T: Marker> Call<'db, T> {
	/// Get the function type for this call, validating it in the process
	pub fn function_type(&self, db: &'db dyn Db, model: &Model<'db, T>) -> FunctionType<'db> {
		match &self.function {
			Callable::Annotation(a) => {
				let params = model[*a]
					.parameters
					.as_ref()
					.expect("Not an annotation function")
					.iter()
					.map(|p| model[*p].ty())
					.collect::<Box<_>>();
				assert_eq!(
					params.len(),
					self.arguments.len(),
					"Wrong number of arguments for annotation constructor {}",
					model[*a]
						.name
						.map(|i| i.pretty_print(db))
						.unwrap_or_default()
				);
				for (arg, param) in self.arguments.iter().zip(params.iter()) {
					assert!(
						arg.ty().is_subtype_of(db, *param),
						"Argument {} not a subtype of {} for annotation constructor {}",
						arg.ty().pretty_print(db),
						param.pretty_print(db),
						model[*a]
							.name
							.map(|i| i.pretty_print(db))
							.unwrap_or_default()
					);
				}
				FunctionType {
					params,
					return_type: TypeRegistry::lookup(db).ann,
				}
			}
			Callable::AnnotationDestructure(a) => {
				assert_eq!(self.arguments.len(), 1);
				assert_eq!(self.arguments[0].ty(), TypeRegistry::lookup(db).ann);
				let params = model[*a]
					.parameters
					.as_ref()
					.expect("Not an annotation function");
				assert!(
					!params.is_empty(),
					"Cannot destructure parameterless annotation function"
				);
				let return_type = if params.len() == 1 {
					model[params[0]].ty()
				} else {
					Ty::tuple(db, params.iter().map(|p| model[*p].ty()))
				};
				FunctionType {
					params: Box::new([TypeRegistry::lookup(db).ann]),
					return_type,
				}
			}
			Callable::EnumConstructor(e) => {
				let kind =
					EnumConstructorKind::from_tys(db, self.arguments.iter().map(|arg| arg.ty()));
				let params = model[*e]
					.parameters
					.as_ref()
					.expect("Not an enum constructor")
					.iter()
					.map(|p| kind.lift(db, model[*p].ty()))
					.collect::<Box<_>>();
				assert!(
					self.arguments.len() == params.len() || self.arguments.is_empty(),
					"Wrong number of arguments for enum constructor {}",
					model[*e]
						.name
						.map(|i| i.pretty_print(db))
						.unwrap_or_default()
				);
				for (arg, param) in self.arguments.iter().zip(params.iter()) {
					assert!(
						arg.ty().is_subtype_of(db, *param),
						"Argument {} not a subtype of {} for enum constructor {}",
						arg.ty().pretty_print(db),
						param.pretty_print(db),
						model[*e]
							.name
							.map(|i| i.pretty_print(db))
							.unwrap_or_default()
					);
				}
				let ty = Ty::par_enum(db, model[e.enumeration_id()].enum_type());
				let return_type = kind.lift(db, ty);
				FunctionType {
					params,
					return_type,
				}
			}
			Callable::EnumDestructor(e) => {
				assert_eq!(self.arguments.len(), 1);
				let (kind, ty) = EnumConstructorKind::from_ty(db, self.arguments[0].ty());
				assert_eq!(
					model[e.enumeration_id()].enum_type(),
					ty.enum_ty(db).unwrap()
				);
				let params = model[*e]
					.parameters
					.as_ref()
					.expect("Not an enum constructor function");
				let return_type = if params.len() == 1 {
					kind.lift(db, model[params[0]].ty())
				} else {
					Ty::tuple(db, params.iter().map(|p| kind.lift(db, model[*p].ty())))
				};
				FunctionType {
					params: Box::new([ty]),
					return_type,
				}
			}
			Callable::Expression(e) => match e.ty().lookup(db) {
				TyData::Function(_, ft) => {
					let tys = self
						.arguments
						.iter()
						.map(|arg| arg.ty())
						.collect::<Vec<_>>();
					ft.matches(db, &tys).unwrap_or_else(|e| {
						panic!("Function does not match: {}", e.pretty_print(db))
					});
					ft.clone()
				}
				_ => unreachable!("Invalid function type"),
			},
			Callable::Function(f) => {
				let arg_tys = self.arguments.iter().map(|e| e.ty()).collect::<Vec<_>>();
				let fe = model[*f].function_entry(model);
				let (_, ft) = fe
					.overload
					.instantiate_ty_params(db, &arg_tys)
					.unwrap_or_else(|e| {
						panic!(
							"Failed to instantiate function {} at {}{}: {}",
							fe.overload
								.pretty_print_item(db, model[*f].name().as_identifier(db)),
							e.pretty_print(db),
							if self.arguments.is_empty() {
								"".to_owned()
							} else {
								format!(
									" with call at {}",
									self.arguments[0].origin().pretty_print(db)
								)
							},
							model[*f].origin().pretty_print(db)
						);
					});
				ft.clone()
			}
		}
	}

	/// Whether or not this is a call to the given builtin (function with no body with the given name).
	pub fn matches_builtin(&self, model: &Model<'db, T>, builtin: Identifier<'db>) -> bool {
		let Callable::Function(f) = &self.function else {
			return false;
		};
		model[*f].name() == builtin && model[*f].body().is_none()
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Call<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(self.function_type(db, model).return_type, self, origin)
	}
}

/// A call to a function with the given name.
///
/// Used only to build expressions. Becomes a `Call` once built.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LookupCall<'db, T: Marker = ()> {
	/// Function name
	pub function: FunctionName<'db>,
	/// Call arguments
	pub arguments: Vec<Expression<'db, T>>,
}

impl<'db, T: Marker> LookupCall<'db, T> {
	/// Perform the call lookup and produce a `Call`
	pub fn resolve(self, db: &'db dyn Db, model: &Model<'db, T>) -> (Call<'db, T>, Ty<'db>) {
		let args: Vec<_> = self.arguments.into_iter().collect();
		let arg_tys: Vec<_> = args.iter().map(|arg| arg.ty()).collect();
		let lookup = model
			.lookup_function(db, self.function, &arg_tys)
			.unwrap_or_else(|e| {
				panic!(
					"Function {}({}) not found:\n{}",
					self.function.pretty_print(db),
					arg_tys
						.iter()
						.map(|ty| ty.pretty_print(db))
						.collect::<Vec<_>>()
						.join(", "),
					e.pretty_print(db)
				)
			});
		let fn_type = lookup.fn_entry.overload.instantiate(db, &lookup.ty_vars);
		let return_ty = fn_type.return_type;

		(
			Call {
				function: Callable::Function(lookup.function),
				arguments: args,
			},
			return_ty,
		)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for LookupCall<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let (call, return_ty) = self.resolve(db, model);
		Expression::new_unchecked(return_ty, call, origin)
	}
}

/// A top-level identifier with the given name
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct LookupIdentifier<'db>(pub Identifier<'db>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for LookupIdentifier<'db> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		model
			.lookup_identifier(db, self.0)
			.unwrap_or_else(|| panic!("Undefined variable '{}'", self.0.pretty_print(db)))
			.build(db, model, origin)
	}
}

/// A let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Let<'db, T: Marker = ()> {
	/// Items in this let expression
	pub items: Vec<LetItem<'db, T>>,
	/// Value of the let expression
	pub in_expression: Box<Expression<'db, T>>,
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Let<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let types = TypeRegistry::lookup(db);
		let mut ty = self.in_expression.ty();
		if ty != types.ann
			&& !ty.contains_var(db)
			&& self.items.iter().any(|item| match item {
				LetItem::Constraint(idx) => model[*idx].expression().ty() == types.var_bool,
				LetItem::Declaration(idx) => {
					model[*idx].definition().is_some()
						&& model[*idx]
							.domain()
							.walk()
							.any(|d| d.ty().inst(db) == Some(VarType::Var) && !d.ty().is_bool(db))
				}
			}) {
			ty = ty
				.make_var(db)
				.unwrap_or_else(|| panic!("Could not make {} var", ty.pretty_print(db)));
		}
		Expression::new_unchecked(ty, self, origin)
	}
}

/// A lambda function
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Lambda<'db, T: Marker = ()>(pub FunctionId<'db, T>);

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Lambda<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let fe = model[self.0].function_entry(model);
		Expression::new_unchecked(
			Ty::function(
				db,
				FunctionType {
					return_type: fe.overload.return_type(),
					params: fe.overload.params().iter().copied().collect(),
				},
			),
			self,
			origin,
		)
	}
}

impl<'db, T: Marker> Deref for Lambda<'db, T> {
	type Target = FunctionId<'db, T>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for AnnotationId<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let ann = TypeRegistry::lookup(db).ann;
		let ty = if let Some(params) = &model[self].parameters {
			Ty::function(
				db,
				FunctionType {
					params: params.iter().map(|d| model[*d].ty()).collect(),
					return_type: ann,
				},
			)
		} else {
			ann
		};
		Expression::new_unchecked(ty, ResolvedIdentifier::Annotation(self), origin)
	}
}

impl<'db, T: Marker> From<DeclarationId<'db, T>> for ExpressionData<'db, T> {
	fn from(idx: DeclarationId<'db, T>) -> Self {
		ResolvedIdentifier::Declaration(idx).into()
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for DeclarationId<'db, T> {
	fn build(
		self,
		_db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		Expression::new_unchecked(model[self].ty(), self, origin)
	}
}

impl<'db, T: Marker> From<EnumerationId<'db, T>> for ExpressionData<'db, T> {
	fn from(idx: EnumerationId<'db, T>) -> Self {
		ResolvedIdentifier::Enumeration(idx).into()
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for EnumerationId<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let ty = Ty::par_set(db, Ty::par_enum(db, model[self].enum_type())).unwrap();
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for Identifier<'db> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let result = model
			.lookup_identifier(db, self)
			.expect("Identifier not found");
		Expression::new(db, model, origin, result)
	}
}

/// An identifier which resolves to a declaration
#[derive(Clone, Debug, Hash, PartialEq, Eq, From, salsa::Update)]
pub enum ResolvedIdentifier<'db, T: Marker = ()> {
	/// Identifier resolves to an annotation atom
	Annotation(AnnotationId<'db, T>),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId<'db, T>),
	/// Identifier resolves to an enumeration defining set
	Enumeration(EnumerationId<'db, T>),
	/// Identifier resolves to an enumeration member atom with the given index
	EnumerationMember(EnumMemberId<'db, T>),
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for ResolvedIdentifier<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		match self {
			ResolvedIdentifier::Annotation(i) => i.build(db, model, origin),
			ResolvedIdentifier::Declaration(i) => i.build(db, model, origin),
			ResolvedIdentifier::Enumeration(i) => i.build(db, model, origin),
			ResolvedIdentifier::EnumerationMember(i) => i.build(db, model, origin),
		}
	}
}

/// Reference to a member of an enum
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct EnumMemberId<'db, T: Marker = ()> {
	parent: EnumerationId<'db, T>,
	index: u32,
}

impl<'db, T: Marker> EnumMemberId<'db, T> {
	/// Create a new reference to a enum member
	pub fn new(enumeration: EnumerationId<'db, T>, index: u32) -> Self {
		Self {
			parent: enumeration,
			index,
		}
	}

	/// Get the enumeration id
	pub fn enumeration_id(&self) -> EnumerationId<'db, T> {
		self.parent
	}

	/// Get the index of the enum member inside the enum
	pub fn member_index(&self) -> u32 {
		self.index
	}
}

impl<'db, T: Marker> ExpressionBuilder<'db, T> for EnumMemberId<'db, T> {
	fn build(
		self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: Origin<'db>,
	) -> Expression<'db, T> {
		let ty = Ty::par_enum(db, model[self.enumeration_id()].enum_type());
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<'db, T: Marker> From<EnumMemberId<'db, T>> for ExpressionData<'db, T> {
	fn from(idx: EnumMemberId<'db, T>) -> Self {
		ResolvedIdentifier::EnumerationMember(idx).into()
	}
}

/// Kind of enum constructor (or destructor)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum EnumConstructorKind {
	/// par enum
	Par,
	/// var enum
	Var,
	/// par opt enum
	Opt,
	/// var opt enum
	VarOpt,
	/// set of enum
	Set,
	/// var set of enum
	VarSet,
}

impl EnumConstructorKind {
	/// Gets the enum constructor kind which was used to create something of this type
	pub fn from_ty<'db>(db: &'db dyn Db, ty: Ty<'db>) -> (Self, Ty<'db>) {
		let is_var = ty.inst(db).unwrap() == VarType::Var;
		let is_opt = ty.opt(db).unwrap() == OptType::Opt;
		let is_set = ty.is_set(db);
		match (is_var, is_opt, is_set) {
			(false, false, false) => (EnumConstructorKind::Par, ty),
			(true, false, false) => (EnumConstructorKind::Var, ty.make_par(db)),
			(false, true, false) => (EnumConstructorKind::Opt, ty.make_occurs(db)),
			(true, true, false) => (EnumConstructorKind::VarOpt, ty.make_par(db).make_occurs(db)),
			(false, false, true) => (EnumConstructorKind::Set, ty.elem_ty(db).unwrap()),
			(true, false, true) => (EnumConstructorKind::VarSet, ty.elem_ty(db).unwrap()),
			_ => unreachable!(),
		}
	}

	/// Gets the enum constructor kind for the given arguments
	pub fn from_tys<'db>(
		db: &'db dyn Db,
		tys: impl IntoIterator<Item = Ty<'db>>,
	) -> EnumConstructorKind {
		let (is_var, is_opt, is_set) =
			tys.into_iter().fold((false, false, None), |(v, o, s), ty| {
				(
					v || ty.inst(db).unwrap() == VarType::Var,
					o || ty.opt(db).unwrap() == OptType::Opt,
					if let Some(is_set) = s {
						assert_eq!(is_set, ty.is_set(db));
						Some(is_set)
					} else {
						Some(ty.is_set(db))
					},
				)
			});

		match (is_var, is_opt, is_set) {
			(_, _, None) => EnumConstructorKind::Set,
			(false, false, Some(false)) => EnumConstructorKind::Par,
			(true, false, Some(false)) => EnumConstructorKind::Var,
			(false, true, Some(false)) => EnumConstructorKind::Opt,
			(true, true, Some(false)) => EnumConstructorKind::VarOpt,
			(false, false, Some(true)) => EnumConstructorKind::Set,
			(true, false, Some(true)) => EnumConstructorKind::VarSet,
			_ => unreachable!(),
		}
	}

	/// Apply this kind of lifting to the given type
	pub fn lift<'db>(&self, db: &'db dyn Db, ty: Ty<'db>) -> Ty<'db> {
		match self {
			EnumConstructorKind::Par => ty,
			EnumConstructorKind::Var => ty.make_var(db).unwrap(),
			EnumConstructorKind::Opt => ty.make_opt(db),
			EnumConstructorKind::VarOpt => ty.make_var(db).unwrap().make_opt(db),
			EnumConstructorKind::Set => Ty::par_set(db, ty).unwrap(),
			EnumConstructorKind::VarSet => Ty::par_set(db, ty).unwrap().make_var(db).unwrap(),
		}
	}
}

/// Comprehension generator
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum Generator<'db, T: Marker = ()> {
	/// Generator which iterates over a collection
	Iterator {
		/// Generator declaration
		declarations: Vec<DeclarationId<'db, T>>,
		/// Expression being iterated over
		collection: Expression<'db, T>,
		/// Where clause
		where_clause: Option<Expression<'db, T>>,
	},
	/// Generator which is an assignment
	Assignment {
		/// The assignment to generate
		assignment: DeclarationId<'db, T>,
		/// Where clause
		where_clause: Option<Expression<'db, T>>,
	},
}

impl<'db, T: Marker> Generator<'db, T> {
	/// Create a generator that iterates over the given collection
	pub fn iterator(
		db: &'db dyn Db,
		count: usize,
		collection: Expression<'db, T>,
		model: &mut Model<'db, T>,
	) -> Self {
		let mut declarations = Vec::with_capacity(count);
		let elem = collection.ty().elem_ty(db).unwrap();
		for _ in 0..count {
			declarations.push(model.add_declaration(Item::new(
				Declaration::new(false, Domain::unbounded(db, collection.origin(), elem)),
				collection.origin(),
			)));
		}
		Self::Iterator {
			declarations,
			collection,
			where_clause: None,
		}
	}

	/// Whether this generator has a var where clause
	pub fn var_where(&self, db: &'db dyn Db) -> bool {
		match self {
			Generator::Iterator {
				where_clause: Some(w),
				..
			}
			| Generator::Assignment {
				where_clause: Some(w),
				..
			} => w.ty().inst(db).unwrap() == VarType::Var,
			_ => false,
		}
	}

	/// Whether this generator iterates over a var set
	pub fn var_set(&self, db: &'db dyn Db) -> bool {
		match self {
			Generator::Iterator { collection, .. } => collection.ty().is_var_set(db),
			_ => false,
		}
	}

	/// Get the where clause for this generator
	pub fn where_clause(&self) -> Option<&Expression<'db, T>> {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => where_clause.as_ref(),
		}
	}

	/// Set the where clause for this generator
	pub fn set_where(&mut self, w: Expression<'db, T>) {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => *where_clause = Some(w),
		}
	}

	/// Update where clause for this generator
	pub fn update_where(
		&mut self,
		f: impl FnOnce(Option<Expression<'db, T>>) -> Option<Expression<'db, T>>,
	) {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => *where_clause = f(where_clause.take()),
		}
	}

	/// Get the declarations/assignment for this generator
	pub fn declarations(&self) -> impl '_ + Iterator<Item = DeclarationId<'db, T>> {
		match self {
			Generator::Iterator { declarations, .. } => declarations.clone().into_iter(),
			Generator::Assignment { assignment, .. } => vec![*assignment].into_iter(),
		}
	}

	/// Validate that this generator is type correct
	pub fn validate(&self, db: &'db dyn Db, model: &Model<'db, T>) {
		match self {
			Generator::Iterator {
				declarations,
				collection,
				where_clause,
			} => {
				let elem_ty = collection.ty().elem_ty(db).unwrap();
				for d in declarations {
					assert!(
						model[*d]
							.domain()
							.walk()
							.all(|d| !matches!(&**d, DomainData::Bounded(_))),
						"Iterator should not have a bounded domain"
					);
					assert!(
						model[*d].definition().is_none(),
						"Iterator should not have a right-hand side"
					);
					assert_eq!(
						model[*d].ty(),
						elem_ty,
						"Iterator is of type {} but collection is {} at {}",
						model[*d].ty().pretty_print(db),
						collection.ty().pretty_print(db),
						collection.origin().pretty_print(db)
					);
					if let Some(w) = where_clause {
						assert!(
							w.ty().is_subtype_of(db, TypeRegistry::lookup(db).var_bool),
							"Where clause is type {} at {}",
							w.ty().pretty_print(db),
							w.origin().pretty_print(db)
						);
					}
				}
			}
			Generator::Assignment {
				assignment,
				where_clause,
			} => {
				assert!(
					model[*assignment]
						.domain()
						.walk()
						.all(|d| !matches!(&**d, DomainData::Bounded(_))),
					"Iterator should not have a bounded domain"
				);
				assert!(
					model[*assignment].definition().is_some(),
					"Assignment generator must have a right-hand side"
				);
				if let Some(w) = where_clause {
					assert!(
						w.ty().is_subtype_of(db, TypeRegistry::lookup(db).var_bool),
						"Where clause is type {} at {}",
						w.ty().pretty_print(db),
						w.origin().pretty_print(db)
					);
				}
			}
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Branch<'db, T: Marker = ()> {
	/// The boolean condition
	pub condition: Expression<'db, T>,
	/// The result if the condition holds
	pub result: Expression<'db, T>,
}

impl<'db, T: Marker> Branch<'db, T> {
	/// Create a new branch for an if-then-else
	pub fn new(condition: Expression<'db, T>, result: Expression<'db, T>) -> Self {
		Self { condition, result }
	}

	/// True if the condition is var
	pub fn var_condition(&self, db: &'db dyn Db) -> bool {
		self.condition.ty().inst(db).unwrap() == VarType::Var
	}
}

/// A branch of a `Case`
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct CaseBranch<'db, T: Marker = ()> {
	/// The pattern to match
	pub pattern: Pattern<'db, T>,
	/// The value if the pattern matches
	pub result: Expression<'db, T>,
}

impl<'db, T: Marker> CaseBranch<'db, T> {
	/// Create a new case branch
	pub fn new(pattern: Pattern<'db, T>, result: Expression<'db, T>) -> Self {
		Self { pattern, result }
	}
}

/// A pattern for a case expression.
///
/// In THIR, patterns are only used for case expressions.
/// Destructuring assignments are represented using multiple declarations.
///
/// Note that patterns at this level do not represent binding to variables.
/// Instead, the anonymous wildcard pattern is used, and destructuring happens
/// via destructor functions.
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Pattern<'db, T: Marker = ()> {
	data: PatternData<'db, T>,
	origin: Origin<'db>,
}

impl<'db, T: Marker> Pattern<'db, T> {
	/// Create an enum constructor pattern
	pub fn enum_constructor(
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: impl Into<Origin<'db>>,
		member: EnumMemberId<'db, T>,
		args: impl IntoIterator<Item = Pattern<'db, T>>,
	) -> Self {
		let origin = origin.into();
		let args = args.into_iter().collect::<Vec<_>>();
		if args
			.iter()
			.all(|arg| matches!(&**arg, PatternData::Expression(_)))
		{
			let arguments = args
				.into_iter()
				.map(|arg| match arg.data {
					PatternData::Expression(e) => *e,
					_ => unreachable!(),
				})
				.collect();
			return Self {
				data: PatternData::Expression(Box::new(Expression::new(
					db,
					model,
					origin,
					Call {
						function: Callable::EnumConstructor(member),
						arguments,
					},
				))),
				origin,
			};
		}
		Self {
			data: PatternData::EnumConstructor { member, args },
			origin,
		}
	}

	/// Create an annotation constructor pattern
	pub fn annotation_constructor(
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: impl Into<Origin<'db>>,
		item: AnnotationId<'db, T>,
		args: impl IntoIterator<Item = Pattern<'db, T>>,
	) -> Self {
		let origin = origin.into();
		let args = args.into_iter().collect::<Vec<_>>();
		if args
			.iter()
			.all(|arg| matches!(&**arg, PatternData::Expression(_)))
		{
			let arguments = args
				.into_iter()
				.map(|arg| match arg.data {
					PatternData::Expression(e) => *e,
					_ => unreachable!(),
				})
				.collect();
			return Self {
				data: PatternData::Expression(Box::new(Expression::new(
					db,
					model,
					origin,
					Call {
						function: Callable::Annotation(item),
						arguments,
					},
				))),
				origin,
			};
		}
		Self {
			data: PatternData::AnnotationConstructor { item, args },
			origin,
		}
	}

	/// Create a tuple pattern
	pub fn tuple(
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: impl Into<Origin<'db>>,
		fields: impl IntoIterator<Item = Pattern<'db, T>>,
	) -> Self {
		let origin = origin.into();
		let fields = fields.into_iter().collect::<Vec<_>>();
		if fields
			.iter()
			.all(|field| matches!(&**field, PatternData::Expression(_)))
		{
			let fields = fields
				.into_iter()
				.map(|field| match field.data {
					PatternData::Expression(e) => *e,
					_ => unreachable!(),
				})
				.collect();
			return Self {
				data: PatternData::Expression(Box::new(Expression::new(
					db,
					model,
					origin,
					TupleLiteral(fields),
				))),
				origin,
			};
		}
		Self {
			data: PatternData::Tuple(fields),
			origin,
		}
	}

	/// Create a record pattern
	pub fn record(
		db: &'db dyn Db,
		model: &Model<'db, T>,
		origin: impl Into<Origin<'db>>,
		fields: impl IntoIterator<Item = (Identifier<'db>, Pattern<'db, T>)>,
	) -> Self {
		let origin = origin.into();
		let fields = fields.into_iter().collect::<Vec<_>>();
		if fields
			.iter()
			.all(|(_, field): &(Identifier<'db>, Pattern<'db, T>)| {
				matches!(&**field, PatternData::Expression(_))
			}) {
			let fields = fields
				.into_iter()
				.map(|(i, field)| match field.data {
					PatternData::Expression(e) => (i, *e),
					_ => unreachable!(),
				})
				.collect();
			return Self {
				data: PatternData::Expression(Box::new(Expression::new(
					db,
					model,
					origin,
					RecordLiteral(fields),
				))),
				origin,
			};
		}
		Self {
			data: PatternData::Record(fields),
			origin,
		}
	}

	/// Create a pattern which matches a value
	pub fn expression(expression: Expression<'db, T>, origin: impl Into<Origin<'db>>) -> Self {
		Self {
			data: PatternData::Expression(Box::new(expression)),
			origin: origin.into(),
		}
	}

	/// Create a wildcard pattern
	pub fn anonymous(ty: Ty<'db>, origin: impl Into<Origin<'db>>) -> Self {
		Self {
			data: PatternData::Anonymous(ty),
			origin: origin.into(),
		}
	}

	/// Get the origin of this pattern
	pub fn origin(&self) -> Origin<'db> {
		self.origin
	}

	/// Get the inner data
	pub fn into_inner(self) -> (PatternData<'db, T>, Origin<'db>) {
		(self.data, self.origin)
	}
}

impl<'db, T: Marker> Deref for Pattern<'db, T> {
	type Target = PatternData<'db, T>;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<'db, T: Marker> DerefMut for Pattern<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

/// A pattern for a case expression.
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum PatternData<'db, T: Marker = ()> {
	/// Enum constructor call
	EnumConstructor {
		/// The enum item member
		member: EnumMemberId<'db, T>,
		/// The constructor call arguments
		args: Vec<Pattern<'db, T>>,
	},
	/// Annotation constructor call
	AnnotationConstructor {
		/// The annotation item
		item: AnnotationId<'db, T>,
		/// The constructor call arguments
		args: Vec<Pattern<'db, T>>,
	},
	/// Tuple
	Tuple(Vec<Pattern<'db, T>>),
	/// Record
	Record(Vec<(Identifier<'db>, Pattern<'db, T>)>),
	/// Literal expression (e.g. enum atoms, numbers, strings, <>)
	Expression(Box<Expression<'db, T>>),
	/// Wildcard pattern _
	Anonymous(Ty<'db>),
}

/// An item in a let expression
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum LetItem<'db, T: Marker = ()> {
	/// A local constraint item
	Constraint(ConstraintId<'db, T>),
	/// A local declaration item
	Declaration(DeclarationId<'db, T>),
}

/// Type-inst variable instantiations
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TyVarInstantiations<'db>(Vec<Ty<'db>>);

impl<'db> TyVarInstantiations<'db> {
	/// Convert from hash map representation
	pub fn new(ty_vars: &[TyVar<'db>], instantiations: &TyParamInstantiations<'db>) -> Self {
		Self(
			ty_vars
				.iter()
				.map(|tv| instantiations[&tv.ty_var])
				.collect(),
		)
	}

	/// Convert to hash map representation
	pub fn as_map(&self, ty_vars: &[TyVar<'db>]) -> TyParamInstantiations<'db> {
		assert!(self.0.len() == ty_vars.len());
		ty_vars
			.iter()
			.zip(self.0.iter())
			.map(|(tv, ty)| (tv.ty_var, *ty))
			.collect()
	}
}

impl<'db> FromIterator<Ty<'db>> for TyVarInstantiations<'db> {
	fn from_iter<T: IntoIterator<Item = Ty<'db>>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl<'db> Deref for TyVarInstantiations<'db> {
	type Target = Vec<Ty<'db>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
