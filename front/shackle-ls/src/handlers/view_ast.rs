use std::sync::Arc;

use lsp_server::{ErrorCode, ResponseError};
use lsp_types::TextDocumentPositionParams;
use shackle::{file::InputFile, hir::db::Hir};

pub fn view_ast(
	db: &mut dyn Hir,
	params: TextDocumentPositionParams,
) -> Result<String, ResponseError> {
	let path = params
		.text_document
		.uri
		.to_file_path()
		.map_err(|_| ResponseError {
			code: ErrorCode::InvalidParams as i32,
			data: None,
			message: "Failed to convert to file path".to_owned(),
		})?;
	db.set_input_files(Arc::new(vec![InputFile::Path(path)]));
	let model_ref = db.input_models()[0];
	match db.ast(*model_ref) {
		Ok(ast) => Ok(format!("{:#?}", ast)),
		Err(e) => Ok(e.to_string()),
	}
}
