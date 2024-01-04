use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_compiler::{
	db::CompilerDatabase,
	file::ModelRef,
	syntax::{ast::ConstraintModel, db::SourceParser},
};
use shackle_fmt::{format_model_debug, MiniZincFormatOptions};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewFormatIr};

#[derive(Debug)]
pub struct ViewFormatIrHandler;

impl RequestHandler<ViewFormatIr, ModelRef> for ViewFormatIrHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, model_ref: ModelRef) -> Result<String, ResponseError> {
		match db.ast(*model_ref) {
			Ok(ConstraintModel::MznModel(ast)) => {
				Ok(format_model_debug(&ast, &MiniZincFormatOptions::default())
					.unwrap_or_else(|| "Failed to format".to_owned()))
			}
			Ok(_) => todo!("no formatter available for this file type"),
			Err(e) => Ok(e.to_string()),
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use super::ViewFormatIrHandler;
	use crate::handlers::test::test_handler_display;

	#[test]
	fn test_view_format_ir() {
		test_handler_display::<ViewFormatIrHandler, _, _>(
			r#"
      int: x   = (1 + 2) + 3 % foo
      ;
      
      % bar
			"#,
			false,
			lsp_types::TextDocumentPositionParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Url::from_str("file:///test.mzn").unwrap(),
				},
				position: lsp_types::Position {
					line: 0,
					character: 0,
				},
			},
			expect!([r#"
    Element::sequence(
        [
            Element::sequence(
                [
                    Element::sequence(
                        [],
                    ),
                    Element::sequence(
                        [
                            Element::sequence(
                                [
                                    Element::text(
                                        "int",
                                    ),
                                    Element::text(
                                        ": ",
                                    ),
                                    Element::text(
                                        "x",
                                    ),
                                    Element::sequence(
                                        [],
                                    ),
                                    Element::text(
                                        " =",
                                    ),
                                    Element::group(
                                        Element::indent(
                                            Element::sequence(
                                                [
                                                    Element::sequence(
                                                        [
                                                            Element::if_broken(
                                                                Element::line_break(),
                                                            ),
                                                            Element::if_unbroken(
                                                                Element::text(
                                                                    " ",
                                                                ),
                                                            ),
                                                        ],
                                                    ),
                                                    Element::group(
                                                        Element::sequence(
                                                            [
                                                                Element::text(
                                                                    "1",
                                                                ),
                                                                Element::indent(
                                                                    Element::sequence(
                                                                        [
                                                                            Element::text(
                                                                                " ",
                                                                            ),
                                                                            Element::text(
                                                                                "+",
                                                                            ),
                                                                            Element::sequence(
                                                                                [
                                                                                    Element::if_broken(
                                                                                        Element::line_break(),
                                                                                    ),
                                                                                    Element::if_unbroken(
                                                                                        Element::text(
                                                                                            " ",
                                                                                        ),
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            Element::text(
                                                                                "2",
                                                                            ),
                                                                            Element::text(
                                                                                " ",
                                                                            ),
                                                                            Element::text(
                                                                                "+",
                                                                            ),
                                                                            Element::sequence(
                                                                                [
                                                                                    Element::if_broken(
                                                                                        Element::line_break(),
                                                                                    ),
                                                                                    Element::if_unbroken(
                                                                                        Element::text(
                                                                                            " ",
                                                                                        ),
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            Element::text(
                                                                                "3",
                                                                            ),
                                                                        ],
                                                                    ),
                                                                ),
                                                            ],
                                                        ),
                                                    ),
                                                ],
                                            ),
                                        ),
                                    ),
                                ],
                            ),
                            Element::text(
                                ";",
                            ),
                        ],
                    ),
                    Element::sequence(
                        [
                            Element::break_parent(),
                            Element::line_suffix(
                                " % foo",
                            ),
                            Element::line_break(),
                            Element::line_break(),
                            Element::text(
                                "% bar",
                            ),
                        ],
                    ),
                ],
            ),
            Element::line_break(),
        ],
    )"#]),
		)
	}
}
