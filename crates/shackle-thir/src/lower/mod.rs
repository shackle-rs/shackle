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
	Item, PatternTy, TypeResult,
	class_analysis::{LocalDomainSource, OccurrenceId, analyse_new_objects, class_pattern_for},
	constants::IdentifierRegistry,
	counts::EntityCounts,
	ids::{EntityRef, ExpressionRef, PatternRef},
	run_hir_phase,
};
use shackle_ty::{Ty, TyData};

use super::{source::Origin, *};
use crate::{Db, db::Intermediate};

mod expression;
mod objects;

// Imported by name rather than glob: `alloc_expression` would otherwise be
// ambiguous with `crate::traverse::fold::alloc_expression`, which `super::*`
// brings into scope.
use self::{
	expression::ExpressionCollector,
	objects::{ClassBodyConstraint, ClassMapInfo, FieldIntroduction, ObjectLoweringPlan},
};

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
