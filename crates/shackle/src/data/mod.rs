//! Functionality related to the input and output of data

pub(crate) mod dzn;
pub(crate) mod json;
pub(crate) mod serde;

use std::{ops::RangeInclusive, sync::Arc};

use itertools::Itertools;

use crate::{
	diagnostics::ShackleError,
	value::{Array, EnumRangeInclusive, Index, Polarity, Value},
	Type,
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
	Record(Vec<(Arc<str>, ParserVal)>),
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

impl ParserVal {
	/// Resolve parsed data value into final value for users and the interpreter
	///
	/// This is the final step in the parsing of data files, resolving enumerated types and creating
	pub(crate) fn resolve_value(self, ty: &Type) -> Result<Value, ShackleError> {
		match self {
			ParserVal::Absent => Ok(Value::Absent),
			ParserVal::Infinity(v) => Ok(Value::Infinity(v)),
			ParserVal::Boolean(v) => Ok(v.into()),
			ParserVal::Integer(v) => Ok(v.into()),
			ParserVal::Float(v) => Ok(v.into()),
			ParserVal::String(v) => Ok(Value::String(v.into())),
			ParserVal::Enum(_, _) => todo!(),
			ParserVal::Ann(_, _) => todo!(),
			ParserVal::SimpleArray(ranges, elements) => {
				let Type::Array { opt: _, dim, element } = ty else { unreachable!() };
				let indices = ranges
					.into_iter()
					.zip_eq(dim.iter())
					.map(|(range, ty)| match range {
						(ParserVal::Integer(from), ParserVal::Integer(to)) => {
							Ok::<_, ShackleError>(Index::Integer(from..=to))
						}
						(from @ ParserVal::Enum(_, _), to @ ParserVal::Enum(_, _)) => {
							let Value::Enum(_) = from.resolve_value(ty)? else {unreachable!()};
							let Value::Enum(_) = to.resolve_value(ty)? else {unreachable!()};
							todo!()
						}
						_ => unreachable!("invalid index range parsed"),
					})
					.collect::<Result<Vec<_>, _>>()?;
				let elements = elements
					.into_iter()
					.map(|el| el.resolve_value(element))
					.collect::<Result<Vec<_>, _>>()?;
				Ok(Array::new(indices, elements).into())
			}
			ParserVal::IndexedArray(_, _) => todo!(),
			ParserVal::SetList(_) => todo!(),
			ParserVal::Range(a, b) => Ok(Value::Set(match (*a, *b) {
				(ParserVal::Integer(from), ParserVal::Integer(to)) => (from..=to).into(),
				(from @ ParserVal::Enum(_, _), to @ ParserVal::Enum(_, _)) => {
					let Value::Enum(a) = from.resolve_value(ty)? else {unreachable!()};
					let Value::Enum(b) = to.resolve_value(ty)? else {unreachable!()};
					EnumRangeInclusive::new(a, b).into()
				}
				_ => unreachable!("invalid ParserVal::Range arguments"),
			})),
			ParserVal::Tuple(v) => {
				let Type::Tuple(_, ty) = ty else {unreachable!()};
				let members = v
					.into_iter()
					.zip_eq(ty.iter())
					.map(|(m, ty)| m.resolve_value(ty))
					.collect::<Result<Vec<_>, _>>()?;
				Ok(Value::Tuple(members))
			}
			ParserVal::Record(_) => todo!(),
			ParserVal::EnumCtor(_) => unreachable!("not a value"),
		}
	}
}
