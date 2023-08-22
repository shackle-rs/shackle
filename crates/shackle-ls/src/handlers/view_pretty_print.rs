use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

use crate::{dispatch::RequestHandler, extensions::ViewPrettyPrint, LanguageServerDatabase};

#[derive(Debug)]
pub struct ViewPrettyPrintHandler;

impl RequestHandler<ViewPrettyPrint, ModelRef> for ViewPrettyPrintHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, _: ModelRef) -> Result<String, ResponseError> {
		let errors = db.all_errors();
		if errors.is_empty() {
			let thir = db.final_thir();
			let printer = PrettyPrinter::new(db, &thir);
			Ok(printer.pretty_print())
		} else {
			Ok("% Errors present.".to_owned())
		}
	}
}
