#![allow(missing_docs)]

//! Compiler query database
//!

use super::hir::db::HirStorage;
use super::syntax::db::SourceParserStorage;
use super::thir::db::ThirStorage;
use crate::error::FileError;
use crate::file::{DefaultFileHandler, FileHandler, FileRef, FileRefData, InputFile, ModelRef};
use crate::hir::db::Hir;
use crate::syntax::db::SourceParser;
use crate::thir::db::Thir;
use crate::ty::{NewType, NewTypeData, Ty, TyData};

use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

/// Queries for inputs
#[salsa::query_group(InputsStorage)]
pub trait Inputs {
	/// Set source input files
	#[salsa::input]
	fn input_files(&self) -> Arc<Vec<InputFile>>;

	/// Set stdlib search directories
	#[salsa::input]
	fn search_directories(&self) -> Arc<Vec<PathBuf>>;
}

/// Queries for reading files
#[salsa::query_group(FileReaderStorage)]
pub trait FileReader: HasFileHandler + Inputs + Upcast<dyn Inputs> {
	/// Get the input file `FileRef`s
	#[salsa::invoke(crate::file::input_file_refs)]
	fn input_file_refs(&self) -> Arc<Vec<FileRef>>;

	/// Read source file
	#[salsa::invoke(crate::file::file_contents)]
	fn file_contents(&self, file: FileRef) -> Result<Arc<String>, FileError>;

	/// Get input model files
	#[salsa::invoke(crate::file::input_models)]
	fn input_models(&self) -> Arc<Vec<ModelRef>>;

	/// Intern a file reference
	#[salsa::interned]
	fn intern_file_ref(&self, item: FileRefData) -> FileRef;
}

/// Queries for interning
#[salsa::query_group(InternerStorage)]
pub trait Interner {
	#[salsa::interned]
	fn intern_string(&self, string: InternedStringData) -> InternedString;

	#[salsa::interned]
	fn intern_ty(&self, item: TyData) -> Ty;

	#[salsa::interned]
	fn intern_newtype(&self, item: NewTypeData) -> NewType;
}

/// An interned string
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternedString(salsa::InternId);

impl InternedString {
	/// Create a new interned string
	pub fn new<T: Into<InternedStringData>>(v: T, db: &dyn Interner) -> Self {
		db.intern_string(v.into())
	}

	/// Get the value of the string
	pub fn value(&self, db: &dyn Interner) -> String {
		db.lookup_intern_string(*self).0
	}
}

impl salsa::InternKey for InternedString {
	fn from_intern_id(id: salsa::InternId) -> Self {
		Self(id)
	}

	fn as_intern_id(&self) -> salsa::InternId {
		self.0
	}
}

/// String data
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InternedStringData(pub String);

impl<T> From<T> for InternedStringData
where
	T: Display,
{
	fn from(v: T) -> Self {
		Self(v.to_string())
	}
}

impl From<InternedStringData> for String {
	fn from(v: InternedStringData) -> Self {
		v.0
	}
}

/// Compiler database implementation
#[salsa::database(
	InputsStorage,
	FileReaderStorage,
	SourceParserStorage,
	HirStorage,
	InternerStorage,
	ThirStorage
)]
pub struct CompilerDatabase {
	storage: salsa::Storage<CompilerDatabase>,
	file_handler: Box<dyn FileHandler>,
}

impl CompilerDatabase {
	/// Create new new compiler database.
	pub fn new() -> Self {
		Self {
			storage: Default::default(),
			file_handler: Box::new(DefaultFileHandler),
		}
	}

	/// Create a new compiler database with the given file handler
	pub fn with_file_handler(file_handler: Box<dyn FileHandler>) -> Self {
		Self {
			storage: Default::default(),
			file_handler,
		}
	}

	/// Snapshot the database
	pub fn snapshot(&self) -> salsa::Snapshot<Self> {
		salsa::ParallelDatabase::snapshot(&self)
	}
}

impl salsa::Database for CompilerDatabase {
	// fn salsa_event(&self, event_fn: salsa::Event) {
	// 	match event_fn.kind {
	// 		salsa::EventKind::WillExecute { database_key } => {
	// 			eprintln!("  Executing {:?}", database_key.debug(self));
	// 		}
	// 		salsa::EventKind::DidValidateMemoizedValue { database_key } => {
	// 			eprintln!("  Using cached {:?}", database_key.debug(self));
	// 		}
	// 		_ => (),
	// 	}
	// }
}

impl salsa::ParallelDatabase for CompilerDatabase {
	fn snapshot(&self) -> salsa::Snapshot<Self> {
		salsa::Snapshot::new(Self {
			storage: self.storage.snapshot(),
			file_handler: self.file_handler.snapshot(),
		})
	}
}

/// Trait for accessing file handler of a database implementation
pub trait HasFileHandler {
	/// Get the file handler
	fn get_file_handler(&self) -> &dyn FileHandler;

	/// Invalid file contents query for the given path
	fn on_file_change(&mut self, file: &PathBuf);
}

impl HasFileHandler for CompilerDatabase {
	fn get_file_handler(&self) -> &dyn FileHandler {
		&*self.file_handler
	}

	fn on_file_change(&mut self, file: &PathBuf) {
		assert!(
			!self.get_file_handler().durable(),
			"Cannot handle file change for durable file query"
		);
		let f = FileRef::new(&file, self);
		FileContentsQuery.in_db_mut(self).invalidate(&f);
	}
}

/// Trait for upcasting the database
pub trait Upcast<T: ?Sized> {
	/// Perform upcast
	fn upcast(&self) -> &T;
}

/// Implement upcasts to the database traits
macro_rules! impl_upcast {
	($name:ident, $upcast:ident) => {
		impl $crate::db::Upcast<dyn $upcast> for $name {
			fn upcast(&self) -> &(dyn $upcast + 'static) {
				&*self
			}
		}
	};
}

impl_upcast!(CompilerDatabase, Inputs);
impl_upcast!(CompilerDatabase, FileReader);
impl_upcast!(CompilerDatabase, SourceParser);
impl_upcast!(CompilerDatabase, Interner);
impl_upcast!(CompilerDatabase, Hir);
impl_upcast!(CompilerDatabase, Thir);
