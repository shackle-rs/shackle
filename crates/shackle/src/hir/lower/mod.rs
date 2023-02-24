//! Functionality for converting AST nodes to HIR nodes
//!
//! The AST is lowered to HIR by performing the following syntactic desugarings:
//!
//! - predicate/test rewritten as functions
//! - prefix/infix/postfix operators rewritten as calls
//! - generator calls rewritten as calls using array comprehensions
//! - string interpolation rewritten into `concat` of `show` calls
//!
//! Any performed desugaring steps need must be formulated to guarantee that no
//! future error messages could refer to non-user-written constructs.
//!
//! During lowering, the AST is also partially validated:
//!
//! - reject invalid array literals
//!   - non-uniform 2d array literals
//!   - mixing index kinds
//!

mod expression;
mod item;

pub use self::expression::*;
pub use self::item::*;

#[cfg(test)]
mod test;
