//! Performs inlining of `:: mzn_inline_call_by_name` functions.
//! - Replaces calls with the bodies of functions
//! - Replaces references to parameters with the corresponding call argument

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::constants::IdentifierRegistry;
use shackle_utils::maybe_grow_stack;

use crate::{
	Callable, Db, Declaration, DeclarationId, Domain, Expression, ExpressionData, FunctionId, Item,
	Let, LetItem, Marker, Model, ResolvedIdentifier,
	pretty_print::PrettyPrinter,
	traverse::{
		Folder, ReplacementMap, add_function, fold_declaration, fold_expression,
		fold_function_body, fold_function_id,
	},
};

struct Inliner<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	ids: &'db IdentifierRegistry<'db>,
	map: FxHashMap<DeclarationId<'db, Src>, Expression<'db, Dst>>,
	depth: u16,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for Inliner<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		if model[f]
			.annotations()
			.has(model, self.ids.annotations.mzn_inline_call_by_name)
			|| model[f]
				.annotations()
				.has(model, self.ids.annotations.mzn_inline)
		{
			// Remove inlined function
			return;
		}
		if model[f].top_level() {
			let _ = fold_function_id(self, db, model, f);
		} else {
			let _ = add_function(self, db, model, f);
		}
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		if model[f]
			.annotations()
			.has(model, self.ids.annotations.mzn_inline_call_by_name)
			|| model[f]
				.annotations()
				.has(model, self.ids.annotations.mzn_inline)
		{
			// Remove inlined function
			return;
		}
		fold_function_body(self, db, model, f)
	}

	fn fold_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		d: &Declaration<'db, Src>,
	) -> Declaration<'db, Dst> {
		let mut folded = fold_declaration(self, db, model, d);
		if !self.map.is_empty() {
			// Alpha rename for safety when inlining
			folded.remove_name();
		}
		folded
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		expression: &Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) => {
					if let Some(e) = self.map.get(d) {
						return e.clone();
					}
				}
				ExpressionData::Call(c) => {
					if let Callable::Function(f) = &c.function
						&& let Some(body) = model[*f].body()
					{
						if model[*f]
							.annotations()
							.has(model, self.ids.annotations.mzn_inline)
							|| is_macro_call(body)
						{
							log::debug!(
								"Inlining {} at {} using call by value semantics",
								PrettyPrinter::new(db, model).pretty_print_signature((*f).into()),
								expression.origin().pretty_print(db)
							);

							let old_depth = self.depth;
							self.depth += 1;

							assert!(
								self.depth < 100,
								"Exceeded maximum inlining depth, likely due to infinite recursion"
							);

							let mut restore = Vec::with_capacity(c.arguments.len());
							let items = c
								.arguments
								.iter()
								.zip(model[*f].parameters().iter())
								.filter_map(|(arg, param)| {
									let origin = arg.origin();
									let folded = self.fold_expression(db, model, arg);
									let param_ty = model[*param].ty();
									if matches!(&*folded, ExpressionData::Identifier(_))
										&& folded.ty() == param_ty
									{
										restore.push(self.map.insert(*param, folded));
										None
									} else {
										let mut decl = Declaration::new(
											false,
											Domain::unbounded(db, origin, param_ty),
										);
										decl.set_definition(folded);
										let idx =
											self.model.add_declaration(Item::new(decl, origin));
										let ident = Expression::new(db, &self.model, origin, idx);
										restore.push(self.map.insert(*param, ident));
										Some(LetItem::Declaration(idx))
									}
								})
								.collect::<Vec<_>>();
							let inlined = self.fold_expression(db, model, body);
							for (param, prev) in model[*f].parameters().iter().zip(restore) {
								if let Some(prev) = prev {
									let _ = self.map.insert(*param, prev);
								} else {
									let _ = self.map.remove(param);
								}
							}

							self.depth = old_depth;
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
							.has(model, self.ids.annotations.mzn_inline_call_by_name)
						{
							log::debug!(
								"Inlining {} at {} using call by name semantics",
								PrettyPrinter::new(db, model).pretty_print_signature((*f).into()),
								expression.origin().pretty_print(db)
							);

							let old_depth = self.depth;
							self.depth += 1;

							assert!(
								self.depth < 100,
								"Exceeded maximum inlining depth, likely due to infinite recursion"
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
									let _ = self.map.insert(*param, prev);
								} else {
									let _ = self.map.remove(param);
								}
							}

							self.depth = old_depth;

							return inlined;
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
pub fn inline_functions<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Inlining functions");
	let mut inliner = Inliner {
		replacement_map: ReplacementMap::default(),
		model: Model::with_capacities(&model.item_counts()),
		ids: IdentifierRegistry::lookup(db),
		map: FxHashMap::default(),
		depth: 0,
	};
	inliner.add_model(db, &model);
	Ok(inliner.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::inline_functions;
	use crate::transform::tests::check_no_stdlib;

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
