//! Cyclic definitions through the objects of a class.
//!
//! Which objects a `new` declaration creates only depends on its structure:
//! the owned children (the `new` attributes) in its initialiser. Its
//! references and attribute values are not part of it, so they may refer to
//! any object, including the ones the declaration creates itself
//! (`new Tree: t = (leaves: [(v: 1), (v: 2)], favourite: min(Leaf))`).
//!
//! A class's set of objects depends on the structure of every declaration that
//! creates objects of the class or of a subclass, and a declaration that
//! creates objects depends on the objects of every class it creates. The
//! structure is found through record and array literals, comprehensions,
//! if-then-else and let, and keeps a record's `set of new` and `opt new`
//! children, and its `new` children whose class has such children. Any other
//! expression is part of it as a whole. A cycle through a class's objects is a
//! cyclic definition, such as `new Tree: t = (leaves: [(v: i) | i in
//! 1..card(Leaf)])`.
//!
//! The THIR lowering sizes the objects from the same structure
//! (`<C>_<x>_structure`), so a model without such a cycle lowers to one
//! without a cyclic definition.

use shackle_diagnostics::CyclicDefinition;
use shackle_utils::hash::{Map, Set};

use crate::{
	Db, Expression, ExpressionId, Generator, Item, ItemData, LetItem, Type,
	class_analysis::{NewShape, OwnedChild, class_owned_children, resolve_class},
	diagnostics::Errors,
	ids::{ExpressionRef, PatternRef},
};

/// Report the declarations, among these items, whose structure depends on the
/// objects they create
pub(crate) fn check_object_cycles<'db>(db: &'db dyn Db, items: &[Item<'db>]) {
	let mut checker = Checker::new(db, items);
	for item in items {
		if !checker.roots.contains_key(item) {
			continue;
		}
		let Some(reference) = checker.find_cycle(*item) else {
			continue;
		};
		let (src, span) = reference.source_span(db);
		let item = reference.item(db);
		let Expression::Identifier(i) = item.data(db)[reference.expression(db)] else {
			unreachable!("a cycle is closed by an identifier")
		};
		Errors::add(
			db,
			CyclicDefinition {
				src,
				span,
				variable: i.pretty_print(db),
			},
		);
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Node<'db> {
	/// A top-level declaration or function
	Item(Item<'db>),
	/// The objects of a class
	Objects(PatternRef<'db>),
	/// The structure of a declaration that creates objects
	Structure(Item<'db>),
}

/// A top-level declaration that creates objects
struct Root<'db> {
	class: PatternRef<'db>,
	shape: NewShape,
	/// Its initialiser, which may be in an assignment item
	definition: Option<ExpressionRef<'db>>,
	/// The classes it creates objects of, superclasses included
	classes: Vec<PatternRef<'db>>,
}

/// A dependency, and the identifier it comes from
type Edge<'db> = (Node<'db>, Option<ExpressionRef<'db>>);

struct Checker<'db> {
	db: &'db dyn Db,
	/// Each class's superclass and owned children
	classes: Map<PatternRef<'db>, (Option<PatternRef<'db>>, Vec<OwnedChild<'db>>)>,
	roots: Map<Item<'db>, Root<'db>>,
	/// The declarations that create objects of each class
	creators: Map<PatternRef<'db>, Vec<Item<'db>>>,
	/// The definitions given by assignment items
	assigned: Map<Item<'db>, ExpressionRef<'db>>,
	edges: Map<Node<'db>, Vec<Edge<'db>>>,
}

impl<'db> Checker<'db> {
	fn new(db: &'db dyn Db, items: &[Item<'db>]) -> Self {
		let mut checker = Checker {
			db,
			classes: class_owned_children(db),
			roots: Map::default(),
			creators: Map::default(),
			assigned: Map::default(),
			edges: Map::default(),
		};
		for item in items {
			if let Item::Assignment(a) = item {
				let assignment = a.assignment(db);
				if let Some(p) = item.types(db).name_resolution(assignment.assignee) {
					let _ = checker
						.assigned
						.entry(p.item(db))
						.or_insert_with(|| ExpressionRef::new(db, *item, assignment.definition));
				}
			}
		}
		for item in items {
			let Item::Declaration(d) = item else {
				continue;
			};
			let declaration = d.declaration(db);
			let data = declaration.data();
			let Some(class) = data[declaration.declared_type]
				.get_new_class(data)
				.and_then(|domain| resolve_class(db, data, domain))
			else {
				continue;
			};
			let definition = declaration
				.definition
				.map(|e| ExpressionRef::new(db, *item, e))
				.or_else(|| checker.assigned.get(item).copied());
			let mut classes = Vec::new();
			checker.created_classes(class, &mut classes);
			for c in classes.iter() {
				checker.creators.entry(*c).or_default().push(*item);
			}
			let _ = checker.roots.insert(
				*item,
				Root {
					class,
					shape: NewShape::of(data, declaration.declared_type),
					definition,
					classes,
				},
			);
		}
		checker
	}

	/// The classes whose objects an object of `class` creates: it, its
	/// superclasses, and those of its owned children
	fn created_classes(&self, class: PatternRef<'db>, classes: &mut Vec<PatternRef<'db>>) {
		if classes.contains(&class) {
			return;
		}
		classes.push(class);
		let Some((superclass, children)) = self.classes.get(&class) else {
			return;
		};
		let mut ancestor = *superclass;
		while let Some(c) = ancestor.filter(|c| !classes.contains(c)) {
			classes.push(c);
			ancestor = self.classes.get(&c).and_then(|(s, _)| *s);
		}
		for child in children {
			self.created_classes(child.class, classes);
		}
	}

	/// The owned children of `class` that its structure keeps
	fn structure_children(&self, class: PatternRef<'db>) -> Vec<&OwnedChild<'db>> {
		self.structure_children_visiting(class, &mut Set::default())
	}

	fn structure_children_visiting(
		&self,
		class: PatternRef<'db>,
		visiting: &mut Set<PatternRef<'db>>,
	) -> Vec<&OwnedChild<'db>> {
		if !visiting.insert(class) {
			return Vec::new();
		}
		let children = self
			.classes
			.get(&class)
			.map(|(_, children)| children.as_slice())
			.unwrap_or(&[])
			.iter()
			.filter(|child| {
				child.shape != NewShape::Singular
					|| !self
						.structure_children_visiting(child.class, visiting)
						.is_empty()
			})
			.collect();
		let _ = visiting.remove(&class);
		children
	}

	/// A dependency of the structure of `root` on the objects of a class it
	/// creates: the identifier that closes the cycle
	fn find_cycle(&mut self, root: Item<'db>) -> Option<ExpressionRef<'db>> {
		let start = Node::Structure(root);
		let targets = self.roots[&root]
			.classes
			.iter()
			.map(|c| Node::Objects(*c))
			.collect::<Vec<_>>();
		let mut visited = Set::default();
		let _ = visited.insert(start);
		let mut todo = vec![(start, None)];
		while let Some((node, reference)) = todo.pop() {
			for (next, edge_reference) in self.edges(node) {
				let reference = edge_reference.or(reference);
				if targets.contains(&next) {
					return Some(reference.expect("a cycle is closed by an identifier"));
				}
				if visited.insert(next) {
					todo.push((next, reference));
				}
			}
		}
		None
	}

	fn edges(&mut self, node: Node<'db>) -> Vec<Edge<'db>> {
		if let Some(edges) = self.edges.get(&node) {
			return edges.clone();
		}
		let db = self.db;
		let mut edges = Vec::new();
		match node {
			Node::Item(item) => {
				match item {
					Item::Declaration(d) => {
						let declaration = d.declaration(db);
						let data = declaration.data();
						// A root's type names the class it creates, which is
						// the edge to the objects it creates below
						if !self.roots.contains_key(&item) {
							for e in Type::expressions(declaration.declared_type, data) {
								self.references(item, e, &mut edges);
							}
						}
						if let Some(definition) = declaration
							.definition
							.map(|e| ExpressionRef::new(db, item, e))
							.or_else(|| self.assigned.get(&item).copied())
						{
							self.references(
								definition.item(db),
								definition.expression(db),
								&mut edges,
							);
						}
					}
					Item::Function(f) => {
						if let Some(body) = f.function(db).body {
							self.references(item, body, &mut edges);
						}
					}
					_ => (),
				}
				if let Some(root) = self.roots.get(&item) {
					edges.extend(root.classes.iter().map(|c| (Node::Objects(*c), None)));
				}
			}
			Node::Objects(class) => {
				edges.extend(
					self.creators
						.get(&class)
						.into_iter()
						.flatten()
						.map(|root| (Node::Structure(*root), None)),
				);
			}
			Node::Structure(root) => {
				let root = &self.roots[&root];
				if let Some(definition) = root.definition {
					let item = definition.item(db);
					let mut expressions = Vec::new();
					self.structure(
						item.data(db),
						definition.expression(db),
						root.class,
						root.shape,
						&mut expressions,
					);
					for e in expressions {
						self.references(item, e, &mut edges);
					}
				}
			}
		}
		let _ = self.edges.insert(node, edges.clone());
		edges
	}

	/// The dependencies of the identifiers in `e`
	fn references(&self, item: Item<'db>, e: ExpressionId<'db>, edges: &mut Vec<Edge<'db>>) {
		let db = self.db;
		let data = item.data(db);
		let types = item.types(db);
		for e in Expression::walk(e, data) {
			if !matches!(data[e], Expression::Identifier(_)) {
				continue;
			}
			let Some(p) = types.name_resolution(e) else {
				continue;
			};
			let target = p.item(db);
			if target == item {
				// A local
				continue;
			}
			let node = match target {
				Item::Class(c) if c.class(db).pattern == p.pattern(db) => Node::Objects(p),
				Item::Declaration(_) | Item::Function(_) => Node::Item(target),
				_ => continue,
			};
			edges.push((node, Some(ExpressionRef::new(db, item, e))));
		}
	}

	/// Collect the expressions that are part of the structure of `e`, whose
	/// value introduces objects of `class` as `shape` says
	fn structure(
		&self,
		data: &ItemData<'db>,
		e: ExpressionId<'db>,
		class: PatternRef<'db>,
		shape: NewShape,
		expressions: &mut Vec<ExpressionId<'db>>,
	) {
		match (&data[e], shape) {
			(Expression::IfThenElse(ite), _) => {
				for branch in ite.branches.iter() {
					expressions.push(branch.condition);
					self.structure(data, branch.result, class, shape, expressions);
				}
				if let Some(e) = ite.else_result {
					self.structure(data, e, class, shape, expressions);
				}
			}
			(Expression::Let(l), _) => {
				for item in l.items.iter() {
					match item {
						LetItem::Declaration(d) => {
							expressions.extend(Type::expressions(d.declared_type, data));
							expressions.extend(d.definition);
						}
						LetItem::Constraint(c) => expressions.push(c.expression),
					}
				}
				self.structure(data, l.in_expression, class, shape, expressions);
			}
			(Expression::ArrayLiteral(al), NewShape::Collection) => {
				for member in al.members.iter() {
					self.structure(data, *member, class, NewShape::Singular, expressions);
				}
			}
			(Expression::ArrayComprehension(c), NewShape::Collection) => {
				for generator in c.generators.iter() {
					match generator {
						Generator::Iterator {
							collection: value,
							where_clause,
							..
						}
						| Generator::Assignment {
							value,
							where_clause,
							..
						} => {
							expressions.push(*value);
							expressions.extend(*where_clause);
						}
					}
				}
				expressions.extend(c.indices);
				self.structure(data, c.template, class, NewShape::Singular, expressions);
			}
			(Expression::Absent, NewShape::Opt) => (),
			(Expression::RecordLiteral(_), NewShape::Opt) => {
				self.structure(data, e, class, NewShape::Singular, expressions);
			}
			(_, NewShape::Singular) => {
				let children = self.structure_children(class);
				if children.is_empty() {
					return;
				}
				let Expression::RecordLiteral(rl) = &data[e] else {
					expressions.push(e);
					return;
				};
				for (field, value) in rl.fields.iter() {
					let name = data[*field].identifier();
					if let Some(child) = children.iter().find(|c| Some(c.attribute) == name) {
						self.structure(data, *value, child.class, child.shape, expressions);
					}
				}
			}
			_ => expressions.push(e),
		}
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_diagnostics::Error;
	use shackle_syntax::InputLang;

	use crate::{
		db::CompilerDatabase,
		input::{InlineModelFile, InputFiles},
		run_hir_phase,
	};

	/// The model's errors, each with the line and column it points at
	fn check_cycles(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		let errors = run_hir_phase(&db)
			.errors
			.iter()
			.map(|e| match e {
				Error::CyclicDefinition(c) => {
					let before = &c.src.contents()[..c.span.offset()];
					let line = before.lines().count();
					let column = before.len() - before.rfind('\n').map_or(0, |i| i + 1) + 1;
					format!("{e} at {line}:{column}")
				}
				_ => e.to_string(),
			})
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&errors);
	}

	#[test]
	fn test_references_into_same_data() {
		// References and attribute values aren't part of the structure
		check_cycles(
			r#"
			class Leaf (1..9: v);
			class Tree (set of new Leaf: leaves; new Leaf: main; Leaf: favourite);
			class Node (1..9: v; opt Node: next);
			set of new Leaf: other = [(v: 3)];
			new Tree: t = (
				leaves: [(v: card(Leaf)) | i in 1..2],
				main: if card(Leaf) > 1 then (v: 1) else (v: 2) endif,
				favourite: min(Leaf),
			);
			new Tree: u = (leaves: [], main: (v: 1), favourite: min(other));
			set of new Node: ns = [(v: i, next: if i = 1 then <> else min(Node) endif) | i in 1..3];
			"#,
			expect![""],
		);
	}

	#[test]
	fn test_number_of_objects_depends_on_them() {
		check_cycles(
			r#"
			class Leaf (1..9: v);
			class Tree (set of new Leaf: leaves);
			new Tree: t = (leaves: [(v: 1) | i in 1..card(Leaf)]);
			"#,
			expect!["Cyclic definition of Leaf at 4:50"],
		);
	}

	#[test]
	fn test_existence_depends_on_objects() {
		check_cycles(
			r#"
			class Leaf (1..9: v);
			class Tree (set of new Leaf: leaves; opt new Leaf: spare);
			class Node (1..9: v);
			new Tree: t = (leaves: [], spare: if card(Leaf) > 0 then (v: 1) else <> endif);
			set of new Node: ns = [(v: i) | i in 1..2 where card(Node) > 0];
			"#,
			expect![[r#"
    Cyclic definition of Leaf at 5:46
    Cyclic definition of Node at 6:57"#]],
		);
	}

	#[test]
	fn test_structure_through_declarations() {
		// A declaration in the structure is part of it as a whole, and so is
		// another root of a class it creates
		check_cycles(
			r#"
			class Leaf (1..9: v);
			class Tree (set of new Leaf: leaves; Leaf: favourite);
			record(array [int] of record(1..9: v): leaves, Leaf: favourite): data =
				(leaves: [(v: 1)], favourite: min(Leaf));
			new Tree: t = data;
			set of new Leaf: other = [(v: 3)];
			new Tree: u = (leaves: [(v: 1) | i in 1..card(other)], favourite: min(other));
			"#,
			expect![[r#"
    Cyclic definition of Leaf at 4:51
    Cyclic definition of other at 8:50"#]],
		);
	}
}
