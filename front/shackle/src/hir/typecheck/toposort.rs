//! Topological sorting of items.
//!
//! Gives the order in which items should be processed (stable topological sort).
//! Checks for cyclic definitions.

use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::{
	error::CyclicDefinition,
	hir::{
		db::Hir,
		ids::{ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		Expression, Goal, Pattern, Type,
	},
	Error,
};

/// Topologically sort items
pub fn topological_sort(db: &dyn Hir) -> (Arc<Vec<ItemRef>>, Arc<Vec<Error>>) {
	let mut topo_sorter = TopoSorter::new(db);
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for it in model.items.iter() {
			let item = ItemRef::new(db, *m, *it);
			topo_sorter.run(item);
		}
	}
	let (sorted, diagnostics) = topo_sorter.finish();
	(Arc::new(sorted), Arc::new(diagnostics))
}

/// Topological sorter
pub struct TopoSorter<'a> {
	db: &'a dyn Hir,
	sorted: Vec<ItemRef>,
	visited: FxHashSet<ItemRef>,
	current: FxHashSet<PatternRef>,
	diagnostics: Vec<Error>,
}

impl<'a> TopoSorter<'a> {
	/// Create a new topological sorter
	pub fn new(db: &'a dyn Hir) -> Self {
		Self {
			db,
			sorted: Vec::new(),
			visited: FxHashSet::default(),
			current: FxHashSet::default(),
			diagnostics: Vec::new(),
		}
	}

	/// Run the topological sorter on an item
	pub fn run(&mut self, item: ItemRef) {
		if self.visited.contains(&item) {
			return;
		}
		self.visited.insert(item);
		let model = item.model(self.db);
		let local_item = item.local_item_ref(self.db);
		match local_item {
			LocalItemRef::Assignment(a) => {
				let types = self.db.lookup_item_types(item);
				if let Some(p) = types.name_resolution(model[a].assignee) {
					self.current.insert(p);
					self.visit_expression(ExpressionRef::new(item, model[a].definition));
					self.current.remove(&p);
				}
			}
			LocalItemRef::Declaration(d) => {
				let data = local_item.data(&*model);
				let pats = Pattern::identifiers(model[d].pattern, data)
					.map(|p| PatternRef::new(item, p))
					.collect::<Vec<_>>();
				self.current.extend(pats.iter().copied());
				for e in Type::expressions(model[d].declared_type, data) {
					self.visit_expression(ExpressionRef::new(item, e));
				}
				for ann in model[d].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann));
				}
				if let Some(def) = model[d].definition {
					self.visit_expression(ExpressionRef::new(item, def));
				}
				for p in pats.iter() {
					self.current.remove(p);
				}
			}
			LocalItemRef::Enumeration(e) => {
				let p = PatternRef::new(item, model[e].pattern);
				self.current.insert(p);
				for ann in model[e].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann));
				}
				if let Some(def) = &model[e].definition {
					let data = local_item.data(&*model);
					for c in def.iter() {
						for param in c.parameters.iter() {
							for e in Type::expressions(*param, data) {
								self.visit_expression(ExpressionRef::new(item, e));
							}
						}
					}
				}
				self.current.remove(&p);
			}
			LocalItemRef::EnumAssignment(e) => {
				let types = self.db.lookup_item_types(item);
				if let Some(p) = types.name_resolution(model[e].assignee) {
					self.current.insert(p);
					let data = local_item.data(&*model);
					for c in model[e].definition.iter() {
						for param in c.parameters.iter() {
							for e in Type::expressions(*param, data) {
								self.visit_expression(ExpressionRef::new(item, e));
							}
						}
					}
					self.current.remove(&p);
				}
			}
			LocalItemRef::Function(f) => {
				let p = PatternRef::new(item, model[f].pattern);
				self.current.insert(p);
				let data = local_item.data(&*model);
				for p in model[f].parameters.iter() {
					for e in Type::expressions(p.declared_type, data) {
						self.visit_expression(ExpressionRef::new(item, e));
					}
				}
				for e in Type::expressions(model[f].return_type, data) {
					self.visit_expression(ExpressionRef::new(item, e));
				}
				for ann in model[f].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann));
				}
				self.current.remove(&p);
			}
			LocalItemRef::Solve(s) => match model[s].goal {
				Goal::Maximize { pattern, objective }
				| Goal::Minimize {
					pattern, objective, ..
				} => {
					let p = PatternRef::new(item, pattern);
					self.current.insert(p);
					for ann in model[s].annotations.iter() {
						self.visit_expression(ExpressionRef::new(item, *ann));
					}
					self.visit_expression(ExpressionRef::new(item, objective));
					self.current.remove(&p);
				}
				_ => {
					for ann in model[s].annotations.iter() {
						self.visit_expression(ExpressionRef::new(item, *ann));
					}
				}
			},
			LocalItemRef::TypeAlias(t) => {
				let p = PatternRef::new(item, model[t].name);
				self.current.insert(p);
				for ann in model[t].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann));
				}
				let data = local_item.data(&*model);
				for e in Type::expressions(model[t].aliased_type, data) {
					self.visit_expression(ExpressionRef::new(item, e));
				}
				self.current.remove(&p);
			}
			LocalItemRef::Constraint(_) | LocalItemRef::Output(_) => (), // Never cyclic, so skip check
		}
		self.sorted.push(item);
	}

	fn visit_expression(&mut self, expression: ExpressionRef) {
		let item = expression.item();
		let model = item.model(self.db);
		let data = item.local_item_ref(self.db).data(&*model);
		let types = self.db.lookup_item_types(item);
		for e in Expression::walk(expression.expression(), data) {
			if let Expression::Identifier(i) = data[e] {
				if let Some(p) = types.name_resolution(e) {
					if self.current.contains(&p) {
						// Cyclic definition, emit error
						let (src, span) =
							NodeRef::from(expression.into_entity(self.db)).source_span(self.db);
						let variable = i.pretty_print(self.db);
						self.diagnostics.push(
							CyclicDefinition {
								src,
								span,
								variable,
							}
							.into(),
						);
						continue;
					}
					self.run(p.item());
				}
			}
		}
	}

	/// Get results of topological sorting
	pub fn finish(self) -> (Vec<ItemRef>, Vec<Error>) {
		(self.sorted, self.diagnostics)
	}
}
