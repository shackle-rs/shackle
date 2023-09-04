use std::sync::Arc;

use itertools::Itertools;
use rustc_hash::FxHashMap;
use serde::{
	de::{DeserializeSeed, Error, IgnoredAny, Unexpected, Visitor},
	ser::{SerializeMap, SerializeSeq},
	Deserialize, Serialize,
};

use super::ParserVal;
use crate::{
	value::{Array, Constructor, EnumInner, EnumValue, Index, Record, Set},
	Enum, OptType, Type, Value,
};

#[derive(Clone)]
pub(crate) struct SerdeValueVisitor<'a>(pub &'a Type);

impl<'de, 'a> Visitor<'de> for SerdeValueVisitor<'a> {
	type Value = ParserVal;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "a value of type \"{}\"", self.0)
	}

	fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
		let ty = self.0;
		match ty {
			Type::Boolean(_) => Ok(ParserVal::Boolean(v)),
			Type::Integer(_) => Ok(ParserVal::Integer(v as i64)),
			Type::Float(_) => Ok(ParserVal::Float(v as i64 as f64)),
			_ => Err(Error::invalid_type(Unexpected::Bool(v), &self)),
		}
	}

	fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
		let ty = self.0;
		match ty {
			Type::Integer(_) => Ok(ParserVal::Integer(v)),
			Type::Float(_) => Ok(ParserVal::Float(v as f64)),
			_ => Err(Error::invalid_type(Unexpected::Signed(v), &self)),
		}
	}

	fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
		match v.try_into() {
			Ok(x) => self.visit_i64(x),
			Err(e) => Err(Error::custom(e.to_string())),
		}
	}

	fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
		let ty = self.0;
		if matches!(ty, Type::Float(_)) {
			Ok(ParserVal::Float(v))
		} else {
			Err(Error::invalid_type(Unexpected::Float(v), &self))
		}
	}

	fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
		self.visit_string(v.to_string())
	}
	fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
		self.visit_str(v) // TODO: avoid copying when possible
	}

	fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
		let ty = self.0;
		match ty {
			Type::Enum(_, _) => Ok(ParserVal::Enum(v, Vec::new())),
			Type::String(_) => Ok(ParserVal::String(v)),
			Type::Annotation(_) => Ok(ParserVal::Ann(v, Vec::new())),
			_ => Err(Error::invalid_type(Unexpected::Str(v.as_str()), &self)),
		}
	}

	fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
		let ty = self.0;
		if ty.is_opt() {
			Ok(ParserVal::Absent)
		} else {
			Err(Error::invalid_type(Unexpected::Unit, &self))
		}
	}
	fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
		let ty = self.0;
		if ty.is_opt() {
			Ok(ParserVal::Absent)
		} else {
			Err(Error::invalid_type(Unexpected::Option, &self))
		}
	}
	fn visit_some<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		let ty = self.0;
		if ty.is_opt() {
			deserializer.deserialize_any(self)
		} else {
			Err(Error::invalid_type(Unexpected::Option, &self))
		}
	}

	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
		let ty = self.0;
		match ty {
			Type::Array {
				opt: _,
				dim,
				element,
			} => {
				let mut sizes = Vec::with_capacity(dim.len());
				let mut data = Vec::new();
				let visitor = SerdeArrayVisitor {
					data: &mut data,
					size: &mut sizes,
					element,
					dim: dim.len() as u8,
					depth: 1,
				};
				visitor.visit_seq(seq)?;
				if data.is_empty() {
					return Ok(ParserVal::SimpleArray(Vec::new(), Vec::new()));
				}
				debug_assert_eq!(dim.len(), sizes.len());
				let mut indices = Vec::with_capacity(sizes.capacity());
				for (ty, len) in dim.iter().zip_eq(sizes.into_iter()) {
					match ty {
						Type::Integer(OptType::NonOpt) => {
							indices.push((ParserVal::Integer(1), ParserVal::Integer(len)))
						}
						Type::Enum(_, _) => todo!(),
						_ => unreachable!("invalid index type"),
					}
				}
				Ok(ParserVal::SimpleArray(indices, data))
			}
			Type::Tuple(_, members) => {
				let mut tup = Vec::with_capacity(members.len());
				for ty in members.iter() {
					let Some(m) = seq.next_element_seed(SerdeValueVisitor(ty))? else {
						return Err(Error::invalid_length(
							tup.len(),
							&members.len().to_string().as_str(),
						));
					};
					tup.push(m);
				}
				if seq.next_element::<IgnoredAny>()?.is_some() {
					return Err(Error::invalid_length(
						tup.len(),
						&members.len().to_string().as_str(),
					));
				}
				Ok(ParserVal::Tuple(tup))
			}
			_ => Err(Error::invalid_type(Unexpected::Seq, &self)),
		}
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let ty = self.0;
		match ty {
			Type::Record(_, ty) => {
				let mut rec = Vec::with_capacity(ty.len());
				let mut types = FxHashMap::from_iter(ty.iter().map(|(a, b)| (a.clone(), b)));
				while let Some(k) = map.next_key::<&str>()? {
					if let Some((k, ty)) = types.remove_entry(k) {
						let val = map.next_value_seed(SerdeValueVisitor(ty))?;
						rec.push((k, val))
					} else {
						map.next_value::<IgnoredAny>()?; // Ignore unknown
					}
				}
				if !types.is_empty() {
					let field = types.into_iter().next().unwrap().0.to_string();
					return Err(Error::missing_field(Box::leak(field.into_boxed_str()))); // TODO: Can we avoid leaking memory here?
				}
				rec.sort_by(|x, y| x.0.cmp(&y.0));
				Ok(ParserVal::Record(rec))
			}
			Type::Set(_, ty) => {
				// Map is expected to have a single key "set" that is assigned an sequence of elements, these elements may be pairs to suggest list of elements.

				// NOTE: The SerdeMaybePairVisitor will not work for types represented usin sequences
				debug_assert!(!matches!(
					**ty,
					Type::Tuple(_, _)
						| Type::Array {
							opt: _,
							dim: _,
							element: _
						}
				));

				let li = match map.next_key::<&str>()? {
					Some("set") => {
						let seed = SerdeSeqVisitor(SerdeMaybePairVisitor(SerdeValueVisitor(ty)));
						map.next_value_seed(seed)?
					}
					Some(k) => return Err(Error::unknown_field(k, &["set"])),
					None => return Err(Error::missing_field("set")),
				};
				if let Some(k) = map.next_key::<&str>()? {
					return Err(Error::unknown_field(k, &["set"]));
				}

				Ok(ParserVal::SetRangeList(
					li.into_iter()
						.map(|(a, b)| {
							let b = b.unwrap_or_else(|| a.clone());
							(a, b)
						})
						.collect(),
				))
			}
			Type::Enum(_, _) => SerdeEnumVisitor.visit_map(map),
			_ => Err(Error::invalid_type(Unexpected::Map, &self)),
		}
	}
}

impl<'a, 'de> DeserializeSeed<'de> for SerdeValueVisitor<'a> {
	type Value = ParserVal;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_any(self)
	}
}

#[derive(Clone)]
struct SerdeEnumVisitor;

impl<'de> DeserializeSeed<'de> for SerdeEnumVisitor {
	type Value = ParserVal;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_any(self)
	}
}

impl<'de> Visitor<'de> for SerdeEnumVisitor {
	type Value = ParserVal;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "an enumerated type argument")
	}

	fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
		Ok(ParserVal::Integer(v))
	}

	fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
		Ok(ParserVal::Enum(v, Vec::new()))
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		const FIELDS: &[&str] = &["e", "a"];
		let mut e = None;
		let mut a = None;

		while let Some(k) = map.next_key::<&str>()? {
			match k {
				"e" => {
					if e.is_some() {
						return Err(Error::duplicate_field("e"));
					}
					e = Some(map.next_value()?);
				}
				"a" => {
					if a.is_some() {
						return Err(Error::duplicate_field("a"));
					}
					a = Some(map.next_value_seed(SerdeSeqVisitor(self.clone()))?);
				}
				_ => return Err(Error::unknown_field(k, FIELDS)),
			}
		}

		match (e, a) {
			(None, _) => Err(Error::missing_field("e")),
			(Some(e), Some(a)) => Ok(ParserVal::Enum(e, a)),
			(Some(e), None) => Ok(ParserVal::Enum(e, Vec::new())),
		}
	}
}

struct SerdeArrayVisitor<'a> {
	data: &'a mut Vec<ParserVal>,
	size: &'a mut Vec<i64>,
	element: &'a Type,
	dim: u8,
	depth: u8,
}

impl<'a, 'de> DeserializeSeed<'de> for SerdeArrayVisitor<'a> {
	type Value = ();

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_seq(self)
	}
}

impl<'de, 'a> Visitor<'de> for SerdeArrayVisitor<'a> {
	type Value = ();

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "an array literal")
	}

	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
		let mut i = 0;
		debug_assert!(self.depth <= self.dim);
		if self.depth >= self.dim {
			// Parse elements
			while let Some(elt) = seq.next_element_seed(SerdeValueVisitor(self.element))? {
				self.data.push(elt);
				i += 1;
			}
		} else {
			// Recurse with one less dimension
			while seq
				.next_element_seed(SerdeArrayVisitor {
					data: self.data,
					size: self.size,
					element: self.element,
					dim: self.dim,
					depth: self.depth + 1,
				})?
				.is_some()
			{
				// values already added to data
				i += 1
			}
		}
		if self.size.len() < self.depth as usize {
			debug_assert_eq!(self.size.len() + 1, self.depth as usize);
			self.size.push(i);
		} else if self.size[self.depth as usize - 1] != i {
			return Err(Error::invalid_length(
				i as usize,
				&self.size[self.depth as usize - 1].to_string().as_str(),
			));
		}
		Ok(())
	}
}

impl<'de> Deserialize<'de> for EnumInner {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		// Internal Visitor that deserialises a single constructor object
		struct EnumCtor;
		impl<'de> Visitor<'de> for EnumCtor {
			type Value = Constructor;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "enumerated type constructor")
			}

			fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
				Ok((v.into(), Vec::new().into_boxed_slice(), 1))
			}

			fn visit_map<A: serde::de::MapAccess<'de>>(
				self,
				mut map: A,
			) -> Result<Self::Value, A::Error> {
				const FIELDS: &[&str] = &["k", "a"];
				let mut name: Option<&str> = None;
				let mut args = Vec::new();
				while let Some(k) = map.next_key()? {
					match k {
						"c" => name = Some(map.next_value()?),
						"a" => {
							let intset_list = Type::Array {
								opt: OptType::NonOpt,
								dim: Box::new([Type::Integer(OptType::NonOpt)]),
								element: Box::new(Type::Set(
									OptType::NonOpt,
									Box::new(Type::Integer(OptType::NonOpt)),
								)),
							};
							let val = map.next_value_seed(SerdeValueVisitor(&intset_list))?;
							let Value::Array(x) = val.resolve_value(&intset_list).unwrap() else {
								unreachable!()
							};
							args = x
								.members
								.iter()
								.map(|i| {
									let Value::Set(Set::IntRangeList(s)) = i else {
										unreachable!()
									};
									if s.len() != 1 {
										todo!("handle non-continuous (and empty) integer sets for constructors")
									}
									Index::Integer(s[0].clone())
								})
								.collect();
						}
						_ => return Err(Error::unknown_field(k, FIELDS)),
					}
				}
				let count = if args.is_empty() {
					1
				} else {
					args.iter().map(|x| x.len()).sum()
				};
				if let Some(name) = name {
					Ok((name.into(), args.into_boxed_slice(), count))
				} else {
					Err(Error::missing_field("c"))
				}
			}
		}
		impl<'de> DeserializeSeed<'de> for EnumCtor {
			type Value = <EnumCtor as Visitor<'de>>::Value;

			fn deserialize<D: serde::Deserializer<'de>>(
				self,
				deserializer: D,
			) -> Result<Self::Value, D::Error> {
				deserializer.deserialize_any(self)
			}
		}
		// Helper visitor that visits a list of enum constructors
		struct EnumDefVisitor;
		impl<'de> Visitor<'de> for EnumDefVisitor {
			type Value = Box<[Constructor]>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "list of constructors of an enumerated type")
			}

			fn visit_seq<A: serde::de::SeqAccess<'de>>(
				self,
				mut seq: A,
			) -> Result<Self::Value, A::Error> {
				let mut v = Vec::new();
				while let Some(el) = seq.next_element_seed(EnumCtor)? {
					v.push(el);
				}
				Ok(v.into_boxed_slice())
			}
		}

		let ctors = deserializer.deserialize_seq(EnumDefVisitor)?;
		Ok(EnumInner::Constructors(ctors))
	}
}

pub(crate) struct SerdeFileVisitor<'a> {
	pub(crate) input_types: &'a FxHashMap<Arc<str>, Type>,
	pub(crate) enum_types: &'a FxHashMap<Arc<str>, Arc<Enum>>,
}

impl<'de, 'a> Visitor<'de> for SerdeFileVisitor<'a> {
	type Value = Vec<(&'a Arc<str>, &'a Type, ParserVal)>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "assignment mapping")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let mut assignments = Vec::with_capacity(map.size_hint().unwrap_or(0));

		while let Some(k) = map.next_key::<&str>()? {
			if let Some((ident, ty)) = self.input_types.get_key_value(k) {
				let value = map.next_value_seed(SerdeValueVisitor(ty))?;
				assignments.push((ident, ty, value));
			} else if let Some(e) = self.enum_types.get(k) {
				let mut inner = e.state.lock().unwrap();
				if *inner == EnumInner::NoDefinition {
					*inner = map.next_value::<EnumInner>()?;
				} else {
					todo!("create custom serde error that will be converted into ShackleError")
				}
			} else {
				map.next_value::<IgnoredAny>()?; // Ignore unknown
			}
		}
		Ok(assignments)
	}
}

struct SerdeSeqVisitor<X: Clone>(X);

impl<'de, X: DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for SerdeSeqVisitor<X> {
	type Value = Vec<X::Value>;

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_seq(self)
	}
}

impl<'de, X: DeserializeSeed<'de> + Clone> Visitor<'de> for SerdeSeqVisitor<X> {
	type Value = Vec<X::Value>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "sequence")
	}

	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
		let mut v = Vec::with_capacity(seq.size_hint().unwrap_or(0));
		while let Some(x) = seq.next_element_seed(self.0.clone())? {
			v.push(x);
		}
		Ok(v)
	}
}

#[derive(Clone)]
struct SerdeMaybePairVisitor<X: Clone>(X);

impl<
		'de,
		X: DeserializeSeed<'de> + Visitor<'de, Value = <X as DeserializeSeed<'de>>::Value> + Clone,
	> DeserializeSeed<'de> for SerdeMaybePairVisitor<X>
{
	type Value = (
		<X as Visitor<'de>>::Value,
		Option<<X as Visitor<'de>>::Value>,
	);

	fn deserialize<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		deserializer.deserialize_any(self)
	}
}

impl<
		'de,
		X: DeserializeSeed<'de> + Visitor<'de, Value = <X as DeserializeSeed<'de>>::Value> + Clone,
	> Visitor<'de> for SerdeMaybePairVisitor<X>
{
	type Value = (
		<X as Visitor<'de>>::Value,
		Option<<X as Visitor<'de>>::Value>,
	);

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "maybe a pair")
	}

	// A sequence is expected to be a pair
	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
		let Some(first) = seq.next_element_seed(self.0.clone())? else {
			return Err(Error::invalid_length(0, &"2"));
		};
		let Some(second) = seq.next_element_seed(self.0.clone())? else {
			return Err(Error::invalid_length(1, &"2"));
		};

		let mut i = 0;
		while let Some(_) = seq.next_element::<IgnoredAny>()? {
			i += 1;
		}
		if i > 0 {
			return Err(Error::invalid_length(2 + i, &"2"));
		}
		Ok((first, Some(second)))
	}

	// Defer to inner visitor
	fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
		Ok((self.0.visit_bool(v)?, None))
	}
	fn visit_i8<E: Error>(self, v: i8) -> Result<Self::Value, E> {
		Ok((self.0.visit_i8(v)?, None))
	}
	fn visit_i16<E: Error>(self, v: i16) -> Result<Self::Value, E> {
		Ok((self.0.visit_i16(v)?, None))
	}
	fn visit_i32<E: Error>(self, v: i32) -> Result<Self::Value, E> {
		Ok((self.0.visit_i32(v)?, None))
	}
	fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
		Ok((self.0.visit_i64(v)?, None))
	}
	fn visit_i128<E: Error>(self, v: i128) -> Result<Self::Value, E> {
		Ok((self.0.visit_i128(v)?, None))
	}
	fn visit_u8<E: Error>(self, v: u8) -> Result<Self::Value, E> {
		Ok((self.0.visit_u8(v)?, None))
	}
	fn visit_u16<E: Error>(self, v: u16) -> Result<Self::Value, E> {
		Ok((self.0.visit_u16(v)?, None))
	}
	fn visit_u32<E: Error>(self, v: u32) -> Result<Self::Value, E> {
		Ok((self.0.visit_u32(v)?, None))
	}
	fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
		Ok((self.0.visit_u64(v)?, None))
	}
	fn visit_u128<E: Error>(self, v: u128) -> Result<Self::Value, E> {
		Ok((self.0.visit_u128(v)?, None))
	}
	fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
		Ok((self.0.visit_f32(v)?, None))
	}
	fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
		Ok((self.0.visit_f64(v)?, None))
	}
	fn visit_char<E: Error>(self, v: char) -> Result<Self::Value, E> {
		Ok((self.0.visit_char(v)?, None))
	}
	fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
		Ok((self.0.visit_str(v)?, None))
	}
	fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
		Ok((self.0.visit_borrowed_str(v)?, None))
	}
	fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
		Ok((self.0.visit_string(v)?, None))
	}
	fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<Self::Value, E> {
		Ok((self.0.visit_bytes(v)?, None))
	}
	fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
		Ok((self.0.visit_borrowed_bytes(v)?, None))
	}
	fn visit_byte_buf<E: Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
		Ok((self.0.visit_byte_buf(v)?, None))
	}
	fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
		Ok((self.0.visit_none()?, None))
	}
	fn visit_some<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		Ok((self.0.visit_some(deserializer)?, None))
	}
	fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
		Ok((self.0.visit_unit()?, None))
	}
	fn visit_newtype_struct<D: serde::Deserializer<'de>>(
		self,
		deserializer: D,
	) -> Result<Self::Value, D::Error> {
		Ok((self.0.visit_newtype_struct(deserializer)?, None))
	}
	fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
		Ok((self.0.visit_map(map)?, None))
	}
	fn visit_enum<A: serde::de::EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
		Ok((self.0.visit_enum(data)?, None))
	}
}

impl Serialize for Value {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		match self {
			Value::Absent => serializer.serialize_none(),
			Value::Infinity(_) => todo!(),
			Value::Boolean(v) => serializer.serialize_bool(*v),
			Value::Integer(v) => serializer.serialize_i64(*v),
			Value::Float(v) => serializer.serialize_f64(*v),
			Value::String(v) => serializer.serialize_str(&*v),
			Value::Enum(v) => v.serialize(serializer),
			Value::Ann(_, _) => todo!(),
			Value::Array(v) => v.serialize(serializer),
			Value::Set(v) => v.serialize(serializer),
			Value::Tuple(v) => {
				let mut seq = serializer.serialize_seq(Some(v.len()))?;
				for i in v {
					seq.serialize_element(i)?;
				}
				seq.end()
			}
			Value::Record(v) => v.serialize(serializer),
		}
	}
}
impl Serialize for Array {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		if self.is_empty() {
			serializer.serialize_seq(Some(0))?.end()
		} else {
			ArraySliceSerializer::new(&self.indices[..], &self.members[..]).serialize(serializer)
		}
	}
}
impl Serialize for Record {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(Some(self.len()))?;
		for (k, v) in self.iter() {
			map.serialize_entry(&*k, v)?;
		}
		map.end()
	}
}
impl Serialize for Set {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(Some(1))?;
		// Note: constructing the vector might not be very efficient. Might be worthwhile implementing Serialize on a "SetInner"/"RangeList" type
		match self {
			Set::EnumRangeList(v) => {
				map.serialize_entry("set", &v.iter().map(|v| (v.start(), v.end())).collect_vec())?
			}
			Set::FloatRangeList(v) => {
				map.serialize_entry("set", &v.iter().map(|v| (v.start(), v.end())).collect_vec())?
			}
			Set::IntRangeList(v) => {
				map.serialize_entry("set", &v.iter().map(|v| (v.start(), v.end())).collect_vec())?
			}
		}
		map.end()
	}
}
impl Serialize for EnumValue {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(Some(if self.args().is_empty() { 1 } else { 2 }))?;
		map.serialize_entry("e", &*self.constructor())?;
		if !self.args().is_empty() {
			map.serialize_entry("a", &self.args())?
		}
		map.end()
	}
}

struct ArraySliceSerializer<'a> {
	indices: &'a [Index],
	members: &'a [Value],
	step: usize,
}
impl<'a> ArraySliceSerializer<'a> {
	fn new(indices: &'a [Index], members: &'a [Value]) -> Self {
		Self {
			indices,
			members,
			step: indices.iter().skip(1).fold(1, |cur, idx| cur * idx.len()),
		}
	}
}
impl<'a> Serialize for ArraySliceSerializer<'a> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		debug_assert!(self.indices.len() > 0);
		let idx = &self.indices[0];
		debug_assert_eq!(self.step * idx.len(), self.members.len());
		let mut seq = serializer.serialize_seq(Some(idx.len()))?;
		if self.indices.len() <= 1 {
			debug_assert_eq!(self.step, 1);
			debug_assert_eq!(self.members.len(), idx.len());
			for v in self.members {
				seq.serialize_element(v)?;
			}
		} else {
			debug_assert_eq!(self.step % self.indices[1].len(), 0);
			let step = self.step / self.indices[1].len();
			let mut v = Self {
				indices: &self.indices[1..],
				members: self.members,
				step,
			};
			for i in 0..idx.len() {
				v.members = &self.members[i * self.step..(i + 1) * self.step];
				seq.serialize_element(&v)?;
			}
		}
		seq.end()
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use expect_test::{expect, Expect};
	use rustc_hash::FxHashMap;
	use serde::Deserializer;

	use super::SerdeFileVisitor;
	use crate::{diagnostics::ShackleError, file::SourceFile, OptType, Type};

	fn check_serialization(input: &str, ty: &Type, expected: Expect) {
		let input_types = FxHashMap::from_iter([("x".into(), ty.clone())].into_iter());
		let enum_types = FxHashMap::default();
		let src = SourceFile::from(Arc::new(format!("{{ \"x\": {input} }}")));
		let assignments = serde_json::Deserializer::from_str(src.contents())
			.deserialize_map(SerdeFileVisitor {
				input_types: &input_types,
				enum_types: &enum_types,
			})
			.map_err(|err| ShackleError::from_serde_json(err, &src))
			.expect("unexpected syntax error");
		assert_eq!(assignments.len(), 1);

		let val = assignments[0]
			.2
			.clone()
			.resolve_value(ty)
			.expect("unexpected resolve error");
		let s = val.to_string();
		expected.assert_eq(&s);

		// Serialize as DZN and then deserialize again ensuring it is equal
		let src = SourceFile::from(Arc::new(format!(
			"{{ \"x\":  {} }}",
			serde_json::to_string(&val).expect("unexpected serialization error")
		)));
		let assignments = serde_json::Deserializer::from_str(src.contents())
			.deserialize_map(SerdeFileVisitor {
				input_types: &input_types,
				enum_types: &enum_types,
			})
			.map_err(|err| ShackleError::from_serde_json(err, &src))
			.expect("unexpected syntax error");
		assert_eq!(assignments.len(), 1);
		let val = assignments[0]
			.clone()
			.2
			.clone()
			.resolve_value(ty)
			.expect("unexpected resolve error");
		assert_eq!(s, val.to_string());
	}

	#[test]
	fn test_parse_absent() {
		check_serialization("null", &Type::Integer(OptType::Opt), expect!("<>"));
	}

	#[test]
	fn test_parse_boolean() {
		check_serialization("true", &Type::Boolean(OptType::NonOpt), expect!("true"));
		check_serialization("false", &Type::Boolean(OptType::NonOpt), expect!("false"));
	}

	#[test]
	fn test_parse_integer() {
		check_serialization("0", &Type::Integer(OptType::NonOpt), expect!("0"));
		check_serialization("1", &Type::Integer(OptType::NonOpt), expect!("1"));
		check_serialization("99", &Type::Integer(OptType::NonOpt), expect!("99"));
		check_serialization("-1", &Type::Integer(OptType::NonOpt), expect!("-1"));
	}

	#[test]
	fn test_parse_float() {
		check_serialization("0.1", &Type::Float(OptType::NonOpt), expect!("0.1"));
		check_serialization("3.65", &Type::Float(OptType::NonOpt), expect!("3.65"));
		check_serialization("-3.65", &Type::Float(OptType::NonOpt), expect!("-3.65"));
		check_serialization(
			"4.5e10",
			&Type::Float(OptType::NonOpt),
			expect!("45000000000"),
		);
		check_serialization(
			"5E-10",
			&Type::Float(OptType::NonOpt),
			expect!("0.0000000005"),
		);
	}

	#[test]
	fn test_parse_string() {
		check_serialization("\"\"", &Type::String(OptType::NonOpt), expect!([r#""""#]));
		check_serialization(
			"\"test\"",
			&Type::String(OptType::NonOpt),
			expect!([r#""test""#]),
		);
		check_serialization(
			"\"    Another test    \"",
			&Type::String(OptType::NonOpt),
			expect!([r#""    Another test    ""#]),
		);
		check_serialization(
			"\"\\t\\n\"",
			&Type::String(OptType::NonOpt),
			expect!([r#""\t\n""#]),
		);
	}

	#[test]
	fn test_parse_tuple() {
		check_serialization(
			"[1]",
			&Type::Tuple(OptType::NonOpt, Arc::new([Type::Integer(OptType::NonOpt)])),
			expect!("(1,)"),
		);
		check_serialization(
			"[1, \"foo\"]",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Integer(OptType::NonOpt),
					Type::String(OptType::NonOpt),
				]),
			),
			expect!([r#"(1, "foo")"#]),
		);
		check_serialization(
			"[2.5, true, null]",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Float(OptType::NonOpt),
					Type::Boolean(OptType::NonOpt),
					Type::Boolean(OptType::Opt),
				]),
			),
			expect!("(2.5, true, <>)"),
		);
		check_serialization(
			"[[1, 2], { \"set\": [3, 4]}, 5]",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Array {
						opt: OptType::NonOpt,
						dim: Box::new([Type::Integer(OptType::NonOpt)]),
						element: Box::new(Type::Integer(OptType::NonOpt)),
					},
					Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
					Type::Integer(OptType::NonOpt),
				]),
			),
			expect!("([1, 2], 3..3 ∪ 4..4, 5)"),
		);
		check_serialization(
			"[1, [2, [4, 5]], 6]",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Integer(OptType::NonOpt),
					Type::Tuple(
						OptType::NonOpt,
						Arc::new([
							Type::Integer(OptType::NonOpt),
							Type::Tuple(
								OptType::NonOpt,
								Arc::new([
									Type::Integer(OptType::NonOpt),
									Type::Integer(OptType::NonOpt),
								]),
							),
						]),
					),
					Type::Integer(OptType::NonOpt),
				]),
			),
			expect!("(1, (2, (4, 5)), 6)"),
		);
	}

	#[test]
	fn test_parse_set() {
		check_serialization(
			"{\"set\": []}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
			expect!("∅"),
		);
		check_serialization(
			"{\"set\": [1.0]}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..1"),
		);
		check_serialization(
			"{\"set\": [1, 1.0]}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..1"),
		);
		check_serialization(
			"{\"set\": [[1, 3]]}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
			expect!("1..3"),
		);
		check_serialization(
			"{\"set\": [[1.0, 3.3]]}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..3.3"),
		);
	}

	#[test]
	fn test_parse_record() {
		let a: Arc<str> = "a".into();
		let b: Arc<str> = "b".into();
		let c: Arc<str> = "c".into();
		let d = "d".into();
		let e = "e".into();
		let f = "f".into();

		check_serialization(
			"{\"a\": 1, \"b\": 2.5}",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(a.clone(), Type::Integer(OptType::NonOpt)),
					(b.clone(), Type::Float(OptType::NonOpt)),
				]),
			),
			expect!("(a: 1, b: 2.5)"),
		);
		check_serialization(
			"{\"b\": [3.5, true], \"a\": {\"set\": [1, 2] }, \"c\": [null]}",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(
						a.clone(),
						Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
					),
					(
						b.clone(),
						Type::Tuple(
							OptType::NonOpt,
							Arc::new([
								Type::Float(OptType::NonOpt),
								Type::Boolean(OptType::NonOpt),
							]),
						),
					),
					(
						c.clone(),
						Type::Array {
							opt: OptType::NonOpt,
							dim: [Type::Integer(OptType::NonOpt)].into(),
							element: Type::Integer(OptType::Opt).into(),
						},
					),
				]),
			),
			expect!("(a: 1..1 ∪ 2..2, b: (3.5, true), c: [<>])"),
		);

		check_serialization(
			"{\"b\": {\"d\": {\"e\": 3, \"f\": 4}, \"c\": 2}, \"a\": 1}",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(a, Type::Integer(OptType::NonOpt)),
					(
						b,
						Type::Record(
							OptType::NonOpt,
							Arc::new([
								(c, Type::Integer(OptType::NonOpt)),
								(
									d,
									Type::Record(
										OptType::NonOpt,
										Arc::new([
											(e, Type::Integer(OptType::NonOpt)),
											(f, Type::Integer(OptType::NonOpt)),
										]),
									),
								),
							]),
						),
					),
				]),
			),
			expect!("(a: 1, b: (c: 2, d: (e: 3, f: 4)))"),
		);
	}

	#[test]
	fn test_parse_simple_array() {
		check_serialization(
			"[]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Integer(OptType::NonOpt).into(),
			},
			expect!("[]"),
		);
		check_serialization(
			"[1.0]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Float(OptType::NonOpt).into(),
			},
			expect!("[1]"),
		);
		check_serialization(
			"[1, 2.2]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Float(OptType::NonOpt).into(),
			},
			expect!("[1, 2.2]"),
		);
		check_serialization(
			"[null, null, 1, null]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Integer(OptType::Opt).into(),
			},
			expect!("[<>, <>, 1, <>]"),
		);
	}
}
