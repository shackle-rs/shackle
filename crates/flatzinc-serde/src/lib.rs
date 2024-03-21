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
//! let fzn = FlatZinc::<String>::default();
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

use std::{collections::BTreeMap, fmt::Display};

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
pub enum Annotation<Identifier = String> {
	/// Atom annotation (i.e., a single [`Identifier`])
	Atom(Identifier),
	/// Call annotation
	Call(Call<Identifier>),
}

impl<Identifier: Display> Display for Annotation<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "::")?;
		match self {
			Annotation::Atom(a) => write!(f, "{a}"),
			Annotation::Call(c) => write!(f, "{c}"),
		}
	}
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
pub struct Array<Identifier = String> {
	/// The values stored within the array literal
	#[serde(rename = "a")]
	pub contents: Vec<Literal<Identifier>>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	/// List of annotations
	pub ann: Vec<Annotation<Identifier>>,
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

impl<Identifier: Ord> Array<Identifier> {
	fn determine_type(&self, fzn: &FlatZinc<Identifier>) -> (&str, bool) {
		let ty = match self.contents.first().unwrap() {
			Literal::Int(_) => "int",
			Literal::Float(_) => "float",
			Literal::Identifier(ident) => match fzn.variables[ident].ty {
				Type::Bool => "bool",
				Type::Int => "int",
				Type::Float => "float",
				Type::IntSet => "set of int",
			},
			Literal::Bool(_) => "bool",
			Literal::IntSet(_) => "set of int",
			Literal::FloatSet(_) => "set of float",
			Literal::String(_) => "string",
		};
		let is_var = self.contents.iter().any(|lit| match lit {
			Literal::Identifier(ident) => fzn.variables[ident].value.is_none(),
			_ => false,
		});
		(ty, is_var)
	}
}

/// The argument type associated with [`Call`]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument<Identifier = String> {
	/// Sequence of [`Literal`]s
	Array(Vec<Literal<Identifier>>),
	/// Literal
	Literal(Literal<Identifier>),
}

impl<Identifier: Display> Display for Argument<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Argument::Array(arr) => {
				write!(f, "[")?;
				let mut first = true;
				for v in arr {
					if !first {
						write!(f, ", ")?
					}
					write!(f, "{v}")?;
					first = false;
				}
				write!(f, "]")
			}
			Argument::Literal(lit) => write!(f, "{lit}"),
		}
	}
}

/// An object depicting a call, used for constraints and annotations
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "call")]
pub struct Call<Identifier = String> {
	/// Identifier of the function being called (e.g., a predicate or annotation)
	pub id: Identifier,
	/// Arguments of the call
	pub args: Vec<Argument<Identifier>>,
	/// List of annotations
	#[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation<Identifier>>,
}

impl<Identifier: Display> Display for Call<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}(", self.id)?;
		let mut first = true;
		for arg in &self.args {
			if !first {
				write!(f, ", ")?
			}
			write!(f, "{arg}")?;
			first = false;
		}
		write!(f, ")")?;
		for a in &self.ann {
			write!(f, " {a}")?
		}
		Ok(())
	}
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

impl Display for Domain {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Domain::Int(is) => write!(f, "{is}"),
			Domain::Float(fs) => write!(f, "{fs}"),
		}
	}
}

/// A name used to refer to an [`Array`], function, or [`Variable`]
// pub type Identifier = String;

/// Literal values
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Literal<Identifier = String> {
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

impl<Identifier: Display> Display for Literal<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Literal::Int(i) => write!(f, "{i}"),
			Literal::Float(x) => write!(f, "{x}"),
			Literal::Identifier(ident) => write!(f, "{ident}"),
			Literal::Bool(b) => write!(f, "{b}"),
			Literal::IntSet(is) => write!(f, "{is}"),
			Literal::FloatSet(fs) => write!(f, "{fs}"),
			Literal::String(s) => write!(f, "{s:?}"),
		}
	}
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

impl Display for Method {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Method::Satisfy => write!(f, "satisfy"),
			Method::Minimize => write!(f, "minimize"),
			Method::Maximize => write!(f, "maximize"),
		}
	}
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

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Bool => write!(f, "bool"),
			Type::Int => write!(f, "int"),
			Type::Float => write!(f, "float"),
			Type::IntSet => write!(f, "set of int"),
		}
	}
}

/// The definition of a decision variable
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename = "variable")]
pub struct Variable<Identifier = String> {
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
	pub value: Option<Literal<Identifier>>,
	/// A list of annotations
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation<Identifier>>,
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
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct SolveObjective<Identifier = String> {
	/// The type of goal
	pub method: Method,
	/// The variable to optimize, or `None` if [`SolveObjective::method`] is [`Method::Satisfy`]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub objective: Option<Literal<Identifier>>,
	/// A list of annotations from the solve statement in the MiniZinc model
	///
	/// Note that this includes the search annotations if they are present in the
	/// model.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub ann: Vec<Annotation<Identifier>>,
}

impl<Identifier> Default for SolveObjective<Identifier> {
	fn default() -> Self {
		Self {
			method: Default::default(),
			objective: None,
			ann: Vec::new(),
		}
	}
}

impl<Identifier: Display> Display for SolveObjective<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "solve ")?;
		for a in &self.ann {
			write!(f, "::{a} ")?;
		}
		write!(f, "{}", self.method)?;
		if let Some(obj) = &self.objective {
			write!(f, " {obj}")?
		}
		Ok(())
	}
}

/// The structure depicting a FlatZinc instance
///
/// FlatZinc is (generally) a format produced by the MiniZinc compiler as a
/// result of instantiating the parameter variables of a MiniZinc model and
/// generating a solver-specific equisatisfiable model.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct FlatZinc<Identifier: Ord = String> {
	/// A mapping from decision variable [`Identifier`] to their definitions
	#[serde(default)]
	pub variables: BTreeMap<Identifier, Variable<Identifier>>,
	/// A mapping from array [`Identifier`] to their definitions
	#[serde(default)]
	pub arrays: BTreeMap<Identifier, Array<Identifier>>,
	/// A list of (solver-specific) constraints, in the form of [`Call`] objects,
	/// that must be satisfied in a solution.
	#[serde(default)]
	pub constraints: Vec<Call<Identifier>>,
	/// A list of all identifiers for which the solver must produce output for each solution
	#[serde(default)]
	pub output: Vec<Identifier>,
	/// A specification of the goal of solving the FlatZinc instance.
	pub solve: SolveObjective<Identifier>,
	/// The version of the FlatZinc serialization specification used
	#[serde(default, skip_serializing_if = "String::is_empty")]
	pub version: String,
}

impl<Identifier: Ord> Default for FlatZinc<Identifier> {
	fn default() -> Self {
		Self {
			variables: Default::default(),
			arrays: BTreeMap::new(),
			constraints: Vec::new(),
			output: Default::default(),
			solve: Default::default(),
			version: "1.0".into(),
		}
	}
}

impl<Identifier: Ord + Display> Display for FlatZinc<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let output_map: BTreeMap<&Identifier, ()> =
			self.output.iter().map(|ident| (ident, ())).collect();

		for (ident, var) in &self.variables {
			write!(f, "var ")?;
			if let Some(dom) = &var.domain {
				write!(f, "{dom}")?
			} else {
				write!(f, "{}", var.ty)?
			}
			write!(f, ": {ident}")?;
			if output_map.contains_key(&ident) {
				write!(f, " ::output_var")?;
			}
			if var.defined {
				write!(f, " ::is_defined_var")?;
			}
			if var.introduced {
				write!(f, " ::var_is_introduced")?;
			}
			for ann in &var.ann {
				write!(f, " {ann}")?
			}
			if let Some(val) = &var.value {
				write!(f, "= {val}")?
			}
			writeln!(f, ";")?
		}
		for (ident, arr) in &self.arrays {
			let (ty, is_var) = arr.determine_type(self);
			write!(
				f,
				"array[1..{}] of {}{ty}: {ident}",
				arr.contents.len(),
				if is_var { "var " } else { "" }
			)?;
			if output_map.contains_key(&ident) {
				write!(f, " ::output_array([1..{}])", arr.contents.len())?;
			}
			if arr.defined {
				write!(f, " ::is_defined_var")?;
			}
			if arr.introduced {
				write!(f, " ::var_is_introduced")?;
			}
			for ann in &arr.ann {
				write!(f, " {ann}")?
			}
			write!(f, " = [")?;
			let mut first = true;
			for v in &arr.contents {
				if !first {
					write!(f, ", ")?;
				}
				write!(f, "{v}")?;
				first = false;
			}
			writeln!(f, "];")?
		}
		for c in &self.constraints {
			writeln!(f, "constraint {c};")?;
		}
		writeln!(f, "{};", self.solve)
	}
}

#[cfg(test)]
mod tests {
	use std::{
		fs::File,
		io::{BufReader, Read},
		path::Path,
	};

	use expect_test::ExpectFile;
	use ustr::Ustr;

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

	#[test]
	fn test_ident_no_copy() {
		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		rdr.read_to_string(&mut content).unwrap();

		let fzn: FlatZinc<&str> = serde_json::from_str(&content).unwrap();
		expect_test::expect_file!["../corpus/documentation_example.debug.txt"].assert_debug_eq(&fzn)
	}

	#[test]
	fn test_ident_interned() {
		let rdr = BufReader::new(
			File::open(Path::new("./corpus/documentation_example.fzn.json")).unwrap(),
		);
		let fzn: FlatZinc<Ustr> = serde_json::from_reader(rdr).unwrap();
		expect_test::expect_file!["../corpus/documentation_example.debug_ustr.txt"]
			.assert_debug_eq(&fzn)
	}

	#[test]
	fn test_print_flatzinc() {
		let mut rdr = BufReader::new(
			File::open(Path::new("./corpus/documentation_example.fzn.json")).unwrap(),
		);
		let mut content = String::new();
		rdr.read_to_string(&mut content).unwrap();

		let fzn: FlatZinc<&str> = serde_json::from_str(&content).unwrap();
		expect_test::expect_file!["../corpus/documentation_example.fzn"].assert_eq(&fzn.to_string())
	}
}
