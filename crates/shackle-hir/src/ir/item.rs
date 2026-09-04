//! HIR representation of items
//!
//! A top-level item `T` is represented as an `Item<T>` which holds
//! the item-specific data `T` as well as the `ItemData` storage for
//! expressions, types, and patterns.
//!
//! Non-top-level items (i.e. let items) currently do not have their own
//! `ItemData` storage, and refer to their top-level item's `ItemData` instead.
//!
//! Since each top-level item contains its own storage, these can be lowered
//! from AST independently (i.e. modifying an item does not need to cause
//! other items to be processed again). Note that currently, this is not fully
//! utilised, as the AST for an entire file is always considered to have
//! changed when modified, so always causes all items in that file to be lowered
//! again (but not ones in other files).

use derive_more::{Deref, DerefMut, From, Index, TryUnwrap, Unwrap};
use shackle_utils::{
	TypedIndex,
	arena::{Arena, ArenaMap},
};

use super::{Expression, Pattern, Type};
use crate::{
	Db, ExpressionId, Model, PatternId, TypeId,
	constants::IdentifierRegistry,
	input::ModelFile,
	source::{Origin, SourceMap},
};

/// An item in a model
#[derive(
	Copy,
	Clone,
	Debug,
	From,
	Hash,
	PartialEq,
	Eq,
	salsa::Supertype,
	salsa::SalsaValue,
	TryUnwrap,
	Unwrap,
)]
#[unwrap(ref)]
#[try_unwrap(ref)]
pub enum Item<'db> {
	/// Annotation item ID
	Annotation(AnnotationItem<'db>),
	/// Assignment item ID
	Assignment(AssignmentItem<'db>),
	/// Class declaration item ID
	Class(ClassItem<'db>),
	/// Constraint item ID
	Constraint(ConstraintItem<'db>),
	/// Declaration item ID
	Declaration(DeclarationItem<'db>),
	/// Enumeration item ID
	Enumeration(EnumerationItem<'db>),
	/// Enum assignment item ID
	EnumAssignment(EnumAssignmentItem<'db>),
	/// Function item ID
	Function(FunctionItem<'db>),
	/// Function item ID
	Output(OutputItem<'db>),
	/// Solve item ID
	Solve(SolveItem<'db>),
	/// Type alias item ID
	TypeAlias(TypeAliasItem<'db>),
}

impl<'db> Item<'db> {
	/// Get the origin of the documentation comment attached to this item, if any.
	pub fn documentation(&self, db: &'db dyn Db) -> Option<&'db Origin> {
		match self {
			Item::Annotation(i) => i.documentation(db).as_ref(),
			Item::Declaration(i) => i.documentation(db).as_ref(),
			Item::Enumeration(i) => i.documentation(db).as_ref(),
			Item::Function(i) => {
				if let Some(doc) = i.documentation(db) {
					return Some(doc);
				}
				for other in i.subsumes_others(db).iter().flat_map(|s| s.iter()) {
					if let Some(d) = other.documentation(db) {
						return Some(d);
					}
				}
				None
			}
			Item::TypeAlias(i) => i.documentation(db).as_ref(),
			_ => None,
		}
	}

	/// Get the data for this item
	pub fn data(&self, db: &'db dyn Db) -> &ItemData<'db> {
		match self {
			Item::Annotation(i) => i.annotation(db).data(),
			Item::Assignment(i) => i.assignment(db).data(),
			Item::Class(i) => i.class(db).data(),
			Item::Constraint(i) => i.constraint(db).data(),
			Item::Declaration(i) => i.declaration(db).data(),
			Item::Enumeration(i) => i.enumeration(db).data(),
			Item::EnumAssignment(i) => i.enum_assignment(db).data(),
			Item::Function(i) => i.function(db).data(),
			Item::Output(i) => i.output(db).data(),
			Item::Solve(i) => i.solve(db).data(),
			Item::TypeAlias(i) => i.type_alias(db).data(),
		}
	}

	/// Get model this item came from
	pub fn model(&self, db: &'db dyn Db) -> Model<'db> {
		self.model_file(db).hir(db)
	}

	/// Get model file this item came from
	pub fn model_file(&self, db: &'db dyn Db) -> ModelFile {
		self.origin(db).file
	}

	/// Get the source map for this item
	pub fn sources(&self, db: &'db dyn Db) -> &'db SourceMap<'db> {
		match self {
			Item::Annotation(i) => i.sources(db),
			Item::Assignment(i) => i.sources(db),
			Item::Class(i) => i.sources(db),
			Item::Constraint(i) => i.sources(db),
			Item::Declaration(i) => i.sources(db),
			Item::Enumeration(i) => i.sources(db),
			Item::EnumAssignment(i) => i.sources(db),
			Item::Function(i) => i.sources(db),
			Item::Output(i) => i.sources(db),
			Item::Solve(i) => i.sources(db),
			Item::TypeAlias(i) => i.sources(db),
		}
	}

	/// Get the origin of this item
	pub fn origin(&self, db: &'db dyn Db) -> &'db Origin {
		match self {
			Item::Annotation(i) => i.origin(db),
			Item::Assignment(i) => i.origin(db),
			Item::Class(i) => i.origin(db),
			Item::Constraint(i) => i.origin(db),
			Item::Declaration(i) => i.origin(db),
			Item::Enumeration(i) => i.origin(db),
			Item::EnumAssignment(i) => i.origin(db),
			Item::Function(i) => i.origin(db),
			Item::Output(i) => i.origin(db),
			Item::Solve(i) => i.origin(db),
			Item::TypeAlias(i) => i.origin(db),
		}
	}

	/// Get this item with its data as a debug trait object
	pub fn get_item_with_data_as_debug(&self, db: &'db dyn Db) -> &dyn std::fmt::Debug {
		match self {
			Item::Annotation(i) => i.annotation(db),
			Item::Assignment(i) => i.assignment(db),
			Item::Class(i) => i.class(db),
			Item::Constraint(i) => i.constraint(db),
			Item::Declaration(i) => i.declaration(db),
			Item::Enumeration(i) => i.enumeration(db),
			Item::EnumAssignment(i) => i.enum_assignment(db),
			Item::Function(i) => i.function(db),
			Item::Output(i) => i.output(db),
			Item::Solve(i) => i.solve(db),
			Item::TypeAlias(i) => i.type_alias(db),
		}
	}
}

/// An item with its data
#[derive(Clone, Deref, Index, Debug, PartialEq, Eq, salsa::SalsaValue)]
pub struct ItemWithData<'db, T> {
	#[deref]
	item: T,
	#[index]
	data: ItemData<'db>,
}

impl<'db, T> ItemWithData<'db, T> {
	/// Create a new item
	pub fn new(item: T, data: ItemData<'db>) -> Self {
		Self { item, data }
	}

	/// Get the data
	pub fn data(&self) -> &ItemData<'db> {
		&self.data
	}

	/// Get the annotations for for the given expression
	pub fn annotations(
		&self,
		index: ExpressionId<'db>,
	) -> impl Iterator<Item = ExpressionId<'db>> + '_ {
		self.data.annotations(index)
	}
}

/// Storage for expressions, types and sub-items owned by an item.
#[derive(Clone, Debug, Default, PartialEq, Eq, TypedIndex, salsa::SalsaValue)]
pub struct ItemData<'db> {
	/// Allocation for expressions
	#[index_mut(ExpressionId<'db>)]
	pub expressions: Arena<Expression<'db>>,
	/// Allocation for types
	#[index_mut(TypeId<'db>)]
	pub types: Arena<Type<'db>>,
	/// Allocation for patterns
	#[index_mut(PatternId<'db>)]
	pub patterns: Arena<Pattern<'db>>,
	/// Annotations for a given expression
	pub annotations: ArenaMap<Expression<'db>, Box<[ExpressionId<'db>]>>,
}

impl<'db> ItemData<'db> {
	/// Create new item data
	pub fn new() -> Self {
		Self::default()
	}

	/// Get the annotations for for the given expression
	pub fn annotations(
		&self,
		index: ExpressionId<'db>,
	) -> impl Iterator<Item = ExpressionId<'db>> + '_ {
		self.annotations
			.get(index)
			.into_iter()
			.flat_map(|v| v.iter())
			.copied()
	}

	/// Resize arenas to be as small as possible
	pub fn shrink_to_fit(&mut self) {
		self.expressions.shrink_to_fit();
		self.types.shrink_to_fit();
		self.patterns.shrink_to_fit();
		self.annotations.shrink_to_fit();
	}
}

/// An assignment item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Assignment<'db> {
	/// Expression being assigned (usually just an identifier)
	pub assignee: ExpressionId<'db>,
	/// Right-hand-side definition
	pub definition: ExpressionId<'db>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod assignment_item {
	use super::*;
	/// An assignment item with data
	#[salsa::tracked(debug)]
	pub struct AssignmentItem<'db> {
		/// The item and data
		#[tracked]
		pub assignment: ItemWithData<'db, Assignment<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use assignment_item::AssignmentItem;

/// Constraint item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Constraint<'db> {
	/// Constraint value
	pub expression: ExpressionId<'db>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod constraint_item {
	use super::*;
	/// An constraint item with data
	#[salsa::tracked(debug)]
	pub struct ConstraintItem<'db> {
		/// The item and data
		#[tracked]
		pub constraint: ItemWithData<'db, Constraint<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use constraint_item::ConstraintItem;

/// A declaration item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Declaration<'db> {
	/// Type of declaration
	pub declared_type: TypeId<'db>,
	/// Pattern being declared (usually just an identifier)
	pub pattern: PatternId<'db>,
	/// Right-hand-side definition
	pub definition: Option<ExpressionId<'db>>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod declaration_item {
	use super::*;
	/// An declaration with data
	#[salsa::tracked(debug)]
	pub struct DeclarationItem<'db> {
		/// The item and data
		#[tracked]
		pub declaration: ItemWithData<'db, Declaration<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this declaration's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use declaration_item::DeclarationItem;

impl<'db> DeclarationItem<'db> {
	/// Check if this declaration is annotated with `:: output_only`
	pub fn output_only(&self, db: &'db dyn Db) -> Option<ExpressionId<'db>> {
		let decl = self.declaration(db);
		let ids = IdentifierRegistry::lookup(db);
		for ann in decl.annotations.iter() {
			if decl.data[*ann]
				.try_unwrap_identifier_ref()
				.ok()
				.map(|i| *i == ids.annotations.output_only)
				.unwrap_or(false)
			{
				return Some(*ann);
			}
		}
		None
	}
}

/// A constructor atom or function for an enum or annotations
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum Constructor<'db> {
	/// Atomic constructor
	Atom {
		/// Pattern being declared (always an identifier)
		pattern: PatternId<'db>,
	},
	/// Functional constructor
	Function {
		/// Pattern being declared (always an identifier)
		constructor: PatternId<'db>,
		/// Pattern for destructor (always an identifier with ^-1)
		destructor: PatternId<'db>,
		/// Constructor parameters
		parameters: Box<[Parameter<'db>]>,
	},
}

impl<'db> Constructor<'db> {
	/// Get the pattern for this constructor
	pub fn constructor_pattern(&self) -> PatternId<'db> {
		match self {
			Constructor::Atom { pattern } => *pattern,
			Constructor::Function { constructor, .. } => *constructor,
		}
	}

	/// Get the parameters for this constructor
	pub fn parameters(&self) -> impl '_ + Iterator<Item = &Parameter<'db>> {
		let params = match self {
			Constructor::Function { parameters, .. } => Some(parameters),
			_ => None,
		};
		params.into_iter().flat_map(|ps| ps.iter())
	}
}

/// An annotation item
#[derive(Clone, Debug, Deref, DerefMut, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Annotation<'db> {
	/// The constructor this annotation item declares
	#[deref]
	pub constructor: Constructor<'db>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod annotation_item {
	use super::*;
	/// An annotation item with data
	#[salsa::tracked(debug)]
	pub struct AnnotationItem<'db> {
		/// The item and data
		#[tracked]
		pub annotation: ItemWithData<'db, Annotation<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this annotation's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use annotation_item::AnnotationItem;

/// An enum item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Enumeration<'db> {
	/// Pattern being declared (an identifier)
	pub pattern: PatternId<'db>,
	/// Right-hand-side definition
	pub definition: Option<Box<[EnumConstructor<'db>]>>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod enumeration_item {
	use super::*;
	/// An enum item with data
	#[salsa::tracked(debug)]
	pub struct EnumerationItem<'db> {
		/// The item and data
		#[tracked]
		pub enumeration: ItemWithData<'db, Enumeration<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this enumeration's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use enumeration_item::EnumerationItem;

/// An assignment item for an enum
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct EnumAssignment<'db> {
	/// Expression being assigned (an identifier)
	pub assignee: ExpressionId<'db>,
	/// Enum definition
	pub definition: Box<[EnumConstructor<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod enum_assignment_item {
	use super::*;
	/// An enum assignment item with data
	#[salsa::tracked(debug)]
	pub struct EnumAssignmentItem<'db> {
		/// The item and data
		#[tracked]
		pub enum_assignment: ItemWithData<'db, EnumAssignment<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use enum_assignment_item::EnumAssignmentItem;

/// An enum constructor (i.e. can be anonymous)
#[derive(Clone, Debug, From, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum EnumConstructor<'db> {
	/// Anonymous constructor
	Anonymous {
		/// Anonymous pattern
		pattern: PatternId<'db>,
		/// Parameters
		parameters: Box<[Parameter<'db>]>,
	},
	/// Named constructor
	#[from]
	Named(Constructor<'db>),
}

impl<'db> EnumConstructor<'db> {
	/// Get the pattern for this enum constructor if there is one
	pub fn constructor_pattern(&self) -> PatternId<'db> {
		match self {
			EnumConstructor::Anonymous { pattern, .. } => *pattern,
			EnumConstructor::Named(c) => c.constructor_pattern(),
		}
	}

	/// Get the parameters for this constructor
	pub fn parameters(&self) -> impl '_ + Iterator<Item = &Parameter<'db>> {
		let params = match self {
			EnumConstructor::Anonymous { parameters, .. }
			| EnumConstructor::Named(Constructor::Function { parameters, .. }) => Some(parameters),
			_ => None,
		};
		params.into_iter().flat_map(|fs| fs.iter())
	}
}

/// Function item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Function<'db> {
	/// Return type of function
	pub return_type: TypeId<'db>,
	/// Pattern (always an identifier)
	pub pattern: PatternId<'db>,
	/// Type-inst vars
	pub type_inst_vars: Box<[TypeInstIdentifierDeclaration<'db>]>,
	/// Function parameters
	pub parameters: Box<[Parameter<'db>]>,
	/// The body of this function
	pub body: Option<ExpressionId<'db>>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod function_item {
	use super::*;
	/// A function item with data
	#[salsa::tracked(debug)]
	pub struct FunctionItem<'db> {
		/// The item and data
		#[tracked]
		pub function: ItemWithData<'db, Function<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this function's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use function_item::FunctionItem;

impl<'db> FunctionItem<'db> {
	/// Pretty print the signature of this function item
	pub fn pretty_print_signature(&self, db: &'db dyn Db) -> String {
		let function = self.function(db);
		Item::from(*self).signature(db).patterns[&function.pattern]
			.pretty_print(db, function.data()[function.pattern].identifier())
			.unwrap()
	}
}

/// Declaration of a type-inst identifier
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct TypeInstIdentifierDeclaration<'db> {
	/// The name of this identifier
	pub name: PatternId<'db>,
	/// Whether this is an anonymous tiid
	pub anonymous: bool,
	/// Whether this is an enum ID
	pub is_enum: bool,
	/// Whether this is varifiable
	pub is_varifiable: bool,
	/// Whether this is indexable
	pub is_indexable: bool,
}

/// Function parameter
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Parameter<'db> {
	/// Type of declaration
	pub declared_type: TypeId<'db>,
	/// Pattern of the parameter (usually just an identifier)
	pub pattern: Option<PatternId<'db>>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
	/// Default value for the parameter
	pub default: Option<ExpressionId<'db>>,
}

/// Output item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Output<'db> {
	/// Section (always a `StringLiteral` or `None`)
	pub section: Option<ExpressionId<'db>>,
	/// Output value
	pub expression: ExpressionId<'db>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod output_item {
	use super::*;
	/// An output item with data
	#[salsa::tracked(debug)]
	pub struct OutputItem<'db> {
		/// The item and data
		#[tracked]
		pub output: ItemWithData<'db, Output<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use output_item::OutputItem;

/// Solve item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Solve<'db> {
	/// Solve goal
	pub goal: Goal<'db>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod solve_item {
	use super::*;
	/// A solve item with data
	#[salsa::tracked(debug)]
	pub struct SolveItem<'db> {
		/// The item and data
		#[tracked]
		pub solve: ItemWithData<'db, Solve<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use solve_item::SolveItem;

/// Solve method and objective
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum Goal<'db> {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize {
		/// Accessor for `_objective`
		pattern: PatternId<'db>,
		/// Objective value
		objective: ExpressionId<'db>,
	},
	/// Minimize the given objective
	Minimize {
		/// Accessor for `_objective`
		pattern: PatternId<'db>,
		/// Objective value
		objective: ExpressionId<'db>,
	},
}

/// Type alias item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct TypeAlias<'db> {
	/// Name of this type alias
	pub name: PatternId<'db>,
	/// The aliased type
	pub aliased_type: TypeId<'db>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod type_alias_item {
	use super::*;
	/// A type alias item with data
	#[salsa::tracked(debug)]
	pub struct TypeAliasItem<'db> {
		/// The item and data
		#[tracked]
		pub type_alias: ItemWithData<'db, TypeAlias<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this type alias's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use type_alias_item::TypeAliasItem;

/// A class declaration item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Class<'db> {
	/// Name of this class (always an identifier)
	pub pattern: PatternId<'db>,
	/// Pattern bound by `this` inside the class body
	pub this_pattern: PatternId<'db>,
	/// The superclass this class extends, if any
	pub extends: Option<ExpressionId<'db>>,
	/// The attributes and constraints declared in this class
	pub items: Box<[ClassMember<'db>]>,
	/// Annotations
	pub annotations: Box<[ExpressionId<'db>]>,
}

/// A member of a class body
#[derive(Clone, Debug, Hash, PartialEq, Eq, From, salsa::SalsaValue)]
pub enum ClassMember<'db> {
	/// An attribute
	Declaration(Declaration<'db>),
	/// A constraint which must hold for every object of the class
	Constraint(Constraint<'db>),
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod class_item {
	use super::*;
	/// A class declaration item with data
	#[salsa::tracked(debug)]
	pub struct ClassItem<'db> {
		/// The item and data
		#[tracked]
		pub class: ItemWithData<'db, Class<'db>>,

		/// The source map for this item
		#[tracked]
		pub sources: SourceMap<'db>,

		/// Origin of this class's documentation comment
		pub documentation: Option<Origin>,

		/// The origin of this item
		pub origin: Origin,
	}
}
pub use class_item::ClassItem;
