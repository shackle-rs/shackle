// #![allow(missing_docs)]

//! Salsa database for THIR operations

// use std::sync::{Arc, RwLock, RwLockReadGuard};

// use shackle_diagnostics::{Diagnostics, Error, Result};

// use super::{Model, transform::thir_transforms};

// /// THIR queries
// #[salsa::query_group(ThirStorage)]
// pub trait Thir: Hir + Upcast<dyn Hir> {
// 	/// Lower a model to THIR
// 	#[salsa::invoke(super::lower::lower_model)]
// 	fn model_thir(&self) -> Arc<Intermediate<Model>>;

// 	/// Get the THIR after all THIR rewritings have been done
// 	fn final_thir(&self) -> Result<Arc<Model>>;

// 	/// Check that the pretty printed THIR is a valid model
// 	#[salsa::invoke(super::sanity_check::sanity_check_thir)]
// 	fn sanity_check_thir(&self) -> Arc<Diagnostics<Error>>;
// }

use std::sync::{RwLock, RwLockReadGuard};

use shackle_diagnostics::Result;
use shackle_hir::{Db, diagnostics::Errors};

use crate::{Model, lower::lower_model, transform::thir_transforms};

/// Represents an intermediate query result which can be taken
/// at some point, making any future reads panic.
///
/// This is used to avoid cloning the model when we know we can actually
/// discard the previous value because it won't be used anymore.
#[derive(Debug)]
pub struct Intermediate<T>(RwLock<Option<T>>);

impl<T> Intermediate<T> {
	/// Create a new intermediate value
	pub fn new(value: T) -> Self {
		Self(RwLock::new(Some(value)))
	}

	/// Take the value of this intermediate, making any future reads fail
	pub fn take(&self) -> T {
		self.0
			.write()
			.unwrap()
			.take()
			.expect("Intermediate already taken")
	}

	/// Read this intermediate value without taking it
	pub fn get(&self) -> IntermediateValue<'_, T> {
		IntermediateValue(self.0.read().unwrap())
	}
}

/// Access an intermediate value
#[derive(Debug)]
pub struct IntermediateValue<'a, T>(RwLockReadGuard<'a, Option<T>>);

impl<'a, T> AsRef<T> for IntermediateValue<'a, T> {
	fn as_ref(&self) -> &T {
		self.0.as_ref().expect("Intermediate already taken")
	}
}

impl<T: PartialEq> PartialEq for Intermediate<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.read().unwrap().as_ref() == other.0.read().unwrap().as_ref()
	}
}

impl<T: Eq> Eq for Intermediate<T> {}

/// Get the final THIR after all transforms have been applied
pub fn final_thir<'db>(db: &'db dyn Db) -> Result<&'db Model<'db>> {
	let model = run_thir_phase_internal(db);
	let error = run_thir_phase_internal::accumulated::<Errors>(db).pop();
	if let Some(e) = error {
		Err((**e).clone())
	} else {
		Ok(model)
	}
}

#[salsa::tracked]
fn run_thir_phase_internal<'db>(db: &'db dyn Db) -> Model<'db> {
	let model = lower_model(db).take();
	let result = thir_transforms()(db, model);
	result.unwrap_or_else(|error| {
		Errors::add(db, error);
		Model::default()
	})
}
