use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::RwLock,
};

use shackle_diagnostics::{FileError, Result};
use shackle_hir::db::FileHandler;

/// Virtual filesystem allowing us to override file reads
///
/// Uses a mutex internally so can be cloned and used by immutable reference.
#[derive(Debug)]
pub(crate) struct Vfs {
	files: RwLock<HashMap<PathBuf, String>>,
}

impl Vfs {
	/// Create a new VFS
	pub(crate) fn new() -> Self {
		Self {
			files: RwLock::new(HashMap::new()),
		}
	}

	/// Use the given string as the contents of this file instead of loading from the filesystem
	pub(crate) fn manage_file(&self, file: &Path, contents: &str) {
		let mut guard = self.files.write().unwrap();
		let _ = guard.insert(file.to_owned(), contents.to_owned());
	}

	/// Load the given file from the filesystem instead of using the managed contents
	pub(crate) fn unmanage_file(&self, file: &Path) {
		let mut guard = self.files.write().unwrap();
		let _ = guard.remove(&file.to_owned());
	}
}

impl FileHandler for Vfs {
	fn read_file(&self, path: &Path) -> Result<String> {
		let guard = self.files.read().unwrap();
		if let Some(s) = guard.get(path) {
			return Ok(s.clone());
		}

		std::fs::read_to_string(path).map_err(|e| {
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

	fn on_resolved_includes(
		&self,
		_db: &dyn shackle_hir::Db,
		_files: &[shackle_hir::input::ModelFile],
	) {
		// TODO: Watch files for changes and mark files as dirty
	}
}
