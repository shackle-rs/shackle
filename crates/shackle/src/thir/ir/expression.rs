//! THIR representation of expressions

use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use super::{
	domain::{Domain, OptType, VarType},
	AnnotationId, Annotations, ConstraintId, DeclarationId, EnumerationId, FunctionId, Identifier,
	Model,
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
		let elem_ty = self.template.ty();
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

/// A function call
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call {
	/// Function being called
	pub function: Box<Expression>,
	/// Call arguments
	pub arguments: Vec<Expression>,
}

impl ExpressionBuilder for Call {
	fn build(self, db: &dyn Thir, _model: &Model, origin: Origin) -> Expression {
		let ty = match self.function.ty().lookup(db.upcast()) {
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
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// A call to a function with the given name.
///
/// Used only to build expressions. Becomes a `Call` once built.
pub struct LookupCall {
	/// Function name
	pub function: Identifier,
	/// Call arguments
	pub arguments: Vec<Expression>,
}

impl LookupCall {
	/// Perform the call lookup and produce a `Call`
	pub fn resolve(self, db: &dyn Thir, model: &Model, function_origin: Origin) -> (Call, Ty) {
		let args: Vec<_> = self.arguments.into_iter().collect();
		let arg_tys: Vec<_> = args.iter().map(|arg| arg.ty()).collect();
		let lookup = model
			.lookup_function(db, self.function, &arg_tys)
			.unwrap_or_else(|e| {
				panic!(
					"Function {}({}) not found:\n{}",
					self.function.pretty_print(db.upcast()),
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

		let function = Expression::new_unchecked(
			Ty::function(db.upcast(), fn_type),
			if lookup.ty_vars.is_empty() {
				ResolvedIdentifier::Function(lookup.function)
			} else {
				ResolvedIdentifier::PolymorphicFunction(
					lookup.function,
					Box::new(TyVarInstantiations::new(
						model[lookup.function].type_inst_vars(),
						&lookup.ty_vars,
					)),
				)
			},
			function_origin,
		);
		(
			Call {
				function: Box::new(function),
				arguments: args,
			},
			return_ty,
		)
	}
}

impl ExpressionBuilder for LookupCall {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let (call, return_ty) = self.resolve(db, model, origin);
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
pub struct Lambda {
	/// Domain of return type
	pub domain: Box<Domain>,
	/// Function parameters
	pub parameters: Vec<DeclarationId>,
	/// Function body
	pub body: Box<Expression>,
}

impl ExpressionBuilder for Lambda {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		Expression::new_unchecked(
			Ty::function(
				db.upcast(),
				FunctionType {
					return_type: self.domain.ty(),
					params: self.parameters.iter().map(|d| model[*d].ty()).collect(),
				},
			),
			self,
			origin,
		)
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

/// An identifier which points to the destructuring function for an annotation.
///
/// Used for building expressions only. Becomes a ResolvedIdentifier when built.
pub struct AnnotationDestructure(pub AnnotationId);

impl From<AnnotationDestructure> for ExpressionData {
	fn from(AnnotationDestructure(idx): AnnotationDestructure) -> Self {
		ResolvedIdentifier::AnnotationDestructure(idx).into()
	}
}

impl ExpressionBuilder for AnnotationDestructure {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let Self(item) = &self;
		let ann = db.type_registry().ann;
		let params = model[*item]
			.parameters
			.as_ref()
			.expect("Cannot destructure annotation atom");
		let ty = if params.len() == 1 {
			Ty::function(
				db.upcast(),
				FunctionType {
					params: Box::new([ann]),
					return_type: model[params[0]].ty(),
				},
			)
		} else {
			Ty::function(
				db.upcast(),
				FunctionType {
					params: Box::new([ann]),
					return_type: Ty::tuple(db.upcast(), params.iter().map(|d| model[*d].ty())),
				},
			)
		};
		Expression::new_unchecked(ty, self, origin)
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

/// An identifier pointing to the defining set of an enum
pub struct EnumDefiningSet(pub EnumerationId);

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

/// An identifier pointing to an enum constructor function or atom.
///
/// Used for building expressions only. Becomes a ResolvedIdentifier when built.
pub struct EnumConstructor(pub EnumMemberId, pub EnumConstructorKind);

impl From<EnumConstructor> for ExpressionData {
	fn from(EnumConstructor(idx, kind): EnumConstructor) -> Self {
		ResolvedIdentifier::EnumerationMember(idx, kind).into()
	}
}

impl ExpressionBuilder for EnumConstructor {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let Self(member, kind) = &self;
		let enum_ty = Ty::par_enum(db.upcast(), model[member.enumeration_id()].enum_type());
		let ctor = &model[*member];
		let ty = if let Some(ps) = ctor.parameters.as_ref() {
			let params = ps.iter().map(|d| model[*d].ty());
			let f = match kind {
				EnumConstructorKind::Par => FunctionType {
					params: params.collect(),
					return_type: enum_ty,
				},
				EnumConstructorKind::Var => FunctionType {
					params: params
						.map(|p| p.with_inst(db.upcast(), VarType::Var).unwrap())
						.collect(),
					return_type: enum_ty.with_inst(db.upcast(), VarType::Var).unwrap(),
				},
				EnumConstructorKind::Opt => FunctionType {
					params: params
						.map(|p| p.with_opt(db.upcast(), OptType::Opt))
						.collect(),
					return_type: enum_ty.with_opt(db.upcast(), OptType::Opt),
				},
				EnumConstructorKind::VarOpt => FunctionType {
					params: params
						.map(|p| {
							p.with_inst(db.upcast(), VarType::Var)
								.unwrap()
								.with_opt(db.upcast(), OptType::Opt)
						})
						.collect(),
					return_type: enum_ty
						.with_inst(db.upcast(), VarType::Var)
						.unwrap()
						.with_opt(db.upcast(), OptType::Opt),
				},
				EnumConstructorKind::Set => FunctionType {
					params: params
						.map(|p| Ty::par_set(db.upcast(), p).unwrap())
						.collect(),
					return_type: Ty::par_set(db.upcast(), enum_ty).unwrap(),
				},
				EnumConstructorKind::VarSet => FunctionType {
					params: params
						.map(|p| {
							Ty::par_set(db.upcast(), p)
								.unwrap()
								.with_inst(db.upcast(), VarType::Var)
								.unwrap()
						})
						.collect(),
					return_type: Ty::par_set(db.upcast(), enum_ty)
						.unwrap()
						.with_inst(db.upcast(), VarType::Var)
						.unwrap(),
				},
			};
			Ty::function(db.upcast(), f)
		} else {
			assert_eq!(
				*kind,
				EnumConstructorKind::Par,
				"Enum atom cannot be {:?}",
				kind
			);
			Ty::par_enum(db.upcast(), model[member.enumeration_id()].enum_type())
		};
		Expression::new_unchecked(ty, self, origin)
	}
}

/// An identifier which points to the destructuring function for an enumeration member.
///
/// Used for building expressions only. Becomes a ResolvedIdentifier when built.
pub struct EnumDestructure(pub EnumMemberId, pub EnumConstructorKind);

impl From<EnumDestructure> for ExpressionData {
	fn from(EnumDestructure(idx, kind): EnumDestructure) -> Self {
		ResolvedIdentifier::EnumerationDestructure(idx, kind).into()
	}
}

impl ExpressionBuilder for EnumDestructure {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let Self(member, kind) = &self;
		let enum_ty = Ty::par_enum(db.upcast(), model[member.enumeration_id()].enum_type());
		let ctor = &model[*member];
		let params = ctor.parameters.as_ref().unwrap();
		let return_type = if params.len() == 1 {
			model[params[0]].ty()
		} else {
			Ty::tuple(db.upcast(), params.iter().map(|d| model[*d].ty()))
		};
		let f = match kind {
			EnumConstructorKind::Par => FunctionType {
				params: Box::new([enum_ty]),
				return_type,
			},
			EnumConstructorKind::Var => FunctionType {
				params: Box::new([enum_ty.with_inst(db.upcast(), VarType::Var).unwrap()]),
				return_type: return_type.with_inst(db.upcast(), VarType::Var).unwrap(),
			},
			_ => unreachable!("Unsupported destructuring kind"),
		};
		Expression::new_unchecked(Ty::function(db.upcast(), f), self, origin)
	}
}

/// An identifier pointing to a function
///
/// Used for building expressions only. Becomes a `ResolvedIdentifier` when built.
pub struct FunctionIdentifier {
	/// The function to call
	pub function: FunctionId,
	/// The argument types
	pub args: Vec<Ty>,
}

impl ExpressionBuilder for FunctionIdentifier {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		let fe = model[self.function].function_entry(model);
		let ty_params = fe
			.overload
			.instantiate_ty_params(db.upcast(), &self.args)
			.expect("Failed to instantiate function");
		let ft = fe.overload.instantiate(db.upcast(), &ty_params);
		Expression::new_unchecked(
			Ty::function(db.upcast(), ft),
			if ty_params.is_empty() {
				ResolvedIdentifier::Function(self.function)
			} else {
				ResolvedIdentifier::PolymorphicFunction(
					self.function,
					Box::new(TyVarInstantiations::new(
						model[self.function].type_inst_vars(),
						&ty_params,
					)),
				)
			},
			origin,
		)
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
	/// Identifier resolves to an annotation
	Annotation(AnnotationId),
	/// Identifier resolves to an annotation destructor
	AnnotationDestructure(AnnotationId),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId),
	/// Identifier resolves to an enumeration
	Enumeration(EnumerationId),
	/// Identifier resolves to an enumeration member with the given index
	EnumerationMember(EnumMemberId, EnumConstructorKind),
	/// Identifier resolves to the destructor for an enumeration member with the given index
	EnumerationDestructure(EnumMemberId, EnumConstructorKind),
	/// Identifier resolves to a non-polymorphic function
	Function(FunctionId),
	/// Identifier resolves to a polymorphic function
	PolymorphicFunction(FunctionId, Box<TyVarInstantiations>),
}

impl ExpressionBuilder for ResolvedIdentifier {
	fn build(self, db: &dyn Thir, model: &Model, origin: Origin) -> Expression {
		match self {
			ResolvedIdentifier::Annotation(i) => i.build(db, model, origin),
			ResolvedIdentifier::AnnotationDestructure(i) => {
				AnnotationDestructure(i).build(db, model, origin)
			}
			ResolvedIdentifier::Declaration(i) => i.build(db, model, origin),
			ResolvedIdentifier::Enumeration(i) => i.build(db, model, origin),
			ResolvedIdentifier::EnumerationDestructure(i, k) => {
				EnumDestructure(i, k).build(db, model, origin)
			}
			ResolvedIdentifier::EnumerationMember(i, k) => {
				EnumConstructor(i, k).build(db, model, origin)
			}
			ResolvedIdentifier::Function(i) => {
				let fe = model[i].function_entry(model);
				let ft = fe
					.overload
					.into_function()
					.expect("Function is polymorphic");
				Expression::new_unchecked(
					Ty::function(db.upcast(), ft),
					ResolvedIdentifier::Function(i),
					origin,
				)
			}
			ResolvedIdentifier::PolymorphicFunction(i, tvs) => {
				let fe = model[i].function_entry(model);
				let ft = fe
					.overload
					.instantiate(db.upcast(), &tvs.as_map(model[i].type_inst_vars()));
				Expression::new_unchecked(
					Ty::function(db.upcast(), ft),
					ResolvedIdentifier::PolymorphicFunction(i, tvs),
					origin,
				)
			}
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
	pub fn from_ty(db: &dyn Thir, ty: Ty) -> Self {
		let is_var = ty.inst(db.upcast()).unwrap() == VarType::Var;
		let is_opt = ty.opt(db.upcast()).unwrap() == OptType::Opt;
		let is_set = ty.is_set(db.upcast());
		match (is_var, is_opt, is_set) {
			(false, false, false) => EnumConstructorKind::Par,
			(true, false, false) => EnumConstructorKind::Var,
			(false, true, false) => EnumConstructorKind::Opt,
			(true, true, false) => EnumConstructorKind::VarOpt,
			(false, false, true) => EnumConstructorKind::Set,
			(true, false, true) => EnumConstructorKind::VarSet,
			_ => unreachable!(),
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
			Generator::Iterator { collection, .. } => matches!(
				collection.ty().lookup(db.upcast()),
				TyData::Set(VarType::Var, _, _)
			),
			_ => false,
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
		/// Type of constructor
		kind: EnumConstructorKind,
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
