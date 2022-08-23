use lsp_server::ResponseError;
use lsp_types::{
	request::HoverRequest, Hover, HoverContents, HoverParams, LanguageString, MarkedString,
};
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::LocalEntityRef,
		source::{find_node, Point},
	},
};

use crate::{dispatch::RequestHandler, utils::node_ref_to_location, LanguageServerDatabase};

#[derive(Debug)]
pub struct HoverHandler;

impl RequestHandler<HoverRequest, (ModelRef, Point)> for HoverHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: HoverParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position_params.text_document)?;
		let start = Point {
			row: params.text_document_position_params.position.line as usize,
			column: params.text_document_position_params.position.character as usize,
		};
		Ok((model, start))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelRef, Point),
	) -> Result<Option<Hover>, ResponseError> {
		let line = Point {
			row: start.row,
			column: 0,
		};
		let found = find_node(db, *model_ref, start, start)
			.or_else(|| find_node(db, *model_ref, line, line));
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
}
