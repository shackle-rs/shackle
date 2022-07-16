//! Scope collection.
//!
//! Determines what identifiers are in scope for each expression in an item.

use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc};

use rustc_hash::FxHashMap;

use crate::{
	arena::{Arena, ArenaIndex, ArenaMap},
	error::{AdditionalSolveItem, IdentifierAlreadyDefined, InvalidPattern, MultipleSolveItems},
	hir::{
		db::Hir,
		ids::{EntityRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		Expression, Goal, Identifier, ItemData, LetItem, Pattern, Type,
	},
	Error, Result,
};

/// Gets all variables in global scope.
///
/// - Checks for multiply defined identifiers
/// - Checks for multiple solve items
pub fn collect_global_scope(db: &dyn Hir) -> (Arc<Scope>, Arc<Vec<Error>>) {
	let mut scope = Scope::new();
	let mut diagnostics = Vec::new();
	let mut solve_items = Vec::new();
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for item in model.items.iter() {
			let item_ref = ItemRef::new(db, *m, *item);
			match item {
				LocalItemRef::Declaration(d) => {
					let d = &model[*d];
					scope.add_irrefutable_pattern(
						db,
						d.pattern,
						0,
						&d.data,
						item_ref,
						&mut diagnostics,
					);
				}
				LocalItemRef::Enumeration(e) => {
					let e = &model[*e];
					match &e.data[e.pattern] {
						Pattern::Identifier(identifier) => {
							if let Err(e) = scope.add_variable(
								db,
								*identifier,
								0,
								PatternRef::new(item_ref, e.pattern),
							) {
								diagnostics.push(e);
							}
						}
						_ => unreachable!("Enumeration must have identifier pattern"),
					}
					if let Some(d) = &e.definition {
						for c in d.iter() {
							match &e.data[c.pattern] {
								Pattern::Identifier(identifier) => {
									let result = if c.parameters.is_empty() {
										// Enum atom, so this is a variable
										scope.add_variable(
											db,
											*identifier,
											0,
											PatternRef::new(item_ref, e.pattern),
										)
									} else {
										// Enum constructor
										scope.add_function(
											db,
											*identifier,
											0,
											PatternRef::new(item_ref, e.pattern),
										)
									};
									if let Err(e) = result {
										diagnostics.push(e);
									}
								}
								Pattern::Anonymous => (),
								_ => unreachable!("Enumeration case must have identifier pattern"),
							}
						}
					}
				}
				LocalItemRef::Function(f) => {
					let f = &model[*f];
					match &f.data[f.pattern] {
						Pattern::Identifier(identifier) => {
							if let Err(e) = scope.add_function(
								db,
								*identifier,
								0,
								PatternRef::new(item_ref, f.pattern),
							) {
								diagnostics.push(e);
							}
						}
						_ => unreachable!("Function must have identifier pattern"),
					}
				}
				LocalItemRef::Solve(s) => {
					solve_items.push(item_ref);
					// Ignore additional solve items
					if solve_items.len() == 1 {
						let s = &model[*s];
						match s.goal {
							Goal::Maximize { pattern, .. } | Goal::Minimize { pattern, .. } => {
								match &s.data[pattern] {
									Pattern::Identifier(identifier) => {
										if let Err(e) = scope.add_variable(
											db,
											*identifier,
											0,
											PatternRef::new(item_ref, pattern),
										) {
											diagnostics.push(e);
										}
									}
									_ => unreachable!("Function must have identifier pattern"),
								}
							}
							_ => (),
						}
					}
				}
				LocalItemRef::TypeAlias(t) => {
					let t = &model[*t];
					match &t.data[t.name] {
						Pattern::Identifier(identifier) => {
							if let Err(e) = scope.add_variable(
								db,
								*identifier,
								0,
								PatternRef::new(item_ref, t.name),
							) {
								diagnostics.push(e);
							}
						}
						_ => unreachable!("Type-alias must have identifier pattern"),
					}
				}
				_ => continue,
			}
		}
	}
	if solve_items.len() > 1 {
		let mut iter = solve_items.into_iter();
		let first = iter.next().unwrap();
		let (src, span) = NodeRef::from(first).source_span(db);
		diagnostics.push(
			MultipleSolveItems {
				src,
				span,
				others: iter
					.map(|i| {
						let (src, span) = NodeRef::from(i).source_span(db);
						AdditionalSolveItem { src, span }
					})
					.collect(),
			}
			.into(),
		);
	}
	(Arc::new(scope), Arc::new(diagnostics))
}

/// Variable scope
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
	functions: FxHashMap<Identifier, Vec<(PatternRef, u32)>>,
	variables: FxHashMap<Identifier, (PatternRef, u32)>,
}

impl Scope {
	/// Create a child scope
	pub fn new() -> Self {
		Self {
			functions: FxHashMap::default(),
			variables: FxHashMap::default(),
		}
	}

	/// Add a (possibly overloaded) function to the current scope
	pub fn add_function(
		&mut self,
		_db: &dyn Hir,
		identifier: Identifier,
		generation: u32,
		pattern: PatternRef,
	) -> Result<()> {
		match self.functions.entry(identifier) {
			Entry::Occupied(mut e) => {
				// Overloaded function
				e.get_mut().push((pattern, generation));
			}
			Entry::Vacant(e) => {
				e.insert(vec![(pattern, generation)]);
			}
		}
		Ok(())
	}

	/// Add a variable to the current scope
	pub fn add_variable(
		&mut self,
		db: &dyn Hir,
		identifier: Identifier,
		generation: u32,
		pattern: PatternRef,
	) -> Result<()> {
		match self.variables.entry(identifier) {
			Entry::Occupied(_) => {
				let (src, span) = NodeRef::from(pattern.into_entity(db)).source_span(db);
				Err(IdentifierAlreadyDefined {
					identifier: identifier.lookup(db),
					src,
					span,
				}
				.into())
			}
			Entry::Vacant(e) => {
				e.insert((pattern, generation));
				Ok(())
			}
		}
	}

	/// Adds identifiers from this irrefutable pattern into scope
	fn add_irrefutable_pattern(
		&mut self,
		db: &dyn Hir,
		p: ArenaIndex<Pattern>,
		generation: u32,
		data: &ItemData,
		item: ItemRef,
		errors: &mut Vec<Error>,
	) {
		match &data[p] {
			Pattern::Identifier(i) => {
				if let Err(e) = self.add_variable(db, *i, generation, PatternRef::new(item, p)) {
					errors.push(e);
				}
			}
			Pattern::Call { .. } => {
				// Refutable pattern, can't be used
				let (src, span) = NodeRef::from(EntityRef::new(db, item, p)).source_span(db);
				errors.push(InvalidPattern {
					span,
					src,
					msg: "This pattern is not valid in this context as it may not match all cases.".to_owned()
				}.into());
			}
			Pattern::Record { fields } => {
				for (_, pat) in fields.iter() {
					self.add_irrefutable_pattern(db, *pat, generation, data, item, errors);
				}
			}
			Pattern::Tuple { fields } => {
				for pat in fields.iter() {
					self.add_irrefutable_pattern(db, *pat, generation, data, item, errors);
				}
			}
			_ => (),
		}
	}

	/// Find the given variable identifier in this scope.
	pub fn find_variable(&self, identifier: Identifier, generation: u32) -> Option<PatternRef> {
		self.variables
			.get(&identifier)
			.and_then(|(p, g)| if generation >= *g { Some(*p) } else { None })
	}

	/// Find the given function identifier in this scope.
	pub fn find_function(&self, identifier: Identifier, generation: u32) -> Vec<PatternRef> {
		self.functions
			.get(&identifier)
			.iter()
			.flat_map(|r| {
				r.iter()
					.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
			})
			.collect()
	}
}

/// A collected scope entry
#[derive(Clone, Debug, PartialEq, Eq)]
enum ScopeEntry {
	/// Global scope
	Global,
	/// Scope inside this item
	Local {
		/// Parent scope
		parent: ArenaIndex<ScopeEntry>,
		/// Scope
		scope: Scope,
	},
}

/// Recursively collects local scopes in an item.
///
/// Produces a mapping between expressions and their scope.
struct ScopeCollector<'a> {
	db: &'a dyn Hir,
	item: ItemRef,
	data: &'a ItemData,
	scopes: Arena<ScopeEntry>,
	current: ArenaIndex<ScopeEntry>,
	generations: Vec<u32>,
	expression_scope: ArenaMap<Expression, (ArenaIndex<ScopeEntry>, u32)>,
	diagnostics: Vec<Error>,
}

impl ScopeCollector<'_> {
	/// Create a new scope collector
	fn new<'a>(db: &'a dyn Hir, item: ItemRef, data: &'a ItemData) -> ScopeCollector<'a> {
		let mut scopes = Arena::new();
		let current = scopes.insert(ScopeEntry::Global);
		ScopeCollector {
			db,
			item,
			data,
			scopes,
			current,
			generations: vec![0],
			expression_scope: ArenaMap::new(),
			diagnostics: Vec::new(),
		}
	}

	/// The 'generation' that we are currently at in the current scope.
	///
	/// This is used to ensure that identifiers are only accessible after they have been defined
	/// in the current scope.
	fn generation(&self) -> u32 {
		*self.generations.last().expect("No current generation")
	}

	/// Increment the generation (should happen on each before local declaration)
	fn increment_generation(&mut self) {
		*self.generations.last_mut().expect("No current generation") += 1;
	}

	/// Add leaves of a pattern into the current scope.
	///
	/// Visits the identifier leaves of the pattern, and adds them to the current scope if they
	/// are not already defined in a parent scope (so a new scope should be pushed before calling).
	fn collect_pattern(&mut self, index: ArenaIndex<Pattern>) {
		match &self.data[index] {
			Pattern::Identifier(i) => {
				// Add to scope only if not already in parent scope
				let mut current = self.current;
				loop {
					match &self.scopes[current] {
						ScopeEntry::Local { parent, scope } => {
							if current == self.current {
								// Skip current scope
								current = *parent;
								continue;
							}
							if let Some(_) = scope.find_variable(*i, self.generation()) {
								break;
							}
							current = *parent;
						}
						ScopeEntry::Global => {
							if let Some(_) = self.db.lookup_global_variable(*i) {
								break;
							}
							self.collect_irrefutable_pattern(index);
							break;
						}
					}
				}
			}
			Pattern::Call {
				function,
				arguments,
			} => {
				self.collect_expression(*function);
				for argument in arguments.iter() {
					self.collect_pattern(*argument);
				}
			}
			Pattern::Tuple { fields } => {
				for field in fields.iter() {
					self.collect_pattern(*field);
				}
			}
			Pattern::Record { fields } => {
				for (_, pattern) in fields.iter() {
					self.collect_pattern(*pattern);
				}
			}
			_ => (),
		}
	}

	/// Add leaves of an irrefutable pattern (used for declarations) into the current scope.
	fn collect_irrefutable_pattern(&mut self, pattern: ArenaIndex<Pattern>) {
		self.increment_generation();
		let g = self.generation();
		let scope = match self.scopes[self.current] {
			ScopeEntry::Local { ref mut scope, .. } => scope,
			_ => panic!("Cannot add to global scope"),
		};
		scope.add_irrefutable_pattern(
			self.db,
			pattern,
			g,
			&self.data,
			self.item,
			&mut self.diagnostics,
		);
	}

	/// Collect scope for an expression
	fn collect_expression(&mut self, index: ArenaIndex<Expression>) {
		let ann = self.data.annotations(index);
		for e in ann {
			self.collect_expression(e);
		}
		let e = &self.data[index];
		match e {
			Expression::ArrayAccess(aa) => {
				self.collect_expression(aa.collection);
				for e in aa.indices.iter() {
					self.collect_expression(*e);
				}
			}
			Expression::ArrayComprehension(c) => {
				self.push();
				for generator in c.generators.iter() {
					self.collect_expression(generator.collection);
					for p in generator.patterns.iter() {
						self.collect_pattern(*p);
					}
					match generator.where_clause {
						Some(e) => self.collect_expression(e),
						None => (),
					}
				}
				for i in c.indices.iter() {
					self.collect_expression(*i);
				}
				self.collect_expression(c.template);
				self.pop();
			}
			Expression::ArrayLiteral(al) => {
				for e in al.members.iter() {
					self.collect_expression(*e);
				}
			}
			Expression::Call(c) => {
				self.collect_expression(c.function);
				for arg in c.arguments.iter() {
					self.collect_expression(*arg);
				}
			}
			Expression::IfThenElse(ite) => {
				for branch in ite.branches.iter() {
					self.collect_expression(branch.condition);
					self.collect_expression(branch.result);
				}
				if let Some(e) = ite.else_result {
					self.collect_expression(e);
				}
			}
			Expression::Let(l) => {
				self.push();
				for let_item in l.items.iter() {
					match let_item {
						LetItem::Constraint(c) => {
							for e in c.annotations.iter() {
								self.collect_expression(*e);
							}
							self.collect_expression(c.expression);
						}
						LetItem::Declaration(d) => {
							for e in d.annotations.iter() {
								self.collect_expression(*e);
							}
							self.collect_type(d.declared_type);
							match d.definition {
								Some(def) => self.collect_expression(def),
								_ => (),
							}
							self.collect_irrefutable_pattern(d.pattern);
						}
					}
				}
				self.collect_expression(l.in_expression);
				self.pop();
			}
			Expression::SetComprehension(c) => {
				self.push();
				for generator in c.generators.iter() {
					self.collect_expression(generator.collection);
					for p in generator.patterns.iter() {
						self.collect_pattern(*p);
					}
					match generator.where_clause {
						Some(e) => self.collect_expression(e),
						None => (),
					}
				}
				self.collect_expression(c.template);
				self.pop();
			}
			Expression::SetLiteral(s) => {
				for e in s.members.iter() {
					self.collect_expression(*e);
				}
			}
			Expression::TupleLiteral(t) => {
				for f in t.fields.iter() {
					self.collect_expression(*f);
				}
			}
			Expression::RecordLiteral(r) => {
				for (_, f) in r.fields.iter() {
					self.collect_expression(*f);
				}
			}
			Expression::TupleAccess(t) => {
				self.collect_expression(t.tuple);
			}
			Expression::RecordAccess(r) => {
				self.collect_expression(r.record);
			}
			Expression::Case(c) => {
				self.collect_expression(c.expression);
				for i in c.cases.iter() {
					self.push();
					self.collect_pattern(i.pattern);
					self.collect_expression(i.value);
					self.pop();
				}
			}
			_ => (),
		}
		self.expression_scope
			.insert(index, (self.current, self.generation()));
	}

	/// Collect scope for a type
	fn collect_type(&mut self, index: ArenaIndex<Type>) {
		match &self.data[index] {
			Type::Bounded { domain, .. } => self.collect_expression(*domain),
			Type::Array {
				dimensions,
				element,
				..
			} => {
				self.collect_type(*dimensions);
				self.collect_type(*element);
			}
			Type::Set { element, .. } => self.collect_type(*element),
			Type::Tuple { fields, .. } => {
				for f in fields.iter() {
					self.collect_type(*f);
				}
			}
			Type::Record { fields, .. } => {
				for (_, f) in fields.iter() {
					self.collect_type(*f);
				}
			}
			Type::Operation {
				return_type,
				parameter_types,
				..
			} => {
				self.collect_type(*return_type);
				for p in parameter_types.iter() {
					self.collect_type(*p)
				}
			}
			_ => {}
		}
	}

	/// Get results
	fn finish(
		self,
	) -> (
		Arena<ScopeEntry>,
		ArenaMap<Expression, (ArenaIndex<ScopeEntry>, u32)>,
		Vec<Error>,
	) {
		(self.scopes, self.expression_scope, self.diagnostics)
	}

	fn push(&mut self) {
		self.current = self.scopes.insert(ScopeEntry::Local {
			parent: self.current,
			scope: Scope::new(),
		});
		self.generations.push(self.generation());
	}

	fn pop(&mut self) {
		self.current = match self.scopes[self.current] {
			ScopeEntry::Local { parent, .. } => parent,
			_ => panic!("Cannot pop global scope"),
		};
		self.generations.pop().expect("No generation left");
	}
}

/// Result of collecting scopes for an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeResult {
	scopes: Arena<ScopeEntry>,
	expression_scopes: ArenaMap<Expression, (ArenaIndex<ScopeEntry>, u32)>,
}

impl ScopeResult {
	/// Return the function identifiers in scope for the given expression
	///
	/// Used for code completion
	pub fn functions_in_scope(
		&self,
		db: &dyn Hir,
		e: ArenaIndex<Expression>,
	) -> Vec<(Identifier, Vec<PatternRef>)> {
		let (mut current, generation) = self.expression_scopes[e];
		let mut combined = FxHashMap::default();
		loop {
			match &self.scopes[current] {
				ScopeEntry::Local { parent, scope } => {
					for (k, v) in scope.functions.iter() {
						combined.entry(*k).or_insert(
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect(),
						);
					}
					current = *parent;
				}
				ScopeEntry::Global => {
					let scope = db.lookup_global_scope();
					for (k, v) in scope.functions.iter() {
						combined.entry(*k).or_insert(
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect(),
						);
					}
					return combined.into_iter().collect();
				}
			}
		}
	}

	/// Return the varaible identifiers in scope for the given expression
	///
	/// Used for code completion
	pub fn variables_in_scope(
		&self,
		db: &dyn Hir,
		e: ArenaIndex<Expression>,
	) -> Vec<(Identifier, PatternRef)> {
		let (mut current, generation) = self.expression_scopes[e];
		let mut combined = FxHashMap::default();
		loop {
			match &self.scopes[current] {
				ScopeEntry::Local { parent, scope } => {
					for (k, (v, g)) in scope.variables.iter() {
						if generation >= *g {
							combined.entry(*k).or_insert(*v);
						}
					}
					current = *parent;
				}
				ScopeEntry::Global => {
					let scope = db.lookup_global_scope();
					for (k, (v, g)) in scope.variables.iter() {
						if generation >= *g {
							combined.entry(*k).or_insert(*v);
						}
					}
					return combined.into_iter().collect();
				}
			}
		}
	}
}

/// Generate a mapping between expressions and the identifiers in scope for the given item.
pub fn collect_item_scope(db: &dyn Hir, item: ItemRef) -> (Arc<ScopeResult>, Arc<Vec<Error>>) {
	let model = db.lookup_model(item.model_ref(db));
	let (scopes, expression_scopes, diagnostics) = match item.local_item_ref(db) {
		LocalItemRef::Assignment(a) => {
			let assignment = &model[a];
			let mut collector = ScopeCollector::new(db, item, assignment.data.as_ref());
			collector.collect_expression(assignment.assignee);
			collector.collect_expression(assignment.definition);
			collector.finish()
		}
		LocalItemRef::Constraint(c) => {
			let constraint = &model[c];
			let mut collector = ScopeCollector::new(db, item, constraint.data.as_ref());
			for ann in constraint.annotations.iter() {
				collector.collect_expression(*ann);
			}
			collector.collect_expression(constraint.expression);
			collector.finish()
		}
		LocalItemRef::Declaration(d) => {
			let declaration = &model[d];
			let mut collector = ScopeCollector::new(db, item, declaration.data.as_ref());
			for ann in declaration.annotations.iter() {
				collector.collect_expression(*ann);
			}
			if let Some(e) = declaration.definition {
				collector.collect_expression(e);
			}
			collector.finish()
		}
		LocalItemRef::Enumeration(e) => {
			let enumeration = &model[e];
			let mut collector = ScopeCollector::new(db, item, enumeration.data.as_ref());
			for ann in enumeration.annotations.iter() {
				collector.collect_expression(*ann);
			}
			if let Some(ref d) = enumeration.definition {
				for c in d.iter() {
					for p in c.parameters.iter() {
						collector.collect_type(*p);
					}
				}
			}
			collector.finish()
		}
		LocalItemRef::Function(f) => {
			let function = &model[f];
			let mut collector = ScopeCollector::new(db, item, function.data.as_ref());
			for ann in function.annotations.iter() {
				collector.collect_expression(*ann);
			}
			for p in function.parameters.iter() {
				for ann in p.annotations.iter() {
					collector.collect_expression(*ann);
				}
			}
			collector.push();
			for t in function.type_inst_vars.iter() {
				collector.collect_irrefutable_pattern(t.name);
			}
			for p in function.parameters.iter() {
				collector.collect_type(p.declared_type);
			}
			collector.push();
			for p in function.parameters.iter() {
				// Add parameters into scope
				if let Some(pat) = p.pattern {
					collector.collect_irrefutable_pattern(pat);
				}
			}
			if let Some(e) = function.body {
				collector.collect_expression(e);
			}
			collector.pop();
			collector.pop();
			collector.finish()
		}
		LocalItemRef::Output(o) => {
			let output = &model[o];
			let mut collector = ScopeCollector::new(db, item, output.data.as_ref());
			collector.collect_expression(output.expression);
			collector.finish()
		}
		LocalItemRef::Solve(s) => {
			let solve = &model[s];
			let mut collector = ScopeCollector::new(db, item, solve.data.as_ref());
			for ann in solve.annotations.iter() {
				collector.collect_expression(*ann);
			}
			match solve.goal {
				Goal::Maximize { objective, .. } | Goal::Minimize { objective, .. } => {
					collector.collect_expression(objective)
				}
				_ => (),
			}
			collector.finish()
		}
		LocalItemRef::TypeAlias(t) => {
			let type_alias = &model[t];
			let mut collector = ScopeCollector::new(db, item, type_alias.data.as_ref());
			for ann in type_alias.annotations.iter() {
				collector.collect_expression(*ann);
			}
			collector.collect_type(type_alias.aliased_type);
			collector.finish()
		}
	};
	(
		Arc::new(ScopeResult {
			scopes,
			expression_scopes,
		}),
		Arc::new(diagnostics),
	)
}
