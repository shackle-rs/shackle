//! Accumulators for diagnostics

use derive_more::{Deref, DerefMut, From, Into};
use salsa::Accumulator;
use shackle_diagnostics::{Error, Warning};

use crate::Db;

/// Accumulator for errors
#[derive(Debug, Clone, PartialEq, Eq, From, Into, Deref, DerefMut)]
#[salsa::accumulator]
pub struct Errors(Error);

impl Errors {
	/// Create and accumulate an error
	pub fn add(db: &dyn Db, error: impl Into<Error>) {
		let e = error.into();
		log::error!("{:#?}", e);
		Errors(e).accumulate(db);
	}

	/// Add multiple errors at once
	pub fn extend(db: &dyn Db, errors: impl IntoIterator<Item = impl Into<Error>>) {
		for error in errors {
			Self::add(db, error);
		}
	}
}

/// Accumulator for warnings
#[derive(Debug, Clone, PartialEq, Eq, From, Into, Deref, DerefMut)]
#[salsa::accumulator]
pub struct Warnings(Warning);

impl Warnings {
	/// Create and accumulate a warning
	pub fn add(db: &dyn Db, warning: impl Into<Warning>) {
		let w = warning.into();
		log::warn!("{}", w);
		Warnings(w).accumulate(db);
	}

	/// Add multiple warnings at once
	pub fn extend(db: &dyn Db, warnings: impl IntoIterator<Item = impl Into<Warning>>) {
		for warning in warnings {
			Self::add(db, warning);
		}
	}
}

/// Helper for emitting diagnostics while avoiding accumulating values
///
/// This is needed for cylclic queries (and their dependencies)
#[derive(Debug, Default, Clone, PartialEq, Eq, salsa::Update)]
pub struct Diagnostics {
	errors: Vec<Error>,
	warnings: Vec<Warning>,
}

impl Diagnostics {
	/// Whether there are no errors
	pub fn is_ok(&self) -> bool {
		self.errors.is_empty()
	}

	/// Add an error
	pub fn add_error(&mut self, error: impl Into<Error>) {
		let e = error.into();
		log::error!("{:#?}", e);
		self.errors.push(e);
	}

	/// Add multiple errors at once
	pub fn extend_errors(&mut self, errors: impl IntoIterator<Item = impl Into<Error>>) {
		for error in errors {
			self.add_error(error);
		}
	}

	/// Add a warning
	pub fn add_warning(&mut self, warning: impl Into<Warning>) {
		let w = warning.into();
		log::warn!("{}", w);
		self.warnings.push(w);
	}

	/// Add multiple warnings at once
	pub fn extend_warnings(&mut self, warnings: impl IntoIterator<Item = impl Into<Warning>>) {
		for warning in warnings {
			self.add_warning(warning);
		}
	}

	/// Accumulate the diagnostics into the database
	pub fn accumulate(&self, db: &dyn Db) {
		Errors::extend(db, self.errors.iter().cloned());
		Warnings::extend(db, self.warnings.iter().cloned());
	}
}
