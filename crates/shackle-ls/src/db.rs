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

use crate::{diagnostics, utils::uri_to_path, vfs::Vfs};

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

pub(crate) struct LanguageServerDatabase {
	vfs: Arc<Vfs>,
	pool: threadpool::ThreadPool,
	sender: Sender<Message>,
	db: CompilerDatabase,
	options: LanguageServerOptions,
}

impl LanguageServerDatabase {
	pub(crate) fn new(connection: &Connection, options: LanguageServerOptions) -> Self {
		let fs = Arc::new(Vfs::new());
		let db = CompilerDatabase::with_file_handler(Arc::clone(&fs));
		Self {
			vfs: fs,
			pool: threadpool::Builder::new().build(),
			sender: connection.sender.clone(),
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
		self.pool.execute(move || {
			f(&db, sender);
		})
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
		log::info!("detected file changed for file {:?}", file);
		invalidate_file(&mut self.db, file);
	}

	pub(crate) fn set_active_file(&mut self, path: &Path) -> ModelFile {
		let model_file = NamedModelFile::new(&self.db, path.to_path_buf()).into();
		let _ = InputFiles::get(&self.db)
			.set_files(&mut self.db)
			.to(vec![model_file]);
		let path_filter = path.to_owned();
		self.execute_async(move |db, sender| {
			let notification = diagnostics::diagnostics_notification(db, path_filter.as_path());
			sender
				.send(Message::Notification(notification))
				.expect("Failed to send diagnostics");
		});
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
		Ok(self.set_active_file(&requested_path))
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
