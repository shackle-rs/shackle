//! Lowering of domains

use shackle_hir::{
	Item, PatternTy,
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef},
};
use shackle_ty::{Ty, TyData};

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect a sub-domain at a non-root (element / field / dimension)
	/// position. Replaces `Class<X>` with `<X>_potential` (par enum) so
	/// user-written types like `var set of B` lower to `var set of
	/// B_potential` — the class actual set `B` is itself widened to
	/// `var set of B_potential` by the field-only `array_union` recipe,
	/// and MiniZinc rejects `var set of <var-set>` at type-check.
	/// Mirrors the storage-record substitution applied by
	/// `class_storage_fields_for_domain`. Outermost-class positions
	/// (e.g. `A: a`, `var new A: a`) are unchanged because callers do
	/// not route through this helper.
	pub(in crate::lower) fn collect_element_domain(
		&mut self,
		t: shackle_hir::TypeId<'db>,
		ty: Ty<'db>,
		is_type_alias: bool,
	) -> Domain<'db> {
		let db = self.parent.db;
		if let TyData::Class(_, _, class_ref) = ty.lookup(db)
			&& let Some(class_pattern) = class_pattern_for(db, *class_ref)
			&& self.parent.objects.class_map.contains_key(&class_pattern)
		{
			let substituted = self.parent.substitute_class_with_potential_enum(ty);
			let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(t));
			return Domain::unbounded(self.parent.db, origin, substituted);
		}
		self.collect_domain(t, ty, is_type_alias)
	}

	// Collect a domain from a user ascribed type
	pub(in crate::lower) fn collect_domain(
		&mut self,
		t: shackle_hir::TypeId<'db>,
		ty: Ty<'db>,
		is_type_alias: bool,
	) -> Domain<'db> {
		let db = self.parent.db;
		let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(t));
		match (&self.data[t], ty.lookup(db)) {
			(shackle_hir::Type::Bounded { domain, .. }, _) => {
				if let Some(res) = self.types.name_resolution(*domain) {
					let res_item = res.item(db);
					let res_types = res_item.types(db);
					let res_data = res_item.data(db);
					match &res_types[res.pattern(db)] {
						// Identifier is actually a type, not a domain expression
						PatternTy::TyVar(_) => {
							return Domain::unbounded(self.parent.db, origin, ty);
						}
						PatternTy::TypeAlias { .. } => match res.item(db) {
							Item::TypeAlias(ta) => {
								let mut c = ExpressionCollector::new(
									self.parent,
									res_data,
									res.item(db),
									&res_types,
								);
								return c.collect_domain(ta.type_alias(db).aliased_type, ty, true);
							}
							_ => unreachable!(),
						},
						// A var-reached class used as an outermost domain (e.g.
						// `var A: a`). Its actual set `A` is `var set of
						// A_potential`, which MiniZinc rejects as a type-inst
						// domain. Substitute to the par potential enum and emit
						// an unbounded domain (`var A_potential: a`), mirroring
						// `collect_element_domain` for nested class positions.
						PatternTy::ClassDecl { .. }
							if self.parent.objects.plan.var_reached_classes.contains(&res) =>
						{
							let substituted = self.parent.substitute_class_with_potential_enum(ty);
							return Domain::unbounded(self.parent.db, origin, substituted);
						}
						_ => (),
					}
				}
				if is_type_alias {
					// Replace expressions with identifiers pointing to declarations for those expressions
					let er = ExpressionRef::new(db, self.item, *domain);
					let origin =
						EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(*domain));
					Domain::bounded(
						db,
						origin,
						ty.inst(db).unwrap(),
						ty.opt(db).unwrap(),
						alloc_expression(self.parent.type_alias_expressions[&er], self, origin),
					)
				} else {
					let e = self.collect_expression(*domain);
					Domain::bounded(db, origin, ty.inst(db).unwrap(), ty.opt(db).unwrap(), e)
				}
			}
			(
				shackle_hir::Type::Array {
					dimensions,
					element,
					..
				},
				TyData::Array {
					opt,
					dim: d,
					element: el,
				},
			) => Domain::array(
				db,
				origin,
				*opt,
				self.collect_element_domain(*dimensions, *d, is_type_alias),
				self.collect_element_domain(*element, *el, is_type_alias),
			),
			(
				shackle_hir::Type::Set {
					cardinality,
					element,
					..
				},
				TyData::Set(inst, opt, e),
			) => {
				let cardinality = cardinality.map(|c| self.collect_expression(c));
				Domain::set_with_card(
					db,
					origin,
					*inst,
					*opt,
					cardinality,
					self.collect_element_domain(*element, *e, is_type_alias),
				)
			}
			(shackle_hir::Type::Tuple { fields, .. }, TyData::Tuple(opt, fs)) => Domain::tuple(
				db,
				origin,
				*opt,
				fs.iter()
					.zip(fields.iter())
					.map(|(ty, f)| self.collect_element_domain(*f, *ty, is_type_alias))
					.collect::<Vec<_>>(),
			),
			(shackle_hir::Type::Record { fields, .. }, TyData::Record(opt, fs)) => Domain::record(
				db,
				origin,
				*opt,
				fs.iter()
					.map(|(i, ty)| {
						let ident = Identifier(*i);
						(
							ident,
							self.collect_element_domain(
								fields
									.iter()
									.find_map(|(p, t)| {
										if self.data[*p].identifier().unwrap() == ident {
											Some(*t)
										} else {
											None
										}
									})
									.unwrap(),
								*ty,
								is_type_alias,
							),
						)
					})
					.collect::<Vec<_>>(),
			),
			(
				shackle_hir::Type::New {
					inst: _,
					opt: _,
					domain,
				},
				_,
			) => {
				// A `new` type in a domain position. Only the shapes routed
				// here by declaration collection are handled; the rest of the
				// `new` lowering happens in `collect_new_declaration`.
				let (e, new_inst, new_opt) = match self.item {
					Item::Declaration(decl_item) => {
						let decl = decl_item.declaration(db);
						let item_ty_idx = decl.declared_type;
						let class_pattern_ref = self.types.name_resolution(*domain).unwrap();

						let class_enum =
							self.parent.objects.class_map[&class_pattern_ref].class_enum;
						let idx = self.parent.model[class_enum]
							.definition()
							.map(|constructors| constructors.len())
							.unwrap_or(0);

						let constr_name = format!(
							"{}_{}",
							class_pattern_ref.identifier(db).unwrap().pretty_print(db),
							decl.data()[decl.pattern]
								.identifier()
								.unwrap()
								.pretty_print(db)
						);
						let enum_member_id = EnumMemberId::new(class_enum, idx as u32);
						let item_ty = &decl.data()[item_ty_idx];
						match item_ty {
							shackle_hir::Type::Set {
								inst: VarType::Par, ..
							} => {
								todo!("Handle new A: x with set(d) of new A: x");
							}
							shackle_hir::Type::Set {
								cardinality: Some(c),
								inst: VarType::Var,
								..
							} => {
								let card_expr = self.collect_expression(*c);

								let origin = card_expr.origin();
								let max_call = LookupCall {
									function: self.parent.ids.builtins.max.into(),
									arguments: vec![card_expr],
								};
								let max_exp = Expression::new(
									self.parent.db,
									&self.parent.model,
									origin,
									max_call,
								);
								let one_exp = Expression::new(
									self.parent.db,
									&self.parent.model,
									origin,
									IntegerLiteral(1),
								);
								let dotdot_call = LookupCall {
									function: self.parent.ids.functions.dot_dot.into(),
									arguments: vec![one_exp, max_exp],
								};
								let dotdot_exp = Expression::new(
									self.parent.db,
									&self.parent.model,
									origin,
									dotdot_call,
								);

								let enum_constr_domain = Domain::bounded(
									self.parent.db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									dotdot_exp.clone(),
								);

								let decl = Declaration::new(false, enum_constr_domain);
								let idx = self
									.parent
									.model
									.add_declaration(DeclarationItem::new(decl, origin));

								self.parent.model[class_enum].add_constructor(Constructor {
									name: Some(Identifier::new(self.parent.db, constr_name)),
									parameters: Some(vec![idx]),
								});

								let call = Call {
									function: Callable::EnumConstructor(enum_member_id),
									arguments: vec![dotdot_exp],
								};
								let call_expr = alloc_expression(call, self, origin);
								(call_expr, VarType::Par, OptType::NonOpt)
							}
							_ => todo!("Handle other cases of new A: x"),
						}
					}
					Item::Class(_) => todo!(),
					_ => unreachable!(),
				};

				Domain::bounded(db, origin, new_inst, new_opt, e)
			}
			_ => Domain::unbounded(self.parent.db, origin, ty),
		}
	}
}
