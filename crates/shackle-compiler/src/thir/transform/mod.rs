//! THIR model transformations.
//!
//! These transformations take a model as input and output a new transformed model.
//! The `crate::thir::Visitor` and `crate::thir::Folder` traits are useful for implementing these.
//! It is the responsibility of implementors to know what constructs are expected to be present at the stage they run.

use self::{
	capturing_fn::decapture_model, comprehension::desugar_comprehension,
	domain_constraint::rewrite_domains, erase_enum::erase_enum, erase_opt::erase_opt,
	erase_record::erase_record, function_dispatch::function_dispatch, inlining::inline_functions,
	name_mangle::mangle_names, output::generate_output, top_down_type::top_down_type,
	type_specialise::type_specialise,
};
use super::{db::Thir, Model};
use crate::Result;

pub mod capturing_fn;
pub mod comprehension;
pub mod domain_constraint;
pub mod erase_enum;
pub mod erase_opt;
pub mod erase_record;
pub mod function_dispatch;
pub mod inlining;
pub mod name_mangle;
pub mod output;
pub mod top_down_type;
pub mod type_specialise;

/// A THIR transform function
pub type TransformFn = fn(&dyn Thir, Model) -> Result<Model>;

/// Create a transformer which runs the given transforms in order on an initial model
pub fn transformer(transforms: Vec<TransformFn>) -> impl FnMut(&dyn Thir, Model) -> Result<Model> {
	let mut iter = transforms.into_iter();
	move |db, model| {
		iter.by_ref()
			.try_fold(model, |m, transform| transform(db, m))
	}
}

/// Get the default THIR transformer
pub fn thir_transforms() -> impl FnMut(&dyn Thir, Model) -> Result<Model> {
	transformer(vec![
		generate_output,
		rewrite_domains,
		top_down_type,
		type_specialise,
		function_dispatch,
		mangle_names,
		erase_record,
		erase_enum,
		desugar_comprehension,
		erase_opt,
		inline_functions,
		decapture_model,
	])
}

#[cfg(test)]
pub mod test {
	use std::sync::Arc;

	use expect_test::Expect;
	use rustc_hash::FxHashMap;

	use crate::{
		db::{CompilerDatabase, FileReader, Inputs},
		file::{InputFile, ModelRef},
		hir::{ids::NodeRef, Identifier},
		thir::{
			db::Thir,
			pretty_print::PrettyPrinter,
			traverse::{visit_annotation, visit_declaration, Visitor},
			AnnotationId, DeclarationId, ItemId, Model, ResolvedIdentifier,
		},
		Result,
	};

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// The expected value only includes items which are from the `source` (i.e. not from stdlib).
	pub fn check(
		transform: impl FnOnce(&dyn Thir, Model) -> Result<Model>,
		source: &str,
		expected: Expect,
	) {
		let mut db = CompilerDatabase::default();
		db.set_input_files(Arc::new(vec![InputFile::ModelString(source.to_owned())]));
		let model_ref = db.input_models()[0];
		let model = db.model_thir();
		let pretty = match transform(&db, model.take()) {
			Ok(mut result) => {
				let to_print = NameMapper::default().run(&db, model_ref, &mut result);
				let printer = PrettyPrinter::new(&db, &result);
				let mut pretty = String::new();
				for item in to_print {
					pretty.push_str(&printer.pretty_print_item(item));
					pretty.push_str(";\n");
				}
				pretty
			}
			Err(e) => e.to_string(),
		};
		expected.assert_eq(&pretty);
	}

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// Turns off stdlib inclusion.
	pub fn check_no_stdlib(
		transform: impl FnOnce(&dyn Thir, Model) -> Result<Model>,
		source: &str,
		expected: Expect,
	) {
		let mut db = CompilerDatabase::default();
		db.set_ignore_stdlib(true);
		db.set_input_files(Arc::new(vec![InputFile::ModelString(source.to_owned())]));
		let model = db.model_thir();
		let pretty = match transform(&db, model.take()) {
			Ok(result) => PrettyPrinter::new(&db, &result).pretty_print(),
			Err(e) => e.to_string(),
		};
		expected.assert_eq(&pretty);
	}

	#[derive(Default)]
	struct NameMapper {
		annotation: FxHashMap<AnnotationId, usize>,
		declaration: FxHashMap<DeclarationId, usize>,
	}

	impl Visitor<'_> for NameMapper {
		fn visit_annotation(&mut self, model: &Model, annotation: AnnotationId) {
			if model[annotation].name.is_none() {
				let count = self.annotation.len();
				self.annotation.entry(annotation).or_insert(count);
			}
			visit_annotation(self, model, annotation)
		}
		fn visit_declaration(&mut self, model: &Model, declaration: DeclarationId) {
			if model[declaration].name().is_none() {
				let count = self.declaration.len();
				self.declaration.entry(declaration).or_insert(count);
			}
			visit_declaration(self, model, declaration);
		}
		fn visit_identifier(&mut self, model: &Model, identifier: &ResolvedIdentifier) {
			match identifier {
				ResolvedIdentifier::Annotation(ann) => self.visit_annotation(model, *ann),
				ResolvedIdentifier::Declaration(decl) => self.visit_declaration(model, *decl),
				_ => (),
			}
		}
	}

	impl NameMapper {
		fn run(&mut self, db: &dyn Thir, model_ref: ModelRef, model: &mut Model) -> Vec<ItemId> {
			let to_print: Vec<ItemId> = model
				.top_level_items()
				.filter(|it| {
					let origin = match it {
						ItemId::Annotation(idx) => model[*idx].origin(),
						ItemId::Constraint(idx) => model[*idx].origin(),
						ItemId::Declaration(idx) => model[*idx].origin(),
						ItemId::Enumeration(idx) => model[*idx].origin(),
						ItemId::Function(idx) => model[*idx].origin(),
						ItemId::Output(idx) => model[*idx].origin(),
						ItemId::Solve => model.solve().unwrap().origin(),
					};
					match origin.node() {
						Some(NodeRef::Item(item)) => item.model_ref(db.upcast()) == model_ref,
						Some(NodeRef::Entity(entity)) => {
							entity.item(db.upcast()).model_ref(db.upcast()) == model_ref
						}
						Some(NodeRef::Model(m)) => m == model_ref,
						None => true,
					}
				})
				.collect::<Vec<_>>();
			for item in to_print.iter() {
				self.visit_item(model, *item);
			}
			for (ann, n) in self.annotation.iter() {
				model[*ann].name = Some(Identifier::new(format!("_ANN_{}", *n + 1), db.upcast()));
			}
			for (decl, n) in self.declaration.iter() {
				model[*decl].set_name(Identifier::new(format!("_DECL_{}", *n + 1), db.upcast()));
			}
			to_print
		}
	}
}
