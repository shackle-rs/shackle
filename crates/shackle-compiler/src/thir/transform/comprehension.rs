//! Desugars comprehensions
//! - Move where clauses as early as possible
//! - Turn var comprehensions into comprehensions over optional values
//! - Change set comprehensions into array comprehensions surrounded by `array2set`.
//!

use std::sync::Arc;

use rustc_hash::FxHashMap;

use super::top_down_type::add_coercion;
use crate::{
	constants::IdentifierRegistry,
	hir::OptType,
	thir::{
		db::Thir,
		traverse::{fold_call, fold_expression, visit_expression, Folder, ReplacementMap, Visitor},
		Absent, ArrayComprehension, ArrayLiteral, Branch, Call, Callable, DeclarationId,
		Expression, ExpressionData, Generator, IfThenElse, IntegerLiteral, LookupCall, Marker,
		Model, ResolvedIdentifier, SetComprehension, VarType,
	},
	utils::maybe_grow_stack,
	Result,
};

enum SurroundingCall {
	Forall,
	Exists,
	Sum,
	Other,
}

struct ComprehensionRewriter<Dst: Marker> {
	result: Model<Dst>,
	replacement_map: ReplacementMap<Dst>,
	ids: Arc<IdentifierRegistry>,
}

impl<Dst: Marker> Folder<'_, Dst> for ComprehensionRewriter<Dst> {
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
		maybe_grow_stack(|| {
			if let ExpressionData::SetComprehension(c) = &**expression {
				// Set comprehensions are turned into array comprehensions surrounded by array2set()
				let array = self.rewrite_set_comprehension(db, model, c, SurroundingCall::Other);
				return Expression::new(
					db,
					&self.result,
					expression.origin(),
					LookupCall {
						function: self.ids.array2set.into(),
						arguments: vec![Expression::new(
							db,
							&self.result,
							expression.origin(),
							array,
						)],
					},
				);
			}
			fold_expression(self, db, model, expression)
		})
	}
}

impl<Dst: Marker> ComprehensionRewriter<Dst> {
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
	) -> ArrayComprehension<Dst> {
		let mut generators = c
			.generators
			.iter()
			.map(|g| self.fold_generator(db, model, g))
			.collect::<Vec<_>>();
		let folded_template = self.fold_expression(db, model, &c.template);
		let template =
			self.desugar_comprehension(db, &mut generators, folded_template, surrounding);

		ArrayComprehension {
			generators,
			indices: None,
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
						Expression::new(
							db,
							&self.result,
							origin,
							IfThenElse {
								branches: vec![Branch {
									condition,
									result: template,
								}],
								else_result: Box::new(Expression::new(
									db,
									&self.result,
									origin,
									IntegerLiteral(0),
								)),
							},
						)
					}
				}
				SurroundingCall::Other => {
					// Rewrite var where clauses into optionality
					// Optionality coercion already done, so requires explicit types
					let opt_ty = template.ty().with_opt(db.upcast(), OptType::Opt);
					let literal = Expression::new(db, &self.result, origin, Absent);
					let absent = add_coercion(db, &mut self.result, opt_ty, literal);
					let result = add_coercion(db, &mut self.result, opt_ty, template);
					Expression::new(
						db,
						&self.result,
						origin,
						IfThenElse {
							branches: vec![Branch { condition, result }],
							else_result: Box::new(absent),
						},
					)
				}
			};
		}
		template
	}
}

struct ScopeTester<'a, T: Marker> {
	scope: &'a FxHashMap<DeclarationId<T>, bool>,
	ok: bool,
}

impl<'a, T: Marker> Visitor<'_, T> for ScopeTester<'a, T> {
	fn visit_expression(&mut self, model: &Model<T>, expression: &Expression<T>) {
		if !self.ok {
			return;
		}
		maybe_grow_stack(|| visit_expression(self, model, expression))
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

/// Desugar comprehensions
pub fn desugar_comprehension(db: &dyn Thir, model: Model) -> Result<Model> {
	log::info!("Desugaring comprehensions");
	let mut r = ComprehensionRewriter {
		ids: db.identifier_registry(),
		replacement_map: ReplacementMap::default(),
		result: Model::default(),
	};
	r.add_model(db, &model);
	Ok(r.result)
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use super::desugar_comprehension;
	use crate::thir::transform::test::check;

	#[test]
	fn test_desugar_array_comprehension_var_where() {
		check(
			desugar_comprehension,
			r#"
				predicate foo(var int: x);
				array [int] of var int: x;
				any: y = [x_i | x_i in x where foo(x_i)];
			"#,
			expect!([r#"
    function var bool: foo(var int: x);
    array [int] of var int: x;
    array [int] of var opt int: y = [if foo(x_i) then let {
      var opt int: _DECL_1 = x_i;
    } in _DECL_1 else let {
      var opt int: _DECL_2 = <>;
    } in _DECL_2 endif | x_i in x];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_var_set() {
		check(
			desugar_comprehension,
			r#"
				var set of int: x;
				any: y = [x_i | x_i in x];
			"#,
			expect!([r#"
    var set of int: x;
    array [int] of var opt int: y = [if 'in'(x_i, x) then let {
      opt int: _DECL_1 = x_i;
    } in _DECL_1 else let {
      opt int: _DECL_2 = <>;
    } in _DECL_2 endif | x_i in ub(x)];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_complex() {
		check(
			desugar_comprehension,
			r"
				var set of int: x;
				predicate foo(var int: x);
				test bar(int: x);
				any: y = [x_i | x_i in x where foo(x_i), x_j in x where bar(x_j) /\ bar(x_i)];
			",
			expect!([r#"
    var set of int: x;
    function var bool: foo(var int: x);
    function bool: bar(int: x);
    array [int] of var opt int: y = [if forall(['in'(x_i, x), 'in'(x_j, x), foo(x_i)]) then let {
      opt int: _DECL_1 = x_i;
    } in _DECL_1 else let {
      opt int: _DECL_2 = <>;
    } in _DECL_2 endif | x_i in ub(x) where bar(x_i), x_j in ub(x) where bar(x_j)];
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_forall() {
		check(
			desugar_comprehension,
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
			desugar_comprehension,
			r#"
				predicate foo(var int: x);
				var set of int: S;
				constraint exists (i in S) (foo(i));
			"#,
			expect!([r"
    function var bool: foo(var int: x);
    var set of int: S;
    constraint exists(['/\'('in'(i, S), foo(i)) | i in ub(S)]);
"]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_sum_par() {
		check(
			desugar_comprehension,
			r#"
				var set of int: S;
				any: x = sum (i in S) (i);
			"#,
			expect!([r#"
    var set of int: S;
    var int: x = sum(['*'('in'(i, S), i) | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_array_comprehension_sum_var() {
		check(
			desugar_comprehension,
			r#"
				var set of int: S;
				function var int: foo(int: x);
				any: x = sum (i in S) (foo(i));
				"#,
			expect!([r#"
    var set of int: S;
    function var int: foo(int: x);
    var int: x = sum([if 'in'(i, S) then foo(i) else 0 endif | i in ub(S)]);
"#]),
		)
	}

	#[test]
	fn test_desugar_set_comprehension() {
		check(
			desugar_comprehension,
			r#"
				set of int: S;
				function var int: foo(int: x);
				any: x = { foo(i) | i in S };
				"#,
			expect!([r#"
    set of int: S;
    function var int: foo(int: x);
    var set of int: x = array2set([foo(i) | i in S]);
"#]),
		)
	}

	#[test]
	fn test_desugar_var_set_comprehension() {
		check(
			desugar_comprehension,
			r#"
				var set of int: S;
				function var int: foo(int: x);
				any: x = { foo(i) | i in S };
				"#,
			expect!([r#"
    var set of int: S;
    function var int: foo(int: x);
    var set of int: x = array2set([if 'in'(i, S) then let {
      var opt int: _DECL_1 = foo(i);
    } in _DECL_1 else let {
      var opt int: _DECL_2 = <>;
    } in _DECL_2 endif | i in ub(S)]);
"#]),
		)
	}
}
