//! Emission of nested occurrence contributions and their cardinality
//! constraints.
//!
//! Registers each nested `new` occurrence against its child class, emits the
//! per-parent contribution blocks, and constrains how many children each parent
//! slot realises — from a fixed par count up to a var collection's fallback
//! bound. Also builds the generator chains that walk a nested path from the
//! root down to the contributing field.

use shackle_hir::{
	Item, TypeResult,
	class_analysis::{LocalDomainSource, OccurrenceId, class_pattern_for},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use crate::{
	lower::{ItemCollector, expression::ExpressionCollector},
	*,
};

impl<'db> ItemCollector<'db> {
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
			.objects
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
			self.objects.plan.contributions_by_occurrence[&source_occurrence].clone();
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
						let child_enum = self.objects.class_map[&field_class].class_enum;
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
					.objects
					.plan
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
							.objects
							.plan
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
			let class_info = self.objects.class_map[&parent_class];
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
}
