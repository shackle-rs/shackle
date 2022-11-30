#![allow(missing_docs)]

//! Salsa database for THIR operations

use std::sync::Arc;

use crate::{db::Upcast, hir::db::Hir, Error};

use super::Model;

/// THIR queries
#[salsa::query_group(ThirStorage)]
pub trait Thir: Hir + Upcast<dyn Hir> {
	/// Lower a model to THIR
	#[salsa::invoke(super::lower::lower_model)]
	fn model_thir(&self) -> Arc<Model>;

	/// Check that the pretty printed THIR is a valid model
	#[salsa::invoke(super::sanity_check::sanity_check_thir)]
	fn sanity_check_thir(&self) -> Arc<Vec<Error>>;
}
