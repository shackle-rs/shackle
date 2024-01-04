use crate::{thir::*, utils::maybe_grow_stack};

/// Trait for visiting THIR nodes
///
/// By default, each `visit_foo()` method recursively traverses the THIR.
/// When overriding a method, you can call the module-level `visit_foo()` functions to invoke
/// the default traversal behaviour before/after applying your own logic to the node (to achieve
/// either post-order/pre-order DFS as required).
pub trait Visitor<'a, T: Marker = ()> {
	/// Visit the model
	fn visit_model(&mut self, model: &'a Model<T>) {
		visit_model(self, model);
	}

	/// Visit an item
	fn visit_item(&mut self, model: &'a Model<T>, item: ItemId<T>) {
		visit_item(self, model, item);
	}

	/// Visit an annotation item
	fn visit_annotation(&mut self, model: &'a Model<T>, item: AnnotationId<T>) {
		visit_annotation(self, model, item);
	}

	/// Visit a constraint item
	fn visit_constraint(&mut self, model: &'a Model<T>, constraint: ConstraintId<T>) {
		visit_constraint(self, model, constraint);
	}

	/// Visit a declaration item
	fn visit_declaration(&mut self, model: &'a Model<T>, declaration: DeclarationId<T>) {
		visit_declaration(self, model, declaration);
	}

	/// Visit an enumeration item
	fn visit_enumeration(&mut self, model: &'a Model<T>, enumeration: EnumerationId<T>) {
		visit_enumeration(self, model, enumeration);
	}

	/// Visit a function item
	fn visit_function(&mut self, model: &'a Model<T>, function: FunctionId<T>) {
		visit_function(self, model, function, true);
	}

	/// Visit an output item
	fn visit_output(&mut self, model: &'a Model<T>, output: OutputId<T>) {
		visit_output(self, model, output);
	}

	/// Visit the solve item
	fn visit_solve(&mut self, model: &'a Model<T>) {
		visit_solve(self, model);
	}

	/// Visit an expression
	///
	/// When overriding this, it is generally a good idea to wrap the body in `utils::maybe_grow_stack`
	/// to prevent stack overflows on highly nested models
	fn visit_expression(&mut self, model: &'a Model<T>, expression: &'a Expression<T>) {
		maybe_grow_stack(|| {
			visit_expression(self, model, expression);
		});
	}

	/// Visit an absent literal
	fn visit_absent(&mut self, _model: &'a Model<T>, _a: &'a Absent) {}

	/// Visit a boolean literal
	fn visit_boolean(&mut self, _model: &'a Model<T>, _b: &'a BooleanLiteral) {}

	/// Visit an integer literal
	fn visit_integer(&mut self, _model: &'a Model<T>, _i: &'a IntegerLiteral) {}

	/// Visit a float literal
	fn visit_float(&mut self, _model: &'a Model<T>, _f: &'a FloatLiteral) {}

	/// Visit a string literal
	fn visit_string(&mut self, _model: &'a Model<T>, _s: &'a StringLiteral) {}

	/// Visit an infinity literal
	fn visit_infinity(&mut self, _model: &'a Model<T>, _i: &'a Infinity) {}

	/// Visit an identifier
	fn visit_identifier(&mut self, _model: &'a Model<T>, _identifier: &'a ResolvedIdentifier<T>) {}

	/// Visit a callable
	fn visit_callable(&mut self, model: &'a Model<T>, callable: &'a Callable<T>) {
		visit_callable(self, model, callable);
	}

	/// Visit an array literal
	fn visit_array_literal(&mut self, model: &'a Model<T>, al: &'a ArrayLiteral<T>) {
		visit_array_literal(self, model, al);
	}

	/// Visit a set literal
	fn visit_set_literal(&mut self, model: &'a Model<T>, sl: &'a SetLiteral<T>) {
		visit_set_literal(self, model, sl);
	}

	/// Visit a tuple literal
	fn visit_tuple_literal(&mut self, model: &'a Model<T>, tl: &'a TupleLiteral<T>) {
		visit_tuple_literal(self, model, tl);
	}

	/// Visit a record literal
	fn visit_record_literal(&mut self, model: &'a Model<T>, rl: &'a RecordLiteral<T>) {
		visit_record_literal(self, model, rl);
	}

	/// Visit an array comprehension
	fn visit_array_comprehension(&mut self, model: &'a Model<T>, c: &'a ArrayComprehension<T>) {
		visit_array_comprehension(self, model, c);
	}
	/// Visit a set comprehension
	fn visit_set_comprehension(&mut self, model: &'a Model<T>, c: &'a SetComprehension<T>) {
		visit_set_comprehension(self, model, c);
	}

	/// Visit a tuple access
	fn visit_tuple_access(&mut self, model: &'a Model<T>, ta: &'a TupleAccess<T>) {
		visit_tuple_access(self, model, ta);
	}

	/// Visit a record access
	fn visit_record_access(&mut self, model: &'a Model<T>, ra: &'a RecordAccess<T>) {
		visit_record_access(self, model, ra);
	}

	/// Visit an if-then-else expression
	fn visit_if_then_else(&mut self, model: &'a Model<T>, ite: &'a IfThenElse<T>) {
		visit_if_then_else(self, model, ite);
	}

	/// Visit a case expression
	fn visit_case(&mut self, model: &'a Model<T>, c: &'a Case<T>) {
		visit_case(self, model, c);
	}

	/// Visit a call expression
	fn visit_call(&mut self, model: &'a Model<T>, call: &'a Call<T>) {
		visit_call(self, model, call);
	}

	/// Visit a let expression
	fn visit_let(&mut self, model: &'a Model<T>, l: &'a Let<T>) {
		visit_let(self, model, l);
	}

	/// Visit a lambda expression
	fn visit_lambda(&mut self, model: &'a Model<T>, l: &'a Lambda<T>) {
		visit_lambda(self, model, l);
	}

	/// Visit a comprehension generator
	fn visit_generator(&mut self, model: &'a Model<T>, generator: &'a Generator<T>) {
		visit_generator(self, model, generator);
	}

	/// Visit a domain
	fn visit_domain(&mut self, model: &'a Model<T>, domain: &'a Domain<T>) {
		visit_domain(self, model, domain);
	}

	/// Visit a case pattern
	fn visit_pattern(&mut self, model: &'a Model<T>, pattern: &'a Pattern<T>) {
		visit_pattern(self, model, pattern);
	}
}

/// Visit the top-level items in the model
pub fn visit_model<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
) {
	for item in model.top_level_items().collect::<Vec<_>>() {
		visitor.visit_item(model, item);
	}
}

/// Visit an item
pub fn visit_item<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: ItemId<T>,
) {
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
pub fn visit_annotation<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: AnnotationId<T>,
) {
	let annotation = &model[node];
	if let Some(params) = &annotation.parameters {
		for param in params.iter() {
			visitor.visit_item(model, (*param).into());
		}
	}
}

/// Visit the children of a constraint item
pub fn visit_constraint<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: ConstraintId<T>,
) {
	let constraint = &model[node];
	for ann in constraint.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	visitor.visit_expression(model, constraint.expression());
}

/// Visit the children of a declaration item
pub fn visit_declaration<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: DeclarationId<T>,
) {
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
pub fn visit_enumeration<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: EnumerationId<T>,
) {
	let enumeration = &model[node];
	for ann in enumeration.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
}

/// Visit the children of a function item
pub fn visit_function<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: FunctionId<T>,
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
pub fn visit_output<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: OutputId<T>,
) {
	let output = &model[node];
	if let Some(section) = output.section() {
		visitor.visit_expression(model, section);
	}
	visitor.visit_expression(model, output.expression());
}

/// Visit the children of a solve item
pub fn visit_solve<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
) {
	let solve = model.solve().unwrap();
	for ann in solve.annotations().iter() {
		visitor.visit_expression(model, ann);
	}
	if let Some(objective) = solve.objective() {
		visitor.visit_item(model, objective.into());
	}
}

/// Visit the children of an array literal
pub fn visit_array_literal<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	al: &'a ArrayLiteral<T>,
) {
	for item in al.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a set literal
pub fn visit_set_literal<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	sl: &'a SetLiteral<T>,
) {
	for item in sl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a tuple literal
pub fn visit_tuple_literal<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	tl: &'a TupleLiteral<T>,
) {
	for item in tl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of a record literal
pub fn visit_record_literal<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	rl: &'a RecordLiteral<T>,
) {
	for (_, item) in rl.iter() {
		visitor.visit_expression(model, item);
	}
}

/// Visit the children of an array comprehension
pub fn visit_array_comprehension<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	c: &'a ArrayComprehension<T>,
) {
	for generator in c.generators.iter() {
		visitor.visit_generator(model, generator);
	}
	if let Some(indices) = &c.indices {
		visitor.visit_expression(model, indices);
	}
	visitor.visit_expression(model, &c.template);
}

/// Visit the children of a set comprehension
pub fn visit_set_comprehension<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	c: &'a SetComprehension<T>,
) {
	for generator in c.generators.iter() {
		visitor.visit_generator(model, generator);
	}
	visitor.visit_expression(model, &c.template);
}

/// Visit the children of a tuple access expression
pub fn visit_tuple_access<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	ta: &'a TupleAccess<T>,
) {
	visitor.visit_expression(model, &ta.tuple);
}

/// Visit the children of an record access expression
pub fn visit_record_access<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	ra: &'a RecordAccess<T>,
) {
	visitor.visit_expression(model, &ra.record);
}

/// Visit the children of an if-then-else expression
pub fn visit_if_then_else<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	ite: &'a IfThenElse<T>,
) {
	for branch in ite.branches.iter() {
		visitor.visit_expression(model, &branch.condition);
		visitor.visit_expression(model, &branch.result);
	}
	visitor.visit_expression(model, &ite.else_result);
}

/// Visit the children of a case expression
pub fn visit_case<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	c: &'a Case<T>,
) {
	visitor.visit_expression(model, &c.scrutinee);
	for branch in c.branches.iter() {
		visitor.visit_pattern(model, &branch.pattern);
		visitor.visit_expression(model, &branch.result);
	}
}

/// Visit the children of a callable
pub fn visit_callable<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	c: &'a Callable<T>,
) {
	if let Callable::Expression(e) = c {
		visitor.visit_expression(model, e);
	}
}

/// Visit the children of a call expression
pub fn visit_call<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	c: &'a Call<T>,
) {
	visitor.visit_callable(model, &c.function);
	for arg in c.arguments.iter() {
		visitor.visit_expression(model, arg);
	}
}

/// Visit the children of a let expression
pub fn visit_let<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	l: &'a Let<T>,
) {
	for item in l.items.iter() {
		match item {
			LetItem::Constraint(c) => visitor.visit_item(model, (*c).into()),
			LetItem::Declaration(d) => visitor.visit_item(model, (*d).into()),
		}
	}
	visitor.visit_expression(model, &l.in_expression);
}

/// Visit the children of a lambda expression
pub fn visit_lambda<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	l: &'a Lambda<T>,
) {
	visitor.visit_function(model, l.0);
}

/// Visit the children of an expression.
///
/// First visits annotations and then calls the specific `visitor.visit_foo()` method for the kind of expression
pub fn visit_expression<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	node: &'a Expression<T>,
) {
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
pub fn visit_generator<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	generator: &'a Generator<T>,
) {
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
pub fn visit_domain<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	domain: &'a Domain<T>,
) {
	match &**domain {
		DomainData::Array(dims, elem) => {
			visitor.visit_domain(model, dims);
			visitor.visit_domain(model, elem);
		}
		DomainData::Bounded(e) => {
			visitor.visit_expression(model, e);
		}
		DomainData::Record(items) => {
			for (_, d) in items.iter() {
				visitor.visit_domain(model, d);
			}
		}
		DomainData::Set(d) => visitor.visit_domain(model, d),
		DomainData::Tuple(items) => {
			for d in items.iter() {
				visitor.visit_domain(model, d);
			}
		}
		_ => (),
	}
}

/// Visit the children of a pattern
pub fn visit_pattern<'a, T: Marker, V: Visitor<'a, T> + ?Sized>(
	visitor: &mut V,
	model: &'a Model<T>,
	pattern: &'a Pattern<T>,
) {
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
		PatternData::Expression(e) => visitor.visit_expression(model, e),
		_ => (),
	}
}
