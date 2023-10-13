use std::iter;

use crate::{
	db::InternedStringData,
	diagnostics::InvalidArrayLiteral,
	hir::{db::Hir, source::Origin, *},
	syntax::{ast::AstNode, eprime},
	utils::arena::ArenaIndex,
	Error,
};

/// Collects AST expressions for owned by an item and lowers them into HIR recursively.
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

	/// Lower an AST expression into HIR
	pub fn collect_expression(&mut self, expression: eprime::Expression) -> ArenaIndex<Expression> {
		let origin = Origin::new(&expression);
		if expression.is_missing() {
			return self.alloc_expression(origin, Expression::Missing);
		}
		let collected: Expression = match expression {
			eprime::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			eprime::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			eprime::Expression::StringLiteral(s) => StringLiteral::new(s.value(), self.db).into(),
			eprime::Expression::MatrixLiteral(m) => return self.collect_matrix_literal(m),
			eprime::Expression::Call(c) => self.collect_call(c).into(),
			eprime::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			eprime::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			eprime::Expression::InfixOperator(o) => return self.collect_infix_operator(o),
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

	/// Lower Domain/Type into HIR
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

	/// Helper function that collects the expressions within the domain. Important for
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
					.map(|i| self.collect_domain(i, VarType::Par))
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

	/// Lower function calls into HIR
	pub fn collect_call(&mut self, c: eprime::Call) -> Call {
		let operator = c.function();
		let function = self.ident_exp(
			Origin::new(&c),
			// Convert Eprime calls to MiniZinc ones
			match operator.name() {
				"toInt" => "booltoint",
				"toSet" => "arraytoset",
				"and" => "forall",
				"or" => "exists",
				"gcc" => "global_cardinality",
				"allDiff" => "all_different",
				_ => operator.name(),
			},
		);
		Call {
			arguments: c.arguments().map(|a| self.collect_expression(a)).collect(),
			function,
		}
	}

	fn collect_infix_operator(&mut self, o: eprime::InfixOperator) -> ArenaIndex<Expression> {
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
				"<lex" => "lex_less",
				"<=lex" => "lex_lesseq",
				">lex" => "lex_greater",
				">=lex" => "lex_greatereq",
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

	fn collect_array_access(&mut self, aa: eprime::ArrayAccess) -> ArrayAccess {
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

	/// Collect a matrix literal into HIR
	pub fn collect_matrix_literal(&mut self, ml: eprime::MatrixLiteral) -> ArenaIndex<Expression> {
		let origin = Origin::new(&ml);
		let mut dimensions = Vec::new();
		let mut is_finding_dimensions = true;
		let mut elem_stack = vec![eprime::Expression::MatrixLiteral(ml.clone())];
		let mut index_sets = Vec::new();
		let mut array_values = Vec::new();

		// Iterate through the matrix literal in depth first manner, with first path used to find
		// dimensions and index set of the matrix before collecting the values.
		// Due to this matrix literals need to be of equal size in each dimension.
		while let Some(elem) = elem_stack.pop() {
			match elem {
				eprime::Expression::MatrixLiteral(ml) => {
					if is_finding_dimensions {
						dimensions.push(ml.members().count());
						if let Some(i) = ml.index() {
							index_sets.push(
								self.collect_domain_expressions(i, VarType::Par)
									.into_expression(),
							);
						}
					}
					let mut members = ml.members().collect::<Vec<_>>();
					members.reverse();
					elem_stack.append(&mut members);
				}
				e => {
					is_finding_dimensions = false;
					array_values.push(self.collect_expression(e))
				}
			}
		}
		let members = array_values.into_boxed_slice();

		match (dimensions.len(), index_sets.len()) {
			// Case of 1d array without index set
			(1, 0) => return self.alloc_expression(origin, ArrayLiteral { members }),
			// Case of 2d array without index set
			(2, 0) => {
				return self.alloc_expression(
					origin,
					ArrayLiteral2D {
						members,
						rows: MaybeIndexSet::NonIndexed(dimensions[0]),
						columns: MaybeIndexSet::NonIndexed(dimensions[1]),
					},
				)
			}
			// Case of nd array with possible index set
			(d, i) => {
				let (src, span) = ml.cst_node().source_span(self.db.upcast());
				if d > 6 {
					self.add_diagnostic(InvalidArrayLiteral {
						src,
						span,
						msg:
							"Support for matrix literals with >6 dimensions not currently supported"
								.to_string(),
					});
					return self.alloc_expression(origin, Expression::Missing);
				}
				if d != i && i != 0 {
					self.add_diagnostic(InvalidArrayLiteral {
						src,
						span,
						msg: "Matrix literal has mismatched dimensions and index sets".to_string(),
					});
					return self.alloc_expression(origin, Expression::Missing);
				}
				// If no index set exists guess the index set dimensions
				if i == 0 {
					self.add_diagnostic(InvalidArrayLiteral{
						src,
						span,
						msg: "Matrix literal has unknown index sets, guessing set of 1..n, prone to failure".to_string()
					});
					index_sets = dimensions
						.iter()
						.map(|n| {
							let one = self.alloc_expression(origin.clone(), IntegerLiteral(1));
							let n =
								self.alloc_expression(origin.clone(), IntegerLiteral(*n as i64));
							let function = self.ident_exp(origin.clone(), "..");
							self.alloc_expression(
								origin.clone(),
								Call {
									function,
									arguments: Box::new([one, n]),
								},
							)
						})
						.collect::<Vec<_>>();
				}
				index_sets.push(self.alloc_expression(origin.clone(), ArrayLiteral { members }));
				let function = self.ident_exp(origin.clone(), format!("array{}d", d));
				return self.alloc_expression(
					origin,
					Call {
						function,
						arguments: index_sets.into_boxed_slice(),
					},
				);
			}
		}
	}

	fn collect_prefix_operator(&mut self, o: eprime::PrefixOperator) -> ArenaIndex<Expression> {
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

	fn collect_matrix_comprehension(&mut self, m: eprime::MatrixComprehension) -> Call {
		let origin = Origin::new(&m);
		let template = self.collect_expression(m.template());
		let generators = m
			.generators()
			.zip(m.conditions().map(Some).chain(iter::repeat(None)))
			.map(|(g, c)| {
				let cond = c.and_then(|c| Some(self.collect_expression(c)));
				self.collect_generator(g, cond)
			})
			.collect();
		let matrix_comprehension = self.alloc_expression(
			origin.clone(),
			ArrayComprehension {
				template,
				indices: None,
				generators,
			},
		);

		// Either use provided index set or 0..n-1 index set
		match m.indices() {
			Some(i) => {
				let index_set = self
					.collect_domain_expressions(i, VarType::Par)
					.into_expression();
				Call {
					function: self.ident_exp(origin, "array1d"),
					arguments: Box::new([index_set, matrix_comprehension]),
				}
			}
			None => Call {
				function: self.ident_exp(origin, "indexing_0"),
				arguments: Box::new([matrix_comprehension]),
			},
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

	fn ident_exp<T: Into<InternedStringData>>(
		&mut self,
		origin: Origin,
		id: T,
	) -> ArenaIndex<Expression> {
		self.alloc_expression(origin, Identifier::new(id, self.db))
	}

	/// Add a diagnostic
	fn add_diagnostic<E: Into<Error>>(&mut self, error: E) {
		self.diagnostics.push(error.into());
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
