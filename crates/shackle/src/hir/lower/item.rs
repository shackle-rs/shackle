use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::error::SyntaxError;
use crate::file::ModelRef;
use crate::hir::db::Hir;
use crate::hir::ids::ItemRef;
use crate::hir::source::{Origin, SourceMap};
use crate::hir::*;
use crate::syntax::ast::{self, AstNode};
use crate::Error;

use super::ExpressionCollector;

/// Lower a model to HIR
pub fn lower_items(db: &dyn Hir, model: ModelRef) -> (Arc<Model>, Arc<SourceMap>, Arc<Vec<Error>>) {
	let ast = match db.ast(*model) {
		Ok(m) => m,
		Err(e) => return (Default::default(), Default::default(), Arc::new(vec![e])),
	};
	let identifiers = IdentifierRegistry::new(db);
	let mut ctx = ItemCollector::new(db, &identifiers, model);
	for item in ast.items() {
		ctx.collect_item(item);
	}
	let (m, sm, e) = ctx.finish();
	(Arc::new(m), Arc::new(sm), Arc::new(e))
}

/// Collects AST items into an HIR model
pub struct ItemCollector<'a> {
	db: &'a dyn Hir,
	identifiers: &'a IdentifierRegistry,
	model: Model,
	source_map: SourceMap,
	diagnostics: Vec<Error>,
	owner: ModelRef,
}

impl ItemCollector<'_> {
	/// Create a new item collector
	pub fn new<'a>(
		db: &'a dyn Hir,
		identifiers: &'a IdentifierRegistry,
		owner: ModelRef,
	) -> ItemCollector<'a> {
		ItemCollector {
			db,
			identifiers,
			model: Model::default(),
			source_map: SourceMap::default(),
			diagnostics: Vec::new(),
			owner,
		}
	}

	/// Lower an AST item to HIR
	pub fn collect_item(&mut self, item: ast::Item) {
		let (it, sm) = match item.clone() {
			ast::Item::Annotation(a) => self.collect_annotation(a),
			ast::Item::Assignment(a) => self.collect_assignment(a),
			ast::Item::Constraint(c) => self.collect_constraint(c),
			ast::Item::Declaration(d) => self.collect_declaration(d),
			ast::Item::Enumeration(e) => self.collect_enumeration(e),
			ast::Item::Function(f) => self.collect_function(f),
			ast::Item::Include(_i) => return,
			ast::Item::Output(i) => self.collect_output(i),
			ast::Item::Predicate(p) => self.collect_predicate(p),
			ast::Item::Solve(s) => self.collect_solve(s),
			ast::Item::TypeAlias(t) => self.collect_type_alias(t),
		};
		self.source_map.insert(it.into(), Origin::new(&item, None));
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	/// Finish lowering
	pub fn finish(self) -> (Model, SourceMap, Vec<Error>) {
		(self.model, self.source_map, self.diagnostics)
	}

	fn collect_annotation(&mut self, a: ast::Annotation) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let pattern = ctx.collect_pattern(a.id().into());
		let parameters = a.parameters().map(|ps| {
			ps.iter()
				.map(|p| {
					let pattern = p.pattern().map(|pat| ctx.collect_pattern(pat));
					let declared_type = ctx.collect_type(p.declared_type());
					ConstructorParameter {
						declared_type,
						pattern,
					}
				})
				.collect()
		});
		let (data, source_map) = ctx.finish();
		let index = self.model.annotations.insert(Item::new(
			Annotation {
				constructor: Constructor {
					pattern,
					parameters,
				},
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_assignment(&mut self, a: ast::Assignment) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let assignee = ctx.collect_expression(a.assignee());

		if let ast::Expression::Identifier(i) = a.assignee() {
			if self.db.enumeration_names().contains(&i.name().to_owned()) {
				// This is an assignment to an enum
				let mut definition = Vec::new();
				let mut todo = vec![a.definition()];
				while let Some(e) = todo.pop() {
					match e {
						ast::Expression::Identifier(i) => {
							definition.push(Constructor {
								pattern: ctx.collect_pattern(i.into()),
								parameters: None,
							});
						}
						ast::Expression::SetLiteral(sl) => {
							todo.extend(sl.members());
						}
						ast::Expression::Call(c) => match c.function() {
							ast::Expression::Identifier(i) => {
								definition.push(Constructor {
									pattern: ctx.collect_pattern(i.into()),
									parameters: Some(
										c.arguments()
											.map(|arg| {
												let origin = Origin::new(&arg, None);
												let domain = ctx.collect_expression(arg);
												ConstructorParameter {
													declared_type: ctx.alloc_type(
														origin,
														Type::Bounded {
															inst: None,
															opt: None,
															domain,
														},
													),
													pattern: None,
												}
											})
											.collect(),
									),
								});
							}
							_ => {}
						},
						ast::Expression::InfixOperator(o) => {
							todo.push(o.left());
							todo.push(o.right());
						}
						_ => {
							let (src, span) = e.cst_node().source_span(self.db.upcast());
							ctx.add_diagnostic(SyntaxError {
								src,
								span,
								msg: "Expression not valid in enumeration assignment".to_string(),
								other: Vec::new(),
							});
						}
					}
				}
				definition.reverse();
				let (data, source_map) = ctx.finish();
				let index = self.model.enum_assignments.insert(Item::new(
					EnumAssignment {
						assignee,
						definition: definition.into_boxed_slice(),
					},
					data,
				));
				self.model.items.push(index.into());
				return (ItemRef::new(self.db, self.owner, index), source_map);
			}
		}

		let definition = ctx.collect_expression(a.definition());
		let (data, source_map) = ctx.finish();
		let index = self.model.assignments.insert(Item::new(
			Assignment {
				assignee,
				definition,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_constraint(&mut self, c: ast::Constraint) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let annotations = c
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let expression = ctx.collect_expression(c.expression());
		let (data, source_map) = ctx.finish();
		let index = self.model.constraints.insert(Item::new(
			Constraint {
				annotations,
				expression,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_declaration(&mut self, d: ast::Declaration) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let pattern = ctx.collect_pattern(d.pattern());
		let declared_type = ctx.collect_type(d.declared_type());
		let annotations = d
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let definition = d.definition().map(|e| ctx.collect_expression(e));
		let (data, source_map) = ctx.finish();
		let index = self.model.declarations.insert(Item::new(
			Declaration {
				pattern,
				declared_type,
				annotations,
				definition,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_enumeration(&mut self, e: ast::Enumeration) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let pattern = ctx.collect_pattern(e.id().into());
		// Flatten cases
		let mut has_rhs = false;
		let mut cases = Vec::new();
		for case in e.cases() {
			match case {
				ast::EnumerationCase::Members(m) => {
					has_rhs = true;
					for i in m.members() {
						let p = ctx.collect_pattern(i.into());
						cases.push(Constructor {
							pattern: p,
							parameters: None,
						});
					}
				}
				ast::EnumerationCase::Constructor(c) => {
					has_rhs = true;
					let pattern = ctx.collect_pattern(c.id().into());
					let parameters = c
						.parameters()
						.map(|param| ConstructorParameter {
							declared_type: ctx.collect_type(param),
							pattern: None,
						})
						.collect();
					cases.push(Constructor {
						pattern,
						parameters: Some(parameters),
					});
				}
				ast::EnumerationCase::Anonymous(a) => {
					has_rhs = true;
					let pattern = ctx.collect_pattern(a.anonymous().into());
					let parameters = a
						.parameters()
						.map(|param| ConstructorParameter {
							declared_type: ctx.collect_type(param),
							pattern: None,
						})
						.collect();
					cases.push(Constructor {
						pattern,
						parameters: Some(parameters),
					});
				}
			}
		}
		let annotations = e
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let (data, source_map) = ctx.finish();
		let index = self.model.enumerations.insert(Item::new(
			Enumeration {
				annotations,
				pattern,
				definition: if has_rhs {
					Some(cases.into_boxed_slice())
				} else {
					None
				},
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_function(&mut self, f: ast::Function) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let annotations = f
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let body = f.body().map(|e| ctx.collect_expression(e));
		let pattern = ctx.collect_pattern(f.id().into());
		let mut tiids = FxHashMap::default();
		let return_type = ctx.collect_type_with_tiids(f.return_type(), &mut tiids, false, false);
		let parameters = f
			.parameters()
			.map(|p| {
				let ty = ctx.collect_type_with_tiids(p.declared_type(), &mut tiids, false, true);
				let annotations = p
					.annotations()
					.map(|ann| ctx.collect_expression(ann))
					.collect();
				let pattern = p.pattern().map(|p| ctx.collect_pattern(p));
				Parameter {
					declared_type: ty,
					pattern,
					annotations,
				}
			})
			.collect();
		let type_inst_vars = tiids.into_values().collect();
		let (data, source_map) = ctx.finish();
		let index = self.model.functions.insert(Item::new(
			Function {
				annotations,
				type_inst_vars,
				body,
				pattern,
				return_type,
				parameters,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_output(&mut self, i: ast::Output) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let section = i.section().map(|s| ctx.collect_expression(s.into()));
		let expression = ctx.collect_expression(i.expression());
		let (data, source_map) = ctx.finish();
		let index = self.model.outputs.insert(Item::new(
			Output {
				section,
				expression,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_predicate(&mut self, f: ast::Predicate) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);

		let annotations = f
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let body = f.body().map(|e| ctx.collect_expression(e));
		let pattern = ctx.collect_pattern(f.id().into());
		let return_type = ctx.alloc_type(
			Origin::new(&f, None),
			Type::Primitive {
				inst: match f.declared_type() {
					ast::PredicateType::Predicate => VarType::Var,
					ast::PredicateType::Test => VarType::Par,
				},
				opt: OptType::NonOpt,
				primitive_type: PrimitiveType::Bool,
			},
		);
		let mut tiids = FxHashMap::default();
		let parameters = f
			.parameters()
			.map(|p| {
				let ty = ctx.collect_type_with_tiids(p.declared_type(), &mut tiids, false, true);
				let annotations = p
					.annotations()
					.map(|ann| ctx.collect_expression(ann))
					.collect();
				let pattern = p.pattern().map(|p| ctx.collect_pattern(p));
				Parameter {
					declared_type: ty,
					pattern,
					annotations,
				}
			})
			.collect();
		let type_inst_vars = tiids.into_values().collect();
		let (data, source_map) = ctx.finish();
		let index = self.model.functions.insert(Item::new(
			Function {
				annotations,
				type_inst_vars,
				body,
				parameters,
				pattern,
				return_type,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_solve(&mut self, s: ast::Solve) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let annotations = s
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let goal = match s.goal() {
			ast::Goal::Maximize(objective) => Goal::Maximize {
				pattern: ctx.alloc_pattern(
					Origin::new(&objective, None),
					Pattern::Identifier(self.identifiers.objective),
				),
				objective: ctx.collect_expression(objective),
			},
			ast::Goal::Minimize(objective) => Goal::Minimize {
				pattern: ctx.alloc_pattern(
					Origin::new(&objective, None),
					Pattern::Identifier(self.identifiers.objective),
				),
				objective: ctx.collect_expression(objective),
			},
			ast::Goal::Satisfy => Goal::Satisfy,
		};
		let (data, source_map) = ctx.finish();
		let index = self
			.model
			.solves
			.insert(Item::new(Solve { annotations, goal }, data));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_type_alias(&mut self, t: ast::TypeAlias) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let annotations = t
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let name = ctx.collect_pattern(t.name().into());
		let aliased_type = ctx.collect_type(t.aliased_type());
		let (data, source_map) = ctx.finish();
		let index = self.model.type_aliases.insert(Item::new(
			TypeAlias {
				name,
				aliased_type,
				annotations,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}
}
