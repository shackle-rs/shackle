//! Functionality related to the input and output of data

pub(crate) mod dzn;
pub(crate) mod serde;

use std::sync::Arc;

use itertools::Itertools;

use crate::{
	value::{Array, EnumRangeInclusive, EnumValue, Index, Polarity, Record, Set, Value},
	Error, OptType, Result, Type,
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
	IndexedArray(usize, Vec<ParserVal>),
	/// A set of values
	SetList(Vec<ParserVal>),
	SetRangeList(Vec<(ParserVal, ParserVal)>),
	Range(Box<(ParserVal, ParserVal)>),
	/// A tuple of values
	Tuple(Vec<ParserVal>),
	/// A record of values
	Record(Vec<(Arc<str>, ParserVal)>),
}

impl ParserVal {
	/// Resolve parsed data value into final value for users and the interpreter
	///
	/// This is the final step in the parsing of data files, resolving enumerated types and creating
	pub(crate) fn resolve_value(self, ty: &Type) -> Result<Value> {
		match self {
			ParserVal::Absent => Ok(Value::Absent),
			ParserVal::Infinity(v) => Ok(Value::Infinity(v)),
			ParserVal::Boolean(v) => Ok(Value::Boolean(v)),
			ParserVal::Integer(v) => Ok(Value::Integer(v)),
			ParserVal::Float(v) => Ok(Value::Float(v)),
			ParserVal::String(v) => Ok(Value::String(v.into())),
			ParserVal::Enum(name, args) => {
				let Type::Enum(_, e) = ty else { unreachable!() };
				let Some((offset, doms)) = e.get(&name) else {
					todo!("add location to throw error for undefined constructor")
				};
				if args.len() != doms.len() {
					todo!("add error for non-matching constructor call");
				}
				let mut offset = offset;
				for (arg, dom) in args.into_iter().zip_eq(doms.iter()) {
					match dom {
						Index::Integer(r) => {
							let Value::Integer(arg) =
								arg.resolve_value(&Type::Integer(OptType::NonOpt))?
							else {
								unreachable!()
							};
							if !r.contains(&arg) {
								todo!("invalid argument - out of domain")
							}
							offset += (arg - r.start()) as usize;
						}
						Index::Enum(r) => {
							let Value::Enum(arg) =
								arg.resolve_value(&Type::Enum(OptType::NonOpt, r.enum_type()))?
							else {
								unreachable!()
							};
							if !r.contains(&arg) {
								todo!("invalid argument - out of domain")
							}
							offset += r.start().int_val() - arg.int_val();
						}
					}
				}
				Ok(Value::Enum(EnumValue::from_enum_and_pos(e.clone(), offset)))
			}
			ParserVal::Ann(_, _) => todo!(),
			ParserVal::SimpleArray(ranges, elements) => {
				let Type::Array {
					opt: _,
					dim,
					element,
				} = ty
				else {
					unreachable!()
				};
				let elements = elements
					.into_iter()
					.map(|el| el.resolve_value(element))
					.collect::<Result<Vec<_>, _>>()?;
				if elements.is_empty() {
					return Ok(Array::empty().into());
				}
				let indices = ranges
					.into_iter()
					.zip_eq(dim.iter())
					.map(|(range, ty)| match range {
						(ParserVal::Integer(start), ParserVal::Integer(end)) => {
							Ok::<_, Error>(Index::Integer(start..=end))
						}
						(start @ ParserVal::Enum(_, _), ParserVal::Infinity(Polarity::Pos)) => {
							debug_assert_eq!(dim.len(), 1);
							let Value::Enum(start) = start.resolve_value(ty)? else {
								unreachable!()
							};
							if start.int_val() + elements.len() > start.enum_type().len() {
								todo!()
							// Err(InvalidArrayLiteral {
							// 	msg: format!("Array literal cannot start at value {start}. There are only {} higher values in its enumerated type, but the array literal has {} members", start.enum_type().len() + 1 - start.int_val(), elements.len()),
							// 	src: todo!(),
							// 	span: todo!(),
							// }
							// .into())
							} else {
								Ok(Index::Enum(EnumRangeInclusive::from_enum_and_positions(
									start.enum_type(),
									start.int_val(),
									start.int_val() + elements.len(),
								)))
							}
						}
						(start @ ParserVal::Enum(_, _), end @ ParserVal::Enum(_, _)) => {
							let Value::Enum(start) = start.resolve_value(ty)? else {
								unreachable!()
							};
							let Value::Enum(end) = end.resolve_value(ty)? else {
								unreachable!()
							};
							Ok(Index::Enum((start, end).into()))
						}
						_ => unreachable!("invalid index range parsed"),
					})
					.collect::<Result<Vec<_>, _>>()?;
				Ok(Array::new(indices, elements).into())
			}
			ParserVal::IndexedArray(_, _) => todo!(),
			ParserVal::SetList(li) => {
				let Type::Set(_, ty) = ty else { unreachable!() };
				let members = li
					.into_iter()
					.map(|m| m.resolve_value(ty))
					.collect::<Result<Vec<_>, _>>()?;
				// TODO: This could likely be optimised to not create ranges first
				match **ty {
					Type::Integer(_) => Ok(Value::Set(
						members
							.into_iter()
							.map(|m| {
								let Value::Integer(i) = m else { unreachable!() };
								i..=i
							})
							.collect(),
					)),
					Type::Float(_) => Ok(Value::Set(
						members
							.into_iter()
							.map(|m| {
								let Value::Float(i) = m else { unreachable!() };
								i..=i
							})
							.collect(),
					)),
					Type::Enum(_, _) => Ok(Value::Set(
						members
							.into_iter()
							.map(|m| {
								let Value::Enum(i) = m else { unreachable!() };
								EnumRangeInclusive::new(i.clone(), i)
							})
							.collect(),
					)),
					_ => unreachable!("invalid set type"),
				}
			}
			ParserVal::SetRangeList(li) => {
				let Type::Set(_, ty) = ty else { unreachable!() };
				Ok(match &**ty {
					Type::Integer(OptType::NonOpt) => Set::from_iter(li.into_iter().map(|r| {
						let (ParserVal::Integer(a), ParserVal::Integer(b)) = r else {
							unreachable!("invalid integer set")
						};
						a..=b
					}))
					.into(),
					Type::Float(OptType::NonOpt) => Set::from_iter(li.into_iter().map(|r| {
						let (ParserVal::Float(a), ParserVal::Float(b)) = r else {
							unreachable!("invalid integer set")
						};
						a..=b
					}))
					.into(),
					e @ Type::Enum(OptType::NonOpt, _) => Set::from_iter(
						li.into_iter()
							.map(|(a, b)| match a.resolve_value(e) {
								Ok(a) => match b.resolve_value(e) {
									Ok(b) => {
										let (Value::Enum(a), Value::Enum(b)) = (a, b) else {
											unreachable!("invalid enum set")
										};
										Ok(EnumRangeInclusive::new(a, b))
									}
									Err(e) => Err(e),
								},
								Err(e) => Err(e),
							})
							.collect::<Result<Vec<EnumRangeInclusive>, _>>()?,
					)
					.into(),
					_ => unreachable!("invalid set type"),
				})
			}
			ParserVal::Range(range) => Ok(Value::Set(match *range {
				(ParserVal::Float(start), ParserVal::Float(end)) => (start..=end).into(),
				(ParserVal::Integer(start), ParserVal::Integer(end)) => (start..=end).into(),
				(from @ ParserVal::Enum(_, _), to @ ParserVal::Enum(_, _)) => {
					let Value::Enum(a) = from.resolve_value(ty)? else {
						unreachable!()
					};
					let Value::Enum(b) = to.resolve_value(ty)? else {
						unreachable!()
					};
					EnumRangeInclusive::new(a, b).into()
				}
				_ => unreachable!("invalid ParserVal::Range arguments"),
			})),
			ParserVal::Tuple(v) => {
				let Type::Tuple(_, ty) = ty else {
					unreachable!()
				};
				let members = v
					.into_iter()
					.zip_eq(ty.iter())
					.map(|(m, ty)| m.resolve_value(ty))
					.collect::<Result<Vec<_>, _>>()?;
				Ok(Value::Tuple(members))
			}
			ParserVal::Record(v) => {
				let Type::Record(_, ty) = ty else {
					unreachable!()
				};
				let rec = v
					.into_iter()
					.zip_eq(ty.iter())
					.map(|((n, v), (name, ty))| {
						debug_assert_eq!(&n, name);
						Ok((name.clone(), v.resolve_value(ty)?))
					})
					.collect::<Result<Record>>()?;
				Ok(Value::Record(rec))
			}
		}
	}
}
