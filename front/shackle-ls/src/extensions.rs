use lsp_types::{request::Request, TextDocumentPositionParams};

/// Request to view CST for a file
pub enum ViewCst {}

impl Request for ViewCst {
	type Params = TextDocumentPositionParams;
	type Result = String;
	const METHOD: &'static str = "shackle-ls/viewCst";
}

/// Request to view AST for a file
pub enum ViewAst {}

impl Request for ViewAst {
	type Params = TextDocumentPositionParams;
	type Result = String;
	const METHOD: &'static str = "shackle-ls/viewAst";
}

/// Request to view HIR for an item
pub enum ViewHir {}

impl Request for ViewHir {
	type Params = TextDocumentPositionParams;
	type Result = String;
	const METHOD: &'static str = "shackle-ls/viewHir";
}
