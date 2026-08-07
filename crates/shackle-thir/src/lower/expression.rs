//! Lowering of HIR expressions into THIR.
//!
//! Holds `ExpressionCollector`, which borrows the `ItemCollector` building the
//! model and lowers one item's expressions, domains and patterns against that
//! item's type results.

use rustc_hash::FxHashMap;
use shackle_hir::{
	Item, PatternTy, TypeResult,
	class_analysis::{OccurrenceId, class_pattern_for},
	ids::{EntityRef, ExpressionRef, NodeRef, PatternRef},
};
use shackle_ty::{Ty, TyData};
use shackle_utils::maybe_grow_stack;

use crate::{
	lower::{ItemCollector, LoweredAnnotation, LoweredIdentifier},
	source::Origin,
	*,
};

pub(in crate::lower) struct ExpressionCollector<'db, 'a, 'b, 'c> {
	pub(in crate::lower) parent: &'a mut ItemCollector<'db>,
	pub(in crate::lower) data: &'b shackle_hir::ItemData<'db>,
	item: Item<'db>,
	types: &'c TypeResult<'db>,
}

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	pub(in crate::lower) fn new(
		parent: &'a mut ItemCollector<'db>,
		data: &'b shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &'c TypeResult<'db>,
	) -> Self {
		Self {
			parent,
			data,
			types,
			item,
		}
	}

	fn introduce_declaration(
		&mut self,
		top_level: bool,
		origin: impl Into<Origin<'db>>,
		f: impl FnOnce(&mut Self) -> Expression<'db>,
	) -> DeclarationId<'db> {
		let origin: Origin = origin.into();
		let def = f(self);
		let decl = Declaration::from_expression(self.parent.db, top_level, def);
		self.parent
			.model
			.add_declaration(DeclarationItem::new(decl, origin))
	}

	/// Collect an expression
	pub(in crate::lower) fn collect_expression(
		&mut self,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		maybe_grow_stack(|| self.collect_expression_inner(idx))
	}

	/// Collect an expression, projecting a class-typed identifier into the
	/// expected class's identity universe when the two differ (a `Sub`-typed
	/// root used where a `Super` is expected).
	pub(in crate::lower) fn collect_expression_as(
		&mut self,
		idx: shackle_hir::ExpressionId<'db>,
		expected_ty: Ty<'db>,
	) -> Expression<'db> {
		let expr = self.collect_expression(idx);
		if expr.ty() == expected_ty {
			return expr;
		}
		let Some(target_class) = expected_ty.class_type(self.parent.db) else {
			return expr;
		};
		let target_class =
			class_pattern_for(self.parent.db, target_class).expect("class item for class type");
		let target_enum =
			self.parent.model[self.parent.objects.class_map[&target_class].class_enum].enum_type();
		if expr.ty().enum_ty(self.parent.db) == Some(target_enum) {
			let coerce_ty = self
				.parent
				.substitute_class_with_potential_enum(expected_ty);
			let mut coerced = Expression::new_unchecked(coerce_ty, (*expr).clone(), expr.origin());
			coerced
				.annotations_mut()
				.extend(expr.annotations().iter().cloned());
			return coerced;
		}
		// The lowered expression carries the potential-enum label, so class
		// subtyping is judged on the HIR types.
		let hir_ty = self.types[idx];
		if !hir_ty.is_subtype_of(self.parent.db, expected_ty) {
			return expr;
		}
		let Some(source_class) = hir_ty.class_type(self.parent.db) else {
			return expr;
		};
		let shackle_hir::Expression::Identifier(_) = &self.data[idx] else {
			return expr;
		};
		let res = self.types.name_resolution(idx).unwrap();
		let Some(source_occurrence) = self.parent.objects.plan.top_level_occurrences.get(&res)
		else {
			return expr;
		};
		let source_class =
			class_pattern_for(self.parent.db, source_class).expect("class item for class type");
		self.project_class_identity(
			expr,
			*source_occurrence,
			source_class,
			target_class,
			EntityRef::new(
				self.parent.db,
				self.item,
				shackle_hir::ids::EntityId::from(idx),
			),
		)
	}

	/// Whether `actual` is a valid lowered form of the typechecker's
	/// `expected` type: the lowering substitutes `Class<C>` with the
	/// `<C>_potential` enum (and par-ifies singular fresh identities), so the
	/// constructed expression's type legitimately differs from the HIR type
	/// at exactly those points.
	fn lowered_ty_matches(&self, actual: Ty<'db>, expected: Ty<'db>) -> bool {
		if actual == expected {
			return true;
		}
		let class_pattern = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(self.parent.db, class).expect("class item for class type")
		};
		match (
			actual.lookup(self.parent.db),
			expected.lookup(self.parent.db),
		) {
			(
				TyData::Class(actual_inst, actual_opt, actual_class),
				TyData::Class(expected_inst, expected_opt, expected_class),
			) if actual_opt == expected_opt
				&& class_pattern(*actual_class) == class_pattern(*expected_class)
				&& (actual_inst == expected_inst
					|| (*actual_inst == VarType::Par && *expected_inst == VarType::Var)) =>
			{
				// par-class is a valid lowering of var-class: the identity is
				// par because the singular fresh introduction collapses it,
				// while the HIR type kept var to drive attribute varification
				// through the field-access cascade.
				true
			}
			(
				TyData::Set(actual_inst, actual_opt, actual_element),
				TyData::Set(expected_inst, expected_opt, expected_element),
			) if actual_inst == expected_inst
				&& actual_opt == expected_opt
				&& self.lowered_ty_matches(*actual_element, *expected_element) =>
			{
				true
			}
			(
				TyData::Array {
					opt: actual_opt,
					dim: actual_dim,
					element: actual_element,
				},
				TyData::Array {
					opt: expected_opt,
					dim: expected_dim,
					element: expected_element,
				},
			) if actual_opt == expected_opt
				&& actual_dim == expected_dim
				&& self.lowered_ty_matches(*actual_element, *expected_element) =>
			{
				// An `array [..] of <C>` attribute lowers its element to the
				// substituted potential enum (`array [..] of var <C>_potential`),
				// so a sibling/field read of the whole array carries the enum
				// element while the HIR keeps the class element. Recurse on the
				// element exactly as the Set arm does — the class/enum element
				// arms below absorb the `<C>_potential`↔`Class<C>` equivalence.
				// The dimension type is object-independent (a plain index set),
				// so it must match exactly.
				true
			}
			(_, _) if actual.enum_ty(self.parent.db).is_some() => {
				let Some(actual_enum) = actual.enum_ty(self.parent.db) else {
					return false;
				};
				let Some(expected_class) = expected.class_type(self.parent.db) else {
					return false;
				};
				self.parent.model
					[self.parent.objects.class_map[&class_pattern(expected_class)].class_enum]
					.enum_type() == actual_enum
			}
			(
				TyData::Set(actual_inst, actual_opt, actual_element),
				TyData::Set(expected_inst, expected_opt, expected_element),
			) if actual_inst == expected_inst && actual_opt == expected_opt => {
				let Some(actual_enum) = actual_element.enum_ty(self.parent.db) else {
					return false;
				};
				let Some(expected_class) = expected_element.class_type(self.parent.db) else {
					return false;
				};
				self.parent.model
					[self.parent.objects.class_map[&class_pattern(expected_class)].class_enum]
					.enum_type() == actual_enum
			}
			_ => false,
		}
	}

	/// Whether `actual` is the same lowered shape as `expected` modulo inst
	/// and opt at every level (and the class/potential-enum identification):
	/// the loosest form of the postcondition, for shapes where a var-set
	/// comprehension lift or storage varification legitimately changed both.
	fn lowered_shape_matches(&self, actual: Ty<'db>, expected: Ty<'db>) -> bool {
		let db = self.parent.db;
		let class_enum = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(db, class)
				.and_then(|p| self.parent.objects.class_map.get(&p))
				.map(|info| self.parent.model[info.class_enum].enum_type())
		};
		match (actual.lookup(db), expected.lookup(db)) {
			(TyData::Boolean(_, _), TyData::Boolean(_, _))
			| (TyData::Integer(_, _), TyData::Integer(_, _))
			| (TyData::Float(_, _), TyData::Float(_, _))
			| (TyData::String(_), TyData::String(_))
			| (TyData::Bottom(_), TyData::Bottom(_)) => true,
			(TyData::Enum(_, _, a), TyData::Enum(_, _, e)) => a == e,
			(TyData::Class(_, _, a), TyData::Class(_, _, e)) => {
				class_pattern_for(db, *a) == class_pattern_for(db, *e)
			}
			(TyData::Enum(_, _, a), TyData::Class(_, _, e)) => class_enum(*e) == Some(*a),
			(TyData::Class(_, _, a), TyData::Enum(_, _, e)) => class_enum(*a) == Some(*e),
			(TyData::Set(_, _, a), TyData::Set(_, _, e)) => self.lowered_shape_matches(*a, *e),
			(
				TyData::Array {
					dim: ad,
					element: ae,
					..
				},
				TyData::Array {
					dim: ed,
					element: ee,
					..
				},
			) => self.lowered_shape_matches(*ad, *ed) && self.lowered_shape_matches(*ae, *ee),
			(TyData::Tuple(_, afs), TyData::Tuple(_, efs)) => {
				afs.len() == efs.len()
					&& afs
						.iter()
						.zip(efs.iter())
						.all(|(a, e)| self.lowered_shape_matches(*a, *e))
			}
			(TyData::Record(_, afs), TyData::Record(_, efs)) => {
				afs.len() == efs.len()
					&& afs
						.iter()
						.zip(efs.iter())
						.all(|((an, a), (en, e))| an == en && self.lowered_shape_matches(*a, *e))
			}
			_ => false,
		}
	}

	fn project_class_identity(
		&mut self,
		expr: Expression<'db>,
		source_occurrence: OccurrenceId,
		source_class: PatternRef<'db>,
		target_class: PatternRef<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let source_contribution = self
			.parent
			.occurrence_contribution(source_occurrence, source_class);
		let target_contribution = self
			.parent
			.occurrence_contribution(source_occurrence, target_class);
		let target_member = EnumMemberId::new(
			self.parent.objects.class_map[&target_class].class_enum,
			target_contribution.constructor_index as u32,
		);
		let source_constructor_index = source_contribution.constructor_index;
		let global_ordinal = alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.enum2int.into(),
				arguments: vec![expr],
			},
			self,
			origin,
		);
		let local_ordinal = if source_constructor_index == 0 {
			global_ordinal
		} else {
			let previous_end = self.parent.objects.contribution_end_map
				[&(source_class, source_constructor_index - 1)];
			let previous_end_expr =
				alloc_expression(ResolvedIdentifier::Declaration(previous_end), self, origin);
			let zero_based = alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.minus.into(),
					arguments: vec![global_ordinal, previous_end_expr],
				},
				self,
				origin,
			);
			alloc_expression(
				LookupCall {
					function: self.parent.ids.builtins.plus.into(),
					arguments: vec![
						alloc_expression(IntegerLiteral(1), self, origin),
						zero_based,
					],
				},
				self,
				origin,
			)
		};
		alloc_expression(
			Call {
				function: Callable::EnumConstructor(target_member),
				arguments: vec![local_ordinal],
			},
			self,
			origin,
		)
	}

	/// The join constructor index for projecting a NON-root reference of
	/// `source_class` into `join_class`'s identity universe, or `None` when
	/// no closed-form projection exists (kept a clean type error).
	///
	/// A root operand carries a static occurrence, so `project_class_identity`
	/// can correct its ordinal per contribution. A *reference* (`var Sub: r`)
	/// holds a runtime `Sub_potential` value, so the projection must be a total
	/// map over the whole potential enum. That map is a closed form ONLY when
	/// `source_class` has a SINGLE contribution across all occurrences: then
	/// `Sub_potential` has one constructor, `enum2int(r)` is already the
	/// contribution-local 1-based ordinal (`contribution_local_ordinal` is the
	/// identity for constructor 0), and the join image is
	/// `Join_occ_ct(enum2int(r))` where `ct` is the constructor the SAME
	/// occurrence contributes to the join (its superclass image, whose slot i
	/// coincides with the direct slot i — the 1:1 mapping
	/// `superclass_projection_contribution_expr` relies on). A
	/// multi-contribution source would need a piecewise per-constructor offset
	/// map; that stays a clean type error.
	fn reference_projection_join_constructor(
		&self,
		source_class: PatternRef<'db>,
		join_class: PatternRef<'db>,
	) -> Option<usize> {
		let mut single: Option<OccurrenceId> = None;
		for occ_contribs in self.parent.objects.plan.contributions_in_occurrence_order() {
			for contribution in occ_contribs
				.iter()
				.filter(|c| c.target_class == source_class)
			{
				// More than one contribution to the source class → its potential
				// enum has multiple constructors, so no single-constructor closed
				// form. Also require constructor 0 (a lone contribution always is),
				// so `enum2int` is the contribution-local ordinal.
				if single.replace(contribution.occurrence).is_some()
					|| contribution.constructor_index != 0
				{
					return None;
				}
			}
		}
		let occurrence = single?;
		self.parent.objects.plan.contributions_by_occurrence[&occurrence]
			.iter()
			.find(|c| c.target_class == join_class)
			.map(|c| c.constructor_index)
	}

	/// Relabel a class-labeled call operand to its potential-enum lowering.
	///
	/// Function resolution and type specialisation instantiate generic
	/// parameters from the argument types, and the standard library is typed
	/// over enums — a `var Class<B>` operand meeting a `var set of
	/// B_potential` operand fails to instantiate `in(var $$E, var set of
	/// $$E)`. The runtime value of a class-labeled expression already IS the
	/// potential-enum identity, so the relabel is cosmetic and makes every
	/// call see consistent enum labels.
	fn relabel_class_operand(&mut self, expr: Expression<'db>) -> Expression<'db> {
		if !expr
			.ty()
			.walk(self.parent.db)
			.any(|t| t.class_type(self.parent.db).is_some())
		{
			return expr;
		}
		let enum_ty = self.parent.substitute_class_with_potential_enum(expr.ty());
		if enum_ty == expr.ty() {
			return expr;
		}
		let mut relabeled = Expression::new_unchecked(enum_ty, (*expr).clone(), expr.origin());
		relabeled
			.annotations_mut()
			.extend(expr.annotations().iter().cloned());
		relabeled
	}

	/// Project a NON-root reference `expr : var Sub` into `join_class`'s
	/// identity universe as `Join_occ_ct(enum2int(expr))`. The caller resolved
	/// `join_constructor` via `reference_projection_join_constructor`, which
	/// guarantees the single-contribution closed form.
	fn project_reference_identity(
		&mut self,
		expr: Expression<'db>,
		join_class: PatternRef<'db>,
		join_constructor: usize,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let global_ordinal = alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.enum2int.into(),
				arguments: vec![expr],
			},
			self,
			origin,
		);
		let join_member = EnumMemberId::new(
			self.parent.objects.class_map[&join_class].class_enum,
			join_constructor as u32,
		);
		alloc_expression(
			Call {
				function: Callable::EnumConstructor(join_member),
				arguments: vec![global_ordinal],
			},
			self,
			origin,
		)
	}

	fn collect_expression_inner(&mut self, idx: shackle_hir::ExpressionId<'db>) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let mut result = match &self.data[idx] {
			shackle_hir::Expression::Absent => alloc_expression(Absent, self, origin),
			shackle_hir::Expression::ArrayAccess(aa) => {
				let is_slice = match self.types[aa.indices].lookup(db) {
					TyData::Tuple(_, fs) => fs.iter().any(|f| f.is_set(db)),
					TyData::Set(_, _, _) => true,
					_ => false,
				};
				if is_slice {
					self.collect_slice(aa.collection, aa.indices, origin)
				} else {
					let c = self.collect_expression(aa.collection);
					let i = self.collect_expression(aa.indices);
					self.collect_array_access(c, i, origin)
				}
			}
			shackle_hir::Expression::ArrayComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				alloc_expression(
					ArrayComprehension {
						generators,
						template: Box::new(self.collect_expression(c.template)),
						indices: c
							.indices
							.map(|indices| Box::new(self.collect_expression(indices))),
					},
					self,
					origin,
				)
			}
			shackle_hir::Expression::ArrayLiteral(al) => alloc_expression(
				ArrayLiteral(
					al.members
						.iter()
						.map(|m| self.collect_expression(*m))
						.collect(),
				),
				self,
				origin,
			),
			// Desugar 2D array literal into array2d call
			shackle_hir::Expression::ArrayLiteral2D(al) => {
				let mut idx_array = |dim: &shackle_hir::MaybeIndexSet<'db>| match dim {
					shackle_hir::MaybeIndexSet::Indexed(es) => alloc_expression(
						ArrayLiteral(es.iter().map(|e| self.collect_expression(*e)).collect()),
						self,
						origin,
					),
					shackle_hir::MaybeIndexSet::NonIndexed(c) => alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.set2array.into(),
							arguments: vec![if *c > 0 {
								alloc_expression(
									LookupCall {
										function: self.parent.ids.functions.dot_dot.into(),
										arguments: vec![
											alloc_expression(IntegerLiteral(1), self, origin),
											alloc_expression(
												IntegerLiteral(*c as i64),
												self,
												origin,
											),
										],
									},
									self,
									origin,
								)
							} else {
								alloc_expression(SetLiteral(Vec::new()), self, origin)
							}],
						},
						self,
						origin,
					),
				};
				let rows = idx_array(&al.rows);
				let columns = idx_array(&al.columns);
				alloc_expression(
					LookupCall {
						function: self.parent.ids.functions.mzn_array_2d_literal.into(),
						arguments: vec![
							rows,
							columns,
							alloc_expression(
								ArrayLiteral(
									al.members
										.iter()
										.map(|e| self.collect_expression(*e))
										.collect(),
								),
								self,
								origin,
							),
						],
					},
					self,
					origin,
				)
			}
			// Desugar indexed array literal into arrayNd call
			shackle_hir::Expression::IndexedArrayLiteral(al) => {
				if al.indices.len() == 1 {
					alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.mzn_start_indexed_array.into(),
							arguments: vec![
								self.collect_expression(al.indices[0]),
								alloc_expression(
									ArrayLiteral(
										al.members
											.iter()
											.map(|e| self.collect_expression(*e))
											.collect(),
									),
									self,
									origin,
								),
							],
						},
						self,
						origin,
					)
				} else {
					alloc_expression(
						LookupCall {
							function: self.parent.ids.builtins.mzn_indexed_array.into(),
							arguments: vec![alloc_expression(
								ArrayLiteral(
									al.indices
										.iter()
										.zip(al.members.iter())
										.map(|(i, e)| {
											alloc_expression(
												TupleLiteral(vec![
													self.collect_expression(*i),
													self.collect_expression(*e),
												]),
												self,
												origin,
											)
										})
										.collect(),
								),
								self,
								origin,
							)],
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::BooleanLiteral(b) => alloc_expression(*b, self, origin),
			shackle_hir::Expression::Call(c) => self.collect_call_expression(c, idx),
			shackle_hir::Expression::Case(c) => {
				let scrutinee_origin = ExpressionRef::new(self.parent.db, self.item, c.expression)
					.into_entity(self.parent.db);
				let scrutinee = self.introduce_declaration(false, scrutinee_origin, |collector| {
					collector.collect_expression(c.expression)
				});
				alloc_expression(
					Let {
						items: vec![LetItem::Declaration(scrutinee)],
						in_expression: Box::new(alloc_expression(
							Case {
								scrutinee: Box::new(alloc_expression(scrutinee, self, origin)),
								branches: c
									.cases
									.iter()
									.map(|case| {
										let pattern_origin = PatternRef::new(
											self.parent.db,
											self.item,
											case.pattern,
										)
										.into_entity(self.parent.db);
										let pattern = self.collect_pattern(case.pattern);
										let decls = self.collect_destructuring(
											scrutinee,
											false,
											case.pattern,
										);
										let result = self.collect_expression(case.value);
										if decls.is_empty() {
											CaseBranch::new(pattern, result)
										} else {
											CaseBranch::new(
												pattern,
												alloc_expression(
													Let {
														items: decls
															.into_iter()
															.map(LetItem::Declaration)
															.collect(),
														in_expression: Box::new(result),
													},
													self,
													pattern_origin,
												),
											)
										}
									})
									.collect(),
							},
							self,
							origin,
						)),
					},
					self,
					origin,
				)
			}
			shackle_hir::Expression::FloatLiteral(f) => alloc_expression(*f, self, origin),
			shackle_hir::Expression::Identifier(_) => self.collect_identifier_expression(idx),
			shackle_hir::Expression::IfThenElse(ite) => alloc_expression(
				IfThenElse {
					branches: ite
						.branches
						.iter()
						.map(|b| {
							Branch::new(
								self.collect_expression(b.condition),
								self.collect_expression(b.result),
							)
						})
						.collect(),
					else_result: Box::new(
						ite.else_result
							.map(|e| self.collect_expression(e))
							.unwrap_or_else(|| self.collect_default_else(ty, origin.into())),
					),
				},
				self,
				origin,
			),
			shackle_hir::Expression::Infinity => alloc_expression(Infinity, self, origin),
			shackle_hir::Expression::IntegerLiteral(i) => alloc_expression(*i, self, origin),
			shackle_hir::Expression::Lambda(l) => {
				let fn_type = match ty.lookup(db) {
					TyData::Function(_, f) => f,
					_ => unreachable!(),
				};
				let return_type = l
					.return_type
					.map(|r| self.collect_domain(r, fn_type.return_type, false))
					.unwrap_or_else(|| {
						Domain::unbounded(self.parent.db, origin, fn_type.return_type)
					});
				let mut decls = Vec::new();
				let parameters = l
					.parameters
					.iter()
					.zip(fn_type.params.iter())
					.map(|(param, ty)| {
						let decl = self
							.parent
							.collect_fn_param(param, *ty, self.data, self.item, self.types);
						if let Some(p) = param.pattern {
							decls.extend(self.collect_destructuring(decl, false, p));
						}
						decl
					})
					.collect::<Vec<_>>();
				let body = self.collect_expression(l.body);
				let function = Function::lambda(
					return_type,
					parameters,
					if decls.is_empty() {
						body
					} else {
						let body_entity = ExpressionRef::new(db, self.item, l.body).into_entity(db);
						alloc_expression(
							Let {
								items: decls.into_iter().map(LetItem::Declaration).collect(),
								in_expression: Box::new(body),
							},
							self,
							body_entity,
						)
					},
				);
				let f = self
					.parent
					.model
					.add_function(FunctionItem::new(function, origin));
				alloc_expression(Lambda(f), self, origin)
			}
			shackle_hir::Expression::Let(l) => alloc_expression(
				Let {
					items: l
						.items
						.iter()
						.flat_map(|i| match i {
							shackle_hir::LetItem::Constraint(c) => {
								let constraint =
									self.parent.collect_constraint(self.item, c, false);
								vec![LetItem::Constraint(constraint)]
							}
							shackle_hir::LetItem::Declaration(d) => self
								.parent
								.collect_declaration(self.item, d, false)
								.into_iter()
								.map(|d| d.into())
								.collect::<Vec<_>>(),
						})
						.collect(),
					in_expression: Box::new(self.collect_expression(l.in_expression)),
				},
				self,
				origin,
			),
			shackle_hir::Expression::RecordAccess(ra) => self.collect_record_access(ra, idx),
			shackle_hir::Expression::RecordLiteral(rl) => alloc_expression(
				RecordLiteral(
					rl.fields
						.iter()
						.map(|(i, v)| {
							(
								self.data[*i].identifier().unwrap(),
								self.collect_expression(*v),
							)
						})
						.collect(),
				),
				self,
				origin,
			),
			shackle_hir::Expression::SetComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				alloc_expression(
					SetComprehension {
						generators,
						template: Box::new(self.collect_expression(c.template)),
					},
					self,
					origin,
				)
			}
			shackle_hir::Expression::SetLiteral(sl) => alloc_expression(
				SetLiteral(
					sl.members
						.iter()
						.map(|m| self.collect_expression(*m))
						.collect(),
				),
				self,
				origin,
			),
			shackle_hir::Expression::Slice(_) => {
				unreachable!("Slice used outside of array access")
			}
			shackle_hir::Expression::StringLiteral(sl) => {
				alloc_expression(sl.clone(), self, origin)
			}
			shackle_hir::Expression::TupleAccess(ta) => {
				let tuple = self.collect_expression(ta.tuple);
				if self.types[ta.tuple].is_array(self.parent.db) {
					// Lift to comprehension
					let tuple_ty = tuple.ty().elem_ty(self.parent.db).unwrap();
					let declaration = Declaration::new(
						false,
						Domain::unbounded(self.parent.db, origin, tuple_ty),
					);
					let idx = self
						.parent
						.model
						.add_declaration(DeclarationItem::new(declaration, origin));
					let g = Generator::Iterator {
						declarations: vec![idx],
						collection: tuple,
						where_clause: None,
					};
					alloc_expression(
						ArrayComprehension {
							generators: vec![g],
							template: Box::new(alloc_expression(
								TupleAccess {
									tuple: Box::new(alloc_expression(idx, self, origin)),
									field: IntegerLiteral(
										self.data[ta.field].integer_value().unwrap(),
									),
								},
								self,
								origin,
							)),
							indices: None,
						},
						self,
						origin,
					)
				} else {
					alloc_expression(
						TupleAccess {
							tuple: Box::new(tuple),
							field: IntegerLiteral(self.data[ta.field].integer_value().unwrap()),
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::TupleLiteral(tl) => alloc_expression(
				TupleLiteral(
					tl.fields
						.iter()
						.map(|f| self.collect_expression(*f))
						.collect(),
				),
				self,
				origin,
			),
			shackle_hir::Expression::Missing => unreachable!("Missing expression"),
		};
		result.annotations_mut().extend(
			self.data
				.annotations(idx)
				.map(|ann| self.collect_expression(ann)),
		);
		// A var where the typechecker said par is a varified storage field
		// read through an unvarified HIR context, and a var-opt lift the
		// typechecker did not apply comes from a comprehension over a var
		// (class) set — the actual inst and opt flow (see the identifier
		// arm), so the postcondition compares the shape modulo inst/opt.
		assert!(
			self.lowered_ty_matches(result.ty(), ty)
				|| self.lowered_ty_matches(result.ty().make_par(db), ty)
				|| self.lowered_shape_matches(result.ty(), ty),
			"Type by construction ({}) disagrees with typechecker ({}) at {:?}",
			result.ty().pretty_print(db),
			ty.pretty_print(db),
			NodeRef::from(origin).source_span(db)
		);
		// Relabel `Class<C>` to its potential enum: class labels must not
		// flow into the transform pipeline (function instantiation and type
		// propagation are enum-based). Only the labels change — the result's
		// own inst and opt are kept, because a value that lowered var where
		// the HIR said par (a varified storage field) or var-opt (a
		// comprehension over a var set) flows that way through transform
		// folds, and a stamped-back HIR shape would go stale at the first
		// fold.
		let relabeled_ty = self
			.parent
			.substitute_class_with_potential_enum(result.ty());
		if result.ty() == relabeled_ty {
			result
		} else {
			let mut coerced =
				Expression::new_unchecked(relabeled_ty, (*result).clone(), result.origin());
			coerced
				.annotations_mut()
				.extend(result.annotations().iter().cloned());
			coerced
		}
	}

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
	fn collect_slice(
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

	fn collect_generator(
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

	fn collect_default_else(&mut self, ty: Ty<'db>, origin: Origin<'db>) -> Expression<'db> {
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

	/// Create declarations which perform destructuring according to the given pattern
	pub(in crate::lower) fn collect_destructuring(
		&mut self,
		root_decl: DeclarationId<'db>,
		top_level: bool,
		pattern: shackle_hir::PatternId<'db>,
	) -> Vec<DeclarationId<'db>> {
		let mut destructuring = Vec::new();
		let mut todo = vec![(0, pattern)];
		while let Some((i, p)) = todo.pop() {
			match &self.data[p] {
				shackle_hir::Pattern::Tuple { fields } => {
					for (idx, field) in fields.iter().enumerate() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				shackle_hir::Pattern::Record { fields } => {
					for (ident, field) in fields.iter() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::RecordAccess(*ident),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				shackle_hir::Pattern::Call {
					function,
					arguments,
				} => {
					let destructuring_pattern = if arguments.len() == 1 {
						// If we have a single arg, destructuring will return the inside directly
						arguments[0]
					} else {
						// Destructuring returns a tuple
						p
					};
					let pat = self.types.pattern_resolution(*function).unwrap();
					let res = &self.parent.resolutions[&pat];
					match res {
						LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Annotation(*ann),
								destructuring_pattern,
							));
						}
						LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Enumeration(*member),
								destructuring_pattern,
							));
						}
						_ => unreachable!(),
					};
					let j = destructuring.len();
					if arguments.len() == 1 {
						todo.push((j, arguments[0]));
					} else {
						for (idx, field) in arguments.iter().enumerate() {
							// Destructuring the tuple returns the field inside
							destructuring.push(DestructuringEntry::new(
								j,
								Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
								*field,
							));
							todo.push((destructuring.len(), *field));
						}
					}
				}
				shackle_hir::Pattern::Identifier(name) => {
					if matches!(
						&self.types[p],
						PatternTy::Variable(_) | PatternTy::Argument(_)
					) {
						if i > 0 {
							destructuring[i - 1].name = Some(*name);
							// Mark used destructurings as to be created
							let mut c = i;
							loop {
								if c == 0 {
									break;
								}
								let item = &mut destructuring[c - 1];
								if item.create {
									break;
								}
								item.create = true;
								c = item.parent;
							}
						} else {
							self.parent.model[root_decl].set_name(*name);
							let _ = self.parent.resolutions.insert(
								PatternRef::new(self.parent.db, self.item, pattern),
								LoweredIdentifier::ResolvedIdentifier(root_decl.into()),
							);
						}
					}
				}
				_ => (),
			}
		}
		let mut decls = Vec::new();
		let mut decl_map = FxHashMap::default();
		for (idx, item) in destructuring
			.into_iter()
			.enumerate()
			.filter(|(_, item)| item.create)
		{
			let origin = EntityRef::new(
				self.parent.db,
				self.item,
				shackle_hir::ids::EntityId::from(item.pattern),
			);
			let decl = self.introduce_declaration(top_level, origin, |collector| {
				let ident = alloc_expression(
					if item.parent == 0 {
						root_decl
					} else {
						decl_map[&item.parent]
					},
					collector,
					origin,
				);
				match item.kind {
					Destructuring::Annotation(a) => alloc_expression(
						Call {
							function: Callable::AnnotationDestructure(a),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::Enumeration(e) => alloc_expression(
						Call {
							function: Callable::EnumDestructor(e),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::RecordAccess(f) => alloc_expression(
						RecordAccess {
							record: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
					Destructuring::TupleAccess(f) => alloc_expression(
						TupleAccess {
							tuple: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
				}
			});
			if let Some(name) = item.name {
				self.parent.model[decl].set_name(name);
				let _ = self.parent.resolutions.insert(
					PatternRef::new(self.parent.db, self.item, item.pattern),
					LoweredIdentifier::ResolvedIdentifier(decl.into()),
				);
			}
			let _ = decl_map.insert(idx + 1, decl);
			decls.push(decl);
		}
		decls
	}

	/// Lower an HIR pattern into a THIR pattern
	fn collect_pattern(&mut self, pattern: shackle_hir::PatternId<'db>) -> Pattern<'db> {
		let db = self.parent.db;
		let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(pattern));
		let ty = match &self.types[pattern] {
			PatternTy::Destructuring(ty) => *ty,
			PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
				return Pattern::anonymous(*ty, origin);
			}
			_ => unreachable!(),
		};
		match &self.data[pattern] {
			shackle_hir::Pattern::Absent => {
				Pattern::expression(alloc_expression(Absent, self, origin), origin)
			}
			shackle_hir::Pattern::Anonymous => Pattern::anonymous(ty, origin),
			shackle_hir::Pattern::Boolean(b) => {
				Pattern::expression(alloc_expression(*b, self, origin), origin)
			}
			shackle_hir::Pattern::Call {
				function,
				arguments,
			} => {
				let args = arguments
					.iter()
					.map(|a| self.collect_pattern(*a))
					.collect::<Vec<_>>();
				let pat = self.types.pattern_resolution(*function).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
						Pattern::annotation_constructor(db, &self.parent.model, origin, *ann, args)
					}
					LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
						Pattern::enum_constructor(db, &self.parent.model, origin, *member, args)
					}
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Float { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Identifier(_) => {
				let pat = self.types.pattern_resolution(pattern).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Annotation(a)) => {
						Pattern::expression(alloc_expression(*a, self, origin), origin)
					}
					LoweredIdentifier::ResolvedIdentifier(
						ResolvedIdentifier::EnumerationMember(m),
					) => Pattern::expression(alloc_expression(*m, self, origin), origin),
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Infinity { negated } => {
				let v = alloc_expression(Infinity, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Integer { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Missing => unreachable!(),
			shackle_hir::Pattern::Record { fields } => {
				let fields = fields
					.iter()
					.map(|(i, p)| (*i, self.collect_pattern(*p)))
					.collect::<Vec<_>>();
				Pattern::record(db, &self.parent.model, origin, fields)
			}
			shackle_hir::Pattern::String(s) => {
				Pattern::expression(alloc_expression(s.clone(), self, origin), origin)
			}
			shackle_hir::Pattern::Tuple { fields } => {
				let fields = fields
					.iter()
					.map(|f| self.collect_pattern(*f))
					.collect::<Vec<_>>();
				Pattern::tuple(db, &self.parent.model, origin, fields)
			}
		}
	}

	/// Lower a call, including the cross-class identity coercion applied to
	/// `=`/`!=` when the two operands' class universes differ.
	fn collect_call_expression(
		&mut self,
		c: &shackle_hir::Call<'db>,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function] {
			let res = self.types.name_resolution(c.function).unwrap_or_else(|| {
				panic!(
					"No name resolution in types for {:?} at {:?}",
					c.function,
					ExpressionRef::new(self.parent.db, self.item, c.function)
						.source_span(self.parent.db)
				);
			});
			let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
				let f = ExpressionRef::new(self.parent.db, self.item, c.function);
				panic!(
					"Did not lower {:?} at {:?} used by {:?} at {:?}",
					res,
					res.into_entity(self.parent.db).source_span(self.parent.db),
					f,
					f.source_span(self.parent.db),
				)
			});
			match ident {
				LoweredIdentifier::Callable(c) => c.clone(),
				_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
			}
		} else {
			Callable::Expression(Box::new(self.collect_expression(c.function)))
		};
		// Cross-class identity coercion for the equality operators.
		// `a = b` / `a != b` with `a : var C1`, `b : var C2` where one
		// class is a subclass of the other lowers each operand into its
		// OWN potential universe (`C1_potential` vs `C2_potential`),
		// which MiniZinc rejects as an enum mismatch. The typer unifies
		// both operands to the join class; each subtype operand is
		// projected into the join's identity universe with
		// `project_class_identity` (the ordinal correction), while an
		// operand already OF the join class keeps its natural enum
		// lowering — so both operands become the same
		// `<Join>_potential` value. Projection needs a top-level
		// occurrence, so it applies to root identifiers (`= s1`); a
		// subtype operand that is not a projectable root is left as-is
		// (still a clean type error, not a crash). Gated on `=`/`!=`
		// (whose two operands share the type variable); every other
		// call keeps natural-type collection so a function with
		// genuinely distinct class parameters is never mis-projected.
		// NOTE: do NOT use `collect_expression_as` here — its
		// class-target relabel would flip the already-join operand to a
		// `Class<Join>` type, re-introducing a Class-vs-enum mismatch
		// against the projected (enum) operand.
		let eq_db = self.parent.db;
		let is_equality_op = matches!(
			&self.data[c.function],
			shackle_hir::Expression::Identifier(id)
				if *id == self.parent.ids.functions.eq
					|| *id == self.parent.ids.builtins.ne
					|| *id == self.parent.ids.builtins.lt
					|| *id == self.parent.ids.builtins.le
					|| *id == self.parent.ids.functions.gt
					|| *id == self.parent.ids.functions.ge
		);
		let class_pattern = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(eq_db, class).expect("class item for class type")
		};
		// Resolve the join class and the per-operand occurrence to
		// project through. The coercion is ALL-OR-NOTHING: it applies
		// only when every operand whose class differs from the join is
		// a root identity with a top-level occurrence (the ordinal
		// correction `project_class_identity` needs) or a
		// single-contribution reference. If any cross-class operand
		// cannot be projected, leave every operand at its natural
		// lowering — a clean MiniZinc type error rather than a
		// mid-lowering THIR panic from partially-coerced operands.
		let join_class: Option<PatternRef<'db>> = if is_equality_op && c.arguments.len() == 2 {
			let arg_classes: Vec<Option<PatternRef<'db>>> = c
				.arguments
				.iter()
				.map(|arg| self.types[*arg].class_type(eq_db).map(class_pattern))
				.collect();
			let all_class = arg_classes.iter().all(|c| c.is_some());
			let distinct = match (arg_classes.first(), arg_classes.get(1)) {
				(Some(a), Some(b)) => a != b,
				_ => false,
			};
			let join = if all_class && distinct {
				Ty::most_specific_supertype(eq_db, c.arguments.iter().map(|arg| self.types[*arg]))
					.and_then(|j| j.class_type(eq_db))
					.map(class_pattern)
			} else {
				None
			};
			join.filter(|jc| {
				c.arguments
					.iter()
					.zip(arg_classes.iter())
					.all(|(arg, sc)| match sc {
						Some(source_class) if source_class != jc => {
							let is_root = self
								.types
								.name_resolution(*arg)
								.map(|res| {
									self.parent
										.objects
										.plan
										.top_level_occurrences
										.contains_key(&res)
								})
								.unwrap_or(false);
							is_root
								|| self
									.reference_projection_join_constructor(*source_class, *jc)
									.is_some()
						}
						_ => true,
					})
			})
		} else {
			None
		};
		// With no projection available and no objects introduced
		// anywhere — neither operand's class universe has a single
		// constructor — both reference domains are empty, so the
		// identities can only be compared as bare ordinals. That
		// keeps the comparison well-typed (the potential enums are
		// distinct types) and is vacuously correct over empty
		// domains.
		let ordinal_compare = join_class.is_none() && is_equality_op && c.arguments.len() == 2 && {
			let classes = c
				.arguments
				.iter()
				.map(|arg| self.types[*arg].class_type(eq_db).map(class_pattern))
				.collect::<Vec<_>>();
			classes.iter().all(|class| class.is_some())
				&& classes[0] != classes[1]
				&& classes.iter().all(|class| {
					class
						.and_then(|p| self.parent.objects.class_map.get(&p))
						.is_some_and(|info| {
							self.parent.model[info.class_enum]
								.definition()
								.is_none_or(|d| d.is_empty())
						})
				})
		};
		let mut arguments = c
			.arguments
			.iter()
			.map(|arg| {
				let expr = self.collect_expression(*arg);
				let Some(join_class) = join_class else {
					let relabeled = self.relabel_class_operand(expr);
					if ordinal_compare && relabeled.ty().enum_ty(eq_db).is_some() {
						let arg_origin = EntityRef::new(
							eq_db,
							self.item,
							shackle_hir::ids::EntityId::from(*arg),
						);
						return alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.enum2int.into(),
								arguments: vec![relabeled],
							},
							self,
							arg_origin,
						);
					}
					return relabeled;
				};
				let arg_origin =
					EntityRef::new(eq_db, self.item, shackle_hir::ids::EntityId::from(*arg));
				// Project a subtype root operand into the join's identity
				// universe (ordinal correction); the result is a
				// `<Join>_potential` enum value.
				let projected = match self.types[*arg].class_type(eq_db).map(class_pattern) {
					Some(source_class) if source_class != join_class => {
						// A root operand projects through its static
						// occurrence; a non-root reference (no occurrence)
						// projects via the single-contribution closed form.
						match self.types.name_resolution(*arg).and_then(|res| {
							self.parent
								.objects
								.plan
								.top_level_occurrences
								.get(&res)
								.copied()
						}) {
							Some(occurrence) => self.project_class_identity(
								expr,
								occurrence,
								source_class,
								join_class,
								arg_origin,
							),
							None => {
								let ct = self
									.reference_projection_join_constructor(source_class, join_class)
									.expect("join filter guarantees a projectable reference");
								self.project_reference_identity(expr, join_class, ct, arg_origin)
							}
						}
					}
					_ => expr,
				};
				// Both operands must share the `<Join>_potential` enum
				// type. `project_class_identity` already yields that
				// enum, but an operand of the join class that lowered as
				// a genuine `var Class<Join>` (a par-actual reference)
				// must be RELABELLED to `var <Join>_potential`, or `=`
				// sees a Class-vs-enum mismatch. The relabel is cosmetic:
				// MiniZinc re-types the pretty-printed identifier from
				// its `var <Join>` declaration, whose values already
				// range over `<Join>_potential`.
				if projected.ty().class_type(eq_db).is_some() {
					let enum_ty = self
						.parent
						.substitute_class_with_potential_enum(projected.ty());
					let mut relabeled = Expression::new_unchecked(
						enum_ty,
						(*projected).clone(),
						projected.origin(),
					);
					relabeled
						.annotations_mut()
						.extend(projected.annotations().iter().cloned());
					relabeled
				} else {
					projected
				}
			})
			.collect::<Vec<_>>();

		let params = match &function {
			Callable::Function(f) => Some(self.parent.model[*f].parameters()),
			Callable::Annotation(a) => self.parent.model[*a].parameters.as_ref().map(|v| &v[..]),
			Callable::EnumConstructor(e) => self.parent.model[e.enumeration_id()]
				.definition()
				.unwrap()[e.member_index() as usize]
				.parameters
				.as_ref()
				.map(|v| &v[..]),
			_ => None,
		};

		if let Some(params) = params
			&& params.len() > arguments.len()
		{
			// Need to fill in default and named arguments
			let params = params[arguments.len()..].to_vec();
			let mut named = c
				.named_arguments
				.iter()
				.map(|(name, arg)| {
					(
						self.data[*name].identifier().unwrap(),
						self.collect_expression(*arg),
					)
				})
				.collect::<FxHashMap<_, _>>();

			for param in params {
				let param_name = self.parent.model[param].name().unwrap();
				if let Some(arg) = named.remove(&param_name) {
					arguments.push(arg);
				} else {
					let default = self.parent.param_defaults[&param].clone();
					arguments.push(default);
				}
			}
		}

		// The HIR-resolved function item may no longer match the
		// lowered argument types: varified storage widens par HIR
		// operands to var, and class operands are relabeled to their
		// potential enums. Re-dispatch by name so the call binds the
		// overload for the actual argument types (a `LookupCall`
		// resolves straight back to a `Call`, so a still-matching
		// resolution is unchanged).
		let needs_redispatch = match &function {
			Callable::Function(f) => {
				let params = self.parent.model[*f].parameters();
				params.len() != arguments.len()
					|| !arguments
						.iter()
						.zip(params.iter())
						.all(|(arg, p)| arg.ty().is_subtype_of(db, self.parent.model[*p].ty()))
			}
			_ => false,
		};
		if needs_redispatch {
			let Callable::Function(f) = &function else {
				unreachable!()
			};
			let name = self.parent.model[*f].name();
			alloc_expression(
				LookupCall {
					function: name,
					arguments,
				},
				self,
				origin,
			)
		} else {
			alloc_expression(
				Call {
					function,
					arguments,
				},
				self,
				origin,
			)
		}
	}

	/// Lower an identifier reference, projecting a class-typed root into the
	/// identity universe expected at this position.
	fn collect_identifier_expression(
		&mut self,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let res = self.types.name_resolution(idx).unwrap();
		let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
			let e = ExpressionRef::new(db, self.item, idx);
			panic!(
				"Did not lower {:?} at {:?} used by {:?} at {:?}",
				res,
				res.into_entity(self.parent.db).source_span(self.parent.db),
				e,
				e.source_span(self.parent.db),
			)
		});
		let expr = alloc_expression(
			match ident {
				LoweredIdentifier::ResolvedIdentifier(i) => i.clone(),
				_ => unreachable!(),
			},
			self,
			origin,
		);

		if self.lowered_ty_matches(expr.ty(), ty) {
			expr
		} else if let (Some(source_class), Some(target_class)) =
			(expr.ty().class_type(db), ty.class_type(db))
		{
			let source_class =
				class_pattern_for(db, source_class).expect("class item for class type");
			let target_class =
				class_pattern_for(db, target_class).expect("class item for class type");
			let source_occurrence = self
				.parent
				.objects
				.plan
				.top_level_occurrences
				.get(&res)
				.copied();
			if let Some(occurrence) = source_occurrence
				&& source_class != target_class
				&& expr.ty().is_subtype_of(db, ty)
			{
				self.project_class_identity(expr, occurrence, source_class, target_class, origin)
			} else if source_class == target_class {
				// A same-class label mismatch: a class-labeled
				// reference where the enum lowering is expected.
				// RELABEL to the potential-enum form of the
				// expression's own type: the underlying value is
				// already a `<A>_potential` member, so only the label
				// changes. The enum label (never `Class<A>`) is what
				// the transform pipeline's function instantiation and
				// type propagation expect; the inst is left alone —
				// a var choice of object stays var.
				let relabel_ty = self.parent.substitute_class_with_potential_enum(expr.ty());
				let mut relabeled =
					Expression::new_unchecked(relabel_ty, (*expr).clone(), expr.origin());
				relabeled
					.annotations_mut()
					.extend(expr.annotations().iter().cloned());
				relabeled
			} else {
				// Cross-class coercion of a bare reference with no
				// top-level occurrence to project through. Reaching this
				// requires an *upcast reference* (`var Sub: r` read
				// where `var Sup` is expected) whose identity would need
				// an ordinal correction between the two potential
				// universes — which only `project_class_identity` can
				// supply, and it needs an occurrence. No such shape
				// exists today: upcast projection of a root goes through
				// `collect_expression_as`, and bare references are
				// same-class (handled above). Kept as a loud panic so a
				// future cross-class reference surfaces here rather than
				// silently emitting a mis-mapped identity.
				unreachable!(
					"class-typed identifier coercion: {:?} at {:?} expected {} but lowered as {}; source {:?} target {:?} top-level occurrence present: {}",
					res,
					NodeRef::from(EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(idx)
					))
					.source_span(self.parent.db),
					ty.pretty_print(db),
					expr.ty().pretty_print(db),
					source_class.identifier(db),
					target_class.identifier(db),
					source_occurrence.is_some(),
				)
			}
		} else {
			assert!(
				self.lowered_ty_matches(expr.ty().make_par(db), ty),
				"identifier {:?} at {:?} expected {} but lowered as {}",
				res,
				NodeRef::from(EntityRef::new(
					self.parent.db,
					self.item,
					shackle_hir::ids::EntityId::from(idx)
				))
				.source_span(self.parent.db),
				ty.pretty_print(db),
				expr.ty().pretty_print(db),
			);
			// Lowered is var (a var-storage class field reached through
			// a var-new path) but the HIR-typer kept the reference par —
			// e.g. a bare attribute name in a class constraint, where
			// `this`'s class type is unvarified. Let the var-ness flow:
			// relabelling to the par HIR type would not survive a
			// transform fold (identifier types are re-derived from
			// their declarations), and `fix()` fails at runtime on a
			// genuine var decision. Calls over the widened value
			// re-dispatch by name to their var overloads.
			expr
		}
	}

	/// Lower a record field access.
	///
	/// Accessing a field of an array of records is rewritten into a
	/// comprehension over the inner value; reads of a class-typed object's
	/// field go through the reconstruction engine.
	fn collect_record_access(
		&mut self,
		ra: &shackle_hir::RecordAccess<'db>,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let record = self.collect_expression(ra.record);
		if self.types[ra.record].is_array(self.parent.db) {
			// Lift to comprehension
			let record_ty = record.ty().elem_ty(self.parent.db).unwrap();
			let declaration =
				Declaration::new(false, Domain::unbounded(self.parent.db, origin, record_ty));
			let idx = self
				.parent
				.model
				.add_declaration(DeclarationItem::new(declaration, origin));
			let g = Generator::Iterator {
				declarations: vec![idx],
				collection: record,
				where_clause: None,
			};
			alloc_expression(
				ArrayComprehension {
					generators: vec![g],
					template: Box::new(alloc_expression(
						RecordAccess {
							record: Box::new(alloc_expression(idx, self, origin)),
							field: self.data[ra.field].identifier().unwrap(),
						},
						self,
						origin,
					)),
					indices: None,
				},
				self,
				origin,
			)
		} else {
			let field_ident = self.data[ra.field].identifier().unwrap();
			let static_class = record
				.ty()
				.class_type(self.parent.db)
				.or_else(|| self.types[ra.record].class_type(self.parent.db));
			if let Some(class_ref) = static_class {
				let class_pattern = class_pattern_for(self.parent.db, class_ref)
					.expect("class item for class type");
				let class_objects = self.parent.objects.class_map[&class_pattern].class_objects;
				let class_objects_expr = alloc_expression(class_objects, self, origin);
				if record.ty().opt(self.parent.db) == Some(OptType::Opt) {
					// Optional-occurrence receiver: indexing
					// `<C>_objects` by `enum2int(<var opt …>)` would
					// pass an `opt int` into integer array access,
					// which MiniZinc rejects. Project through
					// `deopt(.)` and guard the whole access with
					// `occurs(.)` so an absent receiver yields `<>`.
					let deopt_record = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.deopt.into(),
							arguments: vec![record.clone()],
						},
						self,
						origin,
					);
					let object_index = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.enum2int.into(),
							arguments: vec![deopt_record],
						},
						self,
						origin,
					);
					let object_record =
						self.collect_array_access(class_objects_expr, object_index, origin);
					let field_access = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: field_ident,
						},
						self,
						origin,
					);
					let occurs = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.occurs.into(),
							arguments: vec![record],
						},
						self,
						origin,
					);
					let absent = alloc_expression(Absent, self, origin);
					let guarded = IfThenElse {
						branches: vec![Branch::new(occurs, field_access)],
						else_result: Box::new(absent),
					};
					let inferred = alloc_expression(guarded.clone(), self, origin);
					return if inferred.ty() == ty {
						inferred
					} else {
						Expression::new_unchecked(ty, guarded, origin)
					};
				}
				// Reading a single-dimension array-typed attribute
				// through a class-reference receiver. A
				// `<C>_objects[i].arr` access where `i` is a var object
				// identity is a var index into an
				// array-of-records-that-contain-arrays, which MiniZinc
				// rejects ("array access using a variable is not
				// supported for arrays which contain other arrays").
				// The column-projection decomposition of the `'[]'`
				// specialisation avoids this, but only fires on a var
				// THIR index — and a class reference to a par-actual
				// class is relabelled to a PAR potential-enum read, so
				// its index looks par here even though the emitted decl
				// is `var <C>` (var in MiniZinc). Force the index to
				// its var form when the field is a UNIFORM array column
				// and the receiver is NOT provably par, so the
				// decomposition fires for a genuine-var receiver (a
				// `var <C>` reference, or a projected nested identity)
				// while a provably-par receiver (a single-potential
				// root, or a `p in <par set>` generator) keeps the
				// direct access it already lowers correctly. A RAGGED
				// array field (`array [1..l]` with `l` a sibling
				// attribute) is EXCLUDED: its per-object index set
				// makes the single-representative-index-set column
				// projection wrong, and a genuinely-var-receiver ragged
				// read is a type error anyway
				// (`varify_array_class_attribute`), so only
				// effectively-par ragged reads reach here and they
				// lower fine unforced.
				let field_is_array_column = class_objects_expr
					.ty()
					.elem_ty(self.parent.db)
					.and_then(|e| e.record_fields(self.parent.db))
					.map(|fields| {
						fields.iter().any(|(n, fty)| {
							Identifier(*n) == field_ident
								&& matches!(
									fty.lookup(self.parent.db),
									TyData::Array { dim, .. }
										if !dim.is_tuple(self.parent.db)
								)
						})
					})
					.unwrap_or(false);
				let field_is_ragged = field_is_array_column
					&& self
						.parent
						.class_storage_field_decls(class_pattern.item(self.parent.db))
						.into_iter()
						.find(|d| d.ident == field_ident)
						.map(|d| {
							self.parent
								.field_domain_references_attribute(d.owner, d.declared_type)
						})
						.unwrap_or(false);
				// A receiver identifier resolving to a par declaration
				// (a pinned single-potential root, or a par-set
				// generator) is a genuinely-par index — the direct
				// access lowers fine and forcing decomposition would
				// only churn the output.
				let receiver_provably_par = matches!(
					&*record,
					ExpressionData::Identifier(ResolvedIdentifier::Declaration(d))
						if self.parent.model[*d].ty().known_par(self.parent.db)
				);
				let force_column_projection =
					field_is_array_column && !field_is_ragged && !receiver_provably_par;
				let object_index = alloc_expression(
					LookupCall {
						function: self.parent.ids.functions.enum2int.into(),
						arguments: vec![record],
					},
					self,
					origin,
				);
				let object_index = if force_column_projection {
					match object_index.ty().make_var(self.parent.db) {
						Some(var_ty) if var_ty != object_index.ty() => {
							let origin = object_index.origin();
							Expression::new_unchecked(var_ty, (*object_index).clone(), origin)
						}
						_ => object_index,
					}
				} else {
					object_index
				};
				let object_record =
					self.collect_array_access(class_objects_expr, object_index, origin);
				let field_access = RecordAccess {
					record: Box::new(object_record),
					field: field_ident,
				};
				// The projected field may be par where the HIR expected
				// var (a par storage field read through a var context)
				// or var where the HIR kept the attribute par (a
				// varified storage field read through an unvarified
				// context like a class constraint's `this`). Both flow
				// through unchanged: par is a subtype of var, and a
				// par relabel of a genuine var projection would not
				// survive a transform fold. Calls over the value
				// re-dispatch by name.
				alloc_expression(field_access, self, origin)
			} else {
				alloc_expression(
					RecordAccess {
						record: Box::new(self.collect_expression(ra.record)),
						field: field_ident,
					},
					self,
					origin,
				)
			}
		}
	}
}

pub(in crate::lower) fn alloc_expression<'db>(
	data: impl ExpressionBuilder<'db>,
	collector: &ExpressionCollector<'db, '_, '_, '_>,
	origin: impl Into<Origin<'db>>,
) -> Expression<'db> {
	Expression::new(collector.parent.db, &collector.parent.model, origin, data)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DestructuringEntry<'db> {
	parent: usize, // 0 means no parent, otherwise = index of parent + 1
	kind: Destructuring<'db>,
	pattern: shackle_hir::PatternId<'db>,
	name: Option<Identifier<'db>>,
	create: bool,
}

impl<'db> DestructuringEntry<'db> {
	fn new(parent: usize, kind: Destructuring<'db>, pattern: shackle_hir::PatternId<'db>) -> Self {
		Self {
			parent,
			kind,
			pattern,
			name: None,
			create: false,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Destructuring<'db> {
	TupleAccess(IntegerLiteral),
	RecordAccess(Identifier<'db>),
	Enumeration(EnumMemberId<'db>),
	Annotation(AnnotationId<'db>),
}
