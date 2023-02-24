use crate::thir::*;

/// Trait for visiting THIR nodes
///
/// By default, each `visit_foo()` method recursively traverses the THIR.
/// When overriding a method, you can call the module-level `visit_foo()` functions to invoke
/// the default traversal behaviour before/after applying your own logic to the node (to achieve
/// either post-order/pre-order DFS as required).
pub trait Visitor {
	/// Visit the model
	fn visit_model(&mut self, model: &Model) {
		visit_model(self, model);
	}

	/// Visit an item
	fn visit_item(&mut self, model: &Model, item: ItemId) {
		visit_item(self, model, item);
	}

	/// Visit an annotation item
	fn visit_annotation(&mut self, model: &Model, item: AnnotationId) {
		visit_annotation(self, model, item);
	}

	/// Visit a constraint item
	fn visit_constraint(&mut self, model: &Model, constraint: ConstraintId) {
		visit_constraint(self, model, constraint);
	}

	/// Visit a declaration item
	fn visit_declaration(&mut self, model: &Model, declaration: DeclarationId) {
		visit_declaration(self, model, declaration);
	}

	/// Visit an enumeration item
	fn visit_enumeration(&mut self, model: &Model, enumeration: EnumerationId) {
		visit_enumeration(self, model, enumeration);
	}

	/// Visit a function item
	fn visit_function(&mut self, model: &Model, function: FunctionId) {
		visit_function(self, model, function, true);
	}

	/// Visit an output item
	fn visit_output(&mut self, model: &Model, output: OutputId) {
		visit_output(self, model, output);
	}

	/// Visit the solve item
	fn visit_solve(&mut self, model: &Model) {
		visit_solve(self, model);
	}

	/// Visit an expression
	fn visit_expression(&mut self, model: &Model, expression: &Expression) {
		visit_expression(self, model, expression);
	}

	/// Visit an absent literal
	fn visit_absent(&mut self, _model: &Model, _a: &Absent) {}

	/// Visit a boolean literal
	fn visit_boolean(&mut self, _model: &Model, _b: &BooleanLiteral) {}

	/// Visit an integer literal
	fn visit_integer(&mut self, _model: &Model, _i: &IntegerLiteral) {}

	/// Visit a float literal
	fn visit_float(&mut self, _model: &Model, _f: &FloatLiteral) {}

	/// Visit a string literal
	fn visit_string(&mut self, _model: &Model, _s: &StringLiteral) {}

	/// Visit an infinity literal
	fn visit_infinity(&mut self, _model: &Model, _i: &Infinity) {}

	/// Visit an identifier
	fn visit_identifier(&mut self, _model: &Model, _identifier: &ResolvedIdentifier) {}

	/// Visit an array literal
	fn visit_array_literal(&mut self, model: &Model, al: &ArrayLiteral) {
		visit_array_literal(self, model, al);
	}

	/// Visit a set literal
	fn visit_set_literal(&mut self, model: &Model, sl: &SetLiteral) {
		visit_set_literal(self, model, sl);
	}

	/// Visit a tuple literal
	fn visit_tuple_literal(&mut self, model: &Model, tl: &TupleLiteral) {
		visit_tuple_literal(self, model, tl);
	}

	/// Visit a record literal
	fn visit_record_literal(&mut self, model: &Model, rl: &RecordLiteral) {
		visit_record_literal(self, model, rl);
	}

	/// Visit an array comprehension
	fn visit_array_comprehension(&mut self, model: &Model, c: &ArrayComprehension) {
		visit_array_comprehension(self, model, c);
	}
	/// Visit a set comprehension
	fn visit_set_comprehension(&mut self, model: &Model, c: &SetComprehension) {
		visit_set_comprehension(self, model, c);
	}

	/// Visit an array access
	fn visit_array_access(&mut self, model: &Model, aa: &ArrayAccess) {
		visit_array_access(self, model, aa);
	}

	/// Visit a tuple access
	fn visit_tuple_access(&mut self, model: &Model, ta: &TupleAccess) {
		visit_tuple_access(self, model, ta);
	}

	/// Visit a record access
	fn visit_record_access(&mut self, model: &Model, ra: &RecordAccess) {
		visit_record_access(self, model, ra);
	}

	/// Visit an if-then-else expression
	fn visit_if_then_else(&mut self, model: &Model, ite: &IfThenElse) {
		visit_if_then_else(self, model, ite);
	}

	/// Visit a case expression
	fn visit_case(&mut self, model: &Model, c: &Case) {
		visit_case(self, model, c);
	}

	/// Visit a call expression
	fn visit_call(&mut self, model: &Model, call: &Call) {
		visit_call(self, model, call);
	}

	/// Visit a let expression
	fn visit_let(&mut self, model: &Model, l: &Let) {
		visit_let(self, model, l);
	}

	/// Visit a lambda expression
	fn visit_lambda(&mut self, model: &Model, l: &Lambda) {
		visit_lambda(self, model, l);
	}

	/// Visit a comprehension generator
	fn visit_generator(&mut self, model: &Model, generator: &Generator) {
		visit_generator(self, model, generator);
	}

	/// Visit a domain
	fn visit_domain(&mut self, model: &Model, domain: &Domain) {
		visit_domain(self, model, domain);
	}

	/// Visit a case pattern
	fn visit_pattern(&mut self, model: &Model, pattern: &Pattern) {
		visit_pattern(self, model, pattern);
	}
}

/// Visit the top-level items in the model
pub fn visit_model<V: Visitor + ?Sized>(visitor: &mut V, model: &Model) {
	for item in model.top_level_items().collect::<Vec<_>>() {
		visitor.visit_item(model, item);
	}
}

/// Visit an item
pub fn visit_item<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: ItemId) {
	match node {
		ItemId::Annotation(item) => visitor.visit_annotation(model, item),
		ItemId::Constraint(item) => visitor.visit_constraint(model, item),
		ItemId::Declaration(item) => visitor.visit_declaration(model, item),
		ItemId::Enumeration(item) => visitor.visit_enumeration(model, item),
		ItemId::Function(item) => visitor.visit_function(model, item),
		ItemId::Output(item) => visitor.visit_output(model, item),
		ItemId::Solve => visitor.visit_solve(model),
	}
}

/// Visit the children of an annotation item
pub fn visit_annotation<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: AnnotationId) {
	let annotation = &model[node];
	if let Some(params) = &annotation.parameters {
		for param in params.iter() {
			visitor.visit_item(model, (*param).into());
		}
	}
}

/// Visit the children of a constraint item
pub fn visit_constraint<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: ConstraintId) {
	let constraint = &model[node];
	for ann in constraint.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	visitor.visit_expression(model, constraint.expression());
}

/// Visit the children of a declaration item
pub fn visit_declaration<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: DeclarationId) {
	let declaration = &model[node];
	visitor.visit_domain(model, declaration.domain());
	for ann in declaration.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	if let Some(def) = declaration.definition() {
		visitor.visit_expression(model, def);
	}
}

/// Visit the children of an enumeration item
pub fn visit_enumeration<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: EnumerationId) {
	let enumeration = &model[node];
	for ann in enumeration.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
}

/// Visit the children of a function item
pub fn visit_function<V: Visitor + ?Sized>(
	visitor: &mut V,
	model: &Model,
	node: FunctionId,
	visit_body: bool,
) {
	let function = &model[node];
	for ann in function.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	visitor.visit_domain(model, function.domain());
	for param in function.parameters() {
		visitor.visit_item(model, (*param).into());
	}
	if visit_body {
		if let Some(body) = function.body() {
			visitor.visit_expression(model, body);
		}
	}
}

/// Visit the children of an output item
pub fn visit_output<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: OutputId) {
	let output = &model[node];
	if let Some(section) = output.section() {
		visitor.visit_expression(model, section);
	}
	visitor.visit_expression(model, output.expression());
}

/// Visit the children of a solve item
pub fn visit_solve<V: Visitor + ?Sized>(visitor: &mut V, model: &Model) {
	let solve = model.solve().unwrap();
	for ann in solve.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	if let Some(objective) = solve.objective() {
		visitor.visit_item(model, objective.into());
	}
}

/// Visit the children of an array literal
pub fn visit_array_literal<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, al: &ArrayLiteral) {
	for item in al.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a set literal
pub fn visit_set_literal<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, sl: &SetLiteral) {
	for item in sl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a tuple literal
pub fn visit_tuple_literal<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, tl: &TupleLiteral) {
	for item in tl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a record literal
pub fn visit_record_literal<V: Visitor + ?Sized>(
	visitor: &mut V,
	model: &Model,
	rl: &RecordLiteral,
) {
	for (_, item) in rl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of an array comprehension
pub fn visit_array_comprehension<V: Visitor + ?Sized>(
	visitor: &mut V,
	model: &Model,
	c: &ArrayComprehension,
) {
	for generator in c.generators.iter() {
		visitor.visit_generator(model, generator);
	}
	if let Some(indices) = &c.indices {
		visitor.visit_expression(model, &indices);
	}
	visitor.visit_expression(model, &c.template);
}

/// Visit the children of a set comprehension
pub fn visit_set_comprehension<V: Visitor + ?Sized>(
	visitor: &mut V,
	model: &Model,
	c: &SetComprehension,
) {
	for generator in c.generators.iter() {
		visitor.visit_generator(model, generator);
	}
	visitor.visit_expression(model, &c.template);
}

/// Visit the children of an array access expression
pub fn visit_array_access<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, aa: &ArrayAccess) {
	visitor.visit_expression(model, &aa.collection);
	visitor.visit_expression(model, &aa.indices);
}

/// Visit the children of a tuple access expression
pub fn visit_tuple_access<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, ta: &TupleAccess) {
	visitor.visit_expression(model, &ta.tuple);
}

/// Visit the children of an record access expression
pub fn visit_record_access<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, ra: &RecordAccess) {
	visitor.visit_expression(model, &ra.record);
}

/// Visit the children of an if-then-else expression
pub fn visit_if_then_else<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, ite: &IfThenElse) {
	for branch in ite.branches.iter() {
		visitor.visit_expression(model, &branch.condition);
		visitor.visit_expression(model, &branch.result);
	}
	visitor.visit_expression(model, &ite.else_result);
}

/// Visit the children of a case expression
pub fn visit_case<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, c: &Case) {
	visitor.visit_expression(model, &c.scrutinee);
	for branch in c.branches.iter() {
		visitor.visit_pattern(model, &branch.pattern);
		visitor.visit_expression(model, &branch.result);
	}
}

/// Visit the children of a call expression
pub fn visit_call<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, c: &Call) {
	visitor.visit_expression(model, &c.function);
	for arg in c.arguments.iter() {
		visitor.visit_expression(model, arg);
	}
}

/// Visit the children of a let expression
pub fn visit_let<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, l: &Let) {
	for item in l.items.iter() {
		match item {
			LetItem::Constraint(c) => visitor.visit_item(model, (*c).into()),
			LetItem::Declaration(d) => visitor.visit_item(model, (*d).into()),
		}
	}
	visitor.visit_expression(model, &l.in_expression);
}

/// Visit the children of a lambda expression
pub fn visit_lambda<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, l: &Lambda) {
	visitor.visit_domain(model, &l.domain);
	for p in l.parameters.iter() {
		visitor.visit_item(model, (*p).into())
	}
	visitor.visit_expression(model, &l.body);
}

/// Visit the children of an expression.
///
/// First visits annotations and then calls the specific `visitor.visit_foo()` method for the kind of expression
pub fn visit_expression<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, node: &Expression) {
	for ann in node.annotations().iter() {
		visitor.visit_expression(model, ann);
	}

	match &**node {
		ExpressionData::Absent => visitor.visit_absent(model, &Absent),
		ExpressionData::BooleanLiteral(b) => visitor.visit_boolean(model, b),
		ExpressionData::IntegerLiteral(i) => visitor.visit_integer(model, i),
		ExpressionData::FloatLiteral(f) => visitor.visit_float(model, f),
		ExpressionData::StringLiteral(s) => visitor.visit_string(model, s),
		ExpressionData::Infinity => visitor.visit_infinity(model, &Infinity),
		ExpressionData::Identifier(i) => visitor.visit_identifier(model, i),
		ExpressionData::ArrayLiteral(al) => visitor.visit_array_literal(model, al),
		ExpressionData::SetLiteral(sl) => visitor.visit_set_literal(model, sl),
		ExpressionData::TupleLiteral(tl) => visitor.visit_tuple_literal(model, tl),
		ExpressionData::RecordLiteral(rl) => visitor.visit_record_literal(model, rl),
		ExpressionData::ArrayComprehension(c) => visitor.visit_array_comprehension(model, c),
		ExpressionData::SetComprehension(c) => visitor.visit_set_comprehension(model, c),
		ExpressionData::ArrayAccess(aa) => visitor.visit_array_access(model, aa),
		ExpressionData::TupleAccess(ta) => visitor.visit_tuple_access(model, ta),
		ExpressionData::RecordAccess(ra) => visitor.visit_record_access(model, ra),
		ExpressionData::IfThenElse(ite) => visitor.visit_if_then_else(model, ite),
		ExpressionData::Case(c) => visitor.visit_case(model, c),
		ExpressionData::Call(c) => visitor.visit_call(model, c),
		ExpressionData::Let(l) => visitor.visit_let(model, l),
		ExpressionData::Lambda(l) => visitor.visit_lambda(model, l),
	};
}

/// Visit the children of a comprehension generator
pub fn visit_generator<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, generator: &Generator) {
	match generator {
		Generator::Assignment {
			assignment,
			where_clause,
		} => {
			visitor.visit_item(model, (*assignment).into());
			if let Some(w) = where_clause {
				visitor.visit_expression(model, w);
			}
		}
		Generator::Iterator {
			declarations,
			collection,
			where_clause,
		} => {
			for d in declarations.iter() {
				visitor.visit_item(model, (*d).into());
			}
			visitor.visit_expression(model, collection);
			if let Some(w) = where_clause {
				visitor.visit_expression(model, w);
			}
		}
	}
}

/// Visit the children of a domain
pub fn visit_domain<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, domain: &Domain) {
	match &**domain {
		DomainData::Array(dims, elem) => {
			visitor.visit_domain(model, &dims);
			visitor.visit_domain(model, &elem);
		}
		DomainData::Bounded(e) => {
			visitor.visit_expression(model, &e);
		}
		DomainData::Record(items) => {
			for (_, d) in items.iter() {
				visitor.visit_domain(model, d);
			}
		}
		DomainData::Set(d) => visitor.visit_domain(model, &d),
		DomainData::Tuple(items) => {
			for d in items.iter() {
				visitor.visit_domain(model, d);
			}
		}
		_ => (),
	}
}

/// Visit the children of a pattern
pub fn visit_pattern<V: Visitor + ?Sized>(visitor: &mut V, model: &Model, pattern: &Pattern) {
	match &**pattern {
		PatternData::AnnotationConstructor { args, .. }
		| PatternData::EnumConstructor { args, .. }
		| PatternData::Tuple(args) => {
			for arg in args.iter() {
				visitor.visit_pattern(model, arg);
			}
		}
		PatternData::Record(ps) => {
			for (_, p) in ps.iter() {
				visitor.visit_pattern(model, p);
			}
		}
		PatternData::Expression(e) => visitor.visit_expression(model, &e),
		_ => (),
	}
}
