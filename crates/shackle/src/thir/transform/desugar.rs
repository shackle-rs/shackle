//! Transforms var if-then-else into predicate, var comprehensions into comprehensions over optional values.
//!

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::IdentifierRegistry,
	thir::{
		db::Thir, fold_call, fold_expression, source::Origin, visit_expression, Absent,
		ArrayComprehension, ArrayLiteral, BooleanLiteral, Branch, Call, Callable, DeclarationId,
		Expression, ExpressionData, Folder, Generator, IfThenElse, IntegerLiteral, LookupCall,
		Marker, Model, ReplacementMap, ResolvedIdentifier, SetComprehension, VarType, Visitor,
	},
};

enum SurroundingCall {
	Forall,
	Exists,
	Sum,
	Other,
}

struct Rewriter<Dst> {
	result: Model<Dst>,
	replacement_map: ReplacementMap<Dst>,
	ids: Arc<IdentifierRegistry>,
}

impl<Dst: Marker> Folder<Dst> for Rewriter<Dst> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.result
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst> {
		&mut self.replacement_map
	}

	fn fold_array_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &ArrayComprehension,
	) -> ArrayComprehension<Dst> {
		self.rewrite_array_comprehension(db, model, c, SurroundingCall::Other)
	}

	fn fold_set_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &SetComprehension,
	) -> SetComprehension<Dst> {
		self.rewrite_set_comprehension(db, model, c, SurroundingCall::Other)
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model, call: &Call) -> Call<Dst> {
		if let Callable::Function(f) = &call.function {
			// forall, exists and sum comprehensions get special treatment
			let special_cases = [
				(self.ids.forall, SurroundingCall::Forall),
				(self.ids.exists, SurroundingCall::Exists),
				(self.ids.sum, SurroundingCall::Sum),
			];
			for (ident, surround) in special_cases {
				if model[*f].name() == ident && call.arguments.len() == 1 {
					let arg = &call.arguments[0];
					if let ExpressionData::ArrayComprehension(c) = &**arg {
						// May be able to rewrite into non-optional comprehension, so lookup function again
						let comprehension =
							self.rewrite_array_comprehension(db, model, c, surround);
						return LookupCall {
							function: ident.into(),
							arguments: vec![Expression::new(
								db,
								&self.result,
								arg.origin(),
								comprehension,
							)],
						}
						.resolve(db, &self.result)
						.0;
					}
				}
			}
		}
		fold_call(self, db, model, call)
	}

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		expression: &Expression,
	) -> Expression<Dst> {
		let folded = fold_expression(self, db, model, expression);
		if let ExpressionData::IfThenElse(ite) = &*folded {
			if ite
				.branches
				.iter()
				.any(|b| matches!(b.condition.ty().inst(db.upcast()), Some(VarType::Var)))
			{
				// Rewrite as if_then_else function
				return self.desugar_if_then_else(
					db,
					expression.origin(),
					ite.branches.clone(),
					*(ite.else_result).clone(),
				);
			}
		}
		folded
	}
}

impl<Dst: Marker> Rewriter<Dst> {
	fn rewrite_array_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &ArrayComprehension,
		surrounding: SurroundingCall,
	) -> ArrayComprehension<Dst> {
		let mut generators = c
			.generators
			.iter()
			.map(|g| self.fold_generator(db, model, g))
			.collect::<Vec<_>>();
		let folded_template = self.fold_expression(db, model, &c.template);
		let template =
			self.desugar_comprehension(db, &mut generators, folded_template, surrounding);
		let indices = c
			.indices
			.as_ref()
			.map(|i| Box::new(self.fold_expression(db, model, i)));

		ArrayComprehension {
			generators,
			indices,
			template: Box::new(template),
		}
	}

	fn rewrite_set_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &SetComprehension,
		surrounding: SurroundingCall,
	) -> SetComprehension<Dst> {
		let mut generators = c
			.generators
			.iter()
			.map(|g| self.fold_generator(db, model, g))
			.collect::<Vec<_>>();
		let folded_template = self.fold_expression(db, model, &c.template);
		let template =
			self.desugar_comprehension(db, &mut generators, folded_template, surrounding);

		SetComprehension {
			generators,
			template: Box::new(template),
		}
	}

	/// Move par where clauses in generators to earliest possible place, and rewrite var where clauses into optionality.
	///
	/// Returns desugared version of template.
	fn desugar_comprehension(
		&mut self,
		db: &dyn Thir,
		generators: &mut [Generator<Dst>],
		template: Expression<Dst>,
		surrounding: SurroundingCall,
	) -> Expression<Dst> {
		let mut todo = Vec::new();
		let mut par_where = Vec::new();
		let mut var_where = Vec::new();
		for g in generators.iter_mut() {
			match g {
				Generator::Iterator {
					declarations,
					collection,
					where_clause,
				} => {
					if collection.ty().is_var_set(db.upcast()) {
						let c = collection.clone();
						*collection = Expression::new(
							db,
							&self.result,
							c.origin(),
							LookupCall {
								function: self.ids.ub.into(),
								arguments: vec![c.clone()],
							},
						);
						for d in declarations.iter() {
							var_where.push(Expression::new(
								db,
								&self.result,
								c.origin(),
								LookupCall {
									function: self.ids.in_.into(),
									arguments: vec![
										Expression::new(
											db,
											&self.result,
											self.result[*d].origin(),
											*d,
										),
										c.clone(),
									],
								},
							));
						}
					}

					if let Some(w) = where_clause.take() {
						todo.push(w);
					}
				}
				Generator::Assignment { where_clause, .. } => {
					if let Some(w) = where_clause.take() {
						todo.push(w);
					}
				}
			}
		}

		while let Some(w) = todo.pop() {
			if let ExpressionData::Call(c) = &*w {
				if let Callable::Function(f) = &c.function {
					if self.result[*f].name() == self.ids.conj {
						todo.extend(c.arguments.iter().cloned());
						continue;
					} else if self.result[*f].name() == self.ids.forall && c.arguments.len() == 1 {
						if let ExpressionData::ArrayLiteral(al) = &*c.arguments[0] {
							todo.extend(al.iter().cloned());
							continue;
						}
					}
				}
			}
			if w.ty().inst(db.upcast()).unwrap() == VarType::Var {
				var_where.push(w);
			} else {
				par_where.push(Some(w));
			}
		}

		// Place par where clauses as early as possible into generators
		let mut decls = FxHashMap::default();
		for g in generators.iter() {
			for d in g.declarations() {
				decls.insert(d, false);
			}
		}
		for g in generators.iter_mut() {
			let mut clauses = Vec::new();
			for d in g.declarations() {
				*decls.get_mut(&d).unwrap() = true;
			}
			for item in par_where.iter_mut() {
				let valid = if let Some(w) = item {
					ScopeTester::run(&self.result, &decls, w)
				} else {
					false
				};
				if valid {
					clauses.push(item.take().unwrap());
				}
			}
			if !clauses.is_empty() {
				let origin = clauses[0].origin();
				if clauses.len() > 1 {
					g.set_where(Expression::new(
						db,
						&self.result,
						origin,
						LookupCall {
							function: self.ids.forall.into(),
							arguments: vec![Expression::new(
								db,
								&self.result,
								origin,
								ArrayLiteral(clauses),
							)],
						},
					));
				} else {
					g.set_where(clauses.pop().unwrap());
				}
			}
		}
		for w in par_where {
			assert!(w.is_none());
		}

		if !var_where.is_empty() {
			let origin = var_where[0].origin();
			let condition = if var_where.len() > 1 {
				Expression::new(
					db,
					&self.result,
					origin,
					LookupCall {
						function: self.ids.forall.into(),
						arguments: vec![Expression::new(
							db,
							&self.result,
							origin,
							ArrayLiteral(var_where),
						)],
					},
				)
			} else {
				var_where.pop().unwrap()
			};
			return match surrounding {
				SurroundingCall::Forall => {
					// Rewrite var where clauses into implications
					Expression::new(
						db,
						&self.result,
						origin,
						LookupCall {
							function: self.ids.imp.into(),
							arguments: vec![condition, template],
						},
					)
				}
				SurroundingCall::Exists => {
					// Rewrite var where clauses into conjunctions
					Expression::new(
						db,
						&self.result,
						origin,
						LookupCall {
							function: self.ids.conj.into(),
							arguments: vec![condition, template],
						},
					)
				}
				SurroundingCall::Sum => {
					if template.ty().inst(db.upcast()) == Some(VarType::Par) {
						// Rewrite var where clauses into linear sum
						Expression::new(
							db,
							&self.result,
							origin,
							LookupCall {
								function: self.ids.times.into(),
								arguments: vec![condition, template],
							},
						)
					} else {
						// Rewrite var where clauses into if-then-else
						self.desugar_if_then_else(
							db,
							origin,
							vec![Branch {
								condition,
								result: template,
							}],
							Expression::new(db, &self.result, origin, IntegerLiteral(0)),
						)
					}
				}
				SurroundingCall::Other => {
					// Rewrite var where clauses into optionality
					self.desugar_if_then_else(
						db,
						origin,
						vec![Branch {
							condition,
							result: template,
						}],
						Expression::new(db, &self.result, origin, Absent),
					)
				}
			};
		}
		template
	}

	/// Rewrite var if-then-else into if_then_else call
	fn desugar_if_then_else(
		&mut self,
		db: &dyn Thir,
		origin: Origin,
		branches: Vec<Branch<Dst>>,
		else_result: Expression<Dst>,
	) -> Expression<Dst> {
		assert!(!branches.is_empty());
		let var_condition = branches[0].var_condition(db);
		let mut bs = Vec::with_capacity(branches.len());
		let mut rest = Vec::new();
		let mut done = false;
		for b in branches {
			if !done && b.var_condition(db) == var_condition {
				bs.push(b);
			} else {
				done = true;
				rest.push(b);
			}
		}
		let tail = if rest.is_empty() {
			else_result
		} else {
			self.desugar_if_then_else(db, origin, rest, else_result)
		};

		if var_condition {
			// Create if_then_else call
			let mut conditions = Vec::with_capacity(bs.len() + 1);
			let mut results = Vec::with_capacity(bs.len() + 1);

			for b in bs {
				conditions.push(b.condition);
				results.push(b.result);
			}
			conditions.push(Expression::new(
				db,
				&self.result,
				tail.origin(),
				BooleanLiteral(true),
			));
			results.push(tail);

			Expression::new(
				db,
				&self.result,
				origin,
				LookupCall {
					function: self.ids.if_then_else.into(),
					arguments: vec![
						Expression::new(
							db,
							&self.result,
							conditions.last().unwrap().origin(),
							ArrayLiteral(conditions),
						),
						Expression::new(
							db,
							&self.result,
							results.last().unwrap().origin(),
							ArrayLiteral(results),
						),
					],
				},
			)
		} else {
			// Keep as if-then-else
			Expression::new(
				db,
				&self.result,
				origin,
				IfThenElse {
					branches: bs,
					else_result: Box::new(tail),
				},
			)
		}
	}
}

struct ScopeTester<'a, T> {
	scope: &'a FxHashMap<DeclarationId<T>, bool>,
	ok: bool,
}

impl<'a, T: Marker> Visitor<'_, T> for ScopeTester<'a, T> {
	fn visit_expression(&mut self, model: &Model<T>, expression: &Expression<T>) {
		if !self.ok {
			return;
		}
		visit_expression(self, model, expression)
	}

	fn visit_identifier(&mut self, _model: &Model<T>, identifier: &ResolvedIdentifier<T>) {
		if let ResolvedIdentifier::Declaration(idx) = identifier {
			if let Some(false) = self.scope.get(idx) {
				self.ok = false;
			}
		}
	}
}

impl<'a, T: Marker> ScopeTester<'a, T> {
	fn run(
		model: &Model<T>,
		scope: &'a FxHashMap<DeclarationId<T>, bool>,
		expression: &Expression<T>,
	) -> bool {
		let mut st = Self { scope, ok: true };
		st.visit_expression(model, expression);
		st.ok
	}
}

/// Desugar var if-then-else and comprehensions
pub fn desugar_model(db: &dyn Thir, model: &Model) -> Model {
	let mut r = Rewriter {
		ids: db.identifier_registry(),
		replacement_map: ReplacementMap::default(),
		result: Model::default(),
	};
	r.add_model(db, model);
	r.result
}

#[cfg(test)]
mod test {
	use crate::thir::transform::test::check;
	use expect_test::expect;

	use super::desugar_model;

	#[test]
	fn test_desugar_array_comprehension_var_where() {
		check(
			desugar_model,
			r#"
				predicate foo(var int: x);
				array [int] of var int: x;
				any: y = [x_i | x_i in x where foo(x_i)];
			"#,
			expect!([r#"
    function var bool: foo(var int: x);
    array [int] of var int: x;
    array [int] of var opt int: y = [if_then_else([foo(x_i), true], [x_i, <>]) | x_i in x];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_var_set() {
		check(
			desugar_model,
			r#"
				var set of int: x;
				any: y = [x_i | x_i in x];
			"#,
			expect!([r#"
    var set of int: x;
    array [int] of var opt int: y = [if_then_else(['in'(x_i, x), true], [x_i, <>]) | x_i in ub(x)];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_complex() {
		check(
			desugar_model,
			r#"
				var set of int: x;
				predicate foo(var int: x);
				test bar(int: x);
				any: y = [x_i | x_i in x where foo(x_i), x_j in x where bar(x_j) /\ bar(x_i)];
			"#,
			expect!([r#"
    var set of int: x;
    function var bool: foo(var int: x);
    function bool: bar(int: x);
    array [int] of var opt int: y = [if_then_else([forall(['in'(x_i, x), 'in'(x_j, x), foo(x_i)]), true], [x_i, <>]) | x_i in ub(x) where bar(x_i), x_j in ub(x) where bar(x_j)];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_forall() {
		check(
			desugar_model,
			r#"
				predicate foo(var int: x);
				var set of int: S;
				constraint forall (i in S) (foo(i));
			"#,
			expect!([r#"
    function var bool: foo(var int: x);
    var set of int: S;
    constraint forall(['->'('in'(i, S), foo(i)) | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_exists() {
		check(
			desugar_model,
			r#"
				predicate foo(var int: x);
				var set of int: S;
				constraint exists (i in S) (foo(i));
			"#,
			expect!([r#"
    function var bool: foo(var int: x);
    var set of int: S;
    constraint exists(['/\'('in'(i, S), foo(i)) | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_sum_par() {
		check(
			desugar_model,
			r#"
				var set of int: S;
				any: x = sum (i in S) (i);
			"#,
			expect!([r#"
    var set of int: S;
    var int: x = sum(['*'(bool2int('in'(i, S)), i) | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_sum_var() {
		check(
			desugar_model,
			r#"
				var set of int: S;
				function var int: foo(int: x);
				any: x = sum (i in S) (foo(i));
				"#,
			expect!([r#"
    var set of int: S;
    function var int: foo(int: x);
    var int: x = sum([if_then_else(['in'(i, S), true], [foo(i), 0]) | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_var_if_then_else_1() {
		check(
			desugar_model,
			r#"
				var bool: p;
				any: x = if p then 1 else 2 endif;
			"#,
			expect!([r#"
    var bool: p;
    var int: x = if_then_else([p, true], [1, 2]);
"#]),
		)
	}

	#[test]
	fn test_desugar_var_if_then_else_2() {
		check(
			desugar_model,
			r#"
				var bool: p;
				var bool: q;
				any: x = if p then 1 elseif q then 2 else 3 endif;
			"#,
			expect!([r#"
    var bool: p;
    var bool: q;
    var int: x = if_then_else([p, q, true], [1, 2, 3]);
"#]),
		)
	}

	#[test]
	fn test_desugar_if_then_else_mixed() {
		check(
			desugar_model,
			r#"
				var bool: p;
				var bool: q;
				var bool: r;
				any: x = if p then 1 elseif true then 2 elseif q then 3 elseif r then 4 elseif false then 5 elseif true then 6 else 7 endif;
			"#,
			expect!([r#"
    var bool: p;
    var bool: q;
    var bool: r;
    var int: x = if_then_else([p, true], [1, if true then 2 else if_then_else([q, r, true], [3, 4, if false then 5 elseif true then 6 else 7 endif]) endif]);
"#]),
		)
	}
}
