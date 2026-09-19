// //! Rewriting of capturing functions expressions into non-capturing ones
// //!
// //! - Functions which refer to global variables are rewritten to take those variables
// //!   as a tuple parameter
// //! - Each call to the function is changed to pass the tuple literal of captures
// //! - Captures are transitive, so if you call a function A in the body of a function B,
// //!   then function B's captures are a superset of function A's captures

// use rustc_hash::{FxHashMap, FxHashSet};
// use shackle_diagnostics::Result;
// use shackle_hir::OptType;
// use shackle_utils::maybe_grow_stack;

// use crate::{
// 	Call, Callable, Db, Declaration, DeclarationId, Domain, Expression, ExpressionData, FunctionId,
// 	IntegerLiteral, Item, Marker, Model, ResolvedIdentifier, TupleAccess, TupleLiteral,
// 	traverse::{
// 		Folder, ReplacementMap, Visitor, add_function, fold_call, fold_expression,
// 		fold_function_body, visit_callable,
// 	},
// };
// /// Computes all globals this function (transitively) refers to
// #[derive(Debug, Default, Clone, PartialEq, Eq)]
// struct Captures<'db> {
// 	visited: FxHashSet<FunctionId<'db>>,
// 	variables: FxHashSet<DeclarationId<'db>>,
// }

// impl<'db> Visitor<'_, 'db> for Captures<'db> {
// 	fn visit_identifier(&mut self, model: &Model<'db>, identifier: &ResolvedIdentifier<'db>) {
// 		if let ResolvedIdentifier::Declaration(d) = identifier
// 			&& model[*d].top_level()
// 		{
// 			let _ = self.variables.insert(*d);
// 		}
// 	}

// 	fn visit_callable(&mut self, model: &Model<'db>, callable: &Callable<'db>) {
// 		if let Callable::Function(f) = callable {
// 			self.get_function_captures(model, *f);
// 			return;
// 		}
// 		visit_callable(self, model, callable);
// 	}
// }

// impl<'db> Captures<'db> {
// 	fn get_function_captures(&mut self, model: &Model<'db>, function: FunctionId<'db>) {
// 		if self.visited.contains(&function) {
// 			return;
// 		}
// 		let _ = self.visited.insert(function);
// 		// Domain checks will get moved to the body, so have to capture them as well
// 		self.visit_domain(model, model[function].domain());
// 		for p in model[function].parameters().iter() {
// 			self.visit_domain(model, model[*p].domain());
// 		}
// 		if let Some(body) = model[function].body() {
// 			self.visit_expression(model, body);
// 		}
// 	}

// 	fn get(model: &Model<'db>) -> FxHashMap<FunctionId<'db>, FxHashSet<DeclarationId<'db>>> {
// 		let mut captures = FxHashMap::default();
// 		for (f, _) in model.all_functions() {
// 			let mut cc = Captures::default();
// 			cc.get_function_captures(model, f);
// 			if !cc.variables.is_empty() {
// 				let _ = captures.insert(f, cc.variables);
// 			}
// 		}
// 		captures
// 	}
// }

// #[derive(Clone, Debug, PartialEq, Eq)]
// struct Decaptured<'db, Dst: Marker> {
// 	declaration: DeclarationId<'db, Dst>,
// 	parameter: DeclarationId<'db, Dst>,
// 	field: FxHashMap<DeclarationId<'db, Dst>, IntegerLiteral>,
// 	captures: Vec<DeclarationId<'db, Dst>>,
// }

// struct Decapturer<'db, Dst: Marker> {
// 	model: Model<'db, Dst>,
// 	replacement_map: ReplacementMap<'db, Dst>,
// 	captures: FxHashMap<FunctionId<'db>, FxHashSet<DeclarationId<'db>>>,
// 	decaptured: FxHashMap<FunctionId<'db>, Decaptured<'db, Dst>>,
// 	added_declarations: FxHashMap<Vec<DeclarationId<'db, Dst>>, DeclarationId<'db, Dst>>,
// 	current: Option<FunctionId<'db>>,
// }

// impl<'db, Dst: Marker> Folder<'_, 'db, Dst> for Decapturer<'db, Dst> {
// 	fn model(&mut self) -> &mut Model<'db, Dst> {
// 		&mut self.model
// 	}

// 	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst> {
// 		&mut self.replacement_map
// 	}

// 	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
// 		let _ = add_function(self, db, model, f);
// 		self.decapture_fn(db, model, f);
// 		let new_f = self.fold_function_id(db, model, f);
// 		if let Some(d) = self.decaptured.get(&f) {
// 			self.model[new_f].add_parameter(d.parameter);
// 		}
// 	}

// 	fn fold_function_body(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
// 		if self.decaptured.contains_key(&f) {
// 			self.current = Some(f);
// 		}
// 		fold_function_body(self, db, model, f);
// 		self.current = None
// 	}

// 	fn fold_expression(
// 		&mut self,
// 		db: &'db dyn Db,
// 		model: &Model<'db>,
// 		expression: &Expression<'db>,
// 	) -> Expression<'db, Dst> {
// 		maybe_grow_stack(|| {
// 			(|| {
// 				let current = self.current?;
// 				let declaration =
// 					if let ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) =
// 						&**expression
// 					{
// 						Some(self.fold_declaration_id(db, model, *d))
// 					} else {
// 						None
// 					}?;
// 				let decaptured = &self.decaptured[&current];
// 				let field = decaptured.field.get(&declaration)?;
// 				if decaptured.captures.len() > 1 {
// 					Some(Expression::new(
// 						db,
// 						&self.model,
// 						expression.origin(),
// 						TupleAccess {
// 							field: *field,
// 							tuple: Box::new(Expression::new(
// 								db,
// 								&self.model,
// 								expression.origin(),
// 								decaptured.parameter,
// 							)),
// 						},
// 					))
// 				} else {
// 					Some(Expression::new(
// 						db,
// 						&self.model,
// 						expression.origin(),
// 						decaptured.parameter,
// 					))
// 				}
// 			})()
// 			.unwrap_or_else(|| fold_expression(self, db, model, expression))
// 		})
// 	}

// 	fn fold_call(
// 		&mut self,
// 		db: &'db dyn Db,
// 		model: &Model<'db>,
// 		call: &Call<'db>,
// 	) -> Call<'db, Dst> {
// 		if let Callable::Function(f) = &call.function
// 			&& let Some(child) = self.decaptured.get(f).cloned()
// 		{
// 			let function = self.fold_function_id(db, model, *f);
// 			let mut arguments = call
// 				.arguments
// 				.iter()
// 				.map(|arg| self.fold_expression(db, model, arg))
// 				.collect::<Vec<_>>();
// 			if let Some(c) = self.current {
// 				let parent = &self.decaptured[&c];
// 				assert!(child.captures.len() <= parent.captures.len());
// 				if child.captures.len() == parent.captures.len() {
// 					// Same captures, so pass extra argument directly
// 					arguments.push(Expression::new(
// 						db,
// 						&self.model,
// 						self.model[function].origin(),
// 						parent.parameter,
// 					));
// 				} else if parent.captures.len() > 1 {
// 					if child.captures.len() > 1 {
// 						// Get child captures from our captures
// 						arguments.push(Expression::new(
// 							db,
// 							&self.model,
// 							self.model[function].origin(),
// 							TupleLiteral(
// 								child
// 									.captures
// 									.iter()
// 									.map(|d| {
// 										Expression::new(
// 											db,
// 											&self.model,
// 											self.model[function].origin(),
// 											TupleAccess {
// 												field: parent.field[d],
// 												tuple: Box::new(Expression::new(
// 													db,
// 													&self.model,
// 													self.model[function].origin(),
// 													parent.parameter,
// 												)),
// 											},
// 										)
// 									})
// 									.collect(),
// 							),
// 						));
// 					} else {
// 						// Child has single capture, so pass it directly
// 						arguments.push(Expression::new(
// 							db,
// 							&self.model,
// 							self.model[function].origin(),
// 							TupleAccess {
// 								field: parent.field[&child.captures[0]],
// 								tuple: Box::new(Expression::new(
// 									db,
// 									&self.model,
// 									self.model[function].origin(),
// 									parent.parameter,
// 								)),
// 							},
// 						));
// 					}
// 				}
// 			} else {
// 				// Add declaration of captured variables as argument
// 				arguments.push(Expression::new(
// 					db,
// 					&self.model,
// 					self.model[function].origin(),
// 					child.declaration,
// 				));
// 			}
// 			return Call {
// 				function: Callable::Function(function),
// 				arguments,
// 			};
// 		}

// 		fold_call(self, db, model, call)
// 	}
// }

// impl<'db, Dst: Marker> Decapturer<'db, Dst> {
// 	fn decapture_fn(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
// 		if self.decaptured.contains_key(&f) {
// 			return;
// 		}

// 		if let Some(captures) = self.captures.remove(&f) {
// 			let mut declarations = captures
// 				.iter()
// 				.map(|d| self.fold_declaration_id(db, model, *d))
// 				.collect::<Vec<_>>();
// 			declarations.sort();

// 			let origin = model[f].origin();
// 			let domain = if declarations.len() > 1 {
// 				Domain::tuple(
// 					db,
// 					origin,
// 					OptType::NonOpt,
// 					declarations
// 						.iter()
// 						.map(|d| Domain::unbounded(db, origin, self.model[*d].ty())),
// 				)
// 			} else {
// 				self.model[declarations[0]].domain().clone()
// 			};

// 			// Create a declaration for the captures (or reuse existing declaration if there's only one variable)
// 			let captured_values = if declarations.len() > 1 {
// 				Expression::new(
// 					db,
// 					&self.model,
// 					model[f].origin(),
// 					TupleLiteral(
// 						declarations
// 							.iter()
// 							.map(|d| Expression::new(db, &self.model, self.model[*d].origin(), *d))
// 							.collect(),
// 					),
// 				)
// 			} else {
// 				Expression::new(db, &self.model, model[f].origin(), declarations[0])
// 			};
// 			let decl_idx = if declarations.len() > 1 {
// 				assert!(model[f].top_level());
// 				// Top-level functions can reuse first declaration of captures
// 				*self
// 					.added_declarations
// 					.entry(declarations.clone())
// 					.or_insert_with(|| {
// 						let mut declaration = Declaration::new(true, domain.clone());
// 						declaration.set_definition(captured_values);
// 						self.model.add_declaration(Item::new(declaration, origin))
// 					})
// 			} else {
// 				// Can reuse existing declaration
// 				declarations[0]
// 			};

// 			// Add additional parameter for captured variables
// 			let param = Declaration::new(false, domain);
// 			let param_idx = self.model.add_declaration(Item::new(param, origin));

// 			let _ = self.decaptured.insert(
// 				f,
// 				Decaptured {
// 					declaration: decl_idx,
// 					parameter: param_idx,
// 					field: declarations
// 						.iter()
// 						.enumerate()
// 						.map(|(i, d)| (*d, IntegerLiteral((i + 1) as i64)))
// 						.collect(),
// 					captures: declarations,
// 				},
// 			);
// 			let _ = self.captures.insert(f, captures);
// 		}
// 	}
// }

// /// Rewrite capturing functions into non-capturing functions
// pub fn decapture_model<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
// 	log::info!("Rewriting functions to be non-capturing");
// 	let mut d = Decapturer {
// 		model: Model::with_capacities(&model.item_counts()),
// 		replacement_map: ReplacementMap::default(),
// 		captures: Captures::get(&model),
// 		decaptured: FxHashMap::default(),
// 		added_declarations: FxHashMap::default(),
// 		current: None,
// 	};
// 	d.add_model(db, &model);
// 	Ok(d.model)
// }

// #[cfg(test)]
// mod tests {
// 	use expect_test::expect;

// 	use super::decapture_model;
// 	use crate::transform::tests::check_no_stdlib;

// 	#[test]
// 	fn test_decapture() {
// 		check_no_stdlib(
// 			decapture_model,
// 			r#"
//                 var int: x;
//                 function var int: foo() = x;
//                 any: y = foo();
//             "#,
// 			expect!([r#"
//     var int: x;
//     function var int: foo(var int: _DECL_2) = _DECL_2;
//     var int: y = foo(x);
//     solve satisfy;
// "#]),
// 		);
// 	}

// 	#[test]
// 	fn test_decapture_complex() {
// 		check_no_stdlib(
// 			decapture_model,
// 			r#"
//                 var int: x;
//                 var int: y;
//                 function var int: qux(var int: p, var int: q);
//                 function var int: bar() = qux(foo(), y);
//                 function var int: foo() = x;
//                 any: z = bar();
//             "#,
// 			expect!([r#"
//     var int: x;
//     var int: y;
//     function var int: qux(var int: p, var int: q);
//     function var int: bar(tuple(var int, var int): _DECL_6) = qux(foo((_DECL_6).1), (_DECL_6).2);
//     tuple(var int, var int): _DECL_5 = (x, y);
//     function var int: foo(var int: _DECL_7) = _DECL_7;
//     var int: z = bar(_DECL_5);
//     solve satisfy;
// "#]),
// 		);
// 	}

// 	#[test]
// 	fn test_decapture_domains() {
// 		check_no_stdlib(
// 			decapture_model,
// 			r#"
// 				set of int: A;
// 				set of int: B;
// 				set of int: C;
// 				function A: foo(B: x, C: y);
// 				any: x = foo(1, 2);
//             "#,
// 			expect!([r#"
//     set of int: A;
//     set of int: B;
//     set of int: C;
//     function A: foo(B: x, C: y, tuple(set of int, set of int, set of int): _DECL_7);
//     tuple(set of int, set of int, set of int): _DECL_6 = (A, B, C);
//     int: x = foo(1, 2, _DECL_6);
//     solve satisfy;
// "#]),
// 		);
// 	}
// }
