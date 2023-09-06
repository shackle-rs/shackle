use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_compiler::{db::CompilerDatabase, file::ModelRef, syntax::db::SourceParser};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewCst};

#[derive(Debug)]
pub struct ViewCstHandler;

impl RequestHandler<ViewCst, ModelRef> for ViewCstHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(db: &CompilerDatabase, model_ref: ModelRef) -> Result<String, ResponseError> {
		match db.cst(*model_ref) {
			Ok(cst) => {
				let mut w = String::new();
				cst.debug_print(&mut w);
				Ok(w)
			}
			Err(e) => Ok(e.to_string()),
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use crate::handlers::test::test_handler_display;

	use super::ViewCstHandler;

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
					uri: Url::from_str("file:///test.mzn").unwrap(),
				},
				position: lsp_types::Position {
					line: 0,
					character: 0,
				},
			},
			expect!([r#"
    kind="source_file", named=true, error=false, missing=false, extra=false, field=None
      kind="function_item", named=true, error=false, missing=false, extra=false, field=Some("item")
        kind="function", named=false, error=false, missing=false, extra=false, field=None
        kind="set_type", named=true, error=false, missing=false, extra=false, field=Some("type")
          kind="set", named=false, error=false, missing=false, extra=false, field=None
          kind="of", named=false, error=false, missing=false, extra=false, field=None
          kind="type_base", named=true, error=false, missing=false, extra=false, field=Some("type")
            kind="primitive_type", named=true, error=false, missing=false, extra=false, field=Some("domain")
              kind="int", named=false, error=false, missing=false, extra=false, field=None
        kind=":", named=false, error=false, missing=false, extra=false, field=None
        kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("name")
        kind="(", named=false, error=false, missing=false, extra=false, field=None
        kind="parameter", named=true, error=false, missing=false, extra=false, field=Some("parameter")
          kind="type_base", named=true, error=false, missing=false, extra=false, field=Some("type")
            kind="primitive_type", named=true, error=false, missing=false, extra=false, field=Some("domain")
              kind="int", named=false, error=false, missing=false, extra=false, field=None
          kind=":", named=false, error=false, missing=false, extra=false, field=None
          kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("name")
        kind=",", named=false, error=false, missing=false, extra=false, field=None
        kind="parameter", named=true, error=false, missing=false, extra=false, field=Some("parameter")
          kind="type_base", named=true, error=false, missing=false, extra=false, field=Some("type")
            kind="primitive_type", named=true, error=false, missing=false, extra=false, field=Some("domain")
              kind="int", named=false, error=false, missing=false, extra=false, field=None
          kind=":", named=false, error=false, missing=false, extra=false, field=None
          kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("name")
        kind=")", named=false, error=false, missing=false, extra=false, field=None
        kind="=", named=false, error=false, missing=false, extra=false, field=None
        kind="infix_operator", named=true, error=false, missing=false, extra=false, field=Some("body")
          kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("left")
          kind="..", named=false, error=false, missing=false, extra=false, field=Some("operator")
          kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("right")
      kind=";", named=false, error=false, missing=false, extra=false, field=None
      kind="declaration", named=true, error=false, missing=false, extra=false, field=Some("item")
        kind="type_base", named=true, error=false, missing=false, extra=false, field=Some("type")
          kind="primitive_type", named=true, error=false, missing=false, extra=false, field=Some("domain")
            kind="int", named=false, error=false, missing=false, extra=false, field=None
        kind=":", named=false, error=false, missing=false, extra=false, field=None
        kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("name")
        kind="=", named=false, error=false, missing=false, extra=false, field=None
        kind="integer_literal", named=true, error=false, missing=false, extra=false, field=Some("definition")
      kind=";", named=false, error=false, missing=false, extra=false, field=None
      kind="declaration", named=true, error=false, missing=false, extra=false, field=Some("item")
        kind="type_base", named=true, error=false, missing=false, extra=false, field=Some("type")
          kind="var", named=false, error=false, missing=false, extra=false, field=Some("var_par")
          kind="call", named=true, error=false, missing=false, extra=false, field=Some("domain")
            kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("function")
            kind="(", named=false, error=false, missing=false, extra=false, field=None
            kind="integer_literal", named=true, error=false, missing=false, extra=false, field=Some("argument")
            kind=",", named=false, error=false, missing=false, extra=false, field=None
            kind="integer_literal", named=true, error=false, missing=false, extra=false, field=Some("argument")
            kind=")", named=false, error=false, missing=false, extra=false, field=None
        kind=":", named=false, error=false, missing=false, extra=false, field=None
        kind="identifier", named=true, error=false, missing=false, extra=false, field=Some("name")
      kind=";", named=false, error=false, missing=false, extra=false, field=None
"#]),
		)
	}
}
