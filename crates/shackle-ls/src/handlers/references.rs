use lsp_server::ResponseError;
use lsp_types::{Location, Position, ReferenceParams, request::References};
use shackle_hir::{db::CompilerDatabase, input::ModelFile, source::find_leaf};

use crate::{
	db::LanguageServerContext,
	dispatch::RequestHandler,
	utils::{node_ref_to_location, position_to_byte_offset},
};

#[derive(Debug)]
pub(crate) struct ReferencesHandler;

#[derive(Debug)]
pub(crate) struct ReferencesHandlerData {
	model_ref: ModelFile,
	point: Position,
	include_decl: bool,
}

impl RequestHandler<References, ReferencesHandlerData> for ReferencesHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: ReferenceParams,
	) -> Result<ReferencesHandlerData, ResponseError> {
		let model_ref =
			db.set_active_file_from_document(&params.text_document_position.text_document)?;
		Ok(ReferencesHandlerData {
			model_ref,
			point: params.text_document_position.position,
			include_decl: params.context.include_declaration,
		})
	}

	fn execute(
		db: &CompilerDatabase,
		config: ReferencesHandlerData,
	) -> Result<Option<Vec<Location>>, ResponseError> {
		let byte_offset = position_to_byte_offset(&config.model_ref.contents(db), config.point)
			.ok_or_else(|| ResponseError {
				code: lsp_server::ErrorCode::InvalidParams as i32,
				message: "Invalid position".to_owned(),
				data: None,
			})?;

		let Some(entity) = find_leaf(db, config.model_ref, byte_offset) else {
			return Ok(None);
		};
		let Some(declaration) = entity.declaration(db) else {
			return Ok(None);
		};
		let mut references = declaration.references(db);
		if config.include_decl {
			references.insert(0, declaration.into_entity(db));
		}
		Ok(references
			.into_iter()
			.map(|reference| node_ref_to_location(db, reference))
			.collect::<Option<Vec<_>>>())
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::ReferencesHandler;
	use crate::handlers::tests::test_handler;

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
						uri: Uri::from_str("file:///test.mzn").unwrap(),
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
