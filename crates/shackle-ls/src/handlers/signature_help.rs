use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{
	Documentation, MarkupContent, MarkupKind, Position, SignatureHelp, SignatureHelpParams,
	SignatureInformation, request::SignatureHelpRequest,
};
use shackle_hir::{
	CallKind, Expression, Identifier, PatternTy, db::CompilerDatabase, ids::PatternRef,
	input::ModelFile, overloading::FunctionEntry, source::find_item,
};
use shackle_syntax::minizinc::documentation_markdown;

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::position_to_byte_offset};

#[derive(Debug)]
pub(crate) struct SignatureHelpHandler;

impl RequestHandler<SignatureHelpRequest, (ModelFile, Position)> for SignatureHelpHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: SignatureHelpParams,
	) -> Result<(ModelFile, Position), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position_params.text_document)?;
		Ok((model, params.text_document_position_params.position))
	}

	fn execute(
		db: &CompilerDatabase,
		(model, position): (ModelFile, Position),
	) -> Result<Option<SignatureHelp>, ResponseError> {
		let contents = model.contents(db);
		let cursor = position_to_byte_offset(&contents, position).ok_or_else(|| ResponseError {
			code: ErrorCode::InvalidParams as i32,
			message: "Invalid position.".to_owned(),
			data: None,
		})?;
		let Some(item) = find_item(db, model, cursor.saturating_sub(1)) else {
			return Ok(None);
		};
		let data = item.data(db);
		let sources = item.sources(db);
		let Some((call, call_start, call_end)) = data
			.expressions
			.iter()
			.filter_map(|(id, expression)| {
				let Expression::Call(call) = expression else {
					return None;
				};
				if call.kind != CallKind::SourceCall {
					return None;
				}
				let span = sources[id].span;
				let start = span.offset();
				let end = start + span.len();
				(start < cursor && cursor <= end).then_some((call, start, end))
			})
			.min_by_key(|(_, start, end)| end - start)
		else {
			return Ok(None);
		};

		let Expression::Identifier(identifier) = data[call.function] else {
			return Ok(None);
		};
		let patterns = item.scope(db).find_function(db, call.function, identifier);
		let mut signatures = Vec::new();
		for pattern in patterns {
			add_signatures(db, pattern, identifier, &mut signatures);
		}
		if signatures.is_empty() {
			return Ok(None);
		}

		let active_parameter =
			active_parameter(&contents[call_start..call_end], cursor - call_start);
		Ok(Some(SignatureHelp {
			signatures,
			active_signature: None,
			active_parameter: Some(active_parameter),
		}))
	}
}

fn add_signatures<'db>(
	db: &'db CompilerDatabase,
	pattern: PatternRef<'db>,
	identifier: Identifier<'db>,
	signatures: &mut Vec<SignatureInformation>,
) {
	let types = pattern.item(db).types(db);
	match &types[pattern.pattern(db)] {
		PatternTy::Function(function)
		| PatternTy::AnnotationConstructor(function)
		| PatternTy::AnnotationDestructure(function) => {
			signatures.push(signature(db, pattern, identifier, function));
		}
		PatternTy::EnumConstructor(constructors) => signatures.extend(
			constructors
				.iter()
				.map(|entry| signature(db, pattern, identifier, &entry.constructor)),
		),
		PatternTy::EnumDestructure(functions) => signatures.extend(
			functions
				.iter()
				.map(|function| signature(db, pattern, identifier, function)),
		),
		_ => {}
	}
}

fn signature<'db>(
	db: &'db CompilerDatabase,
	pattern: PatternRef<'db>,
	identifier: Identifier<'db>,
	function: &FunctionEntry<'db>,
) -> SignatureInformation {
	let documentation = pattern.item(db).documentation(db).and_then(|origin| {
		let source = origin.file.contents(db);
		let start = origin.span.offset();
		let end = start + origin.span.len();
		let value = documentation_markdown(source.get(start..end)?);
		(!value.is_empty()).then_some(Documentation::MarkupContent(MarkupContent {
			kind: MarkupKind::Markdown,
			value,
		}))
	});
	SignatureInformation {
		label: function.overload.pretty_print_item(db, identifier),
		documentation,
		parameters: None,
		active_parameter: None,
	}
}

fn active_parameter(call: &str, cursor: usize) -> u32 {
	let mut depth = 0_u32;
	let mut parameter = 0_u32;
	let mut quote = None;
	let mut escaped = false;
	for ch in call[..cursor.min(call.len())].chars() {
		if let Some(q) = quote {
			if escaped {
				escaped = false;
			} else if ch == '\\' {
				escaped = true;
			} else if ch == q {
				quote = None;
			}
			continue;
		}
		match ch {
			'\'' | '"' => quote = Some(ch),
			'(' | '[' | '{' => depth += 1,
			')' | ']' | '}' => depth = depth.saturating_sub(1),
			',' if depth == 1 => parameter += 1,
			_ => {}
		}
	}
	parameter
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::{
		Position, SignatureHelpParams, TextDocumentIdentifier, TextDocumentPositionParams, Uri,
		WorkDoneProgressParams,
	};

	use super::SignatureHelpHandler;
	use crate::handlers::tests::test_handler;

	#[test]
	fn shows_all_overloads_and_documentation() {
		test_handler::<SignatureHelpHandler, _, _>(
			r#"
/** Add an integer to itself. */
function int: twice(int: value) = value + value;
/** Repeat a string. */
function string: twice(string: value) = value ++ value;
int: result = twice(1, );
			"#,
			true,
			params(5, 23),
			expect![[r#"
    {
      "Ok": {
        "signatures": [
          {
            "label": "function int: twice(int)",
            "documentation": {
              "kind": "markdown",
              "value": "Add an integer to itself."
            }
          },
          {
            "label": "function string: twice(string)",
            "documentation": {
              "kind": "markdown",
              "value": "Repeat a string."
            }
          }
        ],
        "activeParameter": 1
      }
    }"#]],
		)
	}

	#[test]
	fn ignores_commas_in_nested_expressions() {
		test_handler::<SignatureHelpHandler, _, _>(
			r#"
function int: foo(int: first, int: second) = first + second;
int: result = foo([1, 2][1], );
			"#,
			true,
			params(2, 29),
			expect![[r#"
    {
      "Ok": {
        "signatures": [
          {
            "label": "function int: foo(int, int)"
          }
        ],
        "activeParameter": 1
      }
    }"#]],
		)
	}

	fn params(line: u32, character: u32) -> SignatureHelpParams {
		SignatureHelpParams {
			context: None,
			text_document_position_params: TextDocumentPositionParams {
				text_document: TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				position: Position { line, character },
			},
			work_done_progress_params: WorkDoneProgressParams {
				work_done_token: None,
			},
		}
	}
}
