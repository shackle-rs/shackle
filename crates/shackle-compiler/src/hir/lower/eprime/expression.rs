use std::iter;

use crate::{
	db::InternedStringData,
	hir::{db::Hir, source::Origin, *},
	syntax::{ast::AstNode, eprime},
	utils::arena::ArenaIndex,
	Error,
};

pub struct ExpressionCollector<'a> {
	db: &'a dyn Hir,
	data: ItemData,
	source_map: ItemDataSourceMap,
	diagnostics: &'a mut Vec<Error>,
}

impl ExpressionCollector<'_> {
	/// Create a new expression collector
	pub fn new<'a>(db: &'a dyn Hir, diagnostics: &'a mut Vec<Error>) -> ExpressionCollector<'a> {
		ExpressionCollector {
			db,
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
			eprime::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			eprime::Expression::Call(c) => self.collect_call(c).into(),
			eprime::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			eprime::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			eprime::Expression::InfixOperator(o) => return self.collect_infix_operator(o),
			eprime::Expression::MatrixLiteral(m) => return self.collect_matrix_literal(m),
			eprime::Expression::PrefixOperator(o) => return self.collect_prefix_operator(o),
			eprime::Expression::PostfixOperator(o) => return self.collect_postfix_operator(o),
			eprime::Expression::Quantification(q) => self.collect_quantification(q).into(),
			eprime::Expression::MatrixComprehension(m) => {
				self.collect_matrix_comprehension(m).into()
			}
			eprime::Expression::AbsoluteOperator(a) => return self.collect_absolute_operator(a),
		};
		self.alloc_expression(origin, collected)
	}

	pub fn collect_domain(&mut self, d: eprime::Domain, var_type: VarType) -> ArenaIndex<Type> {
		let origin = Origin::new(&d);
		let domain_expr = self.collect_domain_expressions(d, var_type);
		let domain = match domain_expr {
			CollectedDomain::PrimitiveDomain(p) => Type::Primitive {
				inst: var_type,
				opt: OptType::NonOpt,
				primitive_type: p,
			},
			CollectedDomain::BoundedDomain(b) => Type::Bounded {
				inst: Some(var_type),
				opt: None,
				domain: b,
			},
			CollectedDomain::ArrayDomain(a) => a,
		};
		self.alloc_type(origin, domain)
	}

	/// Helper function that collects the expression within the domain. Important for
	/// compatibility with domain operations
	fn collect_domain_expressions(
		&mut self,
		t: eprime::Domain,
		var_type: VarType,
	) -> CollectedDomain {
		let origin = Origin::new(&t);
		CollectedDomain::BoundedDomain(match t {
			eprime::Domain::Identifier(i) => {
				self.alloc_expression(origin.clone(), Identifier::new(i.name(), self.db))
			}
			eprime::Domain::DomainOperation(d) => {
				let left = self
					.collect_domain_expressions(d.left(), var_type)
					.into_expression();
				let right = self
					.collect_domain_expressions(d.right(), var_type)
					.into_expression();
				let op = d.operator();
				let operator = if op.name() == "-" { "diff" } else { op.name() }; // Convert Eprime operators to MiniZinc ones
				let function = self.ident_exp(Origin::new(&op), operator);
				self.alloc_expression(
					origin,
					Call {
						function,
						arguments: Box::new([left, right]),
					},
				)
			}
			eprime::Domain::MatrixDomain(m) => {
				let domain_indexes = m
					.indexes()
					.map(|i| self.collect_domain(i, var_type))
					.collect::<Box<_>>();
				let dimensions = if domain_indexes.len() > 1 {
					self.alloc_type(
						origin.clone(),
						Type::Tuple {
							opt: OptType::NonOpt,
							fields: domain_indexes,
						},
					)
				} else {
					*domain_indexes.first().unwrap()
				};
				let domain_base = self.collect_domain_expressions(m.base(), var_type);
				let element = self.alloc_type(
					origin,
					match domain_base {
						CollectedDomain::PrimitiveDomain(p) => Type::Primitive {
							inst: var_type,
							opt: OptType::NonOpt,
							primitive_type: p,
						},
						CollectedDomain::BoundedDomain(b) => Type::Bounded {
							inst: Some(var_type),
							opt: None,
							domain: b,
						},
						CollectedDomain::ArrayDomain(a) => a,
					},
				);
				return CollectedDomain::ArrayDomain(Type::Array {
					opt: OptType::NonOpt,
					dimensions,
					element,
				});
			}
			eprime::Domain::BooleanDomain(_) => {
				return CollectedDomain::PrimitiveDomain(PrimitiveType::Bool)
			}
			eprime::Domain::IntegerDomain(i) => {
				let mut call_domain_members = Vec::new();
				let mut literal_domain_members = Vec::new();
				for e in i.domain() {
					match e {
						eprime::Expression::IntegerLiteral(i) => literal_domain_members.push(
							self.alloc_expression(Origin::new(&i), IntegerLiteral(i.value())),
						),
						e => {
							call_domain_members.push(self.collect_expression(e));
						}
					}
				}
				let call_domain = if call_domain_members.len() > 1 {
					let union_expr = self.ident_exp(origin.clone(), "union");
					call_domain_members.into_iter().reduce(|acc, e| {
						self.alloc_expression(
							origin.clone(),
							Call {
								function: union_expr,
								arguments: Box::new([acc, e]),
							},
						)
					})
				} else {
					call_domain_members.into_iter().next()
				};
				let literal_domain = if literal_domain_members.len() > 0 {
					Some(self.alloc_expression(
						origin.clone(),
						SetLiteral {
							members: literal_domain_members.into_boxed_slice(),
						},
					))
				} else {
					None
				};

				match (literal_domain, call_domain) {
					(Some(l), Some(d)) => {
						let union_expr = self.ident_exp(origin.clone(), "union");
						self.alloc_expression(
							origin.clone(),
							Call {
								function: union_expr,
								arguments: Box::new([l, d]),
							},
						)
					}
					(None, Some(d)) => d,
					(Some(l), None) => l,
					(None, None) => return CollectedDomain::PrimitiveDomain(PrimitiveType::Int),
				}
			}
		})
	}

	pub fn collect_call(&mut self, c: eprime::Call) -> Call {
		let operator = c.function();
		let function = self.ident_exp(
			Origin::new(&c),
			// Convert Eprime calls to MiniZinc ones
			match operator.name() {
				"toInt" => "booltoint",
				"toSet" => "arraytoset",
				_ => operator.name(),
			},
		);
		Call {
			arguments: c.arguments().map(|a| self.collect_expression(a)).collect(),
			function,
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
			// Convert Eprime operators to MiniZinc ones
			match operator.name() {
				"==" => "eq",
				"%" => "mod",
				_ => operator.name(),
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

	pub fn collect_matrix_literal(&mut self, ml: eprime::MatrixLiteral) -> ArenaIndex<Expression> {
		let origin = Origin::new(&ml);
		let members = ml
			.members()
			.map(|m| self.collect_expression(m))
			.collect::<Box<_>>();
		let matrix_literal = self.alloc_expression(origin.clone(), ArrayLiteral { members });
		if ml.index().is_none() {
			matrix_literal
		} else {
			let indices = ml.index().and_then(|i| {
				Some(
					self.collect_domain_expressions(i, VarType::Par)
						.into_expression(),
				)
			});
			let dummy_id = Identifier::new("i", self.db);
			let template = self.alloc_expression(origin.clone(), dummy_id.clone());
			let pattern = self.alloc_pattern(origin.clone(), dummy_id);
			let generators = Box::new([Generator::Iterator {
				patterns: Box::new([pattern]),
				collection: matrix_literal,
				where_clause: None,
			}]);
			self.alloc_expression(
				origin,
				ArrayComprehension {
					template,
					indices,
					generators,
				},
			)
		}
	}

	pub fn collect_prefix_operator(&mut self, o: eprime::PrefixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(
			Origin::new(&operator),
			if operator.name() == "!" {
				"not"
			} else {
				operator.name()
			},
		);
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

	fn collect_quantification(&mut self, q: eprime::Quantification) -> Call {
		let origin = Origin::new(&q);
		let comp = ArrayComprehension {
			generators: Box::new([self.collect_generator(q.generator(), None)]),
			indices: None,
			template: self.collect_expression(q.template()),
		};
		let arguments = Box::new([self.alloc_expression(origin.clone(), comp)]);
		let function = self.ident_exp(origin.clone(), q.function().name());
		Call {
			arguments,
			function,
		}
	}

	fn collect_matrix_comprehension(
		&mut self,
		m: eprime::MatrixComprehension,
	) -> ArrayComprehension {
		let template = self.collect_expression(m.template());
		let indices = m.indices().and_then(|i| {
			Some(
				self.collect_domain_expressions(i, VarType::Par)
					.into_expression(),
			)
		});
		let generators = m
			.generators()
			.zip(m.conditions().map(Some).chain(iter::repeat(None)))
			.map(|(g, c)| {
				let cond = c.and_then(|c| Some(self.collect_expression(c)));
				self.collect_generator(g, cond)
			})
			.collect();
		ArrayComprehension {
			template,
			indices,
			generators,
		}
	}

	fn collect_generator(
		&mut self,
		g: eprime::Generator,
		where_clause: Option<ArenaIndex<Expression>>,
	) -> Generator {
		let patterns = g
			.names()
			.map(|i| self.alloc_ident_pattern(Origin::new(&i), i))
			.collect();
		let collection = self
			.collect_domain_expressions(g.collection(), VarType::Par)
			.into_expression();
		Generator::Iterator {
			patterns,
			collection,
			where_clause,
		}
	}

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

	/// Helper to convert an identifier into a pattern
	pub(super) fn alloc_ident_pattern(
		&mut self,
		origin: Origin,
		i: eprime::Identifier,
	) -> ArenaIndex<Pattern> {
		let index = self
			.data
			.patterns
			.insert(Pattern::Identifier(Identifier::new(i.name(), self.db)));
		self.source_map.pattern_source.insert(index, origin);
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

/// Represents a collected domain
enum CollectedDomain {
	PrimitiveDomain(PrimitiveType),
	ArrayDomain(Type),
	BoundedDomain(ArenaIndex<Expression>),
}

impl CollectedDomain {
	fn into_expression(self) -> ArenaIndex<Expression> {
		match self {
			CollectedDomain::ArrayDomain(_) => unreachable!("Can't use array domain as expression"),
			CollectedDomain::PrimitiveDomain(p) => {
				unreachable!("Can't use domain operation on primitive type {:?}", p)
			}
			CollectedDomain::BoundedDomain(b) => b,
		}
	}
}
