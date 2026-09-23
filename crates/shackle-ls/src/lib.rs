//! Embeddable Shackle language-server core.
//!
//! `Server::handle` has no I/O: hosts feed it one JSON-RPC message and forward
//! the returned messages via stdio, a Worker, or a test transcript.
use std::{path::PathBuf, sync::Arc};

use crossbeam_channel::unbounded;
use env_logger as _;
use lsp_server::{Connection, ErrorCode, ExtractError, Message, Response};
use lsp_types::{
	CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, PositionEncodingKind,
	SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
	SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelpOptions,
	TextDocumentSyncKind, Uri,
	notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
};

pub mod db;
mod diagnostics;
mod dispatch;
mod extensions;
mod handlers;
mod utils;
pub use utils::PositionEncoding;
pub mod vfs;

use crate::{
	db::LanguageServerDatabase,
	dispatch::{DispatchNotification, DispatchRequest},
	handlers::*,
};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
	/// Workspace URI
	pub workspace_uri: Uri,
	/// Path to shackle stdlib
	pub stdlib_directory: Option<PathBuf>,
	/// Path to MiniZinc stdlib
	pub minizinc_stdlib_directory: Option<PathBuf>,
}
/// Single-threaded server used by browser hosts.
#[derive(Debug)]
pub struct Server {
	db: LanguageServerDatabase,
	receiver: crossbeam_channel::Receiver<Message>,
	initialized: bool,
}
impl Server {
	/// Create a new server
	pub fn new(config: ServerConfig, files: Arc<vfs::Vfs>) -> Self {
		let (sender, receiver) = unbounded();
		Self {
			db: LanguageServerDatabase::new_embedded(files, sender, config),
			receiver,
			initialized: false,
		}
	}

	/// Handle a message
	pub fn handle(&mut self, message: Message) -> Vec<Message> {
		if !self.initialized {
			if let Message::Request(request) = message
				&& request.method == "initialize"
			{
				let id = request.id;
				return match serde_json::from_value::<InitializeParams>(request.params) {
					Ok(params) => {
						let encoding = negotiate_position_encoding(&params);
						utils::set_position_encoding(encoding);
						self.initialized = true;
						vec![Message::Response(Response::new_ok(
							id,
							serde_json::json!({"capabilities": capabilities(encoding)}),
						))]
					}
					Err(e) => vec![Message::Response(Response::new_err(
						id,
						ErrorCode::InvalidParams as i32,
						e.to_string(),
					))],
				};
			}
			return Vec::new();
		}
		match message {
			Message::Request(req) => self.request(req),
			Message::Notification(not) => self.notification(not),
			Message::Response(_) => Vec::new(),
		}
	}

	fn request(&mut self, req: lsp_server::Request) -> Vec<Message> {
		if req.method == "shutdown" {
			return vec![Message::Response(Response::new_ok(
				req.id,
				serde_json::Value::Null,
			))];
		}
		let id = req.id.clone();
		let result = DispatchRequest::new(req, &mut self.db)
			.on::<ViewCstHandler, _, _>()
			.on::<ViewAstHandler, _, _>()
			.on::<ViewFormatIrHandler, _, _>()
			.on::<ViewHirHandler, _, _>()
			.on::<ViewScopeHandler, _, _>()
			.on::<ViewPrettyPrintHandler, _, _>()
			.on::<ViewMirHandler, _, _>()
			.on::<GotoDefinitionHandler, _, _>()
			.on::<ReferencesHandler, _, _>()
			.on::<RenameHandler, _, _>()
			.on::<HoverHandler, _, _>()
			.on::<SignatureHelpHandler, _, _>()
			.on::<InlayHintHandler, _, _>()
			.on::<CompletionsHandler, _, _>()
			.on::<SemanticTokensHandler, _, _>()
			.on::<FormatHandler, _, _>()
			.finish();
		if let Err(error) = result {
			let response = match error {
				ExtractError::MethodMismatch(req) => Response::new_err(
					req.id,
					ErrorCode::MethodNotFound as i32,
					format!("Unhandled method {}", req.method),
				),
				ExtractError::JsonError { error, .. } => {
					Response::new_err(id, ErrorCode::InvalidParams as i32, error.to_string())
				}
			};
			let _ = self.db.send(Message::Response(response));
		}
		self.drain()
	}

	fn notification(&mut self, not: lsp_server::Notification) -> Vec<Message> {
		let result = DispatchNotification::new(not, &mut self.db)
			.on::<DidOpenTextDocument, _>(on_document_open)
			.on::<DidChangeTextDocument, _>(on_document_changed)
			.on::<DidCloseTextDocument, _>(on_document_closed)
			.finish();
		if let Err(ExtractError::JsonError { error, .. }) = result {
			log::warn!("malformed notification: {error}");
		}
		self.drain()
	}

	fn drain(&self) -> Vec<Message> {
		self.receiver.try_iter().collect()
	}

	/// Unmanage a file
	pub fn remove_project_file(&mut self, path: &std::path::Path) -> Vec<Message> {
		self.db.unmanage_file(path);
		self.drain()
	}
}

/// Run the native stdio language-server host.
///
/// This intentionally retains the desktop server's threaded request and
/// diagnostics execution. The [`Server`] type is the single-threaded core for
/// Worker hosts.
pub fn run_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let io_threads = {
		let (connection, io_threads) = Connection::stdio();
		let (initialize_id, initialize_params) = connection.initialize_start()?;
		let params: InitializeParams = serde_json::from_value(initialize_params)?;
		let encoding = negotiate_position_encoding(&params);
		log::info!("using {:?} position encoding", encoding);
		utils::set_position_encoding(encoding);
		connection.initialize_finish(
			initialize_id,
			serde_json::json!({ "capabilities": capabilities(encoding) }),
		)?;

		let workspace_uri = workspace_uri(&params);
		let mut db = LanguageServerDatabase::new(
			&connection,
			db::LanguageServerOptions {
				workspace_uri,
				close_documents: true,
				publish_diagnostics_on_request: true,
			},
		);
		for message in &connection.receiver {
			match message {
				Message::Request(request) => {
					if connection.handle_shutdown(&request)? {
						break;
					}
					dispatch_request(&mut db, request);
				}
				Message::Notification(notification) => dispatch_notification(&mut db, notification),
				Message::Response(response) => log::info!("got response: {:?}", response),
			}
		}
		io_threads
	};
	io_threads.join()?;
	log::info!("shutting down server");
	Ok(())
}

fn workspace_uri(params: &InitializeParams) -> Option<Uri> {
	if let Some(folder) = params
		.workspace_folders
		.as_ref()
		.and_then(|folders| folders.first())
	{
		return Some(folder.uri.clone());
	}
	#[allow(
		deprecated,
		reason = "rootUri is used by clients without workspace folders"
	)]
	params.root_uri.clone()
}

fn dispatch_request(db: &mut LanguageServerDatabase, request: lsp_server::Request) {
	let id = request.id.clone();
	let result = DispatchRequest::new(request, db)
		.on::<ViewCstHandler, _, _>()
		.on::<ViewAstHandler, _, _>()
		.on::<ViewFormatIrHandler, _, _>()
		.on::<ViewHirHandler, _, _>()
		.on::<ViewScopeHandler, _, _>()
		.on::<ViewPrettyPrintHandler, _, _>()
		.on::<ViewMirHandler, _, _>()
		.on::<GotoDefinitionHandler, _, _>()
		.on::<ReferencesHandler, _, _>()
		.on::<RenameHandler, _, _>()
		.on::<HoverHandler, _, _>()
		.on::<SignatureHelpHandler, _, _>()
		.on::<InlayHintHandler, _, _>()
		.on::<CompletionsHandler, _, _>()
		.on::<SemanticTokensHandler, _, _>()
		.on::<FormatHandler, _, _>()
		.finish();
	if let Err(error) = result {
		let response = match error {
			ExtractError::MethodMismatch(request) => Response::new_err(
				request.id,
				ErrorCode::MethodNotFound as i32,
				format!("Unhandled method {}", request.method),
			),
			ExtractError::JsonError { error, .. } => {
				Response::new_err(id, ErrorCode::InvalidParams as i32, error.to_string())
			}
		};
		let _ = db.send(Message::Response(response));
	}
}

fn dispatch_notification(db: &mut LanguageServerDatabase, notification: lsp_server::Notification) {
	let result = DispatchNotification::new(notification, db)
		.on::<DidOpenTextDocument, _>(on_document_open)
		.on::<DidChangeTextDocument, _>(on_document_changed)
		.on::<DidCloseTextDocument, _>(on_document_closed)
		.finish();
	match result {
		Ok(()) => (),
		Err(ExtractError::JsonError { method, error }) => {
			log::error!("malformed params for {method}: {error}")
		}
		Err(ExtractError::MethodMismatch(notification)) => {
			log::warn!("unhandled {}", notification.method)
		}
	}
}

/// Negociate position encoding method
pub fn negotiate_position_encoding(params: &InitializeParams) -> PositionEncoding {
	match params
		.capabilities
		.general
		.as_ref()
		.and_then(|g| g.position_encodings.as_ref())
	{
		Some(encodings) if encodings.contains(&PositionEncodingKind::UTF8) => {
			PositionEncoding::Utf8
		}
		_ => PositionEncoding::Utf16,
	}
}

/// Get the language server capabilities
pub fn capabilities(encoding: PositionEncoding) -> ServerCapabilities {
	ServerCapabilities {
		position_encoding: Some(encoding.into()),
		definition_provider: Some(OneOf::Left(true)),
		references_provider: Some(OneOf::Left(true)),
		text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
		hover_provider: Some(HoverProviderCapability::Simple(true)),
		signature_help_provider: Some(SignatureHelpOptions {
			trigger_characters: Some(vec!["(".into(), ",".into()]),
			..Default::default()
		}),
		inlay_hint_provider: Some(OneOf::Left(true)),
		rename_provider: Some(OneOf::Left(true)),
		completion_provider: Some(CompletionOptions {
			trigger_characters: Some(vec![".".into()]),
			..Default::default()
		}),
		semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
			SemanticTokensOptions {
				full: Some(SemanticTokensFullOptions::Delta { delta: Some(false) }),
				range: Some(false),
				legend: SemanticTokensLegend {
					token_types: TokenType::legend(),
					token_modifiers: TokenModifier::legend(),
				},
				..Default::default()
			},
		)),
		document_formatting_provider: Some(OneOf::Left(true)),
		..Default::default()
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use lsp_server::{Message, Request, RequestId};
	use lsp_types::WorkspaceFolder;

	use super::{Server, ServerConfig};
	use crate::vfs::Vfs;

	#[test]
	fn embedded_server_handles_shutdown() {
		let workspace_uri: lsp_types::Uri = "file:///workspace/".parse().unwrap();
		let mut server = Server::new(
			ServerConfig {
				workspace_uri,
				stdlib_directory: None,
				minizinc_stdlib_directory: None,
			},
			Arc::new(Vfs::default()),
		);
		let initialize = Request::new(
			RequestId::from(1),
			"initialize".to_owned(),
			serde_json::to_value(lsp_types::InitializeParams::default()).unwrap(),
		);
		assert_eq!(server.handle(Message::Request(initialize)).len(), 1);

		let output = server.handle(Message::Request(Request::new(
			RequestId::from(2),
			"shutdown".to_owned(),
			serde_json::Value::Null,
		)));
		assert!(
			matches!(output.as_slice(), [Message::Response(response)] if response.id == RequestId::from(2) && response.response_result.is_ok())
		);
	}

	#[test]
	fn workspace_folder_takes_precedence_over_root_uri() {
		let mut params = lsp_types::InitializeParams::default();
		#[allow(deprecated, reason = "tests precedence of options")]
		{
			params.root_uri = Some("file:///root/".parse().unwrap());
		}
		let workspace_uri: lsp_types::Uri = "file:///workspace/".parse().unwrap();
		params.workspace_folders = Some(vec![WorkspaceFolder {
			uri: workspace_uri.clone(),
			name: "workspace".to_owned(),
		}]);
		assert_eq!(super::workspace_uri(&params), Some(workspace_uri));
	}
}
