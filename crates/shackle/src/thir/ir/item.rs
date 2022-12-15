//! THIR representation of items

use std::ops::{Deref, DerefMut, Index};

use crate::{
	arena::ArenaIndex,
	thir::source::Origin,
	ty::{
		EnumRef, FunctionEntry, FunctionType, OverloadedFunction, PolymorphicFunctionType, Ty,
		TyVar,
	},
	utils::impl_enum_from,
};

use super::{Domain, Expression, ExpressionAllocator, ExpressionId, Identifier, Model};

/// An item of type `T`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T> {
	item: T,
	expressions: ExpressionAllocator,
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

impl<T> Index<ExpressionId> for Item<T> {
	type Output = Expression;
	fn index(&self, index: ExpressionId) -> &Self::Output {
		&self.expressions[index]
	}
}

impl<T> Item<T> {
	/// Get the expressions allocator
	pub fn expressions(&self) -> &ExpressionAllocator {
		&self.expressions
	}

	/// Get a mutable reference to the expressions allocator
	pub fn expressions_mut(&mut self) -> &mut ExpressionAllocator {
		&mut self.expressions
	}
}

/// Annotation item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
	constructor: Constructor,
}

/// An annotation item and the data it owns
pub type AnnotationItem = Item<Annotation>;

/// ID of an annotation item
pub type AnnotationId = ArenaIndex<AnnotationItem>;

impl Deref for Annotation {
	type Target = Constructor;
	fn deref(&self) -> &Self::Target {
		&self.constructor
	}
}

impl DerefMut for Annotation {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.constructor
	}
}

impl AnnotationItem {
	/// Create a new annotation item with the given name
	pub fn new(name: Identifier, origin: impl Into<Origin>) -> Self {
		Self {
			item: Annotation {
				constructor: Constructor {
					name: Some(name),
					parameters: None,
				},
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}
}

/// Constraint item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint {
	expression: Option<ExpressionId>,
	annotations: Vec<ExpressionId>,
	top_level: bool,
}

/// A constraint item and the data it owns
pub type ConstraintItem = Item<Constraint>;

impl ConstraintItem {
	/// Create a constraint item.
	///
	/// Takes an allocator since the expression has to be set to create the item.
	pub fn new(top_level: bool, origin: impl Into<Origin>) -> Self {
		Self {
			item: Constraint {
				expression: None,
				annotations: Vec::new(),
				top_level,
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the constraint's value
	pub fn expression(&self) -> ExpressionId {
		self.expression.expect("No expression set")
	}

	/// Set the constraint's value
	pub fn set_expression(&mut self, expression: ExpressionId) {
		self.expression = Some(expression);
	}

	/// Get the annotations
	pub fn annotations(&self) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations.iter().copied()
	}

	/// Add the given annotation
	pub fn add_annotation(&mut self, ann: ExpressionId) {
		self.annotations.push(ann);
	}

	/// Remove the given annotation
	pub fn remove_annotation(&mut self, ann: ExpressionId) {
		let found = self.annotations().position(|item| item == ann);
		if let Some(idx) = found {
			self.annotations.swap_remove(idx);
		}
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
pub type ConstraintId = ArenaIndex<ConstraintItem>;

/// A declaration item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
	domain: Option<Domain>,
	name: Option<Identifier>,
	definition: Option<ExpressionId>,
	annotations: Vec<ExpressionId>,
	top_level: bool,
}

/// A declaration item and the data it owns
pub type DeclarationItem = Item<Declaration>;

/// ID of a declaration item
pub type DeclarationId = ArenaIndex<DeclarationItem>;

impl DeclarationItem {
	/// Create a new declaration item.
	pub fn new(top_level: bool, origin: impl Into<Origin>) -> Self {
		Self {
			item: Declaration {
				domain: None,
				name: None,
				definition: None,
				annotations: Vec::new(),
				top_level,
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the domain of this declaration
	pub fn domain(&self) -> &Domain {
		self.domain.as_ref().expect("No domain set")
	}

	/// Set the domain of this declaration
	pub fn set_domain(&mut self, domain: Domain) {
		self.domain = Some(domain)
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
	pub fn definition(&self) -> Option<ExpressionId> {
		self.definition
	}

	/// Set the RHS definition of this declaration
	pub fn set_definition(&mut self, definition: ExpressionId) {
		if self.domain.is_none() {
			// Make the domain match the RHS if not set
			self.set_domain(Domain::unbounded(self.expressions()[definition].ty()));
		}
		self.definition = Some(definition);
	}

	/// Remove RHS definition for this declaration
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Get the annotations
	pub fn annotations(&self) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations.iter().copied()
	}

	/// Add the given annotation
	pub fn add_annotation(&mut self, ann: ExpressionId) {
		self.annotations.push(ann);
	}

	/// Remove the given annotation
	pub fn remove_annotation(&mut self, ann: ExpressionId) {
		let found = self.annotations().position(|item| item == ann);
		if let Some(idx) = found {
			self.annotations.swap_remove(idx);
		}
	}

	/// Whether or not this declaration is top-level
	pub fn top_level(&self) -> bool {
		self.top_level
	}

	/// Set whether or not this declaration is top-level
	pub fn set_top_level(&mut self, top_level: bool) {
		self.top_level = top_level;
	}
}

/// An enumeration item and the data it owns
pub type EnumerationItem = Item<Enumeration>;

/// ID of an enumeration item
pub type EnumerationId = ArenaIndex<EnumerationItem>;

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration {
	enum_type: EnumRef,
	definition: Option<Vec<Constructor>>,
	annotations: Vec<ExpressionId>,
}

impl EnumerationItem {
	/// Create a new enumeration item
	pub fn new(enum_type: EnumRef, origin: impl Into<Origin>) -> Self {
		Self {
			item: Enumeration {
				annotations: Vec::new(),
				definition: None,
				enum_type,
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the enum type for this enum
	pub fn enum_type(&self) -> EnumRef {
		self.enum_type
	}

	/// Get the definition of the enum
	pub fn definition(&self) -> Option<&[Constructor]> {
		self.definition.as_ref().map(|d| &d[..])
	}

	/// Set the definition of this enum
	pub fn set_definition(&mut self, constructors: impl IntoIterator<Item = Constructor>) {
		self.definition = Some(constructors.into_iter().collect())
	}

	/// Add the given constructor to this enum
	pub fn add_constructor(&mut self, constructor: Constructor) {
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
			.swap_remove(index);
	}

	/// Remove the definition of this enum
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Get the annotations
	pub fn annotations(&self) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations.iter().copied()
	}

	/// Add the given annotation
	pub fn add_annotation(&mut self, ann: ExpressionId) {
		self.annotations.push(ann);
	}

	/// Remove the given annotation
	pub fn remove_annotation(&mut self, ann: ExpressionId) {
		let found = self.annotations().position(|item| item == ann);
		if let Some(idx) = found {
			self.annotations.swap_remove(idx);
		}
	}
}

/// A constructor (either atomic or a constructor function)
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Constructor {
	/// The name of this constructor
	pub name: Option<Identifier>,
	/// The constructor function parameters, or `None` if this is atomic
	pub parameters: Option<Vec<DeclarationId>>,
}

/// Function item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
	domain: Option<Domain>,
	name: Identifier,
	type_inst_vars: Vec<TyVar>,
	parameters: Vec<DeclarationId>,
	body: Option<ExpressionId>,
	annotations: Vec<ExpressionId>,
}

/// A function item and the data it owns
pub type FunctionItem = Item<Function>;

/// ID of a function item
pub type FunctionId = ArenaIndex<FunctionItem>;

impl FunctionItem {
	/// Create a new function item.
	pub fn new(name: Identifier, origin: impl Into<Origin>) -> Self {
		Self {
			item: Function {
				annotations: Vec::new(),
				body: None,
				domain: None,
				name,
				parameters: Vec::new(),
				type_inst_vars: Vec::new(),
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the name of this function
	pub fn name(&self) -> Identifier {
		self.name
	}

	/// Set the name of this function
	pub fn set_name(&mut self, name: Identifier) {
		self.name = name;
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

	/// Get the parameters of this function
	pub fn parameters(&self) -> &[DeclarationId] {
		&self.parameters
	}

	/// Set the parameters of this function
	pub fn set_parameters(&mut self, parameters: impl IntoIterator<Item = DeclarationId>) {
		self.parameters = parameters.into_iter().collect();
	}

	/// Get the parameter with the given index
	pub fn parameter(&self, index: usize) -> DeclarationId {
		self.parameters[index]
	}

	/// Get the domain of this function
	pub fn domain(&self) -> &Domain {
		self.domain.as_ref().expect("No domain set")
	}

	/// Set the domain of the return type of this function
	pub fn set_domain(&mut self, value: Domain) {
		self.domain = Some(value);
	}

	/// Get the RHS definition of this function
	pub fn body(&self) -> Option<ExpressionId> {
		self.body
	}

	/// Set the RHS definition of this function
	pub fn set_body(&mut self, value: ExpressionId) {
		self.body = Some(value);
	}

	/// Remove RHS definition for this function
	pub fn remove_body(&mut self) {
		self.body = None;
	}

	/// Get the annotations
	pub fn annotations(&self) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations.iter().copied()
	}

	/// Add the given annotation
	pub fn add_annotation(&mut self, ann: ExpressionId) {
		self.annotations.push(ann);
	}

	/// Remove the given annotation
	pub fn remove_annotation(&mut self, ann: ExpressionId) {
		let found = self.annotations().position(|item| item == ann);
		if let Some(idx) = found {
			self.annotations.swap_remove(idx);
		}
	}

	/// Convert to a function entry
	pub fn function_entry(&self, model: &Model) -> FunctionEntry {
		FunctionEntry {
			has_body: self.body.is_some(),
			overload: if self.type_inst_vars.is_empty() {
				OverloadedFunction::Function(FunctionType {
					params: self
						.parameters
						.iter()
						.map(|p| model[*p].domain().ty())
						.collect(),
					return_type: self.domain().ty(),
				})
			} else {
				OverloadedFunction::PolymorphicFunction(PolymorphicFunctionType {
					ty_params: self.type_inst_vars.iter().map(|t| t.ty_var).collect(),
					params: self
						.parameters
						.iter()
						.map(|p| model[*p].domain().ty())
						.collect(),
					return_type: self.domain().ty(),
				})
			},
		}
	}
}

/// Output item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
	section: Option<ExpressionId>,
	expression: Option<ExpressionId>,
}

/// An output item and the data it owns
pub type OutputItem = Item<Output>;

/// ID of an output item
pub type OutputId = ArenaIndex<OutputItem>;

impl OutputItem {
	/// Create a new output item.
	pub fn new(origin: impl Into<Origin>) -> Self {
		Self {
			item: Output {
				section: None,
				expression: None,
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the section of this output item (always string literal or `None`)
	pub fn section(&self) -> Option<ExpressionId> {
		self.section
	}

	/// Set the section of this output item
	pub fn set_section(&mut self, section: ExpressionId) {
		self.section = Some(section);
	}

	/// Unset the section of this output item
	pub fn remove_section(&mut self) {
		self.section = None;
	}

	/// Get the expression to output
	pub fn expression(&self) -> ExpressionId {
		self.expression.expect("Expression not set")
	}

	/// Set the expression of the output item
	pub fn set_expression(&mut self, expression: ExpressionId) {
		self.expression = Some(expression)
	}
}

/// Solve item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solve {
	/// Solve goal
	goal: Goal,
	/// Annotations
	annotations: Vec<ExpressionId>,
}

/// A solve item and the data it owns
pub type SolveItem = Item<Solve>;

impl SolveItem {
	/// Create a new solve satisfy item
	pub fn satisfy(origin: impl Into<Origin>) -> Self {
		Self {
			item: Solve {
				goal: Goal::Satisfy,
				annotations: Vec::new(),
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Create a new solve satisfy item
	pub fn minimize(objective: DeclarationId, origin: impl Into<Origin>) -> Self {
		Self {
			item: Solve {
				goal: Goal::Minimize { objective },
				annotations: Vec::new(),
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Create a new solve maximize item
	pub fn maximize(objective: DeclarationId, origin: impl Into<Origin>) -> Self {
		Self {
			item: Solve {
				goal: Goal::Maximize { objective },
				annotations: Vec::new(),
			},
			expressions: ExpressionAllocator::default(),
			origin: origin.into(),
		}
	}

	/// Get the annotations
	pub fn annotations(&self) -> impl '_ + Iterator<Item = ExpressionId> {
		self.annotations.iter().copied()
	}

	/// Add the given annotation
	pub fn add_annotation(&mut self, ann: ExpressionId) {
		self.annotations.push(ann);
	}

	/// Remove the given annotation
	pub fn remove_annotation(&mut self, ann: ExpressionId) {
		let found = self.annotations().position(|item| item == ann);
		if let Some(idx) = found {
			self.annotations.swap_remove(idx);
		}
	}

	/// Get the solve goal
	pub fn goal(&self) -> &Goal {
		&self.goal
	}

	/// Set this solve item to be for a satisfaction problem
	pub fn set_satisfy(&mut self) {
		self.goal = Goal::Satisfy;
	}

	/// Set this solve item to be for a maximization problem
	pub fn set_maximize(&mut self, objective: DeclarationId) {
		self.goal = Goal::Maximize { objective };
	}

	/// Set this solve item to be for a minimization problem
	pub fn set_minimize(&mut self, objective: DeclarationId) {
		self.goal = Goal::Minimize { objective };
	}
}

/// Solve method and objective
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize {
		/// Declaration of objective
		objective: DeclarationId,
	},
	/// Minimize the given objective
	Minimize {
		/// Declaration of objective
		objective: DeclarationId,
	},
}

/// ID of an item
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemId {
	/// Annotation item
	Annotation(AnnotationId),
	/// Constraint item
	Constraint(ConstraintId),
	/// Declaration item
	Declaration(DeclarationId),
	/// Enumeration item
	Enumeration(EnumerationId),
	/// Function item
	Function(FunctionId),
	/// Output item
	Output(OutputId),
	/// Solve item
	Solve,
}

impl_enum_from!(ItemId::Annotation(AnnotationId));
impl_enum_from!(ItemId::Constraint(ConstraintId));
impl_enum_from!(ItemId::Declaration(DeclarationId));
impl_enum_from!(ItemId::Enumeration(EnumerationId));
impl_enum_from!(ItemId::Function(FunctionId));
impl_enum_from!(ItemId::Output(OutputId));
