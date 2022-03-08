//! Error handling

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use std::fmt::{Display, Formatter};
use std::panic::Location;
use std::path::PathBuf;

pub use miette::NamedSource;

/// An error internal to Shackle.
///
/// Encountering this error indicates a bug in Shackle.
#[derive(Diagnostic, Debug, Error)]
#[diagnostic(code(shackle::internal_error))]
pub struct InternalError {
	/// The error message
	pub msg: String,
	/// An optional location for the error
	pub loc: Option<(String, u32, u32)>,
}

impl InternalError {
	/// Create an internal error with the given message
	#[track_caller]
	pub fn new(msg: String) -> InternalError {
		let loc = Location::caller();
		InternalError {
			msg,
			loc: Some((loc.file().to_string(), loc.line(), loc.column())),
		}
	}
}

impl Display for InternalError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		if let Some((file, line, col)) = &self.loc {
			write!(f, "{}:{}:{} ", file, line, col)?;
		}
		write!(f, "{}", self.msg)
	}
}

/// A syntax error
#[derive(Error, Debug, Diagnostic)]
#[error("Syntax Error")]
#[diagnostic(code(shackle::syntax_error))]
pub struct SyntaxError {
	/// The source code
	#[source_code]
	pub src: NamedSource,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
	/// The error message
	pub msg: String,
	/// Related syntax errors
	#[related]
	pub other: Vec<SyntaxError>,
}

/// Main Shackle error type
#[derive(Error, Diagnostic, Debug)]
pub enum ShackleError {
	/// A File IO error
	#[error("Could not read file {file:?}")]
	#[diagnostic(code(shackle::io_error))]
	FileError {
		/// The file path
		file: PathBuf,
		/// The underlying error
		source: std::io::Error,
	},
	/// A syntax error
	#[error(transparent)]
	#[diagnostic(transparent)]
	SyntaxError(#[from] SyntaxError),
	/// A type error
	#[error("Something did not match up")]
	#[diagnostic(code(shackle::type_mismatch))]
	TypeError,
	/// An internal error
	#[error("Internal Error - Please report this issue to the Shackle developers")]
	InternalError(#[from] InternalError),
}
