use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};

use shackle::{error::FileError, file::FileHandler};

/// Virtual filesystem allowing us to override file reads
///
/// Uses a mutex internally so can be cloned and used by immutable reference.
#[derive(Debug)]
pub struct Vfs {
	files: Arc<Mutex<HashMap<PathBuf, String>>>,
}

impl Vfs {
	/// Create a new VFS
	pub fn new() -> Self {
		Self {
			files: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	/// Use the given string as the contents of this file instead of loading from the filesystem
	pub fn manage_file(&self, file: &Path, contents: &str) {
		let mut guard = self.files.lock().unwrap();
		guard.insert(file.to_owned(), contents.to_owned());
	}

	/// Load the given file from the filesystem instead of using the managed contents
	pub fn unmanage_file(&self, file: &Path) {
		let mut guard = self.files.lock().unwrap();
		guard.remove(&file.to_owned());
	}
}

impl FileHandler for Vfs {
	fn durable(&self) -> bool {
		false
	}

	fn read_file(&self, path: &PathBuf) -> Result<Arc<String>, FileError> {
		let guard = self.files.lock().unwrap();
		if let Some(s) = guard.get(path) {
			return Ok(Arc::new(s.clone()));
		}

		std::fs::read_to_string(&path)
			.map(Arc::new)
			.map_err(|err| FileError {
				file: path.clone(),
				message: err.to_string(),
				other: Vec::new(),
			})
	}
}

impl Clone for Vfs {
	fn clone(&self) -> Self {
		Self {
			files: self.files.clone(),
		}
	}
}
