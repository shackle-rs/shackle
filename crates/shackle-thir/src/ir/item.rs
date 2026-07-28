//! THIR representation of items

use std::{
	num::NonZeroU32,
	ops::{Deref, DerefMut},
};

use derive_more::{Deref, DerefMut, From};
use shackle_ty::{
	EnumRef, FunctionType, OverloadedFunction, PolymorphicFunctionType, Ty, TyData, TyVar,
};
use shackle_utils::arena::ArenaIndex;

use super::{
	Annotations, Expression, Identifier, Marker, Model,
	domain::{Domain, OptType, VarType},
};
use crate::{Db, source::Origin};

/// An item of type `T`.
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Item<'db, T> {
	item: T,
	origin: Origin<'db>,
}

impl<'db, T> Deref for Item<'db, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.item
	}
}

impl<'db, T> DerefMut for Item<'db, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.item
	}
}

impl<'db, T> Item<'db, T> {
	/// Create a new item
	pub fn new(item: T, origin: impl Into<Origin<'db>>) -> Self {
		Self {
			item,
			origin: origin.into(),
		}
	}

	/// Get the origin of this item
	pub fn origin(&self) -> Origin<'db> {
		self.origin
	}

	/// Get the inner origin and value
	pub fn into_inner(self) -> (Origin<'db>, T) {
		(self.origin, self.item)
	}
}

/// Annotation item
#[derive(Clone, Debug, PartialEq, Eq, Deref, DerefMut, salsa::Update)]
pub struct Annotation<'db, T: Marker = ()> {
	#[deref]
	constructor: Constructor<'db, T>,
}

/// An annotation item and the data it owns
pub type AnnotationItem<'db, T = ()> = Item<'db, Annotation<'db, T>>;

/// ID of an annotation item
pub type AnnotationId<'db, T = ()> = ArenaIndex<AnnotationItem<'db, T>>;

impl<'db, T: Marker> From<Constructor<'db, T>> for Annotation<'db, T> {
	fn from(constructor: Constructor<'db, T>) -> Self {
		assert!(constructor.name.is_some());
		Self { constructor }
	}
}

impl<'db, T: Marker> Annotation<'db, T> {
	/// Create a new annotation item with the given name
	pub fn new(name: Identifier<'db>) -> Self {
		Self {
			constructor: Constructor {
				name: Some(name),
				parameters: None,
			},
		}
	}
}

/// Constraint item
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Constraint<'db, T: Marker = ()> {
	expression: Expression<'db, T>,
	annotations: Annotations<'db, T>,
	top_level: bool,
}

/// A constraint item and the data it owns
pub type ConstraintItem<'db, T = ()> = Item<'db, Constraint<'db, T>>;

impl<'db, T: Marker> Constraint<'db, T> {
	/// Create a constraint item.
	///
	/// Takes an allocator since the expression has to be set to create the item.
	pub fn new(top_level: bool, expression: Expression<'db, T>) -> Self {
		Self {
			expression,
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Get the constraint's value
	pub fn expression(&self) -> &Expression<'db, T> {
		&self.expression
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}

	/// Whether or not this constraint is top-level
	pub fn top_level(&self) -> bool {
		self.top_level
	}

	/// Set whether or not this constraint is top-level
	pub fn set_top_level(&mut self, top_level: bool) {
		self.top_level = top_level;
	}
}

/// ID of a constraint item
pub type ConstraintId<'db, T = ()> = ArenaIndex<ConstraintItem<'db, T>>;

/// A declaration item
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Declaration<'db, T: Marker = ()> {
	domain: Domain<'db, T>,
	name: Option<Identifier<'db>>,
	definition: Option<Expression<'db, T>>,
	annotations: Annotations<'db, T>,
	top_level: bool,
}

/// A declaration item and the data it owns
pub type DeclarationItem<'db, T = ()> = Item<'db, Declaration<'db, T>>;

/// ID of a declaration item
pub type DeclarationId<'db, T = ()> = ArenaIndex<DeclarationItem<'db, T>>;

impl<'db, T: Marker> Declaration<'db, T> {
	/// Create a new declaration item.
	pub fn new(top_level: bool, domain: Domain<'db, T>) -> Self {
		Self {
			domain,
			name: None,
			definition: None,
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Create a new declaration to hold an expression
	pub fn from_expression(
		db: &'db dyn Db,
		top_level: bool,
		expression: Expression<'db, T>,
	) -> Self {
		Self {
			domain: Domain::unbounded(db, expression.origin(), expression.ty()),
			name: None,
			definition: Some(expression),
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Get the domain of this declaration
	pub fn domain(&self) -> &Domain<'db, T> {
		&self.domain
	}

	/// Set the domain of this declaration
	pub fn set_domain(&mut self, domain: Domain<'db, T>) {
		self.domain = domain
	}

	/// Get the type of this declaration
	pub fn ty(&self) -> Ty<'db> {
		self.domain().ty()
	}

	/// Get declaration name
	pub fn name(&self) -> Option<Identifier<'db>> {
		self.name
	}

	/// Set declaration name
	pub fn set_name(&mut self, name: Identifier<'db>) {
		self.name = Some(name)
	}

	/// Remove name
	pub fn remove_name(&mut self) {
		self.name = None;
	}

	/// Get the RHS definition of this declaration
	pub fn definition(&self) -> Option<&Expression<'db, T>> {
		self.definition.as_ref()
	}

	/// Set the RHS definition of this declaration
	pub fn set_definition(&mut self, definition: Expression<'db, T>) {
		self.definition = Some(definition);
	}

	/// Remove RHS definition for this declaration
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Remove the RHS definition and return it (if there was one)
	pub fn take_definition(&mut self) -> Option<Expression<'db, T>> {
		self.definition.take()
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}

	/// Whether or not this declaration is top-level
	pub fn top_level(&self) -> bool {
		self.top_level
	}

	/// Set whether or not this declaration is top-level
	pub fn set_top_level(&mut self, top_level: bool) {
		self.top_level = top_level;
	}

	/// Validate that the RHS is valid for this declaration
	pub fn validate(&self, db: &'db dyn Db) {
		if let Some(rhs) = self.definition() {
			let ty = rhs.ty();
			assert!(
				ty.is_subtype_of(db, self.ty()) || lowered_class_compatible(db, ty, self.ty()),
				"RHS type {} ({}) does not match declaration LHS type {}",
				ty.pretty_print(db),
				rhs.origin().pretty_print(db),
				self.ty().pretty_print(db)
			);
		}
	}
}

/// Whether `actual` and `expected` agree up to the object lowering's
/// class/potential-enum identification: the lowering represents an object of
/// class `C` as a member of the `<C>_potential` enum, so a class-typed leaf
/// on one side legitimately corresponds to an enum-typed leaf on the other
/// (with a par actual accepted for a var expected). Every other shape must
/// pass ordinary subtyping.
fn lowered_class_compatible<'db>(db: &'db dyn Db, actual: Ty<'db>, expected: Ty<'db>) -> bool {
	if actual.is_subtype_of(db, expected) {
		return true;
	}
	let insts_ok = |a: VarType, e: VarType| a == e || a == VarType::Par;
	match (actual.lookup(db), expected.lookup(db)) {
		(TyData::Enum(ai, ao, _), TyData::Class(ei, eo, _))
		| (TyData::Class(ai, ao, _), TyData::Enum(ei, eo, _)) => {
			insts_ok(*ai, *ei) && (ao == eo || (*ao == OptType::NonOpt && *eo == OptType::Opt))
		}
		(TyData::Class(ai, ao, _), TyData::Class(ei, eo, _)) => insts_ok(*ai, *ei) && ao == eo,
		(TyData::Set(ai, ao, ae), TyData::Set(ei, eo, ee)) => {
			insts_ok(*ai, *ei) && ao == eo && lowered_class_compatible(db, *ae, *ee)
		}
		(
			TyData::Array {
				opt: ao,
				dim: ad,
				element: ae,
			},
			TyData::Array {
				opt: eo,
				dim: ed,
				element: ee,
			},
		) => {
			ao == eo
				&& lowered_class_compatible(db, *ad, *ed)
				&& lowered_class_compatible(db, *ae, *ee)
		}
		(TyData::Tuple(ao, afs), TyData::Tuple(eo, efs)) => {
			ao == eo
				&& afs.len() == efs.len()
				&& afs
					.iter()
					.zip(efs.iter())
					.all(|(a, e)| lowered_class_compatible(db, *a, *e))
		}
		(TyData::Record(ao, afs), TyData::Record(eo, efs)) => {
			ao == eo
				&& afs.len() == efs.len()
				&& afs
					.iter()
					.zip(efs.iter())
					.all(|((an, a), (en, e))| an == en && lowered_class_compatible(db, *a, *e))
		}
		_ => false,
	}
}

/// An enumeration item and the data it owns
pub type EnumerationItem<'db, T = ()> = Item<'db, Enumeration<'db, T>>;

/// ID of an enumeration item
pub type EnumerationId<'db, T = ()> = ArenaIndex<EnumerationItem<'db, T>>;

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Enumeration<'db, T: Marker = ()> {
	enum_type: EnumRef<'db>,
	definition: Option<Vec<Constructor<'db, T>>>,
	annotations: Annotations<'db, T>,
}

impl<'db, T: Marker> Enumeration<'db, T> {
	/// Create a new enumeration item
	pub fn new(enum_type: EnumRef<'db>) -> Self {
		Self {
			annotations: Annotations::default(),
			definition: None,
			enum_type,
		}
	}

	/// Get the enum type for this enum
	pub fn enum_type(&self) -> EnumRef<'db> {
		self.enum_type
	}

	/// Get the definition of the enum
	pub fn definition(&self) -> Option<&[Constructor<'db, T>]> {
		self.definition.as_ref().map(|d| &d[..])
	}

	/// Set the definition of this enum
	pub fn set_definition(&mut self, constructors: impl IntoIterator<Item = Constructor<'db, T>>) {
		self.definition = Some(constructors.into_iter().collect())
	}

	/// Add the given constructor to this enum
	pub fn add_constructor(&mut self, constructor: Constructor<'db, T>) {
		if let Some(def) = self.definition.as_mut() {
			def.push(constructor);
		} else {
			self.definition = Some(vec![constructor]);
		}
	}

	/// Remove the constructor with the given index
	pub fn remove_constructor(&mut self, index: usize) {
		let _ = self
			.definition
			.as_mut()
			.expect("No definition for enum")
			.remove(index);
	}

	/// Remove the definition of this enum
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}
}

/// A constructor (either atomic or a constructor function)
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Constructor<'db, T: Marker = ()> {
	/// The name of this constructor
	pub name: Option<Identifier<'db>>,
	/// The constructor function parameters, or `None` if this is atomic
	pub parameters: Option<Vec<DeclarationId<'db, T>>>,
}

/// Function name or identifier for anonymous function
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, From, salsa::Update)]
pub enum FunctionName<'db> {
	/// Named function.
	#[from]
	Named(Identifier<'db>),
	/// Anonymous function.
	///
	/// Allows us to use the same ID for overloading
	Anonymous(u32),
}

impl<'db> FunctionName<'db> {
	/// Create a function name
	pub fn new(identifier: Identifier<'db>) -> Self {
		Self::Named(identifier)
	}

	/// Create a fresh anonymous function name
	pub fn anonymous() -> Self {
		static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
		Self::Anonymous(COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
	}

	/// Pretty print function name
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		match self {
			FunctionName::Named(identifier) => identifier.pretty_print(db),
			FunctionName::Anonymous(v) => format!("FN_{}", v),
		}
	}

	/// Get a mangled identifier for this function
	pub fn mangled(
		&self,
		db: &'db dyn Db,
		params: impl IntoIterator<Item = Ty<'db>>,
	) -> Identifier<'db> {
		let base = match self {
			FunctionName::Named(identifier) => identifier.lookup(db),
			FunctionName::Anonymous(v) => &format!("FN_{}", v),
		};
		Identifier::new(
			db,
			format!(
				"{}<{}>",
				base,
				params
					.into_iter()
					.map(|ty| ty.pretty_print(db))
					.collect::<Vec<_>>()
					.join(", ")
			),
		)
	}

	/// Get this name but inversed
	pub fn inversed(&self, db: &'db dyn Db) -> Self {
		match *self {
			FunctionName::Named(i) => FunctionName::Named(i.inversed(db)),
			_ => Self::anonymous(),
		}
	}

	/// Get this name but with `_root` appended
	pub fn root(&self, db: &'db dyn Db) -> Self {
		match *self {
			FunctionName::Named(i) => FunctionName::Named(i.root(db)),
			_ => Self::anonymous(),
		}
	}

	/// Whether or not this function name ends with `_root`
	pub fn is_root(&self, db: &'db dyn Db) -> bool {
		match *self {
			FunctionName::Named(i) => i.is_root(db),
			_ => false,
		}
	}

	/// Get this name but with `_reif` appended
	pub fn reif(&self, db: &'db dyn Db) -> Self {
		match *self {
			FunctionName::Named(i) => FunctionName::Named(i.reif(db)),
			_ => Self::anonymous(),
		}
	}

	/// Whether or not this function name ends with `_reif`
	pub fn is_reif(&self, db: &'db dyn Db) -> bool {
		match *self {
			FunctionName::Named(i) => i.is_reif(db),
			_ => false,
		}
	}

	/// Get this name but with `_imp` appended
	pub fn imp(&self, db: &'db dyn Db) -> Self {
		match *self {
			FunctionName::Named(i) => FunctionName::Named(i.imp(db)),
			_ => Self::anonymous(),
		}
	}

	/// Whether or not this function name ends with `_imp`
	pub fn is_imp(&self, db: &'db dyn Db) -> bool {
		match *self {
			FunctionName::Named(i) => i.is_imp(db),
			_ => false,
		}
	}

	/// Get as an identifier
	pub fn as_identifier(&self, db: &'db dyn Db) -> Identifier<'db> {
		match self {
			FunctionName::Named(identifier) => *identifier,
			FunctionName::Anonymous(v) => Identifier::new(db, format!("FN_{}", v)),
		}
	}
}

impl<'db> PartialEq<Identifier<'db>> for FunctionName<'db> {
	fn eq(&self, other: &Identifier<'db>) -> bool {
		if let Self::Named(identifier) = self {
			identifier == other
		} else {
			false
		}
	}
}

/// Function item
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Function<'db, T: Marker = ()> {
	domain: Domain<'db, T>,
	name: FunctionName<'db>,
	type_inst_vars: Vec<TyVar<'db>>,
	parameters: Vec<DeclarationId<'db, T>>,
	body: Option<Expression<'db, T>>,
	annotations: Annotations<'db, T>,
	top_level: bool,
	specialised_from: Option<SpecialisedFrom>,
	mangled_param_tys: Option<Vec<Ty<'db>>>,
}

/// A function item and the data it owns
pub type FunctionItem<'db, T = ()> = Item<'db, Function<'db, T>>;

/// ID of a function item
pub type FunctionId<'db, T = ()> = ArenaIndex<FunctionItem<'db, T>>;

impl<'db, T: Marker> Function<'db, T> {
	/// Create a new function item.
	pub fn new(name: FunctionName<'db>, domain: Domain<'db, T>) -> Self {
		Self {
			annotations: Annotations::default(),
			body: None,
			domain,
			name,
			parameters: Vec::new(),
			type_inst_vars: Vec::new(),
			top_level: true,
			specialised_from: None,
			mangled_param_tys: None,
		}
	}

	/// Create an anonymous lambda function
	pub fn lambda(
		domain: Domain<'db, T>,
		parameters: Vec<DeclarationId<'db, T>>,
		body: Expression<'db, T>,
	) -> Self {
		Self {
			annotations: Annotations::default(),
			body: Some(body),
			domain,
			name: FunctionName::anonymous(),
			parameters,
			type_inst_vars: Vec::new(),
			top_level: false,
			specialised_from: None,
			mangled_param_tys: None,
		}
	}

	/// Whether this is a top-level function, or a local function
	pub fn top_level(&self) -> bool {
		self.top_level
	}

	/// Get the name of this function
	pub fn name(&self) -> FunctionName<'db> {
		self.name
	}

	/// Set the name of this function
	pub fn set_name(&mut self, name: Identifier<'db>) {
		self.name = FunctionName::new(name);
	}

	/// Get the mangled name of this function
	pub fn mangled_name(&'db self, db: &'db dyn Db) -> Identifier<'db> {
		self.mangled_param_tys().map_or_else(
			move || self.name().as_identifier(db),
			move |params| self.name().mangled(db, params.iter().copied()),
		)
	}

	/// Get a value uniquely representing the function this was specialised from
	pub fn specialised_from(&self) -> Option<SpecialisedFrom> {
		self.specialised_from
	}

	/// Set a value to represent which function this was specialised from
	pub fn set_specialised(&mut self, from: Option<SpecialisedFrom>) {
		self.specialised_from = from
	}

	/// Get the parameter types as stored for name mangling purposes
	pub fn mangled_param_tys(&self) -> Option<&[Ty<'db>]> {
		self.mangled_param_tys.as_deref()
	}

	/// Store the given parameter types for name mangling purposes
	pub fn set_mangled_param_tys(&mut self, tys: Vec<Ty<'db>>) {
		self.mangled_param_tys = Some(tys);
	}

	/// Get the type-inst var with the given index
	pub fn type_inst_var(&self, index: usize) -> &TyVar<'db> {
		&self.type_inst_vars[index]
	}

	/// Get the type-inst vars for this function
	pub fn type_inst_vars(&self) -> &[TyVar<'db>] {
		&self.type_inst_vars[..]
	}

	/// Set the type-inst vars for this function
	pub fn set_type_inst_vars(&mut self, ty_vars: impl IntoIterator<Item = TyVar<'db>>) {
		self.type_inst_vars = ty_vars.into_iter().collect();
	}

	/// Add a type-inst var to this function
	pub fn add_type_inst_var(&mut self, ty_var: TyVar<'db>) {
		self.type_inst_vars.push(ty_var);
	}

	/// Whether or not this function is polymorphic
	pub fn is_polymorphic(&self) -> bool {
		!self.type_inst_vars().is_empty()
	}

	/// Get the parameters of this function
	pub fn parameters(&self) -> &[DeclarationId<'db, T>] {
		&self.parameters
	}

	/// Set the parameters of this function
	pub fn set_parameters(&mut self, parameters: impl IntoIterator<Item = DeclarationId<'db, T>>) {
		self.parameters = parameters.into_iter().collect();
	}

	/// Add a parameter to this function
	pub fn add_parameter(&mut self, parameter: DeclarationId<'db, T>) {
		self.parameters.push(parameter);
	}

	/// Get the parameter with the given index
	pub fn parameter(&self, index: usize) -> DeclarationId<'db, T> {
		self.parameters[index]
	}

	/// Get the domain of this function
	pub fn domain(&self) -> &Domain<'db, T> {
		&self.domain
	}

	/// Set the domain of the return type of this function
	pub fn set_domain(&mut self, value: Domain<'db, T>) {
		self.domain = value;
	}

	/// Get the return type of this function
	pub fn return_type(&self) -> Ty<'db> {
		self.domain().ty()
	}

	/// Get the RHS definition of this function
	pub fn body(&self) -> Option<&Expression<'db, T>> {
		self.body.as_ref()
	}

	/// Set the RHS definition of this function
	pub fn set_body(&mut self, value: Expression<'db, T>) {
		self.body = Some(value);
	}

	/// Remove RHS definition for this function
	pub fn remove_body(&mut self) {
		self.body = None;
	}

	/// Remove and return RHS definition for this function
	pub fn take_body(&mut self) -> Option<Expression<'db, T>> {
		self.body.take()
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}

	/// Validate that the body of this function is valid
	pub fn validate(&self, db: &'db dyn Db) {
		if let Some(body) = self.body() {
			let ty = body.ty();
			assert!(
				ty.is_subtype_of(db, self.return_type()),
				"Function body type {} does not match return type {} for {} ({})",
				ty.pretty_print(db),
				self.return_type().pretty_print(db),
				self.name().pretty_print(db),
				body.origin().pretty_print(db)
			);
		}
	}

	/// Convert to a function entry
	pub fn function_entry(&self, model: &Model<'db, T>) -> OverloadedFunction<'db> {
		if self.type_inst_vars.is_empty() {
			OverloadedFunction::Function(FunctionType {
				params: self.parameters.iter().map(|p| model[*p].ty()).collect(),
				return_type: self.return_type(),
			})
		} else {
			OverloadedFunction::PolymorphicFunction(PolymorphicFunctionType {
				ty_params: self.type_inst_vars.iter().map(|t| t.ty_var).collect(),
				params: self.parameters.iter().map(|p| model[*p].ty()).collect(),
				return_type: self.return_type(),
			})
		}
	}
}

/// An identifier for the polymorphic function a specialised instantiation comes from.
///
/// This lets us keep track of which specialisations came from the same polymorphic
/// definition and therefore should not be dispatched to one another.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, salsa::Update)]
pub struct SpecialisedFrom(NonZeroU32);

impl<'db, T: Marker> From<FunctionId<'db, T>> for SpecialisedFrom {
	fn from(value: FunctionId<'db, T>) -> Self {
		Self(value.into())
	}
}

/// Output item
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Output<'db, T: Marker = ()> {
	section: Option<Expression<'db, T>>,
	expression: Expression<'db, T>,
}

/// An output item and the data it owns
pub type OutputItem<'db, T = ()> = Item<'db, Output<'db, T>>;

/// ID of an output item
pub type OutputId<'db, T = ()> = ArenaIndex<OutputItem<'db, T>>;

impl<'db, T: Marker> Output<'db, T> {
	/// Create a new output item
	pub fn new(expression: Expression<'db, T>) -> Self {
		Self {
			section: None,
			expression,
		}
	}

	/// Get the section of this output item (always string literal or `None`)
	pub fn section(&self) -> Option<&Expression<'db, T>> {
		self.section.as_ref()
	}

	/// Set the section of this output item
	pub fn set_section(&mut self, section: Expression<'db, T>) {
		self.section = Some(section);
	}

	/// Unset the section of this output item
	pub fn remove_section(&mut self) {
		self.section = None;
	}

	/// Get the expression to output
	pub fn expression(&self) -> &Expression<'db, T> {
		&self.expression
	}

	/// Set the expression of the output item
	pub fn set_expression(&mut self, expression: Expression<'db, T>) {
		self.expression = expression;
	}

	/// Unwrap the underlying section and expression
	pub fn into_inner(self) -> (Option<Expression<'db, T>>, Expression<'db, T>) {
		(self.section, self.expression)
	}
}

/// Solve item
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub struct Solve<'db, T: Marker = ()> {
	/// Solve goal
	goal: Goal<'db, T>,
	/// Annotations
	annotations: Annotations<'db, T>,
}

/// A solve item and the data it owns
pub type SolveItem<'db, T = ()> = Item<'db, Solve<'db, T>>;

impl<'db, T: Marker> Solve<'db, T> {
	/// Create a new solve satisfy item
	pub fn satisfy() -> Self {
		Self {
			goal: Goal::Satisfy,
			annotations: Annotations::default(),
		}
	}

	/// Create a new solve satisfy item
	pub fn minimize(objective: DeclarationId<'db, T>) -> Self {
		Self {
			goal: Goal::Minimize { objective },
			annotations: Annotations::default(),
		}
	}

	/// Create a new solve maximize item
	pub fn maximize(objective: DeclarationId<'db, T>) -> Self {
		Self {
			goal: Goal::Maximize { objective },
			annotations: Annotations::default(),
		}
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<'db, T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<'db, T> {
		&mut self.annotations
	}

	/// Get the solve goal
	pub fn goal(&self) -> &Goal<'db, T> {
		&self.goal
	}

	/// Get the objective value
	pub fn objective(&self) -> Option<DeclarationId<'db, T>> {
		match self.goal() {
			Goal::Maximize { objective } | Goal::Minimize { objective } => Some(*objective),
			_ => None,
		}
	}

	/// Set this solve item to be for a satisfaction problem
	pub fn set_satisfy(&mut self) {
		self.goal = Goal::Satisfy;
	}

	/// Set this solve item to be for a maximization problem
	pub fn set_maximize(&mut self, objective: DeclarationId<'db, T>) {
		self.goal = Goal::Maximize { objective };
	}

	/// Set this solve item to be for a minimization problem
	pub fn set_minimize(&mut self, objective: DeclarationId<'db, T>) {
		self.goal = Goal::Minimize { objective };
	}
}

/// Solve method and objective
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
pub enum Goal<'db, T: Marker = ()> {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize {
		/// Declaration of objective
		objective: DeclarationId<'db, T>,
	},
	/// Minimize the given objective
	Minimize {
		/// Declaration of objective
		objective: DeclarationId<'db, T>,
	},
}

/// ID of an item
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, From, salsa::Update)]
pub enum ItemId<'db, T: Marker = ()> {
	/// Annotation item
	Annotation(AnnotationId<'db, T>),
	/// Constraint item
	Constraint(ConstraintId<'db, T>),
	/// Declaration item
	Declaration(DeclarationId<'db, T>),
	/// Enumeration item
	Enumeration(EnumerationId<'db, T>),
	/// Function item
	Function(FunctionId<'db, T>),
	/// Output item
	Output(OutputId<'db, T>),
	/// Solve item
	Solve,
}
