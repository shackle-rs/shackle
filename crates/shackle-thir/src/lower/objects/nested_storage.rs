//! Nested storage: deciding how a class-typed field's storage is populated.
//!
//! A nested `new` field either reads an existing child object's storage record
//! or must fresh-mint one. This module makes that decision per field, builds
//! the template record for a nested contribution, and selects the contribution
//! expression for the regime in play (par identity, var existence mint, or the
//! var nested storage engine).

use shackle_hir::{
	Item,
	class_analysis::{LocalDomainSource, OccurrenceId},
	ids::PatternRef,
};
use shackle_ty::Ty;

use super::{EngineIdentityRule, EngineRealisationGuard, EngineRealisationTest};
use crate::{lower::ItemCollector, *};

impl<'db> ItemCollector<'db> {
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
			.objects
			.plan
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
		let class_enum = self.objects.class_map[&target_class].class_enum;
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
}
