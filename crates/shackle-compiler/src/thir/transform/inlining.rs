//! Performs inlining of `:: mzn_inline_call_by_name` functions.
//! - Replaces calls with the bodies of functions
//! - Replaces references to parameters with the corresponding call argument

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::IdentifierRegistry,
	thir::{
		db::Thir,
		traverse::{
			add_function, fold_declaration, fold_expression, fold_function_body, Folder,
			ReplacementMap,
		},
		Callable, Declaration, DeclarationId, Expression, ExpressionData, FunctionId, Item, Let,
		LetItem, Marker, Model, ResolvedIdentifier,
	},
	utils::maybe_grow_stack,
	Result,
};

struct Inliner<Dst: Marker, Src: Marker = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	map: FxHashMap<DeclarationId<Src>, Expression<Dst>>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for Inliner<Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f]
			.annotations()
			.has(model, self.ids.mzn_inline_call_by_name)
			|| model[f].annotations().has(model, self.ids.mzn_inline)
		{
			// Remove inlined function
			return;
		}
		add_function(self, db, model, f);
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f]
			.annotations()
			.has(model, self.ids.mzn_inline_call_by_name)
			|| model[f].annotations().has(model, self.ids.mzn_inline)
		{
			// Remove inlined function
			return;
		}
		fold_function_body(self, db, model, f)
	}

	fn fold_declaration(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		d: &Declaration<Src>,
	) -> Declaration<Dst> {
		let mut folded = fold_declaration(self, db, model, d);
		if !self.map.is_empty() {
			// Alpha rename for safety when inlining
			folded.remove_name();
		}
		folded
	}

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		expression: &Expression<Src>,
	) -> Expression<Dst> {
		maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) => {
					if let Some(e) = self.map.get(d) {
						return e.clone();
					}
				}
				ExpressionData::Call(c) => {
					if let Callable::Function(f) = &c.function {
						if let Some(body) = model[*f].body() {
							if model[*f].annotations().has(model, self.ids.mzn_inline)
								|| is_macro_call(body)
							{
								log::debug!(
									"Inlining {} using call by value semantics",
									model[*f].name().pretty_print(db)
								);

								let mut restore = Vec::with_capacity(c.arguments.len());
								let items = c
									.arguments
									.iter()
									.zip(model[*f].parameters().iter())
									.filter_map(|(arg, param)| {
										let origin = arg.origin();
										let folded = self.fold_expression(db, model, arg);
										if matches!(&*folded, ExpressionData::Identifier(_)) {
											restore.push(self.map.insert(*param, folded));
											None
										} else {
											let decl =
												Declaration::from_expression(db, false, folded);
											let idx =
												self.model.add_declaration(Item::new(decl, origin));
											let ident =
												Expression::new(db, &self.model, origin, idx);
											restore.push(self.map.insert(*param, ident));
											Some(LetItem::Declaration(idx))
										}
									})
									.collect::<Vec<_>>();
								let inlined = self.fold_expression(db, model, body);
								for (param, prev) in model[*f].parameters().iter().zip(restore) {
									if let Some(prev) = prev {
										self.map.insert(*param, prev);
									} else {
										self.map.remove(param);
									}
								}
								return if items.is_empty() {
									inlined
								} else {
									Expression::new(
										db,
										&self.model,
										expression.origin(),
										Let {
											items,
											in_expression: Box::new(inlined),
										},
									)
								};
							}
							if model[*f]
								.annotations()
								.has(model, self.ids.mzn_inline_call_by_name)
							{
								log::debug!(
									"Inlining {} using call by name semantics",
									model[*f].name().pretty_print(db)
								);
								let mut restore = Vec::with_capacity(c.arguments.len());
								for (param, arg) in
									model[*f].parameters().iter().zip(c.arguments.iter())
								{
									let e = self.fold_expression(db, model, arg);
									restore.push(self.map.insert(*param, e));
								}
								let mut inlined = self.fold_expression(db, model, body);
								inlined.set_origin(expression.origin());
								for (param, prev) in model[*f].parameters().iter().zip(restore) {
									if let Some(prev) = prev {
										self.map.insert(*param, prev);
									} else {
										self.map.remove(param);
									}
								}
								return inlined;
							}
						}
					}
				}
				_ => (),
			}
			fold_expression(self, db, model, expression)
		})
	}
}

fn is_macro_call<T: Marker>(expression: &Expression<T>) -> bool {
	let ExpressionData::Call(c) = &**expression else {
		return false;
	};
	c.arguments
		.iter()
		.all(|arg| matches!(&**arg, ExpressionData::Identifier(_)))
}

/// Perform inlining to implement call-by-name semantics for functions annotated with
/// `:: mzn_inline_call_by_name`.
pub fn inline_functions(db: &dyn Thir, model: Model) -> Result<Model> {
	log::info!("Inlining functions");
	let mut inliner = Inliner {
		replacement_map: ReplacementMap::default(),
		model: Model::with_capacities(&model.entity_counts()),
		ids: db.identifier_registry(),
		map: FxHashMap::default(),
	};
	inliner.add_model(db, &model);
	Ok(inliner.model)
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use super::inline_functions;
	use crate::thir::transform::test::check_no_stdlib;

	#[test]
	fn test_inline_call_by_name() {
		check_no_stdlib(
			inline_functions,
			r#"
				annotation mzn_inline_call_by_name;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline_call_by_name =
                    if a then b else c endif;
                any: x = foo(true, 1, 2);
			"#,
			expect!([r#"
    annotation mzn_inline_call_by_name;
    int: x = if true then 1 else 2 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_inline_alpha_rename() {
		check_no_stdlib(
			inline_functions,
			r#"
				annotation mzn_inline_call_by_name;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline_call_by_name =
                    let {
                        float: p = 2.5;
                    } in if a then b else c endif;
                int: p = 1;
                any: x = foo(true, p, 2);
			"#,
			expect!([r#"
    annotation mzn_inline_call_by_name;
    int: p = 1;
    int: x = let {
      float: _DECL_2 = 2.5;
    } in if true then p else 2 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_inline_call_by_value() {
		check_no_stdlib(
			inline_functions,
			r#"
				annotation mzn_inline;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline =
                    if a then bar(b, b) else bar(b, c) endif;
				function int: bar(int: x, int: y);
                any: x = foo(true, 1, 2);
			"#,
			expect!([r#"
    annotation mzn_inline;
    function int: bar(int: x, int: y);
    int: x = let {
      bool: _DECL_3 = true;
      int: _DECL_4 = 1;
      int: _DECL_5 = 2;
    } in if _DECL_3 then bar(_DECL_4, _DECL_4) else bar(_DECL_4, _DECL_5) endif;
    solve satisfy;
"#]),
		)
	}
}
