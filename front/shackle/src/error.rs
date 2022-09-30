//! Error handling

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use std::fmt::{Display, Formatter};
use std::panic::Location;
use std::path::PathBuf;

use crate::file::SourceFile;

/// An error internal to Shackle.
///
/// Encountering this error indicates a bug in Shackle.
#[derive(Diagnostic, Debug, Error, PartialEq, Eq, Clone)]
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
	pub fn new(msg: impl AsRef<str>) -> InternalError {
		let loc = Location::caller();
		InternalError {
			msg: msg.as_ref().to_string(),
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

/// Multiple errors
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Multiple errors")]
#[diagnostic()]
pub struct MultipleErrors {
	/// The errors
	#[related]
	pub errors: Vec<ShackleError>,
}

/// A file error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Could not read file {file:?}")]
#[diagnostic(code(shackle::io_error))]
pub struct FileError {
	/// The file path
	pub file: PathBuf,
	/// The underlying error message
	pub message: String,
	/// Other related file errors
	#[related]
	pub other: Vec<FileError>,
}

/// A syntax error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Syntax Error")]
#[diagnostic(code(shackle::syntax_error))]
pub struct SyntaxError {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
	/// The error message
	pub msg: String,
	/// Related syntax errors
	#[related]
	pub other: Vec<SyntaxError>,
}

/// Could not resolve include
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Include error")]
#[diagnostic(code(shackle::include_error))]
pub struct IncludeError {
	/// The included path string
	pub include: String,
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("Failed to resolve include \"{include}\".")]
	pub span: SourceSpan,
}

/// Multiple solve items error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Multiple solve items not allowed")]
#[diagnostic(code(shackle::multiple_solve_items), severity(advice))]
pub struct MultipleSolveItems {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("The first solve item was defined here")]
	pub span: SourceSpan,
	/// The additional solve item errors
	#[related]
	pub others: Vec<AdditionalSolveItem>,
}

/// Indicates an extraneous solve item
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Multiple solve items not allowed")]
#[diagnostic(
	code(shackle::multiple_solve_items),
	help("Try removing this solve item.")
)]
pub struct AdditionalSolveItem {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label]
	pub span: SourceSpan,
}

/// An undefined identifier error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Undefined identifier")]
#[diagnostic(code(shackle::undefined_identifier))]
pub struct UndefinedIdentifier {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("{identifier} is undefined")]
	pub span: SourceSpan,
	/// The identifier which is undefined
	pub identifier: String,
}

/// An identifier already defined error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Identifier already defined")]
#[diagnostic(code(shackle::identifier_already_defined))]
pub struct IdentifierAlreadyDefined {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("{identifier} already defined")]
	pub span: SourceSpan,
	/// The identifier which is already defined
	pub identifier: String,
}

/// An invalid pattern error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Invalid pattern used")]
#[diagnostic(code(shackle::invalid_pattern))]
pub struct InvalidPattern {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Illegal type
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Illegal type")]
#[diagnostic(code(shackle::illegal_type))]
pub struct IllegalType {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("The type '{ty}' is not allowed.")]
	pub span: SourceSpan,
	/// The illegal type
	pub ty: String,
}

/// A type mismatch error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Type mismatch")]
#[diagnostic(code(shackle::type_mismatch))]
pub struct TypeMismatch {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Invalid array literal
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Invalid array literal")]
#[diagnostic(code(shackle::invalid_array_literal))]
pub struct InvalidArrayLiteral {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// No matching function found
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("No matching function")]
#[diagnostic(code(shackle::no_matching_fn))]
pub struct NoMatchingFunction {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Ambiguous call
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Ambiguous call")]
#[diagnostic(code(shackle::ambiguous_call))]
pub struct AmbiguousCall {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Illegal overloading
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Illegal overloading")]
#[diagnostic(code(shackle::illegal_overload))]
pub struct IllegalOverload {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Type inference failure
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Type cannot be determined")]
#[diagnostic(code(shackle::type_inference_failure))]
pub struct TypeInferenceFailure {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Invalid field access
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Invalid field access")]
#[diagnostic(code(shackle::invalid_field_access))]
pub struct InvalidFieldAccess {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Main Shackle error type
#[derive(Error, Diagnostic, Debug, PartialEq, Eq, Clone)]
pub enum ShackleError {
	/// Multiple errors
	#[error(transparent)]
	#[diagnostic(transparent)]
	MultipleErrors(#[from] MultipleErrors),
	/// A File IO error
	#[error(transparent)]
	#[diagnostic(transparent)]
	FileError(#[from] FileError),
	/// A syntax error
	#[error(transparent)]
	#[diagnostic(transparent)]
	SyntaxError(#[from] SyntaxError),
	/// Failed to find standard library
	#[error("Failed to located standard library.")]
	#[diagnostic(code(shackle::stdlib_not_found))]
	StandardLibraryNotFound,
	/// Include error
	#[error(transparent)]
	#[diagnostic(transparent)]
	IncludeError(#[from] IncludeError),
	/// Multiple solve items
	#[error(transparent)]
	#[diagnostic(transparent)]
	MultipleSolveItems(#[from] MultipleSolveItems),
	/// Identifier already declared
	#[error(transparent)]
	#[diagnostic(transparent)]
	IdentifierAlreadyDefined(#[from] IdentifierAlreadyDefined),
	/// Undefined identifier
	#[error(transparent)]
	#[diagnostic(transparent)]
	UndefinedIdentifier(#[from] UndefinedIdentifier),
	/// Invalid pattern
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidPattern(#[from] InvalidPattern),
	/// Illegal type
	#[error(transparent)]
	#[diagnostic(transparent)]
	IllegalType(#[from] IllegalType),
	/// Type mismatch
	#[error(transparent)]
	#[diagnostic(transparent)]
	TypeMismatch(#[from] TypeMismatch),
	/// Invalid array literal
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidArrayLiteral(#[from] InvalidArrayLiteral),
	/// No matching function found
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoMatchingFunction(#[from] NoMatchingFunction),
	/// Ambiguous call
	#[error(transparent)]
	#[diagnostic(transparent)]
	AmbiguousCall(#[from] AmbiguousCall),
	/// Illegal overloading
	#[error(transparent)]
	#[diagnostic(transparent)]
	IllegalOverload(#[from] IllegalOverload),
	/// Type inference failure
	#[error(transparent)]
	#[diagnostic(transparent)]
	TypeInferenceFailure(#[from] TypeInferenceFailure),
	/// Invalid field access
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidFieldAccess(#[from] InvalidFieldAccess),
	/// An internal error
	#[error("Internal Error - Please report this issue to the Shackle developers")]
	InternalError(#[from] InternalError),
}
