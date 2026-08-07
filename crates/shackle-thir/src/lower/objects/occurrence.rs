//! Class predeclaration and occurrence-plan lookup.
//!
//! Every class is predeclared before any item is collected — its `<C>_potential`
//! enum, `<C>_objects` storage array and `<C>` actual set — so cross-class
//! references resolve regardless of item order. The rest of this module is the
//! read side of `ObjectLoweringPlan`: mapping a pattern or nested path to its
//! occurrence, and registering that occurrence's enum constructors.

use shackle_hir::{
	Item,
	class_analysis::{LocalDomainSource, OccurrenceContribution, OccurrenceId},
	ids::PatternRef,
};
use shackle_ty::{EnumRef, Ty};

use super::ClassMapInfo;
use crate::{
	lower::{ItemCollector, LoweredIdentifier},
	*,
};

impl<'db> ItemCollector<'db> {
	pub(in crate::lower) fn predeclare_class(&mut self, it: shackle_hir::ClassItem<'db>) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_pattern = PatternRef::new(self.db, item, c.pattern);
		if self.class_map.contains_key(&class_pattern) {
			return;
		}
		let class_name = class_pattern.identifier(self.db).unwrap();
		let enum_name =
			Identifier::new(self.db, format!("{}_potential", class_name.lookup(self.db)));
		let obj_name = Identifier::new(self.db, format!("{}_objects", class_name.lookup(self.db)));

		let class_enum_ref = EnumRef::new(enum_name.0);
		let class_enum = Enumeration::new(class_enum_ref);
		let class_enum_idx = self
			.model
			.add_enumeration(EnumerationItem::new(class_enum, item));

		let class_objects_decl = self.add_class_objects_decl(item, obj_name);
		let class_objects_idx = self.model.add_declaration(class_objects_decl);

		// Emit the actual-set declaration at its final var-ness up front.
		// `var_actual_set_classes` reports the classes whose existence is a
		// solver decision (`var set of new`, `var opt new`, var-existence
		// nested set fields); their actual set is a `var set`. Everything else
		// (par introductions, singular `var new`) stays a par set. The HIR
		// `defining_set_ty` consults the same predicate, so the two agree. No
		// after-the-fact widening (which would leave stale-typed references)
		// is needed.
		let par_class_set_ty = Ty::par_set(self.db, Ty::par_enum(self.db, class_enum_ref)).unwrap();
		let class_set_ty = if self
			.object_lowering
			.var_actual_set_classes
			.contains(&class_pattern)
		{
			par_class_set_ty
				.make_var(self.db)
				.unwrap_or(par_class_set_ty)
		} else {
			par_class_set_ty
		};
		let mut class_set_decl =
			Declaration::new(true, Domain::unbounded(self.db, item, class_set_ty));
		class_set_decl.set_name(class_name);
		let class_set_idx = self
			.model
			.add_declaration(DeclarationItem::new(class_set_decl, item));
		let _ = self.resolutions.insert(
			class_pattern,
			LoweredIdentifier::ResolvedIdentifier(class_set_idx.into()),
		);

		let _ = self.class_map.insert(
			class_pattern,
			ClassMapInfo {
				class_enum: class_enum_idx,
				class_objects: class_objects_idx,
				class_set: class_set_idx,
			},
		);
	}

	pub(in crate::lower) fn ensure_class_predeclared(&mut self, class_pattern: PatternRef<'db>) {
		if self.class_map.contains_key(&class_pattern) {
			return;
		}
		let Item::Class(c) = class_pattern.item(self.db) else {
			unreachable!("expected class item for class pattern")
		};
		self.predeclare_class(c);
	}

	pub(in crate::lower) fn top_level_occurrence(&self, pattern: PatternRef<'db>) -> OccurrenceId {
		self.object_lowering.top_level_occurrences[&pattern]
	}

	pub(in crate::lower) fn maybe_top_level_occurrence(
		&self,
		pattern: PatternRef<'db>,
	) -> Option<OccurrenceId> {
		self.object_lowering
			.top_level_occurrences
			.get(&pattern)
			.copied()
	}

	pub(in crate::lower) fn nested_occurrence(
		&self,
		root_pattern: PatternRef<'db>,
		path: &[Identifier<'db>],
	) -> OccurrenceId {
		self.object_lowering.nested_occurrences[&(root_pattern, path.to_vec())]
	}

	pub(in crate::lower) fn maybe_nested_occurrence(
		&self,
		root_pattern: PatternRef<'db>,
		path: &[Identifier<'db>],
	) -> Option<OccurrenceId> {
		self.object_lowering
			.nested_occurrences
			.get(&(root_pattern, path.to_vec()))
			.copied()
	}

	pub(in crate::lower) fn add_occurrence_constructors(
		&mut self,
		occurrence: OccurrenceId,
		parameter_decl: DeclarationId<'db>,
	) {
		let target_classes = self.object_lowering.contributions_by_occurrence[&occurrence]
			.iter()
			.map(|contribution| contribution.target_class)
			.collect::<Vec<_>>();
		for target_class in target_classes {
			self.ensure_class_predeclared(target_class);
		}
		for contribution in &self.object_lowering.contributions_by_occurrence[&occurrence] {
			let class_enum = self.class_map[&contribution.target_class].class_enum;
			let next_index = self.model[class_enum]
				.definition()
				.map(|constructors| constructors.len())
				.unwrap_or(0);
			assert_eq!(
				next_index, contribution.constructor_index,
				"constructor order diverged from object lowering plan"
			);
			let target_name = contribution
				.target_class
				.identifier(self.db)
				.unwrap()
				.lookup(self.db);
			self.model[class_enum].add_constructor(Constructor {
				name: Some(Identifier::new(
					self.db,
					format!("{target_name}_occ_{}", occurrence.0),
				)),
				parameters: Some(vec![parameter_decl]),
			});
		}
	}

	pub(in crate::lower) fn occurrence_constructors_available(
		&self,
		occurrence: OccurrenceId,
	) -> bool {
		self.object_lowering.contributions_by_occurrence[&occurrence]
			.iter()
			.all(|contribution| {
				let class_enum = self.class_map[&contribution.target_class].class_enum;
				self.model[class_enum]
					.definition()
					.map(|constructors| constructors.len() > contribution.constructor_index)
					.unwrap_or(false)
			})
	}

	pub(in crate::lower) fn ensure_occurrence_constructors(
		&mut self,
		occurrence: OccurrenceId,
		parameter_decl: DeclarationId<'db>,
	) {
		if !self.occurrence_constructors_available(occurrence) {
			self.add_occurrence_constructors(occurrence, parameter_decl);
		}
	}

	pub(in crate::lower) fn occurrence_contribution(
		&self,
		occurrence: OccurrenceId,
		target_class: PatternRef<'db>,
	) -> &OccurrenceContribution<'db> {
		self.object_lowering.contributions_by_occurrence[&occurrence]
			.iter()
			.find(|contribution| contribution.target_class == target_class)
			.expect("missing occurrence contribution for target class")
	}

	pub(in crate::lower) fn occurrence_local_domain_source(
		&self,
		occurrence: OccurrenceId,
	) -> LocalDomainSource {
		self.object_lowering.local_domain_sources[&occurrence]
	}
}
