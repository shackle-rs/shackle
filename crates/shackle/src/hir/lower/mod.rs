//! Functionality for converting AST nodes to HIR nodes
//!
//! The AST is lowered to HIR by performing the following syntactic desugarings:
//!
//! - predicate/test rewritten as functions
//! - prefix/infix/postfix operators rewritten as calls
//! - generator calls rewritten as calls using array comprehensions
//! - string interpolation rewritten into `concat` of `show` calls
//! - indexed array literals rewritten into `arrayNd` calls
//! - 2D array literals rewritten into array2d calls
//!
//! Any performed desugaring steps need to be recorded in case error messages
//! need the information to avoid referring to an introduced construct. Otherwise,
//! the desugaring step must be formulated to guarantee that no future error
//! messages could refer to non-user-written constructs.
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
