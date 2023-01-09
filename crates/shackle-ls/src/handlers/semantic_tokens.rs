use lsp_server::ResponseError;
use lsp_types::{
	request::SemanticTokensFullRequest, SemanticToken, SemanticTokenModifier, SemanticTokenType,
	SemanticTokens, SemanticTokensParams, SemanticTokensResult,
};
use miette::SourceCode;
use shackle::{
	db::CompilerDatabase,
	file::ModelRef,
	hir::{db::Hir, ids::NodeRef},
	hir::{
		ids::{LocalEntityRef, PatternRef},
		PatternTy,
	},
	syntax::db::SourceParser,
};

use crate::{dispatch::RequestHandler, utils::span_contents_to_range, LanguageServerDatabase};

#[derive(Debug)]
pub struct SemanticTokensHandler;

impl RequestHandler<SemanticTokensFullRequest, ModelRef> for SemanticTokensHandler {
	fn prepare(
		db: &mut LanguageServerDatabase,
		params: SemanticTokensParams,
	) -> Result<ModelRef, ResponseError> {
		db.set_active_file_from_document(&params.text_document)
	}
	fn execute(
		db: &CompilerDatabase,
		model_ref: ModelRef,
	) -> Result<Option<SemanticTokensResult>, ResponseError> {
		if let Ok(cst) = db.cst(*model_ref) {
			let query = tree_sitter::Query::new(
				tree_sitter_minizinc::language(),
				tree_sitter_minizinc::IDENTIFIERS_QUERY,
			)
			.expect("Failed to create query");
			let mut cursor = tree_sitter::QueryCursor::new();
			let captures = cursor.captures(&query, cst.root_node(), cst.text().as_bytes());
			let nodes = captures.map(|(c, _)| c.captures[0].node);
			let source_map = db.lookup_source_map(model_ref);
			let mut tokens = Vec::new();
			let mut prev_line = 0;
			let mut prev_char = 0;
			for node in nodes {
				if let Some(node_ref @ NodeRef::Entity(entity)) = source_map.find_node(node) {
					let item = entity.item(db);
					let types = db.lookup_item_types(item);
					let mut token_type = TokenType::Variable;
					let mut is_par = false;
					let pattern = match entity.entity(db) {
						LocalEntityRef::Expression(e) => {
							is_par = is_par
								|| types
									.get_expression(e)
									.map(|ty| ty.known_par(db))
									.unwrap_or_default();
							types.name_resolution(e)
						}
						LocalEntityRef::Pattern(p) => Some(PatternRef::new(item, p)),
						_ => None,
					};

					if let Some(p) = pattern {
						let item = p.item();
						let types = db.lookup_item_types(item);
						match types.get_pattern(p.pattern()) {
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
							Some(PatternTy::TypeAlias(_)) => token_type = TokenType::Type,
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
							_ => (),
						}
					}

					let (src, span) = node_ref.source_span(db);
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
						token_modifiers_bitset: u32::from(is_par),
					});
					prev_line = range.start.line;
					prev_char = range.start.character;
				}
			}

			return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
				data: tokens,
				..Default::default()
			})));
		}
		Ok(None)
	}
}

macro_rules! legend {
    ($name:ident<$type:ty> {$($tn:ident: $te:expr),* $(,)?}) => {
        pub enum $name {
			#[allow(dead_code)]
            $($tn),*
        }

        impl $name {
            pub fn legend() -> Vec<$type> {
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
	}
);

legend!(
	TokenModifier<SemanticTokenModifier> {
		ReadOnly: SemanticTokenModifier::READONLY
	}
);
