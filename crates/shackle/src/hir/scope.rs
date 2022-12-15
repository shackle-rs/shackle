//! Scope collection.
//!
//! Determines what identifiers are in scope for each expression in an item.
//! This happens before type-checking, so we can't resolve overloading or field access yet.

use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
	arena::{Arena, ArenaIndex, ArenaMap},
	diagnostics::{IdentifierAlreadyDefined, IdentifierShadowing, InvalidPattern},
	hir::{
		db::Hir,
		ids::{EntityRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		Expression, Goal, Identifier, ItemData, LetItem, Pattern, Type,
	},
	Error, Result, Warning,
};

use super::{Constructor, EnumConstructor, Generator};

/// Gets all variables in global scope.
///
/// - Checks for multiply defined identifiers
pub fn collect_global_scope(db: &dyn Hir) -> (Arc<ScopeData>, Arc<Vec<Error>>) {
	let mut scope = ScopeData::default();
	let mut diagnostics = Vec::new();
	let mut had_solve_item = false;
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for (i, a) in model.annotations.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			match &a.constructor {
				Constructor::Atom { pattern } => {
					let identifier = a.data[*pattern]
						.identifier()
						.expect("Annotation item must have identifier pattern");
					if let Err(e) =
						scope.add_variable(db, identifier, 0, PatternRef::new(item_ref, *pattern))
					{
						diagnostics.push(e);
					} else {
						scope.atoms.insert(identifier);
					}
				}
				Constructor::Function {
					constructor,
					destructor,
					..
				} => {
					let ctor_ident = a.data[*constructor]
						.identifier()
						.expect("Annotation item must have identifier pattern");
					let dtor_ident = a.data[*destructor]
						.identifier()
						.expect("Annotation item must have identifier pattern");
					if let Err(e) = scope.add_function(
						db,
						ctor_ident,
						0,
						PatternRef::new(item_ref, *constructor),
					) {
						diagnostics.push(e);
					}
					if let Err(e) = scope.add_function(
						db,
						dtor_ident,
						0,
						PatternRef::new(item_ref, *destructor),
					) {
						diagnostics.push(e);
					}
				}
			}
		}
		for (i, d) in model.declarations.iter() {
			scope.add_irrefutable_pattern(
				db,
				d.pattern,
				0,
				&d.data,
				ItemRef::new(db, *m, i),
				&mut diagnostics,
			);
		}

		let process_enum_constructor = |scope: &mut ScopeData,
		                                diagnostics: &mut Vec<Error>,
		                                item_ref: ItemRef,
		                                data: &ItemData,
		                                ec: &EnumConstructor| {
			if let EnumConstructor::Named(c) = ec {
				match c {
					Constructor::Atom { pattern } => {
						// Enum atom, so this is a variable
						let identifier = data[*pattern].identifier().unwrap();
						if let Err(e) = scope.add_variable(
							db,
							identifier,
							0,
							PatternRef::new(item_ref, *pattern),
						) {
							diagnostics.push(e);
						} else {
							scope.atoms.insert(identifier);
						}
					}
					Constructor::Function {
						constructor,
						destructor,
						..
					} => {
						// Enum constructor (overloads handled later in type checker)
						let ctor = data[*constructor].identifier().unwrap();
						if let Err(e) =
							scope.add_function(db, ctor, 0, PatternRef::new(item_ref, *constructor))
						{
							diagnostics.push(e);
						}
						let dtor = data[*destructor].identifier().unwrap();
						if let Err(e) =
							scope.add_function(db, dtor, 0, PatternRef::new(item_ref, *destructor))
						{
							diagnostics.push(e);
						}
					}
				}
			}
		};

		for (i, e) in model.enumerations.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			match &e.data[e.pattern] {
				Pattern::Identifier(identifier) => {
					if let Err(e) =
						scope.add_variable(db, *identifier, 0, PatternRef::new(item_ref, e.pattern))
					{
						diagnostics.push(e);
					}
				}
				_ => unreachable!("Enumeration must have identifier pattern"),
			}
			if let Some(d) = &e.definition {
				for ec in d.iter() {
					process_enum_constructor(&mut scope, &mut diagnostics, item_ref, &e.data, ec);
				}
			}
		}
		for (i, e) in model.enum_assignments.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			for ec in e.definition.iter() {
				process_enum_constructor(&mut scope, &mut diagnostics, item_ref, &e.data, ec)
			}
		}
		for (i, f) in model.functions.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			let identifier = &f.data[f.pattern]
				.identifier()
				.expect("Function must have identifier pattern");
			if let Err(e) =
				scope.add_function(db, *identifier, 0, PatternRef::new(item_ref, f.pattern))
			{
				diagnostics.push(e);
			}
		}
		for (i, s) in model.solves.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			// Ignore subsequent solve items (but emit error later)
			if !had_solve_item {
				had_solve_item = true;
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
		for (i, t) in model.type_aliases.iter() {
			match &t.data[t.name] {
				Pattern::Identifier(identifier) => {
					if let Err(e) = scope.add_variable(
						db,
						*identifier,
						0,
						PatternRef::new(ItemRef::new(db, *m, i), t.name),
					) {
						diagnostics.push(e);
					}
				}
				_ => unreachable!("Type-alias must have identifier pattern"),
			}
		}
	}
	(Arc::new(scope), Arc::new(diagnostics))
}

/// Variable scope
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScopeData {
	functions: FxHashMap<Identifier, Vec<(PatternRef, u32)>>,
	variables: FxHashMap<Identifier, (PatternRef, u32)>,
	/// Identifiers which do not cause pattern matching to add new variable bindings
	atoms: FxHashSet<Identifier>,
}

impl ScopeData {
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
					identifier: identifier.pretty_print(db),
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
			_ => {
				// Refutable pattern, can't be used
				let (src, span) = NodeRef::from(EntityRef::new(db, item, p)).source_span(db);
				errors.push(InvalidPattern {
					span,
					src,
					msg: "This pattern is not valid in this context as it may not match all cases.".to_owned()
				}.into());
			}
		}
	}

	/// Return whether this identifier is an atom in this scope
	pub fn is_atom(&self, identifier: Identifier, generation: u32) -> bool {
		self.find_variable(identifier, generation).is_some() && self.atoms.contains(&identifier)
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

	/// Get the variables in this scope
	pub fn variables(
		&self,
		generation: u32,
	) -> impl '_ + Iterator<Item = (Identifier, PatternRef)> {
		self.variables.iter().filter_map(move |(i, (p, g))| {
			if generation >= *g {
				Some((*i, *p))
			} else {
				None
			}
		})
	}

	/// Get the functions in this scope
	pub fn functions(
		&self,
		generation: u32,
	) -> impl '_ + Iterator<Item = (Identifier, Vec<PatternRef>)> {
		self.functions.iter().map(move |(i, ps)| {
			(
				*i,
				ps.iter()
					.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
					.collect(),
			)
		})
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PatternMode {
	/// Destructuring for declarations
	Destructuring,
	/// Case expressions
	Case,
	/// Comprehension generators
	Generator,
}

/// A collected scope entry
#[derive(Clone, Debug, PartialEq, Eq)]
enum Scope {
	/// Global scope
	Global,
	/// Scope inside this item
	Local {
		/// Parent scope
		parent: ArenaIndex<Scope>,
		/// Scope
		scope: ScopeData,
	},
}

/// Recursively collects local scopes in an item.
///
/// Produces a mapping between expressions and their scope.
struct ScopeCollector<'a> {
	db: &'a dyn Hir,
	item: ItemRef,
	data: &'a ItemData,
	scopes: Arena<Scope>,
	current: ArenaIndex<Scope>,
	generations: Vec<u32>,
	expression_scope: ArenaMap<Expression, (ArenaIndex<Scope>, u32)>,
	diagnostics: Vec<Error>,
	warnings: Vec<Warning>,
}

impl ScopeCollector<'_> {
	/// Create a new scope collector
	fn new<'a>(db: &'a dyn Hir, item: ItemRef, data: &'a ItemData) -> ScopeCollector<'a> {
		let mut scopes = Arena::new();
		let current = scopes.insert(Scope::Global);
		ScopeCollector {
			db,
			item,
			data,
			scopes,
			current,
			generations: vec![0],
			expression_scope: ArenaMap::new(),
			diagnostics: Vec::new(),
			warnings: Vec::new(),
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
	fn collect_pattern(&mut self, index: ArenaIndex<Pattern>, mode: PatternMode) {
		self.increment_generation();
		self.collect_pattern_inner(index, mode, false);
	}

	fn collect_pattern_inner(
		&mut self,
		index: ArenaIndex<Pattern>,
		mode: PatternMode,
		mut had_error: bool,
	) {
		let generation = self.generation();
		let mut refutable_pattern = || {
			// When destructuring, patterns must be irrefutable
			if let PatternMode::Destructuring = mode {
				if !had_error {
					let (src, span) = NodeRef::from(EntityRef::new(self.db, self.item, index))
						.source_span(self.db);
					self.diagnostics.push(
					InvalidPattern {
						span,
						src,
						msg: "This pattern is not valid in this context as it may not match all cases.".to_owned()
					}.into()
				);
				}
				had_error = true;
			}
		};

		let mut shadowed = |p: PatternRef| {
			let (src_orig, span_orig) = NodeRef::from(p.into_entity(self.db)).source_span(self.db);
			let (src_new, span_new) =
				NodeRef::from(PatternRef::new(self.item, index).into_entity(self.db))
					.source_span(self.db);
			if src_orig == src_new {
				// Same file, so warn about shadowing
				self.warnings.push(
					IdentifierShadowing {
						name: self.data[index].identifier().unwrap().pretty_print(self.db),
						src: src_new,
						span: span_new,
						original: span_orig,
					}
					.into(),
				);
			}
		};

		match &self.data[index] {
			Pattern::Identifier(i) => {
				let mut current = self.current;
				loop {
					match &self.scopes[current] {
						Scope::Local { parent, scope } => {
							if current == self.current {
								// Skip current scope
								current = *parent;
								continue;
							}
							if let PatternMode::Case = mode {
								if scope.is_atom(*i, generation) {
									// This identifier refers to this atom and does not create a new binding
									break;
								}
							}
							if let Some(p) = scope.find_variable(*i, generation) {
								shadowed(p);
							}
							current = *parent;
						}
						Scope::Global => {
							if let PatternMode::Case = mode {
								if self.db.lookup_global_atom(*i) {
									// This identifier refers to this atom and does not create a new binding
									break;
								}
							}
							if let Some(p) = self.db.lookup_global_variable(*i) {
								shadowed(p);
							}
							let scope = match self.scopes[self.current] {
								Scope::Local { ref mut scope, .. } => scope,
								_ => panic!("Cannot add to global scope"),
							};
							if let Err(e) = scope.add_variable(
								self.db,
								*i,
								generation,
								PatternRef::new(self.item, index),
							) {
								self.diagnostics.push(e);
							}
							break;
						}
					}
				}
			}
			Pattern::Call { arguments, .. } => {
				refutable_pattern();
				for argument in arguments.iter() {
					self.collect_pattern_inner(*argument, mode, had_error);
				}
			}
			Pattern::Tuple { fields } => {
				for field in fields.iter() {
					self.collect_pattern_inner(*field, mode, had_error);
				}
			}
			Pattern::Record { fields } => {
				for (_, pattern) in fields.iter() {
					self.collect_pattern_inner(*pattern, mode, had_error);
				}
			}
			_ => refutable_pattern(),
		}
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
				self.collect_expression(aa.indices);
			}
			Expression::ArrayComprehension(c) => {
				self.push();
				for generator in c.generators.iter() {
					self.collect_generator(generator);
				}
				if let Some(i) = c.indices {
					self.collect_expression(i);
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
							if let Some(def) = d.definition {
								self.collect_expression(def)
							}
							self.collect_pattern(d.pattern, PatternMode::Destructuring);
						}
					}
				}
				self.collect_expression(l.in_expression);
				self.pop();
			}
			Expression::SetComprehension(c) => {
				self.push();
				for generator in c.generators.iter() {
					self.collect_generator(generator);
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
					self.collect_pattern(i.pattern, PatternMode::Case);
					self.collect_expression(i.value);
					self.pop();
				}
			}
			Expression::Lambda(l) => {
				if let Some(r) = l.return_type {
					self.collect_type(r);
				}
				for param in l.parameters.iter() {
					for ann in param.annotations.iter() {
						self.collect_expression(*ann);
					}
					self.collect_type(param.declared_type);
				}
				self.push();
				for pattern in l.parameters.iter().filter_map(|param| param.pattern) {
					self.collect_pattern(pattern, PatternMode::Destructuring);
				}
				self.collect_expression(l.body);
				self.pop();
			}
			_ => (),
		}
		self.expression_scope
			.insert(index, (self.current, self.generation()));
	}

	fn collect_generator(&mut self, generator: &Generator) {
		match generator {
			Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				self.collect_expression(*collection);
				for p in patterns.iter() {
					self.collect_pattern(*p, PatternMode::Generator);
				}
				if let Some(e) = where_clause {
					self.collect_expression(*e)
				}
			}
			Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				self.collect_expression(*value);
				self.collect_pattern(*pattern, PatternMode::Destructuring);
				if let Some(e) = where_clause {
					self.collect_expression(*e)
				}
			}
		}
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
	fn finish(self) -> ScopeCollectorResult {
		ScopeCollectorResult {
			result: Arc::new(ScopeResult {
				scopes: self.scopes,
				expression_scopes: self.expression_scope,
			}),
			diagnostics: Arc::new(self.diagnostics),
			warnings: Arc::new(self.warnings),
		}
	}

	fn push(&mut self) {
		self.current = self.scopes.insert(Scope::Local {
			parent: self.current,
			scope: ScopeData::default(),
		});
		self.generations.push(self.generation());
	}

	fn pop(&mut self) {
		self.current = match self.scopes[self.current] {
			Scope::Local { parent, .. } => parent,
			_ => panic!("Cannot pop global scope"),
		};
		self.generations.pop().expect("No generation left");
	}
}

/// Result of collecting scopes for an item, including diagnostics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeCollectorResult {
	/// Collected scope
	pub result: Arc<ScopeResult>,
	/// Diagnostics
	pub diagnostics: Arc<Vec<Error>>,
	/// Warnings
	pub warnings: Arc<Vec<Warning>>,
}

/// Result of collecting scopes for an item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeResult {
	scopes: Arena<Scope>,
	expression_scopes: ArenaMap<Expression, (ArenaIndex<Scope>, u32)>,
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
				Scope::Local { parent, scope } => {
					for (k, v) in scope.functions.iter() {
						combined.entry(*k).or_insert_with(|| {
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect()
						});
					}
					current = *parent;
				}
				Scope::Global => {
					let scope = db.lookup_global_scope();
					for (k, v) in scope.functions.iter() {
						combined.entry(*k).or_insert_with(|| {
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect()
						});
					}
					return combined.into_iter().collect();
				}
			}
		}
	}

	/// Return the variable identifiers in scope for the given expression
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
				Scope::Local { parent, scope } => {
					for (k, (v, g)) in scope.variables.iter() {
						if generation >= *g {
							combined.entry(*k).or_insert(*v);
						}
					}
					current = *parent;
				}
				Scope::Global => {
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

	/// Find the given function in this expression's scope by its identifier.
	///
	/// Functions in inner scopes shadow ones from outer scopes (but can be overloaded in the same scope).
	pub fn find_function(
		&self,
		db: &dyn Hir,
		e: ArenaIndex<Expression>,
		i: Identifier,
	) -> Arc<Vec<PatternRef>> {
		let (mut current, generation) = self.expression_scopes[e];
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					let found = scope.find_function(i, generation);
					if !found.is_empty() {
						return Arc::new(found);
					}
					current = *parent;
				}
				Scope::Global => return db.lookup_global_function(i),
			}
		}
	}

	/// Find the given variable in this expression's scope by its identifier.
	pub fn find_variable(
		&self,
		db: &dyn Hir,
		e: ArenaIndex<Expression>,
		i: Identifier,
	) -> Option<PatternRef> {
		let (mut current, generation) = self.expression_scopes[e];
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					if let Some(p) = scope.find_variable(i, generation) {
						return Some(p);
					}
					current = *parent;
				}
				Scope::Global => {
					let scope = db.lookup_global_scope();
					return scope.find_variable(i, generation);
				}
			}
		}
	}
}

/// Generate a mapping between expressions and the identifiers in scope for the given item.
pub fn collect_item_scope(db: &dyn Hir, item: ItemRef) -> ScopeCollectorResult {
	let model = db.lookup_model(item.model_ref(db));
	match item.local_item_ref(db) {
		LocalItemRef::Annotation(a) => {
			let annotation = &model[a];
			let mut collector = ScopeCollector::new(db, item, annotation.data.as_ref());
			for p in annotation.parameters() {
				collector.collect_type(p.declared_type);
			}
			collector.finish()
		}
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
			collector.collect_type(declaration.declared_type);
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
					for p in c.parameters() {
						collector.collect_type(p.declared_type);
					}
				}
			}
			collector.finish()
		}
		LocalItemRef::EnumAssignment(e) => {
			let assignment = &model[e];
			let mut collector = ScopeCollector::new(db, item, assignment.data.as_ref());
			collector.collect_expression(assignment.assignee);
			for c in assignment.definition.iter() {
				for p in c.parameters() {
					collector.collect_type(p.declared_type);
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
				if !t.anonymous {
					collector.collect_pattern(t.name, PatternMode::Destructuring);
				}
			}
			for p in function.parameters.iter() {
				collector.collect_type(p.declared_type);
			}
			collector.collect_type(function.return_type);
			collector.push();
			for p in function.parameters.iter() {
				// Add parameters into scope
				if let Some(pat) = p.pattern {
					collector.collect_pattern(pat, PatternMode::Destructuring);
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
	}
}
