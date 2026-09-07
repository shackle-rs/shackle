use lsp_server::{ErrorCode::InvalidRequest, ResponseError};
use lsp_types::{
	AnnotatedTextEdit, ChangeAnnotation, DocumentChanges, OneOf,
	OptionalVersionedTextDocumentIdentifier, Position, RenameParams, TextDocumentEdit, TextEdit,
	Uri, WorkspaceEdit, request::Rename,
};
use shackle_hir::{
	Identifier, RenameCheck, db::CompilerDatabase, input::ModelFile, source::find_leaf,
};
use shackle_syntax::minizinc::pretty_print_identifier;

use crate::{
	db::LanguageServerContext,
	dispatch::RequestHandler,
	utils::{node_ref_to_location, position_to_byte_offset, uri_to_path},
};

#[derive(Debug)]
pub(crate) struct RenameHandler;

pub(crate) struct SymbolHandlerData {
	model_ref: ModelFile,
	cursor_pos: Position,
	new_name: String,
	workspace_uri: Option<Uri>,
}

fn create_error(msg: &str) -> ResponseError {
	ResponseError {
		code: InvalidRequest as i32,
		message: msg.into(),
		data: None,
	}
}

impl RequestHandler<Rename, SymbolHandlerData> for RenameHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: RenameParams,
	) -> Result<SymbolHandlerData, ResponseError> {
		// cannot include single quotes
		if params.new_name.chars().any(|ch| ch == '\'') {
			return Err(create_error("Identifier cannot include single quotes"));
		}

		// the file it is in
		let model_ref =
			db.set_active_file_from_document(&params.text_document_position.text_document)?;

		// pretty print it to add single quotes, etc as necessary
		let new_name = pretty_print_identifier(&params.new_name);

		Ok(SymbolHandlerData {
			cursor_pos: params.text_document_position.position,
			new_name,
			model_ref,
			workspace_uri: db.get_options().workspace_uri.clone(),
		})
	}

	fn execute(
		db: &CompilerDatabase,
		data: SymbolHandlerData,
	) -> Result<Option<WorkspaceEdit>, ResponseError> {
		let byte_offset = position_to_byte_offset(&data.model_ref.contents(db), data.cursor_pos)
			.ok_or_else(|| create_error("Invalid position"))?;
		let entity = find_leaf(db, data.model_ref, byte_offset)
			.or_else(|| find_leaf(db, data.model_ref, byte_offset.saturating_sub(1)))
			.ok_or_else(|| create_error("No symbol found at cursor position"))?;
		let declaration = entity
			.declaration(db)
			.ok_or_else(|| create_error("No declaration found for symbol"))?;
		let workspace_path = data
			.workspace_uri
			.map(|uri| uri_to_path(&uri))
			.unwrap_or_else(|| match entity.item(db).model_file(db) {
				ModelFile::Named(n) => n.path(db).clone(),
				_ => unreachable!(),
			});

		let check = RenameCheck::check(db, declaration, Identifier::new(db, &data.new_name));

		if !declaration
			.item(db)
			.model_file(db)
			.unwrap_named()
			.path(db)
			.starts_with(&workspace_path)
		{
			return Err(create_error(
				"Cannot rename symbols in files ouside workspace",
			));
		}

		let decl_loc = node_ref_to_location(db, declaration.into_entity(db))
			.ok_or_else(|| create_error("Failed to get location of symbol declaration"))?;
		let mut changes = vec![TextDocumentEdit {
			edits: vec![OneOf::Right(AnnotatedTextEdit {
				annotation_id: "rename".to_owned(),
				text_edit: TextEdit::new(decl_loc.range, data.new_name.clone()),
			})],
			text_document: OptionalVersionedTextDocumentIdentifier {
				uri: decl_loc.uri,
				version: None,
			},
		}];

		let references = declaration.references(db);
		for reference in references {
			if !reference
				.item(db)
				.model_file(db)
				.unwrap_named()
				.path(db)
				.starts_with(&workspace_path)
			{
				return Err(create_error(
					"Cannot rename symbols in files ouside workspace",
				));
			}

			let ref_loc = node_ref_to_location(db, reference)
				.ok_or_else(|| create_error("Failed to get location of symbol reference"))?;
			changes.push(TextDocumentEdit {
				edits: vec![OneOf::Right(AnnotatedTextEdit {
					annotation_id: "rename".to_owned(),
					text_edit: TextEdit::new(ref_loc.range, data.new_name.clone()),
				})],
				text_document: OptionalVersionedTextDocumentIdentifier {
					uri: ref_loc.uri,
					version: None,
				},
			});
		}

		Ok(Some(WorkspaceEdit {
			document_changes: Some(DocumentChanges::Edits(changes)),
			change_annotations: Some(
				[(
					"rename".to_owned(),
					ChangeAnnotation {
						label: if matches!(check, RenameCheck::Ok) {
							"Rename"
						} else {
							"Rename confict"
						}
						.to_owned(),
						needs_confirmation: Some(!matches!(check, RenameCheck::Ok)),
						description: match check {
							RenameCheck::Ok => None,
							RenameCheck::IdentifierAlreadyDefined => {
								Some("Renaming this symbol will cause a naming conflict".to_owned())
							}
							RenameCheck::ShadowConflict => Some(
								"Renaming this symbol will change the meaning of the program"
									.to_owned(),
							),
							RenameCheck::InvalidOverload => Some(
								"Renaming this symbol will cause invalid function overloading"
									.to_owned(),
							),
						},
					},
				)]
				.into_iter()
				.collect(),
			),
			..Default::default()
		}))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::{RenameParams, TextDocumentPositionParams, Uri};

	use super::RenameHandler;
	use crate::handlers::tests::test_handler;

	#[test]
	fn test_references() {
		test_handler::<RenameHandler, _, _>(
			r#"
var int: x;
any: y = let {
    int: x = 1;
} in x + let {
    int: x = 2;
} in x;
any: z = x;
			"#,
			false,
			RenameParams {
				new_name: "abc 123 !@# \"".to_owned(),
				text_document_position: TextDocumentPositionParams {
					text_document: lsp_types::TextDocumentIdentifier {
						uri: Uri::from_str("file:///test.mzn").unwrap(),
					},
					position: lsp_types::Position {
						line: 6,
						character: 6,
					},
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
			},
			expect!([r#"
    {
      "Ok": {
        "documentChanges": [
          {
            "textDocument": {
              "uri": "file:///test.mzn",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 5,
                    "character": 9
                  },
                  "end": {
                    "line": 5,
                    "character": 10
                  }
                },
                "newText": "'abc 123 !@# \"'",
                "annotationId": "rename"
              }
            ]
          },
          {
            "textDocument": {
              "uri": "file:///test.mzn",
              "version": null
            },
            "edits": [
              {
                "range": {
                  "start": {
                    "line": 6,
                    "character": 5
                  },
                  "end": {
                    "line": 6,
                    "character": 6
                  }
                },
                "newText": "'abc 123 !@# \"'",
                "annotationId": "rename"
              }
            ]
          }
        ],
        "changeAnnotations": {
          "rename": {
            "label": "Rename",
            "needsConfirmation": false
          }
        }
      }
    }"#]),
		)
	}
}
