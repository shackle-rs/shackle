use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle::{
	file::ModelRef,
	hir::{
		db::Hir,
		ids::NodeRef,
		source::{find_node, Point},
	},
	utils::DebugPrint,
};

use crate::{dispatch::RequestHandler, extensions::ViewHir, LanguageServerDatabase};

#[derive(Debug)]
pub struct ViewHirHandler;

impl RequestHandler<ViewHir, (ModelRef, Point)> for ViewHirHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: TextDocumentPositionParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model_ref = db.set_active_file_from_document(&params.text_document)?;
		let start = Point {
			row: params.position.line as usize,
			column: params.position.character as usize,
		};
		Ok((model_ref, start))
	}

	fn execute(
		db: &dyn Hir,
		(model_ref, start): (ModelRef, Point),
	) -> Result<String, ResponseError> {
		let line = Point {
			row: start.row,
			column: 0,
		};
		let found = find_node(db, *model_ref, start, start)
			.or_else(|| find_node(db, *model_ref, line, line));
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
}
