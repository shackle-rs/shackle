use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_hir::{db::CompilerDatabase, input::ModelFile};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewAst};

#[derive(Debug)]
pub(crate) struct ViewAstHandler;

impl RequestHandler<ViewAst, ModelFile> for ViewAstHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelFile, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}

	fn execute(db: &CompilerDatabase, model_ref: ModelFile) -> Result<String, ResponseError> {
		Ok(format!("{:#?}", model_ref.ast(db).ast(db)))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::ViewAstHandler;
	use crate::handlers::tests::test_handler_display;

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
					uri: Uri::from_str("file:///test.mzn").unwrap(),
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
                                cardinality: None,
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
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                                default: None,
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
                                            },
                                        ),
                                    ),
                                ),
                                annotations: [],
                                default: None,
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
                                                    },
                                                ),
                                            ),
                                            arguments: [
                                                ArgOrParam {
                                                    cst_kind: "arg_or_param",
                                                    left: TypeBase(
                                                        TypeBase {
                                                            cst_kind: "type_base",
                                                            var_type: None,
                                                            opt_type: None,
                                                            any_type: false,
                                                            domain: Bounded(
                                                                IntegerLiteral(
                                                                    IntegerLiteral {
                                                                        cst_kind: "integer_literal",
                                                                    },
                                                                ),
                                                            ),
                                                        },
                                                    ),
                                                    right: None,
                                                    default: None,
                                                },
                                                ArgOrParam {
                                                    cst_kind: "arg_or_param",
                                                    left: TypeBase(
                                                        TypeBase {
                                                            cst_kind: "type_base",
                                                            var_type: None,
                                                            opt_type: None,
                                                            any_type: false,
                                                            domain: Bounded(
                                                                IntegerLiteral(
                                                                    IntegerLiteral {
                                                                        cst_kind: "integer_literal",
                                                                    },
                                                                ),
                                                            ),
                                                        },
                                                    ),
                                                    right: None,
                                                    default: None,
                                                },
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
