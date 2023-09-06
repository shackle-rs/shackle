use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_compiler::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewPrettyPrint};

#[derive(Debug)]
pub struct ViewPrettyPrintHandler;

impl RequestHandler<ViewPrettyPrint, ModelRef> for ViewPrettyPrintHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, _: ModelRef) -> Result<String, ResponseError> {
		let errors = db.all_errors();
		if errors.is_empty() {
			let thir = match db.final_thir() {
				Ok(m) => m,
				Err(e) => return Ok(format!("%: THIR error: {}", e)),
			};
			let printer = PrettyPrinter::new(db, &thir);
			Ok(printer.pretty_print())
		} else {
			Ok("% Errors present.".to_owned())
		}
	}
}
