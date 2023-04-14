//! THIR model transformations.
//!
//! These transformations take a model as input and output a new transformed model.
//! The `crate::thir::Visitor` and `crate::thir::Folder` traits are useful for implementing these.
//! It is the responsibility of implementors to know what constructs are expected to be present at the stage they run.

use self::capturing_fn::decapture_model;
use self::desugar::desugar_model;
use self::type_erase::type_erase;
use self::type_specialise::type_specialise;
use super::db::Thir;
use super::Model;

pub mod capturing_fn;
pub mod desugar;
pub mod type_erase;
pub mod type_specialise;

/// Create a transformer which runs the given transforms in order on an initial model
pub fn transformer(
	transforms: Vec<fn(&dyn Thir, &Model) -> Model>,
) -> impl FnMut(&dyn Thir, &Model) -> Model {
	let mut iter = transforms.into_iter();
	move |db, model| {
		let mut m = iter
			.next()
			.map_or_else(|| model.clone(), |initial| initial(db, model));
		for transform in iter.by_ref() {
			m = transform(db, &m);
		}
		m
	}
}

/// Get the default THIR transformer
pub fn thir_transforms() -> impl FnMut(&dyn Thir, &Model) -> Model {
	transformer(vec![
		desugar_model,
		type_specialise,
		type_erase,
		decapture_model,
	])
}

#[cfg(test)]
pub mod test {
	use std::sync::Arc;

	use expect_test::Expect;

	use crate::{
		db::{CompilerDatabase, FileReader, Inputs},
		file::InputFile,
		hir::ids::NodeRef,
		thir::{db::Thir, pretty_print::PrettyPrinter, ItemId, Model},
	};

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// The expected value only includes items which are from the `source` (i.e. not from stdlib).
	pub fn check(
		transform: impl FnOnce(&dyn Thir, &Model) -> Model,
		source: &str,
		expected: Expect,
	) {
		let mut db = CompilerDatabase::default();
		db.set_input_files(Arc::new(vec![InputFile::ModelString(source.to_owned())]));
		let model_ref = db.input_models()[0];
		let model = db.model_thir();
		let result = transform(&db, &model);
		let to_print = result
			.top_level_items()
			.filter(|it| {
				let origin = match it {
					ItemId::Annotation(idx) => result[*idx].origin(),
					ItemId::Constraint(idx) => result[*idx].origin(),
					ItemId::Declaration(idx) => result[*idx].origin(),
					ItemId::Enumeration(idx) => result[*idx].origin(),
					ItemId::Function(idx) => result[*idx].origin(),
					ItemId::Output(idx) => result[*idx].origin(),
					ItemId::Solve => result.solve().unwrap().origin(),
				};
				if let NodeRef::Item(item) = *origin {
					item.model_ref(&db) == model_ref
				} else {
					false
				}
			})
			.collect::<Vec<_>>();
		let printer = PrettyPrinter::new(&db, &result);
		let mut pretty = String::new();
		for item in to_print {
			pretty.push_str(&printer.pretty_print_item(item));
			pretty.push_str(";\n");
		}
		expected.assert_eq(&pretty);
	}

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// Turns off stdlib inclusion.
	pub fn check_no_stdlib(
		transform: impl FnOnce(&dyn Thir, &Model) -> Model,
		source: &str,
		expected: Expect,
	) {
		let mut db = CompilerDatabase::default();
		db.set_ignore_stdlib(true);
		db.set_input_files(Arc::new(vec![InputFile::ModelString(source.to_owned())]));
		let model = db.model_thir();
		let result = transform(&db, &model);
		let pretty = PrettyPrinter::new(&db, &result).pretty_print();
		expected.assert_eq(&pretty);
	}
}
