use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle::{db::CompilerDatabase, file::ModelRef, syntax::db::SourceParser};

use crate::{dispatch::RequestHandler, extensions::ViewCst, LanguageServerDatabase};

#[derive(Debug)]
pub struct ViewCstHandler;

impl RequestHandler<ViewCst, ModelRef> for ViewCstHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, model_ref: ModelRef) -> Result<String, ResponseError> {
		match db.cst(*model_ref) {
			Ok(cst) => {
				let mut w = String::new();
				cst.debug_print(&mut w);
				Ok(w)
			}
			Err(e) => Ok(e.to_string()),
		}
	}
}
