//! Generate function dispatch headers and mangle names
//!
//!

use std::sync::Arc;

use crate::{
	constants::IdentifierRegistry,
	thir::{
		db::Thir,
		traverse::{Folder, ReplacementMap},
		ArrayLiteral, Branch, Call, Callable, Expression, FunctionId, IfThenElse, LookupCall,
		Marker, Model,
	},
};
struct DispatchRewriter<Dst, Src = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for DispatchRewriter<Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		let dst = self.fold_function_id(db, model, f);
		let folded = self.fold_expression(db, model, model[f].body().unwrap());
		let (orig_tys, par_tys) = self.model[dst]
			.parameters()
			.iter()
			.map(|p| {
				let ty = self.model[*p].ty();
				(ty, ty.make_par(db.upcast()))
			})
			.unzip::<_, _, Vec<_>, Vec<_>>();
		if orig_tys.iter().zip(par_tys.iter()).any(|(o, p)| *o != *p) {
			if let Ok(f) = self.model.lookup_function(db, model[f].name(), &par_tys) {
				if f.function != dst {
					let mut is_fixed = Vec::with_capacity(orig_tys.len());
					let mut arguments = Vec::with_capacity(orig_tys.len());

					for (p, (orig_ty, par_ty)) in self.model[dst]
						.parameters()
						.iter()
						.zip(orig_tys.iter().zip(par_tys.iter()))
					{
						let origin = self.model[*p].origin();
						let param = Expression::new(db, &self.model, origin, *p);
						if *orig_ty != *par_ty {
							is_fixed.push(Expression::new(
								db,
								&self.model,
								origin,
								LookupCall {
									function: self.ids.is_fixed.into(),
									arguments: vec![param.clone()],
								},
							));
							arguments.push(Expression::new(
								db,
								&self.model,
								origin,
								LookupCall {
									function: self.ids.fix.into(),
									arguments: vec![param.clone()],
								},
							));
						} else {
							arguments.push(param);
						}
					}

					let condition = if is_fixed.len() > 1 {
						Expression::new(
							db,
							&self.model,
							self.model[dst].origin(),
							LookupCall {
								function: self.ids.forall.into(),
								arguments: vec![Expression::new(
									db,
									&self.model,
									self.model[dst].origin(),
									ArrayLiteral(is_fixed),
								)],
							},
						)
					} else {
						is_fixed.pop().unwrap()
					};
					let result = Expression::new(
						db,
						&self.model,
						self.model[dst].origin(),
						Call {
							function: Callable::Function(f.function),
							arguments,
						},
					);

					let body = Expression::new(
						db,
						&self.model,
						self.model[dst].origin(),
						IfThenElse {
							branches: vec![Branch { condition, result }],
							else_result: Box::new(folded),
						},
					);
					self.model[dst].set_body(body);

					return;
				}
			}
		}
		self.model[dst].set_body(folded);
	}
}

/// Add function dispatch headers
pub fn function_dispatch(db: &dyn Thir, model: &Model) -> Model {
	let mut c = DispatchRewriter {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
	};
	c.add_model(db, model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check_no_stdlib;

	use super::function_dispatch;

	#[test]
	fn test_function_dispatch() {
		check_no_stdlib(
			function_dispatch,
			r#"
            predicate foo(var int: x) = true;
            test foo(int: x) = false;
            test is_fixed(var int: x);
            function int: fix(var int: x);
            "#,
			expect!([r#"
    function var bool: foo(var int: x) = if is_fixed(x) then foo(fix(x)) else true endif;
    function bool: foo(int: x) = false;
    function bool: is_fixed(var int: x);
    function int: fix(var int: x);
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_function_dispatch_2() {
		check_no_stdlib(
			function_dispatch,
			r#"
            predicate foo(var int: x, var bool: b) = true;
            test foo(int: x, bool: b) = false;
            test is_fixed(var int: x);
            test is_fixed(var bool: x);
            function int: fix(var int: x);
            function bool: fix(var bool: x);
            test 'forall'(array [int] of bool: x);
            "#,
			expect!([r#"
    function var bool: foo(var int: x, var bool: b) = if forall([is_fixed(x), is_fixed(b)]) then foo(fix(x), fix(b)) else true endif;
    function bool: foo(int: x, bool: b) = false;
    function bool: is_fixed(var int: x);
    function bool: is_fixed(var bool: x);
    function int: fix(var int: x);
    function bool: fix(var bool: x);
    function bool: forall(array [int] of bool: x);
    solve satisfy;
"#]),
		);
	}
}
