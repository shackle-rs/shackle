//! Rewriting of capturing functions/lambda expressions into non-capturing ones
//!
//! - Functions which refer to global variables are rewritten to take those variables a parameter
//! - Lambda expressions which refer to global variables, or local variables in the enclosing scope
//!   are rewritten to take them as a parameter.
//! - Lambda expressions can then be made into top-level functions.
//! - functions taking lambda expressions as parameters are made to accept a top-level function identifier
//!   the captured variables.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::thir::{
	add_function, db::Thir, fold_call, fold_expression, fold_function_body, visit_callable, Call,
	Callable, Declaration, DeclarationId, Domain, Expression, ExpressionData, Folder, FunctionId,
	IntegerLiteral, Item, Marker, Model, ReplacementMap, ResolvedIdentifier, TupleAccess,
	TupleLiteral, Visitor,
};

/// Computes all globals this function (transitively) refers to
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Captures {
	visited: FxHashSet<FunctionId>,
	variables: FxHashSet<DeclarationId>,
}

impl Visitor for Captures {
	fn visit_identifier(&mut self, model: &Model, identifier: &ResolvedIdentifier) {
		if let ResolvedIdentifier::Declaration(d) = identifier {
			if model[*d].top_level() {
				self.variables.insert(*d);
			}
		}
	}

	fn visit_callable(&mut self, model: &Model, callable: &Callable) {
		if let Callable::Function(f) = callable {
			self.get_function_captures(model, *f);
			return;
		}
		visit_callable(self, model, callable);
	}
}

impl Captures {
	fn get_function_captures(&mut self, model: &Model, function: FunctionId) {
		if self.visited.contains(&function) {
			return;
		}
		self.visited.insert(function);
		if let Some(body) = model[function].body() {
			self.visit_expression(model, body);
		}
	}

	fn get(model: &Model) -> FxHashMap<FunctionId, FxHashSet<DeclarationId>> {
		let mut captures = FxHashMap::default();
		for (f, function) in model.all_functions() {
			if function.body().is_some() {
				let mut cc = Captures::default();
				cc.get_function_captures(model, f);
				if !cc.variables.is_empty() {
					captures.insert(f, cc.variables);
				}
			}
		}
		captures
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Decaptured<Dst> {
	declaration: DeclarationId<Dst>,
	parameter: DeclarationId<Dst>,
	field: FxHashMap<DeclarationId<Dst>, IntegerLiteral>,
	captures: Vec<DeclarationId<Dst>>,
}

struct Decapturer<Dst> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst>,
	captures: FxHashMap<FunctionId, FxHashSet<DeclarationId>>,
	decaptured: FxHashMap<FunctionId, Decaptured<Dst>>,
	added_declarations: FxHashMap<Vec<DeclarationId<Dst>>, DeclarationId<Dst>>,
	current: Option<FunctionId>,
}

impl<Dst: Marker> Folder<Dst> for Decapturer<Dst> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		add_function(self, db, model, f);
		self.decapture_fn(db, model, f);
		let new_f = self.fold_function_id(db, model, f);
		if let Some(d) = self.decaptured.get(&f) {
			self.model[new_f].add_parameter(d.parameter);
		}
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		if self.decaptured.contains_key(&f) {
			self.current = Some(f);
		}
		fold_function_body(self, db, model, f);
		self.current = None
	}

	// fn fold_declaration(&mut self, db: &dyn Thir, model: &Model, d: &Declaration) -> Declaration {
	// 	if let Some(def) = d.definition() {
	// 		let mut domain = self.fold_domain(db, model, d.domain());
	// 		let definition = self.fold_expression(db, model, def);

	// 		if !definition.ty().is_subtype_of(db.upcast(), domain.ty()) {
	// 			// Must have transformed a lambda
	// 			domain = Domain::unbounded(domain.origin(), definition.ty());
	// 		}

	// 		let mut declaration = Declaration::new(d.top_level(), domain);
	// 		if let Some(name) = d.name() {
	// 			declaration.set_name(name);
	// 		}
	// 		declaration.annotations_mut().extend(
	// 			d.annotations()
	// 				.iter()
	// 				.map(|ann| self.fold_expression(db, model, ann)),
	// 		);
	// 		declaration.set_definition(definition);
	// 		declaration
	// 	} else {
	// 		fold_declaration(self, db, model, d)
	// 	}
	// }

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		expression: &Expression,
	) -> Expression<Dst> {
		(|| {
			let current = self.current?;
			let declaration =
				if let ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) =
					&**expression
				{
					Some(self.fold_declaration_id(db, model, *d))
				} else {
					None
				}?;
			let decaptured = &self.decaptured[&current];
			let field = decaptured.field.get(&declaration)?;
			if decaptured.captures.len() > 1 {
				Some(Expression::new(
					db,
					&self.model,
					expression.origin(),
					TupleAccess {
						field: *field,
						tuple: Box::new(Expression::new(
							db,
							&self.model,
							expression.origin(),
							decaptured.parameter,
						)),
					},
				))
			} else {
				Some(Expression::new(
					db,
					&self.model,
					expression.origin(),
					decaptured.parameter,
				))
			}
		})()
		.unwrap_or_else(|| fold_expression(self, db, model, expression))
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model, call: &Call) -> Call<Dst> {
		if let Callable::Function(f) = &call.function {
			if let Some(child) = self.decaptured.get(f).cloned() {
				let function = self.fold_function_id(db, model, *f);
				let mut arguments = call
					.arguments
					.iter()
					.map(|arg| self.fold_expression(db, model, arg))
					.collect::<Vec<_>>();
				if let Some(c) = self.current {
					let parent = &self.decaptured[&c];
					assert!(child.captures.len() <= parent.captures.len());
					if child.captures.len() == parent.captures.len() {
						// Same captures, so pass extra argument directly
						arguments.push(Expression::new(
							db,
							&self.model,
							self.model[function].origin(),
							parent.parameter,
						));
					} else if parent.captures.len() > 1 {
						if child.captures.len() > 1 {
							// Get child captures from our captures
							arguments.push(Expression::new(
								db,
								&self.model,
								self.model[function].origin(),
								TupleLiteral(
									child
										.captures
										.iter()
										.map(|d| {
											Expression::new(
												db,
												&self.model,
												self.model[function].origin(),
												TupleAccess {
													field: parent.field[d],
													tuple: Box::new(Expression::new(
														db,
														&self.model,
														self.model[function].origin(),
														parent.parameter,
													)),
												},
											)
										})
										.collect(),
								),
							));
						} else {
							// Child has single capture, so pass it directly
							arguments.push(Expression::new(
								db,
								&self.model,
								self.model[function].origin(),
								TupleAccess {
									field: parent.field[&child.captures[0]],
									tuple: Box::new(Expression::new(
										db,
										&self.model,
										self.model[function].origin(),
										parent.parameter,
									)),
								},
							));
						}
					}
				} else {
					// Add declaration of captured variables as argument
					arguments.push(Expression::new(
						db,
						&self.model,
						self.model[function].origin(),
						child.declaration,
					));
				}
				return Call {
					function: Callable::Function(function),
					arguments,
				};
			}
		}

		fold_call(self, db, model, call)
	}
}

impl<Dst: Marker> Decapturer<Dst> {
	fn decapture_fn(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		if self.decaptured.contains_key(&f) {
			return;
		}

		if let Some(captures) = self.captures.remove(&f) {
			let mut declarations = captures
				.iter()
				.map(|d| self.fold_declaration_id(db, model, *d))
				.collect::<Vec<_>>();
			declarations.sort();

			let origin = model[f].origin();
			let domain = if declarations.len() > 1 {
				Domain::tuple(
					db,
					origin,
					declarations
						.iter()
						.map(|d| Domain::unbounded(origin, self.model[*d].domain().ty())),
				)
			} else {
				self.model[declarations[0]].domain().clone()
			};

			// Create a declaration for the captures (or reuse existing declaration if there's only one variable)
			let captured_values = if declarations.len() > 1 {
				Expression::new(
					db,
					&self.model,
					model[f].origin(),
					TupleLiteral(
						declarations
							.iter()
							.map(|d| Expression::new(db, &self.model, self.model[*d].origin(), *d))
							.collect(),
					),
				)
			} else {
				Expression::new(db, &self.model, model[f].origin(), declarations[0])
			};
			let decl_idx = if declarations.len() > 1 {
				if model[f].top_level() {
					// Top-level functions can reuse first declaration of captures
					*self
						.added_declarations
						.entry(declarations.clone())
						.or_insert_with(|| {
							let mut declaration = Declaration::new(true, domain.clone());
							declaration.set_definition(captured_values);
							self.model.add_declaration(Item::new(declaration, origin))
						})
				} else {
					// Lambda functions always need new declarations
					let mut declaration = Declaration::new(false, domain.clone());
					declaration.set_definition(captured_values);
					self.model.add_declaration(Item::new(declaration, origin))
				}
			} else {
				// Can reuse existing declaration
				declarations[0]
			};

			// Add additional parameter for captured variables
			let param = Declaration::new(false, domain);
			let param_idx = self.model.add_declaration(Item::new(param, origin));

			self.decaptured.insert(
				f,
				Decaptured {
					declaration: decl_idx,
					parameter: param_idx,
					field: declarations
						.iter()
						.enumerate()
						.map(|(i, d)| (*d, IntegerLiteral((i + 1) as i64)))
						.collect(),
					captures: declarations,
				},
			);
			self.captures.insert(f, captures);
		}
	}
}

/// Rewrite capturing functions into non-capturing functions
pub fn decapture_model(db: &dyn Thir, model: &Model) -> Model {
	let mut d = Decapturer {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		captures: Captures::get(model),
		decaptured: FxHashMap::default(),
		added_declarations: FxHashMap::default(),
		current: None,
	};
	d.add_model(db, model);
	d.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check_no_stdlib;

	use super::decapture_model;

	#[test]
	fn test_decapture() {
		check_no_stdlib(
			decapture_model,
			r#"
                var int: x;
                function var int: foo() = x;
                any: y = foo();
            "#,
			expect!([r#"
    var int: x;
    function var int: foo(var int: _DECL_2) = _DECL_2;
    var int: y = foo(x);
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_decapture_complex() {
		check_no_stdlib(
			decapture_model,
			r#"
                var int: x;
                var int: y;
                function var int: qux(var int, var int);
                function var int: bar() = qux(foo(), y);
                function var int: foo() = x;
                any: z = bar();
            "#,
			expect!([r#"
    var int: x;
    var int: y;
    function var int: qux(var int: _DECL_3, var int: _DECL_4);
    function var int: bar(tuple(var int, var int): _DECL_6) = qux(foo(_DECL_6.1), _DECL_6.2);
    tuple(var int, var int): _DECL_5 = (x, y);
    function var int: foo(var int: _DECL_7) = _DECL_7;
    var int: z = bar(_DECL_5);
    solve satisfy;
"#]),
		);
	}
}
