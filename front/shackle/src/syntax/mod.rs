//! Syntax representations.
//!
//! A concrete syntax tree is created by the parser.
//! This completely represents the source text, including comments and whitespace.
//!
//! Since this is not convenient for most stages of compilation, an abstract syntax tree is
//! generated which provides type-safe access to children. The AST includes all language constructs
//! (i.e. no desugaring is performed, just removal of non-semantic nodes).
//!
//! The AST is then lowered into HIR, which is the main representation used by the compiler.
//!
pub mod ast;
pub mod cst;
pub mod db;
