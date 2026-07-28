use lsp_server::ResponseError;
use lsp_types::{Position, TextDocumentPositionParams};
use shackle_hir::{db::CompilerDatabase, input::ModelFile, source::find_item};

use crate::{
	db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewHir,
	utils::position_to_byte_offset,
};

#[derive(Debug)]
pub(crate) struct ViewHirHandler;

impl RequestHandler<ViewHir, (ModelFile, Position)> for ViewHirHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<(ModelFile, Position), ResponseError> {
		let model_ref = db.set_active_file_from_document(&params.text_document)?;
		Ok((model_ref, params.position))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, position): (ModelFile, Position),
	) -> Result<String, ResponseError> {
		let Some(byte_offset) = position_to_byte_offset(&model_ref.contents(db), position) else {
			return Ok("Invalid position.".to_owned());
		};
		let Some(item) = find_item(db, model_ref, byte_offset) else {
			return Ok("Not an item.".to_owned());
		};
		let item_info = item.get_item_with_data_as_debug(db);
		let types = item.types(db);
		let result = shackle_hir::db::attach(db, || format!("{:#?}\n\n{:#?}", item_info, types));
		Ok(result)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::ViewHirHandler;
	use crate::handlers::tests::test_handler_display;

	#[test]
	fn test_view_hir() {
		test_handler_display::<ViewHirHandler, _, _>(
			r#"
function var int: foo(opt int: a);
var {1, 2, 3}: x = foo(<>);
			"#,
			false,
			lsp_types::TextDocumentPositionParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				position: lsp_types::Position {
					line: 2,
					character: 0,
				},
			},
			expect!([r#"
    ItemWithData {
        item: Declaration {
            declared_type: <Type::1>,
            pattern: <Pattern::1>,
            definition: Some(
                <Expression::7>,
            ),
            annotations: [],
        },
        data: ItemData {
            expressions: Arena {
                len: 7,
                data: {
                    <Expression::1>: IntegerLiteral(
                        1,
                    ),
                    <Expression::2>: IntegerLiteral(
                        2,
                    ),
                    <Expression::3>: IntegerLiteral(
                        3,
                    ),
                    <Expression::4>: SetLiteral {
                        members: [
                            <Expression::1>,
                            <Expression::2>,
                            <Expression::3>,
                        ],
                    },
                    <Expression::5>: Identifier(
                        "foo",
                    ),
                    <Expression::6>: Absent,
                    <Expression::7>: Call {
                        kind: SourceCall,
                        function: <Expression::5>,
                        arguments: [
                            <Expression::6>,
                        ],
                        named_arguments: [],
                    },
                },
            },
            types: Arena {
                len: 1,
                data: {
                    <Type::1>: Bounded {
                        inst: Some(
                            Var,
                        ),
                        opt: None,
                        domain: <Expression::4>,
                    },
                },
            },
            patterns: Arena {
                len: 1,
                data: {
                    <Pattern::1>: Identifier(
                        Identifier(
                            "x",
                        ),
                    ),
                },
            },
            annotations: {},
        },
    }

    TypeResult {
        patterns: {
            <Pattern::1>: Variable(
                var int,
            ),
        },
        expressions: {
            <Expression::1>: int,
            <Expression::2>: int,
            <Expression::3>: int,
            <Expression::4>: set of int,
            <Expression::5>: op(var int: (opt int)),
            <Expression::6>: opt ..,
            <Expression::7>: var int,
        },
        identifier_resolutions: {
            <Expression::5>: PatternRef {
                item: test.mzn:2.1-34,
                pattern: <Pattern::1>,
            },
        },
        pattern_resolutions: {},
    }"#]),
		)
	}
}
