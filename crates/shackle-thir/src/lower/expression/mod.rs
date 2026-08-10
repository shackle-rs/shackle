//! Lowering of HIR expressions into THIR.
//!
//! Holds `ExpressionCollector`, which borrows the `ItemCollector` building the
//! model and lowers one item's expressions, domains and patterns against that
//! item's type results.

use shackle_hir::{
	Item, TypeResult,
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef, NodeRef, PatternRef},
};
use shackle_ty::{Ty, TyData};
use shackle_utils::maybe_grow_stack;

use crate::{lower::ItemCollector, source::Origin, *};

mod array_access;
mod array_literal;
mod call;
mod class;
mod comprehension;
mod declaration_annotation;
mod domain;
mod identifier;
mod if_then_else;
mod pattern;
mod record_access;

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

	fn collect_expression_inner(&mut self, idx: shackle_hir::ExpressionId<'db>) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let mut result = match &self.data[idx] {
			shackle_hir::Expression::Absent => alloc_expression(Absent, self, origin),
			shackle_hir::Expression::ArrayAccess(aa) => self.collect_array_access(aa, origin),
			shackle_hir::Expression::ArrayComprehension(c) => {
				self.collect_array_comprehension(c, origin)
			}
			shackle_hir::Expression::ArrayLiteral(al) => self.collect_array_literal(al, origin),
			shackle_hir::Expression::ArrayLiteral2D(al) => {
				self.collect_array_literal_2d(al, origin)
			}
			shackle_hir::Expression::IndexedArrayLiteral(al) => {
				self.collect_indexed_array_literal(al, origin)
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
			shackle_hir::Expression::Identifier(_) => self.collect_identifier(idx),
			shackle_hir::Expression::IfThenElse(ite) => self.collect_if_then_else(ite, ty, origin),
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
				self.collect_set_comprehension(c, origin)
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
}

pub(in crate::lower) fn alloc_expression<'db>(
	data: impl ExpressionBuilder<'db>,
	collector: &ExpressionCollector<'db, '_, '_, '_>,
	origin: impl Into<Origin<'db>>,
) -> Expression<'db> {
	Expression::new(collector.parent.db, &collector.parent.model, origin, data)
}
