//! Desugars comprehensions
//! - Move where clauses as early as possible
//! - Turn var comprehensions into comprehensions over optional values
//! - Change set comprehensions into array comprehensions surrounded by `array2set`.
//! - Change indexed comprehensions into comprehensions over tuples surrounded by `mzn_indexed_array`
//!

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::{OptType, constants::IdentifierRegistry};
use shackle_ty::Ty;
use shackle_utils::maybe_grow_stack;

use super::top_down_type::add_coercion;
use crate::{
	Absent, ArrayComprehension, ArrayLiteral, Branch, Call, Callable, Db, Declaration,
	DeclarationId, Expression, ExpressionData, Generator, IfThenElse, IntegerLiteral, Item,
	LookupCall, LookupIdentifier, Marker, Model, ResolvedIdentifier, SetComprehension,
	TupleLiteral, VarType,
	traverse::{Folder, ReplacementMap, Visitor, fold_call, fold_expression},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SurroundingCall {
	Forall,
	Exists,
	Sum,
	Other,
}

struct ComprehensionRewriter<'db, Dst: Marker> {
	result: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst>,
	ids: &'db IdentifierRegistry<'db>,
}

impl<'db, Dst: Marker> Folder<'_, 'db, Dst> for ComprehensionRewriter<'db, Dst> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.result
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst> {
		&mut self.replacement_map
	}

	fn fold_call(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		call: &Call<'db>,
	) -> Call<'db, Dst> {
		if let Callable::Function(f) = &call.function {
			// forall, exists and sum comprehensions get special treatment
			let special_cases = [
				(self.ids.functions.forall, SurroundingCall::Forall),
				(self.ids.functions.exists, SurroundingCall::Exists),
				(self.ids.functions.sum, SurroundingCall::Sum),
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
		db: &'db dyn Db,
		model: &Model<'db>,
		expression: &Expression<'db>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::ArrayComprehension(c) => {
					let mut array =
						self.rewrite_array_comprehension(db, model, c, SurroundingCall::Other);
					if let Some(indices) = array.indices.take() {
						array.template = Box::new(Expression::new(
							db,
							&self.result,
							expression.origin(),
							TupleLiteral(vec![*indices, *array.template]),
						));

						return Expression::new(
							db,
							&self.result,
							expression.origin(),
							LookupCall {
								function: self.ids.builtins.mzn_indexed_array.into(),
								arguments: vec![Expression::new(
									db,
									&self.result,
									expression.origin(),
									array,
								)],
							},
						);
					}
					Expression::new(db, &self.result, expression.origin(), array)
				}
				ExpressionData::SetComprehension(c) => {
					// Set comprehensions are turned into array comprehensions surrounded by array2set()
					let array =
						self.rewrite_set_comprehension(db, model, c, SurroundingCall::Other);
					let desugared = Expression::new(
						db,
						&self.result,
						expression.origin(),
						LookupCall {
							function: self.ids.builtins.mzn_array2set.into(),
							arguments: vec![Expression::new(
								db,
								&self.result,
								expression.origin(),
								array,
							)],
						},
					);
					assert_eq!(
						expression.ty(),
						desugared.ty(),
						"Desugared type has changed from {} to {} at {}",
						expression.ty().pretty_print(db),
						desugared.ty().pretty_print(db),
						expression.origin().pretty_print(db)
					);
					desugared
				}
				_ => fold_expression(self, db, model, expression),
			}
		})
	}
}

impl<'db, Dst: Marker> ComprehensionRewriter<'db, Dst> {
	fn rewrite_array_comprehension(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		c: &ArrayComprehension<'db>,
		surrounding: SurroundingCall,
	) -> ArrayComprehension<'db, Dst> {
		let mut generators = c
			.generators
			.iter()
			.map(|g| self.fold_generator(db, model, g))
			.collect::<Vec<_>>();
		let folded_template = if surrounding != SurroundingCall::Other
			&& let ExpressionData::Call(c) = &**c.template
			&& c.matches_builtin(model, self.ids.functions.val2opt)
		{
			// Remove opt coercion since these cases can be rewritten to be non-optional
			self.fold_expression(db, model, &c.arguments[0])
		} else {
			self.fold_expression(db, model, &c.template)
		};
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
		db: &'db dyn Db,
		model: &Model<'db>,
		c: &SetComprehension<'db>,
		surrounding: SurroundingCall,
	) -> ArrayComprehension<'db, Dst> {
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
		db: &'db dyn Db,
		generators: &mut Vec<Generator<'db, Dst>>,
		template: Expression<'db, Dst>,
		surrounding: SurroundingCall,
	) -> Expression<'db, Dst> {
		let mut gen_idx = FxHashMap::default();
		for (i, g) in generators.iter().enumerate() {
			for d in g.declarations() {
				let _ = gen_idx.insert(d, i + 1);
			}
		}
		let mut todo = Vec::new();
		let mut par_where = Vec::new();
		let mut var_where = Vec::new();
		for (i, g) in generators.iter_mut().enumerate() {
			match g {
				Generator::Iterator {
					declarations,
					collection,
					where_clause,
				} => {
					// Turn var set comprehension into fixed comprehension with where clause
					if collection.ty().is_var_set(db) {
						let c = collection.clone();
						let has_set2iter = self
							.result
							.lookup_function(db, self.ids.functions.set2iter.into(), &[c.ty()])
							.is_ok_and(|f| self.result[f.function].body().is_some());
						*collection = Expression::new(
							db,
							&self.result,
							c.origin(),
							LookupCall {
								function: if has_set2iter {
									self.ids.functions.set2iter.into()
								} else {
									self.ids.functions.ub.into()
								},
								arguments: vec![c.clone()],
							},
						);
						for d in declarations.iter() {
							var_where.push((
								Expression::new(
									db,
									&self.result,
									c.origin(),
									LookupCall {
										function: self.ids.functions.in_.into(),
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
								),
								i + 1,
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

		// Break apart where clauses and sort them into var and par
		while let Some(w) = todo.pop() {
			if let ExpressionData::Call(c) = &*w
				&& let Callable::Function(f) = &c.function
			{
				if self.result[*f].name() == self.ids.functions.and {
					todo.extend(c.arguments.iter().cloned());
					continue;
				} else if self.result[*f].name() == self.ids.functions.forall
					&& c.arguments.len() == 1
					&& let ExpressionData::ArrayLiteral(al) = &*c.arguments[0]
				{
					todo.extend(al.iter().cloned());
					continue;
				}
			}
			let idx = ScopeTester::run(&self.result, &gen_idx, &w);
			if w.ty().inst(db).unwrap() == VarType::Var {
				var_where.push((w, idx));
			} else {
				par_where.push((w, idx));
			}
		}

		let mut has_dummy = false;

		// Place par where clauses as early as possible into generators
		for (w, idx) in par_where {
			if idx == 0 && !has_dummy {
				has_dummy = true;
				let decl = Declaration::from_expression(
					db,
					false,
					Expression::new(db, &self.result, w.origin(), IntegerLiteral(0)),
				);
				let decl_idx = self.result.add_declaration(Item::new(decl, w.origin()));
				generators.insert(
					0,
					Generator::Assignment {
						assignment: decl_idx,
						where_clause: Some(w),
					},
				);
			} else {
				let index = if has_dummy { idx } else { idx - 1 };
				generators[index].update_where(|where_clause| {
					if let Some(old_where) = where_clause {
						Some(Expression::new(
							db,
							&self.result,
							w.origin(),
							LookupCall {
								function: self.ids.functions.and.into(),
								arguments: vec![old_where, w],
							},
						))
					} else {
						Some(w)
					}
				})
			}
		}

		// Var where clauses need special treatment for undefinedness
		//
		// Since these should be able to guard against undefinedness in subsequent
		// generators, we can't just move them inside the body and remove the
		// where clause.
		//
		// Here we rewrite [x | i in foo where bar] (where bar is var bool)
		// into [if w then x else <> endif | i in foo, w :: mzn_var_where_clause = bar]
		//
		// This way the where clause can be detected during totalisation but the
		// option type part has already been introduced and can be erased accordingly.
		if !var_where.is_empty() {
			// Add new assignment generators for each var where clause as early
			// as possible.
			let mut clauses = Vec::with_capacity(var_where.len());
			var_where.sort_by_key(|(_, i)| *i);
			for (w, idx) in var_where.into_iter().rev() {
				let origin = w.origin();
				let mut decl = Declaration::from_expression(db, false, w);
				decl.annotations_mut().push(Expression::new(
					db,
					&self.result,
					origin,
					LookupIdentifier(self.ids.annotations.mzn_var_where_clause),
				));
				let decl_idx = self.result.add_declaration(Item::new(decl, origin));
				let e = Expression::new(db, &self.result, origin, decl_idx);
				generators.insert(
					idx,
					Generator::Assignment {
						assignment: decl_idx,
						where_clause: None,
					},
				);
				clauses.push(e);
			}

			// Transform var where into optionality in the template
			let origin = clauses[0].origin();
			let condition = if clauses.len() > 1 {
				Expression::new(
					db,
					&self.result,
					origin,
					LookupCall {
						function: self.ids.functions.forall.into(),
						arguments: vec![Expression::new(
							db,
							&self.result,
							origin,
							ArrayLiteral(clauses),
						)],
					},
				)
			} else {
				clauses.pop().unwrap()
			};

			// The template may already be optional (an inner rewrite of a var
			// where clause produces one); the resolved implication or
			// conjunction is then the opt-bool overload, and the non-opt
			// condition needs its coercion materialised — opt erasure only
			// rewrites explicit `val2opt` coercions.
			let coerce_pair =
				|s: &mut Self, condition: Expression<'db, Dst>, template: Expression<'db, Dst>| {
					let coercion_target =
						Ty::most_specific_supertype(db, [template.ty(), condition.ty()]).unwrap();
					(
						add_coercion(db, &mut s.result, coercion_target, condition),
						add_coercion(db, &mut s.result, coercion_target, template),
					)
				};
			return match surrounding {
				SurroundingCall::Forall => {
					// Rewrite var where clauses into implications
					let (condition, template) = coerce_pair(self, condition, template);
					Expression::new(
						db,
						&self.result,
						origin,
						LookupCall {
							function: self.ids.functions.implies.into(),
							arguments: vec![condition, template],
						},
					)
				}
				SurroundingCall::Exists => {
					// Rewrite var where clauses into conjunctions
					let (condition, template) = coerce_pair(self, condition, template);
					Expression::new(
						db,
						&self.result,
						origin,
						LookupCall {
							function: self.ids.functions.and.into(),
							arguments: vec![condition, template],
						},
					)
				}
				SurroundingCall::Sum => {
					if template.ty().inst(db) == Some(VarType::Par) {
						// Rewrite var where clauses into linear sum
						let coercion_target =
							Ty::most_specific_supertype(db, [template.ty(), condition.ty()])
								.unwrap();
						let coerced_condition =
							add_coercion(db, &mut self.result, coercion_target, condition);
						Expression::new(
							db,
							&self.result,
							origin,
							LookupCall {
								function: self.ids.functions.times.into(),
								arguments: vec![coerced_condition, template],
							},
						)
					} else {
						// Rewrite var where clauses into if-then-else
						let zero = Expression::new(db, &self.result, origin, IntegerLiteral(0));
						let coerced_zero = add_coercion(db, &mut self.result, template.ty(), zero);
						Expression::new(
							db,
							&self.result,
							origin,
							IfThenElse {
								branches: vec![Branch {
									condition,
									result: template,
								}],
								else_result: Box::new(coerced_zero),
							},
						)
					}
				}
				SurroundingCall::Other => {
					// Rewrite var where clauses into optionality
					// Optionality coercion already done, so requires explicit types
					let opt_ty = template.ty().with_opt(db, OptType::Opt);
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

struct ScopeTester<'a, 'db, T: Marker> {
	gen_idx: &'a FxHashMap<DeclarationId<'db, T>, usize>,
	idx: usize,
}

impl<'a, 'db, T: Marker> Visitor<'_, 'db, T> for ScopeTester<'a, 'db, T> {
	fn visit_identifier(
		&mut self,
		_model: &Model<'db, T>,
		identifier: &ResolvedIdentifier<'db, T>,
	) {
		if let ResolvedIdentifier::Declaration(idx) = identifier
			&& let Some(idx) = self.gen_idx.get(idx)
		{
			self.idx = self.idx.max(*idx);
		}
	}
}

impl<'a, 'db, T: Marker> ScopeTester<'a, 'db, T> {
	/// Get the index plus one of the earliest comprehension to attach this to
	fn run(
		model: &Model<'db, T>,
		gen_idx: &'a FxHashMap<DeclarationId<'db, T>, usize>,
		expression: &Expression<'db, T>,
	) -> usize {
		let mut st = Self { gen_idx, idx: 0 };
		st.visit_expression(model, expression);
		st.idx
	}
}

/// Desugar comprehensions
pub fn desugar_comprehension<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Desugaring comprehensions");
	let mut r = ComprehensionRewriter {
		ids: IdentifierRegistry::lookup(db),
		replacement_map: ReplacementMap::default(),
		result: Model::default(),
	};
	r.add_model(db, &model);
	Ok(r.result)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::desugar_comprehension;
	use crate::transform::tests::check;

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
    predicate foo(var int: x);
    array [int] of var int: x;
    array [int] of var opt int: y = [if _DECL_1 then val2opt(x_i) else let {
      var opt int: _DECL_2 = <>;
    } in _DECL_2 endif | x_i in x, _DECL_1 :: (mzn_var_where_clause) = foo(x_i)];
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
    array [int] of var opt int: y = [if _DECL_1 then val2opt(x_i) else let {
      opt int: _DECL_2 = <>;
    } in _DECL_2 endif | x_i in ub(x), _DECL_1 :: (mzn_var_where_clause) = 'in'(x_i, x)];
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
    predicate foo(var int: x);
    function bool: bar(int: x);
    array [int] of var opt int: y = [if forall([_DECL_3, _DECL_2, _DECL_1]) then val2opt(x_i) else let {
      opt int: _DECL_4 = <>;
    } in _DECL_4 endif | x_i in ub(x) where bar(x_i), _DECL_1 :: (mzn_var_where_clause) = 'in'(x_i, x), _DECL_2 :: (mzn_var_where_clause) = foo(x_i), x_j in ub(x) where bar(x_j), _DECL_3 :: (mzn_var_where_clause) = 'in'(x_j, x)];
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
    predicate foo(var int: x);
    var set of int: S;
    constraint forall(['->'(_DECL_1, foo(i)) | i in ub(S), _DECL_1 :: (mzn_var_where_clause) = 'in'(i, S)]);
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
			expect!([r#"
    predicate foo(var int: x);
    var set of int: S;
    constraint exists(['/\'(_DECL_1, foo(i)) | i in ub(S), _DECL_1 :: (mzn_var_where_clause) = 'in'(i, S)]);
"#]),
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
    var int: x = sum(['*'(bool2int(_DECL_1), i) | i in ub(S), _DECL_1 :: (mzn_var_where_clause) = 'in'(i, S)]);
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
    var int: x = sum([if _DECL_1 then foo(i) else 0 endif | i in ub(S), _DECL_1 :: (mzn_var_where_clause) = 'in'(i, S)]);
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
    var set of int: x = mzn_array2set([foo(i) | i in S]);
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
    var set of int: x = mzn_array2set([if _DECL_1 then val2opt(foo(i)) else let {
      var opt int: _DECL_2 = <>;
    } in _DECL_2 endif | i in ub(S), _DECL_1 :: (mzn_var_where_clause) = 'in'(i, S)]);
"#]),
		)
	}
}
