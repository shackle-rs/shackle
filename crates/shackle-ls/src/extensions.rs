use lsp_types::{TextDocumentPositionParams, request::Request};

/// Request to view CST for a file
pub(crate) enum ViewCst {}

impl Request for ViewCst {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewCst";
}

/// Request to view AST for a file
pub(crate) enum ViewAst {}

impl Request for ViewAst {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewAst";
}

/// Request to view formatting IR for a file
pub(crate) enum ViewFormatIr {}

impl Request for ViewFormatIr {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewFormatIr";
}

/// Request to view HIR for an item
pub(crate) enum ViewHir {}

impl Request for ViewHir {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewHir";
}

/// Request to view identifiers in scope for an expression
pub(crate) enum ViewScope {}

impl Request for ViewScope {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewScope";
}

/// Request to view THIR before transforms for a file
pub(crate) enum ViewLoweredThir {}

impl Request for ViewLoweredThir {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewLoweredThir";
}

/// Request to view THIR after transforms for a file
pub(crate) enum ViewTransformedThir {}

impl Request for ViewTransformedThir {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewTransformedThir";
}

/// Request to view pretty printed MIR for a file
pub(crate) enum ViewMir {}

impl Request for ViewMir {
	type Params = TextDocumentPositionParams;
	type Result = String;

	const METHOD: &'static str = "shackle-ls/viewMir";
}
