//! Lowering of comprehensions

use shackle_hir::{
	PatternTy,
	ids::{EntityRef, PatternRef},
};

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect array comprehension
	pub(super) fn collect_array_comprehension(
		&mut self,
		c: &shackle_hir::ArrayComprehension<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
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

	/// Collect set comprehension
	pub(super) fn collect_set_comprehension(
		&mut self,
		c: &shackle_hir::SetComprehension<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
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

	/// Collect generator
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
					.inherit_output(self.in_output)
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
}
