//! Counts of entities in the HIR
//!
//! Used to pre-allocate data structures in later phases
use crate::{Db, lower::lower_models};

/// Counts of entities
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct EntityCounts {
	/// Annotation item count
	pub annotations: u32,
	/// Assignment item count
	pub assignments: u32,
	/// Enum assignment item count
	pub enum_assignments: u32,
	/// Class declaration item count
	pub classes: u32,
	/// Constraint item count
	pub constraints: u32,
	/// Declaration item count
	pub declarations: u32,
	/// Enumeration item count
	pub enumerations: u32,
	/// Function item count
	pub functions: u32,
	/// Output item count
	pub outputs: u32,
	/// Solve item count
	pub solves: u32,
	/// Type alias item count
	pub type_aliases: u32,
	/// Expression count
	pub expressions: u32,
	/// (Ascribed) type count
	pub types: u32,
	/// Pattern count
	pub patterns: u32,
}

impl EntityCounts {
	/// Compute counts of entities in the HIR
	///
	/// Used to pre-allocate data structures in later phases
	pub fn lookup(db: &dyn Db) -> &Self {
		entity_counts(db)
	}
}

#[salsa::tracked]
fn entity_counts(db: &dyn Db) -> EntityCounts {
	let models = lower_models(db);
	let mut counts = EntityCounts::default();
	for model in models.iter() {
		let items = model.items(db);
		for item in items.iter() {
			match item {
				crate::ir::Item::Annotation(_) => counts.annotations += 1,
				crate::ir::Item::Assignment(_) => counts.assignments += 1,
				crate::ir::Item::EnumAssignment(_) => counts.enum_assignments += 1,
				crate::ir::Item::Class(_) => counts.classes += 1,
				crate::ir::Item::Constraint(_) => counts.constraints += 1,
				crate::ir::Item::Declaration(_) => counts.declarations += 1,
				crate::ir::Item::Enumeration(_) => counts.enumerations += 1,
				crate::ir::Item::Function(_) => counts.functions += 1,
				crate::ir::Item::Output(_) => counts.outputs += 1,
				crate::ir::Item::Solve(_) => counts.solves += 1,
				crate::ir::Item::TypeAlias(_) => counts.type_aliases += 1,
			}
			let data = item.data(db);
			counts.expressions += data.expressions.len();
			counts.types += data.types.len();
			counts.patterns += data.patterns.len();
		}
	}
	counts
}
