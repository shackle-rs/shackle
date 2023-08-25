use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_compiler::{db::CompilerDatabase, file::ModelRef, syntax::db::SourceParser};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewAst};

#[derive(Debug)]
pub struct ViewAstHandler;

impl RequestHandler<ViewAst, ModelRef> for ViewAstHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, model_ref: ModelRef) -> Result<String, ResponseError> {
		match db.ast(*model_ref) {
			Ok(ast) => Ok(format!("{:#?}", ast)),
			Err(e) => Ok(e.to_string()),
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use super::ViewAstHandler;
	use crate::handlers::test::test_handler_display;

	#[test]
	fn test_view_ast() {
		test_handler_display::<ViewAstHandler, _, _>(
			r#"
function set of int: foo(int: a, int: b) = a..b;
int: x = 1;
var foo(1, 3): y;
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
    MznModel(
        Model {
            items: [
                Function(
                    Function {
                        cst_kind: "function_item",
                        return_type: SetType(
                            SetType {
                                cst_kind: "set_type",
                                var_type: Par,
                                opt_type: NonOpt,
                                element_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                        id: UnquotedIdentifier(
                            UnquotedIdentifier {
                                cst_kind: "identifier",
                                name: "foo",
                            },
                        ),
                        parameters: [
                            Parameter {
                                cst_kind: "parameter",
                                declared_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                                pattern: Some(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "a",
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                            },
                            Parameter {
                                cst_kind: "parameter",
                                declared_type: TypeBase(
                                    TypeBase {
                                        cst_kind: "type_base",
                                        var_type: None,
                                        opt_type: None,
                                        any_type: false,
                                        domain: Unbounded(
                                            UnboundedDomain {
                                                cst_kind: "primitive_type",
                                                primitive_type: Int,
                                            },
                                        ),
                                    },
                                ),
                                pattern: Some(
                                    Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                            },
                        ],
                        body: Some(
                            InfixOperator(
                                InfixOperator {
                                    cst_kind: "infix_operator",
                                    left: Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "a",
                                            },
                                        ),
                                    ),
                                    operator: Operator {
                                        cst_kind: "..",
                                        name: "..",
                                    },
                                    right: Identifier(
                                        UnquotedIdentifier(
                                            UnquotedIdentifier {
                                                cst_kind: "identifier",
                                                name: "b",
                                            },
                                        ),
                                    ),
                                },
                            ),
                        ),
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "x",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: None,
                                opt_type: None,
                                any_type: false,
                                domain: Unbounded(
                                    UnboundedDomain {
                                        cst_kind: "primitive_type",
                                        primitive_type: Int,
                                    },
                                ),
                            },
                        ),
                        definition: Some(
                            IntegerLiteral(
                                IntegerLiteral {
                                    cst_kind: "integer_literal",
                                    value: Ok(
                                        1,
                                    ),
                                },
                            ),
                        ),
                        annotations: [],
                    },
                ),
                Declaration(
                    Declaration {
                        cst_kind: "declaration",
                        pattern: Identifier(
                            UnquotedIdentifier(
                                UnquotedIdentifier {
                                    cst_kind: "identifier",
                                    name: "y",
                                },
                            ),
                        ),
                        declared_type: TypeBase(
                            TypeBase {
                                cst_kind: "type_base",
                                var_type: Some(
                                    Var,
                                ),
                                opt_type: None,
                                any_type: false,
                                domain: Bounded(
                                    Call(
                                        Call {
                                            cst_kind: "call",
                                            function: Identifier(
                                                UnquotedIdentifier(
                                                    UnquotedIdentifier {
                                                        cst_kind: "identifier",
                                                        name: "foo",
                                                    },
                                                ),
                                            ),
                                            arguments: [
                                                IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: Ok(
                                                            1,
                                                        ),
                                                    },
                                                ),
                                                IntegerLiteral(
                                                    IntegerLiteral {
                                                        cst_kind: "integer_literal",
                                                        value: Ok(
                                                            3,
                                                        ),
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            },
                        ),
                        definition: None,
                        annotations: [],
                    },
                ),
            ],
        },
    )"#]),
		)
	}
}
