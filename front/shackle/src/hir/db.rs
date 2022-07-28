#![allow(missing_docs)]

//! Salsa database for HIR operations

use std::fmt::Display;
use std::path::Path;
use std::sync::Arc;

use rustc_hash::FxHashSet;

use crate::db::{FileReader, Upcast};
use crate::error::{IncludeError, MultipleErrors};
use crate::file::{FileRef, ModelRef};
use crate::syntax::ast::{self, AstNode};
use crate::syntax::db::SourceParser;
use crate::{Error, Result};

use super::ids::{EntityRef, EntityRefData, ItemRef, ItemRefData, PatternRef};
use super::scope::{ScopeData, ScopeResult};
use super::source::SourceMap;
use super::ty::{Ty, TyData};
use super::typecheck::{BodyTypes, SignatureTypes, TypeDiagnostics, TypeResult};
use super::{Identifier, Model};

/// HIR queries
#[salsa::query_group(HirStorage)]
pub trait Hir: SourceParser + FileReader + Upcast<dyn SourceParser> {
	/// Resolve input files and include items (only visits each model once).
	/// The result gives a list of models which need to be lowered into HIR.
	///
	/// If resolving files fails, then abort (but collect as many errors as possible).
	fn resolve_includes(&self) -> Result<Arc<Vec<ModelRef>>>;

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
	fn lookup_lowering_diagnostics(&self, model: ModelRef) -> Arc<Vec<Error>>;
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
	fn lookup_global_scope_diagnostics(&self) -> Arc<Vec<Error>>;

	/// Get whether there is an enum atom with the given name in global scope
	fn lookup_global_enum_atom(&self, identifier: Identifier) -> bool;

	/// Resolve this variable identifier in global scope.
	fn lookup_global_variable(&self, identifier: Identifier) -> Option<PatternRef>;

	/// Resolve this function identifier in global scope to retrieve the possible overloads.
	fn lookup_global_function(&self, identifier: Identifier) -> Arc<Vec<PatternRef>>;

	/// Collect the identifiers in scope for all expressions in an item.
	///
	/// Avoid using this query directly, and instead use the `lookup_item_scope` query to remain
	/// diagnostic independent.
	#[salsa::invoke(super::scope::collect_item_scope)]
	fn collect_item_scope(&self, item: ItemRef) -> (Arc<ScopeResult>, Arc<Vec<Error>>);

	/// Get the identifiers in scope for all expression in this item.
	fn lookup_item_scope(&self, item: ItemRef) -> Arc<ScopeResult>;

	/// Get the diagnostics produced when assigning scopes to all expressions in this item.
	fn lookup_item_scope_diagnostics(&self, item: ItemRef) -> Arc<Vec<Error>>;

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
	fn lookup_item_signature_diagnostics(&self, item: ItemRef) -> Arc<Vec<Error>>;

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
	fn lookup_item_body_diagnostics(&self, item: ItemRef) -> Arc<Vec<Error>>;

	/// Get the result of typing this item.
	#[salsa::invoke(super::typecheck::TypeResult::new)]
	fn lookup_item_types(&self, item: ItemRef) -> TypeResult;

	/// Get the diagnostics produced when computing the types for this item.
	#[salsa::invoke(super::typecheck::TypeDiagnostics::new)]
	fn lookup_item_type_diagnostics(&self, item: ItemRef) -> TypeDiagnostics;

	/// Validate HIR
	#[salsa::invoke(super::validate::validate_hir)]
	fn validate_hir(&self) -> Arc<Vec<Error>>;

	/// Get all diagnostics for this module.
	fn all_diagnostics(&self) -> Arc<Vec<Error>>;

	#[salsa::interned]
	fn intern_string(&self, string: HirStringData) -> HirString;

	#[salsa::interned]
	fn intern_item_ref(&self, item: ItemRefData) -> ItemRef;

	#[salsa::interned]
	fn intern_entity_ref(&self, item: EntityRefData) -> EntityRef;

	#[salsa::interned]
	fn intern_ty(&self, item: TyData) -> Ty;
}

fn resolve_includes(db: &dyn Hir) -> Result<Arc<Vec<ModelRef>>> {
	let mut errors: Vec<Error> = Vec::new();
	let mut todo = (*db.input_models()).clone();
	let mut models = Vec::new();

	// Add stdlib
	let search_dirs = db.search_directories();
	let auto_includes = ["solver_redefinitions.mzn", "stdlib.mzn"];
	for include in auto_includes {
		let path = Path::new(include);
		let resolved_path = search_dirs
			.iter()
			.map(|p| p.join(path))
			.filter(|p| p.exists())
			.next();
		match resolved_path {
			Some(ref p) => todo.push(FileRef::new(p, db.upcast()).into()),
			None => errors.push(Error::StandardLibraryNotFound),
		}
	}

	// Resolve includes
	let mut seen = FxHashSet::default();
	while let Some(file) = todo.pop() {
		if seen.contains(&file) {
			continue;
		}
		seen.insert(file);
		let model = match db.ast(*file) {
			Ok(m) => m,
			Err(e) => {
				errors.push(e);
				continue;
			}
		};
		models.push(file);
		for item in model.items() {
			match item {
				ast::Item::Include(i) => {
					let value = i.file().value();
					let included = Path::new(&value);

					let resolved_file = if included.is_absolute() {
						included.to_owned()
					} else {
						// Resolve relative to search directories, then current file
						let file_dir = model
							.cst_node()
							.cst()
							.file()
							.path(db.upcast())
							.and_then(|p| p.parent().map(|p| p.to_owned()));

						let resolved = search_dirs
							.iter()
							.chain(file_dir.iter())
							.map(|p| p.join(included))
							.filter(|p| p.exists())
							.next();

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
				_ => (),
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

fn lookup_model(db: &dyn Hir, model: ModelRef) -> Arc<Model> {
	db.lower_items(model).0
}

fn lookup_source_map(db: &dyn Hir, model: ModelRef) -> Arc<SourceMap> {
	db.lower_items(model).1
}

fn lookup_lowering_diagnostics(db: &dyn Hir, model: ModelRef) -> Arc<Vec<Error>> {
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

fn lookup_global_scope_diagnostics(db: &dyn Hir) -> Arc<Vec<Error>> {
	db.collect_global_scope().1
}

fn lookup_global_enum_atom(db: &dyn Hir, identifier: Identifier) -> bool {
	db.lookup_global_scope().is_enum_atom(identifier, 0)
}

fn lookup_global_variable(db: &dyn Hir, identifier: Identifier) -> Option<PatternRef> {
	db.lookup_global_scope().find_variable(identifier, 0)
}

fn lookup_global_function(db: &dyn Hir, identifier: Identifier) -> Arc<Vec<PatternRef>> {
	let fns = db.lookup_global_scope().find_function(identifier, 0);
	Arc::new(fns)
}

fn lookup_item_scope(db: &dyn Hir, item: ItemRef) -> Arc<ScopeResult> {
	db.collect_item_scope(item).0
}

fn lookup_item_scope_diagnostics(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_scope(item).1
}

fn lookup_item_signature(db: &dyn Hir, item: ItemRef) -> Arc<SignatureTypes> {
	db.collect_item_signature(item).0
}

fn lookup_item_signature_diagnostics(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_signature(item).1
}

fn lookup_item_body(db: &dyn Hir, item: ItemRef) -> Arc<BodyTypes> {
	db.collect_item_body(item).0
}

fn lookup_item_body_diagnostics(db: &dyn Hir, item: ItemRef) -> Arc<Vec<Error>> {
	db.collect_item_body(item).1
}

fn all_diagnostics(db: &dyn Hir) -> Arc<Vec<Error>> {
	match db.resolve_includes() {
		Ok(r) => {
			// Collect syntax errors
			let mut errors: Vec<Error> = r
				.iter()
				.filter_map(|m| db.cst(**m).unwrap().error(db.upcast()))
				.map(|e| e.into())
				.collect();
			for m in r.iter() {
				// Collect lowering errors
				errors.extend(db.lookup_lowering_diagnostics(*m).iter().cloned());
				for i in db.lookup_items(*m).iter() {
					// Collect scoping errors
					errors.extend(db.lookup_item_scope_diagnostics(*i).iter().cloned());
					// Collect type errors
					errors.extend(db.lookup_item_type_diagnostics(*i).iter().cloned());
				}
			}
			// Collect global scope errors
			errors.extend(db.lookup_global_scope_diagnostics().iter().cloned());
			// Collect final validation errors
			errors.extend(db.validate_hir().iter().cloned());
			Arc::new(errors)
		}
		Err(e) => Arc::new(vec![e]),
	}
}

/// An interned string
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct HirString(salsa::InternId);

impl HirString {
	/// Get the value of the string
	pub fn value(&self, db: &dyn Hir) -> String {
		db.lookup_intern_string(*self).0
	}
}

impl salsa::InternKey for HirString {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

/// String data
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct HirStringData(pub String);

impl<T> From<T> for HirStringData
where
	T: Display,
{
	fn from(v: T) -> Self {
		Self(v.to_string())
	}
}

impl From<HirStringData> for String {
	fn from(v: HirStringData) -> Self {
		v.0
	}
}
