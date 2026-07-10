use lsp_server::ResponseError;
use lsp_types::{
	InlayHint, InlayHintKind, InlayHintLabel, InlayHintParams, Position, Range,
	request::InlayHintRequest,
};
use miette::SourceCode;
use shackle_hir::{
	CallKind, Constructor, EnumConstructor, Expression, Item, PatternId,
	db::CompilerDatabase,
	ids::{ExpressionRef, PatternRef},
	input::ModelFile,
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::span_contents_to_range};

#[derive(Debug)]
pub(crate) struct InlayHintHandler;

impl RequestHandler<InlayHintRequest, (ModelFile, Range)> for InlayHintHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: InlayHintParams,
	) -> Result<(ModelFile, Range), ResponseError> {
		let model = db.set_active_file_from_document(&params.text_document)?;
		Ok((model, params.range))
	}

	fn execute(
		db: &CompilerDatabase,
		(model, requested_range): (ModelFile, Range),
	) -> Result<Option<Vec<InlayHint>>, ResponseError> {
		let mut hints = Vec::new();

		for item in model.hir(db).items(db).iter().copied() {
			let data = item.data(db);
			let types = item.types(db);
			for (_, value) in data.expressions.iter() {
				let Expression::Call(call) = value else {
					continue;
				};
				if call.kind != CallKind::SourceCall {
					continue;
				}

				let parameter_names = match &data[call.function] {
					Expression::Lambda(lambda) => parameter_names(
						db,
						item,
						lambda.parameters.iter().map(|parameter| parameter.pattern),
					),
					_ => {
						let Some(target) = types.name_resolution(call.function) else {
							continue;
						};
						let Some(names) = parameter_names_for_target(db, target) else {
							continue;
						};
						names
					}
				};

				for (argument, name) in call.arguments.iter().zip(parameter_names) {
					let Some(name) = name else {
						continue;
					};
					if let Expression::Identifier(identifier) = &data[*argument]
						&& identifier.lookup(db) == name
					{
						continue;
					}
					let Some(position) = expression_start(db, item, *argument) else {
						continue;
					};
					if !range_contains_position(requested_range, position) {
						continue;
					}
					hints.push(InlayHint {
						position,
						label: InlayHintLabel::String(format!("{}:", name)),
						kind: Some(InlayHintKind::PARAMETER),
						text_edits: None,
						tooltip: None,
						padding_left: None,
						padding_right: Some(true),
						data: None,
					});
				}
			}
		}

		hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
		Ok(Some(hints))
	}
}

fn parameter_names_for_target<'db>(
	db: &'db CompilerDatabase,
	target: PatternRef<'db>,
) -> Option<Vec<Option<String>>> {
	let item = target.item(db);
	let target_pattern = target.pattern(db);
	match item {
		Item::Function(function) => {
			let function = function.function(db);
			(function.pattern == target_pattern).then(|| {
				parameter_names(
					db,
					item,
					function
						.parameters
						.iter()
						.map(|parameter| parameter.pattern),
				)
			})
		}
		Item::Declaration(declaration) => {
			let declaration = declaration.declaration(db);
			if declaration.pattern != target_pattern {
				return None;
			}
			let Expression::Lambda(lambda) = &declaration[declaration.definition?] else {
				return None;
			};
			Some(parameter_names(
				db,
				item,
				lambda.parameters.iter().map(|parameter| parameter.pattern),
			))
		}
		Item::Annotation(annotation) => {
			let annotation = annotation.annotation(db);
			let Constructor::Function {
				constructor,
				parameters,
				..
			} = &annotation.constructor
			else {
				return None;
			};
			(*constructor == target_pattern).then(|| {
				parameter_names(
					db,
					item,
					parameters.iter().map(|parameter| parameter.pattern),
				)
			})
		}
		Item::Enumeration(enumeration) => {
			let enumeration = enumeration.enumeration(db);
			for constructor in enumeration.definition.iter().flatten() {
				if let EnumConstructor::Named(Constructor::Function {
					constructor,
					parameters,
					..
				}) = constructor && *constructor == target_pattern
				{
					return Some(parameter_names(
						db,
						item,
						parameters.iter().map(|parameter| parameter.pattern),
					));
				}
			}
			None
		}
		Item::EnumAssignment(assignment) => {
			let assignment = assignment.enum_assignment(db);
			for constructor in assignment.definition.iter() {
				if let EnumConstructor::Named(Constructor::Function {
					constructor,
					parameters,
					..
				}) = constructor && *constructor == target_pattern
				{
					return Some(parameter_names(
						db,
						item,
						parameters.iter().map(|parameter| parameter.pattern),
					));
				}
			}
			None
		}
		_ => None,
	}
}

fn parameter_names<'db>(
	db: &'db CompilerDatabase,
	item: Item<'db>,
	patterns: impl IntoIterator<Item = Option<PatternId<'db>>>,
) -> Vec<Option<String>> {
	patterns
		.into_iter()
		.map(|pattern| {
			item.data(db)[pattern?]
				.identifier()
				.map(|identifier| identifier.lookup(db).to_owned())
		})
		.collect()
}

fn expression_start<'db>(
	db: &'db CompilerDatabase,
	item: Item<'db>,
	expression: shackle_hir::ExpressionId<'db>,
) -> Option<Position> {
	let (source, span) = ExpressionRef::new(db, item, expression).source_span(db);
	let contents = source.read_span(&span, 0, 0).ok()?;
	Some(span_contents_to_range(&*contents).start)
}

fn range_contains_position(range: Range, position: Position) -> bool {
	(range.start.line, range.start.character) <= (position.line, position.character)
		&& (position.line, position.character) <= (range.end.line, range.end.character)
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::{InlayHintParams, Position, Range, TextDocumentIdentifier, Uri};

	use super::InlayHintHandler;
	use crate::handlers::tests::test_handler;

	#[test]
	fn test_parameter_name_hints() {
		test_handler::<InlayHintHandler, _, _>(
			r#"function int: foo(int: param1, int: param2) = param1 + param2;
int: result = foo(1, foo(2, 3));"#,
			false,
			InlayHintParams {
				work_done_progress_params: Default::default(),
				text_document: TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				range: Range {
					start: Position::new(0, 0),
					end: Position::new(1, 36),
				},
			},
			expect![[r#"
    {
      "Ok": [
        {
          "position": {
            "line": 1,
            "character": 18
          },
          "label": "param1:",
          "kind": 2,
          "paddingRight": true
        },
        {
          "position": {
            "line": 1,
            "character": 21
          },
          "label": "param2:",
          "kind": 2,
          "paddingRight": true
        },
        {
          "position": {
            "line": 1,
            "character": 25
          },
          "label": "param1:",
          "kind": 2,
          "paddingRight": true
        },
        {
          "position": {
            "line": 1,
            "character": 28
          },
          "label": "param2:",
          "kind": 2,
          "paddingRight": true
        }
      ]
    }"#]],
		)
	}

	#[test]
	fn test_hints_use_resolved_overload_and_requested_range() {
		test_handler::<InlayHintHandler, _, _>(
			r#"function int: foo(int: integer_value) = integer_value;
function float: foo(float: float_value) = float_value;
float: result = foo(1.0);"#,
			false,
			InlayHintParams {
				work_done_progress_params: Default::default(),
				text_document: TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				range: Range {
					start: Position::new(2, 18),
					end: Position::new(2, 21),
				},
			},
			expect![[r#"
    {
      "Ok": [
        {
          "position": {
            "line": 2,
            "character": 20
          },
          "label": "float_value:",
          "kind": 2,
          "paddingRight": true
        }
      ]
    }"#]],
		)
	}

	#[test]
	fn test_matching_identifier_arguments_do_not_get_hints() {
		test_handler::<InlayHintHandler, _, _>(
			r#"function int: foo(int: width, int: height) = width + height;
int: width = 1;
int: other = 2;
int: result = foo(width, other);"#,
			false,
			InlayHintParams {
				work_done_progress_params: Default::default(),
				text_document: TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				range: Range {
					start: Position::new(0, 0),
					end: Position::new(3, 32),
				},
			},
			expect![[r#"
    {
      "Ok": [
        {
          "position": {
            "line": 3,
            "character": 25
          },
          "label": "height:",
          "kind": 2,
          "paddingRight": true
        }
      ]
    }"#]],
		)
	}

	#[test]
	fn test_operators_do_not_get_hints() {
		test_handler::<InlayHintHandler, _, _>(
			r#"predicate foo(bool: value) = not (value);
constraint foo(true);"#,
			false,
			InlayHintParams {
				work_done_progress_params: Default::default(),
				text_document: TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				range: Range {
					start: Position::new(0, 0),
					end: Position::new(1, 21),
				},
			},
			expect![[r#"
    {
      "Ok": [
        {
          "position": {
            "line": 1,
            "character": 15
          },
          "label": "value:",
          "kind": 2,
          "paddingRight": true
        }
      ]
    }"#]],
		)
	}
}
