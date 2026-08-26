//! Compiler input module

use std::{
	ops::Deref,
	path::{Path, PathBuf},
	sync::{RwLock, RwLockReadGuard},
};

use derive_more::From;
use salsa::Setter;
use shackle_diagnostics::{Error as ShackleError, IncludeError, SourceFile};
use shackle_syntax::{
	InputLang,
	ast::{AstNode, ConstraintModel},
	cst::Cst,
	eprime::EPrimeModel,
	minizinc::{self, MznModel},
};
use shackle_utils::hash::{Map, Set};

use crate::{Db, Identifier, diagnostics::Errors, source::Origin};

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod input_files {
	use super::*;
	/// Input model files
	///
	/// This is a singleton input which is created when the database is created.
	/// Use `InputFiles::get(db)` to access it.
	#[salsa::input(debug, singleton)]
	pub struct InputFiles {
		/// The set of initial input files
		pub files: Vec<ModelFile>,
	}
}
pub use input_files::InputFiles;

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod model_file {
	use derive_more::{TryUnwrap, Unwrap};

	use super::*;

	#[derive(
		Debug,
		Copy,
		Clone,
		PartialEq,
		Eq,
		Hash,
		From,
		salsa::Supertype,
		salsa::SalsaValue,
		Unwrap,
		TryUnwrap,
	)]
	pub enum ModelFile {
		/// A named model file from the file system
		Named(NamedModelFile),
		/// An inline model file with specified contents
		Inline(InlineModelFile),
	}

	/// A model source file from the file system.
	#[salsa::input]
	pub struct NamedModelFile {
		/// The file path
		pub path: PathBuf,
		/// The contents of the file
		#[default]
		pub text: Cached<String>,
	}

	impl NamedModelFile {
		/// Get the contents of this model file
		pub fn contents<'a, 'db: 'a>(&self, db: &'db dyn Db) -> CachedValue<'a, String> {
			self.text(db).get(|| {
				db.file_handler()
					.read_file(self.path(db))
					.unwrap_or_else(|e| {
						Errors::add(db, e);
						"".to_owned()
					})
			})
		}

		/// Get the language of this model file based on its extension
		pub fn language(&self, db: &dyn Db) -> InputLang {
			InputLang::from_path(self.path(db))
		}

		/// Remove the contents of this model file from the database without
		/// invalidating queries.
		pub fn release(&self, db: &dyn Db) {
			let _ = self.text(db).forget();
		}

		/// Force a re-read of the contents of this file.
		pub fn invalidate(&self, db: &mut dyn Db) {
			log::debug!("Invalidating file {}", self.path(db).display());
			let _ = self.set_text(db).to(Cached::default());
		}
	}

	impl std::fmt::Debug for NamedModelFile {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			crate::db::with_attached_database(|db| {
				f.debug_struct("NamedModelFile")
					.field("path", &self.path(db))
					.finish()
			})
			.unwrap_or_else(|| f.debug_struct("NamedModelFile").finish())
		}
	}

	/// An in-memory model source file
	#[salsa::input]
	pub struct InlineModelFile {
		/// The contents of the file
		pub contents: String,
		#[returns(copy)]
		/// The language of the file
		pub language: InputLang,

		/// Name of the file for error messaging
		#[default]
		pub name: Option<String>,

		/// Directory used to resolve relative includes
		#[default]
		pub base_directory: Option<PathBuf>,
	}

	impl std::fmt::Debug for InlineModelFile {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			crate::db::with_attached_database(|db| {
				f.debug_struct("InlineModelFile")
					.field("name", &self.name(db))
					.field("base_directory", self.base_directory(db))
					.finish()
			})
			.unwrap_or_else(|| f.debug_struct("InlineModelFile").finish())
		}
	}
}

pub use model_file::{InlineModelFile, ModelFile, NamedModelFile};

impl ModelFile {
	/// Get the contents of this model file
	pub fn contents<'a, 'db: 'a>(&self, db: &'db dyn Db) -> ModelFileContents<'a, 'db> {
		match self {
			ModelFile::Named(n) => ModelFileContents(ModelFileContentsInner::Named(n.contents(db))),
			ModelFile::Inline(i) => {
				ModelFileContents(ModelFileContentsInner::Inline(i.contents(db)))
			}
		}
	}

	/// Get the language of this model file
	pub fn language(&self, db: &dyn Db) -> InputLang {
		match self {
			ModelFile::Named(n) => n.language(db),
			ModelFile::Inline(i) => i.language(db),
		}
	}

	/// Get the directory used to resolve relative includes from this model.
	pub fn base_directory(&self, db: &dyn Db) -> Option<PathBuf> {
		match self {
			ModelFile::Named(n) => n.path(db).parent().map(Path::to_owned),
			ModelFile::Inline(i) => i.base_directory(db).clone(),
		}
	}

	/// Get the name of this model file
	pub fn name(&self, db: &dyn Db) -> Option<String> {
		match self {
			ModelFile::Named(n) => n
				.path(db)
				.file_name()
				.map(|s| s.to_string_lossy().to_string()),
			ModelFile::Inline(i) => i.name(db).clone(),
		}
	}

	/// Get the resolved include items for this model
	pub fn include_items<'db>(&self, db: &'db dyn Db) -> Vec<IncludeItem<'db>> {
		includes_for_file(db, *self)
			.iter()
			.map(|(origin, model)| IncludeItem {
				origin,
				included_file: *model,
			})
			.collect()
	}
}

/// An include item in a model
///
/// Not part of HIR, but used to track the origin of included files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IncludeItem<'db> {
	origin: &'db Origin,
	included_file: ModelFile,
}

impl<'db> IncludeItem<'db> {
	/// Get the origin of the string literal for the included file
	pub fn origin(&self) -> &'db Origin {
		self.origin
	}

	/// Get the included file path
	pub fn file(&self, db: &'db dyn Db) -> &'db PathBuf {
		self.included_file.unwrap_named().path(db)
	}
}

/// Contents of a model file.
#[derive(Debug)]
pub struct ModelFileContents<'a, 'db>(ModelFileContentsInner<'a, 'db>);

impl<'a, 'db> Deref for ModelFileContents<'a, 'db> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		match &self.0 {
			ModelFileContentsInner::Named(cached) => cached,
			ModelFileContentsInner::Inline(s) => s,
		}
	}
}

#[derive(Debug)]
enum ModelFileContentsInner<'a, 'db> {
	Named(CachedValue<'a, String>),
	Inline(&'db str),
}

impl std::fmt::Display for ModelFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		crate::db::with_attached_database(|db| match self {
			ModelFile::Named(n) => write!(f, "{}", n.path(db).display()),
			ModelFile::Inline(i) => {
				write!(f, "{}", i.name(db).as_deref().unwrap_or("<unnamed file>"))
			}
		})
		.unwrap_or_else(|| write!(f, "<model file>"))
	}
}

impl ModelFile {
	/// Get the contents of this model
	pub fn ast<'db>(&self, db: &'db dyn Db) -> ModelAst<'db> {
		model_ast(db, *self)
	}

	/// Convert into a source file for error handling
	pub fn source_file(&self, db: &dyn Db) -> SourceFile {
		model_source_file(db, *self)
	}
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod compiler_settings {
	use super::*;
	/// Settings for the compiler.
	///
	/// This is a singleton input which is created when the database is created.
	/// Use `CompilerSettings::get(db)` to access it.
	#[salsa::input(debug, singleton)]
	pub struct CompilerSettings {
		/// Include search directories.
		pub search_directories: Vec<PathBuf>,

		/// The directory for the standard library.
		pub stdlib_directory: Option<PathBuf>,

		/// The directory for the globals library.
		pub globals_directory: Option<PathBuf>,

		/// Whether or not to ignore stdlib.
		#[returns(copy)]
		pub ignore_stdlib: bool,
	}
}
pub use compiler_settings::CompilerSettings;

impl CompilerSettings {
	/// Create default settings
	pub(crate) fn default(db: &dyn Db) -> Self {
		let mzn_stdlib_dir = std::env::var("MZN_STDLIB_DIR").ok().map(PathBuf::from);
		Self::new(db, vec![], mzn_stdlib_dir, None, false)
	}

	/// Copy the compiler settings from one database to another.
	pub fn copy_to(source: &dyn Db, target: &mut dyn Db) {
		let source_settings = Self::get(source);
		let search_directories = source_settings.search_directories(source).clone();
		let stdlib_directory = source_settings.stdlib_directory(source).clone();
		let globals_directory = source_settings.globals_directory(source).clone();
		let ignore_stdlib = source_settings.ignore_stdlib(source);
		let target_settings = Self::get(target);
		let _ = target_settings
			.set_search_directories(target)
			.to(search_directories);
		let _ = target_settings
			.set_stdlib_directory(target)
			.to(stdlib_directory);
		let _ = target_settings
			.set_globals_directory(target)
			.to(globals_directory);
		let _ = target_settings.set_ignore_stdlib(target).to(ignore_stdlib);
	}
}

#[salsa::tracked]
fn share_directory(db: &dyn Db) -> Option<PathBuf> {
	if let Some(p) = CompilerSettings::get(db).stdlib_directory(db) {
		// If set with MZN_STDLIB_DIR then just use it
		return Some(p.clone());
	}

	// TODO: For now, force use of MZN_STDLIB_DIR to grab old compiler's library
	// if let Some(p) = shackle_share_directory(db) {
	// 	return Ok(p);
	// }
	None
}

/// Get the shackle share directory
#[salsa::tracked]
pub fn shackle_share_directory(_db: &dyn Db) -> Option<PathBuf> {
	if let Ok(p) = std::env::current_exe() {
		// Otherwise find /share/minizinc/std from this executable
		for path in p.ancestors() {
			if path.join("share/minizinc/std/shackle.mzn").exists() {
				return Some(path.join("share/minizinc"));
			}
		}
	}
	None
}

#[salsa::tracked]
fn include_search_dirs(db: &dyn Db) -> Vec<PathBuf> {
	let settings = CompilerSettings::get(db);
	let mut include_dirs = settings.search_directories(db).clone();
	if let Some(globals) = settings.globals_directory(db) {
		if globals.is_absolute() || globals.exists() {
			include_dirs.push((*globals).clone());
		} else if let Some(share) = share_directory(db) {
			let path = share.join(globals);
			if path.exists() {
				include_dirs.push(path);
			}
		}
	}
	if let Some(shackle_share) = shackle_share_directory(db) {
		// For now, add shackle's stdlib dir to override the old compiler's one
		include_dirs.push(shackle_share.join("std"));
	}
	if let Some(share) = share_directory(db) {
		// Add the old compiler stdlib dir
		include_dirs.push(share.join("std"));
	}
	log::info!(
		"Include search directories:\n  {}",
		include_dirs
			.iter()
			.map(|d| d.display().to_string())
			.collect::<Vec<_>>()
			.join("\n  ")
	);
	include_dirs
}

#[allow(missing_docs, reason = "Salsa generates code with missing docs")]
mod model_ast_struct {
	use super::*;
	/// Parsed AST of a model
	#[salsa::tracked(debug)]
	pub struct ModelAst<'db> {
		/// The model file this AST is for
		#[returns(copy)]
		pub input_file: ModelFile,

		/// The model's abstract syntax tree
		pub ast: ConstraintModel,
	}
}
pub use model_ast_struct::ModelAst;

#[salsa::tracked(returns(copy))]
fn model_ast<'db>(db: &'db dyn Db, input_file: ModelFile) -> ModelAst<'db> {
	log::info!("Parsing {}", input_file);
	let cst = Cst::new(&input_file.contents(db), input_file.language(db));
	let ast = match input_file.language(db) {
		InputLang::MiniZinc => MznModel::new(cst).into(),
		InputLang::EPrime => EPrimeModel::new(cst).into(),
		x => unreachable!("Input language {:?} not supported", x),
	};
	ModelAst::new(db, input_file, ast)
}

#[salsa::tracked]
fn model_syntax_errors(db: &dyn Db, input_file: ModelFile) -> Vec<ShackleError> {
	let cst = Cst::new(&input_file.contents(db), input_file.language(db));
	if !cst.has_errors() {
		return Vec::new();
	}
	let src = input_file.source_file(db);
	cst.errors(&src).map(ShackleError::from).collect()
}

/// Accumulate syntax errors for all resolved models.
#[salsa::tracked(returns(copy))]
pub fn accumulate_syntax_errors(db: &dyn Db) {
	for model in resolve_includes(db) {
		Errors::extend(db, model_syntax_errors(db, *model).iter().cloned());
	}
}

#[salsa::tracked(returns(clone))]
fn model_source_file(db: &dyn Db, model_file: ModelFile) -> SourceFile {
	match model_file {
		ModelFile::Named(m) => SourceFile::new(m.path(db).to_owned(), m.contents(db).to_owned()),
		ModelFile::Inline(i) => SourceFile::unnamed(i.contents(db).to_owned()),
	}
}

/// Get the models included from a file
///
/// Query creates fresh model files, so duplicates have to be filtered out by caller
#[salsa::tracked]
fn includes_for_file(db: &dyn Db, model_file: ModelFile) -> Vec<(Origin, ModelFile)> {
	log::debug!("Resolving includes for {}", model_file);
	let mut result = Vec::new();
	let model = model_file.ast(db);
	let ast = model.ast(db);
	let search_dirs = include_search_dirs(db);
	match ast {
		ConstraintModel::MznModel(m) => {
			for item in m.items() {
				if let minizinc::Item::Include(i) = item {
					let value = i.file().value(&model_file.contents(db));
					let included = Path::new(&value);

					let resolved_file = if included.is_absolute() {
						included.to_owned()
					} else {
						// Resolve relative to search directories, then current file
						let file_dir = model_file.base_directory(db);
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
								let src = model_file.source_file(db);
								let span = i.file().span();
								Errors::add(
									db,
									IncludeError {
										src,
										span,
										include: value,
									},
								);
								continue;
							}
						}
					};
					result.push((
						Origin::new(model_file, i.file().span()),
						NamedModelFile::new(db, resolved_file.clone()).into(),
					));
				}
			}
		}
		ConstraintModel::EPrimeModel(_e) => {}
	}
	result
}

#[salsa::tracked(returns(copy))]
fn needs_eprime_redefs(db: &dyn Db) -> bool {
	let inputs = InputFiles::get(db);
	inputs
		.files(db)
		.iter()
		.any(|f| f.language(db) == InputLang::EPrime)
}

/// Get the automatically included models (e.g. stdlib)
#[salsa::tracked]
pub fn auto_includes(db: &dyn Db) -> Vec<ModelFile> {
	log::debug!("Computing automatic includes");

	let settings = CompilerSettings::get(db);
	if settings.ignore_stdlib(db) {
		return vec![];
	}
	if !share_directory(db).as_ref().is_some_and(|share_dir| {
		share_dir.join("std/stdlib.mzn").is_file()
			&& share_dir.join("std/solver_redefinitions.mzn").is_file()
	}) {
		Errors::add(db, ShackleError::MiniZincStandardLibraryNotFound);
	}

	let Some(share_dir) = shackle_share_directory(db) else {
		// share/minizinc directory does not exist
		Errors::add(db, ShackleError::StandardLibraryNotFound);
		return vec![];
	};

	let shackle_mzn = share_dir.join("std/shackle.mzn");
	log::debug!("Automatically including {}", shackle_mzn.display());
	let shackle_redefs = NamedModelFile::new(db, shackle_mzn).into();
	if needs_eprime_redefs(db) {
		// If any E' models, include the E' redefinitions
		let eprime_mzn = share_dir.join("std/eprime/eprime_redefinitions.mzn");
		log::debug!("Automatically including {}", eprime_mzn.display());
		let eprime_redefs = NamedModelFile::new(db, eprime_mzn).into();
		return vec![shackle_redefs, eprime_redefs];
	}
	vec![shackle_redefs]
}

/// Get the included models
#[salsa::tracked]
pub fn resolve_includes(db: &dyn Db) -> Vec<ModelFile> {
	log::info!("Resolving includes");
	let inputs = InputFiles::get(db);
	let mut result = resolve_auto_includes(db).clone();
	let mut seen = Set::from_iter(result.iter().map(|i| i.unwrap_named().path(db).clone()));
	let mut todo = inputs.files(db).iter().rev().copied().collect::<Vec<_>>();
	while let Some(model) = todo.pop() {
		if let ModelFile::Named(n) = model
			&& !seen.insert(n.path(db).to_owned())
		{
			continue;
		}
		result.push(model);
		todo.extend(includes_for_file(db, model).iter().map(|(_, f)| *f).rev());
	}
	db.file_handler().on_resolved_includes(db, &result);
	result
}

/// Get the automatically included models (e.g. stdlib) and their includes
#[salsa::tracked]
pub fn resolve_auto_includes(db: &dyn Db) -> Vec<ModelFile> {
	let mut result = vec![];
	let mut seen = Set::default();
	let mut todo = auto_includes(db).iter().copied().rev().collect::<Vec<_>>();
	while let Some(model) = todo.pop() {
		let n = model.unwrap_named();
		if seen.insert(n.path(db).to_owned()) {
			result.push(model);
			todo.extend(includes_for_file(db, model).iter().map(|(_, f)| *f).rev());
		}
	}
	result
}

#[salsa::tracked]
fn model_file_map(db: &dyn Db) -> Map<PathBuf, NamedModelFile> {
	resolve_includes(db)
		.iter()
		.filter_map(|f| f.try_unwrap_named().ok().map(|n| (n.path(db).clone(), n)))
		.collect()
}

/// Invalidates the given file, causing it to be re-read from disk on next access.
pub fn invalidate_file(db: &mut dyn Db, path: &Path) {
	let Some(model_file) = model_file_map(db).get(path).copied() else {
		log::warn!(
			"Received invalidation request for file {}, but it was not in the model file map",
			path.display()
		);
		return;
	};
	model_file.invalidate(db);
}

/// Get the names of enum items across all models
#[salsa::tracked]
pub fn enumeration_names<'db>(db: &'db dyn Db) -> Vec<Identifier<'db>> {
	let mut result = Vec::new();
	for model_file in resolve_includes(db).iter() {
		let model_ast = model_file.ast(db);
		let model = model_ast.ast(db);
		if let ConstraintModel::MznModel(mzn) = model {
			let contents = model_file.contents(db);
			let text = &*contents;
			for item in mzn.items() {
				if let minizinc::Item::Enumeration(e) = item {
					result.push(Identifier::new(db, e.id().name(text)));
				}
			}
		}
	}
	result
}

/// A cached value that can be recomputed or forgotten
#[derive(Debug, Default)]
pub struct Cached<T>(RwLock<Option<T>>);

impl<T> Cached<T> {
	/// Create a new cached value
	pub fn new(value: T) -> Self {
		Self(RwLock::new(Some(value)))
	}

	/// Remove the stored value and return it (if any)
	pub fn forget(&self) -> Option<T> {
		self.0.write().unwrap().take()
	}

	/// Gets the cached value if already set, or uses the provided function to compute, store, and return it otherwise.
	pub fn get<'a, 'b: 'a>(&'b self, f: impl FnOnce() -> T) -> CachedValue<'a, T> {
		let guard = self.0.read().unwrap();
		if guard.as_ref().is_some() {
			return CachedValue(guard);
		}
		drop(guard);

		let mut guard = self.0.write().unwrap();
		assert!(guard.replace(f()).is_none());
		drop(guard);

		let guard = self.0.read().unwrap();
		CachedValue(guard)
	}
}

/// Access a cached value
#[derive(Debug)]
pub struct CachedValue<'a, T>(RwLockReadGuard<'a, Option<T>>);

impl Deref for CachedValue<'_, String> {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref().expect("Cached value not set")
	}
}

#[cfg(test)]
mod tests {
	use std::{fs, path::PathBuf};

	use shackle_syntax::InputLang;

	use super::{CompilerSettings, InlineModelFile, InputFiles, ModelFile, resolve_includes};
	use crate::{CompilerDatabase, db::Setter};

	#[test]
	fn copy_compiler_settings_to_another_database() {
		let mut source = CompilerDatabase::default();
		let source_settings = CompilerSettings::get(&source);
		let _ = source_settings
			.set_search_directories(&mut source)
			.to(vec![PathBuf::from("search")]);
		let _ = source_settings
			.set_stdlib_directory(&mut source)
			.to(Some(PathBuf::from("stdlib")));
		let _ = source_settings
			.set_globals_directory(&mut source)
			.to(Some(PathBuf::from("globals")));
		let _ = source_settings.set_ignore_stdlib(&mut source).to(true);

		let mut target = CompilerDatabase::default();
		CompilerSettings::copy_to(&source, &mut target);

		let target_settings = CompilerSettings::get(&target);
		assert_eq!(
			target_settings.search_directories(&target),
			&[PathBuf::from("search")]
		);
		assert_eq!(
			target_settings.stdlib_directory(&target),
			&Some(PathBuf::from("stdlib"))
		);
		assert_eq!(
			target_settings.globals_directory(&target),
			&Some(PathBuf::from("globals"))
		);
		assert!(target_settings.ignore_stdlib(&target));
	}

	#[test]
	fn inline_model_resolves_includes_from_its_base_directory() {
		let directory = tempfile::tempdir().unwrap();
		let included_path = directory.path().join("included.mzn");
		fs::write(&included_path, "int: included;").unwrap();

		let mut db = CompilerDatabase::default();
		let settings = CompilerSettings::get(&db);
		let _ = settings.set_ignore_stdlib(&mut db).to(true);
		let model = InlineModelFile::new(
			&db,
			"include \"./included.mzn\";".to_owned(),
			InputLang::MiniZinc,
		);
		let _ = model
			.set_base_directory(&mut db)
			.to(Some(directory.path().to_owned()));
		let model: ModelFile = model.into();
		assert_eq!(model.base_directory(&db).as_deref(), Some(directory.path()));
		assert_eq!(model.include_items(&db).len(), 1);
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model]);

		let included_path = included_path.canonicalize().unwrap();
		assert!(resolve_includes(&db).iter().any(|file| {
			matches!(file, ModelFile::Named(file) if file.path(&db).canonicalize().ok().as_ref() == Some(&included_path))
		}));
	}
}
