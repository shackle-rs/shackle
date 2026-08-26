//! Lowering of top-level `new` declarations.
//!
//! `collect_new_declaration` handles every shape that introduces objects at the
//! top level (`new C`, `var opt new C`, `set of new C`, `var set(c) of new C`,
//! `array [d] of new C`): it mints the class enum constructors, sizes the
//! inputs and `_objects` storage arrays, registers per-parent slice arrays and
//! contributions, and returns the declaration for the user-named identity set.

use shackle_hir::{
	ClassMember, Item, PatternTy, TypeResult,
	class_analysis::{LocalDomainSource, OccurrenceId},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use super::{FieldIntroduction, FieldIntroductionKind, RootRealisationGuard};
use crate::{
	lower::{
		ItemCollector,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

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

/// The immutable setup a top-level `new` root declaration is lowered against.
///
/// `collect_new_declaration` threads these values through every phase of the
/// lowering; bundling them keeps the extracted phase helpers to a readable
/// signature.
struct NewRootContext<'db, 'a> {
	ty: Ty<'db>,
	item: Item<'db>,
	top_level: bool,
	types: &'a TypeResult<'db>,
	d: &'a shackle_hir::Declaration<'db>,
	data: &'a shackle_hir::ItemData<'db>,
	/// The declared type of the root, e.g. `set of new C`.
	item_ty: &'a shackle_hir::Type<'db>,
	class_pattern_ref: PatternRef<'db>,
	root_pattern: PatternRef<'db>,
	root_occurrence: OccurrenceId,
	enum_member_id: EnumMemberId<'db>,
	/// `<Class>_<declName>`, the prefix for every decl this root introduces.
	class_and_decl_name: String,
}

impl<'db> ItemCollector<'db> {
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
		let collector = ExpressionCollector::new(self, data, item, types);

		let class_domain = data[d.declared_type]
			.get_new_class(data)
			.expect("new declarations should have a class domain");

		let class_pattern_ref = types.name_resolution(class_domain).unwrap();
		let class_enum = collector.parent.objects.class_map[&class_pattern_ref].class_enum;
		let root_pattern = PatternRef::new(collector.parent.db, item, d.pattern);
		let root_occurrence = collector.parent.top_level_occurrence(root_pattern);

		let enum_member_id = EnumMemberId::new(
			class_enum,
			collector.parent.objects.plan.contributions_by_occurrence[&root_occurrence][0]
				.constructor_index as u32,
		);
		let item_ty: &shackle_hir::Type = &data[d.declared_type];

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

		let cx = NewRootContext {
			ty,
			item,
			top_level,
			types,
			d,
			data,
			item_ty,
			class_pattern_ref,
			root_pattern,
			root_occurrence,
			enum_member_id,
			class_and_decl_name: class_and_decl_name.clone(),
		};

		let (domain_decl, inputs_expr, pending_root_collection) = match item_ty {
			shackle_hir::Type::New { inst, opt, .. } => {
				self.lower_singular_new_root(&cx, inst, opt)
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
			} => self.lower_set_of_new_root(&cx, cardinality, inst),
			_ => todo!("Handle other cases of new A: x"),
		};

		let mut pending_slices = Vec::new();
		let nested_iteration_expr = pending_root_collection
			.as_ref()
			.and_then(|root_collection| root_collection.nested_iteration_expr.clone())
			.unwrap_or_else(|| inputs_expr.clone());
		self.push_root_passthrough_slice(&cx, &pending_root_collection, &mut pending_slices);

		// Calculate the sizes of the different objects arrays based on the contained classes

		self.register_reachable_occurrence_constructors(&cx, &nested_iteration_expr);

		self.size_class_storage_and_collect_slices(
			&cx,
			&inputs_expr,
			&nested_iteration_expr,
			&mut pending_slices,
		);

		self.emit_pending_slices(&cx, pending_slices, &nested_iteration_expr);

		self.emit_root_collection_contribution(&cx, &inputs_expr, &pending_root_collection);

		self.register_singular_root_contribution(&cx, &inputs_expr);

		self.finish_root_collection(&cx, domain_decl, pending_root_collection)
	}

	/// Finish a collection-shaped root (`set of new`, `var set(c) of new`,
	/// `array [d] of new`): give the user-named declaration its domain and its
	/// definition over the contribution's identity block.
	fn finish_root_collection(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		mut domain_decl: Declaration<'db>,
		pending_root_collection: Option<PendingRootCollection<'db>>,
	) -> Declaration<'db> {
		let &NewRootContext {
			item,
			data,
			types,
			item_ty,
			class_pattern_ref,
			root_occurrence,
			enum_member_id,
			..
		} = cx;
		let mut collector = ExpressionCollector::new(self, data, item, types);
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
				let end_decl = collector.parent.objects.contribution_end_map
					[&(class_pattern_ref, root_collection.contribution_index - 1)];
				Expression::new(
					collector.parent.db,
					&collector.parent.model,
					item,
					ResolvedIdentifier::Declaration(end_decl),
				)
			};
			let end_decl = collector.parent.objects.contribution_end_map
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

	/// Register the contribution for a singular root (`new C`, `var new C`,
	/// `var opt new C`), whose potential block is exactly one slot.
	fn register_singular_root_contribution(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		inputs_expr: &Expression<'db>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			item_ty,
			class_pattern_ref,
			root_pattern,
			root_occurrence,
			ref class_and_decl_name,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);
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
				collector.parent.objects.plan.contributions_by_occurrence[&root_occurrence].clone();
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
	}

	/// Emit a collection root's contribution: pre-register the occurrence's
	/// enum constructor when existence is a decision (the realisation guard
	/// needs it in scope), then reconstruct the contribution block from the
	/// root's inputs and register it against the class.
	fn emit_root_collection_contribution(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		inputs_expr: &Expression<'db>,
		pending_root_collection: &Option<PendingRootCollection<'db>>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			class_pattern_ref,
			root_pattern,
			root_occurrence,
			ref class_and_decl_name,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);

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
				let root_contributions = collector.parent.objects.plan.contributions_by_occurrence
					[&root_occurrence]
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
	}

	/// Emit every per-parent slice array collected while walking the class
	/// graph, and register each one as a contribution to its child class.
	///
	/// Slices are emitted in a deterministic order (see the sort below) so the
	/// model item order does not depend on the traversal's hash iteration.
	fn emit_pending_slices(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		mut pending_slices: Vec<PendingSlice<'db>>,
		nested_iteration_expr: &Expression<'db>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			ref class_and_decl_name,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);
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
				let end_decl = collector.parent.objects.contribution_end_map[&(
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
						function: collector.parent.ids.functions.plus.into(),
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
			let _ = collector.parent.objects.contribution_end_map.insert(
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
				let child_enum =
					collector.parent.objects.class_map[&pending_slice.target_class].class_enum;
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
						function: collector.parent.ids.functions.plus.into(),
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
				let _ = collector.parent.objects.slice_array_decls.insert(
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
						.objects
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
					.objects
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
	}

	/// Walk the class graph below the root, sizing each reached class's
	/// `_objects` storage array for this root's contribution and recording a
	/// `PendingSlice` for every nested `new` field encountered.
	fn size_class_storage_and_collect_slices(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		inputs_expr: &Expression<'db>,
		nested_iteration_expr: &Expression<'db>,
		pending_slices: &mut Vec<PendingSlice<'db>>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			class_pattern_ref,
			root_pattern,
			root_occurrence,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);
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
					nested_iteration_expr,
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
							.objects
							.plan
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
	}

	/// Register the enum constructors for every occurrence reachable from this
	/// root, walking down the class graph before any storage is sized — the
	/// contribution engine's realisation guards reference `<C>_occ_k(..)`, so
	/// the constructors must already exist.
	fn register_reachable_occurrence_constructors(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		nested_iteration_expr: &Expression<'db>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			class_pattern_ref,
			root_pattern,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);
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
					nested_iteration_expr,
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
	}

	/// Record the root's own one-slot passthrough slice.
	///
	/// A later contribution to the same class needs this root's end offset in
	/// `contribution_end_map` — a subclass root's projection passthrough, or
	/// `project_class_identity` on a later constructor — so chain the root
	/// exactly like a collection slice, gated on such a contribution existing
	/// so single-root models do not grow unused `_root_start`/`_root_end`
	/// declarations.
	fn push_root_passthrough_slice(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		pending_root_collection: &Option<PendingRootCollection<'db>>,
		pending_slices: &mut Vec<PendingSlice<'db>>,
	) {
		let &NewRootContext {
			item,
			data,
			types,
			item_ty,
			class_pattern_ref,
			root_occurrence,
			..
		} = cx;
		let collector = ExpressionCollector::new(self, data, item, types);
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
				.objects
				.plan
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
	}

	/// Lower a singular root (`new C`, `var new C`, `var opt new C`), whose
	/// potential block is exactly one slot built from the declaration's own
	/// initialiser.
	fn lower_singular_new_root(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		inst: &VarType,
		opt: &OptType,
	) -> (
		Declaration<'db>,
		Expression<'db>,
		Option<PendingRootCollection<'db>>,
	) {
		let &NewRootContext {
			ty,
			item,
			data,
			types,
			d,
			top_level,
			class_pattern_ref,
			root_occurrence,
			enum_member_id,
			ref class_and_decl_name,
			..
		} = cx;
		let mut collector = ExpressionCollector::new(self, data, item, types);
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
				Domain::unbounded(collector.parent.db, item, Ty::par_int(collector.parent.db)),
				Domain::unbounded(collector.parent.db, item, lowered_input_record_ty),
			);
			let mut array_decl = Declaration::new(true, array_dom);
			let inputs_name = format!("{}_inputs", class_and_decl_name);
			array_decl.set_name(Identifier::new(collector.parent.db, inputs_name));
			let input_value = if let Some(rhs) = d.definition {
				let collected = collector.collect_expression(rhs);
				if opt_new_input {
					collector
						.parent
						.lower_opt_new_input_value(item, collected, input_record_ty)
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
			collector.parent.objects.plan.contributions_by_occurrence[&root_occurrence].clone();
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
						[collector.parent.objects.class_map[&contribution.target_class].class_set]
						.ty()
						.inst(collector.parent.db)
						== Some(VarType::Var)
				}),
				"var opt new occurrence has a par actual-set declaration \
				 among its contributions; var_actual_set_classes is too \
				 narrow"
			);
			let direct_class_set = collector.parent.objects.class_map[&class_pattern_ref].class_set;
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
					.objects
					.opt_contribution_slots
					.insert((contribution.target_class, contribution.constructor_index));
				let _ = collector
					.parent
					.objects
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
					collector.parent.objects.class_map[&contribution.target_class].class_enum;
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
					collector.parent.objects.class_map[&contribution.target_class].class_set;
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
				.objects
				.plan
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
				let presence_id = collector
					.parent
					.model
					.add_constraint(ConstraintItem::new(Constraint::new(false, presence), item));
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
				let value_id = collector
					.parent
					.model
					.add_constraint(ConstraintItem::new(Constraint::new(false, value_imp), item));
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
					collector.parent.objects.class_map[&contribution.target_class].class_enum;
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

	/// Lower a collection root (`set of new C`, `var set(c) of new C`): size the
	/// potential block, build the inputs array and the user-named declaration,
	/// and return the pending contribution the caller finishes once the class
	/// graph has been walked.
	fn lower_set_of_new_root(
		&mut self,
		cx: &NewRootContext<'db, '_>,
		cardinality: &Option<shackle_hir::ExpressionId<'db>>,
		inst: &VarType,
	) -> (
		Declaration<'db>,
		Expression<'db>,
		Option<PendingRootCollection<'db>>,
	) {
		let &NewRootContext {
			ty,
			item,
			data,
			types,
			d,
			top_level,
			class_pattern_ref,
			root_occurrence,
			ref class_and_decl_name,
			..
		} = cx;
		let mut collector = ExpressionCollector::new(self, data, item, types);
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
							function: collector.parent.ids.functions.max.into(),
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
				Domain::unbounded(collector.parent.db, item, Ty::par_int(collector.parent.db))
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
				collector
					.parent
					.build_class_storage_record_domain(class_pattern_ref, elem_ty, item)
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
		let contribution_expr = storage_idx.map(|idx| alloc_expression(idx, &collector, item));
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
						function: collector.parent.ids.functions.length.into(),
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
					function: collector.parent.ids.functions.length.into(),
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
}
