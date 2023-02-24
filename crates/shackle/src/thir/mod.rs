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
pub mod type_specialise;

mod ir;

pub use self::ir::*;
pub use crate::hir::Identifier;
