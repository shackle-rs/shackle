//! Language server database

use std::{ops::Deref, path::Path, sync::Arc};

use crossbeam_channel::{SendError, Sender};
use lsp_server::{Connection, ErrorCode, Message, ResponseError};
use lsp_types::{TextDocumentIdentifier, Uri};
use shackle_hir::{
	CompilerDatabase,
	db::Setter,
	input::{CompilerSettings, InputFiles, ModelFile, NamedModelFile, invalidate_file},
};
use shackle_syntax::InputLang;

use crate::{ServerConfig, diagnostics, utils::uri_to_path, vfs::Vfs};

/// Whether this file can be compiled as a model.
///
/// Data inputs (DataZinc, JSON) have no model AST, so registering one as an
/// input file panics further down in the compiler.
fn is_model_file(path: &Path) -> bool {
	matches!(
		InputLang::from_path(path),
		InputLang::MiniZinc | InputLang::EPrime
	)
}

#[derive(Debug, Clone, Default)]
pub(crate) struct LanguageServerOptions {
	/// Workspace URI, if any
	pub workspace_uri: Option<Uri>,
	/// Whether the LSP `didClose` lifecycle event releases the VFS override.
	pub close_documents: bool,
	/// Whether requests selecting a document also publish its diagnostics.
	pub publish_diagnostics_on_request: bool,
}

/// Trait for handler preparation
pub(crate) trait LanguageServerContext: Deref<Target = CompilerDatabase> {
	/// Set the input file for the compiler database
	fn set_active_file_from_document(
		&mut self,
		doc: &TextDocumentIdentifier,
	) -> Result<ModelFile, ResponseError>;

	/// Create an independent compiler database with the same file handler and settings.
	fn new_scratch_database(&self) -> CompilerDatabase;

	/// Get the language server options
	fn get_options(&self) -> &LanguageServerOptions;
}

#[derive(Debug)]
pub(crate) struct LanguageServerDatabase {
	vfs: Arc<Vfs>,
	pool: Option<threadpool::ThreadPool>,
	sender: Sender<Message>,
	db: CompilerDatabase,
	options: LanguageServerOptions,
}

impl LanguageServerDatabase {
	pub(crate) fn new(connection: &Connection, options: LanguageServerOptions) -> Self {
		let fs = Arc::new(Vfs::default());
		Self::build(fs, connection.sender.clone(), options, true, None)
	}

	pub(crate) fn new_embedded(
		fs: Arc<Vfs>,
		sender: Sender<Message>,
		config: ServerConfig,
	) -> Self {
		let options = LanguageServerOptions {
			workspace_uri: Some(config.workspace_uri.clone()),
			close_documents: false,
			publish_diagnostics_on_request: false,
		};
		Self::build(fs, sender, options, false, Some(config))
	}

	fn build(
		fs: Arc<Vfs>,
		sender: Sender<Message>,
		options: LanguageServerOptions,
		threaded: bool,
		config: Option<ServerConfig>,
	) -> Self {
		let mut db = CompilerDatabase::with_file_handler(Arc::clone(&fs));
		if let Some(config) = config {
			let settings = CompilerSettings::get(&db);
			let _ = settings
				.set_stdlib_directory(&mut db)
				.to(config.stdlib_directory);
			let _ = settings
				.set_minizinc_stdlib_directory(&mut db)
				.to(config.minizinc_stdlib_directory);
		}
		Self {
			vfs: fs,
			pool: threaded.then(|| threadpool::Builder::new().build()),
			sender,
			db,
			options,
		}
	}

	pub(crate) fn send(&self, message: Message) -> Result<(), SendError<Message>> {
		self.sender.send(message)
	}

	pub(crate) fn execute_async<F>(&self, f: F)
	where
		F: FnOnce(&CompilerDatabase, Sender<Message>) + Send + 'static,
	{
		let db = self.db.clone();
		let sender = self.sender.clone();
		if let Some(pool) = &self.pool {
			pool.execute(move || f(&db, sender));
		} else {
			f(&db, sender);
		}
	}

	pub(crate) fn manage_file(&mut self, file: &Path, contents: &str) {
		log::info!("detected file changed for file {:?}", file);
		self.vfs.manage_file(file, contents);
		invalidate_file(&mut self.db, file);
		// Data files are still tracked in the VFS, but must not become the
		// active model.
		if is_model_file(file) {
			let _ = self.set_active_file(file);
		}
	}

	pub(crate) fn unmanage_file(&mut self, file: &Path) {
		self.vfs.unmanage_file(file);
		log::info!("detected file closed for file {:?}", file);
		invalidate_file(&mut self.db, file);
		let _ = self.send(Message::Notification(diagnostics::clear_notification(file)));
	}

	/// Process a document-close event according to the host's lifecycle policy.
	pub(crate) fn close_document(&mut self, file: &Path) {
		if self.options.close_documents {
			self.unmanage_file(file);
		}
	}

	pub(crate) fn set_active_file(&mut self, path: &Path) -> ModelFile {
		self.set_active_file_inner(path, true)
	}

	/// Select a file for a request without producing an unrelated diagnostics
	/// notification. A notification between a hover request and its response
	/// causes CodeMirror to discard the pending hover tooltip and retry forever.
	fn set_active_file_without_diagnostics(&mut self, path: &Path) -> ModelFile {
		self.set_active_file_inner(path, false)
	}

	fn set_active_file_inner(&mut self, path: &Path, publish_diagnostics: bool) -> ModelFile {
		let model_file = NamedModelFile::new(&self.db, path.to_path_buf()).into();
		let _ = InputFiles::get(&self.db)
			.set_files(&mut self.db)
			.to(vec![model_file]);
		if publish_diagnostics {
			let path_filter = path.to_owned();
			self.execute_async(move |db, sender| {
				let notification = diagnostics::diagnostics_notification(db, path_filter.as_path());
				sender
					.send(Message::Notification(notification))
					.expect("Failed to send diagnostics");
			});
		}
		model_file
	}
}

impl Deref for LanguageServerDatabase {
	type Target = CompilerDatabase;

	fn deref(&self) -> &Self::Target {
		&self.db
	}
}

impl LanguageServerContext for LanguageServerDatabase {
	fn set_active_file_from_document(
		&mut self,
		doc: &TextDocumentIdentifier,
	) -> Result<ModelFile, ResponseError> {
		let requested_path = uri_to_path(&doc.uri);
		if !is_model_file(&requested_path) {
			return Err(ResponseError {
				code: ErrorCode::InvalidRequest as i32,
				message: format!(
					"{:?} files are not supported by the language server",
					InputLang::from_path(&requested_path)
				),
				data: None,
			});
		}
		if self.options.publish_diagnostics_on_request {
			Ok(self.set_active_file(&requested_path))
		} else {
			Ok(self.set_active_file_without_diagnostics(&requested_path))
		}
	}

	fn new_scratch_database(&self) -> CompilerDatabase {
		let mut db = CompilerDatabase::with_file_handler(Arc::clone(&self.vfs));
		CompilerSettings::copy_to(&self.db, &mut db);
		db
	}

	fn get_options(&self) -> &LanguageServerOptions {
		&self.options
	}
}

#[cfg(test)]
mod tests {
	use std::{path::Path, sync::Arc};

	use crossbeam_channel::unbounded;
	use shackle_hir::db::FileHandler;

	use super::{LanguageServerDatabase, LanguageServerOptions};
	use crate::vfs::Vfs;

	fn database(close_documents: bool) -> LanguageServerDatabase {
		let (sender, _) = unbounded();
		LanguageServerDatabase::build(
			Arc::new(Vfs::default()),
			sender,
			LanguageServerOptions {
				workspace_uri: None,
				close_documents,
				publish_diagnostics_on_request: false,
			},
			false,
			None,
		)
	}

	#[test]
	fn desktop_close_releases_the_vfs_override() {
		let path = Path::new("/not-on-disk/test.dzn");
		let mut db = database(true);
		db.manage_file(path, "x = 1;");
		db.close_document(path);
		assert!(db.vfs.read_file(path).is_err());
	}

	#[test]
	fn native_database_uses_a_thread_pool() {
		let (connection, _) = lsp_server::Connection::memory();
		let db = LanguageServerDatabase::new(
			&connection,
			LanguageServerOptions {
				workspace_uri: None,
				close_documents: true,
				publish_diagnostics_on_request: true,
			},
		);
		assert!(db.pool.is_some());
	}

	#[test]
	fn browser_close_keeps_the_project_file() {
		let path = Path::new("/not-on-disk/test.dzn");
		let mut db = database(false);
		db.manage_file(path, "x = 1;");
		db.close_document(path);
		assert_eq!(db.vfs.read_file(path).unwrap(), "x = 1;");
	}
}
