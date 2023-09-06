//! Utilities for traversing THIR trees.
//!
//! The `Folder` trait is used to create a modified copy of the model (i.e. a tree-to-tree transformation).
//! The `Visitor` trait is used to visit a model recursively (without copying it).
//!

mod fold;
mod visit;

pub use self::{fold::*, visit::*};
