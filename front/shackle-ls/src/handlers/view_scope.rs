use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{
		db::Hir,
		ids::{LocalEntityRef, NodeRef},
		source::{find_node, Point},
	},
};
use std::fmt::Write;

use crate::{dispatch::RequestHandler, extensions::ViewScope, LanguageServerDatabase};

#[derive(Debug)]
pub struct ViewScopeHandler;

impl RequestHandler<ViewScope, (ModelRef, Point)> for ViewScopeHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
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
		if let Some(NodeRef::Entity(e)) = found {
			if let LocalEntityRef::Expression(expr) = e.entity(db) {
				let scopes = db.lookup_item_scope(e.item(db));
				let mut fns = Vec::new();
				let mut vars = Vec::new();
				for (i, r) in scopes.functions_in_scope(db, expr) {
					fns.push(format!("{} ({} overloads)", i.pretty_print(db), r.len()));
				}
				for (i, _) in scopes.variables_in_scope(db, expr) {
					vars.push(i.pretty_print(db));
				}
				fns.sort();
				vars.sort();
				let mut out = String::new();
				writeln!(&mut out, "Scope for current expression:").unwrap();
				writeln!(&mut out, "  Functions:",).unwrap();
				for f in fns {
					writeln!(&mut out, "    {}", f).unwrap();
				}
				writeln!(&mut out, "  Variables:",).unwrap();
				for v in vars {
					writeln!(&mut out, "    {}", v).unwrap();
				}
				return Ok(out);
			}
		}
		Ok("Not an expression.".to_owned())
	}
}
