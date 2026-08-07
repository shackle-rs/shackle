//! Functionality for converting HIR nodes into THIR nodes.
//!
//! The following is performed during lowering:
//! - Assignment items are moved into declarations/constraints
//! - Destructuring declarations are rewritten as separate declarations
//! - Destructuring in generators is rewritten into a where clause
//! - Type alias items removed as they have been resolved
//! - 2D array literals are re-written using `mzn_array_kd` calls
//! - Indexed array literals are re-written using `mzn_indexed_array` calls
//! - Array access and slicing is re-written using calls to `[]`
//! - Tuple/record access into arrays of structs are rewritten using a
//!   comprehension accessing the inner value

// Emitted model item order must not depend on hash order: `PatternRef` and
// friends hash their salsa ids, so iterating a map/set keyed by them yields an
// order that shifts whenever interning order does (a different standard
// library is enough). That produced snapshot failures on CI that were pure
// reorderings and unreproducible locally. Sort into source order — see
// `ObjectLoweringPlan::class_rank` — or justify the exception in place.
#![deny(clippy::iter_over_hash_type)]

use derive_more::From;
use rustc_hash::{FxHashMap, FxHashSet};
use shackle_hir::{
	ClassMember, Item, PatternTy, TypeResult,
	class_analysis::{
		LocalDomainSource, OccurrenceContribution, OccurrenceId, OccurrenceSource,
		analyse_new_objects, class_pattern_for, introduces_var_existence,
	},
	constants::IdentifierRegistry,
	counts::EntityCounts,
	ids::{EntityRef, ExpressionRef, PatternRef},
	run_hir_phase,
};
use shackle_ty::{EnumRef, Ty, TyData};

use super::{source::Origin, *};
use crate::{Db, db::Intermediate};

mod expression;

// Imported by name rather than glob: `alloc_expression` would otherwise be
// ambiguous with `crate::traverse::fold::alloc_expression`, which `super::*`
// brings into scope.
use self::expression::{ExpressionCollector, alloc_expression};

#[derive(Copy, Clone, Debug, PartialEq, Eq, From)]
enum DeclOrConstraint<'db> {
	Declaration(DeclarationId<'db>),
	Constraint(ConstraintId<'db>),
}

impl<'db> From<DeclOrConstraint<'db>> for LetItem<'db> {
	fn from(d: DeclOrConstraint<'db>) -> Self {
		match d {
			DeclOrConstraint::Constraint(c) => LetItem::Constraint(c),
			DeclOrConstraint::Declaration(d) => LetItem::Declaration(d),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredAnnotation<'db> {
	Items(Vec<DeclOrConstraint<'db>>),
	Expression(Expression<'db>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredIdentifier<'db> {
	ResolvedIdentifier(ResolvedIdentifier<'db>),
	Callable(Callable<'db>),
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
	ident: Identifier<'db>,
	pattern: PatternRef<'db>,
	definition: Option<shackle_hir::ExpressionId<'db>>,
	declared_type: shackle_hir::TypeId<'db>,
	owner: Item<'db>,
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
	var_reached_classes: FxHashSet<PatternRef<'db>>,
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

/// Collects HIR items and lowers them to THIR
struct ItemCollector<'db> {
	db: &'db dyn Db,
	ids: &'db IdentifierRegistry<'db>,
	resolutions: FxHashMap<PatternRef<'db>, LoweredIdentifier<'db>>,
	param_defaults: FxHashMap<DeclarationId<'db>, Expression<'db>>,
	class_map: FxHashMap<PatternRef<'db>, ClassMapInfo<'db>>,
	object_lowering: ObjectLoweringPlan<'db>,
	model: Model<'db>,
	type_alias_expressions: FxHashMap<ExpressionRef<'db>, DeclarationId<'db>>,
	deferred: Vec<(FunctionId<'db>, Item<'db>)>,
	contribution_end_map: FxHashMap<(PatternRef<'db>, usize), DeclarationId<'db>>,
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

impl<'db> ItemCollector<'db> {
	/// Create a new item collector
	fn new(
		db: &'db dyn Db,
		ids: &'db IdentifierRegistry<'db>,
		entity_counts: &EntityCounts,
	) -> Self {
		Self {
			db,
			ids,
			resolutions: FxHashMap::default(),
			param_defaults: FxHashMap::default(),
			class_map: FxHashMap::default(),
			object_lowering: ObjectLoweringPlan::new(db),
			model: Model::with_capacities(&entity_counts.into()),
			type_alias_expressions: FxHashMap::default(),
			deferred: Vec::new(),
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

	/// Collect an item
	fn collect_item(&mut self, item: Item<'db>) {
		log::debug!(
			"Lowering {:?} at {} to THIR",
			item.get_item_with_data_as_debug(self.db),
			Origin::from(item).pretty_print(self.db)
		);
		match item {
			Item::Annotation(a) => {
				let _ = self.collect_annotation(a);
			}
			Item::Assignment(a) => self.collect_assignment(a),
			Item::Constraint(c) => {
				let _ = self.collect_constraint(item, c.constraint(self.db), true);
			}
			Item::Declaration(d) => {
				let _ = self.collect_declaration(item, d.declaration(self.db), true);
			}
			Item::Enumeration(e) => {
				let _ = self.collect_enumeration(e);
			}
			Item::EnumAssignment(a) => self.collect_enumeration_assignment(a),
			Item::Function(f) => {
				let _ = self.collect_function(f);
			}
			Item::Output(o) => {
				let _ = self.collect_output(o);
			}
			Item::Solve(s) => self.collect_solve(s),
			Item::TypeAlias(t) => self.collect_type_alias(t),
			Item::Class(c) => self.collect_class(c),
		}
	}

	/// Collect an annotation item
	fn collect_annotation(&mut self, it: shackle_hir::AnnotationItem<'db>) -> AnnotationId<'db> {
		let item: Item<'_> = it.into();
		let a = it.annotation(self.db);
		let types = item.types(self.db);
		let ty = &types[a.constructor_pattern()];
		match (&a.constructor, ty) {
			(shackle_hir::Constructor::Atom { pattern }, PatternTy::AnnotationAtom) => {
				let annotation = Annotation::new(
					a[*pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				let idx = self
					.model
					.add_annotation(AnnotationItem::new(annotation, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
				);
				idx
			}
			(
				shackle_hir::Constructor::Function {
					constructor,
					destructor,
					parameters: params,
				},
				PatternTy::AnnotationConstructor(fn_entry),
			) => {
				let parameters = params
					.iter()
					.zip(fn_entry.overload.params())
					.map(|(param, ty)| self.collect_fn_param(param, *ty, a.data(), item, &types))
					.collect::<Vec<_>>();

				let mut annotation = Annotation::new(
					a[*constructor]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				annotation.parameters = Some(parameters);
				let idx = self
					.model
					.add_annotation(AnnotationItem::new(annotation, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *constructor),
					LoweredIdentifier::Callable(Callable::Annotation(idx)),
				);
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *destructor),
					LoweredIdentifier::Callable(Callable::AnnotationDestructure(idx)),
				);

				idx
			}
			_ => unreachable!(),
		}
	}

	/// Collect an assignment item
	fn collect_assignment(&mut self, it: shackle_hir::AssignmentItem<'db>) {
		let item: Item<'_> = it.into();
		let db = self.db;
		let a = it.assignment(db);
		let types = item.types(db);
		let res = types.name_resolution(a.assignee).unwrap();
		let decl = match &self.resolutions[&res] {
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Declaration(d)) => *d,
			_ => unreachable!(),
		};
		if self.model[decl].definition().is_some() {
			// Turn subsequent assignment items into equality constraints
			let mut collector = ExpressionCollector::new(self, a.data(), item, &types);
			let call = LookupCall {
				function: collector.parent.ids.functions.eq.into(),
				arguments: vec![
					collector.collect_expression(a.assignee),
					collector.collect_expression(a.definition),
				],
			};
			let constraint = Constraint::new(
				true,
				Expression::new(db, &collector.parent.model, item, call),
			);
			let _ = collector
				.parent
				.model
				.add_constraint(ConstraintItem::new(constraint, item));
		} else {
			let mut declaration = self.model[decl].clone();
			let mut collector = ExpressionCollector::new(self, a.data(), item, &types);
			let def = collector.collect_expression(a.definition);
			declaration.set_definition(def);
			self.model[decl] = declaration;
		}
	}

	/// Collect a constraint item
	fn collect_constraint(
		&mut self,
		item: Item<'db>,
		c: &shackle_hir::Constraint<'db>,
		top_level: bool,
	) -> ConstraintId<'db> {
		let db = self.db;
		let types = item.types(db);
		let mut collector = ExpressionCollector::new(self, item.data(db), item, &types);
		let mut constraint = Constraint::new(top_level, collector.collect_expression(c.expression));
		constraint.annotations_mut().extend(
			c.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model
			.add_constraint(ConstraintItem::new(constraint, item))
	}

	/// Collect a declaration item
	fn collect_declaration(
		&mut self,
		item: Item<'db>,
		d: &shackle_hir::Declaration<'db>,
		top_level: bool,
	) -> Vec<DeclOrConstraint<'db>> {
		let db = self.db;
		let types = item.types(db);
		let ty = match &types[d.pattern] {
			PatternTy::Variable(ty) => *ty,
			PatternTy::Destructuring(ty) => *ty,
			_ => unreachable!(),
		};
		let data = item.data(db);
		let mut collector = ExpressionCollector::new(self, data, item, &types);
		let root_pattern = PatternRef::new(db, item, d.pattern);
		let uses_occurrence_lowering = data[d.declared_type].is_new(data)
			&& collector
				.parent
				.maybe_top_level_occurrence(root_pattern)
				.is_some();
		let decl = if uses_occurrence_lowering {
			collector
				.parent
				.collect_new_declaration(ty, &types, item, d, data, top_level)
		} else {
			let domain = collector.collect_domain(d.declared_type, ty, false);
			let mut decl = Declaration::new(top_level, domain);
			if let Some(def) = d.definition {
				decl.set_definition(collector.collect_expression_as(def, ty));
			}
			decl
		};

		let idx = collector
			.parent
			.model
			.add_declaration(DeclarationItem::new(decl, item));
		let mut ids = vec![idx.into()];

		// Top-level set/array `new` introductions register their identity-set
		// declaration as a contribution to the direct class's actual set;
		// `finish()` then defines `<C>` as `array_union([decl_1, decl_2, ...])`
		// (or just `decl_1` when there's only one contribution).
		// Singular `(var [opt]) new C: x` cases set the class set definition
		// inline (see `collect_new_declaration`); registering here is harmless
		// because the `finish()` wave skips classes whose definition is
		// already set.
		if uses_occurrence_lowering
			&& matches!(
				&data[d.declared_type],
				shackle_hir::Type::Set { .. } | shackle_hir::Type::Array { .. }
			) && let Some(class_domain) = data[d.declared_type].get_new_class(data)
		{
			let class_pattern_ref = types.name_resolution(class_domain).unwrap();
			if let Some(class_info) = collector.parent.class_map.get(&class_pattern_ref).copied() {
				let root_occurrence = collector.parent.top_level_occurrence(root_pattern);
				let contributions = collector.parent.object_lowering.contributions_by_occurrence
					[&root_occurrence]
					.clone();
				// `set of new` / `var set(...) of new` contribute the
				// user-named identity-set decl directly. `array of new`
				// contributes its own contribution BLOCK
				// (`<C>_occ_k(1..n)` — every slot of an array root is
				// realized by construction): the array decl's element type
				// references `<C>` so the decl itself would be circular as
				// the class-set RHS, and registering the full potential enum
				// would force OTHER contributions' potentials into the class
				// set the moment introductions mixed (an
				// `array [1..2] of new A` root plus `var set(1..2) of new A`
				// pinned both var slots realised).
				let contribution_expr = match &data[d.declared_type] {
					shackle_hir::Type::Set { .. } => Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						ResolvedIdentifier::Declaration(idx),
					),
					shackle_hir::Type::Array { .. } => {
						let block_set = collector.parent.par_contribution_block_set(
							item,
							class_pattern_ref,
							collector
								.parent
								.occurrence_contribution(root_occurrence, class_pattern_ref)
								.constructor_index,
						);
						match block_set {
							Some(block_set) => block_set,
							// No chained block boundaries: fall back to the
							// full potential enum, exact whenever the array
							// root is the only contribution.
							None => Expression::new(
								collector.parent.db,
								&collector.parent.model,
								item,
								class_info.class_enum,
							),
						}
					}
					_ => unreachable!(),
				};
				for contribution in contributions {
					if contribution.target_class == class_pattern_ref {
						collector.parent.register_class_set_top_level_contribution(
							class_pattern_ref,
							contribution.constructor_index,
							contribution_expr.clone(),
						);
					}
				}
			}
		}

		if uses_occurrence_lowering
			&& matches!(
				&data[d.declared_type],
				shackle_hir::Type::New { .. } | shackle_hir::Type::Set { .. }
			) {
			// This fires for every `new` root, including a PAR singular
			// `new A: a`. A par singular root reconstructs its owned objects
			// from par input records, but a var-existence object field
			// (`var set of new D`) transitively below it is realised as a free
			// var subset that MUST be confined to its per-parent block — that
			// constraint is emitted by the walk below. Firing for par roots is
			// safe: the walk itself is side-effect-free (pure occurrence
			// lookups) and `emit_per_parent_subset_constraint` no-ops (returns
			// `None` before creating any decl) unless the child has a slice
			// array, which is registered only for identity-typed
			// `var set of <child>` storage — i.e. exactly the var-existence
			// fields. Par `set of new` fields get dense length-sized universes
			// with no slice, so no subset constraint is emitted for them.
			let class_domain = data[d.declared_type]
				.get_new_class(data)
				.expect("bounded object declarations should have a class domain");
			let class_pattern_ref = types.name_resolution(class_domain).unwrap();
			let root_occurrence = collector.parent.top_level_occurrence(root_pattern);
			// Walk the class graph starting from the root decl, emitting a
			// per-parent subset constraint at every (parent_class, field)
			// pair whose child occurrence is `FlattenedChildCollection`. The
			// recursion is needed for nested `set of new` chains like
			// `Expedition.vehicles : set of new Vehicle` containing
			// `Vehicle.crew : set of new CrewMember`: without it, only the
			// outer `e.vehicles ⊆ vehicles_potential[e]` constraint would be
			// emitted and the inner `v.crew` would range over the full
			// `CrewMember_potential` universe.
			let mut stack: Vec<(PatternRef<'db>, OccurrenceId, Vec<Identifier<'db>>)> =
				vec![(class_pattern_ref, root_occurrence, Vec::new())];
			while let Some((parent_class, parent_occurrence, path)) = stack.pop() {
				for (field_ident, field_ty) in collector
					.parent
					.class_storage_fields(parent_class)
					.into_iter()
				{
					let Some(child_class_ref) = field_ty.class_type(collector.parent.db) else {
						continue;
					};
					let child_class = class_pattern_for(collector.parent.db, child_class_ref)
						.expect("class item for class type");
					let mut child_path = path.clone();
					child_path.push(field_ident);
					let Some(child_occurrence) = collector
						.parent
						.maybe_nested_occurrence(root_pattern, &child_path)
					else {
						continue;
					};
					if matches!(
						collector
							.parent
							.occurrence_local_domain_source(child_occurrence),
						LocalDomainSource::FlattenedChildCollection
					) && let Some(constraint_idx) =
						collector.parent.emit_per_parent_subset_constraint(
							item,
							top_level,
							parent_occurrence,
							parent_class,
							field_ident,
							child_occurrence,
							child_class,
						) {
						ids.push(constraint_idx.into());
					}
					stack.push((child_class, child_occurrence, child_path));
				}
			}
		}
		let decls = collector.collect_destructuring(idx, top_level, d.pattern);
		collector.parent.model[idx]
			.annotations_mut()
			.reserve(d.annotations.len());
		for ann in d.annotations.iter().copied() {
			match collector.collect_declaration_annotation(idx, ann) {
				LoweredAnnotation::Expression(e) => {
					collector.parent.model[idx].annotations_mut().push(e)
				}
				LoweredAnnotation::Items(items) => ids.extend(items),
			}
		}
		ids.extend(decls.into_iter().map(DeclOrConstraint::Declaration));
		ids
	}

	/// Collect an enumeration item
	fn collect_enumeration(&mut self, it: shackle_hir::EnumerationItem<'db>) -> EnumerationId<'db> {
		let item: Item<'_> = it.into();
		let e = it.enumeration(self.db);
		let db = self.db;
		let types = item.types(db);
		let ty = &types[e.pattern];
		match ty {
			PatternTy::Enum(ty) => match ty.lookup(self.db) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(self.db) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = Enumeration::new(*t);
							{
								let mut collector =
									ExpressionCollector::new(self, e.data(), item, &types);
								enumeration.annotations_mut().extend(
									e.annotations
										.iter()
										.map(|ann| collector.collect_expression(*ann)),
								);
							}
							if let Some(def) = &e.definition {
								enumeration.set_definition(
									def.iter()
										.map(|c| self.collect_enum_case(c, e.data(), item, &types)),
								)
							}
							let idx = self
								.model
								.add_enumeration(EnumerationItem::new(enumeration, item));
							let _ = self.resolutions.insert(
								PatternRef::new(self.db, item, e.pattern),
								LoweredIdentifier::ResolvedIdentifier(idx.into()),
							);
							self.add_enum_resolutions(
								idx,
								item,
								e.definition.iter().flat_map(|cs| cs.iter()),
							);
							idx
						}
						_ => unreachable!(),
					}
				}
				_ => unreachable!(),
			},
			_ => unreachable!(),
		}
	}

	/// Collect an enum assignment item
	fn collect_enumeration_assignment(&mut self, it: shackle_hir::EnumAssignmentItem<'db>) {
		let item: Item<'_> = it.into();
		let a = it.enum_assignment(self.db);
		let types = item.types(self.db);
		let res = types.name_resolution(a.assignee).unwrap();
		let idx = match &self.resolutions[&res] {
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Enumeration(e)) => *e,
			_ => unreachable!(),
		};
		let def = a
			.definition
			.iter()
			.map(|c| self.collect_enum_case(c, a.data(), item, &types))
			.collect::<Vec<_>>();
		self.model[idx].set_definition(def);
		self.add_enum_resolutions(idx, item, a.definition.iter());
	}

	fn add_enum_resolutions<'a>(
		&mut self,
		idx: EnumerationId<'db>,
		item: Item<'db>,
		ecs: impl Iterator<Item = &'a shackle_hir::EnumConstructor<'db>>,
	) where
		'db: 'a,
	{
		for (i, ec) in ecs.enumerate() {
			match ec {
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Atom { pattern }) => {
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *pattern),
						LoweredIdentifier::ResolvedIdentifier(
							EnumMemberId::new(idx, i as u32).into(),
						),
					);
				}
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Function {
					constructor,
					destructor,
					..
				}) => {
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *constructor),
						LoweredIdentifier::Callable(Callable::EnumConstructor(EnumMemberId::new(
							idx, i as u32,
						))),
					);
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *destructor),
						LoweredIdentifier::Callable(Callable::EnumDestructor(EnumMemberId::new(
							idx, i as u32,
						))),
					);
				}
				_ => (),
			}
		}
	}

	fn collect_enum_case(
		&mut self,
		c: &shackle_hir::EnumConstructor<'db>,
		data: &shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &TypeResult<'db>,
	) -> Constructor<'db> {
		let (name, params) = match (c, &types[c.constructor_pattern()]) {
			(
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Atom { pattern }),
				_,
			) => {
				return Constructor {
					name: data[*pattern].identifier(),
					parameters: None,
				};
			}
			(
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Function {
					constructor,
					parameters,
					..
				}),
				PatternTy::EnumConstructor(ecs),
			) => (
				data[*constructor].identifier(),
				ecs[0]
					.overload
					.params()
					.iter()
					.zip(parameters.iter())
					.map(|(ty, t)| self.collect_fn_param(t, *ty, data, item, types))
					.collect::<Vec<_>>(),
			),
			(
				shackle_hir::EnumConstructor::Anonymous { parameters, .. },
				PatternTy::AnonymousEnumConstructor(f),
			) => (
				None,
				f.overload
					.params()
					.iter()
					.zip(parameters.iter())
					.map(|(ty, t)| self.collect_fn_param(t, *ty, data, item, types))
					.collect::<Vec<_>>(),
			),
			_ => unreachable!(),
		};

		Constructor {
			name,
			parameters: Some(params),
		}
	}

	/// Collect a function item
	fn collect_function(&mut self, it: shackle_hir::FunctionItem<'db>) -> FunctionId<'db> {
		let item: Item<'_> = it.into();
		let f = it.function(self.db);
		let types = item.types(self.db);
		let mut collector = ExpressionCollector::new(self, f.data(), item, &types);
		let res = PatternRef::new(collector.parent.db, item, f.pattern);
		match &types[f.pattern] {
			PatternTy::Function(fn_entry) => {
				let domain =
					collector.collect_domain(f.return_type, fn_entry.overload.return_type(), false);
				let name = f[f.pattern].identifier().unwrap();
				let mut function = Function::new(name.into(), domain);
				function.annotations_mut().extend(
					f.annotations
						.iter()
						.map(|ann| collector.collect_expression(*ann)),
				);
				function.set_type_inst_vars(f.type_inst_vars.iter().map(|t| {
					match &types[t.name] {
						PatternTy::TyVar(tv) => tv.clone(),
						_ => unreachable!(),
					}
				}));

				let parameters = f
					.parameters
					.iter()
					.zip(fn_entry.overload.params())
					.map(|(param, ty)| {
						collector
							.parent
							.collect_fn_param(param, *ty, f.data(), item, &types)
					})
					.collect::<Vec<_>>();
				function.set_parameters(parameters);

				let idx = self.model.add_function(FunctionItem::new(function, item));
				let _ = self
					.resolutions
					.insert(res, LoweredIdentifier::Callable(Callable::Function(idx)));
				if f.body.is_some() {
					self.deferred.push((idx, item));
				}
				idx
			}
			_ => unreachable!(),
		}
	}

	fn collect_fn_param(
		&mut self,
		param: &shackle_hir::Parameter<'db>,
		ty: Ty<'db>,
		data: &shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &TypeResult<'db>,
	) -> DeclarationId<'db> {
		let mut collector = ExpressionCollector::new(self, data, item, types);
		let domain = collector.collect_domain(param.declared_type, ty, false);
		let mut declaration = Declaration::new(false, domain);
		if let Some(p) = param.pattern.and_then(|p| data[p].identifier()) {
			declaration.set_name(p);
		}
		declaration.annotations_mut().extend(
			param
				.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		let default = param.default.map(|def| collector.collect_expression(def));
		let idx = self
			.model
			.add_declaration(DeclarationItem::new(declaration, item));
		if let Some(def) = default {
			let _ = self.param_defaults.insert(idx, def);
		}
		idx
	}

	/// Collect an output item
	fn collect_output(&mut self, it: shackle_hir::OutputItem<'db>) -> OutputId<'db> {
		let item: Item<'_> = it.into();
		let o = it.output(self.db);
		let types = item.types(self.db);
		let mut collector = ExpressionCollector::new(self, o.data(), item, &types);
		let mut output = Output::new(collector.collect_expression(o.expression));
		if let Some(s) = o.section {
			output.set_section(collector.collect_expression(s));
		}
		self.model.add_output(OutputItem::new(output, item))
	}

	/// Collect solve item
	fn collect_solve(&mut self, it: shackle_hir::SolveItem<'db>) {
		let item: Item<'_> = it.into();
		let s = it.solve(self.db);
		let types = item.types(self.db);
		let mut optimise = |pattern: shackle_hir::PatternId<'db>,
		                    objective: shackle_hir::ExpressionId<'db>,
		                    is_maximize: bool| match &types[pattern] {
			PatternTy::Variable(ty) => {
				let objective_origin =
					EntityRef::new(self.db, item, shackle_hir::ids::EntityId::from(objective));
				let mut collector = ExpressionCollector::new(self, s.data(), item, &types);
				let mut declaration = Declaration::new(
					true,
					Domain::unbounded(collector.parent.db, objective_origin, *ty),
				);
				if let Some(name) = s[pattern].identifier() {
					declaration.set_name(name);
				}
				let obj = collector.collect_expression(objective);
				declaration.set_definition(obj);
				let idx = self
					.model
					.add_declaration(DeclarationItem::new(declaration, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
				);
				if is_maximize {
					Solve::maximize(idx)
				} else {
					Solve::minimize(idx)
				}
			}
			_ => unreachable!(),
		};
		let mut si = match &s.goal {
			shackle_hir::Goal::Maximize { pattern, objective } => {
				optimise(*pattern, *objective, true)
			}
			shackle_hir::Goal::Minimize { pattern, objective } => {
				optimise(*pattern, *objective, false)
			}
			shackle_hir::Goal::Satisfy => Solve::satisfy(),
		};
		let mut collector = ExpressionCollector::new(self, s.data(), item, &types);
		si.annotations_mut().extend(
			s.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		let _ = self.model.set_solve(SolveItem::new(si, item));
	}

	fn collect_type_alias(&mut self, it: shackle_hir::TypeAliasItem<'db>) {
		let item: Item<'_> = it.into();
		let ta = it.type_alias(self.db);
		let types = item.types(self.db);
		let data = item.data(self.db);
		for e in shackle_hir::Type::expressions(ta.aliased_type, ta.data()) {
			if let Some(res) = types.name_resolution(e) {
				let res_types = res.item(self.db).types(self.db);
				if matches!(
					&res_types[res.pattern(self.db)],
					PatternTy::TypeAlias { .. }
				) {
					// Skip type aliases inside other type aliases (already will be processed)
					continue;
				}
			}
			// Create a declaration with the value of each expression used in a type alias
			let expression =
				ExpressionCollector::new(self, data, item, &types).collect_expression(e);
			let decl = Declaration::from_expression(self.db, true, expression);
			let idx = self.model.add_declaration(DeclarationItem::new(
				decl,
				EntityRef::new(self.db, item, shackle_hir::ids::EntityId::from(e)),
			));
			let _ = self
				.type_alias_expressions
				.insert(ExpressionRef::new(self.db, item, e), idx);
		}
	}

	/// Collect deferred function bodies
	fn collect_deferred(&mut self) {
		for (func, item) in self.deferred.clone().into_iter() {
			let types = item.types(self.db);
			let data = item.data(self.db);
			match item {
				Item::Function(f) => {
					let mut function = self.model[func].clone();
					let param_decls = function.parameters().to_owned();
					let mut decls = Vec::new();
					let mut collector = ExpressionCollector::new(self, data, item, &types);
					let ff = f.function(collector.parent.db);
					for (decl, param) in param_decls.into_iter().zip(ff.parameters.iter()) {
						if let Some(p) = param.pattern {
							let dsts = collector.collect_destructuring(decl, false, p);
							decls.extend(dsts);
						}
					}
					let body = ff.body.unwrap();
					let collected_body = collector.collect_expression(body);
					let e = if decls.is_empty() {
						collected_body
					} else {
						let origin = EntityRef::new(
							collector.parent.db,
							item,
							shackle_hir::ids::EntityId::from(body),
						);
						Expression::new(
							collector.parent.db,
							&collector.parent.model,
							origin,
							Let {
								items: decls.into_iter().map(LetItem::Declaration).collect(),
								in_expression: Box::new(collected_body),
							},
						)
					};
					function.set_body(e);
					collector.parent.model[func] = function;
				}
				_ => unreachable!(),
			}
		}
	}

	/// Finish lowering
	pub(in crate::lower) fn finish(mut self) -> Model<'db> {
		// For field-only-introduced classes, derive the actual-set from the
		// class's contributions (`field_only_class_set_array_union`): an
		// `array_union(...)` of per-contribution, ITE-guarded expressions —
		// realisation-guarded parent field values for collection intros,
		// occurs-/realisation-guarded static identity singletons for
		// singular intros (plus a channelling pin on the field value), and
		// membership-gated identity images of the direct class's set for
		// superclass projections. Introductions are recorded one hop up
		// (immediate parent + direct field name), so multi-hop nesting works
		// through the intermediate class.
		//
		// The universe fallback below remains ONLY for contributions the
		// recording doesn't cover — par-existence nested collections whose
		// storage is an array of inline records (no per-parent slice array,
		// so no recorded introduction). It is sound exactly there: par
		// existence means every potential is realised, so the universe IS the
		// actual set. For var-existence shapes it would over-realise (phantom
		// members), which is what the assert below guards: a var-actual class
		// must never take the fallback.
		let introductions_map = std::mem::take(&mut self.class_set_field_introductions);
		let mut field_only_classes: Vec<PatternRef<'db>> = self
			.object_lowering
			.field_only_introduced_classes
			.iter()
			.copied()
			.collect();
		self.object_lowering.in_class_order(&mut field_only_classes);
		for child_class in field_only_classes {
			let Some(class_info) = self.class_map.get(&child_class) else {
				continue;
			};
			let class_set = class_info.class_set;
			if self.model[class_set].definition().is_some() {
				continue;
			}
			let class_enum = class_info.class_enum;
			let item = child_class.item(self.db);

			let definition_expr = self
				.field_only_class_set_array_union(item, child_class, &introductions_map)
				.unwrap_or_else(|| {
					debug_assert!(
						!self
							.object_lowering
							.var_actual_set_classes
							.contains(&child_class),
						"field-only class {:?} with a var actual set fell back to \
						 the potential universe — this over-realises (phantom \
						 members); its contributions must be derivable",
						child_class.identifier(self.db)
					);
					Expression::new(self.db, &self.model, item, class_enum)
				});

			// The actual-set declaration was already emitted at its final
			// var-ness by `predeclare_class` (from `var_actual_set_classes`),
			// so no widening happens here. The derived `array_union(...)` can
			// be a var set when a contribution's guard is var; assert that
			// the predicate predeclared a var set in that case. A par
			// definition assigned to a var declaration is fine (the
			// predicate's class-level reach may over-approximate the
			// per-occurrence emission), so only the
			// definition-var-implies-decl-var direction is checked. If this
			// fires, the predicate is too narrow — fix it rather than
			// re-introducing widening, since references froze their type at
			// build time.
			debug_assert!(
				definition_expr.ty().inst(self.db) != Some(VarType::Var)
					|| self.model[class_set].ty().inst(self.db) == Some(VarType::Var),
				"field-only class set {:?} has a var `array_union` definition but \
				 a par declaration; var_actual_set_classes is too narrow",
				child_class
			);

			self.model[class_set].set_definition(definition_expr);
		}

		// Top-level introductions (`set of new`, `var set(...) of new`,
		// `array of new`, singular roots) register their identity-set
		// contribution expression. Define `<C>` as the union over ALL of the
		// class's contributions: the registered top-level pieces PLUS any
		// nested field introductions and unregistered superclass projections.
		// The registered pieces alone are NOT the whole class — a class with
		// both a top-level root and a nested `new`-field introduction would
		// lose the nested member entirely (`new A: a3;` plus `class P (new A:
		// kid;)` solved with `A = {a3}` even in a pure-par model), because
		// nested contributions never register here. Nested pieces reuse
		// exactly the field-only engine's per-contribution derivations:
		// recorded field introductions, par-instantiated block images, and
		// projection images of the direct class's set.
		// `<C>` was already predeclared `var set of <potential>` when its
		// existence is a decision (`var set(...) of new` / `var opt new`), so
		// no widening happens here (see the field-only loop above).
		let mut top_level_contributions =
			std::mem::take(&mut self.class_set_top_level_contributions);
		// An opt-reached class whose only definite introductions are NESTED
		// fields (no registered top-level contribution) is in neither this
		// loop nor the field-only loop (a `var opt new` root makes it
		// `directly_introduced`, so it is not field-only). Add it here with an
		// empty registered list — the unregistered scan below then derives its
		// nested pieces and emits them as the subset lower bound. Classes with
		// no definite pieces at all fall through harmlessly (empty lower
		// bound).
		// Hash order is fine here: this only seeds empty entries, and the keys
		// are put into source order before anything is emitted from them.
		#[allow(
			clippy::iter_over_hash_type,
			reason = "seeds map entries only — order-independent"
		)]
		for &opt_class in self.opt_free_subset_classes.iter() {
			let _ = top_level_contributions.entry(opt_class).or_default();
		}
		let analysis = analyse_new_objects(self.db);
		let mut contribution_classes: Vec<PatternRef<'db>> =
			top_level_contributions.keys().copied().collect();
		self.object_lowering
			.in_class_order(&mut contribution_classes);
		for class_pattern in contribution_classes {
			let mut contributions = top_level_contributions
				.remove(&class_pattern)
				.expect("key came from this map");
			let Some(class_info) = self.class_map.get(&class_pattern).copied() else {
				continue;
			};
			let class_set = class_info.class_set;
			if self.model[class_set].definition().is_some() {
				continue;
			}
			contributions.sort_by_key(|(contribution_index, _)| *contribution_index);
			let item = class_pattern.item(self.db);

			// Contributions with no registered top-level expression, in
			// constructor order.
			let mut unregistered: Vec<(usize, usize, PatternRef<'db>, usize, OccurrenceId)> =
				Vec::new();
			for occurrence_contributions in self.object_lowering.contributions_in_occurrence_order()
			{
				let Some(direct) = occurrence_contributions
					.iter()
					.find(|contribution| contribution.projection_depth == 0)
				else {
					continue;
				};
				for contribution in occurrence_contributions
					.iter()
					.filter(|contribution| contribution.target_class == class_pattern)
				{
					if contributions
						.iter()
						.any(|(index, _)| *index == contribution.constructor_index)
					{
						continue;
					}
					// An opt root's contribution is never a definitional /
					// lower-bound union piece — its membership is the free
					// decision and its superclass image is pinned by an occurs
					// biconditional. Skip it (otherwise it would be counted
					// underivable and drop members, or force the opt member in).
					if self
						.opt_contribution_slots
						.contains(&(class_pattern, contribution.constructor_index))
					{
						continue;
					}
					unregistered.push((
						contribution.constructor_index,
						contribution.projection_depth,
						direct.target_class,
						direct.constructor_index,
						contribution.occurrence,
					));
				}
			}
			unregistered.sort_by_key(|(constructor_index, ..)| *constructor_index);
			let mut nested_pieces: Vec<Expression<'db>> = Vec::new();
			let mut underivable = false;
			for (constructor_index, depth, direct_class, direct_constructor_index, occurrence) in
				unregistered
			{
				let piece = if depth == 0 {
					match introductions_map.get(&class_pattern).and_then(|intros| {
						intros
							.iter()
							.find(|intro| intro.child_contribution_index == constructor_index)
					}) {
						Some(intro) => {
							self.field_introduction_contribution_expr(item, class_pattern, intro)
						}
						None if !analysis.occurrences[occurrence.0 as usize].is_var => self
							.par_contribution_block_image(item, class_pattern, constructor_index),
						None => None,
					}
				} else {
					self.superclass_projection_contribution_expr(
						item,
						class_pattern,
						constructor_index,
						direct_class,
						direct_constructor_index,
					)
				};
				match piece {
					Some(piece) => nested_pieces.push(piece),
					None => underivable = true,
				}
			}
			// A `var opt new` root's direct contribution is intentionally not
			// definitional (membership IS the decision) — mixing it with other
			// roots of the same hierarchy is fenced at HIR validation. Any
			// other underivable contribution here would silently drop members,
			// so it must not happen.
			debug_assert!(
				!underivable,
				"class {:?} has an unregistered, non-derivable contribution; \
				 its actual-set definition would drop members",
				class_pattern.identifier(self.db)
			);

			// An opt-reached class with no definite pieces (a superclass
			// reached only by an opt subclass, or an opt root with no
			// co-roots) needs no lower bound — its set stays free with the
			// potential universe as its upper bound. Skip to avoid an empty
			// `array_union`.
			if self.opt_free_subset_classes.contains(&class_pattern)
				&& nested_pieces.is_empty()
				&& contributions.is_empty()
			{
				continue;
			}
			let element_exprs: Vec<Expression<'db>> =
				contributions.into_iter().map(|(_, expr)| expr).collect();
			let definition_expr = if nested_pieces.is_empty() && element_exprs.len() == 1 {
				element_exprs.into_iter().next().unwrap()
			} else if nested_pieces.is_empty() {
				let array_lit =
					Expression::new(self.db, &self.model, item, ArrayLiteral(element_exprs));
				Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_union.into(),
						arguments: vec![array_lit],
					},
				)
			} else {
				// `array_union([<top-level sets>] ++ <nested piece arrays>)` —
				// each nested piece is already an array of (guarded) sets.
				let mut combined =
					Expression::new(self.db, &self.model, item, ArrayLiteral(element_exprs));
				for piece in nested_pieces {
					combined = Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.plus_plus.into(),
							arguments: vec![combined, piece],
						},
					);
				}
				Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_union.into(),
						arguments: vec![combined],
					},
				)
			};

			// Predeclared at its final var-ness; see the field-only loop. A var
			// contribution requires a var declaration; a par definition into a
			// var declaration is fine.
			debug_assert!(
				definition_expr.ty().inst(self.db) != Some(VarType::Var)
					|| self.model[class_set].ty().inst(self.db) == Some(VarType::Var),
				"top-level class set {:?} has a var definition but a par \
				 declaration; var_actual_set_classes is too narrow",
				class_pattern
			);

			if self.opt_free_subset_classes.contains(&class_pattern) {
				// A `var opt new` root reaches this class, so its actual set
				// stays FREE (bounded above by its declaration domain
				// `<C>_potential`). The definite contributions collected above
				// are its LOWER bound — pin `<definite union> subset <C>` — and
				// the opt occurrence's own membership is the free decision (its
				// superclass image constrained by the occurs biconditional).
				let class_set_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_set),
				);
				let lower_erased = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.enum2int.into(),
						arguments: vec![definition_expr],
					},
				);
				let class_set_erased = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.enum2int.into(),
						arguments: vec![class_set_expr],
					},
				);
				let subset_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.subset.into(),
						arguments: vec![lower_erased, class_set_erased],
					},
				);
				let _ = self.model.add_constraint(ConstraintItem::new(
					Constraint::new(true, subset_call),
					item,
				));
			} else {
				self.model[class_set].set_definition(definition_expr);
			}
		}

		let mut class_object_contributions: Vec<_> =
			self.class_object_contributions.drain().collect();
		class_object_contributions
			.sort_by_key(|(class_pattern, _)| self.object_lowering.class_rank(*class_pattern));
		for (class_pattern, mut contributions) in class_object_contributions {
			contributions.sort_by_key(|(contribution_index, _)| *contribution_index);
			let class_objects = self.class_map[&class_pattern].class_objects;
			let mut contributions = contributions.into_iter();
			let Some((_, first_decl)) = contributions.next() else {
				continue;
			};
			let mut definition = Expression::new(
				self.db,
				&self.model,
				class_pattern.item(self.db),
				first_decl,
			);
			for (_, declaration) in contributions {
				let contribution_expr = Expression::new(
					self.db,
					&self.model,
					class_pattern.item(self.db),
					declaration,
				);
				definition = Expression::new(
					self.db,
					&self.model,
					class_pattern.item(self.db),
					LookupCall {
						function: self.ids.functions.plus_plus.into(),
						arguments: vec![definition, contribution_expr],
					},
				);
			}
			// The combined `<C>_objects` array must stay INT-indexed: it was
			// predeclared `array [int] of record`, and every consumer indexes
			// it with `enum2int(<identity>)` (a global 1-based ordinal). A
			// `'++'` of contributions is int-indexed already, but a single
			// nested contribution keeps its enum-image dim — reindex it so
			// the declaration's type does not change after references to it
			// were built.
			if definition.ty().dim_ty(self.db) != Some(Ty::par_int(self.db)) {
				definition = Expression::new(
					self.db,
					&self.model,
					class_pattern.item(self.db),
					LookupCall {
						function: self.ids.functions.array1d.into(),
						arguments: vec![definition],
					},
				);
			}
			let class_objects_ty = definition.ty();
			let class_objects_domain = self.build_class_storage_array_domain(
				class_pattern,
				class_objects_ty,
				class_pattern.item(self.db),
			);
			self.model[class_objects].set_domain(class_objects_domain);
			self.model[class_objects].set_definition(definition);
		}

		// Gated forall-drop: a computed attribute's class-body forall
		// `forall(this in <C>)(this.<attr> = <rhs>)` is redundant once EVERY
		// contribution to <C> alias-defines its defined fields — the engine's
		// root contributions (realisation-guarded where slots can be
		// unrealised) and their projections from the direct objects array —
		// because the equation then holds by construction on realised
		// objects. The gate is deliberately the SAME per-class flag that
		// drives the symmetry-wave skip below
		// (`class_contributions_all_determined`): contributions that
		// fresh-mint defined fields register `false` and keep their class's
		// forall. Classes with no registered contribution at all keep the
		// forall too (it is vacuous over an empty class set).
		let deferred_definition_foralls =
			std::mem::take(&mut self.pending_class_definition_foralls);
		for (class_pattern, class_item, attribute, value) in deferred_definition_foralls {
			if self
				.class_contributions_all_determined
				.get(&class_pattern)
				.copied()
				.unwrap_or(false)
			{
				continue;
			}
			let body = ClassBodyConstraint::Definition { attribute, value };
			self.emit_class_body_constraint(class_item, &body);
		}

		// Third wave: symmetry-break unused potential objects. For every
		// class reached through a fresh-variable introduction (`var new`,
		// `var opt new`, `var set(...) of new`), pin each defaultable
		// storage field of an unused potential to its canonical default.
		// Runs last so it sees the now-defined class set and class objects
		// array.
		let mut var_reached: Vec<PatternRef<'db>> = self
			.object_lowering
			.var_reached_classes
			.iter()
			.copied()
			.collect();
		self.object_lowering.in_class_order(&mut var_reached);
		for class_pattern in var_reached {
			self.emit_unused_potential_default_constraints(class_pattern);
		}

		self.model
	}
}

/// Object lowering: turns class items and `new` declarations into the
/// potential-enum / storage-array / actual-set encoding.
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

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn reconstructed_root_field_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		current_input: Expression<'db>,
		index_expr: Expression<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
	) -> Expression<'db> {
		// If the field is already available in the input record in a form
		// that matches storage, read it directly. This covers both non-class
		// fields and class-typed fields that the singular-new storage holds
		// as identities (e.g. var-storage with `b: var B_potential` after
		// `substitute_class_with_potential_enum`). Without this early return,
		// the class-typed branch below would fabricate a fresh identity even
		// when storage holds a valid decision.
		//
		// For a class-typed field whose input representation is a par-inlined
		// record (par-mode singular-nested `new A: a = (b: (y: 1), ...)`),
		// reading the record directly is wrong — A's storage expects a
		// `<child>_potential` identity. Fall through to the class-typed
		// branch below, which mints a fresh `<C>_occ_K(p)` identity via the
		// `OnePerParent` path.
		let input_field_ty = current_input
			.ty()
			.record_fields(self.db)
			.and_then(|fields| {
				fields
					.iter()
					.find(|(field, _)| Identifier(*field) == field_ident)
					.map(|(_, ty)| *ty)
			});
		let input_matches_storage = match input_field_ty {
			Some(ty) if field_ty.class_type(self.db).is_some() => {
				// A class-typed field's storage holds `<Child>_potential`
				// identities. `class_type` peels `set of`/`array of`, so this
				// arm also covers `set of new`/`array of new` fields. The input
				// matches storage only when it already carries identities — i.e.
				// it contains no inline record at any level. A par-inlined record
				// (singular `new A: a = (..)`) or an array of such records (`set
				// of new A: cas = [..]`, whose input is `array of record(..)`)
				// must be reconstructed into identities below, not read straight
				// from the input.
				!ty.walk(self.db)
					.any(|t| matches!(t.lookup(self.db), TyData::Record(_, _)))
			}
			Some(_) => true,
			None => false,
		};
		if input_matches_storage {
			return Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(current_input),
					field: field_ident,
				},
			);
		}
		let Some(child_class) = field_ty.class_type(self.db) else {
			let field_available_in_input = current_input
				.ty()
				.record_fields(self.db)
				.map(|fields| {
					fields
						.iter()
						.any(|(field, _)| Identifier(*field) == field_ident)
				})
				.unwrap_or(false);
			if !field_available_in_input {
				let mut fresh_decl =
					Declaration::new(false, Domain::unbounded(self.db, item, field_ty));
				fresh_decl.set_name(Identifier::new(
					self.db,
					format!("{}_{}", field_ident.pretty_print(self.db), "init"),
				));
				let fresh_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(fresh_decl, item));
				let fresh_expr = Expression::new(self.db, &self.model, item, fresh_decl_idx);
				return Expression::new(
					self.db,
					&self.model,
					item,
					Let {
						items: vec![LetItem::Declaration(fresh_decl_idx)],
						in_expression: Box::new(fresh_expr),
					},
				);
			}
			return Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(current_input),
					field: field_ident,
				},
			);
		};
		let child_class =
			class_pattern_for(self.db, child_class).expect("class item for class type");
		let Some(child_occurrence) = self.maybe_nested_occurrence(root_pattern, &[field_ident])
		else {
			// A class-typed *reference* attribute (`var A: ref`, no `new`) is
			// NOT a nested `new` introduction — it selects an existing object,
			// so there is no nested occurrence to mint an identity from. On a
			// par root the field is a free reference-identity decision, exactly
			// like a var scalar attribute: mint a fresh `var <Child>_potential`
			// decision. This is the same storage shape a var-reached owner
			// reads directly (`Owner_objects: record(var A_potential: ref)`);
			// membership/target is the user's to constrain
			// (`constraint o.ref = a` or `o.ref in A`).
			let subst_ty = self.substitute_class_with_potential_enum(field_ty);
			let mut fresh_decl =
				Declaration::new(false, Domain::unbounded(self.db, item, subst_ty));
			fresh_decl.set_name(Identifier::new(
				self.db,
				format!("{}_{}", field_ident.pretty_print(self.db), "init"),
			));
			let fresh_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(fresh_decl, item));
			let fresh_expr = Expression::new(self.db, &self.model, item, fresh_decl_idx);
			return Expression::new(
				self.db,
				&self.model,
				item,
				Let {
					items: vec![LetItem::Declaration(fresh_decl_idx)],
					in_expression: Box::new(fresh_expr),
				},
			);
		};
		let child_contribution = self.occurrence_contribution(child_occurrence, child_class);
		let child_enum = self.class_map[&child_class].class_enum;
		let child_enum_member =
			EnumMemberId::new(child_enum, child_contribution.constructor_index as u32);
		match self.occurrence_local_domain_source(child_occurrence) {
			LocalDomainSource::OnePerParent => Expression::new(
				self.db,
				&self.model,
				item,
				Call {
					function: Callable::EnumConstructor(child_enum_member),
					arguments: vec![index_expr],
				},
			),
			LocalDomainSource::FlattenedChildCollection => {
				// The input record doesn't supply this `set of new` field
				// (par-mode `set of new C: cs = [(other: ...), ...]` where
				// `C` declares a `set of new` attribute the input omits).
				// There's no input collection to measure for a per-record
				// ordinal range, so emit a fresh `var set of <child>_potential`
				// decision per parent. The per-parent subset constraint
				// (`<C>_objects[p].<field> subset <slice>[p]`) and the slice
				// array are emitted separately by the pending-slices
				// machinery, so this slot only needs to be a free decision
				// of the right storage type.
				if input_field_ty.is_none() {
					let storage_field_ty = self.substitute_class_with_potential_enum(field_ty);
					let mut fresh_decl =
						Declaration::new(false, Domain::unbounded(self.db, item, storage_field_ty));
					fresh_decl.set_name(Identifier::new(
						self.db,
						format!("{}_init", field_ident.pretty_print(self.db)),
					));
					let fresh_decl_idx = self
						.model
						.add_declaration(DeclarationItem::new(fresh_decl, item));
					let fresh_expr = Expression::new(self.db, &self.model, item, fresh_decl_idx);
					return Expression::new(
						self.db,
						&self.model,
						item,
						Let {
							items: vec![LetItem::Declaration(fresh_decl_idx)],
							in_expression: Box::new(fresh_expr),
						},
					);
				}
				let one_expr = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
				let current_children = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(current_input),
						field: field_ident,
					},
				);
				let current_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![current_children],
					},
				);
				let prev_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![index_expr.clone(), one_expr.clone()],
					},
				);
				let prev_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), prev_end],
					},
				);
				let mut prev_index_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				prev_index_decl.set_name(Identifier::new(self.db, "q"));
				let prev_index_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(prev_index_decl, item));
				let prev_index_expr =
					Expression::new(self.db, &self.model, item, prev_index_decl_idx);
				let prev_input = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![inputs_expr, prev_index_expr],
					},
				);
				let prev_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(prev_input),
								field: field_ident,
							},
						)],
					},
				);
				let prefix_sum = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[Generator::Iterator {
									declarations: vec![prev_index_decl_idx],
									collection: prev_range,
									where_clause: None,
								}],
								prev_length,
							),
						)],
					},
				);
				let ordinal_start = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![one_expr.clone(), prefix_sum.clone()],
					},
				);
				if field_ty.opt(self.db) == Some(OptType::Opt) {
					// An `opt new C` field holds the single realised child
					// identity or `<>`. Its input list has length 0 or 1, so the
					// realised identity (when present) is the one at
					// `ordinal_start`. Unlike the `set of new` case there is no
					// range — reconstruct `if length > 0 then C_occ_k(ordinal_start)
					// else <> endif` (storage type `opt <C>_potential`).
					return self.opt_child_identity_or_absent(
						item,
						child_enum_member,
						ordinal_start,
						current_length,
					);
				}
				let ordinal_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![prefix_sum, current_length],
					},
				);
				let ordinal_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![ordinal_start, ordinal_end],
					},
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal_range],
					},
				)
			}
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("nested occurrence had unexpected root-only domain source")
			}
		}
	}

	/// `if <child_length> > 0 then <C>_occ_k(<ordinal_start>) else <>
	/// endif` — the parent storage value for an optional-child (`opt new C`)
	/// field. The input list has length 0 or 1; when present the single child's
	/// ordinal is `ordinal_start`. The result type is `opt <C>_potential`,
	/// matching the field's storage; the occurs-guarded read-back
	/// reconstruction already emitted handles absence.
	pub(in crate::lower) fn opt_child_identity_or_absent(
		&mut self,
		item: Item<'db>,
		child_enum_member: EnumMemberId<'db>,
		ordinal_start: Expression<'db>,
		child_length: Expression<'db>,
	) -> Expression<'db> {
		let zero = Expression::new(self.db, &self.model, item, IntegerLiteral(0));
		let present = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.gt.into(),
				arguments: vec![child_length, zero],
			},
		);
		let identity = Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(child_enum_member),
				arguments: vec![ordinal_start],
			},
		);
		let absent = Expression::new(self.db, &self.model, item, Absent);
		Expression::new(
			self.db,
			&self.model,
			item,
			IfThenElse {
				branches: vec![Branch::new(present, identity)],
				else_result: Box::new(absent),
			},
		)
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn reconstructed_nested_flattened_field_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		current_collection: Expression<'db>,
		current_input: Expression<'db>,
		input_index_expr: Expression<'db>,
		child_index_expr: Expression<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
	) -> Expression<'db> {
		let Some(field_class) = field_ty.class_type(self.db) else {
			// A non-class storage field: read it from the input record when
			// present, otherwise mint a fresh decision. A `var` attribute on a
			// par-introduced nested object (e.g. `var by` in `class B(set of
			// new A: bas; var by)` under a par `set of new C`) is dropped from
			// the input record — it's a per-object decision, not data — so each
			// reconstructed object needs its own free decision.
			if !self.record_ty_has_field(&current_input, field_ident) {
				return self.fresh_storage_field_decision(item, field_ident, field_ty);
			}
			return Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(current_input),
					field: field_ident,
				},
			);
		};
		let field_class =
			class_pattern_for(self.db, field_class).expect("class item for class type");
		if let Some(mint) =
			self.var_existence_field_mint(item, field_class, field_ident, field_ty, &current_input)
		{
			// A var-existence (`var set of new` / `var opt new`) object
			// field on a par owner one hop below the root — dropped from the
			// par input, so mint a fresh var subset of its block rather than a
			// par range read off `length(input.<field>)`.
			return mint;
		}
		let child_occurrence = self.nested_occurrence(root_pattern, &[attribute, field_ident]);
		let child_contribution = self.occurrence_contribution(child_occurrence, field_class);
		let child_enum = self.class_map[&field_class].class_enum;
		let child_enum_member =
			EnumMemberId::new(child_enum, child_contribution.constructor_index as u32);
		let one_expr = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		match self.occurrence_local_domain_source(child_occurrence) {
			LocalDomainSource::OnePerParent => {
				let previous_input_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![input_index_expr.clone(), one_expr.clone()],
					},
				);
				let previous_input_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), previous_input_end],
					},
				);
				let mut previous_input_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				previous_input_decl.set_name(Identifier::new(self.db, "q"));
				let previous_input_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(previous_input_decl, item));
				let previous_input_expr =
					Expression::new(self.db, &self.model, item, previous_input_decl_idx);
				let previous_root = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![inputs_expr, previous_input_expr],
					},
				);
				let previous_collection = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(previous_root),
						field: attribute,
					},
				);
				let previous_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![previous_collection],
					},
				);
				let previous_count = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[Generator::Iterator {
									declarations: vec![previous_input_decl_idx],
									collection: previous_input_range,
									where_clause: None,
								}],
								previous_length,
							),
						)],
					},
				);
				let zero_based_child = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![child_index_expr, one_expr.clone()],
					},
				);
				let local_ordinal = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![
							one_expr,
							Expression::new(
								self.db,
								&self.model,
								item,
								LookupCall {
									function: self.ids.builtins.plus.into(),
									arguments: vec![previous_count, zero_based_child],
								},
							),
						],
					},
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![local_ordinal],
					},
				)
			}
			LocalDomainSource::FlattenedChildCollection => {
				let current_children = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(current_input.clone()),
						field: field_ident,
					},
				);
				let current_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![current_children],
					},
				);
				let previous_input_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![input_index_expr.clone(), one_expr.clone()],
					},
				);
				let previous_input_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), previous_input_end],
					},
				);
				let mut previous_input_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				previous_input_decl.set_name(Identifier::new(self.db, "q"));
				let previous_input_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(previous_input_decl, item));
				let previous_input_expr =
					Expression::new(self.db, &self.model, item, previous_input_decl_idx);
				let previous_root = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![inputs_expr, previous_input_expr],
					},
				);
				let previous_collection = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(previous_root),
						field: attribute,
					},
				);
				let mut previous_child_decl = Declaration::new(
					false,
					Domain::unbounded(
						self.db,
						item,
						previous_collection.ty().elem_ty(self.db).unwrap(),
					),
				);
				previous_child_decl.set_name(Identifier::new(self.db, "j"));
				let previous_child_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(previous_child_decl, item));
				let previous_child_expr =
					Expression::new(self.db, &self.model, item, previous_child_decl_idx);
				let previous_root_child_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(previous_child_expr),
								field: field_ident,
							},
						)],
					},
				);
				let previous_roots_count = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[
									Generator::Iterator {
										declarations: vec![previous_input_decl_idx],
										collection: previous_input_range,
										where_clause: None,
									},
									Generator::Iterator {
										declarations: vec![previous_child_decl_idx],
										collection: previous_collection,
										where_clause: None,
									},
								],
								previous_root_child_length,
							),
						)],
					},
				);
				let previous_sibling_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![child_index_expr.clone(), one_expr.clone()],
					},
				);
				let previous_sibling_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), previous_sibling_end],
					},
				);
				let mut previous_sibling_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				previous_sibling_decl.set_name(Identifier::new(self.db, "s"));
				let previous_sibling_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(previous_sibling_decl, item));
				let previous_sibling_expr =
					Expression::new(self.db, &self.model, item, previous_sibling_decl_idx);
				let previous_sibling = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![current_collection, previous_sibling_expr],
					},
				);
				let previous_sibling_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(previous_sibling),
								field: field_ident,
							},
						)],
					},
				);
				let previous_siblings_count = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[Generator::Iterator {
									declarations: vec![previous_sibling_decl_idx],
									collection: previous_sibling_range,
									where_clause: None,
								}],
								previous_sibling_length,
							),
						)],
					},
				);
				let prefix_sum = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![previous_roots_count, previous_siblings_count],
					},
				);
				let ordinal_start = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![one_expr.clone(), prefix_sum.clone()],
					},
				);
				if field_ty.opt(self.db) == Some(OptType::Opt) {
					// As in `reconstructed_root_field_expr`, an `opt new C`
					// field on a collection-reached object holds the single
					// realised child identity or `<>` — not a range set.
					return self.opt_child_identity_or_absent(
						item,
						child_enum_member,
						ordinal_start,
						current_length,
					);
				}
				let ordinal_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![prefix_sum, current_length],
					},
				);
				let ordinal_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![ordinal_start, ordinal_end],
					},
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal_range],
					},
				)
			}
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("nested object field had unexpected root-only domain source")
			}
		}
	}

	pub(in crate::lower) fn occurrence_local_domain_size_expr(
		&mut self,
		item: Item<'db>,
		local_domain_source: LocalDomainSource,
		record_access: Option<Expression<'db>>,
		fallback_cardinality: Option<Expression<'db>>,
	) -> Expression<'db> {
		match local_domain_source {
			LocalDomainSource::OnePerParent => {
				Expression::new(self.db, &self.model, item, IntegerLiteral(1))
			}
			LocalDomainSource::FlattenedChildCollection => {
				// For `var set of <child>` storage we have to use the declared
				// cardinality bound (par) instead of `card(record_access)`
				// (var) — enum sizing must be par. For
				// `array [_] of <input record>` storage we still derive the
				// size from `length(record_access)` since the declared bound
				// was already enforced as a constraint elsewhere.
				let use_record_access = record_access
					.as_ref()
					.map(|ra| !ra.ty().is_set(self.db))
					.unwrap_or(false);
				if use_record_access {
					let record_access = record_access.unwrap();
					Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.builtins.length.into(),
							arguments: vec![record_access],
						},
					)
				} else {
					fallback_cardinality
						.expect("nested var child collection missing fallback cardinality")
				}
			}
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("nested occurrence had unexpected root-only domain source")
			}
		}
	}

	/// Wrap a defined field's collected RHS in the slot realisation guard:
	/// `if realised then RHS else <witness> endif`. The witness is ONE
	/// top-level decision per (root, field) carrying the field's *declared*
	/// per-object domain, pinned to its canonical default by
	/// `build_field_default_expr` (`mzn_safe_default` / `lb` shapes, `<>` for
	/// opt) — so the else value is in-domain by construction and the binding
	/// element-record domain stays satisfiable on unrealised slots (e.g.
	/// `var 3..4: z = x1 + x2` under `card(as) = 0`). A free let-decl inside
	/// the else branch would be rejected by MiniZinc ("free variable in
	/// non-positive context"), hence the hoisted, pinned witness.
	///
	/// Two distinct reasons return the RHS unguarded instead:
	/// - **cannot guard** — no canonical in-domain default exists (arrays,
	///   class-typed fields, cardinality-bounded set types whose card bound
	///   `lb()` = `{}` would violate): the pre-guard semantics is the only
	///   option;
	/// - **need not guard** (guard elision) — the RHS is provably total and
	///   the declared domain provably non-binding, so the guard would buy
	///   nothing and its var if-then-else can be saved.
	pub(in crate::lower) fn realisation_guarded_alias_def(
		&mut self,
		item: Item<'db>,
		decl: &StorageFieldDecl<'db>,
		field_ty: Ty<'db>,
		guard_name_prefix: &str,
		realised_expr: Expression<'db>,
		rhs: Expression<'db>,
	) -> Expression<'db> {
		// CANNOT guard: no canonical in-domain default to use as the else
		// value. Card-bound relocation makes `{}` in-domain for
		// cardinality-bounded sets, and class-typed fields witness with the
		// first potential identity, so the only shape that legitimately
		// reaches this bail is a `new`-introducing defined field, whose
		// identity feeds the contribution/actual-set machinery rather than a
		// value default. Arrays (computed-array validation), records/tuples
		// (unsupported attribute types), and non-varifiable leaves are all
		// rejected upstream on var-reached classes, and guard contexts only
		// exist on unrealisable storage, which is var-reached by
		// construction — assert so a new unguardable shape surfaces loudly
		// instead of silently reintroducing an unguarded-alias soundness
		// hole.
		if !self.field_has_canonical_unrealised_default(decl, field_ty) {
			#[cfg(debug_assertions)]
			{
				if let Item::Class(ci) = decl.owner {
					let owner_data = ci.class(self.db).data();
					debug_assert!(
						owner_data[decl.declared_type]
							.get_new_class(owner_data)
							.is_some(),
						"defined field `{}` of type {} has no canonical unrealised default — \
						 its alias stays unguarded on unrealisable storage",
						decl.ident.pretty_print(self.db),
						field_ty.pretty_print(self.db),
					);
				}
			}
			return rhs;
		}
		// NEED NOT guard (elision): total RHS + non-binding declared domain
		// means the unguarded alias is already sound on unrealised slots, so
		// skip the witness/pin/if-then-else entirely.
		if self.defined_field_elides_realisation_guard(decl) {
			return rhs;
		}
		// GUARDED ELSEWHERE (relocation): total RHS, binding declared domain —
		// the domain has been relaxed out of the element record and re-imposed
		// as a realised-set class invariant, so the alias stays unguarded
		// here.
		if self.field_relocates_declared_domain(decl) {
			return rhs;
		}
		let owner = decl.owner;
		// The declared domain is closed over pars only: validation rejects
		// sibling-dependent domains on varified classes, so no alias scope is
		// needed here. The witness domain must equal the storage record's
		// field domain — the pinned else-value has to satisfy the element
		// record on unrealised slots — so it mirrors
		// `build_class_storage_record_domain` arm for arm: card-relocated
		// sets drop their card bound, class-containing fields take the
		// substituted potential-enum domain, everything else collects the
		// declared domain.
		let witness_domain = if self.field_relocates_set_card(decl) {
			self.card_stripped_set_field_domain(owner, decl.declared_type, field_ty, item)
		} else {
			let subst_ty = self.substitute_class_with_potential_enum(field_ty);
			if subst_ty != field_ty {
				self.class_storage_field_domain(owner, decl.declared_type, field_ty, item.into())
			} else {
				let Item::Class(owner_ci) = owner else {
					unreachable!()
				};
				let owner_data = owner_ci.class(self.db).data();
				let owner_types = owner.types(self.db);
				let mut collector = ExpressionCollector::new(self, owner_data, owner, &owner_types);
				collector.collect_domain(decl.declared_type, subst_ty, false)
			}
		};
		let mut witness_decl = Declaration::new(true, witness_domain);
		witness_decl.set_name(Identifier::new(
			self.db,
			format!(
				"{}_{}_unrealised_default",
				guard_name_prefix,
				decl.ident.pretty_print(self.db)
			),
		));
		let witness_idx = self
			.model
			.add_declaration(DeclarationItem::new(witness_decl, item));
		let witness_expr = Expression::new(self.db, &self.model, item, witness_idx);
		let default_expr = self
			.build_field_default_expr(item, witness_expr.clone())
			.expect("field_has_canonical_unrealised_default checked");
		let pin = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.eq.into(),
				arguments: vec![witness_expr.clone(), default_expr],
			},
		);
		let _ = self
			.model
			.add_constraint(ConstraintItem::new(Constraint::new(true, pin), item));
		// A class-typed RHS lowers structurally as `<Child>_potential`
		// identities but keeps its `Class<...>` THIR label, while the witness
		// carries the substituted potential-enum type. Relabel the RHS to the
		// witness type so the guard if-then-else typechecks — a relabel, not a
		// coercion (the runtime value already IS the identity).
		let rhs = if rhs.ty() != witness_expr.ty()
			&& rhs
				.ty()
				.walk(self.db)
				.any(|t| t.class_type(self.db).is_some())
		{
			let mut relabeled =
				Expression::new_unchecked(witness_expr.ty(), (*rhs).clone(), rhs.origin());
			relabeled
				.annotations_mut()
				.extend(rhs.annotations().iter().cloned());
			relabeled
		} else {
			rhs
		};
		Expression::new(
			self.db,
			&self.model,
			item,
			IfThenElse {
				branches: vec![Branch::new(realised_expr, rhs)],
				else_result: Box::new(witness_expr),
			},
		)
	}

	/// Whether a defined field has a canonical in-domain default the
	/// realisation guard can use as its else value: the type shapes
	/// `build_field_default_expr` handles. Cardinality-bounded set
	/// declarations qualify exactly when their card bound is relocated out
	/// of the storage domain (`field_relocates_set_card` — always the case
	/// on unrealisable storage, the only place guards exist), making the
	/// canonical `lb` default `{}` in-domain. Class-typed fields qualify
	/// too: storage substitutes them with `<Child>_potential` enums, whose
	/// `lb` (the first potential identity) is exactly the value the
	/// unused-potential pin gives a FREE reference field — a phantom slot's
	/// dangling identity constrains nothing. Only `new`-introducing declared
	/// types keep the bail: their identities feed contributions and the
	/// actual-set derivation, not a value default.
	pub(in crate::lower) fn field_has_canonical_unrealised_default(
		&self,
		decl: &StorageFieldDecl<'db>,
		field_ty: Ty<'db>,
	) -> bool {
		{
			let Item::Class(ci) = decl.owner else {
				return false;
			};
			let owner_data = ci.class(self.db).data();
			if owner_data[decl.declared_type]
				.get_new_class(owner_data)
				.is_some()
			{
				return false;
			}
			if matches!(
				&owner_data[decl.declared_type],
				shackle_hir::Type::Set {
					cardinality: Some(_),
					..
				}
			) && !self.field_relocates_set_card(decl)
			{
				return false;
			}
		}
		fn defaultable<'db>(db: &'db dyn Db, ty: Ty<'db>) -> bool {
			if ty.opt(db) == Some(OptType::Opt) {
				return true;
			}
			if ty.class_type(db).is_some() {
				// Substituted to a `<Child>_potential` enum in storage;
				// `lb` = first potential identity.
				return true;
			}
			match ty.lookup(db) {
				TyData::Integer(_, _)
				| TyData::Float(_, _)
				| TyData::Boolean(_, _)
				| TyData::Enum(_, _, _)
				| TyData::Set(_, _, _) => true,
				TyData::Record(_, fs) => fs.iter().all(|(_, f)| defaultable(db, *f)),
				TyData::Tuple(_, fs) => fs.iter().all(|f| defaultable(db, *f)),
				_ => false,
			}
		}
		defaultable(self.db, field_ty)
	}

	/// Guard elision: a defined field NEED NOT be realisation-guarded when
	/// its RHS is provably total and its declared domain is provably
	/// non-binding. On an unrealised slot the alias value is then just
	/// RHS-at-the-pinned-frees — defined (total RHS), free to take that value
	/// (non-binding domain), and still functionally determined — which is
	/// exactly the pre-guard semantics minus the two failure channels.
	pub(in crate::lower) fn defined_field_elides_realisation_guard(
		&self,
		decl: &StorageFieldDecl<'db>,
	) -> bool {
		self.field_declared_domain_nonbinding(decl) && self.defined_field_rhs_provably_total(decl)
	}

	/// Whether a defined field on a root with unrealisable slots keeps its
	/// realisation guard: it must HAVE a canonical in-domain default to
	/// guard with ("cannot guard" otherwise) and NOT satisfy the elision
	/// rule ("need not guard") or the relocation rule ("guarded elsewhere").
	/// Drives the per-slot `realised` alias emission in the engine — no
	/// guarded field, no alias.
	pub(in crate::lower) fn defined_field_keeps_realisation_guard(
		&self,
		decl: &StorageFieldDecl<'db>,
		field_ty: Ty<'db>,
	) -> bool {
		self.field_has_canonical_unrealised_default(decl, field_ty)
			&& !self.defined_field_elides_realisation_guard(decl)
			&& !self.field_relocates_declared_domain(decl)
	}

	/// Domain relocation: a defined field whose RHS is provably total but
	/// whose declared domain is binding (e.g. `var 3..4: z = x1 + x2`) trades
	/// the per-slot value guard for a cheaper encoding — unguarded alias,
	/// element-record domain relaxed to unbounded, and the declared domain
	/// re-imposed on realised objects only, as the class invariant
	/// `forall(this in <C>)(this.f in <dom>)`. Sound because the total RHS
	/// defines the field on EVERY slot (its bounds propagate from the
	/// definition, so the relaxed decl never introduces a free unbounded
	/// decision), while the invariant restores exactly the class-body scope
	/// semantics. All three emission sites (the shared element-record domain,
	/// the engine's guard routing, and the invariant) key on this ONE
	/// predicate, and the predicate keys on the field's OWNER class, so the
	/// owner's and every subclass's `_objects` domains cannot diverge.
	pub(in crate::lower) fn field_relocates_declared_domain(
		&self,
		decl: &StorageFieldDecl<'db>,
	) -> bool {
		if decl.definition.is_none() || !self.field_declared_domain_relocatable(decl) {
			return false;
		}
		let Item::Class(ci) = decl.owner else {
			return false;
		};
		let owner_class_pattern = PatternRef::new(self.db, decl.owner, ci.class(self.db).pattern);
		if !self
			.object_lowering
			.domain_relocation_classes
			.contains(&owner_class_pattern)
		{
			return false;
		}
		self.defined_field_rhs_provably_total(decl)
	}

	/// Set-cardinality relocation: a cardinality-bounded set field (free OR
	/// defined, but not a `set of new` introduction — those never carry
	/// their card in the record domain and are covered by the
	/// nested-cardinality invariants) whose owner's storage can hold
	/// unrealised slots. The card bound is dropped from every storage record
	/// domain (`build_class_storage_record_domain` / the guard witness) and
	/// re-imposed on realised objects only by
	/// `emit_nested_set_cardinality_class_invariants`; the canonical
	/// unrealised-slot value is then `{}` — consistent for the
	/// unused-potential pin of a free field AND available as the guard
	/// witness default of a defined field. Keying on the field's OWNER keeps
	/// the owner's and every subclass's `_objects` domains in agreement,
	/// mirroring `field_relocates_declared_domain`.
	pub(in crate::lower) fn field_relocates_set_card(&self, decl: &StorageFieldDecl<'db>) -> bool {
		let Item::Class(ci) = decl.owner else {
			return false;
		};
		let owner_data = ci.class(self.db).data();
		if !matches!(
			&owner_data[decl.declared_type],
			shackle_hir::Type::Set {
				cardinality: Some(_),
				..
			}
		) || owner_data[decl.declared_type]
			.get_new_class(owner_data)
			.is_some()
		{
			return false;
		}
		let owner_class_pattern = PatternRef::new(self.db, decl.owner, ci.class(self.db).pattern);
		self.object_lowering
			.unrealisable_storage_classes
			.contains(&owner_class_pattern)
	}

	/// The storage domain of a card-relocated set field: the declared
	/// element bound is kept, the cardinality bound is dropped. Shared by
	/// the element-record domain and the realisation-guard witness so the
	/// pinned `{}` unrealised-slot value satisfies the storage domain by
	/// construction.
	pub(in crate::lower) fn card_stripped_set_field_domain(
		&mut self,
		class_item: Item<'db>,
		declared_type: shackle_hir::TypeId<'db>,
		field_ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		let subst = self.substitute_class_with_potential_enum(field_ty);
		let Item::Class(local) = class_item else {
			return Domain::unbounded(db, origin, subst);
		};
		let class_data = local.class(db).data();
		let shackle_hir::Type::Set { element, .. } = &class_data[declared_type] else {
			return Domain::unbounded(db, origin, subst);
		};
		let element = *element;
		let inst = subst.inst(db).unwrap_or(VarType::Var);
		let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
		let elem_ty = subst.elem_ty(db).unwrap_or(subst);
		let elem_domain = if subst != field_ty {
			// Class-element reference set: the element bound lives in the
			// substituted `<Child>_potential` enum type itself.
			Domain::unbounded(db, origin, elem_ty)
		} else {
			let class_types = class_item.types(db);
			let mut inner = ExpressionCollector::new(self, class_data, class_item, &class_types);
			inner.collect_domain(element, elem_ty, false)
		};
		Domain::set_with_card(db, origin, inst, opt, None, elem_domain)
	}

	/// Relocatable declared-domain shape: the declared type is directly a
	/// scalar `Bounded` domain whose domain expression is not a bare
	/// identifier — an identifier domain is an enum, class, or type-alias
	/// name (or a par set alias), where "relaxing" is meaningless or needs
	/// resolution we don't attempt; those keep the value guard. Set/array
	/// shapes and cardinality-bounded sets are also excluded: their
	/// re-imposition is not a plain `in` and their guard story is the
	/// existing bail.
	pub(in crate::lower) fn field_declared_domain_relocatable(
		&self,
		decl: &StorageFieldDecl<'db>,
	) -> bool {
		let Item::Class(ci) = decl.owner else {
			return false;
		};
		let owner_data = ci.class(self.db).data();
		let shackle_hir::Type::Bounded { domain, .. } = &owner_data[decl.declared_type] else {
			return false;
		};
		!matches!(&owner_data[*domain], shackle_hir::Expression::Identifier(_))
	}

	/// Elision condition (2): the declared domain is provably non-binding —
	/// purely syntactically, no explicit domain anywhere in the declared
	/// type (`int: n = card(children)`): no `Bounded` node (explicit domain,
	/// enum, or type alias), no set cardinality bound. An interval-arithmetic
	/// proof that a declared domain contains the RHS image is a possible
	/// later refinement.
	pub(in crate::lower) fn field_declared_domain_nonbinding(
		&self,
		decl: &StorageFieldDecl<'db>,
	) -> bool {
		let Item::Class(ci) = decl.owner else {
			return false;
		};
		let owner_data = ci.class(self.db).data();
		shackle_hir::Type::walk(decl.declared_type, owner_data).all(|t| {
			matches!(
				&owner_data[t],
				shackle_hir::Type::Primitive { .. }
					| shackle_hir::Type::Set {
						cardinality: None,
						..
					} | shackle_hir::Type::Array { .. }
					| shackle_hir::Type::Tuple { .. }
					| shackle_hir::Type::Record { .. }
					| shackle_hir::Type::Any
			)
		})
	}

	/// Elision condition (1): the RHS is provably total, by a conservative
	/// syntactic whitelist. Anything not whitelisted —
	/// `div`/`mod`/`'[]'`/`min`/`max`/`deopt`/`assert`/`pow`, lets,
	/// unresolvable calls — means NOT proven, keep the guard. This
	/// deliberately ignores bool-context-benign partiality
	/// (`ok = (x div y > 3)`).
	pub(in crate::lower) fn defined_field_rhs_provably_total(
		&self,
		decl: &StorageFieldDecl<'db>,
	) -> bool {
		let Some(definition) = decl.definition else {
			return false;
		};
		let mut in_progress = Vec::new();
		self.hir_expr_provably_total(decl.owner, definition, &mut in_progress)
	}

	/// The totality whitelist walker. Read-only over item data + TypeResult
	/// (never touches `self.resolutions` — it runs on HIR, before
	/// collection). Total node shapes: literals, identifiers (reading any
	/// declaration — including a sibling alias — is total; a *partial*
	/// sibling keeps its own guard, so its alias value is always defined),
	/// tuple/record/set/array literals, tuple/record access, if-then-else
	/// with an else, and comprehensions (total on empty). Calls recurse
	/// through the resolved function: a body means a user (or stdlib-defined)
	/// function — analyse the body rather than trusting the name, so a user
	/// function shadowing `card` cannot smuggle partiality in; bodyless means
	/// a true builtin, accepted only from the total-ops whitelist.
	/// `in_progress` is the call chain — recursive functions bail (their
	/// termination is not provable here).
	pub(in crate::lower) fn hir_expr_provably_total(
		&self,
		item: Item<'db>,
		root: shackle_hir::ExpressionId<'db>,
		in_progress: &mut Vec<Item<'db>>,
	) -> bool {
		let db = self.db;
		let data = item.data(db);
		let types = item.types(db);
		let mut todo = vec![root];
		while let Some(e) = todo.pop() {
			match &data[e] {
				shackle_hir::Expression::IntegerLiteral(_)
				| shackle_hir::Expression::FloatLiteral(_)
				| shackle_hir::Expression::BooleanLiteral(_)
				| shackle_hir::Expression::StringLiteral(_)
				| shackle_hir::Expression::Identifier(_)
				| shackle_hir::Expression::Absent
				| shackle_hir::Expression::Infinity => (),
				shackle_hir::Expression::SetLiteral(sl) => todo.extend(sl.members.iter().copied()),
				shackle_hir::Expression::TupleLiteral(tl) => todo.extend(tl.fields.iter().copied()),
				shackle_hir::Expression::RecordLiteral(rl) => {
					todo.extend(rl.fields.iter().map(|(_, f)| *f))
				}
				shackle_hir::Expression::ArrayLiteral(al) => {
					todo.extend(al.members.iter().copied())
				}
				shackle_hir::Expression::TupleAccess(ta) => todo.push(ta.tuple),
				shackle_hir::Expression::RecordAccess(ra) => todo.push(ra.record),
				shackle_hir::Expression::IfThenElse(ite) => {
					// An else-less if-then-else is only total in bool contexts;
					// don't bother distinguishing, just require the else.
					let Some(else_result) = ite.else_result else {
						return false;
					};
					todo.push(else_result);
					todo.extend(ite.branches.iter().flat_map(|b| [b.condition, b.result]));
				}
				shackle_hir::Expression::ArrayComprehension(c) => {
					for g in c.generators.iter() {
						match g {
							shackle_hir::Generator::Iterator {
								collection,
								where_clause,
								..
							} => {
								todo.push(*collection);
								todo.extend(*where_clause);
							}
							shackle_hir::Generator::Assignment {
								value,
								where_clause,
								..
							} => {
								todo.push(*value);
								todo.extend(*where_clause);
							}
						}
					}
					todo.extend(c.indices);
					todo.push(c.template);
				}
				shackle_hir::Expression::SetComprehension(c) => {
					for g in c.generators.iter() {
						match g {
							shackle_hir::Generator::Iterator {
								collection,
								where_clause,
								..
							} => {
								todo.push(*collection);
								todo.extend(*where_clause);
							}
							shackle_hir::Generator::Assignment {
								value,
								where_clause,
								..
							} => {
								todo.push(*value);
								todo.extend(*where_clause);
							}
						}
					}
					todo.push(c.template);
				}
				shackle_hir::Expression::Call(c) => {
					let shackle_hir::Expression::Identifier(ident) = &data[c.function] else {
						return false;
					};
					let Some(res) = types.name_resolution(c.function) else {
						return false;
					};
					let Item::Function(f) = res.item(db) else {
						// Enum constructors and identifier-typed callees are
						// not analysed — not proven.
						return false;
					};
					let function = f.function(db);
					if let Some(body) = function.body {
						if in_progress.contains(&res.item(db)) {
							return false;
						}
						// A parameter or return domain is a definedness
						// side-condition of its own (`function int: f(1..3: x)`
						// is undefined at `x = 0`), so a body-carrying
						// function must also be domain-free to count.
						let function_data = function.data();
						let domain_free = function
							.parameters
							.iter()
							.map(|p| p.declared_type)
							.chain([function.return_type])
							.all(|t| {
								shackle_hir::Type::walk(t, function_data).all(|t| {
									!matches!(&function_data[t], shackle_hir::Type::Bounded { .. })
								})
							});
						if !domain_free {
							return false;
						}
						in_progress.push(res.item(db));
						let body_total =
							self.hir_expr_provably_total(res.item(db), body, in_progress);
						let _ = in_progress.pop();
						if !body_total {
							return false;
						}
					} else if !self.total_builtin_call(*ident) {
						return false;
					}
					todo.extend(c.arguments.iter().copied());
				}
				// Not whitelisted: array access, lets (domained declarations),
				// case, lambdas, indexed array literals, slices — not proven.
				_ => return false,
			}
		}
		true
	}

	/// Field-wise projection/reconstruction WITHOUT the engine's alias chain:
	/// each target field is read from the source element when its
	/// representation matches storage, and fresh-minted otherwise. Root
	/// contributions run `engine_reconstructed_root_contribution_expr`
	/// instead; this remains for the top-level inheritance projections
	/// (singular and collection roots alike), which read every target field
	/// from the already-reconstructed direct-class objects array — so nothing
	/// is fresh-minted in practice and the projection inherits the direct
	/// contribution's determined flag.
	pub(in crate::lower) fn reconstructed_root_contribution_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		root_fields: &[(Identifier<'db>, Ty<'db>)],
		needs_reconstruction: bool,
	) -> Expression<'db> {
		if !needs_reconstruction {
			return inputs_expr;
		}
		let index_set_expr = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![inputs_expr.clone()],
			},
		);
		let mut index_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		index_decl.set_name(Identifier::new(self.db, "p"));
		let index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(index_decl, item));
		let index_expr = Expression::new(self.db, &self.model, item, index_decl_idx);
		let current_input = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![inputs_expr.clone(), index_expr.clone()],
			},
		);
		let mut record_fields = Vec::new();
		for (field_ident, field_ty) in root_fields.iter().copied() {
			let field_expr = self.reconstructed_root_field_expr(
				item,
				root_pattern,
				inputs_expr.clone(),
				current_input.clone(),
				index_expr.clone(),
				field_ident,
				field_ty,
			);
			record_fields.push((field_ident, field_expr));
		}
		Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![index_decl_idx],
					collection: index_set_expr,
					where_clause: None,
				}],
				Expression::new(self.db, &self.model, item, RecordLiteral(record_fields)),
			),
		)
	}

	/// The single root-reconstruction engine: build a class's per-root
	/// contribution array from its inputs (a par `_inputs` array of input
	/// records, or a var root's free `_storage` array), iterating INDEXED —
	/// `p in index_set(inputs)`, `input = inputs[p]` — with one generator
	/// alias per storage field, each defined by a per-field rule selected
	/// from the *input element type* (never from class-global predicates):
	///
	/// - **defined** (`definition.is_some()`): alias = collected RHS — the
	///   computed attribute is *defined*, not a free decision pinned by the
	///   class-body forall;
	/// - **identity** (class-typed, input holds inline records or lacks the
	///   field): mint the `<Child>_potential` identity via the occurrence's
	///   regime (`reconstructed_root_field_expr` — `<C>_occ_k(p)` for
	///   one-per-parent fields, prefix-sum ordinal ranges for flattened
	///   `set of new` collections; both need `index_expr`/`inputs_expr`,
	///   which the indexed iteration provides);
	/// - **read** (input representation already matches storage): `input.f`;
	/// - **free** (storage-only, non-computed): fresh decision with the
	///   *declared* per-object domain (which may reference earlier aliases,
	///   `var 1..z: s`).
	///
	/// When `realisation_guard` is set (roots whose slots can be UNREALISED:
	/// `var set(..) of new` and `var opt new`), defined fields are
	/// realisation-guarded: one `realised = <C>_occ_k(p) in <C>` alias per
	/// slot, and each defined field becomes `f = if realised then RHS else
	/// <in-domain default> endif`. Class-body semantics bind realised objects
	/// only — an unguarded alias would impose the RHS's definedness and the
	/// field's declared domain on unrealised slots evaluated at their pinned
	/// sibling defaults (e.g. `var 3..4: z = x1 + x2` would make
	/// `card(as) = 0` unsatisfiable). Par roots and singular `var new` have
	/// no unrealisable slots and elide the guard unconditionally (pass
	/// `None`). Per field, the guard is also elided when neither channel can
	/// fire — provably total RHS AND provably non-binding declared domain
	/// (`defined_field_elides_realisation_guard`); the `realised` alias is
	/// only emitted if some defined field actually keeps its guard.
	pub(in crate::lower) fn engine_reconstructed_root_contribution_expr(
		&mut self,
		item: Item<'db>,
		class_pattern: PatternRef<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		root_fields: &[(Identifier<'db>, Ty<'db>)],
		realisation_guard: Option<RootRealisationGuard>,
	) -> Expression<'db> {
		let input_ty = inputs_expr
			.ty()
			.elem_ty(self.db)
			.expect("root inputs should be an array");
		let index_set_expr = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![inputs_expr.clone()],
			},
		);
		let mut index_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		index_decl.set_name(Identifier::new(self.db, "p"));
		let index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(index_decl, item));
		let index_expr = Expression::new(self.db, &self.model, item, index_decl_idx);
		let mut input_decl = Declaration::new(false, Domain::unbounded(self.db, item, input_ty));
		input_decl.set_name(Identifier::new(self.db, "input"));
		input_decl.set_definition(Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![inputs_expr.clone(), index_expr.clone()],
			},
		));
		let input_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_decl, item));
		let current_input = Expression::new(self.db, &self.model, item, input_decl_idx);

		let generators: Vec<Generator<'db>> = vec![
			Generator::Iterator {
				declarations: vec![index_decl_idx],
				collection: index_set_expr,
				where_clause: None,
			},
			Generator::Assignment {
				assignment: input_decl_idx,
				where_clause: None,
			},
		];

		let guard = realisation_guard.map(|guard| EngineRealisationGuard {
			name_prefix: guard.name_prefix,
			test: EngineRealisationTest::ConstructorOrdinal {
				constructor_index: guard.constructor_index,
				ordinal: index_expr.clone(),
			},
		});

		self.engine_reconstructed_contribution_expr(
			item,
			class_pattern,
			generators,
			current_input,
			root_fields,
			EngineIdentityRule::Root {
				root_pattern,
				inputs_expr,
				index_expr,
			},
			guard,
		)
	}

	/// The engine core, shared by every reconstructing contribution site: one
	/// generator alias per storage field over a caller-supplied iteration
	/// context (`generators` establishing one slot per iteration and
	/// `current_input` naming that slot's input record), each field defined by
	/// a per-field rule selected from the input element type — defined /
	/// identity / read / free (see
	/// `engine_reconstructed_root_contribution_expr`). Class-typed identity
	/// minting is the only context-dependent rule and is dispatched through
	/// `identity_rule`; the realisation guard's slot test is dispatched
	/// through `realisation_guard.test`.
	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn engine_reconstructed_contribution_expr(
		&mut self,
		item: Item<'db>,
		class_pattern: PatternRef<'db>,
		mut generators: Vec<Generator<'db>>,
		current_input: Expression<'db>,
		root_fields: &[(Identifier<'db>, Ty<'db>)],
		identity_rule: EngineIdentityRule<'db>,
		realisation_guard: Option<EngineRealisationGuard<'db>>,
	) -> Expression<'db> {
		// A class body's attribute declarations behave like a `let`: a field may
		// reference siblings declared before it (`int: z = y + 4;`) and a var
		// field's domain may depend on an earlier computed field (`var 1..z: s`).
		// So emit one generator assignment per storage field, in declaration
		// order, and build the record from those aliases. Sibling references
		// inside a computed RHS / var domain resolve to the alias decls through
		// `self.resolutions` — the same mechanism the class-body forall uses.
		// Because identity-minted class-typed fields are aliases too, a computed
		// RHS may reference an identity-minted sibling (`n = card(children)` on
		// a par object-field root).
		let field_decls = self.class_storage_field_decls(class_pattern.item(self.db));

		// One realisation test per slot, shared by every guarded field:
		// `realised = <slot> in <C>` — a single reified set membership. Only
		// emitted when this contribution's slots can be unrealised AND some
		// defined field actually KEEPS its guard — a field whose guard is
		// elided (total RHS, non-binding domain) or bailed (no canonical
		// default) must not leave a dead `realised` alias behind.
		let has_guarded_defined_field = field_decls.iter().any(|d| {
			d.definition.is_some()
				&& root_fields.iter().any(|(ident, field_ty)| {
					*ident == d.ident && self.defined_field_keeps_realisation_guard(d, *field_ty)
				})
		});
		let realised_expr = match &realisation_guard {
			Some(guard) if has_guarded_defined_field => {
				let class_info = self.class_map[&class_pattern];
				let slot_expr = match &guard.test {
					EngineRealisationTest::ConstructorOrdinal {
						constructor_index,
						ordinal,
					} => {
						let enum_member =
							EnumMemberId::new(class_info.class_enum, *constructor_index as u32);
						Expression::new(
							self.db,
							&self.model,
							item,
							Call {
								function: Callable::EnumConstructor(enum_member),
								arguments: vec![ordinal.clone()],
							},
						)
					}
					EngineRealisationTest::Identity(identity) => identity.clone(),
				};
				let class_set_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_info.class_set),
				);
				let in_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.in_.into(),
						arguments: vec![slot_expr, class_set_expr],
					},
				);
				let mut realised_decl = Declaration::from_expression(self.db, false, in_call);
				realised_decl.set_name(Identifier::new(self.db, "realised"));
				let realised_idx = self
					.model
					.add_declaration(DeclarationItem::new(realised_decl, item));
				generators.push(Generator::Assignment {
					assignment: realised_idx,
					where_clause: None,
				});
				Some(Expression::new(self.db, &self.model, item, realised_idx))
			}
			_ => None,
		};

		// The iteration generators bind internal names (`p` for the position
		// index, `input` for the current slot, `realised` for the membership
		// test). A user attribute named the same (a field named `input`, or
		// `p`, ...) becomes a same-named generator alias below, and MiniZinc
		// rejects the duplicate binding. Collect the generator names so a
		// colliding field alias can take a distinct *cosmetic* name — the
		// record field label stays the attribute name and sibling references
		// resolve through `self.resolutions` (by pattern, not name), so
		// nothing downstream is affected. Non-colliding fields keep their own
		// name, so existing output is unchanged.
		let mut reserved_generator_names: FxHashSet<Identifier<'db>> = generators
			.iter()
			.flat_map(|g| match g {
				Generator::Iterator { declarations, .. } => declarations.clone(),
				Generator::Assignment { assignment, .. } => vec![*assignment],
			})
			.filter_map(|d| self.model[d].name())
			.collect();

		// The alias declaration for each field, keyed by identifier. The record
		// literal is assembled from these in storage order afterwards.
		let mut alias_by_ident: Vec<(Identifier<'db>, DeclarationId<'db>)> = Vec::new();
		let mut prev_resolutions: Vec<(PatternRef<'db>, Option<LoweredIdentifier<'db>>)> =
			Vec::new();

		// Process fields in *declaration* order — which `field_decls` preserves
		// but the storage record type (`root_fields`) does not — so a computed
		// RHS or a var field's domain only ever references siblings whose
		// aliases already exist.
		for decl in field_decls.iter().copied() {
			let field_ident = decl.ident;
			let Some(field_ty) = root_fields
				.iter()
				.find(|(ident, _)| *ident == field_ident)
				.map(|(_, ty)| *ty)
			else {
				// A class declaration that isn't a stored field (shouldn't
				// happen for attributes, but stay defensive).
				continue;
			};
			let field_available_in_input = current_input
				.ty()
				.record_fields(self.db)
				.map(|fields| {
					fields
						.iter()
						.any(|(field, _)| Identifier(*field) == field_ident)
				})
				.unwrap_or(false);

			let alias_def = if let Some(definition) = decl.definition {
				// Defined rule. Collect the RHS against its owning class item (a
				// superclass item for inherited fields), with the already-built
				// sibling aliases in scope. This *defines* the field — the only
				// valid form for par storage. (A defined field is never in the
				// input: it is excluded from both `input_record_ty` and the free
				// `_storage` element type.)
				let owner = decl.owner;
				let Item::Class(owner_ci) = owner else {
					unreachable!()
				};
				let owner_data = owner_ci.class(self.db).data();
				let owner_types = owner.types(self.db);
				let rhs = {
					let mut collector =
						ExpressionCollector::new(self, owner_data, owner, &owner_types);
					collector.collect_expression(definition)
				};
				match (&realisation_guard, &realised_expr) {
					(Some(guard), Some(realised)) => {
						let name_prefix = guard.name_prefix.clone();
						self.realisation_guarded_alias_def(
							item,
							&decl,
							field_ty,
							&name_prefix,
							realised.clone(),
							rhs,
						)
					}
					_ => rhs,
				}
			} else if field_ty.class_type(self.db).is_some() {
				// Identity-or-read rule for class-typed fields (including `set
				// of`/`array of` class fields): read the input through when it
				// already holds `<Child>_potential` identities (var `_storage`),
				// mint fresh identities via the iteration context's regime when
				// the input carries inline records or lacks the field (par
				// roots / par nested collections).
				match &identity_rule {
					EngineIdentityRule::Root {
						root_pattern,
						inputs_expr,
						index_expr,
					} => {
						let (root_pattern, inputs_expr, index_expr) =
							(*root_pattern, inputs_expr.clone(), index_expr.clone());
						self.reconstructed_root_field_expr(
							item,
							root_pattern,
							inputs_expr,
							current_input.clone(),
							index_expr,
							field_ident,
							field_ty,
						)
					}
					EngineIdentityRule::NestedFlattened {
						root_pattern,
						inputs_expr,
						attribute,
						current_collection,
						input_index_expr,
						child_index_expr,
					} => {
						let (root_pattern, attribute) = (*root_pattern, *attribute);
						let (inputs_expr, current_collection, input_index_expr, child_index_expr) = (
							inputs_expr.clone(),
							current_collection.clone(),
							input_index_expr.clone(),
							child_index_expr.clone(),
						);
						self.reconstructed_nested_flattened_field_expr(
							item,
							root_pattern,
							inputs_expr,
							attribute,
							current_collection,
							current_input.clone(),
							input_index_expr,
							child_index_expr,
							field_ident,
							field_ty,
						)
					}
					EngineIdentityRule::NestedSingular {
						root_pattern,
						inputs_expr,
						attribute,
						input_index_expr,
					} => {
						let (root_pattern, attribute) = (*root_pattern, *attribute);
						let (inputs_expr, input_index_expr) =
							(inputs_expr.clone(), input_index_expr.clone());
						self.reconstructed_nested_singular_field_expr(
							item,
							root_pattern,
							inputs_expr,
							attribute,
							current_input.clone(),
							input_index_expr,
							field_ident,
							field_ty,
						)
					}
					EngineIdentityRule::NestedDeep {
						root_pattern,
						full_path,
						flat_inputs_expr,
						flat_index_expr,
					} => {
						let root_pattern = *root_pattern;
						let (full_path, flat_inputs_expr, flat_index_expr) = (
							full_path.clone(),
							flat_inputs_expr.clone(),
							flat_index_expr.clone(),
						);
						self.reconstructed_deep_nested_field_expr(
							item,
							root_pattern,
							&full_path,
							flat_inputs_expr,
							flat_index_expr,
							current_input.clone(),
							field_ident,
							field_ty,
						)
					}
					EngineIdentityRule::ReadOrMint => {
						if field_available_in_input {
							Expression::new(
								self.db,
								&self.model,
								item,
								RecordAccess {
									record: Box::new(current_input.clone()),
									field: field_ident,
								},
							)
						} else {
							let mint_ty = self.substitute_class_with_potential_enum(field_ty);
							self.fresh_storage_field_decision(item, field_ident, mint_ty)
						}
					}
				}
			} else if field_available_in_input {
				// Read rule: a non-class field supplied by the input record (a
				// par input attribute, or a var root's free `_storage` decision).
				Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(current_input.clone()),
						field: field_ident,
					},
				)
			} else {
				// Storage-only, non-computed field (e.g. an explicitly `var`
				// attribute the input doesn't supply). Mint a fresh decision with
				// its *declared* per-object domain — which may reference earlier
				// computed aliases (`var 1..z: s`) — rather than an unbounded one.
				let owner = decl.owner;
				let Item::Class(owner_ci) = owner else {
					unreachable!()
				};
				let owner_data = owner_ci.class(self.db).data();
				let owner_types = owner.types(self.db);
				let domain = {
					let mut collector =
						ExpressionCollector::new(self, owner_data, owner, &owner_types);
					collector.collect_domain(decl.declared_type, field_ty, false)
				};
				let mut fresh_decl = Declaration::new(false, domain);
				fresh_decl.set_name(Identifier::new(
					self.db,
					format!("{}_init", field_ident.pretty_print(self.db)),
				));
				let fresh_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(fresh_decl, owner));
				let fresh_expr = Expression::new(self.db, &self.model, owner, fresh_decl_idx);
				Expression::new(
					self.db,
					&self.model,
					owner,
					Let {
						items: vec![LetItem::Declaration(fresh_decl_idx)],
						in_expression: Box::new(fresh_expr),
					},
				)
			};

			// Materialise the field as a named generator assignment so later
			// fields can reference it, and resolve its pattern to the alias.
			//
			// A computed attribute's RHS is typed against the *class body* HIR,
			// which is never var-forced — so `c = b + 1` collects as `par int`
			// even when the class is var-reached and the field's storage type is
			// `var int`. Left as-is, the reconstructed record column would be par
			// while the field is used as var elsewhere (`C_objects[..].c`),
			// which MiniZinc rejects. When the storage field type (`field_ty`) is
			// var but the collected value is par, declare the alias with the
			// varified storage type (the value stays a valid par→var coercion).
			// The declared alias type must have `Class<X>` elements substituted
			// with `<X>_potential`: the raw storage field type of a class-typed
			// field is e.g. `var set of Class<B>`, which would render as the
			// derived class set and trip the class-identifier coercion arm.
			let field_is_var = field_ty.inst(self.db) == Some(VarType::Var);
			let value_is_var = alias_def.ty().inst(self.db) == Some(VarType::Var);
			let mut alias_decl = if field_is_var && !value_is_var {
				let alias_ty = self.substitute_class_with_potential_enum(field_ty);
				let mut decl = Declaration::new(false, Domain::unbounded(self.db, item, alias_ty));
				decl.set_definition(alias_def);
				decl
			} else {
				Declaration::from_expression(self.db, false, alias_def)
			};
			// Disambiguate the alias's cosmetic name if it collides with an
			// iteration generator name (`p`/`input`/`realised`). The record field
			// label below stays `field_ident`; only the generator binding is
			// renamed, so a field named e.g. `input` no longer duplicates the
			// slot generator.
			let alias_name = if reserved_generator_names.contains(&field_ident) {
				let base = field_ident.pretty_print(self.db);
				let mut prefix = String::from("_");
				loop {
					let candidate = Identifier::new(self.db, format!("{prefix}{base}"));
					if !reserved_generator_names.contains(&candidate) {
						break candidate;
					}
					prefix.push('_');
				}
			} else {
				field_ident
			};
			// Reserve the chosen name too, so a later field can't pick it (a
			// model with both `input` and `_input` fields would otherwise
			// re-collide).
			let _ = reserved_generator_names.insert(alias_name);
			alias_decl.set_name(alias_name);
			let alias_idx = self
				.model
				.add_declaration(DeclarationItem::new(alias_decl, item));
			let old = self.resolutions.insert(
				decl.pattern,
				LoweredIdentifier::ResolvedIdentifier(alias_idx.into()),
			);
			prev_resolutions.push((decl.pattern, old));
			generators.push(Generator::Assignment {
				assignment: alias_idx,
				where_clause: None,
			});
			alias_by_ident.push((field_ident, alias_idx));
		}

		for (pattern, old) in prev_resolutions {
			match old {
				Some(old) => {
					let _ = self.resolutions.insert(pattern, old);
				}
				None => {
					let _ = self.resolutions.remove(&pattern);
				}
			}
		}

		// Assemble the record literal in storage order from the aliases.
		let record_fields: Vec<(Identifier<'db>, Expression<'db>)> = root_fields
			.iter()
			.map(|(field_ident, _)| {
				let alias_idx = alias_by_ident
					.iter()
					.find(|(ident, _)| ident == field_ident)
					.map(|(_, idx)| *idx)
					.expect("every storage field has a declaration alias");
				(
					*field_ident,
					Expression::new(self.db, &self.model, item, alias_idx),
				)
			})
			.collect();

		Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				generators,
				Expression::new(self.db, &self.model, item, RecordLiteral(record_fields)),
			),
		)
	}

	pub(in crate::lower) fn projected_contribution_expr_from_declaration(
		&mut self,
		item: Item<'db>,
		source_contribution_decl: DeclarationId<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Expression<'db> {
		let source_decl_expr =
			Expression::new(self.db, &self.model, item, source_contribution_decl);
		let source_elem_ty = source_decl_expr
			.ty()
			.elem_ty(self.db)
			.expect("source contribution declaration should be an array");
		let mut source_elem_decl =
			Declaration::new(false, Domain::unbounded(self.db, item, source_elem_ty));
		source_elem_decl.set_name(Identifier::new(self.db, "proj"));
		let source_elem_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(source_elem_decl, item));
		let source_elem_expr = Expression::new(self.db, &self.model, item, source_elem_decl_idx);
		Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![source_elem_decl_idx],
					collection: source_decl_expr,
					where_clause: None,
				}],
				Expression::new(
					self.db,
					&self.model,
					item,
					RecordLiteral(
						target_fields
							.iter()
							.map(|(field_ident, _)| {
								(
									*field_ident,
									Expression::new(
										self.db,
										&self.model,
										item,
										RecordAccess {
											record: Box::new(source_elem_expr.clone()),
											field: *field_ident,
										},
									),
								)
							})
							.collect(),
					),
				),
			),
		)
	}

	pub(in crate::lower) fn projected_nested_contribution_expr(
		&mut self,
		item: Item<'db>,
		source_occurrence: OccurrenceId,
		child_class: PatternRef<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Option<Expression<'db>> {
		let source_contribution = self.occurrence_contribution(source_occurrence, child_class);
		self.class_object_contribution_declaration(
			child_class,
			source_contribution.constructor_index,
		)
		.map(|source_decl| {
			self.projected_contribution_expr_from_declaration(item, source_decl, target_fields)
		})
	}

	/// Engine iteration context for a depth-1 nested flattened contribution:
	/// `p in index_set(inputs)`, `r in index_set((inputs[p]).<attribute>)`,
	/// `input = (inputs[p]).<attribute>[r]`. Class-typed grandchild fields
	/// mint via the nested flattened regimes
	/// (`reconstructed_nested_flattened_field_expr`), which need the indexed
	/// iteration for their prefix-sum ordinal arithmetic. Par-only (the input
	/// is a par inline-record collection), so slots are always realised and
	/// no realisation guard is passed.
	pub(in crate::lower) fn reconstructed_nested_flattened_contribution_expr(
		&mut self,
		item: Item<'db>,
		target_class: PatternRef<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Expression<'db> {
		let input_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![inputs_expr.clone()],
			},
		);
		let mut input_index_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		input_index_decl.set_name(Identifier::new(self.db, "p"));
		let input_index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_index_decl, item));
		let input_index_expr = Expression::new(self.db, &self.model, item, input_index_decl_idx);
		let current_root = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![inputs_expr.clone(), input_index_expr.clone()],
			},
		);
		let current_collection = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(current_root),
				field: attribute,
			},
		);
		let child_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![current_collection.clone()],
			},
		);
		let mut child_index_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		child_index_decl.set_name(Identifier::new(self.db, "r"));
		let child_index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(child_index_decl, item));
		let child_index_expr = Expression::new(self.db, &self.model, item, child_index_decl_idx);
		let mut input_decl = Declaration::new(
			false,
			Domain::unbounded(
				self.db,
				item,
				current_collection
					.ty()
					.elem_ty(self.db)
					.expect("nested flattened collection should be an array"),
			),
		);
		input_decl.set_name(Identifier::new(self.db, "input"));
		input_decl.set_definition(Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![current_collection.clone(), child_index_expr.clone()],
			},
		));
		let input_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_decl, item));
		let current_input = Expression::new(self.db, &self.model, item, input_decl_idx);
		let generators = vec![
			Generator::Iterator {
				declarations: vec![input_index_decl_idx],
				collection: input_index_set,
				where_clause: None,
			},
			Generator::Iterator {
				declarations: vec![child_index_decl_idx],
				collection: child_index_set,
				where_clause: None,
			},
			Generator::Assignment {
				assignment: input_decl_idx,
				where_clause: None,
			},
		];
		self.engine_reconstructed_contribution_expr(
			item,
			target_class,
			generators,
			current_input,
			target_fields,
			EngineIdentityRule::NestedFlattened {
				root_pattern,
				inputs_expr,
				attribute,
				current_collection,
				input_index_expr,
				child_index_expr,
			},
			None,
		)
	}

	/// Reconstruct the contribution array for a par `new X` (singular)
	/// attribute of a par-introduced parent, minting `<X>_potential`
	/// identities for X's object-typed fields.
	///
	/// The `OnePerParent` twin of
	/// `reconstructed_nested_flattened_contribution_expr`: each parent `p`
	/// contributes exactly one child record `inputs[p].<attribute>` (not an
	/// array of children), so there is no inner sibling iteration.
	/// Object-typed fields of the child are minted through
	/// `EngineIdentityRule::NestedSingular` — otherwise the par input
	/// record's inline child records would be stored where the identity
	/// model (`<Child>_potential`) is expected, which MiniZinc rejects.
	pub(in crate::lower) fn reconstructed_nested_singular_contribution_expr(
		&mut self,
		item: Item<'db>,
		target_class: PatternRef<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Expression<'db> {
		let input_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![inputs_expr.clone()],
			},
		);
		let mut input_index_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		input_index_decl.set_name(Identifier::new(self.db, "p"));
		let input_index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_index_decl, item));
		let input_index_expr = Expression::new(self.db, &self.model, item, input_index_decl_idx);
		let current_root = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![inputs_expr.clone(), input_index_expr.clone()],
			},
		);
		// The single child record `inputs[p].<attribute>` — a plain record
		// projection (contrast the flattened path's array element).
		let child_record = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(current_root),
				field: attribute,
			},
		);
		let mut input_decl =
			Declaration::new(false, Domain::unbounded(self.db, item, child_record.ty()));
		input_decl.set_name(Identifier::new(self.db, "input"));
		input_decl.set_definition(child_record);
		let input_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_decl, item));
		let current_input = Expression::new(self.db, &self.model, item, input_decl_idx);
		let generators = vec![
			Generator::Iterator {
				declarations: vec![input_index_decl_idx],
				collection: input_index_set,
				where_clause: None,
			},
			Generator::Assignment {
				assignment: input_decl_idx,
				where_clause: None,
			},
		];
		self.engine_reconstructed_contribution_expr(
			item,
			target_class,
			generators,
			current_input,
			target_fields,
			EngineIdentityRule::NestedSingular {
				root_pattern,
				inputs_expr,
				attribute,
				input_index_expr,
			},
			None,
		)
	}

	/// Mint the identity for an object-typed field of a singular nested
	/// child (`EngineIdentityRule::NestedSingular`). The `OnePerParent`
	/// twin of `reconstructed_nested_flattened_field_expr`: prefix sums run
	/// over parents only — each parent owns exactly one child, so there is
	/// no sibling term, and the "collection" for a previous parent is its
	/// single child record (a plain projection, not an array iteration).
	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn reconstructed_nested_singular_field_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		attribute: Identifier<'db>,
		current_input: Expression<'db>,
		input_index_expr: Expression<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
	) -> Expression<'db> {
		let Some(field_class) = field_ty.class_type(self.db) else {
			// A non-class storage field: read it from the input record when
			// present, otherwise mint a fresh decision (a dropped `var`
			// attribute of the singular child).
			if !self.record_ty_has_field(&current_input, field_ident) {
				return self.fresh_storage_field_decision(item, field_ident, field_ty);
			}
			return Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(current_input),
					field: field_ident,
				},
			);
		};
		let field_class =
			class_pattern_for(self.db, field_class).expect("class item for class type");
		if let Some(mint) =
			self.var_existence_field_mint(item, field_class, field_ident, field_ty, &current_input)
		{
			// A var-existence object field on a par owner reached one hop
			// below the root through a singular (`new`) edge — mint a fresh var
			// subset of its block (see `var_existence_field_mint`).
			return mint;
		}
		let child_occurrence = self.nested_occurrence(root_pattern, &[attribute, field_ident]);
		let child_contribution = self.occurrence_contribution(child_occurrence, field_class);
		let child_enum = self.class_map[&field_class].class_enum;
		let child_enum_member =
			EnumMemberId::new(child_enum, child_contribution.constructor_index as u32);
		let one_expr = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		match self.occurrence_local_domain_source(child_occurrence) {
			LocalDomainSource::OnePerParent => {
				// Doubly-singular chain: one grand-child per (one child per
				// parent), so the grand-child's ordinal is the parent index
				// `p` itself (`1 + (p-1) previous parents + 0 siblings`).
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![input_index_expr],
					},
				)
			}
			LocalDomainSource::FlattenedChildCollection => {
				let current_children = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(current_input.clone()),
						field: field_ident,
					},
				);
				let current_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![current_children],
					},
				);
				// `sum(q in 1..p-1)( length(inputs[q].<attribute>.<field>) )`
				// — each earlier parent's single child contributes its own
				// grand-collection length to the flattened private universe.
				let previous_input_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![input_index_expr.clone(), one_expr.clone()],
					},
				);
				let previous_input_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), previous_input_end],
					},
				);
				let mut previous_input_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				previous_input_decl.set_name(Identifier::new(self.db, "q"));
				let previous_input_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(previous_input_decl, item));
				let previous_input_expr =
					Expression::new(self.db, &self.model, item, previous_input_decl_idx);
				let previous_root = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![inputs_expr, previous_input_expr],
					},
				);
				let previous_child = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(previous_root),
						field: attribute,
					},
				);
				let previous_child_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(previous_child),
								field: field_ident,
							},
						)],
					},
				);
				let prefix_sum = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[Generator::Iterator {
									declarations: vec![previous_input_decl_idx],
									collection: previous_input_range,
									where_clause: None,
								}],
								previous_child_length,
							),
						)],
					},
				);
				let ordinal_start = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![one_expr.clone(), prefix_sum.clone()],
					},
				);
				if field_ty.opt(self.db) == Some(OptType::Opt) {
					// An `opt new C` grand-field holds the single realised
					// child identity or `<>`, not a range set.
					return self.opt_child_identity_or_absent(
						item,
						child_enum_member,
						ordinal_start,
						current_length,
					);
				}
				let ordinal_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![prefix_sum, current_length],
					},
				);
				let ordinal_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![ordinal_start, ordinal_end],
					},
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal_range],
					},
				)
			}
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("nested singular object field had unexpected root-only domain source")
			}
		}
	}

	/// Flatten a par root's input records down the `full_path` chain (root →
	/// field-owning class), returning the flattening generators and the cursor
	/// bound to the field owner's input record. Par-only companion of
	/// `nested_path_generators_and_cursor`: every hop is inlined in the par
	/// input record, so a SET edge (`cursor.<attr>` is an array of records)
	/// adds an iterator `j<i> in cursor.<attr>` and a SINGULAR edge
	/// (`cursor.<attr>` is a record) merely projects — the same left-to-right
	/// canonical order the universe sum and the leaf `<C>_objects` flattening
	/// use, which is what keeps the minted identity ranges pointing at the
	/// right objects.
	pub(in crate::lower) fn deep_flatten_generators_and_cursor(
		&mut self,
		item: Item<'db>,
		inputs_expr: &Expression<'db>,
		full_path: &[Identifier<'db>],
	) -> (Vec<Generator<'db>>, Expression<'db>) {
		let mut top_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, inputs_expr.ty().elem_ty(self.db).unwrap()),
		);
		top_decl.set_name(Identifier::new(self.db, "i"));
		let top_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(top_decl, item));
		let mut generators = vec![Generator::Iterator {
			declarations: vec![top_decl_idx],
			collection: inputs_expr.clone(),
			where_clause: None,
		}];
		let mut cursor = Expression::new(self.db, &self.model, item, top_decl_idx);
		for (idx, attrib) in full_path.iter().enumerate() {
			let record_access = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(cursor),
					field: *attrib,
				},
			);
			match record_access.ty().elem_ty(self.db) {
				Some(elem_ty) => {
					// SET edge: iterate the inlined child array.
					let mut attrib_decl =
						Declaration::new(false, Domain::unbounded(self.db, item, elem_ty));
					attrib_decl.set_name(Identifier::new(self.db, format!("j{}", idx + 1)));
					let attrib_decl_idx = self
						.model
						.add_declaration(DeclarationItem::new(attrib_decl, item));
					generators.push(Generator::Iterator {
						declarations: vec![attrib_decl_idx],
						collection: record_access,
						where_clause: None,
					});
					cursor = Expression::new(self.db, &self.model, item, attrib_decl_idx);
				}
				// SINGULAR edge: the inlined child record — project and continue.
				None => cursor = record_access,
			}
		}
		(generators, cursor)
	}

	/// Reconstruct the contribution array for a par nested object class
	/// introduced ≥ 2 `new`-hops below a par root. Flattens the field
	/// owner's par input records once (`deep_flatten_generators_and_cursor`),
	/// then reconstructs each storage record over the flat position `ci`,
	/// minting object fields through `EngineIdentityRule::NestedDeep` with a
	/// 1-D prefix sum. Depth-agnostic: the same builder serves depth 2, 3, …
	/// because the flattening absorbs every intermediate hop.
	pub(in crate::lower) fn reconstructed_deep_nested_contribution_expr(
		&mut self,
		item: Item<'db>,
		target_class: PatternRef<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		full_path: &[Identifier<'db>],
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Expression<'db> {
		// The deep builder flattens PAR input records and mints PAR identity
		// ranges for par-existence object grand-fields. A var-existence
		// (var-actual-set) grand-field is dropped from the par input record;
		// the field-minting arm (`reconstructed_deep_nested_field_expr` via
		// `var_existence_field_mint`) mints it as a fresh free var subset of
		// its block instead of reading `length(input.<field>)`, so such
		// shapes are handled here rather than fenced.
		let (flat_generators, flat_cursor) =
			self.deep_flatten_generators_and_cursor(item, &inputs_expr, full_path);
		let flat_compr = Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(flat_generators, flat_cursor),
		);
		let mut flat_decl = Declaration::from_expression(self.db, false, flat_compr);
		flat_decl.set_name(Identifier::new(
			self.db,
			format!(
				"{}_flat_inputs",
				full_path
					.iter()
					.map(|a| a.pretty_print(self.db))
					.collect::<Vec<_>>()
					.join("_")
			),
		));
		// Bound in a `let` wrapping the whole contribution comprehension (below):
		// a bare model declaration would not be reached by the emitter, and the
		// flat list must be materialised (indexable by `ci`/`cj`) for the 1-D
		// prefix sum.
		let flat_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(flat_decl, item));
		let flat_inputs_expr = Expression::new(self.db, &self.model, item, flat_decl_idx);

		let flat_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![flat_inputs_expr.clone()],
			},
		);
		let mut ci_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		ci_decl.set_name(Identifier::new(self.db, "ci"));
		let ci_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(ci_decl, item));
		let ci_expr = Expression::new(self.db, &self.model, item, ci_decl_idx);
		let mut input_decl = Declaration::new(
			false,
			Domain::unbounded(
				self.db,
				item,
				flat_inputs_expr
					.ty()
					.elem_ty(self.db)
					.expect("flattened input list should be an array"),
			),
		);
		input_decl.set_name(Identifier::new(self.db, "input"));
		input_decl.set_definition(Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![flat_inputs_expr.clone(), ci_expr.clone()],
			},
		));
		let input_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(input_decl, item));
		let current_input = Expression::new(self.db, &self.model, item, input_decl_idx);
		let generators = vec![
			Generator::Iterator {
				declarations: vec![ci_decl_idx],
				collection: flat_index_set,
				where_clause: None,
			},
			Generator::Assignment {
				assignment: input_decl_idx,
				where_clause: None,
			},
		];
		let comprehension = self.engine_reconstructed_contribution_expr(
			item,
			target_class,
			generators,
			current_input,
			target_fields,
			EngineIdentityRule::NestedDeep {
				root_pattern,
				full_path: full_path.to_vec(),
				flat_inputs_expr,
				flat_index_expr: ci_expr,
			},
			None,
		);
		Expression::new(
			self.db,
			&self.model,
			item,
			Let {
				items: vec![LetItem::Declaration(flat_decl_idx)],
				in_expression: Box::new(comprehension),
			},
		)
	}

	/// Mint the identity for an object-typed field of a deep (≥ depth-2) par
	/// nested child (`EngineIdentityRule::NestedDeep`). The flat position
	/// `flat_index_expr` (`ci`) is the field owner's ordinal in canonical path
	/// order, so a single 1-D prefix sum over `flat_inputs_expr[cj].<field>`
	/// lengths locates the grand-child identity range — no multi-level offset
	/// arithmetic, the flattening already spanned every hop.
	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn reconstructed_deep_nested_field_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		full_path: &[Identifier<'db>],
		flat_inputs_expr: Expression<'db>,
		flat_index_expr: Expression<'db>,
		current_input: Expression<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
	) -> Expression<'db> {
		let Some(field_class) = field_ty.class_type(self.db) else {
			// Non-class storage field: read from the flat input record when
			// present, otherwise mint a fresh decision (a dropped `var`
			// attribute of the deep child).
			if !self.record_ty_has_field(&current_input, field_ident) {
				return self.fresh_storage_field_decision(item, field_ident, field_ty);
			}
			return Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(current_input),
					field: field_ident,
				},
			);
		};
		let field_class =
			class_pattern_for(self.db, field_class).expect("class item for class type");
		if let Some(mint) =
			self.var_existence_field_mint(item, field_class, field_ident, field_ty, &current_input)
		{
			// A var-existence object grand-field on a par owner two or more
			// `new`-hops below the root — mint a fresh var subset of its block
			// (see `var_existence_field_mint`). The flattening spans only the
			// par field-owner inputs; the var field is realised as a free
			// subset, not read off the (dropped) input length.
			return mint;
		}
		let mut child_path = full_path.to_vec();
		child_path.push(field_ident);
		let child_occurrence = self.nested_occurrence(root_pattern, &child_path);
		let child_contribution = self.occurrence_contribution(child_occurrence, field_class);
		let child_enum = self.class_map[&field_class].class_enum;
		let child_enum_member =
			EnumMemberId::new(child_enum, child_contribution.constructor_index as u32);
		match self.occurrence_local_domain_source(child_occurrence) {
			LocalDomainSource::OnePerParent => {
				// Singular grand-child: exactly one per field-owner instance, so
				// the grand-children are in bijection with the field owners in
				// the same canonical order — the grand-child's ordinal IS the
				// flat position `ci`.
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![flat_index_expr],
					},
				)
			}
			LocalDomainSource::FlattenedChildCollection => {
				let one_expr = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
				let current_children = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(current_input.clone()),
						field: field_ident,
					},
				);
				let current_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![current_children],
					},
				);
				// prefix = sum([ length(flat_inputs[cj].<field>) | cj in 1..ci-1 ])
				let prev_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.minus.into(),
						arguments: vec![flat_index_expr.clone(), one_expr.clone()],
					},
				);
				let prev_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![one_expr.clone(), prev_end],
					},
				);
				let mut cj_decl = Declaration::new(
					false,
					Domain::unbounded(self.db, item, Ty::par_int(self.db)),
				);
				cj_decl.set_name(Identifier::new(self.db, "cj"));
				let cj_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(cj_decl, item));
				let cj_expr = Expression::new(self.db, &self.model, item, cj_decl_idx);
				let prev_input = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![flat_inputs_expr, cj_expr],
					},
				);
				let prev_field_length = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.length.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(prev_input),
								field: field_ident,
							},
						)],
					},
				);
				let prefix_sum = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.sum.into(),
						arguments: vec![Expression::new(
							self.db,
							&self.model,
							item,
							ArrayComprehension::new(
								[Generator::Iterator {
									declarations: vec![cj_decl_idx],
									collection: prev_range,
									where_clause: None,
								}],
								prev_field_length,
							),
						)],
					},
				);
				let ordinal_start = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![one_expr, prefix_sum.clone()],
					},
				);
				if field_ty.opt(self.db) == Some(OptType::Opt) {
					// An `opt new` grand-field holds the single realised child
					// identity or `<>`, not a range set.
					return self.opt_child_identity_or_absent(
						item,
						child_enum_member,
						ordinal_start,
						current_length,
					);
				}
				let ordinal_end = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.builtins.plus.into(),
						arguments: vec![prefix_sum, current_length],
					},
				);
				let ordinal_range = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.dot_dot.into(),
						arguments: vec![ordinal_start, ordinal_end],
					},
				);
				Expression::new(
					self.db,
					&self.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal_range],
					},
				)
			}
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("deep nested object field had unexpected root-only domain source")
			}
		}
	}

	/// Whether `record_expr`'s record type declares `field`. Used to decide
	/// whether a storage field can be read straight from the (par) input
	/// record or must be reconstructed.
	pub(in crate::lower) fn record_ty_has_field(
		&self,
		record_expr: &Expression<'db>,
		field: Identifier<'db>,
	) -> bool {
		record_expr
			.ty()
			.record_fields(self.db)
			.map(|fields| fields.iter().any(|(f, _)| Identifier(*f) == field))
			.unwrap_or(false)
	}

	/// Mint a fresh decision variable of `field_ty` for a storage field that
	/// the par input record doesn't supply. A `var` attribute is dropped from
	/// the input record (it's a decision, not data — see
	/// `class_type_to_input_record_type`), but it is still a storage field, so
	/// each contributed object needs its own free decision of the field type.
	pub(in crate::lower) fn fresh_storage_field_decision(
		&mut self,
		item: Item<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
	) -> Expression<'db> {
		let mut fresh_decl = Declaration::new(false, Domain::unbounded(self.db, item, field_ty));
		fresh_decl.set_name(Identifier::new(
			self.db,
			format!("{}_init", field_ident.pretty_print(self.db)),
		));
		let fresh_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(fresh_decl, item));
		let fresh_expr = Expression::new(self.db, &self.model, item, fresh_decl_idx);
		Expression::new(
			self.db,
			&self.model,
			item,
			Let {
				items: vec![LetItem::Declaration(fresh_decl_idx)],
				in_expression: Box::new(fresh_expr),
			},
		)
	}

	/// Mint a par owner's *var-existence* object field (`var set of new
	/// D` / `var opt new D`) as a fresh free `var set of <D>_potential` / `var
	/// opt <D>_potential` decision. Such a field's existence is a solver
	/// decision, so — like a `var` scalar attribute — it is dropped from the
	/// par input record (`class_type_to_input_record_type`). The par-owner
	/// reconstruction builders would otherwise mint a par identity range
	/// (`D_occ(prefix+1 .. prefix + length(input.<field>))`), which panics
	/// reading the dropped field. The block is realised as a *free subset*
	/// instead: the per-parent block-subset constraint (set) / occurs pin
	/// (opt) confining it to its slice and the actual-set union are emitted
	/// separately by the slice-array / `var_actual_set_classes` machinery, so
	/// this slot only needs to be a free decision of the substituted storage
	/// type. This is the par-owner composition of the two regimes that already
	/// work: a par-reconstructed owner (which already mints free `var` scalars)
	/// hosting a var-subset-realised object field (the var-root regime).
	///
	/// Returns `None` for par-existence class fields (par `set of new` /
	/// singular `new` / `var new` var-storage), which keep their par
	/// identity-range / read-through minting. The gate has three parts:
	///
	/// - the field's class is var-actual-set (`var_actual_set_classes`);
	/// - the field itself is var (not just the class) — so a par `set of new D`
	///   field of a class `D` that is var-actual-set only through *another*
	///   (var) introduction site still mints its dense par range;
	/// - the field is DROPPED from the par input record. A genuine
	///   var-existence field carries no data, so it is absent from the input
	///   (`class_type_to_input_record_type` drops it). A field that IS present
	///   in the input — e.g. a par `set of new B` on a class that is var-reached
	///   from elsewhere, whose type is varified but whose value is still concrete
	///   input data on a par object (`P.kid = (children: [(x: 2)])`) — must be
	///   reconstructed from that data as identities, NOT replaced by a free
	///   decision (which would drop the data and over-generate).
	pub(in crate::lower) fn var_existence_field_mint(
		&mut self,
		item: Item<'db>,
		field_class: PatternRef<'db>,
		field_ident: Identifier<'db>,
		field_ty: Ty<'db>,
		current_input: &Expression<'db>,
	) -> Option<Expression<'db>> {
		if self
			.object_lowering
			.var_actual_set_classes
			.contains(&field_class)
			&& field_ty.inst(self.db) == Some(VarType::Var)
			&& !self.record_ty_has_field(current_input, field_ident)
		{
			let storage_field_ty = self.substitute_class_with_potential_enum(field_ty);
			Some(self.fresh_storage_field_decision(item, field_ident, storage_field_ty))
		} else {
			None
		}
	}

	/// Build the per-record template for a nested contribution comprehension:
	/// for each storage field, read it from `contribution_input` when present,
	/// otherwise mint a fresh decision (a dropped `var` attribute). Projecting
	/// every storage field unconditionally would panic in `RecordAccess::build`
	/// when a `var` field was dropped from the par input record.
	pub(in crate::lower) fn nested_contribution_template_record(
		&mut self,
		item: Item<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
		contribution_input: &Expression<'db>,
	) -> Expression<'db> {
		let mut record_fields = Vec::with_capacity(target_fields.len());
		for (field_ident, field_ty) in target_fields.iter().copied() {
			let value = if self.record_ty_has_field(contribution_input, field_ident) {
				Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(contribution_input.clone()),
						field: field_ident,
					},
				)
			} else {
				self.fresh_storage_field_decision(item, field_ident, field_ty)
			};
			record_fields.push((field_ident, value));
		}
		Expression::new(self.db, &self.model, item, RecordLiteral(record_fields))
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn selected_nested_contribution_expr(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		source_occurrence: OccurrenceId,
		child_class: PatternRef<'db>,
		target_class: PatternRef<'db>,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
		local_domain_source: LocalDomainSource,
		attrib_path: &[Identifier<'db>],
		attribute: Identifier<'db>,
		contribution_generators: &[Generator<'db>],
		contribution_input: Expression<'db>,
		needs_storage_projection: bool,
		object_field_constructors_available: bool,
	) -> Expression<'db> {
		if needs_storage_projection {
			return self
				.projected_nested_contribution_expr(
					item,
					source_occurrence,
					child_class,
					target_fields,
				)
				.expect("source child contribution should exist before inherited projection");
		}

		let has_object_fields = target_fields
			.iter()
			.any(|(_, field_ty)| field_ty.class_type(self.db).is_some());

		if target_class != child_class {
			// Build the projection template lazily: an eager template that
			// projected every storage field would panic on a `var` attribute
			// dropped from the par input record before we even reach the
			// identity-reconstruction branch below.
			let contribution_template =
				self.nested_contribution_template_record(item, target_fields, &contribution_input);
			return Expression::new(
				self.db,
				&self.model,
				item,
				ArrayComprehension::new(contribution_generators.to_vec(), contribution_template),
			);
		}

		if has_object_fields
			&& matches!(
				local_domain_source,
				LocalDomainSource::FlattenedChildCollection
			) && attrib_path.is_empty()
			&& object_field_constructors_available
		{
			return self.reconstructed_nested_flattened_contribution_expr(
				item,
				target_class,
				root_pattern,
				inputs_expr,
				attribute,
				target_fields,
			);
		}

		if has_object_fields
			&& matches!(local_domain_source, LocalDomainSource::OnePerParent)
			&& attrib_path.is_empty()
			&& object_field_constructors_available
		{
			// A par `new X` (singular) attribute whose child X owns
			// object-typed fields. The default `ReadOrMint` engine below would
			// store the input's inline child records where the identity model
			// (`<Child>_potential`) is expected — MiniZinc rejects that shape.
			// Mint identities instead.
			return self.reconstructed_nested_singular_contribution_expr(
				item,
				target_class,
				root_pattern,
				inputs_expr,
				attribute,
				target_fields,
			);
		}

		if has_object_fields
			&& !attrib_path.is_empty()
			&& matches!(
				local_domain_source,
				LocalDomainSource::FlattenedChildCollection | LocalDomainSource::OnePerParent
			) && object_field_constructors_available
		{
			// An object-carrying class introduced ≥ 2 `new`-hops below a par
			// root. The depth-1 builders above hardcode a 2-level generator
			// stack that can't span the path; the default `ReadOrMint` engine
			// below would store the input's inline grand-child records where
			// the identity model (`<GrandChild>_potential`) is expected — an
			// invalid emission. Flatten the field owner's par inputs once and
			// mint identities from a 1-D prefix sum (depth-agnostic). This
			// runs for a VAR-REACHED deep target too: the deep contribution
			// mints par identity ranges for data-supplied object fields and
			// free `var set`/`var opt` decisions for var-existence ones
			// (`var_existence_field_mint`), and the var-actual-set machinery
			// `++`s it with any var contributions (`var new C` /
			// `var set of new C`) into the class's var storage — the same
			// composition depth-1 var-reached nesting already uses.
			let mut full_path = attrib_path.to_vec();
			full_path.push(attribute);
			return self.reconstructed_deep_nested_contribution_expr(
				item,
				target_class,
				root_pattern,
				inputs_expr,
				&full_path,
				target_fields,
			);
		}

		// Input passthrough: the input carries every storage field (which also
		// means the class has no defined or dropped-var fields — those never
		// appear in par input records), so the per-element input IS the storage
		// record. Vacuously determined.
		let all_fields_present = target_fields
			.iter()
			.all(|(field_ident, _)| self.record_ty_has_field(&contribution_input, *field_ident));
		if !has_object_fields && all_fields_present {
			return Expression::new(
				self.db,
				&self.model,
				item,
				ArrayComprehension::new(contribution_generators.to_vec(), contribution_input),
			);
		}

		// Every other input-carrying nested shape runs the engine over the
		// caller's element iteration: defined fields alias-define their
		// collected RHS (a plain template would fresh-mint them, emitting a
		// valueless `let { int: y_init; }`), dropped-var fields mint fresh
		// decisions with their declared per-object domains, and readable
		// fields read through. Class-typed fields have no minting regime in
		// this context (`ReadOrMint`): they read through when the input
		// carries them and fresh-mint otherwise. Par-only input, so no
		// realisation guard.
		self.engine_reconstructed_contribution_expr(
			item,
			target_class,
			contribution_generators.to_vec(),
			contribution_input,
			target_fields,
			EngineIdentityRule::ReadOrMint,
			None,
		)
	}

	/// Build the set expression `<C>_occ_k(<local-universe>)` — the image
	/// of the class enum's `contribution_index`-th constructor applied to
	/// its full parameter domain. Returns `None` if the constructor is not
	/// yet present, is atomic, or has no bounded parameter domain; the
	/// caller falls back to the full class enum in that case.
	pub(in crate::lower) fn class_enum_constructor_image_set(
		&self,
		item: Item<'db>,
		class_enum: EnumerationId<'db>,
		contribution_index: usize,
	) -> Option<Expression<'db>> {
		let constructors = self.model[class_enum].definition()?;
		let constructor = constructors.get(contribution_index)?;
		let parameters = constructor.parameters.as_ref()?;
		let parameter_decl = *parameters.first()?;
		let range_expr = match &**self.model[parameter_decl].domain() {
			DomainData::Bounded(expr) => (**expr).clone(),
			_ => return None,
		};
		let member_id = EnumMemberId::new(class_enum, contribution_index as u32);
		Some(Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(member_id),
				arguments: vec![range_expr],
			},
		))
	}

	/// Build the engine contribution for a var nested occurrence whose class
	/// has defined fields. The free decisions live in a fresh uninitialized
	/// `<C>_<intro>_storage` array — element type `free_storage_record_ty`
	/// (computed / domain-dependent fields excluded), dim the constructor's
	/// enum image so positions align with the private `1..sum` universe — and
	/// the returned comprehension reconstructs the full storage record from
	/// it: free fields read through, defined fields alias-define their
	/// collected RHS. The realisation test is `p in <C>` directly: the
	/// enum-typed storage index IS the slot identity, so no
	/// constructor-ordinal arithmetic is needed (the child's actual set is
	/// derived from its parents' realised fields, which already encodes the
	/// whole parent-realisation chain).
	pub(in crate::lower) fn nested_var_storage_engine_contribution_expr(
		&mut self,
		item: Item<'db>,
		target_class: PatternRef<'db>,
		start_decl_name: &str,
		contribution_index: usize,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
	) -> Expression<'db> {
		let class_enum = self.class_map[&target_class].class_enum;
		let image_set_expr = self
			.class_enum_constructor_image_set(item, class_enum, contribution_index)
			.unwrap_or_else(|| Expression::new(self.db, &self.model, item, class_enum));
		let target_class_name = target_class
			.identifier(self.db)
			.unwrap()
			.pretty_print(self.db);
		let name_prefix = format!("{}_{}", target_class_name, start_decl_name);

		let full_record_ty = Ty::record(self.db, target_fields.to_vec());
		let free_record_ty = self.free_storage_record_ty(target_class, full_record_ty);
		let has_free_fields = free_record_ty
			.record_fields(self.db)
			.map(|fields| !fields.is_empty())
			.unwrap_or(false);

		let index_ty = Ty::par_enum(self.db, self.model[class_enum].enum_type());
		let mut index_decl = Declaration::new(false, Domain::unbounded(self.db, item, index_ty));
		index_decl.set_name(Identifier::new(self.db, "p"));
		let index_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(index_decl, item));
		let index_expr = Expression::new(self.db, &self.model, item, index_decl_idx);
		let mut generators = vec![Generator::Iterator {
			declarations: vec![index_decl_idx],
			collection: image_set_expr.clone(),
			where_clause: None,
		}];

		let current_input = if has_free_fields {
			let varified = free_record_ty
				.with_inst(self.db, VarType::Var)
				.unwrap_or(free_record_ty);
			let storage_elem_ty = self.substitute_class_with_potential_enum(varified);
			let storage_elem_dom =
				self.build_class_storage_record_domain(target_class, storage_elem_ty, item);
			let dim_domain =
				Domain::bounded(self.db, item, VarType::Par, OptType::NonOpt, image_set_expr);
			let storage_domain =
				Domain::array(self.db, item, OptType::NonOpt, dim_domain, storage_elem_dom);
			let mut storage_decl = Declaration::new(true, storage_domain);
			let storage_base = format!("{}_storage", name_prefix);
			let mut storage_ident = Identifier::new(self.db, storage_base.clone());
			let mut suffix = 2;
			while self
				.model
				.top_level_declarations()
				.any(|(_, declaration)| declaration.name() == Some(storage_ident))
			{
				storage_ident = Identifier::new(self.db, format!("{}_{}", storage_base, suffix));
				suffix += 1;
			}
			storage_decl.set_name(storage_ident);
			let storage_idx = self
				.model
				.add_declaration(DeclarationItem::new(storage_decl, item));
			let storage_expr = Expression::new(self.db, &self.model, item, storage_idx);
			let mut input_decl = Declaration::new(
				false,
				Domain::unbounded(
					self.db,
					item,
					storage_expr
						.ty()
						.elem_ty(self.db)
						.expect("nested free storage should be an array"),
				),
			);
			input_decl.set_name(Identifier::new(self.db, "input"));
			input_decl.set_definition(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![storage_expr, index_expr.clone()],
				},
			));
			let input_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(input_decl, item));
			generators.push(Generator::Assignment {
				assignment: input_decl_idx,
				where_clause: None,
			});
			Expression::new(self.db, &self.model, item, input_decl_idx)
		} else {
			// Every storage field is defined: no free storage to read from.
			// The engine never touches the input (no read rule can fire), so a
			// placeholder is passed purely to satisfy the signature.
			Expression::new(self.db, &self.model, item, BooleanLiteral(true))
		};

		self.engine_reconstructed_contribution_expr(
			item,
			target_class,
			generators,
			current_input,
			target_fields,
			EngineIdentityRule::ReadOrMint,
			Some(EngineRealisationGuard {
				name_prefix,
				test: EngineRealisationTest::Identity(index_expr),
			}),
		)
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn register_nested_class_object_contribution(
		&mut self,
		item: Item<'db>,
		target_class: PatternRef<'db>,
		start_decl_name: &str,
		contribution_index: usize,
		target_fields: &[(Identifier<'db>, Ty<'db>)],
		contribution_expr: Option<Expression<'db>>,
		defined_fields_determined: bool,
	) {
		// When the contribution is uninitialized (registered with no
		// definition), the element type is just the storage record. Opt-ness
		// from a `var opt new` source lives on the *parent's* identity
		// reference (`b: var opt <child>_potential`), not on the stored
		// records themselves — MiniZinc rejects `opt record(...)`.
		let contribution_elem_ty = contribution_expr
			.as_ref()
			.and_then(|expr| expr.ty().elem_ty(self.db))
			.unwrap_or_else(|| Ty::record(self.db, target_fields.to_vec()));
		// When the contribution is uninitialized (fresh nested child
		// storage), MZN needs par-known dimensions to flatten. Index by the
		// constructor's *enum image* (`<C>_occ_k(<local-universe>)`) so each
		// per-contribution `<C>_<intro>_objects` ends up with
		// `card(constructor)` slots, and the `'++'` concatenation that
		// `finish()` performs aligns int-positions exactly with global
		// ordinals in `<C>_potential` (each constructor occupies a contiguous
		// global-ordinal range). Without this, both per-contribution arrays
		// are sized to `card(<C>_potential)` and `'++'` produces a
		// `2 * card(<C>_potential)`-slot array; consumers using
		// `<C>_objects[enum2int(this)]` then only ever land in the first
		// contribution's range regardless of which constructor `this` came
		// from.
		let target_enum_decl = self
			.class_map
			.get(&target_class)
			.map(|info| info.class_enum);
		let dim_domain = if contribution_expr.is_none() {
			if let Some(enum_id) = target_enum_decl {
				let dim_expr = self
					.class_enum_constructor_image_set(item, enum_id, contribution_index)
					.unwrap_or_else(|| Expression::new(self.db, &self.model, item, enum_id));
				Domain::bounded(self.db, item, VarType::Par, OptType::NonOpt, dim_expr)
			} else {
				Domain::unbounded(self.db, item, Ty::par_int(self.db))
			}
		} else {
			Domain::unbounded(self.db, item, Ty::par_int(self.db))
		};
		let elem_domain =
			self.build_class_storage_record_domain(target_class, contribution_elem_ty, item);
		let contribution_domain =
			Domain::array(self.db, item, OptType::NonOpt, dim_domain, elem_domain);
		let mut contribution_decl = Declaration::new(true, contribution_domain);
		let target_class_name = target_class
			.identifier(self.db)
			.unwrap()
			.pretty_print(self.db);
		let mut contribution_name = format!("{}_{}_objects", target_class_name, start_decl_name);
		let mut contribution_ident = Identifier::new(self.db, contribution_name.clone());
		if self
			.model
			.top_level_declarations()
			.any(|(_, declaration)| declaration.name() == Some(contribution_ident))
		{
			contribution_name = format!(
				"{}_{}_occ_{}_objects",
				target_class_name, start_decl_name, contribution_index
			);
			contribution_ident = Identifier::new(self.db, contribution_name.clone());
			let mut suffix = 2;
			while self
				.model
				.top_level_declarations()
				.any(|(_, declaration)| declaration.name() == Some(contribution_ident))
			{
				contribution_ident =
					Identifier::new(self.db, format!("{}_{}", contribution_name, suffix));
				suffix += 1;
			}
		}
		contribution_decl.set_name(contribution_ident);
		if let Some(contribution_expr) = contribution_expr {
			contribution_decl.set_definition(contribution_expr);
		}
		let contribution_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(contribution_decl, item));
		self.register_class_object_contribution(
			target_class,
			contribution_index,
			contribution_decl_idx,
			defined_fields_determined,
		);
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn emit_nested_occurrence_contributions(
		&mut self,
		item: Item<'db>,
		root_pattern: PatternRef<'db>,
		inputs_expr: Expression<'db>,
		source_occurrence: OccurrenceId,
		child_class: PatternRef<'db>,
		local_domain_source: LocalDomainSource,
		attrib_path: &[Identifier<'db>],
		attribute: Identifier<'db>,
		contribution_generators: &[Generator<'db>],
		maybe_contribution_input: Option<Expression<'db>>,
		start_decl_name: &str,
	) {
		let mut occurrence_contributions =
			self.object_lowering.contributions_by_occurrence[&source_occurrence].clone();
		occurrence_contributions.sort_by_key(|contribution| contribution.projection_depth);

		if let Some(contribution_input) = maybe_contribution_input {
			for contribution in occurrence_contributions.iter() {
				let target_class = contribution.target_class;
				let target_fields = self.class_storage_fields(target_class);
				let needs_storage_projection = target_class != child_class
					&& target_fields.iter().any(|(field_ident, field_ty)| {
						// A field missing from the input record (a defined/dropped-var
						// field) must be projected from the child's minted contribution.
						// So must an OBJECT field even when it IS in the input: the par
						// input carries it as an inline record, but the child's minted
						// contribution stores it as a `<GrandChild>_potential` identity,
						// and the superclass projection must read the identity, not the
						// inline record (the inline template arm below would store the
						// wrong shape and MiniZinc rejects the identity read).
						!self.record_expr_has_field(&contribution_input, *field_ident)
							|| field_ty.class_type(self.db).is_some()
					});
				let object_field_constructors_available =
					target_fields.iter().all(|(field_ident, field_ty)| {
						let Some(field_class) = field_ty.class_type(self.db) else {
							return true;
						};
						let field_class = class_pattern_for(self.db, field_class)
							.expect("class item for class type");
						// The grand-child occurrence sits at the FULL path from the
						// root: the parent's `attrib_path`, this `attribute`, then
						// the object field. At depth 1 `attrib_path` is empty so
						// this is `[attribute, field_ident]` (unchanged); at depth
						// ≥ 2 the prefix is what locates the grand-child.
						let mut grandchild_path = attrib_path.to_vec();
						grandchild_path.push(attribute);
						grandchild_path.push(*field_ident);
						let Some(child_occurrence) =
							self.maybe_nested_occurrence(root_pattern, &grandchild_path)
						else {
							return false;
						};
						let child_contribution =
							self.occurrence_contribution(child_occurrence, field_class);
						let child_enum = self.class_map[&field_class].class_enum;
						self.model[child_enum]
							.definition()
							.map(|constructors| {
								constructors.len() > child_contribution.constructor_index
							})
							.unwrap_or(false)
					});
				let contribution_expr = self.selected_nested_contribution_expr(
					item,
					root_pattern,
					inputs_expr.clone(),
					source_occurrence,
					child_class,
					target_class,
					&target_fields,
					local_domain_source,
					attrib_path,
					attribute,
					contribution_generators,
					contribution_input.clone(),
					needs_storage_projection,
					object_field_constructors_available,
				);
				// Determinedness: the `target == child` arms either run the
				// engine (defined fields alias-defined) or are vacuously
				// determined passthroughs, and the `target != child` template
				// arm only fires when the input carries every target field —
				// i.e. the target has no defined fields. The storage
				// projection reads every field from the child's registered
				// contribution decl and inherits exactly its flag.
				let defined_fields_determined = if needs_storage_projection {
					let source_index = self
						.occurrence_contribution(source_occurrence, child_class)
						.constructor_index;
					self.contribution_determined(child_class, source_index)
						.unwrap_or(false)
				} else {
					true
				};
				self.register_nested_class_object_contribution(
					item,
					target_class,
					start_decl_name,
					contribution.constructor_index,
					&target_fields,
					Some(contribution_expr),
					defined_fields_determined,
				);
			}
		} else if matches!(
			local_domain_source,
			LocalDomainSource::OnePerParent | LocalDomainSource::FlattenedChildCollection
		) {
			// A nested contribution only lacks a record-typed input when the
			// parent-side collection is identity-typed VAR storage or the
			// field is excluded from the parent's par input record because it
			// is explicitly var — par chains inline child INPUT records at
			// every hop (singular fields as nested records, collections as
			// arrays of records), so both
			// `nested_contribution_generators_and_input` `None` arms imply a
			// var introduction edge. Var-ness cascades through every
			// `new`-attribute edge and up the inheritance chain
			// (`var_reached_classes`), so every projection target here is
			// var-reached: a par-reached identity-mode fallback is
			// unreachable.
			debug_assert!(
				occurrence_contributions.iter().all(|contribution| self
					.object_lowering
					.var_reached_classes
					.contains(&contribution.target_class)),
				"nested contribution without record input reached a par-reached target"
			);
			for contribution in occurrence_contributions.iter() {
				let target_class = contribution.target_class;
				let target_fields = self.class_storage_fields(target_class);
				let target_has_defined_field = self
					.class_storage_field_decls(target_class.item(self.db))
					.iter()
					.any(|d| {
						d.definition.is_some()
							|| self.field_domain_references_attribute(d.owner, d.declared_type)
					});
				let (contribution_expr, defined_fields_determined) = if target_class == child_class
				{
					if target_has_defined_field
						&& self
							.object_lowering
							.var_reached_classes
							.contains(&target_class)
					{
						// Var nested storage with defined fields: the free
						// decisions live in a separate `<C>_<intro>_storage`
						// array and the contribution is the engine
						// reconstruction over it — computed / domain-dependent
						// fields are alias-defined per slot,
						// realisation-guarded on `p in <C>` (a nested slot
						// under a var-existence chain can be unrealised).
						let engine_expr = self.nested_var_storage_engine_contribution_expr(
							item,
							target_class,
							start_decl_name,
							contribution.constructor_index,
							&target_fields,
						);
						(Some(engine_expr), true)
					} else {
						// All-free storage: uninitialized var-record storage,
						// vacuously determined exactly when the class has no
						// defined fields. (Always a var-reached target — see
						// the assert above.)
						(None, !target_has_defined_field)
					}
				} else {
					let projection = self.projected_nested_contribution_expr(
						item,
						source_occurrence,
						child_class,
						&target_fields,
					);
					let determined = match &projection {
						Some(_) => {
							let source_index = self
								.occurrence_contribution(source_occurrence, child_class)
								.constructor_index;
							self.contribution_determined(child_class, source_index)
								.unwrap_or(false)
						}
						None => !target_has_defined_field,
					};
					(projection, determined)
				};
				self.register_nested_class_object_contribution(
					item,
					target_class,
					start_decl_name,
					contribution.constructor_index,
					&target_fields,
					contribution_expr,
					defined_fields_determined,
				);
			}
		}
	}

	pub(in crate::lower) fn nested_occurrence_sum_expr(
		&mut self,
		item: Item<'db>,
		generators: Vec<Generator<'db>>,
		local_domain_source: LocalDomainSource,
		record_access: Option<Expression<'db>>,
		fallback_cardinality: Option<Expression<'db>>,
		parent_class: PatternRef<'db>,
	) -> Expression<'db> {
		// When the per-parent slice size is a static constant (the nested
		// fresh-child case), emit `card(<parent>_potential) * fallback`
		// directly. Iterating over the parent's storage to sum the same
		// constant works mathematically but creates a circular type dependency:
		// the parent's storage record type references `<child>_potential`,
		// whose size we're trying to compute, which would then reference the
		// storage's index_set... cycle. Going through the parent's potential
		// enum (par, defined independently) breaks the cycle.
		//
		// Two shapes hit this:
		//  - `FlattenedChildCollection` with an explicit `fallback_cardinality`
		//    from a declared `set of new <C>: <bound>` field.
		//  - `OnePerParent` (singular nested `new <C>: <attr>`), where the
		//    per-parent count is the implicit constant `1`.
		let cycle_break_fallback: Option<Expression<'db>> = match local_domain_source {
			LocalDomainSource::OnePerParent => Some(Expression::new(
				self.db,
				&self.model,
				item,
				IntegerLiteral(1),
			)),
			LocalDomainSource::FlattenedChildCollection => {
				let is_set_record_access = record_access
					.as_ref()
					.map(|ra| ra.ty().is_set(self.db))
					.unwrap_or(true);
				if is_set_record_access {
					fallback_cardinality.clone()
				} else {
					None
				}
			}
			_ => None,
		};
		if let Some(fallback) = cycle_break_fallback {
			let class_info = self.class_map[&parent_class];
			let parent_enum_expr =
				Expression::new(self.db, &self.model, item, class_info.class_enum);
			let card_expr = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.card.into(),
					arguments: vec![parent_enum_expr],
				},
			);
			return Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.times.into(),
					arguments: vec![card_expr, fallback],
				},
			);
		}
		let compr_template = self.occurrence_local_domain_size_expr(
			item,
			local_domain_source,
			record_access,
			fallback_cardinality,
		);
		let compr = Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(generators, compr_template),
		);
		Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.sum.into(),
				arguments: vec![compr],
			},
		)
	}

	pub(in crate::lower) fn ensure_nested_occurrence_constructor_domain(
		&mut self,
		item: Item<'db>,
		occurrence: OccurrenceId,
		sum: Expression<'db>,
	) {
		if self.occurrence_constructors_available(occurrence) {
			return;
		}
		let one_expr = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		let local_range = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.dot_dot.into(),
				arguments: vec![one_expr, sum],
			},
		);
		let local_domain =
			Domain::bounded(self.db, item, VarType::Par, OptType::NonOpt, local_range);
		let local_decl = Declaration::new(false, local_domain);
		let local_idx = self
			.model
			.add_declaration(DeclarationItem::new(local_decl, item));
		self.ensure_occurrence_constructors(occurrence, local_idx);
	}

	pub(in crate::lower) fn nested_var_collection_fallback_cardinality(
		&mut self,
		owner_item: Item<'db>,
		declared_type: Option<shackle_hir::TypeId<'db>>,
		data: &shackle_hir::ItemData<'db>,
		types: &TypeResult<'db>,
	) -> Expression<'db> {
		// The cardinality bound is taken from the declared set type regardless
		// of whether the set inst is par or var: under a var-new parent, even
		// a par-set field is varified through the path, but its declared
		// cardinality still gives the per-parent child-count bound.
		let declared_type = declared_type
			.and_then(|declared_type| match &data[declared_type] {
				shackle_hir::Type::Set {
					cardinality: Some(cardinality),
					..
				} => Some(cardinality),
				_ => None,
			})
			.expect("nested var child collection missing cardinality bound");
		let mut nested_collector = ExpressionCollector::new(self, data, owner_item, types);
		let card_expr = nested_collector.collect_expression(*declared_type);
		Expression::new(
			self.db,
			&self.model,
			card_expr.origin(),
			LookupCall {
				function: self.ids.builtins.max.into(),
				arguments: vec![card_expr],
			},
		)
	}

	pub(in crate::lower) fn nested_par_collection_cardinality(
		&mut self,
		owner_item: Item<'db>,
		declared_type: Option<shackle_hir::TypeId<'db>>,
		data: &shackle_hir::ItemData<'db>,
		types: &TypeResult<'db>,
	) -> Option<Expression<'db>> {
		let cardinality = declared_type.and_then(|declared_type| match &data[declared_type] {
			shackle_hir::Type::Set {
				inst: VarType::Par,
				cardinality: Some(cardinality),
				..
			} => Some(*cardinality),
			_ => None,
		})?;
		let mut nested_collector = ExpressionCollector::new(self, data, owner_item, types);
		Some(nested_collector.collect_expression(cardinality))
	}

	pub(in crate::lower) fn emit_nested_cardinality_constraint(
		&mut self,
		item: Item<'db>,
		generators: Vec<Generator<'db>>,
		record_access: Expression<'db>,
		cardinality: Expression<'db>,
	) {
		// Pick `card(...)` for set-typed fields (nested `set of new <child>`
		// attributes) and `length(...)` for array-typed fields (the
		// `array of input-record` shape).
		let size_fn = if record_access.ty().is_set(self.db) {
			self.ids.functions.card
		} else {
			self.ids.builtins.length
		};
		let length_expr = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: size_fn.into(),
				arguments: vec![record_access],
			},
		);
		let membership = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.in_.into(),
				arguments: vec![length_expr, cardinality],
			},
		);
		let quantified = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![Expression::new(
					self.db,
					&self.model,
					item,
					ArrayComprehension::new(generators, membership),
				)],
			},
		);
		let constraint = Constraint::new(true, quantified);
		let _ = self
			.model
			.add_constraint(ConstraintItem::new(constraint, item));
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn nested_child_record_access_and_fallback_cardinality(
		&mut self,
		item: Item<'db>,
		prev_attrib: Expression<'db>,
		attrib: Identifier<'db>,
		local_domain_source: LocalDomainSource,
		declared_type: Option<shackle_hir::TypeId<'db>>,
		data: &shackle_hir::ItemData<'db>,
		owner_item: Item<'db>,
		types: &TypeResult<'db>,
	) -> (Option<Expression<'db>>, Option<Expression<'db>>) {
		let record_access = match prev_attrib.ty().lookup(self.db) {
			TyData::Record(_, fields) if fields.iter().any(|(field, _)| *field == attrib.0) => {
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(prev_attrib),
						field: attrib,
					},
				))
			}
			_ => None,
		};
		// Compute the static cardinality bound when we'd otherwise need to
		// take `card(...)` of a var-set field. Reading the size from a
		// `var set of <child>` (the identity shape) gives var int, but enum
		// sizing must be par; the declared cardinality (`max(0..n)`) supplies
		// that par bound. For array-of-input-record storage, the existing
		// `length(...)` path stays correct and no fallback is needed (and may
		// not exist — e.g. for a `set of new B` field without an explicit
		// cardinality bound).
		let needs_static_fallback = matches!(
			local_domain_source,
			LocalDomainSource::FlattenedChildCollection
		) && record_access
			.as_ref()
			.map(|ra| ra.ty().is_set(self.db))
			.unwrap_or(true);
		let fallback_cardinality = if needs_static_fallback {
			Some(self.nested_var_collection_fallback_cardinality(
				owner_item,
				declared_type,
				data,
				types,
			))
		} else {
			None
		};
		(record_access, fallback_cardinality)
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "nested reconstruction threads the full per-occurrence context"
	)]
	pub(in crate::lower) fn nested_path_generators_and_cursor(
		&mut self,
		item: Item<'db>,
		inputs_expr: &Expression<'db>,
		root_pattern: PatternRef<'db>,
		source_occurrence: OccurrenceId,
		attrib_path: &[Identifier<'db>],
		local_domain_source: LocalDomainSource,
		attrib_class_pattern_ref: PatternRef<'db>,
		declared_type: Option<shackle_hir::TypeId<'db>>,
		data: &shackle_hir::ItemData<'db>,
		types: &TypeResult<'db>,
	) -> (Vec<Generator<'db>>, Expression<'db>) {
		let mut toplevel_generator_decl = Declaration::new(
			false,
			Domain::unbounded(self.db, item, inputs_expr.ty().elem_ty(self.db).unwrap()),
		);
		toplevel_generator_decl.set_name(Identifier::new(self.db, "i"));
		let toplevel_generator_decl_idx = self
			.model
			.add_declaration(DeclarationItem::new(toplevel_generator_decl, item));
		let toplevel_generator_decl_expr =
			Expression::new(self.db, &self.model, item, toplevel_generator_decl_idx);
		let mut generators = vec![Generator::Iterator {
			declarations: vec![toplevel_generator_decl_idx],
			collection: inputs_expr.clone(),
			where_clause: None,
		}];

		let mut prev_attrib = toplevel_generator_decl_expr;
		for (idx, attrib) in attrib_path.iter().enumerate() {
			if !self.record_expr_has_field(&prev_attrib, *attrib) {
				if matches!(
					local_domain_source,
					LocalDomainSource::FlattenedChildCollection
				) && idx + 1 == attrib_path.len()
				{
					let fallback_cardinality = if declared_type.is_some() {
						self.nested_var_collection_fallback_cardinality(
							attrib_class_pattern_ref.item(self.db),
							declared_type,
							data,
							types,
						)
					} else {
						let source_contribution = self
							.occurrence_contribution(source_occurrence, attrib_class_pattern_ref);
						let source_decl = self
							.class_object_contribution_declaration(
								attrib_class_pattern_ref,
								source_contribution.constructor_index,
							)
							.expect(
								"source class contribution should exist before inherited projection sizing",
							);
						let source_decl_expr =
							Expression::new(self.db, &self.model, item, source_decl);
						Expression::new(
							self.db,
							&self.model,
							item,
							LookupCall {
								function: self.ids.builtins.length.into(),
								arguments: vec![source_decl_expr],
							},
						)
					};
					let range_expr = Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.dot_dot.into(),
							arguments: vec![
								Expression::new(self.db, &self.model, item, IntegerLiteral(1)),
								fallback_cardinality,
							],
						},
					);
					let mut attrib_generator_decl = Declaration::new(
						false,
						Domain::unbounded(self.db, item, Ty::par_int(self.db)),
					);
					attrib_generator_decl
						.set_name(Identifier::new(self.db, format!("j{}", idx + 1)));
					let attrib_generator_decl_idx = self
						.model
						.add_declaration(DeclarationItem::new(attrib_generator_decl, item));
					let attrib_generator_decl_expr =
						Expression::new(self.db, &self.model, item, attrib_generator_decl_idx);
					generators.push(Generator::Iterator {
						declarations: vec![attrib_generator_decl_idx],
						collection: range_expr,
						where_clause: None,
					});
					prev_attrib = attrib_generator_decl_expr;
					continue;
				}
				break;
			}

			let record_access = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(prev_attrib),
					field: *attrib,
				},
			);
			let Some(elem_ty) = record_access.ty().elem_ty(self.db) else {
				prev_attrib = record_access;
				continue;
			};

			let mut attrib_generator_decl =
				Declaration::new(false, Domain::unbounded(self.db, item, elem_ty));
			attrib_generator_decl.set_name(Identifier::new(self.db, format!("j{}", idx + 1)));
			let attrib_generator_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(attrib_generator_decl, item));
			let attrib_generator_decl_expr =
				Expression::new(self.db, &self.model, item, attrib_generator_decl_idx);

			generators.push(Generator::Iterator {
				declarations: vec![attrib_generator_decl_idx],
				collection: record_access,
				where_clause: None,
			});
			prev_attrib = attrib_generator_decl_expr;
		}

		let _ = root_pattern;
		(generators, prev_attrib)
	}

	pub(in crate::lower) fn nested_contribution_generators_and_input(
		&mut self,
		item: Item<'db>,
		local_domain_source: LocalDomainSource,
		generators: &[Generator<'db>],
		record_access: Option<Expression<'db>>,
	) -> (Vec<Generator<'db>>, Option<Expression<'db>>) {
		let mut contribution_generators = generators.to_vec();
		let maybe_contribution_input = match local_domain_source {
			LocalDomainSource::OnePerParent => match record_access.as_ref() {
				// Par-inlined nested storage: A's record holds `b: record(...)`
				// (the child's fields inlined). Projecting `(i).b` gives the
				// child record — the existing path is correct.
				Some(ra) if matches!(ra.ty().lookup(self.db), TyData::Record(_, _)) => {
					record_access
				}
				// Identity-typed nested storage: A's record holds
				// `b: var <child>_potential` (the child identity). Projecting
				// `(i).b` would produce an array of identities, not records,
				// for the child's `<C>_objects` storage. Return None so the
				// contribution registers as uninitialized var-record storage,
				// matching the bounded-collection (FlattenedChildCollection)
				// branch below.
				_ => None,
			},
			LocalDomainSource::FlattenedChildCollection => match record_access.clone() {
				Some(record_access) => {
					// Identity-set shape: the parent's field is a var-set
					// of child identities. Iterating it would produce
					// `var opt <child>` elements and an array-of-var-opt
					// contribution shape — wrong for child storage. Instead
					// fall back to no contribution input, so the contribution
					// is registered as uninitialized var-record storage.
					if record_access.ty().is_set(self.db) {
						None
					} else {
						let mut child_generator_decl = Declaration::new(
							false,
							Domain::unbounded(
								self.db,
								item,
								record_access.ty().elem_ty(self.db).unwrap(),
							),
						);
						child_generator_decl.set_name(Identifier::new(self.db, "k"));
						let child_generator_decl_idx = self
							.model
							.add_declaration(DeclarationItem::new(child_generator_decl, item));
						contribution_generators.push(Generator::Iterator {
							declarations: vec![child_generator_decl_idx],
							collection: record_access.clone(),
							where_clause: None,
						});
						Some(Expression::new(
							self.db,
							&self.model,
							item,
							child_generator_decl_idx,
						))
					}
				}
				None => None,
			},
			LocalDomainSource::SingleObject | LocalDomainSource::TopLevelCollection => {
				unreachable!("nested occurrence had unexpected root-only domain source")
			}
		};
		(contribution_generators, maybe_contribution_input)
	}

	pub(in crate::lower) fn add_class_objects_decl(
		&self,
		class_item: Item<'db>,
		class_objects_name: Identifier<'db>,
	) -> DeclarationItem<'db> {
		let class_record_ty = match class_item {
			Item::Class(sc) => {
				let fields = self.class_storage_fields_for_domain(PatternRef::new(
					self.db,
					class_item,
					sc.class(self.db).pattern,
				));
				Ty::array(self.db, Ty::par_int(self.db), Ty::record(self.db, fields)).unwrap()
			}
			_ => unreachable!(),
		};
		let mut class_objects_decl = Declaration::new(
			true,
			Domain::unbounded(self.db, class_item, class_record_ty),
		);
		class_objects_decl.set_name(class_objects_name);

		DeclarationItem::new(class_objects_decl, class_item)
	}

	/// Second predeclare phase: rebuild each `<C>_objects` declaration's
	/// storage-record domain now that every class is registered.
	///
	/// Classes are predeclared in topological item order, but class
	/// *reference* fields may form cycles (`Seat` ↔ `Handrail`) for which no
	/// order exists: `substitute_class_with_potential_enum` then leaves the
	/// not-yet-registered `Class<X>` fields of whichever class predeclares
	/// first unsubstituted. Rebuilding after all classes are registered (and
	/// before any item is collected, so no expression has frozen the stale
	/// type yet) makes the storage record independent of predeclare order.
	pub(in crate::lower) fn repair_predeclared_class_objects_domains(&mut self) {
		let entries = self
			.class_map
			.iter()
			.map(|(class_pattern, info)| (*class_pattern, info.class_objects))
			.collect::<Vec<_>>();
		for (class_pattern, class_objects) in entries {
			let fields = self.class_storage_fields_for_domain(class_pattern);
			let record_ty =
				Ty::array(self.db, Ty::par_int(self.db), Ty::record(self.db, fields)).unwrap();
			if self.model[class_objects].ty() == record_ty {
				continue;
			}
			let origin = class_pattern.item(self.db);
			self.model[class_objects].set_domain(Domain::unbounded(self.db, origin, record_ty));
		}
	}

	pub(in crate::lower) fn collect_class(&mut self, it: shackle_hir::ClassItem<'db>) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_pattern = PatternRef::new(self.db, item, c.pattern);
		self.predeclare_class(it);
		// Class-body constraints lowered over the realised class set: explicit
		// `constraint` items are emitted here. The defining equation of a
		// computed attribute (`forall(this in <C>)(this.<attr> = <rhs>)`) is
		// DEFERRED to `finish()` and only emitted for classes with a
		// contribution that does NOT alias-define its defined fields (the
		// gated forall-drop): once every contribution to the class is
		// engine-reconstructed, the equation holds by construction on
		// realised objects and the forall is redundant.
		// NB: class-level annotations (`c.annotations`) are intentionally not
		// lowered yet — a class has no single MiniZinc output construct to carry
		// them. They are still parsed and type-checked; emitting them is a
		// follow-up.
		for class_item in c.items.iter() {
			match class_item {
				ClassMember::Constraint(ct) => {
					let body = ClassBodyConstraint::Constraint {
						expression: ct.expression,
						annotations: ct.annotations.to_vec(),
					};
					self.emit_class_body_constraint(item, &body);
				}
				ClassMember::Declaration(d) => {
					if let Some(value) = d.definition
						&& let Some(attribute) = c.data()[d.pattern].identifier()
					{
						self.pending_class_definition_foralls.push((
							class_pattern,
							item,
							attribute,
							value,
						));
					}
				}
			}
		}
		self.emit_nested_set_cardinality_class_invariants(it, class_pattern);
		self.emit_relocated_domain_class_invariants(it, class_pattern);
		self.emit_dependent_domain_conformance_assertions(it);
	}

	/// Emit `forall(this in <C>)(assert(<conformance>, "..."))` for every
	/// PAR input-supplied field whose declared type carries an
	/// attribute-referencing domain (`1..l: x`, `array [1..l] of 0..hi: xs`).
	/// Those fields route through the unbounded storage-record path
	/// (`field_domain_references_attribute`), so without this assertion the
	/// supplied data is never checked against the dependent domain — a
	/// wrong-length array only errors if an out-of-range index is actually
	/// read, and an out-of-domain scalar passes silently. VAR fields need no
	/// assertion: their per-object domain is enforced by the reconstruction
	/// let-mint (`var 1..l: x` enumerates exactly `1..l`). Computed fields
	/// are alias-defined and skipped. Set-typed and multi-dimension shapes
	/// have no enforceable check here yet.
	pub(in crate::lower) fn emit_dependent_domain_conformance_assertions(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let types = item.types(self.db);
		let mut pending: Vec<ClassBodyConstraint<'db>> = Vec::new();
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			if d.definition.is_some() {
				continue;
			}
			let Some(attribute) = c.data()[d.pattern].identifier() else {
				continue;
			};
			if !self.field_domain_references_attribute(item, d.declared_type) {
				continue;
			}
			// Only par, non-opt fields: `assert` needs a par condition, and
			// var fields are already mint-enforced. (Var-reached classes
			// reject dependent domains at validation, so par-ness here is
			// the declared par-ness.)
			let field_ty = match &types[d.pattern] {
				PatternTy::Variable(ty) => *ty,
				_ => continue,
			};
			if !field_ty.known_par(self.db) || field_ty.opt(self.db) == Some(OptType::Opt) {
				continue;
			}
			// Only shapes with an enforceable check (see doc comment).
			let checkable = match &c.data()[d.declared_type] {
				shackle_hir::Type::Bounded { .. } => true,
				shackle_hir::Type::Array {
					dimensions,
					element,
					..
				} => {
					matches!(&c.data()[*dimensions], shackle_hir::Type::Bounded { .. })
						|| matches!(&c.data()[*element], shackle_hir::Type::Bounded { .. })
				}
				_ => false,
			};
			if !checkable {
				continue;
			}
			pending.push(ClassBodyConstraint::DomainConformance {
				attribute,
				declared_type: d.declared_type,
			});
		}
		for conformance in pending {
			self.emit_class_body_constraint(item, &conformance);
		}
	}

	/// Emit one class-body constraint quantified over the realised class set:
	/// `forall(this in <C>)(<body>)` — either an explicit class `constraint`
	/// expression or a computed attribute's defining equation
	/// `this.<attr> = <rhs>`. Bare attribute references resolve to per-object
	/// `<C>_objects` projections via let-bound field aliases. Definition
	/// bodies are emitted from `finish()` (see the gated forall-drop in
	/// `collect_class`), so this method derives everything from `item`.
	pub(in crate::lower) fn emit_class_body_constraint(
		&mut self,
		item: Item<'db>,
		class_body: &ClassBodyConstraint<'db>,
	) {
		let Item::Class(class_ref) = item else {
			unreachable!()
		};
		let c = class_ref.class(self.db);
		let class_pattern = PatternRef::new(self.db, item, c.pattern);
		let class_info = &self.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		let class_constraint_fields = self.class_constraint_fields(item);
		{
			let scan_exprs: Vec<shackle_hir::ExpressionId<'db>> = match class_body {
				ClassBodyConstraint::Constraint { expression, .. } => vec![*expression],
				ClassBodyConstraint::Definition { value, .. } => vec![*value],
				// The sibling references live in the declared type's domain
				// expressions (`array [1..l] of 0..hi`).
				ClassBodyConstraint::DomainConformance { declared_type, .. } => {
					shackle_hir::Type::walk(*declared_type, c.data())
						.filter_map(|t| match &c.data()[t] {
							shackle_hir::Type::Bounded { domain, .. } => Some(*domain),
							_ => None,
						})
						.collect()
				}
			};
			let this_ty = match &types[c.this_pattern] {
				PatternTy::Variable(ty) => *ty,
				_ => unreachable!(),
			};
			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let previous_resolution = self.resolutions.insert(
				PatternRef::new(self.db, item, c.this_pattern),
				LoweredIdentifier::ResolvedIdentifier(this_decl_idx.into()),
			);

			// Only fields actually used as bare identifiers (not `this.x`)
			// need a projection alias materialised — otherwise the Let
			// below would dump dead bindings into every class-constraint.
			let referenced_field_patterns: FxHashSet<PatternRef<'db>> = {
				let field_pattern_set: FxHashSet<PatternRef<'db>> =
					class_constraint_fields.iter().map(|(p, _)| *p).collect();
				let mut referenced = FxHashSet::default();
				for scan_expr in scan_exprs.iter().copied() {
					for sub in shackle_hir::Expression::walk(scan_expr, c.data()) {
						if let shackle_hir::Expression::Identifier(_) = &c.data()[sub]
							&& let Some(res) = types.name_resolution(sub)
							&& field_pattern_set.contains(&res)
						{
							let _ = referenced.insert(res);
						}
					}
				}
				referenced
			};

			let field_aliases = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				class_constraint_fields
					.iter()
					.filter(|(field_pattern, _)| referenced_field_patterns.contains(field_pattern))
					.map(|(field_pattern, field_name)| {
						let this_expr = alloc_expression(this_decl_idx, &collector, item);
						let class_objects_expr =
							alloc_expression(class_objects_idx, &collector, item);
						let object_index = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.enum2int.into(),
								arguments: vec![this_expr],
							},
							&collector,
							item,
						);
						let object_record =
							collector.collect_array_access(class_objects_expr, object_index, item);
						let field_expr = alloc_expression(
							RecordAccess {
								record: Box::new(object_record),
								field: *field_name,
							},
							&collector,
							item,
						);
						(*field_pattern, *field_name, field_expr)
					})
					.collect::<Vec<_>>()
			};
			let mut previous_field_resolutions = Vec::new();
			let mut alias_decl_idxs = Vec::new();
			for (field_pattern, field_name, field_expr) in field_aliases {
				let mut field_decl = Declaration::from_expression(self.db, false, field_expr);
				// Substitute `Class<X>` -> `X_potential` in the alias domain: a
				// var-reached class field projects as `var set of Class<X>` /
				// `var Class<X>`, which would render the (var) actual set as a
				// type-inst domain (`var set of Seat`), rejected by MiniZinc. The
				// potential enum is equivalent under `lowered_ty_matches`.
				let dom_origin = field_decl.domain().origin();
				let substituted = self.substitute_class_with_potential_enum(field_decl.ty());
				field_decl.set_domain(Domain::unbounded(self.db, dom_origin, substituted));
				field_decl.set_name(field_name);
				let field_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(field_decl, item));
				let old = self.resolutions.insert(
					field_pattern,
					LoweredIdentifier::ResolvedIdentifier(field_decl_idx.into()),
				);
				previous_field_resolutions.push((field_pattern, old));
				alias_decl_idxs.push(field_decl_idx);
			}
			let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
			let constraint_expr = match class_body {
				ClassBodyConstraint::Constraint { expression, .. } => {
					collector.collect_expression(*expression)
				}
				ClassBodyConstraint::Definition { attribute, value } => {
					// Build `this.<attr> = <definition>`. The LHS is the per-object
					// storage projection; the RHS is the collected definition, with
					// bare attribute references resolved to their projections via
					// the aliases set up above.
					let rhs = collector.collect_expression(*value);
					let this_expr = alloc_expression(this_decl_idx, &collector, item);
					let class_objects_expr = alloc_expression(class_objects_idx, &collector, item);
					let object_index = alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.enum2int.into(),
							arguments: vec![this_expr],
						},
						&collector,
						item,
					);
					let object_record =
						collector.collect_array_access(class_objects_expr, object_index, item);
					let lhs = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: *attribute,
						},
						&collector,
						item,
					);
					alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.eq.into(),
							arguments: vec![lhs, rhs],
						},
						&collector,
						item,
					)
				}
				ClassBodyConstraint::DomainConformance {
					attribute,
					declared_type,
				} => {
					// The per-object field projection (same shape as the
					// Definition LHS above).
					let this_expr = alloc_expression(this_decl_idx, &collector, item);
					let class_objects_expr = alloc_expression(class_objects_idx, &collector, item);
					let object_index = alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.enum2int.into(),
							arguments: vec![this_expr],
						},
						&collector,
						item,
					);
					let object_record =
						collector.collect_array_access(class_objects_expr, object_index, item);
					let field_proj = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: *attribute,
						},
						&collector,
						item,
					);
					// One check per enforceable declared-domain part: value
					// membership for a scalar `Bounded`, index-set equality
					// for a single-dimension array's `Bounded` dimension,
					// element membership for a `Bounded` array element (the
					// whole field domain went unbounded in storage, so even
					// non-dependent parts lost their enforcement).
					let mut checks: Vec<Expression<'db>> = Vec::new();
					match &c.data()[*declared_type] {
						shackle_hir::Type::Bounded { domain, .. } => {
							let dom = collector.collect_expression(*domain);
							checks.push(alloc_expression(
								LookupCall {
									function: collector.parent.ids.functions.in_.into(),
									arguments: vec![field_proj.clone(), dom],
								},
								&collector,
								item,
							));
						}
						shackle_hir::Type::Array {
							dimensions,
							element,
							..
						} => {
							if let shackle_hir::Type::Bounded { domain, .. } =
								&c.data()[*dimensions]
							{
								let dom = collector.collect_expression(*domain);
								let index_set_expr = alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.index_set.into(),
										arguments: vec![field_proj.clone()],
									},
									&collector,
									item,
								);
								checks.push(alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.eq.into(),
										arguments: vec![index_set_expr, dom],
									},
									&collector,
									item,
								));
							}
							if let shackle_hir::Type::Bounded { domain, .. } = &c.data()[*element] {
								let dom = collector.collect_expression(*domain);
								let elem_ty = field_proj
									.ty()
									.elem_ty(collector.parent.db)
									.expect("array field projection has an element type");
								let mut e_decl = Declaration::new(
									false,
									Domain::unbounded(collector.parent.db, item, elem_ty),
								);
								e_decl.set_name(Identifier::new(collector.parent.db, "e"));
								let e_decl_idx = collector
									.parent
									.model
									.add_declaration(DeclarationItem::new(e_decl, item));
								let e_expr = alloc_expression(e_decl_idx, &collector, item);
								let membership = alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.in_.into(),
										arguments: vec![e_expr, dom],
									},
									&collector,
									item,
								);
								let compr = alloc_expression(
									ArrayComprehension::new(
										[Generator::Iterator {
											declarations: vec![e_decl_idx],
											collection: field_proj.clone(),
											where_clause: None,
										}],
										membership,
									),
									&collector,
									item,
								);
								checks.push(alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.forall.into(),
										arguments: vec![compr],
									},
									&collector,
									item,
								));
							}
						}
						_ => {}
					}
					let mut checks = checks.into_iter();
					let mut cond = checks
						.next()
						.expect("conformance emission called for an uncheckable declared type");
					for check in checks {
						cond = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.and.into(),
								arguments: vec![cond, check],
							},
							&collector,
							item,
						);
					}
					let message = format!(
						"the value supplied for attribute `{}` of class `{}` does not agree \
						 with its declared type",
						attribute.pretty_print(collector.parent.db),
						class_pattern
							.identifier(collector.parent.db)
							.map(|i| i.lookup(collector.parent.db))
							.unwrap_or_default(),
					);
					let message_expr = alloc_expression(
						StringLiteral::new(collector.parent.db, message),
						&collector,
						item,
					);
					alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.assert_.into(),
							arguments: vec![cond, message_expr],
						},
						&collector,
						item,
					)
				}
			};
			// Bind the per-field projection aliases as let-items inside
			// the comprehension body so bare field references (e.g.
			// `constraint x >= 1` in a class body) are in MZN scope.
			// Without the Let, the aliases are model-resident but never
			// reachable from the lowered output.
			let body_expr = if alias_decl_idxs.is_empty() {
				constraint_expr
			} else {
				alloc_expression(
					Let {
						items: alias_decl_idxs
							.iter()
							.copied()
							.map(LetItem::Declaration)
							.collect(),
						in_expression: Box::new(constraint_expr),
					},
					&collector,
					item,
				)
			};
			let quantified = alloc_expression(
				LookupCall {
					function: collector.parent.ids.functions.forall.into(),
					arguments: vec![alloc_expression(
						ArrayComprehension::new(
							[Generator::Iterator {
								declarations: vec![this_decl_idx],
								collection: alloc_expression(class_set_idx, &collector, item),
								where_clause: None,
							}],
							body_expr,
						),
						&collector,
						item,
					)],
				},
				&collector,
				item,
			);
			let mut constraint = Constraint::new(true, quantified);
			let body_annotations: &[shackle_hir::ExpressionId<'db>] = match class_body {
				ClassBodyConstraint::Constraint { annotations, .. } => annotations,
				ClassBodyConstraint::Definition { .. }
				| ClassBodyConstraint::DomainConformance { .. } => &[],
			};
			constraint.annotations_mut().extend(
				body_annotations
					.iter()
					.map(|ann| collector.collect_expression(*ann)),
			);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(constraint, item));

			if let Some(old) = previous_resolution {
				let _ = self
					.resolutions
					.insert(PatternRef::new(self.db, item, c.this_pattern), old);
			} else {
				let _ = self
					.resolutions
					.remove(&PatternRef::new(self.db, item, c.this_pattern));
			}
			for (field_pattern, old) in previous_field_resolutions {
				if let Some(old) = old {
					let _ = self.resolutions.insert(field_pattern, old);
				} else {
					let _ = self.resolutions.remove(&field_pattern);
				}
			}
			let _ = this_ty;
		}
	}

	/// A nested `set(<card>) of new` field carries a cardinality bound
	/// that is otherwise only used for `<child>_potential` universe
	/// sizing. Emit it as an implicit class invariant
	/// `forall(this in <C>)(card(this.<field>) in <card>)` over the
	/// realised class set — iterating the actual set (not the potential
	/// storage) keeps unrealised potentials, whose field defaults to the
	/// empty set, out of the constraint. This covers var-declared fields
	/// (always) and par-declared fields of var-reached classes (whose
	/// storage-iterating walker emission is suppressed: it wrongly
	/// constrained unrealised potentials — a `var opt new` root's
	/// `absent(a)` was unsatisfiable — and was missing entirely on
	/// `var set of new` roots). Par-declared fields of par-reached classes
	/// keep the walker's `emit_nested_cardinality_constraint` emission,
	/// where every iterated instance is realised.
	pub(in crate::lower) fn emit_nested_set_cardinality_class_invariants(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
		class_pattern: PatternRef<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_info = &self.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			let shackle_hir::Type::Set {
				inst,
				cardinality: Some(cardinality),
				..
			} = &c.data()[d.declared_type]
			else {
				continue;
			};
			let Some(field_ident) = c.data()[d.pattern].identifier() else {
				continue;
			};
			if c.data()[d.declared_type].get_new_class(c.data()).is_some() {
				if *inst != VarType::Var
					&& !self
						.object_lowering
						.var_reached_classes
						.contains(&class_pattern)
				{
					continue;
				}
			} else {
				// Set-cardinality relocation: a non-`new` card-bounded set
				// field whose bound was dropped from the storage record domain
				// (`field_relocates_set_card`) — this realised-set invariant is
				// then the ONLY site enforcing the bound, keeping unrealised
				// slots (pinned/witnessed to `{}`) out of it. Non-relocated
				// fields keep their bound in the record domain and need no
				// invariant.
				let decl = StorageFieldDecl {
					ident: field_ident,
					pattern: PatternRef::new(self.db, item, d.pattern),
					definition: d.definition,
					declared_type: d.declared_type,
					owner: item,
				};
				if !self.field_relocates_set_card(&decl) {
					continue;
				}
			}
			let cardinality = *cardinality;
			let cardinality = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				collector.collect_expression(cardinality)
			};

			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let this_expr = Expression::new(self.db, &self.model, item, this_decl_idx);
			let class_objects_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_objects_idx),
			);
			let object_index = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.enum2int.into(),
					arguments: vec![this_expr],
				},
			);
			let object_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![class_objects_expr, object_index],
				},
			);
			let field_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(object_at_this),
					field: field_ident,
				},
			);
			let class_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_set_idx),
			);
			self.emit_nested_cardinality_constraint(
				item,
				vec![Generator::Iterator {
					declarations: vec![this_decl_idx],
					collection: class_set_expr,
					where_clause: None,
				}],
				field_at_this,
				cardinality,
			);
		}
	}

	/// Re-impose a relocated defined field's declared domain on realised
	/// objects only:
	/// `forall(this in <C>)(<C>_objects[enum2int(this)].<f> in <dom>)`.
	/// The domain was relaxed out of the shared element record
	/// (`build_class_storage_record_domain`), so this invariant is the ONLY
	/// site enforcing it — unrealised slots hold the (total) RHS value at
	/// their pinned frees, unconstrained, which is what keeps models like
	/// `var 3..4: z = x1 + x2` under `card(as) = 0` satisfiable. Emitted on
	/// the field's OWNER class: subclass objects are members of the owner's
	/// realised set, so one invariant covers every contribution.
	pub(in crate::lower) fn emit_relocated_domain_class_invariants(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
		class_pattern: PatternRef<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_info = &self.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			let Some(field_ident) = c.data()[d.pattern].identifier() else {
				continue;
			};
			let decl = StorageFieldDecl {
				ident: field_ident,
				pattern: PatternRef::new(self.db, item, d.pattern),
				definition: d.definition,
				declared_type: d.declared_type,
				owner: item,
			};
			if !self.field_relocates_declared_domain(&decl) {
				continue;
			}
			let shackle_hir::Type::Bounded { domain, .. } = &c.data()[d.declared_type] else {
				continue;
			};
			let domain_expr = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				collector.collect_expression(*domain)
			};

			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let this_expr = Expression::new(self.db, &self.model, item, this_decl_idx);
			let class_objects_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_objects_idx),
			);
			let object_index = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.enum2int.into(),
					arguments: vec![this_expr],
				},
			);
			let object_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![class_objects_expr, object_index],
				},
			);
			let field_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(object_at_this),
					field: field_ident,
				},
			);
			let membership = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.in_.into(),
					arguments: vec![field_at_this, domain_expr],
				},
			);
			let class_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_set_idx),
			);
			let quantified = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.forall.into(),
					arguments: vec![Expression::new(
						self.db,
						&self.model,
						item,
						ArrayComprehension::new(
							[Generator::Iterator {
								declarations: vec![this_decl_idx],
								collection: class_set_expr,
								where_clause: None,
							}],
							membership,
						),
					)],
				},
			);
			let constraint = Constraint::new(true, quantified);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(constraint, item));
		}
	}

	pub(in crate::lower) fn class_constraint_fields(
		&self,
		class_item: Item<'db>,
	) -> Vec<(PatternRef<'db>, Identifier<'db>)> {
		fn collect_fields<'db>(
			lowerer: &ItemCollector<'db>,
			class_item: Item<'db>,
			fields: &mut Vec<(PatternRef<'db>, Identifier<'db>)>,
		) {
			let Item::Class(class_ref) = class_item else {
				return;
			};
			let class = class_ref.class(lowerer.db);
			let types = class_item.types(lowerer.db);
			if let Some(base) = class.extends.and_then(|base| types.name_resolution(base)) {
				collect_fields(lowerer, base.item(lowerer.db), fields);
			}
			for field_item in class.items.iter() {
				if let ClassMember::Declaration(d) = field_item {
					for pattern in shackle_hir::Pattern::identifiers(d.pattern, class.data()) {
						let pattern_ref = PatternRef::new(lowerer.db, class_item, pattern);
						if let Some(identifier) = pattern_ref.identifier(lowerer.db) {
							fields.push((pattern_ref, identifier));
						}
					}
				}
			}
		}

		let mut fields = Vec::new();
		collect_fields(self, class_item, &mut fields);
		fields
	}

	/// Walk a class's storage-field declarations in storage order
	/// (superclass fields first, matching `class_storage_fields`), capturing
	/// for each field the HIR `Declaration` data the reconstruction
	/// comprehension needs: the field pattern (for sibling resolution), the
	/// optional RHS `definition` (a computed attribute), the `declared_type`
	/// (so a var field's per-object domain can be re-collected), and the
	/// owning item (a superclass item for inherited fields, so the RHS/domain
	/// is collected against the right `ItemData`).
	pub(in crate::lower) fn class_storage_field_decls(
		&self,
		class_item: Item<'db>,
	) -> Vec<StorageFieldDecl<'db>> {
		fn collect<'db>(
			lowerer: &ItemCollector<'db>,
			class_item: Item<'db>,
			out: &mut Vec<StorageFieldDecl<'db>>,
		) {
			let Item::Class(class_ref) = class_item else {
				return;
			};
			let class = class_ref.class(lowerer.db);
			let types = class_item.types(lowerer.db);
			if let Some(base) = class.extends.and_then(|base| types.name_resolution(base)) {
				collect(lowerer, base.item(lowerer.db), out);
			}
			for field_item in class.items.iter() {
				if let ClassMember::Declaration(d) = field_item {
					let pattern = PatternRef::new(lowerer.db, class_item, d.pattern);
					if let Some(ident) = pattern.identifier(lowerer.db) {
						out.push(StorageFieldDecl {
							ident,
							pattern,
							definition: d.definition,
							declared_type: d.declared_type,
							owner: class_item,
						});
					}
				}
			}
		}

		let mut fields = Vec::new();
		collect(self, class_item, &mut fields);
		fields
	}

	/// Lower a top-level declaration whose type introduces objects (`new C`,
	/// `var opt new C`, `set of new C`, `var set(c) of new C`, ...): mint the
	/// class enum constructors, the inputs/storage arrays, the contribution
	/// `_objects` arrays and slice arrays, and return the declaration for the
	/// user-named identity (set).
	pub(in crate::lower) fn collect_new_declaration(
		&mut self,
		ty: Ty<'db>,
		types: &TypeResult<'db>,
		item: Item<'db>,
		d: &shackle_hir::Declaration<'db>,
		data: &shackle_hir::ItemData<'db>,
		top_level: bool,
	) -> Declaration<'db> {
		let mut collector = ExpressionCollector::new(self, data, item, types);

		let class_domain = data[d.declared_type]
			.get_new_class(data)
			.expect("new declarations should have a class domain");

		let class_pattern_ref = types.name_resolution(class_domain).unwrap();
		let class_enum = collector.parent.class_map[&class_pattern_ref].class_enum;
		let root_pattern = PatternRef::new(collector.parent.db, item, d.pattern);
		let root_occurrence = collector.parent.top_level_occurrence(root_pattern);

		let enum_member_id = EnumMemberId::new(
			class_enum,
			collector.parent.object_lowering.contributions_by_occurrence[&root_occurrence][0]
				.constructor_index as u32,
		);
		let item_ty: &shackle_hir::Type = &collector.data[d.declared_type];

		let class_and_decl_name = format!(
			"{}_{}",
			class_pattern_ref
				.identifier(collector.parent.db)
				.unwrap()
				.pretty_print(collector.parent.db),
			collector.data[d.pattern]
				.identifier()
				.unwrap()
				.pretty_print(collector.parent.db)
		);

		#[derive(Clone)]
		struct PendingRootCollection<'db> {
			contribution_index: usize,
			inst: VarType,
			emit_root_contribution: bool,
			contribution_expr: Option<Expression<'db>>,
			nested_iteration_expr: Option<Expression<'db>>,
			needs_reconstruction: bool,
			sum_expr: Expression<'db>,
			potential_ordinal_domain: Option<Expression<'db>>,
		}

		let (mut domain_decl, inputs_expr, pending_root_collection) = match item_ty {
			shackle_hir::Type::New { inst, opt, .. } => {
				let class_types = class_pattern_ref
					.item(collector.parent.db)
					.types(collector.parent.db);
				let (input_record_ty, _storage_record_ty) =
					match &class_types[class_pattern_ref.pattern(collector.parent.db)] {
						PatternTy::ClassDecl {
							input_record_ty,
							storage_record_ty,
							..
						} => (*input_record_ty, *storage_record_ty),
						_ => unreachable!(),
					};

				let one_expr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					IntegerLiteral(1),
				);
				let singleton_ordinal_set =
					alloc_expression(SetLiteral(vec![one_expr.clone()]), &collector, item);
				let root_fields = collector.parent.class_storage_fields(class_pattern_ref);

				let storage_backed_var_root = *inst == VarType::Var && d.definition.is_none();
				let inputs_expr = if storage_backed_var_root {
					// Include every storage field directly — including class-typed
					// and set-of-class fields. Under a var-new root, the
					// var-attribute storage rule makes every field a free
					// decision (the var-set-of-class field is bounded by the
					// per-parent slice constraint, emitted separately). Substitute
					// class types with the child class's potential enum to avoid
					// a circular type definition
					// (see `substitute_class_with_potential_enum`).
					let root_storage_record_ty = Ty::record(
						collector.parent.db,
						root_fields
							.iter()
							.map(|(name, ty)| {
								(
									*name,
									collector.parent.substitute_class_with_potential_enum(*ty),
								)
							})
							.collect::<Vec<_>>(),
					);
					// Computed and domain-dependent fields are aliases defined in
					// the reconstruction comprehension, not free `_storage`
					// decisions — drop them from the free element type (also the
					// only valid form for `array`/unbounded-`var set` computed
					// fields, which can't be free decisions at all).
					let root_storage_record_ty = collector
						.parent
						.free_storage_record_ty(class_pattern_ref, root_storage_record_ty);
					let storage_elem_ty = root_storage_record_ty
						.with_inst(collector.parent.db, VarType::Var)
						.unwrap_or(root_storage_record_ty);
					let storage_elem_dom = collector.parent.build_class_storage_record_domain(
						class_pattern_ref,
						storage_elem_ty,
						item,
					);
					let storage_array_dom = Domain::array(
						collector.parent.db,
						item,
						OptType::NonOpt,
						Domain::bounded(
							collector.parent.db,
							item,
							VarType::Par,
							OptType::NonOpt,
							singleton_ordinal_set.clone(),
						),
						storage_elem_dom,
					);
					let mut storage_decl = Declaration::new(true, storage_array_dom);
					storage_decl.set_name(Identifier::new(
						collector.parent.db,
						format!("{}_storage", class_and_decl_name),
					));
					let storage_idx = collector
						.parent
						.model
						.add_declaration(DeclarationItem::new(storage_decl, item));
					alloc_expression(storage_idx, &collector, item)
				} else {
					// An `opt new C` attribute yields an `opt record` input
					// slot, which MiniZinc rejects. Lower the slot (and the value)
					// to a non-opt 0/1-length list; other classes are untouched.
					let opt_new_input = collector
						.parent
						.input_ty_needs_opt_new_lowering(input_record_ty);
					let lowered_input_record_ty = if opt_new_input {
						collector.parent.lower_opt_new_input_ty(input_record_ty)
					} else {
						input_record_ty
					};
					let array_dom = Domain::array(
						collector.parent.db,
						item,
						OptType::NonOpt,
						Domain::unbounded(
							collector.parent.db,
							item,
							Ty::par_int(collector.parent.db),
						),
						Domain::unbounded(collector.parent.db, item, lowered_input_record_ty),
					);
					let mut array_decl = Declaration::new(true, array_dom);
					let inputs_name = format!("{}_inputs", class_and_decl_name);
					array_decl.set_name(Identifier::new(collector.parent.db, inputs_name));
					let input_value = if let Some(rhs) = d.definition {
						let collected = collector.collect_expression(rhs);
						if opt_new_input {
							collector.parent.lower_opt_new_input_value(
								item,
								collected,
								input_record_ty,
							)
						} else {
							collected
						}
					} else {
						alloc_expression(DummyValue(lowered_input_record_ty), &collector, item)
					};
					array_decl.set_definition(alloc_expression(
						ArrayLiteral(vec![input_value]),
						&collector,
						item,
					));

					let inputs_idx = collector
						.parent
						.model
						.add_declaration(DeclarationItem::new(array_decl, item));
					alloc_expression(inputs_idx, &collector, item)
				};
				let enum_constr_domain = Domain::bounded(
					collector.parent.db,
					item,
					VarType::Par,
					OptType::NonOpt,
					singleton_ordinal_set.clone(),
				);
				let decl = Declaration::new(false, enum_constr_domain);
				let idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(decl, item));
				collector
					.parent
					.add_occurrence_constructors(root_occurrence, idx);

				let call_expr = alloc_expression(
					Call {
						function: Callable::EnumConstructor(enum_member_id),
						arguments: vec![one_expr],
					},
					&collector,
					item,
				);
				let root_contributions =
					collector.parent.object_lowering.contributions_by_occurrence[&root_occurrence]
						.clone();
				if *opt == OptType::Opt {
					// `var opt new C: x` is an optional occurrence. The direct
					// class's actual set is a free `var set of <C>_potential`
					// decision (its members are `{}` or the lone potential
					// identity); the identity `x` is lowered to a `var opt
					// <C>_potential` defined as present iff that potential is
					// realised. Superclass actual sets are derived from the same
					// occurrence test. The direct class and its superclasses
					// were already predeclared `var set` via
					// `var_actual_set_classes` (a `var opt new` introduces var
					// existence), so no widening is needed here.
					debug_assert!(
						root_contributions.iter().all(|contribution| {
							collector.parent.model
								[collector.parent.class_map[&contribution.target_class].class_set]
								.ty()
								.inst(collector.parent.db)
								== Some(VarType::Var)
						}),
						"var opt new occurrence has a par actual-set declaration \
						 among its contributions; var_actual_set_classes is too \
						 narrow"
					);
					let direct_class_set = collector.parent.class_map[&class_pattern_ref].class_set;
					let direct_set_expr = alloc_expression(direct_class_set, &collector, item);
					// `<C>_occ_0(1) in <C>` — true exactly when the optional
					// occurrence is realised.
					let occurs_expr = alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.in_.into(),
							arguments: vec![call_expr.clone(), direct_set_expr],
						},
						&collector,
						item,
					);
					for contribution in &root_contributions {
						// Every opt-root contribution (direct AND superclass)
						// is skipped by `finish`'s definitional/lower-bound union —
						// its membership is the free decision — and every reached
						// class emits its actual set FREE + subset lower bound
						// rather than an `=` union (so a co-occurring definite root
						// isn't clobbered).
						let _ = collector
							.parent
							.opt_contribution_slots
							.insert((contribution.target_class, contribution.constructor_index));
						let _ = collector
							.parent
							.opt_free_subset_classes
							.insert(contribution.target_class);
						if contribution.target_class == class_pattern_ref {
							// The direct class set stays a free decision: the opt
							// occurrence's identity `x` (below) is present iff its
							// lone potential is in the direct class set, so
							// membership IS the presence decision — no constraint.
							continue;
						}
						// A superclass image: `x present` (the direct occurrence
						// test) must be MIRRORED by the projected identity's
						// membership in the superclass set. Eagerly DEFINING
						// `<super> = if occurs then {img} else {}` would clobber
						// any co-occurring definite root's members. Emit a
						// biconditional constraint instead and leave `<super>`
						// free (bounded by `<super>_potential`, lower-bounded by
						// the definite roots in `finish`):
						//   `(<super>_occ_k(1) in <super>) <-> (x realised)`.
						let target_enum =
							collector.parent.class_map[&contribution.target_class].class_enum;
						let target_enum_member =
							EnumMemberId::new(target_enum, contribution.constructor_index as u32);
						let one_expr = alloc_expression(IntegerLiteral(1), &collector, item);
						let image_ident = alloc_expression(
							Call {
								function: Callable::EnumConstructor(target_enum_member),
								arguments: vec![one_expr],
							},
							&collector,
							item,
						);
						let class_set_decl =
							collector.parent.class_map[&contribution.target_class].class_set;
						let class_set_expr = alloc_expression(class_set_decl, &collector, item);
						let image_in_super = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.in_.into(),
								arguments: vec![image_ident, class_set_expr],
							},
							&collector,
							item,
						);
						let biconditional = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.iff.into(),
								arguments: vec![image_in_super, occurs_expr.clone()],
							},
							&collector,
							item,
						);
						let _ = collector.parent.model.add_constraint(ConstraintItem::new(
							Constraint::new(top_level, biconditional),
							item,
						));
					}
					// `var opt new C: x` lowers to a `var opt <C>_potential: x`
					// identity that is present iff its lone potential is in the
					// (free) direct class set.
					let domain_ty = collector.parent.substitute_class_with_potential_enum(ty);
					let mut domain_decl = Declaration::new(
						top_level,
						Domain::unbounded(collector.parent.db, item, domain_ty),
					);
					// Whether ANOTHER introduction also reaches the direct
					// class — i.e. `<C>_potential` has more than the opt root's own
					// constructor. Only then does the naive defining form below
					// (`x = if occurs then <C>_occ_k(1) else <>`) misbehave: with a
					// multi-constructor enum MiniZinc flags a model inconsistency
					// and drops the absent branch, silently forcing the optional
					// occurrence present. The unmixed case keeps the byte-identical
					// defining form (its enum is single-member).
					let direct_is_mixed = collector
						.parent
						.object_lowering
						.contributions_by_occurrence
						.iter()
						.filter(|(occ, _)| **occ != root_occurrence)
						.flat_map(|(_, cs)| cs.iter())
						.any(|c| c.target_class == class_pattern_ref);
					let identity_def = if direct_is_mixed {
						// Constraint-decomposed identity, wrapped in a `let` so `x`
						// stays a defined declaration:
						//   let { var opt <C>_potential: t;
						//         constraint occurs(t) <-> <occurs>;
						//         constraint occurs(t) -> deopt(t) = <C>_occ_k(1);
						//   } in t
						let t_decl = Declaration::new(
							false,
							Domain::unbounded(collector.parent.db, item, domain_ty),
						);
						let t_idx = collector
							.parent
							.model
							.add_declaration(DeclarationItem::new(t_decl, item));
						let t_expr = alloc_expression(t_idx, &collector, item);
						let occurs_t = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.occurs.into(),
								arguments: vec![t_expr.clone()],
							},
							&collector,
							item,
						);
						let presence = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.iff.into(),
								arguments: vec![occurs_t, occurs_expr],
							},
							&collector,
							item,
						);
						let presence_id = collector.parent.model.add_constraint(
							ConstraintItem::new(Constraint::new(false, presence), item),
						);
						let occurs_t2 = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.occurs.into(),
								arguments: vec![t_expr.clone()],
							},
							&collector,
							item,
						);
						let deopt_t = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.deopt.into(),
								arguments: vec![t_expr.clone()],
							},
							&collector,
							item,
						);
						let value_eq = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.eq.into(),
								arguments: vec![deopt_t, call_expr],
							},
							&collector,
							item,
						);
						let value_imp = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.implies.into(),
								arguments: vec![occurs_t2, value_eq],
							},
							&collector,
							item,
						);
						let value_id = collector.parent.model.add_constraint(ConstraintItem::new(
							Constraint::new(false, value_imp),
							item,
						));
						alloc_expression(
							Let {
								items: vec![
									LetItem::Declaration(t_idx),
									LetItem::Constraint(presence_id),
									LetItem::Constraint(value_id),
								],
								in_expression: Box::new(t_expr),
							},
							&collector,
							item,
						)
					} else {
						let absent = alloc_expression(Absent, &collector, item);
						alloc_expression(
							IfThenElse {
								branches: vec![Branch::new(occurs_expr, call_expr)],
								else_result: Box::new(absent),
							},
							&collector,
							item,
						)
					};
					domain_decl.set_definition(identity_def);
					(domain_decl, inputs_expr, None)
				} else {
					// A non-opt singular root's identity always exists, so each
					// contribution is the static singleton `<T>_occ_k({1})`.
					// Register it through the SAME channel collection roots use
					// (`class_set_top_level_contributions`, unioned in `finish`)
					// instead of eagerly defining the class-set decl here: the
					// eager definition made `finish`'s union loop skip the class
					// (`definition().is_some()`), silently DROPPING any
					// collection root's registered contribution — a par
					// `new A: a` plus `var set(..) of new A: as` defined `A`
					// as just `A_occ_0({1})`, forcing `as`'s members out of
					// existence.
					for contribution in root_contributions {
						let target_enum =
							collector.parent.class_map[&contribution.target_class].class_enum;
						let target_enum_member =
							EnumMemberId::new(target_enum, contribution.constructor_index as u32);
						let class_set_definition = alloc_expression(
							Call {
								function: Callable::EnumConstructor(target_enum_member),
								arguments: vec![singleton_ordinal_set.clone()],
							},
							&collector,
							item,
						);
						collector.parent.register_class_set_top_level_contribution(
							contribution.target_class,
							contribution.constructor_index,
							class_set_definition,
						);
					}
					// `var new C: x` produces HIR type `var Class<C>`, which encodes
					// "attributes reached through x are var-storage". The lowered
					// identity itself is par because the singular fresh root has a
					// fixed identity (one potential, must-pick); var storage is
					// emitted separately. Par-ify the declaration domain and bind
					// it by the potential universe (`C_potential: x = …`) —
					// membership in the actual set is definitional through the
					// class-set union, and the class name itself can be a var set
					// (when the class's existence is a decision elsewhere), which
					// cannot serve as a declaration domain.
					let domain_ty = collector
						.parent
						.substitute_class_with_potential_enum(ty)
						.make_par(collector.parent.db);
					let mut domain_decl = Declaration::new(
						top_level,
						Domain::unbounded(collector.parent.db, item, domain_ty),
					);
					domain_decl.set_definition(call_expr);
					(domain_decl, inputs_expr, None)
				}
			}
			shackle_hir::Type::Array { .. } => {
				// Array-of-new roots are rejected in `validate_root_decl`: an
				// `array [d] of new C` conflates object identity with the array
				// index and adds no expressivity over `set of new C`, so it is
				// disallowed. This arm is therefore never reached.
				unreachable!(
					"array-of-new root reached THIR lowering; it must be rejected \
					 in validate_root_decl"
				)
			}
			shackle_hir::Type::Set {
				cardinality, inst, ..
			} => {
				let class_types = class_pattern_ref
					.item(collector.parent.db)
					.types(collector.parent.db);
				let (input_record_ty, storage_record_ty) =
					match &class_types[class_pattern_ref.pattern(collector.parent.db)] {
						PatternTy::ClassDecl {
							input_record_ty,
							storage_record_ty,
							..
						} => (*input_record_ty, *storage_record_ty),
						_ => unreachable!(),
					};
				let needs_reconstruction = input_record_ty != storage_record_ty;
				let root_fields = collector.parent.class_storage_fields(class_pattern_ref);
				let has_object_fields = root_fields.iter().any(|(_, field_ty)| {
					field_ty
						.walk(collector.parent.db)
						.any(|nested_ty| nested_ty.class_type(collector.parent.db).is_some())
				});
				let scalar_storage_only_var_root = *inst == VarType::Var
					&& d.definition.is_none()
					&& root_fields.iter().all(|(_, field_ty)| {
						field_ty
							.walk(collector.parent.db)
							.all(|nested_ty| nested_ty.class_type(collector.parent.db).is_none())
					});
				let object_storage_backed_var_root =
					*inst == VarType::Var && d.definition.is_none() && has_object_fields;

				// Compute the per-introduction potential ordinal domain (e.g.
				// `1..max(c)` for `var set(c) of new C`). Used as the index
				// domain for both the inputs and storage arrays below.
				let potential_ordinal_domain = match (cardinality, inst) {
					(Some(c), VarType::Var) => {
						let card_expr = collector.collect_expression(*c);
						let upper_bound = match &data[*c] {
							shackle_hir::Expression::Call(call)
								if call.arguments.len() == 2
									&& matches!(&data[call.function], shackle_hir::Expression::Identifier(identifier) if *identifier == collector.parent.ids.functions.dot_dot) =>
							{
								collector.collect_expression(call.arguments[1])
							}
							_ => Expression::new(
								collector.parent.db,
								&collector.parent.model,
								item,
								LookupCall {
									function: collector.parent.ids.builtins.max.into(),
									arguments: vec![card_expr],
								},
							),
						};
						let one_expr = Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							IntegerLiteral(1),
						);
						Some(Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							LookupCall {
								function: collector.parent.ids.functions.dot_dot.into(),
								arguments: vec![one_expr, upper_bound],
							},
						))
					}
					(_, VarType::Par) => None,
					_ => unreachable!(),
				};

				let build_index_domain = |potential_ordinal_domain: &Option<Expression<'db>>| {
					if let Some(potential_ordinal_domain) = potential_ordinal_domain {
						Domain::bounded(
							collector.parent.db,
							item,
							VarType::Par,
							OptType::NonOpt,
							potential_ordinal_domain.clone(),
						)
					} else {
						Domain::unbounded(
							collector.parent.db,
							item,
							Ty::par_int(collector.parent.db),
						)
					}
				};

				// When the root is var with object-typed fields and has no
				// RHS, the inputs array would be a phantom: its element type
				// collapses to the empty record (all class fields are
				// constructed via `new`), it has no consumers downstream, and
				// MiniZinc rejects the missing initializer on the empty-record
				// array. Skip the inputs decl entirely in that case and route
				// `inputs_expr` to the `_storage` array we create below.
				// A par `opt new C` field yields an `opt record` input
				// slot, which MiniZinc rejects. Lower the slot type (and any
				// inline value) to a non-opt 0/1-length list, exactly as the
				// singular-root site does — so a `set of new` root whose member
				// carries an optional child (inline OR via `.dzn`) reconstructs
				// through the opt-aware `length(input.f) > 0` read-back instead
				// of panicking on `length(opt record)`. Gated on
				// `input_ty_needs_opt_new_lowering`, so non-opt-new roots and
				// var-reached owners (where the opt-new field is a free decision,
				// never in the input record) are byte-identical.
				let set_root_opt_new_input = collector
					.parent
					.input_ty_needs_opt_new_lowering(input_record_ty);
				let maybe_inputs_idx = if object_storage_backed_var_root {
					None
				} else {
					let elem_ty = if scalar_storage_only_var_root {
						// Free-storage element type excludes computed /
						// domain-dependent fields (defined as reconstruction
						// aliases instead), matching the singular `var new` path.
						let free_storage_record_ty = collector
							.parent
							.free_storage_record_ty(class_pattern_ref, storage_record_ty);
						free_storage_record_ty
							.with_inst(collector.parent.db, VarType::Var)
							.unwrap_or(free_storage_record_ty)
					} else if set_root_opt_new_input {
						collector.parent.lower_opt_new_input_ty(input_record_ty)
					} else {
						input_record_ty
					};
					let elem_dom = if scalar_storage_only_var_root {
						collector.parent.build_class_storage_record_domain(
							class_pattern_ref,
							elem_ty,
							item,
						)
					} else {
						Domain::unbounded(collector.parent.db, item, elem_ty)
					};
					let array_dom = Domain::array(
						collector.parent.db,
						item,
						OptType::NonOpt,
						build_index_domain(&potential_ordinal_domain),
						elem_dom,
					);
					let mut array_decl = Declaration::new(true, array_dom);
					let array_name = if scalar_storage_only_var_root {
						format!("{}_storage", class_and_decl_name)
					} else {
						format!("{}_inputs", class_and_decl_name)
					};
					array_decl.set_name(Identifier::new(collector.parent.db, array_name));
					if matches!(inst, VarType::Par)
						&& let Some(rhs) = d.definition
					{
						let inputs = collector.collect_expression(rhs);
						let inputs = if set_root_opt_new_input {
							collector.parent.lower_opt_new_input_collection_value(
								item,
								inputs,
								input_record_ty,
							)
						} else {
							inputs
						};
						array_decl.set_definition(inputs);
					}
					Some(
						collector
							.parent
							.model
							.add_declaration(DeclarationItem::new(array_decl, item)),
					)
				};

				let storage_idx = if object_storage_backed_var_root {
					// Computed / domain-dependent fields are reconstruction
					// aliases, not free `_storage` decisions — drop them from the
					// free element type (keeping class-typed fields, which stay
					// free decisions bounded by the per-parent slice constraint).
					let free_storage_record_ty = collector
						.parent
						.free_storage_record_ty(class_pattern_ref, storage_record_ty);
					let varified = free_storage_record_ty
						.with_inst(collector.parent.db, VarType::Var)
						.unwrap_or(free_storage_record_ty);
					// Substitute class types with their potential enums so the
					// storage record doesn't reference the (derived) class set
					// — see `substitute_class_with_potential_enum`.
					let storage_elem_ty = collector
						.parent
						.substitute_class_with_potential_enum(varified);
					let storage_elem_dom = collector.parent.build_class_storage_record_domain(
						class_pattern_ref,
						storage_elem_ty,
						item,
					);
					let storage_domain = Domain::array(
						collector.parent.db,
						item,
						OptType::NonOpt,
						build_index_domain(&potential_ordinal_domain),
						storage_elem_dom,
					);
					let mut storage_decl = Declaration::new(true, storage_domain);
					storage_decl.set_name(Identifier::new(
						collector.parent.db,
						format!("{}_storage", class_and_decl_name),
					));
					Some(
						collector
							.parent
							.model
							.add_declaration(DeclarationItem::new(storage_decl, item)),
					)
				} else {
					None
				};

				let inputs_expr = match (maybe_inputs_idx, storage_idx) {
					(Some(idx), _) => alloc_expression(idx, &collector, item),
					(None, Some(idx)) => alloc_expression(idx, &collector, item),
					(None, None) => {
						unreachable!("expected inputs or storage declaration")
					}
				};
				let contribution_expr =
					storage_idx.map(|idx| alloc_expression(idx, &collector, item));
				let nested_iteration_expr = contribution_expr.as_ref().map(|storage_expr| {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						LookupCall {
							function: collector.parent.ids.functions.index_set.into(),
							arguments: vec![storage_expr.clone()],
						},
					)
				});
				let sum_expr = if let Some(rhs) = d.definition {
					match &data[rhs] {
						shackle_hir::Expression::ArrayLiteral(array_literal) => Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							IntegerLiteral(array_literal.members.len() as i64),
						),
						_ => Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							LookupCall {
								function: collector.parent.ids.builtins.length.into(),
								arguments: vec![inputs_expr.clone()],
							},
						),
					}
				} else {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						LookupCall {
							function: collector.parent.ids.builtins.length.into(),
							arguments: vec![inputs_expr.clone()],
						},
					)
				};
				let contribution_index = collector
					.parent
					.occurrence_contribution(root_occurrence, class_pattern_ref)
					.constructor_index;
				(
					Declaration::new(top_level, Domain::unbounded(collector.parent.db, item, ty)),
					inputs_expr,
					Some(PendingRootCollection {
						contribution_index,
						inst: *inst,
						emit_root_contribution: *inst != VarType::Var
							|| scalar_storage_only_var_root
							|| object_storage_backed_var_root,
						contribution_expr,
						nested_iteration_expr,
						needs_reconstruction,
						sum_expr,
						potential_ordinal_domain,
					}),
				)
			}
			_ => todo!("Handle other cases of new A: x"),
		};

		struct PendingSlice<'db> {
			target_class: PatternRef<'db>,
			contribution_index: usize,
			name_suffix: String,
			sum_expr: Expression<'db>,
			/// Per-parent slice size for nested fresh-child collections.
			/// Set when the slice corresponds to a `set of new <child>: <field>`
			/// attribute on a parent that has more than one potential identity
			/// (or even just one). Used to emit the per-parent slice array
			/// `<parent>_<field>_potential`.
			max_card_per_parent: Option<Expression<'db>>,
			/// Immediate parent class for this slice's `<field>` attribute. For
			/// a nested-of-nested slice (e.g. `Vehicle.crew` introduced via
			/// `Expedition.vehicles`), this is the *immediate* parent (Vehicle),
			/// not the root user decl class (Expedition). The slice-emission
			/// loop uses `class_object_contribution_declaration(parent_class,
			/// parent_contribution_index)` to find the parent's contribution
			/// storage array, which becomes the slice array's index domain so
			/// `slice[v]` lines up with `parent_storage[v].<field>` in the
			/// per-parent subset constraint.
			parent_class: PatternRef<'db>,
			parent_contribution_index: usize,
			/// The immediate `<field>` attribute name on `parent_class` (the
			/// last path component), unlike `name_suffix` which joins the full
			/// path from the root (`ps_ns`). This is what the actual-set
			/// field-introduction record needs: the field is a direct record
			/// field of the *parent's* storage element type, not the root's.
			/// `None` for the root passthrough slice (no introducing field).
			field_attribute: Option<Identifier<'db>>,
			/// `Some(opt)` when the field is a singular `new`/`opt new`
			/// attribute (`LocalDomainSource::OnePerParent`), recording its
			/// opt-ness for the actual-set singleton contribution. `None` for
			/// collection-shaped fields and the root passthrough slice.
			singular_opt: Option<bool>,
		}

		let mut pending_slices = Vec::new();
		let nested_iteration_expr = pending_root_collection
			.as_ref()
			.and_then(|root_collection| root_collection.nested_iteration_expr.clone())
			.unwrap_or_else(|| inputs_expr.clone());
		if let Some(root_collection) = &pending_root_collection
			&& root_collection.emit_root_contribution
			&& root_collection.inst != VarType::Var
		{
			pending_slices.push(PendingSlice {
				target_class: class_pattern_ref,
				contribution_index: root_collection.contribution_index,
				name_suffix: "root".to_owned(),
				sum_expr: root_collection.sum_expr.clone(),
				max_card_per_parent: None,
				parent_class: class_pattern_ref,
				parent_contribution_index: root_collection.contribution_index,
				field_attribute: None,
				singular_opt: None,
			});
		}

		// Singular roots (`new C: x`, `var new C: x`, `var opt new C: x`)
		// have a one-slot potential block. Any LATER contribution to the same
		// class needs its predecessor's end offset in `contribution_end_map` —
		// a subclass root's projection passthrough slice, or
		// `project_class_identity` on a later constructor — so chain a
		// one-slot root passthrough exactly like collection roots do, gated on
		// a later contribution existing so single-root models don't grow
		// unused `_root_start`/`_root_end` declarations.
		if pending_root_collection.is_none() && matches!(item_ty, shackle_hir::Type::New { .. }) {
			let contribution_index = collector
				.parent
				.occurrence_contribution(root_occurrence, class_pattern_ref)
				.constructor_index;
			let has_later_contribution = collector
				.parent
				.object_lowering
				.contributions_by_occurrence
				.values()
				.flatten()
				.any(|contribution| {
					contribution.target_class == class_pattern_ref
						&& contribution.constructor_index > contribution_index
				});
			if has_later_contribution {
				pending_slices.push(PendingSlice {
					target_class: class_pattern_ref,
					contribution_index,
					name_suffix: "root".to_owned(),
					sum_expr: Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						IntegerLiteral(1),
					),
					max_card_per_parent: None,
					parent_class: class_pattern_ref,
					parent_contribution_index: contribution_index,
					field_attribute: None,
					singular_opt: None,
				});
			}
		}

		// Calculate the sizes of the different objects arrays based on the contained classes

		let mut constructor_stack = vec![(vec![], class_pattern_ref)];

		while let Some((attrib_path, attrib_class_pattern_ref)) = constructor_stack.pop() {
			let mut child_classes = vec![];
			let Item::Class(class_item_ref) = attrib_class_pattern_ref.item(collector.parent.db)
			else {
				unreachable!()
			};
			let class_hir = class_item_ref.class(collector.parent.db);
			let class_item_types = attrib_class_pattern_ref
				.item(collector.parent.db)
				.types(collector.parent.db);
			let class_item_data = class_hir.data();
			if let Some(c) = class_hir.extends {
				let child_pattern = class_item_types.name_resolution(c).unwrap();
				child_classes.push((None, child_pattern, None));
			}
			for member in class_hir.items.iter() {
				if let ClassMember::Declaration(d) = member
					&& let Some(c) = class_item_data[d.declared_type].get_new_class(class_item_data)
				{
					let child_pattern = class_item_types.name_resolution(c).unwrap();
					child_classes.push((
						Some(class_item_data[d.pattern].identifier().unwrap()),
						child_pattern,
						Some(d.declared_type),
					));
				}
			}

			for (attrib, _child_class, declared_type) in child_classes.iter() {
				let Some(attrib) = attrib else {
					continue;
				};
				let occurrence_path = attrib_path
					.iter()
					.copied()
					.chain(std::iter::once(*attrib))
					.collect::<Vec<_>>();
				let source_occurrence = collector
					.parent
					.nested_occurrence(root_pattern, &occurrence_path);
				if collector
					.parent
					.occurrence_constructors_available(source_occurrence)
				{
					continue;
				}
				let local_domain_source = collector
					.parent
					.occurrence_local_domain_source(source_occurrence);

				let (generators, prev_attrib) = collector.parent.nested_path_generators_and_cursor(
					item,
					&nested_iteration_expr,
					root_pattern,
					source_occurrence,
					&attrib_path,
					local_domain_source,
					attrib_class_pattern_ref,
					*declared_type,
					class_item_data,
					&class_item_types,
				);

				let (record_access, fallback_cardinality) = collector
					.parent
					.nested_child_record_access_and_fallback_cardinality(
						item,
						prev_attrib,
						*attrib,
						local_domain_source,
						*declared_type,
						class_item_data,
						attrib_class_pattern_ref.item(collector.parent.db),
						&class_item_types,
					);
				let sum = collector.parent.nested_occurrence_sum_expr(
					item,
					generators,
					local_domain_source,
					record_access,
					fallback_cardinality,
					attrib_class_pattern_ref,
				);
				collector
					.parent
					.ensure_nested_occurrence_constructor_domain(item, source_occurrence, sum);
			}

			for (attrib, child_class, _) in child_classes.iter().rev() {
				if let Some(a) = attrib {
					let mut new_attrib_path = attrib_path.clone();
					new_attrib_path.push(*a);
					constructor_stack.push((new_attrib_path, *child_class));
				} else {
					constructor_stack.push((attrib_path.clone(), *child_class));
				}
			}
		}

		let mut class_stack = vec![(vec![], class_pattern_ref)];

		while let Some((attrib_path, attrib_class_pattern_ref)) = class_stack.pop() {
			let mut child_classes = vec![];
			// 1. Collect the "new" attributes of the class (and the superclass)
			let Item::Class(class_item_ref) = attrib_class_pattern_ref.item(collector.parent.db)
			else {
				unreachable!()
			};
			let class_hir = class_item_ref.class(collector.parent.db);
			let class_item_types = attrib_class_pattern_ref
				.item(collector.parent.db)
				.types(collector.parent.db);
			let class_item_data = class_hir.data();
			// Push superclass first (if any)
			if let Some(c) = class_hir.extends {
				let child_pattern = class_item_types.name_resolution(c).unwrap();
				child_classes.push((None, child_pattern, None));
			}
			// Then push all attribute classes
			for member in class_hir.items.iter() {
				if let ClassMember::Declaration(d) = member
					&& let Some(c) = class_item_data[d.declared_type].get_new_class(class_item_data)
				{
					let child_pattern = class_item_types.name_resolution(c).unwrap();
					child_classes.push((
						Some(class_item_data[d.pattern].identifier().unwrap()),
						child_pattern,
						Some(d.declared_type),
					));
				}
			}

			// 2. Process the collected attributes
			for (attrib, child_class, declared_type) in child_classes.iter() {
				let occurrence_path = attrib_path
					.iter()
					.copied()
					.chain(attrib.iter().copied())
					.collect::<Vec<_>>();
				let source_occurrence = collector
					.parent
					.nested_occurrence(root_pattern, &occurrence_path);
				let local_domain_source = collector
					.parent
					.occurrence_local_domain_source(source_occurrence);
				let contribution_index = collector
					.parent
					.occurrence_contribution(source_occurrence, *child_class)
					.constructor_index;
				let start_decl_name = attrib_path
					.iter()
					.chain(attrib.iter())
					.map(|a: &Identifier<'db>| a.pretty_print(collector.parent.db))
					.collect::<Vec<_>>()
					.join("_");
				let (generators, prev_attrib) = collector.parent.nested_path_generators_and_cursor(
					item,
					&nested_iteration_expr,
					root_pattern,
					source_occurrence,
					&attrib_path,
					local_domain_source,
					attrib_class_pattern_ref,
					*declared_type,
					class_item_data,
					&class_item_types,
				);

				let mut captured_max_card_per_parent: Option<Expression<'db>> = None;
				let compr_template = if let Some(a) = attrib {
					let (record_access, fallback_cardinality) = collector
						.parent
						.nested_child_record_access_and_fallback_cardinality(
							item,
							prev_attrib,
							*a,
							local_domain_source,
							*declared_type,
							class_item_data,
							attrib_class_pattern_ref.item(collector.parent.db),
							&class_item_types,
						);
					if let (Some(record_access), Some(cardinality)) = (
						record_access.clone(),
						collector.parent.nested_par_collection_cardinality(
							attrib_class_pattern_ref.item(collector.parent.db),
							*declared_type,
							class_item_data,
							&class_item_types,
						),
					) {
						// This emission iterates the walker's cursor — potential
						// storage for var roots — which over-constrains whenever a
						// slot can be unrealised: a `var opt new` root with a nested
						// `set(2..2) of new` field made `absent(a)` unsatisfiable
						// (the unrealised slot's field defaults to `{}`,
						// card 0 ∉ 2..2). For a var-reached owner the invariant is
						// instead emitted once, over the *realised* class set, in
						// `collect_class` (the same shape the var-declared nested
						// set field already uses). Par-reached owners keep this
						// emission: their iterated instances are all realised, and
						// `collect_class` skips them to avoid double-emitting.
						if !collector
							.parent
							.object_lowering
							.var_reached_classes
							.contains(&attrib_class_pattern_ref)
						{
							collector.parent.emit_nested_cardinality_constraint(
								item,
								generators.clone(),
								record_access,
								cardinality,
							);
						}
					}
					let (contribution_generators, maybe_contribution_input) =
						collector.parent.nested_contribution_generators_and_input(
							item,
							local_domain_source,
							&generators,
							record_access.clone(),
						);
					collector.parent.emit_nested_occurrence_contributions(
						item,
						root_pattern,
						inputs_expr.clone(),
						source_occurrence,
						*child_class,
						local_domain_source,
						&attrib_path,
						*a,
						&contribution_generators,
						maybe_contribution_input,
						&start_decl_name,
					);
					// `fallback_cardinality` is the static per-parent slice
					// size (max of the declared cardinality bound) when the
					// nested child collection is identity-shaped. Capture
					// it for the slice array emission below.
					captured_max_card_per_parent = fallback_cardinality.clone();
					collector.parent.nested_occurrence_sum_expr(
						item,
						generators.clone(),
						local_domain_source,
						record_access,
						fallback_cardinality,
						attrib_class_pattern_ref,
					)
				} else {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						IntegerLiteral(1),
					)
				};

				let sum = compr_template;
				if attrib.is_some()
					&& !collector
						.parent
						.occurrence_constructors_available(source_occurrence)
				{
					collector
						.parent
						.ensure_nested_occurrence_constructor_domain(
							item,
							source_occurrence,
							sum.clone(),
						);
				}
				let name_suffix = if attrib.is_none() && !attrib_path.is_empty() {
					format!(
						"{}_{}",
						start_decl_name,
						child_class
							.identifier(collector.parent.db)
							.unwrap()
							.pretty_print(collector.parent.db)
					)
				} else {
					start_decl_name
				};
				// Immediate parent occurrence for this slice's `<field>`
				// attribute: `attrib_path = []` means the parent is the root
				// user decl class; a non-empty `attrib_path` means the parent
				// is a nested class one level deeper. The parent's
				// contribution index in its own class enum is what
				// `class_object_contribution_declaration` keys on, so look it
				// up here and pass through `PendingSlice` for use during
				// slice-array emission.
				let parent_occurrence = if attrib_path.is_empty() {
					root_occurrence
				} else {
					collector
						.parent
						.nested_occurrence(root_pattern, &attrib_path)
				};
				let parent_contribution_index = collector
					.parent
					.occurrence_contribution(parent_occurrence, attrib_class_pattern_ref)
					.constructor_index;
				let singular_opt = (matches!(local_domain_source, LocalDomainSource::OnePerParent)
					&& attrib.is_some())
				.then(|| {
					declared_type
						.map(|dt| {
							matches!(
								class_item_data[dt],
								shackle_hir::Type::New {
									opt: OptType::Opt,
									..
								}
							)
						})
						.unwrap_or(false)
				});
				pending_slices.push(PendingSlice {
					target_class: *child_class,
					contribution_index,
					name_suffix,
					sum_expr: sum,
					max_card_per_parent: captured_max_card_per_parent,
					parent_class: attrib_class_pattern_ref,
					parent_contribution_index,
					field_attribute: *attrib,
					singular_opt,
				});
			}

			// 3. Push the classes of the "new" attributes onto the stack in reverse order of child_classes
			for (attrib, child_class, _) in child_classes.iter().rev() {
				if let Some(a) = attrib {
					let mut new_attrib_path = attrib_path.clone();
					new_attrib_path.push(*a);
					class_stack.push((new_attrib_path, *child_class));
				} else {
					class_stack.push((attrib_path.clone(), *child_class));
				}
			}
		}

		pending_slices.sort_by(|left, right| {
			let left_name = left
				.target_class
				.identifier(collector.parent.db)
				.unwrap()
				.lookup(collector.parent.db);
			let right_name = right
				.target_class
				.identifier(collector.parent.db)
				.unwrap()
				.lookup(collector.parent.db);
			left_name
				.cmp(right_name)
				.then(left.contribution_index.cmp(&right.contribution_index))
		});

		for pending_slice in pending_slices {
			let start = if pending_slice.contribution_index == 0 {
				Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					IntegerLiteral(1),
				)
			} else {
				let end_decl = collector.parent.contribution_end_map[&(
					pending_slice.target_class,
					pending_slice.contribution_index - 1,
				)];
				Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ResolvedIdentifier::Declaration(end_decl),
				)
			};
			let mut start_decl =
				Declaration::from_expression(collector.parent.db, true, start.clone());
			start_decl.set_name(Identifier::new(
				collector.parent.db,
				format!(
					"{}_{}_start",
					class_and_decl_name, pending_slice.name_suffix
				),
			));
			let start_idx = collector
				.parent
				.model
				.add_declaration(DeclarationItem::new(start_decl, item));
			let start_expr = Expression::new(
				collector.parent.db,
				&collector.parent.model,
				item,
				start_idx,
			);
			let end_expr = match (&*start, &*pending_slice.sum_expr) {
				(ExpressionData::IntegerLiteral(start), ExpressionData::IntegerLiteral(sum)) => {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						IntegerLiteral(start.0 + sum.0),
					)
				}
				_ => Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.builtins.plus.into(),
						arguments: vec![start_expr, pending_slice.sum_expr],
					},
				),
			};
			let mut end_decl = Declaration::from_expression(collector.parent.db, true, end_expr);
			end_decl.set_name(Identifier::new(
				collector.parent.db,
				format!("{}_{}_end", class_and_decl_name, pending_slice.name_suffix),
			));
			let end_idx = collector
				.parent
				.model
				.add_declaration(DeclarationItem::new(end_decl, item));
			let _ = collector.parent.contribution_end_map.insert(
				(pending_slice.target_class, pending_slice.contribution_index),
				end_idx,
			);

			// Per-parent slice array `<parent>_<field>_potential`. Emitted
			// alongside the start/end boundaries: for each potential parent p,
			// build `<child>_occ_N(start + (p-1)*max .. start + p*max - 1)`,
			// the contiguous block of child identities that p's `<field>` is
			// allowed to draw from. Consumed by the per-parent subset
			// constraint.
			if let Some(max_card_per_parent) = pending_slice.max_card_per_parent.clone() {
				let child_enum = collector.parent.class_map[&pending_slice.target_class].class_enum;
				let child_enum_member =
					EnumMemberId::new(child_enum, pending_slice.contribution_index as u32);

				// Pick the iter collection + iterator element type:
				//  - If the immediate parent's contribution storage decl is
				//    already registered (nested-of-nested case, e.g. the
				//    `Vehicle.crew` slice with `Vehicle_vehicles_objects` as
				//    parent), iterate `index_set(parent_storage)` so the slice
				//    array's dim matches the parent storage's dim. This makes
				//    the per-parent subset constraint's `slice[p]` line up with
				//    `parent_storage[p].<field>` when both iterate the same
				//    index set.
				//  - Otherwise (root-class case: the root contribution decl
				//    isn't registered until *after* the slice loop), fall back
				//    to the precomputed `nested_iteration_expr`.
				//
				// Nested per-contribution storage is enum-indexed by
				// `<Parent>_occ_k(<local-universe>)`, so when we use it as the
				// iter source we also need to wrap the iterator with `enum2int`
				// for the `(p-1)*max+1..p*max` arithmetic.
				let parent_storage_decl_opt =
					collector.parent.class_object_contribution_declaration(
						pending_slice.parent_class,
						pending_slice.parent_contribution_index,
					);
				let (parent_iter_collection, parent_index_ty) = match parent_storage_decl_opt {
					Some(parent_decl) => {
						let parent_expr = Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							ResolvedIdentifier::Declaration(parent_decl),
						);
						let dim_ty = match parent_expr.ty().lookup(collector.parent.db) {
							TyData::Array { dim, .. } => *dim,
							_ => Ty::par_int(collector.parent.db),
						};
						let iter = Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							LookupCall {
								function: collector.parent.ids.functions.index_set.into(),
								arguments: vec![parent_expr],
							},
						);
						(iter, dim_ty)
					}
					None => {
						// `nested_iteration_expr` is sometimes the parent
						// storage array (singular `var new`) and sometimes
						// already an `index_set(...)` expression (the bounded
						// var-set path sets it to that). Pick
						// `index_set(...)` only when we're holding an array.
						let iter = if nested_iteration_expr.ty().is_set(collector.parent.db) {
							nested_iteration_expr.clone()
						} else {
							Expression::new(
								collector.parent.db,
								&collector.parent.model,
								item,
								LookupCall {
									function: collector.parent.ids.functions.index_set.into(),
									arguments: vec![nested_iteration_expr.clone()],
								},
							)
						};
						(iter, Ty::par_int(collector.parent.db))
					}
				};
				let parent_index_decl = Declaration::new(
					false,
					Domain::unbounded(collector.parent.db, item, parent_index_ty),
				);
				let parent_index_idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(parent_index_decl, item));
				let parent_index_expr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					parent_index_idx,
				);
				// Convert enum-typed iterator to par int for arithmetic.
				let par_int_ty = Ty::par_int(collector.parent.db);
				let parent_index_int_expr = if parent_index_ty == par_int_ty {
					parent_index_expr.clone()
				} else {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						LookupCall {
							function: collector.parent.ids.functions.enum2int.into(),
							arguments: vec![parent_index_expr.clone()],
						},
					)
				};

				// The per-parent slice draws constructor arguments from the
				// constructor's *private* universe `1..card(parent)*max` declared
				// by `ensure_nested_occurrence_constructor_domain`, not from the
				// chained `_start_/_end_` offsets in `contribution_end_map`. The
				// chain stays intact (consumed by `project_class_identity` to
				// recover constructor-local ordinals from `enum2int` globals),
				// but mixing it in here would emit `<Child>_occ_N(start..)` with
				// `start > 1` against a constructor whose universe is `1..sum`.
				let one_expr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					IntegerLiteral(1),
				);
				// (p - 1)
				let p_minus_one = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.functions.minus.into(),
						arguments: vec![parent_index_int_expr.clone(), one_expr.clone()],
					},
				);
				// (p - 1) * max
				let lower_offset = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.functions.times.into(),
						arguments: vec![p_minus_one, max_card_per_parent.clone()],
					},
				);
				// 1 + (p - 1) * max
				let lower_bound = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.builtins.plus.into(),
						arguments: vec![one_expr, lower_offset],
					},
				);
				// p * max  (== 1 + p*max - 1, so the constructor argument fits
				// in the private 1..card(parent)*max universe)
				let upper_bound = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.functions.times.into(),
						arguments: vec![parent_index_int_expr, max_card_per_parent],
					},
				);
				let ordinal_range = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					LookupCall {
						function: collector.parent.ids.functions.dot_dot.into(),
						arguments: vec![lower_bound, upper_bound],
					},
				);
				let slice_template = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					Call {
						function: Callable::EnumConstructor(child_enum_member),
						arguments: vec![ordinal_range],
					},
				);
				// MZN array comprehensions are int-indexed by default. For the
				// nested-of-nested case we want the slice array to take the
				// same dim type as the parent storage so the subset constraint's
				// `slice[p]` matches `parent_storage[p].<field>` when iterating
				// `index_set(parent_storage)`. Use the indexed-comprehension form
				// (`[p: <template> | p in <collection>]`) — the resulting array
				// then has dim type = `parent_index_expr.ty()`.
				let slice_compr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ArrayComprehension::indexed(
						[Generator::Iterator {
							declarations: vec![parent_index_idx],
							collection: parent_iter_collection,
							where_clause: None,
						}],
						parent_index_expr.clone(),
						slice_template,
					),
				);
				// MZN enforces strict index-set matching for declared arrays:
				// the parent storage's dim might be `Vehicle_occ_1(1..4)` (a
				// constructor-applied subset of `Vehicle_potential`), and an
				// `array [Vehicle_potential]` declaration with a 4-entry RHS
				// would be flagged as an index-set mismatch. Clone the parent
				// storage's dim Domain into the slice's declared Array domain
				// so the declared and assigned dim match exactly.
				let slice_compr_ty = slice_compr.ty();
				let slice_decl = match parent_storage_decl_opt {
					Some(parent_decl) => {
						let parent_dim_domain =
							match &**collector.parent.model[parent_decl].domain() {
								DomainData::Array(dim, _) => Some((**dim).clone()),
								_ => None,
							};
						match parent_dim_domain {
							Some(dim_domain) => {
								let elem_ty = slice_compr_ty
									.elem_ty(collector.parent.db)
									.unwrap_or(slice_compr_ty);
								let elem_domain =
									Domain::unbounded(collector.parent.db, item, elem_ty);
								let array_domain = Domain::array(
									collector.parent.db,
									item,
									OptType::NonOpt,
									dim_domain,
									elem_domain,
								);
								let mut decl = Declaration::new(true, array_domain);
								decl.set_definition(slice_compr);
								decl
							}
							None => {
								Declaration::from_expression(collector.parent.db, true, slice_compr)
							}
						}
					}
					None => Declaration::from_expression(collector.parent.db, true, slice_compr),
				};
				let mut slice_decl = slice_decl;
				slice_decl.set_name(Identifier::new(
					collector.parent.db,
					format!(
						"{}_{}_potential",
						class_and_decl_name, pending_slice.name_suffix
					),
				));
				let slice_decl_idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(slice_decl, item));
				let _ = collector.parent.slice_array_decls.insert(
					(pending_slice.target_class, pending_slice.contribution_index),
					slice_decl_idx,
				);
				// Record this slice as a contribution to the child class's
				// actual-set definition, keyed ONE HOP up: the *immediate*
				// parent class/contribution and the direct `<field>` name —
				// not the root class with the joined path name, which for
				// multi-hop intros (`ps_ns`) is not a record field of anything
				// and would force the unsound universe fallback. `finish`
				// assembles per-class `array_union(...)` expressions from
				// these records.
				if let Some(field_attribute) = pending_slice.field_attribute {
					collector
						.parent
						.class_set_field_introductions
						.entry(pending_slice.target_class)
						.or_default()
						.push(FieldIntroduction {
							parent_class: pending_slice.parent_class,
							parent_contribution_index: pending_slice.parent_contribution_index,
							attribute: field_attribute,
							child_contribution_index: pending_slice.contribution_index,
							kind: FieldIntroductionKind::Collection,
						});
				}
			}

			// Singular `new`/`opt new` fields have no per-parent slice array
			// (their per-parent block is statically one identity), but they
			// still introduce children: record the introduction so `finish`
			// derives the occurs-/realisation-guarded identity singleton for
			// the child's actual set instead of the unsound universe fallback.
			if let (Some(field_attribute), Some(opt)) =
				(pending_slice.field_attribute, pending_slice.singular_opt)
			{
				collector
					.parent
					.class_set_field_introductions
					.entry(pending_slice.target_class)
					.or_default()
					.push(FieldIntroduction {
						parent_class: pending_slice.parent_class,
						parent_contribution_index: pending_slice.parent_contribution_index,
						attribute: field_attribute,
						child_contribution_index: pending_slice.contribution_index,
						kind: FieldIntroductionKind::Singular { opt },
					});
			}
		}

		// A var collection root's enum constructors must exist BEFORE the
		// contribution engine runs: the realisation guard's per-slot test
		// references `<C>_occ_k(p)`, and building an enum-constructor call
		// requires the enum definition. Nested/child occurrence constructors
		// are registered by the machinery above; par collection roots keep
		// registering theirs in the tail block below (nothing in their
		// contributions references their own constructor).
		if let Some(root_collection) = &pending_root_collection
			&& root_collection.inst == VarType::Var
		{
			let ordinal_domain = root_collection
				.potential_ordinal_domain
				.clone()
				.expect("top-level var collections must have an ordinal domain");
			let enum_constr_domain = Domain::bounded(
				collector.parent.db,
				item,
				VarType::Par,
				OptType::NonOpt,
				ordinal_domain,
			);
			let decl = Declaration::new(false, enum_constr_domain);
			let idx = collector
				.parent
				.model
				.add_declaration(DeclarationItem::new(decl, item));
			collector
				.parent
				.add_occurrence_constructors(root_occurrence, idx);
		}

		if let Some(root_collection) = &pending_root_collection {
			let root_fields = collector.parent.class_storage_fields(class_pattern_ref);
			if root_collection.emit_root_contribution {
				// Every reconstructing root shape runs the same engine
				// (`engine_reconstructed_root_contribution_expr`): per-field
				// rules — defined / identity / read / free — selected from the
				// source array's element type. The two shapes that skip it are
				// trivial passthroughs where the source already IS the full
				// storage record, so there is nothing to define or mint and the
				// contribution is vacuously determined.
				//
				// Only a var collection root has unrealisable slots (a par
				// root realises every input; `array of var new` realises every
				// potential by construction), so only it pays the realisation
				// guard on its defined fields.
				let collection_realisation_guard = || {
					(root_collection.inst == VarType::Var).then(|| RootRealisationGuard {
						constructor_index: root_collection.contribution_index,
						name_prefix: class_and_decl_name.clone(),
					})
				};
				let (contribution_expr, contribution_determined) =
					if let Some(contribution_expr) = &root_collection.contribution_expr {
						// A var set-of-new object-field root sources its free
						// `_storage` array. Computed / domain-dependent fields
						// are excluded from `_storage` (they aren't free
						// decisions — `free_storage_record_ty`), so if any
						// storage field is absent from the free element record,
						// run the engine over `_storage` (the missing fields are
						// alias-defined; free fields read through). Without
						// this, `<C>_objects` is missing the computed field
						// entirely and every downstream `.<attr>` access (e.g.
						// the symmetry-break default loop) panics in
						// `RecordAccess::build`.
						let storage_elem_fields = contribution_expr
							.ty()
							.elem_ty(collector.parent.db)
							.and_then(|elem| elem.record_fields(collector.parent.db));
						let missing_storage_field = root_fields.iter().any(|(field_ident, _)| {
							!storage_elem_fields
								.as_ref()
								.map(|fields| {
									fields
										.iter()
										.any(|(field, _)| Identifier(*field) == *field_ident)
								})
								.unwrap_or(false)
						});
						if missing_storage_field {
							(
								collector
									.parent
									.engine_reconstructed_root_contribution_expr(
										item,
										class_pattern_ref,
										root_pattern,
										contribution_expr.clone(),
										&root_fields,
										collection_realisation_guard(),
									),
								true,
							)
						} else {
							// Free-storage passthrough: every storage field is a
							// free decision, so there are no defined fields —
							// vacuously determined.
							(contribution_expr.clone(), true)
						}
					} else if root_collection.needs_reconstruction {
						(
							collector
								.parent
								.engine_reconstructed_root_contribution_expr(
									item,
									class_pattern_ref,
									root_pattern,
									inputs_expr.clone(),
									&root_fields,
									collection_realisation_guard(),
								),
							true,
						)
					} else {
						// Input-record passthrough: the input representation
						// equals the storage record (par roots whose class has
						// no defined, var-only or object-typed fields) —
						// vacuously determined.
						(inputs_expr.clone(), true)
					};
				let contribution_array_ty = contribution_expr.ty();
				let contribution_domain = collector.parent.build_class_storage_array_domain(
					class_pattern_ref,
					contribution_array_ty,
					item,
				);
				let mut contribution_decl = Declaration::new(true, contribution_domain);
				contribution_decl.set_name(Identifier::new(
					collector.parent.db,
					format!("{}_objects", class_and_decl_name),
				));
				contribution_decl.set_definition(contribution_expr);
				let contribution_decl_idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(contribution_decl, item));
				collector.parent.register_class_object_contribution(
					class_pattern_ref,
					root_collection.contribution_index,
					contribution_decl_idx,
					contribution_determined,
				);

				// Inheritance: for each superclass contribution from this
				// introduction, build a `_objects` array by projecting the
				// superclass's storage fields out of the direct-class objects
				// array. Mirrors the singular `var new` path's per-target
				// reconstruction.
				let direct_objects_expr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ResolvedIdentifier::Declaration(contribution_decl_idx),
				);
				let root_contributions =
					collector.parent.object_lowering.contributions_by_occurrence[&root_occurrence]
						.clone();
				for contribution in root_contributions {
					let target_class = contribution.target_class;
					if target_class == class_pattern_ref {
						continue;
					}
					let target_fields = collector.parent.class_storage_fields(target_class);
					let projection_expr = collector.parent.reconstructed_root_contribution_expr(
						item,
						root_pattern,
						direct_objects_expr.clone(),
						&target_fields,
						true,
					);
					let projection_array_ty = projection_expr.ty();
					let projection_domain = collector.parent.build_class_storage_array_domain(
						target_class,
						projection_array_ty,
						item,
					);
					let target_class_name = target_class
						.identifier(collector.parent.db)
						.unwrap()
						.pretty_print(collector.parent.db);
					let mut projection_decl = Declaration::new(true, projection_domain);
					projection_decl.set_name(Identifier::new(
						collector.parent.db,
						format!("{}_{}_objects", target_class_name, class_and_decl_name),
					));
					projection_decl.set_definition(projection_expr);
					let projection_decl_idx = collector
						.parent
						.model
						.add_declaration(DeclarationItem::new(projection_decl, item));
					// The projection reads every target field out of the
					// direct-class objects array, which carries the direct
					// contribution's (possibly alias-defined) values — so the
					// projected columns are exactly as determined as the direct
					// contribution's.
					collector.parent.register_class_object_contribution(
						target_class,
						contribution.constructor_index,
						projection_decl_idx,
						contribution_determined,
					);
				}
			}
		}

		if matches!(item_ty, shackle_hir::Type::New { .. }) {
			// Of the singular root shapes, only `var opt new` has an
			// unrealisable slot (par `new` and plain `var new` realise their
			// single potential unconditionally), so only it pays the
			// realisation guard on defined fields.
			let singular_slot_may_be_unrealised = matches!(
				item_ty,
				shackle_hir::Type::New {
					inst: VarType::Var,
					opt: OptType::Opt,
					..
				}
			);
			let mut root_contributions =
				collector.parent.object_lowering.contributions_by_occurrence[&root_occurrence]
					.clone();
			// The direct contribution must be registered before the
			// inheritance projections: they read the superclass's storage
			// fields out of the already-reconstructed direct-class objects
			// array (symmetric with the collection-root path) rather than
			// fresh-minting defined fields from the raw inputs.
			root_contributions.sort_by_key(|contribution| contribution.projection_depth);
			let mut direct_contribution: Option<(Expression<'db>, bool)> = None;
			for contribution in root_contributions.iter() {
				let target_class = contribution.target_class;
				let root_fields = collector.parent.class_storage_fields(target_class);
				let has_object_fields = root_fields
					.iter()
					.any(|(_, field_ty)| field_ty.class_type(collector.parent.db).is_some());
				// A storage field the input record doesn't carry (a computed
				// attribute, or an explicitly-`var` attribute) means the input
				// must be reconstructed into the full storage record rather than
				// aliased straight through.
				let input_elem_fields = inputs_expr
					.ty()
					.elem_ty(collector.parent.db)
					.and_then(|elem| elem.record_fields(collector.parent.db));
				let has_storage_only_field = root_fields.iter().any(|(field_ident, _)| {
					!input_elem_fields
						.as_ref()
						.map(|fields| {
							fields
								.iter()
								.any(|(field, _)| Identifier(*field) == *field_ident)
						})
						.unwrap_or(false)
				});
				let (contribution_expr, contribution_determined) = if target_class
					== class_pattern_ref
				{
					if has_object_fields || has_storage_only_field {
						// The engine reconstructs the direct contribution with
						// per-field rules: computed attributes are *defined*
						// (`n = card(children)`), class-typed fields are read
						// through when the input holds identities (var
						// `_storage`) or identity-minted when it holds inline
						// records (par roots), var-only fields become fresh
						// decisions with their declared per-object domains.
						(
							collector
								.parent
								.engine_reconstructed_root_contribution_expr(
									item,
									class_pattern_ref,
									root_pattern,
									inputs_expr.clone(),
									&root_fields,
									singular_slot_may_be_unrealised.then(|| RootRealisationGuard {
										constructor_index: contribution.constructor_index,
										name_prefix: class_and_decl_name.clone(),
									}),
								),
							true,
						)
					} else {
						// Input-record passthrough: every storage field is
						// supplied by the input in storage form — no defined
						// fields, vacuously determined.
						(inputs_expr.clone(), true)
					}
				} else {
					// Inheritance projection: read the superclass's storage
					// fields out of the already-registered direct-class objects
					// array, which carries the direct contribution's (possibly
					// alias-defined) values — so the projected columns are
					// exactly as determined as the direct contribution's.
					let (direct_expr, direct_determined) = direct_contribution
						.clone()
						.expect("direct contribution registered before singular-root projections");
					(
						collector.parent.reconstructed_root_contribution_expr(
							item,
							root_pattern,
							direct_expr,
							&root_fields,
							true,
						),
						direct_determined,
					)
				};
				let contribution_array_ty = contribution_expr.ty();
				let contribution_domain = collector.parent.build_class_storage_array_domain(
					target_class,
					contribution_array_ty,
					item,
				);
				let mut contribution_decl = Declaration::new(true, contribution_domain);
				let target_class_name = target_class
					.identifier(collector.parent.db)
					.unwrap()
					.pretty_print(collector.parent.db);
				let contribution_name = if target_class == class_pattern_ref {
					format!("{}_objects", class_and_decl_name)
				} else {
					format!("{}_{}_objects", target_class_name, class_and_decl_name)
				};
				contribution_decl.set_name(Identifier::new(collector.parent.db, contribution_name));
				contribution_decl.set_definition(contribution_expr);
				let contribution_decl_idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(contribution_decl, item));
				collector.parent.register_class_object_contribution(
					target_class,
					contribution.constructor_index,
					contribution_decl_idx,
					contribution_determined,
				);
				if target_class == class_pattern_ref {
					direct_contribution = Some((
						Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							ResolvedIdentifier::Declaration(contribution_decl_idx),
						),
						contribution_determined,
					));
				}
			}
		}

		if let Some(root_collection) = pending_root_collection {
			if root_collection.inst == VarType::Var {
				// Constructor registration for this occurrence happened before
				// the contribution block (the realisation guard needs it).
				let ordinal_domain = root_collection
					.potential_ordinal_domain
					.expect("top-level var collections must have an ordinal domain");
				let call_expr = alloc_expression(
					Call {
						function: Callable::EnumConstructor(enum_member_id),
						arguments: vec![ordinal_domain],
					},
					&collector,
					item,
				);
				let new_domain = Domain::bounded(
					collector.parent.db,
					item,
					VarType::Par,
					OptType::NonOpt,
					call_expr,
				);
				// Carry the declared cardinality bound (`set(<card>) of new C`)
				// into the domain. The pretty printer emits the `set(<card>)
				// of …` syntax and the target MiniZinc desugars it to a
				// `card(…) in <card>` constraint.
				let cardinality = match item_ty {
					shackle_hir::Type::Set { cardinality, .. } => {
						cardinality.map(|c| collector.collect_expression(c))
					}
					_ => None,
				};
				domain_decl.set_domain(Domain::set_with_card(
					collector.parent.db,
					item,
					root_collection.inst,
					OptType::NonOpt,
					cardinality,
					new_domain,
				));
				return domain_decl;
			}

			let start_expr = if root_collection.contribution_index == 0 {
				Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					IntegerLiteral(1),
				)
			} else {
				let end_decl = collector.parent.contribution_end_map
					[&(class_pattern_ref, root_collection.contribution_index - 1)];
				Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ResolvedIdentifier::Declaration(end_decl),
				)
			};
			let end_decl = collector.parent.contribution_end_map
				[&(class_pattern_ref, root_collection.contribution_index)];
			let end_expr = Expression::new(
				collector.parent.db,
				&collector.parent.model,
				item,
				ResolvedIdentifier::Declaration(end_decl),
			);
			let slice_range = match (&*start_expr, &*root_collection.sum_expr) {
				(ExpressionData::IntegerLiteral(start), ExpressionData::IntegerLiteral(sum)) => {
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						SetLiteral(
							(start.0..(start.0 + sum.0))
								.map(|value| {
									Expression::new(
										collector.parent.db,
										&collector.parent.model,
										item,
										IntegerLiteral(value),
									)
								})
								.collect(),
						),
					)
				}
				_ => {
					let one_expr = Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						IntegerLiteral(1),
					);
					let end_minus_one = Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						LookupCall {
							function: collector.parent.ids.functions.minus.into(),
							arguments: vec![end_expr, one_expr],
						},
					);
					Expression::new(
						collector.parent.db,
						&collector.parent.model,
						item,
						LookupCall {
							function: collector.parent.ids.functions.dot_dot.into(),
							arguments: vec![start_expr, end_minus_one],
						},
					)
				}
			};
			let enum_constr_domain = Domain::bounded(
				collector.parent.db,
				item,
				VarType::Par,
				OptType::NonOpt,
				slice_range.clone(),
			);
			let decl = Declaration::new(false, enum_constr_domain);
			let idx = collector
				.parent
				.model
				.add_declaration(DeclarationItem::new(decl, item));
			collector
				.parent
				.add_occurrence_constructors(root_occurrence, idx);
			let call_expr = alloc_expression(
				Call {
					function: Callable::EnumConstructor(enum_member_id),
					arguments: vec![slice_range.clone()],
				},
				&collector,
				item,
			);
			if matches!(item_ty, shackle_hir::Type::Array { .. }) {
				let ordinal_decl = Declaration::new(
					false,
					Domain::unbounded(collector.parent.db, item, Ty::par_int(collector.parent.db)),
				);
				let ordinal_idx = collector
					.parent
					.model
					.add_declaration(DeclarationItem::new(ordinal_decl, item));
				let ordinal_expr = Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ordinal_idx,
				);
				domain_decl.set_definition(Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![ordinal_idx],
							collection: slice_range,
							where_clause: None,
						}],
						Expression::new(
							collector.parent.db,
							&collector.parent.model,
							item,
							Call {
								function: Callable::EnumConstructor(enum_member_id),
								arguments: vec![ordinal_expr],
							},
						),
					),
				));
				return domain_decl;
			}
			let new_domain = Domain::bounded(
				collector.parent.db,
				item,
				VarType::Par,
				OptType::NonOpt,
				call_expr.clone(),
			);
			// `set(<card>) of new C` roots carry the declared cardinality
			// bound into the domain (`var set(<card>) of <C>_potential`).
			// The pretty printer emits the `set(<card>) of …` syntax and the
			// target MiniZinc desugars it to a `card(…) in <card>` constraint.
			// Applies to both var and par roots.
			let cardinality = match item_ty {
				shackle_hir::Type::Set { cardinality, .. } => {
					cardinality.map(|c| collector.collect_expression(c))
				}
				_ => None,
			};
			let domain = Domain::set_with_card(
				collector.parent.db,
				item,
				root_collection.inst,
				OptType::NonOpt,
				cardinality,
				new_domain,
			);
			domain_decl.set_domain(domain);
			domain_decl.set_definition(call_expr);
		}

		domain_decl
	}

	/// The bodyless-builtin total-ops whitelist: arithmetic that cannot
	/// overflow-trap (`+ - *`), `card`, `sum`/`exists`/`forall` (total on
	/// empty), set construction and set ops, comparisons, boolean ops.
	/// `div`/`mod`/`'[]'`/`min`/`max`/`deopt`/`assert`/`pow` are deliberately
	/// absent.
	pub(in crate::lower) fn total_builtin_call(&self, ident: Identifier<'db>) -> bool {
		let ids = self.ids;
		[
			ids.builtins.plus,
			ids.functions.minus,
			ids.builtins.times,
			ids.functions.card,
			ids.functions.sum,
			ids.functions.forall,
			ids.functions.exists,
			ids.functions.dot_dot,
			ids.functions.in_,
			ids.builtins.subset,
			ids.builtins.superset,
			ids.builtins.intersect,
			ids.builtins.union,
			ids.builtins.diff,
			ids.builtins.symdiff,
			ids.functions.eq,
			ids.builtins.ne,
			ids.builtins.lt,
			ids.builtins.le,
			ids.functions.gt,
			ids.functions.ge,
			ids.functions.and,
			ids.functions.or,
			ids.functions.implies,
			ids.functions.rev_imp,
			ids.functions.iff,
			ids.functions.not,
			ids.builtins.xor,
		]
		.contains(&ident)
	}
}

/// Lower the HIR program into THIR
pub fn lower_model<'db>(db: &'db dyn Db) -> Intermediate<Model<'db>> {
	log::info!("Lowering model to THIR");
	let hir = run_hir_phase(db);
	let ids = IdentifierRegistry::lookup(db);
	let counts = EntityCounts::lookup(db);
	let mut collector = ItemCollector::new(db, ids, counts);
	// Predeclare every class (enum, `_objects` array, actual set) before any
	// item is collected, so cross-class references resolve regardless of item
	// order, then rebuild the storage domains once all classes are registered
	// (class reference cycles have no valid predeclare order).
	for item in hir.items.iter() {
		if let Item::Class(c) = item {
			collector.predeclare_class(*c);
		}
	}
	// Source order, not hash order: predeclaring a class allocates its enum,
	// `_objects` array and actual-set declarations, so this loop fixes the id
	// allocation order that the whole emitted model is then ordered by.
	let mut contribution_targets = collector
		.object_lowering
		.contributions_in_occurrence_order()
		.flat_map(|contributions| {
			contributions
				.iter()
				.map(|contribution| contribution.target_class)
		})
		.collect::<Vec<_>>();
	collector
		.object_lowering
		.in_class_order(&mut contribution_targets);
	for target_class in contribution_targets {
		collector.ensure_class_predeclared(target_class);
	}
	collector.repair_predeclared_class_objects_domains();
	for item in hir.items.iter() {
		collector.collect_item(*item);
	}
	collector.collect_deferred();
	// Object lowering emits items grouped by class rather than in source
	// order; restore a stable, source-ordered top-level item list.
	let item_order = hir
		.items
		.iter()
		.enumerate()
		.map(|(index, item)| (*item, index))
		.collect::<FxHashMap<_, _>>();
	collector
		.model
		.reorder_top_level_items_by_hir_order(db, &item_order);
	Intermediate::new(collector.finish())
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use crate::{lower::lower_model, pretty_print::PrettyPrinter};

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// Turns off stdlib inclusion.
	pub(crate) fn check_no_stdlib(source: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = lower_model(&db).take();
		let pretty = PrettyPrinter::new(&db, &model).pretty_print();
		expected.assert_eq(&pretty);
	}

	#[test]
	fn test_lower_named_args() {
		check_no_stdlib(
			r#"
			test foo(int: hello, int: world, int: bar, int: qux);
			any: x = foo(1, 2, qux: 4, bar: 3);
			"#,
			expect![[r#"
    function bool: foo(int: hello, int: world, int: bar, int: qux);
    bool: x = foo(1, 2, 3, 4);
    solve satisfy;
"#]],
		);
	}
	#[test]
	fn test_lower_named_and_default_args() {
		check_no_stdlib(
			r#"
			test foo(int: hello, int: world, int: bar = 3, int: qux = 4);
			any: x = foo(1, world: 2, qux: 10);
			"#,
			expect![[r#"
    function bool: foo(int: hello, int: world, int: bar, int: qux);
    bool: x = foo(1, 2, 3, 10);
    solve satisfy;
"#]],
		);
	}
}
