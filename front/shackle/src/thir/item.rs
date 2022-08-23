//! THIR representation of items

use std::ops::{Deref, DerefMut};

use crate::{
	arena::{Arena, ArenaIndex, ArenaMap},
	hir::source::Origin,
	ty::{
		EnumRef, FunctionEntry, FunctionResolutionError, FunctionType, OverloadedFunction,
		PolymorphicFunctionType, Ty, TyVarRef,
	},
	utils::impl_enum_from,
};

use super::{
	db::Thir, Domain, DomainBuilder, Expression, ExpressionBuilder, Identifier, ResolvedIdentifier,
};

/// An item of type `T`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T> {
	/// The item
	pub item: T,
	/// The data for the item
	pub data: Box<ItemData>,
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
	/// Origins of expressions
	pub origins: Arena<Origin>,
}

/// Constraint item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constraint {
	/// Constraint value
	pub expression: ArenaIndex<Expression>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

impl Item<Constraint> {
	/// Create a new constraint item with the given expression
	pub fn new(expression: &dyn ExpressionBuilder) -> Self {
		let mut data = ItemData::default();
		let idx = expression.finish(&mut data);
		Item {
			item: Constraint {
				expression: idx,
				annotations: Vec::new(),
			},
			data: Box::new(data),
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
}

impl Item<Declaration> {
	/// Create a new declaration item
	pub fn new(domain: &DomainBuilder) -> Self {
		let mut data = Box::new(ItemData::default());
		Item {
			item: Declaration {
				domain: domain.finish(&mut data),
				name: None,
				definition: None,
				annotations: Vec::new(),
			},
			data,
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

/// A enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Enumeration {
	/// Enum type
	pub enum_type: EnumRef,
	/// Right-hand-side definition
	pub definition: Option<Vec<EnumerationCase>>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

impl Item<Enumeration> {
	/// Create a new enumeration item
	pub fn new(enum_type: EnumRef) -> Self {
		Item {
			data: Box::new(ItemData::default()),
			item: Enumeration {
				annotations: Vec::new(),
				definition: None,
				enum_type,
			},
		}
	}

	/// Annotate this enumeration item
	pub fn add_annotation(&mut self, annotation: &dyn ExpressionBuilder) {
		let idx = annotation.finish(&mut self.data);
		self.annotations.push(idx);
	}
}

/// An enumeration case definition
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumerationCase {
	/// The name of this case
	pub name: Option<Identifier>,
	/// The types this case contains
	pub parameters: Vec<ArenaIndex<Item<Declaration>>>,
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
	pub parameters: Vec<ArenaIndex<Item<Declaration>>>,
	/// The body of this function
	pub body: Option<ArenaIndex<Expression>>,
	/// Annotations
	pub annotations: Vec<ArenaIndex<Expression>>,
}

impl Item<Function> {
	/// Create a new function item
	pub fn new(name: Identifier, return_type: &DomainBuilder) -> Self {
		let mut data = Box::new(ItemData::default());
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
	pub fn function_entry(&self, decls: &Arena<Item<Declaration>>) -> FunctionEntry {
		FunctionEntry {
			has_body: self.body.is_some(),
			overload: if self.type_inst_vars.is_empty() {
				OverloadedFunction::Function(FunctionType {
					params: self
						.parameters
						.iter()
						.map(|p| decls[*p].domain.ty())
						.collect(),
					return_type: self.domain.ty(),
				})
			} else {
				OverloadedFunction::PolymorphicFunction(PolymorphicFunctionType {
					ty_params: self.type_inst_vars.clone().into_boxed_slice(),
					params: self
						.parameters
						.iter()
						.map(|p| decls[*p].domain.ty())
						.collect(),
					return_type: self.domain.ty(),
				})
			},
		}
	}
}

/// Lookup a function by its signature
pub fn lookup_function(
	db: &dyn Thir,
	name: Identifier,
	args: &[Ty],
	functions: &Arena<Item<Function>>,
	declarations: &Arena<Item<Declaration>>,
) -> Result<
	(ArenaIndex<Item<Function>>, FunctionEntry, FunctionType),
	FunctionResolutionError<ArenaIndex<Item<Function>>>,
> {
	let overloads = functions.iter().filter_map(|(i, f)| {
		if f.name == name {
			Some((i, f.function_entry(&declarations)))
		} else {
			None
		}
	});
	FunctionEntry::match_fn(db.upcast(), overloads, args)
}

/// Output item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
	/// Section (always a `StringLiteral` or `None`)
	pub section: Option<ArenaIndex<Expression>>,
	/// Output value
	pub expression: ArenaIndex<Expression>,
}

impl Item<Output> {
	/// Create a new output item
	pub fn new(value: &dyn ExpressionBuilder) -> Self {
		let mut data = ItemData::default();
		Item {
			item: Output {
				section: None,
				expression: value.finish(&mut data),
			},
			data: Box::new(data),
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

impl Item<Solve> {
	/// Create a new solve item
	pub fn new() -> Self {
		Item {
			item: Solve {
				goal: Goal::Satisfy,
				annotations: Vec::new(),
			},
			data: Box::new(ItemData::default()),
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
	pub fn set_maximize(&mut self, objective: ArenaIndex<Item<Declaration>>) {
		self.goal = Goal::Maximize {
			objective: ResolvedIdentifier::Declaration(objective),
		};
	}

	/// Set this solve item to be for a minimization problem
	pub fn set_minimize(&mut self, objective: ArenaIndex<Item<Declaration>>) {
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemId {
	/// Constraint item
	Constraint(ArenaIndex<Item<Constraint>>),
	/// Declaration item
	Declaration(ArenaIndex<Item<Declaration>>),
	/// Enumeration item
	Enumeration(ArenaIndex<Item<Enumeration>>),
	/// Function item
	Function(ArenaIndex<Item<Function>>),
	/// Output item
	Output(ArenaIndex<Item<Output>>),
}

impl_enum_from!(ItemId::Constraint(ArenaIndex<Item<Constraint>>));
impl_enum_from!(ItemId::Declaration(ArenaIndex<Item<Declaration>>));
impl_enum_from!(ItemId::Enumeration(ArenaIndex<Item<Enumeration>>));
impl_enum_from!(ItemId::Function(ArenaIndex<Item<Function>>));
impl_enum_from!(ItemId::Output(ArenaIndex<Item<Output>>));
