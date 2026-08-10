//! Lowering of `class` items themselves.
//!
//! Emits the per-class declarations (`<C>_objects`, the actual set), the class
//! body's constraints and computed-attribute definitions as realised-set
//! foralls, and the class invariants that carry relocated field domains and
//! nested set cardinalities.

use rustc_hash::FxHashSet;
use shackle_hir::{ClassMember, Item, PatternTy, ids::PatternRef};
use shackle_ty::Ty;

use super::{ClassBodyConstraint, StorageFieldDecl};
use crate::{
	lower::{
		ItemCollector, LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db> ItemCollector<'db> {
	pub(in crate::lower) fn add_class_objects_decl(
		&self,
		class_item: Item<'db>,
		class_objects_name: Identifier<'db>,
	) -> DeclarationItem<'db> {
		let class_record_ty = match class_item {
			Item::Class(sc) => {
				let fields = self.class_storage_fields_for_domain(PatternRef::new(
					self.db,
					class_item,
					sc.class(self.db).pattern,
				));
				Ty::array(self.db, Ty::par_int(self.db), Ty::record(self.db, fields)).unwrap()
			}
			_ => unreachable!(),
		};
		let mut class_objects_decl = Declaration::new(
			true,
			Domain::unbounded(self.db, class_item, class_record_ty),
		);
		class_objects_decl.set_name(class_objects_name);

		DeclarationItem::new(class_objects_decl, class_item)
	}

	/// Second predeclare phase: rebuild each `<C>_objects` declaration's
	/// storage-record domain now that every class is registered.
	///
	/// Classes are predeclared in topological item order, but class
	/// *reference* fields may form cycles (`Seat` ↔ `Handrail`) for which no
	/// order exists: `substitute_class_with_potential_enum` then leaves the
	/// not-yet-registered `Class<X>` fields of whichever class predeclares
	/// first unsubstituted. Rebuilding after all classes are registered (and
	/// before any item is collected, so no expression has frozen the stale
	/// type yet) makes the storage record independent of predeclare order.
	pub(in crate::lower) fn repair_predeclared_class_objects_domains(&mut self) {
		let entries = self
			.objects
			.class_map
			.iter()
			.map(|(class_pattern, info)| (*class_pattern, info.class_objects))
			.collect::<Vec<_>>();
		for (class_pattern, class_objects) in entries {
			let fields = self.class_storage_fields_for_domain(class_pattern);
			let record_ty =
				Ty::array(self.db, Ty::par_int(self.db), Ty::record(self.db, fields)).unwrap();
			if self.model[class_objects].ty() == record_ty {
				continue;
			}
			let origin = class_pattern.item(self.db);
			self.model[class_objects].set_domain(Domain::unbounded(self.db, origin, record_ty));
		}
	}

	pub(in crate::lower) fn collect_class(&mut self, it: shackle_hir::ClassItem<'db>) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_pattern = PatternRef::new(self.db, item, c.pattern);
		self.predeclare_class(it);
		// Class-body constraints lowered over the realised class set: explicit
		// `constraint` items are emitted here. The defining equation of a
		// computed attribute (`forall(this in <C>)(this.<attr> = <rhs>)`) is
		// DEFERRED to `finish()` and only emitted for classes with a
		// contribution that does NOT alias-define its defined fields (the
		// gated forall-drop): once every contribution to the class is
		// engine-reconstructed, the equation holds by construction on
		// realised objects and the forall is redundant.
		// NB: class-level annotations (`c.annotations`) are intentionally not
		// lowered yet — a class has no single MiniZinc output construct to carry
		// them. They are still parsed and type-checked; emitting them is a
		// follow-up.
		for class_item in c.items.iter() {
			match class_item {
				ClassMember::Constraint(ct) => {
					let body = ClassBodyConstraint::Constraint {
						expression: ct.expression,
						annotations: ct.annotations.to_vec(),
					};
					self.emit_class_body_constraint(item, &body);
				}
				ClassMember::Declaration(d) => {
					if let Some(value) = d.definition
						&& let Some(attribute) = c.data()[d.pattern].identifier()
					{
						self.objects.pending_class_definition_foralls.push((
							class_pattern,
							item,
							attribute,
							value,
						));
					}
				}
			}
		}
		self.emit_nested_set_cardinality_class_invariants(it, class_pattern);
		self.emit_relocated_domain_class_invariants(it, class_pattern);
		self.emit_dependent_domain_conformance_assertions(it);
	}

	/// Emit `forall(this in <C>)(assert(<conformance>, "..."))` for every
	/// PAR input-supplied field whose declared type carries an
	/// attribute-referencing domain (`1..l: x`, `array [1..l] of 0..hi: xs`).
	/// Those fields route through the unbounded storage-record path
	/// (`field_domain_references_attribute`), so without this assertion the
	/// supplied data is never checked against the dependent domain — a
	/// wrong-length array only errors if an out-of-range index is actually
	/// read, and an out-of-domain scalar passes silently. VAR fields need no
	/// assertion: their per-object domain is enforced by the reconstruction
	/// let-mint (`var 1..l: x` enumerates exactly `1..l`). Computed fields
	/// are alias-defined and skipped. Set-typed and multi-dimension shapes
	/// have no enforceable check here yet.
	pub(in crate::lower) fn emit_dependent_domain_conformance_assertions(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let types = item.types(self.db);
		let mut pending: Vec<ClassBodyConstraint<'db>> = Vec::new();
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			if d.definition.is_some() {
				continue;
			}
			let Some(attribute) = c.data()[d.pattern].identifier() else {
				continue;
			};
			if !self.field_domain_references_attribute(item, d.declared_type) {
				continue;
			}
			// Only par, non-opt fields: `assert` needs a par condition, and
			// var fields are already mint-enforced. (Var-reached classes
			// reject dependent domains at validation, so par-ness here is
			// the declared par-ness.)
			let field_ty = match &types[d.pattern] {
				PatternTy::Variable(ty) => *ty,
				_ => continue,
			};
			if !field_ty.known_par(self.db) || field_ty.opt(self.db) == Some(OptType::Opt) {
				continue;
			}
			// Only shapes with an enforceable check (see doc comment).
			let checkable = match &c.data()[d.declared_type] {
				shackle_hir::Type::Bounded { .. } => true,
				shackle_hir::Type::Array {
					dimensions,
					element,
					..
				} => {
					matches!(&c.data()[*dimensions], shackle_hir::Type::Bounded { .. })
						|| matches!(&c.data()[*element], shackle_hir::Type::Bounded { .. })
				}
				_ => false,
			};
			if !checkable {
				continue;
			}
			pending.push(ClassBodyConstraint::DomainConformance {
				attribute,
				declared_type: d.declared_type,
			});
		}
		for conformance in pending {
			self.emit_class_body_constraint(item, &conformance);
		}
	}

	/// Emit one class-body constraint quantified over the realised class set:
	/// `forall(this in <C>)(<body>)` — either an explicit class `constraint`
	/// expression or a computed attribute's defining equation
	/// `this.<attr> = <rhs>`. Bare attribute references resolve to per-object
	/// `<C>_objects` projections via let-bound field aliases. Definition
	/// bodies are emitted from `finish()` (see the gated forall-drop in
	/// `collect_class`), so this method derives everything from `item`.
	pub(in crate::lower) fn emit_class_body_constraint(
		&mut self,
		item: Item<'db>,
		class_body: &ClassBodyConstraint<'db>,
	) {
		let Item::Class(class_ref) = item else {
			unreachable!()
		};
		let c = class_ref.class(self.db);
		let class_pattern = PatternRef::new(self.db, item, c.pattern);
		let class_info = &self.objects.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		let class_constraint_fields = self.class_constraint_fields(item);
		{
			let scan_exprs: Vec<shackle_hir::ExpressionId<'db>> = match class_body {
				ClassBodyConstraint::Constraint { expression, .. } => vec![*expression],
				ClassBodyConstraint::Definition { value, .. } => vec![*value],
				// The sibling references live in the declared type's domain
				// expressions (`array [1..l] of 0..hi`).
				ClassBodyConstraint::DomainConformance { declared_type, .. } => {
					shackle_hir::Type::walk(*declared_type, c.data())
						.filter_map(|t| match &c.data()[t] {
							shackle_hir::Type::Bounded { domain, .. } => Some(*domain),
							_ => None,
						})
						.collect()
				}
			};
			let this_ty = match &types[c.this_pattern] {
				PatternTy::Variable(ty) => *ty,
				_ => unreachable!(),
			};
			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let previous_resolution = self.resolutions.insert(
				PatternRef::new(self.db, item, c.this_pattern),
				LoweredIdentifier::ResolvedIdentifier(this_decl_idx.into()),
			);

			// Only fields actually used as bare identifiers (not `this.x`)
			// need a projection alias materialised — otherwise the Let
			// below would dump dead bindings into every class-constraint.
			let referenced_field_patterns: FxHashSet<PatternRef<'db>> = {
				let field_pattern_set: FxHashSet<PatternRef<'db>> =
					class_constraint_fields.iter().map(|(p, _)| *p).collect();
				let mut referenced = FxHashSet::default();
				for scan_expr in scan_exprs.iter().copied() {
					for sub in shackle_hir::Expression::walk(scan_expr, c.data()) {
						if let shackle_hir::Expression::Identifier(_) = &c.data()[sub]
							&& let Some(res) = types.name_resolution(sub)
							&& field_pattern_set.contains(&res)
						{
							let _ = referenced.insert(res);
						}
					}
				}
				referenced
			};

			let field_aliases = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				class_constraint_fields
					.iter()
					.filter(|(field_pattern, _)| referenced_field_patterns.contains(field_pattern))
					.map(|(field_pattern, field_name)| {
						let this_expr = alloc_expression(this_decl_idx, &collector, item);
						let class_objects_expr =
							alloc_expression(class_objects_idx, &collector, item);
						let object_index = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.enum2int.into(),
								arguments: vec![this_expr],
							},
							&collector,
							item,
						);
						let object_record = collector.introduce_array_access(
							class_objects_expr,
							object_index,
							item,
						);
						let field_expr = alloc_expression(
							RecordAccess {
								record: Box::new(object_record),
								field: *field_name,
							},
							&collector,
							item,
						);
						(*field_pattern, *field_name, field_expr)
					})
					.collect::<Vec<_>>()
			};
			let mut previous_field_resolutions = Vec::new();
			let mut alias_decl_idxs = Vec::new();
			for (field_pattern, field_name, field_expr) in field_aliases {
				let mut field_decl = Declaration::from_expression(self.db, false, field_expr);
				// Substitute `Class<X>` -> `X_potential` in the alias domain: a
				// var-reached class field projects as `var set of Class<X>` /
				// `var Class<X>`, which would render the (var) actual set as a
				// type-inst domain (`var set of Seat`), rejected by MiniZinc. The
				// potential enum is equivalent under `lowered_ty_matches`.
				let dom_origin = field_decl.domain().origin();
				let substituted = self.substitute_class_with_potential_enum(field_decl.ty());
				field_decl.set_domain(Domain::unbounded(self.db, dom_origin, substituted));
				field_decl.set_name(field_name);
				let field_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(field_decl, item));
				let old = self.resolutions.insert(
					field_pattern,
					LoweredIdentifier::ResolvedIdentifier(field_decl_idx.into()),
				);
				previous_field_resolutions.push((field_pattern, old));
				alias_decl_idxs.push(field_decl_idx);
			}
			let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
			let constraint_expr = match class_body {
				ClassBodyConstraint::Constraint { expression, .. } => {
					collector.collect_expression(*expression)
				}
				ClassBodyConstraint::Definition { attribute, value } => {
					// Build `this.<attr> = <definition>`. The LHS is the per-object
					// storage projection; the RHS is the collected definition, with
					// bare attribute references resolved to their projections via
					// the aliases set up above.
					let rhs = collector.collect_expression(*value);
					let this_expr = alloc_expression(this_decl_idx, &collector, item);
					let class_objects_expr = alloc_expression(class_objects_idx, &collector, item);
					let object_index = alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.enum2int.into(),
							arguments: vec![this_expr],
						},
						&collector,
						item,
					);
					let object_record =
						collector.introduce_array_access(class_objects_expr, object_index, item);
					let lhs = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: *attribute,
						},
						&collector,
						item,
					);
					alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.eq.into(),
							arguments: vec![lhs, rhs],
						},
						&collector,
						item,
					)
				}
				ClassBodyConstraint::DomainConformance {
					attribute,
					declared_type,
				} => {
					// The per-object field projection (same shape as the
					// Definition LHS above).
					let this_expr = alloc_expression(this_decl_idx, &collector, item);
					let class_objects_expr = alloc_expression(class_objects_idx, &collector, item);
					let object_index = alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.enum2int.into(),
							arguments: vec![this_expr],
						},
						&collector,
						item,
					);
					let object_record =
						collector.introduce_array_access(class_objects_expr, object_index, item);
					let field_proj = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: *attribute,
						},
						&collector,
						item,
					);
					// One check per enforceable declared-domain part: value
					// membership for a scalar `Bounded`, index-set equality
					// for a single-dimension array's `Bounded` dimension,
					// element membership for a `Bounded` array element (the
					// whole field domain went unbounded in storage, so even
					// non-dependent parts lost their enforcement).
					let mut checks: Vec<Expression<'db>> = Vec::new();
					match &c.data()[*declared_type] {
						shackle_hir::Type::Bounded { domain, .. } => {
							let dom = collector.collect_expression(*domain);
							checks.push(alloc_expression(
								LookupCall {
									function: collector.parent.ids.functions.in_.into(),
									arguments: vec![field_proj.clone(), dom],
								},
								&collector,
								item,
							));
						}
						shackle_hir::Type::Array {
							dimensions,
							element,
							..
						} => {
							if let shackle_hir::Type::Bounded { domain, .. } =
								&c.data()[*dimensions]
							{
								let dom = collector.collect_expression(*domain);
								let index_set_expr = alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.index_set.into(),
										arguments: vec![field_proj.clone()],
									},
									&collector,
									item,
								);
								checks.push(alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.eq.into(),
										arguments: vec![index_set_expr, dom],
									},
									&collector,
									item,
								));
							}
							if let shackle_hir::Type::Bounded { domain, .. } = &c.data()[*element] {
								let dom = collector.collect_expression(*domain);
								let elem_ty = field_proj
									.ty()
									.elem_ty(collector.parent.db)
									.expect("array field projection has an element type");
								let mut e_decl = Declaration::new(
									false,
									Domain::unbounded(collector.parent.db, item, elem_ty),
								);
								e_decl.set_name(Identifier::new(collector.parent.db, "e"));
								let e_decl_idx = collector
									.parent
									.model
									.add_declaration(DeclarationItem::new(e_decl, item));
								let e_expr = alloc_expression(e_decl_idx, &collector, item);
								let membership = alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.in_.into(),
										arguments: vec![e_expr, dom],
									},
									&collector,
									item,
								);
								let compr = alloc_expression(
									ArrayComprehension::new(
										[Generator::Iterator {
											declarations: vec![e_decl_idx],
											collection: field_proj.clone(),
											where_clause: None,
										}],
										membership,
									),
									&collector,
									item,
								);
								checks.push(alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.forall.into(),
										arguments: vec![compr],
									},
									&collector,
									item,
								));
							}
						}
						_ => {}
					}
					let mut checks = checks.into_iter();
					let mut cond = checks
						.next()
						.expect("conformance emission called for an uncheckable declared type");
					for check in checks {
						cond = alloc_expression(
							LookupCall {
								function: collector.parent.ids.functions.and.into(),
								arguments: vec![cond, check],
							},
							&collector,
							item,
						);
					}
					let message = format!(
						"the value supplied for attribute `{}` of class `{}` does not agree \
						 with its declared type",
						attribute.pretty_print(collector.parent.db),
						class_pattern
							.identifier(collector.parent.db)
							.map(|i| i.lookup(collector.parent.db))
							.unwrap_or_default(),
					);
					let message_expr = alloc_expression(
						StringLiteral::new(collector.parent.db, message),
						&collector,
						item,
					);
					alloc_expression(
						LookupCall {
							function: collector.parent.ids.functions.assert_.into(),
							arguments: vec![cond, message_expr],
						},
						&collector,
						item,
					)
				}
			};
			// Bind the per-field projection aliases as let-items inside
			// the comprehension body so bare field references (e.g.
			// `constraint x >= 1` in a class body) are in MZN scope.
			// Without the Let, the aliases are model-resident but never
			// reachable from the lowered output.
			let body_expr = if alias_decl_idxs.is_empty() {
				constraint_expr
			} else {
				alloc_expression(
					Let {
						items: alias_decl_idxs
							.iter()
							.copied()
							.map(LetItem::Declaration)
							.collect(),
						in_expression: Box::new(constraint_expr),
					},
					&collector,
					item,
				)
			};
			let quantified = alloc_expression(
				LookupCall {
					function: collector.parent.ids.functions.forall.into(),
					arguments: vec![alloc_expression(
						ArrayComprehension::new(
							[Generator::Iterator {
								declarations: vec![this_decl_idx],
								collection: alloc_expression(class_set_idx, &collector, item),
								where_clause: None,
							}],
							body_expr,
						),
						&collector,
						item,
					)],
				},
				&collector,
				item,
			);
			let mut constraint = Constraint::new(true, quantified);
			let body_annotations: &[shackle_hir::ExpressionId<'db>] = match class_body {
				ClassBodyConstraint::Constraint { annotations, .. } => annotations,
				ClassBodyConstraint::Definition { .. }
				| ClassBodyConstraint::DomainConformance { .. } => &[],
			};
			constraint.annotations_mut().extend(
				body_annotations
					.iter()
					.map(|ann| collector.collect_expression(*ann)),
			);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(constraint, item));

			if let Some(old) = previous_resolution {
				let _ = self
					.resolutions
					.insert(PatternRef::new(self.db, item, c.this_pattern), old);
			} else {
				let _ = self
					.resolutions
					.remove(&PatternRef::new(self.db, item, c.this_pattern));
			}
			for (field_pattern, old) in previous_field_resolutions {
				if let Some(old) = old {
					let _ = self.resolutions.insert(field_pattern, old);
				} else {
					let _ = self.resolutions.remove(&field_pattern);
				}
			}
			let _ = this_ty;
		}
	}

	/// A nested `set(<card>) of new` field carries a cardinality bound
	/// that is otherwise only used for `<child>_potential` universe
	/// sizing. Emit it as an implicit class invariant
	/// `forall(this in <C>)(card(this.<field>) in <card>)` over the
	/// realised class set — iterating the actual set (not the potential
	/// storage) keeps unrealised potentials, whose field defaults to the
	/// empty set, out of the constraint. This covers var-declared fields
	/// (always) and par-declared fields of var-reached classes (whose
	/// storage-iterating walker emission is suppressed: it wrongly
	/// constrained unrealised potentials — a `var opt new` root's
	/// `absent(a)` was unsatisfiable — and was missing entirely on
	/// `var set of new` roots). Par-declared fields of par-reached classes
	/// keep the walker's `emit_nested_cardinality_constraint` emission,
	/// where every iterated instance is realised.
	pub(in crate::lower) fn emit_nested_set_cardinality_class_invariants(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
		class_pattern: PatternRef<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_info = &self.objects.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			let shackle_hir::Type::Set {
				inst,
				cardinality: Some(cardinality),
				..
			} = &c.data()[d.declared_type]
			else {
				continue;
			};
			let Some(field_ident) = c.data()[d.pattern].identifier() else {
				continue;
			};
			if c.data()[d.declared_type].get_new_class(c.data()).is_some() {
				if *inst != VarType::Var
					&& !self
						.objects
						.plan
						.var_reached_classes
						.contains(&class_pattern)
				{
					continue;
				}
			} else {
				// Set-cardinality relocation: a non-`new` card-bounded set
				// field whose bound was dropped from the storage record domain
				// (`field_relocates_set_card`) — this realised-set invariant is
				// then the ONLY site enforcing the bound, keeping unrealised
				// slots (pinned/witnessed to `{}`) out of it. Non-relocated
				// fields keep their bound in the record domain and need no
				// invariant.
				let decl = StorageFieldDecl {
					ident: field_ident,
					pattern: PatternRef::new(self.db, item, d.pattern),
					definition: d.definition,
					declared_type: d.declared_type,
					owner: item,
				};
				if !self.field_relocates_set_card(&decl) {
					continue;
				}
			}
			let cardinality = *cardinality;
			let cardinality = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				collector.collect_expression(cardinality)
			};

			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let this_expr = Expression::new(self.db, &self.model, item, this_decl_idx);
			let class_objects_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_objects_idx),
			);
			let object_index = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.enum2int.into(),
					arguments: vec![this_expr],
				},
			);
			let object_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![class_objects_expr, object_index],
				},
			);
			let field_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(object_at_this),
					field: field_ident,
				},
			);
			let class_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_set_idx),
			);
			self.emit_nested_cardinality_constraint(
				item,
				vec![Generator::Iterator {
					declarations: vec![this_decl_idx],
					collection: class_set_expr,
					where_clause: None,
				}],
				field_at_this,
				cardinality,
			);
		}
	}

	/// Re-impose a relocated defined field's declared domain on realised
	/// objects only:
	/// `forall(this in <C>)(<C>_objects[enum2int(this)].<f> in <dom>)`.
	/// The domain was relaxed out of the shared element record
	/// (`build_class_storage_record_domain`), so this invariant is the ONLY
	/// site enforcing it — unrealised slots hold the (total) RHS value at
	/// their pinned frees, unconstrained, which is what keeps models like
	/// `var 3..4: z = x1 + x2` under `card(as) = 0` satisfiable. Emitted on
	/// the field's OWNER class: subclass objects are members of the owner's
	/// realised set, so one invariant covers every contribution.
	pub(in crate::lower) fn emit_relocated_domain_class_invariants(
		&mut self,
		it: shackle_hir::ClassItem<'db>,
		class_pattern: PatternRef<'db>,
	) {
		let item: Item<'db> = it.into();
		let c = it.class(self.db);
		let class_info = &self.objects.class_map[&class_pattern];
		let class_enum_ref = self.model[class_info.class_enum].enum_type();
		let class_objects_idx = class_info.class_objects;
		let class_set_idx = class_info.class_set;
		let types = item.types(self.db);
		for class_item in c.items.iter() {
			let ClassMember::Declaration(d) = class_item else {
				continue;
			};
			let Some(field_ident) = c.data()[d.pattern].identifier() else {
				continue;
			};
			let decl = StorageFieldDecl {
				ident: field_ident,
				pattern: PatternRef::new(self.db, item, d.pattern),
				definition: d.definition,
				declared_type: d.declared_type,
				owner: item,
			};
			if !self.field_relocates_declared_domain(&decl) {
				continue;
			}
			let shackle_hir::Type::Bounded { domain, .. } = &c.data()[d.declared_type] else {
				continue;
			};
			let domain_expr = {
				let mut collector = ExpressionCollector::new(self, c.data(), item, &types);
				collector.collect_expression(*domain)
			};

			let mut this_decl = Declaration::new(
				false,
				Domain::unbounded(self.db, item, Ty::par_enum(self.db, class_enum_ref)),
			);
			this_decl.set_name(Identifier::new(self.db, "this"));
			let this_decl_idx = self
				.model
				.add_declaration(DeclarationItem::new(this_decl, item));
			let this_expr = Expression::new(self.db, &self.model, item, this_decl_idx);
			let class_objects_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_objects_idx),
			);
			let object_index = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.enum2int.into(),
					arguments: vec![this_expr],
				},
			);
			let object_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![class_objects_expr, object_index],
				},
			);
			let field_at_this = Expression::new(
				self.db,
				&self.model,
				item,
				RecordAccess {
					record: Box::new(object_at_this),
					field: field_ident,
				},
			);
			let membership = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.in_.into(),
					arguments: vec![field_at_this, domain_expr],
				},
			);
			let class_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(class_set_idx),
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
						ArrayComprehension::new(
							[Generator::Iterator {
								declarations: vec![this_decl_idx],
								collection: class_set_expr,
								where_clause: None,
							}],
							membership,
						),
					)],
				},
			);
			let constraint = Constraint::new(true, quantified);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(constraint, item));
		}
	}

	pub(in crate::lower) fn class_constraint_fields(
		&self,
		class_item: Item<'db>,
	) -> Vec<(PatternRef<'db>, Identifier<'db>)> {
		fn collect_fields<'db>(
			lowerer: &ItemCollector<'db>,
			class_item: Item<'db>,
			fields: &mut Vec<(PatternRef<'db>, Identifier<'db>)>,
		) {
			let Item::Class(class_ref) = class_item else {
				return;
			};
			let class = class_ref.class(lowerer.db);
			let types = class_item.types(lowerer.db);
			if let Some(base) = class.extends.and_then(|base| types.name_resolution(base)) {
				collect_fields(lowerer, base.item(lowerer.db), fields);
			}
			for field_item in class.items.iter() {
				if let ClassMember::Declaration(d) = field_item {
					for pattern in shackle_hir::Pattern::identifiers(d.pattern, class.data()) {
						let pattern_ref = PatternRef::new(lowerer.db, class_item, pattern);
						if let Some(identifier) = pattern_ref.identifier(lowerer.db) {
							fields.push((pattern_ref, identifier));
						}
					}
				}
			}
		}

		let mut fields = Vec::new();
		collect_fields(self, class_item, &mut fields);
		fields
	}

	/// Walk a class's storage-field declarations in storage order
	/// (superclass fields first, matching `class_storage_fields`), capturing
	/// for each field the HIR `Declaration` data the reconstruction
	/// comprehension needs: the field pattern (for sibling resolution), the
	/// optional RHS `definition` (a computed attribute), the `declared_type`
	/// (so a var field's per-object domain can be re-collected), and the
	/// owning item (a superclass item for inherited fields, so the RHS/domain
	/// is collected against the right `ItemData`).
	pub(in crate::lower) fn class_storage_field_decls(
		&self,
		class_item: Item<'db>,
	) -> Vec<StorageFieldDecl<'db>> {
		fn collect<'db>(
			lowerer: &ItemCollector<'db>,
			class_item: Item<'db>,
			out: &mut Vec<StorageFieldDecl<'db>>,
		) {
			let Item::Class(class_ref) = class_item else {
				return;
			};
			let class = class_ref.class(lowerer.db);
			let types = class_item.types(lowerer.db);
			if let Some(base) = class.extends.and_then(|base| types.name_resolution(base)) {
				collect(lowerer, base.item(lowerer.db), out);
			}
			for field_item in class.items.iter() {
				if let ClassMember::Declaration(d) = field_item {
					let pattern = PatternRef::new(lowerer.db, class_item, d.pattern);
					if let Some(ident) = pattern.identifier(lowerer.db) {
						out.push(StorageFieldDecl {
							ident,
							pattern,
							definition: d.definition,
							declared_type: d.declared_type,
							owner: class_item,
						});
					}
				}
			}
		}

		let mut fields = Vec::new();
		collect(self, class_item, &mut fields);
		fields
	}
}
