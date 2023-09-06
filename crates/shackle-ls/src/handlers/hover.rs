use lsp_server::ResponseError;
use lsp_types::{
	request::HoverRequest, Hover, HoverContents, HoverParams, LanguageString, MarkedString,
};
use shackle_compiler::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::LocalEntityRef,
		source::{find_node, Point},
	},
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::node_ref_to_location};

#[derive(Debug)]
pub struct HoverHandler;

impl RequestHandler<HoverRequest, (ModelRef, Point)> for HoverHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: HoverParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position_params.text_document)?;
		let start = Point {
			row: params.text_document_position_params.position.line as usize,
			column: params.text_document_position_params.position.character as usize,
		};
		Ok((model, start))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelRef, Point),
	) -> Result<Option<Hover>, ResponseError> {
		let found = find_node(db, *model_ref, start, start);
		Ok((|| {
			let node = found?;
			match node {
				NodeRef::Entity(e) => {
					let item = e.item(db);
					match e.entity(db) {
						LocalEntityRef::Expression(e) => {
							let types = db.lookup_item_types(item);
							let model = item.model(db);
							let data = item.local_item_ref(db).data(&model);
							let value =
								types.pretty_print_expression_ty(db, data, e).or_else(|| {
									let res = types.name_resolution(e)?;
									let types = db.lookup_item_types(res.item());
									let model = res.item().model(db);
									let data = res.item().local_item_ref(db).data(&model);
									types.pretty_print_pattern_ty(db, data, res.pattern())
								})?;
							Some(Hover {
								contents: HoverContents::Scalar(MarkedString::LanguageString(
									LanguageString {
										language: "minizinc".to_owned(),
										value,
									},
								)),
								range: Some(node_ref_to_location(db, node)?.range),
							})
						}
						LocalEntityRef::Pattern(p) => {
							let types = db.lookup_item_types(item);
							let model = item.model(db);
							let data = item.local_item_ref(db).data(&model);
							Some(Hover {
								contents: HoverContents::Scalar(MarkedString::LanguageString(
									LanguageString {
										language: "minizinc".to_owned(),
										value: types.pretty_print_pattern_ty(db, data, p)?,
									},
								)),
								range: Some(node_ref_to_location(db, node)?.range),
							})
						}
						_ => None,
					}
				}
				_ => None,
			}
		})())
	}
}

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Url;

	use crate::handlers::test::test_handler;

	use super::HoverHandler;

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
						uri: Url::from_str("file:///test.mzn").unwrap(),
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
}
