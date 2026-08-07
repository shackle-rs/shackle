//! The `finish()` wave: materialising class actual-sets and the `<C>_objects`
//! storage arrays once every item has been collected.
//!
//! Runs in order: field-only-introduced classes get their actual set derived
//! from recorded parent-field introductions; top-level and unregistered
//! contributions are unioned into each class set (or emitted as a subset lower
//! bound for opt-reached classes); per-class `_objects` contributions are
//! concatenated; deferred computed-attribute foralls are emitted unless the
//! gated forall-drop applies; and unused potential objects are pinned to their
//! canonical defaults for symmetry breaking.

use shackle_hir::{
	class_analysis::{OccurrenceId, analyse_new_objects},
	ids::PatternRef,
};
use shackle_ty::Ty;

use crate::{
	lower::{ItemCollector, objects::ClassBodyConstraint},
	*,
};

impl<'db> ItemCollector<'db> {
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
