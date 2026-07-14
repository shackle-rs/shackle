use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{
	Hover, HoverContents, HoverParams, LanguageString, MarkedString, MarkupContent, MarkupKind,
	Position, request::HoverRequest,
};
use shackle_hir::{db::CompilerDatabase, ids::EntityId, input::ModelFile, source::find_leaf};
use shackle_syntax::minizinc::documentation_markdown;

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

		Ok(value.map(|value| {
			let documentation = found
				.declaration(db)
				.and_then(|declaration| {
					let origin = declaration.item(db).documentation(db)?;
					let source = origin.file.contents(db);
					let start = origin.span.offset();
					let end = start + origin.span.len();
					source.get(start..end).map(documentation_markdown)
				})
				.filter(|documentation| !documentation.is_empty());
			let contents = if let Some(documentation) = documentation {
				HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: format!("```minizinc\n{value}\n```\n\n{documentation}"),
				})
			} else {
				HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
					language: "minizinc".to_owned(),
					value,
				}))
			};
			Hover { contents, range }
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

	#[test]
	fn test_hover_tuple_field_access() {
		test_handler::<HoverHandler, _, _>(
			r#"
any: x = (1, 2).2;
			"#,
			true,
			lsp_types::HoverParams {
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 1,
						character: 16,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
        "contents": {
          "language": "minizinc",
          "value": "(tuple field) int"
        },
        "range": {
          "start": {
            "line": 1,
            "character": 16
          },
          "end": {
            "line": 1,
            "character": 17
          }
        }
      }
    }"#]),
		)
	}

	#[test]
	fn test_hover_record_field_access() {
		test_handler::<HoverHandler, _, _>(
			r#"
any: x = (hello: 1).hello;
			"#,
			true,
			lsp_types::HoverParams {
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
				text_document_position_params: lsp_types::TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 1,
						character: 22,
					},
				},
			},
			expect!([r#"
    {
      "Ok": {
        "contents": {
          "language": "minizinc",
          "value": "(record field) int: hello"
        },
        "range": {
          "start": {
            "line": 1,
            "character": 20
          },
          "end": {
            "line": 1,
            "character": 25
          }
        }
      }
    }"#]),
		)
	}

	#[test]
	fn test_hover_documentation() {
		test_handler::<HoverHandler, _, _>(
			r#"
/** @group stdlib.test Add one to \a x.
    @param x: The ``input`` value. */
function int: foo(int: x) = x + 1;
int: y = foo(1);
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
						line: 4,
						character: 10,
					},
				},
			},
			expect![[r#"
    {
      "Ok": {
        "contents": {
          "kind": "markdown",
          "value": "```minizinc\nfunction int: foo(int)\n```\n\nAdd one to `x`.\n\n**Parameters**\n\n- `x`: The `input` value."
        },
        "range": {
          "start": {
            "line": 4,
            "character": 9
          },
          "end": {
            "line": 4,
            "character": 12
          }
        }
      }
    }"#]],
		)
	}

	#[test]
	fn test_hover_declaration_documentation() {
		test_handler::<HoverHandler, _, _>(
			r#"
/** The number of widgets. */
int: widgets;
int: copy = widgets;
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
						character: 13,
					},
				},
			},
			expect![[r#"
    {
      "Ok": {
        "contents": {
          "kind": "markdown",
          "value": "```minizinc\nint\n```\n\nThe number of widgets."
        },
        "range": {
          "start": {
            "line": 3,
            "character": 12
          },
          "end": {
            "line": 3,
            "character": 19
          }
        }
      }
    }"#]],
		)
	}
}
