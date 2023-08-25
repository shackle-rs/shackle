//! THIR representation of items

use std::ops::{Deref, DerefMut};

use crate::{
	arena::ArenaIndex,
	thir::{db::Thir, source::Origin},
	ty::{
		EnumRef, FunctionEntry, FunctionType, OverloadedFunction, PolymorphicFunctionType, Ty,
		TyVar,
	},
	utils::impl_enum_from,
};

use super::{domain::Domain, Annotations, Expression, Identifier, Marker, Model};

/// An item of type `T`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T> {
	item: T,
	origin: Origin,
}

impl<T> Deref for Item<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.item
	}
}

impl<T> DerefMut for Item<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.item
	}
}

impl<T> Item<T> {
	/// Create a new item
	pub fn new(item: T, origin: impl Into<Origin>) -> Self {
		Self {
			item,
			origin: origin.into(),
		}
	}

	/// Get the origin of this item
	pub fn origin(&self) -> Origin {
		self.origin
	}

	/// Get the inner origin and value
	pub fn into_inner(self) -> (Origin, T) {
		(self.origin, self.item)
	}
}

/// Annotation item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation<T: Marker = ()> {
	constructor: Constructor<T>,
}

/// An annotation item and the data it owns
pub type AnnotationItem<T = ()> = Item<Annotation<T>>;

/// ID of an annotation item
pub type AnnotationId<T = ()> = ArenaIndex<AnnotationItem<T>>;

impl<T: Marker> Deref for Annotation<T> {
	type Target = Constructor<T>;
	fn deref(&self) -> &Self::Target {
		&self.constructor
	}
}

impl<T: Marker> DerefMut for Annotation<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.constructor
	}
}

impl<T: Marker> From<Constructor<T>> for Annotation<T> {
	fn from(constructor: Constructor<T>) -> Self {
		assert!(constructor.name.is_some());
		Self { constructor }
	}
}

impl<T: Marker> Annotation<T> {
	/// Create a new annotation item with the given name
	pub fn new(name: Identifier) -> Self {
		Self {
			constructor: Constructor {
				name: Some(name),
				parameters: None,
			},
		}
	}
}

/// Constraint item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint<T: Marker = ()> {
	expression: Expression<T>,
	annotations: Annotations<T>,
	top_level: bool,
}

/// A constraint item and the data it owns
pub type ConstraintItem<T = ()> = Item<Constraint<T>>;

impl<T: Marker> Constraint<T> {
	/// Create a constraint item.
	///
	/// Takes an allocator since the expression has to be set to create the item.
	pub fn new(top_level: bool, expression: Expression<T>) -> Self {
		Self {
			expression,
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Get the constraint's value
	pub fn expression(&self) -> &Expression<T> {
		&self.expression
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
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
pub type ConstraintId<T = ()> = ArenaIndex<ConstraintItem<T>>;

/// A declaration item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration<T: Marker = ()> {
	domain: Domain<T>,
	name: Option<Identifier>,
	definition: Option<Expression<T>>,
	annotations: Annotations<T>,
	top_level: bool,
}

/// A declaration item and the data it owns
pub type DeclarationItem<T = ()> = Item<Declaration<T>>;

/// ID of a declaration item
pub type DeclarationId<T = ()> = ArenaIndex<DeclarationItem<T>>;

impl<T: Marker> Declaration<T> {
	/// Create a new declaration item.
	pub fn new(top_level: bool, domain: Domain<T>) -> Self {
		Self {
			domain,
			name: None,
			definition: None,
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Create a new declaration to hold an expression
	pub fn from_expression(db: &dyn Thir, top_level: bool, expression: Expression<T>) -> Self {
		Self {
			domain: Domain::unbounded(db, expression.origin(), expression.ty()),
			name: None,
			definition: Some(expression),
			annotations: Annotations::default(),
			top_level,
		}
	}

	/// Get the domain of this declaration
	pub fn domain(&self) -> &Domain<T> {
		&self.domain
	}

	/// Set the domain of this declaration
	pub fn set_domain(&mut self, domain: Domain<T>) {
		self.domain = domain
	}

	/// Get the type of this declaration
	pub fn ty(&self) -> Ty {
		self.domain().ty()
	}

	/// Get declaration name
	pub fn name(&self) -> Option<Identifier> {
		self.name
	}

	/// Set declaration name
	pub fn set_name(&mut self, name: Identifier) {
		self.name = Some(name)
	}

	/// Remove name
	pub fn remove_name(&mut self) {
		self.name = None;
	}

	/// Get the RHS definition of this declaration
	pub fn definition(&self) -> Option<&Expression<T>> {
		self.definition.as_ref()
	}

	/// Set the RHS definition of this declaration
	pub fn set_definition(&mut self, definition: Expression<T>) {
		self.definition = Some(definition);
	}

	/// Remove RHS definition for this declaration
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Remove the RHS definition and return it (if there was one)
	pub fn take_definition(&mut self) -> Option<Expression<T>> {
		self.definition.take()
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
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
	pub fn validate(&self, db: &dyn Thir) {
		if let Some(rhs) = self.definition() {
			let ty = rhs.ty();
			assert!(
				ty.is_subtype_of(db.upcast(), self.ty()),
				"RHS type {} does not match declaration LHS type {}",
				ty.pretty_print(db.upcast()),
				self.ty().pretty_print(db.upcast())
			);
		}
	}
}

/// An enumeration item and the data it owns
pub type EnumerationItem<T = ()> = Item<Enumeration<T>>;

/// ID of an enumeration item
pub type EnumerationId<T = ()> = ArenaIndex<EnumerationItem<T>>;

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration<T: Marker = ()> {
	enum_type: EnumRef,
	definition: Option<Vec<Constructor<T>>>,
	annotations: Annotations<T>,
}

impl<T: Marker> Enumeration<T> {
	/// Create a new enumeration item
	pub fn new(enum_type: EnumRef) -> Self {
		Self {
			annotations: Annotations::default(),
			definition: None,
			enum_type,
		}
	}

	/// Get the enum type for this enum
	pub fn enum_type(&self) -> EnumRef {
		self.enum_type
	}

	/// Get the definition of the enum
	pub fn definition(&self) -> Option<&[Constructor<T>]> {
		self.definition.as_ref().map(|d| &d[..])
	}

	/// Set the definition of this enum
	pub fn set_definition(&mut self, constructors: impl IntoIterator<Item = Constructor<T>>) {
		self.definition = Some(constructors.into_iter().collect())
	}

	/// Add the given constructor to this enum
	pub fn add_constructor(&mut self, constructor: Constructor<T>) {
		if let Some(def) = self.definition.as_mut() {
			def.push(constructor);
		} else {
			self.definition = Some(vec![constructor]);
		}
	}

	/// Remove the constructor with the given index
	pub fn remove_constructor(&mut self, index: usize) {
		self.definition
			.as_mut()
			.expect("No definition for enum")
			.remove(index);
	}

	/// Remove the definition of this enum
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
		&mut self.annotations
	}
}

/// A constructor (either atomic or a constructor function)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Constructor<T: Marker = ()> {
	/// The name of this constructor
	pub name: Option<Identifier>,
	/// The constructor function parameters, or `None` if this is atomic
	pub parameters: Option<Vec<DeclarationId<T>>>,
}

/// Function name or identifier for anonymous function
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FunctionName {
	/// Named function.
	Named(Identifier),
	/// Anonymous function.
	///
	/// Allows us to use the same ID for overloading
	Anonymous(u32),
}

impl FunctionName {
	/// Create a function name
	pub fn new(identifier: Identifier) -> Self {
		Self::Named(identifier)
	}

	/// Create a fresh anonymous function name
	pub fn anonymous() -> Self {
		static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
		Self::Anonymous(COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
	}

	/// Pretty print function name
	pub fn pretty_print(&self, db: &dyn Thir) -> String {
		match self {
			FunctionName::Named(identifier) => identifier.pretty_print(db.upcast()),
			FunctionName::Anonymous(v) => format!("FN_{}", v),
		}
	}

	/// Get a mangled identifier for this function
	pub fn mangled(&self, db: &dyn Thir, params: impl IntoIterator<Item = Ty>) -> Identifier {
		let base = match self {
			FunctionName::Named(identifier) => identifier.lookup(db.upcast()),
			FunctionName::Anonymous(v) => format!("FN_{}", v),
		};
		Identifier::new(
			format!(
				"{}<{}>",
				base,
				params
					.into_iter()
					.map(|ty| ty.pretty_print(db.upcast()))
					.collect::<Vec<_>>()
					.join(", ")
			),
			db.upcast(),
		)
	}

	/// Get this name but inversed
	pub fn inversed(&self, db: &dyn Thir) -> Self {
		match *self {
			FunctionName::Named(i) => FunctionName::Named(i.inversed(db.upcast())),
			_ => Self::anonymous(),
		}
	}
}

impl From<Identifier> for FunctionName {
	fn from(identifier: Identifier) -> Self {
		FunctionName::new(identifier)
	}
}

impl PartialEq<Identifier> for FunctionName {
	fn eq(&self, other: &Identifier) -> bool {
		if let Self::Named(identifier) = self {
			identifier == other
		} else {
			false
		}
	}
}

/// Function item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function<T: Marker = ()> {
	domain: Domain<T>,
	name: FunctionName,
	type_inst_vars: Vec<TyVar>,
	parameters: Vec<DeclarationId<T>>,
	body: Option<Expression<T>>,
	annotations: Annotations<T>,
	top_level: bool,
	is_specialisation: bool,
	mangled_param_tys: Option<Vec<Ty>>,
}

/// A function item and the data it owns
pub type FunctionItem<T = ()> = Item<Function<T>>;

/// ID of a function item
pub type FunctionId<T = ()> = ArenaIndex<FunctionItem<T>>;

impl<T: Marker> Function<T> {
	/// Create a new function item.
	pub fn new(name: FunctionName, domain: Domain<T>) -> Self {
		Self {
			annotations: Annotations::default(),
			body: None,
			domain,
			name,
			parameters: Vec::new(),
			type_inst_vars: Vec::new(),
			top_level: true,
			is_specialisation: false,
			mangled_param_tys: None,
		}
	}

	/// Create an anonymous lambda function
	pub fn lambda(
		domain: Domain<T>,
		parameters: Vec<DeclarationId<T>>,
		body: Expression<T>,
	) -> Self {
		Self {
			annotations: Annotations::default(),
			body: Some(body),
			domain,
			name: FunctionName::anonymous(),
			parameters,
			type_inst_vars: Vec::new(),
			top_level: false,
			is_specialisation: false,
			mangled_param_tys: None,
		}
	}

	/// Whether this is a top-level function, or a local function
	pub fn top_level(&self) -> bool {
		self.top_level
	}

	/// Get the name of this function
	pub fn name(&self) -> FunctionName {
		self.name
	}

	/// Set the name of this function
	pub fn set_name(&mut self, name: Identifier) {
		self.name = FunctionName::new(name);
	}

	/// Whether or not this function is the result of type specialisation
	pub fn is_specialisation(&self) -> bool {
		self.is_specialisation
	}

	/// Set whether or not this function is the result of type specialisation
	pub fn set_specialised(&mut self, specialised: bool) {
		self.is_specialisation = specialised;
	}

	/// Get the parameter types as stored for name mangling purposes
	pub fn mangled_param_tys(&self) -> Option<&[Ty]> {
		self.mangled_param_tys.as_deref()
	}

	/// Store the given parameter types for name mangling purposes
	pub fn set_mangled_param_tys(&mut self, tys: Vec<Ty>) {
		self.mangled_param_tys = Some(tys);
	}

	/// Get the type-inst var with the given index
	pub fn type_inst_var(&self, index: usize) -> &TyVar {
		&self.type_inst_vars[index]
	}

	/// Get the type-inst vars for this function
	pub fn type_inst_vars(&self) -> &[TyVar] {
		&self.type_inst_vars[..]
	}

	/// Set the type-inst vars for this function
	pub fn set_type_inst_vars(&mut self, ty_vars: impl IntoIterator<Item = TyVar>) {
		self.type_inst_vars = ty_vars.into_iter().collect();
	}

	/// Add a type-inst var to this function
	pub fn add_type_inst_var(&mut self, ty_var: TyVar) {
		self.type_inst_vars.push(ty_var);
	}

	/// Whether or not this function is polymorphic
	pub fn is_polymorphic(&self) -> bool {
		!self.type_inst_vars().is_empty()
	}

	/// Get the parameters of this function
	pub fn parameters(&self) -> &[DeclarationId<T>] {
		&self.parameters
	}

	/// Set the parameters of this function
	pub fn set_parameters(&mut self, parameters: impl IntoIterator<Item = DeclarationId<T>>) {
		self.parameters = parameters.into_iter().collect();
	}

	/// Add a parameter to this function
	pub fn add_parameter(&mut self, parameter: DeclarationId<T>) {
		self.parameters.push(parameter);
	}

	/// Get the parameter with the given index
	pub fn parameter(&self, index: usize) -> DeclarationId<T> {
		self.parameters[index]
	}

	/// Get the domain of this function
	pub fn domain(&self) -> &Domain<T> {
		&self.domain
	}

	/// Set the domain of the return type of this function
	pub fn set_domain(&mut self, value: Domain<T>) {
		self.domain = value;
	}

	/// Get the return type of this function
	pub fn return_type(&self) -> Ty {
		self.domain().ty()
	}

	/// Get the RHS definition of this function
	pub fn body(&self) -> Option<&Expression<T>> {
		self.body.as_ref()
	}

	/// Set the RHS definition of this function
	pub fn set_body(&mut self, value: Expression<T>) {
		self.body = Some(value);
	}

	/// Remove RHS definition for this function
	pub fn remove_body(&mut self) {
		self.body = None;
	}

	/// Remove and return RHS definition for this function
	pub fn take_body(&mut self) -> Option<Expression<T>> {
		self.body.take()
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
		&mut self.annotations
	}

	/// Validate that the body of this function is valid
	pub fn validate(&self, db: &dyn Thir) {
		if let Some(body) = self.body() {
			let ty = body.ty();
			assert!(
				ty.is_subtype_of(db.upcast(), self.return_type()),
				"Function body type {} does not match return type {} for {}",
				ty.pretty_print(db.upcast()),
				self.return_type().pretty_print(db.upcast()),
				self.name().pretty_print(db)
			);
		}
	}

	/// Convert to a function entry
	pub fn function_entry(&self, model: &Model<T>) -> FunctionEntry {
		FunctionEntry {
			has_body: self.body.is_some(),
			overload: if self.type_inst_vars.is_empty() {
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
			},
		}
	}
}

/// Output item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output<T: Marker = ()> {
	section: Option<Expression<T>>,
	expression: Expression<T>,
}

/// An output item and the data it owns
pub type OutputItem<T = ()> = Item<Output<T>>;

/// ID of an output item
pub type OutputId<T = ()> = ArenaIndex<OutputItem<T>>;

impl<T: Marker> Output<T> {
	/// Create a new output item
	pub fn new(expression: Expression<T>) -> Self {
		Self {
			section: None,
			expression,
		}
	}

	/// Get the section of this output item (always string literal or `None`)
	pub fn section(&self) -> Option<&Expression<T>> {
		self.section.as_ref()
	}

	/// Set the section of this output item
	pub fn set_section(&mut self, section: Expression<T>) {
		self.section = Some(section);
	}

	/// Unset the section of this output item
	pub fn remove_section(&mut self) {
		self.section = None;
	}

	/// Get the expression to output
	pub fn expression(&self) -> &Expression<T> {
		&self.expression
	}

	/// Set the expression of the output item
	pub fn set_expression(&mut self, expression: Expression<T>) {
		self.expression = expression;
	}

	/// Unwrap the underlying section and expression
	pub fn into_inner(self) -> (Option<Expression<T>>, Expression<T>) {
		(self.section, self.expression)
	}
}

/// Solve item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solve<T: Marker = ()> {
	/// Solve goal
	goal: Goal<T>,
	/// Annotations
	annotations: Annotations<T>,
}

/// A solve item and the data it owns
pub type SolveItem<T = ()> = Item<Solve<T>>;

impl<T: Marker> Solve<T> {
	/// Create a new solve satisfy item
	pub fn satisfy() -> Self {
		Self {
			goal: Goal::Satisfy,
			annotations: Annotations::default(),
		}
	}

	/// Create a new solve satisfy item
	pub fn minimize(objective: DeclarationId<T>) -> Self {
		Self {
			goal: Goal::Minimize { objective },
			annotations: Annotations::default(),
		}
	}

	/// Create a new solve maximize item
	pub fn maximize(objective: DeclarationId<T>) -> Self {
		Self {
			goal: Goal::Maximize { objective },
			annotations: Annotations::default(),
		}
	}

	/// Get the annotations attached to this expression
	pub fn annotations(&self) -> &Annotations<T> {
		&self.annotations
	}

	/// Get a mutable reference to the annotations attached to this expression
	pub fn annotations_mut(&mut self) -> &mut Annotations<T> {
		&mut self.annotations
	}

	/// Get the solve goal
	pub fn goal(&self) -> &Goal<T> {
		&self.goal
	}

	/// Get the objective value
	pub fn objective(&self) -> Option<DeclarationId<T>> {
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
	pub fn set_maximize(&mut self, objective: DeclarationId<T>) {
		self.goal = Goal::Maximize { objective };
	}

	/// Set this solve item to be for a minimization problem
	pub fn set_minimize(&mut self, objective: DeclarationId<T>) {
		self.goal = Goal::Minimize { objective };
	}
}

/// Solve method and objective
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal<T: Marker = ()> {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize {
		/// Declaration of objective
		objective: DeclarationId<T>,
	},
	/// Minimize the given objective
	Minimize {
		/// Declaration of objective
		objective: DeclarationId<T>,
	},
}

/// ID of an item
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemId<T: Marker = ()> {
	/// Annotation item
	Annotation(AnnotationId<T>),
	/// Constraint item
	Constraint(ConstraintId<T>),
	/// Declaration item
	Declaration(DeclarationId<T>),
	/// Enumeration item
	Enumeration(EnumerationId<T>),
	/// Function item
	Function(FunctionId<T>),
	/// Output item
	Output(OutputId<T>),
	/// Solve item
	Solve,
}

impl_enum_from!(ItemId<T: Marker>::Annotation(AnnotationId<T>));
impl_enum_from!(ItemId<T: Marker>::Constraint(ConstraintId<T>));
impl_enum_from!(ItemId<T: Marker>::Declaration(DeclarationId<T>));
impl_enum_from!(ItemId<T: Marker>::Enumeration(EnumerationId<T>));
impl_enum_from!(ItemId<T: Marker>::Function(FunctionId<T>));
impl_enum_from!(ItemId<T: Marker>::Output(OutputId<T>));
