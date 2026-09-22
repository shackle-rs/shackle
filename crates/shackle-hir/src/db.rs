//! Salsa database for HIR operations

use std::{fmt::Debug, fs::read_to_string, panic::RefUnwindSafe, path::Path, sync::Arc};

use salsa::Storage;
pub use salsa::{Setter, attach};
use shackle_diagnostics::{FileError, Result};

use crate::input::{CompilerSettings, InputFiles, ModelFile};

/// Crate database trait
///
/// Adds file reading functionality
#[salsa::db]
pub trait Db: salsa::Database + Debug {
	/// Get the file handler
	fn file_handler(&self) -> &dyn FileHandler;
}

/// The Shackle database.
#[salsa::db]
#[derive(Clone)]
pub struct CompilerDatabase {
	storage: Storage<Self>,
	file_handler: Arc<dyn FileHandler>,
}

impl CompilerDatabase {
	/// Create a new database with the given file handler
	pub fn with_file_handler(file_handler: Arc<impl FileHandler>) -> Self {
		let db = Self {
			storage: Storage::default(),
			file_handler,
		};
		let _ = InputFiles::new(&db, vec![]);
		let _ = CompilerSettings::default(&db);
		db
	}
}

impl Default for CompilerDatabase {
	fn default() -> Self {
		Self::with_file_handler(Arc::new(DefaultFileHandler))
	}
}

#[salsa::db]
impl salsa::Database for CompilerDatabase {}

#[salsa::db]
impl Db for CompilerDatabase {
	fn file_handler(&self) -> &dyn FileHandler {
		self.file_handler.as_ref()
	}
}

impl Debug for CompilerDatabase {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ShackleDatabase").finish()
	}
}

/// Trait for handling filesystem queries.
///
/// The `DefaultFileHandler` provides a default implementation which reads directly from the filesystem.
pub trait FileHandler: Send + Sync + RefUnwindSafe + 'static {
	/// Read a file and return its contents.
	fn read_file(&self, path: &Path) -> Result<String>;
	/// Whether the given path is a directory.
	fn is_dir(&self, path: &Path) -> bool;
	/// Whether the given path is a file.
	fn is_file(&self, path: &Path) -> bool;

	/// Notification of resolved includes, allowing the handler to watch contents and update if required.
	fn on_resolved_includes(&self, db: &dyn Db, files: &[ModelFile]);
}

/// Default file handler which reads from filesystem
#[derive(Clone, Debug)]
pub struct DefaultFileHandler;

impl FileHandler for DefaultFileHandler {
	fn read_file(&self, path: &Path) -> Result<String> {
		read_to_string(path).map_err(|e| {
			FileError {
				file: path.to_path_buf(),
				message: e.to_string(),
				other: vec![],
			}
			.into()
		})
	}

	fn is_dir(&self, path: &Path) -> bool {
		path.is_dir()
	}

	fn is_file(&self, path: &Path) -> bool {
		path.is_file()
	}

	fn on_resolved_includes(&self, _db: &dyn Db, _files: &[ModelFile]) {}
}

/// Access the "attached" database. Returns None if no database is attached.
pub fn with_attached_database<R>(op: impl FnOnce(&dyn Db) -> R) -> Option<R> {
	salsa::with_attached_database(|db| {
		let any_db: &dyn std::any::Any = db;
		if let Some(db) = any_db.downcast_ref::<CompilerDatabase>() {
			return Some(op(db));
		}
		None
	})
	.flatten()
}
