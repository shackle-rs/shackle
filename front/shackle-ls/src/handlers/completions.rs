use std::sync::Arc;

use lsp_server::{ErrorCode, ResponseError};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse};
use shackle::{
	file::InputFile,
	hir::{
		db::Hir,
		source::{find_expression, Point},
		Expression, PatternTy, TyData,
	},
};

pub fn completions(
	db: &mut dyn Hir,
	params: CompletionParams,
) -> Result<Option<CompletionResponse>, ResponseError> {
	let path = params
		.text_document_position
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
		row: params.text_document_position.position.line as usize,
		column: (params.text_document_position.position.character.max(1) - 1) as usize,
	};
	let found = find_expression(db, *model_ref, start, start);
	Ok((|| {
		let expression = found?;
		let model = expression.item().model(db);
		let types = db.lookup_item_types(expression.item());
		let data = expression.item().local_item_ref(db).data(&*model);
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
						label: i.lookup(db),
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
							label: i.lookup(db),
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
				PatternTy::Function(f) => completions.push(CompletionItem {
					label: i.lookup(db),
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
				PatternTy::EnumConstructor(cs) => {
					let c = cs.first().unwrap();
					additional_overloads += cs.len() - 1;
					completions.push(CompletionItem {
						label: i.lookup(db),
						kind: Some(CompletionItemKind::ENUM_MEMBER),
						detail: Some(if additional_overloads == 0 {
							c.overload.pretty_print_item(db, i)
						} else if additional_overloads == 1 {
							format!("{} + 1 overload", c.overload.pretty_print_item(db, i),)
						} else {
							format!(
								"{} + {} overloads",
								c.overload.pretty_print_item(db, i),
								additional_overloads,
							)
						}),
						..Default::default()
					})
				}
				_ => (),
			}
		}
		for (i, p) in scope.variables_in_scope(db, expression.expression()) {
			let types = db.lookup_item_types(p.item());
			match types[p.pattern()] {
				PatternTy::Variable(ty) => completions.push(CompletionItem {
					label: i.lookup(db),
					kind: Some(CompletionItemKind::VARIABLE),
					detail: Some(ty.pretty_print(db)),
					..Default::default()
				}),
				PatternTy::EnumAtom(ty) => completions.push(CompletionItem {
					label: i.lookup(db),
					kind: Some(CompletionItemKind::ENUM_MEMBER),
					detail: Some(ty.pretty_print(db)),
					..Default::default()
				}),
				_ => (),
			}
		}
		Some(CompletionResponse::Array(completions))
	})())
}
