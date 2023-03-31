#![allow(missing_docs)]

//! Salsa database for THIR operations

use std::sync::Arc;

use crate::{db::Upcast, diagnostics::Diagnostics, hir::db::Hir, Error};

use super::{transform::thir_transforms, Model};

/// THIR queries
#[salsa::query_group(ThirStorage)]
pub trait Thir: Hir + Upcast<dyn Hir> {
	/// Lower a model to THIR
	#[salsa::invoke(super::lower::lower_model)]
	fn model_thir(&self) -> Arc<Model>;

	/// Get the THIR after all THIR rewritings have been done
	fn final_thir(&self) -> Arc<Model>;

	/// Check that the pretty printed THIR is a valid model
	#[salsa::invoke(super::sanity_check::sanity_check_thir)]
	fn sanity_check_thir(&self) -> Arc<Diagnostics<Error>>;
}

fn final_thir(db: &dyn Thir) -> Arc<Model> {
	let model = db.model_thir();
	Arc::new(thir_transforms()(db, &model))
}
