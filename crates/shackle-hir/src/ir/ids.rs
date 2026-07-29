//! IDs for referencing HIR entities.

use derive_more::{From, TryUnwrap, Unwrap};
use shackle_diagnostics::{SourceFile, SourceSpan};

use crate::{
	Db, ExpressionId, Identifier, Item, PatternId, TypeId,
	input::{ModelFile, resolve_includes},
};

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod entity_id {
	use super::*;
	/// Global reference to an expression.
	#[salsa::interned(debug)]
	pub struct ExpressionRef<'db> {
		/// The item containing this expression
		#[returns(copy)]
		pub item: Item<'db>,
		/// The index of this expression
		#[returns(copy)]
		pub expression: ExpressionId<'db>,
	}
}
pub use entity_id::ExpressionRef;

impl<'db> ExpressionRef<'db> {
	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &'db dyn Db) -> EntityRef<'db> {
		EntityRef::new(db, self.item(db), EntityId::from(self.expression(db)))
	}

	/// Get the source and span for emitting a diagnostic
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		self.item(db).sources(db)[self.expression(db)].source_span(db)
	}
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod type_ref {
	use super::*;
	/// Global reference to a type.
	#[salsa::interned(debug)]
	pub struct TypeRef<'db> {
		/// The item containing this type
		#[returns(copy)]
		pub item: Item<'db>,
		/// The index of this type
		#[returns(copy)]
		pub type_id: TypeId<'db>,
	}
}
pub use type_ref::TypeRef;

impl<'db> TypeRef<'db> {
	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &'db dyn Db) -> EntityRef<'db> {
		EntityRef::new(db, self.item(db), EntityId::from(self.type_id(db)))
	}

	/// Get the source and span for emitting a diagnostic
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		self.item(db).sources(db)[self.type_id(db)].source_span(db)
	}
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod pattern_ref {
	use super::*;
	/// Global reference to a pattern.
	#[salsa::interned]
	pub struct PatternRef<'db> {
		/// The item containing this pattern
		#[returns(copy)]
		pub item: Item<'db>,
		/// The index of this pattern
		#[returns(copy)]
		pub pattern: PatternId<'db>,
	}
}
pub use pattern_ref::PatternRef;

impl<'db> PatternRef<'db> {
	/// Convert into a generic entity reference
	pub fn into_entity(self, db: &'db dyn Db) -> EntityRef<'db> {
		EntityRef::new(db, self.item(db), EntityId::from(self.pattern(db)))
	}

	/// Get this pattern as an identifier if it is one
	pub fn identifier(&self, db: &'db dyn Db) -> Option<Identifier<'db>> {
		let item = self.item(db);
		let data = item.data(db);
		data[self.pattern(db)].identifier()
	}

	/// Get references to this pattern (excluding the pattern itself)
	pub fn references(&self, db: &'db dyn Db) -> Vec<EntityRef<'db>> {
		let mut result = Vec::new();
		for model in resolve_includes(db).iter() {
			for reference_item in model.hir(db).items(db).iter() {
				for reference in reference_item.types(db).reverse_resolutions(*self) {
					result.push(EntityRef::new(db, reference_item, reference));
				}
			}
		}
		result
	}

	/// Get the source and span for emitting a diagnostic
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		self.item(db).sources(db)[self.pattern(db)].source_span(db)
	}
}

impl<'db> std::fmt::Debug for PatternRef<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		crate::db::with_attached_database(|db| {
			f.debug_struct("PatternRef")
				.field("item", &self.item(db).origin(db))
				.field("pattern", &self.pattern(db))
				.finish()
		})
		.unwrap_or_else(|| f.debug_struct("PatternRef").finish())
	}
}

/// Local reference to an entity (expression, type, or pattern) owned by an item.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, From, salsa::SalsaValue, Unwrap, TryUnwrap)]
pub enum EntityId<'db> {
	/// Expression ID
	Expression(ExpressionId<'db>),
	/// Type ID
	Type(TypeId<'db>),
	/// Pattern ID
	Pattern(PatternId<'db>),
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod entity_ref {
	use super::*;
	/// Global reference to an entity (expression, type, or pattern)
	#[salsa::interned(debug)]
	pub struct EntityRef<'db> {
		/// THe item this entity is owned by
		#[returns(copy)]
		pub item: Item<'db>,
		/// The local entity ID
		#[returns(copy)]
		pub entity: EntityId<'db>,
	}
}
pub use entity_ref::EntityRef;

impl<'db> EntityRef<'db> {
	/// Get as an `ExpressionRef` if this is one
	pub fn as_expression_ref(&self, db: &'db dyn Db) -> Option<ExpressionRef<'db>> {
		match self.entity(db) {
			EntityId::Expression(e) => Some(ExpressionRef::new(db, self.item(db), e)),
			_ => None,
		}
	}

	/// Get as an `TypeRef` if this is one
	pub fn as_type_ref(&self, db: &'db dyn Db) -> Option<TypeRef<'db>> {
		match self.entity(db) {
			EntityId::Type(t) => Some(TypeRef::new(db, self.item(db), t)),
			_ => None,
		}
	}

	/// Get as an `PatternRef` if this is one
	pub fn as_pattern_ref(&self, db: &'db dyn Db) -> Option<PatternRef<'db>> {
		match self.entity(db) {
			EntityId::Pattern(p) => Some(PatternRef::new(db, self.item(db), p)),
			_ => None,
		}
	}

	/// Get the pattern that defines this entity if there is one
	pub fn declaration(&self, db: &'db dyn Db) -> Option<PatternRef<'db>> {
		match self.entity(db) {
			EntityId::Expression(e) => {
				let types = self.item(db).types(db);
				types.name_resolution(e)
			}
			EntityId::Pattern(p) => {
				let types = self.item(db).types(db);
				Some(
					types
						.pattern_resolution(p)
						.unwrap_or_else(|| PatternRef::new(db, self.item(db), p)),
				)
			}
			EntityId::Type(_) => None,
		}
	}

	/// Get the model file this entity is defined in
	pub fn model_file(&self, db: &'db dyn Db) -> ModelFile {
		self.item(db).model_file(db)
	}

	/// Get the source and span for emitting a diagnostic
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		match self.entity(db) {
			EntityId::Expression(i) => self.item(db).sources(db)[i].source_span(db),
			EntityId::Type(i) => self.item(db).sources(db)[i].source_span(db),
			EntityId::Pattern(i) => self.item(db).sources(db)[i].source_span(db),
		}
	}
}

/// Reference to an HIR node (used to map back to AST).
#[derive(
	Copy,
	Clone,
	Debug,
	Hash,
	PartialEq,
	Eq,
	From,
	Unwrap,
	TryUnwrap,
	salsa::Supertype,
	salsa::SalsaValue,
)]
pub enum NodeRef<'db> {
	/// Model reference
	Model(ModelFile),
	/// Item reference
	Item(Item<'db>),
	/// Entity reference
	Entity(EntityRef<'db>),
}

impl<'db> NodeRef<'db> {
	/// Get the model file this node is defined in
	pub fn model_file(&self, db: &'db dyn Db) -> ModelFile {
		match *self {
			NodeRef::Model(m) => m,
			NodeRef::Item(i) => i.model_file(db),
			NodeRef::Entity(e) => e.model_file(db),
		}
	}

	/// Get the source and span for emitting a diagnostic
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		match *self {
			NodeRef::Model(m) => (m.source_file(db), SourceSpan::new(0.into(), 0)),
			NodeRef::Item(i) => i.origin(db).source_span(db),
			NodeRef::Entity(e) => e.source_span(db),
		}
	}
}
