use db::LanguageServerDatabase;
use lsp_types::{
	notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
	CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, SemanticTokensFullOptions,
	SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
	ServerCapabilities, TextDocumentSyncKind,
};
use std::error::Error;

use lsp_server::{Connection, ExtractError, Message};

use crate::{
	dispatch::{DispatchNotification, DispatchRequest},
	handlers::*,
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

	let server_capabilities = serde_json::to_value(ServerCapabilities {
		definition_provider: Some(OneOf::Left(true)),
		references_provider: Some(OneOf::Left(true)),
		text_document_sync: Some(TextDocumentSyncKind::FULL.into()),
		hover_provider: Some(HoverProviderCapability::Simple(true)),
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
		..Default::default()
	})
	.unwrap();
	let initialization_params = connection.initialize(server_capabilities)?;
	main_loop(connection, initialization_params)?;
	io_threads.join()?;
	log::info!("shutting down server");
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
					.on::<ViewPrettyPrintHandler, _, _>()
					.on::<GotoDefinitionHandler, _, _>()
					.on::<ReferencesHandler, _, _>()
					.on::<HoverHandler, _, _>()
					.on::<CompletionsHandler, _, _>()
					.on::<SemanticTokensHandler, _, _>()
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
					Err(ExtractError::MethodMismatch(not)) => {
						log::warn!("unhandled {}", not.method)
					}
				}
			}
		}
	}
	Ok(())
}
