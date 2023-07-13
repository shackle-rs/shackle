use lsp_server::ResponseError;
use lsp_types::{request::References, Location, ReferenceParams};
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::{LocalEntityRef, PatternRef},
		source::{find_node, Point},
	},
	syntax::db::SourceParser,
};

use crate::{dispatch::RequestHandler, utils::node_ref_to_location, LanguageServerDatabase};

#[derive(Debug)]
pub struct ReferencesHandler;

impl RequestHandler<References, (ModelRef, Point)> for ReferencesHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: ReferenceParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position.text_document)?;
		let start = Point {
			row: params.text_document_position.position.line as usize,
			column: params.text_document_position.position.character as usize,
		};
		Ok((model, start))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelRef, Point),
	) -> Result<Option<Vec<Location>>, ResponseError> {
		Ok((|| {
			let node = find_node(db, *model_ref, start, start)?;
			let pattern = match node {
				NodeRef::Entity(e) => {
					let item = e.item(db);
					match e.entity(db) {
						LocalEntityRef::Expression(e) => {
							let types = db.lookup_item_types(item);
							types.name_resolution(e)
						}
						LocalEntityRef::Pattern(p) => {
							let types = db.lookup_item_types(item);
							Some(
								types
									.pattern_resolution(p)
									.unwrap_or_else(|| PatternRef::new(item, p)),
							)
						}
						_ => None,
					}
				}
				_ => None,
			}?;
			let models = db.resolve_includes().ok()?;
			let mut locations = Vec::new();
			for m in models.iter().copied() {
				let cst = db.cst(*m).ok()?;
				let query = tree_sitter::Query::new(
					tree_sitter_minizinc::language(),
					tree_sitter_minizinc::IDENTIFIERS_QUERY,
				)
				.expect("Failed to create query");
				let mut cursor = tree_sitter::QueryCursor::new();
				let captures = cursor.captures(&query, cst.root_node(), cst.text().as_bytes());
				let nodes = captures.map(|(c, _)| c.captures[0].node);
				let source_map = db.lookup_source_map(m);
				for node in nodes {
					if let Some(node_ref @ NodeRef::Entity(entity)) = source_map.find_node(node) {
						let item = entity.item(db);
						let types = db.lookup_item_types(item);
						let def = match entity.entity(db) {
							LocalEntityRef::Expression(e) => types.name_resolution(e),
							LocalEntityRef::Pattern(p) => Some(PatternRef::new(item, p)),
							_ => None,
						};
						if def == Some(pattern) {
							if let Some(loc) = node_ref_to_location(db, node_ref) {
								locations.push(loc);
							}
						}
					}
				}
			}
			Some(locations)
		})())
	}
}
