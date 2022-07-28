use std::sync::Arc;

use lsp_server::{ErrorCode, ResponseError};
use lsp_types::TextDocumentPositionParams;
use shackle::{
	file::InputFile,
	hir::{
		db::Hir,
		ids::NodeRef,
		source::{find_node, Point},
	},
	utils::DebugPrint,
};

pub fn view_hir(
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

	let start = Point {
		row: params.position.line as usize,
		column: params.position.character as usize,
	};
	let line = Point {
		row: start.row,
		column: 0,
	};
	let found =
		find_node(db, *model_ref, start, start).or_else(|| find_node(db, *model_ref, line, line));
	if let Some(node) = found {
		let item = match node {
			NodeRef::Entity(e) => e.item(db),
			NodeRef::Item(i) => i,
			_ => return Ok("".to_owned()),
		};
		let item_info = item.debug_print(db);
		let types = db.lookup_item_types(item);
		let type_info = types.debug_print(db);

		Ok(format!("{}\n{}", item_info, type_info))
	} else {
		Ok("Not an item.".to_owned())
	}
}
