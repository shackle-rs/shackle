//! File-related functionality.
//!
//! `FileRef` is an interned data structure used to represent a pointer to a file (or inline string).

use crate::{db::FileReader, error::FileError};
use miette::{MietteSpanContents, SourceCode};
use std::{
	ops::Deref,
	path::{Path, PathBuf},
	sync::Arc,
};

/// Input files
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InputFile {
	/// File from filesystem
	Path(PathBuf),
	/// Inline model string
	ModelString(String),
	/// Inline dzn string
	DznString(String),
	/// Inline JSON data string
	JsonString(String),
}

/// Source file/text for error reporting
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
	name: Option<String>,
	source: Arc<String>,
}

impl std::fmt::Debug for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SourceFile")
			.field("name", &self.name)
			.field("source", &format!("<{} byte string>", self.source.len()));
		Ok(())
	}
}

impl SourceFile {
	/// Create a new source file from a `FileRef`
	pub fn new(file: FileRef, db: &dyn FileReader) -> Self {
		Self {
			name: file.path(db).map(|p| {
				p.canonicalize()
					.ok()
					.and_then(|p| {
						std::env::current_dir()
							.ok()
							.and_then(|c| c.canonicalize().ok())
							.and_then(move |c| p.strip_prefix(c).ok().map(|p| p.to_owned()))
					})
					.unwrap_or(p)
					.to_string_lossy()
					.to_string()
			}),
			source: file.contents(db).unwrap_or_default(),
		}
	}

	/// Get the name of this source file
	pub fn name(&self) -> &str {
		match self.name {
			Some(ref n) => n,
			None => "<unnamed file>",
		}
	}

	/// Get the contents of this source file
	pub fn contents(&self) -> &str {
		&self.source
	}
}

impl std::ops::Deref for SourceFile {
	type Target = String;

	fn deref(&self) -> &Self::Target {
		&self.source
	}
}

impl SourceCode for SourceFile {
	fn read_span<'a>(
		&'a self,
		span: &miette::SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		let contents = self
			.source
			.read_span(span, context_lines_before, context_lines_after)?;

		Ok(Box::new(match self.name {
			Some(ref p) => MietteSpanContents::new_named(
				p.clone(),
				contents.data(),
				contents.span().clone(),
				contents.line(),
				contents.column(),
				contents.line_count(),
			),
			None => MietteSpanContents::new(
				contents.data(),
				contents.span().clone(),
				contents.line(),
				contents.column(),
				contents.line_count(),
			),
		}))
	}
}

/// Interned reference to an input file or external file
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct FileRef(salsa::InternId);

impl salsa::InternKey for FileRef {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

impl FileRef {
	/// Create a new file reference for an external (included) file
	pub fn new(path: &Path, db: &dyn FileReader) -> Self {
		db.intern_file_ref(FileRefData::ExternalFile(
			path.canonicalize().unwrap_or_else(|_| path.to_owned()),
		))
	}

	/// Get the file path if any
	pub fn path(&self, db: &dyn FileReader) -> Option<PathBuf> {
		match db.lookup_intern_file_ref(*self) {
			FileRefData::InputFile(i) => match db.input_files()[i] {
				InputFile::Path(ref p) => Some(p.clone()),
				_ => None,
			},
			FileRefData::ExternalFile(p) => Some(p),
		}
	}

	/// Get the contents of this file
	pub fn contents(&self, db: &dyn FileReader) -> Result<Arc<String>, FileError> {
		db.file_contents(*self)
	}
}

/// Reference to an input file or external file
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FileRefData {
	/// From input
	InputFile(usize),
	/// From external source (included file)
	ExternalFile(PathBuf),
}

/// Get `FileRef`s for all input files
pub fn input_file_refs(db: &dyn FileReader) -> Arc<Vec<FileRef>> {
	let size = db.input_files().len();
	Arc::new(
		(0..size)
			.map(|i| db.intern_file_ref(FileRefData::InputFile(i)))
			.collect(),
	)
}

/// Get the contents of a file
pub fn file_contents(db: &dyn FileReader, file: FileRef) -> Result<Arc<String>, FileError> {
	match db.lookup_intern_file_ref(file) {
		FileRefData::InputFile(i) => match db.input_files()[i] {
			InputFile::Path(ref p) => {
				let h = db.get_file_handler();
				if !h.durable() {
					db.salsa_runtime()
						.report_synthetic_read(salsa::Durability::LOW);
				}
				h.read_file(p.canonicalize().as_ref().unwrap_or_else(|_| p))
			}
			InputFile::ModelString(ref s) => Ok(Arc::new(s.clone())),
			InputFile::DznString(ref s) => Ok(Arc::new(s.clone())),
			InputFile::JsonString(ref s) => Ok(Arc::new(s.clone())),
		},
		FileRefData::ExternalFile(p) => {
			let h = db.get_file_handler();
			if !h.durable() {
				db.salsa_runtime()
					.report_synthetic_read(salsa::Durability::LOW);
			}
			h.read_file(&p)
		}
	}
}

/// A reference to model file
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ModelRef(FileRef);

impl From<FileRef> for ModelRef {
	fn from(r: FileRef) -> Self {
		Self(r)
	}
}

impl Deref for ModelRef {
	type Target = FileRef;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Get all input model files
pub fn input_models(db: &dyn FileReader) -> Arc<Vec<ModelRef>> {
	Arc::new(
		db.input_files()
			.iter()
			.enumerate()
			.filter_map(|(idx, f)| match f {
				InputFile::Path(p) => match p.extension() {
					Some(e) => {
						if e.to_str() == Some("mzn") {
							Some(db.intern_file_ref(FileRefData::InputFile(idx)).into())
						} else {
							None
						}
					}
					None => None,
				},
				InputFile::ModelString(_) => {
					Some(db.intern_file_ref(FileRefData::InputFile(idx)).into())
				}
				_ => None,
			})
			.collect(),
	)
}

/// Trait for handling filesystem queries.
///
/// The `DefaultFileHandler` provides a default implementation which reads directly from the filesystem.
pub trait FileHandler {
	/// Whether the results are durable (return false if file contents may change)
	fn durable(&self) -> bool {
		return true;
	}

	/// Read a file and return its contents.
	fn read_file(&self, path: &PathBuf) -> Result<Arc<String>, FileError>;
}

/// Default file handler which reads from filesystem
pub struct DefaultFileHandler;

impl FileHandler for DefaultFileHandler {
	fn read_file(&self, path: &PathBuf) -> Result<Arc<String>, FileError> {
		std::fs::read_to_string(&path)
			.map(Arc::new)
			.map_err(|err| FileError {
				file: path.clone(),
				message: err.to_string(),
				other: Vec::new(),
			})
	}
}
