use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{
	Hover, HoverContents, HoverParams, LanguageString, MarkedString, Position,
	request::HoverRequest,
};
use shackle_hir::{db::CompilerDatabase, ids::EntityId, input::ModelFile, source::find_leaf};

use crate::{
	db::LanguageServerContext,
	dispatch::RequestHandler,
	utils::{node_ref_to_location, position_to_byte_offset},
};

#[derive(Debug)]
pub(crate) struct HoverHandler;

impl RequestHandler<HoverRequest, (ModelFile, Position)> for HoverHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: HoverParams,
	) -> Result<(ModelFile, Position), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position_params.text_document)?;
		let position = params.text_document_position_params.position;
		Ok((model, position))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelFile, Position),
	) -> Result<Option<Hover>, ResponseError> {
		let byte_offset =
			position_to_byte_offset(&model_ref.contents(db), start).ok_or_else(|| {
				ResponseError {
					code: ErrorCode::InvalidRequest as i32,
					message: "Invalid position.".to_owned(),
					data: None,
				}
			})?;
		let Some(found) = find_leaf(db, model_ref, byte_offset) else {
			return Ok(None);
		};
		let item = found.item(db);
		let data = item.data(db);
		let types = item.types(db);
		let range = node_ref_to_location(db, found).map(|loc| loc.range);
		let value = match found.entity(db) {
			EntityId::Expression(e) => types.pretty_print_expression_ty(data, e),
			EntityId::Pattern(p) => types.pretty_print_pattern_ty(data, p),
			_ => None,
		};

		Ok(value.map(|value| Hover {
			contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
				language: "minizinc".to_owned(),
				value,
			})),
			range,
		}))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::HoverHandler;
	use crate::handlers::tests::test_handler;

	#[test]
	fn test_hover() {
		test_handler::<HoverHandler, _, _>(
			r#"
type Foo = tuple(int, int);
Foo: x;
any: y = x.1;
			"#,
			false,
			lsp_types::HoverParams {
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 3,
						character: 9,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
        "contents": {
          "language": "minizinc",
          "value": "tuple(int, int)"
        },
        "range": {
          "start": {
            "line": 3,
            "character": 9
          },
          "end": {
            "line": 3,
            "character": 10
          }
        }
      }
    }"#]),
		)
	}

	#[test]
	fn test_hover_objective_identifier() {
		test_handler::<HoverHandler, _, _>(
			r#"
int: foo;
int: bar;
solve minimize foo + bar;
			"#,
			false,
			lsp_types::HoverParams {
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 3,
						character: 16,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
        "contents": {
          "language": "minizinc",
          "value": "int"
        },
        "range": {
          "start": {
            "line": 3,
            "character": 15
          },
          "end": {
            "line": 3,
            "character": 18
          }
        }
      }
    }"#]),
		)
	}
}
