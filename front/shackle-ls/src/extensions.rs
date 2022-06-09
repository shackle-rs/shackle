use lsp_types::{request::Request, TextDocumentPositionParams};

/// Request to view HIR for an item
pub enum ViewHir {}

impl Request for ViewHir {
	type Params = TextDocumentPositionParams;
	type Result = String;
	const METHOD: &'static str = "shackle-ls/viewHir";
}
