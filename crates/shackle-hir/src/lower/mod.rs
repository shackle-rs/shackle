//! Functionality for converting AST nodes to HIR nodes
//! for the respective modelling languages.

pub mod eprime;
pub mod minizinc;
#[cfg(test)]
mod tests;

use eprime::ItemCollector as EPrimeItemCollector;
use minizinc::ItemCollector;
use shackle_syntax::ast::ConstraintModel;

use crate::{
	Db, Model,
	diagnostics::Diagnostics,
	input::{ModelFile, resolve_includes},
};

#[salsa::tracked]
fn lower_model_with_diagnostics<'db>(
	db: &'db dyn Db,
	model_file: ModelFile,
) -> (Model<'db>, Diagnostics) {
	log::info!("Lowering model to HIR: {}", model_file);
	let model_ast = model_file.ast(db);
	match model_ast.ast(db) {
		ConstraintModel::MznModel(model) => {
			let mut ctx = ItemCollector::new(db, model_file);
			for item in model.items() {
				ctx.collect_item(&item);
			}
			ctx.finish()
		}
		ConstraintModel::EPrimeModel(model) => {
			let mut ctx = EPrimeItemCollector::new(db, model_file);
			ctx.preprocess(model.items());
			for item in model.items() {
				ctx.collect_item(&item);
			}
			ctx.add_solve();
			ctx.finish()
		}
	}
}

/// Accumulate lowering diagnostics for all resolved models.
#[salsa::tracked(returns(copy))]
pub fn accumulate_lower_errors(db: &dyn Db) {
	for model in resolve_includes(db) {
		let (_, diagnostics) = lower_model_with_diagnostics(db, *model);
		diagnostics.accumulate(db);
	}
}

impl ModelFile {
	/// Lower this model to HIR
	pub fn hir<'db>(&self, db: &'db dyn Db) -> Model<'db> {
		lower_model_with_diagnostics(db, *self).0
	}
}

/// Lower all models to HIR
#[salsa::tracked]
pub fn lower_models<'db>(db: &'db dyn Db) -> Vec<Model<'db>> {
	let models = resolve_includes(db);
	models.iter().map(|m| m.hir(db)).collect()
}
