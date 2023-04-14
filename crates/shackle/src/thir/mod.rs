//! Typed high-level intermediate representation.
//!
//! This module provides (almost) all constructs available in the HIR, along
//! with type and name resolution information computed during typechecking.
//!
//! Since this phase is post-HIR, it is not designed to be incremental.
//! An API is provided to allow us to perform transformations/modifications.
//!
//! This representation is used to generate the MIR.

pub mod db;
pub mod lower;
pub mod pretty_print;
pub mod sanity_check;
pub mod source;
pub mod transform;

mod ir;

pub use self::ir::*;
use self::pretty_print::PrettyPrinter;
use crate::db::CompilerDatabase;
pub use crate::hir::Identifier;

/// Print the model for debugging
#[no_mangle]
pub fn debug_print_thir_model(db: &CompilerDatabase, model: &Model) {
	eprintln!("{}", PrettyPrinter::new(db, model).pretty_print());
}

/// Print an item for debugging
#[no_mangle]
pub fn debug_print_thir_item(db: &CompilerDatabase, model: &Model, item: ItemId) {
	eprintln!("{}", PrettyPrinter::new(db, model).pretty_print_item(item));
}

/// Print an expression for debugging
#[no_mangle]
pub fn debug_print_thir_expression(db: &CompilerDatabase, model: &Model, expression: &Expression) {
	eprintln!(
		"{}",
		PrettyPrinter::new(db, model).pretty_print_expression(expression),
	);
}
