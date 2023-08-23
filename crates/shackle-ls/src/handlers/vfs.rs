use crate::LanguageServerDatabase;
use lsp_types::{
	DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};

pub fn on_document_open(db: &mut LanguageServerDatabase, params: DidOpenTextDocumentParams) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path");
	db.manage_file(file.as_path(), &params.text_document.text);
}

pub fn on_document_changed(db: &mut LanguageServerDatabase, params: DidChangeTextDocumentParams) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path");
	db.manage_file(
		file.as_path(),
		&params
			.content_changes
			.iter()
			.map(|c| c.text.clone())
			.collect::<String>(),
	);
}

pub fn on_document_closed(db: &mut LanguageServerDatabase, params: DidCloseTextDocumentParams) {
	let file = params
		.text_document
		.uri
		.to_file_path()
		.expect("Failed to convert URI to file path");
	db.unmanage_file(file.as_path());
}
