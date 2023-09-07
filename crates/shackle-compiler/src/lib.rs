//! Shackle internal compiler library.
//!
//! This library is considered internal and no stability guarantees are given.

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

pub mod constants;
pub mod db;
pub mod diagnostics;
pub mod file;
pub mod hir;
pub mod mir;
pub mod syntax;
pub mod thir;
pub mod ty;
pub mod utils;

pub use db::CompilerDatabase;
pub use diagnostics::{Error, Warning};

/// Result type for Shackle operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {}
