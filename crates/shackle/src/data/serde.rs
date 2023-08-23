use itertools::Itertools;
use rustc_hash::FxHashMap;
use serde::de::{DeserializeSeed, Error, IgnoredAny, Unexpected, Visitor};

use super::ParserVal;
use crate::{OptType, Type};

pub(crate) struct SerdeValueVisitor<'a>(pub &'a Type);

impl<'de, 'a> Visitor<'de> for SerdeValueVisitor<'a> {
	type Value = ParserVal;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "a value of type \"{}\"", self.0)
	}

	fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
		let ty = self.0;
		if matches!(ty, Type::Boolean(_) | Type::Integer(_) | Type::Float(_)) {
			Ok(ParserVal::Boolean(v))
		} else {
			Err(Error::invalid_type(Unexpected::Bool(v), &self))
		}
	}

	fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
		let ty = self.0;
		if matches!(ty, Type::Integer(_) | Type::Float(_)) {
			Ok(ParserVal::Integer(v))
		} else {
			Err(Error::invalid_type(Unexpected::Signed(v), &self))
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

	fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
		let ty = self.0;
		if ty.is_opt() {
			Ok(ParserVal::Absent)
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
						return Err(Error::unknown_field(
							k,
							&[], //TODO: Can this be made to work?
						));
					}
				}
				if !types.is_empty() {
					let field = types.into_iter().next().unwrap().0.to_string();
					return Err(Error::missing_field(Box::leak(field.into_boxed_str()))); // TODO: Can we avoid leaking memory here?
				}
				Ok(ParserVal::Record(rec))
			}
			Type::Set(_, _) => todo!(),
			Type::Enum(_, _) => todo!(),
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

pub(crate) struct SerdeFileVisitor<'a>(pub &'a FxHashMap<String, Type>);

impl<'de, 'a> Visitor<'de> for SerdeFileVisitor<'a> {
	type Value = Vec<(&'a String, &'a Type, ParserVal)>;

	fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(formatter, "assignment mapping")
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let type_map = self.0;
		let mut assignments = Vec::with_capacity(map.size_hint().unwrap_or(0));

		while let Some(k) = map.next_key::<&str>()? {
			if let Some((ident, ty)) = type_map.get_key_value(k) {
				let value = map.next_value_seed(SerdeValueVisitor(ty))?;
				assignments.push((ident, ty, value));
			}
			// Ignore unknown
		}
		Ok(assignments)
	}
}
