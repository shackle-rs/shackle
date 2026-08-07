//! Deriving each class's actual set from its contributions.
//!
//! A class set is the union of every piece that introduces objects into it:
//! top-level roots, parent-field introductions recorded one hop up, par
//! contribution block images, and membership-gated identity images projected
//! from a subclass. This module builds those per-contribution expressions and
//! emits the per-parent subset constraint tying a nested slice array back to
//! the child class's storage.

use rustc_hash::FxHashMap;
use shackle_hir::{
	Item,
	class_analysis::{OccurrenceId, analyse_new_objects},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use super::{FieldIntroduction, FieldIntroductionKind};
use crate::{lower::ItemCollector, *};

impl<'db> ItemCollector<'db> {
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
			.objects
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
		for occurrence_contributions in self.objects.plan.contributions_in_occurrence_order() {
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
			.objects
			.contribution_end_map
			.get(&(child_class, contribution_index))?;
		let start_expr = if contribution_index == 0 {
			Expression::new(self.db, &self.model, item, IntegerLiteral(1))
		} else {
			let previous_end = *self
				.objects
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
			self.objects.class_map[&child_class].class_enum,
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
			.objects
			.plan
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
					ResolvedIdentifier::Declaration(
						self.objects.class_map[&intro.parent_class].class_set,
					),
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
					self.objects.class_map[&child_class].class_enum,
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
			self.objects.class_map[&super_class].class_enum,
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
			.objects
			.plan
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
				ResolvedIdentifier::Declaration(self.objects.class_map[&direct_class].class_set),
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
}
