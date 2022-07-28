use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::file::ModelRef;
use crate::hir::db::Hir;
use crate::hir::ids::ItemRef;
use crate::hir::source::{Origin, SourceMap};
use crate::hir::*;
use crate::{syntax::ast, Error};

use super::ExpressionCollector;

/// Lower a model to HIR
pub fn lower_items(db: &dyn Hir, model: ModelRef) -> (Arc<Model>, Arc<SourceMap>, Arc<Vec<Error>>) {
	let ast = match db.ast(*model) {
		Ok(m) => m,
		Err(e) => return (Default::default(), Default::default(), Arc::new(vec![e])),
	};
	let mut ctx = ItemCollector::new(db, model);
	for item in ast.items() {
		ctx.collect_item(item);
	}
	let (m, sm, e) = ctx.finish();
	(Arc::new(m), Arc::new(sm), Arc::new(e))
}

/// Collects AST items into an HIR model
pub struct ItemCollector<'a> {
	db: &'a dyn Hir,
	model: Model,
	source_map: SourceMap,
	diagnostics: Vec<Error>,
	owner: ModelRef,
}

impl ItemCollector<'_> {
	/// Create a new item collector
	pub fn new<'a>(db: &'a dyn Hir, owner: ModelRef) -> ItemCollector<'a> {
		ItemCollector {
			db,
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
		self.source_map.insert(it.into(), item.into());
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	/// Finish lowering
	pub fn finish(self) -> (Model, SourceMap, Vec<Error>) {
		(self.model, self.source_map, self.diagnostics)
	}

	fn collect_annotation(&mut self, a: ast::Annotation) -> (ItemRef, ItemDataSourceMap) {
		// Desugar annotation into either a function declaration or a variable declaration
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let mut tiids = FxHashMap::default();
		let parameters = a
			.parameters()
			.map(|p| {
				let ty = ctx.collect_type_with_tiids(p.declared_type(), &mut tiids, false, true);
				let annotations = p
					.annotations()
					.map(|ann| ctx.collect_expression(ann))
					.collect();
				let pattern = p.pattern().map(|pat| ctx.collect_pattern(pat));
				Parameter {
					declared_type: ty,
					pattern,
					annotations,
				}
			})
			.collect::<Vec<_>>();
		let ty = ctx.alloc_type(
			ast::Item::from(a.clone()).into(),
			Type::Primitive {
				inst: VarType::Par,
				opt: OptType::NonOpt,
				primitive_type: PrimitiveType::Ann,
			},
		);
		let type_inst_vars = tiids.into_values().collect();
		let body = a.body().map(|e| ctx.collect_expression(e));
		if body.is_none() && parameters.is_empty() {
			let pattern = ctx.collect_pattern(a.id().into());
			let (data, source_map) = ctx.finish();
			let index = self.model.declarations.insert(Item::new(
				Declaration {
					annotations: Box::new([]),
					declared_type: ty,
					definition: None,
					pattern,
				},
				data,
			));
			self.model.items.push(index.into());
			(ItemRef::new(self.db, self.owner, index), source_map)
		} else {
			let pattern = ctx.collect_pattern(a.id().into());
			let (data, source_map) = ctx.finish();
			let index = self.model.functions.insert(Item::new(
				Function {
					annotations: Box::new([]),
					type_inst_vars,
					body,
					pattern,
					return_type: ty,
					parameters: parameters.into_boxed_slice(),
				},
				data,
			));
			self.model.items.push(index.into());
			(ItemRef::new(self.db, self.owner, index), source_map)
		}
	}

	fn collect_assignment(&mut self, a: ast::Assignment) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let assignee = ctx.collect_expression(a.assignee());
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
						cases.push(EnumerationCase {
							pattern: p,
							parameters: Box::new([]),
						});
					}
				}
				ast::EnumerationCase::Constructor(c) => {
					has_rhs = true;
					let pattern = ctx.collect_pattern(c.id().into());
					let parameters = c
						.arguments()
						.map(|arg| {
							let argument = ctx.collect_expression(arg.clone());
							ctx.alloc_type(
								arg.into(),
								Type::Bounded {
									inst: Some(VarType::Par),
									opt: Some(OptType::NonOpt),
									domain: argument,
								},
							)
						})
						.collect();
					cases.push(EnumerationCase {
						pattern,
						parameters,
					});
				}
				ast::EnumerationCase::Anonymous(a) => {
					has_rhs = true;
					let pattern = ctx.collect_pattern(a.anonymous().into());
					let parameters = a
						.arguments()
						.map(|arg| {
							let argument = ctx.collect_expression(arg.clone());
							ctx.alloc_type(
								arg.into(),
								Type::Bounded {
									inst: Some(VarType::Par),
									opt: Some(OptType::NonOpt),
									domain: argument,
								},
							)
						})
						.collect();
					cases.push(EnumerationCase {
						pattern,
						parameters,
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);

		let annotations = f
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let body = f.body().map(|e| ctx.collect_expression(e));
		let pattern = ctx.collect_pattern(f.id().into());
		let return_type = ctx.alloc_type(
			ast::Item::from(f.clone()).into(),
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let annotations = s
			.annotations()
			.map(|ann| ctx.collect_expression(ann))
			.collect();
		let goal = match s.goal() {
			ast::Goal::Maximize(objective) => Goal::Maximize {
				pattern: ctx.alloc_pattern(
					Origin::new(objective.clone(), None),
					Pattern::Identifier(Identifier::new("_objective", self.db)),
				),
				objective: ctx.collect_expression(objective),
			},
			ast::Goal::Minimize(objective) => Goal::Minimize {
				pattern: ctx.alloc_pattern(
					Origin::new(objective.clone(), None),
					Pattern::Identifier(Identifier::new("_objective", self.db)),
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
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
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
