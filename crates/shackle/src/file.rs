//! File-related functionality.
//!
//! `FileRef` is an interned data structure used to represent a pointer to a file (or inline string).

use crate::{db::FileReader, diagnostics::FileError};
use miette::{MietteSpanContents, SourceCode};
use std::{
	ops::Deref,
	panic::{RefUnwindSafe, UnwindSafe},
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
pub struct SourceFile(SourceFileInner);

#[derive(Clone, PartialEq, Eq, Hash)]
enum SourceFileInner {
	Text {
		name: Option<PathBuf>,
		source: Arc<String>,
	},
	Introduced(&'static str),
}

impl std::fmt::Debug for SourceFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SourceFile")
			.field("name", &self.name())
			.field(
				"source",
				&format!("<{} byte string>", self.contents().len()),
			);
		Ok(())
	}
}

impl SourceFile {
	/// Create a new source file from a `FileRef`
	pub fn new(file: FileRef, db: &dyn FileReader) -> Self {
		Self(SourceFileInner::Text {
			name: file.path(db),
			source: file.contents(db).unwrap_or_default(),
		})
	}

	/// Create a new introduced source file
	pub fn introduced(name: &'static str) -> Self {
		Self(SourceFileInner::Introduced(name))
	}

	/// Get the path for this source file if any
	pub fn path(&self) -> Option<&Path> {
		match &self.0 {
			SourceFileInner::Text { name, .. } => name.as_deref(),
			_ => None,
		}
	}

	/// Get the pretty name of this source file if any
	pub fn name(&self) -> Option<String> {
		match &self.0 {
			SourceFileInner::Text { name, .. } => name
				.as_deref()
				.map(|p| {
					std::env::current_dir()
						.ok()
						.and_then(|c| c.canonicalize().ok())
						.and_then(move |c| p.strip_prefix(c).ok().map(|p| p.to_owned()))
						.unwrap_or_else(|| p.to_owned())
				})
				.map(|p| p.to_string_lossy().to_string()),
			SourceFileInner::Introduced(name) => Some(name.to_string()),
		}
	}

	/// Get the contents of this source file
	pub fn contents(&self) -> &str {
		match &self.0 {
			SourceFileInner::Text { source, .. } => source,
			SourceFileInner::Introduced(_) => "",
		}
	}
}

impl SourceCode for SourceFile {
	fn read_span<'a>(
		&'a self,
		span: &miette::SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		let contents =
			self.contents()
				.read_span(span, context_lines_before, context_lines_after)?;

		Ok(Box::new(match self.name() {
			Some(name) => MietteSpanContents::new_named(
				name,
				contents.data(),
				*contents.span(),
				contents.line(),
				contents.column(),
				contents.line_count(),
			),
			None => MietteSpanContents::new(
				contents.data(),
				*contents.span(),
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
		db.intern_file_ref(FileRefData::ExternalFile(path.to_owned()))
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

	/// Pretty print file name for debugging
	pub fn pretty_print(&self, db: &dyn FileReader) -> String {
		self.path(db)
			.map(|p| p.to_string_lossy().to_string())
			.unwrap_or_else(|| "<unnamed file>".to_owned())
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
				h.read_file(p)
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
pub trait FileHandler: Send + UnwindSafe {
	/// Whether the results are durable (return false if file contents may change)
	fn durable(&self) -> bool {
		true
	}

	/// Read a file and return its contents.
	fn read_file(&self, path: &Path) -> Result<Arc<String>, FileError>;

	/// Create a snapshot of the file handler
	fn snapshot(&self) -> Box<dyn FileHandler + RefUnwindSafe>;
}

/// Default file handler which reads from filesystem
#[derive(Clone, Debug)]
pub struct DefaultFileHandler;

impl FileHandler for DefaultFileHandler {
	fn read_file(&self, path: &Path) -> Result<Arc<String>, FileError> {
		std::fs::read_to_string(path)
			.map(Arc::new)
			.map_err(|err| FileError {
				file: path.to_path_buf(),
				message: err.to_string(),
				other: Vec::new(),
			})
	}

	fn snapshot(&self) -> Box<dyn FileHandler + RefUnwindSafe> {
		Box::new(self.clone())
	}
}
