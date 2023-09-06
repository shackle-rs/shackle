use lsp_server::ResponseError;
use lsp_types::{request::References, Location, ReferenceParams};
use shackle_compiler::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::{LocalEntityRef, PatternRef},
		source::{find_node, Point},
	},
	syntax::db::SourceParser,
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::node_ref_to_location};

#[derive(Debug)]
pub struct ReferencesHandler;

#[derive(Debug)]
pub struct ReferencesHandlerData {
	model_ref: ModelRef,
	point: Point,
	include_decl: bool,
}

impl RequestHandler<References, ReferencesHandlerData> for ReferencesHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: ReferenceParams,
	) -> Result<ReferencesHandlerData, ResponseError> {
		let model_ref =
			db.set_active_file_from_document(&params.text_document_position.text_document)?;
		let point = Point {
			row: params.text_document_position.position.line as usize,
			column: params.text_document_position.position.character as usize,
		};
		Ok(ReferencesHandlerData {
			model_ref,
			point,
			include_decl: params.context.include_declaration,
		})
	}

	fn execute(
		db: &CompilerDatabase,
		config: ReferencesHandlerData,
	) -> Result<Option<Vec<Location>>, ResponseError> {
		Ok((|| {
			let node = find_node(db, *config.model_ref, config.point, config.point)?;
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
							LocalEntityRef::Pattern(p) if config.include_decl => {
								Some(PatternRef::new(item, p))
							}
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

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use crate::handlers::test::test_handler;

	use super::ReferencesHandler;

	#[test]
	fn test_references() {
		test_handler::<ReferencesHandler, _, _>(
			r#"
int: hello;
int: y = hello + 1;
int: z = hello + let { int: hello = int; } in hello;
			"#,
			false,
			lsp_types::ReferenceParams {
				context: lsp_types::ReferenceContext {
					include_declaration: true,
				},
				partial_result_params: lsp_types::PartialResultParams {
					partial_result_token: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Url::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 1,
						character: 8,
					},
				},
			},
			expect!([r#"
    {
      "Ok": [
        {
          "uri": "file:///test.mzn",
          "range": {
            "start": {
              "line": 1,
              "character": 5
            },
            "end": {
              "line": 1,
              "character": 10
            }
          }
        },
        {
          "uri": "file:///test.mzn",
          "range": {
            "start": {
              "line": 2,
              "character": 9
            },
            "end": {
              "line": 2,
              "character": 14
            }
          }
        },
        {
          "uri": "file:///test.mzn",
          "range": {
            "start": {
              "line": 3,
              "character": 9
            },
            "end": {
              "line": 3,
              "character": 14
            }
          }
        }
      ]
    }"#]),
		)
	}
}
