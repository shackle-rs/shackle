use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod range_list;
pub use range_list::RangeList;

/// Helper function to help skip in serialisation
fn is_false(b: &bool) -> bool {
	!(*b)
}

/// Encapsulated String helper struct
#[derive(Deserialize, Serialize)]
#[serde(rename = "string")]
struct StringLiteral {
	string: String,
}
/// Deserialisation function to resolve the encapsulation of string literals in
/// the FlatZinc serialization format
fn deserialize_encapsulated_string<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<String, D::Error> {
	let s: StringLiteral = Deserialize::deserialize(deserializer)?;
	Ok(s.string)
}

/// Serialization function to be used for the encapsulation of string literals
/// required by the FlatZinc serialization format
fn serialize_encapsulate_string<S: Serializer>(s: &str, serializer: S) -> Result<S::Ok, S::Error> {
	Serialize::serialize(
		&StringLiteral {
			string: String::from(s),
		},
		serializer,
	)
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Annotation {
	Call(Call),
	Atom(String),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "array")]
pub struct Array {
	#[serde(rename = "a")]
	pub contents: Vec<Literal>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub defined: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	pub introduced: bool,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
	Array(Vec<Literal>),
	Literal(Literal),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "call")]
pub struct Call {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	pub args: Vec<Argument>,
	pub id: Identifier,
}

pub type Domain = RangeList<i64>;

pub type Identifier = String;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Literal {
	Int(i64),
	Float(f64),
	Identifier(Identifier),
	Bool(bool),
	Set(SetLiteral),
	#[serde(
		serialize_with = "serialize_encapsulate_string",
		deserialize_with = "deserialize_encapsulated_string"
	)]
	String(String),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "method")]
pub enum Method {
	#[serde(rename = "satisfy")]
	Satisfy,
	#[serde(rename = "minimize")]
	Minimize,
	#[serde(rename = "maximize")]
	Maximize,
}

// TODO: Specialise for IntSet
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "set")]
pub struct SetLiteral {
	pub set: RangeList<f64>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "variable")]
pub struct Variable {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub defined: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<Domain>,
	#[serde(default, skip_serializing_if = "is_false")]
	pub introduced: bool,
	#[serde(rename = "rhs", skip_serializing_if = "Option::is_none")]
	pub value: Option<Literal>,
	#[serde(rename = "type")]
	pub ty: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct SolveObjective {
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	pub method: Method,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub objective: Option<Literal>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct FlatZinc {
	#[serde(default)]
	pub arrays: BTreeMap<String, Array>,
	#[serde(default)]
	pub constraints: Vec<Call>,
	#[serde(default)]
	pub output: Vec<Identifier>,
	pub solve: SolveObjective,
	#[serde(default)]
	pub variables: BTreeMap<String, Variable>,
	#[serde(default, skip_serializing_if = "String::is_empty")]
	pub version: String,
}

#[cfg(test)]
mod tests {
	use std::{fs::File, io::BufReader, path::Path};

	use expect_test::ExpectFile;

	use crate::FlatZinc;

	test_file!(documentation_example);
	test_file!(encapsulated_string);
	test_file!(set_literals);

	fn test_succesful_serialization(file: &Path, exp: ExpectFile) {
		let rdr = BufReader::new(File::open(file).unwrap());
		let fzn: FlatZinc = serde_json::from_reader(rdr).unwrap();
		exp.assert_debug_eq(&fzn);
		let fzn2: FlatZinc = serde_json::from_str(&serde_json::to_string(&fzn).unwrap()).unwrap();
		assert_eq!(fzn, fzn2)
	}

	macro_rules! test_file {
		($file: ident) => {
			#[test]
			fn $file() {
				test_succesful_serialization(
					std::path::Path::new(&format!("./corpus/{}.fzn.json", stringify!($file))),
					expect_test::expect_file![&format!(
						"../corpus/{}.debug.txt",
						stringify!($file)
					)],
				)
			}
		};
	}
	pub(crate) use test_file;
}
