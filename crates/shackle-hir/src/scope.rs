//! Scope collection.
//!
//! Determines what identifiers are in scope for each expression in an item.
//! This happens before type-checking, so we can't resolve overloading or field access yet.

use std::collections::hash_map::Entry;

use derive_more::Deref;
use shackle_diagnostics::{IdentifierAlreadyDefined, IdentifierShadowing, InvalidPattern};
use shackle_ty::FunctionEntry;
use shackle_utils::{
	InternedString,
	arena::{Arena, ArenaIndex, ArenaMap},
	hash::{Map, Set},
	maybe_grow_stack,
};

use super::{Constructor, Generator, MaybeIndexSet};
use crate::{
	AnnotationItem, AssignmentItem, ClassItem, ClassMember, ConstraintItem, Db, DeclarationItem,
	EnumAssignmentItem, EnumConstructor, EnumerationItem, Expression, ExpressionId, FunctionItem,
	Goal, Identifier, Item, ItemData, LetItem, Model, OutputItem, Pattern, PatternId, PatternTy,
	SolveItem, Type, TypeAliasItem, TypeId,
	constants::IdentifierRegistry,
	db::with_attached_database,
	diagnostics::Diagnostics,
	ids::{EntityId, NodeRef, PatternRef},
	lower::lower_models,
};

fn process_enum_constructor<'db>(
	db: &'db dyn Db,
	scope: &mut ScopeData<'db>,
	diagnostics: &mut Diagnostics,
	item: Item<'db>,
	data: &ItemData<'db>,
	ec: &EnumConstructor<'db>,
) {
	if let EnumConstructor::Named(c) = ec {
		match c {
			Constructor::Atom { pattern } => {
				// Enum atom, so this is a variable
				let identifier = data[*pattern].identifier().unwrap();
				scope.add_variable(
					db,
					identifier,
					0,
					PatternRef::new(db, item, *pattern),
					true,
					diagnostics,
				);
			}
			Constructor::Function {
				constructor,
				destructor,
				..
			} => {
				// Enum constructor (overloads handled later in type checker)
				let ctor = data[*constructor].identifier().unwrap();
				scope.add_function(db, ctor, 0, PatternRef::new(db, item, *constructor));
				let dtor = data[*destructor].identifier().unwrap();
				scope.add_function(db, dtor, 0, PatternRef::new(db, item, *destructor));
			}
		}
	}
}

/// Names in global scope
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalScope;

impl GlobalScope {
	fn collect<'db>(db: &'db dyn Db) -> &'db ScopeData<'db> {
		&collect_global_scope_with_diagnostics(db).0
	}

	/// Get whether there is an atom with the given name in global scope
	pub fn is_atom<'db>(db: &'db dyn Db, identifier: Identifier<'db>) -> bool {
		lookup_global_atom_internal(db, InternedString::from(identifier))
	}

	/// Resolve this variable identifier in global scope.
	pub fn find_variable<'db>(
		db: &'db dyn Db,
		identifier: Identifier<'db>,
	) -> Option<PatternRef<'db>> {
		lookup_global_variable_internal(db, InternedString::from(identifier))
	}

	/// Resolve this function identifier in global scope to retrieve the possible overloads.
	pub fn find_function<'db>(
		db: &'db dyn Db,
		identifier: Identifier<'db>,
	) -> &'db [PatternRef<'db>] {
		lookup_global_function_internal(db, InternedString::from(identifier))
	}

	/// Get the variables in global scope
	pub fn variables<'db>(
		db: &'db dyn Db,
	) -> impl Iterator<Item = (Identifier<'db>, PatternRef<'db>)> {
		GlobalScope::collect(db).variables(0)
	}

	/// Get the functions global scope
	pub fn functions<'db>(
		db: &'db dyn Db,
	) -> impl Iterator<Item = (Identifier<'db>, Vec<PatternRef<'db>>)> {
		GlobalScope::collect(db).functions(0)
	}
}

impl std::fmt::Debug for GlobalScope {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		with_attached_database(|db| {
			f.debug_struct("GlobalScope")
				.field("functions", &Map::from_iter(GlobalScope::functions(db)))
				.field("variables", &Map::from_iter(GlobalScope::variables(db)))
				.finish()
		})
		.unwrap_or_else(|| f.debug_struct("GlobalScope").finish())
	}
}

// Cache these lookups (requires salsa struct arg, so use `InternedString` rather than `Identifier`)

#[salsa::tracked]
fn lookup_global_atom_internal<'db>(db: &'db dyn Db, identifier: InternedString<'db>) -> bool {
	GlobalScope::collect(db).is_atom(identifier.into(), 0)
}

#[salsa::tracked]
fn lookup_global_variable_internal<'db>(
	db: &'db dyn Db,
	identifier: InternedString<'db>,
) -> Option<PatternRef<'db>> {
	GlobalScope::collect(db).find_variable(identifier.into(), 0)
}

#[salsa::tracked(returns(ref))]
fn lookup_global_function_internal<'db>(
	db: &'db dyn Db,
	identifier: InternedString<'db>,
) -> Vec<PatternRef<'db>> {
	GlobalScope::collect(db).find_function(identifier.into(), 0)
}

/// Force collection of all scopes (including global scope)
///
/// To access the results, use `GlobalScope` or `item.scope(db)` for each item.
#[salsa::tracked]
pub fn collect_scopes(db: &dyn Db) {
	let _ = GlobalScope::collect(db);
	let models = lower_models(db);
	for m in models.iter() {
		m.collect_scopes(db);
	}
}

/// Accumulate scope diagnostics for the whole program.
#[salsa::tracked]
pub fn accumulate_scope_diagnostics(db: &dyn Db) {
	let (_, diagnostics) = collect_global_scope_with_diagnostics(db);
	diagnostics.accumulate(db);
	for model in lower_models(db).iter() {
		for item in model.items(db) {
			collect_item_scope_diagnostics(db, *item).accumulate(db);
		}
	}
}

impl<'db> Model<'db> {
	/// Force collection of scopes for items in this model
	///
	/// To access the results, use `item.scope(db)` for each item.
	pub fn collect_scopes(&self, db: &'db dyn Db) {
		collect_scopes_for_model(db, *self)
	}

	/// Get the names this model places in global scope
	pub fn global_scope(&self, db: &'db dyn Db) -> &'db ScopeData<'db> {
		collect_model_global_scope(db, *self)
	}
}

#[salsa::tracked]
fn collect_scopes_for_model<'db>(db: &'db dyn Db, model: Model<'db>) {
	log::info!(
		"Computing scopes for expressions in model {}",
		model.file(db)
	);
	for item in model.items(db) {
		let _ = item.scope(db);
	}
}

#[salsa::tracked(returns(ref))]
fn collect_model_global_scope<'db>(db: &'db dyn Db, model: Model<'db>) -> ScopeData<'db> {
	log::info!("Computing names in scope for model {}", model.file(db));
	let mut scope = ScopeData::default();
	let mut diagnostics = Diagnostics::default();
	let mut had_solve_item = false;

	for item in model.items(db).iter() {
		match item {
			Item::Annotation(annotation_item) => {
				let a = annotation_item.annotation(db);
				match &a.constructor {
					Constructor::Atom { pattern } => {
						let identifier = a[*pattern]
							.identifier()
							.expect("Annotation item must have identifier pattern");
						scope.add_variable(
							db,
							identifier,
							0,
							PatternRef::new(db, *item, *pattern),
							true,
							&mut diagnostics,
						);
					}
					Constructor::Function {
						constructor,
						destructor,
						..
					} => {
						let ctor_ident = a[*constructor]
							.identifier()
							.expect("Annotation item must have identifier pattern");
						let dtor_ident = a[*destructor]
							.identifier()
							.expect("Annotation item must have identifier pattern");
						scope.add_function(
							db,
							ctor_ident,
							0,
							PatternRef::new(db, *item, *constructor),
						);
						scope.add_function(
							db,
							dtor_ident,
							0,
							PatternRef::new(db, *item, *destructor),
						);
					}
				}
			}
			Item::Declaration(declaration_item) => {
				let d = declaration_item.declaration(db);
				scope.add_irrefutable_pattern(db, d.pattern, 0, d.data(), *item, &mut diagnostics);
			}
			Item::Enumeration(enumeration_item) => {
				let e = enumeration_item.enumeration(db);
				match &e[e.pattern] {
					Pattern::Identifier(identifier) => {
						scope.add_variable(
							db,
							*identifier,
							0,
							PatternRef::new(db, *item, e.pattern),
							false,
							&mut diagnostics,
						);
					}
					_ => unreachable!("Enumeration must have identifier pattern"),
				}
				if let Some(d) = &e.definition {
					for ec in d.iter() {
						process_enum_constructor(
							db,
							&mut scope,
							&mut diagnostics,
							*item,
							e.data(),
							ec,
						);
					}
				}
			}
			Item::EnumAssignment(enum_assignment_item) => {
				let e = enum_assignment_item.enum_assignment(db);
				for ec in e.definition.iter() {
					process_enum_constructor(db, &mut scope, &mut diagnostics, *item, e.data(), ec)
				}
			}
			Item::Function(function_item) => {
				let f = function_item.function(db);
				let identifier = &f[f.pattern]
					.identifier()
					.expect("Function must have identifier pattern");
				scope.add_function(db, *identifier, 0, PatternRef::new(db, *item, f.pattern));
			}
			Item::Solve(solve_item) => {
				let s = solve_item.solve(db);
				// Ignore subsequent solve items (but emit error later)
				if !had_solve_item {
					had_solve_item = true;
					match s.goal {
						Goal::Maximize { pattern, .. } | Goal::Minimize { pattern, .. } => {
							match &s[pattern] {
								Pattern::Identifier(identifier) => {
									scope.add_variable(
										db,
										*identifier,
										0,
										PatternRef::new(db, *item, pattern),
										false,
										&mut diagnostics,
									);
								}
								_ => unreachable!("Function must have identifier pattern"),
							}
						}
						_ => (),
					}
				}
			}
			Item::TypeAlias(type_alias_item) => {
				let t = type_alias_item.type_alias(db);
				match &t[t.name] {
					Pattern::Identifier(identifier) => {
						scope.add_variable(
							db,
							*identifier,
							0,
							PatternRef::new(db, *item, t.name),
							false,
							&mut diagnostics,
						);
					}
					_ => unreachable!("Type-alias must have identifier pattern"),
				}
			}
			Item::Class(class_item) => {
				let c = class_item.class(db);
				match &c[c.pattern] {
					Pattern::Identifier(identifier) => {
						scope.add_variable(
							db,
							*identifier,
							0,
							PatternRef::new(db, *item, c.pattern),
							false,
							&mut diagnostics,
						);
					}
					_ => unreachable!("Class declaration must have identifier pattern"),
				}
			}
			Item::Assignment(_) | Item::Constraint(_) | Item::Output(_) => (),
		}
	}
	log::info!(
		"{} variables ({} atoms), {} functions added to global namespace",
		scope.variables.len(),
		scope.atoms.len(),
		scope.functions.len()
	);
	scope
}

/// Gets all variables in global scope.
///
/// - Checks for multiply defined identifiers
#[salsa::tracked(returns(ref))]
fn collect_global_scope_with_diagnostics<'db>(db: &'db dyn Db) -> (ScopeData<'db>, Diagnostics) {
	log::info!("Computing full global scope for program");
	let mut diagnostics = Diagnostics::default();
	let scope = ScopeData::from_iter(
		db,
		lower_models(db).iter().map(|model| model.global_scope(db)),
		&mut diagnostics,
	);
	log::info!(
		"{} variables ({} atoms), {} functions in global namespace",
		scope.variables.len(),
		scope.atoms.len(),
		scope.functions.len()
	);
	(scope, diagnostics)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum PatternMode {
	/// Must be irrefutable, atoms bind new variables
	Destructuring,
	/// Can be refutable, atoms bind new variables
	Generator,
	/// Can be refutable, atoms resolve to existing variables
	Case,
}

/// Variable scope
#[derive(Clone, Default, PartialEq, Eq, salsa::Update)]
pub struct ScopeData<'db> {
	functions: Map<Identifier<'db>, Vec<(PatternRef<'db>, u32)>>,
	function_names: Vec<Identifier<'db>>,
	variables: Map<Identifier<'db>, (PatternRef<'db>, u32)>,
	variable_names: Vec<Identifier<'db>>,
	/// Identifiers which do not cause pattern matching to add new variable bindings
	atoms: Set<Identifier<'db>>,
}

impl<'db> ScopeData<'db> {
	fn from_iter<T: IntoIterator<Item = &'db Self>>(
		db: &'db dyn Db,
		iter: T,
		diagnostics: &mut Diagnostics,
	) -> Self {
		let mut result = Self::default();
		for scope in iter {
			for i in scope.function_names.iter() {
				let fs = &scope.functions[i];
				match result.functions.entry(*i) {
					Entry::Vacant(e) => {
						let _ = e.insert(fs.clone());
						result.function_names.push(*i);
					}
					Entry::Occupied(mut e) => {
						e.get_mut().extend(fs.iter().copied());
					}
				}
			}
			for i in scope.variable_names.iter() {
				let (p, g) = scope.variables[i];
				result.add_variable(db, *i, g, p, false, diagnostics);
			}
			result.atoms.extend(scope.atoms.iter().copied());
		}
		result
	}

	/// Add a (possibly overloaded) function to the current scope
	pub fn add_function(
		&mut self,
		_db: &'db dyn Db,
		identifier: Identifier<'db>,
		generation: u32,
		pattern: PatternRef<'db>,
	) {
		match self.functions.entry(identifier) {
			Entry::Occupied(mut e) => {
				// Overloaded function
				e.get_mut().push((pattern, generation));
			}
			Entry::Vacant(e) => {
				let _ = e.insert(vec![(pattern, generation)]);
				self.function_names.push(identifier);
			}
		}
	}

	/// Add a variable to the current scope
	pub fn add_variable(
		&mut self,
		db: &'db dyn Db,
		identifier: Identifier<'db>,
		generation: u32,
		pattern: PatternRef<'db>,
		is_atom: bool,
		diagnostics: &mut Diagnostics,
	) {
		match self.variables.entry(identifier) {
			Entry::Occupied(_) => {
				let (src, span) = NodeRef::from(pattern.into_entity(db)).source_span(db);
				diagnostics.add_error(IdentifierAlreadyDefined {
					identifier: identifier.pretty_print(db),
					src,
					span,
				})
			}
			Entry::Vacant(e) => {
				let _ = e.insert((pattern, generation));
				if is_atom {
					let _ = self.atoms.insert(identifier);
				}
				self.variable_names.push(identifier);
			}
		}
	}

	/// Adds identifiers from this irrefutable pattern into scope
	fn add_irrefutable_pattern(
		&mut self,
		db: &'db dyn Db,
		p: PatternId<'db>,
		generation: u32,
		data: &ItemData<'db>,
		item: Item<'db>,
		diagnostics: &mut Diagnostics,
	) {
		match &data[p] {
			Pattern::Identifier(i) => {
				self.add_variable(
					db,
					*i,
					generation,
					PatternRef::new(db, item, p),
					false,
					diagnostics,
				);
			}
			Pattern::Record { fields } => {
				for (_, pat) in fields.iter() {
					self.add_irrefutable_pattern(db, *pat, generation, data, item, diagnostics);
				}
			}
			Pattern::Tuple { fields } => {
				for pat in fields.iter() {
					self.add_irrefutable_pattern(db, *pat, generation, data, item, diagnostics);
				}
			}
			_ => {
				// Refutable pattern, can't be used
				let (src, span) = item.sources(db)[p].source_span(db);
				diagnostics.add_error(InvalidPattern {
					span,
					src,
					msg: "This pattern is not valid in this context as it may not match all cases."
						.to_owned(),
				});
			}
		}
	}

	/// Return whether this identifier is an atom in this scope
	pub fn is_atom(&self, identifier: Identifier<'db>, generation: u32) -> bool {
		self.find_variable(identifier, generation).is_some() && self.atoms.contains(&identifier)
	}

	/// Find the given variable identifier in this scope.
	pub fn find_variable(
		&self,
		identifier: Identifier<'db>,
		generation: u32,
	) -> Option<PatternRef<'db>> {
		self.variables
			.get(&identifier)
			.and_then(|(p, g)| if generation >= *g { Some(*p) } else { None })
	}

	/// Find the given function identifier in this scope.
	pub fn find_function(
		&self,
		identifier: Identifier<'db>,
		generation: u32,
	) -> Vec<PatternRef<'db>> {
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
	) -> impl Iterator<Item = (Identifier<'db>, PatternRef<'db>)> {
		self.variable_names.iter().filter_map(move |i| {
			let (p, g) = self.variables[i];
			if generation >= g { Some((*i, p)) } else { None }
		})
	}

	/// Get the functions in this scope
	pub fn functions(
		&self,
		generation: u32,
	) -> impl Iterator<Item = (Identifier<'db>, Vec<PatternRef<'db>>)> {
		self.function_names.iter().map(move |i| {
			(
				*i,
				self.functions[i]
					.iter()
					.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
					.collect(),
			)
		})
	}
}

impl<'db> std::fmt::Debug for ScopeData<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ScopeData")
			.field("functions", &self.functions)
			.field("variables", &self.variables)
			.field("atoms", &self.atoms)
			.finish()
	}
}

/// A collected scope entry
#[allow(variant_size_differences, reason = "Size difference is expected")]
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
enum Scope<'db> {
	/// Global scope external to current model
	Global,
	/// Global scope but inside current model
	Model {
		/// Parent scope (global scope)
		global_scope: ArenaIndex<Scope<'db>>,
	},
	/// Scope inside this item
	Local {
		/// Parent scope
		parent: ArenaIndex<Scope<'db>>,
		/// Scope
		scope: ScopeData<'db>,
	},
}

/// Recursively collects local scopes in an item.
///
/// Produces a mapping between expressions and their scope.
struct ScopeCollector<'db> {
	db: &'db dyn Db,
	item: Item<'db>,
	data: &'db ItemData<'db>,
	scopes: Arena<Scope<'db>>,
	current: ArenaIndex<Scope<'db>>,
	generations: Vec<u32>,
	/// The scope for each expression (mapping to the index of the scope in `scopes` and the generation at which it was introduced)
	expression_scope: ArenaMap<Expression<'db>, (ArenaIndex<Scope<'db>>, u32)>,
	/// The scope which introduces this pattern
	pattern_scope: ArenaMap<Pattern<'db>, ArenaIndex<Scope<'db>>>,
	diagnostics: Diagnostics,
}

impl<'db> ScopeCollector<'db> {
	/// Create a new scope collector
	fn new(db: &'db dyn Db, item: Item<'db>, data: &'db ItemData<'db>) -> ScopeCollector<'db> {
		log::debug!(
			"Computing scopes for expressions in {:?}",
			item.get_item_with_data_as_debug(db)
		);
		let mut scopes = Arena::new();
		let global_scope = scopes.insert(Scope::Global);
		let current = scopes.insert(Scope::Model { global_scope });
		ScopeCollector {
			db,
			item,
			data,
			scopes,
			current,
			generations: vec![0],
			expression_scope: ArenaMap::new(),
			pattern_scope: ArenaMap::new(),
			diagnostics: Diagnostics::default(),
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
	fn collect_pattern(&mut self, index: PatternId<'db>, mode: PatternMode) {
		self.increment_generation();
		self.collect_pattern_inner(index, mode, false);
	}

	fn collect_pattern_inner(
		&mut self,
		index: PatternId<'db>,
		mode: PatternMode,
		mut had_error: bool,
	) {
		let generation = self.generation();

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
							if mode == PatternMode::Case && scope.is_atom(*i, generation) {
								// This identifier refers to this atom and does not create a new binding
								break;
							}
							if let Some(p) = scope.find_variable(*i, generation) {
								let (src_orig, span_orig) = p.item(self.db).sources(self.db)
									[p.pattern(self.db)]
								.source_span(self.db);
								let (src_new, span_new) =
									self.item.sources(self.db)[index].source_span(self.db);

								assert_eq!(
									src_orig, src_new,
									"Shadowing should only be reported within the same file"
								);

								self.diagnostics.add_warning(IdentifierShadowing {
									name: self.data[index]
										.identifier()
										.unwrap()
										.pretty_print(self.db),
									src: src_new,
									span: span_new,
									original: span_orig,
								});
							}
							current = *parent;
						}
						Scope::Model { global_scope } => {
							let scope = self.item.model(self.db).global_scope(self.db);
							if mode == PatternMode::Case && scope.is_atom(*i, 0) {
								// This identifier refers to this atom and does not create a new binding
								break;
							}
							if let Some(p) = scope.find_variable(*i, 0) {
								let (src_orig, span_orig) = p.item(self.db).sources(self.db)
									[p.pattern(self.db)]
								.source_span(self.db);
								let (src_new, span_new) =
									self.item.sources(self.db)[index].source_span(self.db);

								assert_eq!(
									src_orig, src_new,
									"Shadowing should only be reported within the same file"
								);

								self.diagnostics.add_warning(IdentifierShadowing {
									name: self.data[index]
										.identifier()
										.unwrap()
										.pretty_print(self.db),
									src: src_new,
									span: span_new,
									original: span_orig,
								});
							}
							current = *global_scope;
						}
						Scope::Global => {
							if mode == PatternMode::Case && GlobalScope::is_atom(self.db, *i) {
								// This identifier refers to this atom and does not create a new binding
								break;
							}
							let scope = match self.scopes[self.current] {
								Scope::Local { ref mut scope, .. } => scope,
								_ => panic!("Cannot add to global scope"),
							};
							scope.add_variable(
								self.db,
								*i,
								generation,
								PatternRef::new(self.db, self.item, index),
								false,
								&mut self.diagnostics,
							);
							self.pattern_scope.insert(index, self.current);
							break;
						}
					}
				}
			}
			Pattern::Call { arguments, .. } => {
				if mode == PatternMode::Destructuring {
					if !had_error {
						let (src, span) = self.item.sources(self.db)[index].source_span(self.db);
						self.diagnostics.add_error(InvalidPattern {
							span,
							src,
							msg: "This pattern is not valid in this context as it may not match all cases."
								.to_owned(),
						});
					}
					had_error = true;
				}
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
			_ => {
				if mode == PatternMode::Destructuring && !had_error {
					let (src, span) = self.item.sources(self.db)[index].source_span(self.db);
					self.diagnostics.add_error(InvalidPattern {
						span,
						src,
						msg: "This pattern is not valid in this context as it may not match all cases."
							.to_owned(),
					});
				}
			}
		}
	}

	/// Collect scope for an expression
	fn collect_expression(&mut self, index: ExpressionId<'db>) {
		maybe_grow_stack(|| self.collect_expression_inner(index))
	}

	fn collect_expression_inner(&mut self, index: ExpressionId<'db>) {
		let ann = self.data.annotations(index);
		for e in ann {
			self.collect_expression(e);
		}
		let e = &self.data[index];
		match e {
			Expression::Absent
			| Expression::BooleanLiteral(_)
			| Expression::FloatLiteral(_)
			| Expression::Identifier(_)
			| Expression::Infinity
			| Expression::IntegerLiteral(_)
			| Expression::Missing
			| Expression::Slice(_)
			| Expression::StringLiteral(_) => (),
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
			Expression::ArrayLiteral2D(al) => {
				if let MaybeIndexSet::Indexed(es) = &al.rows {
					for e in es.iter() {
						self.collect_expression(*e);
					}
				}
				if let MaybeIndexSet::Indexed(es) = &al.columns {
					for e in es.iter() {
						self.collect_expression(*e);
					}
				}
				for e in al.members.iter() {
					self.collect_expression(*e);
				}
			}
			Expression::IndexedArrayLiteral(al) => {
				for e in al.indices.iter() {
					self.collect_expression(*e);
				}
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
		}
		self.expression_scope
			.insert(index, (self.current, self.generation()));
	}

	fn collect_generator(&mut self, generator: &Generator<'db>) {
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
				self.collect_pattern(*pattern, PatternMode::Generator);
				if let Some(e) = where_clause {
					self.collect_expression(*e)
				}
			}
		}
	}

	/// Bring the attributes a class inherits from its superclass chain into the current scope
	fn add_inherited_class_fields_to_scope(&mut self, item: Item<'db>, extends: ExpressionId<'db>) {
		if let Some(base_class) = self.resolve_class_from_expression(item, extends) {
			self.add_class_fields_to_current_scope(base_class);
		}
	}

	/// Resolve an `extends` expression to the class item it names, if it names one
	fn resolve_class_from_expression(
		&self,
		item: Item<'db>,
		expr: ExpressionId<'db>,
	) -> Option<ClassItem<'db>> {
		let Expression::Identifier(identifier) = &item.data(self.db)[expr] else {
			return None;
		};
		let pattern = GlobalScope::find_variable(self.db, *identifier)?;
		match pattern.item(self.db) {
			Item::Class(c) => Some(c),
			_ => None,
		}
	}

	/// Add a class's attributes to the current scope, superclasses first so that an
	/// attribute redeclared in a subclass shadows the inherited one
	fn add_class_fields_to_current_scope(&mut self, class_item: ClassItem<'db>) {
		let db = self.db;
		let item: Item<'db> = class_item.into();
		let class = class_item.class(db);
		if let Some(base) = class.extends
			&& let Some(base_class) = self.resolve_class_from_expression(item, base)
		{
			self.add_class_fields_to_current_scope(base_class);
		}
		let generation = self.generation();
		let diagnostics = &mut self.diagnostics;
		let Scope::Local { scope, .. } = &mut self.scopes[self.current] else {
			unreachable!("Class fields must be added to a local scope")
		};
		for member in class.items.iter() {
			if let ClassMember::Declaration(d) = member {
				for pattern in Pattern::identifiers(d.pattern, class.data()) {
					let pattern_ref = PatternRef::new(db, item, pattern);
					let identifier = pattern_ref
						.identifier(db)
						.expect("Pattern::identifiers yields identifier patterns");
					if scope.find_variable(identifier, generation).is_none() {
						scope.add_variable(db, identifier, 0, pattern_ref, false, diagnostics);
					}
				}
			}
		}
	}

	/// Collect scope for a type
	fn collect_type(&mut self, index: TypeId<'db>) {
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
			Type::Set {
				element,
				cardinality,
				..
			} => {
				self.collect_type(*element);
				if let Some(c) = cardinality {
					self.collect_expression(*c);
				}
			}
			Type::New { domain, .. } => self.collect_expression(*domain),
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
	fn finish(self) -> (ScopeResult<'db>, Diagnostics) {
		(
			ScopeResult {
				model: self.item.model(self.db),
				scopes: self.scopes,
				expression_scopes: self.expression_scope,
				pattern_scopes: self.pattern_scope,
			},
			self.diagnostics,
		)
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
		let _ = self.generations.pop().expect("No generation left");
	}
}

/// Result of collecting scopes for an item
#[derive(Clone, PartialEq, Eq, salsa::Update)]
pub struct ScopeResult<'db> {
	model: Model<'db>,
	scopes: Arena<Scope<'db>>,
	expression_scopes: ArenaMap<Expression<'db>, (ArenaIndex<Scope<'db>>, u32)>,
	pattern_scopes: ArenaMap<Pattern<'db>, ArenaIndex<Scope<'db>>>,
}

impl<'db> ScopeResult<'db> {
	/// Return the function identifiers in scope for the given expression
	///
	/// Used for code completion
	pub fn functions_in_scope(
		&self,
		db: &'db dyn Db,
		e: ExpressionId<'db>,
	) -> Vec<(Identifier<'db>, Vec<PatternRef<'db>>)> {
		let (mut current, generation) = self.expression_scopes[e];
		let mut combined = Map::default();
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					for (k, v) in scope.functions.iter() {
						let _ = combined.entry(*k).or_insert_with(|| {
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect()
						});
					}
					current = *parent;
				}
				Scope::Model { global_scope } => {
					current = *global_scope;
				}
				Scope::Global => {
					let scope = GlobalScope::collect(db);
					for (k, v) in scope.functions.iter() {
						let _ = combined.entry(*k).or_insert_with(|| {
							v.iter()
								.filter_map(|(p, g)| if generation >= *g { Some(*p) } else { None })
								.collect()
						});
					}
					let mut result = combined.into_iter().collect::<Vec<_>>();
					result.sort_by_cached_key(|(i, _)| i.lookup(db));
					return result;
				}
			}
		}
	}

	/// Return the variable identifiers in scope for the given expression
	///
	/// Used for code completion
	pub fn variables_in_scope(
		&self,
		db: &'db dyn Db,
		e: ExpressionId<'db>,
	) -> Vec<(Identifier<'db>, PatternRef<'db>)> {
		let (mut current, generation) = self.expression_scopes[e];
		let mut combined = Map::default();
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					for (k, (v, g)) in scope.variables.iter() {
						if generation >= *g {
							let _ = combined.entry(*k).or_insert(*v);
						}
					}
					current = *parent;
				}
				Scope::Model { global_scope } => {
					current = *global_scope;
				}
				Scope::Global => {
					let scope = GlobalScope::collect(db);
					for (k, (v, g)) in scope.variables.iter() {
						if generation >= *g {
							let _ = combined.entry(*k).or_insert(*v);
						}
					}
					let mut result = combined.into_iter().collect::<Vec<_>>();
					result.sort_by_cached_key(|(i, _)| i.lookup(db));
					return result;
				}
			}
		}
	}

	/// Find the given function in this expression's scope by its identifier.
	///
	/// Functions in inner scopes shadow ones from outer scopes (but can be overloaded in the same scope).
	pub fn find_function(
		&self,
		db: &'db dyn Db,
		e: ExpressionId<'db>,
		i: Identifier<'db>,
	) -> Vec<PatternRef<'db>> {
		let (mut current, generation) = self.expression_scopes[e];
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					let found = scope.find_function(i, generation);
					if !found.is_empty() {
						return found;
					}
					current = *parent;
				}
				Scope::Model { global_scope } => {
					// We have to look through the entire global namespace for
					// functions since they can be overloaded across files
					current = *global_scope;
				}
				Scope::Global => return GlobalScope::find_function(db, i).to_owned(),
			}
		}
	}

	/// Find the given variable in this expression's scope by its identifier.
	pub fn find_variable(
		&self,
		db: &'db dyn Db,
		e: ExpressionId<'db>,
		i: Identifier<'db>,
	) -> Option<PatternRef<'db>> {
		let (mut current, generation) = self.expression_scopes[e];
		loop {
			match &self.scopes[current] {
				Scope::Local { parent, scope } => {
					if let Some(p) = scope.find_variable(i, generation) {
						return Some(p);
					}
					current = *parent;
				}
				Scope::Model { global_scope } => {
					let scope = self.model.global_scope(db);
					if let Some(p) = scope.find_variable(i, generation) {
						return Some(p);
					}
					current = *global_scope;
				}
				Scope::Global => {
					return GlobalScope::find_variable(db, i);
				}
			}
		}
	}
}

impl<'db> std::fmt::Debug for ScopeResult<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ScopeResult")
			.field("scopes", &self.scopes)
			.field("expression_scopes", &self.expression_scopes)
			.finish()
	}
}

/// The type of conflict that would be caused by renaming a pattern to a new identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameCheck {
	/// Rename is safe
	Ok,
	/// The new name is already defined in the same scope, so would cause an "identifier already defined" error
	IdentifierAlreadyDefined,
	/// The new name is used in a position where it would refer to the renamed pattern after the rename, which would change the meaning of the program
	ShadowConflict,
	/// The new name would cause invalid function overloading
	InvalidOverload,
}

impl RenameCheck {
	/// Check whether renaming the given pattern to the new identifier would cause a conflict
	///
	/// The pattern must be the defining occurrence of the identifier.
	pub fn check<'db>(
		db: &'db dyn Db,
		pattern: PatternRef<'db>,
		new_name: Identifier<'db>,
	) -> Self {
		if let PatternTy::Function(f) = &pattern.item(db).types(db)[pattern.pattern(db)] {
			let mut overloads = vec![((), (**f).clone())];
			for p in GlobalScope::find_function(db, new_name) {
				if let PatternTy::Function(f) = &p.item(db).types(db)[p.pattern(db)] {
					overloads.push(((), (**f).clone()));
				}
			}
			if FunctionEntry::check_overloading(db, overloads).is_empty() {
				return RenameCheck::Ok;
			} else {
				return RenameCheck::InvalidOverload;
			}
		}
		let old_name = pattern.identifier(db).unwrap();
		let item_scope = pattern.item(db).scope(db);
		let defining_scope = item_scope
			.pattern_scopes
			.get(pattern.pattern(db))
			.copied()
			.unwrap_or_else(|| item_scope.scopes.keys().nth(1).unwrap());

		let is_toplevel = match &item_scope.scopes[defining_scope] {
			Scope::Local { scope, .. } => {
				if scope.variables.contains_key(&new_name) {
					// Would cause identifier already defined error
					return RenameCheck::IdentifierAlreadyDefined;
				}
				false
			}
			Scope::Model { .. } => {
				let scope = item_scope.model.global_scope(db);
				if scope.variables.contains_key(&new_name) {
					// Would cause identifier already defined error
					return RenameCheck::IdentifierAlreadyDefined;
				}
				if GlobalScope::find_variable(db, new_name).is_some() {
					// Would cause identifier already defined error
					return RenameCheck::IdentifierAlreadyDefined;
				}
				true
			}
			Scope::Global => unreachable!(),
		};

		let types = pattern.item(db).types(db);

		if is_toplevel {
			// Check if any uses of this name will now refer to something else which shadows the new name
			for entity in pattern.references(db) {
				if let EntityId::Expression(e) = entity.entity(db) {
					let scope = entity.item(db).scope(db);
					if scope.find_variable(db, e, new_name).is_some() {
						// Identifier will now refer to the shadowing new name
						return RenameCheck::ShadowConflict;
					}
				}
			}
		} else {
			// Not top-level, shadowing could occur in both directions.

			// Find where this identifier is used that would end up shadowed by something else
			for entity in types.reverse_resolutions(pattern) {
				if let EntityId::Expression(e) = entity {
					let mut current = item_scope.expression_scopes[e].0;
					loop {
						if current == defining_scope {
							// Found our defining scope first, so will not change meaning
							break;
						}
						match &item_scope.scopes[current] {
							Scope::Local { parent, scope } => {
								if scope.variables.contains_key(&new_name) {
									// Will cause this identifier to refer to the shadowing name instead of our identifier
									return RenameCheck::ShadowConflict;
								}
								current = *parent;
							}
							_ => unreachable!(),
						}
					}
				}
			}

			// Look at scopes outward from the defining scope for ones that define the new name (i.e. ones that would be shadowed by the rename)
			let mut to_check = vec![];
			let mut current = defining_scope;
			loop {
				match &item_scope.scopes[current] {
					Scope::Local { parent, scope } => {
						if let Some((p, _)) = scope.variables.get(&new_name) {
							to_check.extend(types.reverse_resolutions(*p));
						}
						current = *parent;
					}
					Scope::Model { global_scope } => {
						if let Some((p, _)) =
							item_scope.model.global_scope(db).variables.get(&new_name)
						{
							to_check.extend(types.reverse_resolutions(*p));
							break;
						}
						current = *global_scope;
					}
					Scope::Global => {
						if let Some(p) = GlobalScope::find_variable(db, new_name) {
							to_check.extend(types.reverse_resolutions(p));
						}
						break;
					}
				}
			}

			// If the new identifier name is used in a position where the renamed pattern is closer in scope,
			// this would cause the rename to change the meaning of the program
			for entity in to_check {
				if let EntityId::Expression(e) = entity {
					let mut current = item_scope.expression_scopes[e].0;
					loop {
						if current == defining_scope {
							// Will cause this identifier to refer to the new variable instead of the old one
							return RenameCheck::ShadowConflict;
						}
						match &item_scope.scopes[current] {
							Scope::Local { parent, scope } => {
								if scope.variables.contains_key(&old_name) {
									// Found the old name first, so this will remain referring to the old variable and just cause a shadowing warning
									break;
								}
								current = *parent;
							}
							_ => unreachable!(),
						}
					}
				}
			}
		}

		RenameCheck::Ok
	}
}

/// Names in scope for expression in an item
#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct ItemScope<'db>(&'db ScopeResult<'db>);

impl<'db> Item<'db> {
	/// Get the scope for this item
	///
	/// This a mapping between expressions and the identifiers in scope for the given item.
	pub fn scope(&self, db: &'db dyn Db) -> ItemScope<'db> {
		ItemScope(collect_item_scope(db, *self))
	}
}

#[salsa::tracked(returns(ref))]
fn collect_item_scope<'db>(db: &'db dyn Db, item: Item<'db>) -> ScopeResult<'db> {
	collect_item_scope_with_diagnostics(db, item).0
}

#[salsa::tracked(returns(ref))]
fn collect_item_scope_diagnostics<'db>(db: &'db dyn Db, item: Item<'db>) -> Diagnostics {
	collect_item_scope_with_diagnostics(db, item).1
}

fn collect_item_scope_with_diagnostics<'db>(
	db: &'db dyn Db,
	item: Item<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	match item {
		Item::Annotation(item) => collect_annotation_scope(db, item),
		Item::Assignment(item) => collect_assignment_scope(db, item),
		Item::Constraint(item) => collect_constraint_scope(db, item),
		Item::Declaration(item) => collect_declaration_scope(db, item),
		Item::Enumeration(item) => collect_enumeration_scope(db, item),
		Item::EnumAssignment(item) => collect_enum_assignment_scope(db, item),
		Item::Function(item) => collect_function_scope(db, item),
		Item::Output(item) => collect_output_scope(db, item),
		Item::Solve(item) => collect_solve_scope(db, item),
		Item::TypeAlias(item) => collect_type_alias_scope(db, item),
		Item::Class(item) => collect_class_scope(db, item),
	}
}

fn collect_class_scope<'db>(
	db: &'db dyn Db,
	item: ClassItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let class_ref: Item<'db> = item.into();
	let class = item.class(db);
	let mut collector = ScopeCollector::new(db, class_ref, class.data());
	// The superclass and the class-level annotations resolve in the enclosing scope,
	// outside the `this`/member scope pushed below
	if let Some(base) = class.extends {
		collector.collect_expression(base);
	}
	for ann in class.annotations.iter() {
		collector.collect_expression(*ann);
	}
	collector.push();
	let identifiers = IdentifierRegistry::lookup(db);
	let this_ref = PatternRef::new(db, class_ref, class.this_pattern);
	let Scope::Local { scope, .. } = &mut collector.scopes[collector.current] else {
		unreachable!("Just pushed a local scope")
	};
	scope.add_variable(
		db,
		identifiers.names.this,
		0,
		this_ref,
		false,
		&mut collector.diagnostics,
	);
	if let Some(base) = class.extends {
		collector.add_inherited_class_fields_to_scope(class_ref, base);
	}
	for member in class.items.iter() {
		match member {
			ClassMember::Constraint(c) => {
				for ann in c.annotations.iter() {
					collector.collect_expression(*ann);
				}
				collector.collect_expression(c.expression);
			}
			ClassMember::Declaration(d) => {
				for ann in d.annotations.iter() {
					collector.collect_expression(*ann);
				}
				collector.collect_type(d.declared_type);
				if let Some(def) = d.definition {
					collector.collect_expression(def);
				}
				collector.collect_pattern(d.pattern, PatternMode::Destructuring);
			}
		}
	}
	collector.pop();
	collector.finish()
}

fn collect_annotation_scope<'db>(
	db: &'db dyn Db,
	item: AnnotationItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let annotation = item.annotation(db);
	let mut collector = ScopeCollector::new(db, item.into(), annotation.data());
	for p in annotation.parameters() {
		collector.collect_type(p.declared_type);
	}
	collector.finish()
}

fn collect_assignment_scope<'db>(
	db: &'db dyn Db,
	item: AssignmentItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let assignment = item.assignment(db);
	let mut collector = ScopeCollector::new(db, item.into(), assignment.data());
	collector.collect_expression(assignment.assignee);
	collector.collect_expression(assignment.definition);
	collector.finish()
}

fn collect_constraint_scope<'db>(
	db: &'db dyn Db,
	item: ConstraintItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let constraint = item.constraint(db);
	let mut collector = ScopeCollector::new(db, item.into(), constraint.data());
	for ann in constraint.annotations.iter() {
		collector.collect_expression(*ann);
	}
	collector.collect_expression(constraint.expression);
	collector.finish()
}

fn collect_declaration_scope<'db>(
	db: &'db dyn Db,
	item: DeclarationItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let declaration = item.declaration(db);
	let mut collector = ScopeCollector::new(db, item.into(), declaration.data());
	collector.collect_type(declaration.declared_type);
	for ann in declaration.annotations.iter() {
		collector.collect_expression(*ann);
	}
	if let Some(e) = declaration.definition {
		collector.collect_expression(e);
	}
	collector.finish()
}

fn collect_enumeration_scope<'db>(
	db: &'db dyn Db,
	item: EnumerationItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let enumeration = item.enumeration(db);
	let mut collector = ScopeCollector::new(db, item.into(), enumeration.data());
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

fn collect_enum_assignment_scope<'db>(
	db: &'db dyn Db,
	item: EnumAssignmentItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let assignment = item.enum_assignment(db);
	let mut collector = ScopeCollector::new(db, item.into(), assignment.data());
	collector.collect_expression(assignment.assignee);
	for c in assignment.definition.iter() {
		for p in c.parameters() {
			collector.collect_type(p.declared_type);
		}
	}
	collector.finish()
}

fn collect_function_scope<'db>(
	db: &'db dyn Db,
	item: FunctionItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let function = item.function(db);
	let mut collector = ScopeCollector::new(db, item.into(), function.data());
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

fn collect_output_scope<'db>(
	db: &'db dyn Db,
	item: OutputItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let output = item.output(db);
	let mut collector = ScopeCollector::new(db, item.into(), output.data());
	collector.collect_expression(output.expression);
	collector.finish()
}

fn collect_solve_scope<'db>(
	db: &'db dyn Db,
	item: SolveItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let solve = item.solve(db);
	let mut collector = ScopeCollector::new(db, item.into(), solve.data());
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

fn collect_type_alias_scope<'db>(
	db: &'db dyn Db,
	item: TypeAliasItem<'db>,
) -> (ScopeResult<'db>, Diagnostics) {
	let type_alias = item.type_alias(db);
	let mut collector = ScopeCollector::new(db, item.into(), type_alias.data());
	for ann in type_alias.annotations.iter() {
		collector.collect_expression(*ann);
	}
	collector.collect_type(type_alias.aliased_type);
	collector.finish()
}

#[cfg(test)]
mod tests {
	use expect_test::expect;
	use salsa::{Setter, attach};
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase, GlobalScope, Identifier, RenameCheck,
		ids::PatternRef,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};

	#[test]
	fn test_scopes() {
		let mut db = CompilerDatabase::default();
		let file_1 = InlineModelFile::builder(
			r#"
			test test_fn(int: x) = true;
			any: foo = 1;
			any: bar = foo;
			any: qux = let {
				any: foo = 2;
			} in foo + bar;
		"#
			.to_owned(),
			InputLang::MiniZinc,
		)
		.name(Some("file_1".to_owned()))
		.new(&db)
		.into();
		let file_2 = InlineModelFile::builder(
			r#"
			any: hello = test_fn(qux);
		"#
			.to_owned(),
			InputLang::MiniZinc,
		)
		.name(Some("file_2".to_owned()))
		.new(&db)
		.into();
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![file_1, file_2]);
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let mut actual = vec![];
		attach(&db, || {
			actual.push(format!("{:#?}", GlobalScope));
			for file in [file_1, file_2] {
				actual.extend(file.hir(&db).items(&db).iter().map(|item| {
					format!(
						"{:#?}\n{:#?}",
						item.get_item_with_data_as_debug(&db),
						item.scope(&db)
					)
				}));
			}
		});
		expect![[r#"
    GlobalScope {
        functions: {
            Identifier(
                "test_fn",
            ): [
                PatternRef {
                    item: file_1:2.4-31,
                    pattern: <Pattern::1>,
                },
            ],
        },
        variables: {
            Identifier(
                "bar",
            ): PatternRef {
                item: file_1:4.4-18,
                pattern: <Pattern::1>,
            },
            Identifier(
                "foo",
            ): PatternRef {
                item: file_1:3.4-16,
                pattern: <Pattern::1>,
            },
            Identifier(
                "hello",
            ): PatternRef {
                item: file_2:2.4-29,
                pattern: <Pattern::1>,
            },
            Identifier(
                "qux",
            ): PatternRef {
                item: file_1:5.4-7.18,
                pattern: <Pattern::1>,
            },
        },
    }

    ItemWithData {
        item: Function {
            return_type: <Type::1>,
            pattern: <Pattern::1>,
            type_inst_vars: [],
            parameters: [
                Parameter {
                    declared_type: <Type::2>,
                    pattern: Some(
                        <Pattern::2>,
                    ),
                    annotations: [],
                },
            ],
            body: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: BooleanLiteral(
                        true,
                    ),
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Bool,
                    },
                    <Type::2>: Primitive {
                        inst: Par,
                        opt: NonOpt,
                        primitive_type: Int,
                    },
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "test_fn",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
    ItemScope(
        ScopeResult {
            scopes: Arena {
                len: 4,
                data: {
                    <Scope::1>: Global,
                    <Scope::2>: Model {
                        global_scope: <Scope::1>,
                    },
                    <Scope::3>: Local {
                        parent: <Scope::2>,
                        scope: ScopeData {
                            functions: {},
                            variables: {},
                            atoms: {},
                        },
                    },
                    <Scope::4>: Local {
                        parent: <Scope::3>,
                        scope: ScopeData {
                            functions: {},
                            variables: {
                                Identifier(
                                    "x",
                                ): (
                                    PatternRef {
                                        item: file_1:2.4-31,
                                        pattern: <Pattern::2>,
                                    },
                                    1,
                                ),
                            },
                            atoms: {},
                        },
                    },
                },
            },
            expression_scopes: {
                <Expression::1>: (
                    <Scope::4>,
                    1,
                ),
            },
        },
    )

    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
    ItemScope(
        ScopeResult {
            scopes: Arena {
                len: 2,
                data: {
                    <Scope::1>: Global,
                    <Scope::2>: Model {
                        global_scope: <Scope::1>,
                    },
                },
            },
            expression_scopes: {
                <Expression::1>: (
                    <Scope::2>,
                    0,
                ),
            },
        },
    )

    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::1>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 1,
                data: {
                    <Expression::1>: Identifier(
                        "foo",
                    ),
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "bar",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
    ItemScope(
        ScopeResult {
            scopes: Arena {
                len: 2,
                data: {
                    <Scope::1>: Global,
                    <Scope::2>: Model {
                        global_scope: <Scope::1>,
                    },
                },
            },
            expression_scopes: {
                <Expression::1>: (
                    <Scope::2>,
                    0,
                ),
            },
        },
    )

    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::6>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 6,
                data: {
                    <Expression::1>: IntegerLiteral(
                        2,
                    ),
                    <Expression::2>: Identifier(
                        "foo",
                    ),
                    <Expression::3>: Identifier(
                        "bar",
                    ),
                    <Expression::4>: Identifier(
                        "+",
                    ),
                    <Expression::5>: Call {
                        kind: Operator,
                        function: <Expression::4>,
                        arguments: [
                            <Expression::2>,
                            <Expression::3>,
                        ],
                    },
                    <Expression::6>: Let {
                        items: [
                            Declaration(
                                Declaration {
                                    declared_type: <Type::2>,
                                    pattern: <Pattern::2>,
                                    definition: Some(
                                        <Expression::1>,
                                    ),
                                    annotations: [],
                                },
                            ),
                        ],
                        in_expression: <Expression::5>,
                    },
                },
            },
            types: Arena {
                len: 2,
                data: {
                    <Type::1>: Any,
                    <Type::2>: Any,
                },
            },
            patterns: Arena {
                len: 2,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "qux",
                        ),
                    ),
                    <Pattern::2>: Identifier(
                        Identifier(
                            "foo",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
    ItemScope(
        ScopeResult {
            scopes: Arena {
                len: 3,
                data: {
                    <Scope::1>: Global,
                    <Scope::2>: Model {
                        global_scope: <Scope::1>,
                    },
                    <Scope::3>: Local {
                        parent: <Scope::2>,
                        scope: ScopeData {
                            functions: {},
                            variables: {
                                Identifier(
                                    "foo",
                                ): (
                                    PatternRef {
                                        item: file_1:5.4-7.18,
                                        pattern: <Pattern::2>,
                                    },
                                    1,
                                ),
                            },
                            atoms: {},
                        },
                    },
                },
            },
            expression_scopes: {
                <Expression::1>: (
                    <Scope::3>,
                    0,
                ),
                <Expression::2>: (
                    <Scope::3>,
                    1,
                ),
                <Expression::3>: (
                    <Scope::3>,
                    1,
                ),
                <Expression::4>: (
                    <Scope::3>,
                    1,
                ),
                <Expression::5>: (
                    <Scope::3>,
                    1,
                ),
                <Expression::6>: (
                    <Scope::2>,
                    0,
                ),
            },
        },
    )

    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::3>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 3,
                data: {
                    <Expression::1>: Identifier(
                        "qux",
                    ),
                    <Expression::2>: Identifier(
                        "test_fn",
                    ),
                    <Expression::3>: Call {
                        kind: SourceCall,
                        function: <Expression::2>,
                        arguments: [
                            <Expression::1>,
                        ],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Any,
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "hello",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }
    ItemScope(
        ScopeResult {
            scopes: Arena {
                len: 2,
                data: {
                    <Scope::1>: Global,
                    <Scope::2>: Model {
                        global_scope: <Scope::1>,
                    },
                },
            },
            expression_scopes: {
                <Expression::1>: (
                    <Scope::2>,
                    0,
                ),
                <Expression::2>: (
                    <Scope::2>,
                    0,
                ),
                <Expression::3>: (
                    <Scope::2>,
                    0,
                ),
            },
        },
    )"#]]
		.assert_eq(&actual.join("\n\n"));
	}

	#[test]
	fn test_rename_check() {
		let mut db = CompilerDatabase::default();
		let model_file = InlineModelFile::new(
			&db,
			r#"
			any: foo = 1;
			any: bar = let {
				any: qux = 2;
			} in foo + qux;
		"#
			.into(),
			InputLang::MiniZinc,
		)
		.into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let items = model_file.hir(&db).items(&db);
		let foo = items[0];
		let bar = items[1];

		let foo_decl = foo.unwrap_declaration().declaration(&db);
		let foo_pattern = PatternRef::new(&db, foo, foo_decl.pattern);

		let safe_rename = RenameCheck::check(&db, foo_pattern, Identifier::new(&db, "hello"));
		assert_eq!(safe_rename, RenameCheck::Ok);

		let invalid_rename = RenameCheck::check(&db, foo_pattern, Identifier::new(&db, "bar"));
		assert_eq!(invalid_rename, RenameCheck::IdentifierAlreadyDefined);

		let bar_decl = bar.unwrap_declaration().declaration(&db);
		let bar_rhs = &bar_decl[bar_decl.definition.unwrap()];
		let bar_let = bar_rhs.unwrap_let_ref();
		let qux = PatternRef::new(&db, bar, bar_let.items[0].unwrap_declaration_ref().pattern);

		let valid_shadow_rename = RenameCheck::check(&db, foo_pattern, Identifier::new(&db, "baz"));
		assert_eq!(valid_shadow_rename, RenameCheck::Ok);

		let invalid_shadow_rename = RenameCheck::check(&db, qux, Identifier::new(&db, "foo"));
		assert_eq!(invalid_shadow_rename, RenameCheck::ShadowConflict);
	}
}
