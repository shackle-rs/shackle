//! Topological sorting of items.
//!
//! Gives the order in which items should be processed (stable topological sort).
//! Checks for cyclic definitions.
//!
//! Also ensures that globals (possibly transitively) used in function bodies
//! appear before the function declaration.

use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::{
	diagnostics::CyclicDefinition,
	hir::{
		db::Hir,
		ids::{ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		Expression, Goal, Pattern, Type,
	},
	ty::FunctionEntry,
	Error,
};

use super::PatternTy;

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
			LocalItemRef::Annotation(a) => {
				let data = local_item.data(&model);
				for p in model[a].parameters() {
					for e in Type::expressions(p.declared_type, data) {
						self.visit_expression(ExpressionRef::new(item, e), None);
					}
				}
			}
			LocalItemRef::Assignment(a) => {
				let types = self.db.lookup_item_types(item);
				if let Some(p) = types.name_resolution(model[a].assignee) {
					self.current.insert(p);
					self.visit_expression(ExpressionRef::new(item, model[a].definition), None);
					self.current.remove(&p);
				}
			}
			LocalItemRef::Constraint(c) => {
				for ann in model[c].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann), None);
				}
				self.visit_expression(ExpressionRef::new(item, model[c].expression), None);
			}
			LocalItemRef::Declaration(d) => {
				let data = local_item.data(&model);
				let pats = Pattern::identifiers(model[d].pattern, data)
					.map(|p| PatternRef::new(item, p))
					.collect::<Vec<_>>();
				self.current.extend(pats.iter().copied());
				for e in Type::expressions(model[d].declared_type, data) {
					self.visit_expression(ExpressionRef::new(item, e), None);
				}
				for ann in model[d].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann), None);
				}
				if let Some(def) = model[d].definition {
					self.visit_expression(ExpressionRef::new(item, def), None);
				}
				for p in pats.iter() {
					self.current.remove(p);
				}
			}
			LocalItemRef::Enumeration(e) => {
				let p = PatternRef::new(item, model[e].pattern);
				self.current.insert(p);
				for ann in model[e].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann), None);
				}
				if let Some(def) = &model[e].definition {
					let data = local_item.data(&model);
					for c in def.iter() {
						for param in c.parameters() {
							for e in Type::expressions(param.declared_type, data) {
								self.visit_expression(ExpressionRef::new(item, e), None);
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
					let data = local_item.data(&model);
					for c in model[e].definition.iter() {
						for param in c.parameters() {
							for e in Type::expressions(param.declared_type, data) {
								self.visit_expression(ExpressionRef::new(item, e), None);
							}
						}
					}
					self.current.remove(&p);
				}
			}
			LocalItemRef::Function(f) => {
				let name = model[f].data[model[f].pattern].identifier().unwrap();
				let mut overloads = Vec::new();
				let ps = self.db.lookup_global_function(name);
				for p in ps.iter() {
					let signature = self.db.lookup_item_signature(p.item());
					match &signature.patterns[p] {
						PatternTy::Function(f)
						| PatternTy::AnnotationConstructor(f)
						| PatternTy::AnnotationDestructure(f) => {
							overloads.push((p.item() == item, *f.clone()));
						}
						PatternTy::EnumConstructor(ec) => {
							overloads.extend(
								ec.iter().map(|f| (p.item() == item, f.constructor.clone())),
							);
						}
						PatternTy::EnumDestructure(fs) => {
							overloads.extend(fs.iter().map(|f| (p.item() == item, f.clone())));
						}
						_ => unreachable!(),
					}
				}
				let p = PatternRef::new(item, model[f].pattern);
				let types = self.db.lookup_item_signature(item);
				match &types.patterns[&p] {
					PatternTy::Function(f) => {
						let (is_self, _, _) = FunctionEntry::match_fn(
							self.db.upcast(),
							overloads,
							f.overload.params(),
						)
						.unwrap();
						if !is_self {
							// Ignore this function since it has been subsumed by another
							return;
						}
					}
					_ => unreachable!(),
				}
				self.current.insert(p);
				let data = local_item.data(&model);
				for p in model[f].parameters.iter() {
					for ann in p.annotations.iter() {
						self.visit_expression(ExpressionRef::new(item, *ann), None);
					}
					for e in Type::expressions(p.declared_type, data) {
						self.visit_expression(ExpressionRef::new(item, e), None);
					}
				}
				for e in Type::expressions(model[f].return_type, data) {
					self.visit_expression(ExpressionRef::new(item, e), None);
				}
				for ann in model[f].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann), None);
				}
				if let Some(body) = model[f].body {
					// Inside this expression, don't visit function items for calls, instead visit the body since we
					// only care about globals and recursive functions are allowed.
					self.visit_expression(ExpressionRef::new(item, body), Some(p));
				}
				self.current.remove(&p);
			}
			LocalItemRef::Output(o) => {
				if let Some(s) = model[o].section {
					self.visit_expression(ExpressionRef::new(item, s), None);
				}
				self.visit_expression(ExpressionRef::new(item, model[o].expression), None);
			}
			LocalItemRef::Solve(s) => match model[s].goal {
				Goal::Maximize { pattern, objective }
				| Goal::Minimize {
					pattern, objective, ..
				} => {
					let p = PatternRef::new(item, pattern);
					self.current.insert(p);
					for ann in model[s].annotations.iter() {
						self.visit_expression(ExpressionRef::new(item, *ann), None);
					}
					self.visit_expression(ExpressionRef::new(item, objective), None);
					self.current.remove(&p);
				}
				_ => {
					for ann in model[s].annotations.iter() {
						self.visit_expression(ExpressionRef::new(item, *ann), None);
					}
				}
			},
			LocalItemRef::TypeAlias(t) => {
				let p = PatternRef::new(item, model[t].name);
				self.current.insert(p);
				for ann in model[t].annotations.iter() {
					self.visit_expression(ExpressionRef::new(item, *ann), None);
				}
				let data = local_item.data(&model);
				for e in Type::expressions(model[t].aliased_type, data) {
					self.visit_expression(ExpressionRef::new(item, e), None);
				}
				self.current.remove(&p);
			}
		}
		self.sorted.push(item);
	}

	fn visit_expression(&mut self, expression: ExpressionRef, visit_call_body: Option<PatternRef>) {
		let mut todo = vec![expression];
		let mut seen = visit_call_body.into_iter().collect::<FxHashSet<_>>();
		while let Some(expression) = todo.pop() {
			let item = expression.item();
			let model = item.model(self.db);
			let data = item.local_item_ref(self.db).data(&model);
			let types = self.db.lookup_item_types(item);
			for e in Expression::walk(expression.expression(), data) {
				if let Expression::Identifier(i) = data[e] {
					if let Some(p) = types.name_resolution(e) {
						if (visit_call_body.is_none() || !seen.contains(&p))
							&& self.current.contains(&p)
						{
							// Cyclic definition, emit error
							let (src, span) =
								NodeRef::from(ExpressionRef::new(item, e).into_entity(self.db))
									.source_span(self.db);
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
						if visit_call_body.is_some() {
							if let LocalItemRef::Function(f) = p.item().local_item_ref(self.db) {
								if !seen.contains(&p) {
									if let Some(body) = p.item().model(self.db)[f].body {
										seen.insert(p);
										todo.push(ExpressionRef::new(p.item(), body));
									}
								}
								continue;
							}
						}
						self.run(p.item());
					}
				}
			}
		}
	}

	/// Get results of topological sorting
	pub fn finish(self) -> (Vec<ItemRef>, Vec<Error>) {
		(self.sorted, self.diagnostics)
	}
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use expect_test::{expect, Expect};

	use crate::{
		db::{CompilerDatabase, FileReader, Inputs},
		file::InputFile,
		hir::db::Hir,
	};

	fn check_toposort(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		db.set_ignore_stdlib(true);
		db.set_input_files(Arc::new(vec![InputFile::ModelString(model.to_owned())]));
		let model = db.input_models()[0];
		let sm = db.lookup_source_map(model);
		let items = db.lookup_topological_sorted_items();
		let mut actual = String::new();
		for item in items.iter().copied() {
			let origin = sm.get_origin(item.into()).unwrap();
			let (source, span) = origin.source_span(&db);
			actual.push_str(&source.contents()[span.offset()..span.offset() + span.len()]);
			actual.push_str(";\n");
		}
		expected.assert_eq(&actual);
	}

	#[test]
	fn test_topological_sort() {
		check_toposort(
			r#"
			constraint x;
			var bool: x;
		"#,
			expect!([r#"
    var bool: x;
    constraint x;
"#]),
		);

		check_toposort(
			r#"
			constraint let {
				int: y = 3;
				constraint x;
			} in foo(y);
			var bool: x;
			predicate foo(int: a);
		"#,
			expect!([r#"
    predicate foo(int: a);
    var bool: x;
    constraint let {
    				int: y = 3;
    				constraint x;
    			} in foo(y);
"#]),
		);
	}
}
