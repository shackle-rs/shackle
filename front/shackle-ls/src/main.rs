use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use lsp_types::{
	notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
	request::{Completion, GotoDefinition, HoverRequest},
	CompletionOptions, HoverProviderCapability, InitializeParams, OneOf, ServerCapabilities,
	TextDocumentSyncKind,
};

use lsp_server::{Connection, ExtractError, Message};

use shackle::db::{CompilerDatabase, Inputs};

use crate::{
	dispatch::{DispatchNotification, DispatchRequest},
	extensions::*,
};

mod dispatch;
mod extensions;
mod handlers;
mod utils;
mod vfs;

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
	let pool = threadpool::Builder::new().build();
	let fs = vfs::Vfs::new();
	let mut db = CompilerDatabase::with_file_handler(Box::new(fs.clone()));
	let mut search_dirs = Vec::new();
	let stdlib_dir = env::var("MZN_STDLIB_DIR");
	match stdlib_dir {
		Ok(v) => search_dirs.push(PathBuf::from(v)),
		_ => {}
	}
	db.set_search_directories(Arc::new(search_dirs));

	let _params: InitializeParams = serde_json::from_value(params).unwrap();
	eprintln!("starting example main loop");
	for msg in &connection.receiver {
		match msg {
			Message::Request(req) => {
				if connection.handle_shutdown(&req)? {
					return Ok(());
				}

				let result = DispatchRequest::new(req, &mut db)
					.on::<ViewCst, _>(|ctx, params| handlers::view_cst(ctx, params))
					.on::<ViewAst, _>(|ctx, params| handlers::view_ast(ctx, params))
					.on::<ViewHir, _>(|ctx, params| handlers::view_hir(ctx, params))
					.on::<ViewScope, _>(|ctx, params| handlers::view_scope(ctx, params))
					.on::<GotoDefinition, _>(|ctx, params| handlers::goto_definition(ctx, params))
					.on::<HoverRequest, _>(|ctx, params| handlers::hover(ctx, params))
					.on::<Completion, _>(|ctx, params| handlers::completions(ctx, params))
					.finish();

				match result {
					Ok(response) => connection.sender.send(Message::Response(response))?,
					Err(err @ ExtractError::JsonError { .. }) => panic!("{:?}", err),
					Err(ExtractError::MethodMismatch(req)) => eprintln!("unhandled {}", req.method),
				}
			}
			Message::Response(resp) => {
				eprintln!("got response: {:?}", resp);
			}
			Message::Notification(not) => {
				let result = DispatchNotification::new(not, (&mut db, &fs, &pool, &connection))
					.on::<DidOpenTextDocument, _>(|(db, fs, pool, c), params| {
						handlers::on_document_open(db, fs, pool, c, params)
					})
					.on::<DidChangeTextDocument, _>(|(db, fs, pool, c), params| {
						handlers::on_document_changed(db, fs, pool, c, params)
					})
					.on::<DidCloseTextDocument, _>(|(db, fs, _, _), params| {
						handlers::on_document_closed(db, fs, params)
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
