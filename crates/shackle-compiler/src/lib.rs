//! Shackle library

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

pub use diagnostics::Error;

// Export OptType enumeration used in [`Type`]
pub use ty::OptType;

/// Result type for Shackle operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use diagnostics::Warning;

#[cfg(test)]
mod tests {}
