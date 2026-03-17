//! Helper structures and functions to parse and output using `serde` certain
//! types in the FlatZinc JSON serialization

use std::{fmt, marker::PhantomData};

use serde::{
	de::{MapAccess, Visitor},
	ser::SerializeMap,
	Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{Annotation, Literal, Method, RangeList, SolveObjective, Type, Variable};

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

/// Encapsulated set helper struct
#[derive(Deserialize, Serialize)]
#[serde(rename = "set")]
struct SetLiteral<E: PartialOrd> {
	/// Range list used to represent the content of the set.
	set: Vec<(E, E)>,
}

/// Encapsulated String helper struct
#[derive(Deserialize, Serialize)]
#[serde(rename = "string")]
struct StringLiteral {
	/// Content of the string literal.
	string: String,
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
/// Deserialization function to resolve the encapsulation of string literals in
/// the FlatZinc serialization format
pub(crate) fn deserialize_encapsulated_string<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<String, D::Error> {
	let s: StringLiteral = Deserialize::deserialize(deserializer)?;
	Ok(s.string)
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

/// Helper function used by serde field attributes to omit `false` flags.
pub(crate) fn is_false(b: &bool) -> bool {
	!(*b)
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

/// Serialization function to be used for the encapsulation of set literals
/// required by the FlatZinc serialization format
pub(crate) fn serialize_set<E: PartialOrd + Serialize + Copy, S: Serializer>(
	r: &RangeList<E>,
	serializer: S,
) -> Result<S::Ok, S::Error> {
	let x: Vec<(E, E)> = r.iter().map(|r| (*r.start(), *r.end())).collect();
	Serialize::serialize(&x, serializer)
}

impl<'de, Identifier: Deserialize<'de>> Deserialize<'de> for SolveObjective<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(rename = "solve")]
		#[serde(bound(deserialize = "Identifier: Deserialize<'de>"))]
		struct SolveObjectiveRepr<Identifier> {
			#[serde(rename = "method")]
			method: String,
			#[serde(default)]
			objective: Option<Literal<Identifier>>,
			#[serde(default, skip_serializing_if = "Vec::is_empty")]
			ann: Vec<Annotation<Identifier>>,
		}

		let repr = SolveObjectiveRepr::deserialize(deserializer)?;
		let method = match (repr.method.as_str(), repr.objective) {
			("satisfy", None) => Method::Satisfy,
			("satisfy", Some(_)) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"satisfy solve items cannot have an objective",
				));
			}
			("minimize", Some(objective)) => Method::Minimize(objective),
			("minimize", None) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"minimize solve items require an objective",
				));
			}
			("maximize", Some(objective)) => Method::Maximize(objective),
			("maximize", None) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"maximize solve items require an objective",
				));
			}
			(method, _) => {
				return Err(<D::Error as ::serde::de::Error>::custom(format!(
					"unknown solve method '{method}'",
				)));
			}
		};

		Ok(SolveObjective {
			method,
			ann: repr.ann,
		})
	}
}

impl<Identifier: Serialize> Serialize for SolveObjective<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(rename = "solve")]
		struct SolveObjectiveRepr<'a, Identifier> {
			#[serde(rename = "method")]
			method: &'static str,
			#[serde(skip_serializing_if = "Option::is_none")]
			objective: Option<&'a Literal<Identifier>>,
			#[serde(default, skip_serializing_if = "Vec::is_empty")]
			ann: &'a Vec<Annotation<Identifier>>,
		}

		let (method, objective) = match &self.method {
			Method::Satisfy => ("satisfy", None),
			Method::Minimize(objective) => ("minimize", Some(objective)),
			Method::Maximize(objective) => ("maximize", Some(objective)),
		};

		SolveObjectiveRepr {
			method,
			objective,
			ann: &self.ann,
		}
		.serialize(serializer)
	}
}

impl<'de, Identifier: Deserialize<'de>> Deserialize<'de> for Variable<Identifier> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(Deserialize)]
		#[serde(rename = "variable")]
		#[serde(bound(deserialize = "Identifier: Deserialize<'de>"))]
		struct VariableRepr<Identifier> {
			/// Base type stored in the JSON `"type"` field.
			#[serde(rename = "type")]
			ty: BaseType,
			/// Optional domain stored in the JSON `"domain"` field.
			#[serde(skip_serializing_if = "Option::is_none")]
			domain: Option<VariableDomain>,
			/// Optional right-hand side stored in the JSON `"rhs"` field.
			#[serde(rename = "rhs", skip_serializing_if = "Option::is_none")]
			value: Option<Literal<Identifier>>,
			/// Variable annotations stored in the JSON `"ann"` field.
			#[serde(default, skip_serializing_if = "Vec::is_empty")]
			ann: Vec<Annotation<Identifier>>,
			/// Whether the variable is solver-defined.
			#[serde(default, skip_serializing_if = "is_false")]
			defined: bool,
			/// Whether the variable was introduced during MiniZinc lowering.
			#[serde(default, skip_serializing_if = "is_false")]
			introduced: bool,
		}

		let repr = VariableRepr::deserialize(deserializer)?;
		let ty = match (repr.ty, repr.domain) {
			(BaseType::Bool, None) => Type::Bool,
			(BaseType::Bool, Some(_)) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"bool variables cannot have a domain",
				));
			}
			(BaseType::Int, None) => Type::Int(None),
			(BaseType::Int, Some(VariableDomain::Int(domain))) => Type::Int(Some(domain)),
			(BaseType::Int, Some(VariableDomain::Float(_))) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"int variables require an int domain",
				));
			}
			(BaseType::Float, None) => Type::Float(None),
			(BaseType::Float, Some(VariableDomain::Float(domain))) => Type::Float(Some(domain)),
			(BaseType::Float, Some(VariableDomain::Int(_))) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"float variables require a float domain",
				));
			}
			(BaseType::IntSet, None) => Type::IntSet(None),
			(BaseType::IntSet, Some(VariableDomain::Int(domain))) => Type::IntSet(Some(domain)),
			(BaseType::IntSet, Some(VariableDomain::Float(_))) => {
				return Err(<D::Error as ::serde::de::Error>::custom(
					"set of int variables require an int domain",
				));
			}
		};

		Ok(Variable {
			ty,
			value: repr.value,
			ann: repr.ann,
			defined: repr.defined,
			introduced: repr.introduced,
		})
	}
}

impl<Identifier: Serialize> Serialize for Variable<Identifier> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(Serialize)]
		#[serde(rename = "variable")]
		struct VariableRepr<'a, Identifier> {
			/// Base type stored in the JSON `"type"` field.
			#[serde(rename = "type")]
			ty: BaseType,
			/// Optional domain stored in the JSON `"domain"` field.
			#[serde(skip_serializing_if = "Option::is_none")]
			domain: Option<VariableDomain>,
			/// Optional right-hand side stored in the JSON `"rhs"` field.
			#[serde(rename = "rhs", skip_serializing_if = "Option::is_none")]
			value: Option<&'a Literal<Identifier>>,
			/// Variable annotations stored in the JSON `"ann"` field.
			#[serde(default, skip_serializing_if = "Vec::is_empty")]
			ann: &'a Vec<Annotation<Identifier>>,
			/// Whether the variable is solver-defined.
			#[serde(default, skip_serializing_if = "is_false")]
			defined: bool,
			/// Whether the variable was introduced during MiniZinc lowering.
			#[serde(default, skip_serializing_if = "is_false")]
			introduced: bool,
		}

		let (ty, domain) = match &self.ty {
			Type::Bool => (BaseType::Bool, None),
			Type::Int(domain) => (BaseType::Int, domain.clone().map(VariableDomain::Int)),
			Type::Float(domain) => (BaseType::Float, domain.clone().map(VariableDomain::Float)),
			Type::IntSet(domain) => (BaseType::IntSet, domain.clone().map(VariableDomain::Int)),
		};

		VariableRepr {
			ty,
			domain,
			value: self.value.as_ref(),
			ann: &self.ann,
			defined: self.defined,
			introduced: self.introduced,
		}
		.serialize(serializer)
	}
}

#[cfg(test)]
mod tests {
	macro_rules! test_file {
		($file: ident) => {
			#[test]
			fn $file() {
				test_successful_serialization(
					std::path::Path::new(&format!("./corpus/json/{}.fzn.json", stringify!($file))),
					expect_test::expect_file![&format!(
						"../corpus/json/{}.debug.txt",
						stringify!($file)
					)],
				)
			}
		};
	}

	use std::{
		collections::{BTreeMap, HashMap},
		fs::File,
		io::{BufReader, Read},
		path::Path,
	};

	use expect_test::ExpectFile;
	use rangelist::RangeList;
	use ustr::Ustr;

	use crate::{
		Annotation, AnnotationArgument, AnnotationCall, AnnotationLiteral, Array, FlatZinc,
		Literal, Method, SolveObjective, Type, Variable,
	};

	#[test]
	fn test_default_with_custom_map_types() {
		let fzn = FlatZinc::<
			String,
			HashMap<String, Variable<String>>,
			Vec<(String, Array<String>)>,
		>::default();
		assert!(fzn.variables.is_empty());
		assert!(fzn.arrays.is_empty());
		assert!(fzn.constraints.is_empty());
		assert!(fzn.output.is_empty());
		assert_eq!(fzn.version, "1.0");
	}

	#[test]
	fn test_hashmap_backed_maps_deserialize() {
		type HashMapFlatZinc =
			FlatZinc<String, HashMap<String, Variable<String>>, HashMap<String, Array<String>>>;

		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/json/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		let _ = rdr.read_to_string(&mut content).unwrap();

		let fzn: HashMapFlatZinc = serde_json::from_str(&content).unwrap();

		let fzn2: HashMapFlatZinc = {
			let json = serde_json::to_string(&fzn).unwrap();
			serde_json::from_str(&json).unwrap()
		};
		assert_eq!(fzn, fzn2);
	}

	#[test]
	fn test_ident_interned() {
		let rdr = BufReader::new(
			File::open(Path::new("./corpus/json/documentation_example.fzn.json")).unwrap(),
		);
		let fzn: FlatZinc<Ustr> = serde_json::from_reader(rdr).unwrap();
		expect_test::expect_file!["../corpus/json/documentation_example.debug_ustr.txt"]
			.assert_debug_eq(&fzn)
	}

	#[test]
	fn test_ident_no_copy() {
		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/json/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		let _ = rdr.read_to_string(&mut content).unwrap();

		let fzn: FlatZinc<&str> = serde_json::from_str(&content).unwrap();
		expect_test::expect_file!["../corpus/json/documentation_example.debug.txt"]
			.assert_debug_eq(&fzn)
	}

	#[test]
	fn test_print_flatzinc() {
		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/json/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		let _ = rdr.read_to_string(&mut content).unwrap();

		let fzn: FlatZinc<&str> = serde_json::from_str(&content).unwrap();
		expect_test::expect_file!["../corpus/fzn/documentation_example.fzn"]
			.assert_eq(&fzn.to_string());

		let ann: Annotation<&str> = Annotation::Call(AnnotationCall {
			id: "bool_search",
			args: vec![
				AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(Literal::Identifier(
					"input_order",
				))),
				AnnotationArgument::Literal(AnnotationLiteral::BaseLiteral(Literal::Identifier(
					"indomain_min",
				))),
			],
		});
		assert_eq!(ann.to_string(), "::bool_search(input_order, indomain_min)");

		let ty = Type::Bool;
		assert_eq!(ty.to_string(), "bool");
		let ty = Type::Int(None);
		assert_eq!(ty.to_string(), "int");
		let ty = Type::Float(None);
		assert_eq!(ty.to_string(), "float");
		let ty = Type::IntSet(None);
		assert_eq!(ty.to_string(), "set of int");
		let ty = Type::Float(Some(RangeList::from(1.0..=4.0)));
		assert_eq!(ty.to_string(), "1.0..4.0");

		let lit = Literal::<&str>::Int(1);
		assert_eq!(lit.to_string(), "1");
		let lit = Literal::<&str>::Float(1.0);
		assert_eq!(lit.to_string(), "1.0");
		let lit = Literal::<&str>::Identifier("x");
		assert_eq!(lit.to_string(), "x");
		let lit = Literal::<&str>::Bool(true);
		assert_eq!(lit.to_string(), "true");
		let lit = Literal::<&str>::IntSet(RangeList::from(2..=3));
		assert_eq!(lit.to_string(), "2..3");
		let lit = Literal::<&str>::FloatSet(RangeList::from(2.0..=3.0));
		assert_eq!(lit.to_string(), "2.0..3.0");
		let lit = Literal::<&str>::String(String::from("hello"));
		assert_eq!(lit.to_string(), "\"hello\"");

		let fzn = FlatZinc {
			variables: BTreeMap::from([(
				"x",
				Variable {
					ty: Type::IntSet(None),
					ann: vec![Annotation::Atom("special")],
					defined: false,
					introduced: true,
					value: Some(Literal::IntSet(RangeList::from(1..=4))),
				},
			)]),
			arrays: BTreeMap::from([(
				"y",
				Array {
					ann: vec![Annotation::Atom("special")],
					contents: vec![Literal::Int(1), Literal::Int(2), Literal::Int(3)],
					introduced: true,
					defined: true,
				},
			)]),
			output: vec!["y"],
			..Default::default()
		};
		assert_eq!(
			fzn.to_string(),
			"var set of int: x ::var_is_introduced ::special = 1..4;\narray[1..3] of int: y ::output_array([1..3]) ::is_defined_var ::var_is_introduced ::special = [1, 2, 3];\nsolve satisfy;\n"
		);

		let sat = SolveObjective {
			method: Method::Minimize(Literal::Identifier("x")),
			ann: vec![ann],
		};
		assert_eq!(
			sat.to_string(),
			"solve ::bool_search(input_order, indomain_min) minimize x"
		);
	}

	fn test_successful_serialization(file: &Path, exp: ExpectFile) {
		let rdr = BufReader::new(File::open(file).unwrap());
		let fzn: FlatZinc = serde_json::from_reader(rdr).unwrap();
		exp.assert_debug_eq(&fzn);
		let fzn2: FlatZinc = serde_json::from_str(&serde_json::to_string(&fzn).unwrap()).unwrap();
		assert_eq!(fzn, fzn2)
	}

	#[test]
	fn test_vec_backed_maps_deserialize_from_object_shape() {
		type VecFlatZinc =
			FlatZinc<String, Vec<(String, Variable<String>)>, Vec<(String, Array<String>)>>;

		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/json/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		let _ = rdr.read_to_string(&mut content).unwrap();

		let fzn: VecFlatZinc = serde_json::from_str(&content).unwrap();
		assert!(!fzn.variables.is_empty());
		assert!(!fzn.arrays.is_empty());
	}

	test_file!(documentation_example);
	test_file!(encapsulated_string);
	test_file!(float_sets);
	test_file!(set_literals);
	test_file!(unit_test_example);
}
