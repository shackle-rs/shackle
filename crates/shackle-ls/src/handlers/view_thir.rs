use lsp_server::ResponseError;
use lsp_types::TextDocumentPositionParams;
use shackle_fmt::{MiniZincFormatOptions, format_str};
use shackle_hir::{db::CompilerDatabase, input::ModelFile, run_hir_phase};
use shackle_thir::{db::final_thir, lower::lower_model, pretty_print::PrettyPrinter};

use crate::{
	db::LanguageServerContext,
	dispatch::RequestHandler,
	extensions::{ViewLoweredThir, ViewTransformedThir},
};

#[derive(Debug)]
pub(crate) struct ViewTransformedThirHandler;

impl RequestHandler<ViewTransformedThir, ModelFile> for ViewTransformedThirHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelFile, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}

	fn execute(db: &CompilerDatabase, _: ModelFile) -> Result<String, ResponseError> {
		view_thir(db, true)
	}
}

#[derive(Debug)]
pub(crate) struct ViewLoweredThirHandler;

impl RequestHandler<ViewLoweredThir, ModelFile> for ViewLoweredThirHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: TextDocumentPositionParams,
	) -> Result<ModelFile, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}

	fn execute(db: &CompilerDatabase, _: ModelFile) -> Result<String, ResponseError> {
		view_thir(db, false)
	}
}

fn view_thir(db: &CompilerDatabase, transformed: bool) -> Result<String, ResponseError> {
	let result = run_hir_phase(db);
	if result.errors.is_empty() {
		let lowered;
		let thir = if transformed {
			match final_thir(db) {
				Ok(m) => m,
				Err(e) => return Ok(format!("%: THIR error: {}", e)),
			}
		} else {
			lowered = lower_model(db).take();
			&lowered
		};
		let printer = PrettyPrinter::new(db, thir);
		let text = printer.pretty_print();
		if let Ok(f) = format_str(
			&text,
			&MiniZincFormatOptions {
				keep_parentheses: false,
				..Default::default()
			},
		) {
			return Ok(f);
		}
		Ok(format!("% Failed to format parsed text:\n{}", text))
	} else {
		Ok("% Errors present.".to_owned())
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use lsp_types::{TextDocumentIdentifier, Uri};

	use super::*;
	use crate::handlers::tests::run_handler;

	fn params() -> TextDocumentPositionParams {
		TextDocumentPositionParams {
			text_document: TextDocumentIdentifier {
				uri: Uri::from_str("file:///test.mzn").unwrap(),
			},
			position: Default::default(),
		}
	}

	#[test]
	fn views_show_distinct_thir_stages() {
		let model = "enum E = { A, B }; E: x = A; output [show(x)];";
		let lowered =
			run_handler::<ViewLoweredThirHandler, ViewLoweredThir, _>(model, false, params())
				.unwrap();
		let transformed = run_handler::<ViewTransformedThirHandler, ViewTransformedThir, _>(
			model,
			false,
			params(),
		)
		.unwrap();
		assert!(lowered.contains("enum E"), "{lowered}");
		assert!(!transformed.starts_with('%'), "{transformed}");
		assert!(!transformed.contains("enum E"), "{transformed}");
		assert!(transformed.contains("solve satisfy;"), "{transformed}");
	}

	#[test]
	fn views_report_model_errors() {
		let model = "int: x = missing;";
		let lowered =
			run_handler::<ViewLoweredThirHandler, ViewLoweredThir, _>(model, true, params())
				.unwrap();
		let transformed = run_handler::<ViewTransformedThirHandler, ViewTransformedThir, _>(
			model,
			true,
			params(),
		)
		.unwrap();
		assert_eq!(lowered, "% Errors present.");
		assert_eq!(transformed, "% Errors present.");
	}
}
