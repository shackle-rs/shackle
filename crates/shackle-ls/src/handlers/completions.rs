use lsp_server::ResponseError;
use lsp_types::{
	request::Completion, CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
};
use shackle::db::CompilerDatabase;
use shackle::file::ModelRef;
use shackle::hir::{
	db::Hir,
	source::{find_expression, Point},
	Expression, PatternTy,
};
use shackle::ty::TyData;

use crate::dispatch::RequestHandler;
use crate::LanguageServerDatabase;

#[derive(Debug)]
pub struct CompletionsHandler;

impl RequestHandler<Completion, (ModelRef, Point)> for CompletionsHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: CompletionParams,
	) -> Result<(ModelRef, Point), ResponseError> {
		let model =
			db.set_active_file_from_document(&params.text_document_position.text_document)?;
		Ok((
			model,
			Point {
				row: params.text_document_position.position.line as usize,
				column: (params.text_document_position.position.character) as usize,
			},
		))
	}

	fn execute(
		db: &CompilerDatabase,
		(model_ref, start): (ModelRef, Point),
	) -> Result<Option<CompletionResponse>, ResponseError> {
		let found = find_expression(db, *model_ref, start, start).or_else(|| {
			let mut prev = start;
			if prev.column > 0 {
				prev.column -= 1;
				find_expression(db, *model_ref, prev, prev)
			} else {
				None
			}
		});
		Ok((|| {
			let expression = found?;
			let model = expression.item().model(db);
			let types = db.lookup_item_types(expression.item());
			let data = expression.item().local_item_ref(db).data(&model);
			let structure = match &data[expression.expression()] {
				Expression::TupleAccess(ta) => Some(ta.tuple),
				Expression::RecordAccess(ra) => Some(ra.record),
				_ => None,
			};
			if let Some(e) = structure {
				// Give completions for tuple/record access
				let completions = match types[e].lookup(db) {
					TyData::Tuple(_, fs) => fs
						.iter()
						.enumerate()
						.map(|(i, t)| CompletionItem {
							label: format!("{}", i + 1),
							kind: Some(CompletionItemKind::FIELD),
							detail: Some(t.pretty_print(db)),
							..Default::default()
						})
						.collect(),
					TyData::Record(_, fs) => fs
						.iter()
						.map(|(i, t)| CompletionItem {
							label: i.value(db),
							kind: Some(CompletionItemKind::FIELD),
							detail: Some(t.pretty_print(db)),
							..Default::default()
						})
						.collect(),
					TyData::Array { element, .. } => match element.lookup(db) {
						TyData::Tuple(_, fs) => fs
							.iter()
							.enumerate()
							.map(|(i, t)| CompletionItem {
								label: format!("{}", i + 1),
								kind: Some(CompletionItemKind::FIELD),
								detail: Some(t.pretty_print(db)),
								..Default::default()
							})
							.collect(),
						TyData::Record(_, fs) => fs
							.iter()
							.map(|(i, t)| CompletionItem {
								label: i.value(db),
								kind: Some(CompletionItemKind::FIELD),
								detail: Some(t.pretty_print(db)),
								..Default::default()
							})
							.collect(),
						_ => vec![],
					},
					_ => vec![],
				};
				return Some(CompletionResponse::Array(completions));
			}

			// Give completions for identifiers in scope
			let scope = db.lookup_item_scope(expression.item());
			let mut completions = Vec::new();
			for (i, ps) in scope.functions_in_scope(db, expression.expression()) {
				let p = ps.first().unwrap();
				let mut additional_overloads = ps.len() - 1;
				let types = db.lookup_item_types(p.item());
				match &types[p.pattern()] {
					PatternTy::Function(f)
					| PatternTy::AnnotationConstructor(f)
					| PatternTy::AnnotationDestructure(f) => completions.push(CompletionItem {
						label: i.pretty_print(db),
						kind: Some(CompletionItemKind::FUNCTION),
						detail: Some(if additional_overloads == 0 {
							f.overload.pretty_print_item(db, i)
						} else if additional_overloads == 1 {
							format!("{} + 1 overload", f.overload.pretty_print_item(db, i),)
						} else {
							format!(
								"{} + {} overloads",
								f.overload.pretty_print_item(db, i),
								additional_overloads,
							)
						}),
						..Default::default()
					}),
					PatternTy::EnumConstructor(ec) => {
						let func = &ec[0];
						additional_overloads += ec.len() - 1;
						completions.push(CompletionItem {
							label: i.pretty_print(db),
							kind: Some(CompletionItemKind::ENUM_MEMBER),
							detail: Some(if additional_overloads == 0 {
								func.overload.pretty_print_item(db, i)
							} else if additional_overloads == 1 {
								format!("{} + 1 overload", func.overload.pretty_print_item(db, i),)
							} else {
								format!(
									"{} + {} overloads",
									func.overload.pretty_print_item(db, i),
									additional_overloads,
								)
							}),
							..Default::default()
						});
					}
					PatternTy::EnumDestructure(ec) => {
						let func = &ec[0];
						additional_overloads += ec.len() - 1;
						completions.push(CompletionItem {
							label: i.pretty_print(db),
							kind: Some(CompletionItemKind::ENUM_MEMBER),
							detail: Some(if additional_overloads == 0 {
								func.overload.pretty_print_item(db, i)
							} else if additional_overloads == 1 {
								format!("{} + 1 overload", func.overload.pretty_print_item(db, i),)
							} else {
								format!(
									"{} + {} overloads",
									func.overload.pretty_print_item(db, i),
									additional_overloads,
								)
							}),
							..Default::default()
						});
					}
					_ => (),
				}
			}
			for (i, p) in scope.variables_in_scope(db, expression.expression()) {
				let types = db.lookup_item_types(p.item());
				match types[p.pattern()] {
					PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
						completions.push(CompletionItem {
							label: i.pretty_print(db),
							kind: Some(CompletionItemKind::VARIABLE),
							detail: Some(ty.pretty_print(db)),
							..Default::default()
						})
					}
					PatternTy::Enum(ty) => completions.push(CompletionItem {
						label: i.pretty_print(db),
						kind: Some(CompletionItemKind::ENUM),
						detail: Some(ty.pretty_print(db)),
						..Default::default()
					}),
					PatternTy::EnumAtom(ty) => completions.push(CompletionItem {
						label: i.pretty_print(db),
						kind: Some(CompletionItemKind::ENUM_MEMBER),
						detail: Some(ty.pretty_print(db)),
						..Default::default()
					}),
					PatternTy::AnnotationAtom => completions.push(CompletionItem {
						label: i.pretty_print(db),
						kind: Some(CompletionItemKind::CONSTANT),
						detail: Some("ann".to_owned()),
						..Default::default()
					}),
					_ => (),
				}
			}
			Some(CompletionResponse::Array(completions))
		})())
	}
}
