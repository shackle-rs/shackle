//! Functionality for converting AST nodes to HIR nodes
//! for the respective modelling languages.

pub mod eprime;
pub mod minizinc;
#[cfg(test)]
pub mod test;

use std::sync::Arc;

use self::{eprime::ItemCollector as EPrimeItemCollector, minizinc::ItemCollector};
use crate::{
	constants::IdentifierRegistry,
	file::ModelRef,
	hir::{db::Hir, source::SourceMap, *},
	syntax::ast::ConstraintModel,
	Error,
};

/// Lower a model to HIR
pub fn lower_items(db: &dyn Hir, model: ModelRef) -> (Arc<Model>, Arc<SourceMap>, Arc<Vec<Error>>) {
	let ast = match db.ast(*model) {
		Ok(m) => m,
		Err(e) => return (Default::default(), Default::default(), Arc::new(vec![e])),
	};
	let identifiers = IdentifierRegistry::new(db);
	match ast {
		ConstraintModel::MznModel(ast) => {
			let mut ctx = ItemCollector::new(db, &identifiers, model);
			for item in ast.items() {
				ctx.collect_item(item);
			}
			let (m, sm, e) = ctx.finish();
			(Arc::new(m), Arc::new(sm), Arc::new(e))
		}
		ConstraintModel::EPrimeModel(ast) => {
			let mut ctx = EPrimeItemCollector::new(db, &identifiers, model);
			for item in ast.items() {
				ctx.collect_item(item);
			}
			let (m, sm, e) = ctx.finish();
			(Arc::new(m), Arc::new(sm), Arc::new(e))
		}
	}
}
