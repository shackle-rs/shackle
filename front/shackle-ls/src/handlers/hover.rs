use std::sync::Arc;

use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{Hover, HoverContents, HoverParams, LanguageString, MarkedString};
use shackle::{
	file::InputFile,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::LocalEntityRef,
		source::{find_node, Point},
	},
};

use crate::utils::node_ref_to_location;

pub fn hover(db: &mut dyn Hir, params: HoverParams) -> Result<Option<Hover>, ResponseError> {
	let path = params
		.text_document_position_params
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
		row: params.text_document_position_params.position.line as usize,
		column: params.text_document_position_params.position.character as usize,
	};
	let line = Point {
		row: start.row,
		column: 0,
	};
	let found =
		find_node(db, *model_ref, start, start).or_else(|| find_node(db, *model_ref, line, line));
	Ok((|| {
		let node = found?;
		match node {
			NodeRef::Entity(e) => {
				let item = e.item(db);
				match e.entity(db) {
					LocalEntityRef::Expression(e) => {
						let types = db.lookup_item_types(item);
						let model = item.model(db);
						let data = item.local_item_ref(db).data(&*model);
						Some(Hover {
							contents: HoverContents::Scalar(MarkedString::LanguageString(
								LanguageString {
									language: "minizinc".to_owned(),
									value: types.pretty_print_expression_ty(db, data, e)?,
								},
							)),
							range: Some(node_ref_to_location(db, node)?.range),
						})
					}
					LocalEntityRef::Pattern(p) => {
						let types = db.lookup_item_types(item);
						let model = item.model(db);
						let data = item.local_item_ref(db).data(&*model);
						Some(Hover {
							contents: HoverContents::Scalar(MarkedString::LanguageString(
								LanguageString {
									language: "minizinc".to_owned(),
									value: types.pretty_print_pattern_ty(db, data, p)?,
								},
							)),
							range: Some(node_ref_to_location(db, node)?.range),
						})
					}
					_ => None,
				}
			}
			_ => None,
		}
	})())
}
