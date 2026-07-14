use lsp_server::ResponseError;
use lsp_types::{
	SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensParams,
	SemanticTokensResult, request::SemanticTokensFullRequest,
};
use miette::SourceCode;
use shackle_hir::{
	PatternTy,
	db::CompilerDatabase,
	ids::{EntityId, PatternRef},
	input::ModelFile,
	source::model_leaves,
};

use crate::{db::LanguageServerContext, dispatch::RequestHandler, utils::span_contents_to_range};

#[derive(Debug)]
pub(crate) struct SemanticTokensHandler;

impl RequestHandler<SemanticTokensFullRequest, ModelFile> for SemanticTokensHandler {
	fn prepare(
		db: &mut impl LanguageServerContext,
		params: SemanticTokensParams,
	) -> Result<ModelFile, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}

	fn execute(
		db: &CompilerDatabase,
		model_ref: ModelFile,
	) -> Result<Option<SemanticTokensResult>, ResponseError> {
		let mut tokens = Vec::new();
		let mut prev_line = 0;
		let mut prev_char = 0;
		for entity in model_leaves(db, model_ref).iter().copied() {
			let item = entity.item(db);
			let types = item.types(db);
			let mut token_type = TokenType::Variable;
			let mut is_par = false;
			let pattern = match entity.entity(db) {
				EntityId::Expression(e) => {
					is_par = is_par
						|| types
							.get_expression(e)
							.map(|ty| ty.known_par(db))
							.unwrap_or_default();
					types.name_resolution(e)
				}
				EntityId::Pattern(p) => Some(PatternRef::new(db, item, p)),
				EntityId::Type(_) => {
					continue;
				}
			};

			if let Some(p) = pattern {
				let item = p.item(db);
				let types = item.types(db);
				match types.get_pattern(p.pattern(db)) {
					Some(
						PatternTy::AnnotationAtom
						| PatternTy::AnnotationConstructor(_)
						| PatternTy::AnnotationDestructure(_)
						| PatternTy::AnonymousEnumConstructor(_)
						| PatternTy::EnumAtom(_)
						| PatternTy::EnumConstructor(_)
						| PatternTy::EnumDestructure(_),
					) => {
						token_type = TokenType::EnumMember;
					}
					Some(PatternTy::Function(_) | PatternTy::DestructuringFn { .. }) => {
						token_type = TokenType::Function
					}
					Some(PatternTy::TyVar(_)) => token_type = TokenType::TypeParameter,
					Some(PatternTy::TypeAlias { .. }) => token_type = TokenType::Type,
					Some(PatternTy::Variable(ty)) => {
						is_par = is_par || ty.known_par(db);
						if ty.is_function(db) {
							token_type = TokenType::Function;
						}
					}
					Some(PatternTy::Argument(ty)) => {
						is_par = is_par || ty.known_par(db);
						token_type = TokenType::Parameter
					}
					Some(PatternTy::Enum(_)) => token_type = TokenType::Enum,
					Some(PatternTy::TupleField(ty)) => {
						is_par = is_par || ty.known_par(db);
						token_type = TokenType::Field
					}
					Some(PatternTy::RecordField(ty)) => {
						is_par = is_par || ty.known_par(db);
						token_type = TokenType::Field
					}
					_ => (),
				}
			}

			let (src, span) = entity.source_span(db);
			let span_contents = src.read_span(&span, 0, 0).unwrap();
			let range = span_contents_to_range(&*span_contents);
			if range.start.line != range.end.line {
				continue;
			}
			tokens.push(SemanticToken {
				delta_line: range.start.line - prev_line,
				delta_start: if range.start.line == prev_line {
					range.start.character - prev_char
				} else {
					range.start.character
				},
				length: range.end.character - range.start.character,
				token_type: token_type as u32,
				token_modifiers_bitset: (is_par as u32) << (TokenModifier::ReadOnly as u32),
			});
			prev_line = range.start.line;
			prev_char = range.start.character;
		}

		Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
			data: tokens,
			..Default::default()
		})))
	}
}

macro_rules! legend {
    ($name:ident<$type:ty> {$($tn:ident: $te:expr),* $(,)?}) => {
        pub(crate) enum $name {
            $($tn),*
        }

        impl $name {
            pub(crate) fn legend() -> Vec<$type> {
                vec![
                    $($te),*
                ]
            }
        }
    };
}

legend!(
	TokenType<SemanticTokenType> {
		Type: SemanticTokenType::TYPE,
		Enum: SemanticTokenType::ENUM,
		TypeParameter: SemanticTokenType::TYPE_PARAMETER,
		Parameter: SemanticTokenType::PARAMETER,
		EnumMember: SemanticTokenType::ENUM_MEMBER,
		Function: SemanticTokenType::FUNCTION,
		Variable: SemanticTokenType::VARIABLE,
		Field: SemanticTokenType::PROPERTY,
	}
);

legend!(
	TokenModifier<SemanticTokenModifier> {
		ReadOnly: SemanticTokenModifier::READONLY
	}
);

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use expect_test::expect;
	use lsp_types::Uri;

	use super::SemanticTokensHandler;
	use crate::handlers::tests::test_handler;

	#[test]
	fn test_semantic_tokens() {
		test_handler::<SemanticTokensHandler, _, _>(
			r#"
enum Foo = {A, B, C};
int: x;
var 1..3: y;
function var int: bar(int: a, var int: b);
any: z = bar(x, y);
any: w = bar((hello: 1).hello, (3, 4).2);
			"#,
			false,
			lsp_types::SemanticTokensParams {
				text_document: lsp_types::TextDocumentIdentifier {
					uri: Uri::from_str("file:///test.mzn").unwrap(),
				},
				partial_result_params: lsp_types::PartialResultParams {
					partial_result_token: None,
				},
				work_done_progress_params: lsp_types::WorkDoneProgressParams {
					work_done_token: None,
				},
			},
			expect!([r#"
    {
      "Ok": {
        "data": [
          1,
          5,
          3,
          1,
          0,
          0,
          7,
          1,
          4,
          0,
          0,
          3,
          1,
          4,
          0,
          0,
          3,
          1,
          4,
          0,
          1,
          5,
          1,
          6,
          1,
          1,
          4,
          1,
          6,
          1,
          0,
          1,
          2,
          5,
          1,
          0,
          2,
          1,
          6,
          1,
          0,
          3,
          1,
          6,
          0,
          1,
          18,
          3,
          5,
          0,
          0,
          9,
          1,
          3,
          1,
          0,
          12,
          1,
          3,
          0,
          1,
          5,
          1,
          6,
          0,
          0,
          4,
          3,
          5,
          1,
          0,
          4,
          1,
          6,
          1,
          0,
          3,
          1,
          6,
          0,
          1,
          5,
          1,
          6,
          0,
          0,
          4,
          3,
          5,
          1,
          0,
          5,
          5,
          7,
          1,
          0,
          7,
          1,
          6,
          1,
          0,
          3,
          5,
          7,
          1,
          0,
          8,
          1,
          6,
          1,
          0,
          3,
          1,
          6,
          1,
          0,
          3,
          1,
          7,
          1
        ]
      }
    }"#]),
		)
	}
}
