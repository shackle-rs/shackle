use std::fmt::Debug;

use shackle_diagnostics::{InvalidArrayLiteral, InvalidNumericLiteral, SyntaxError};
use shackle_syntax::{ast::AstNode, minizinc};
use shackle_utils::{hash::Map, maybe_grow_stack};

use crate::{
	Db, constants::IdentifierRegistry, diagnostics::Diagnostics, input::ModelFile, ir::*,
	source::SourceMap,
};

/// Collects AST expressions for owned by an item and lowers them into HIR recursively.
pub struct ExpressionCollector<'db, 'a> {
	db: &'db dyn Db,
	identifiers: &'db IdentifierRegistry<'db>,
	data: ItemData<'db>,
	source_map: SourceMap<'db>,
	pub(super) diagnostics: &'a mut Diagnostics,
	file: ModelFile,
	text: &'a str,
}

impl<'db, 'a> ExpressionCollector<'db, 'a> {
	/// Create a new expression collector
	pub fn new(
		db: &'db dyn Db,
		file: ModelFile,
		text: &'a str,
		diagnostics: &'a mut Diagnostics,
	) -> Self {
		let identifiers = IdentifierRegistry::lookup(db);
		ExpressionCollector {
			db,
			identifiers,
			data: ItemData::new(),
			source_map: SourceMap::default(),
			diagnostics,
			file,
			text,
		}
	}

	/// Lower an AST expression into HIR
	pub fn collect_expression(&mut self, expression: &minizinc::Expression) -> ExpressionId<'db> {
		maybe_grow_stack(|| self.collect_expression_inner(expression))
	}

	fn collect_expression_inner(&mut self, expression: &minizinc::Expression) -> ExpressionId<'db> {
		// log::debug!("Lowering {:?} to HIR", expression.cst_kind());
		if expression.is_missing() {
			return self.alloc_expression(expression, Expression::Missing);
		}
		let collected: Expression = match expression {
			minizinc::Expression::IntegerLiteral(i) => {
				IntegerLiteral(i.value(self.text).unwrap_or_else(|e| {
					let src = self.file.source_file(self.db);
					let span = i.span();
					self.diagnostics.add_error(InvalidNumericLiteral {
						src,
						span,
						msg: e.to_string(),
					});
					0
				}))
				.into()
			}
			minizinc::Expression::FloatLiteral(f) => {
				FloatLiteral::new(f.value(self.text).unwrap_or_else(|e| {
					let src = self.file.source_file(self.db);
					let span = f.span();
					self.diagnostics.add_error(InvalidNumericLiteral {
						src,
						span,
						msg: e.to_string(),
					});
					0.0
				}))
				.into()
			}
			minizinc::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			minizinc::Expression::StringLiteral(s) => {
				StringLiteral::new(self.db, s.value(self.text)).into()
			}
			minizinc::Expression::Absent(_) => Expression::Absent,
			minizinc::Expression::Infinity(_) => Expression::Infinity,
			minizinc::Expression::Anonymous(a) => {
				// No longer support anonymous variables, instead use opt
				let src = self.file.source_file(self.db);
				let span = a.span();
				self.diagnostics.add_error(SyntaxError {
					src,
					span,
					msg: "Anonymous variables in expressions are not supported".to_owned(),
				});
				Expression::Missing
			}
			minizinc::Expression::Identifier(i) => {
				Identifier::new(self.db, i.name(self.text)).into()
			}
			minizinc::Expression::TupleLiteral(t) => self.collect_tuple_literal(t).into(),
			minizinc::Expression::RecordLiteral(r) => self.collect_record_literal(r).into(),
			minizinc::Expression::SetLiteral(sl) => self.collect_set_literal(sl).into(),
			minizinc::Expression::ArrayLiteral(al) => return self.collect_array_literal(al),
			minizinc::Expression::ArrayLiteral2D(al) => return self.collect_2d_array_literal(al),
			minizinc::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			minizinc::Expression::ArrayComprehension(c) => {
				self.collect_array_comprehension(c).into()
			}
			minizinc::Expression::SetComprehension(c) => self.collect_set_comprehension(c).into(),
			minizinc::Expression::IfThenElse(i) => self.collect_if_then_else(i).into(),
			minizinc::Expression::Call(c) => self.collect_call(c).into(),
			minizinc::Expression::InfixOperator(o) => return self.collect_infix_operator(o),
			minizinc::Expression::PrefixOperator(o) => return self.collect_prefix_operator(o),
			minizinc::Expression::PostfixOperator(o) => return self.collect_postfix_operator(o),
			minizinc::Expression::GeneratorCall(c) => return self.collect_generator_call(c),
			minizinc::Expression::StringInterpolation(s) => {
				return self.collect_string_interpolation(s);
			}
			minizinc::Expression::Case(c) => self.collect_case(c).into(),
			minizinc::Expression::Let(l) => self.collect_let(l).into(),
			minizinc::Expression::TupleAccess(t) => self.collect_tuple_access(t).into(),
			minizinc::Expression::RecordAccess(t) => self.collect_record_access(t).into(),
			minizinc::Expression::Lambda(l) => self.collect_lambda(l).into(),
			minizinc::Expression::AnnotatedExpression(e) => {
				return self.collect_annotated_expression(e);
			}
		};
		self.alloc_expression(expression, collected)
	}

	/// Lower an AST type into HIR
	pub fn collect_type(&mut self, t: &minizinc::Type) -> TypeId<'db> {
		let mut tiids = TypeInstIdentifiers::default();
		self.collect_type_with_tiids(t, &mut tiids, false, false)
	}

	/// Lower a member of a class body into HIR
	pub fn collect_class_item(&mut self, i: &minizinc::ClassItem) -> ClassMember<'db> {
		match i {
			minizinc::ClassItem::Declaration(d) => {
				let declared_type = self.collect_type(&d.declared_type());
				Declaration {
					pattern: self.collect_pattern(&d.pattern()),
					definition: d.definition().map(|def| self.collect_expression(&def)),
					declared_type,
					annotations: d
						.annotations()
						.map(|ann| self.collect_expression(&ann))
						.collect(),
				}
				.into()
			}
			minizinc::ClassItem::Constraint(c) => Constraint {
				expression: self.collect_expression(&c.expression()),
				annotations: c
					.annotations()
					.map(|ann| self.collect_expression(&ann))
					.collect(),
			}
			.into(),
		}
	}

	/// Lower an AST type into HIR and collect implicit type inst ID declarations
	pub fn collect_type_with_tiids(
		&mut self,
		t: &minizinc::Type,
		tiids: &mut TypeInstIdentifiers<'db>,
		is_array_dim: bool,
		is_fn_parameter: bool,
	) -> TypeId<'db> {
		if t.is_missing() {
			return self.alloc_type(t, Type::Missing);
		}
		let ty = match t {
			minizinc::Type::ArrayType(a) => Type::Array {
				opt: OptType::NonOpt,
				dimension_pattern: {
					let patterns = a
						.dimensions()
						.map(|dim| dim.name().map(|n| self.collect_pattern(&n.into())))
						.collect::<Box<[_]>>();
					if patterns.len() == 1 {
						patterns[0]
					} else {
						let fields = patterns
							.into_iter()
							.map(|p| p.unwrap_or_else(|| self.alloc_pattern(a, Pattern::Anonymous)))
							.collect();
						Some(self.alloc_pattern(t, Pattern::Tuple { fields }))
					}
				},
				dimensions: {
					let dims: Box<[_]> = a
						.dimensions()
						.map(|dim| {
							self.collect_type_with_tiids(
								&dim.dim_type(),
								tiids,
								true,
								is_fn_parameter,
							)
						})
						.collect();
					if dims.len() == 1 {
						dims[0]
					} else {
						self.alloc_type(
							t,
							Type::Tuple {
								opt: OptType::NonOpt,
								fields: dims,
							},
						)
					}
				},
				element: self.collect_type_with_tiids(
					&a.element_type(),
					tiids,
					false,
					is_fn_parameter,
				),
			},
			minizinc::Type::ListType(l) => Type::Array {
				opt: OptType::NonOpt,
				dimension_pattern: None,
				dimensions: self.alloc_type(
					t,
					Type::Primitive {
						inst: VarType::Par,
						opt: OptType::NonOpt,
						primitive_type: PrimitiveType::Int,
					},
				),
				element: self.collect_type_with_tiids(
					&l.element_type(),
					tiids,
					false,
					is_fn_parameter,
				),
			},
			minizinc::Type::SetType(s) => Type::Set {
				inst: s.var_type(),
				opt: s.opt_type(),
				cardinality: s.cardinality().map(|c| self.collect_expression(&c)),
				element: self.collect_type_with_tiids(
					&s.element_type(),
					tiids,
					false,
					is_fn_parameter,
				),
			},
			minizinc::Type::TupleType(t) => Type::Tuple {
				opt: OptType::NonOpt,
				fields: t
					.fields()
					.map(|f| self.collect_type_with_tiids(&f, tiids, false, is_fn_parameter))
					.collect(),
			},
			minizinc::Type::RecordType(r) => Type::Record {
				opt: OptType::NonOpt,
				fields: r
					.fields()
					.map(|f| {
						(
							self.collect_pattern(&f.name().into()),
							self.collect_type_with_tiids(
								&f.field_type(),
								tiids,
								false,
								is_fn_parameter,
							),
						)
					})
					.collect(),
			},
			minizinc::Type::OperationType(o) => Type::Operation {
				opt: OptType::NonOpt,
				return_type: self.collect_type_with_tiids(
					&o.return_type(),
					tiids,
					false,
					is_fn_parameter,
				),
				parameter_types: o
					.parameter_types()
					.map(|p| self.collect_type_with_tiids(&p, tiids, false, is_fn_parameter))
					.collect(),
			},
			minizinc::Type::TypeBase(b) => {
				self.collect_type_base(b, tiids, is_array_dim, is_fn_parameter)
			}
			minizinc::Type::AnyType(_) => Type::Any,
		};
		self.alloc_type(t, ty)
	}

	/// Lower an AST pattern into HIR
	pub fn collect_pattern(&mut self, p: &minizinc::Pattern) -> PatternId<'db> {
		if p.is_missing() {
			return self.alloc_pattern(p, Pattern::Missing);
		}
		match &p {
			minizinc::Pattern::Identifier(i) => {
				let identifier = Identifier::new(self.db, i.name(self.text));
				self.alloc_pattern(p, identifier)
			}
			minizinc::Pattern::Anonymous(_) => self.alloc_pattern(p, Pattern::Anonymous),
			minizinc::Pattern::Absent(_) => self.alloc_pattern(p, Pattern::Absent),
			minizinc::Pattern::BooleanLiteral(b) => {
				self.alloc_pattern(p, Pattern::Boolean(BooleanLiteral(b.value())))
			}
			minizinc::Pattern::StringLiteral(s) => {
				let sl = StringLiteral::new(self.db, s.value(self.text));
				self.alloc_pattern(p, Pattern::String(sl))
			}
			minizinc::Pattern::PatternNumericLiteral(n) => match n.value() {
				minizinc::NumericLiteral::IntegerLiteral(i) => {
					let pat = Pattern::Integer {
						negated: n.negated(),
						value: IntegerLiteral(i.value(self.text).unwrap_or_else(|e| {
							let src = self.file.source_file(self.db);
							let span = i.span();
							self.diagnostics.add_error(InvalidNumericLiteral {
								src,
								span,
								msg: e.to_string(),
							});
							0
						})),
					};
					self.alloc_pattern(p, pat)
				}
				minizinc::NumericLiteral::FloatLiteral(f) => {
					let pat = Pattern::Float {
						negated: n.negated(),
						value: FloatLiteral::new(f.value(self.text).unwrap_or_else(|e| {
							let src = self.file.source_file(self.db);
							let span = f.span();
							self.diagnostics.add_error(InvalidNumericLiteral {
								src,
								span,
								msg: e.to_string(),
							});
							0.0
						})),
					};
					self.alloc_pattern(p, pat)
				}
				minizinc::NumericLiteral::Infinity(_) => self.alloc_pattern(
					p,
					Pattern::Infinity {
						negated: n.negated(),
					},
				),
			},
			minizinc::Pattern::Call(c) => {
				let ident = c.identifier();
				let hir_ident = Identifier::new(self.db, ident.name(self.text));
				let pattern = Pattern::Call {
					function: self.alloc_pattern(&ident, hir_ident),
					arguments: c.arguments().map(|a| self.collect_pattern(&a)).collect(),
				};
				self.alloc_pattern(p, pattern)
			}
			minizinc::Pattern::Tuple(t) => {
				let pattern = Pattern::Tuple {
					fields: t.fields().map(|f| self.collect_pattern(&f)).collect(),
				};
				self.alloc_pattern(p, pattern)
			}
			minizinc::Pattern::Record(r) => {
				let pattern = Pattern::Record {
					fields: r
						.fields()
						.map(|f| {
							let ident = Identifier::new(self.db, f.name().name(self.text));
							(ident, self.collect_pattern(&f.value()))
						})
						.collect(),
				};
				self.alloc_pattern(p, pattern)
			}
		}
	}

	/// Get the collected expressions
	pub fn finish<T>(mut self, item: T) -> (ItemWithData<'db, T>, SourceMap<'db>) {
		self.data.shrink_to_fit();
		(ItemWithData::new(item, self.data), self.source_map)
	}

	fn collect_type_base(
		&mut self,
		b: &minizinc::TypeBase,
		tiids: &mut TypeInstIdentifiers<'db>,
		is_array_dim: bool,
		is_fn_parameter: bool,
	) -> Type<'db> {
		match b.domain() {
			minizinc::Domain::Bounded(e) => {
				if is_array_dim
					&& b.var_type().is_none()
					&& b.opt_type().is_none()
					&& let minizinc::Expression::Anonymous(_) = e
				{
					if is_fn_parameter {
						let pattern = self.alloc_pattern(&e, Identifier::new(self.db, "_"));
						tiids.anons.push(TypeInstIdentifierDeclaration {
							name: pattern,
							anonymous: true,
							is_enum: true,
							is_varifiable: true,
							is_indexable: false,
						});
						return Type::AnonymousTypeInstVar {
							inst: Some(VarType::Par),
							opt: Some(OptType::NonOpt),
							pattern,
						};
					} else {
						return Type::Any;
					}
				}
				Type::Bounded {
					inst: b.var_type(),
					opt: b.opt_type(),
					domain: self.collect_expression(&e),
				}
			}
			minizinc::Domain::Unbounded(u) => Type::Primitive {
				inst: b.var_type().unwrap_or(VarType::Par),
				opt: b.opt_type().unwrap_or(OptType::NonOpt),
				primitive_type: u.primitive_type(),
			},
			minizinc::Domain::TypeInstIdentifier(tiid) => {
				let ident = Identifier::new(self.db, tiid.name(self.text));
				let (inst, opt) = match (b.any_type(), b.var_type(), b.opt_type()) {
					(true, _, _) => (None, None), // Unrestricted
					(_, None, None) => (Some(VarType::Par), Some(OptType::NonOpt)), // No prefix means par non-opt
					(_, None, o) => (Some(VarType::Par), o), // opt prefix means par opt
					(_, i, None) => (i, Some(OptType::NonOpt)), // var prefix means var non-opt
					(_, i, o) => (i, o),          // var opt means var opt
				};
				let _ = tiids
					.tiids
					.entry(ident)
					.and_modify(|(in_param, tiid)| {
						tiid.is_varifiable =
							tiid.is_varifiable || inst == Some(VarType::Var) || is_array_dim;
						tiid.is_indexable = tiid.is_indexable || is_array_dim;
						*in_param = *in_param || is_fn_parameter;
					})
					.or_insert((
						is_fn_parameter,
						TypeInstIdentifierDeclaration {
							name: self.alloc_pattern(&tiid, ident),
							anonymous: false,
							is_enum: false,
							is_varifiable: inst == Some(VarType::Var) || is_array_dim,
							is_indexable: is_array_dim,
						},
					));
				Type::Bounded {
					inst,
					opt,
					domain: self.alloc_expression(&tiid, ident),
				}
			}
			minizinc::Domain::TypeInstEnumIdentifier(tiid) => {
				let ident = Identifier::new(self.db, tiid.name(self.text));
				let _ = tiids
					.tiids
					.entry(ident)
					.and_modify(|(in_param, tiid)| {
						tiid.is_enum = true;
						*in_param = *in_param || is_fn_parameter;
					})
					.or_insert((
						is_fn_parameter,
						TypeInstIdentifierDeclaration {
							name: self.alloc_pattern(&tiid, ident),
							anonymous: false,
							is_enum: true,
							is_varifiable: true,
							is_indexable: false,
						},
					));
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
					domain: self.alloc_expression(&tiid, ident),
				}
			}
			minizinc::Domain::NewType(n) => Type::New {
				inst: b.var_type().unwrap_or(VarType::Par),
				opt: b.opt_type().unwrap_or(OptType::NonOpt),
				domain: self.collect_expression(&n.name().into()),
			},
		}
	}

	fn collect_set_literal(&mut self, sl: &minizinc::SetLiteral) -> SetLiteral<'db> {
		SetLiteral {
			members: sl.members().map(|e| self.collect_expression(&e)).collect(),
		}
	}

	fn collect_array_literal(&mut self, al: &minizinc::ArrayLiteral) -> ExpressionId<'db> {
		let (indices, values): (Vec<_>, Vec<_>) = al
			.members()
			.map(|m| {
				(
					m.indices().map(|i| self.collect_expression(&i)),
					self.collect_expression(&m.value()),
				)
			})
			.unzip();
		if indices.iter().all(|is| is.is_none()) {
			// Non-indexed
			self.alloc_expression(
				al,
				ArrayLiteral {
					members: values.into_boxed_slice(),
				},
			)
		} else {
			let mut start_indexed = indices[0].is_some();
			let mut fully_indexed = start_indexed;
			for is in indices[1..].iter() {
				if is.is_some() {
					start_indexed = false;
				} else {
					fully_indexed = false;
				}
				if !start_indexed && !fully_indexed {
					let src = self.file.source_file(self.db);
					let span = al.span();
					self.diagnostics.add_error(InvalidArrayLiteral {
						src,
						span,
						msg: "Indexed array literal must be fully indexed, or only have an index for the first element".to_owned(),
					});
					return self.alloc_expression(al, Expression::Missing);
				}
			}
			self.alloc_expression(
				al,
				IndexedArrayLiteral {
					indices: indices.into_iter().flatten().collect(),
					members: values.into_boxed_slice(),
				},
			)
		}
	}

	fn collect_2d_array_literal(&mut self, al: &minizinc::ArrayLiteral2D) -> ExpressionId<'db> {
		// Desugar into array2d call
		let col_indices = al
			.column_indices()
			.map(|i| self.collect_expression(&i))
			.collect::<Vec<_>>();
		let mut first = true;
		let mut col_count = 0;
		let mut row_indices = Vec::new();
		let mut row_count = 0;
		let mut values = Vec::new();
		for row in al.rows() {
			let members = row
				.members()
				.map(|m| self.collect_expression(&m))
				.collect::<Vec<_>>();
			let index = row.index();
			if let Some(ref i) = index {
				row_indices.push(self.collect_expression(&i.clone()));
			}

			if first {
				col_count = members.len();
				first = false;

				if !col_indices.is_empty() && col_count != col_indices.len() {
					let src = self.file.source_file(self.db);
					let span = al.span();
					self.diagnostics.add_error(InvalidArrayLiteral {
						src,
						span,
						msg: "2D array literal has different row length to index row".to_owned(),
					});
					return self.alloc_expression(al, Expression::Missing);
				}
			} else if members.len() != col_count {
				let src = self.file.source_file(self.db);
				let span = al.span();
				self.diagnostics.add_error(InvalidArrayLiteral {
					src,
					span,
					msg: "Non-uniform 2D array literal row length".to_owned(),
				});
				return self.alloc_expression(al, Expression::Missing);
			}

			if index.is_none() != row_indices.is_empty() {
				let src = self.file.source_file(self.db);
				let span = al.span();
				self.diagnostics.add_error(InvalidArrayLiteral {
					src,
					span,
					msg: "Mixing indexed and non-indexed rows not allowed".to_owned(),
				});
				return self.alloc_expression(al, Expression::Missing);
			}

			values.extend(members);
			row_count += 1;
		}

		self.alloc_expression(
			al,
			ArrayLiteral2D {
				rows: if row_indices.is_empty() {
					MaybeIndexSet::NonIndexed(row_count)
				} else {
					MaybeIndexSet::Indexed(row_indices.into_boxed_slice())
				},
				columns: if col_indices.is_empty() {
					MaybeIndexSet::NonIndexed(col_count)
				} else {
					MaybeIndexSet::Indexed(col_indices.into_boxed_slice())
				},
				members: values.into_boxed_slice(),
			},
		)
	}

	fn collect_array_access(&mut self, aa: &minizinc::ArrayAccess) -> ArrayAccess<'db> {
		let indices = aa
			.indices()
			.map(|i| match i {
				minizinc::ArrayIndex::Expression(e) => self.collect_expression(&e),
				minizinc::ArrayIndex::IndexSlice(s) => self.alloc_expression(
					&s,
					Expression::Slice(Identifier::new(self.db, s.operator())),
				),
			})
			.collect::<Box<[_]>>();
		ArrayAccess {
			collection: self.collect_expression(&aa.collection()),
			indices: if indices.len() == 1 {
				indices[0]
			} else {
				self.alloc_expression(aa, TupleLiteral { fields: indices })
			},
		}
	}

	fn collect_array_comprehension(
		&mut self,
		c: &minizinc::ArrayComprehension,
	) -> ArrayComprehension<'db> {
		ArrayComprehension {
			generators: c.generators().map(|g| self.collect_generator(&g)).collect(),
			indices: c.indices().map(|i| self.collect_expression(&i)),
			template: self.collect_expression(&c.template()),
		}
	}

	fn collect_set_comprehension(
		&mut self,
		c: &minizinc::SetComprehension,
	) -> SetComprehension<'db> {
		SetComprehension {
			generators: c.generators().map(|g| self.collect_generator(&g)).collect(),
			template: self.collect_expression(&c.template()),
		}
	}

	fn collect_generator(&mut self, g: &minizinc::Generator) -> Generator<'db> {
		match g {
			minizinc::Generator::IteratorGenerator(i) => Generator::Iterator {
				patterns: i.patterns().map(|p| self.collect_pattern(&p)).collect(),
				collection: self.collect_expression(&i.collection()),
				where_clause: i.where_clause().map(|w| self.collect_expression(&w)),
			},
			minizinc::Generator::AssignmentGenerator(a) => Generator::Assignment {
				pattern: self.collect_pattern(&a.pattern()),
				value: self.collect_expression(&a.value()),
				where_clause: a.where_clause().map(|w| self.collect_expression(&w)),
			},
		}
	}

	fn collect_if_then_else(&mut self, ite: &minizinc::IfThenElse) -> IfThenElse<'db> {
		IfThenElse {
			branches: ite
				.branches()
				.map(|b| Branch {
					condition: self.collect_expression(&b.condition),
					result: self.collect_expression(&b.result),
				})
				.collect(),
			else_result: ite.else_result().map(|e| self.collect_expression(&e)),
		}
	}

	fn collect_call(&mut self, c: &minizinc::Call) -> Call<'db> {
		let function = self.collect_expression(&c.function());
		let mut positional = Vec::new();
		let mut named = Vec::new();
		for arg in c.arguments() {
			if let Some(e) = arg.right() {
				let name = arg
					.left()
					.as_identifier()
					.map(|i| self.collect_pattern(&i.into()))
					.unwrap_or_else(|| {
						let src = self.file.source_file(self.db);
						let span = arg.left().span();
						self.diagnostics.add_error(SyntaxError {
							src,
							span,
							msg: format!("Expected identifier, but got {}", arg.left().cst_kind()),
						});
						self.alloc_pattern(&arg.left(), Pattern::Missing)
					});
				named.push((name, self.collect_expression(&e)));
			} else {
				let argument = if named.is_empty() {
					arg.left()
						.as_expression()
						.map(|e| self.collect_expression(&e))
						.unwrap_or_else(|| {
							let src = self.file.source_file(self.db);
							let span = arg.left().span();
							self.diagnostics.add_error(SyntaxError {
								src,
								span,
								msg: "Positional arguments must appear before all named arguments"
									.to_owned(),
							});
							self.alloc_expression(&arg.left(), Expression::Missing)
						})
				} else {
					let src = self.file.source_file(self.db);
					let span = arg.left().span();
					self.diagnostics.add_error(SyntaxError {
						src,
						span,
						msg: "Positional arguments must appear before all named arguments"
							.to_owned(),
					});
					self.alloc_expression(&arg.left(), Expression::Missing)
				};
				positional.push(argument);
			}
		}
		Call {
			kind: CallKind::SourceCall,
			function,
			arguments: positional.into(),
			named_arguments: named.into(),
		}
	}

	fn collect_infix_operator(&mut self, o: &minizinc::InfixOperator) -> ExpressionId<'db> {
		let arguments = [o.left(), o.right()]
			.into_iter()
			.map(|a| self.collect_expression(&a))
			.collect();
		let operator = o.operator();
		let function = self.ident_exp(
			&operator,
			if operator.name() == "==" {
				// Desugar == into =
				"="
			} else {
				operator.name()
			},
		);
		self.alloc_expression(
			o,
			Call {
				kind: CallKind::Operator,
				function,
				arguments,
				named_arguments: Box::new([]),
			},
		)
	}

	fn collect_prefix_operator(&mut self, o: &minizinc::PrefixOperator) -> ExpressionId<'db> {
		let arguments = Box::new([self.collect_expression(&o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(
			&operator,
			if matches!(operator.name(), ".." | "<.." | "..<" | "<..<") {
				format!("o{}", operator.name())
			} else {
				operator.name().to_owned()
			},
		);
		self.alloc_expression(
			o,
			Call {
				kind: CallKind::Operator,
				function,
				arguments,
				named_arguments: Box::new([]),
			},
		)
	}

	fn collect_postfix_operator(&mut self, o: &minizinc::PostfixOperator) -> ExpressionId<'db> {
		let arguments = Box::new([self.collect_expression(&o.operand())]);
		let operator = o.operator();
		let function = self.ident_exp(&operator, format!("{}o", operator.name()));
		self.alloc_expression(
			o,
			Call {
				kind: CallKind::Operator,
				function,
				arguments,
				named_arguments: Box::new([]),
			},
		)
	}

	fn collect_generator_call(&mut self, c: &minizinc::GeneratorCall) -> ExpressionId<'db> {
		// Desugar into call with comprehension as argument
		let comp = ArrayComprehension {
			generators: c.generators().map(|g| self.collect_generator(&g)).collect(),
			indices: None,
			template: self.collect_expression(&c.template()),
		};
		let arguments = Box::new([self.alloc_expression(c, comp)]);
		let function = self.collect_expression(&c.function());
		self.alloc_expression(
			c,
			Call {
				kind: CallKind::GeneratorCall,
				arguments,
				function,
				named_arguments: Box::new([]),
			},
		)
	}

	fn collect_string_interpolation(
		&mut self,
		s: &minizinc::StringInterpolation,
	) -> ExpressionId<'db> {
		// Desugar into concat() of show() calls
		let strings = s
			.contents()
			.map(|c| match c.value(self.text) {
				minizinc::InterpolationValue::Expression(e) => {
					let arguments = Box::new([self.collect_expression(e)]);
					let function = self.alloc_expression(e, self.identifiers.functions.show);
					self.alloc_expression(
						e,
						Call {
							kind: CallKind::Synthetic,
							function,
							arguments,
							named_arguments: Box::new([]),
						},
					)
				}
				minizinc::InterpolationValue::String(s) => {
					let sl = StringLiteral::new(self.db, s);
					self.alloc_expression(&c, sl)
				}
			})
			.collect();
		let arguments = Box::new([self.alloc_expression(s, ArrayLiteral { members: strings })]);
		let function = self.alloc_expression(s, self.identifiers.functions.concat);

		self.alloc_expression(
			s,
			Call {
				kind: CallKind::Synthetic,
				function,
				arguments,
				named_arguments: Box::new([]),
			},
		)
	}

	fn collect_case(&mut self, c: &minizinc::Case) -> Case<'db> {
		let expression = self.collect_expression(&c.expression());
		let cases = c
			.cases()
			.map(|i| CaseItem {
				pattern: self.collect_pattern(&i.pattern()),
				value: self.collect_expression(&i.value()),
			})
			.collect();
		Case { expression, cases }
	}

	fn collect_let(&mut self, l: &minizinc::Let) -> Let<'db> {
		let items = l.items().map(|i| self.collect_let_item(&i)).collect();
		let in_expression = self.collect_expression(&l.in_expression());
		Let {
			items,
			in_expression,
		}
	}

	fn collect_let_item(&mut self, i: &minizinc::LetItem) -> LetItem<'db> {
		match i {
			minizinc::LetItem::Declaration(d) => {
				let declared_type = self.collect_type(&d.declared_type());
				Declaration {
					pattern: self.collect_pattern(&d.pattern()),
					definition: d.definition().map(|def| self.collect_expression(&def)),
					declared_type,
					annotations: d
						.annotations()
						.map(|ann| self.collect_expression(&ann))
						.collect(),
				}
				.into()
			}
			minizinc::LetItem::Constraint(c) => Constraint {
				expression: self.collect_expression(&c.expression()),
				annotations: c
					.annotations()
					.map(|ann| self.collect_expression(&ann))
					.collect(),
			}
			.into(),
		}
	}

	fn collect_tuple_literal(&mut self, t: &minizinc::TupleLiteral) -> TupleLiteral<'db> {
		TupleLiteral {
			fields: t.members().map(|m| self.collect_expression(&m)).collect(),
		}
	}

	fn collect_record_literal(&mut self, r: &minizinc::RecordLiteral) -> RecordLiteral<'db> {
		RecordLiteral {
			fields: r
				.members()
				.map(|m| {
					(
						self.collect_pattern(&m.name().into()),
						self.collect_expression(&m.value()),
					)
				})
				.collect(),
		}
	}

	fn collect_tuple_access(&mut self, t: &minizinc::TupleAccess) -> TupleAccess<'db> {
		let value = IntegerLiteral(t.field().value(self.text).unwrap_or_else(|e| {
			let src = self.file.source_file(self.db);
			let span = t.field().span();
			self.diagnostics.add_error(InvalidNumericLiteral {
				src,
				span,
				msg: e.to_string(),
			});
			1
		}));
		let field = self.alloc_pattern(
			&t.field(),
			Pattern::Integer {
				negated: false,
				value,
			},
		);
		TupleAccess {
			field,
			tuple: self.collect_expression(&t.tuple()),
		}
	}

	fn collect_record_access(&mut self, r: &minizinc::RecordAccess) -> RecordAccess<'db> {
		RecordAccess {
			record: self.collect_expression(&r.record()),
			field: self.collect_pattern(&r.field().into()),
		}
	}

	fn collect_lambda(&mut self, l: &minizinc::Lambda) -> Lambda<'db> {
		Lambda {
			return_type: l.return_type().map(|r| self.collect_type(&r)),
			parameters: l
				.parameters()
				.map(|p| {
					let ty = self.collect_type(&p.declared_type());
					let annotations = p
						.annotations()
						.map(|ann| self.collect_expression(&ann))
						.collect();
					let pattern = p.pattern().map(|p| self.collect_pattern(&p));
					let default = p.default().map(|d| self.collect_expression(&d));
					Parameter {
						declared_type: ty,
						pattern,
						annotations,
						default,
					}
				})
				.collect(),
			body: self.collect_expression(&l.body()),
		}
	}

	fn collect_annotated_expression(
		&mut self,
		e: &minizinc::AnnotatedExpression,
	) -> ExpressionId<'db> {
		let annotations = e
			.annotations()
			.map(|ann| self.collect_expression(&ann))
			.collect();
		let idx = self.collect_expression(&e.expression());
		self.data.annotations.insert(idx, annotations);
		idx
	}

	fn ident_exp<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		id: impl AsRef<str>,
	) -> ExpressionId<'db> {
		self.alloc_expression(ast, Identifier::new(self.db, id))
	}

	pub(super) fn alloc_expression<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		v: impl Into<Expression<'db>>,
	) -> ExpressionId<'db> {
		let index = self.data.expressions.insert(v.into());
		self.source_map.insert(self.db, self.file, index, ast);
		index
	}

	pub(super) fn alloc_type<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		v: impl Into<Type<'db>>,
	) -> TypeId<'db> {
		let index = self.data.types.insert(v);
		self.source_map.insert(self.db, self.file, index, ast);
		index
	}

	pub(super) fn alloc_pattern<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		v: impl Into<Pattern<'db>>,
	) -> PatternId<'db> {
		let index = self.data.patterns.insert(v);
		self.source_map.insert(self.db, self.file, index, ast);
		index
	}
}

impl<'db, 'a> Debug for ExpressionCollector<'db, 'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ExpressionCollector")
			.field("data", &self.data)
			.field("file", &self.file)
			.finish()
	}
}

/// Tracks type-inst identifiers used in a function item
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TypeInstIdentifiers<'db> {
	/// The named type-inst ids
	pub tiids: Map<Identifier<'db>, (bool, TypeInstIdentifierDeclaration<'db>)>,
	/// Anonymous type-inst ids
	pub anons: Vec<TypeInstIdentifierDeclaration<'db>>,
}

impl<'db> TypeInstIdentifiers<'db> {
	/// Get the `TypeInstIdentifierDeclaration`s
	pub fn into_vec(self) -> Vec<TypeInstIdentifierDeclaration<'db>> {
		let mut tiids = self
			.tiids
			.into_values()
			.filter_map(|(ok, d)| if ok { Some(d) } else { None })
			.chain(self.anons)
			.collect::<Vec<_>>();
		tiids.sort_by_key(|tiid| tiid.name);
		tiids
	}
}
