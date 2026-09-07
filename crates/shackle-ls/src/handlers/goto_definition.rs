use lsp_server::ResponseError;
use lsp_types::{
	GotoDefinitionParams, GotoDefinitionResponse, Location, Position, Range,
	request::GotoDefinition,
};
use shackle_hir::{db::CompilerDatabase, input::ModelFile, source::find_leaf};

use crate::{
	db::LanguageServerContext,
	dispatch::RequestHandler,
	utils::{node_ref_to_location, path_to_uri, position_to_byte_offset},
};

#[derive(Debug)]
pub(crate) struct GotoDefinitionHandler;

impl RequestHandler<GotoDefinition, (ModelFile, Position)> for GotoDefinitionHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: GotoDefinitionParams,
	) -> Result<(ModelFile, Position), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position_params.text_document)?;
		Ok((model, params.text_document_position_params.position))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelFile, Position),
	) -> Result<Option<GotoDefinitionResponse>, ResponseError> {
		let byte_offset =
			position_to_byte_offset(&model_ref.contents(db), start).ok_or_else(|| {
				ResponseError {
					code: lsp_server::ErrorCode::InvalidParams as i32,
					message: "Invalid position".to_owned(),
					data: None,
				}
			})?;

		let Some(entity) = find_leaf(db, model_ref, byte_offset) else {
			// Could not find anything, so check if it's an include item
			if let Some(include) = model_ref.include_items(db).into_iter().find(|include| {
				let span = include.origin().span;
				span.offset() <= byte_offset && byte_offset < span.offset() + span.len()
			}) {
				return Ok(Some(GotoDefinitionResponse::Scalar(Location {
					uri: path_to_uri(include.file(db)),
					range: Range::default(),
				})));
			}

			return Ok(None);
		};
		let Some(declaration) = entity.declaration(db) else {
			return Ok(None);
		};

		Ok(node_ref_to_location(db, declaration.into_entity(db))
			.map(GotoDefinitionResponse::Scalar))
	}
}

#[cfg(test)]
mod tests {
	use std::{path::Path, str::FromStr};

	use expect_test::expect;
	use lsp_types::{GotoDefinitionResponse, Location, Range, Uri, request::GotoDefinition};

	use super::GotoDefinitionHandler;
	use crate::{
		handlers::tests::{run_handler, test_handler},
		utils::path_to_uri,
	};

	#[test]
	fn test_goto_definition_include() {
		let included = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
		let model = format!("include \"{}\";", included.display());
		let response = run_handler::<GotoDefinitionHandler, GotoDefinition, _>(
			&model,
			true,
			lsp_types::GotoDefinitionParams {
				partial_result_params: lsp_types::PartialResultParams {
					partial_result_token: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 0,
						character: 10,
					},
				},
			},
		)
		.unwrap();

		assert_eq!(
			response,
			Some(GotoDefinitionResponse::Scalar(Location {
				uri: path_to_uri(&included),
				range: Range::default(),
			}))
		);
	}

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
						uri: Uri::from_str("file:///test.mzn").unwrap(),
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
						uri: Uri::from_str("file:///test.mzn").unwrap(),
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
