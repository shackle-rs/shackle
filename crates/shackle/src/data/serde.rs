use rustc_hash::FxHashMap;
use serde::de::{DeserializeSeed, Error, Unexpected, Visitor};

use super::ParserVal;
use crate::Type;

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
		let ty = self.0;
		if matches!(ty, Type::String(_)) {
			Ok(ParserVal::String(v.to_string()))
		} else {
			Err(Error::invalid_type(Unexpected::Str(v), &self))
		}
	}

	fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
		self.visit_str(v) // TODO: avoid copying when possible
	}

	fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
		let ty = self.0;
		if ty.is_opt() {
			Ok(ParserVal::Absent)
		} else {
			Err(Error::invalid_type(Unexpected::Option, &self))
		}
	}

	fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
		let _ = seq;
		Err(Error::invalid_type(Unexpected::Seq, &self))
	}

	fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
		let ty = self.0;
		match ty {
			Type::Record(_, ty) => {
				let mut rec = Vec::with_capacity(ty.len());
				let mut types = FxHashMap::from_iter(ty.iter().map(|(a, b)| (a.as_str(), b)));
				while let Some(k) = map.next_key::<&str>()? {
					if let Some(ty) = types.remove(k) {
						let val = map.next_value_seed(SerdeValueVisitor(ty))?;
						rec.push((k.to_string(), val))
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
			Type::Set(_, _) => {
				todo!()
			}
			Type::Enum(_, _) => {
				todo!()
			}
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
