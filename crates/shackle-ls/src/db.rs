use std::{ops::Deref, path::Path, sync::Arc};

use crossbeam_channel::{SendError, Sender};
use lsp_server::{Connection, ErrorCode, Message, ResponseError};
use lsp_types::TextDocumentIdentifier;
use shackle::{
	db::{CompilerDatabase, FileReader, HasFileHandler, Inputs},
	file::{InputFile, ModelRef},
};

use crate::{diagnostics, vfs::Vfs};

/// Trait for handler preparation
pub trait LanguageServerContext: Deref<Target = CompilerDatabase> {
	fn set_active_file_from_document(
		&mut self,
		doc: &TextDocumentIdentifier,
	) -> Result<ModelRef, ResponseError>;
}

pub struct LanguageServerDatabase {
	vfs: Vfs,
	pool: threadpool::ThreadPool,
	sender: Sender<Message>,
	db: CompilerDatabase,
}

impl LanguageServerDatabase {
	pub fn new(connection: &Connection) -> Self {
		let fs = Vfs::new();
		let db = CompilerDatabase::with_file_handler(Box::new(fs.clone()));
		Self {
			vfs: fs,
			pool: threadpool::Builder::new().build(),
			sender: connection.sender.clone(),
			db,
		}
	}

	pub fn send(&self, message: Message) -> Result<(), SendError<Message>> {
		self.sender.send(message)
	}

	pub fn execute_async<F>(&self, f: F)
	where
		F: FnOnce(&CompilerDatabase, Sender<Message>) + Send + 'static,
	{
		let db = self.db.snapshot();
		let sender = self.sender.clone();
		self.pool.execute(move || {
			f(&db, sender);
		})
	}

	pub fn manage_file(&mut self, file: &Path, contents: &str) {
		log::info!("detected file changed for file {:?}", file);
		self.vfs.manage_file(file, contents);
		self.db.on_file_change(file);
		self.set_active_file(file);
	}

	pub fn unmanage_file(&mut self, file: &Path) {
		self.vfs.unmanage_file(file);
		log::info!("detected file changed for file {:?}", file);
		self.db.on_file_change(file);
	}

	pub fn set_active_file(&mut self, path: &Path) {
		self.db
			.set_input_files(Arc::new(vec![InputFile::Path(path.to_owned())]));
		let path_filter = path.to_owned();
		self.execute_async(move |db, sender| {
			let notification = diagnostics::diagnostics_notification(db, path_filter.as_path());
			sender
				.send(Message::Notification(notification))
				.expect("Failed to send diagnostics");
		});
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
	) -> Result<ModelRef, ResponseError> {
		let requested_path = doc.uri.to_file_path().map_err(|_| ResponseError {
			code: ErrorCode::InvalidParams as i32,
			data: None,
			message: "Failed to convert URI to file path".to_owned(),
		})?;
		self.set_active_file(&requested_path);
		Ok(self.input_models()[0])
	}
}
