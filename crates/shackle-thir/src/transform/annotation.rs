//! Rewriting annotations on expressions
//!
//! - Moves annotations from outer expressions to inner-most calls
//! - Explicitly attaches incoming annotations to a function to a body's subexpressions.

use shackle_diagnostics::Result;
use shackle_hir::{Db, constants::IdentifierRegistry};
use shackle_utils::maybe_grow_stack;

use crate::{
	Call, Callable, Declaration, Expression, ExpressionData, Item, Let, LetItem, Marker, Model,
	traverse::{
		Folder, ReplacementMap, fold_call, fold_expression, fold_expression_without_annotations,
		fold_function_body,
	},
};

struct AnnotationRewriter<'db, Dst: Marker, Src: Marker = ()> {
	replacement_map: ReplacementMap<'db, Dst, Src>,
	model: Model<'db, Dst>,
	ids: &'db IdentifierRegistry<'db>,
	in_function_body: bool,
	/// Annotations from surrounding expressions
	annotations: Vec<Expression<'db, Dst>>,
	/// Offset into the annotations vector for the current expression
	offsets: Vec<usize>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src>
	for AnnotationRewriter<'db, Dst, Src>
{
	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &'_ Model<'db, Src>,
		f: crate::FunctionId<'db, Src>,
	) {
		self.in_function_body = true;
		fold_function_body(self, db, model, f);
		self.in_function_body = false;
	}

	fn fold_call(
		&mut self,
		db: &'db dyn Db,
		model: &'_ Model<'db, Src>,
		call: &'_ Call<'db, Src>,
	) -> Call<'db, Dst> {
		if let Callable::Function(f) = &call.function
			&& (model[*f].name() == self.ids.functions.and
				|| model[*f].name() == self.ids.functions.forall)
		{
			// Propagate annotations into arguments
			return fold_call(self, db, model, call);
		}

		let function = self.fold_callable(db, model, &call.function);
		// Don't propagate annotations to the arguments
		self.offsets.push(self.annotations.len());
		let arguments = call
			.arguments
			.iter()
			.map(|arg| self.fold_expression(db, model, arg))
			.collect();
		let _ = self.offsets.pop();
		Call {
			function,
			arguments,
		}
	}

	fn fold_let(
		&mut self,
		db: &'db dyn Db,
		model: &'_ Model<'db, Src>,
		l: &'_ Let<'db, Src>,
	) -> Let<'db, Dst> {
		Let {
			items: l
				.items
				.iter()
				.map(|i| match i {
					LetItem::Constraint(c) => {
						self.add_constraint(db, model, *c);
						LetItem::Constraint(self.fold_constraint_id(db, model, *c))
					}
					LetItem::Declaration(d) => {
						// Don't propagate annotations to the declaration
						self.offsets.push(self.annotations.len());
						self.add_variable_declaration(db, model, *d);
						let idx = self.fold_declaration_id(db, model, *d);
						let _ = self.offsets.pop();
						LetItem::Declaration(idx)
					}
				})
				.collect(),
			in_expression: Box::new(self.fold_expression(db, model, &l.in_expression)),
		}
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &'_ Model<'db, Src>,
		expression: &'_ Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			if !expression.annotations().is_empty() {
				let mut items = Vec::new();
				for ann in expression.annotations().iter() {
					let folded = self.fold_expression(db, model, ann);
					if let ExpressionData::Identifier(_) = &*folded {
						self.annotations.push(folded);
					} else {
						let origin = folded.origin();
						let decl = Declaration::from_expression(db, false, folded);
						let idx = self.model.add_declaration(Item::new(decl, origin));
						self.annotations
							.push(Expression::new(db, &self.model, origin, idx));
						items.push(LetItem::Declaration(idx));
					}
				}
				let mut folded = fold_expression_without_annotations(self, db, model, expression);
				folded = self.attach_annotations(db, model, folded);
				for _ in expression.annotations().iter() {
					let _ = self.annotations.pop();
				}
				if !items.is_empty() {
					return Expression::new(
						db,
						&self.model,
						expression.origin(),
						Let {
							items,
							in_expression: Box::new(folded),
						},
					);
				}
				return folded;
			}

			let folded = fold_expression(self, db, model, expression);
			self.attach_annotations(db, model, folded)
		})
	}
}

impl<'db, Dst: Marker, Src: Marker> AnnotationRewriter<'db, Dst, Src> {
	fn attach_annotations(
		&mut self,
		db: &'db dyn Db,
		model: &'_ Model<'db, Src>,
		mut expr: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		if let ExpressionData::Call(_) = &*expr {
			let origin = expr.origin();
			for ann in self.annotations[self.offsets.last().copied().unwrap_or(0)..].iter() {
				expr.annotations_mut().push(ann.clone());
			}

			if self.in_function_body && self.offsets.is_empty() {
				let ident = model
					.lookup_identifier(db, self.ids.annotations.mzn_forward_annotations)
					.unwrap();
				let folded_ident = self.fold_identifier(db, model, &ident);
				expr.annotations_mut()
					.push(Expression::new(db, &self.model, origin, folded_ident));
			}
		}
		expr
	}
}

/// Desugar annotations
pub fn rewrite_annotations<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Desugaring comprehensions");
	let mut r = AnnotationRewriter {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: IdentifierRegistry::lookup(db),
		in_function_body: false,
		annotations: Vec::new(),
		offsets: Vec::new(),
	};
	r.add_model(db, &model);
	Ok(r.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check_no_stdlib};

	#[test]
	fn test_rewrite_annotations() {
		check_no_stdlib(
			vec![Transform::RewriteAnnotations],
			r#"
                annotation my_ann;
                predicate foo(var int: x);
                var int: x;
				any: y = [let {
                    var bool: z = foo(x);
                    constraint foo(x);
                } in 1] :: my_ann;
			"#,
			expect!([r#"
    annotation my_ann;
    predicate foo(var int: x);
    var int: x;
    array [int] of var int: y = [let {
      var bool: z = foo(x);
      constraint foo(x) :: (my_ann);
    } in 1];
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_rewrite_annotations_in_function() {
		check_no_stdlib(
			vec![Transform::RewriteAnnotations],
			r#"
                annotation mzn_forward_annotations;
                annotation my_ann;
                predicate foo(var int: x);
                function tuple(var int): bar() = let {
                    var int: x;
                    any: y = [let {
                        constraint foo(x);
                    } in 1];
                    constraint foo(x);
                } in (let {
                    constraint foo(x);
                } in (x,)) :: my_ann;
			"#,
			expect!([r#"
    annotation mzn_forward_annotations;
    annotation my_ann;
    predicate foo(var int: x);
    function tuple(var int): bar() = let {
      var int: x;
      array [int] of var int: y = [let {
      constraint foo(x);
    } in 1];
      constraint foo(x) :: (mzn_forward_annotations);
    } in let {
      constraint foo(x) :: (my_ann) :: (mzn_forward_annotations);
    } in (x,);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_rewrite_annotations_respect_call_boundaries() {
		check_no_stdlib(
			vec![Transform::RewriteAnnotations],
			r#"
                annotation outer;
                predicate p(var int: x);
                predicate '/\'(var bool: x, var bool: y);
                predicate 'forall'(array [int] of var bool: xs);
                predicate wrapper(array [int] of var bool: xs);
                constraint (p(1) /\ p(2)) :: outer;
                constraint 'forall'([p(3), p(4)]) :: outer;
                constraint wrapper([p(5), p(6)]) :: outer;
			"#,
			expect![[r#"
    annotation outer;
    predicate p(var int: x);
    function var bool: '/\'(var bool: x, var bool: y);
    predicate forall(array [int] of var bool: xs);
    predicate wrapper(array [int] of var bool: xs);
    constraint '/\'(p(1) :: (outer), p(2) :: (outer)) :: (outer);
    constraint forall([p(3) :: (outer), p(4) :: (outer)]) :: (outer);
    constraint wrapper([p(5), p(6)]) :: (outer);
    solve satisfy;
"#]],
		)
	}

	#[test]
	fn test_rewrite_parameterised_annotations() {
		check_no_stdlib(
			vec![Transform::RewriteAnnotations],
			r#"
                annotation ann(int: value);
                predicate p(var int: x);
                constraint p(1) :: ann(2);
			"#,
			expect![[r#"
    annotation 'ann'(int: value);
    predicate p(var int: x);
    constraint let {
      ann: _DECL_3 = 'ann'(2);
    } in p(1) :: (_DECL_3);
    solve satisfy;
"#]],
		)
	}
}
