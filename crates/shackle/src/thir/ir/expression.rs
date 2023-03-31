//! THIR representation of expressions

use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use super::{
	domain::{OptType, VarType},
	AnnotationId, Annotations, ConstraintId, DeclarationId, EnumerationId, FunctionId,
	FunctionName, Identifier, Model,
};
pub use crate::hir::{BooleanLiteral, FloatLiteral, IntegerLiteral, StringLiteral};
use crate::{
	thir::{db::Thir, source::Origin},
	ty::{FunctionType, Ty, TyData, TyParamInstantiations, TyVar},
	utils::{impl_enum_from, DebugPrint},
};

/// Trait for building expressions
pub trait ExpressionBuilder {
	/// Build the expression
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression;
}

/// An expression.
///
/// The data inside an expression is immutable (as modifying the data could invalidate the type).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Expression {
	ty: Ty,
	data: ExpressionData,
	annotations: Annotations,
	origin: Origin,
}

impl Expression {
	/// Create a new expression
	pub fn new(
		db: &dyn Thir,
		model: &Model,
		origin: impl Into<Origin>,
		value: impl ExpressionBuilder,
	) -> Self {
		value.build(db, model, origin.into())
	}

	/// Create a new expression without checking if the type is correct
	pub fn new_unchecked(
		ty: Ty,
		data: impl Into<ExpressionData>,
		origin: impl Into<Origin>,
	) -> Self {
		Self {
			ty,
			data: data.into(),
			annotations: Annotations::default(),
			origin: origin.into(),
		}
	}

	/// Get the type of this expression
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations {
		&mut self.annotations
	}

	/// Get the origin of this expression
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Get the inner data
	pub fn into_inner(self) -> (Ty, ExpressionData, Annotations, Origin) {
		(self.ty, self.data, self.annotations, self.origin)
	}
}

impl Deref for Expression {
	type Target = ExpressionData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

/// An expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExpressionData {
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
	Identifier(ResolvedIdentifier),
	/// Array literal
	ArrayLiteral(ArrayLiteral),
	/// Set literal
	SetLiteral(SetLiteral),
	/// Tuple literal
	TupleLiteral(TupleLiteral),
	/// Record literal
	RecordLiteral(RecordLiteral),
	/// Array comprehension
	ArrayComprehension(ArrayComprehension),
	/// Set comprehension
	SetComprehension(SetComprehension),
	/// Array access
	ArrayAccess(ArrayAccess),
	/// Tuple access
	TupleAccess(TupleAccess),
	/// Record access
	RecordAccess(RecordAccess),
	/// If-then-else
	IfThenElse(IfThenElse),
	/// Case expression
	Case(Case),
	/// Function call
	Call(Call),
	/// Let expression
	Let(Let),
	/// Lambda function
	Lambda(Lambda),
}

impl_enum_from!(ExpressionData::BooleanLiteral(BooleanLiteral));
impl_enum_from!(ExpressionData::IntegerLiteral(IntegerLiteral));
impl_enum_from!(ExpressionData::FloatLiteral(FloatLiteral));
impl_enum_from!(ExpressionData::StringLiteral(StringLiteral));
impl_enum_from!(ExpressionData::Identifier(ResolvedIdentifier));
impl_enum_from!(ExpressionData::ArrayLiteral(ArrayLiteral));
impl_enum_from!(ExpressionData::SetLiteral(SetLiteral));
impl_enum_from!(ExpressionData::TupleLiteral(TupleLiteral));
impl_enum_from!(ExpressionData::RecordLiteral(RecordLiteral));
impl_enum_from!(ExpressionData::ArrayComprehension(ArrayComprehension));
impl_enum_from!(ExpressionData::SetComprehension(SetComprehension));
impl_enum_from!(ExpressionData::ArrayAccess(ArrayAccess));
impl_enum_from!(ExpressionData::TupleAccess(TupleAccess));
impl_enum_from!(ExpressionData::RecordAccess(RecordAccess));
impl_enum_from!(ExpressionData::IfThenElse(IfThenElse));
impl_enum_from!(ExpressionData::Case(Case));
impl_enum_from!(ExpressionData::Call(Call));
impl_enum_from!(ExpressionData::Let(Let));
impl_enum_from!(ExpressionData::Lambda(Lambda));
impl From<Absent> for ExpressionData {
	fn from(_: Absent) -> Self {
		ExpressionData::Absent
	}
}
impl From<Infinity> for ExpressionData {
	fn from(_: Infinity) -> Self {
		ExpressionData::Infinity
	}
}

/// Absent `<>`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Absent;

impl ExpressionBuilder for Absent {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().opt_bottom, self, origin)
	}
}

impl ExpressionBuilder for BooleanLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().par_bool, self, origin)
	}
}

impl ExpressionBuilder for IntegerLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().par_int, self, origin)
	}
}

impl ExpressionBuilder for FloatLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().par_float, self, origin)
	}
}

impl ExpressionBuilder for StringLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().string, self, origin)
	}
}

/// Infinity
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Infinity;

impl ExpressionBuilder for Infinity {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(db.type_registry().par_int, self, origin)
	}
}

/// Array literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ArrayLiteral(pub Vec<Expression>);

impl ExpressionBuilder for ArrayLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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

impl Deref for ArrayLiteral {
	type Target = Vec<Expression>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ArrayLiteral {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Set literal
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct SetLiteral(pub Vec<Expression>);

impl ExpressionBuilder for SetLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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

impl Deref for SetLiteral {
	type Target = Vec<Expression>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for SetLiteral {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Tuple literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TupleLiteral(pub Vec<Expression>);

impl ExpressionBuilder for TupleLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		let TupleLiteral(items) = &self;
		Expression::new_unchecked(
			Ty::tuple(db.upcast(), items.iter().map(|e| e.ty())),
			self,
			origin,
		)
	}
}

impl Deref for TupleLiteral {
	type Target = Vec<Expression>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for TupleLiteral {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Record literal
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecordLiteral(pub Vec<(Identifier, Expression)>);

impl ExpressionBuilder for RecordLiteral {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		let RecordLiteral(items) = &self;
		let ty = Ty::record(db.upcast(), items.iter().map(|(i, e)| (*i, e.ty())));
		Expression::new_unchecked(ty, self, origin)
	}
}

impl Deref for RecordLiteral {
	type Target = Vec<(Identifier, Expression)>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for RecordLiteral {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Array comprehension
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArrayComprehension {
	/// Value of the comprehension
	pub template: Box<Expression>,
	/// The indices to generate
	pub indices: Option<Box<Expression>>,
	/// Generators of the comprehension
	pub generators: Vec<Generator>,
}

impl ArrayComprehension {
	/// Create an non-indexed array comprehension
	pub fn new(generators: impl IntoIterator<Item = Generator>, template: Expression) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: None,
			template: Box::new(template),
		}
	}

	/// Create an indexed array comprehension
	pub fn indexed(
		generators: impl IntoIterator<Item = Generator>,
		indices: Expression,
		template: Expression,
	) -> Self {
		Self {
			generators: generators.into_iter().collect(),
			indices: Some(Box::new(indices)),
			template: Box::new(template),
		}
	}
}

impl ExpressionBuilder for ArrayComprehension {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct SetComprehension {
	/// Value of the comprehension
	pub template: Box<Expression>,
	/// Generators of the comprehension
	pub generators: Vec<Generator>,
}

impl ExpressionBuilder for SetComprehension {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct ArrayAccess {
	/// The array being indexed into
	pub collection: Box<Expression>,
	/// The indices
	pub indices: Box<Expression>,
}

impl ExpressionBuilder for ArrayAccess {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct TupleAccess {
	/// Tuple being accessed
	pub tuple: Box<Expression>,
	/// Field being accessed
	pub field: IntegerLiteral,
}

impl ExpressionBuilder for TupleAccess {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct RecordAccess {
	/// Record being accessed
	pub record: Box<Expression>,
	/// Field being accessed
	pub field: Identifier,
}

impl ExpressionBuilder for RecordAccess {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct IfThenElse {
	/// The if-then and elseif-then branches
	pub branches: Vec<Branch>,
	/// The else result
	pub else_result: Box<Expression>,
}

impl ExpressionBuilder for IfThenElse {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub struct Case {
	/// The expression being matched on
	pub scrutinee: Box<Expression>,
	/// The case match arms
	pub branches: Vec<CaseBranch>,
}

impl ExpressionBuilder for Case {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
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
pub enum Callable {
	/// Call to a function item
	Function(FunctionId),
	/// Call to an annotation constructor function
	Annotation(AnnotationId),
	/// Call to an annotation destructor function
	AnnotationDestructure(AnnotationId),
	/// Call to an enum constructor function
	EnumConstructor(EnumMemberId),
	/// Call to an enum destructor function
	EnumDestructor(EnumMemberId),
	/// Call to a lambda expression
	Expression(Box<Expression>),
}

/// A function call
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call {
	/// Function being called
	pub function: Callable,
	/// Call arguments
	pub arguments: Vec<Expression>,
}

impl ExpressionBuilder for Call {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
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
				let mut kind = None;
				for (arg, param) in self.arguments.iter().zip(params.iter()) {
					let (arg_k, arg_ty) = EnumConstructorKind::from_ty(db, arg.ty());
					if let Some(k) = kind {
						assert_eq!(k, arg_k, "Invalid enum constructor arguments");
					} else {
						kind = Some(arg_k);
					}
					assert!(arg_ty.is_subtype_of(db.upcast(), model[*param].ty()));
				}
				let ty = Ty::par_enum(db.upcast(), model[e.enumeration_id()].enum_type());
				kind.unwrap().lift(db, ty)
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
pub struct LookupCall {
	/// Function name
	pub function: FunctionName,
	/// Call arguments
	pub arguments: Vec<Expression>,
}

impl LookupCall {
	/// Perform the call lookup and produce a `Call`
	pub fn resolve(self, db: &dyn Thir, model: &Model) -> (Call, Ty) {
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

impl ExpressionBuilder for LookupCall {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let (call, return_ty) = self.resolve(db, model);
		Expression::new_unchecked(return_ty, call, origin)
	}
}

/// A let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Let {
	/// Items in this let expression
	pub items: Vec<LetItem>,
	/// Value of the let expression
	pub in_expression: Box<Expression>,
}

impl ExpressionBuilder for Let {
	fn build(self, _db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(self.in_expression.ty(), self, origin)
	}
}

/// A lambda function
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Lambda(pub FunctionId);

impl ExpressionBuilder for Lambda {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
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

impl Deref for Lambda {
	type Target = FunctionId;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl ExpressionBuilder for AnnotationId {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
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

impl From<DeclarationId> for ExpressionData {
	fn from(idx: DeclarationId) -> Self {
		ResolvedIdentifier::Declaration(idx).into()
	}
}

impl ExpressionBuilder for DeclarationId {
	fn build(self, _db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(model[self].ty(), self, origin)
	}
}

impl From<EnumerationId> for ExpressionData {
	fn from(idx: EnumerationId) -> Self {
		ResolvedIdentifier::Enumeration(idx).into()
	}
}

impl ExpressionBuilder for EnumerationId {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let ty = Ty::par_set(
			db.upcast(),
			Ty::par_enum(db.upcast(), model[self].enum_type()),
		)
		.unwrap();
		Expression::new_unchecked(ty, self, origin)
	}
}

impl ExpressionBuilder for Identifier {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let result = model
			.lookup_identifier(db, self)
			.expect("Identifier not found");
		Expression::new(db, model, origin, result)
	}
}

/// An identifier which resolves to a declaration
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ResolvedIdentifier {
	/// Identifier resolves to an annotation atom
	Annotation(AnnotationId),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId),
	/// Identifier resolves to an enumeration defining set
	Enumeration(EnumerationId),
	/// Identifier resolves to an enumeration member atom with the given index
	EnumerationMember(EnumMemberId),
}

impl_enum_from!(ResolvedIdentifier::Annotation(AnnotationId));
impl_enum_from!(ResolvedIdentifier::Declaration(DeclarationId));
impl_enum_from!(ResolvedIdentifier::Enumeration(EnumerationId));
impl_enum_from!(ResolvedIdentifier::EnumerationMember(EnumMemberId));

impl ExpressionBuilder for ResolvedIdentifier {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
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
pub struct EnumMemberId {
	parent: EnumerationId,
	index: u32,
}

impl EnumMemberId {
	/// Create a new reference to a enum member
	pub fn new(enumeration: EnumerationId, index: u32) -> Self {
		Self {
			parent: enumeration,
			index,
		}
	}

	/// Get the enumeration id
	pub fn enumeration_id(&self) -> EnumerationId {
		self.parent
	}

	/// Get the index of the enum member inside the enum
	pub fn member_index(&self) -> u32 {
		self.index
	}
}

impl ExpressionBuilder for EnumMemberId {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let ty = Ty::par_enum(db.upcast(), model[self.enumeration_id()].enum_type());
		Expression::new_unchecked(ty, self, origin)
	}
}

impl From<EnumMemberId> for ExpressionData {
	fn from(idx: EnumMemberId) -> Self {
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
pub enum Generator {
	/// Generator which iterates over a collection
	Iterator {
		/// Generator declaration
		declarations: Vec<DeclarationId>,
		/// Expression being iterated over
		collection: Expression,
		/// Where clause
		where_clause: Option<Expression>,
	},
	/// Generator which is an assignment
	Assignment {
		/// The assignment to generate
		assignment: DeclarationId,
		/// Where clause
		where_clause: Option<Expression>,
	},
}

impl Generator {
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
	pub fn where_clause(&self) -> Option<&Expression> {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => where_clause.as_ref(),
		}
	}

	/// Set the where clause for this generator
	pub fn set_where(&mut self, w: Expression) {
		match self {
			Generator::Iterator { where_clause, .. }
			| Generator::Assignment { where_clause, .. } => *where_clause = Some(w),
		}
	}

	/// Get the declarations/assignment for this generator
	pub fn declarations(&self) -> impl '_ + Iterator<Item = DeclarationId> {
		match self {
			Generator::Iterator { declarations, .. } => declarations.clone().into_iter(),
			Generator::Assignment { assignment, .. } => vec![*assignment].into_iter(),
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Branch {
	/// The boolean condition
	pub condition: Expression,
	/// The result if the condition holds
	pub result: Expression,
}

impl Branch {
	/// Create a new branch for an if-then-else
	pub fn new(condition: Expression, result: Expression) -> Self {
		Self { condition, result }
	}

	/// True if the condition is var
	pub fn var_condition(&self, db: &dyn Thir) -> bool {
		self.condition.ty().inst(db.upcast()).unwrap() == VarType::Var
	}
}

/// A branch of a `Case`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CaseBranch {
	/// The pattern to match
	pub pattern: Pattern,
	/// The value if the pattern matches
	pub result: Expression,
}

impl CaseBranch {
	/// Create a new case branch
	pub fn new(pattern: Pattern, result: Expression) -> Self {
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
pub struct Pattern {
	data: PatternData,
	origin: Origin,
}

impl Pattern {
	/// Create a new pattern
	pub fn new(data: PatternData, origin: impl Into<Origin>) -> Self {
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
	pub fn into_inner(self) -> (PatternData, Origin) {
		(self.data, self.origin)
	}
}

impl Deref for Pattern {
	type Target = PatternData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

impl DerefMut for Pattern {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}

/// A pattern for a case expression.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PatternData {
	/// Enum constructor call
	EnumConstructor {
		/// The enum item member
		member: EnumMemberId,
		/// The constructor call arguments
		args: Vec<Pattern>,
	},
	/// Annotation constructor call
	AnnotationConstructor {
		/// The annotation item
		item: AnnotationId,
		/// The constructor call arguments
		args: Vec<Pattern>,
	},
	/// Tuple
	Tuple(Vec<Pattern>),
	/// Record
	Record(Vec<(Identifier, Pattern)>),
	/// Literal expression (e.g. enum atoms, numbers, strings, <>)
	Expression(Box<Expression>),
	/// Wildcard pattern _
	Anonymous(Ty),
}

/// An item in a let expression
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LetItem {
	/// A local constraint item
	Constraint(ConstraintId),
	/// A local declaration item
	Declaration(DeclarationId),
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
