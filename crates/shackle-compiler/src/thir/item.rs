//! THIR representation of items

use std::ops::{Deref, DerefMut};

use crate::{
	arena::{Arena, ArenaIndex, ArenaMap},
	ty::{
		EnumRef, FunctionEntry, FunctionType, OverloadedFunction, PolymorphicFunctionType, TyVarRef,
	},
	utils::impl_enum_from,
};

use super::{
	source::Origin, Domain, DomainBuilder, Expression, ExpressionBuilder, Identifier, Model,
	ResolvedIdentifier,
};

/// An item of type `T`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T> {
	/// The item
	pub item: T,
	/// The data for the item
	pub data: Box<ItemData>,
	/// The HIR node which produced this item
	pub origin: Origin,
}

impl<T> Item<T> {
	/// Add an expression to this item
	pub fn add_expression(&mut self, expression: Expression) -> ArenaIndex<Expression> {
		self.data.expressions.insert(expression)
	}
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

/// Data for an item
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct ItemData {
	/// Allocation for expressions
	pub expressions: Arena<Expression>,
	/// Annotations for a given expression
	pub annotations: ArenaMap<Expression, Vec<ArenaIndex<Expression>>>,
}

/// Annotation item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
	/// The constructor for this annotation
	pub constructor: Constructor,
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
		Item {
			item: Annotation {
				constructor: Constructor {
					name: Some(name),
					parameters: None,
				},
			},
			data: Box::default(),
			origin: origin.into(),
		}
	}
}

/// Constraint item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint {
	/// Constraint value
	pub expression: ArenaIndex<Expression>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
	/// Whether this is a top-level constraint (otherwise it inside a let)
	pub top_level: bool,
}

/// A constraint item and the data it owns
pub type ConstraintItem = Item<Constraint>;

/// ID of a constraint item
pub type ConstraintId = ArenaIndex<ConstraintItem>;

impl ConstraintItem {
	/// Create a new constraint item with the given expression
	pub fn new(
		expression: &dyn ExpressionBuilder,
		top_level: bool,
		origin: impl Into<Origin>,
	) -> Self {
		let mut data = ItemData::default();
		let idx = expression.finish(&mut data);
		Item {
			item: Constraint {
				expression: idx,
				annotations: Vec::new(),
				top_level,
			},
			data: Box::new(data),
			origin: origin.into(),
		}
	}

	/// Annotate this constraint item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
	}
}

/// A declaration item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
	/// Domain and type of the declaration
	pub domain: Domain,
	/// Name of the declaration
	pub name: Option<Identifier>,
	/// Right-hand-side definition
	pub definition: Option<ArenaIndex<Expression>>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
	/// Whether this is a top-level constraint (otherwise it inside a let)
	pub top_level: bool,
}

/// A declaration item and the data it owns
pub type DeclarationItem = Item<Declaration>;

/// ID of a declaration item
pub type DeclarationId = ArenaIndex<DeclarationItem>;

impl DeclarationItem {
	/// Create a new declaration item
	pub fn new(domain: &DomainBuilder, top_level: bool, origin: impl Into<Origin>) -> Self {
		let mut data = Box::default();
		Item {
			item: Declaration {
				domain: domain.finish(&mut data),
				name: None,
				definition: None,
				annotations: Vec::new(),
				top_level,
			},
			data,
			origin: origin.into(),
		}
	}

	/// Set the RHS definition of this declaration
	pub fn set_definition(&mut self, value: &dyn ExpressionBuilder) {
		self.definition = Some(value.finish(&mut self.data));
	}

	/// Remove RHS definition for this declaration
	pub fn remove_definition(&mut self) {
		self.definition = None;
	}

	/// Set the domain of this declaration
	pub fn set_domain(&mut self, value: &DomainBuilder) {
		self.domain = value.finish(&mut self.data);
	}

	/// Annotate this declaration item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
	}
}

/// An enumeration item and the data it owns
pub type EnumerationItem = Item<Enumeration>;

/// ID of an enumeration item
pub type EnumerationId = ArenaIndex<EnumerationItem>;

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration {
	/// Enum type
	pub enum_type: EnumRef,
	/// Right-hand-side definition
	pub definition: Option<Vec<Constructor>>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

impl EnumerationItem {
	/// Create a new enumeration item
	pub fn new(enum_type: EnumRef, origin: impl Into<Origin>) -> Self {
		Item {
			data: Box::default(),
			item: Enumeration {
				annotations: Vec::new(),
				definition: None,
				enum_type,
			},
			origin: origin.into(),
		}
	}

	/// Annotate this enumeration item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
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
	/// Domain and type of the return type
	pub domain: Domain,
	/// Pattern (always an identifier)
	pub name: Identifier,
	/// Type-inst vars
	pub type_inst_vars: Vec<TyVarRef>,
	/// Function parameters
	pub parameters: Vec<DeclarationId>,
	/// The body of this function
	pub body: Option<ArenaIndex<Expression>>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

/// A function item and the data it owns
pub type FunctionItem = Item<Function>;

/// ID of a function item
pub type FunctionId = ArenaIndex<FunctionItem>;

impl FunctionItem {
	/// Create a new function item
	pub fn new(name: Identifier, return_type: &DomainBuilder, origin: impl Into<Origin>) -> Self {
		let mut data = Box::default();
		Item {
			item: Function {
				annotations: Vec::new(),
				body: None,
				domain: return_type.finish(&mut data),
				name,
				parameters: Vec::new(),
				type_inst_vars: Vec::new(),
			},
			data,
			origin: origin.into(),
		}
	}

	/// Set the domain of the return type of this function
	pub fn set_domain(&mut self, value: &DomainBuilder) {
		self.domain = value.finish(&mut self.data);
	}

	/// Set the RHS definition of this function
	pub fn set_body(&mut self, value: &dyn ExpressionBuilder) {
		self.body = Some(value.finish(&mut self.data));
	}

	/// Remove RHS definition for this function
	pub fn remove_body(&mut self) {
		self.body = None;
	}

	/// Annotate this function item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
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
						.map(|p| model[*p].domain.ty())
						.collect(),
					return_type: self.domain.ty(),
				})
			} else {
				OverloadedFunction::PolymorphicFunction(PolymorphicFunctionType {
					ty_params: self.type_inst_vars.clone().into_boxed_slice(),
					params: self
						.parameters
						.iter()
						.map(|p| model[*p].domain.ty())
						.collect(),
					return_type: self.domain.ty(),
				})
			},
		}
	}
}

/// Output item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
	/// Section (always a `StringLiteral` or `None`)
	pub section: Option<ArenaIndex<Expression>>,
	/// Output value
	pub expression: ArenaIndex<Expression>,
}

/// An output item and the data it owns
pub type OutputItem = Item<Output>;

/// ID of an output item
pub type OutputId = ArenaIndex<OutputItem>;

impl OutputItem {
	/// Create a new output item
	pub fn new(value: &dyn ExpressionBuilder, origin: impl Into<Origin>) -> Self {
		let mut data = ItemData::default();
		Item {
			item: Output {
				section: None,
				expression: value.finish(&mut data),
			},
			data: Box::new(data),
			origin: origin.into(),
		}
	}

	/// Set the section of this output item
	pub fn set_section(&mut self, section: &dyn ExpressionBuilder) {
		self.section = Some(section.finish(&mut self.data));
	}
}

/// Solve item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solve {
	/// Solve goal
	pub goal: Goal,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

/// A solve item and the data it owns
pub type SolveItem = Item<Solve>;

impl SolveItem {
	/// Create a new solve item
	pub fn new(origin: impl Into<Origin>) -> Self {
		Item {
			item: Solve {
				goal: Goal::Satisfy,
				annotations: Vec::new(),
			},
			data: Box::default(),
			origin: origin.into(),
		}
	}

	/// Annotate this solve item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
	}

	/// Set this solve item to be for a satisfaction problem
	pub fn set_satisfy(&mut self) {
		self.goal = Goal::Satisfy;
	}

	/// Set this solve item to be for a maximization problem
	pub fn set_maximize(&mut self, objective: DeclarationId) {
		self.goal = Goal::Maximize {
			objective: ResolvedIdentifier::Declaration(objective),
		};
	}

	/// Set this solve item to be for a minimization problem
	pub fn set_minimize(&mut self, objective: DeclarationId) {
		self.goal = Goal::Minimize {
			objective: ResolvedIdentifier::Declaration(objective),
		};
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
		objective: ResolvedIdentifier,
	},
	/// Minimize the given objective
	Minimize {
		/// Declaration of objective
		objective: ResolvedIdentifier,
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
}

impl_enum_from!(ItemId::Annotation(AnnotationId));
impl_enum_from!(ItemId::Constraint(ConstraintId));
impl_enum_from!(ItemId::Declaration(DeclarationId));
impl_enum_from!(ItemId::Enumeration(EnumerationId));
impl_enum_from!(ItemId::Function(FunctionId));
impl_enum_from!(ItemId::Output(OutputId));
