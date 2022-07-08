//! IDs for referencing HIR nodes.

use crate::{
	arena::ArenaIndex,
	file::ModelRef,
	utils::{impl_enum_from, DebugPrint},
};

use super::{
	db::Hir, Assignment, Constraint, Declaration, Enumeration, Expression, Function, Item,
	ItemData, Model, Output, Pattern, Solve, Type,
};

/// Reference to an item local to a model.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LocalItemRef {
	/// Assignment item ID
	Assignment(ArenaIndex<Item<Assignment>>),
	/// Constraint item ID
	Constraint(ArenaIndex<Item<Constraint>>),
	/// Declaration item ID
	Declaration(ArenaIndex<Item<Declaration>>),
	/// Enumeration item ID
	Enumeration(ArenaIndex<Item<Enumeration>>),
	/// Function item ID
	Function(ArenaIndex<Item<Function>>),
	/// Function item ID
	Output(ArenaIndex<Item<Output>>),
	/// Solve item ID
	Solve(ArenaIndex<Item<Solve>>),
}

impl LocalItemRef {
	/// Get the item data for this item
	pub fn data<'a>(&self, model: &'a Model) -> &'a ItemData {
		match *self {
			LocalItemRef::Assignment(i) => &model[i].data,
			LocalItemRef::Constraint(i) => &model[i].data,
			LocalItemRef::Declaration(i) => &model[i].data,
			LocalItemRef::Enumeration(i) => &model[i].data,
			LocalItemRef::Function(i) => &model[i].data,
			LocalItemRef::Output(i) => &model[i].data,
			LocalItemRef::Solve(i) => &model[i].data,
		}
	}
}

impl_enum_from!(LocalItemRef::Assignment(ArenaIndex<Item<Assignment>>));
impl_enum_from!(LocalItemRef::Constraint(ArenaIndex<Item<Constraint>>));
impl_enum_from!(LocalItemRef::Declaration(ArenaIndex<Item<Declaration>>));
impl_enum_from!(LocalItemRef::Enumeration(ArenaIndex<Item<Enumeration>>));
impl_enum_from!(LocalItemRef::Function(ArenaIndex<Item<Function>>));
impl_enum_from!(LocalItemRef::Output(ArenaIndex<Item<Output>>));
impl_enum_from!(LocalItemRef::Solve(ArenaIndex<Item<Solve>>));

/// Global reference to an item.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemRef(salsa::InternId);

impl ItemRef {
	/// Create a new item reference
	pub fn new<T: Into<LocalItemRef>>(db: &dyn Hir, model: ModelRef, item: T) -> Self {
		db.intern_item_ref(ItemRefData(model, item.into()))
	}

	/// Get the model this item is in
	pub fn model(&self, db: &dyn Hir) -> ModelRef {
		db.lookup_intern_item_ref(*self).0
	}

	/// The the local reference to this item
	pub fn item(&self, db: &dyn Hir) -> LocalItemRef {
		db.lookup_intern_item_ref(*self).1
	}
}

impl<'a> DebugPrint<'a> for ItemRef {
	type Database = dyn Hir + 'a;
	fn debug_print(&self, db: &Self::Database) -> String {
		let ItemRefData(model, item) = db.lookup_intern_item_ref(*self);
		let model = db.lookup_model(model);
		match item {
			LocalItemRef::Assignment(i) => model[i].debug_print(db),
			LocalItemRef::Constraint(i) => model[i].debug_print(db),
			LocalItemRef::Declaration(i) => model[i].debug_print(db),
			LocalItemRef::Enumeration(i) => model[i].debug_print(db),
			LocalItemRef::Function(i) => model[i].debug_print(db),
			LocalItemRef::Output(i) => model[i].debug_print(db),
			LocalItemRef::Solve(i) => model[i].debug_print(db),
		}
	}
}

impl salsa::InternKey for ItemRef {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

/// Global reference to an item.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ItemRefData(ModelRef, LocalItemRef);

/// Global reference to an expression.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ExpressionRef(ItemRef, ArenaIndex<Expression>);

impl ExpressionRef {
	/// Create a new expression reference
	pub fn new(item: ItemRef, e: ArenaIndex<Expression>) -> Self {
		Self(item, e)
	}

	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &dyn Hir) -> EntityRef {
		EntityRef::new(db, self.0, self.1)
	}

	/// Get the item this expression belongs to
	pub fn item(&self) -> ItemRef {
		self.0
	}

	/// Get the index of the expression
	pub fn expression(&self) -> ArenaIndex<Expression> {
		self.1
	}
}

/// Global reference to a type.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TypeRef(ItemRef, ArenaIndex<Type>);

impl TypeRef {
	/// Create a new type reference
	pub fn new(item: ItemRef, t: ArenaIndex<Type>) -> Self {
		Self(item, t)
	}

	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &dyn Hir) -> EntityRef {
		EntityRef::new(db, self.0, self.1)
	}

	/// Get the item this type belongs to
	pub fn item(&self) -> ItemRef {
		self.0
	}

	/// Get the index of the type
	pub fn get_type(&self) -> ArenaIndex<Type> {
		self.1
	}
}

/// Global reference to a pattern.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PatternRef(ItemRef, ArenaIndex<Pattern>);

impl PatternRef {
	/// Create a new pattern reference
	pub fn new(item: ItemRef, p: ArenaIndex<Pattern>) -> Self {
		Self(item, p)
	}

	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &dyn Hir) -> EntityRef {
		EntityRef::new(db, self.0, self.1)
	}

	/// Get the item this pattern belongs to
	pub fn item(&self) -> ItemRef {
		self.0
	}

	/// Get the index of the pattern
	pub fn pattern(&self) -> ArenaIndex<Pattern> {
		self.1
	}
}

/// Local reference to an entity (expression, type, or pattern) owned by an item.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LocalEntityRef {
	/// Expression ID
	Expression(ArenaIndex<Expression>),
	/// Type ID
	Type(ArenaIndex<Type>),
	/// Pattern ID
	Pattern(ArenaIndex<Pattern>),
}

impl_enum_from!(LocalEntityRef::Expression(ArenaIndex<Expression>));
impl_enum_from!(LocalEntityRef::Type(ArenaIndex<Type>));
impl_enum_from!(LocalEntityRef::Pattern(ArenaIndex<Pattern>));

/// Global reference to an entity (expression, type, or pattern)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct EntityRef(salsa::InternId);

impl EntityRef {
	/// Create a new item reference
	pub fn new<T: Into<LocalEntityRef>>(db: &dyn Hir, item: ItemRef, entity: T) -> Self {
		db.intern_entity_ref(EntityRefData(item, entity.into()))
	}

	/// Get the underlying item refernce
	pub fn item(&self, db: &dyn Hir) -> ItemRef {
		db.lookup_intern_entity_ref(*self).0
	}

	/// Get the local entity reference
	pub fn entity(&self, db: &dyn Hir) -> LocalEntityRef {
		db.lookup_intern_entity_ref(*self).1
	}
}

impl salsa::InternKey for EntityRef {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

/// Global reference to an entity (expression, type, or pattern).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EntityRefData(ItemRef, LocalEntityRef);

/// Reference to an HIR node (used to map back to AST).
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum NodeRef {
	/// Model reference
	Model(ModelRef),
	/// Item reference
	Item(ItemRef),
	/// Entity reference
	Entity(EntityRef),
}

impl_enum_from!(NodeRef::Model(ModelRef));
impl_enum_from!(NodeRef::Item(ItemRef));
impl_enum_from!(NodeRef::Entity(EntityRef));
