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
use rustc_hash::FxHashMap;
use shackle_hir::{
	Item, PatternTy, TypeResult,
	class_analysis::{LocalDomainSource, OccurrenceId, class_pattern_for},
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
use self::{expression::ExpressionCollector, objects::ObjectState};

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
	model: Model<'db>,
	type_alias_expressions: FxHashMap<ExpressionRef<'db>, DeclarationId<'db>>,
	deferred: Vec<(FunctionId<'db>, Item<'db>)>,
	/// Object-lowering state: the derived occurrence plan plus everything
	/// accumulated while collecting class items and `new` declarations.
	objects: ObjectState<'db>,
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
			model: Model::with_capacities(&entity_counts.into()),
			type_alias_expressions: FxHashMap::default(),
			deferred: Vec::new(),
			objects: ObjectState::new(db),
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
				let _ = self.collect_constraint(item, c.constraint(self.db), true, false);
			}
			Item::Declaration(d) => {
				// A top-level declaration is in an output context only if it says
				// so itself, which `collect_declaration` reads off its annotations.
				let _ = self.collect_declaration(item, d.declaration(self.db), true, false);
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
		in_output: bool,
	) -> ConstraintId<'db> {
		let db = self.db;
		let types = item.types(db);
		let mut collector =
			ExpressionCollector::new(self, item.data(db), item, &types).inherit_output(in_output);
		let mut constraint = Constraint::new(top_level, collector.collect_expression(c.expression));
		constraint.annotations_mut().extend(
			c.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model
			.add_constraint(ConstraintItem::new(constraint, item))
	}

	/// Whether a declaration carries `::output_only`. This is the condition the
	/// HIR typer uses to type the definition in output mode
	/// (`collect_output_declaration` / `typecheck_output`), so the lowering has
	/// to agree about it or the two disagree on the definition's inst.
	fn is_output_only(
		&self,
		data: &shackle_hir::ItemData<'db>,
		d: &shackle_hir::Declaration<'db>,
	) -> bool {
		d.annotations.iter().any(|ann| {
			matches!(
				&data[*ann],
				shackle_hir::Expression::Identifier(i) if *i == self.ids.annotations.output_only
			)
		})
	}

	/// Collect a declaration item
	fn collect_declaration(
		&mut self,
		item: Item<'db>,
		d: &shackle_hir::Declaration<'db>,
		top_level: bool,
		in_output: bool,
	) -> Vec<DeclOrConstraint<'db>> {
		let db = self.db;
		let types = item.types(db);
		let ty = match &types[d.pattern] {
			PatternTy::Variable(ty) => *ty,
			PatternTy::Destructuring(ty) => *ty,
			_ => unreachable!(),
		};
		let data = item.data(db);
		// A declaration is in an output context either because it carries
		// `::output_only` itself, or because it is a `let` item inside one that
		// does — the caller knows the latter.
		let output_only = in_output || self.is_output_only(data, d);
		let mut collector =
			ExpressionCollector::new(self, data, item, &types).inherit_output(output_only);
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
			if let Some(class_info) = collector
				.parent
				.objects
				.class_map
				.get(&class_pattern_ref)
				.copied()
			{
				let root_occurrence = collector.parent.top_level_occurrence(root_pattern);
				let contributions = collector.parent.objects.plan.contributions_by_occurrence
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
		let mut collector = ExpressionCollector::new(self, o.data(), item, &types).in_output();
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
		.objects
		.plan
		.contributions_in_occurrence_order()
		.flat_map(|contributions| {
			contributions
				.iter()
				.map(|contribution| contribution.target_class)
		})
		.collect::<Vec<_>>();
	collector
		.objects
		.plan
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
mod tests;
