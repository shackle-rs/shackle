//! Functionality for converting AST nodes to HIR nodes
//!
//! The AST is lowered to HIR by performing the following syntactic desugarings:
//!

mod expression;
mod item;
#[cfg(test)]
mod test;

pub use self::{expression::*, item::*};

/*
===================
Temporary Checklist
===================
source_file
param_decl
const_def (done) (tested)
domain_alias
decision_decl
objective
branching
constraint
heuristic
_expression
call (done) (tested)
quantification
matrix_comprehension (help)
generator (help)
indexed_access (done) (tested)
infix_operator (done) (tested)
absolute_operator (done) (tested)
prefix_operator (done) (tested)
postfix_operator (done) (tested)
_domain
matrix_domain
_base_domain
domain_operation
boolean_domain
integer_domain
matrix_literal
boolean_literal (done) (tested)
integer_literal (done) (tested)
identifier (done) (tested)


TODO: Support for lex comparison operators
*/
