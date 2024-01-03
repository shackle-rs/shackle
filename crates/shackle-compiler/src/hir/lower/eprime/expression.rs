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
			eprime::Expression::Infinity(_) => Expression::Infinity,
			eprime::Expression::StringLiteral(s) => StringLiteral::new(s.value(), self.db).into(),
			eprime::Expression::MatrixLiteral(m) => return self.collect_matrix_literal(m),
			eprime::Expression::Call(c) => self
				.collect_operator_call(c.function().name(), c.arguments(), origin.clone())
				.into(),
			eprime::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			eprime::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			eprime::Expression::InfixOperator(o) => self
				.collect_operator_call(
					o.operator().name(),
					vec![o.left(), o.right()].into_iter(),
					origin.clone(),
				)
				.into(),
			eprime::Expression::PrefixOperator(o) => self
				.collect_operator_call(o.operator().name(), iter::once(o.operand()), origin.clone())
				.into(),
			eprime::Expression::PrefixSetConstructor(o) => self
				.collect_operator_call(o.operator().name(), iter::once(o.operand()), origin.clone())
				.into(),
			eprime::Expression::PostfixSetConstructor(o) => self
				.collect_operator_call(
					format!("{}o", o.operator().name()).as_str(),
					iter::once(o.operand()),
					origin.clone(),
				)
				.into(),
			eprime::Expression::Quantification(q) => self.collect_quantification(q).into(),
			eprime::Expression::MatrixComprehension(m) => {
				return self.collect_matrix_comprehension(m)
			}
			eprime::Expression::AbsoluteOperator(a) => self
				.collect_operator_call("abs", iter::once(a.operand()), origin.clone())
				.into(),
			eprime::Expression::SetConstructor(o) => self
				.collect_operator_call(
					o.operator().name(),
					vec![o.left(), o.right()].into_iter(),
					origin.clone(),
				)
				.into(),
		};
		self.alloc_expression(origin, collected)
	}

	/// Lower Domain/Type into HIR
	pub fn collect_domain(&mut self, d: eprime::Domain, var_type: VarType) -> ArenaIndex<Type> {
		let origin = Origin::new(&d);
		let domain_expr = self.collect_domain_expressions(d, var_type);
		let domain = match domain_expr {
			CollectedDomain::ArrayDomain(a) => a,
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
		};
		self.alloc_type(origin, domain)
	}

	/// Helper function that collects the expressions within the domain. Important for
	/// compatibility with domain operations
	pub(super) fn collect_domain_expressions(
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
					.into_expression(self, origin.clone());
				let right = self
					.collect_domain_expressions(d.right(), var_type)
					.into_expression(self, origin.clone());
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
				let mut set_constructor_domain_members = Vec::new();
				let mut domain_members = Vec::new();
				for e in i.domain() {
					match e {
						eprime::Expression::PrefixSetConstructor(_)
						| eprime::Expression::PostfixSetConstructor(_)
						| eprime::Expression::SetConstructor(_) => {
							set_constructor_domain_members.push(self.collect_expression(e.into()))
						}
						e => {
							domain_members.push(self.collect_expression(e));
						}
					}
				}
				let call_domain = if set_constructor_domain_members.len() > 1 {
					let union_expr = self.ident_exp(origin.clone(), "union");
					set_constructor_domain_members.into_iter().reduce(|acc, e| {
						self.alloc_expression(
							origin.clone(),
							Call {
								function: union_expr,
								arguments: Box::new([acc, e]),
							},
						)
					})
				} else {
					set_constructor_domain_members.into_iter().next()
				};
				let domain = if domain_members.len() > 0 {
					Some(self.alloc_expression(
						origin.clone(),
						SetLiteral {
							members: domain_members.into_boxed_slice(),
						},
					))
				} else {
					None
				};

				match (domain, call_domain) {
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
			eprime::Domain::AnyDomain(_) => return CollectedDomain::ArrayDomain(Type::Any),
		})
	}

	fn collect_operator_call(
		&mut self,
		o: &str,
		args: impl Iterator<Item = eprime::Expression>,
		origin: Origin,
	) -> Call {
		let arguments = args
			.into_iter()
			.map(|a| self.collect_expression(a))
			.collect::<Box<_>>();
		let function = self.ident_exp(
			origin.clone(),
			// Convert Eprime operators to MiniZinc ones
			match o {
				"==" => "eq",
				"%" => "mod",
				"<lex" => "lex_less",
				"<=lex" => "lex_lesseq",
				">lex" => "lex_greater",
				">=lex" => "lex_greatereq",
				"!" => "not",
				"/" => "div",
				"toInt" => "booltoint",
				"toSet" => "arraytoset",
				"and" => "forall",
				"or" => "exists",
				"allDiff" => "all_different",
				o => o,
			},
		);
		Call {
			function,
			arguments,
		}
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
									.into_expression(self, origin.clone()),
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
					return self
						.add_array_over_dims_diagnostic(eprime::Expression::MatrixLiteral(ml));
				}
				if d != i && i != 0 {
					self.add_diagnostic(InvalidArrayLiteral {
						src,
						span,
						msg: "Matrix literal has mismatched dimensions and index sets".to_string(),
					});
					return self.alloc_expression(origin, Expression::Missing);
				}
				// If no index set exists use index set sized at dimensions
				if i == 0 {
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

	fn collect_quantification(&mut self, q: eprime::Quantification) -> Call {
		let origin = Origin::new(&q);
		let comp = ArrayComprehension {
			generators: Box::new([self.collect_generator(q.generator(), None)]),
			indices: None,
			template: self.collect_expression(q.template()),
		};
		let arguments = Box::new([self.alloc_expression(origin.clone(), comp)]);
		let function = self.ident_exp(
			origin.clone(),
			match q.function().name() {
				"forAll" => "forall",
				q => q,
			},
		);
		Call {
			arguments,
			function,
		}
	}

	fn collect_matrix_comprehension(
		&mut self,
		m: eprime::MatrixComprehension,
	) -> ArenaIndex<Expression> {
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

		match m.indices() {
			Some(i) => {
				let index_set = self
					.collect_domain_expressions(i, VarType::Par)
					.into_expression(self, origin.clone());
				let function = self.ident_exp(origin.clone(), "array1d");
				self.alloc_expression(
					origin,
					Call {
						function,
						arguments: Box::new([index_set, matrix_comprehension]),
					},
				)
			}
			None => matrix_comprehension,
		}
	}

	fn collect_generator(
		&mut self,
		g: eprime::Generator,
		where_clause: Option<ArenaIndex<Expression>>,
	) -> Generator {
		let origin = Origin::new(&g);
		let patterns = g
			.names()
			.map(|i| self.alloc_ident_pattern(origin.clone(), i))
			.collect();
		let collection = self
			.collect_domain_expressions(g.collection(), VarType::Par)
			.into_expression(self, origin);
		Generator::Iterator {
			patterns,
			collection,
			where_clause,
		}
	}

	/// Helper to create an identifier expression
	pub fn ident_exp<T: Into<InternedStringData>>(
		&mut self,
		origin: Origin,
		id: T,
	) -> ArenaIndex<Expression> {
		self.alloc_expression(origin, Identifier::new(id, self.db))
	}

	/// Add a diagnostic
	pub fn add_diagnostic<E: Into<Error>>(&mut self, error: E) {
		self.diagnostics.push(error.into());
	}

	/// Add diagnostic for array literals with >6 dimensions
	pub fn add_array_over_dims_diagnostic<N: AstNode>(&mut self, n: N) -> ArenaIndex<Expression> {
		let (src, span) = n.cst_node().source_span(self.db.upcast());
		self.add_diagnostic(InvalidArrayLiteral {
			src,
			span,
			msg: "Support for matrix literals with >6 dimensions not currently supported"
				.to_string(),
		});
		self.alloc_expression(Origin::new(&n), Expression::Missing)
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

/// Represents a collected domain in the expression collector
/// Preserves relevant information depending on the type of domain
pub(super) enum CollectedDomain {
	ArrayDomain(Type),
	PrimitiveDomain(PrimitiveType),
	BoundedDomain(ArenaIndex<Expression>),
}

impl CollectedDomain {
	/// Convert a collected domain into a usable expression
	pub(super) fn into_expression(
		self,
		ctx: &mut ExpressionCollector,
		origin: Origin,
	) -> ArenaIndex<Expression> {
		match self {
			// This is inline with the specification which restricts domain expressions to be int and bool
			// Additionally this can't be represented in MiniZinc
			CollectedDomain::ArrayDomain(_) => unreachable!("Can't use array domain as expression"),
			CollectedDomain::PrimitiveDomain(p) => {
				// Convert into a primitive range between domains min and max
				let (l, r): (Expression, Expression) = match p {
					PrimitiveType::Bool => {
						(BooleanLiteral(false).into(), BooleanLiteral(true).into())
					}
					PrimitiveType::Int => {
						let inf = ctx.alloc_expression(origin.clone(), Expression::Infinity);
						(
							Call {
								function: ctx.ident_exp(origin.clone(), "-"),
								arguments: Box::new([inf]),
							}
							.into(),
							Expression::Infinity.into(),
						)
					}
					PrimitiveType::Float | PrimitiveType::String | PrimitiveType::Ann => {
						unreachable!("These primatives aren't implemented in EPrime")
					}
				};
				let l = ctx.alloc_expression(origin.clone(), l);
				let r = ctx.alloc_expression(origin.clone(), r);
				let function = ctx.ident_exp(origin.clone(), "..");
				ctx.alloc_expression(
					origin,
					Call {
						function,
						arguments: Box::new([l, r]),
					},
				)
			}
			CollectedDomain::BoundedDomain(b) => b,
		}
	}
}
