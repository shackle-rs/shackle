//! Error types for the XCSP3 serde crate.

use thiserror::Error;

#[derive(Clone, Debug, Error)]
/// Error type returned when instantiating a [`Group`] fails.
pub enum UnrollError {
	#[error("placeholder with index {placeholder} cannot be instantiated using only {args_len} arguments")]
	/// Missing argument value
	ArgMissing {
		/// The index of the missing argument.
		placeholder: usize,
		/// The number of arguments provided.
		args_len: usize,
	},
	#[error("placeholder in position that expects type {placeholder_ty}, but got argument of type {arg_ty}")]
	/// Invalid type for argument
	InvalidType {
		/// Type deduced by the position of the placeholder.
		placeholder_ty: &'static str,
		/// Type of the argument expression provided.
		arg_ty: &'static str,
	},
	/// Unexpected number of expressions
	#[error(
		"unrolling expression evaluated to {args_len} expressions, but expected {expected_len}"
	)]
	UnexpectedLength {
		/// The expected number of expressions
		expected_len: usize,
		/// The number of expressions that unrolling evaluated to.
		args_len: usize,
	},
	/// Unexpected number of indexes in array access
	#[error("array access with {args_len} indexes, but expected {expected_len} indexes")]
	UnexpectedIndexes {
		/// The expected number of indexing operations
		expected_len: usize,
		/// The number of indexing operations found.
		args_len: usize,
	},
	/// Unknown identifier
	#[error("unknown identifier {0}")]
	UnknownIdentifier(String),
}
