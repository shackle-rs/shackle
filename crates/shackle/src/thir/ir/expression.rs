//! THIR representation of expressions

use std::{
	fmt::Debug,
	marker::PhantomData,
	ops::{Deref, DerefMut},
};

use super::{
	domain::{OptType, VarType},
	AnnotationId, Annotations, ConstraintId, DeclarationId, EnumerationId, FunctionId,
	FunctionName, Identifier, Marker, Model,
};
pub use crate::hir::{BooleanLiteral, FloatLiteral, IntegerLiteral, StringLiteral};
use crate::{
	thir::{db::Thir, source::Origin},
	ty::{FunctionType, Ty, TyData, TyParamInstantiations, TyVar},
	utils::{impl_enum_from, DebugPrint},
};

/// Trait for building expressions
pub trait ExpressionBuilder<T = ()> {
	/// Build the expression
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T>;
}

/// An expression.
///
/// The data inside an expression is immutable (as modifying the data could invalidate the type).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Expression<T = ()> {
	ty: Ty,
	data: ExpressionData<T>,
	annotations: Annotations<T>,
	origin: Origin,
	phantom: PhantomData<T>,
}

impl<T: Marker> Expression<T> {
	/// Create a new expression
	pub fn new(
		db: &dyn Thir,
		model: &Model<T>,
		origin: impl Into<Origin>,
		value: impl ExpressionBuilder<T>,
	) -> Self {
		value.build(db, model, origin.into())
	}

	/// Create a new expression without checking if the type is correct
	pub fn new_unchecked(
		ty: Ty,
		data: impl Into<ExpressionData<T>>,
		origin: impl Into<Origin>,
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
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
		&mut self.annotations
	}

	/// Get the origin of this expression
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Get the inner data
	pub fn into_inner(self) -> (Ty, ExpressionData<T>, Annotations<T>, Origin) {
		(self.ty, self.data, self.annotations, self.origin)
	}
}

impl<T: Marker> Deref for Expression<T> {
	type Target = ExpressionData<T>;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

/// An expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExpressionData<T = ()> {
	/// Bottom
	Bottom,
	/// Absent `<>`
	Absent,
	/// Bool literal
	BooleanLiteral(BooleanLiteral),
	/// Integer literal
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	FloatLiteral(FloatLiteral),
	/// String literal
	StringLiteral(StringLiteral),
	/// Infinity
	Infinity,
	/// Identifier
	Identifier(ResolvedIdentifier<T>),
	/// Array literal
	ArrayLiteral(ArrayLiteral<T>),
	/// Set literal
	SetLiteral(SetLiteral<T>),
	/// Tuple literal
	TupleLiteral(TupleLiteral<T>),
	/// Record literal
	RecordLiteral(RecordLiteral<T>),
	/// Array comprehension
	ArrayComprehension(ArrayComprehension<T>),
	/// Set comprehension
	SetComprehension(SetComprehension<T>),
	/// Array access
	ArrayAccess(ArrayAccess<T>),
	/// Tuple access
	TupleAccess(TupleAccess<T>),
	/// Record access
	RecordAccess(RecordAccess<T>),
	/// If-then-else
	IfThenElse(IfThenElse<T>),
	/// Case expression
	Case(Case<T>),
	/// Function call
	Call(Call<T>),
	/// Let expression
	Let(Let<T>),
	/// Lambda function
	Lambda(Lambda<T>),
}

impl_enum_from!(ExpressionData<T>::BooleanLiteral(BooleanLiteral));
impl_enum_from!(ExpressionData<T>::IntegerLiteral(IntegerLiteral));
impl_enum_from!(ExpressionData<T>::FloatLiteral(FloatLiteral));
impl_enum_from!(ExpressionData<T>::StringLiteral(StringLiteral));
impl_enum_from!(ExpressionData<T>::Identifier(ResolvedIdentifier<T>));
impl_enum_from!(ExpressionData<T>::ArrayLiteral(ArrayLiteral<T>));
impl_enum_from!(ExpressionData<T>::SetLiteral(SetLiteral<T>));
impl_enum_from!(ExpressionData<T>::TupleLiteral(TupleLiteral<T>));
impl_enum_from!(ExpressionData<T>::RecordLiteral(RecordLiteral<T>));
impl_enum_from!(ExpressionData<T>::ArrayComprehension(ArrayComprehension<T>));
impl_enum_from!(ExpressionData<T>::SetComprehension(SetComprehension<T>));
impl_enum_from!(ExpressionData<T>::ArrayAccess(ArrayAccess<T>));
impl_enum_from!(ExpressionData<T>::TupleAccess(TupleAccess<T>));
impl_enum_from!(ExpressionData<T>::RecordAccess(RecordAccess<T>));
impl_enum_from!(ExpressionData<T>::IfThenElse(IfThenElse<T>));
impl_enum_from!(ExpressionData<T>::Case(Case<T>));
impl_enum_from!(ExpressionData<T>::Call(Call<T>));
impl_enum_from!(ExpressionData<T>::Let(Let<T>));
impl_enum_from!(ExpressionData<T>::Lambda(Lambda<T>));
impl<T: Marker> From<Bottom> for ExpressionData<T> {
	fn from(_: Bottom) -> Self {
		ExpressionData::Bottom
	}
}
impl<T: Marker> From<Absent> for ExpressionData<T> {
	fn from(_: Absent) -> Self {
		ExpressionData::Absent
	}
}
impl<T: Marker> From<Infinity> for ExpressionData<T> {
	fn from(_: Infinity) -> Self {
		ExpressionData::Infinity
	}
}

/// Bottom
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Bottom;

/// Absent `<>`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Absent;

impl<T: Marker> ExpressionBuilder<T> for Bottom {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().bottom, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for Absent {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().opt_bottom, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for BooleanLiteral {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().par_bool, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for IntegerLiteral {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().par_int, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for FloatLiteral {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().par_float, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for StringLiteral {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().string, self, origin)
	}
}

/// Infinity
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Infinity;

impl<T: Marker> ExpressionBuilder<T> for Infinity {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(db.type_registry().par_int, self, origin)
	}
}

/// Array literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ArrayLiteral<T = ()>(pub Vec<Expression<T>>);

impl<T: Marker> ExpressionBuilder<T> for ArrayLiteral<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let Self(items) = &self;
		let ty = if items.is_empty() {
			db.type_registry().array_of_bottom
		} else {
			let tys = items.iter().map(|e| e.ty());
			Ty::array(
				db.upcast(),
				db.type_registry().par_int,
				Ty::most_specific_supertype(db.upcast(), tys).expect("Non uniform array literal"),
			)
			.expect("Invalid array type")
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<T: Marker> Deref for ArrayLiteral<T> {
	type Target = Vec<Expression<T>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Marker> DerefMut for ArrayLiteral<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Set literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct SetLiteral<T = ()>(pub Vec<Expression<T>>);

impl<T: Marker> ExpressionBuilder<T> for SetLiteral<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let Self(items) = &self;
		if items.is_empty() {
			return Expression::new_unchecked(db.type_registry().set_of_bottom, self, origin);
		}
		let elem_ty = Ty::most_specific_supertype(db.upcast(), items.iter().map(|e| e.ty()))
			.expect("Non uniform set literal");
		let ty = if let VarType::Var = elem_ty.inst(db.upcast()).expect("No inst for set literal") {
			Ty::par_set(
				db.upcast(),
				elem_ty.with_inst(db.upcast(), VarType::Par).unwrap(),
			)
			.unwrap()
			.with_inst(db.upcast(), VarType::Var)
			.expect("Cannot make set var")
		} else {
			Ty::par_set(db.upcast(), elem_ty).expect("Invalid set type")
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<T: Marker> Deref for SetLiteral<T> {
	type Target = Vec<Expression<T>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Marker> DerefMut for SetLiteral<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Tuple literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TupleLiteral<T = ()>(pub Vec<Expression<T>>);

impl<T: Marker> ExpressionBuilder<T> for TupleLiteral<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let TupleLiteral(items) = &self;
		Expression::new_unchecked(
			Ty::tuple(db.upcast(), items.iter().map(|e| e.ty())),
			self,
			origin,
		)
	}
}

impl<T: Marker> Deref for TupleLiteral<T> {
	type Target = Vec<Expression<T>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Marker> DerefMut for TupleLiteral<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Record literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecordLiteral<T = ()>(pub Vec<(Identifier, Expression<T>)>);

impl<T: Marker> ExpressionBuilder<T> for RecordLiteral<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let RecordLiteral(items) = &self;
		let ty = Ty::record(db.upcast(), items.iter().map(|(i, e)| (*i, e.ty())));
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<T: Marker> Deref for RecordLiteral<T> {
	type Target = Vec<(Identifier, Expression<T>)>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Marker> DerefMut for RecordLiteral<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Array comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayComprehension<T = ()> {
	/// Value of the comprehension
	pub template: Box<Expression<T>>,
	/// The indices to generate
	pub indices: Option<Box<Expression<T>>>,
	/// Generators of the comprehension
	pub generators: Vec<Generator<T>>,
}

impl<T: Marker> ArrayComprehension<T> {
	/// Create an non-indexed array comprehension
	pub fn new(
		generators: impl IntoIterator<Item = Generator<T>>,
		template: Expression<T>,
	) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: None,
			template: Box::new(template),
		}
	}

	/// Create an indexed array comprehension
	pub fn indexed(
		generators: impl IntoIterator<Item = Generator<T>>,
		indices: Expression<T>,
		template: Expression<T>,
	) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: Some(Box::new(indices)),
			template: Box::new(template),
		}
	}
}

impl<T: Marker> ExpressionBuilder<T> for ArrayComprehension<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let lift_to_opt = self
			.generators
			.iter()
			.any(|g| g.var_where(db) || g.var_set(db));
		let ty = Ty::array(
			db.upcast(),
			self.indices
				.as_ref()
				.map(|i| i.ty())
				.unwrap_or_else(|| db.type_registry().par_int),
			if lift_to_opt {
				self.template
					.ty()
					.with_inst(db.upcast(), VarType::Var)
					.unwrap()
					.with_opt(db.upcast(), OptType::Opt)
			} else {
				self.template.ty()
			},
		)
		.expect("Invalid array type");
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Set comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SetComprehension<T = ()> {
	/// Value of the comprehension
	pub template: Box<Expression<T>>,
	/// Generators of the comprehension
	pub generators: Vec<Generator<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for SetComprehension<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let is_var = self
			.generators
			.iter()
			.any(|g| g.var_where(db) || g.var_set(db));
		let elem_ty = self.template.ty().with_opt(db.upcast(), OptType::NonOpt);
		let ty = if let VarType::Var = elem_ty
			.inst(db.upcast())
			.expect("Invalid template inst for set comprehension")
		{
			Ty::par_set(
				db.upcast(),
				elem_ty.with_inst(db.upcast(), VarType::Par).unwrap(),
			)
			.expect("Invalid set type")
			.with_inst(db.upcast(), VarType::Var)
			.expect("Cannot make set var")
		} else {
			let st = Ty::par_set(db.upcast(), elem_ty).expect("Invalid set type");
			if is_var {
				st.with_inst(db.upcast(), VarType::Var)
					.expect("Cannot make set var")
			} else {
				st
			}
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Array access
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayAccess<T = ()> {
	/// The array being indexed into
	pub collection: Box<Expression<T>>,
	/// The indices
	pub indices: Box<Expression<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for ArrayAccess<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let (mut make_opt, mut ty) = match self.collection.ty().lookup(db.upcast()) {
			TyData::Array { opt, element, .. } => (opt == OptType::Opt, element),
			_ => unreachable!("Not an array"),
		};
		let mut make_var = false;
		let idx_ty = self.indices.ty();
		if idx_ty.inst(db.upcast()).expect("No inst for index") == VarType::Var {
			make_var = true;
		}
		if idx_ty.opt(db.upcast()).expect("No optionality for index") == OptType::Opt {
			make_opt = true;
		}
		if let TyData::Tuple(_, fields) = idx_ty.lookup(db.upcast()) {
			for ty in fields.iter() {
				if ty.inst(db.upcast()).expect("No inst for index") == VarType::Var {
					make_var = true;
				}
				if ty.opt(db.upcast()).expect("No optionality for index") == OptType::Opt {
					make_opt = true;
				}
			}
		}
		if make_var {
			ty = ty
				.with_inst(db.upcast(), VarType::Var)
				.expect("Cannot make var");
		}
		if make_opt {
			ty = ty.with_opt(db.upcast(), OptType::Opt);
		}
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Tuple access
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TupleAccess<T = ()> {
	/// Tuple being accessed
	pub tuple: Box<Expression<T>>,
	/// Field being accessed
	pub field: IntegerLiteral,
}

impl<T: Marker> ExpressionBuilder<T> for TupleAccess<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let ty = match self.tuple.ty().lookup(db.upcast()) {
			TyData::Tuple(opt, fields) => {
				let field_ty = fields[self.field.0 as usize - 1];
				if opt == OptType::Opt {
					field_ty.with_opt(db.upcast(), OptType::Opt)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform tuple access on {}",
				self.tuple.ty().pretty_print(db.upcast())
			),
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Record access
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecordAccess<T = ()> {
	/// Record being accessed
	pub record: Box<Expression<T>>,
	/// Field being accessed
	pub field: Identifier,
}

impl<T: Marker> ExpressionBuilder<T> for RecordAccess<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let ty = match self.record.ty().lookup(db.upcast()) {
			TyData::Record(opt, fields) => {
				let field_ty = fields
					.iter()
					.find_map(|(i, f)| if *i == self.field.0 { Some(*f) } else { None })
					.expect("Record field doesn't exist");
				if opt == OptType::Opt {
					field_ty.with_opt(db.upcast(), OptType::Opt)
				} else {
					field_ty
				}
			}
			_ => unreachable!(
				"Tried to perform record access on {}",
				self.record.ty().pretty_print(db.upcast())
			),
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// If-then-else
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IfThenElse<T = ()> {
	/// The if-then and elseif-then branches
	pub branches: Vec<Branch<T>>,
	/// The else result
	pub else_result: Box<Expression<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for IfThenElse<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let types = db.type_registry();
		let result_ty = Ty::most_specific_supertype(
			db.upcast(),
			self.branches
				.iter()
				.map(|b| b.result.ty())
				.chain([self.else_result.ty()]),
		)
		.expect("Invalid if-then-else type");
		let make_var = self
			.branches
			.iter()
			.any(|b| b.condition.ty() == types.var_bool);
		let ty = if make_var {
			result_ty
				.with_inst(db.upcast(), VarType::Var)
				.expect("Cannot make var")
		} else {
			result_ty
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Case expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Case<T = ()> {
	/// The expression being matched on
	pub scrutinee: Box<Expression<T>>,
	/// The case match arms
	pub branches: Vec<CaseBranch<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for Case<T> {
	fn build(self, db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		let make_var = self
			.scrutinee
			.ty()
			.inst(db.upcast())
			.expect("No inst for case scrutinee")
			== VarType::Var;
		let result_ty =
			Ty::most_specific_supertype(db.upcast(), self.branches.iter().map(|b| b.result.ty()))
				.expect("Invalid case result type");
		let ty = if make_var {
			result_ty
				.with_inst(db.upcast(), VarType::Var)
				.expect("Cannot make var")
		} else {
			result_ty
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// Target of a function call
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Callable<T = ()> {
	/// Call to a function item
	Function(FunctionId<T>),
	/// Call to an annotation constructor function
	Annotation(AnnotationId<T>),
	/// Call to an annotation destructor function
	AnnotationDestructure(AnnotationId<T>),
	/// Call to an enum constructor function
	EnumConstructor(EnumMemberId<T>),
	/// Call to an enum destructor function
	EnumDestructor(EnumMemberId<T>),
	/// Call to a lambda expression
	Expression(Box<Expression<T>>),
}

/// A function call
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call<T = ()> {
	/// Function being called
	pub function: Callable<T>,
	/// Call arguments
	pub arguments: Vec<Expression<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for Call<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let ty = match &self.function {
			Callable::Annotation(a) => {
				let params = model[*a]
					.parameters
					.as_ref()
					.expect("Not an annotation function");
				assert_eq!(params.len(), self.arguments.len());
				for (arg, param) in self.arguments.iter().zip(params.iter()) {
					assert!(arg.ty().is_subtype_of(db.upcast(), model[*param].ty()));
				}
				db.type_registry().ann
			}
			Callable::AnnotationDestructure(a) => {
				assert_eq!(self.arguments.len(), 1);
				assert_eq!(self.arguments[0].ty(), db.type_registry().ann);
				let params = model[*a]
					.parameters
					.as_ref()
					.expect("Not an annotation function");
				assert!(
					!params.is_empty(),
					"Cannot destructure parameterless annotation function"
				);
				if params.len() == 1 {
					model[params[0]].ty()
				} else {
					Ty::tuple(db.upcast(), params.iter().map(|p| model[*p].ty()))
				}
			}
			Callable::EnumConstructor(e) => {
				let params = model[*e]
					.parameters
					.as_ref()
					.expect("Not an enum constructor");
				assert_eq!(self.arguments.len(), params.len());
				let kind =
					EnumConstructorKind::from_tys(db, self.arguments.iter().map(|arg| arg.ty()));
				for (arg, param) in self.arguments.iter().zip(params.iter()) {
					assert!(arg
						.ty()
						.is_subtype_of(db.upcast(), kind.lift(db, model[*param].ty())));
				}
				let ty = Ty::par_enum(db.upcast(), model[e.enumeration_id()].enum_type());
				kind.lift(db, ty)
			}
			Callable::EnumDestructor(e) => {
				assert_eq!(self.arguments.len(), 1);
				let (kind, ty) = EnumConstructorKind::from_ty(db, self.arguments[0].ty());
				assert_eq!(
					model[e.enumeration_id()].enum_type(),
					ty.enum_ty(db.upcast()).unwrap()
				);
				let params = model[*e]
					.parameters
					.as_ref()
					.expect("Not an enum constructor function");
				if params.len() == 1 {
					kind.lift(db, model[params[0]].ty())
				} else {
					Ty::tuple(
						db.upcast(),
						params.iter().map(|p| kind.lift(db, model[*p].ty())),
					)
				}
			}
			Callable::Expression(e) => match e.ty().lookup(db.upcast()) {
				TyData::Function(_, ft) => {
					let tys = self
						.arguments
						.iter()
						.map(|arg| arg.ty())
						.collect::<Vec<_>>();
					assert!(
						ft.matches(db.upcast(), &tys).is_ok(),
						"Function does not accept argument types"
					);
					ft.return_type
				}
				_ => unreachable!("Invalid function type"),
			},
			Callable::Function(f) => {
				let arg_tys = self.arguments.iter().map(|e| e.ty()).collect::<Vec<_>>();
				let fe = model[*f].function_entry(model);
				let ty_params = fe
					.overload
					.instantiate_ty_params(db.upcast(), &arg_tys)
					.unwrap_or_else(|e| {
						panic!(
							"Failed to instantiate function {} ({}): {}",
							model[*f].name().pretty_print(db),
							fe.overload.pretty_print(db.upcast()),
							e.debug_print(db.upcast())
						);
					});
				let ft = fe.overload.instantiate(db.upcast(), &ty_params);
				ft.return_type
			}
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// A call to a function with the given name.
///
/// Used only to build expressions. Becomes a `Call` once built.
pub struct LookupCall<T = ()> {
	/// Function name
	pub function: FunctionName,
	/// Call arguments
	pub arguments: Vec<Expression<T>>,
}

impl<T: Marker> LookupCall<T> {
	/// Perform the call lookup and produce a `Call`
	pub fn resolve(self, db: &dyn Thir, model: &Model<T>) -> (Call<T>, Ty) {
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
						.map(|ty| ty.pretty_print(db.upcast()))
						.collect::<Vec<_>>()
						.join(", "),
					e.debug_print(db.upcast())
				)
			});
		let fn_type = lookup
			.fn_entry
			.overload
			.instantiate(db.upcast(), &lookup.ty_vars);
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

impl<T: Marker> ExpressionBuilder<T> for LookupCall<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let (call, return_ty) = self.resolve(db, model);
		Expression::new_unchecked(return_ty, call, origin)
	}
}

/// A let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Let<T = ()> {
	/// Items in this let expression
	pub items: Vec<LetItem<T>>,
	/// Value of the let expression
	pub in_expression: Box<Expression<T>>,
}

impl<T: Marker> ExpressionBuilder<T> for Let<T> {
	fn build(self, _db: &dyn Thir, _model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(self.in_expression.ty(), self, origin)
	}
}

/// A lambda function
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Lambda<T = ()>(pub FunctionId<T>);

impl<T: Marker> ExpressionBuilder<T> for Lambda<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let fe = model[self.0].function_entry(model);
		Expression::new_unchecked(
			Ty::function(
				db.upcast(),
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

impl<T: Marker> Deref for Lambda<T> {
	type Target = FunctionId<T>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T: Marker> ExpressionBuilder<T> for AnnotationId<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let ann = db.type_registry().ann;
		let ty = if let Some(params) = &model[self].parameters {
			Ty::function(
				db.upcast(),
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

impl<T: Marker> From<DeclarationId<T>> for ExpressionData<T> {
	fn from(idx: DeclarationId<T>) -> Self {
		ResolvedIdentifier::Declaration(idx).into()
	}
}

impl<T: Marker> ExpressionBuilder<T> for DeclarationId<T> {
	fn build(self, _db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		Expression::new_unchecked(model[self].ty(), self, origin)
	}
}

impl<T: Marker> From<EnumerationId<T>> for ExpressionData<T> {
	fn from(idx: EnumerationId<T>) -> Self {
		ResolvedIdentifier::Enumeration(idx).into()
	}
}

impl<T: Marker> ExpressionBuilder<T> for EnumerationId<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let ty = Ty::par_set(
			db.upcast(),
			Ty::par_enum(db.upcast(), model[self].enum_type()),
		)
		.unwrap();
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<T: Marker> ExpressionBuilder<T> for Identifier {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let result = model
			.lookup_identifier(db, self)
			.expect("Identifier not found");
		Expression::new(db, model, origin, result)
	}
}

/// An identifier which resolves to a declaration
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ResolvedIdentifier<T = ()> {
	/// Identifier resolves to an annotation atom
	Annotation(AnnotationId<T>),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId<T>),
	/// Identifier resolves to an enumeration defining set
	Enumeration(EnumerationId<T>),
	/// Identifier resolves to an enumeration member atom with the given index
	EnumerationMember(EnumMemberId<T>),
}

impl_enum_from!(ResolvedIdentifier<T>::Annotation(AnnotationId<T>));
impl_enum_from!(ResolvedIdentifier<T>::Declaration(DeclarationId<T>));
impl_enum_from!(ResolvedIdentifier<T>::Enumeration(EnumerationId<T>));
impl_enum_from!(ResolvedIdentifier<T>::EnumerationMember(EnumMemberId<T>));

impl<T: Marker> ExpressionBuilder<T> for ResolvedIdentifier<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		match self {
			ResolvedIdentifier::Annotation(i) => i.build(db, model, origin),
			ResolvedIdentifier::Declaration(i) => i.build(db, model, origin),
			ResolvedIdentifier::Enumeration(i) => i.build(db, model, origin),
			ResolvedIdentifier::EnumerationMember(i) => i.build(db, model, origin),
		}
	}
}

/// Reference to a member of an enum
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumMemberId<T = ()> {
	parent: EnumerationId<T>,
	index: u32,
}

impl<T: Marker> EnumMemberId<T> {
	/// Create a new reference to a enum member
	pub fn new(enumeration: EnumerationId<T>, index: u32) -> Self {
		Self {
			parent: enumeration,
			index,
		}
	}

	/// Get the enumeration id
	pub fn enumeration_id(&self) -> EnumerationId<T> {
		self.parent
	}

	/// Get the index of the enum member inside the enum
	pub fn member_index(&self) -> u32 {
		self.index
	}
}

impl<T: Marker> ExpressionBuilder<T> for EnumMemberId<T> {
	fn build(self, db: &dyn Thir, model: &Model<T>, origin: Origin) -> Expression<T> {
		let ty = Ty::par_enum(db.upcast(), model[self.enumeration_id()].enum_type());
		Expression::new_unchecked(ty, self, origin)
	}
}

impl<T: Marker> From<EnumMemberId<T>> for ExpressionData<T> {
	fn from(idx: EnumMemberId<T>) -> Self {
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
	pub fn from_ty(db: &dyn Thir, ty: Ty) -> (Self, Ty) {
		let is_var = ty.inst(db.upcast()).unwrap() == VarType::Var;
		let is_opt = ty.opt(db.upcast()).unwrap() == OptType::Opt;
		let is_set = ty.is_set(db.upcast());
		match (is_var, is_opt, is_set) {
			(false, false, false) => (EnumConstructorKind::Par, ty),
			(true, false, false) => (
				EnumConstructorKind::Var,
				ty.with_inst(db.upcast(), VarType::Par).unwrap(),
			),
			(false, true, false) => (
				EnumConstructorKind::Opt,
				ty.with_opt(db.upcast(), OptType::NonOpt),
			),
			(true, true, false) => (
				EnumConstructorKind::VarOpt,
				ty.with_inst(db.upcast(), VarType::Par)
					.unwrap()
					.with_opt(db.upcast(), OptType::NonOpt),
			),
			(false, false, true) => (EnumConstructorKind::Set, ty.elem_ty(db.upcast()).unwrap()),
			(true, false, true) => (
				EnumConstructorKind::VarSet,
				ty.elem_ty(db.upcast()).unwrap(),
			),
			_ => unreachable!(),
		}
	}

	/// Gets the enum constructor kind for the given arguments
	pub fn from_tys(db: &dyn Thir, tys: impl IntoIterator<Item = Ty>) -> EnumConstructorKind {
		let (is_var, is_opt, is_set) =
			tys.into_iter().fold((false, false, None), |(v, o, s), ty| {
				(
					v || ty.inst(db.upcast()).unwrap() == VarType::Var,
					o || ty.opt(db.upcast()).unwrap() == OptType::Opt,
					if let Some(is_set) = s {
						assert_eq!(is_set, ty.is_set(db.upcast()));
						Some(is_set)
					} else {
						Some(ty.is_set(db.upcast()))
					},
				)
			});

		match (is_var, is_opt, is_set.unwrap()) {
			(false, false, false) => EnumConstructorKind::Par,
			(true, false, false) => EnumConstructorKind::Var,
			(false, true, false) => EnumConstructorKind::Opt,
			(true, true, false) => EnumConstructorKind::VarOpt,
			(false, false, true) => EnumConstructorKind::Set,
			(true, false, true) => EnumConstructorKind::VarSet,
			_ => unreachable!(),
		}
	}

	/// Apply this kind of lifting to the given type
	pub fn lift(&self, db: &dyn Thir, ty: Ty) -> Ty {
		match self {
			EnumConstructorKind::Par => ty,
			EnumConstructorKind::Var => ty.with_inst(db.upcast(), VarType::Var).unwrap(),
			EnumConstructorKind::Opt => ty.with_opt(db.upcast(), OptType::Opt),
			EnumConstructorKind::VarOpt => ty
				.with_inst(db.upcast(), VarType::Var)
				.unwrap()
				.with_opt(db.upcast(), OptType::Opt),
			EnumConstructorKind::Set => Ty::par_set(db.upcast(), ty).unwrap(),
			EnumConstructorKind::VarSet => Ty::par_set(db.upcast(), ty)
				.unwrap()
				.with_inst(db.upcast(), VarType::Var)
				.unwrap(),
		}
	}
}

/// Comprehension generator
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Generator<T = ()> {
	/// Generator which iterates over a collection
	Iterator {
		/// Generator declaration
		declarations: Vec<DeclarationId<T>>,
		/// Expression being iterated over
		collection: Expression<T>,
		/// Where clause
		where_clause: Option<Expression<T>>,
	},
	/// Generator which is an assignment
	Assignment {
		/// The assignment to generate
		assignment: DeclarationId<T>,
		/// Where clause
		where_clause: Option<Expression<T>>,
	},
}

impl<T: Marker> Generator<T> {
	/// Whether this generator has a var where clause
	pub fn var_where(&self, db: &dyn Thir) -> bool {
		match self {
			Generator::Iterator {
				where_clause: Some(w),
				..
			}
			| Generator::Assignment {
				where_clause: Some(w),
				..
			} => w.ty().inst(db.upcast()).unwrap() == VarType::Var,
			_ => false,
		}
	}

	/// Whether this generator iterates over a var set
	pub fn var_set(&self, db: &dyn Thir) -> bool {
		match self {
			Generator::Iterator { collection, .. } => collection.ty().is_var_set(db.upcast()),
			_ => false,
		}
	}

	/// Get the where clause for this generator
	pub fn where_clause(&self) -> Option<&Expression<T>> {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => where_clause.as_ref(),
		}
	}

	/// Set the where clause for this generator
	pub fn set_where(&mut self, w: Expression<T>) {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => *where_clause = Some(w),
		}
	}

	/// Get the declarations/assignment for this generator
	pub fn declarations(&self) -> impl '_ + Iterator<Item = DeclarationId<T>> {
		match self {
			Generator::Iterator { declarations, .. } => declarations.clone().into_iter(),
			Generator::Assignment { assignment, .. } => vec![*assignment].into_iter(),
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Branch<T = ()> {
	/// The boolean condition
	pub condition: Expression<T>,
	/// The result if the condition holds
	pub result: Expression<T>,
}

impl<T: Marker> Branch<T> {
	/// Create a new branch for an if-then-else
	pub fn new(condition: Expression<T>, result: Expression<T>) -> Self {
		Self { condition, result }
	}

	/// True if the condition is var
	pub fn var_condition(&self, db: &dyn Thir) -> bool {
		self.condition.ty().inst(db.upcast()).unwrap() == VarType::Var
	}
}

/// A branch of a `Case`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CaseBranch<T = ()> {
	/// The pattern to match
	pub pattern: Pattern<T>,
	/// The value if the pattern matches
	pub result: Expression<T>,
}

impl<T: Marker> CaseBranch<T> {
	/// Create a new case branch
	pub fn new(pattern: Pattern<T>, result: Expression<T>) -> Self {
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pattern<T = ()> {
	data: PatternData<T>,
	origin: Origin,
}

impl<T: Marker> Pattern<T> {
	/// Create a new pattern
	pub fn new(data: PatternData<T>, origin: impl Into<Origin>) -> Self {
		Self {
			data,
			origin: origin.into(),
		}
	}

	/// Get the origin of this pattern
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Get the inner data
	pub fn into_inner(self) -> (PatternData<T>, Origin) {
		(self.data, self.origin)
	}
}

impl<T: Marker> Deref for Pattern<T> {
	type Target = PatternData<T>;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl<T: Marker> DerefMut for Pattern<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

/// A pattern for a case expression.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PatternData<T = ()> {
	/// Enum constructor call
	EnumConstructor {
		/// The enum item member
		member: EnumMemberId<T>,
		/// The constructor call arguments
		args: Vec<Pattern<T>>,
	},
	/// Annotation constructor call
	AnnotationConstructor {
		/// The annotation item
		item: AnnotationId<T>,
		/// The constructor call arguments
		args: Vec<Pattern<T>>,
	},
	/// Tuple
	Tuple(Vec<Pattern<T>>),
	/// Record
	Record(Vec<(Identifier, Pattern<T>)>),
	/// Literal expression (e.g. enum atoms, numbers, strings, <>)
	Expression(Box<Expression<T>>),
	/// Wildcard pattern _
	Anonymous(Ty),
}

/// An item in a let expression
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LetItem<T = ()> {
	/// A local constraint item
	Constraint(ConstraintId<T>),
	/// A local declaration item
	Declaration(DeclarationId<T>),
}

/// Type-inst variable instantiations
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TyVarInstantiations(Vec<Ty>);

impl TyVarInstantiations {
	/// Convert from hash map representation
	pub fn new(ty_vars: &[TyVar], instantiations: &TyParamInstantiations) -> Self {
		Self(
			ty_vars
				.iter()
				.map(|tv| instantiations[&tv.ty_var])
				.collect(),
		)
	}

	/// Convert to hash map representation
	pub fn as_map(&self, ty_vars: &[TyVar]) -> TyParamInstantiations {
		assert!(self.0.len() == ty_vars.len());
		ty_vars
			.iter()
			.zip(self.0.iter())
			.map(|(tv, ty)| (tv.ty_var, *ty))
			.collect()
	}
}

impl FromIterator<Ty> for TyVarInstantiations {
	fn from_iter<T: IntoIterator<Item = Ty>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl Deref for TyVarInstantiations {
	type Target = Vec<Ty>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
