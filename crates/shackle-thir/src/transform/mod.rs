//! THIR model transformations.
//!
//! These transformations take a model as input and output a new transformed model.
//! The `crate::Visitor` and `crate::Folder` traits are useful for implementing these.
//! It is the responsibility of implementors to know what constructs are expected to be present at the stage they run.

use salsa::Setter;
use shackle_diagnostics::Result;
use shackle_hir::diagnostics::Errors;
use totalise::totalise;

use self::{
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
use crate::{Db, lower::lower_model};

// pub mod capturing_fn;
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

/// Transforms which can be applied to a model
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Transform {
	/// Eliminate dead code conservatively
	EliminateDeadCodeConservative,
	/// Create output variables
	GenerateOutput,
	/// Rewrite domain constraints into constraints
	RewriteDomains,
	/// Determine true types of bottom expressions
	TopDownType,
	/// Instantiate polymorphic calls
	TypeSpecialise,
	/// Add subinst function dispatch headers
	FunctionDispatch,
	/// Mangle names of overloaded functions
	MangleNames,
	/// Erase records into tuples
	EraseRecord,
	/// Erase enums into ints
	EraseEnum,
	/// Desugar comprehensions
	DesugarComprehension,
	/// Erase option types
	EraseOpt,
	/// Inline function calls
	InlineFunctions,
	/// Totalise the model
	Totalise,
	/// Eliminate dead code aggressively
	EliminateDeadCode,
}

impl Transform {
	/// Run this transform on a model
	pub fn run<'db>(&self, db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
		match self {
			Transform::EliminateDeadCodeConservative => eliminate_dead_code_conservative(db, model),
			Transform::GenerateOutput => generate_output(db, model),
			Transform::RewriteDomains => rewrite_domains(db, model),
			Transform::TopDownType => top_down_type(db, model),
			Transform::TypeSpecialise => type_specialise(db, model),
			Transform::FunctionDispatch => function_dispatch(db, model),
			Transform::MangleNames => mangle_names(db, model),
			Transform::EraseRecord => erase_record(db, model),
			Transform::EraseEnum => erase_enum(db, model),
			Transform::DesugarComprehension => desugar_comprehension(db, model),
			Transform::EraseOpt => erase_opt(db, model),
			Transform::InlineFunctions => inline_functions(db, model),
			Transform::Totalise => totalise(db, model),
			Transform::EliminateDeadCode => eliminate_dead_code(db, model),
		}
	}
}

/// Default transforms to run on a model
pub const DEFAULT_TRANSFORMS: &[Transform] = &[
	Transform::EliminateDeadCodeConservative,
	Transform::GenerateOutput,
	Transform::RewriteDomains,
	Transform::TopDownType,
	Transform::TypeSpecialise,
	Transform::FunctionDispatch,
	Transform::MangleNames,
	Transform::EraseRecord,
	Transform::EraseEnum,
	Transform::DesugarComprehension,
	Transform::EraseOpt,
	Transform::InlineFunctions,
	Transform::Totalise,
	Transform::EliminateDeadCode,
];

#[salsa::input(debug, singleton)]
struct TransformerSingleton {
	/// The transforms, in order
	pub transforms: Vec<Transform>,
}

/// A transformer which runs a sequence of transforms on a model
#[derive(Copy, Clone)]
pub struct Transformer;

impl std::fmt::Debug for Transformer {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Transformer").finish()
	}
}

impl Transformer {
	/// Run the transformer on the model
	pub fn run<'db>(db: &'db dyn Db) -> Result<&'db Model<'db>> {
		let model = run_thir_transforms(db);
		let error = run_thir_transforms::accumulated::<Errors>(db).pop();
		if let Some(e) = error {
			Err((**e).clone())
		} else {
			Ok(model)
		}
	}

	/// Get the transforms
	pub fn get_transforms(db: &dyn Db) -> &[Transform] {
		TransformerSingleton::try_get(db)
			.unwrap_or_else(|| TransformerSingleton::new(db, DEFAULT_TRANSFORMS.to_vec()))
			.transforms(db)
	}

	/// Set the transforms
	pub fn set_transforms(db: &mut dyn Db, transforms: impl IntoIterator<Item = Transform>) {
		let _ = TransformerSingleton::try_get(db)
			.unwrap_or_else(|| TransformerSingleton::new(db, vec![]))
			.set_transforms(db)
			.to(transforms.into_iter().collect());
	}
}

#[salsa::tracked]
fn run_thir_transforms<'db>(db: &'db dyn Db) -> Model<'db> {
	let mut model = lower_model(db).take();
	for transform in Transformer::get_transforms(db) {
		match transform.run(db, model) {
			Ok(transformed) => model = transformed,
			Err(error) => {
				Errors::add(db, error);
				return Model::default();
			}
		}
	}
	model
}

/// A THIR transform function
pub type TransformFn = for<'db> fn(&'db dyn Db, Model<'db>) -> Result<Model<'db>>;

#[cfg(test)]
pub(crate) mod tests {
	use std::fmt::Write;

	use expect_test::Expect;
	use rustc_hash::FxHashMap;
	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase, Db, Identifier,
		ids::NodeRef,
		input::{CompilerSettings, InlineModelFile, InputFiles, ModelFile},
	};
	use shackle_syntax::InputLang;

	use crate::{
		AnnotationId, DeclarationId, Model, ResolvedIdentifier,
		db::final_thir,
		pretty_print::{
			PrettyPrinter, Printer, print_annotation_id, print_declaration_id, print_item,
		},
		transform::{Transform, Transformer},
		traverse::{Visitor, visit_annotation, visit_declaration},
	};

	pub(crate) trait TransformList {
		fn set_transforms(self, db: &mut dyn Db);
	}

	impl TransformList for Transform {
		fn set_transforms(self, db: &mut dyn Db) {
			Transformer::set_transforms(db, vec![self]);
		}
	}

	impl<T: IntoIterator<Item = Transform>> TransformList for T {
		fn set_transforms(self, db: &mut dyn Db) {
			Transformer::set_transforms(db, self);
		}
	}

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
	pub(crate) fn check(transform: impl TransformList, source: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		transform.set_transforms(&mut db);
		let pretty = match final_thir(&db) {
			Ok(result) => NameMapper::default().run(&db, model_file, result),
			Err(e) => e.to_string(),
		};
		expected.assert_eq(&pretty);
	}

	/// Perform a transform on the THIR, and verify the result matches an expected value.
	///
	/// Turns off stdlib inclusion.
	pub(crate) fn check_no_stdlib(transform: impl TransformList, source: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		transform.set_transforms(&mut db);
		let pretty = match final_thir(&db) {
			Ok(result) => PrettyPrinter::new(&db, result).pretty_print(),
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

	impl<'db> Printer<'db, ()> for NameMapper<'db> {
		fn print_annotation_id(
			&self,
			db: &'db dyn Db,
			model: &Model<'db, ()>,
			a: AnnotationId<'db, ()>,
		) -> String {
			if model[a].name.is_none()
				&& let Some(n) = self.annotation.get(&a)
			{
				return Identifier::new(db, format!("_ANN_{}", *n + 1)).pretty_print(db);
			}
			print_annotation_id(self, db, model, a)
		}

		fn print_declaration_id(
			&self,
			db: &'db dyn Db,
			model: &Model<'db, ()>,
			d: DeclarationId<'db, ()>,
		) -> String {
			if model[d].name().is_none()
				&& let Some(n) = self.declaration.get(&d)
			{
				return Identifier::new(db, format!("_DECL_{}", *n + 1)).pretty_print(db);
			}
			print_declaration_id(self, db, model, d)
		}
	}

	impl<'db> NameMapper<'db> {
		pub(crate) fn run(
			&mut self,
			db: &'db dyn Db,
			model_ref: ModelFile,
			model: &Model<'db>,
		) -> String {
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
			let mut buf = String::new();
			for item in to_print {
				writeln!(&mut buf, "{};", print_item(self, db, model, item)).unwrap();
			}
			buf
		}
	}
}
