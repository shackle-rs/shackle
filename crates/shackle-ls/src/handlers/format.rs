use lsp_server::ResponseError;
use lsp_types::{request::Formatting, DocumentFormattingParams, Position, TextEdit};
use shackle_compiler::{db::CompilerDatabase, file::ModelRef, syntax::db::SourceParser};
use shackle_fmt::{format_model, FormatOptions, MiniZincFormatOptions};

use crate::{db::LanguageServerContext, dispatch::RequestHandler};

#[derive(Debug)]
pub struct FormatHandler;

impl RequestHandler<Formatting, (ModelRef, MiniZincFormatOptions)> for FormatHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: DocumentFormattingParams,
	) -> Result<(ModelRef, MiniZincFormatOptions), ResponseError> {
		Ok((
			db.set_active_file_from_document(&params.text_document)?,
			MiniZincFormatOptions {
				core: FormatOptions {
					use_tabs: !params.options.insert_spaces,
					indent_size: params.options.tab_size as usize,
					..Default::default()
				},
				..Default::default()
			},
		))
	}
	fn execute(
		db: &CompilerDatabase,
		(model_ref, options): (ModelRef, MiniZincFormatOptions),
	) -> Result<Option<Vec<TextEdit>>, ResponseError> {
		match db.ast(*model_ref) {
			Ok(ast) => {
				let Some(formatted) = format_model(&ast, &options) else {
					return Ok(None);
				};
				let end = ast.cst().root_node().end_position();
				Ok(Some(vec![TextEdit {
					range: lsp_types::Range {
						end: Position::new(end.row as u32, end.column as u32),
						..Default::default()
					},
					new_text: formatted,
				}]))
			}
			Err(_) => Ok(None),
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use super::FormatHandler;
	use crate::handlers::test::test_handler;

	#[test]
	fn test_format() {
		test_handler::<FormatHandler, _, _>(
			r#"
int: x   = (1 + 2) + 3 % foo
;

% bar
			"#,
			false,
			lsp_types::DocumentFormattingParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Url::from_str("file:///test.mzn").unwrap(),
				},
				options: lsp_types::FormattingOptions {
					tab_size: 4,
					insert_spaces: false,
					properties: Default::default(),
					trim_trailing_whitespace: None,
					insert_final_newline: None,
					trim_final_newlines: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
			},
			expect!([r#"
    {
      "Ok": [
        {
          "range": {
            "start": {
              "line": 0,
              "character": 0
            },
            "end": {
              "line": 5,
              "character": 3
            }
          },
          "newText": "int: x = 1 + 2 + 3; % foo\n\n% bar\n"
        }
      ]
    }"#]),
		)
	}
}
