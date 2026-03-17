//! Serialization of the FlatZinc data format
//!
//! FlatZinc is the language in which data and solver specific constraint models
//! are produced by the [MiniZinc](https://www.minizinc.org) compiler. This
//! crate implements the FlatZinc serialization format as described in the
//! [Interfacing Solvers to
//! FlatZinc](https://www.minizinc.org/doc-latest/en/fzn-spec.html#specification-of-flatzinc-json)
//! section of the MiniZinc reference manual. It supports both the JSON-based
//! FlatZinc representation, via [serde](https://serde.rs), and the older
//! textual `.fzn` format. For the JSON format, we suggest using
//! [`serde_json`](https://crates.io/crates/serde_json) with the specification
//! in this crate to parse the FlatZinc JSON files produced by the MiniZinc
//! compiler.
//!
//! # Feature Flags
//!
//! - `serde` (default): enables JSON serialization and deserialization support
//!   via the [`serde`](https://serde.rs) crate.
//! - `fzn`: enables parsing of the original `.fzn` text format via
//!   [`winnow`](https://crates.io/crates/winnow).
//!
//! # Getting Started
//!
//! For the default JSON-based workflow, install `flatzinc-serde` and
//! `serde_json` for your package:
//!
//! ```bash
//! cargo add flatzinc-serde serde_json
//! ```
//!
//! If you disable the default `serde` feature and only use the older textual
//! `.fzn` support, `serde_json` is not required.
//!
//! Once these dependencies have been installed to your crate, you can
//! deserialize a FlatZinc JSON file as follows:
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! # use flatzinc_serde::FlatZinc;
//! # use std::{fs::File, io::BufReader, path::Path};
//! # let path = Path::new("./corpus/json/documentation_example.fzn.json");
//! // let path = Path::new("/lorem/ipsum/model.fzn.json");
//! let rdr = BufReader::new(File::open(path).unwrap());
//! let fzn: FlatZinc = serde_json::from_reader(rdr).unwrap();
//! // ... process FlatZinc ...
//! # }
//! ```
//!
//! The older textual `.fzn` format is also supported when the `fzn` feature is
//! enabled:
//!
//! ```
//! # #[cfg(feature = "fzn")] {
//! # use flatzinc_serde::FlatZinc;
//! # use std::{fs::File, io::BufReader, path::Path};
//! # let path = Path::new("./corpus/fzn/documentation_example.fzn");
//! // let path = Path::new("/lorem/ipsum/model.fzn");
//! let rdr = BufReader::new(File::open(path).unwrap());
//! let fzn: FlatZinc = FlatZinc::from_fzn(rdr).unwrap();
//! // ... process FlatZinc ...
//! # }
//! ```
//!
//! To serialize a FlatZinc JSON value, you can use the usual `serde_json`
//! APIs:
//!
//! ```
//! # #[cfg(feature = "serde")] {
//! # use flatzinc_serde::FlatZinc;
//! let fzn = FlatZinc::<String>::default();
//! // ... create  solver constraint model ...
//! let json_str = serde_json::to_string(&fzn).unwrap();
//! # }
//! ```
//! Note that `serde_json::to_writer`, using a buffered file writer, would be
//! preferred when writing larger FlatZinc files.
//!
//! To serialize a FlatZinc value to the older textual `.fzn` format, use its
//! [`Display`] implementation:
//!
//! ```
//! # use flatzinc_serde::FlatZinc;
//! let fzn = FlatZinc::<String>::default();
//! let fzn_text = fzn.to_string();
//! ```
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
#![warn(variant_size_differences)]

#[cfg(feature = "fzn")]
mod fzn;
#[cfg(feature = "serde")]
mod serde;

use std::{collections::BTreeMap, fmt::Display};
#[cfg(feature = "fzn")]
use std::{fmt::Debug, str::FromStr};

#[cfg(feature = "serde")]
use ::serde::{Deserialize, Serialize};
pub use rangelist::RangeList;

#[cfg(feature = "fzn")]
pub use crate::fzn::FznParseError;

/// Additional information provided in a standardized format for declarations,
/// constraints, or solve objectives
///
/// In MiniZinc annotations can both be added explicitly in the model, or can be
/// added during compilation process.
///
/// Note that annotations are generally defined either in the MiniZinc standard
/// library or in a solver's redefinition library. Solvers are encouraged to
/// rewrite annotations in their redefinitions library when required.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, PartialEq, Debug)]
pub enum Annotation<Identifier = String> {
	/// Atom annotation (i.e., a single `Identifier`)
	Atom(Identifier),
	/// Call annotation
	Call(AnnotationCall<Identifier>),
}

/// The argument type associated with [`AnnotationCall`]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, PartialEq, Debug)]
pub enum AnnotationArgument<Identifier = String> {
	/// Sequence of [`Literal`]s
	Array(Vec<AnnotationLiteral<Identifier>>),
	/// Singular argument
	Literal(AnnotationLiteral<Identifier>),
}

/// An object depicting an annotation in the form of a call
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename = "annotation_call"))]
#[derive(Clone, PartialEq, Debug)]
pub struct AnnotationCall<Identifier = String> {
	/// Identifier of the constraint predicate
	pub id: Identifier,
	/// Arguments of the constraint
	pub args: Vec<AnnotationArgument<Identifier>>,
}

///Literal values as arguments to [`AnnotationCall`]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, PartialEq, Debug)]
pub enum AnnotationLiteral<Identifier = String> {
	/// Basic FlatZinc literal (including annotation identifiers).
	BaseLiteral(Literal<Identifier>),
	/// An annotation call object.
	Annotation(AnnotationCall<Identifier>),
}

/// The argument type associated with [`Constraint`]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, PartialEq, Debug)]
pub enum Argument<Identifier = String> {
	/// Sequence of [`Literal`]s
	Array(Vec<Literal<Identifier>>),
	/// Literal
	Literal(Literal<Identifier>),
}

/// A definition of a named array literal in FlatZinc
///
/// FlatZinc Arrays are a simple (one-dimensional) sequence of [`Literal`]s.
/// These values are stored as the [`Array::contents`] member. Additional
/// information, in the form of [`Annotation`]s, from the MiniZinc model is
/// stored in [`Array::ann`] when present. When [`Array::defined`] is set to
/// `true`, then
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename = "array"))]
#[derive(Clone, PartialEq, Debug)]
pub struct Array<Identifier = String> {
	/// The values stored within the array literal
	#[cfg_attr(feature = "serde", serde(rename = "a"))]
	pub contents: Vec<Literal<Identifier>>,
	#[cfg_attr(
		feature = "serde",
		serde(default, skip_serializing_if = "Vec::is_empty")
	)]
	/// List of annotations
	pub ann: Vec<Annotation<Identifier>>,
	#[cfg_attr(
		feature = "serde",
		serde(default, skip_serializing_if = "serde::is_false")
	)]
	/// This field is set to `true` when there is a constraint that has been marked as
	/// defining this array.
	pub defined: bool,
	#[cfg_attr(
		feature = "serde",
		serde(default, skip_serializing_if = "serde::is_false")
	)]
	/// This field is set to `true` when the array has been introduced by the
	/// MiniZinc compiler, rather than being explicitly defined at the top-level
	/// of the MiniZinc model.
	pub introduced: bool,
}

/// An object depicting a constraint
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(rename = "constraint"))]
#[derive(Clone, PartialEq, Debug)]
pub struct Constraint<Identifier = String> {
	/// Identifier of the constraint predicate
	pub id: Identifier,
	/// Arguments of the constraint
	pub args: Vec<Argument<Identifier>>,
	/// Identifier of the variable that the constraint defines
	#[cfg_attr(
		feature = "serde",
		serde(default, skip_serializing_if = "Option::is_none")
	)]
	pub defines: Option<Identifier>,
	/// List of annotations
	#[cfg_attr(
		feature = "serde",
		serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")
	)]
	pub ann: Vec<Annotation<Identifier>>,
}

/// The structure depicting a FlatZinc instance
///
/// FlatZinc is (generally) a format produced by the MiniZinc compiler as a
/// result of instantiating the parameter variables of a MiniZinc model and
/// generating a solver-specific equisatisfiable model.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct FlatZinc<
	Identifier = String,
	VarMap = BTreeMap<Identifier, Variable<Identifier>>,
	ArrayMap = BTreeMap<Identifier, Array<Identifier>>,
> {
	/// A mapping from decision variable `Identifier` to their definitions
	#[cfg_attr(
		feature = "serde",
		serde(
			default,
			bound(
				serialize = "Identifier: Serialize, for<'a> &'a VarMap: IntoIterator<Item = (&'a Identifier, &'a Variable<Identifier>)>",
				deserialize = "Identifier: Deserialize<'de>, VarMap: FromIterator<(Identifier, Variable<Identifier>)>"
			),
			deserialize_with = "serde::deserialize_key_value_object",
			serialize_with = "serde::serialize_key_value_object"
		)
	)]
	pub variables: VarMap,
	/// A mapping from array `Identifier` to their definitions
	#[cfg_attr(
		feature = "serde",
		serde(
			default,
			bound(
				serialize = "Identifier: Serialize, for<'a> &'a ArrayMap: IntoIterator<Item = (&'a Identifier, &'a Array<Identifier>)>",
				deserialize = "Identifier: Deserialize<'de>, ArrayMap: FromIterator<(Identifier, Array<Identifier>)>"
			),
			deserialize_with = "serde::deserialize_key_value_object",
			serialize_with = "serde::serialize_key_value_object"
		)
	)]
	pub arrays: ArrayMap,
	/// A list of (solver-specific) constraints, that must be satisfied in a solution.
	#[cfg_attr(feature = "serde", serde(default))]
	pub constraints: Vec<Constraint<Identifier>>,
	/// A list of all identifiers for which the solver must produce output for each solution
	#[cfg_attr(feature = "serde", serde(default))]
	pub output: Vec<Identifier>,
	/// A specification of the goal of solving the FlatZinc instance.
	pub solve: SolveObjective<Identifier>,
	/// The version of the FlatZinc serialization specification used
	#[cfg_attr(
		feature = "serde",
		serde(default, skip_serializing_if = "String::is_empty")
	)]
	pub version: String,
}

// /// A name used to refer to an [`Array`], function, or [`Variable`]
// pub type Identifier = String;

/// Literal values
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Clone, PartialEq, Debug)]
pub enum Literal<Identifier = String> {
	/// Integer value
	Int(i64),
	/// Floating point value
	Float(f64),
	/// Identifier, i.e., reference to an [`Array`] or [`Variable`]
	Identifier(Identifier),
	/// Boolean value
	Bool(bool),
	#[cfg_attr(
		feature = "serde",
		serde(
			serialize_with = "serde::serialize_encapsulate_set",
			deserialize_with = "serde::deserialize_encapsulated_set"
		)
	)]
	/// Set of integers, represented as a list of integer ranges
	IntSet(RangeList<i64>),
	#[cfg_attr(
		feature = "serde",
		serde(
			serialize_with = "serde::serialize_encapsulate_set",
			deserialize_with = "serde::deserialize_encapsulated_set"
		)
	)]
	/// Set of floating point values, represented as a list of floating point
	/// ranges
	FloatSet(RangeList<f64>),
	#[cfg_attr(
		feature = "serde",
		serde(
			serialize_with = "serde::serialize_encapsulate_string",
			deserialize_with = "serde::deserialize_encapsulated_string"
		)
	)]
	/// String value
	String(String),
}

/// Goal of solving a FlatZinc instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Method<Identifier = String> {
	#[default]
	/// Find any solution.
	Satisfy,
	/// Find the solution with the lowest value for the given objective.
	Minimize(Literal<Identifier>),
	/// Find the solution with the highest value for the given objective.
	Maximize(Literal<Identifier>),
}

/// A specification of objective of a FlatZinc instance
#[derive(Clone, PartialEq, Debug)]
pub struct SolveObjective<Identifier = String> {
	/// The method expected to be used for solving the instance.
	pub method: Method<Identifier>,
	/// A list of annotations from the solve statement in the MiniZinc model
	///
	/// Note that this includes the search annotations if they are present in the
	/// model.
	pub ann: Vec<Annotation<Identifier>>,
}

/// Used to signal the type of (decision) [`Variable`]
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
	/// Boolean decision variable
	Bool,
	/// Integer decision variable
	Int(Option<RangeList<i64>>),
	/// Floating point decision variable
	Float(Option<RangeList<f64>>),
	/// Integer set decision variable
	IntSet(Option<RangeList<i64>>),
}

/// The definition of a decision variable
#[derive(Clone, PartialEq, Debug)]
pub struct Variable<Identifier = String> {
	/// The type of the decision variable, and set of potential values  from which
	/// the decision variable must take its value in a solution, i.e. its domain.
	///
	/// If domain has the value `None`, then all values of the decision variable's
	/// `Type` are allowed in a solution.
	pub ty: Type,
	/// The “right hand side” of the variable, i.e., its value or alias to another
	/// variable
	pub value: Option<Literal<Identifier>>,
	/// A list of annotations
	pub ann: Vec<Annotation<Identifier>>,
	/// This field is set to `true` when there is a constraint that has been marked as
	/// defining this variable.
	pub defined: bool,
	/// This field is set to `true` when the variable has been introduced by the
	/// MiniZinc compiler, rather than being explicitly defined at the top-level
	/// of the MiniZinc model.
	pub introduced: bool,
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

impl<Idenfier: Display> Display for AnnotationArgument<Idenfier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnnotationArgument::Array(arr) => {
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
			AnnotationArgument::Literal(lit) => write!(f, "{lit}"),
		}
	}
}

impl<Identifier: Display> Display for AnnotationCall<Identifier> {
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
		write!(f, ")")
	}
}

impl<Idenfier: Display> Display for AnnotationLiteral<Idenfier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AnnotationLiteral::BaseLiteral(lit) => write!(f, "{lit}"),
			AnnotationLiteral::Annotation(ann) => write!(f, "{ann}"),
		}
	}
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

impl<Identifier: Ord> Array<Identifier> {
	/// Heuristic to determine the type of the array
	fn determine_type(&self, fzn: &FlatZinc<Identifier>) -> (&str, bool) {
		let ty = match self.contents.first().unwrap() {
			Literal::Int(_) => "int",
			Literal::Float(_) => "float",
			Literal::Identifier(ident) => fzn.variables[ident].ty.base_name(),
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

impl<Identifier: Display> Display for Constraint<Identifier> {
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
		if let Some(defines) = &self.defines {
			write!(f, " ::defines_var({defines})")?
		}
		for a in &self.ann {
			write!(f, " {a}")?
		}
		Ok(())
	}
}

#[cfg(feature = "fzn")]
impl<Identifier, VarMap, ArrayMap> FlatZinc<Identifier, VarMap, ArrayMap>
where
	Identifier: Clone + Debug + FromStr,
	<Identifier as FromStr>::Err: Display,
	VarMap: FromIterator<(Identifier, Variable<Identifier>)>,
	ArrayMap: FromIterator<(Identifier, Array<Identifier>)>,
{
	/// Parse a `.fzn` source into a [`FlatZinc`] instance.
	pub fn from_fzn(source: impl std::io::BufRead) -> Result<Self, FznParseError> {
		fzn::parse(source)
	}
}

impl<Identifier, VarMap, ArrayMap> Default for FlatZinc<Identifier, VarMap, ArrayMap>
where
	VarMap: Default,
	ArrayMap: Default,
{
	fn default() -> Self {
		Self {
			variables: Default::default(),
			arrays: Default::default(),
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
			write!(f, "var {}", var.ty)?;
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
				write!(f, " = {val}")?
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

impl<Identifier: Display> Display for Literal<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Literal::Int(i) => write!(f, "{i}"),
			Literal::Float(x) => write!(f, "{x:?}"),
			Literal::Identifier(ident) => write!(f, "{ident}"),
			Literal::Bool(b) => write!(f, "{b}"),
			Literal::IntSet(is) => write!(f, "{is}"),
			Literal::FloatSet(fs) => write!(f, "{fs}"),
			Literal::String(s) => write!(f, "{s:?}"),
		}
	}
}

impl<Identifier: Display> Display for Method<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Method::Satisfy => write!(f, "satisfy"),
			Method::Minimize(objective) => write!(f, "minimize {objective}"),
			Method::Maximize(objective) => write!(f, "maximize {objective}"),
		}
	}
}

impl<Identifier> Default for SolveObjective<Identifier> {
	fn default() -> Self {
		Self {
			method: Default::default(),
			ann: Vec::new(),
		}
	}
}

impl<Identifier: Display> Display for SolveObjective<Identifier> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "solve ")?;
		for a in &self.ann {
			write!(f, "{a} ")?;
		}
		write!(f, "{}", self.method)
	}
}

impl Type {
	fn base_name(&self) -> &'static str {
		match self {
			Type::Bool => "bool",
			Type::Int(_) => "int",
			Type::Float(_) => "float",
			Type::IntSet(_) => "set of int",
		}
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Bool => write!(f, "bool"),
			Type::Int(Some(domain)) => write!(f, "{domain}"),
			Type::Int(None) => write!(f, "int"),
			Type::Float(Some(domain)) => write!(f, "{domain}"),
			Type::Float(None) => write!(f, "float"),
			Type::IntSet(Some(domain)) => write!(f, "set of {domain}"),
			Type::IntSet(None) => write!(f, "set of int"),
		}
	}
}
