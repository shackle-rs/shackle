use std::iter;

use shackle_diagnostics::InvalidArrayLiteral;
use shackle_syntax::{ast::AstNode, eprime};
use shackle_utils::maybe_grow_stack;

use crate::{Db, diagnostics::Diagnostics, input::ModelFile, source::SourceMap, *};

/// Collects AST expressions for owned by an item and lowers them into HIR recursively.
#[derive(Debug)]
pub struct ExpressionCollector<'db, 'a> {
	db: &'db dyn Db,
	data: ItemData<'db>,
	source_map: SourceMap<'db>,
	diagnostics: &'a mut Diagnostics,
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
		ExpressionCollector {
			db,
			data: ItemData::new(),
			source_map: SourceMap::default(),
			diagnostics,
			file,
			text,
		}
	}

	/// Lower an AST expression into HIR
	pub fn collect_expression(&mut self, expression: &eprime::Expression) -> ExpressionId<'db> {
		maybe_grow_stack(|| self.collect_expression_inner(expression))
	}

	fn collect_expression_inner(&mut self, expression: &eprime::Expression) -> ExpressionId<'db> {
		log::debug!("Lowering {:?} to HIR", expression);
		if expression.is_missing() {
			return self.alloc_expression(expression, Expression::Missing);
		}
		let collected: Expression = match expression {
			eprime::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			eprime::Expression::IntegerLiteral(i) => IntegerLiteral(i.value(self.text)).into(),
			eprime::Expression::Infinity(_) => Expression::Infinity,
			eprime::Expression::StringLiteral(s) => {
				StringLiteral::new(self.db, s.value(self.text)).into()
			}
			eprime::Expression::MatrixLiteral(m) => return self.collect_matrix_literal(m, false),
			eprime::Expression::Call(c) => {
				let f = c.function().name(self.text);
				self.collect_call(f, c.arguments(), c, CallKind::SourceCall)
					.into()
			}
			eprime::Expression::Identifier(i) => Identifier::new(self.db, i.name(self.text)).into(),
			eprime::Expression::ArrayAccess(aa) => self.collect_array_access(aa).into(),
			eprime::Expression::InfixOperator(o) => self
				.collect_call(
					o.operator().name(),
					vec![o.left(), o.right()].into_iter(),
					o,
					CallKind::Operator,
				)
				.into(),
			eprime::Expression::PrefixOperator(o) => self
				.collect_call(
					o.operator().name(),
					iter::once(o.operand()),
					o,
					CallKind::Operator,
				)
				.into(),
			eprime::Expression::UnarySetConstructor(o) => self
				.collect_call(
					o.operator().name(),
					iter::once(o.operand()),
					o,
					CallKind::Operator,
				)
				.into(),
			eprime::Expression::Quantification(q) => self.collect_quantification(q).into(),
			eprime::Expression::MatrixComprehension(m) => {
				return self.collect_matrix_comprehension(m);
			}
			eprime::Expression::AbsoluteOperator(a) => self
				.collect_call("abs", iter::once(a.operand()), a, CallKind::Operator)
				.into(),
			eprime::Expression::SetConstructor(o) => self
				.collect_call(
					o.operator().name(),
					vec![o.left(), o.right()].into_iter(),
					o,
					CallKind::Operator,
				)
				.into(),
		};
		self.alloc_expression(expression, collected)
	}

	/// Lower Domain/Type into HIR
	pub fn collect_domain(&mut self, d: &eprime::Domain, var_type: VarType) -> TypeId<'db> {
		let domain_expr = self.collect_domain_expressions(d, var_type);
		let domain = match domain_expr {
			CollectedDomain::Array(a) => a,
			CollectedDomain::Primitive(p) => Type::Primitive {
				inst: var_type,
				opt: OptType::NonOpt,
				primitive_type: p,
			},
			CollectedDomain::Bounded(b) => Type::Bounded {
				inst: Some(var_type),
				opt: None,
				domain: b,
			},
		};
		self.alloc_type(d, domain)
	}

	/// Helper function that collects the expressions within the domain. Important for
	/// compatibility with domain operations
	pub(super) fn collect_domain_expressions(
		&mut self,
		t: &eprime::Domain,
		var_type: VarType,
	) -> CollectedDomain<'db> {
		CollectedDomain::Bounded(match t {
			eprime::Domain::Identifier(i) => {
				let ident = Identifier::new(self.db, i.name(self.text));
				self.alloc_expression(t, ident)
			}
			eprime::Domain::DomainOperation(d) => {
				let left = self
					.collect_domain_expressions(&d.left(), var_type)
					.into_expression(self, t);
				let right = self
					.collect_domain_expressions(&d.right(), var_type)
					.into_expression(self, t);
				let op = d.operator();
				let operator = if op.name() == "-" { "diff" } else { op.name() }; // Convert Eprime operators to MiniZinc ones
				let function = self.ident_exp(&op, operator);
				self.alloc_expression(
					d,
					Call {
						kind: CallKind::Operator,
						function,
						arguments: Box::new([left, right]),
						named_arguments: Box::new([]),
					},
				)
			}
			eprime::Domain::MatrixDomain(m) => {
				let domain_indexes = m
					.indexes()
					.map(|i| self.collect_domain(&i, VarType::Par))
					.collect::<Box<_>>();
				let dimensions = if domain_indexes.len() > 1 {
					self.alloc_type(
						m,
						Type::Tuple {
							opt: OptType::NonOpt,
							fields: domain_indexes,
						},
					)
				} else {
					*domain_indexes.first().unwrap()
				};
				let domain_base = self.collect_domain_expressions(&m.base(), var_type);
				let element = self.alloc_type(
					m,
					match domain_base {
						CollectedDomain::Primitive(p) => Type::Primitive {
							inst: var_type,
							opt: OptType::NonOpt,
							primitive_type: p,
						},
						CollectedDomain::Bounded(b) => Type::Bounded {
							inst: Some(var_type),
							opt: None,
							domain: b,
						},
						CollectedDomain::Array(a) => a,
					},
				);
				return CollectedDomain::Array(Type::Array {
					opt: OptType::NonOpt,
					dimension_pattern: None,
					dimensions,
					element,
				});
			}
			eprime::Domain::BooleanDomain(_) => {
				return CollectedDomain::Primitive(PrimitiveType::Bool);
			}
			eprime::Domain::IntegerDomain(i) => {
				let mut set_constructor_domain_members = Vec::new();
				let mut domain_members = Vec::new();
				for e in i.domain() {
					match e {
						eprime::Expression::UnarySetConstructor(_)
						| eprime::Expression::SetConstructor(_) => {
							set_constructor_domain_members.push(self.collect_expression(&e))
						}
						e => {
							domain_members.push(self.collect_expression(&e));
						}
					}
				}
				let call_domain = if set_constructor_domain_members.len() > 1 {
					let union_expr = self.ident_exp(i, "union");
					set_constructor_domain_members.into_iter().reduce(|acc, e| {
						self.alloc_expression(
							i,
							Call {
								kind: CallKind::Synthetic,
								function: union_expr,
								arguments: Box::new([acc, e]),
								named_arguments: Box::new([]),
							},
						)
					})
				} else {
					set_constructor_domain_members.into_iter().next()
				};
				let domain = if !domain_members.is_empty() {
					Some(self.alloc_expression(
						i,
						SetLiteral {
							members: domain_members.into_boxed_slice(),
						},
					))
				} else {
					None
				};

				match (domain, call_domain) {
					(Some(l), Some(d)) => {
						let union_expr = self.ident_exp(i, "union");
						self.alloc_expression(
							i,
							Call {
								kind: CallKind::Synthetic,
								function: union_expr,
								arguments: Box::new([l, d]),
								named_arguments: Box::new([]),
							},
						)
					}
					(None, Some(d)) => d,
					(Some(l), None) => l,
					(None, None) => return CollectedDomain::Primitive(PrimitiveType::Int),
				}
			}
			eprime::Domain::AnyDomain(_) => return CollectedDomain::Array(Type::Any),
		})
	}

	fn collect_call<'tree>(
		&mut self,
		o: &str,
		args: impl Iterator<Item = eprime::Expression<'tree>>,
		ast: &impl AstNode<'tree>,
		kind: CallKind,
	) -> Call<'db> {
		let arguments = args
			.into_iter()
			.map(|a| self.collect_expression(&a))
			.collect::<Box<_>>();
		let function = self.ident_exp(
			ast,
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
			kind,
			function,
			arguments,
			named_arguments: Box::new([]),
		}
	}

	fn collect_array_access(&mut self, aa: &eprime::ArrayAccess) -> ArrayAccess<'db> {
		let indices = aa
			.indices()
			.map(|i| match i {
				eprime::ArrayIndex::Expression(e) => self.collect_expression(&e),
				eprime::ArrayIndex::IndexSlice(s) => self.alloc_expression(
					&s,
					Expression::Slice(Identifier::new(self.db, s.operator())),
				),
			})
			.collect::<Box<_>>();
		ArrayAccess {
			collection: self.collect_expression(&aa.collection()),
			indices: if indices.len() == 1 {
				indices[0]
			} else {
				self.alloc_expression(aa, TupleLiteral { fields: indices })
			},
		}
	}

	/// Collect a matrix literal into HIR
	/// is_comprehension_template is used for array comprehensions to turn the first dimension into a tuple
	pub fn collect_matrix_literal(
		&mut self,
		ml: &eprime::MatrixLiteral,
		is_comp_template: bool,
	) -> ExpressionId<'db> {
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
								self.collect_domain_expressions(&i, VarType::Par)
									.into_expression(self, &i),
							);
						}
					}
					let mut members = ml.members().collect::<Vec<_>>();
					members.reverse();
					elem_stack.append(&mut members);
				}
				e => {
					is_finding_dimensions = false;
					array_values.push(self.collect_expression(&e))
				}
			}
		}
		let members = array_values.into_boxed_slice();

		match (dimensions.len(), index_sets.len(), is_comp_template) {
			// Case of 1d array without index set
			(1, 0, false) => self.alloc_expression(ml, ArrayLiteral { members }),
			// Case of 1d array in matrix comprehension without index set
			(1, 0, true) => self.alloc_expression(ml, TupleLiteral { fields: members }),
			// Case of 2d array without index set
			(2, 0, false) => self.alloc_expression(
				ml,
				ArrayLiteral2D {
					members,
					rows: MaybeIndexSet::NonIndexed(dimensions[0]),
					columns: MaybeIndexSet::NonIndexed(dimensions[1]),
				},
			),
			// Case of nd array with possible index set
			(d, i, c) => {
				let src = self.file.source_file(self.db);
				let span = ml.span();
				if d > 6 {
					return self.add_array_over_dims_diagnostic(ml);
				}
				if d != i && i != 0 {
					self.diagnostics.add_error(InvalidArrayLiteral {
						src,
						span,
						msg: "Matrix literal has mismatched dimensions and index sets".to_owned(),
					});
					return self.alloc_expression(ml, Expression::Missing);
				}
				// If no index set exists use index set sized at dimensions
				if i == 0 {
					index_sets = dimensions
						.iter()
						.map(|n| {
							let one = self.alloc_expression(ml, IntegerLiteral(1));
							let n = self.alloc_expression(ml, IntegerLiteral(*n as i64));
							let function = self.ident_exp(ml, "..");
							self.alloc_expression(
								ml,
								Call {
									kind: CallKind::Synthetic,
									function,
									arguments: Box::new([one, n]),
									named_arguments: Box::new([]),
								},
							)
						})
						.collect::<Vec<_>>();
				}
				if c {
					let _ = index_sets.remove(0);
					index_sets.push(self.alloc_expression(ml, TupleLiteral { fields: members }));
				} else {
					index_sets.push(self.alloc_expression(ml, ArrayLiteral { members }));
				}
				let function = self.ident_exp(ml, format!("array{}d", if c { d - 1 } else { d }));
				self.alloc_expression(
					ml,
					Call {
						kind: CallKind::Synthetic,
						function,
						arguments: index_sets.into_boxed_slice(),
						named_arguments: Box::new([]),
					},
				)
			}
		}
	}

	fn collect_quantification(&mut self, q: &eprime::Quantification) -> Call<'db> {
		let comp = ArrayComprehension {
			generators: Box::new([self.collect_generator(&q.generator(), None)]),
			indices: None,
			template: self.collect_expression(&q.template()),
		};
		let arguments = Box::new([self.alloc_expression(q, comp)]);
		let ident = match q.function().name(self.text) {
			"forAll" => "forall",
			q => q,
		};
		let function = self.ident_exp(q, ident);
		Call {
			kind: CallKind::GeneratorCall,
			arguments,
			function,
			named_arguments: Box::new([]),
		}
	}

	fn collect_matrix_comprehension(
		&mut self,
		m: &eprime::MatrixComprehension,
	) -> ExpressionId<'db> {
		let mut generators = self.collect_generators(m);
		let mut indices = self.collect_generator_names(m);
		let initial_indices_len = indices.len();
		let template = match m.template() {
			eprime::Expression::MatrixLiteral(ml) => self.collect_matrix_literal(&ml, true),
			eprime::Expression::MatrixComprehension(_) => {
				let mut current_comp = m.template();
				while let eprime::Expression::MatrixComprehension(mc) = current_comp {
					generators.extend(self.collect_generators(&mc));
					indices.extend(self.collect_generator_names(&mc));
					current_comp = mc.template();
				}
				self.collect_expression(&current_comp)
			}
			t => self.collect_expression(&t),
		};
		// If it is a nested matrix comprehension, create a tuple literal for the indices (e.g. (i,j))
		let indices = if indices.len() > initial_indices_len {
			Some(self.alloc_expression(
				m,
				TupleLiteral {
					fields: indices.into_boxed_slice(),
				},
			))
		} else {
			None
		};
		let matrix_comprehension = self.alloc_expression(
			m,
			ArrayComprehension {
				template,
				indices,
				generators: generators.into_boxed_slice(),
			},
		);

		match m.indices() {
			Some(i) => {
				let index_set = self
					.collect_domain_expressions(&i, VarType::Par)
					.into_expression(self, m);
				let function = self.ident_exp(m, "array1d");
				self.alloc_expression(
					m,
					Call {
						kind: CallKind::Synthetic,
						function,
						arguments: Box::new([index_set, matrix_comprehension]),
						named_arguments: Box::new([]),
					},
				)
			}
			None => matrix_comprehension,
		}
	}

	fn collect_generator_names(
		&mut self,
		m: &eprime::MatrixComprehension,
	) -> Vec<ExpressionId<'db>> {
		m.generators()
			.flat_map(|g| {
				g.names()
					.map(|n| {
						let ident = Identifier::new(self.db, n.name(self.text));
						self.alloc_expression(m, ident)
					})
					.collect::<Vec<_>>()
			})
			.collect()
	}

	fn collect_generators(&mut self, m: &eprime::MatrixComprehension) -> Vec<Generator<'db>> {
		m.generators()
			.zip(m.conditions().map(Some).chain(iter::repeat(None)))
			.map(|(g, c)| {
				let cond = c.map(|c| self.collect_expression(&c));
				self.collect_generator(&g, cond)
			})
			.collect::<Vec<_>>()
	}

	fn collect_generator(
		&mut self,
		g: &eprime::Generator,
		where_clause: Option<ExpressionId<'db>>,
	) -> Generator<'db> {
		let patterns = g.names().map(|i| self.alloc_ident_pattern(g, i)).collect();
		let collection = self
			.collect_domain_expressions(&g.collection(), VarType::Par)
			.into_expression(self, g);
		Generator::Iterator {
			patterns,
			collection,
			where_clause,
		}
	}

	/// Helper to create an identifier expression
	pub fn ident_exp<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		id: impl AsRef<str>,
	) -> ExpressionId<'db> {
		self.alloc_expression(ast, Identifier::new(self.db, id))
	}

	/// Add diagnostic for array literals with >6 dimensions
	pub fn add_array_over_dims_diagnostic<'tree>(
		&mut self,
		n: &impl AstNode<'tree>,
	) -> ExpressionId<'db> {
		let src = self.file.source_file(self.db);
		let span = n.span();
		self.diagnostics.add_error(InvalidArrayLiteral {
			src,
			span,
			msg: "Support for matrix literals with >6 dimensions not currently supported"
				.to_owned(),
		});
		self.alloc_expression(n, Expression::Missing)
	}

	/// Get the collected expressions
	pub fn finish<T: salsa::Update>(mut self, item: T) -> (ItemWithData<'db, T>, SourceMap<'db>) {
		self.data.shrink_to_fit();
		(ItemWithData::new(item, self.data), self.source_map)
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

	/// Helper to convert an identifier into a pattern
	pub(super) fn alloc_ident_pattern<'tree>(
		&mut self,
		ast: &impl AstNode<'tree>,
		i: eprime::Identifier,
	) -> PatternId<'db> {
		let ident = Identifier::new(self.db, i.name(self.text));
		let index = self.data.patterns.insert(Pattern::Identifier(ident));
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

/// Represents a collected domain in the expression collector
/// Preserves relevant information depending on the type of domain
pub(super) enum CollectedDomain<'db> {
	Array(Type<'db>),
	Primitive(PrimitiveType),
	Bounded(ExpressionId<'db>),
}

impl<'db, 'a> CollectedDomain<'db> {
	/// Convert a collected domain into a usable expression
	pub(super) fn into_expression<'tree>(
		self,
		ctx: &mut ExpressionCollector<'db, 'a>,
		ast: &impl AstNode<'tree>,
	) -> ExpressionId<'db> {
		match self {
			// This is inline with the specification which restricts domain expressions to be int and bool
			// Additionally this can't be represented in MiniZinc
			CollectedDomain::Array(_) => unreachable!("Can't use array domain as expression"),
			CollectedDomain::Primitive(p) => {
				// Convert into a primitive range between domains min and max
				let (l, r): (Expression, Expression) = match p {
					PrimitiveType::Bool => {
						(BooleanLiteral(false).into(), BooleanLiteral(true).into())
					}
					PrimitiveType::Int => {
						let inf = ctx.alloc_expression(ast, Expression::Infinity);
						(
							Call {
								kind: CallKind::Synthetic,
								function: ctx.ident_exp(ast, "-"),
								arguments: Box::new([inf]),
								named_arguments: Box::new([]),
							}
							.into(),
							Expression::Infinity,
						)
					}
					PrimitiveType::Float | PrimitiveType::String | PrimitiveType::Ann => {
						unreachable!("These primatives aren't implemented in EPrime")
					}
				};
				let l = ctx.alloc_expression(ast, l);
				let r = ctx.alloc_expression(ast, r);
				let function = ctx.ident_exp(ast, "..");
				ctx.alloc_expression(
					ast,
					Call {
						kind: CallKind::Synthetic,
						function,
						arguments: Box::new([l, r]),
						named_arguments: Box::new([]),
					},
				)
			}
			CollectedDomain::Bounded(b) => b,
		}
	}
}
