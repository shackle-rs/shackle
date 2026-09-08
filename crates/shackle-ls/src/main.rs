//! The MiniZinc language server, providing IDE features such as go-to-definition, hover, and completions.
use std::error::Error;

use db::LanguageServerDatabase;
use lsp_server::{Connection, ExtractError, Message};
use lsp_types::{
	CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, PositionEncodingKind,
	SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
	SemanticTokensServerCapabilities, ServerCapabilities, SignatureHelpOptions,
	TextDocumentSyncKind,
	notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
};

use crate::{
	db::LanguageServerOptions,
	dispatch::{DispatchNotification, DispatchRequest},
	handlers::*,
	utils::PositionEncoding,
};

mod db;
mod diagnostics;
mod dispatch;
mod extensions;
mod handlers;
mod utils;
mod vfs;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
	env_logger::Builder::new()
		.format_target(false)
		.format_module_path(true)
		.filter_level(log::LevelFilter::Trace)
		.filter_module("salsa", log::LevelFilter::Warn)
		.filter_module("shackle", log::LevelFilter::Warn)
		.parse_default_env()
		.init();

	log::info!("starting MiniZinc language server");
	let (connection, io_threads) = Connection::stdio();

	// Capabilities depend on the client's, so the handshake is driven manually
	// rather than through `Connection::initialize`.
	let (initialize_id, initialize_params) = connection.initialize_start()?;
	let params: InitializeParams = serde_json::from_value(initialize_params)?;
	let encoding = negotiate_position_encoding(&params);
	log::info!("using {:?} position encoding", encoding);
	utils::set_position_encoding(encoding);

	let server_capabilities = serde_json::to_value(ServerCapabilities {
		position_encoding: Some(encoding.into()),
		definition_provider: Some(OneOf::Left(true)),
		references_provider: Some(OneOf::Left(true)),
		text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
		hover_provider: Some(HoverProviderCapability::Simple(true)),
		signature_help_provider: Some(SignatureHelpOptions {
			trigger_characters: Some(vec!["(".to_owned(), ",".to_owned()]),
			..Default::default()
		}),
		inlay_hint_provider: Some(OneOf::Left(true)),
		rename_provider: Some(OneOf::Left(true)),
		completion_provider: Some(CompletionOptions {
			trigger_characters: Some(vec![".".to_owned()]),
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
	})
	.unwrap();
	connection.initialize_finish(
		initialize_id,
		serde_json::json!({ "capabilities": server_capabilities }),
	)?;
	main_loop(connection, params)?;
	io_threads.join()?;
	log::info!("shutting down server");
	Ok(())
}

/// Pick the encoding for `Position::character`.
///
/// UTF-8 avoids converting the byte offsets the compiler works in, but may only
/// be chosen when the client offers it; UTF-16 is the protocol's default and is
/// the only encoding some clients accept.
fn negotiate_position_encoding(params: &InitializeParams) -> PositionEncoding {
	let offered = params
		.capabilities
		.general
		.as_ref()
		.and_then(|general| general.position_encodings.as_ref());
	match offered {
		Some(encodings) if encodings.contains(&PositionEncodingKind::UTF8) => {
			PositionEncoding::Utf8
		}
		_ => PositionEncoding::Utf16,
	}
}

fn main_loop(
	connection: Connection,
	params: InitializeParams,
) -> Result<(), Box<dyn Error + Sync + Send>> {
	let mut db = LanguageServerDatabase::new(
		&connection,
		LanguageServerOptions {
			workspace_uri: params
				.workspace_folders
				.map(|folders| folders[0].uri.clone()),
		},
	);
	for msg in &connection.receiver {
		match msg {
			Message::Request(req) => {
				if connection.handle_shutdown(&req)? {
					return Ok(());
				}

				let result = DispatchRequest::new(req, &mut db)
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

				match result {
					Ok(_) => (),
					Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
					Err(ExtractError::MethodMismatch(req)) => {
						log::warn!("unhandled {}", req.method)
					}
				};
			}
			Message::Response(resp) => {
				log::info!("got response: {:?}", resp);
			}
			Message::Notification(not) => {
				let result = DispatchNotification::new(not, &mut db)
					.on::<DidOpenTextDocument, _>(on_document_open)
					.on::<DidChangeTextDocument, _>(on_document_changed)
					.on::<DidCloseTextDocument, _>(on_document_closed)
					.finish();
				match result {
					Ok(()) => (),
					Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
					Err(ExtractError::MethodMismatch(not)) => {
						log::warn!("unhandled {}", not.method)
					}
				}
			}
		}
	}
	Ok(())
}
