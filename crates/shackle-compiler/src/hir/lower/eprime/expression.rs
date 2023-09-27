use crate::{
	constants::IdentifierRegistry,
	db::InternedStringData,
	hir::{db::Hir, source::Origin, *},
	syntax::{ast::AstNode, eprime},
	utils::arena::ArenaIndex,
	Error,
};

pub struct ExpressionCollector<'a> {
	db: &'a dyn Hir,
	identifiers: &'a IdentifierRegistry,
	data: ItemData,
	source_map: ItemDataSourceMap,
	diagnostics: &'a mut Vec<Error>,
}

impl ExpressionCollector<'_> {
	/// Create a new expression collector
	pub fn new<'a>(
		db: &'a dyn Hir,
		identifiers: &'a IdentifierRegistry,
		diagnostics: &'a mut Vec<Error>,
	) -> ExpressionCollector<'a> {
		ExpressionCollector {
			db,
			identifiers,
			data: ItemData::new(),
			source_map: ItemDataSourceMap::new(),
			diagnostics,
		}
	}

	pub fn collect_expression(&mut self, expression: eprime::Expression) -> ArenaIndex<Expression> {
		let origin = Origin::new(&expression);
		if expression.is_missing() {
			return self.alloc_expression(origin, Expression::Missing);
		}
		let collected: Expression = match expression {
			eprime::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			eprime::Expression::Call(c) => self.collect_call(c).into(),
			eprime::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			eprime::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			eprime::Expression::InfixOperator(o) => return self.collect_infix_operator(o),
			eprime::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			// eprime::Expression::MatrixLiteral(m) => ,
			eprime::Expression::PrefixOperator(o) => return self.collect_prefix_operator(o),
			eprime::Expression::PostfixOperator(o) => return self.collect_postfix_operator(o),
			// eprime::Expression::Quantification(q) => ,
			// eprime::Expression::MatrixComprehension(m) => ,
			eprime::Expression::AbsoluteOperator(a) => return self.collect_absolute_operator(a),
			_ => unimplemented!("Expression not implemented"),
		};
		self.alloc_expression(origin, collected)
	}

	// pub fn collect_domain(&mut self, t: eprime::Domain, var_type:VarType) -> ArenaIndex<Type> {
	// 	let mut tiids = TypeInstIdentifiers::default(); // Not Anonymous so with tiids
	// 	let origin = Origin::new(&t);
	// 	if t.is_missing() {
	// 		return self.alloc_type(origin, Type::Missing);
	// 	}
	// 	let domain = match t {
	// 		eprime::Domain::BooleanDomain(_) => Type::Primitive {
	// 			inst: var_type,
	// 			opt: OptType::NonOpt, // No Option type Support
	// 			primitive_type: PrimitiveType::Bool,
	// 		},
	// 		eprime::Domain::IntegerDomain(i) => {
	// 			// if i.domain().
	// 			// let x = i.domain();
	// 			// Type::Bounded {
	// 			// inst:Some(var_type),
	// 			// opt: Some(OptType::NonOpt),
	// 			// domain: self.collect_expr_union(i.domain()),
	// 		}},
	// 		eprime::Domain::MatrixDomain(_) => todo!(),
	// 		eprime::Domain::DomainOperation(_) => todo!(),
	// 		eprime::Domain::Identifier(_) => todo!(),
	// 	};
	// 	self.alloc_type(origin, domain)
	// }

	pub fn collect_call(&mut self, c: eprime::Call) -> Call {
		Call {
			arguments: c.arguments().map(|a| self.collect_expression(a)).collect(),
			function: self.collect_expression(c.function()),
		}
	}

	pub fn collect_infix_operator(&mut self, o: eprime::InfixOperator) -> ArenaIndex<Expression> {
		let arguments: Box<[ArenaIndex<Expression>]> = [o.left(), o.right()]
			.into_iter()
			.map(|e| self.collect_expression(e))
			.collect();
		let operator = o.operator();
		let function = self.ident_exp(
			Origin::new(&operator),
			// Convert Eprime operators not in MiniZinc to MiniZinc ones
			if operator.name() == "==" {
				"="
			} else if operator.name() == "%" {
				"mod"
			} else {
				operator.name()
			},
		);
		self.alloc_expression(
			Origin::new(&o),
			Call {
				function,
				arguments,
			},
		)
	}

	pub fn collect_array_access(&mut self, aa: eprime::ArrayAccess) -> ArrayAccess {
		let indices = aa
			.indices()
			.map(|i| match i {
				eprime::ArrayIndex::Expression(e) => self.collect_expression(e),
				eprime::ArrayIndex::IndexSlice(s) => self.alloc_expression(
					Origin::new(&s),
					Expression::Slice(Identifier::new(s.operator(), self.db)),
				),
			})
			.collect::<Box<_>>();
		ArrayAccess {
			collection: self.collect_expression(aa.collection()),
			indices: if indices.len() == 1 {
				indices[0]
			} else {
				self.alloc_expression(Origin::new(&aa), TupleLiteral { fields: indices })
			},
		}
	}

	pub fn collect_prefix_operator(&mut self, o: eprime::PrefixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(Origin::new(&operator), operator.name());
		self.alloc_expression(
			Origin::new(&o),
			Call {
				arguments,
				function,
			},
		)
	}

	fn collect_postfix_operator(&mut self, o: eprime::PostfixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(Origin::new(&operator), format!("{}o", operator.name()));
		self.alloc_expression(
			Origin::new(&o),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_absolute_operator(&mut self, o: eprime::AbsoluteOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let function = self.ident_exp(Origin::new(&o), "abs");
		self.alloc_expression(
			Origin::new(&o),
			Call {
				function,
				arguments,
			},
		)
	}

	// The Issuue with this part is first converting domain into expression is difficult (we need int(1..x) to be converted to expr)
	// The next issue is that the where clause needs to be present for both generators ([ i+j | i: int(1..3), j : int(1..3), i<j]) needs to be possible despite
	// this being two generators in MiniZinc, thus this needs to be possibly converted into a single expression maybe?
	// fn colect_matrix_comprehension(&mut self, c: eprime::MatrixComprehension) -> ArrayComprehension {
	// 	// No Idea how to Convert Where
	// }

	// fn collect_generator(&mut self, g: eprime::Generator) -> Generator {
	// 	self.collect_generator_where(g, None)
	// }

	// fn collect_generator_where(&mut self, g: eprime::Generator, where_clause: Option<ArenaIndex<Expression>>) -> Generator {
	// 	let origin = Origin::new(&g);
	// 	let patterns = g
	// 		.names()
	// 		.map(|n| self.alloc_pattern(origin, Identifier::new(n.name(), self.db)))
	// 		.collect();
	// 	Generator::Iterator {
	// 		patterns,
	// 		collection: self.collect_expression(g.collection()), // Convert Domain to Expression
	// 		where_clause
	// 	}
	// }

	/// Collect a list of expressions as the union of one expression
	/// TODO: Better Implementation must exist
	// fn fold_expression(&mut self, expressions: Children<'_, eprime:: Expression>, func:String) -> ArenaIndex<Expression> {
	// 	let origin = Origin::new(&expressions);
	// 	let reduced_expr = expressions
	// 		.reduce(|acc, e| self.alloc_expression(Origin::new(&e), Call {
	// 			function: self.ident_exp(Origin::new(&e), func),
	// 			arguments: Box::new([acc, self.collect_expression(e)]),
	// 		}));
	// 	match reduced_expr {
	// 		Some(e) => self.alloc_expression(origin, reduced_expr),
	// 		None => self.alloc_expression(origin, Expression::Missing),
	// 	}
	// }

	pub fn ident_exp<T: Into<InternedStringData>>(
		&mut self,
		origin: Origin,
		id: T,
	) -> ArenaIndex<Expression> {
		self.alloc_expression(origin, Identifier::new(id, self.db))
	}

	/// Get the collected expressions
	pub fn finish(mut self) -> (ItemData, ItemDataSourceMap) {
		self.data.shrink_to_fit();
		(self.data, self.source_map)
	}

	pub(super) fn alloc_expression<V: Into<Expression>>(
		&mut self,
		origin: Origin,
		v: V,
	) -> ArenaIndex<Expression> {
		let index = self.data.expressions.insert(v.into());
		self.source_map.expression_source.insert(index, origin);
		index
	}

	pub(super) fn alloc_type<V: Into<Type>>(&mut self, origin: Origin, v: V) -> ArenaIndex<Type> {
		let index = self.data.types.insert(v);
		self.source_map.type_source.insert(index, origin);
		index
	}

	pub(super) fn alloc_pattern<V: Into<Pattern>>(
		&mut self,
		origin: Origin,
		v: V,
	) -> ArenaIndex<Pattern> {
		let index = self.data.patterns.insert(v);
		self.source_map.pattern_source.insert(index, origin);
		index
	}
}
