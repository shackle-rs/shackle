use std::fmt::Debug;

use shackle_diagnostics::SyntaxError;
use shackle_syntax::{
	ast::AstNode,
	minizinc::{self, OptType, PrimitiveType, VarType},
};

use super::{ExpressionCollector, TypeInstIdentifiers};
use crate::{
	Db, Identifier,
	constants::IdentifierRegistry,
	diagnostics::Diagnostics,
	input::{ModelFile, ModelFileContents, enumeration_names},
	ir::{Model, Pattern, Type, item::*},
	source::Origin,
};

/// Collects AST items into an HIR model
pub struct ItemCollector<'db, 'a> {
	db: &'db dyn Db,
	identifiers: &'db IdentifierRegistry<'db>,
	items: Vec<Item<'db>>,
	diagnostics: Diagnostics,
	file: ModelFile,
	text: ModelFileContents<'a, 'db>,
}

impl<'db: 'a, 'a> ItemCollector<'db, 'a> {
	/// Create a new item collector
	pub fn new(db: &'db dyn Db, file: ModelFile) -> Self {
		let identifiers = IdentifierRegistry::lookup(db);
		Self {
			db,
			identifiers,
			items: Vec::new(),
			diagnostics: Diagnostics::default(),
			file,
			text: file.contents(db),
		}
	}

	/// Lower an AST item to HIR
	pub fn collect_item(&mut self, item: &minizinc::Item) {
		log::debug!("Lowering {} to HIR", item.cst_kind());
		match item {
			minizinc::Item::Annotation(a) => self.collect_annotation(a),
			minizinc::Item::Assignment(a) => self.collect_assignment(a),
			minizinc::Item::ClassDecl(c) => self.collect_class(c),
			minizinc::Item::Constraint(c) => self.collect_constraint(c),
			minizinc::Item::Declaration(d) => self.collect_declaration(d),
			minizinc::Item::Enumeration(e) => self.collect_enumeration(e),
			minizinc::Item::Function(f) => self.collect_function(f),
			minizinc::Item::Include(_i) => (),
			minizinc::Item::Output(i) => self.collect_output(i),
			minizinc::Item::Predicate(p) => self.collect_predicate(p),
			minizinc::Item::Solve(s) => self.collect_solve(s),
			minizinc::Item::TypeAlias(t) => self.collect_type_alias(t),
		}
	}

	/// Finish lowering
	pub fn finish(self) -> (Model<'db>, Diagnostics) {
		(Model::new(self.db, self.file, self.items), self.diagnostics)
	}

	fn collect_annotation(&mut self, a: &minizinc::Annotation) {
		let documentation = a
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let name = Identifier::new(self.db, a.id().name(self.text.as_ref()));
		let pattern = ctx.alloc_pattern(&a.id(), name);
		let constructor = if let Some(ps) = a.parameters() {
			let destructor = ctx.alloc_pattern(&a.id(), name.inversed(self.db));
			Constructor::Function {
				constructor: pattern,
				destructor,
				parameters: ps
					.iter()
					.map(|p| {
						let pattern = p.pattern().map(|pat| ctx.collect_pattern(&pat));
						let declared_type = ctx.collect_type(&p.declared_type());
						ConstructorParameter {
							declared_type,
							pattern,
						}
					})
					.collect(),
			}
		} else {
			Constructor::Atom { pattern }
		};
		let (data, source_map) = ctx.finish(Annotation { constructor });
		if let Some(body) = a.body() {
			let instead = if a.parameters().is_some() {
				"function item"
			} else {
				"variable declaration"
			};
			let src = self.file.source_file(self.db);
			let span = body.span();
			self.diagnostics.add_error(SyntaxError {
				src,
				span,
				msg: format!(
					"Annotation items cannot have right-hand side definitions. Use a {} instead",
					instead
				),
			});
		}
		self.items.push(
			AnnotationItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, a.span()),
			)
			.into(),
		);
	}

	fn collect_assignment(&mut self, a: &minizinc::Assignment) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let assignee = ctx.collect_expression(&a.assignee());
		let origin = Origin::new(self.file, a.span());

		if let minizinc::Expression::Identifier(i) = a.assignee()
			&& enumeration_names(self.db)
				.contains(&Identifier::new(self.db, i.name(self.text.as_ref())))
		{
			// This is an assignment to an enum
			let mut definition = Vec::new();
			let mut todo = vec![a.definition()];
			while let Some(e) = todo.pop() {
				match e {
					minizinc::Expression::Identifier(i) => {
						definition.push(
							Constructor::Atom {
								pattern: ctx.collect_pattern(&i.into()),
							}
							.into(),
						);
					}
					minizinc::Expression::SetLiteral(sl) => {
						todo.extend(sl.members());
					}
					minizinc::Expression::Call(c) => {
						if let minizinc::Expression::Identifier(i) = c.function() {
							let parameters = c
								.arguments()
								.map(|arg| {
									let domain = ctx.collect_expression(&arg);
									ConstructorParameter {
										declared_type: ctx.alloc_type(
											&arg,
											Type::Bounded {
												inst: None,
												opt: None,
												domain,
											},
										),
										pattern: None,
									}
								})
								.collect();
							if i.name(self.text.as_ref()) == "_" {
								let pattern = ctx.alloc_pattern(&i, Pattern::Anonymous);
								definition.push(EnumConstructor::Anonymous {
									pattern,
									parameters,
								})
							} else {
								let name = Identifier::new(self.db, i.name(self.text.as_ref()));
								definition.push(
									Constructor::Function {
										constructor: ctx.alloc_pattern(&i, name),
										destructor: ctx.alloc_pattern(&i, name.inversed(self.db)),
										parameters,
									}
									.into(),
								);
							}
						}
					}
					minizinc::Expression::InfixOperator(o) => {
						todo.push(o.left());
						todo.push(o.right());
					}
					_ => {
						let src = self.file.source_file(self.db);
						let span = e.span();
						ctx.diagnostics.add_error(SyntaxError {
							src,
							span,
							msg: "Expression not valid in enumeration assignment".to_owned(),
						});
					}
				}
			}
			definition.reverse();
			let (item, sources) = ctx.finish(EnumAssignment {
				assignee,
				definition: definition.into_boxed_slice(),
			});
			self.items
				.push(EnumAssignmentItem::new(self.db, item, sources, origin).into());
			return;
		}

		let definition = ctx.collect_expression(&a.definition());
		let (item, sources) = ctx.finish(Assignment {
			assignee,
			definition,
		});
		self.items
			.push(AssignmentItem::new(self.db, item, sources, origin).into());
	}

	fn collect_class(&mut self, c: &minizinc::ClassDecl) {
		let documentation = c
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let pattern = ctx.collect_pattern(&c.name().into());
		// `this` is bound to its own pattern inside the class body, so that it can
		// be given the class's instance type independently of the class name itself
		let this_pattern = ctx.collect_pattern(&c.name().into());
		let extends = c.extends().map(|e| ctx.collect_expression(&e.into()));
		let items = c.items().map(|i| ctx.collect_class_item(&i)).collect();
		let annotations = c
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let (data, source_map) = ctx.finish(Class {
			pattern,
			this_pattern,
			extends,
			items,
			annotations,
		});
		self.items.push(
			ClassItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, c.span()),
			)
			.into(),
		);
	}

	fn collect_constraint(&mut self, c: &minizinc::Constraint) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let annotations = c
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let expression = ctx.collect_expression(&c.expression());
		let (data, source_map) = ctx.finish(Constraint {
			annotations,
			expression,
		});
		self.items.push(
			ConstraintItem::new(self.db, data, source_map, Origin::new(self.file, c.span())).into(),
		);
	}

	fn collect_declaration(&mut self, d: &minizinc::Declaration) {
		let documentation = d
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let pattern = ctx.collect_pattern(&d.pattern());
		let declared_type = ctx.collect_type(&d.declared_type());
		let annotations = d
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let definition = d.definition().map(|e| ctx.collect_expression(&e));
		let (data, source_map) = ctx.finish(Declaration {
			pattern,
			declared_type,
			annotations,
			definition,
		});
		self.items.push(
			DeclarationItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, d.span()),
			)
			.into(),
		);
	}

	fn collect_enumeration(&mut self, e: &minizinc::Enumeration) {
		let documentation = e
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let pattern = ctx.collect_pattern(&e.id().into());
		// Flatten cases
		let mut has_rhs = false;
		let mut cases = Vec::new();
		for case in e.cases() {
			match case {
				minizinc::EnumerationCase::Members(m) => {
					has_rhs = true;
					for i in m.members() {
						let pattern = ctx.collect_pattern(&i.into());
						cases.push(Constructor::Atom { pattern }.into());
					}
				}
				minizinc::EnumerationCase::Constructor(c) => {
					has_rhs = true;
					let name = Identifier::new(self.db, c.id().name(self.text.as_ref()));
					let parameters = c
						.parameters()
						.map(|param| ConstructorParameter {
							declared_type: ctx.collect_type(&param),
							pattern: None,
						})
						.collect();
					cases.push(
						Constructor::Function {
							constructor: ctx.alloc_pattern(&c.id(), name),
							destructor: ctx.alloc_pattern(&c.id(), name.inversed(self.db)),
							parameters,
						}
						.into(),
					);
				}
				minizinc::EnumerationCase::Anonymous(a) => {
					has_rhs = true;
					let pattern = ctx.collect_pattern(&a.anonymous().into());
					let parameters = a
						.parameters()
						.map(|param| ConstructorParameter {
							declared_type: ctx.collect_type(&param),
							pattern: None,
						})
						.collect();
					cases.push(EnumConstructor::Anonymous {
						pattern,
						parameters,
					});
				}
			}
		}
		let annotations = e
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let (data, source_map) = ctx.finish(Enumeration {
			annotations,
			pattern,
			definition: if has_rhs {
				Some(cases.into_boxed_slice())
			} else {
				None
			},
		});
		self.items.push(
			EnumerationItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, e.span()),
			)
			.into(),
		);
	}

	fn collect_function(&mut self, f: &minizinc::Function) {
		let documentation = f
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let annotations = f
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let body = f.body().map(|e| ctx.collect_expression(&e));
		let pattern = ctx.collect_pattern(&f.id().into());
		let mut tiids = TypeInstIdentifiers::default();
		let return_type = ctx.collect_type_with_tiids(&f.return_type(), &mut tiids, false, false);
		let parameters = f
			.parameters()
			.map(|p| {
				let ty = ctx.collect_type_with_tiids(&p.declared_type(), &mut tiids, false, true);
				let annotations = p
					.annotations()
					.map(|ann| ctx.collect_expression(&ann))
					.collect();
				let pattern = p.pattern().map(|p| ctx.collect_pattern(&p));
				Parameter {
					declared_type: ty,
					pattern,
					annotations,
				}
			})
			.collect();
		let type_inst_vars = tiids.into_vec().into_boxed_slice();
		let (data, source_map) = ctx.finish(Function {
			annotations,
			type_inst_vars,
			body,
			pattern,
			return_type,
			parameters,
		});
		self.items.push(
			FunctionItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, f.span()),
			)
			.into(),
		);
	}

	fn collect_output(&mut self, o: &minizinc::Output) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let section = o.section().map(|s| ctx.collect_expression(&s.into()));
		let expression = ctx.collect_expression(&o.expression());
		let (data, source_map) = ctx.finish(Output {
			section,
			expression,
		});
		self.items.push(
			OutputItem::new(self.db, data, source_map, Origin::new(self.file, o.span())).into(),
		);
	}

	fn collect_predicate(&mut self, f: &minizinc::Predicate) {
		let documentation = f
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);

		let annotations = f
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let body = f.body().map(|e| ctx.collect_expression(&e));
		let pattern = ctx.collect_pattern(&f.id().into());
		let return_type = ctx.alloc_type(
			f,
			Type::Primitive {
				inst: match f.declared_type() {
					minizinc::PredicateType::Predicate => VarType::Var,
					minizinc::PredicateType::Test => VarType::Par,
				},
				opt: OptType::NonOpt,
				primitive_type: PrimitiveType::Bool,
			},
		);
		let mut tiids = TypeInstIdentifiers::default();
		let parameters = f
			.parameters()
			.map(|p| {
				let ty = ctx.collect_type_with_tiids(&p.declared_type(), &mut tiids, false, true);
				let annotations = p
					.annotations()
					.map(|ann| ctx.collect_expression(&ann))
					.collect();
				let pattern = p.pattern().map(|p| ctx.collect_pattern(&p));
				Parameter {
					declared_type: ty,
					pattern,
					annotations,
				}
			})
			.collect();
		let type_inst_vars = tiids.into_vec().into_boxed_slice();
		let (data, source_map) = ctx.finish(Function {
			annotations,
			type_inst_vars,
			body,
			parameters,
			pattern,
			return_type,
		});
		self.items.push(
			FunctionItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, f.span()),
			)
			.into(),
		);
	}

	fn collect_solve(&mut self, s: &minizinc::Solve) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let annotations = s
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let goal = match s.goal() {
			minizinc::Goal::Maximize(objective) => Goal::Maximize {
				pattern: ctx.alloc_pattern(
					&objective,
					Pattern::Identifier(self.identifiers.names.objective),
				),
				objective: ctx.collect_expression(&objective),
			},
			minizinc::Goal::Minimize(objective) => Goal::Minimize {
				pattern: ctx.alloc_pattern(
					&objective,
					Pattern::Identifier(self.identifiers.names.objective),
				),
				objective: ctx.collect_expression(&objective),
			},
			minizinc::Goal::Satisfy => Goal::Satisfy,
		};
		let (data, source_map) = ctx.finish(Solve { annotations, goal });

		self.items.push(
			SolveItem::new(self.db, data, source_map, Origin::new(self.file, s.span())).into(),
		);
	}

	fn collect_type_alias(&mut self, t: &minizinc::TypeAlias) {
		let documentation = t
			.doc_comment()
			.map(|comment| Origin::new(self.file, comment.span()));
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let annotations = t
			.annotations()
			.map(|ann| ctx.collect_expression(&ann))
			.collect();
		let name = ctx.collect_pattern(&t.name().into());
		let aliased_type = ctx.collect_type(&t.aliased_type());
		let (data, source_map) = ctx.finish(TypeAlias {
			name,
			aliased_type,
			annotations,
		});
		self.items.push(
			TypeAliasItem::new(
				self.db,
				data,
				source_map,
				documentation,
				Origin::new(self.file, t.span()),
			)
			.into(),
		);
	}
}

impl<'db, 'a> Debug for ItemCollector<'db, 'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ItemCollector")
			.field("items", &self.items)
			.field("file", &self.file)
			.finish()
	}
}
