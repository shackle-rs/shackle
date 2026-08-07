//! Lowering of an expression's structural forms and of user-ascribed types.
//!
//! Array access and slicing are rewritten into calls to `[]`, comprehension
//! generators have their destructuring turned into a `where` clause, and
//! declaration annotations are lowered into items or an expression. Domain
//! lowering resolves a user-written type into a THIR `Domain`.

use shackle_hir::{
	Item, PatternTy,
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef, NodeRef, PatternRef},
};
use shackle_ty::{Ty, TyData};
use shackle_utils::maybe_grow_stack;

use crate::{
	lower::{
		LoweredAnnotation, LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	pub(in crate::lower) fn collect_declaration_annotation(
		&mut self,
		decl: DeclarationId<'db>,
		ann: shackle_hir::ExpressionId<'db>,
	) -> LoweredAnnotation<'db> {
		// Declarations can have annotations which point to functions using ::annotated_expression.
		// These need to be desugared into constraints.
		match &self.data[ann] {
			shackle_hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(ann).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					let e = ExpressionRef::new(self.parent.db, self.item, ann);
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						NodeRef::from(res.into_entity(self.parent.db)).source_span(self.parent.db),
						e,
						e.source_span(self.parent.db),
					)
				});
				if let LoweredIdentifier::Callable(function) = ident.clone() {
					let origin = ExpressionRef::new(self.parent.db, self.item, ann)
						.into_entity(self.parent.db);
					let ann_decl = self.introduce_declaration(
						self.parent.model[decl].top_level(),
						origin,
						|collector| {
							// Call annotation function using the annotated declaration
							let arguments = vec![alloc_expression(
								ResolvedIdentifier::Declaration(decl),
								collector,
								origin,
							)];
							alloc_expression(
								Call {
									function: function.clone(),
									arguments,
								},
								collector,
								origin,
							)
						},
					);

					let annotate = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.annotate.into(),
							arguments: vec![
								alloc_expression(
									ResolvedIdentifier::Declaration(decl),
									self,
									origin,
								),
								alloc_expression(
									ResolvedIdentifier::Declaration(ann_decl),
									self,
									origin,
								),
							],
						},
						self,
						origin,
					);
					let constraint = Constraint::new(self.parent.model[decl].top_level(), annotate);
					let c_idx = self
						.parent
						.model
						.add_constraint(ConstraintItem::new(constraint, origin));

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}
			}
			shackle_hir::Expression::Call(c) => {
				let origin =
					ExpressionRef::new(self.parent.db, self.item, ann).into_entity(self.parent.db);
				let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function]
				{
					let res = self.types.name_resolution(c.function).unwrap();
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						let e = ExpressionRef::new(self.parent.db, self.item, c.function);
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							NodeRef::from(res.into_entity(self.parent.db))
								.source_span(self.parent.db),
							e,
							e.source_span(self.parent.db),
						)
					});
					match ident {
						LoweredIdentifier::Callable(c) => c.clone(),
						_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
					}
				} else {
					Callable::Expression(Box::new(self.collect_expression(c.function)))
				};

				if let Callable::Function(f) = &function
					&& self.parent.model[*f].parameters().len() > c.arguments.len()
				{
					// Add the annotated declaration identifier as first argument
					let mut arguments = Vec::with_capacity(c.arguments.len() + 1);
					arguments.push(alloc_expression(
						ResolvedIdentifier::Declaration(decl),
						self,
						origin,
					));
					arguments.extend(c.arguments.iter().map(|arg| self.collect_expression(*arg)));

					let ann_decl = self.introduce_declaration(
						self.parent.model[decl].top_level(),
						origin,
						|collector| {
							alloc_expression(
								Call {
									function,
									arguments,
								},
								collector,
								origin,
							)
						},
					);

					let annotate = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.annotate.into(),
							arguments: vec![
								alloc_expression(
									ResolvedIdentifier::Declaration(decl),
									self,
									origin,
								),
								alloc_expression(
									ResolvedIdentifier::Declaration(ann_decl),
									self,
									origin,
								),
							],
						},
						self,
						origin,
					);
					let constraint = Constraint::new(self.parent.model[decl].top_level(), annotate);
					let c_idx = self
						.parent
						.model
						.add_constraint(ConstraintItem::new(constraint, origin));

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}

				// Return as is
				return LoweredAnnotation::Expression(alloc_expression(
					Call {
						function,
						arguments: c
							.arguments
							.iter()
							.map(|arg| self.collect_expression(*arg))
							.collect(),
					},
					self,
					origin,
				));
			}
			_ => (),
		}
		LoweredAnnotation::Expression(self.collect_expression(ann))
	}

	/// Rewrite index slicing into a call
	///
	/// Turns all indices into sets to match the slicing builtin function, and then coerces to the correct output index set.
	pub(super) fn collect_slice(
		&mut self,
		collection: shackle_hir::ExpressionId<'db>,
		indices: shackle_hir::ExpressionId<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let collection_entity =
			ExpressionRef::new(self.parent.db, self.item, collection).into_entity(self.parent.db);
		let indices_entity =
			ExpressionRef::new(self.parent.db, self.item, indices).into_entity(self.parent.db);

		let mut decls = Vec::new();
		let collection_decl = if matches!(
			&self.data[collection],
			shackle_hir::Expression::Identifier(_)
		) {
			let expr = self.collect_expression(collection);
			match &*expr {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(decl)) => *decl,
				_ => unreachable!(),
			}
		} else {
			// Add declaration to store collection
			let origin = collection_entity;
			let decl = self.introduce_declaration(false, origin, |collector| {
				collector.collect_expression(collection)
			});
			decls.push(decl);
			decl
		};
		let mut index_sets_for_infinite_slice = None;
		let array_dims = self.types[collection].dims(self.parent.db).unwrap();
		let mut slices = Vec::with_capacity(array_dims);
		match self.types[indices].lookup(self.parent.db) {
			TyData::Tuple(_, fs) => {
				if let shackle_hir::Expression::TupleLiteral(tl) = &self.data[indices] {
					for (i, (ty, e)) in fs.iter().zip(tl.fields.iter()).enumerate() {
						let index_entity = ExpressionRef::new(self.parent.db, self.item, *e)
							.into_entity(self.parent.db);
						let mut is_set = true;
						let decl = self.introduce_declaration(false, index_entity, |collector| {
							if let shackle_hir::Expression::Slice(s) = &collector.data[*e] {
								// Rewrite infinite slice .. into `'..'(index_set_mofn(c))`
								if index_sets_for_infinite_slice.is_none() {
									let decl = collector.introduce_declaration(
										false,
										origin,
										|collector| {
											alloc_expression(
												LookupCall {
													function: self
														.parent
														.ids
														.functions
														.index_sets
														.into(),
													arguments: vec![alloc_expression(
														collection_decl,
														collector,
														collection_entity,
													)],
												},
												collector,
												origin,
											)
										},
									);
									decls.push(decl);
									index_sets_for_infinite_slice = Some(decl);
								}
								alloc_expression(
									LookupCall {
										function: (*s).into(),
										arguments: vec![alloc_expression(
											TupleAccess {
												tuple: Box::new(alloc_expression(
													index_sets_for_infinite_slice.unwrap(),
													collector,
													index_entity,
												)),
												field: IntegerLiteral(i as i64 + 1),
											},
											collector,
											index_entity,
										)],
									},
									collector,
									index_entity,
								)
							} else if ty.is_set(collector.parent.db) {
								// Slice
								collector.collect_expression(*e)
							} else {
								// Rewrite index as slice of {i}
								is_set = false;
								alloc_expression(
									SetLiteral(vec![collector.collect_expression(*e)]),
									collector,
									index_entity,
								)
							}
						});
						slices.push((decl, is_set, index_entity));
						decls.push(decl);
					}
				} else {
					// Expression which evaluates to a tuple
					let indices_decl =
						self.introduce_declaration(false, indices_entity, |collector| {
							collector.collect_expression(indices)
						});
					decls.push(indices_decl);
					for (i, f) in fs.iter().enumerate() {
						// Create declaration for each index
						let is_set = f.is_set(self.parent.db);
						let accessor =
							self.introduce_declaration(false, indices_entity, |collector| {
								let ta = alloc_expression(
									TupleAccess {
										tuple: Box::new(alloc_expression(
											indices_decl,
											collector,
											indices_entity,
										)),
										field: IntegerLiteral(i as i64 + 1),
									},
									collector,
									indices_entity,
								);
								if is_set {
									ta
								} else {
									// Rewrite as {i}
									alloc_expression(
										SetLiteral(vec![ta]),
										collector,
										indices_entity,
									)
								}
							});

						slices.push((accessor, is_set, indices_entity));
						decls.push(accessor);
					}
				}
			}
			_ => {
				// 1D slicing, so must be a set index
				let decl = self.introduce_declaration(false, indices_entity, |collector| {
					if let shackle_hir::Expression::Slice(s) = &collector.data[indices] {
						// Rewrite infinite slice .. into `'..'(index_set(c))`
						alloc_expression(
							LookupCall {
								function: (*s).into(),
								arguments: vec![alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.index_set.into(),
										arguments: vec![alloc_expression(
											collection_decl,
											collector,
											collection_entity,
										)],
									},
									collector,
									indices_entity,
								)],
							},
							collector,
							indices_entity,
						)
					} else {
						collector.collect_expression(indices)
					}
				});
				slices.push((decl, true, indices_entity));
				decls.push(decl);
			}
		}
		let collection_ident = alloc_expression(collection_decl, self, collection_entity);
		let slice_tuple = alloc_expression(
			TupleLiteral(
				slices
					.iter()
					.map(|(decl, _, origin)| alloc_expression(*decl, self, *origin))
					.collect(),
			),
			self,
			indices_entity,
		);
		let arguments = slices
			.iter()
			.filter_map(|(decl, is_slice, origin)| {
				if *is_slice {
					Some(alloc_expression(*decl, self, *origin))
				} else {
					None
				}
			})
			.chain([alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.mzn_slice.into(),
					arguments: vec![collection_ident, slice_tuple],
				},
				self,
				origin,
			)])
			.collect::<Vec<_>>();
		alloc_expression(
			Let {
				items: decls.into_iter().map(LetItem::Declaration).collect(),
				in_expression: Box::new(alloc_expression(
					LookupCall {
						function: Identifier::new(
							self.parent.db,
							format!("array{}d", arguments.len() - 1),
						)
						.into(),
						arguments,
					},
					self,
					origin,
				)),
			},
			self,
			origin,
		)
	}

	pub(in crate::lower) fn collect_array_access(
		&mut self,
		collection: Expression<'db>,
		indices: Expression<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		maybe_grow_stack(|| {
			let origin = origin.into();
			alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.array_access.into(),
					arguments: vec![collection, indices],
				},
				self,
				origin,
			)
		})
	}

	pub(super) fn collect_generator(
		&mut self,
		generator: &shackle_hir::Generator<'db>,
		generators: &mut Vec<Generator<'db>>,
	) {
		let pattern_to_where = |c: &mut Self,
		                        decl: DeclarationId<'db>,
		                        p: shackle_hir::PatternId<'db>,
		                        origin: Origin<'db>| {
			// Turn destructuring into where clause of case matching pattern
			let pattern = c.collect_pattern(p);
			alloc_expression(
				Case {
					scrutinee: Box::new(alloc_expression(decl, c, origin)),
					branches: vec![
						CaseBranch::new(pattern, alloc_expression(BooleanLiteral(true), c, origin)),
						CaseBranch::new(
							Pattern::anonymous(
								match &c.types[p] {
									PatternTy::Destructuring(ty) => *ty,
									_ => unreachable!(),
								},
								origin,
							),
							alloc_expression(BooleanLiteral(false), c, origin),
						),
					],
				},
				c,
				origin,
			)
		};

		match generator {
			shackle_hir::Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				let mut assignments = Vec::new();
				let mut where_clauses = Vec::new();
				let declarations = patterns
					.iter()
					.map(|p| {
						let origin = PatternRef::new(self.parent.db, self.item, *p)
							.into_entity(self.parent.db);
						let ty = match &self.types[*p] {
							PatternTy::Variable(ty) | PatternTy::Destructuring(ty) => *ty,
							_ => unreachable!(),
						};
						// A class-set iterator binds the par object identity —
						// a `<C>_potential` enum value. The var-ness of a class
						// reference lives in the storage fields, and a THIR
						// set's element type is par (opt lifting happens at the
						// comprehension type); the HIR typer keeps the
						// reference var and class-typed to drive attribute
						// varification, so par-ify and substitute here.
						let ty = if ty.class_type(self.parent.db).is_some() {
							self.parent
								.substitute_class_with_potential_enum(ty)
								.make_par(self.parent.db)
						} else {
							ty
						};
						let declaration =
							Declaration::new(false, Domain::unbounded(self.parent.db, origin, ty));
						let decl = self
							.parent
							.model
							.add_declaration(DeclarationItem::new(declaration, origin));
						let asgs = self.collect_destructuring(decl, false, *p);
						if !asgs.is_empty() && shackle_hir::Pattern::is_refutable(*p, self.data) {
							where_clauses.push(pattern_to_where(self, decl, *p, origin.into()));
						}
						assignments.extend(asgs);
						decl
					})
					.collect();
				let collection = self.collect_expression(*collection);
				let where_clause = where_clause.map(|w| self.collect_expression(w));
				if assignments.is_empty() {
					generators.push(Generator::Iterator {
						declarations,
						collection,
						where_clause,
					});
				} else {
					// Add destructuring assignments and new where clause
					let origin = EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(patterns[0]),
					);
					if where_clauses.len() == 1 {
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(where_clauses.pop().unwrap()),
						});
					} else {
						let call = alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.forall.into(),
								arguments: vec![alloc_expression(
									ArrayLiteral(where_clauses),
									self,
									origin,
								)],
							},
							self,
							origin,
						);
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(call),
						});
					}
					let mut iter = assignments.into_iter();
					let mut assignment = iter.next().unwrap();
					for next in iter {
						generators.push(Generator::Assignment {
							assignment,
							where_clause: None,
						});
						assignment = next;
					}
					generators.push(Generator::Assignment {
						assignment,
						where_clause,
					});
				}
			}
			shackle_hir::Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let def = ExpressionCollector::new(self.parent, self.data, self.item, self.types)
					.collect_expression(*value);
				let assignment = Declaration::from_expression(self.parent.db, false, def);
				let idx = self.parent.model.add_declaration(DeclarationItem::new(
					assignment,
					EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(*pattern),
					),
				));
				let mut asgs = self.collect_destructuring(idx, false, *pattern);
				generators.push(Generator::Assignment {
					assignment: idx,
					where_clause: where_clause.map(|w| self.collect_expression(w)),
				});
				if !asgs.is_empty() {
					if shackle_hir::Pattern::is_refutable(*pattern, self.data) {
						let w = pattern_to_where(
							self,
							idx,
							*pattern,
							EntityRef::new(
								self.parent.db,
								self.item,
								shackle_hir::ids::EntityId::from(*pattern),
							)
							.into(),
						);
						let last = asgs.pop().unwrap();
						generators.extend(asgs.iter().map(|asg| Generator::Assignment {
							assignment: *asg,
							where_clause: None,
						}));
						generators.push(Generator::Assignment {
							assignment: last,
							where_clause: Some(w),
						});
					} else {
						generators.extend(asgs.iter().map(|asg| Generator::Assignment {
							assignment: *asg,
							where_clause: None,
						}));
					}
				}
			}
		}
	}

	pub(super) fn collect_default_else(
		&mut self,
		ty: Ty<'db>,
		origin: Origin<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		match ty.lookup(db) {
			TyData::Boolean(_, OptType::Opt)
			| TyData::Integer(_, OptType::Opt)
			| TyData::Float(_, OptType::Opt)
			| TyData::Enum(_, OptType::Opt, _)
			| TyData::Bottom(OptType::Opt)
			| TyData::Array {
				opt: OptType::Opt, ..
			}
			| TyData::Set(_, OptType::Opt, _)
			| TyData::Tuple(OptType::Opt, _)
			| TyData::Record(OptType::Opt, _)
			| TyData::Function(OptType::Opt, _)
			| TyData::TyVar(_, Some(OptType::Opt), _) => alloc_expression(Absent, self, origin),
			TyData::Boolean(_, _) => alloc_expression(BooleanLiteral(true), self, origin),
			TyData::String(_) => {
				alloc_expression(StringLiteral::new(self.parent.db, ""), self, origin)
			}
			TyData::Annotation(_) => {
				alloc_expression(self.parent.ids.annotations.empty_annotation, self, origin)
			}
			TyData::Array { .. } => alloc_expression(ArrayLiteral::default(), self, origin),
			TyData::Set(_, _, _) => alloc_expression(SetLiteral::default(), self, origin),
			TyData::Tuple(_, fs) => alloc_expression(
				TupleLiteral(
					fs.iter()
						.map(|f| self.collect_default_else(*f, origin))
						.collect(),
				),
				self,
				origin,
			),
			TyData::Record(_, fs) => alloc_expression(
				RecordLiteral(
					fs.iter()
						.map(|(i, t)| (Identifier(*i), self.collect_default_else(*t, origin)))
						.collect(),
				),
				self,
				origin,
			),
			_ => unreachable!("No default value for this type"),
		}
	}

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
