use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{
		db::Hir,
		ids::NodeRef,
		source::{find_node, Point},
	},
	utils::DebugPrint,
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, extensions::ViewHir};

#[derive(Debug)]
pub struct ViewHirHandler;

impl RequestHandler<ViewHir, (ModelRef, Point)> for ViewHirHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model_ref = db.set_active_file_from_document(&params.text_document)?;
		let start = Point {
			row: params.position.line as usize,
			column: params.position.character as usize,
		};
		Ok((model_ref, start))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelRef, Point),
	) -> Result<String, ResponseError> {
		let line = Point {
			row: start.row,
			column: 0,
		};
		let found = find_node(db, *model_ref, start, start)
			.or_else(|| find_node(db, *model_ref, line, line));
		if let Some(node) = found {
			let item = match node {
				NodeRef::Entity(e) => e.item(db),
				NodeRef::Item(i) => i,
				_ => return Ok("".to_owned()),
			};
			let item_info = item.debug_print(db);
			let types = db.lookup_item_types(item);
			let type_info = types.debug_print(db);

			Ok(format!("{}\n{}", item_info, type_info))
		} else {
			Ok("Not an item.".to_owned())
		}
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use crate::handlers::test::test_handler_display;

	use super::ViewHirHandler;

	#[test]
	fn test_view_hir() {
		test_handler_display::<ViewHirHandler, _, _>(
			r#"
function var int: foo(opt int: a);
var 1..3: x = foo(<>);
			"#,
			false,
			lsp_types::TextDocumentPositionParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Url::from_str("file:///test.mzn").unwrap(),
				},
				position: lsp_types::Position {
					line: 2,
					character: 0,
				},
			},
			expect!([r#"
    Item: Declaration { declared_type: <Type::1>, pattern: <Pattern::1>, definition: Some(<Expression::7>), annotations: [] }
      Expressions:
        <Expression::1>: IntegerLiteral(1)
        <Expression::2>: IntegerLiteral(3)
        <Expression::3>: Identifier("..")
        <Expression::4>: Call { function: <Expression::3>, arguments: [<Expression::1>, <Expression::2>] }
        <Expression::5>: Absent
        <Expression::6>: Identifier("foo")
        <Expression::7>: Call { function: <Expression::6>, arguments: [<Expression::5>] }
      Types:
        <Type::1>: Bounded { inst: Some(Var), opt: None, domain: <Expression::4> }
      Patterns:
        <Pattern::1>: Identifier(Identifier("x"))
      Annotations:

    Computed types:
      Declarations:
        <Pattern::1>: Variable(var int)
      Expressions:
        <Expression::1>: int
        <Expression::2>: int
        <Expression::3>: op(set of int: (int, int))
        <Expression::4>: set of int
        <Expression::5>: opt ..
        <Expression::6>: op(var int: (opt int))
        <Expression::7>: var int
      Name resolution:
        <Expression::3>: PatternRef(ItemRef(562), <Pattern::1>)
        <Expression::6>: PatternRef(ItemRef(0), <Pattern::1>)
"#]),
		)
	}
}
