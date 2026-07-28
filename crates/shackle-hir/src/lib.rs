#![recursion_limit = "256"]

//! High-level intermediate representation.
//!
//! This representation is used for name resolution and type checking.
//!
//! This is also the final representation used by the language server, and as
//! such is the final representation which needs to be continue as far as
//! possible in the presence of errors.
//!
//! The steps which occur using this representation are
//! - Include resolution (see the `input` module)
//! - Lowering of AST to HIR (see the `lower` module)
//! - Scope collection (see the `scope` module)
//! - Computing types of expressions and declarations, identifier resolution
//!   (see the `typecheck` module)
//! - Topological sorting of items (see the `toposort` module)
//! - Checking case expressions for exhaustiveness (see the `pattern_matching`)
//!   module
//! - Validation of whole program (see the `validate` module)
//!
//! The overall process is orchestrated by the `run_hir_phase` function.

pub mod class_analysis;
pub mod constants;
pub mod counts;
pub mod db;
pub mod diagnostics;
pub mod input;
pub mod interface;
pub mod ir;
pub mod lower;
pub mod object_validation;
pub mod overloading;
pub mod pattern_matching;
pub mod scope;
pub mod source;
pub mod toposort;
pub mod typecheck;
pub mod validate;

pub use db::{CompilerDatabase, Db};
pub use ir::*;
pub use scope::*;
use shackle_diagnostics::{Error, Warning};
pub use typecheck::*;

use crate::{
	diagnostics::{Errors, Warnings},
	input::{accumulate_syntax_errors, resolve_includes},
	lower::{accumulate_lower_errors, lower_models},
	object_validation::validate_object_lowering,
	pattern_matching::check_case_exhaustiveness,
	toposort::topological_sort,
	validate::validate_hir,
};

/// Result of running the HIR phase
#[derive(Debug)]
pub struct HirResult<'db> {
	/// The program items (topologically sorted)
	pub items: &'db Vec<Item<'db>>,
	/// Errors produced during the HIR phase
	pub errors: Vec<&'db Error>,
	/// Warnings produced during the HIR phase
	pub warnings: Vec<&'db Warning>,
}

/// Runs the HIR phase
///
/// This runs all the steps of the HIR phase and forces a reasonable order
/// of execution so that the logs are easier to understand.
pub fn run_hir_phase<'db>(db: &'db dyn Db) -> HirResult<'db> {
	run_hir_phase_internal(db);
	// for model in resolve_includes(db).iter() {
	// At this point we shouldn't the text content anymore
	// (diagnostics will have their own copies of the required source)
	// model.release(db);
	// }
	HirResult {
		items: topological_sort(db),
		errors: run_hir_phase_internal::accumulated::<Errors>(db)
			.into_iter()
			.map(|e| &**e)
			.collect(),
		warnings: run_hir_phase_internal::accumulated::<Warnings>(db)
			.into_iter()
			.map(|w| &**w)
			.collect(),
	}
}

/// Get all errors that occurred during the HIR phase
pub fn all_errors(db: &dyn Db) -> Vec<&Error> {
	run_hir_phase_internal(db);
	run_hir_phase_internal::accumulated::<Errors>(db)
		.into_iter()
		.map(|e| &**e)
		.collect()
}

/// Get all warnings that occurred during the HIR phase
pub fn all_warnings(db: &dyn Db) -> Vec<&Warning> {
	run_hir_phase_internal(db);
	run_hir_phase_internal::accumulated::<Warnings>(db)
		.into_iter()
		.map(|w| &**w)
		.collect()
}

#[salsa::tracked]
fn run_hir_phase_internal(db: &dyn Db) {
	let _ = resolve_includes(db);
	accumulate_syntax_errors(db);
	let _ = lower_models(db);
	accumulate_lower_errors(db);
	collect_scopes(db);
	accumulate_scope_diagnostics(db);
	typecheck(db);
	accumulate_typecheck_diagnostics(db);
	let _ = topological_sort(db);
	check_case_exhaustiveness(db);
	validate_hir(db);
	validate_object_lowering(db);
}
