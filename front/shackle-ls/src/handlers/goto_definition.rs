use lsp_server::ResponseError;
use lsp_types::{request::GotoDefinition, GotoDefinitionParams, GotoDefinitionResponse};
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
pub struct GotoDefinitionHandler;

impl RequestHandler<GotoDefinition, (ModelRef, Point)> for GotoDefinitionHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: GotoDefinitionParams,
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
	) -> Result<Option<GotoDefinitionResponse>, ResponseError> {
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
							let resolution = types.name_resolution(e)?;
							Some(GotoDefinitionResponse::Scalar(node_ref_to_location(
								db,
								resolution.into_entity(db),
							)?))
						}
						LocalEntityRef::Pattern(p) => {
							let types = db.lookup_item_types(item);
							let resolution = types.pattern_resolution(p)?;
							Some(GotoDefinitionResponse::Scalar(node_ref_to_location(
								db,
								resolution.into_entity(db),
							)?))
						}
						_ => None,
					}
				}
				_ => None,
			}
		})())
	}
}
