#![allow(missing_docs)]

//! Salsa database for THIR operations

use std::sync::{Arc, RwLock, RwLockReadGuard};

use rustc_hash::FxHashMap;

use super::{transform::thir_transforms, Declaration, Model};
use crate::{
	db::{InternedString, Upcast},
	diagnostics::Diagnostics,
	hir::db::Hir,
	Enum, Error, Result,
};

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

	/// Get a mapping from input/output identifiers to their computed types or enumerated type declaration
	fn model_io_interface(&self) -> Arc<ModelIoInterface>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelIoInterface {
	pub input: FxHashMap<Arc<str>, crate::Type>,
	pub output: FxHashMap<Arc<str>, crate::Type>,
	pub enums: FxHashMap<Arc<str>, Arc<crate::Enum>>,
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
	let _ = db.model_io_interface();
	let model = db.model_thir();
	thir_transforms()(db, model.take()).map(Arc::new)
}

fn model_io_interface(db: &dyn Thir) -> Arc<ModelIoInterface> {
	let sh = db.model_thir();
	let val = sh.get();
	let model = val.as_ref();

	// Local interner
	let mut interner: FxHashMap<InternedString, Arc<str>> = FxHashMap::default();
	let mut resolve_name = |s| {
		interner
			.entry(s)
			.or_insert_with(|| Arc::from(s.value(db.upcast())))
			.clone()
	};
	let mut type_map = FxHashMap::default();

	// Create a map of enumerations
	let mut enums = FxHashMap::default();
	for (_, e) in model.enumerations() {
		let name = resolve_name(e.enum_type().name(db.upcast()));
		if e.definition().is_some() {
			todo!();
		} else {
			enums.insert(name.clone(), Arc::new(Enum::from_data(name)));
		}
	}

	// Find the annotation identifiers
	let reg = db.identifier_registry();
	let output_ann = reg.output;
	let no_output_ann = reg.no_output;

	// Determine input and output from declarations
	let mut input = FxHashMap::default();
	let mut output = FxHashMap::default();
	let mut insert_decl = |map: &mut FxHashMap<Arc<str>, crate::Type>, decl: &Declaration| {
		let name = resolve_name(decl.name().unwrap().0);
		let ty = crate::Type::from_compiler(
			db.upcast(),
			&mut resolve_name,
			&mut type_map,
			&enums,
			decl.domain().ty(),
		);
		map.insert(name, ty);
	};
	for (_, decl) in model.all_declarations() {
		// Determine whether declaration is part of input
		if decl.top_level()
			&& decl.domain().ty().known_par(db.upcast())
			&& decl.definition().is_none()
		{
			insert_decl(&mut input, decl)
		}

		// Determine whether declaration is part of output
		let mut should_output = None;
		if decl.annotations().has(model, output_ann) {
			should_output = Some(true)
		} else if decl.annotations().has(model, no_output_ann) {
			should_output = Some(false)
		}
		if should_output == Some(true)
			|| (should_output.is_none()
				&& decl.top_level()
				&& !decl.domain().ty().known_par(db.upcast())
				&& decl.definition().is_none())
		{
			insert_decl(&mut output, decl)
		}
	}

	Arc::new(ModelIoInterface {
		input,
		output,
		enums,
	})
}
