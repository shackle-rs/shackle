use lsp_types::{
	DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};

use crate::{LanguageServerDatabase, utils::uri_to_path};

pub(crate) fn on_document_open(db: &mut LanguageServerDatabase, params: DidOpenTextDocumentParams) {
	let file = uri_to_path(&params.text_document.uri);
	db.manage_file(&file, &params.text_document.text);
}

pub(crate) fn on_document_changed(
	db: &mut LanguageServerDatabase,
	params: DidChangeTextDocumentParams,
) {
	let file = uri_to_path(&params.text_document.uri);
	// Sync is FULL, so every change is a whole document and only the last one
	// counts. Concatenating them duplicates the file.
	if let Some(change) = params.content_changes.last() {
		db.manage_file(&file, &change.text);
	}
}

pub(crate) fn on_document_closed(
	db: &mut LanguageServerDatabase,
	params: DidCloseTextDocumentParams,
) {
	let file = uri_to_path(&params.text_document.uri);
	db.unmanage_file(&file);
}
