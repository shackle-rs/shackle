//! Functionality related to the input and output of data

pub(crate) mod dzn;
pub(crate) mod json;
pub(crate) mod serde;

use std::ops::RangeInclusive;

use crate::{
	diagnostics::ShackleError,
	value::{Polarity, Value},
	Program, Type,
};

/// Value parsed in a data file.
///
/// These values can still contain unmatched enum values or enum constructors,
/// for which the internal value has not yet been determined.
///
/// TODO: Can we avoid copying the actual strings and use Cow/&str
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParserVal {
	/// Absence of an optional value
	Absent,
	/// Infinity (+∞ or -∞)
	Infinity(Polarity),
	/// Boolean
	Boolean(bool),
	/// Signed integer
	Integer(i64),
	/// Floating point
	Float(f64),
	/// String
	String(String),
	/// Identifier of a value of an enumerated type
	Enum(String, Vec<ParserVal>),
	/// Annotation
	Ann(String, Vec<ParserVal>),
	/// An array of values
	SimpleArray(Vec<(ParserVal, ParserVal)>, Vec<ParserVal>),
	IndexedArray(u64, Vec<ParserVal>),
	/// A set of values
	SetList(Vec<ParserVal>),
	Range(Box<ParserVal>, Box<ParserVal>),
	/// A tuple of values
	Tuple(Vec<ParserVal>),
	/// A record of values
	Record(Vec<(String, ParserVal)>),
	/// Constructor used to define an enumerated type, or create a value of an enumerated type.
	EnumCtor(EnumCtor),
}

/// Constructor for an enumerated type
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumCtor {
	/// List of identifiers describing an enumerated type
	ValueList(Vec<String>),
	/// Constructor call with a set as an argument
	SetArg((String, RangeInclusive<i64>)),
	/// The concatenation of multiple other types of constructors
	Concat(Vec<EnumCtor>),
}

impl Program {
	/// Resolve parsed data value into final value for users and the interpreter
	///
	/// This is the final step in the parsing of data files, resolving enumerated types and creating
	pub(crate) fn resolve_value(&self, ty: &Type, v: ParserVal) -> Result<Value, ShackleError> {
		Ok(Value::Absent)
	}
}
