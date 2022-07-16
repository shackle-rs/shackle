use lsp_server::{ErrorCode, ResponseError};
use lsp_types::TextDocumentPositionParams;
use shackle::{
	file::InputFile,
	hir::{
		db::Hir,
		ids::{LocalEntityRef, NodeRef},
		source::{find_node, Point},
	},
};
use std::fmt::Write;
use std::sync::Arc;

pub fn view_scope(
	db: &mut dyn Hir,
	params: TextDocumentPositionParams,
) -> Result<String, ResponseError> {
	let path = params
		.text_document
		.uri
		.to_file_path()
		.map_err(|_| ResponseError {
			code: ErrorCode::InvalidParams as i32,
			data: None,
			message: "Failed to convert to file path".to_owned(),
		})?;
	db.set_input_files(Arc::new(vec![InputFile::Path(path)]));
	let model_ref = db.input_models()[0];

	let start = Point {
		row: params.position.line as usize,
		column: params.position.character as usize,
	};
	let line = Point {
		row: start.row,
		column: 0,
	};
	let found =
		find_node(db, *model_ref, start, start).or_else(|| find_node(db, *model_ref, line, line));
	if let Some(NodeRef::Entity(e)) = found {
		if let LocalEntityRef::Expression(expr) = e.entity(db) {
			let scopes = db.lookup_item_scope(e.item(db));
			let mut fns = Vec::new();
			let mut vars = Vec::new();
			for (i, r) in scopes.functions_in_scope(db, expr) {
				fns.push(format!("{} ({} overloads)", i.lookup(db), r.len()));
			}
			for (i, _) in scopes.variables_in_scope(db, expr) {
				vars.push(i.lookup(db));
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
