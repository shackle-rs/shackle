use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_hir::{db::CompilerDatabase, input::ModelFile};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewCst};

#[derive(Debug)]
pub(crate) struct ViewCstHandler;

impl RequestHandler<ViewCst, ModelFile> for ViewCstHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelFile, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}

	fn execute(db: &CompilerDatabase, model_ref: ModelFile) -> Result<String, ResponseError> {
		Ok(format!("{:#?}", model_ref.ast(db).ast(db).cst()))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::ViewCstHandler;
	use crate::handlers::tests::test_handler_display;

	#[test]
	fn test_view_cst() {
		test_handler_display::<ViewCstHandler, _, _>(
			r#"
function set of int: foo(int: a, int: b) = a..b;
int: x = 1;
var foo(1, 3): y;
			"#,
			false,
			lsp_types::TextDocumentPositionParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				position: lsp_types::Position {
					line: 0,
					character: 0,
				},
			},
			expect!([r#"
    CstNode {
        kind: "source_file",
        start: Point {
            row: 1,
            column: 0,
        },
        end: Point {
            row: 4,
            column: 3,
        },
        is_named: true,
        has_error: false,
        is_error: false,
        is_missing: false,
        is_extra: false,
        field: None,
        children: [
            CstNode {
                kind: "function_item",
                start: Point {
                    row: 1,
                    column: 0,
                },
                end: Point {
                    row: 1,
                    column: 47,
                },
                is_named: true,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [
                    CstNode {
                        kind: "function",
                        start: Point {
                            row: 1,
                            column: 0,
                        },
                        end: Point {
                            row: 1,
                            column: 8,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "set_type",
                        start: Point {
                            row: 1,
                            column: 9,
                        },
                        end: Point {
                            row: 1,
                            column: 19,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "set",
                                start: Point {
                                    row: 1,
                                    column: 9,
                                },
                                end: Point {
                                    row: 1,
                                    column: 12,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "of",
                                start: Point {
                                    row: 1,
                                    column: 13,
                                },
                                end: Point {
                                    row: 1,
                                    column: 15,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "type_base",
                                start: Point {
                                    row: 1,
                                    column: 16,
                                },
                                end: Point {
                                    row: 1,
                                    column: 19,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [
                                    CstNode {
                                        kind: "primitive_type",
                                        start: Point {
                                            row: 1,
                                            column: 16,
                                        },
                                        end: Point {
                                            row: 1,
                                            column: 19,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [
                                            CstNode {
                                                kind: "int",
                                                start: Point {
                                                    row: 1,
                                                    column: 16,
                                                },
                                                end: Point {
                                                    row: 1,
                                                    column: 19,
                                                },
                                                is_named: false,
                                                has_error: false,
                                                is_error: false,
                                                is_missing: false,
                                                is_extra: false,
                                                field: None,
                                                children: [],
                                            },
                                        ],
                                    },
                                ],
                            },
                        ],
                    },
                    CstNode {
                        kind: ":",
                        start: Point {
                            row: 1,
                            column: 19,
                        },
                        end: Point {
                            row: 1,
                            column: 20,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "identifier",
                        start: Point {
                            row: 1,
                            column: 21,
                        },
                        end: Point {
                            row: 1,
                            column: 24,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "(",
                        start: Point {
                            row: 1,
                            column: 24,
                        },
                        end: Point {
                            row: 1,
                            column: 25,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "parameter",
                        start: Point {
                            row: 1,
                            column: 25,
                        },
                        end: Point {
                            row: 1,
                            column: 31,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "type_base",
                                start: Point {
                                    row: 1,
                                    column: 25,
                                },
                                end: Point {
                                    row: 1,
                                    column: 28,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [
                                    CstNode {
                                        kind: "primitive_type",
                                        start: Point {
                                            row: 1,
                                            column: 25,
                                        },
                                        end: Point {
                                            row: 1,
                                            column: 28,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [
                                            CstNode {
                                                kind: "int",
                                                start: Point {
                                                    row: 1,
                                                    column: 25,
                                                },
                                                end: Point {
                                                    row: 1,
                                                    column: 28,
                                                },
                                                is_named: false,
                                                has_error: false,
                                                is_error: false,
                                                is_missing: false,
                                                is_extra: false,
                                                field: None,
                                                children: [],
                                            },
                                        ],
                                    },
                                ],
                            },
                            CstNode {
                                kind: ":",
                                start: Point {
                                    row: 1,
                                    column: 28,
                                },
                                end: Point {
                                    row: 1,
                                    column: 29,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "identifier",
                                start: Point {
                                    row: 1,
                                    column: 30,
                                },
                                end: Point {
                                    row: 1,
                                    column: 31,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                        ],
                    },
                    CstNode {
                        kind: ",",
                        start: Point {
                            row: 1,
                            column: 31,
                        },
                        end: Point {
                            row: 1,
                            column: 32,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "parameter",
                        start: Point {
                            row: 1,
                            column: 33,
                        },
                        end: Point {
                            row: 1,
                            column: 39,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "type_base",
                                start: Point {
                                    row: 1,
                                    column: 33,
                                },
                                end: Point {
                                    row: 1,
                                    column: 36,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [
                                    CstNode {
                                        kind: "primitive_type",
                                        start: Point {
                                            row: 1,
                                            column: 33,
                                        },
                                        end: Point {
                                            row: 1,
                                            column: 36,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [
                                            CstNode {
                                                kind: "int",
                                                start: Point {
                                                    row: 1,
                                                    column: 33,
                                                },
                                                end: Point {
                                                    row: 1,
                                                    column: 36,
                                                },
                                                is_named: false,
                                                has_error: false,
                                                is_error: false,
                                                is_missing: false,
                                                is_extra: false,
                                                field: None,
                                                children: [],
                                            },
                                        ],
                                    },
                                ],
                            },
                            CstNode {
                                kind: ":",
                                start: Point {
                                    row: 1,
                                    column: 36,
                                },
                                end: Point {
                                    row: 1,
                                    column: 37,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "identifier",
                                start: Point {
                                    row: 1,
                                    column: 38,
                                },
                                end: Point {
                                    row: 1,
                                    column: 39,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                        ],
                    },
                    CstNode {
                        kind: ")",
                        start: Point {
                            row: 1,
                            column: 39,
                        },
                        end: Point {
                            row: 1,
                            column: 40,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "=",
                        start: Point {
                            row: 1,
                            column: 41,
                        },
                        end: Point {
                            row: 1,
                            column: 42,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "infix_operator",
                        start: Point {
                            row: 1,
                            column: 43,
                        },
                        end: Point {
                            row: 1,
                            column: 47,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "identifier",
                                start: Point {
                                    row: 1,
                                    column: 43,
                                },
                                end: Point {
                                    row: 1,
                                    column: 44,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "..",
                                start: Point {
                                    row: 1,
                                    column: 44,
                                },
                                end: Point {
                                    row: 1,
                                    column: 46,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "identifier",
                                start: Point {
                                    row: 1,
                                    column: 46,
                                },
                                end: Point {
                                    row: 1,
                                    column: 47,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                        ],
                    },
                ],
            },
            CstNode {
                kind: ";",
                start: Point {
                    row: 1,
                    column: 47,
                },
                end: Point {
                    row: 1,
                    column: 48,
                },
                is_named: false,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [],
            },
            CstNode {
                kind: "declaration",
                start: Point {
                    row: 2,
                    column: 0,
                },
                end: Point {
                    row: 2,
                    column: 10,
                },
                is_named: true,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [
                    CstNode {
                        kind: "type_base",
                        start: Point {
                            row: 2,
                            column: 0,
                        },
                        end: Point {
                            row: 2,
                            column: 3,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "primitive_type",
                                start: Point {
                                    row: 2,
                                    column: 0,
                                },
                                end: Point {
                                    row: 2,
                                    column: 3,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [
                                    CstNode {
                                        kind: "int",
                                        start: Point {
                                            row: 2,
                                            column: 0,
                                        },
                                        end: Point {
                                            row: 2,
                                            column: 3,
                                        },
                                        is_named: false,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [],
                                    },
                                ],
                            },
                        ],
                    },
                    CstNode {
                        kind: ":",
                        start: Point {
                            row: 2,
                            column: 3,
                        },
                        end: Point {
                            row: 2,
                            column: 4,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "identifier",
                        start: Point {
                            row: 2,
                            column: 5,
                        },
                        end: Point {
                            row: 2,
                            column: 6,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "=",
                        start: Point {
                            row: 2,
                            column: 7,
                        },
                        end: Point {
                            row: 2,
                            column: 8,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "integer_literal",
                        start: Point {
                            row: 2,
                            column: 9,
                        },
                        end: Point {
                            row: 2,
                            column: 10,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                ],
            },
            CstNode {
                kind: ";",
                start: Point {
                    row: 2,
                    column: 10,
                },
                end: Point {
                    row: 2,
                    column: 11,
                },
                is_named: false,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [],
            },
            CstNode {
                kind: "declaration",
                start: Point {
                    row: 3,
                    column: 0,
                },
                end: Point {
                    row: 3,
                    column: 16,
                },
                is_named: true,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [
                    CstNode {
                        kind: "type_base",
                        start: Point {
                            row: 3,
                            column: 0,
                        },
                        end: Point {
                            row: 3,
                            column: 13,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [
                            CstNode {
                                kind: "var",
                                start: Point {
                                    row: 3,
                                    column: 0,
                                },
                                end: Point {
                                    row: 3,
                                    column: 3,
                                },
                                is_named: false,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [],
                            },
                            CstNode {
                                kind: "call",
                                start: Point {
                                    row: 3,
                                    column: 4,
                                },
                                end: Point {
                                    row: 3,
                                    column: 13,
                                },
                                is_named: true,
                                has_error: false,
                                is_error: false,
                                is_missing: false,
                                is_extra: false,
                                field: None,
                                children: [
                                    CstNode {
                                        kind: "identifier",
                                        start: Point {
                                            row: 3,
                                            column: 4,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 7,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [],
                                    },
                                    CstNode {
                                        kind: "(",
                                        start: Point {
                                            row: 3,
                                            column: 7,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 8,
                                        },
                                        is_named: false,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [],
                                    },
                                    CstNode {
                                        kind: "arg_or_param",
                                        start: Point {
                                            row: 3,
                                            column: 8,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 9,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [
                                            CstNode {
                                                kind: "type_base",
                                                start: Point {
                                                    row: 3,
                                                    column: 8,
                                                },
                                                end: Point {
                                                    row: 3,
                                                    column: 9,
                                                },
                                                is_named: true,
                                                has_error: false,
                                                is_error: false,
                                                is_missing: false,
                                                is_extra: false,
                                                field: None,
                                                children: [
                                                    CstNode {
                                                        kind: "integer_literal",
                                                        start: Point {
                                                            row: 3,
                                                            column: 8,
                                                        },
                                                        end: Point {
                                                            row: 3,
                                                            column: 9,
                                                        },
                                                        is_named: true,
                                                        has_error: false,
                                                        is_error: false,
                                                        is_missing: false,
                                                        is_extra: false,
                                                        field: None,
                                                        children: [],
                                                    },
                                                ],
                                            },
                                        ],
                                    },
                                    CstNode {
                                        kind: ",",
                                        start: Point {
                                            row: 3,
                                            column: 9,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 10,
                                        },
                                        is_named: false,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [],
                                    },
                                    CstNode {
                                        kind: "arg_or_param",
                                        start: Point {
                                            row: 3,
                                            column: 11,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 12,
                                        },
                                        is_named: true,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [
                                            CstNode {
                                                kind: "type_base",
                                                start: Point {
                                                    row: 3,
                                                    column: 11,
                                                },
                                                end: Point {
                                                    row: 3,
                                                    column: 12,
                                                },
                                                is_named: true,
                                                has_error: false,
                                                is_error: false,
                                                is_missing: false,
                                                is_extra: false,
                                                field: None,
                                                children: [
                                                    CstNode {
                                                        kind: "integer_literal",
                                                        start: Point {
                                                            row: 3,
                                                            column: 11,
                                                        },
                                                        end: Point {
                                                            row: 3,
                                                            column: 12,
                                                        },
                                                        is_named: true,
                                                        has_error: false,
                                                        is_error: false,
                                                        is_missing: false,
                                                        is_extra: false,
                                                        field: None,
                                                        children: [],
                                                    },
                                                ],
                                            },
                                        ],
                                    },
                                    CstNode {
                                        kind: ")",
                                        start: Point {
                                            row: 3,
                                            column: 12,
                                        },
                                        end: Point {
                                            row: 3,
                                            column: 13,
                                        },
                                        is_named: false,
                                        has_error: false,
                                        is_error: false,
                                        is_missing: false,
                                        is_extra: false,
                                        field: None,
                                        children: [],
                                    },
                                ],
                            },
                        ],
                    },
                    CstNode {
                        kind: ":",
                        start: Point {
                            row: 3,
                            column: 13,
                        },
                        end: Point {
                            row: 3,
                            column: 14,
                        },
                        is_named: false,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                    CstNode {
                        kind: "identifier",
                        start: Point {
                            row: 3,
                            column: 15,
                        },
                        end: Point {
                            row: 3,
                            column: 16,
                        },
                        is_named: true,
                        has_error: false,
                        is_error: false,
                        is_missing: false,
                        is_extra: false,
                        field: None,
                        children: [],
                    },
                ],
            },
            CstNode {
                kind: ";",
                start: Point {
                    row: 3,
                    column: 16,
                },
                end: Point {
                    row: 3,
                    column: 17,
                },
                is_named: false,
                has_error: false,
                is_error: false,
                is_missing: false,
                is_extra: false,
                field: None,
                children: [],
            },
        ],
    }"#]),
		)
	}
}
