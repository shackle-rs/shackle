use lsp_server::ResponseError;
use lsp_types::{request::GotoDefinition, GotoDefinitionParams, GotoDefinitionResponse};
use shackle_compiler::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{
		db::Hir,
		ids::{LocalEntityRef, NodeRef, PatternRef},
		source::{find_node, Point},
	},
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::node_ref_to_location};

#[derive(Debug)]
pub struct GotoDefinitionHandler;

impl RequestHandler<GotoDefinition, (ModelRef, Point)> for GotoDefinitionHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
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
		let found = find_node(db, *model_ref, start, start);
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
							let resolution = types
								.pattern_resolution(p)
								.unwrap_or_else(|| PatternRef::new(item, p));
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

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use super::GotoDefinitionHandler;
	use crate::handlers::test::test_handler;

	#[test]
	fn test_goto_definition_1() {
		test_handler::<GotoDefinitionHandler, _, _>(
			r#"
int: hello;
int: y = hello + 1;
int: z = hello + let { int: hello = int; } in hello;
			"#,
			false,
			lsp_types::GotoDefinitionParams {
				partial_result_params: lsp_types::PartialResultParams {
					partial_result_token: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Url::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 2,
						character: 11,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
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
      }
    }"#]),
		)
	}

	#[test]
	fn test_goto_definition_2() {
		test_handler::<GotoDefinitionHandler, _, _>(
			r#"
int: hello;
int: y = hello + 1;
int: z = hello + let { int: hello = int; } in hello;
			"#,
			false,
			lsp_types::GotoDefinitionParams {
				partial_result_params: lsp_types::PartialResultParams {
					partial_result_token: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Url::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 3,
						character: 48,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
        "uri": "file:///test.mzn",
        "range": {
          "start": {
            "line": 3,
            "character": 28
          },
          "end": {
            "line": 3,
            "character": 33
          }
        }
      }
    }"#]),
		)
	}
}
