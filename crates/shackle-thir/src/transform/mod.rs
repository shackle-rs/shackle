//! THIR model transformations.
//!
//! These transformations take a model as input and output a new transformed model.
//! The `crate::Visitor` and `crate::Folder` traits are useful for implementing these.
//! It is the responsibility of implementors to know what constructs are expected to be present at the stage they run.

use shackle_diagnostics::Result;
use totalise::totalise;

use self::{
	compat::old_compat,
	// capturing_fn::decapture_model,
	comprehension::desugar_comprehension,
	dead_code::{eliminate_dead_code, eliminate_dead_code_conservative},
	domain_constraint::rewrite_domains,
	erase_enum::erase_enum,
	erase_opt::erase_opt,
	erase_record::erase_record,
	function_dispatch::function_dispatch,
	inlining::inline_functions,
	name_mangle::mangle_names,
	output::generate_output,
	top_down_type::top_down_type,
	type_specialise::type_specialise,
};
use super::Model;
use crate::Db;

pub mod capturing_fn;
pub mod compat;
pub mod comprehension;
pub mod dead_code;
pub mod domain_constraint;
pub mod erase_enum;
pub mod erase_opt;
pub mod erase_record;
pub mod function_dispatch;
pub mod inlining;
pub mod name_mangle;
pub mod output;
pub mod top_down_type;
pub mod totalise;
pub mod type_specialise;

/// A THIR transform function
pub type TransformFn = for<'db> fn(&'db dyn Db, Model<'db>) -> Result<Model<'db>>;

/// Create a transformer which runs the given transforms in order on an initial model
pub fn transformer(
	transforms: Vec<TransformFn>,
) -> impl for<'db> FnMut(&'db dyn Db, Model<'db>) -> Result<Model<'db>> {
	let mut iter = transforms.into_iter();
	move |db, model| {
		iter.by_ref()
			.try_fold(model, |m, transform| transform(db, m))
	}
}

/// Get the default THIR transformer
pub fn thir_transforms() -> impl for<'db> FnMut(&'db dyn Db, Model<'db>) -> Result<Model<'db>> {
	let fns = vec![
		eliminate_dead_code_conservative,
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
		// decapture_model,
		inline_functions,
		totalise,
		eliminate_dead_code,
		old_compat,
	];
	transformer(fns)
}

#[cfg(test)]
pub(crate) mod tests {
	use expect_test::Expect;
	use rustc_hash::FxHashMap;
	use salsa::Setter;
	use shackle_diagnostics::Result;
	use shackle_hir::{
		CompilerDatabase, Db, Identifier,
		ids::NodeRef,
		input::{CompilerSettings, InlineModelFile, InputFiles, ModelFile},
	};
	use shackle_syntax::InputLang;

	use crate::{
		AnnotationId, DeclarationId, ItemId, Model, ResolvedIdentifier,
		db::final_thir,
		lower::lower_model,
		pretty_print::PrettyPrinter,
		traverse::{Visitor, visit_annotation, visit_declaration},
	};

	#[test]
	fn test_thir_transforms() {
		let mut db = CompilerDatabase::default();
		let file = InlineModelFile::new(&db, "".to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![file.into()]);
		assert!(final_thir(&db).is_ok());
	}

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// The expected value only includes items which are from the `source` (i.e. not from stdlib).
	pub(crate) fn check<F>(transform: F, source: &str, expected: Expect)
	where
		F: for<'db> FnOnce(&'db dyn Db, Model<'db>) -> Result<Model<'db>>,
	{
		let mut db = CompilerDatabase::default();
		let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = lower_model(&db);
		let pretty = match transform(&db, model.take()) {
			Ok(mut result) => {
				let to_print = NameMapper::default().run(&db, model_file, &mut result);
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
	pub(crate) fn check_no_stdlib<F>(transform: F, source: &str, expected: Expect)
	where
		F: for<'db> FnOnce(&'db dyn Db, Model<'db>) -> Result<Model<'db>>,
	{
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = lower_model(&db);
		let pretty = match transform(&db, model.take()) {
			Ok(result) => PrettyPrinter::new(&db, &result).pretty_print(),
			Err(e) => e.to_string(),
		};
		expected.assert_eq(&pretty);
	}

	#[derive(Default)]
	pub(crate) struct NameMapper<'db> {
		annotation: FxHashMap<AnnotationId<'db>, usize>,
		declaration: FxHashMap<DeclarationId<'db>, usize>,
	}

	impl<'db> Visitor<'_, 'db> for NameMapper<'db> {
		fn visit_annotation(&mut self, model: &Model<'db>, annotation: AnnotationId<'db>) {
			if model[annotation].name.is_none() {
				let count = self.annotation.len();
				let _ = self.annotation.entry(annotation).or_insert(count);
			}
			visit_annotation(self, model, annotation)
		}

		fn visit_declaration(&mut self, model: &Model<'db>, declaration: DeclarationId<'db>) {
			if model[declaration].name().is_none() {
				let count = self.declaration.len();
				let _ = self.declaration.entry(declaration).or_insert(count);
			}
			visit_declaration(self, model, declaration);
		}

		fn visit_identifier(&mut self, model: &Model<'db>, identifier: &ResolvedIdentifier<'db>) {
			match identifier {
				ResolvedIdentifier::Annotation(ann) => self.visit_annotation(model, *ann),
				ResolvedIdentifier::Declaration(decl) => self.visit_declaration(model, *decl),
				_ => (),
			}
		}
	}

	impl<'db> NameMapper<'db> {
		pub(crate) fn run(
			&mut self,
			db: &'db dyn Db,
			model_ref: ModelFile,
			model: &mut Model<'db>,
		) -> Vec<ItemId<'db>> {
			let to_print = model
				.top_level_items()
				.filter(|it| match model.item_origin(*it).node() {
					Some(NodeRef::Item(item)) => item.model_file(db) == model_ref,
					Some(NodeRef::Entity(entity)) => entity.item(db).model_file(db) == model_ref,
					Some(NodeRef::Model(m)) => m == model_ref,
					None => true,
				})
				.collect::<Vec<_>>();
			for item in to_print.iter() {
				self.visit_item(model, *item);
			}
			for (ann, n) in self.annotation.iter() {
				model[*ann].name = Some(Identifier::new(db, format!("_ANN_{}", *n + 1)));
			}
			for (decl, n) in self.declaration.iter() {
				model[*decl].set_name(Identifier::new(db, format!("_DECL_{}", *n + 1)));
			}
			to_print
		}
	}
}
