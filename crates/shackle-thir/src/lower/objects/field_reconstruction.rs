//! Field-level reconstruction: rebuilding one field's value for a class
//! object out of storage.
//!
//! These builders produce the per-field expressions the reconstruction engine
//! assembles into whole contributions — reading the root's own storage record,
//! or projecting through a flattened nested occurrence — and guard each read on
//! the slot actually being realised.

use shackle_hir::{
	Item,
	class_analysis::{LocalDomainSource, class_pattern_for},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use crate::{lower::ItemCollector, *};

impl<'db> ItemCollector<'db> {
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
		let child_enum = self.objects.class_map[&child_class].class_enum;
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.plus.into(),
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
						function: self.ids.functions.plus.into(),
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
		let child_enum = self.objects.class_map[&field_class].class_enum;
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.plus.into(),
						arguments: vec![
							one_expr,
							Expression::new(
								self.db,
								&self.model,
								item,
								LookupCall {
									function: self.ids.functions.plus.into(),
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.length.into(),
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
						function: self.ids.functions.plus.into(),
						arguments: vec![previous_roots_count, previous_siblings_count],
					},
				);
				let ordinal_start = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.plus.into(),
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
						function: self.ids.functions.plus.into(),
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
}
