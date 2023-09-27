//! Functionality for converting AST nodes to HIR nodes
//!
//! The AST is lowered to HIR by performing the following syntactic desugarings:
//!

mod expression;
mod item;
#[cfg(test)]
mod test;

pub use self::{expression::*, item::*};
