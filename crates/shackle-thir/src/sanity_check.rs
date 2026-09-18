//! Sanity checks for THIR.
//!

use salsa::Setter;
use shackle_diagnostics::Error;
use shackle_hir::{
	CompilerDatabase,
	input::{CompilerSettings, InlineModelFile, InputFiles},
	run_hir_phase,
};
use shackle_syntax::InputLang;
use shackle_utils::maybe_grow_stack;

use crate::{
	Db, Marker, Model,
	lower::lower_model,
	pretty_print::{Printer, print_expression, print_model},
};

/// Get the diagnostics for running the pretty printed THIR.
///
/// This should give no errors (as for the THIR to exist, it must have come
/// from a valid source program).
pub fn sanity_check_thir(db: &dyn Db) -> Vec<Error> {
	let initial_thir = lower_model(db);
	let model = initial_thir.get();

	// Pretty print with extra info for sanity checking types
	let code = print_model(&TypeAnnotatedPrettyPrinter, db, model.as_ref());

	let mut new_db = CompilerDatabase::default();
	let _ = CompilerSettings::get(&new_db)
		.set_ignore_stdlib(&mut new_db)
		.to(true);
	let model_file = InlineModelFile::new(&new_db, code, InputLang::MiniZinc).into();
	let _ = InputFiles::get(&new_db)
		.set_files(&mut new_db)
		.to(vec![model_file]);
	run_hir_phase(&new_db).errors.into_iter().cloned().collect()
}

struct TypeAnnotatedPrettyPrinter;

impl<'db, T: Marker> Printer<'db, T> for TypeAnnotatedPrettyPrinter {
	fn print_expression(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		expression: &crate::Expression<'db, T>,
	) -> String {
		maybe_grow_stack(|| {
			let inner = print_expression(self, db, model, expression);
			format!(
				"({} :: shackle_type({:?}))",
				inner,
				expression.ty().pretty_print(db)
			)
		})
	}
}
