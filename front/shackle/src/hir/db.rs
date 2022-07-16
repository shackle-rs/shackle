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

use super::ids::{EntityRef, EntityRefData, ItemRef, ItemRefData};
use super::source::SourceMap;
use super::Model;

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

	/// Get all diagnostics
	fn all_diagnostics(&self) -> Arc<Vec<Error>>;

	#[salsa::interned]
	fn intern_string(&self, string: HirStringData) -> HirString;

	#[salsa::interned]
	fn intern_item_ref(&self, item: ItemRefData) -> ItemRef;

	#[salsa::interned]
	fn intern_entity_ref(&self, item: EntityRefData) -> EntityRef;
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

fn all_diagnostics(db: &dyn Hir) -> Arc<Vec<Error>> {
	match db.resolve_includes() {
		Ok(r) => {
			// Collect syntax errors
			let mut errors: Vec<Error> = r
				.iter()
				.filter_map(|m| db.cst(**m).unwrap().error(db.upcast()))
				.map(|e| e.into())
				.collect();
			// Collect lowering errors
			for m in r.iter() {
				errors.extend(db.lookup_lowering_diagnostics(*m).iter().cloned());
			}
			// TODO: Collect type errors

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
