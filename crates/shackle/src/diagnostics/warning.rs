//! Warning handling

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::file::SourceFile;

/// Identifier shadows another
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Variable shadows identifier")]
#[diagnostic(code(shackle::shadowed_variable), severity(Warning))]
pub struct IdentifierShadowing {
	/// The name of the variable
	pub name: String,
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span of the new variable declaration
	#[label("Variable {name} shadows identifier with same name")]
	pub span: SourceSpan,
	/// The span of the original variable declaration
	#[label("This identifier is shadowed")]
	pub original: SourceSpan,
}

/// Unreachable case expression arm
#[derive(Error, Debug, Diagnostic, PartialEq, Eq, Clone)]
#[error("Unreachable pattern")]
#[diagnostic(code(shackle::unreachable_pattern), severity(Warning))]
pub struct UnreachablePattern {
	/// The source code
	#[source_code]
	pub src: SourceFile,
	/// The span associated with the error
	#[label("Pattern is unreachable")]
	pub span: SourceSpan,
}

/// Shackle warning type
#[derive(Error, Diagnostic, Debug, PartialEq, Eq, Clone)]
pub enum Warning {
	/// Identifier shadows another
	#[error(transparent)]
	#[diagnostic(transparent)]
	IdentifierShadowing(#[from] IdentifierShadowing),
	/// Unreachable case expression arm
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnreachablePattern(#[from] UnreachablePattern),
}
