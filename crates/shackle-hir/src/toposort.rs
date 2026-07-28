//! Topological sorting of items.
//!
//! Gives the order in which items should be processed (stable topological sort).
//! Checks for cyclic definitions.
//!
//! Also ensures that globals (possibly transitively) used in function bodies
//! appear before the function declaration.

use shackle_diagnostics::CyclicDefinition;
use shackle_ty::match_fn;
use shackle_utils::hash::{Map, Set};

use super::PatternTy;
use crate::{
	ClassMember, Db, Expression, GlobalScope, Goal, Item, Pattern, Type,
	class_analysis::class_subclasses,
	diagnostics::Errors,
	ids::{ExpressionRef, NodeRef, PatternRef},
	lower::lower_models,
};

/// Topologically sort items
#[salsa::tracked(returns(ref))]
pub fn topological_sort<'db>(db: &'db dyn Db) -> Vec<Item<'db>> {
	log::info!("Topologically sorting items");
	let models = lower_models(db);
	let mut items = Vec::with_capacity(models.iter().map(|m| m.items(db).len()).sum());
	let mut assignments = Map::default();
	for m in models.iter() {
		for item in m.items(db) {
			match item {
				Item::Assignment(a) => {
					let types = item.types(db);
					if let Some(p) = types.name_resolution(a.assignment(db).assignee) {
						let _ = assignments.entry(p.item(db)).or_insert(*item);
					}
				}
				Item::EnumAssignment(a) => {
					let types = item.types(db);
					if let Some(p) = types.name_resolution(a.enum_assignment(db).assignee) {
						let _ = assignments.entry(p.item(db)).or_insert(*item);
					}
				}
				_ => (),
			}
			items.push(*item);
		}
	}
	let mut topo_sorter = TopoSorter::new(db, assignments);
	for item in items.iter() {
		topo_sorter.run(*item);
	}

	topo_sorter.finish()
}

/// Topological sorter
#[derive(Debug)]
pub struct TopoSorter<'db> {
	db: &'db dyn Db,
	sorted: Vec<Item<'db>>,
	visited: Set<Item<'db>>,
	current: Set<PatternRef<'db>>,
	assignments: Map<Item<'db>, Item<'db>>,
}

impl<'db> TopoSorter<'db> {
	/// Create a new topological sorter
	pub fn new(db: &'db dyn Db, assignments: Map<Item<'db>, Item<'db>>) -> Self {
		Self {
			db,
			sorted: Vec::new(),
			visited: Set::default(),
			current: Set::default(),
			assignments,
		}
	}

	/// Run the topological sorter on an item
	pub fn run(&mut self, item: Item<'db>) {
		if !self.visited.insert(item) {
			return;
		}

		match item {
			Item::Annotation(a) => {
				let annotation = a.annotation(self.db);
				for p in annotation.parameters() {
					for e in Type::expressions(p.declared_type, annotation.data()) {
						self.visit_type_expression(ExpressionRef::new(self.db, item, e));
					}
				}
			}
			Item::Assignment(a) => {
				let assignment = a.assignment(self.db);
				let types = item.types(self.db);
				if let Some(p) = types.name_resolution(assignment.assignee) {
					self.run(p.item(self.db));
					let _ = self.current.insert(p);
					self.visit_expression(
						ExpressionRef::new(self.db, item, assignment.definition),
						None,
					);
					let _ = self.current.remove(&p);
				}
			}
			Item::Constraint(c) => {
				let constraint = c.constraint(self.db);
				for ann in constraint.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				self.visit_expression(
					ExpressionRef::new(self.db, item, constraint.expression),
					None,
				);
			}
			Item::Declaration(d) => {
				let declaration = d.declaration(self.db);
				let pats = Pattern::identifiers(declaration.pattern, declaration.data())
					.map(|p| PatternRef::new(self.db, item, p))
					.collect::<Vec<_>>();
				self.current.extend(pats.iter().copied());
				for e in Type::expressions(declaration.declared_type, declaration.data()) {
					self.visit_type_expression(ExpressionRef::new(self.db, item, e));
				}
				for ann in declaration.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				if let Some(def) = declaration.definition {
					self.visit_expression(ExpressionRef::new(self.db, item, def), None);
				} else if let Some(asg) = self.assignments.remove(&item) {
					match asg {
						Item::Assignment(a) => self.visit_expression(
							ExpressionRef::new(self.db, asg, a.assignment(self.db).definition),
							None,
						),
						_ => unreachable!(),
					}
				}

				for p in pats.iter() {
					let _ = self.current.remove(p);
				}
			}
			Item::Enumeration(e) => {
				let enumeration = e.enumeration(self.db);
				let p = PatternRef::new(self.db, item, enumeration.pattern);
				let _ = self.current.insert(p);
				for ann in enumeration.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				if let Some(def) = &enumeration.definition {
					let data = enumeration.data();
					for c in def.iter() {
						for param in c.parameters() {
							for e in Type::expressions(param.declared_type, data) {
								self.visit_type_expression(ExpressionRef::new(self.db, item, e));
							}
						}
					}
				} else if let Some(asg) = self.assignments.remove(&item) {
					match asg {
						Item::EnumAssignment(e) => {
							let assignment = e.enum_assignment(self.db);
							let data = assignment.data();
							for c in assignment.definition.iter() {
								for param in c.parameters() {
									for e in Type::expressions(param.declared_type, data) {
										self.visit_type_expression(ExpressionRef::new(
											self.db, item, e,
										));
									}
								}
							}
						}
						_ => unreachable!(),
					}
				}
				let _ = self.current.remove(&p);
			}
			Item::EnumAssignment(e) => {
				let enum_assignment = e.enum_assignment(self.db);
				let types = item.types(self.db);
				if let Some(p) = types.name_resolution(enum_assignment.assignee) {
					self.run(p.item(self.db));
					let _ = self.current.insert(p);
					let data = enum_assignment.data();
					for c in enum_assignment.definition.iter() {
						for param in c.parameters() {
							for e in Type::expressions(param.declared_type, data) {
								self.visit_type_expression(ExpressionRef::new(self.db, item, e));
							}
						}
					}
					let _ = self.current.remove(&p);
				}
			}
			Item::Function(f) => {
				let function = f.function(self.db);
				let name = function[function.pattern].identifier().unwrap();
				let mut overloads = Vec::new();
				let ps = GlobalScope::find_function(self.db, name);
				for p in ps.iter() {
					let signature = p.item(self.db).signature(self.db);
					match &signature.patterns[&p.pattern(self.db)] {
						PatternTy::Function(f)
						| PatternTy::AnnotationConstructor(f)
						| PatternTy::AnnotationDestructure(f) => {
							overloads.push((p.item(self.db) == item, *f.clone()));
						}
						PatternTy::EnumConstructor(ec) => {
							overloads.extend(
								ec.iter()
									.map(|f| (p.item(self.db) == item, f.constructor.clone())),
							);
						}
						PatternTy::EnumDestructure(fs) => {
							overloads
								.extend(fs.iter().map(|f| (p.item(self.db) == item, f.clone())));
						}
						_ => unreachable!(),
					}
				}
				let p = PatternRef::new(self.db, item, function.pattern);
				let types = item.signature(self.db);
				match &types.patterns[&p.pattern(self.db)] {
					PatternTy::Function(f) => {
						let res = match_fn(self.db, overloads, f.overload.params())
							.unwrap_or_else(|e| unreachable!("Unexpected error: {:#?}", e));
						if !res.function.0 {
							// Ignore this function since it has been subsumed by another
							return;
						}
					}
					_ => unreachable!(),
				}

				let _ = self.current.insert(p);
				let data = function.data();
				for p in function.parameters.iter() {
					for ann in p.annotations.iter() {
						self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
					}
					for e in Type::expressions(p.declared_type, data) {
						self.visit_type_expression(ExpressionRef::new(self.db, item, e));
					}
				}
				for e in Type::expressions(function.return_type, data) {
					self.visit_type_expression(ExpressionRef::new(self.db, item, e));
				}
				for ann in function.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				if let Some(body) = function.body {
					// Inside this expression, don't visit function items for calls, instead visit the body since we
					// only care about globals and recursive functions are allowed.
					self.visit_expression(ExpressionRef::new(self.db, item, body), Some(p));
				}
				let _ = self.current.remove(&p);
			}
			Item::Output(o) => {
				let output = o.output(self.db);
				if let Some(s) = output.section {
					self.visit_expression(ExpressionRef::new(self.db, item, s), None);
				}
				self.visit_expression(ExpressionRef::new(self.db, item, output.expression), None);
			}
			Item::Solve(s) => {
				let solve = s.solve(self.db);
				match solve.goal {
					Goal::Maximize { pattern, objective }
					| Goal::Minimize {
						pattern, objective, ..
					} => {
						let p = PatternRef::new(self.db, item, pattern);
						let _ = self.current.insert(p);
						for ann in solve.annotations.iter() {
							self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
						}
						self.visit_expression(ExpressionRef::new(self.db, item, objective), None);
						let _ = self.current.remove(&p);
					}
					_ => {
						for ann in solve.annotations.iter() {
							self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
						}
					}
				}
			}
			Item::TypeAlias(t) => {
				let type_alias = t.type_alias(self.db);
				let p = PatternRef::new(self.db, item, type_alias.name);
				let _ = self.current.insert(p);
				for ann in type_alias.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				let data = type_alias.data();
				for e in Type::expressions(type_alias.aliased_type, data) {
					self.visit_type_expression(ExpressionRef::new(self.db, item, e));
				}
				let _ = self.current.remove(&p);
			}
			Item::Class(c) => {
				let class = c.class(self.db);
				let data = class.data();
				let p = PatternRef::new(self.db, item, class.pattern);
				let _ = self.current.insert(p);
				// A subclass's signature contributes to its superclass's lowering, so
				// sort subclasses before the class they extend.
				if let Some(subclasses) = class_subclasses(self.db).get(&p) {
					for subclass in subclasses.clone() {
						self.run(subclass.item(self.db));
					}
				}
				for ann in class.annotations.iter() {
					self.visit_expression(ExpressionRef::new(self.db, item, *ann), None);
				}
				for member in class.items.iter() {
					match member {
						ClassMember::Declaration(d) => {
							let pats = Pattern::identifiers(d.pattern, data)
								.map(|p| PatternRef::new(self.db, item, p))
								.collect::<Vec<_>>();
							self.current.extend(pats.iter().copied());
							for e in Type::expressions(d.declared_type, data) {
								self.visit_type_expression(ExpressionRef::new(self.db, item, e));
							}
							for ann in d.annotations.iter() {
								self.visit_expression(
									ExpressionRef::new(self.db, item, *ann),
									None,
								);
							}
							if let Some(def) = d.definition {
								self.visit_expression(ExpressionRef::new(self.db, item, def), None);
							}
							for p in pats.iter() {
								let _ = self.current.remove(p);
							}
						}
						ClassMember::Constraint(constraint) => {
							// A class constraint may refer to the class's own objects
							// (through `this`), which is not a cyclic definition.
							let _ = self.current.remove(&p);
							for ann in constraint.annotations.iter() {
								self.visit_expression(
									ExpressionRef::new(self.db, item, *ann),
									None,
								);
							}
							self.visit_expression(
								ExpressionRef::new(self.db, item, constraint.expression),
								None,
							);
							let _ = self.current.insert(p);
						}
					}
				}
				let _ = self.current.remove(&p);
			}
		}
		self.sorted.push(item);
	}

	fn visit_expression(
		&mut self,
		expression: ExpressionRef<'db>,
		visit_call_body: Option<PatternRef<'db>>,
	) {
		self.visit_expression_inner(expression, visit_call_body, false)
	}

	/// Visit an expression appearing inside a declared type.
	///
	/// A class named in a type position is not a cyclic definition even while that
	/// class is being defined: classes may refer to one another through their
	/// attribute types, and the reference is to the class's set of objects rather
	/// than to a value being defined.
	fn visit_type_expression(&mut self, expression: ExpressionRef<'db>) {
		self.visit_expression_inner(expression, None, true)
	}

	fn visit_expression_inner(
		&mut self,
		expression: ExpressionRef<'db>,
		visit_call_body: Option<PatternRef<'db>>,
		in_type_expression: bool,
	) {
		let mut todo = vec![expression];
		let mut seen = visit_call_body.into_iter().collect::<Set<_>>();
		while let Some(expression) = todo.pop() {
			let item = expression.item(self.db);
			let data = item.data(self.db);
			let types = item.types(self.db);
			for e in Expression::walk(expression.expression(self.db), data) {
				if let Expression::Identifier(i) = data[e]
					&& let Some(p) = types.name_resolution(e)
				{
					// A class name is the only identifier resolving to a class item's
					// own name pattern, so this needs no signature lookup.
					let is_class_type_reference = in_type_expression
						&& matches!(
							p.item(self.db),
							Item::Class(c) if c.class(self.db).pattern == p.pattern(self.db)
						);
					if (visit_call_body.is_none() || !seen.contains(&p))
						&& self.current.contains(&p)
						&& !is_class_type_reference
					{
						// Cyclic definition, emit error
						let (src, span) = NodeRef::from(
							ExpressionRef::new(self.db, item, e).into_entity(self.db),
						)
						.source_span(self.db);
						let variable = i.pretty_print(self.db);
						Errors::add(
							self.db,
							CyclicDefinition {
								src,
								span,
								variable,
							},
						);
						continue;
					}
					let it = p.item(self.db);
					if visit_call_body.is_some()
						&& let Item::Function(f) = it
					{
						if !seen.contains(&p)
							&& let Some(body) = f.function(self.db).body
						{
							let _ = seen.insert(p);
							todo.push(ExpressionRef::new(self.db, it, body));
						}
						continue;
					}
					if it != item {
						self.run(it);
					}
				}
			}
		}
	}

	/// Get results of topological sorting
	pub fn finish(self) -> Vec<Item<'db>> {
		self.sorted
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
		toposort::topological_sort,
	};

	fn check_toposort(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		let items = topological_sort(&db);
		let mut actual = String::new();
		for item in items {
			let origin = item.origin(&db);
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

		check_toposort(
			r#"
			int: x;
			x = y;
			int: y;
		"#,
			expect!([r#"
    int: y;
    int: x;
    x = y;
"#]),
		);

		check_toposort(
			r#"
			x = y;
			int: x;
			int: y;
		"#,
			expect!([r#"
    int: y;
    int: x;
    x = y;
"#]),
		);
	}

	#[test]
	fn test_class_topological_sort() {
		// Classes may be named in each other's attribute types before they are
		// declared, and a subclass is sorted before the class it extends.
		check_toposort(
			r#"
			class C (set of new A: as);
			var new A: a1;
			var new B: b2;
			class A (int: x);
			class B extends A(int: y);

			var new B: b1;
		"#,
			expect!([r#"
    class B extends A(int: y);
    class A (int: x);
    class C (set of new A: as);
    var new A: a1;
    var new B: b2;
    var new B: b1;
"#]),
		);
	}
}
