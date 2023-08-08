#![allow(missing_docs)]

//! Salsa database for THIR operations

use std::sync::{Arc, RwLock, RwLockReadGuard};

use rustc_hash::FxHashMap;

use super::{transform::thir_transforms, Identifier, Model};
use crate::{db::Upcast, diagnostics::Diagnostics, hir::db::Hir, ty, Error, Result};

/// THIR queries
#[salsa::query_group(ThirStorage)]
pub trait Thir: Hir + Upcast<dyn Hir> {
	/// Lower a model to THIR
	#[salsa::invoke(super::lower::lower_model)]
	fn model_thir(&self) -> Arc<Intermediate<Model>>;

	/// Get the THIR after all THIR rewritings have been done
	fn final_thir(&self) -> Result<Arc<Model>>;

	/// Check that the pretty printed THIR is a valid model
	#[salsa::invoke(super::sanity_check::sanity_check_thir)]
	fn sanity_check_thir(&self) -> Arc<Diagnostics<Error>>;

	/// Get a mapping from variable identifiers to their computed types
	fn input_type_map(&self) -> Arc<FxHashMap<Identifier, ty::Ty>>;

	/// Get a mapping from variable identifiers to their computed types
	fn output_type_map(&self) -> Arc<FxHashMap<Identifier, ty::Ty>>;
}

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
	pub fn get(&self) -> IntermediateValue<T> {
		IntermediateValue(self.0.read().unwrap())
	}
}

/// Access an intermediate value
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

fn final_thir(db: &dyn Thir) -> Result<Arc<Model>> {
	let model = db.model_thir();
	thir_transforms()(db, model.take()).map(Arc::new)
}

fn input_type_map(db: &dyn Thir) -> Arc<FxHashMap<Identifier, ty::Ty>> {
	let model = db.final_thir().unwrap();

	let mut result = FxHashMap::default();
	for (_, decl) in model.all_declarations() {
		if decl.top_level()
			&& decl.domain().ty().known_par(db.upcast())
			&& decl.definition().is_none()
		{
			result.insert(decl.name().unwrap(), decl.domain().ty());
		}
	}
	Arc::new(result)
}

fn output_type_map(db: &dyn Thir) -> Arc<FxHashMap<Identifier, ty::Ty>> {
	let model = db.final_thir().unwrap();

	// Find the annotation identifiers
	let reg = db.identifier_registry();
	let output = reg.output;
	let no_output = reg.no_output;

	let mut result = FxHashMap::default();
	for (_, decl) in model.all_declarations() {
		let mut should_output = None;
		if decl.annotations().has(&model, output) {
			should_output = Some(true)
		} else if decl.annotations().has(&model, no_output) {
			should_output = Some(false)
		}
		if should_output == Some(true)
			|| (should_output.is_none()
				&& decl.top_level()
				&& !decl.domain().ty().known_par(db.upcast())
				&& decl.definition().is_none())
		{
			result.insert(decl.name().unwrap(), decl.domain().ty());
		}
	}
	Arc::new(result)
}
