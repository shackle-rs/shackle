//! Sanity checks for THIR.
//!

use std::sync::Arc;

use crate::{
	db::{CompilerDatabase, Inputs},
	diagnostics::Diagnostics,
	file::InputFile,
	hir::db::Hir,
	Error,
};

use super::{db::Thir, pretty_print::PrettyPrinter};

/// Get the diagnostics for running the pretty printed THIR.
///
/// This should give no errors (as for the THIR to exist, it must have come
/// from a valid source program).
pub fn sanity_check_thir(db: &dyn Thir) -> Arc<Diagnostics<Error>> {
	let model = db.model_thir();

	// Pretty print with extra info for sanity checking types
	let mut printer = PrettyPrinter::new(db, &model);
	printer.old_compat = false;
	printer.debug_types = true;
	let code = printer.pretty_print();

	let mut new_db = CompilerDatabase::default();
	new_db.set_ignore_stdlib(true);
	new_db.set_input_files(Arc::new(vec![InputFile::ModelString(code)]));
	new_db.all_errors()
}
