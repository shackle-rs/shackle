use crate::vfs::Vfs;
use lsp_server::Connection;
use lsp_types::{
	DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use shackle::{db::FileReader, hir::db::Hir};

use super::publish_diagnostics;

pub fn on_document_open(
	db: &mut dyn Hir,
	vfs: &Vfs,
	sender: &Connection,
	params: DidOpenTextDocumentParams,
) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path")
		.canonicalize()
		.expect("Failed to canonicalize path");
	vfs.manage_file(&file, &params.text_document.text);
	db.on_file_change(&file);
	let _ = publish_diagnostics(db, file.as_ref(), sender);
}

pub fn on_document_changed(
	db: &mut dyn Hir,
	vfs: &Vfs,
	sender: &Connection,
	params: DidChangeTextDocumentParams,
) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path")
		.canonicalize()
		.expect("Failed to canonicalize path");
	vfs.manage_file(
		&file,
		&params
			.content_changes
			.iter()
			.map(|c| c.text.clone())
			.collect::<String>(),
	);
	db.on_file_change(&file);
	let _ = publish_diagnostics(db, file.as_ref(), sender);
}

pub fn on_document_closed(db: &mut dyn FileReader, vfs: &Vfs, params: DidCloseTextDocumentParams) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path")
		.canonicalize()
		.expect("Failed to canonicalize path");
	vfs.unmanage_file(&file);
	db.on_file_change(&file);
}
