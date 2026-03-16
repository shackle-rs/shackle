//! Helper structures and functions to parse and output using `serde` certain
//! types in the FlatZinc JSON serialization

use std::{fmt, marker::PhantomData};

use serde::{
	de::{MapAccess, Visitor},
	ser::SerializeMap,
	Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{RangeList, Type};

/// Base variable type used for the `"type"` field in FlatZinc JSON.
///
/// This mirrors the JSON encoding, where the type name and optional domain are
/// serialized as separate fields.
#[derive(Clone, Copy, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "type")]
pub(crate) enum BaseType {
	/// Boolean decision variable type encoded as `"bool"`.
	#[serde(rename = "bool")]
	Bool,
	/// Integer decision variable type encoded as `"int"`.
	#[serde(rename = "int")]
	Int,
	/// Floating-point decision variable type encoded as `"float"`.
	#[serde(rename = "float")]
	Float,
	/// Integer set decision variable type encoded as `"set of int"`.
	#[serde(rename = "set of int")]
	IntSet,
}

/// Domain payload used for the optional `"domain"` field in FlatZinc JSON.
///
/// This is kept separate from [`BaseType`] so [`crate::Variable`] can preserve
/// the historical JSON shape while internally storing domains inside [`Type`].
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum VariableDomain {
	/// Integer domain payload serialized as a JSON array of inclusive bounds.
	#[serde(deserialize_with = "deserialize_set", serialize_with = "serialize_set")]
	Int(RangeList<i64>),
	/// Floating-point domain payload serialized as a JSON array of inclusive bounds.
	#[serde(deserialize_with = "deserialize_set", serialize_with = "serialize_set")]
	Float(RangeList<f64>),
}

/// Encapsulated String helper struct
#[derive(Deserialize, Serialize)]
#[serde(rename = "string")]
struct StringLiteral {
	/// Content of the string literal.
	string: String,
}
/// Deserialization function to resolve the encapsulation of string literals in
/// the FlatZinc serialization format
pub(crate) fn deserialize_encapsulated_string<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<String, D::Error> {
	let s: StringLiteral = Deserialize::deserialize(deserializer)?;
	Ok(s.string)
}

/// Serialization function to be used for the encapsulation of string literals
/// required by the FlatZinc serialization format
pub(crate) fn serialize_encapsulate_string<S: Serializer>(
	s: &str,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	Serialize::serialize(
		&StringLiteral {
			string: String::from(s),
		},
		serializer,
	)
}

/// Encapsulated set helper struct
#[derive(Deserialize, Serialize)]
#[serde(rename = "set")]
struct SetLiteral<E: PartialOrd> {
	/// Range list used to represent the content of the set.
	set: Vec<(E, E)>,
}
/// Deserialization function to resolve the encapsulation of set literals in the
/// FlatZinc serialization format
pub(crate) fn deserialize_encapsulated_set<
	'de,
	D: Deserializer<'de>,
	E: Copy + Deserialize<'de> + PartialOrd + 'static,
>(
	deserializer: D,
) -> Result<RangeList<E>, D::Error> {
	let s: SetLiteral<E> = Deserialize::deserialize(deserializer)?;
	let range = s.set.into_iter().map(|(a, b)| a..=b).collect();
	Ok(range)
}

/// Serialization function to be used for the encapsulation of set literals
/// required by the FlatZinc serialization format
pub(crate) fn serialize_encapsulate_set<E: PartialOrd + Serialize + Copy, S: Serializer>(
	r: &RangeList<E>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	Serialize::serialize(
		&SetLiteral {
			set: r.iter().map(|r| (*r.start(), *r.end())).collect(),
		},
		serializer,
	)
}

pub(crate) fn deserialize_set<
	'de,
	D: Deserializer<'de>,
	E: Copy + Deserialize<'de> + PartialOrd + 'static,
>(
	deserializer: D,
) -> Result<RangeList<E>, D::Error> {
	let s: Vec<(E, E)> = Deserialize::deserialize(deserializer)?;
	let range = s.into_iter().map(|(a, b)| a..=b).collect();
	Ok(range)
}

/// Serialization function to be used for the encapsulation of set literals
/// required by the FlatZinc serialization format
pub(crate) fn serialize_set<E: PartialOrd + Serialize + Copy, S: Serializer>(
	r: &RangeList<E>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let x: Vec<(E, E)> = r.iter().map(|r| (*r.start(), *r.end())).collect();
	Serialize::serialize(&x, serializer)
}

/// Deserialization function for object-like fields that can collect `(K, V)`
/// pairs into arbitrary map-like containers (e.g. `BTreeMap`, `HashMap`,
/// `Vec<(K, V)>`).
pub(crate) fn deserialize_key_value_object<'de, D, M, K, V>(deserializer: D) -> Result<M, D::Error>
where
	D: Deserializer<'de>,
	M: FromIterator<(K, V)>,
	K: Deserialize<'de>,
	V: Deserialize<'de>,
{
	struct MapAccessIter<'de, 'a, A, K, V>
	where
		A: MapAccess<'de>,
	{
		map: &'a mut A,
		phantom: PhantomData<(&'de (), K, V)>,
	}

	impl<'de, 'a, A, K, V> MapAccessIter<'de, 'a, A, K, V>
	where
		A: MapAccess<'de>,
	{
		fn new(map: &'a mut A) -> Self {
			Self {
				map,
				phantom: PhantomData,
			}
		}
	}

	impl<'de, A, K, V> Iterator for MapAccessIter<'de, '_, A, K, V>
	where
		A: MapAccess<'de>,
		K: Deserialize<'de>,
		V: Deserialize<'de>,
	{
		type Item = Result<(K, V), A::Error>;

		fn next(&mut self) -> Option<Self::Item> {
			self.map.next_entry().transpose()
		}

		fn size_hint(&self) -> (usize, Option<usize>) {
			match self.map.size_hint() {
				Some(n) => (n, Some(n)),
				None => (0, None),
			}
		}
	}

	struct KeyValueObjectVisitor<M, K, V>(PhantomData<(M, K, V)>);

	impl<'de, M, K, V> Visitor<'de> for KeyValueObjectVisitor<M, K, V>
	where
		M: FromIterator<(K, V)>,
		K: Deserialize<'de>,
		V: Deserialize<'de>,
	{
		type Value = M;

		fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
			formatter.write_str("a JSON object")
		}

		fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
		where
			A: MapAccess<'de>,
		{
			MapAccessIter::<A, K, V>::new(&mut map).collect()
		}
	}

	deserializer.deserialize_map(KeyValueObjectVisitor(PhantomData))
}

/// Serialization function for map-like containers represented as iterables of
/// key-value pairs.
pub(crate) fn serialize_key_value_object<'a, S, M, K, V>(
	map_like: &'a M,
	serializer: S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	&'a M: IntoIterator<Item = (&'a K, &'a V)>,
	K: Serialize + 'a,
	V: Serialize + 'a,
{
	let mut ser_map = serializer.serialize_map(None)?;
	for (key, value) in map_like {
		ser_map.serialize_entry(key, value)?;
	}
	ser_map.end()
}
