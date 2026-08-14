//! Realisation guards, domain relocation and totality analysis.
//!
//! A potential object's slot may not be realised, so reads of its fields are
//! wrapped in a realisation guard (`.. in <C>`) and unrealised slots take a
//! canonical default. This module decides when that guard can be elided, when a
//! field's declared domain or set cardinality must be relocated off the shared
//! storage element type onto a class invariant, and whether a defining
//! expression is provably total.

use shackle_hir::{Item, class_analysis::LocalDomainSource, ids::PatternRef};
use shackle_ty::{Ty, TyData};

use super::StorageFieldDecl;
use crate::{
	lower::{ItemCollector, expression::ExpressionCollector},
	source::Origin,
	*,
};

impl<'db> ItemCollector<'db> {
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
			.objects
			.plan
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
		self.objects
			.plan
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
			return self.substitute_class_with_potential_enum_domain(field_ty, origin);
		};
		let class_data = local.class(db).data();
		let shackle_hir::Type::Set { element, .. } = &class_data[declared_type] else {
			return self.substitute_class_with_potential_enum_domain(field_ty, origin);
		};
		let element = *element;
		let inst = subst.inst(db).unwrap_or(VarType::Var);
		let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
		let elem_ty = subst.elem_ty(db).unwrap_or(subst);
		let elem_domain = if subst != field_ty {
			// Class-element reference set: the element bound is the child's
			// `<Child>_potential` identity universe.
			self.substitute_class_with_potential_enum_domain(
				field_ty.elem_ty(db).unwrap_or(field_ty),
				origin,
			)
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
						// `::promise_total` is the author's declaration that the
						// function is defined everywhere, so take it and skip
						// both checks below — it covers the parameter domains
						// too, which is what the annotation means to MiniZinc.
						// Without this the whitelist only ever sees the
						// *bodyless* half of a stdlib pair: `card(set of $T)` is
						// a true builtin and counts, while its var twin
						// `card(var set of $$E) ::promise_total` fails the
						// domain check (`$$E` is a `Bounded` type-inst alias)
						// and then the body walk (a `let` with a domained
						// decision) — so the same attribute would count as total
						// or not depending only on whether its class is
						// var-reached.
						if self.function_promises_total(f) {
							todo.extend(c.arguments.iter().copied());
							continue;
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

	/// The bodyless-builtin total-ops whitelist: arithmetic that cannot
	/// overflow-trap (`+ - *`), `card`, `sum`/`exists`/`forall` (total on
	/// empty), set construction and set ops, comparisons, boolean ops.
	/// `div`/`mod`/`'[]'`/`min`/`max`/`deopt`/`assert`/`pow` are deliberately
	/// absent.
	/// Whether a function item carries `::promise_total`.
	fn function_promises_total(&self, f: shackle_hir::ir::item::FunctionItem<'db>) -> bool {
		let db = self.db;
		let function = f.function(db);
		let data = function.data();
		function.annotations.iter().any(|a| {
			matches!(
				&data[*a],
				shackle_hir::Expression::Identifier(i) if *i == self.ids.annotations.promise_total
			)
		})
	}

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
