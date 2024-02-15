//! Serialization of the FlatZinc data format
//!
//! FlatZinc is the language in which data and solver specific constraint models
//! are produced by the [MiniZinc](https://www.minizinc.org) compiler. This
//! crate implements the FlatZinc serialization format as described in the
//! [Interfacing Solvers to
//! FlatZinc](https://www.minizinc.org/doc-latest/en/fzn-spec.html#specification-of-flatzinc-json)
//! section of the MiniZinc reference manual. Although the
//! [serde](https://serde.rs) specification in this crate could be used with a
//! range of data formats, MiniZinc currently only outputs this formulation
//! using the JSON data format. We suggest using
//! [`serde_json`](https://crates.io/crates/serde_json) with the specification
//! in this crate to parse the FlatZinc JSON files produced by the MiniZinc
//! compiler.
//!
//! # Getting Started
//!
//! Install `flatzinc_serde` and `serde_json` for your package:
//!
//! ```bash
//! cargo add flatzinc_serde serde_json
//! ```
//!
//! Once these dependencies have been installed to your crate, you could
//! deserialize a FlatZinc JSON file as follows:
//!
//! ```
//! # use flatzinc_serde::FlatZinc;
//! # use std::{fs::File, io::BufReader, path::Path};
//! # let path = Path::new("./corpus/documentation_example.fzn.json");
//! // let path = Path::new("/lorem/ipsum/model.fzn.json");
//! let rdr = BufReader::new(File::open(path).unwrap());
//! let fzn: FlatZinc = serde_json::from_reader(rdr).unwrap();
//! // ... process FlatZinc ...
//! ```
//!
//! If, however, you want to serialize a FlatZinc format you could follow the
//! following fragment:
//!
//! ```
//! # use flatzinc_serde::FlatZinc;
//! let fzn = FlatZinc::default();
//! // ... creat solver constraint model ...
//! let json_str = serde_json::to_string(&fzn).unwrap();
//! ```
//! Note that `serde_json::to_writer`, using a buffered file writer, would be
//! preferred when writing larger FlatZinc files.
//!
//! # Register your solver with MiniZinc
//!
//! If your goal is to deserialize FlatZinc to implement a MiniZinc solver, then
//! the next step is to register your solver executable with MiniZinc. This can
//! be done by creating a [MiniZinc Solver
//! Configuration](https://www.minizinc.org/doc-2.8.2/en/fzn-spec.html#solver-configuration-files)
//! (`.msc`) file, and adding it to a folder on the `MZN_SOLVER_PATH` or a
//! standardized path, like `~/.minizinc/solvers/`. A basic solver configuration
//! for a solver that accepts JSON input would look as follows:
//!
//! ```json
//! {
//!   "name" : "My Solver",
//!   "version": "0.0.1",
//!   "id": "my.organisation.mysolver",
//!   "inputType": "JSON",
//!   "executable": "../../../bin/fzn-my-solver",
//!   "mznlib": "../mysolver"
//!   "stdFlags": [],
//!   "extraFlags": []
//! }
//! ```
//!
//! Once you have placed your configuration file on the correct path, then you
//! solver will be listed by `minizinc --solvers`. Calling `minizinc --solver
//! mysolver model.mzn data.dzn`, assuming a valid MiniZinc instance, will
//! (after compilation) invoke the registered executable with a path of a
//! FlatZinc JSON file, and potentially any registered standard and extra flags
//! (e.g., `../../../bin/fzn-my-solver model.fzn.json`).

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::encapsulate::{
	deserialize_encapsulated_set, deserialize_encapsulated_string, serialize_encapsulate_set,
	serialize_encapsulate_string,
};

mod range_list;
pub use range_list::RangeList;
mod encapsulate;

/// Helper function to help skip in serialization
fn is_false(b: &bool) -> bool {
	!(*b)
}

/// Additional information provided in a standardized format for declarations,
/// constraints, or solve objectives
///
/// In MiniZinc annotations can both be added explicitly in the model, or can be
/// added during compilation process.
///
/// Note that annotations are generally defined either in the MiniZinc standard
/// library or in a solver's redefinition library. Solvers are encouraged to
/// rewrite annotations in their redefinitions library when required.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Annotation {
	/// Atom annotation (i.e., a single [`Identifier`])
	Atom(Identifier),
	/// Call annotation
	Call(Call),
}

/// A definition of a named array literal in FlatZinc
///
/// FlatZinc Arrays are a simple (one-dimensional) sequence of [`Literal`]s.
/// These values are stored as the [`Array::contents`] member. Additional
/// information, in the form of [`Annotation`]s, from the MiniZinc model is
/// stored in [`Array::ann`] when present. When [`Array::defined`] is set to
/// `true`, then
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "array")]
pub struct Array {
	/// The values stored within the array literal
	#[serde(rename = "a")]
	pub contents: Vec<Literal>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	/// List of annotations
	pub ann: Vec<Annotation>,
	#[serde(default, skip_serializing_if = "is_false")]
	/// This field is set to `true` when there is a constraint that has been marked as
	/// defining this array.
	pub defined: bool,
	#[serde(default, skip_serializing_if = "is_false")]
	/// This field is set to `true` when the array has been introduced by the
	/// MiniZinc compiler, rather than being explicitly defined at the top-level
	/// of the MiniZinc model.
	pub introduced: bool,
}

/// The argument type associated with [`Call`]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
	/// Sequence of [`Literal`]s
	Array(Vec<Literal>),
	/// Literal
	Literal(Literal),
}

/// An object depicting a call, used for constraints and annotations
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "call")]
pub struct Call {
	/// Identifier of the function being called (e.g., a predicate or annotation)
	pub id: Identifier,
	/// Arguments of the call
	pub args: Vec<Argument>,
	/// List of annotations
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
}

/// The possible values that a (decision) [`Variable`] can take
///
/// In the case of a integer or floating point variable, a solution for the FlatZinc instance must
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Domain {
	/// Integer (or set of integer) decision variable domain
	Int(RangeList<i64>),
	/// Floating point decision variable domain
	Float(RangeList<f64>),
}

/// A name used to refer to an [`Array`], function, or [`Variable`]
pub type Identifier = String;

/// Literal values
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Literal {
	/// Integer value
	Int(i64),
	/// Floating point value
	Float(f64),
	/// Identifier, i.e., reference to an [`Array`] or [`Variable`]
	Identifier(Identifier),
	/// Boolean value
	Bool(bool),
	#[serde(
		serialize_with = "serialize_encapsulate_set",
		deserialize_with = "deserialize_encapsulated_set"
	)]
	/// Set of integers, represented as a list of integer ranges
	IntSet(RangeList<i64>),
	#[serde(
		serialize_with = "serialize_encapsulate_set",
		deserialize_with = "deserialize_encapsulated_set"
	)]
	/// Set of floating point values, represented as a list of floating point
	/// ranges
	FloatSet(RangeList<f64>),
	#[serde(
		serialize_with = "serialize_encapsulate_string",
		deserialize_with = "deserialize_encapsulated_string"
	)]
	/// String value
	String(String),
}

/// Goal of solving a FlatZinc instance
#[derive(Default, Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "method")]
pub enum Method {
	/// Find any solution
	#[serde(rename = "satisfy")]
	#[default]
	Satisfy,
	/// Find the solution with the lowest objective value
	#[serde(rename = "minimize")]
	Minimize,
	/// Find the solution with the highest objective value
	#[serde(rename = "maximize")]
	Maximize,
}

/// Used to signal the type of (decision) [`Variable`]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "type")]
pub enum Type {
	/// Boolean decision variable
	#[serde(rename = "bool")]
	Bool,
	/// Integer decision variable
	#[serde(rename = "int")]
	Int,
	/// Floating point decision variable
	#[serde(rename = "float")]
	Float,
	/// Integer set decision variable
	#[serde(rename = "set of int")]
	IntSet,
}

/// The definition of a decision variable
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "variable")]
pub struct Variable {
	/// The type of the decision variable
	#[serde(rename = "type")]
	pub ty: Type,
	/// The set of potential values from which the decision variable must take its
	/// value in a solution
	///
	/// If domain has the value `None`, then all values of the decision variable's
	/// `Type` are allowed in a solution.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<Domain>,
	/// The “right hand side” of the variable, i.e., its value or alias to another
	/// variable
	#[serde(rename = "rhs", skip_serializing_if = "Option::is_none")]
	pub value: Option<Literal>,
	/// A list of annotations
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
	/// This field is set to `true` when there is a constraint that has been marked as
	/// defining this variable.
	#[serde(default, skip_serializing_if = "is_false")]
	pub defined: bool,
	/// This field is set to `true` when the variable has been introduced by the
	/// MiniZinc compiler, rather than being explicitly defined at the top-level
	/// of the MiniZinc model.
	#[serde(default, skip_serializing_if = "is_false")]
	pub introduced: bool,
}

/// A specification of objective of a FlatZinc instance
#[derive(Default, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct SolveObjective {
	/// The type of goal
	pub method: Method,
	/// The variable to optimize, or `None` if [`SolveObjective::method`] is [`Method::Satisfy`]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub objective: Option<Literal>,
	/// A list of annotations from the solve statement in the MiniZinc model
	///
	/// Note that this includes the search annotations if they are present in the
	/// model.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation>,
}

/// The structure depicting a FlatZinc instance
///
/// FlatZinc is (generally) a format produced by the MiniZinc compiler as a
/// result of instantiating the parameter variables of a MiniZinc model and
/// generating a solver-specific equisatisfiable model.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct FlatZinc {
	/// A mapping from decision variable [`Identifier`] to their definitions
	#[serde(default)]
	pub variables: BTreeMap<Identifier, Variable>,
	/// A mapping from array [`Identifier`] to their definitions
	#[serde(default)]
	pub arrays: BTreeMap<Identifier, Array>,
	/// A list of (solver-specific) constraints, in the form of [`Call`] objects,
	/// that must be satisfied in a solution.
	#[serde(default)]
	pub constraints: Vec<Call>,
	/// A list of all identifiers for which the solver must produce output for each solution
	#[serde(default)]
	pub output: Vec<Identifier>,
	/// A specification of the goal of solving the FlatZinc instance.
	pub solve: SolveObjective,
	/// The version of the FlatZinc serialization specification used
	#[serde(default, skip_serializing_if = "String::is_empty")]
	pub version: String,
}

impl Default for FlatZinc {
	fn default() -> Self {
		Self {
			variables: Default::default(),
			arrays: Default::default(),
			constraints: Default::default(),
			output: Default::default(),
			solve: Default::default(),
			version: "1.0".into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{fs::File, io::BufReader, path::Path};

	use expect_test::ExpectFile;

	use crate::FlatZinc;

	test_file!(documentation_example);
	test_file!(encapsulated_string);
	test_file!(float_sets);
	test_file!(set_literals);

	fn test_successful_serialization(file: &Path, exp: ExpectFile) {
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
				test_successful_serialization(
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
