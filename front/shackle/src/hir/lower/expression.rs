use crate::{
	arena::ArenaIndex,
	error::SyntaxError,
	hir::source::{DesugarKind, Origin},
	syntax::ast::{self, AstNode},
	Error,
};

use crate::hir::{db::Hir, *};

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
	pub fn collect_expression(&mut self, expression: ast::Expression) -> ArenaIndex<Expression> {
		let origin = expression.clone().into();
		if expression.is_missing() {
			return self.alloc_expression(origin, Expression::Missing);
		}
		let collected: Expression = match expression {
			ast::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			ast::Expression::FloatLiteral(f) => FloatLiteral::new(f.value()).into(),
			ast::Expression::BoolLiteral(b) => BoolLiteral(b.value()).into(),
			ast::Expression::StringLiteral(s) => StringLiteral::new(s.value(), self.db).into(),
			ast::Expression::Absent(_) => Expression::Absent,
			ast::Expression::Infinity(_) => Expression::Infinity,
			ast::Expression::Anonymous(_) => Expression::Anonymous,
			ast::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			ast::Expression::TupleLiteral(t) => self.collect_tuple_literal(t).into(),
			ast::Expression::RecordLiteral(r) => self.collect_record_literal(r).into(),
			ast::Expression::SetLiteral(sl) => self.collect_set_literal(sl).into(),
			ast::Expression::ArrayLiteral(al) => return self.collect_array_literal(al),
			ast::Expression::ArrayLiteral2D(al) => return self.collect_2d_array_literal(al),
			ast::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			ast::Expression::ArrayComprehension(c) => self.collect_array_comprehension(c).into(),
			ast::Expression::SetComprehension(c) => self.collect_set_comprehension(c).into(),
			ast::Expression::IfThenElse(i) => self.collect_if_then_else(i).into(),
			ast::Expression::Call(c) => self.collect_call(c).into(),
			ast::Expression::InfixOperator(o) => return self.collect_infix_operator(o),
			ast::Expression::PrefixOperator(o) => return self.collect_prefix_operator(o),
			ast::Expression::PostfixOperator(o) => return self.collect_postfix_operator(o),
			ast::Expression::GeneratorCall(c) => return self.collect_generator_call(c),
			ast::Expression::StringInterpolation(s) => return self.collect_string_interpolation(s),
			ast::Expression::Case(c) => self.collect_case(c).into(),
			ast::Expression::Let(l) => self.collect_let(l).into(),
			ast::Expression::TupleAccess(t) => self.collect_tuple_access(t).into(),
			ast::Expression::RecordAccess(t) => self.collect_record_access(t).into(),
			ast::Expression::AnnotatedExpression(e) => return self.collect_annotated_expression(e),
		};
		self.alloc_expression(origin, collected)
	}

	/// Lower an AST type into HIR
	pub fn collect_type(&mut self, t: ast::Type) -> ArenaIndex<Type> {
		let origin = t.clone().into();
		if t.is_missing() {
			return self.alloc_type(origin, Type::Missing);
		}
		let ty = match t {
			ast::Type::ArrayType(a) => Type::Array {
				dimensions: a
					.dimensions()
					.map(|dim| self.collect_array_dimension(dim))
					.collect(),
				element: self.collect_type(a.element_type()),
			},
			ast::Type::TupleType(t) => {
				Type::Tuple(t.fields().map(|f| self.collect_type(f)).collect())
			}
			ast::Type::RecordType(r) => Type::Record(
				r.fields()
					.map(|f| {
						(
							Identifier::new(f.name().name(), self.db),
							self.collect_type(f.field_type()),
						)
					})
					.collect(),
			),
			ast::Type::OperationType(o) => Type::Operation {
				return_type: self.collect_type(o.return_type()),
				parameter_types: o.parameter_types().map(|p| self.collect_type(p)).collect(),
			},
			ast::Type::TypeBase(b) => Type::Base(self.collect_type_base(b)),
		};
		self.alloc_type(origin, ty)
	}

	/// Lower an AST pattern into HIR
	pub fn collect_pattern(&mut self, p: ast::Pattern) -> ArenaIndex<Pattern> {
		let origin = p.clone().into();
		if p.is_missing() {
			return self.alloc_pattern(origin, Pattern::Missing);
		}
		match p {
			ast::Pattern::Identifier(i) => {
				let identifier = Identifier::new(i.name(), self.db);
				self.alloc_pattern(origin, identifier)
			}
			ast::Pattern::Anonymous(_) => self.alloc_pattern(origin, Pattern::Anonymous),
			ast::Pattern::Absent(_) => self.alloc_pattern(origin, Pattern::Absent),
			ast::Pattern::BoolLiteral(b) => {
				self.alloc_pattern(origin, Pattern::Boolean(BoolLiteral(b.value())))
			}
			ast::Pattern::StringLiteral(s) => self.alloc_pattern(
				origin,
				Pattern::String(StringLiteral::new(s.value(), self.db)),
			),
			ast::Pattern::PatternNumericLiteral(n) => match n.value() {
				ast::NumericLiteral::IntegerLiteral(i) => self.alloc_pattern(
					origin,
					Pattern::Integer {
						negated: n.negated(),
						value: IntegerLiteral(i.value()),
					},
				),
				ast::NumericLiteral::FloatLiteral(f) => self.alloc_pattern(
					origin,
					Pattern::Float {
						negated: n.negated(),
						value: FloatLiteral::new(f.value()),
					},
				),
				ast::NumericLiteral::Infinity(_) => self.alloc_pattern(
					origin,
					Pattern::Infinity {
						negated: n.negated(),
					},
				),
			},
			ast::Pattern::Call(c) => {
				let pattern = Pattern::Call {
					function: self.collect_expression(c.identifier().into()),
					arguments: c.arguments().map(|a| self.collect_pattern(a)).collect(),
				};
				self.alloc_pattern(origin, pattern)
			}
			ast::Pattern::Tuple(t) => {
				let pattern = Pattern::Tuple {
					fields: t.fields().map(|f| self.collect_pattern(f)).collect(),
				};
				self.alloc_pattern(origin, pattern)
			}
			ast::Pattern::Record(r) => {
				let pattern = Pattern::Record {
					fields: r
						.fields()
						.map(|f| {
							(
								self.collect_expression(f.name().into()),
								self.collect_pattern(f.value()),
							)
						})
						.collect(),
				};
				self.alloc_pattern(origin, pattern)
			}
		}
	}

	/// Get the collected expressions
	pub fn finish(self) -> (ItemData, ItemDataSourceMap) {
		(self.data, self.source_map)
	}

	fn collect_type_base(&mut self, b: ast::TypeBase) -> TypeBase {
		match b.any_type() {
			ast::AnyType::Any => match b.domain() {
				Some(d) => TypeBase::AnyTi(TypeInstIdentifier::new(
					d.cast::<ast::TypeInstIdentifier>().unwrap().name(),
					self.db,
				)),
				None => TypeBase::Any,
			},
			ast::AnyType::NonAny => {
				let domain = match b.domain().unwrap() {
					ast::Domain::Bounded(e) => Domain::Bounded(self.collect_expression(e)),
					ast::Domain::Unbounded(u) => Domain::Unbounded(match u.primitive_type() {
						ast::PrimitiveType::Ann => PrimitiveType::Ann,
						ast::PrimitiveType::Bool => PrimitiveType::Bool,
						ast::PrimitiveType::Float => PrimitiveType::Float,
						ast::PrimitiveType::Int => PrimitiveType::Int,
						ast::PrimitiveType::String => PrimitiveType::String,
					}),
					ast::Domain::TypeInstIdentifier(tiid) => {
						Domain::TypeInstIdentifier(TypeInstIdentifier::new(tiid.name(), self.db))
					}
					ast::Domain::TypeInstEnumIdentifier(tiid) => Domain::TypeInstEnumIdentifier(
						TypeInstEnumIdentifier::new(tiid.name(), self.db),
					),
				};
				TypeBase::NonAny {
					is_var: b.var_type() == ast::VarType::Var,
					is_opt: b.opt_type() == ast::OptType::Opt,
					set_type: b.set_type() == ast::SetType::Set,
					domain,
				}
			}
		}
	}

	fn collect_array_dimension(&mut self, dim: ast::TypeBase) -> ArrayDimension {
		match (
			dim.var_type(),
			dim.opt_type(),
			dim.set_type(),
			dim.any_type(),
		) {
			(
				ast::VarType::Par,
				ast::OptType::NonOpt,
				ast::SetType::NonSet,
				ast::AnyType::NonAny,
			) => match dim.domain().unwrap() {
				ast::Domain::Unbounded(p) => match p.primitive_type() {
					ast::PrimitiveType::Int => ArrayDimension::Integer,
					_ => {
						// self.diagnostics.push()
						ArrayDimension::Missing
					}
				},
				ast::Domain::Bounded(is) => ArrayDimension::Expression(self.collect_expression(is)),
				ast::Domain::TypeInstEnumIdentifier(tiid) => {
					ArrayDimension::TypeInstEnumIdentifier(TypeInstEnumIdentifier::new(
						tiid.name(),
						self.db,
					))
				}
				ast::Domain::TypeInstIdentifier(tiid) => ArrayDimension::TypeInstIdentifier(
					TypeInstIdentifier::new(tiid.name(), self.db),
				),
			},
			_ => {
				// only plain par allowed
				ArrayDimension::Missing
			}
		}
	}

	fn collect_set_literal(&mut self, sl: ast::SetLiteral) -> SetLiteral {
		SetLiteral {
			members: sl.members().map(|e| self.collect_expression(e)).collect(),
		}
	}

	fn collect_array_literal(&mut self, al: ast::ArrayLiteral) -> ArenaIndex<Expression> {
		let expr: ast::Expression = al.clone().into();
		let indices = al
			.members()
			.map(|m| {
				let mut is = Vec::new();
				if let Some(i) = m.indices() {
					if let Some(t) = i.cast_ref::<ast::TupleLiteral>() {
						is.extend(t.members().map(|e| self.collect_expression(e)));
					} else {
						is.push(self.collect_expression(i));
					}
				}
				is
			})
			.collect::<Vec<Vec<_>>>();
		let values = al
			.members()
			.map(|m| self.collect_expression(m.value()))
			.collect::<Vec<_>>();
		let array = ArrayLiteral {
			members: values.into_boxed_slice(),
		};
		if indices.iter().all(|is| is.is_empty()) {
			// Non-indexed
			self.alloc_expression(expr.into(), array)
		} else if indices[0].len() == 1 && indices[1..].iter().all(|is| is.is_empty()) {
			// Start indexed, so desugar into arrayNd call
			let origin = Origin::new(expr, Some(DesugarKind::IndexedArrayLiteral));
			let function =
				self.alloc_expression(origin.clone(), Identifier::new("arrayNd", self.db));
			let arguments = Box::new([indices[0][0], self.alloc_expression(origin.clone(), array)]);
			self.alloc_expression(
				origin.clone(),
				Call {
					function,
					arguments,
				},
			)
		} else {
			// Fully indexed, so desugar into arrayNd call
			let origin = Origin::new(expr, Some(DesugarKind::IndexedArrayLiteral));
			let num_dims = indices[0].len();
			let mut dims = std::iter::repeat(Vec::new())
				.take(num_dims)
				.collect::<Vec<Vec<ArenaIndex<Expression>>>>();
			for it in indices {
				if it.len() != num_dims {
					let (src, span) = al.cst_node().source_span(self.db.upcast());
					self.diagnostics.push(
						SyntaxError {
							src,
							span,
							msg: "Non-uniform indexed array literal".to_string(),
							other: Vec::new(),
						}
						.into(),
					);
					return self.alloc_expression(origin, Expression::Missing);
				}
				for (idx, is) in it.iter().enumerate() {
					dims[idx].push(*is);
				}
			}
			let function =
				self.alloc_expression(origin.clone(), Identifier::new("arrayNd", self.db));
			let mut arguments = dims
				.into_iter()
				.map(|d| {
					self.alloc_expression(
						origin.clone(),
						ArrayLiteral {
							members: d.into_boxed_slice(),
						},
					)
				})
				.collect::<Vec<_>>();
			arguments.push(self.alloc_expression(origin.clone(), array));
			self.alloc_expression(
				origin.clone(),
				Call {
					function,
					arguments: arguments.into_boxed_slice(),
				},
			)
		}
	}

	fn collect_2d_array_literal(&mut self, al: ast::ArrayLiteral2D) -> ArenaIndex<Expression> {
		// Desugar into array2d call
		let origin = Origin::new(
			ast::Expression::from(al.clone()),
			Some(DesugarKind::ArrayLiteral2D),
		);
		let col_indices = al
			.column_indices()
			.map(|i| self.collect_expression(i))
			.collect::<Vec<_>>();
		let mut first = true;
		let mut col_count = 0;
		let mut row_indices = Vec::new();
		let mut row_count = 0;
		let mut values = Vec::new();
		for row in al.rows() {
			let members = row
				.members()
				.map(|m| self.collect_expression(m))
				.collect::<Vec<_>>();
			let index = row.index();
			if let Some(ref i) = index {
				row_indices.push(self.collect_expression(i.clone()));
			}

			if first {
				col_count = members.len();
				first = false;

				if !col_indices.is_empty() && col_count != col_indices.len() {
					let (src, span) = al.cst_node().source_span(self.db.upcast());
					self.diagnostics.push(
						SyntaxError {
							src,
							span,
							msg: "2D array literal has different row length to index row"
								.to_string(),
							other: Vec::new(),
						}
						.into(),
					);
					return self.alloc_expression(origin, Expression::Missing);
				}
			} else if members.len() != col_count {
				let (src, span) = al.cst_node().source_span(self.db.upcast());
				self.diagnostics.push(
					SyntaxError {
						src,
						span,
						msg: "Non-uniform 2D array literal row length".to_string(),
						other: Vec::new(),
					}
					.into(),
				);
				return self.alloc_expression(origin, Expression::Missing);
			}

			if index.is_none() != row_indices.is_empty() {
				let (src, span) = al.cst_node().source_span(self.db.upcast());
				self.diagnostics.push(
					SyntaxError {
						src,
						span,
						msg: "Mixing indexed and non-indexed rows not allowed".to_string(),
						other: Vec::new(),
					}
					.into(),
				);
				return self.alloc_expression(origin, Expression::Missing);
			}

			values.extend(members);
			row_count += 1;
		}

		let column_index_set = if col_indices.is_empty() {
			let range_args = vec![
				self.alloc_expression(origin.clone(), IntegerLiteral(1)),
				self.alloc_expression(origin.clone(), IntegerLiteral(col_count as i64)),
			];
			let range_fn = self.alloc_expression(origin.clone(), Identifier::new("..", self.db));
			self.alloc_expression(
				origin.clone(),
				Call {
					arguments: range_args.into_boxed_slice(),
					function: range_fn,
				},
			)
		} else {
			self.alloc_expression(
				origin.clone(),
				SetLiteral {
					members: col_indices.into_boxed_slice(),
				},
			)
		};

		let row_index_set = if row_indices.is_empty() {
			let range_args = vec![
				self.alloc_expression(origin.clone(), IntegerLiteral(1)),
				self.alloc_expression(origin.clone(), IntegerLiteral(row_count as i64)),
			];
			let range_fn = self.alloc_expression(origin.clone(), Identifier::new("..", self.db));
			self.alloc_expression(
				origin.clone(),
				Call {
					function: range_fn,
					arguments: range_args.into_boxed_slice(),
				},
			)
		} else {
			self.alloc_expression(
				origin.clone(),
				SetLiteral {
					members: row_indices.into_boxed_slice(),
				},
			)
		};

		let flat_array = self.alloc_expression(
			origin.clone(),
			ArrayLiteral {
				members: values.into_boxed_slice(),
			},
		);

		let function = self.alloc_expression(origin.clone(), Identifier::new("array2d", self.db));
		let arguments = Box::new([row_index_set, column_index_set, flat_array]);
		self.alloc_expression(
			origin,
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_array_access(&mut self, aa: ast::ArrayAccess) -> ArrayAccess {
		ArrayAccess {
			collection: self.collect_expression(aa.collection()),
			indices: aa
				.indices()
				.map(|i| match i {
					ast::ArrayIndex::Expression(e) => self.collect_expression(e),
					ast::ArrayIndex::IndexSlice(s) => {
						let function = self.alloc_expression(
							s.clone().into(),
							Identifier::new(s.operator(), self.db),
						);
						self.alloc_expression(
							s.clone().into(),
							Call {
								function,
								arguments: Box::new([]),
							},
						)
					}
				})
				.collect(),
		}
	}

	fn collect_array_comprehension(&mut self, c: ast::ArrayComprehension) -> ArrayComprehension {
		ArrayComprehension {
			generators: c.generators().map(|g| self.collect_generator(g)).collect(),
			indices: c.indices().map(|i| self.collect_expression(i)),
			template: self.collect_expression(c.template()),
		}
	}

	fn collect_set_comprehension(&mut self, c: ast::SetComprehension) -> SetComprehension {
		SetComprehension {
			generators: c.generators().map(|g| self.collect_generator(g)).collect(),
			template: self.collect_expression(c.template()),
		}
	}

	fn collect_generator(&mut self, g: ast::Generator) -> Generator {
		Generator {
			collection: self.collect_expression(g.collection()),
			patterns: g.patterns().map(|p| self.collect_pattern(p)).collect(),
			where_clause: g.where_clause().map(|w| self.collect_expression(w)),
		}
	}

	fn collect_if_then_else(&mut self, ite: ast::IfThenElse) -> IfThenElse {
		IfThenElse {
			branches: ite
				.branches()
				.map(|b| Branch {
					condition: self.collect_expression(b.condition),
					result: self.collect_expression(b.result),
				})
				.collect(),
			else_result: ite.else_result().map(|e| self.collect_expression(e)),
		}
	}

	fn collect_call(&mut self, c: ast::Call) -> Call {
		Call {
			arguments: c
				.arguments()
				.into_iter()
				.map(|a| self.collect_expression(a))
				.collect(),
			function: self.collect_expression(c.function()),
		}
	}

	fn collect_infix_operator(&mut self, o: ast::InfixOperator) -> ArenaIndex<Expression> {
		let arguments = [o.left(), o.right()]
			.into_iter()
			.map(|a| self.collect_expression(a))
			.collect();
		let function = self.alloc_expression(
			ast::Expression::from(o.clone()).into(),
			Identifier::new(o.operator(), self.db),
		);
		self.alloc_expression(
			Origin::new(ast::Expression::from(o), Some(DesugarKind::InfixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_prefix_operator(&mut self, o: ast::PrefixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let function = self.alloc_expression(
			ast::Expression::from(o.clone()).into(),
			Identifier::new(o.operator(), self.db),
		);
		self.alloc_expression(
			Origin::new(ast::Expression::from(o), Some(DesugarKind::PrefixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_postfix_operator(&mut self, o: ast::PostfixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let function = self.alloc_expression(
			ast::Expression::from(o.clone()).into(),
			// Add o suffix to postfix operators to avoid conflict with prefix version
			Identifier::new(format!("{}o", o.operator()), self.db),
		);
		self.alloc_expression(
			Origin::new(ast::Expression::from(o), Some(DesugarKind::PostfixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_generator_call(&mut self, c: ast::GeneratorCall) -> ArenaIndex<Expression> {
		// Desugar into call with comprehension as argument
		let origin = Origin::new(
			ast::Expression::from(c.clone()),
			Some(DesugarKind::GeneratorCall),
		);
		let comp = ArrayComprehension {
			generators: c.generators().map(|g| self.collect_generator(g)).collect(),
			indices: None,
			template: self.collect_expression(c.template()),
		};
		let arguments = Box::new([self.alloc_expression(origin.clone(), comp)]);
		let function = self.collect_expression(c.function());
		self.alloc_expression(
			origin.clone(),
			Call {
				arguments,
				function,
			},
		)
	}

	fn collect_string_interpolation(
		&mut self,
		s: ast::StringInterpolation,
	) -> ArenaIndex<Expression> {
		// Desugar into concat() of show() calls
		let origin = Origin::new(
			ast::Expression::from(s.clone()),
			Some(DesugarKind::StringInterpolation),
		);
		let arguments = s
			.contents()
			.map(|c| match c {
				ast::InterpolationItem::String(v) => {
					self.alloc_expression(origin.clone(), StringLiteral::new(v, self.db))
				}
				ast::InterpolationItem::Expression(e) => {
					let arguments = Box::new([self.collect_expression(e.clone())]);
					let function =
						self.alloc_expression(e.clone().into(), Identifier::new("show", self.db));
					self.alloc_expression(
						e.into(),
						Call {
							function,
							arguments,
						},
					)
				}
			})
			.collect();
		let function = self.alloc_expression(origin.clone(), Identifier::new("concat", self.db));

		self.alloc_expression(
			origin.clone(),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_case(&mut self, c: ast::Case) -> Case {
		let expression = self.collect_expression(c.expression());
		let cases = c
			.cases()
			.map(|i| CaseItem {
				pattern: self.collect_pattern(i.pattern()),
				value: self.collect_expression(i.value()),
			})
			.collect();
		Case { expression, cases }
	}

	fn collect_let(&mut self, l: ast::Let) -> Let {
		let items = l
			.items()
			.map(|i| self.collect_let_item(i.clone()))
			.collect();
		let in_expression = self.collect_expression(l.in_expression());
		Let {
			items,
			in_expression,
		}
	}

	fn collect_let_item(&mut self, i: ast::LetItem) -> LetItem {
		match i {
			ast::LetItem::Declaration(d) => Declaration {
				pattern: self.collect_pattern(d.pattern()),
				definition: d.definition().map(|def| self.collect_expression(def)),
				declared_type: self.collect_type(d.declared_type()),
				annotations: d
					.annotations()
					.map(|ann| self.collect_expression(ann))
					.collect(),
			}
			.into(),
			ast::LetItem::Constraint(c) => Constraint {
				expression: self.collect_expression(c.expression()),
				annotations: c
					.annotations()
					.map(|ann| self.collect_expression(ann))
					.collect(),
			}
			.into(),
		}
	}

	fn collect_tuple_literal(&mut self, t: ast::TupleLiteral) -> TupleLiteral {
		TupleLiteral {
			fields: t.members().map(|m| self.collect_expression(m)).collect(),
		}
	}

	fn collect_record_literal(&mut self, r: ast::RecordLiteral) -> RecordLiteral {
		RecordLiteral {
			fields: r
				.members()
				.map(|m| {
					(
						self.collect_pattern(m.name().into()),
						self.collect_expression(m.value()),
					)
				})
				.collect(),
		}
	}

	fn collect_tuple_access(&mut self, t: ast::TupleAccess) -> TupleAccess {
		TupleAccess {
			tuple: self.collect_expression(t.tuple()),
			field: IntegerLiteral(t.field().value()),
		}
	}

	fn collect_record_access(&mut self, r: ast::RecordAccess) -> RecordAccess {
		RecordAccess {
			record: self.collect_expression(r.record()),
			field: Identifier::new(r.field().name(), self.db),
		}
	}

	fn collect_annotated_expression(
		&mut self,
		e: ast::AnnotatedExpression,
	) -> ArenaIndex<Expression> {
		let annotations = e
			.annotations()
			.map(|ann| self.collect_expression(ann))
			.collect();
		let idx = self.collect_expression(e.expression());
		self.data.annotations.insert(idx, annotations);
		idx
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
