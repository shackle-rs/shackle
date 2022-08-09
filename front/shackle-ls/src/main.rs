use crossbeam_channel::{SendError, Sender};
use lsp_types::{
	notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
	CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, ServerCapabilities,
	TextDocumentIdentifier, TextDocumentSyncKind,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, ops::Deref};
use std::{error::Error, path::Path};

use lsp_server::{Connection, ErrorCode, ExtractError, Message, ResponseError};

use shackle::{
	db::{CompilerDatabase, FileReader, HasFileHandler, Inputs},
	file::{InputFile, ModelRef},
	hir::db::Hir,
};

use crate::{
	dispatch::{DispatchNotification, DispatchRequest},
	handlers::*,
};

mod diagnostics;
mod dispatch;
mod extensions;
mod handlers;
mod utils;
mod vfs;

pub struct LanguageServerDatabase {
	vfs: vfs::Vfs,
	pool: threadpool::ThreadPool,
	sender: Sender<Message>,
	db: CompilerDatabase,
}

impl LanguageServerDatabase {
	pub fn new(connection: &Connection) -> Self {
		let fs = vfs::Vfs::new();
		let mut db = CompilerDatabase::with_file_handler(Box::new(fs.clone()));
		let mut search_dirs = Vec::new();
		let stdlib_dir = env::var("MZN_STDLIB_DIR");
		match stdlib_dir {
			Ok(v) => search_dirs.push(PathBuf::from(v)),
			_ => {}
		}
		db.set_search_directories(Arc::new(search_dirs));
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
		F: FnOnce(&dyn Hir, Sender<Message>) + Send + 'static,
	{
		let db = self.db.snapshot();
		let sender = self.sender.clone();
		self.pool.execute(move || {
			f(&*db, sender);
		})
	}

	pub fn manage_file(&mut self, file: &Path, contents: &str) {
		self.vfs.manage_file(file, contents);
		self.db.on_file_change(&file.to_owned());
		self.set_active_file(file);
	}

	pub fn unmanage_file(&mut self, file: &Path) {
		self.vfs.unmanage_file(&file);
		self.db.on_file_change(&file.to_owned());
	}

	pub fn set_active_file(&mut self, path: &Path) {
		self.db
			.set_input_files(Arc::new(vec![InputFile::Path(path.to_owned())]));
		let path_filter = path.to_owned();
		self.execute_async(move |db, sender| {
			let notification = diagnostics::diagnostics_notification(&*db, path_filter.as_path());
			sender
				.send(Message::Notification(notification))
				.expect("Failed to send diagnostics");
		});
	}

	pub fn set_active_file_from_document(
		&mut self,
		doc: &TextDocumentIdentifier,
	) -> Result<ModelRef, ResponseError> {
		let requested_path = doc
			.uri
			.to_file_path()
			.map_err(|_| ResponseError {
				code: ErrorCode::InvalidParams as i32,
				data: None,
				message: "Failed to convert URI to file path".to_owned(),
			})?
			.canonicalize()
			.map_err(|_| ResponseError {
				code: ErrorCode::InvalidParams as i32,
				data: None,
				message: "Failed to canonicalise file path".to_owned(),
			})?;
		self.set_active_file(&requested_path);
		Ok(self.input_models()[0])
	}
}

impl Deref for LanguageServerDatabase {
	type Target = CompilerDatabase;
	fn deref(&self) -> &Self::Target {
		&self.db
	}
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
	eprintln!("starting MiniZinc language server");
	let (connection, io_threads) = Connection::stdio();

	let server_capabilities = serde_json::to_value(&ServerCapabilities {
		definition_provider: Some(OneOf::Left(true)),
		text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
		hover_provider: Some(HoverProviderCapability::Simple(true)),
		completion_provider: Some(CompletionOptions {
			trigger_characters: Some(vec![".".to_owned()]),
			..Default::default()
		}),
		..Default::default()
	})
	.unwrap();
	let initialization_params = connection.initialize(server_capabilities)?;
	main_loop(connection, initialization_params)?;
	io_threads.join()?;
	eprintln!("shutting down server");
	Ok(())
}

fn main_loop(
	connection: Connection,
	params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
	let _params: InitializeParams = serde_json::from_value(params).unwrap();
	let mut db = LanguageServerDatabase::new(&connection);
	for msg in &connection.receiver {
		match msg {
			Message::Request(req) => {
				if connection.handle_shutdown(&req)? {
					return Ok(());
				}

				let result = DispatchRequest::new(req, &mut db)
					.on::<ViewCstHandler, _, _>()
					.on::<ViewAstHandler, _, _>()
					.on::<ViewHirHandler, _, _>()
					.on::<ViewScopeHandler, _, _>()
					.on::<GotoDefinitionHandler, _, _>()
					.on::<HoverHandler, _, _>()
					.on::<CompletionsHandler, _, _>()
					.finish();

				match result {
					Ok(_) => (),
					Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
					Err(ExtractError::MethodMismatch(req)) => {
						eprintln!("unhandled {}", req.method)
					}
				};
			}
			Message::Response(resp) => {
				eprintln!("got response: {:?}", resp);
			}
			Message::Notification(not) => {
				let result = DispatchNotification::new(not, &mut db)
					.on::<DidOpenTextDocument, _>(|db, params| {
						handlers::on_document_open(db, params)
					})
					.on::<DidChangeTextDocument, _>(|db, params| {
						handlers::on_document_changed(db, params)
					})
					.on::<DidCloseTextDocument, _>(|db, params| {
						handlers::on_document_closed(db, params)
					})
					.finish();
				match result {
					Ok(()) => (),
					Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
					Err(ExtractError::MethodMismatch(not)) => eprintln!("unhandled {}", not.method),
				}
			}
		}
	}
	Ok(())
}
