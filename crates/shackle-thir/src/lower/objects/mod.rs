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
	Item,
	class_analysis::{
		LocalDomainSource, OccurrenceContribution, OccurrenceId, OccurrenceSource,
		analyse_new_objects, introduces_var_existence,
	},
	ids::PatternRef,
};

use crate::*;

mod class_item;
mod class_set;
mod contribution;
mod defaults;
mod engine;
mod field_reconstruction;
mod finish;
mod guards;
mod nested_emission;
mod nested_storage;
mod new_declaration;
mod occurrence;
mod storage;

/// Everything the object lowering accumulates while collecting items,
/// alongside the read-only occurrence plan it is driven by.
///
/// Split out of `ItemCollector` so the objects feature owns its own state:
/// `plan` is derived once from `analyse_new_objects` and never mutated, while
/// the remaining fields accumulate as class items and `new` declarations are
/// collected, and are consumed by `finish`.
pub(in crate::lower) struct ObjectState<'db> {
	/// Derived once from `analyse_new_objects`; read-only afterwards.
	pub(in crate::lower) class_map: FxHashMap<PatternRef<'db>, ClassMapInfo<'db>>,
	pub(in crate::lower) plan: ObjectLoweringPlan<'db>,
	pub(in crate::lower) contribution_end_map:
		FxHashMap<(PatternRef<'db>, usize), DeclarationId<'db>>,
	class_object_contributions: FxHashMap<PatternRef<'db>, Vec<(usize, DeclarationId<'db>)>>,
	/// Per-occurrence slice array declarations (`<parent>_<field>_potential`),
	/// keyed by the child class and contribution index. Consumed by the
	/// per-parent subset constraint.
	slice_array_decls: FxHashMap<(PatternRef<'db>, usize), DeclarationId<'db>>,
	/// Records, per field-only-introduced child class, each parent-field
	/// introduction contributing children — keyed one hop up (the *immediate*
	/// parent class/contribution and the direct attribute name). Materialized
	/// in `finish` as the class's actual-set definition:
	/// `array_union(...)` over per-contribution, ITE-guarded expressions
	/// (see `field_only_class_set_array_union`).
	class_set_field_introductions: FxHashMap<PatternRef<'db>, Vec<FieldIntroduction<'db>>>,
	/// Per top-level-introduced class, the identity-set expressions
	/// contributed by each top-level introduction. Each entry is the
	/// constructor's `contribution_index` paired with a contribution
	/// expression:
	///   - `set of new`, `var set(...) of new`: the user-named decl
	///     reference (typed as `(var) set of <C>_occ_i(...)`).
	///   - `array [d] of new`: the full enum reference (`<C>_potential`)
	///     because every potential is realized by construction; the
	///     `as` decl itself is array-typed and its element type already
	///     references `<C>`, which would be circular if used as the
	///     class-set definition.
	///
	/// Consumed by `finish()` as
	/// `<C> = array_union([expr_1, expr_2, ...])` (single contribution
	/// is assigned directly without the wrapping call).
	class_set_top_level_contributions: FxHashMap<PatternRef<'db>, Vec<(usize, Expression<'db>)>>,
	/// Per class, whether *every* registered `_objects` contribution leaves the
	/// class's defined fields (computed attributes and domain-dependent fields)
	/// functionally determined — alias-defined by the reconstruction chain, or
	/// read through from an already-determined contribution. Only then may the
	/// symmetry-break default wave skip pinning those fields: a pin on a
	/// determined field is at best redundant and at worst inconsistent (a
	/// non-monotone RHS evaluated at the frees' pinned defaults need not equal
	/// the field's own flatten-time `lb`, which forces unrealised potentials
	/// into the class set and silently removes solutions). A contribution that
	/// fresh-mints a defined field (par-reached identity-mode nested storage,
	/// singular-root inheritance projections from raw inputs) still relies on
	/// the pin for symmetry breaking, so any such registration keeps the pins
	/// for the whole class.
	class_contributions_all_determined: FxHashMap<PatternRef<'db>, bool>,
	/// Per (class, contribution index): the `defined_fields_determined` flag
	/// each contribution registered with. Projections that read every field
	/// from an already-registered contribution decl inherit exactly that
	/// contribution's determinedness rather than guessing per class.
	contribution_determined_by_index: FxHashMap<(PatternRef<'db>, usize), bool>,
	/// Computed attributes' class-body foralls, deferred to `finish()` so
	/// they can be dropped for classes whose contributions all alias-define
	/// their defined fields (the gated forall-drop). One entry per computed
	/// attribute: (class pattern, class item, attribute, RHS).
	pending_class_definition_foralls: Vec<(
		PatternRef<'db>,
		Item<'db>,
		Identifier<'db>,
		shackle_hir::ExpressionId<'db>,
	)>,
	/// Classes whose actual set must be emitted FREE with a subset lower
	/// bound rather than an `=` union definition, because a `var opt new`
	/// root reaches the class. Populated by the opt-root branch of
	/// `collect_declaration` with the opt root's direct class and every
	/// superclass. In `finish()` the definite contributions of such a class
	/// are unioned into a `<union> subset <C>` constraint (lower bound); the
	/// upper bound is the declaration domain `<C>_potential`, and the opt
	/// occurrence's own membership stays the free decision.
	opt_free_subset_classes: FxHashSet<PatternRef<'db>>,
	/// `(target_class, constructor_index)` of every contribution belonging to
	/// a `var opt new` root. Skipped by `finish()`'s unregistered-contribution
	/// scan so the opt occurrence is never materialised as a definitional (or
	/// lower-bound) union piece — its membership IS the decision, and its
	/// superclass image is pinned by an occurs biconditional instead.
	opt_contribution_slots: FxHashSet<(PatternRef<'db>, usize)>,
}

impl<'db> ObjectState<'db> {
	pub(in crate::lower) fn new(db: &'db dyn Db) -> Self {
		Self {
			plan: ObjectLoweringPlan::new(db),
			class_map: FxHashMap::default(),
			contribution_end_map: FxHashMap::default(),
			class_object_contributions: FxHashMap::default(),
			slice_array_decls: FxHashMap::default(),
			class_set_field_introductions: FxHashMap::default(),
			class_set_top_level_contributions: FxHashMap::default(),
			class_contributions_all_determined: FxHashMap::default(),
			contribution_determined_by_index: FxHashMap::default(),
			pending_class_definition_foralls: Vec::new(),
			opt_free_subset_classes: FxHashSet::default(),
			opt_contribution_slots: FxHashSet::default(),
		}
	}
}

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
