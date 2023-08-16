//! Error handling

use miette::{Diagnostic, SourceOffset, SourceSpan};
use thiserror::Error;

use std::fmt::{Display, Formatter};
use std::panic::Location;
use std::path::PathBuf;

use crate::file::SourceFile;

use super::Diagnostics;

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

/// Multiple assignments to same variable error
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("The variable '{variable}' is already assigned")]
#[diagnostic(code(shackle::multiple_assignments), severity(advice))]
pub struct MultipleAssignments {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("The first assignment was here")]
	pub span: SourceSpan,
	/// The variable being assigned
	pub variable: String,
	/// The additional assignment item errors
	#[related]
	pub others: Vec<DuplicateAssignment>,
}

/// Indicates an extraneous solve item
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Multiple assignments to the same variable not allowed")]
#[diagnostic(
	code(shackle::multiple_assignments),
	help("Try removing this assignment.")
)]
pub struct DuplicateAssignment {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label]
	pub span: SourceSpan,
}

/// Cyclic variable definition
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Cyclic definition of {variable}")]
#[diagnostic(code(shackle::cyclic_definition))]
pub struct CyclicDefinition {
	/// The cyclic variable definition
	pub variable: String,
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("Cyclic definition not allowed.")]
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

/// A mismatch in branch/arm types
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Type mismatch")]
#[diagnostic(code(shackle::type_mismatch))]
pub struct BranchMismatch {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
	/// The expected branch type
	#[label("Expected because of this")]
	pub original_span: SourceSpan,
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
#[error("Return type conflicts with return type of other overloads")]
#[diagnostic(code(shackle::illegal_overload))]
pub struct IllegalOverloading {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("The function was first defined here")]
	pub span: SourceSpan,
	/// The related errors
	#[related]
	pub others: Vec<IllegalOverload>,
}

/// Function with same signature already defined
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Return type conflicts with another overload")]
#[diagnostic(code(shackle::illegal_overload))]
pub struct IllegalOverload {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label]
	pub span: SourceSpan,
}

/// Function with same signature already defined
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Function with the signature '{signature}' already defined")]
#[diagnostic(code(shackle::function_already_defined))]
pub struct FunctionAlreadyDefined {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The signature
	pub signature: String,
	/// The span associated with the error
	#[label("The function was first defined here")]
	pub span: SourceSpan,
	/// The duplicate functions
	#[related]
	pub others: Vec<DuplicateFunction>,
}

/// Function with same signature already defined
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Function already defined")]
#[diagnostic(
	code(shackle::function_already_defined),
	help("Try removing this function.")
)]
pub struct DuplicateFunction {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label]
	pub span: SourceSpan,
}

/// Constructor already defined
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Constructor function already defined")]
#[diagnostic(code(shackle::constructor_already_defined))]
pub struct ConstructorAlreadyDefined {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("The constructor function was first defined here")]
	pub span: SourceSpan,
	/// The duplicate constructors
	#[related]
	pub others: Vec<DuplicateConstructor>,
}

/// Constructor already defined
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Constructor function already defined")]
#[diagnostic(code(shackle::constructor_already_defined), help("{help}"))]
pub struct DuplicateConstructor {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The help string
	pub help: String,
	/// The span associated with the error
	#[label]
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

/// Non-exhaustive case expression pattern matching
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Non-exhaustive pattern matching")]
#[diagnostic(code(shackle::non_exhaustive_pattern_matching))]
pub struct NonExhaustivePatternMatching {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Invalid numeric literal
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Invalid numeric literal")]
#[diagnostic(code(shackle::invalid_numeric_literal))]
pub struct InvalidNumericLiteral {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The error message
	pub msg: String,
	/// The span associated with the error
	#[label("{msg}")]
	pub span: SourceSpan,
}

/// Reached function instantiation recursion limit
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Function instantiation error")]
#[diagnostic(code(shackle::instantiation_recursion_limit))]
pub struct TypeSpecialisationRecursionLimit {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The function being instantiated
	pub name: String,
	/// The span associated with the error
	#[label("Reached recursion limit while instantiating {name}")]
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
	#[error("Failed to locate the standard library.")]
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
	/// Multiple assignments to same variable
	#[error(transparent)]
	#[diagnostic(transparent)]
	MultipleAssignments(#[from] MultipleAssignments),
	/// Cyclic definition of variable
	#[error(transparent)]
	#[diagnostic(transparent)]
	CyclicDefinition(#[from] CyclicDefinition),
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
	/// Branch mismatch
	#[error(transparent)]
	#[diagnostic(transparent)]
	BranchMismatch(#[from] BranchMismatch),
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
	IllegalOverloading(#[from] IllegalOverloading),
	/// Function already defined
	#[error(transparent)]
	#[diagnostic(transparent)]
	FunctionAlreadyDefined(#[from] FunctionAlreadyDefined),
	/// Constructor already defined
	#[error(transparent)]
	#[diagnostic(transparent)]
	ConstructorAlreadyDefined(#[from] ConstructorAlreadyDefined),
	/// Type inference failure
	#[error(transparent)]
	#[diagnostic(transparent)]
	TypeInferenceFailure(#[from] TypeInferenceFailure),
	/// Invalid field access
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidFieldAccess(#[from] InvalidFieldAccess),
	/// Non-exhaustive pattern matching
	#[error(transparent)]
	#[diagnostic(transparent)]
	NonExhaustivePatternMatching(#[from] NonExhaustivePatternMatching),
	/// Invalid numeric literal
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidNumericLiteral(#[from] InvalidNumericLiteral),
	/// Reached function instantiation recursion limit
	#[error(transparent)]
	#[diagnostic(transparent)]
	TypeSpecialisationRecursionLimit(#[from] TypeSpecialisationRecursionLimit),
	/// An internal error
	#[error("Internal Error - Please report this issue to the Shackle developers")]
	InternalError(#[from] InternalError),
}

/// Unable to convert an empty vector into a ShackleError
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Unable to convert empty vector to a ShackleError")]
#[diagnostic(code(shackle::empty_error_vec))]
pub struct EmptyErrorVec;

impl TryFrom<Vec<ShackleError>> for ShackleError {
	type Error = EmptyErrorVec;

	fn try_from(value: Vec<ShackleError>) -> Result<Self, Self::Error> {
		match value.len() {
			0 => Err(EmptyErrorVec),
			1 => Ok(value.last().unwrap().clone()),
			_ => Ok(MultipleErrors { errors: value }.into()),
		}
	}
}

impl TryFrom<Diagnostics<ShackleError>> for ShackleError {
	type Error = EmptyErrorVec;

	fn try_from(value: Diagnostics<ShackleError>) -> Result<Self, Self::Error> {
		match value.len() {
			0 => Err(EmptyErrorVec),
			1 => Ok(value.iter().next().unwrap().clone()),
			_ => Ok(MultipleErrors {
				errors: value.iter().cloned().collect(),
			}
			.into()),
		}
	}
}

impl ShackleError {
	pub(crate) fn from_serde_json(err: serde_json::Error, src: &SourceFile) -> Self {
		use serde_json::error::Category;

		match err.classify() {
			Category::Io => FileError {
				file: src
					.path()
					.expect("I/O error can only occur when source is a file")
					.to_owned(),
				message: err.to_string(),
				other: Vec::new(),
			}
			.into(),
			Category::Syntax | Category::Eof => SyntaxError {
				src: src.clone(),
				span: SourceOffset::from_location(src.contents(), err.line(), err.column()).into(),
				msg: err.to_string(),
				other: Vec::new(),
			}
			.into(),
			Category::Data => TypeMismatch {
				src: src.clone(),
				msg: err.to_string(),
				span: SourceOffset::from_location(src.contents(), err.line(), err.column()).into(),
			}
			.into(),
		}
	}
}
