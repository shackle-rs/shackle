//! Object lowering: turns class items and `new` declarations into the
//! potential-enum / storage-array / actual-set encoding.
//!
//! Each class gets a `<C>_potential` storage record holding every object that
//! could exist, an actual set `<C>_objects` selecting the ones that do, and
//! occurrence functions `<C>_occ_k` mapping a contribution's slots into that
//! universe.
//!
//! Note on layout: the methods here are inherent `impl ItemCollector` blocks
//! spread across this directory's files, rather than living beside the struct
//! in `lower/mod.rs`. `ItemCollector` is a private builder whose whole impl
//! surface stays inside `crate::lower`, and this feature is ~80% of the
//! lowering; splitting the impl keeps each file reviewable without paying for
//! a wrapper type that would carry no state of its own. `ItemCollector` stays
//! declared in `lower/mod.rs` so its private fields remain visible to every
//! module here.

use rustc_hash::{FxHashMap, FxHashSet};
use shackle_hir::{
	ClassMember, Item, PatternTy,
	class_analysis::{
		LocalDomainSource, OccurrenceContribution, OccurrenceId, OccurrenceSource,
		analyse_new_objects, class_pattern_for, introduces_var_existence,
	},
	ids::PatternRef,
};
use shackle_ty::{EnumRef, Ty, TyData};

use crate::{
	lower::{ItemCollector, LoweredIdentifier, expression::ExpressionCollector},
	source::Origin,
	*,
};

mod class_item;
mod engine;
mod field_reconstruction;
mod finish;
mod guards;
mod nested_emission;
mod nested_storage;
mod new_declaration;

/// Realisation-guard request for the root-reconstruction engine: identifies
/// the root contribution's slots (`<C>_occ_<constructor_index>(p)`) so
/// defined-field aliases can be guarded on `.. in <C>`, plus the root's
/// lowered name prefix for the per-field unrealised-default witness decls.
pub(in crate::lower) struct RootRealisationGuard {
	constructor_index: usize,
	name_prefix: String,
}

/// How the reconstruction engine tests a slot's realisation
/// (`<slot> in <C>`), for the per-slot `realised` alias.
pub(in crate::lower) enum EngineRealisationTest<'db> {
	/// The iteration index is an ordinal into the contribution's constructor:
	/// the slot is `<C>_occ_<constructor_index>(<ordinal>)`. Root collections
	/// iterate this way (`p in index_set(inputs)`).
	ConstructorOrdinal {
		constructor_index: usize,
		ordinal: Expression<'db>,
	},
	/// The iteration index IS the slot identity — enum-indexed nested free
	/// storage (`p in index_set(<C>_<intro>_storage)` where the dim is the
	/// constructor's enum image), so no constructor call is needed.
	Identity(Expression<'db>),
}

/// Realisation-guard request for the generalised reconstruction engine:
/// the slot test plus the contribution's lowered name prefix for the
/// per-field unrealised-default witness decls.
pub(in crate::lower) struct EngineRealisationGuard<'db> {
	name_prefix: String,
	test: EngineRealisationTest<'db>,
}

/// How the reconstruction engine defines class-typed fields whose input
/// representation does not already match storage (`<Child>_potential`
/// identities must be minted, not read).
pub(in crate::lower) enum EngineIdentityRule<'db> {
	/// Top-level root regimes (`reconstructed_root_field_expr`):
	/// `<C>_occ_k(p)` for one-per-parent fields, prefix-sum ordinal ranges
	/// over the root inputs for flattened `set of new` collections.
	Root {
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		index_expr: Expression<'db>,
	},
	/// Depth-1 nested flattened regimes
	/// (`reconstructed_nested_flattened_field_expr`): private-universe
	/// prefix sums over roots × siblings.
	NestedFlattened {
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		current_collection: Expression<'db>,
		input_index_expr: Expression<'db>,
		child_index_expr: Expression<'db>,
	},
	/// Depth-1 nested SINGULAR regimes
	/// (`reconstructed_nested_singular_field_expr`): a `new X` attribute
	/// contributes exactly one child per parent (`OnePerParent`), so the
	/// per-object universe prefix sums over parents only — there is no
	/// sibling term and the "collection" for a parent is the single child
	/// record, not an array. Object-typed fields of that child (e.g. a
	/// `set of new B` grand-collection) still mint `<B>_potential`
	/// identities, so par nested singular storage matches the identity
	/// model the constraint lowering assumes.
	NestedSingular {
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		input_index_expr: Expression<'db>,
	},
	/// Deep (≥ depth-2) par nested regime
	/// (`reconstructed_deep_nested_field_expr`). The field-owning class is
	/// introduced two or more `new`-hops below a par root, so the depth-1
	/// builders' fixed 2-level generator stack (parent × sibling) no longer
	/// spans the path. Instead the field owner's par input records are
	/// flattened once, in canonical path order, into `flat_inputs_expr`, and
	/// each object field's `<GrandChild>_potential` identity is minted from a
	/// single 1-D prefix sum over the flat position `flat_index_expr` (`ci`) —
	/// depth-agnostic, because the flattening absorbs every intermediate hop.
	/// `full_path` (root → field owner) locates each object field's grand-child
	/// occurrence for the enum constructor.
	NestedDeep {
		root_pattern: PatternRef<'db>,
		full_path: Vec<Identifier<'db>>,
		flat_inputs_expr: Expression<'db>,
		flat_index_expr: Expression<'db>,
	},
	/// No minting regime available (element-iterating nested contexts, and
	/// free-storage reads where identities are already in storage): read the
	/// field through when the input carries it, mint a fresh free decision
	/// otherwise — the pre-engine template behaviour.
	ReadOrMint,
}

/// A class-body constraint emitted as a realised-set forall — an explicit
/// class `constraint` item, or a computed attribute's defining equation.
/// Constraint bodies are emitted during `collect_class`; Definition bodies
/// are deferred to `finish()` so the redundant forall can be DROPPED for
/// classes whose contributions all alias-define their defined fields (the
/// gated forall-drop).
pub(in crate::lower) enum ClassBodyConstraint<'db> {
	Constraint {
		expression: shackle_hir::ExpressionId<'db>,
		annotations: Vec<shackle_hir::ExpressionId<'db>>,
	},
	Definition {
		attribute: Identifier<'db>,
		value: shackle_hir::ExpressionId<'db>,
	},
	/// Data-conformance assertion for a PAR field whose declared type
	/// carries an attribute-referencing (dependent) domain: the shared
	/// storage record domain is unbounded for such fields
	/// (`field_domain_references_attribute`), so nothing else checks the
	/// supplied value. Emits
	/// `forall(this in <C>)(assert(<conformance>, "..."))` — index-set
	/// equality for a dependent array dimension, `in` for scalar/element
	/// domains — with sibling references resolving per object through the
	/// same alias scope as ordinary class-body constraints.
	DomainConformance {
		attribute: Identifier<'db>,
		declared_type: shackle_hir::TypeId<'db>,
	},
}

/// One step of a scalar-leaf access path inside a structured (tuple or
/// record) storage field, used by the leaf-wise unused-potential pins
/// (`pin_leaf_paths`).
#[derive(Copy, Clone)]
pub(in crate::lower) enum PinLeafStep<'db> {
	Tuple(i64),
	Record(Identifier<'db>),
}

/// One class storage-field declaration, as gathered by
/// `class_storage_field_decls` for the reconstruction comprehension.
#[derive(Copy, Clone)]
pub(in crate::lower) struct StorageFieldDecl<'db> {
	pub(in crate::lower) ident: Identifier<'db>,
	pattern: PatternRef<'db>,
	definition: Option<shackle_hir::ExpressionId<'db>>,
	pub(in crate::lower) declared_type: shackle_hir::TypeId<'db>,
	pub(in crate::lower) owner: Item<'db>,
}

#[derive(Copy, Clone)]
pub(in crate::lower) struct ClassMapInfo<'db> {
	pub(in crate::lower) class_enum: EnumerationId<'db>,
	pub(in crate::lower) class_objects: DeclarationId<'db>,
	class_set: DeclarationId<'db>,
}

pub(in crate::lower) struct ObjectLoweringPlan<'db> {
	pub(in crate::lower) top_level_occurrences: FxHashMap<PatternRef<'db>, OccurrenceId>,
	nested_occurrences: FxHashMap<(PatternRef<'db>, Vec<Identifier<'db>>), OccurrenceId>,
	local_domain_sources: FxHashMap<OccurrenceId, LocalDomainSource>,
	pub(in crate::lower) contributions_by_occurrence:
		FxHashMap<OccurrenceId, Vec<OccurrenceContribution<'db>>>,
	pub(in crate::lower) var_reached_classes: FxHashSet<PatternRef<'db>>,
	/// Classes whose actual-set declaration is emitted as a `var set` because
	/// their existence is a solver decision (`var set of new`, `var opt new`,
	/// or a var-existence nested set field). A strict subset of
	/// `var_reached_classes`; drives the var-ness of the class-set declaration
	/// at predeclare time so no after-the-fact widening is needed.
	var_actual_set_classes: FxHashSet<PatternRef<'db>>,
	/// Classes that are introduced only via parent-field occurrences
	/// (no top-level declaration). For these classes the actual-set
	/// is derived as `array_union(...)` over parent fields, and the
	/// class-set declaration is emitted as `var set of <class>_potential`
	/// rather than the default par-set.
	field_only_introduced_classes: FxHashSet<PatternRef<'db>>,
	/// Classes eligible for domain relocation of their OWN defined fields
	/// (the predicate is keyed on the field's *owner* class so every storage
	/// site — the owner's and every subclass's `_objects` element domain —
	/// reaches the same verdict). A class qualifies iff
	///
	/// - every contribution to it AND to every transitive subclass is
	///   engine-alias-defined. That is every top-level contribution
	///   (singular-root projections read the direct objects array) and every
	///   nested contribution whose target is var-reached (the var nested
	///   storage engine); par-reached nested targets stay conservative — the
	///   identity-mode fallback registers uninitialized full-record storage,
	///   where a relaxed element domain would leave an unpinned free
	///   decision; and
	/// - at least one contribution has unrealisable slots: a top-level
	///   `var set(..) of new` / `var opt new` root
	///   (`introduces_var_existence`), or a nested introduction whose child
	///   class has a var actual set (`var_actual_set_classes` — existence is
	///   a solver decision somewhere along the chain). Otherwise every slot
	///   is realised, the binding domain is harmless, and relocation would
	///   churn the model for nothing.
	domain_relocation_classes: FxHashSet<PatternRef<'db>>,
	/// Classes whose storage arrays can hold UNREALISED slots: the upward
	/// superclass closure of every contribution target with an unrealisable
	/// root (a field of class `O` is stored in `O`'s and every subclass's
	/// `_objects`; a subclass row can be a phantom even when `O` itself is
	/// always fully realised). Drives set-cardinality relocation
	/// (`field_relocates_set_card`): a cardinality-bounded set field stored
	/// here cannot keep its card bound in the shared element record — the
	/// canonical unrealised-slot value (`lb` = `{}`) would violate it.
	unrealisable_storage_classes: FxHashSet<PatternRef<'db>>,
	/// Source-order rank of every class, used to give the class-keyed hash
	/// maps and sets above a deterministic iteration order.
	///
	/// Iterating an `FxHashMap`/`FxHashSet` keyed by `PatternRef` walks the
	/// keys in hashed-salsa-id order. That is stable for a fixed set of
	/// interned ids but shifts as soon as interning order does — e.g. under a
	/// different standard library — so any emission driven by it produces
	/// models that differ by a reordering across machines. `analyse_new_objects`
	/// builds `class_descriptors` by walking models and items in source order,
	/// which is stable everywhere; rank by position in that list instead.
	class_order: FxHashMap<PatternRef<'db>, u32>,
}

impl<'db> ObjectLoweringPlan<'db> {
	/// Deterministic sort key for a class.
	///
	/// Every class reached during lowering has a descriptor, so ranks are
	/// total and unique; the fallback only keeps this a total order if that
	/// invariant is ever broken.
	pub(in crate::lower) fn class_rank(&self, class: PatternRef<'db>) -> u32 {
		self.class_order.get(&class).copied().unwrap_or(u32::MAX)
	}

	/// Sort classes into source order, for iteration that emits model items.
	pub(in crate::lower) fn in_class_order(&self, classes: &mut [PatternRef<'db>]) {
		classes.sort_by_key(|class| self.class_rank(*class));
	}

	/// The contribution lists, in ascending occurrence order.
	///
	/// `contributions_by_occurrence` is keyed by `OccurrenceId`, whose numeric
	/// order follows `analysis.occurrences` and is therefore stable across
	/// platforms — unlike the map's own hash order.
	pub(in crate::lower) fn contributions_in_occurrence_order(
		&self,
	) -> impl Iterator<Item = &Vec<OccurrenceContribution<'db>>> {
		let mut ids: Vec<OccurrenceId> = self.contributions_by_occurrence.keys().copied().collect();
		ids.sort_by_key(|id| id.0);
		ids.into_iter()
			.map(move |id| &self.contributions_by_occurrence[&id])
	}

	pub(in crate::lower) fn new(db: &'db dyn Db) -> Self {
		let analysis = analyse_new_objects(db);
		let mut top_level_occurrences = FxHashMap::default();
		let mut nested_occurrences = FxHashMap::default();
		let mut local_domain_sources = FxHashMap::default();
		let mut contributions_by_occurrence: FxHashMap<_, Vec<_>> = FxHashMap::default();

		for occurrence in analysis.occurrences.iter() {
			let _ = local_domain_sources.insert(occurrence.id, occurrence.local_domain_source);
			if let OccurrenceSource::TopLevelDeclaration(pattern) = occurrence.source {
				let _ = top_level_occurrences.insert(pattern, occurrence.id);
			}
		}

		for occurrence in analysis.occurrences.iter() {
			let mut root = occurrence;
			while let Some(parent) = root.parent {
				root = &analysis.occurrences[parent.0 as usize];
			}
			if let OccurrenceSource::TopLevelDeclaration(pattern) = root.source {
				let _ =
					nested_occurrences.insert((pattern, occurrence.path.clone()), occurrence.id);
			}
		}

		for contribution in analysis.contributions.iter() {
			contributions_by_occurrence
				.entry(contribution.occurrence)
				.or_default()
				.push(contribution.clone());
		}

		// Classes with no top-level introduction are "field-only" — their
		// actual set must be derived rather than supplied by the user. Walk
		// over all contributions (not just `introduced_class`): superclasses
		// of field-only classes are reached only via projection, so they
		// don't appear as `introduced_class` anywhere but still need their
		// actual-set defined.
		let mut directly_introduced = FxHashSet::<PatternRef<'db>>::default();
		let mut all_reached = FxHashSet::<PatternRef<'db>>::default();
		for occurrence in analysis.occurrences.iter() {
			if matches!(occurrence.source, OccurrenceSource::TopLevelDeclaration(_)) {
				let _ = directly_introduced.insert(occurrence.introduced_class);
			}
		}
		for contribution in analysis.contributions.iter() {
			let _ = all_reached.insert(contribution.target_class);
		}
		let field_only_introduced_classes = all_reached
			.difference(&directly_introduced)
			.copied()
			.collect();

		// Domain-relocation eligibility. Per target class, scan every
		// contribution: it must be engine-alias-defined, and track whether
		// any contribution has unrealisable slots. The per-class verdicts are
		// then closed over subclasses: a field's values live in the owner's
		// AND every subclass's `_objects`, so the owner only qualifies if the
		// whole subtree does.
		let mut alias_defined_everywhere: FxHashMap<PatternRef<'db>, bool> = FxHashMap::default();
		let mut has_unrealisable_root: FxHashSet<PatternRef<'db>> = FxHashSet::default();
		for contribution in analysis.contributions.iter() {
			let occurrence = &analysis.occurrences[contribution.occurrence.0 as usize];
			// Static mirror of the lowering's contribution routing: top-level
			// contributions are always engine-alias-defined (direct engine
			// runs, vacuous passthroughs, or projections reading the direct
			// objects array); nested contributions are whenever their target
			// class is var-reached (the var nested storage engine).
			// Par-reached nested targets always take the record-input path
			// (par chains inline child input records at every hop; the
			// identity-mode no-input fallback only arises on var paths), so
			// marking them false here is conservative only for MIXED classes
			// (par-nested + var contributions), which then keep per-slot
			// guards instead of relocating — sound, just the cheaper encoding
			// missed.
			let engine_sourced = match occurrence.source {
				OccurrenceSource::TopLevelDeclaration(_) => true,
				OccurrenceSource::ClassAttribute { .. } => analysis
					.var_reached_classes
					.contains(&contribution.target_class),
			};
			let _ = alias_defined_everywhere
				.entry(contribution.target_class)
				.and_modify(|all| *all &= engine_sourced)
				.or_insert(engine_sourced);
			match occurrence.source {
				OccurrenceSource::TopLevelDeclaration(pattern) => {
					if let Item::Declaration(it) = pattern.item(db) {
						let declaration = it.declaration(db);
						if introduces_var_existence(declaration.data(), declaration.declared_type) {
							let _ = has_unrealisable_root.insert(contribution.target_class);
						}
					}
				}
				OccurrenceSource::ClassAttribute { .. } => {
					// A nested introduction has unrealisable slots exactly
					// when the introduced child's actual set is a solver
					// decision (directly, or through a var-existence
					// ancestor chain).
					if analysis
						.var_actual_set_classes
						.contains(&occurrence.introduced_class)
					{
						let _ = has_unrealisable_root.insert(contribution.target_class);
					}
				}
			}
		}
		let subtree_alias_defined = |class: PatternRef<'db>| -> bool {
			let mut stack = vec![class];
			while let Some(current) = stack.pop() {
				if !alias_defined_everywhere
					.get(&current)
					.copied()
					.unwrap_or(true)
				{
					return false;
				}
				if let Some(subclasses) = analysis.map_class_to_subclasses.get(&current) {
					stack.extend(subclasses.iter().copied());
				}
			}
			true
		};
		let domain_relocation_classes = has_unrealisable_root
			.iter()
			.copied()
			.filter(|class| subtree_alias_defined(*class))
			.collect();

		// Upward superclass closure of `has_unrealisable_root`: a superclass
		// whose (transitive) subclass can hold phantom slots stores that
		// subclass's projected rows, so its OWN fields also need
		// phantom-consistent storage domains.
		let mut unrealisable_storage_classes: FxHashSet<PatternRef<'db>> =
			has_unrealisable_root.iter().copied().collect();
		let mut changed = true;
		while changed {
			changed = false;
			// Hash order is fine here: this runs to a fixpoint and only ever
			// inserts into a set, so the result is the same whatever order the
			// edges are visited in. Nothing is emitted from this loop.
			#[allow(
				clippy::iter_over_hash_type,
				reason = "fixpoint over a set — order-independent"
			)]
			for (super_class, subclasses) in analysis.map_class_to_subclasses.iter() {
				if !unrealisable_storage_classes.contains(super_class)
					&& subclasses
						.iter()
						.any(|sub| unrealisable_storage_classes.contains(sub))
				{
					let _ = unrealisable_storage_classes.insert(*super_class);
					changed = true;
				}
			}
		}

		Self {
			top_level_occurrences,
			nested_occurrences,
			local_domain_sources,
			contributions_by_occurrence,
			var_reached_classes: analysis.var_reached_classes.iter().copied().collect(),
			var_actual_set_classes: analysis.var_actual_set_classes.iter().copied().collect(),
			field_only_introduced_classes,
			domain_relocation_classes,
			unrealisable_storage_classes,
			class_order: analysis
				.class_descriptors
				.iter()
				.enumerate()
				.map(|(rank, descriptor)| (descriptor.class_pattern, rank as u32))
				.collect(),
		}
	}
}

/// One recorded parent-field introduction contributing to a
/// field-only-introduced class's actual set. Keyed one hop up: the
/// *immediate* parent class/contribution and the direct attribute name (a
/// record field of the parent's storage element type), never a joined path.
pub(in crate::lower) struct FieldIntroduction<'db> {
	parent_class: PatternRef<'db>,
	parent_contribution_index: usize,
	attribute: Identifier<'db>,
	/// The introduced child's constructor index in its class enum for this
	/// introduction (the `k` in `<Child>_occ_k`). Matches the recorded
	/// introduction against the child's contribution list in
	/// `field_only_class_set_array_union`'s completeness check.
	child_contribution_index: usize,
	kind: FieldIntroductionKind,
}

/// Shape of a recorded field introduction, selecting the contribution
/// expression `field_only_class_set_array_union` builds for it.
pub(in crate::lower) enum FieldIntroductionKind {
	/// `set of new` / `var set(...) of new` field: the storage field value
	/// is a set of child identities and is unioned directly (guarded by the
	/// parent slot's realisation when the parent is var-actual).
	Collection,
	/// Singular `new` / `opt new` field: exactly one STATIC child identity
	/// per parent slot (1:1 slot mapping), so the contribution is a guarded
	/// identity singleton `{<Child>_occ_k(<ordinal of p>)}` — no `deopt` of
	/// the field value is needed.
	Singular {
		/// Whether the field is `opt new` (contribution guarded by
		/// `occurs(<field>)`).
		opt: bool,
	},
}

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

	/// Replace any `Class<X>` element inside `ty` with `<X>_potential` (the
	/// par enum representing that class's potential identity universe).
	/// Used when building storage records so that `var set of new B: bs`
	/// lowers to `var set of B_potential` rather than `var set of B` — the
	/// latter would create a circular type definition once `B` is itself
	/// derived as `array_union(...)` over the very `bs` field.
	pub(in crate::lower) fn substitute_class_with_potential_enum(&self, ty: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Class(inst, opt, class_ref) => {
				let Some(class_pattern) = class_pattern_for(db, *class_ref) else {
					return ty;
				};
				let Some(class_map_info) = self.class_map.get(&class_pattern) else {
					// Class hasn't been registered yet. This only happens while
					// predeclaring a class that participates in a reference
					// cycle (`Seat` ↔ `Handrail`): items are predeclared in
					// topological order, but a cycle has no valid order, so
					// whichever class comes first sees the other unregistered.
					// The unsubstituted `Class<X>` must not survive into item
					// collection — expression lowering freezes decl types at
					// build time and a class-typed storage field defeats the
					// potential-enum arm of `lowered_ty_matches` — so
					// `repair_predeclared_class_objects_domains` rebuilds every
					// `<C>_objects` domain once all classes are registered.
					return ty;
				};
				let enum_ref = self.model[class_map_info.class_enum].enum_type();
				// Preserve the source `inst`: `var Class<B>` (field reached
				// through a `var new` path) must lower to `var B_potential`,
				// not `par B_potential`. Without this,
				// `class_storage_fields_for_domain` returns par-typed fields
				// for a var-reached class's storage record, the per-class
				// `*_objects` decl is initially declared par-typed at
				// `predeclare_class` time, constraint-emission captures that
				// par type in let-decomposition decls, and the later
				// `finish()` update of the decl to var-typed leaves the
				// captured decls stale (par-vs-var mismatch in lowered MZN).
				let par_enum = Ty::par_enum(db, enum_ref);
				par_enum
					.with_inst(db, *inst)
					.unwrap_or(par_enum)
					.with_opt(db, *opt)
			}
			TyData::Set(inst, opt, element) => {
				let new_element = self.substitute_class_with_potential_enum(*element);
				let par_set = Ty::par_set(db, new_element).unwrap_or(*element);
				par_set
					.with_inst(db, *inst)
					.unwrap_or(par_set)
					.with_opt(db, *opt)
			}
			TyData::Array { opt, dim, element } => {
				let new_element = self.substitute_class_with_potential_enum(*element);
				Ty::array(db, *dim, new_element)
					.unwrap_or(ty)
					.with_opt(db, *opt)
			}
			TyData::Tuple(opt, fields) => {
				let new_fields = fields
					.iter()
					.map(|f| self.substitute_class_with_potential_enum(*f))
					.collect::<Vec<_>>();
				Ty::tuple(db, new_fields).with_opt(db, *opt)
			}
			TyData::Record(opt, fields) => {
				let new_fields = fields
					.iter()
					.map(|(name, f)| (*name, self.substitute_class_with_potential_enum(*f)))
					.collect::<Vec<_>>();
				Ty::record(db, new_fields).with_opt(db, *opt)
			}
			_ => ty,
		}
	}

	pub(in crate::lower) fn class_storage_fields(
		&self,
		class_pattern: PatternRef<'db>,
	) -> Vec<(Identifier<'db>, Ty<'db>)> {
		let types = class_pattern.item(self.db).types(self.db);
		match &types[class_pattern.pattern(self.db)] {
			PatternTy::ClassDecl {
				storage_record_ty,
				var_storage_record_ty,
				..
			} => {
				// For classes reached via any `var new` introduction path,
				// use the varified storage record so `*_objects` arrays are
				// declared with var-typed fields from the start. Otherwise
				// use the par-typed record from the class declaration.
				let record_ty = if self
					.object_lowering
					.var_reached_classes
					.contains(&class_pattern)
				{
					*var_storage_record_ty
				} else {
					*storage_record_ty
				};
				record_ty
					.record_fields(self.db)
					.unwrap_or_default()
					.into_iter()
					.map(|(field, ty)| (Identifier(field), ty))
					.collect()
			}
			_ => unreachable!(),
		}
	}

	/// Same as `class_storage_fields` but with each `Class<X>` element
	/// substituted with `<X>_potential` (par enum). Use at the points that
	/// emit the storage record domain — both `*_storage` declarations and
	/// per-class `*_objects` arrays — so the lowered model references the
	/// child class's potential universe rather than the child class's
	/// actual-set (which would create a circular type definition since
	/// the actual-set is derived from these very fields).
	pub(in crate::lower) fn class_storage_fields_for_domain(
		&self,
		class_pattern: PatternRef<'db>,
	) -> Vec<(Identifier<'db>, Ty<'db>)> {
		self.class_storage_fields(class_pattern)
			.into_iter()
			.map(|(field, ty)| (field, self.substitute_class_with_potential_enum(ty)))
			.collect()
	}

	// Recursively gather `(class_item, declared_type, original_ty)` per field
	// identifier for every storage field of `class_pattern`, walking
	// superclass items first so that inherited fields appear before own
	// fields. Used to build per-field domains for the `_storage` record that
	// preserve declared bounds (`var 5..10: i`, `var set of 0..3: s`, etc.).
	pub(in crate::lower) fn collect_class_field_descriptors(
		&self,
		class_pattern: PatternRef<'db>,
		descriptors: &mut FxHashMap<
			Identifier<'db>,
			(Item<'db>, shackle_hir::TypeId<'db>, Ty<'db>),
		>,
	) {
		let class_item = class_pattern.item(self.db);
		let Item::Class(class_item_ref) = class_item else {
			return;
		};
		let class = class_item_ref.class(self.db);
		let class_types = class_item.types(self.db);
		if let Some(base) = class.extends.and_then(|b| class_types.name_resolution(b)) {
			self.collect_class_field_descriptors(base, descriptors);
		}
		for field_item in class.items.iter() {
			let ClassMember::Declaration(d) = field_item else {
				continue;
			};
			for pattern in shackle_hir::Pattern::identifiers(d.pattern, class.data()) {
				let pat_ref = PatternRef::new(self.db, class_item, pattern);
				let Some(ident) = pat_ref.identifier(self.db) else {
					continue;
				};
				let original_ty = match &class_types[pattern] {
					PatternTy::Variable(ty) => *ty,
					_ => continue,
				};
				let _ = descriptors.insert(ident, (class_item, d.declared_type, original_ty));
			}
		}
	}

	// Wrap `build_class_storage_record_domain` for the `array [...] of
	// record(...)` shape that every `*_objects` / `*_storage` declaration
	// uses. The index domain is intentionally left unbounded — callers
	// that need a tighter (par-enum or singleton ordinal) index domain
	// should still call `build_class_storage_record_domain` directly and
	// assemble the `Domain::array` themselves.
	pub(in crate::lower) fn build_class_storage_array_domain(
		&mut self,
		class_pattern: PatternRef<'db>,
		array_ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		let (Some(elem_ty), Some(dim_ty)) = (array_ty.elem_ty(db), array_ty.dim_ty(db)) else {
			return Domain::unbounded(db, origin, array_ty);
		};
		let opt = array_ty.opt(db).unwrap_or(OptType::NonOpt);
		let elem_dom = self.build_class_storage_record_domain(class_pattern, elem_ty, origin);
		let dim_dom = Domain::unbounded(db, origin, dim_ty);
		Domain::array(db, origin, opt, dim_dom, elem_dom)
	}

	// Build a record `Domain` for the `_storage` element with declared bounds
	// preserved per field. Scalar fields like `var 5..10: i` come back as
	// `Domain::bounded(5..10)` instead of `Domain::unbounded`, which keeps the
	// downstream FlatZinc within solver-supported domain widths (gecode
	// otherwise errors during SOLVING on unbounded `var int` / `var set of int`
	// introduced by the per-field symmetry-breaking `= lb(...)` constraints).
	// Class-typed fields keep their unbounded form — bounds for those live in
	// the substituted potential-enum element type.
	pub(in crate::lower) fn build_class_storage_record_domain(
		&mut self,
		class_pattern: PatternRef<'db>,
		storage_elem_ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		let Some(record_fields) = storage_elem_ty.record_fields(db) else {
			return Domain::unbounded(db, origin, storage_elem_ty);
		};
		let opt = storage_elem_ty.opt(db).unwrap_or(OptType::NonOpt);
		let mut descriptors: FxHashMap<
			Identifier<'db>,
			(Item<'db>, shackle_hir::TypeId<'db>, Ty<'db>),
		> = FxHashMap::default();
		self.collect_class_field_descriptors(class_pattern, &mut descriptors);
		let mut field_domains: Vec<(Identifier<'db>, Domain<'db>)> =
			Vec::with_capacity(record_fields.len());
		let storage_field_decls = self.class_storage_field_decls(class_pattern.item(db));
		for (raw_ident, field_ty) in record_fields {
			let ident = Identifier(raw_ident);
			// A relocated defined field's declared domain must not bind in
			// the shared element record — it is re-imposed as a realised-set
			// class invariant instead. The per-field decision is the same at
			// every site (owner-keyed predicate), so the `_storage`,
			// predeclare, contribution, and `finish()` domains all relax
			// together.
			let relocated = storage_field_decls
				.iter()
				.find(|d| d.ident == ident)
				.map(|d| self.field_relocates_declared_domain(d))
				.unwrap_or(false);
			// Set-cardinality relocation: the card bound must not bind on
			// unrealised slots, whose canonical pin/witness value is `{}`. It
			// is re-imposed on realised objects by
			// `emit_nested_set_cardinality_class_invariants`.
			let card_relocated = storage_field_decls
				.iter()
				.find(|d| d.ident == ident)
				.map(|d| self.field_relocates_set_card(d))
				.unwrap_or(false);
			let domain = match descriptors.get(&ident).copied() {
				Some(_) if relocated => Domain::unbounded(
					db,
					origin,
					self.substitute_class_with_potential_enum(field_ty),
				),
				Some((class_item, declared_type, _original_ty)) if card_relocated => {
					self.card_stripped_set_field_domain(class_item, declared_type, field_ty, origin)
				}
				Some((class_item, declared_type, original_ty)) => {
					let substituted = self.substitute_class_with_potential_enum(original_ty);
					if substituted != original_ty {
						self.class_storage_field_domain(class_item, declared_type, field_ty, origin)
					} else if self.field_domain_references_attribute(class_item, declared_type) {
						// A scalar field whose declared domain references another
						// attribute (`var 1..z: s`, with `z` computed) can't carry
						// that per-object bound in the shared record element type —
						// it would need `z` in scope, which isn't meaningful for a
						// record type. Leave the element type unbounded; the tight
						// per-object bound is enforced in the reconstruction
						// comprehension's fresh `let {var 1..z: ..} in ..` decl.
						Domain::unbounded(
							db,
							origin,
							self.substitute_class_with_potential_enum(field_ty),
						)
					} else {
						let Item::Class(class_item_ref) = class_item else {
							unreachable!()
						};
						let class = class_item_ref.class(db);
						let class_types = class_item.types(db);
						let mut inner =
							ExpressionCollector::new(self, class.data(), class_item, &class_types);
						inner.collect_domain(declared_type, field_ty, false)
					}
				}
				None => Domain::unbounded(
					db,
					origin,
					self.substitute_class_with_potential_enum(field_ty),
				),
			};
			field_domains.push((ident, domain));
		}
		Domain::record(db, origin, opt, field_domains)
	}

	/// Whether a field's declared type carries a `Bounded` domain expression —
	/// directly (`var 1..z: s`) or nested inside array dimensions/elements or
	/// set elements (`array [1..l] of int: xs`, the par-ragged shape) — that
	/// references another class attribute. Such a per-object bound cannot
	/// live in the shared storage record element type (the sibling isn't in
	/// scope there): the element type goes unbounded and the value's own
	/// index sets / the reconstruction comprehension carry the per-object
	/// shape instead. Var-reached classes never get here for these shapes —
	/// validation rejects attribute-referencing domains on them outright.
	pub(in crate::lower) fn field_domain_references_attribute(
		&self,
		class_item: Item<'db>,
		declared_type: shackle_hir::TypeId<'db>,
	) -> bool {
		let db = self.db;
		let field_patterns: Vec<PatternRef<'db>> = self
			.class_storage_field_decls(class_item)
			.into_iter()
			.map(|f| f.pattern)
			.collect();
		let class_types = class_item.types(db);
		let Item::Class(class_item_ref) = class_item else {
			return false;
		};
		let class_data = class_item_ref.class(db).data();
		for type_node in shackle_hir::Type::walk(declared_type, class_data) {
			let shackle_hir::Type::Bounded { domain, .. } = &class_data[type_node] else {
				continue;
			};
			for sub in shackle_hir::Expression::walk(*domain, class_data) {
				if matches!(&class_data[sub], shackle_hir::Expression::Identifier(_))
					&& let Some(res) = class_types.name_resolution(sub)
					&& field_patterns.contains(&res)
				{
					return true;
				}
			}
		}
		false
	}

	/// Does this object input record type contain an optional-child slot
	/// (`opt record`, produced by an `opt new C` attribute) at any depth?
	/// MiniZinc forbids `opt record` declarations and values, so such a slot
	/// must be lowered to a non-opt 0-or-1-length list before it can appear in
	/// a `_inputs` array. Used to gate the (snapshot-affecting) normalisation
	/// so classes with no optional child are left byte-identical.
	pub(in crate::lower) fn input_ty_needs_opt_new_lowering(&self, ty: Ty<'db>) -> bool {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Record(opt, fields) => {
				*opt == OptType::Opt
					|| fields
						.iter()
						.any(|(_, f)| self.input_ty_needs_opt_new_lowering(*f))
			}
			TyData::Array { element, .. } => self.input_ty_needs_opt_new_lowering(*element),
			_ => false,
		}
	}

	/// Lower an object input record TYPE by replacing each optional-child
	/// slot (`opt record`, from `opt new C`) with a non-opt 0-or-1-length list
	/// (`array [int] of record`). Recurses through records / lists so nested
	/// optional children are normalised too. The 0/1-length list feeds the
	/// same flattened-collection machinery a `set(0..1) of new C` field uses;
	/// see `lower_opt_new_input_value` for the matching value transform.
	pub(in crate::lower) fn lower_opt_new_input_ty(&self, ty: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Record(opt, fields) => {
				let lowered = Ty::record(
					db,
					fields
						.iter()
						.map(|(name, f)| (*name, self.lower_opt_new_input_ty(*f)))
						.collect::<Vec<_>>(),
				);
				if *opt == OptType::Opt {
					Ty::array(db, Ty::par_int(db), lowered).expect("opt-new child list type")
				} else {
					lowered
				}
			}
			TyData::Array { opt, dim, element } => {
				Ty::array(db, *dim, self.lower_opt_new_input_ty(*element))
					.map(|t| t.with_opt(db, *opt))
					.unwrap_or(ty)
			}
			_ => ty,
		}
	}

	/// Lower an object input record VALUE to match `lower_opt_new_input_ty`.
	/// A present optional child `(f: (…))` becomes `(f: [(…)])`; an absent one
	/// `(f: <>)` becomes `(f: [])` (an empty list typed as the child input
	/// record list). Recurses through record / list literals. Because MiniZinc
	/// forbids `opt record` values, only STATICALLY present/absent literal
	/// inputs are handled here; a non-literal optional input is left unchanged
	/// (rejected earlier by validation).
	pub(in crate::lower) fn lower_opt_new_input_value(
		&mut self,
		item: Item<'db>,
		value: Expression<'db>,
		orig_ty: Ty<'db>,
	) -> Expression<'db> {
		let db = self.db;
		match orig_ty.lookup(db) {
			TyData::Record(OptType::Opt, fields) => {
				let inner_ty = Ty::record(db, fields.to_vec());
				match &*value {
					ExpressionData::Absent => {
						let list_ty =
							Ty::array(db, Ty::par_int(db), self.lower_opt_new_input_ty(inner_ty))
								.expect("opt-new child list type");
						Expression::new_unchecked(list_ty, ArrayLiteral(vec![]), value.origin())
					}
					_ => {
						let normalized = self.lower_opt_new_input_value(item, value, inner_ty);
						Expression::new(self.db, &self.model, item, ArrayLiteral(vec![normalized]))
					}
				}
			}
			TyData::Record(_, fields) => {
				if !self.input_ty_needs_opt_new_lowering(orig_ty) {
					return value;
				}
				match &*value {
					ExpressionData::RecordLiteral(rl) => {
						let fields = fields.clone();
						let new_fields = rl
							.iter()
							.map(|(ident, fv)| {
								let fty = fields
									.iter()
									.find(|(n, _)| Identifier(*n) == *ident)
									.map(|(_, t)| *t)
									.expect("input record field type");
								(
									*ident,
									self.lower_opt_new_input_value(item, fv.clone(), fty),
								)
							})
							.collect::<Vec<_>>();
						Expression::new(self.db, &self.model, item, RecordLiteral(new_fields))
					}
					_ => value,
				}
			}
			TyData::Array { element, .. } => {
				if !self.input_ty_needs_opt_new_lowering(orig_ty) {
					return value;
				}
				let element = *element;
				match &*value {
					ExpressionData::ArrayLiteral(al) => {
						let new_elems = al
							.iter()
							.map(|e| self.lower_opt_new_input_value(item, e.clone(), element))
							.collect::<Vec<_>>();
						Expression::new(self.db, &self.model, item, ArrayLiteral(new_elems))
					}
					_ => value,
				}
			}
			_ => value,
		}
	}

	/// Lower a top-level collection input VALUE (the RHS of a
	/// `set of new C: cs = [..]` / `array [d] of new C: pool = [..]` root) whose
	/// member records carry optional children. The collection is an array *of*
	/// input records, so the element records must be lowered one level down:
	/// `element_record_ty` is the per-member input record type (`opt new`
	/// children still `opt record`), which we wrap in an array type so
	/// `lower_opt_new_input_value`'s array arm maps the transform over each
	/// member. Passing the element record type directly (as if the value were a
	/// single record) would fall through unchanged — the value is an array, not
	/// a record. Non-literal collections are left unchanged (rejected earlier).
	pub(in crate::lower) fn lower_opt_new_input_collection_value(
		&mut self,
		item: Item<'db>,
		value: Expression<'db>,
		element_record_ty: Ty<'db>,
	) -> Expression<'db> {
		let db = self.db;
		let collection_ty = Ty::array(db, Ty::par_int(db), element_record_ty)
			.expect("opt-new input collection type");
		self.lower_opt_new_input_value(item, value, collection_ty)
	}

	/// Restrict a class's full storage record type to the fields that are
	/// *free decisions* in the `_storage` array. Two kinds of field are dropped
	/// because they are instead *defined* per object in the reconstruction
	/// comprehension (as generator aliases), not stored freely:
	///
	/// - **Computed attributes** (`d.definition.is_some()`, e.g.
	///   `array[int] of var 1..x: y = f(x)`). A free `_storage` decision for
	///   such a field would be an invalid MiniZinc declaration anyway — you
	///   cannot allocate an unknown-length `array[int] of var int` or a free
	///   unbounded `var set of int`; only the alias form (`y = f(x)`) is legal.
	/// - **Domain-dependent fields** (`field_domain_references_attribute`, e.g.
	///   `var 1..z: s` with `z` a sibling). Their per-object bound can't live in
	///   the shared record element type; they are minted per object in the
	///   comprehension via `let { var 1..z: .. } in ..`.
	///
	/// Class-typed fields are *kept* — under a var-new root every attribute is a
	/// free decision and class-typed fields carry their `<X>_potential` identity
	/// in storage (bounded by the per-parent slice constraint).
	pub(in crate::lower) fn free_storage_record_ty(
		&self,
		class_pattern: PatternRef<'db>,
		storage_record_ty: Ty<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let Some(record_fields) = storage_record_ty.record_fields(db) else {
			return storage_record_ty;
		};
		let excluded: FxHashSet<Identifier<'db>> = self
			.class_storage_field_decls(class_pattern.item(db))
			.into_iter()
			.filter(|d| {
				d.definition.is_some()
					|| self.field_domain_references_attribute(d.owner, d.declared_type)
			})
			.map(|d| d.ident)
			.collect();
		if excluded.is_empty() {
			return storage_record_ty;
		}
		let opt = storage_record_ty.opt(db).unwrap_or(OptType::NonOpt);
		Ty::record(
			db,
			record_fields
				.into_iter()
				.filter(|(name, _)| !excluded.contains(&Identifier(*name)))
				.collect::<Vec<_>>(),
		)
		.with_opt(db, opt)
	}

	// Domain for a class-typed storage field (the `substituted != original`
	// case). The base type is the `Class<X>`→`<X>_potential`-substituted,
	// otherwise-unbounded form. If the field is a *reference* set field
	// `set(<card>) of <Class>` (no `new`), carry its cardinality bound into the
	// type natively as `var set(<card>) of <X>_potential` instead of dropping
	// it — the native `set(c) of S` syntax desugars to a `card(...) in <card>`
	// constraint in the target MiniZinc. Fresh `set of new` collections keep
	// the unbounded form and instead get their bound from the nested
	// cardinality invariant / slice loop. Only a constant bound belongs in the
	// shared element type; a per-object bound would still need the `forall`.
	pub(in crate::lower) fn class_storage_field_domain(
		&mut self,
		class_item: Item<'db>,
		declared_type: shackle_hir::TypeId<'db>,
		field_ty: Ty<'db>,
		origin: Origin<'db>,
	) -> Domain<'db> {
		let db = self.db;
		let subst = self.substitute_class_with_potential_enum(field_ty);
		let Item::Class(class_item_ref) = class_item else {
			return Domain::unbounded(db, origin, subst);
		};
		let class_data = class_item_ref.class(db).data();
		// A single-dimension array-of-class-reference field
		// (`array [1..2] of B`). The declared per-object dimensions live in the
		// AST declared type — the resolved `Ty` erases `1..2` to `int` — so a
		// bare `Domain::unbounded(subst)` gives the free `_storage` decision an
		// `array [int] of var <B>_potential` element with an unknown index set,
		// which MiniZinc cannot allocate. Preserve the declared dimensions and
		// give each slot the substituted `<B>_potential` identity domain, exactly
		// a scalar class-reference field's storage (`var <B>_potential`) per
		// array position. Only single-dim, scalar-class-element arrays are
		// supported here; other array-of-class shapes stay rejected by
		// object validation.
		if let shackle_hir::Type::Array { dimensions, .. } = &class_data[declared_type] {
			let dimensions = *dimensions;
			if let (Some(dim_ty), Some(elem_ty)) = (field_ty.dim_ty(db), field_ty.elem_ty(db)) {
				let elem_subst = self.substitute_class_with_potential_enum(elem_ty);
				if !dim_ty.is_tuple(db) && matches!(elem_subst.lookup(db), TyData::Enum(_, _, _)) {
					let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
					let class_types = class_item.types(db);
					let dim_dom = {
						let mut inner = ExpressionCollector::new(
							self,
							class_item_ref.class(db).data(),
							class_item,
							&class_types,
						);
						inner.collect_element_domain(dimensions, dim_ty, false)
					};
					let elem_dom = Domain::unbounded(db, origin, elem_subst);
					return Domain::array(db, origin, opt, dim_dom, elem_dom);
				}
			}
		}
		let card_idx = match &class_data[declared_type] {
			shackle_hir::Type::Set {
				cardinality: Some(card),
				..
			} if class_data[declared_type]
				.get_new_class(class_data)
				.is_none() =>
			{
				*card
			}
			_ => return Domain::unbounded(db, origin, subst),
		};
		let class_types = class_item.types(db);
		let card_expr = {
			let mut inner = ExpressionCollector::new(
				self,
				class_item_ref.class(db).data(),
				class_item,
				&class_types,
			);
			inner.collect_expression(card_idx)
		};
		let elem_ty = subst.elem_ty(db).unwrap_or(subst);
		let inst = subst.inst(db).unwrap_or(VarType::Var);
		let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
		Domain::set_with_card(
			db,
			origin,
			inst,
			opt,
			Some(card_expr),
			Domain::unbounded(db, origin, elem_ty),
		)
	}

	pub(in crate::lower) fn register_class_object_contribution(
		&mut self,
		target_class: PatternRef<'db>,
		contribution_index: usize,
		declaration: DeclarationId<'db>,
		defined_fields_determined: bool,
	) {
		self.class_object_contributions
			.entry(target_class)
			.or_default()
			.push((contribution_index, declaration));
		let _ = self
			.class_contributions_all_determined
			.entry(target_class)
			.and_modify(|all| *all &= defined_fields_determined)
			.or_insert(defined_fields_determined);
		let _ = self.contribution_determined_by_index.insert(
			(target_class, contribution_index),
			defined_fields_determined,
		);
	}

	/// The `defined_fields_determined` flag a specific contribution registered
	/// with (`None` when not yet registered). Projections reading every field
	/// from that contribution's decl inherit exactly this flag.
	pub(in crate::lower) fn contribution_determined(
		&self,
		target_class: PatternRef<'db>,
		contribution_index: usize,
	) -> Option<bool> {
		self.contribution_determined_by_index
			.get(&(target_class, contribution_index))
			.copied()
	}

	pub(in crate::lower) fn class_object_contribution_declaration(
		&self,
		target_class: PatternRef<'db>,
		contribution_index: usize,
	) -> Option<DeclarationId<'db>> {
		self.class_object_contributions
			.get(&target_class)
			.and_then(|contributions| {
				contributions.iter().find_map(|(index, declaration)| {
					(*index == contribution_index).then_some(*declaration)
				})
			})
	}

	/// The class identity of contribution storage slot `p` for
	/// `(class, contribution_index)`. Enum-indexed storage (nested
	/// per-contribution arrays) uses the slot index directly — it already IS
	/// the `<C>_occ_k(...)` identity. Int-indexed storage (top-level root
	/// contributions) is positioned by constructor-local ordinals, so wrap the
	/// index in the contribution's enum constructor — the same
	/// `EnumMemberId::new(class_enum, contribution_index)` mapping the
	/// per-parent slice arrays use.
	pub(in crate::lower) fn contribution_slot_identity(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		contribution_index: usize,
		index_ty: Ty<'db>,
		index_expr: Expression<'db>,
	) -> Expression<'db> {
		if index_ty == Ty::par_int(self.db) {
			let enum_member =
				EnumMemberId::new(self.class_map[&class].class_enum, contribution_index as u32);
			Expression::new(
				self.db,
				&self.model,
				item,
				Call {
					function: Callable::EnumConstructor(enum_member),
					arguments: vec![index_expr],
				},
			)
		} else {
			index_expr
		}
	}

	/// The constructor-LOCAL ordinal of contribution storage slot `p` for
	/// `(class, contribution_index)`. Int-indexed storage is positioned by
	/// local ordinals already. Enum-indexed storage erases to the class
	/// enum's global position, corrected back to constructor-local via the
	/// previous contribution's end offset — the same arithmetic as
	/// `project_class_identity` (falling back to the global position when no
	/// end offset was chained, matching the per-parent slice arithmetic).
	pub(in crate::lower) fn contribution_local_ordinal(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		contribution_index: usize,
		index_ty: Ty<'db>,
		index_expr: Expression<'db>,
	) -> Expression<'db> {
		if index_ty == Ty::par_int(self.db) {
			return index_expr;
		}
		let global_ordinal = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.enum2int.into(),
				arguments: vec![index_expr],
			},
		);
		if contribution_index == 0 {
			return global_ordinal;
		}
		let Some(previous_end) = self
			.contribution_end_map
			.get(&(class, contribution_index - 1))
			.copied()
		else {
			return global_ordinal;
		};
		let previous_end_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(previous_end),
		);
		let zero_based = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.minus.into(),
				arguments: vec![global_ordinal, previous_end_expr],
			},
		);
		let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.builtins.plus.into(),
				arguments: vec![one, zero_based],
			},
		)
	}

	pub(in crate::lower) fn register_class_set_top_level_contribution(
		&mut self,
		target_class: PatternRef<'db>,
		contribution_index: usize,
		expression: Expression<'db>,
	) {
		self.class_set_top_level_contributions
			.entry(target_class)
			.or_default()
			.push((contribution_index, expression));
	}

	pub(in crate::lower) fn record_expr_has_field(
		&self,
		expr: &Expression<'db>,
		field_ident: Identifier<'db>,
	) -> bool {
		matches!(
			expr.ty().lookup(self.db),
			TyData::Record(_, fields) if fields.iter().any(|(field, _)| *field == field_ident.0)
		)
	}

	/// Emit the per-parent subset constraint
	/// `forall(p in index_set(<parent>_storage))
	///     ((<parent>_storage[p]).<field> subset <parent>_<field>_potential[p])`.
	///
	/// `parent_occurrence` + `parent_class` identify the *immediate* parent
	/// for this `<field>`. For root-class fields (e.g. `e.vehicles` where the
	/// root decl is `var set of new Expedition`), pass the root occurrence
	/// and the root class. For nested-of-nested fields (e.g. `v.crew` where
	/// the parent Vehicle was itself introduced by `Expedition.vehicles`),
	/// pass the *immediate* parent occurrence and class — not the root.
	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn emit_per_parent_subset_constraint(
		&mut self,
		item: Item<'db>,
		top_level: bool,
		parent_occurrence: OccurrenceId,
		parent_class: PatternRef<'db>,
		field_ident: Identifier<'db>,
		child_occurrence: OccurrenceId,
		child_class: PatternRef<'db>,
	) -> Option<ConstraintId<'db>> {
		let child_contribution = self.occurrence_contribution(child_occurrence, child_class);
		let slice_decl_idx = *self
			.slice_array_decls
			.get(&(child_class, child_contribution.constructor_index))?;

		let parent_contribution_index = self
			.occurrence_contribution(parent_occurrence, parent_class)
			.constructor_index;
		let parent_contribution_decl_idx =
			self.class_object_contribution_declaration(parent_class, parent_contribution_index)?;

		let parent_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(parent_contribution_decl_idx),
		);
		// The parent storage may be int-indexed (a top-level root contribution
		// like `Expedition_expeditions_objects`) or enum-indexed by the
		// constructor's image (nested per-contribution storage like
		// `Vehicle_vehicles_objects`). The iterator `p` must take the dim
		// type; using par-int over an enum-indexed array would fail to
		// dispatch `array_access`.
		let parent_index_ty = match parent_expr.ty().lookup(self.db) {
			TyData::Array { dim, .. } => *dim,
			_ => Ty::par_int(self.db),
		};
		let parent_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![parent_expr.clone()],
			},
		);

		let p_decl = Declaration::new(false, Domain::unbounded(self.db, item, parent_index_ty));
		let p_idx = self
			.model
			.add_declaration(DeclarationItem::new(p_decl, item));
		let p_expr = Expression::new(self.db, &self.model, item, p_idx);

		let parent_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![parent_expr, p_expr.clone()],
			},
		);
		let parent_field_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(parent_at_p),
				field: field_ident,
			},
		);

		let slice_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(slice_decl_idx),
		);
		let slice_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![slice_expr, p_expr],
			},
		);

		// stdlib `subset` for var sets is typed `var set of int × var set of int`,
		// so erase the enum-typed sides to int sets to dispatch.
		let parent_field_erased = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.enum2int.into(),
				arguments: vec![parent_field_at_p],
			},
		);
		let slice_erased = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.enum2int.into(),
				arguments: vec![slice_at_p],
			},
		);
		let subset_call = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.builtins.subset.into(),
				arguments: vec![parent_field_erased, slice_erased],
			},
		);

		let compr = Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![p_idx],
					collection: parent_index_set,
					where_clause: None,
				}],
				subset_call,
			),
		);

		let forall = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![compr],
			},
		);

		Some(self.model.add_constraint(ConstraintItem::new(
			Constraint::new(top_level, forall),
			item,
		)))
	}

	/// Emit the `<child> = array_union(...)` definition over parent storage
	/// fields. Each recorded introduction is keyed one hop up (the *immediate*
	/// parent class/contribution and the direct `<field>` name), so the field
	/// access `<parent>_objects[p].<field>` is always a direct record field.
	/// Per-contribution templates:
	///
	/// - collection fields contribute their set value, guarded by the parent
	///   slot's own realisation — `if <identity(p)> in <Parent> then <field>
	///   else {} endif` — when the parent's actual set is var (rather than
	///   leaning on the symmetry wave pinning unrealised slots' set fields to
	///   `lb = {}`);
	/// - singular fields contribute the STATIC per-slot identity singleton
	///   `{<Child>_occ_k(<ordinal of p>)}` (1:1 slot mapping, no `deopt` of
	///   the field value), guarded by `occurs(<field>)` for `opt new` fields
	///   and by the parent slot's realisation when the parent is var-actual.
	///   Because the var storage field itself is a *free* identity decision,
	///   a channelling pin (`occurs(<field>) -> <field> = <static identity>`)
	///   is emitted alongside so the field value agrees with the identity the
	///   actual set claims (`emit_singular_field_identity_pin`).
	///
	/// Returns `None` when some contribution can't be derived: a depth-0
	/// contribution with no recorded introduction (legacy par-existence
	/// nested collections stored as arrays of inline records), a missing
	/// contribution declaration, or an attribute that isn't a top-level
	/// record field. The caller then falls back to the potential universe —
	/// sound only for par existence (every potential realised); the call
	/// site asserts the class is not var-actual when that happens.
	pub(in crate::lower) fn field_only_class_set_array_union(
		&mut self,
		item: Item<'db>,
		child_class: PatternRef<'db>,
		introductions_map: &FxHashMap<PatternRef<'db>, Vec<FieldIntroduction<'db>>>,
	) -> Option<Expression<'db>> {
		// Every contribution to the class, across all occurrences: the
		// constructor index in the child's enum, the projection depth, and
		// the occurrence's DIRECT class with its own constructor index
		// (identity source for projection images). Direct (depth-0)
		// contributions must each have a recorded field introduction — a
		// shape the recording doesn't cover (e.g. nested `array of new`
		// fields) means the set can't be derived and the caller falls back.
		let mut contributions: Vec<(usize, usize, PatternRef<'db>, usize, OccurrenceId)> =
			Vec::new();
		for occurrence_contributions in self.object_lowering.contributions_in_occurrence_order() {
			let Some(direct) = occurrence_contributions
				.iter()
				.find(|contribution| contribution.projection_depth == 0)
			else {
				continue;
			};
			for contribution in occurrence_contributions
				.iter()
				.filter(|contribution| contribution.target_class == child_class)
			{
				contributions.push((
					contribution.constructor_index,
					contribution.projection_depth,
					direct.target_class,
					direct.constructor_index,
					contribution.occurrence,
				));
			}
		}
		if contributions.is_empty() {
			return None;
		}
		contributions.sort_by_key(|(constructor_index, ..)| *constructor_index);

		let introductions = introductions_map
			.get(&child_class)
			.map(|intros| intros.as_slice())
			.unwrap_or(&[]);

		let analysis = analyse_new_objects(self.db);
		let mut combined: Option<Expression<'db>> = None;
		for (constructor_index, depth, direct_class, direct_constructor_index, occurrence) in
			contributions
		{
			let compr = if depth == 0 {
				match introductions
					.iter()
					.find(|intro| intro.child_contribution_index == constructor_index)
				{
					Some(intro) => {
						self.field_introduction_contribution_expr(item, child_class, intro)?
					}
					// No recorded introduction — par occurrences skip the
					// pending-slice recording (their collection fields have
					// no per-parent slice array; storage is exactly the
					// data). For a par-instantiated occurrence (no `var new`
					// anywhere on its introduction chain, `!is_var`) the
					// contribution block is data-sized and par existence
					// realises every slot, so the block's identity image IS
					// the contribution — the universe fallback's soundness
					// argument applied per contribution instead of
					// class-globally. A var occurrence without a recording
					// stays non-derivable and forces the caller's fallback.
					None if !analysis.occurrences[occurrence.0 as usize].is_var => {
						self.par_contribution_block_image(item, child_class, constructor_index)?
					}
					None => return None,
				}
			} else {
				self.superclass_projection_contribution_expr(
					item,
					child_class,
					constructor_index,
					direct_class,
					direct_constructor_index,
				)?
			};
			combined = Some(match combined {
				None => compr,
				Some(prev) => Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.plus_plus.into(),
						arguments: vec![prev, compr],
					},
				),
			});
		}

		let combined = combined?;
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_union.into(),
				arguments: vec![combined],
			},
		))
	}

	/// The identity image of one par-instantiated contribution block for a
	/// field-only class's actual set: `[<C>_occ_k('..'(1, <end> - <start>))]`.
	/// The par regime sizes the block exactly from the data (chained
	/// universes, no per-parent padding), and par existence realises every
	/// slot, so the block image is precisely the contribution. Constructor
	/// ordinals are contribution-local and 1-based, hence `1..end - start`
	/// (the chained `_start`/`_end` offsets are class-global positions).
	/// `None` when the block's boundaries were never chained (legacy shapes
	/// with no pending slice) — the caller then falls back to the universe
	/// as before.
	pub(in crate::lower) fn par_contribution_block_image(
		&mut self,
		item: Item<'db>,
		child_class: PatternRef<'db>,
		contribution_index: usize,
	) -> Option<Expression<'db>> {
		let block = self.par_contribution_block_set(item, child_class, contribution_index)?;
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			ArrayLiteral(vec![block]),
		))
	}

	/// The set of identities in one par-instantiated contribution block,
	/// `<C>_occ_k('..'(1, <end> - <start>))` (see
	/// `par_contribution_block_image` for the soundness argument). Also the
	/// exact top-level contribution of a par `array [..] of new C` root —
	/// registering this instead of the whole potential enum keeps other
	/// contributions' potentials out of the class set when introductions mix.
	pub(in crate::lower) fn par_contribution_block_set(
		&mut self,
		item: Item<'db>,
		child_class: PatternRef<'db>,
		contribution_index: usize,
	) -> Option<Expression<'db>> {
		// Stdlib-less models (the `ignore_stdlib` snapshot harness) have no
		// `'-'`/`'..'` to build the block arithmetic with — fall back like
		// any other underivable piece rather than failing the lookup.
		let par_int = Ty::par_int(self.db);
		if self
			.model
			.lookup_function(
				self.db,
				self.ids.functions.minus.into(),
				&[par_int, par_int],
			)
			.is_err() || self
			.model
			.lookup_function(
				self.db,
				self.ids.functions.dot_dot.into(),
				&[par_int, par_int],
			)
			.is_err()
		{
			return None;
		}
		let end_decl = *self
			.contribution_end_map
			.get(&(child_class, contribution_index))?;
		let start_expr = if contribution_index == 0 {
			Expression::new(self.db, &self.model, item, IntegerLiteral(1))
		} else {
			let previous_end = *self
				.contribution_end_map
				.get(&(child_class, contribution_index - 1))?;
			Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(previous_end),
			)
		};
		let end_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(end_decl),
		);
		let size = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.minus.into(),
				arguments: vec![end_expr, start_expr],
			},
		);
		let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		let local_range = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.dot_dot.into(),
				arguments: vec![one, size],
			},
		);
		let member = EnumMemberId::new(
			self.class_map[&child_class].class_enum,
			contribution_index as u32,
		);
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(member),
				arguments: vec![local_range],
			},
		))
	}

	/// The per-contribution comprehension for one recorded field
	/// introduction (see `field_only_class_set_array_union` for the emitted
	/// shapes). Also emits the singular channelling pin as a side effect.
	pub(in crate::lower) fn field_introduction_contribution_expr(
		&mut self,
		item: Item<'db>,
		child_class: PatternRef<'db>,
		intro: &FieldIntroduction<'db>,
	) -> Option<Expression<'db>> {
		let parent_decl_idx = self.class_object_contribution_declaration(
			intro.parent_class,
			intro.parent_contribution_index,
		)?;

		let parent_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(parent_decl_idx),
		);
		let elem_ty = parent_expr.ty().elem_ty(self.db)?;
		let field_ty = match elem_ty.lookup(self.db) {
			TyData::Record(_, fields) => fields
				.iter()
				.find(|(field, _)| *field == intro.attribute.0)
				.map(|(_, ty)| *ty),
			_ => None,
		}?;

		let parent_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![parent_expr.clone()],
			},
		);
		// The parent storage may be int-indexed (top-level root
		// contributions) or enum-indexed by the constructor's image
		// (nested per-contribution storage); the iterator must take the dim
		// type (see `emit_per_parent_subset_constraint`).
		let parent_index_ty = match parent_expr.ty().lookup(self.db) {
			TyData::Array { dim, .. } => *dim,
			_ => Ty::par_int(self.db),
		};
		let p_decl = Declaration::new(false, Domain::unbounded(self.db, item, parent_index_ty));
		let p_idx = self
			.model
			.add_declaration(DeclarationItem::new(p_decl, item));
		let p_expr = Expression::new(self.db, &self.model, item, p_idx);
		let parent_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![parent_expr.clone(), p_expr.clone()],
			},
		);
		let field_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(parent_at_p),
				field: intro.attribute,
			},
		);
		// Parent-slot realisation test. Only var-actual parents can have
		// unrealised slots; a par parent's slots are all realised, so the
		// guard would be vacuous noise there.
		let realised = self
			.object_lowering
			.var_actual_set_classes
			.contains(&intro.parent_class)
			.then(|| {
				let parent_identity = self.contribution_slot_identity(
					item,
					intro.parent_class,
					intro.parent_contribution_index,
					parent_index_ty,
					p_expr.clone(),
				);
				let parent_set_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(self.class_map[&intro.parent_class].class_set),
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.in_.into(),
						arguments: vec![parent_identity, parent_set_expr],
					},
				)
			});
		let (contribution, guard) = match &intro.kind {
			FieldIntroductionKind::Collection => (field_at_p, realised),
			FieldIntroductionKind::Singular { opt } => {
				// The per-slot child ordinal: singular fields map the
				// parent's constructor-local slot ordinal to the same child
				// ordinal (per-parent block size 1).
				let ordinal = self.contribution_local_ordinal(
					item,
					intro.parent_class,
					intro.parent_contribution_index,
					parent_index_ty,
					p_expr.clone(),
				);
				let child_enum_member = EnumMemberId::new(
					self.class_map[&child_class].class_enum,
					intro.child_contribution_index as u32,
				);
				let identity = Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal],
					},
				);
				let singleton = Expression::new(
					self.db,
					&self.model,
					item,
					SetLiteral(vec![identity.clone()]),
				);
				self.emit_singular_field_identity_pin(
					item,
					intro,
					field_ty,
					&parent_expr,
					parent_index_ty,
					child_enum_member,
				);
				let guard = if *opt {
					let occurs = Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.occurs.into(),
							arguments: vec![field_at_p],
						},
					);
					Some(match realised {
						Some(realised) => Expression::new(
							self.db,
							&self.model,
							item,
							LookupCall {
								function: self.ids.functions.and.into(),
								arguments: vec![realised, occurs],
							},
						),
						None => occurs,
					})
				} else {
					realised
				};
				(singleton, guard)
			}
		};
		let template = match guard {
			Some(guard) => {
				let empty_set = Expression::new(self.db, &self.model, item, SetLiteral(vec![]));
				Expression::new(
					self.db,
					&self.model,
					item,
					IfThenElse {
						branches: vec![Branch::new(guard, contribution)],
						else_result: Box::new(empty_set),
					},
				)
			}
			None => contribution,
		};
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![p_idx],
					collection: parent_index_set,
					where_clause: None,
				}],
				template,
			),
		))
	}

	/// The per-contribution comprehension for a superclass projection: the
	/// identity image of the (already-derived) direct class's actual set,
	/// `[if <D-identity(i)> in D then {<Super>_occ_j(<local i>)} else {}
	/// endif | i in index_set(<D storage>)]`. The direct class's slot i and
	/// the superclass's constructor-local ordinal coincide (the occurrence
	/// contributes the same slots to every projection target 1:1, the
	/// `project_class_identity` arithmetic). The guard is skipped for
	/// non-var-actual direct classes (all slots realised).
	pub(in crate::lower) fn superclass_projection_contribution_expr(
		&mut self,
		item: Item<'db>,
		super_class: PatternRef<'db>,
		super_constructor_index: usize,
		direct_class: PatternRef<'db>,
		direct_constructor_index: usize,
	) -> Option<Expression<'db>> {
		let direct_decl_idx =
			self.class_object_contribution_declaration(direct_class, direct_constructor_index)?;
		let direct_expr = Expression::new(
			self.db,
			&self.model,
			item,
			ResolvedIdentifier::Declaration(direct_decl_idx),
		);
		let direct_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![direct_expr.clone()],
			},
		);
		let direct_index_ty = match direct_expr.ty().lookup(self.db) {
			TyData::Array { dim, .. } => *dim,
			_ => Ty::par_int(self.db),
		};
		let p_decl = Declaration::new(false, Domain::unbounded(self.db, item, direct_index_ty));
		let p_idx = self
			.model
			.add_declaration(DeclarationItem::new(p_decl, item));
		let p_expr = Expression::new(self.db, &self.model, item, p_idx);

		let ordinal = self.contribution_local_ordinal(
			item,
			direct_class,
			direct_constructor_index,
			direct_index_ty,
			p_expr.clone(),
		);
		let super_enum_member = EnumMemberId::new(
			self.class_map[&super_class].class_enum,
			super_constructor_index as u32,
		);
		let super_identity = Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(super_enum_member),
				arguments: vec![ordinal],
			},
		);
		let singleton =
			Expression::new(self.db, &self.model, item, SetLiteral(vec![super_identity]));
		let template = if self
			.object_lowering
			.var_actual_set_classes
			.contains(&direct_class)
		{
			let direct_identity = self.contribution_slot_identity(
				item,
				direct_class,
				direct_constructor_index,
				direct_index_ty,
				p_expr,
			);
			let direct_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(self.class_map[&direct_class].class_set),
			);
			let realised = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.in_.into(),
					arguments: vec![direct_identity, direct_set_expr],
				},
			);
			let empty_set = Expression::new(self.db, &self.model, item, SetLiteral(vec![]));
			Expression::new(
				self.db,
				&self.model,
				item,
				IfThenElse {
					branches: vec![Branch::new(realised, singleton)],
					else_result: Box::new(empty_set),
				},
			)
		} else {
			singleton
		};
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![p_idx],
					collection: direct_index_set,
					where_clause: None,
				}],
				template,
			),
		))
	}

	/// Channel a singular `new`/`opt new` storage field to its STATIC per-slot
	/// identity: `forall(p)(<guard> -> <parent>[p].<field> = <Child>_occ_k(p))`.
	///
	/// The var storage field is a *free* `(var) (opt) <Child>_potential`
	/// decision read through from `_storage` — nothing else ties its value to
	/// the slot's own potential identity, so without this pin the field could
	/// point at a sibling slot's identity while the derived actual set claims
	/// the static one (and two parents could alias one child). The guard is
	/// `occurs(<field>)` for opt fields (an absent field pins nothing), the
	/// parent slot's realisation for non-opt fields of var-actual parents
	/// (an unrealised slot's field is symmetry-pinned to `lb`, which need not
	/// be the static identity), and nothing otherwise. Par fields are skipped:
	/// their values are minted statically by the reconstruction engine.
	pub(in crate::lower) fn emit_singular_field_identity_pin(
		&mut self,
		item: Item<'db>,
		intro: &FieldIntroduction<'db>,
		field_ty: Ty<'db>,
		parent_expr: &Expression<'db>,
		parent_index_ty: Ty<'db>,
		child_enum_member: EnumMemberId<'db>,
	) {
		if field_ty.inst(self.db) != Some(VarType::Var) {
			return;
		}
		let FieldIntroductionKind::Singular { opt, .. } = &intro.kind else {
			return;
		};
		let parent_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![parent_expr.clone()],
			},
		);
		let p_decl = Declaration::new(false, Domain::unbounded(self.db, item, parent_index_ty));
		let p_idx = self
			.model
			.add_declaration(DeclarationItem::new(p_decl, item));
		let p_expr = Expression::new(self.db, &self.model, item, p_idx);
		let parent_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![parent_expr.clone(), p_expr.clone()],
			},
		);
		let field_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(parent_at_p),
				field: intro.attribute,
			},
		);
		let ordinal = self.contribution_local_ordinal(
			item,
			intro.parent_class,
			intro.parent_contribution_index,
			parent_index_ty,
			p_expr.clone(),
		);
		let identity = Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(child_enum_member),
				arguments: vec![ordinal],
			},
		);
		let field_eq_identity = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.eq.into(),
				arguments: vec![field_at_p.clone(), identity],
			},
		);
		let guard = if *opt {
			Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.occurs.into(),
					arguments: vec![field_at_p],
				},
			))
		} else if self
			.object_lowering
			.var_actual_set_classes
			.contains(&intro.parent_class)
		{
			let parent_identity = self.contribution_slot_identity(
				item,
				intro.parent_class,
				intro.parent_contribution_index,
				parent_index_ty,
				p_expr,
			);
			let parent_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(self.class_map[&intro.parent_class].class_set),
			);
			Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.in_.into(),
					arguments: vec![parent_identity, parent_set_expr],
				},
			))
		} else {
			None
		};
		let template = match guard {
			Some(guard) => Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.implies.into(),
					arguments: vec![guard, field_eq_identity],
				},
			),
			None => field_eq_identity,
		};
		let compr = Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![p_idx],
					collection: parent_index_set,
					where_clause: None,
				}],
				template,
			),
		);
		let forall = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![compr],
			},
		);
		let _ = self
			.model
			.add_constraint(ConstraintItem::new(Constraint::new(true, forall), item));
	}

	/// Build the symmetry-breaking default expression for a storage-field
	/// access. Returns `None` when the field type has no canonical default
	/// (e.g. functions). Numeric fields route through `mzn_safe_default`
	/// (`lb` of an unbounded var is `-infinity`, and `field = -infinity` is
	/// invalid — the helper picks the first finite bound, falling back to
	/// `0`); bools, enums, `<Class>_potential` refs, and var sets use `lb`
	/// directly (always a valid finite default — `false` / first member /
	/// `{}`); `<>` for any opt field; field-wise recursion for records and
	/// tuples.
	pub(in crate::lower) fn build_field_default_expr(
		&mut self,
		item: Item<'db>,
		field_access_expr: Expression<'db>,
	) -> Option<Expression<'db>> {
		let db = self.db;
		let ty = field_access_expr.ty();
		if ty.opt(db) == Some(OptType::Opt) {
			return Some(Expression::new(self.db, &self.model, item, Absent));
		}
		match ty.lookup(db) {
			TyData::Integer(_, _) | TyData::Float(_, _) => Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.mzn_safe_default.into(),
					arguments: vec![field_access_expr],
				},
			)),
			TyData::Boolean(_, _) | TyData::Enum(_, _, _) | TyData::Set(_, _, _) => {
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.lb.into(),
						arguments: vec![field_access_expr],
					},
				))
			}
			TyData::Record(_, fs) => {
				let fs = fs.clone();
				let mut record_fields: Vec<(Identifier<'db>, Expression<'db>)> =
					Vec::with_capacity(fs.len());
				for (field_id, _) in fs.iter() {
					let field_ident = Identifier(*field_id);
					let inner_access = Expression::new(
						self.db,
						&self.model,
						item,
						RecordAccess {
							record: Box::new(field_access_expr.clone()),
							field: field_ident,
						},
					);
					let inner_default = self.build_field_default_expr(item, inner_access)?;
					record_fields.push((field_ident, inner_default));
				}
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					RecordLiteral(record_fields),
				))
			}
			TyData::Tuple(_, fs) => {
				let len = fs.len();
				let mut tuple_fields: Vec<Expression<'db>> = Vec::with_capacity(len);
				for i in 0..len {
					let inner_access = Expression::new(
						self.db,
						&self.model,
						item,
						TupleAccess {
							tuple: Box::new(field_access_expr.clone()),
							field: IntegerLiteral((i + 1) as i64),
						},
					);
					let inner_default = self.build_field_default_expr(item, inner_access)?;
					tuple_fields.push(inner_default);
				}
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					TupleLiteral(tuple_fields),
				))
			}
			TyData::Array { .. } => {
				// Pin an array field element-wise, re-indexed to the field's
				// own index sets:
				// `f = arrayXd(f, [<default>(f[j]) | j in index_set(f)])`.
				let j_decl =
					Declaration::new(false, Domain::unbounded(self.db, item, Ty::par_int(db)));
				let j_idx = self
					.model
					.add_declaration(DeclarationItem::new(j_decl, item));
				let j_expr = Expression::new(self.db, &self.model, item, j_idx);
				let f_at_j = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![field_access_expr.clone(), j_expr],
					},
				);
				let inner_default = self.build_field_default_expr(item, f_at_j)?;
				let index_set = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.index_set.into(),
						arguments: vec![field_access_expr.clone()],
					},
				);
				let compr = Expression::new(
					self.db,
					&self.model,
					item,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![j_idx],
							collection: index_set,
							where_clause: None,
						}],
						inner_default,
					),
				);
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_xd.into(),
						arguments: vec![field_access_expr, compr],
					},
				))
			}
			_ => None,
		}
	}

	/// Emit one symmetry-breaking constraint per defaultable storage field of
	/// `class_pattern`:
	///
	/// ```ignore
	/// constraint forall(x in <C>_potential)
	///                  (x in <C> \/ <C>_objects[x].<f> = <default>(x));
	/// ```
	///
	/// This is the stdlib-thinner rewrite of
	/// `forall(x in <C>_potential diff <C>)(...)`: keeping the iterator par
	/// avoids dispatching a var-set generator and instead half-reifies the
	/// membership through a disjunction. Skips fields without a canonical
	/// default (see `build_field_default_expr`).
	///
	/// We don't short-circuit on potential cardinality at THIR time because
	/// the cardinality of e.g. `A_occ_0(1..n)` is a runtime parameter — even
	/// when there is only one constructor. Instead we skip whenever the
	/// class actual set is statically pinned: par-typed AND defined. That
	/// covers singular `var new C` and `var opt new C` (definition is the
	/// full potential constructor) and singular field-only chains that fell
	/// back to defining `<C>` as the full enum — in both shapes no potential
	/// can ever be unused so the recipe is vacuous. The `array_union(...)`
	/// definition path widens `<C>` to var and stays in the emit set.
	pub(in crate::lower) fn emit_unused_potential_default_constraints(
		&mut self,
		class_pattern: PatternRef<'db>,
	) {
		let Some(class_info) = self.class_map.get(&class_pattern).copied() else {
			return;
		};
		let class_enum = class_info.class_enum;
		let class_set = class_info.class_set;
		let class_objects = class_info.class_objects;

		let class_set_decl = &self.model[class_set];
		let class_set_is_var = class_set_decl.ty().inst(self.db) == Some(VarType::Var);
		let class_set_defined = class_set_decl.definition().is_some();
		if !class_set_is_var && class_set_defined {
			return;
		}

		// Skip if the stdlib isn't loaded: the recipe needs `lb`,
		// `enum2int`, `forall`, etc. The cases that survive the gate
		// above without stdlib (e.g. `array of var new`) realise every
		// potential by construction, so the constraint is vacuous and
		// skipping is safe.
		let par_int_ty = Ty::par_int(self.db);
		let par_enum_ty = Ty::par_enum(self.db, self.model[class_enum].enum_type());
		if self
			.model
			.lookup_function(self.db, self.ids.functions.lb.into(), &[par_enum_ty])
			.is_err() || self
			.model
			.lookup_function(self.db, self.ids.functions.enum2int.into(), &[par_enum_ty])
			.is_err() || self
			.model
			.lookup_function(
				self.db,
				self.ids.functions.forall.into(),
				&[Ty::array(self.db, par_int_ty, Ty::par_bool(self.db)).unwrap()],
			)
			.is_err()
		{
			return;
		}

		let item = class_pattern.item(self.db);
		let fields = self.class_storage_fields_for_domain(class_pattern);

		// Skip the pin for defined fields (computed attributes and
		// domain-dependent fields) when every contribution to this class leaves
		// them functionally determined (alias-defined, or read through from a
		// determined contribution). A determined field's unrealised-slot value
		// is a function of its pinned free siblings, so skipping loses no
		// symmetry breaking — while pinning it to `mzn_safe_default` (its own
		// flatten-time `lb`) is inconsistent whenever the defining RHS evaluated
		// at the siblings' pinned defaults differs from that `lb` (any
		// non-monotone RHS), which forces unrealised potentials into the class
		// set and silently removes solutions. Where some contribution still
		// fresh-mints a defined field, the pin stays load-bearing and is kept.
		let skip_defined_fields = self
			.class_contributions_all_determined
			.get(&class_pattern)
			.copied()
			.unwrap_or(false);
		let defined_fields: FxHashSet<Identifier<'db>> = if skip_defined_fields {
			self.class_storage_field_decls(class_pattern.item(self.db))
				.into_iter()
				.filter(|d| {
					d.definition.is_some()
						|| self.field_domain_references_attribute(d.owner, d.declared_type)
				})
				.map(|d| d.ident)
				.collect()
		} else {
			Default::default()
		};

		let enum_ref = self.model[class_enum].enum_type();
		let x_ty = Ty::par_enum(self.db, enum_ref);
		// `<C>_objects` may be indexed by `int` (top-level storage) or by
		// `<C>_potential` (field-only chains whose contribution decl was
		// declared with the enum-typed dimension). Read the array's
		// dimension type and only emit `enum2int(x)` when we need a par
		// int index — passing enum2int into an enum-typed dimension would
		// itself fail to dispatch.
		let class_objects_index_is_int = match self.model[class_objects].ty().lookup(self.db) {
			TyData::Array { dim, .. } => *dim != x_ty,
			_ => return,
		};

		for (field_ident, field_ty) in fields {
			if defined_fields.contains(&field_ident) {
				continue;
			}
			// Structured (tuple/record) fields are pinned LEAF-WISE — one
			// forall per scalar leaf, `f.1 = <default>(f.1)` — instead of one
			// whole-value equality `f = (<default>, ...)`. The whole-value
			// form cannot be evaluated by the target MiniZinc when the field
			// reads a var-tuple-containing record through the generic `'[]'`
			// helper inside the reified disjunction (an upstream limitation);
			// component pins evaluate fine and pin exactly the same values.
			// An opt structured field stays a single leaf (`f = <>`).
			for leaf_path in Self::pin_leaf_paths(self.db, field_ty) {
				let mut x_decl = Declaration::new(false, Domain::unbounded(self.db, item, x_ty));
				x_decl.set_name(Identifier::new(self.db, "x"));
				let x_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(x_decl, item));
				let x_expr = Expression::new(self.db, &self.model, item, x_decl_idx);

				let class_objects_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_objects),
				);
				let x_index = if class_objects_index_is_int {
					Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.enum2int.into(),
							arguments: vec![x_expr.clone()],
						},
					)
				} else {
					x_expr.clone()
				};
				let object_at_x = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![class_objects_expr, x_index],
					},
				);
				let field_at_x = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(object_at_x),
						field: field_ident,
					},
				);
				let mut leaf_at_x = field_at_x;
				for step in leaf_path.iter() {
					leaf_at_x = match step {
						PinLeafStep::Tuple(i) => Expression::new(
							self.db,
							&self.model,
							item,
							TupleAccess {
								tuple: Box::new(leaf_at_x),
								field: IntegerLiteral(*i),
							},
						),
						PinLeafStep::Record(ident) => Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(leaf_at_x),
								field: *ident,
							},
						),
					};
				}
				let Some(default_expr) = self.build_field_default_expr(item, leaf_at_x.clone())
				else {
					continue;
				};
				let eq_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.eq.into(),
						arguments: vec![leaf_at_x, default_expr],
					},
				);

				let class_set_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_set),
				);
				let in_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.in_.into(),
						arguments: vec![x_expr, class_set_expr],
					},
				);

				let disj_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.or.into(),
						arguments: vec![in_call, eq_call],
					},
				);

				let enum_set_expr = Expression::new(self.db, &self.model, item, class_enum);
				let compr = Expression::new(
					self.db,
					&self.model,
					item,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![x_decl_idx],
							collection: enum_set_expr,
							where_clause: None,
						}],
						disj_call,
					),
				);
				let forall_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.forall.into(),
						arguments: vec![compr],
					},
				);

				let _ = self.model.add_constraint(ConstraintItem::new(
					Constraint::new(true, forall_call),
					item,
				));
			}
		}
	}

	/// The scalar-leaf access paths of a storage field type for the
	/// unused-potential pins: a non-structured (or opt) type is a single
	/// leaf at the empty path; tuple/record types expand field-wise. See
	/// the leaf-wise pin note in
	/// `emit_unused_potential_default_constraints`.
	pub(in crate::lower) fn pin_leaf_paths(
		db: &'db dyn Db,
		ty: Ty<'db>,
	) -> Vec<Vec<PinLeafStep<'db>>> {
		let mut out = Vec::new();
		let mut todo: Vec<(Vec<PinLeafStep<'db>>, Ty<'db>)> = vec![(Vec::new(), ty)];
		while let Some((path, t)) = todo.pop() {
			if t.opt(db) == Some(OptType::Opt) {
				out.push(path);
				continue;
			}
			match t.lookup(db) {
				TyData::Tuple(_, fs) => {
					for (i, f) in fs.iter().enumerate() {
						let mut p = path.clone();
						p.push(PinLeafStep::Tuple((i + 1) as i64));
						todo.push((p, *f));
					}
				}
				TyData::Record(_, fs) => {
					for (ident, f) in fs.iter() {
						let mut p = path.clone();
						p.push(PinLeafStep::Record(Identifier(*ident)));
						todo.push((p, *f));
					}
				}
				_ => out.push(path),
			}
		}
		out
	}
}
