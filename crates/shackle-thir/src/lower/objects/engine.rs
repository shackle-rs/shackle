//! The reconstruction engine: assembling whole contribution expressions.
//!
//! One engine serves every contribution shape — top-level roots, flattened and
//! singular nested occurrences, and deep nesting — so par and var reach,
//! inheritance and nesting all share a single code path. Each contribution is
//! rebuilt from storage as an array comprehension over the occurrence's slots,
//! with per-field aliases supplied by `field_reconstruction`.

use rustc_hash::FxHashSet;
use shackle_hir::{
	Item,
	class_analysis::{LocalDomainSource, OccurrenceId, class_pattern_for},
	ids::PatternRef,
};
use shackle_ty::Ty;

use super::{
	EngineIdentityRule, EngineRealisationGuard, EngineRealisationTest, RootRealisationGuard,
};
use crate::{
	lower::{ItemCollector, LoweredIdentifier, expression::ExpressionCollector},
	*,
};

impl<'db> ItemCollector<'db> {
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
}
