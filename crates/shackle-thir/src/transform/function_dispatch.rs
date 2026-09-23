//! Generate function preambles to dispatch to par/non-opt versions of functions.
//!

use rustc_hash::{FxHashMap, FxHashSet};
use shackle_diagnostics::Result;
use shackle_hir::{Identifier, IntegerLiteral, OptType, VarType, constants::IdentifierRegistry};
use shackle_ty::{Ty, TyData};

use super::top_down_type::add_coercion;
use crate::{
	ArrayComprehension, ArrayLiteral, Branch, Call, Callable, Db, Declaration, Domain, Expression,
	FunctionId, FunctionName, Generator, IfThenElse, Item, LookupCall, Marker, Model, RecordAccess,
	RecordLiteral, SetComprehension, TupleAccess, TupleLiteral,
	pretty_print::PrettyPrinter,
	traverse::{Folder, ReplacementMap, add_function, fold_function_body},
};

struct DispatchRewriter<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	ids: &'db IdentifierRegistry<'db>,
	dispatch_to: FxHashMap<FunctionId<'db, Src>, Vec<FunctionId<'db, Src>>>,
	overloaded: FxHashMap<Identifier<'db>, Vec<FunctionId<'db, Dst>>>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for DispatchRewriter<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		let idx = add_function(self, db, model, f);
		if model[f].top_level()
			&& model[f].body().is_some()
			&& let FunctionName::Named(name) = model[f].name()
		{
			self.overloaded.entry(name).or_default().push(idx);
		}
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		fold_function_body(self, db, model, f);
		if let Some(mut dispatch_to) = self.dispatch_to.remove(&f) {
			dispatch_to.sort();
			let mut branches = Vec::with_capacity(dispatch_to.len());
			for target in dispatch_to {
				log::debug!(
					"Adding dispatch from {} to {}",
					PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
					PrettyPrinter::new(db, model).pretty_print_signature(target.into()),
				);
				let mut condition = Vec::new();
				let mut arguments = Vec::with_capacity(model[f].parameters().len());
				for (from, to) in model[f].parameters().iter().zip(model[target].parameters()) {
					let d = self.fold_declaration_id(db, model, *from);
					let param = Expression::new(db, &self.model, model[*from].origin(), d);
					let dispatched = self.dispatch_param(
						db,
						param.clone(),
						param,
						model[*from].ty(),
						model[*to].ty(),
						&mut condition,
					);
					arguments.push(dispatched);
				}
				log::debug!(
					"Conditions: {}",
					condition
						.iter()
						.map(|c| PrettyPrinter::new(db, &self.model).pretty_print_expression(c))
						.collect::<Vec<_>>()
						.join(" /\\ ")
				);
				log::debug!(
					"Arguments: {}",
					arguments
						.iter()
						.map(|e| PrettyPrinter::new(db, &self.model).pretty_print_expression(e))
						.collect::<Vec<_>>()
						.join(", ")
				);
				let folded_target = self.fold_function_id(db, model, target);
				let call = Expression::new(
					db,
					&self.model,
					model[f].origin(),
					Call {
						function: Callable::Function(folded_target),
						arguments,
					},
				);
				let result = add_coercion(db, &mut self.model, model[f].return_type(), call);
				branches.push(Branch {
					condition: if condition.len() == 1 {
						condition.pop().unwrap()
					} else {
						Expression::new(
							db,
							&self.model,
							model[f].origin(),
							LookupCall {
								function: self.ids.functions.forall.into(),
								arguments: vec![Expression::new(
									db,
									&self.model,
									model[f].origin(),
									ArrayLiteral(condition),
								)],
							},
						)
					},
					result,
				});
			}
			let idx = self.fold_function_id(db, model, f);
			let body = self.model[idx].take_body().unwrap();
			let with_preamble = Expression::new(
				db,
				&self.model,
				model[f].origin(),
				IfThenElse {
					branches,
					else_result: Box::new(body),
				},
			);
			self.model[idx].set_body(with_preamble);
			self.model[idx].validate(db);
		}
	}
}

impl<'db, Src: Marker, Dst: Marker> DispatchRewriter<'db, Dst, Src> {
	fn call(
		&self,
		db: &'db dyn Db,
		name: Identifier<'db>,
		arg: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.model,
			arg.origin(),
			LookupCall {
				function: name.into(),
				arguments: vec![arg],
			},
		)
	}

	fn and(
		&self,
		db: &'db dyn Db,
		lhs: Expression<'db, Dst>,
		rhs: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.model,
			lhs.origin(),
			LookupCall {
				function: self.ids.functions.and.into(),
				arguments: vec![lhs, rhs],
			},
		)
	}

	fn occurs(&self, db: &'db dyn Db, e: Expression<'db, Dst>) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.model,
			e.origin(),
			TupleAccess {
				tuple: Box::new(e),
				field: IntegerLiteral(1),
			},
		)
	}

	fn deopt(&self, db: &'db dyn Db, e: Expression<'db, Dst>) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.model,
			e.origin(),
			TupleAccess {
				tuple: Box::new(e),
				field: IntegerLiteral(2),
			},
		)
	}

	fn pair(
		&self,
		db: &'db dyn Db,
		occurs: Expression<'db, Dst>,
		deopt: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.model,
			deopt.origin(),
			TupleLiteral(vec![occurs, deopt]),
		)
	}

	fn dispatch_param(
		&mut self,
		db: &'db dyn Db,
		ce: Expression<'db, Dst>,
		ve: Expression<'db, Dst>,
		a: Ty<'db>,
		b: Ty<'db>,
		condition: &mut Vec<Expression<'db, Dst>>,
	) -> Expression<'db, Dst> {
		if a == b {
			// Same type, nothing needed
			return ve;
		}
		let origin = ve.origin();
		match (
			a.inst(db).unwrap(),
			a.opt(db).unwrap(),
			b.inst(db).unwrap(),
			b.opt(db).unwrap(),
		) {
			(VarType::Var, OptType::Opt, VarType::Var, OptType::Opt)
			| (VarType::Par, OptType::Opt, VarType::Par, OptType::Opt) => {
				// var opt T -> var opt U, opt T -> opt U
				let destruct_ce = self.call(db, self.ids.functions.mzn_destruct_opt, ce);
				let destruct_ve = self.call(db, self.ids.functions.mzn_destruct_opt, ve);
				let deopt_ce = self.deopt(db, destruct_ce);
				let deopt_ve = self.deopt(db, destruct_ve.clone());
				let deopt_dispatch = self.dispatch_param(
					db,
					deopt_ce,
					deopt_ve,
					a.make_occurs(db),
					b.make_occurs(db),
					condition,
				);
				return self.call(
					db,
					self.ids.functions.mzn_construct_opt,
					self.pair(db, self.occurs(db, destruct_ve), deopt_dispatch),
				);
			}
			(VarType::Var, OptType::Opt, VarType::Var, OptType::NonOpt) => {
				// var opt T -> var U
				let destruct_ce = self.call(db, self.ids.functions.mzn_destruct_opt, ce);
				let destruct_ve = self.call(db, self.ids.functions.mzn_destruct_opt, ve);
				let occurs = self.occurs(db, destruct_ce.clone());
				let occurs_is_fixed = self.call(db, self.ids.functions.is_fixed, occurs.clone());
				let fix_occurs = self.call(db, self.ids.functions.fix, occurs);
				condition.push(self.and(db, occurs_is_fixed, fix_occurs));
				let deopt_ce = self.deopt(db, destruct_ce);
				let deopt_ve = self.deopt(db, destruct_ve);
				return self.dispatch_param(
					db,
					deopt_ce,
					deopt_ve,
					a.make_occurs(db),
					b,
					condition,
				);
			}
			(VarType::Var, OptType::Opt, VarType::Par, _) => {
				// var opt T -> opt U, var opt T -> U
				let destruct_ce = self.call(db, self.ids.functions.mzn_destruct_opt, ce);
				let destruct_ve = self.call(db, self.ids.functions.mzn_destruct_opt, ve);
				condition.push(self.call(db, self.ids.functions.is_fixed, destruct_ce.clone()));
				let fixed_ce = self.call(
					db,
					self.ids.functions.mzn_construct_opt,
					self.call(db, self.ids.functions.fix, destruct_ce),
				);
				let fixed_ve = self.call(
					db,
					self.ids.functions.mzn_construct_opt,
					self.call(db, self.ids.functions.fix, destruct_ve),
				);
				return self.dispatch_param(db, fixed_ce, fixed_ve, a.make_par(db), b, condition);
			}
			(VarType::Var, OptType::NonOpt, VarType::Par, OptType::NonOpt) => {
				// var T -> U
				condition.push(self.call(db, self.ids.functions.is_fixed, ce.clone()));
				let fix_ce = self.call(db, self.ids.functions.fix, ce);
				let fix_ve = self.call(db, self.ids.functions.fix, ve);
				return self.dispatch_param(db, fix_ce, fix_ve, a.make_par(db), b, condition);
			}
			(VarType::Par, OptType::Opt, VarType::Par, OptType::NonOpt) => {
				// opt T -> U
				let destruct_ce = self.call(db, self.ids.functions.mzn_destruct_opt, ce);
				let destruct_ve = self.call(db, self.ids.functions.mzn_destruct_opt, ve);
				condition.push(self.occurs(db, destruct_ce.clone()));
				let deopt_ce = self.deopt(db, destruct_ce);
				let deopt_ve = self.deopt(db, destruct_ve);
				return self.dispatch_param(
					db,
					deopt_ce,
					deopt_ve,
					a.make_occurs(db),
					b,
					condition,
				);
			}
			(VarType::Par, OptType::NonOpt, _, _) => (),
			(a, b, c, d) => unreachable!("Invalid dispatch {:?}, {:?} to {:?}, {:?}", a, b, c, d),
		}
		match (a.lookup(db), b.lookup(db)) {
			(TyData::Array { element: e1, .. }, TyData::Array { element: e2, .. }) => {
				let c_decl = Declaration::new(false, Domain::unbounded(db, origin, *e1));
				let c_idx = self.model.add_declaration(Item::new(c_decl, origin));
				let c_exp = Expression::new(db, &self.model, origin, c_idx);

				let v_decl = Declaration::new(false, Domain::unbounded(db, origin, *e1));
				let v_idx = self.model.add_declaration(Item::new(v_decl, origin));
				let v_exp = Expression::new(db, &self.model, origin, v_idx);

				let mut cs = Vec::new();
				let template = Box::new(self.dispatch_param(db, c_exp, v_exp, *e1, *e2, &mut cs));
				condition.push(self.call(
					db,
					self.ids.functions.forall,
					Expression::new(
						db,
						&self.model,
						origin,
						ArrayComprehension {
							generators: vec![Generator::Iterator {
								declarations: vec![c_idx],
								collection: ce,
								where_clause: None,
							}],
							indices: None,
							template: Box::new(if cs.len() == 1 {
								cs.pop().unwrap()
							} else {
								self.call(
									db,
									self.ids.functions.forall,
									Expression::new(db, &self.model, origin, ArrayLiteral(cs)),
								)
							}),
						},
					),
				));

				let array = Expression::new(
					db,
					&self.model,
					origin,
					ArrayComprehension {
						generators: vec![Generator::Iterator {
							declarations: vec![v_idx],
							collection: ve.clone(),
							where_clause: None,
						}],
						indices: None,
						template,
					},
				);
				Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.array_xd.into(),
						arguments: vec![ve, array],
					},
				)
			}
			(TyData::Set(_, _, e1), TyData::Set(_, _, e2)) => {
				let c_decl = Declaration::new(false, Domain::unbounded(db, origin, *e1));
				let c_idx = self.model.add_declaration(Item::new(c_decl, origin));
				let c_exp = Expression::new(db, &self.model, origin, c_idx);

				let v_decl = Declaration::new(false, Domain::unbounded(db, origin, *e1));
				let v_idx = self.model.add_declaration(Item::new(v_decl, origin));
				let v_exp = Expression::new(db, &self.model, origin, v_idx);

				let mut cs = Vec::new();
				let template = Box::new(self.dispatch_param(db, c_exp, v_exp, *e1, *e2, &mut cs));
				condition.push(Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.forall.into(),
						arguments: vec![Expression::new(
							db,
							&self.model,
							origin,
							ArrayComprehension {
								generators: vec![Generator::Iterator {
									declarations: vec![c_idx],
									collection: ce,
									where_clause: None,
								}],
								indices: None,
								template: Box::new(if cs.len() == 1 {
									cs.pop().unwrap()
								} else {
									Expression::new(
										db,
										&self.model,
										origin,
										LookupCall {
											function: self.ids.functions.forall.into(),
											arguments: vec![Expression::new(
												db,
												&self.model,
												origin,
												ArrayLiteral(cs),
											)],
										},
									)
								}),
							},
						)],
					},
				));
				Expression::new(
					db,
					&self.model,
					origin,
					SetComprehension {
						generators: vec![Generator::Iterator {
							declarations: vec![v_idx],
							collection: ve,
							where_clause: None,
						}],
						template,
					},
				)
			}
			(TyData::Tuple(_, f1), TyData::Tuple(_, f2)) => {
				let fields = f1
					.iter()
					.zip(f2.iter())
					.enumerate()
					.map(|(i, (d1, d2))| {
						let ce = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(ce.clone()),
								field: IntegerLiteral(i as i64 + 1),
							},
						);
						let ve = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(ve.clone()),
								field: IntegerLiteral(i as i64 + 1),
							},
						);
						self.dispatch_param(db, ce, ve, *d1, *d2, condition)
					})
					.collect::<Vec<_>>();
				Expression::new(db, &self.model, origin, TupleLiteral(fields))
			}
			(TyData::Record(_, f1), TyData::Record(_, f2)) => {
				let fields = f1
					.iter()
					.zip(f2.iter())
					.map(|((i, d1), (_, d2))| {
						let ce = Expression::new(
							db,
							&self.model,
							origin,
							RecordAccess {
								record: Box::new(ce.clone()),
								field: (*i).into(),
							},
						);
						let ve = Expression::new(
							db,
							&self.model,
							origin,
							RecordAccess {
								record: Box::new(ve.clone()),
								field: (*i).into(),
							},
						);
						(
							Identifier::from(*i),
							self.dispatch_param(db, ce, ve, *d1, *d2, condition),
						)
					})
					.collect::<Vec<_>>();
				Expression::new(db, &self.model, origin, RecordLiteral(fields))
			}
			_ => unreachable!(
				"Cannot dispatch {} to {}",
				a.pretty_print(db),
				b.pretty_print(db)
			),
		}
	}
}

fn dispatches_to<'db>(db: &'db dyn Db, a: Ty<'db>, b: Ty<'db>) -> bool {
	if a == b {
		return true;
	}
	if a.inst(db) == Some(VarType::Var) && b.inst(db) == Some(VarType::Par) {
		// Dispatch to par
		return dispatches_to(db, a.make_par(db), b);
	}
	if a.inst(db) != b.inst(db) {
		return false;
	}
	if a.opt(db) == Some(OptType::Opt) && b.opt(db) == Some(OptType::NonOpt) {
		// Dispatch to non-opt
		return dispatches_to(db, a.make_occurs(db), b);
	}
	if a.opt(db) != b.opt(db) {
		return false;
	}
	match (a.lookup(db), b.lookup(db)) {
		(
			TyData::Array {
				dim: d1,
				element: e1,
				..
			},
			TyData::Array {
				dim: d2,
				element: e2,
				..
			},
		) => d1 == d2 && dispatches_to(db, *e1, *e2),
		(TyData::Set(_, _, e1), TyData::Set(_, _, e2)) => dispatches_to(db, *e1, *e2),
		(TyData::Tuple(_, f1), TyData::Tuple(_, f2)) => {
			f1.len() == f2.len()
				&& f1
					.iter()
					.zip(f2.iter())
					.all(|(x, y)| dispatches_to(db, *x, *y))
		}
		(TyData::Record(_, f1), TyData::Record(_, f2)) => {
			f1.len() == f2.len()
				&& f1
					.iter()
					.zip(f2.iter())
					.all(|((i1, x), (i2, y))| *i1 == *i2 && dispatches_to(db, *x, *y))
		}
		_ => false,
	}
}

/// Add function dispatch headers
pub fn function_dispatch<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Generating function dispatch preambles");

	let ids = IdentifierRegistry::lookup(db);
	let mut overloaded = model.overload_map();
	overloaded.filter(|function| {
		!function
			.annotations()
			.has(&model, ids.annotations.mzn_inline_call_by_name)
			&& !function
				.annotations()
				.has(&model, ids.annotations.mzn_inline)
			&& function.body().is_some()
	});

	let mut dispatch_to: FxHashMap<_, Vec<_>> = FxHashMap::default();
	for overloads in overloaded.values() {
		if overloads.len() <= 1 {
			continue;
		}
		let mut edges = FxHashSet::default();
		for a in overloads.iter().copied() {
			for b in overloads.iter().copied() {
				if a != b
					&& model[a].parameters().len() == model[b].parameters().len()
					&& model[a]
						.parameters()
						.iter()
						.zip(model[b].parameters().iter())
						.all(|(pa, pb)| model[*pa].name() == model[*pb].name())
					&& (model[a].specialised_from().is_none()
						|| model[a].specialised_from().is_none()
						|| model[a].specialised_from() != model[b].specialised_from())
				{
					let b_more_specific = model[a]
						.parameters()
						.iter()
						.zip(model[b].parameters())
						.all(|(pa, pb)| dispatches_to(db, model[*pa].ty(), model[*pb].ty()));
					if b_more_specific {
						let _ = edges.insert((a, b));
					}
				}
			}
		}

		for a in overloads.iter().copied() {
			for b in overloads.iter().copied() {
				if edges.contains(&(a, b)) {
					for c in overloads.iter().copied() {
						if edges.contains(&(b, c)) {
							let _ = edges.remove(&(a, c));
						}
					}
				}
			}
		}

		for (a, b) in edges {
			dispatch_to.entry(a).or_default().push(b);
		}
	}

	let mut c = DispatchRewriter {
		model: Model::with_capacities(&model.item_counts()),
		replacement_map: ReplacementMap::default(),
		ids,
		dispatch_to,
		overloaded: FxHashMap::default(),
	};
	c.add_model(db, &model);
	Ok(c.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check};

	#[test]
	fn test_function_dispatch() {
		check(
			Transform::FunctionDispatch,
			r#"
            predicate foo(var int: x) = true;
            predicate foo(var int: x, var bool: b) = true;
            test foo(int: x, bool: b) = false;
            test foo(int: x) = false;
            "#,
			expect!([r#"
    function var bool: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else true endif;
    function var bool: foo(var int: x, var bool: b) = if forall([is_fixed(x), is_fixed(b)]) then foo(fix(x), fix(b)) else true endif;
    function bool: foo(int: x, bool: b) = false;
    function bool: foo(int: x) = false;
"#]),
		);
	}

	#[test]
	fn test_function_dispatch_2() {
		check(
			Transform::FunctionDispatch,
			r#"
            predicate foo(var opt int: x) = true;
            predicate foo(var int: x) = true;
            predicate foo(opt int: x) = true;
            predicate foo(int: x) = true;
            "#,
			expect!([r#"
    function var bool: foo(var opt int: x) = if '/\'(is_fixed((mzn_destruct_opt(x)).1), fix((mzn_destruct_opt(x)).1)) then foo((mzn_destruct_opt(x)).2) elseif is_fixed(mzn_destruct_opt(x)) then foo(mzn_construct_opt(fix(mzn_destruct_opt(x)))) else true endif;
    function var bool: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else true endif;
    function var bool: foo(opt int: x) = if (mzn_destruct_opt(x)).1 then foo((mzn_destruct_opt(x)).2) else true endif;
    function var bool: foo(int: x) = true;
"#]),
		);
	}

	#[test]
	fn test_function_dispatch_struct() {
		check(
			Transform::FunctionDispatch,
			r#"
            predicate foo(tuple(tuple(var int)): x) = true;
            predicate foo(tuple(tuple(int)): x) = true;
            predicate bar(tuple(tuple(var int, var int)): x) = true;
            predicate bar(tuple(tuple(var int, int)): x) = true;
            "#,
			expect!([r#"
    function var bool: foo(tuple(tuple(var int)): x) = if is_fixed(((x).1).1) then foo(((fix(((x).1).1),),)) else true endif;
    function var bool: foo(tuple(tuple(int)): x) = true;
    function var bool: bar(tuple(tuple(var int, var int)): x) = if is_fixed(((x).1).2) then bar(((((x).1).1, fix(((x).1).2)),)) else true endif;
    function var bool: bar(tuple(tuple(var int, int)): x) = true;
"#]),
		);
	}

	#[test]
	fn test_function_dispatch_specialised() {
		check(
			vec![Transform::TypeSpecialise, Transform::FunctionDispatch],
			r#"
            test foo(any $T: x) = true;
            test foo($$E: x) = false;
			var float: x;
			var int: y;
			constraint foo(x);
			constraint foo(y);
			constraint foo(1.5);
            "#,
			expect!([r#"
    function bool: foo(float: x) = true;
    function bool: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else true endif;
    function bool: foo(var float: x) = true;
    function bool: foo(int: x) = false;
    var float: x;
    var int: y;
    constraint foo(x);
    constraint foo(y);
    constraint foo(1.5);
"#]),
		);
	}

	#[test]
	fn test_function_dispatch_names() {
		check(
			Transform::FunctionDispatch,
			r#"
            function int: foo(int: x) = 3;
			function int: foo(int: y) = 4;
			function var int: foo(var int: x) = 5;
            "#,
			expect!([r#"
    function int: foo(int: x) = 3;
    function int: foo(int: y) = 4;
    function var int: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else 5 endif;
"#]),
		);
	}
}
