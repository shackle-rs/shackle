use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use std::fmt::{Display, Formatter};
use std::panic::Location;
use std::path::PathBuf;

pub use miette::NamedSource;

#[derive(Diagnostic, Debug, Error)]
#[diagnostic(code(shackle::internal_error))]
pub struct InternalError {
	pub msg: String,
	pub loc: Option<(String, u32, u32)>,
}

impl InternalError {
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

#[derive(Error, Debug, Diagnostic)]
#[error("Syntax Error")]
#[diagnostic(code(shackle::syntax_error))]
pub struct SyntaxError {
	#[source_code]
	pub src: NamedSource,
	#[label("{msg}")]
	pub span: SourceSpan,
	pub msg: String,
	#[related]
	pub other: Vec<SyntaxError>,
}

#[derive(Error, Diagnostic, Debug)]
pub enum ShackleError {
	#[error("Could not read file {file:?}")]
	#[diagnostic(code(shackle::io_error))]
	FileError {
		file: PathBuf,
		source: std::io::Error,
	},

	#[error(transparent)]
	#[diagnostic(transparent)]
	SyntaxError(#[from] SyntaxError),
	#[error("Something did not match up")]
	#[diagnostic(code(shackle::type_mismatch))]
	TypeError,
	#[error("Internal Error - Please report this issue to the Shackle developers")]
	InternalError(#[from] InternalError),
}
