//! The error produced by the fzn parser.

use std::fmt::Display;

use winnow::error::{ContextError, ParseError};

use crate::fzn::Stream;

/// Errors that can occur when parsing `.fzn` models.
#[derive(Debug)]
pub enum FznParseError {
	/// Error reading from the source.
	Io(std::io::Error),
	/// Error converting to utf8.
	Utf8Error(std::str::Utf8Error),
	/// Missing solve item in the model.
	MissingSolveItem,
	/// Multiple solve items were encountered in the model.
	MultipleSolveItems,
	/// An error in the syntax of the `fzn`.
	SyntaxError(String),
	/// An error in the syntax of the `fzn`.
	IdentifierError {
		/// The string attempted to parse as an identifier.
		ident: String,
		/// The error that occurred.
		err: String,
	},
}

impl Display for FznParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FznParseError::Io(error) => write!(f, "error reading from source: {error}"),
			FznParseError::Utf8Error(error) => write!(f, "invalid utf8: {error}"),
			FznParseError::MissingSolveItem => write!(f, "missing solve item"),
			FznParseError::MultipleSolveItems => write!(f, "multiple solve items"),
			FznParseError::SyntaxError(error) => write!(f, "syntax error: {error}"),
			FznParseError::IdentifierError { ident, err } => {
				write!(f, "error parsing identifier `{ident}`: {err}")
			}
		}
	}
}

impl<I> From<ParseError<Stream<'_, '_, I>, ContextError>> for FznParseError {
	fn from(value: ParseError<Stream<'_, '_, I>, ContextError>) -> Self {
		FznParseError::SyntaxError(value.to_string())
	}
}

impl From<std::io::Error> for FznParseError {
	fn from(value: std::io::Error) -> Self {
		FznParseError::Io(value)
	}
}

impl From<std::str::Utf8Error> for FznParseError {
	fn from(value: std::str::Utf8Error) -> Self {
		FznParseError::Utf8Error(value)
	}
}

impl std::error::Error for FznParseError {}
