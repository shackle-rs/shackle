#![allow(missing_docs)]

//! Salsa database for HIR operations

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::constants::IdentifierRegistry;
use crate::db::{CompilerSettings, FileReader, Interner, Upcast};
use crate::diagnostics::{Diagnostics, IncludeError, MultipleErrors};
use crate::file::{FileRef, ModelRef};
use crate::syntax::ast::{self, AstNode};
use crate::syntax::db::SourceParser;
use crate::ty::{EnumRef, Ty};
use crate::{Error, Result, Warning};

use super::ids::{EntityRef, EntityRefData, ItemRef, ItemRefData, PatternRef};
use super::scope::{ScopeData, ScopeResult};
use super::source::SourceMap;
use super::typecheck::{BodyTypes, SignatureTypes, TypeDiagnostics, TypeResult};
use super::{Identifier, Model, Pattern, PatternTy, ScopeCollectorResult};

/// HIR queries
#[salsa::query_group(HirStorage)]
pub trait Hir:
	Interner
	+ CompilerSettings
	+ SourceParser
	+ FileReader
	+ Upcast<dyn Interner>
	+ Upcast<dyn CompilerSettings>
	+ Upcast<dyn SourceParser>
	+ Upcast<dyn FileReader>
{
	/// Resolve input files and include items (only visits each model once).
	/// The result gives a list of models which need to be lowered into HIR.
	///
	/// If resolving files fails, then abort (but collect as many errors as possible).
	fn resolve_includes(&self) -> Result<Arc<Vec<ModelRef>>>;

	/// Get the syntax errors (only allowed if resolving includes succeeds)
	fn syntax_errors(&self) -> Arc<Vec<Error>>;

	/// Get the names of the enumeration items
	fn enumeration_names(&self) -> Arc<HashSet<Identifier>>;

	/// Lower the items of the given model to HIR.
	///
	/// Avoid using this query directly, and instead use `lookup_model` to retrieve the lowered model
	/// without the source map/diagnostics.
	#[salsa::invoke(super::lower::lower_items)]
	fn lower_items(&self, model: ModelRef) -> (Arc<Model>, Arc<SourceMap>, Arc<Vec<Error>>);

	/// Get the HIR for the given model
	fn lookup_model(&self, model: ModelRef) -> Arc<Model>;
	/// Get the source map for the given model
	fn lookup_source_map(&self, model: ModelRef) -> Arc<SourceMap>;
	/// Get the lowering diagnostics for the given model
	fn lookup_lowering_errors(&self, model: ModelRef) -> Arc<Vec<Error>>;
	/// Get the items for the given model
	fn lookup_items(&self, model: ModelRef) -> Arc<Vec<ItemRef>>;

	/// Collect the identifiers in global scope.
	///
	/// Avoid using this query directly, and instead use `lookup_global_variable` or
	/// `lookup_global_function` which is more likely to stay up to date and prevent extra
	/// recomputation.
	#[salsa::invoke(super::scope::collect_global_scope)]
	fn collect_global_scope(&self) -> (Arc<ScopeData>, Arc<Vec<Error>>);

	/// Get the identifiers in global scope.
	///
	/// Avoid using this query directly, and instead use `lookup_global_variable` or
	/// `lookup_global_function` which is more likely to stay up to date and prevent extra
	/// recomputation.
	fn lookup_global_scope(&self) -> Arc<ScopeData>;

	/// Get the errors from collecting global scope
	fn lookup_global_scope_errors(&self) -> Arc<Vec<Error>>;

	/// Get whether there is an atom with the given name in global scope
	fn lookup_global_atom(&self, identifier: Identifier) -> bool;

	/// Resolve this variable identifier in global scope.
	fn lookup_global_variable(&self, identifier: Identifier) -> Option<PatternRef>;

	/// Resolve this function identifier in global scope to retrieve the possible overloads.
	fn lookup_global_function(&self, identifier: Identifier) -> Arc<Vec<PatternRef>>;

	/// Collect the identifiers in scope for all expressions in an item.
	///
	/// Avoid using this query directly, and instead use the `lookup_item_scope` query to remain
	/// diagnostic independent.
	#[salsa::invoke(super::scope::collect_item_scope)]
	fn collect_item_scope(&self, item: ItemRef) -> ScopeCollectorResult;

	/// Get the identifiers in scope for all expression in this item.
	fn lookup_item_scope(&self, item: ItemRef) -> Arc<ScopeResult>;

	/// Get the diagnostics produced when assigning scopes to all expressions in this item.
	fn lookup_item_scope_errors(&self, item: ItemRef) -> Arc<Vec<Error>>;

	/// Get the warnings produced when assigning scopes to all expressions in this item.
	fn lookup_item_scope_warnings(&self, item: ItemRef) -> Arc<Vec<Warning>>;

	/// Compute the signature for this `item`.
	/// Panics if item does not have a signature.
	///
	/// Use `lookup_item_types` to get the result of typing the entire item.
	#[salsa::invoke(super::typecheck::collect_item_signature)]
	fn collect_item_signature(&self, item: ItemRef) -> (Arc<SignatureTypes>, Arc<Vec<Error>>);

	/// Get the signature for this item.
	///
	/// Use `lookup_item_types` to get the result of typing the entire item.
	fn lookup_item_signature(&self, item: ItemRef) -> Arc<SignatureTypes>;

	/// Get the diagnostics produced when computing the signature of this item.
	fn lookup_item_signature_errors(&self, item: ItemRef) -> Arc<Vec<Error>>;

	/// Compute the types of RHS expressions in this item.
	/// Panics if item does not have a body.
	///
	/// Use `lookup_item_types` to get the result of typing the entire item.
	#[salsa::invoke(super::typecheck::collect_item_body)]
	fn collect_item_body(&self, item: ItemRef) -> (Arc<BodyTypes>, Arc<Vec<Error>>);

	/// Get the types of expressions and declarations in this item.
	///
	/// Use `lookup_item_types` to get the result of typing the entire item.
	fn lookup_item_body(&self, item: ItemRef) -> Arc<BodyTypes>;

	/// Get the diagnostics produced when computing types of expressions and declarations in this item.
	fn lookup_item_body_errors(&self, item: ItemRef) -> Arc<Vec<Error>>;

	/// Get the result of typing this item.
	#[salsa::invoke(super::typecheck::TypeResult::new)]
	fn lookup_item_types(&self, item: ItemRef) -> TypeResult;

	/// Get the diagnostics produced when computing the types for this item.
	#[salsa::invoke(super::typecheck::TypeDiagnostics::new)]
	fn lookup_item_type_errors(&self, item: ItemRef) -> TypeDiagnostics;

	/// Topologically sort items
	///
	/// Use `lookup_topological_sorted_items` to remain diagnostics independent.
	#[salsa::invoke(super::typecheck::topological_sort)]
	fn topological_sort_items(&self) -> (Arc<Vec<ItemRef>>, Arc<Vec<Error>>);

	/// Lookup the topologically sorted item order
	fn lookup_topological_sorted_items(&self) -> Arc<Vec<ItemRef>>;

	/// Lookup errors from topologically sorting items
	fn lookup_topological_sorted_items_errors(&self) -> Arc<Vec<Error>>;

	/// Validate HIR
	#[salsa::invoke(super::validate::validate_hir)]
	fn validate_hir(&self) -> Arc<Vec<Error>>;

	/// Get all diagnostics for this module.
	fn all_errors(&self) -> Arc<Diagnostics<Error>>;

	/// Get all the warnings
	fn all_warnings(&self) -> Arc<Diagnostics<Warning>>;

	#[salsa::interned]
	fn intern_item_ref(&self, item: ItemRefData) -> ItemRef;

	#[salsa::interned]
	fn intern_entity_ref(&self, item: EntityRefData) -> EntityRef;

	/// Get identifier constants
	fn identifier_registry(&self) -> Arc<IdentifierRegistry>;

	/// Get a mapping from variable identifiers to their computed types
	fn variable_type_map(&self) -> Arc<FxHashMap<Identifier, Ty>>;

	/// Get a mapping from enum type to constructor patterns
	///
	/// Prefer `lookup_enum_constructors` instead.
	#[salsa::invoke(super::pattern_matching::enum_constructors)]
	fn enum_constructors(&self) -> Arc<FxHashMap<EnumRef, Arc<Vec<PatternRef>>>>;

	/// Lookup the enum constructors for the given enum type
	#[salsa::invoke(super::pattern_matching::lookup_enum_constructors)]
	fn lookup_enum_constructors(&self, e: EnumRef) -> Option<Arc<Vec<PatternRef>>>;

	/// Check that case expressions are exhaustive
	#[salsa::invoke(super::pattern_matching::check_case_exhaustiveness)]
	fn check_case_exhaustiveness(&self, item: ItemRef) -> (Arc<Vec<Error>>, Arc<Vec<Warning>>);

	/// Lookup diagnostics from checking case expression exhaustiveness
	fn lookup_case_exhaustiveness_errors(&self, item: ItemRef) -> Arc<Vec<Error>>;

	/// Lookup warnings from checking case expression exhaustiveness
	fn lookup_case_exhaustiveness_warnings(&self, item: ItemRef) -> Arc<Vec<Warning>>;
}

fn identifier_registry(db: &dyn Hir) -> Arc<IdentifierRegistry> {
	Arc::new(IdentifierRegistry::new(db))
}

fn variable_type_map(db: &dyn Hir) -> Arc<FxHashMap<Identifier, Ty>> {
	let mut result = FxHashMap::default();
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for (idx, declaration) in model.declarations.iter() {
			let types = db.lookup_item_types(ItemRef::new(db, *m, idx));
			for ident in Pattern::identifiers(declaration.pattern, &declaration.data) {
				let pattern_ty = types.get_pattern(ident).unwrap();
				let ty = match &pattern_ty {
					PatternTy::Variable(ty) => *ty,
					_ => unreachable!(),
				};
				result.insert(declaration.data[ident].identifier().unwrap(), ty);
			}
		}
		for (idx, solve) in model.solves.iter() {
			let types = db.lookup_item_types(ItemRef::new(db, *m, idx));
			let ident = Identifier::new("_objective", db);
			let pattern_ty = match solve.goal {
				super::Goal::Satisfy => None,
				super::Goal::Maximize {
					pattern,
					objective: _,
				} => Some(types.get_pattern(pattern).unwrap()),
				super::Goal::Minimize {
					pattern,
					objective: _,
				} => Some(types.get_pattern(pattern).unwrap()),
			};
			match &pattern_ty {
				Some(PatternTy::Variable(ty)) => {
					result.insert(ident, *ty);
				}
				Some(_) => unreachable!(),
				None => {}
			};
		}
	}
	Arc::new(result)
}

fn resolve_includes(db: &dyn Hir) -> Result<Arc<Vec<ModelRef>>> {
	let mut errors: Vec<Error> = Vec::new();
	let mut todo = (*db.input_models()).clone();

	let search_dirs = db.include_search_dirs();
	let auto_includes = ["solver_redefinitions.mzn", "stdlib.mzn"];

	if !db.ignore_stdlib() {
		if let Err(e) = db.share_directory() {
			// share/minizinc directory does not exist
			errors.push(e);
		} else {
			let mut found_stdlib = false;
			for dir in search_dirs.iter() {
				let found = auto_includes
					.iter()
					.map(|i| dir.join(*i))
					.filter(|p| p.exists())
					.collect::<Vec<_>>();
				if found.len() == auto_includes.len() {
					for f in found {
						todo.push(FileRef::new(&f, db.upcast()).into());
					}
					found_stdlib = true;
					break;
				}
			}
			if !found_stdlib {
				// Could not find the files even though there was a share/minizinc directory
				errors.push(Error::StandardLibraryNotFound);
			}
		}
	}
	let mut models = Vec::new();

	// Resolve includes
	let mut seen = FxHashSet::default();
	while let Some(file) = todo.pop() {
		if let Some(path) = file
			.path(db.upcast())
			.map(|p| p.canonicalize().unwrap_or(p))
		{
			if seen.contains(&path) {
				continue;
			}
			seen.insert(path);
		}
		let model = match db.ast(*file) {
			Ok(m) => m,
			Err(e) => {
				errors.push(e);
				continue;
			}
		};
		models.push(file);
		for item in model.items() {
			if let ast::Item::Include(i) = item {
				let value = i.file().value();
				let included = Path::new(&value);

				let resolved_file = if included.is_absolute() {
					included.to_owned()
				} else {
					// Resolve relative to search directories, then current file
					let file_dir = model
						.cst()
						.file()
						.path(db.upcast())
						.and_then(|p| p.parent().map(|p| p.to_owned()));

					let resolved = if included.starts_with("./") {
						file_dir.map(|p| p.join(included)).filter(|p| p.exists())
					} else {
						search_dirs
							.iter()
							.chain(file_dir.iter())
							.map(|p| p.join(included))
							.find(|p| p.exists())
					};

					match resolved {
						Some(r) => r,
						None => {
							let (src, span) = i.cst_node().source_span(db.upcast());
							errors.push(
								IncludeError {
									src,
									span,
									include: value,
								}
								.into(),
							);
							continue;
						}
					}
				};
				todo.push(FileRef::new(&resolved_file, db.upcast()).into());
			}
		}
	}

	if errors.is_empty() {
		Ok(Arc::new(models))
	} else if errors.len() == 1 {
		Err(errors.pop().unwrap())
	} else {
		Err(MultipleErrors { errors }.into())
	}
}

fn enumeration_names(db: &dyn Hir) -> Arc<HashSet<Identifier>> {
	// When lowering we need to know the enumeration item names so that we can
	// correctly handle assignments to them
	let mut result = HashSet::default();
	let models = db.resolve_includes().unwrap();
	for model in models.iter() {
		let ast = db.ast(**model).unwrap();
		for item in ast.items() {
			if let ast::Item::Enumeration(e) = item {
				result.insert(Identifier::new(e.id().name(), db));
			}
		}
	}
	Arc::new(result)
}

fn lookup_model(db: &dyn Hir, model: ModelRef) -> Arc<Model> {
	db.lower_items(model).0
}

fn lookup_source_map(db: &dyn Hir, model: ModelRef) -> Arc<SourceMap> {
	db.lower_items(model).1
}

fn lookup_lowering_errors(db: &dyn Hir, model: ModelRef) -> Arc<Vec<Error>> {
	db.lower_items(model).2
}

fn lookup_items(db: &dyn Hir, model: ModelRef) -> Arc<Vec<ItemRef>> {
	Arc::new(
		db.lookup_model(model)
			.items
			.iter()
			.map(|i| ItemRef::new(db, model, *i))
			.collect(),
	)
}

fn lookup_global_scope(db: &dyn Hir) -> Arc<ScopeData> {
	db.collect_global_scope().0
}

fn lookup_global_scope_errors(db: &dyn Hir) -> Arc<Vec<Error>> {
	db.collect_global_scope().1
}

fn lookup_global_atom(db: &dyn Hir, identifier: Identifier) -> bool {
	db.lookup_global_scope().is_atom(identifier, 0)
}

fn lookup_global_variable(db: &dyn Hir, identifier: Identifier) -> Option<PatternRef> {
	db.lookup_global_scope().find_variable(identifier, 0)
}

fn lookup_global_function(db: &dyn Hir, identifier: Identifier) -> Arc<Vec<PatternRef>> {
	let fns = db.lookup_global_scope().find_function(identifier, 0);
	Arc::new(fns)
}

fn lookup_item_scope(db: &dyn Hir, item: ItemRef) -> Arc<ScopeResult> {
	db.collect_item_scope(item).result
}

fn lookup_item_scope_errors(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_scope(item).diagnostics
}

fn lookup_item_scope_warnings(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Warning>> {
	db.collect_item_scope(item).warnings
}

fn lookup_item_signature(db: &dyn Hir, item: ItemRef) -> Arc<SignatureTypes> {
	db.collect_item_signature(item).0
}

fn lookup_item_signature_errors(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_signature(item).1
}

fn lookup_item_body(db: &dyn Hir, item: ItemRef) -> Arc<BodyTypes> {
	db.collect_item_body(item).0
}

fn lookup_item_body_errors(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_body(item).1
}

fn lookup_topological_sorted_items(db: &dyn Hir) -> Arc<Vec<ItemRef>> {
	db.topological_sort_items().0
}

fn lookup_topological_sorted_items_errors(db: &dyn Hir) -> Arc<Vec<Error>> {
	db.topological_sort_items().1
}

fn lookup_case_exhaustiveness_errors(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.check_case_exhaustiveness(item).0
}

fn lookup_case_exhaustiveness_warnings(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Warning>> {
	db.check_case_exhaustiveness(item).1
}

fn syntax_errors(db: &dyn Hir) -> Arc<Vec<Error>> {
	let errors = db
		.resolve_includes()
		.expect("Can't get syntax errors when resolving includes failed")
		.iter()
		.filter_map(|m| db.cst(**m).unwrap().error(db.upcast()))
		.map(|e| e.into())
		.collect::<Vec<_>>();
	Arc::new(errors)
}

fn all_errors(db: &dyn Hir) -> Arc<Diagnostics<Error>> {
	let mut diagnostics = Diagnostics::default();
	match db.resolve_includes() {
		Ok(r) => {
			// Collect syntax errors
			diagnostics.extend(db.syntax_errors());
			for m in r.iter() {
				// Collect lowering errors
				diagnostics.extend(db.lookup_lowering_errors(*m));
				for i in db.lookup_items(*m).iter() {
					// Collect scoping errors
					diagnostics.extend(db.lookup_item_scope_errors(*i));
					// Collect type errors
					for e in db.lookup_item_type_errors(*i).outer_iter() {
						diagnostics.extend(e);
					}
					// Collect pattern matching exhaustiveness errors
					diagnostics.extend(db.lookup_case_exhaustiveness_errors(*i));
				}
			}
			// Collect global scope errors
			diagnostics.extend(db.lookup_global_scope_errors());
			// Collect topological sort errors
			diagnostics.extend(db.lookup_topological_sorted_items_errors());
			// Collect final validation errors
			diagnostics.extend(db.validate_hir());
		}
		Err(e) => diagnostics.push(e),
	}
	Arc::new(diagnostics)
}

fn all_warnings(db: &dyn Hir) -> Arc<Diagnostics<Warning>> {
	let mut diagnostics = Diagnostics::default();
	if let Ok(r) = db.resolve_includes() {
		for m in r.iter() {
			for i in db.lookup_items(*m).iter() {
				// Collect scoping warnings
				diagnostics.extend(db.lookup_item_scope_warnings(*i));
				// Collect case exhaustiveness warnings
				diagnostics.extend(db.lookup_case_exhaustiveness_warnings(*i));
			}
		}
	}
	Arc::new(diagnostics)
}
