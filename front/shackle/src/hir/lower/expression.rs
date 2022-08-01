use rustc_hash::FxHashMap;

use crate::{
	arena::ArenaIndex,
	error::{InvalidArrayLiteral, SyntaxError},
	hir::{
		db::HirStringData,
		source::{DesugarKind, Origin},
	},
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
		let origin = Origin::new(self.db, &expression, None);
		if expression.is_missing() {
			return self.alloc_expression(origin, Expression::Missing);
		}
		let collected: Expression = match expression {
			ast::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			ast::Expression::FloatLiteral(f) => FloatLiteral::new(f.value()).into(),
			ast::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			ast::Expression::StringLiteral(s) => StringLiteral::new(s.value(), self.db).into(),
			ast::Expression::Absent(_) => Expression::Absent,
			ast::Expression::Infinity(_) => Expression::Infinity,
			ast::Expression::Anonymous(a) => {
				// No longer support anonymous variables, instead use opt
				let (src, span) = a.cst_node().source_span(self.db.upcast());
				self.diagnostics.push(
					SyntaxError {
						src,
						span,
						msg: "Anonymous variables in expressions are not supported".to_string(),
						other: Vec::new(),
					}
					.into(),
				);
				Expression::Missing
			}
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
		let mut tiids = FxHashMap::default();
		self.collect_type_with_tiids(t, &mut tiids, false, false)
	}

	/// Lower an AST type into HIR and collect implicit type inst ID declarations
	pub fn collect_type_with_tiids(
		&mut self,
		t: ast::Type,
		tiids: &mut FxHashMap<Identifier, TypeInstIdentifierDeclaration>,
		is_array_dim: bool,
		is_fn_parameter: bool,
	) -> ArenaIndex<Type> {
		let origin = Origin::new(self.db, &t, None);
		if t.is_missing() {
			return self.alloc_type(origin, Type::Missing);
		}
		let ty = match t {
			ast::Type::ArrayType(a) => Type::Array {
				opt: OptType::NonOpt,
				dimensions: {
					let dims: Box<[_]> = a
						.dimensions()
						.map(|dim| self.collect_type_with_tiids(dim, tiids, true, is_fn_parameter))
						.collect();
					if dims.len() == 1 {
						dims[0]
					} else {
						self.alloc_type(
							origin.clone(),
							Type::Tuple {
								opt: OptType::NonOpt,
								fields: dims,
							},
						)
					}
				},
				element: self.collect_type_with_tiids(
					a.element_type(),
					tiids,
					false,
					is_fn_parameter,
				),
			},
			ast::Type::SetType(s) => Type::Set {
				inst: s.var_type(),
				opt: s.opt_type(),
				element: self.collect_type_with_tiids(
					s.element_type(),
					tiids,
					false,
					is_fn_parameter,
				),
			},
			ast::Type::TupleType(t) => Type::Tuple {
				opt: OptType::NonOpt,
				fields: t
					.fields()
					.map(|f| self.collect_type_with_tiids(f, tiids, false, is_fn_parameter))
					.collect(),
			},
			ast::Type::RecordType(r) => Type::Record {
				opt: OptType::NonOpt,
				fields: r
					.fields()
					.map(|f| {
						(
							self.collect_pattern(f.name().into()),
							self.collect_type_with_tiids(
								f.field_type(),
								tiids,
								false,
								is_fn_parameter,
							),
						)
					})
					.collect(),
			},
			ast::Type::OperationType(o) => Type::Operation {
				opt: OptType::NonOpt,
				return_type: self.collect_type_with_tiids(
					o.return_type(),
					tiids,
					false,
					is_fn_parameter,
				),
				parameter_types: o
					.parameter_types()
					.map(|p| self.collect_type_with_tiids(p, tiids, false, is_fn_parameter))
					.collect(),
			},
			ast::Type::TypeBase(b) => {
				self.collect_type_base(b, tiids, is_array_dim, is_fn_parameter)
			}
			ast::Type::AnyType(_) => Type::Any,
		};
		self.alloc_type(origin, ty)
	}

	/// Lower an AST pattern into HIR
	pub fn collect_pattern(&mut self, p: ast::Pattern) -> ArenaIndex<Pattern> {
		let origin = Origin::new(self.db, &p, None);
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
			ast::Pattern::BooleanLiteral(b) => {
				self.alloc_pattern(origin, Pattern::Boolean(BooleanLiteral(b.value())))
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
				let ident = c.identifier();
				let pattern = Pattern::Call {
					function: self.alloc_pattern(
						Origin::new(self.db, &ident, None),
						Identifier::new(ident.name(), self.db),
					),
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
								Identifier::new(f.name().name(), self.db),
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
	pub fn finish(mut self) -> (ItemData, ItemDataSourceMap) {
		self.data.shrink_to_fit();
		(self.data, self.source_map)
	}

	fn collect_type_base(
		&mut self,
		b: ast::TypeBase,
		tiids: &mut FxHashMap<Identifier, TypeInstIdentifierDeclaration>,
		is_array_dim: bool,
		is_fn_parameter: bool,
	) -> Type {
		match b.domain() {
			ast::Domain::Bounded(e) => {
				if is_array_dim && b.var_type().is_none() && b.opt_type().is_none() {
					if let ast::Expression::Anonymous(_) = e {
						if is_fn_parameter {
							return Type::AnonymousTypeInstVar {
								inst: Some(VarType::Par),
								opt: Some(OptType::NonOpt),
								pattern: self.alloc_pattern(
									Origin::new(self.db, &e, None),
									Identifier::new("_", self.db),
								),
								enumerable: true,
								varifiable: true,
								indexable: false,
							};
						} else {
							return Type::Any;
						}
					}
				}
				Type::Bounded {
					inst: b.var_type(),
					opt: b.opt_type(),
					domain: self.collect_expression(e),
				}
			}
			ast::Domain::Unbounded(u) => Type::Primitive {
				inst: b.var_type().unwrap_or(VarType::Par),
				opt: b.opt_type().unwrap_or(OptType::NonOpt),
				primitive_type: u.primitive_type(),
			},
			ast::Domain::TypeInstIdentifier(tiid) => {
				let ident = Identifier::new(tiid.name(), self.db);
				let origin = Origin::new(self.db, &tiid, None);
				let (inst, opt) = match (b.any_type(), b.var_type(), b.opt_type()) {
					(true, _, _) => (None, None), // Unrestricted
					(_, None, None) => (Some(VarType::Par), Some(OptType::NonOpt)), // No prefix means par non-opt
					(_, None, o) => (Some(VarType::Par), o), // opt prefix means par opt
					(_, i, None) => (i, Some(OptType::NonOpt)), // var prefix means var non-opt
					(_, i, o) => (i, o),          // var opt means var opt
				};
				tiids
					.entry(ident)
					.and_modify(|tiid| {
						tiid.is_varifiable =
							tiid.is_varifiable || inst == Some(VarType::Var) || is_array_dim;
						tiid.is_indexable = tiid.is_indexable || is_array_dim;
					})
					.or_insert(TypeInstIdentifierDeclaration {
						name: self.alloc_pattern(origin.clone(), ident),
						is_enum: false,
						is_varifiable: inst == Some(VarType::Var) || is_array_dim,
						is_indexable: is_array_dim,
					});
				Type::Bounded {
					inst,
					opt,
					domain: self.alloc_expression(origin.clone(), ident),
				}
			}
			ast::Domain::TypeInstEnumIdentifier(tiid) => {
				let ident = Identifier::new(tiid.name(), self.db);
				let origin = Origin::new(self.db, &tiid, None);
				tiids
					.entry(ident)
					.or_insert(TypeInstIdentifierDeclaration {
						name: self.alloc_pattern(origin.clone(), ident),
						is_enum: true,
						is_varifiable: true,
						is_indexable: false,
					})
					.is_enum = true;
				let (inst, opt) = match (b.any_type(), b.var_type(), b.opt_type()) {
					(true, _, _) => (None, None), // Unrestricted
					(_, None, None) => (Some(VarType::Par), Some(OptType::NonOpt)), // No prefix means par non-opt
					(_, None, o) => (Some(VarType::Par), o), // opt prefix means par opt
					(_, i, None) => (i, Some(OptType::NonOpt)), // var prefix means var non-opt
					(_, i, o) => (i, o),          // var opt means var opt
				};
				Type::Bounded {
					inst,
					opt,
					domain: self.alloc_expression(origin.clone(), ident),
				}
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
			self.alloc_expression(Origin::new(self.db, &expr, None), array)
		} else if indices[0].len() == 1 && indices[1..].iter().all(|is| is.is_empty()) {
			// Start indexed, so desugar into arrayNd call
			let origin = Origin::new(self.db, &expr, Some(DesugarKind::IndexedArrayLiteral));
			let function = self.ident_exp(origin.clone(), "arrayNd");
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
			let origin = Origin::new(self.db, &expr, Some(DesugarKind::IndexedArrayLiteral));
			let num_dims = indices[0].len();
			let mut dims = std::iter::repeat(Vec::new())
				.take(num_dims)
				.collect::<Vec<Vec<ArenaIndex<Expression>>>>();
			for it in indices {
				if it.len() != num_dims {
					let (src, span) = al.cst_node().source_span(self.db.upcast());
					self.diagnostics.push(
						InvalidArrayLiteral {
							src,
							span,
							msg: "Non-uniform indexed array literal".to_string(),
						}
						.into(),
					);
					return self.alloc_expression(origin, Expression::Missing);
				}
				for (idx, is) in it.iter().enumerate() {
					dims[idx].push(*is);
				}
			}
			let function = self.ident_exp(origin.clone(), "arrayNd");
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
		let origin = Origin::new(self.db, &al, Some(DesugarKind::ArrayLiteral2D));
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
						InvalidArrayLiteral {
							src,
							span,
							msg: "2D array literal has different row length to index row"
								.to_string(),
						}
						.into(),
					);
					return self.alloc_expression(origin, Expression::Missing);
				}
			} else if members.len() != col_count {
				let (src, span) = al.cst_node().source_span(self.db.upcast());
				self.diagnostics.push(
					InvalidArrayLiteral {
						src,
						span,
						msg: "Non-uniform 2D array literal row length".to_string(),
					}
					.into(),
				);
				return self.alloc_expression(origin, Expression::Missing);
			}

			if index.is_none() != row_indices.is_empty() {
				let (src, span) = al.cst_node().source_span(self.db.upcast());
				self.diagnostics.push(
					InvalidArrayLiteral {
						src,
						span,
						msg: "Mixing indexed and non-indexed rows not allowed".to_string(),
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
			let range_fn = self.ident_exp(origin.clone(), "..");
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
			let range_fn = self.ident_exp(origin.clone(), "..");
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

		let function = self.ident_exp(origin.clone(), "array2d");
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
		let indices = aa
			.indices()
			.map(|i| match i {
				ast::ArrayIndex::Expression(e) => self.collect_expression(e),
				ast::ArrayIndex::IndexSlice(s) => self.alloc_expression(
					Origin::new(self.db, &s, None),
					Expression::Slice(Identifier::new(s.operator(), self.db)),
				),
			})
			.collect::<Box<[_]>>();
		ArrayAccess {
			collection: self.collect_expression(aa.collection()),
			indices: if indices.len() == 1 {
				indices[0]
			} else {
				self.alloc_expression(
					Origin::new(self.db, &aa, None),
					TupleLiteral { fields: indices },
				)
			},
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
		let operator = o.operator();
		let function = self.ident_exp(
			Origin::new(self.db, &operator, None),
			if operator.name() == "==" {
				// Desugar == into =
				"="
			} else {
				operator.name()
			},
		);
		self.alloc_expression(
			Origin::new(self.db, &o, Some(DesugarKind::InfixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_prefix_operator(&mut self, o: ast::PrefixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(Origin::new(self.db, &operator, None), operator.name());
		self.alloc_expression(
			Origin::new(self.db, &o, Some(DesugarKind::PrefixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_postfix_operator(&mut self, o: ast::PostfixOperator) -> ArenaIndex<Expression> {
		let arguments = Box::new([self.collect_expression(o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(
			Origin::new(self.db, &operator, None),
			format!("{}o", operator.name()),
		);
		self.alloc_expression(
			Origin::new(self.db, &o, Some(DesugarKind::PostfixOperator)),
			Call {
				function,
				arguments,
			},
		)
	}

	fn collect_generator_call(&mut self, c: ast::GeneratorCall) -> ArenaIndex<Expression> {
		// Desugar into call with comprehension as argument
		let origin = Origin::new(self.db, &c, Some(DesugarKind::GeneratorCall));
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
		let origin = Origin::new(self.db, &s, Some(DesugarKind::StringInterpolation));
		let strings = s
			.contents()
			.map(|c| match c {
				ast::InterpolationItem::String(v) => {
					self.alloc_expression(origin.clone(), StringLiteral::new(v, self.db))
				}
				ast::InterpolationItem::Expression(e) => {
					let arguments = Box::new([self.collect_expression(e.clone())]);
					let function = self.ident_exp(Origin::new(self.db, &e, None), "show");
					self.alloc_expression(
						Origin::new(self.db, &e, None),
						Call {
							function,
							arguments,
						},
					)
				}
			})
			.collect();
		let arguments =
			Box::new([self.alloc_expression(origin.clone(), ArrayLiteral { members: strings })]);
		let function = self.ident_exp(origin.clone(), "concat");

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
			field: IntegerLiteral(t.field().value()),
			tuple: self.collect_expression(t.tuple()),
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

	fn ident_exp<T: Into<HirStringData>>(
		&mut self,
		origin: Origin,
		id: T,
	) -> ArenaIndex<Expression> {
		self.alloc_expression(origin, Identifier::new(id, self.db))
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
