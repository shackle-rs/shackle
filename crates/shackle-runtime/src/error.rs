use miette::Diagnostic;
use thiserror::Error;

/// Undefined arithmetic operation
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Undefined arithmetic operation")]
#[diagnostic(code(shackle::arithmetic_error))]
pub struct ArithmeticError {
	/// The reason for the undefined operation
	pub reason: &'static str,
}
