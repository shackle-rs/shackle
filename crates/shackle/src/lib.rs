//! Shackle library

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

pub mod arena;
pub mod constants;
pub mod db;
pub mod diagnostics;
pub mod dzn;
pub mod file;
pub mod hir;
mod legacy;
pub mod mir;
pub mod refmap;
pub mod syntax;
pub mod thir;
pub mod ty;
pub mod utils;

use db::{CompilerDatabase, Inputs};
use diagnostics::ShackleError;
use file::InputFile;
use itertools::Itertools;
use serde_json::Map;

use std::{
	collections::BTreeMap,
	fmt::{self, Display},
	io::Write,
	ops::RangeInclusive,
	path::PathBuf,
	sync::Arc,
	time::Duration,
};

use crate::{
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

/// Shackle error type
pub type Error = ShackleError;
/// Result type for Shackle operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use diagnostics::Warning;

/// Structure used to build a shackle model
pub struct Model {
	db: CompilerDatabase,
}

impl Model {
	/// Create a Model from the file at the given path
	pub fn from_file(path: PathBuf) -> Model {
		let mut db = db::CompilerDatabase::default();
		db.set_input_files(Arc::new(vec![InputFile::Path(path)]));
		Model { db }
	}

	/// Create a Model from the given string
	pub fn from_string(m: String) -> Model {
		let mut db = db::CompilerDatabase::default();
		db.set_input_files(Arc::new(vec![InputFile::ModelString(m)]));
		Model { db }
	}

	/// Check whether a model contains any (non-runtime) errors
	pub fn check(&self, _slv: &Solver, _data: &[PathBuf], _complete: bool) -> Vec<Error> {
		// TODO: Check data files
		self.db
			.run_hir_phase()
			.map(|_| Vec::new())
			.unwrap_or_else(|e| e.iter().cloned().collect())
	}

	/// Compile current model into a Program that can be used by the Shackle interpreter
	pub fn compile(self, slv: &Solver) -> Result<Program> {
		let errors = self.check(slv, &[], false);
		if !errors.is_empty() {
			return Err(ShackleError::try_from(errors).unwrap());
		}
		let prg_model = self.db.final_thir()?;
		Ok(Program {
			db: self.db,
			slv: slv.clone(),
			code: prg_model,
			enable_stats: false,
			time_limit: None,
		})
	}
}

/// Solver specification to compile and solve Model instances.
#[derive(Clone)]
pub struct Solver {
	// TODO: actual information (Load from solver configurations)
	/// Identifier of the solver
	ident: String,
}

impl Solver {
	/// Lookup a solver specification in default locations that best matches the given identifier
	pub fn lookup(ident: &str) -> Option<Solver> {
		Some(Solver {
			ident: ident.into(),
		})
	}
}

/// Structure to capture the result of succesful compilation of a Model object
pub struct Program {
	// FIXME: CompilerDatabase should not be part of Program anymore
	db: CompilerDatabase,
	slv: Solver,
	code: Arc<thir::Model>,
	// run() options
	enable_stats: bool,
	time_limit: Option<Duration>,
}

/// Status of running and solving a Program
#[derive(Debug, Clone)]
pub enum Status {
	/// No solutions exist
	Infeasible,
	/// A solution has been found
	Satisfied,
	/// A solution with the best possible objective value has been found
	Optimal,
	/// All possible solutions have been found
	AllSolutions,
	/// No result reached within the given limits
	Unknown,
	/// An error occurred
	Err(ShackleError),
}

/// Value types that can be part of a Solution
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
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
	// FIXME this should probably have the actual structuring of enumerated types
	Enum(String),
	/// An array of values
	/// All values are of the same type
	Array(Array),
	/// A set of values
	/// All values are of the same type and only occur once
	Set(Set),
	/// A tuple of values
	Tuple(Vec<Value>),
	/// A record of values
	Record(Record),
}

/// Whether an value is negative or positive
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Polarity {
	/// Positive
	Pos,
	/// Negative
	Neg,
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Absent => write!(f, "<>"),
			Value::Infinity(p) => {
				if p == &Polarity::Neg {
					write!(f, "-")?;
				};
				write!(f, "∞")
			}
			Value::Boolean(v) => write!(f, "{v}"),
			Value::Integer(v) => write!(f, "{v}"),
			Value::Float(v) => write!(f, "{v}"),
			Value::String(v) => write!(f, "{:?}", v),
			Value::Enum(v) => write!(f, "{v}"),
			Value::Array(arr) => {
				write!(f, "{arr}")
			}
			Value::Set(v) => {
				write!(f, "{v}")
			}
			Value::Tuple(v) => {
				let mut first = true;
				write!(f, "(")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}", x)?;
					first = false;
				}
				write!(f, ")")
			}
			Value::Record(rec) => {
				write!(f, "{rec}")
			}
		}
	}
}

/// Representation of an (multidimensional) indexed array
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array {
	indexes: Box<[Index]>,
	members: Box<[Value]>,
}

impl Array {
	/// Create a new array that contains the values in `elements` indexes by the given index sets
	pub fn new(indexes: Vec<Index>, elements: Vec<Value>) -> Self {
		assert_eq!(
			indexes.iter().map(|i| i.len()).product::<usize>(),
			elements.len(),
			"the size suggested by the index sets {} does not match the number of elements {}",
			indexes.iter().map(|i| i.len()).product::<usize>(),
			elements.len()
		);
		Self {
			indexes: indexes.into_boxed_slice(),
			members: elements.into_boxed_slice(),
		}
	}
}

impl std::ops::Index<&[Value]> for Array {
	type Output = Value;
	fn index(&self, index: &[Value]) -> &Self::Output {
		let mut idx = 0;
		let mut mult = 1;
		for (ii, ctx) in index.iter().zip_eq(self.indexes.iter()) {
			idx *= mult;
			match &ctx {
				&Index::Integer(r) => {
					if let Value::Integer(ii) = ii {
						assert!(
							r.contains(ii),
							"index out of bounds: the index set is {}..={} but the index is {ii}",
							r.start(),
							r.end()
						);
						idx += (ii - r.start()) as usize;
					} else {
						panic!("incorrect index type: using {ii} for an integer index")
					}
				}
			}
			mult *= ctx.len();
		}
		&self.members[idx]
	}
}

impl Display for Array {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let it = self
			.indexes
			.iter()
			.map(|ii| match ii {
				Index::Integer(ii) => ii.clone(),
			})
			.multi_cartesian_product()
			.zip_eq(self.members.iter());
		let mut first = true;
		write!(f, "[")?;
		for (ii, x) in it {
			if !first {
				write!(f, ", ")?;
			}
			match &ii[..] {
				[i] => write!(f, "{i}: "),
				ii => {
					write!(f, "(")?;
					let mut tup_first = true;
					for i in ii {
						if !tup_first {
							write!(f, ",")?;
						}
						write!(f, "{i}")?;
						tup_first = false;
					}
					write!(f, "): ")
				}
			}?;
			write!(f, "{x}")?;
			first = false;
		}
		write!(f, "]")
	}
}

/// Representation of Array indexes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
	/// Closed integer range index
	Integer(RangeInclusive<i64>),
	// Enum(Arc<Enum>),
}

impl Index {
	/// Returns the cardinality of the index set
	pub fn len(&self) -> usize {
		match self {
			Index::Integer(r) => {
				if r.is_empty() {
					0
				} else {
					(r.end() - r.start()) as usize + 1
				}
			}
		}
	}

	/// Returns whether the index set contains any members
	pub fn is_empty(&self) -> bool {
		match &self {
			Index::Integer(r) => r.is_empty(),
		}
	}
}

/// Member declaration of an enumerated type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {}

/// Different representations used to represent sets in [`Value`]
#[derive(Debug, Clone, PartialEq)]
pub enum Set {
	/// List of (unique) Value elements
	SetList(Vec<Value>),
	/// Sorted list of non-overlapping inclusive integer ranges
	IntRangeList(Vec<RangeInclusive<i64>>),
	/// Sorted list of non-overlapping inclusive floating point ranges
	FloatRangeList(Vec<RangeInclusive<f64>>),
}

impl Display for Set {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Set::SetList(v) => {
				if v.is_empty() {
					return write!(f, "∅");
				}
				let mut first = true;
				write!(f, "{{")?;
				for x in v {
					if !first {
						write!(f, ", ")?;
					}
					write!(f, "{}", x)?;
					first = false;
				}
				write!(f, "}}")
			}
			Set::IntRangeList(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				let mut first = true;
				for range in ranges {
					if !first {
						write!(f, " union ")?;
					}
					write!(f, "{}..{}", range.start(), range.end())?;
					first = false;
				}
				Ok(())
			}
			Set::FloatRangeList(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				let mut first = true;
				for range in ranges {
					if !first {
						write!(f, " union ")?;
					}
					write!(f, "{}..{}", range.start(), range.end())?;
					first = false;
				}
				Ok(())
			}
		}
	}
}

/// A value of a record type
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Record {
	// fields are hidden to possibly replace inner implementation in the future
	fields: Vec<(Arc<String>, Value)>,
}

impl FromIterator<(Arc<String>, Value)> for Record {
	fn from_iter<T: IntoIterator<Item = (Arc<String>, Value)>>(iter: T) -> Self {
		let mut fields: Vec<(Arc<String>, Value)> = iter.into_iter().collect();
		fields.sort_by(|(k1, _), (k2, _)| k1.as_str().cmp(k2.as_str()));
		Self { fields }
	}
}
impl<'a> IntoIterator for &'a Record {
	type Item = &'a (Arc<String>, Value);
	type IntoIter = std::slice::Iter<'a, (Arc<String>, Value)>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.fields.iter()
	}
}
impl std::ops::Index<&str> for Record {
	type Output = Value;

	fn index(&self, index: &str) -> &Self::Output {
		for (k, v) in &self.fields {
			if k.as_str() == index {
				return v;
			}
		}
		panic!("no entry found for key");
	}
}

impl Display for Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut first = true;
		write!(f, "(")?;
		for (k, v) in &self.fields {
			if !first {
				write!(f, ", ")?;
			}
			write!(f, "{}: {}", *k, v)?;
			first = false;
		}
		write!(f, ")")
	}
}

/// Intermediate messages emitted by shackle in processing and solving a program
#[derive(Debug)]
pub enum Message<'a> {
	/// (Intermediate) solution emitted in the process
	Solution(BTreeMap<String, Value>),
	/// Statistical information of the shackle or solving process
	Statistic(&'a Map<String, serde_json::Value>),
	/// Trace messages emitted during the shackle process
	Trace(&'a str),
	/// Warning messages emitted by shackle or the solver
	Warning(&'a str),
}

impl<'a> Display for Message<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Message::Solution(sol) => {
				for (name, val) in sol {
					writeln!(f, "{} = {};", name, val)?;
				}
				writeln!(f, "----------")
			}
			Message::Statistic(map) => {
				for (name, val) in *map {
					writeln!(f, "%%%mzn-stat: {}={}", name, val)?;
				}
				writeln!(f, "%%%mzn-stat-end")
			}
			Message::Trace(msg) => write!(f, "% mzn-trace: {}", msg),
			Message::Warning(msg) => write!(f, "% WARNING: {}", msg),
		}
	}
}

impl Program {
	/// Set whether messages containing statistical information regarding running the program should be sent
	pub fn with_statistics(mut self, stats: bool) -> Self {
		self.enable_stats = stats;
		self
	}
	/// Add the maximum duration that the run method is allowed to take before it will be canceled
	pub fn with_time_limit(mut self, dur: Duration) -> Self {
		self.time_limit = Some(dur);
		self
	}
	/// Output the [`Pogram`] using the given output interface, using the [`Write`] trait
	pub fn write<W: Write>(&self, out: &mut W) -> Result<(), std::io::Error> {
		let printer = PrettyPrinter::new_compat(&self.db, &self.code);
		out.write_all(printer.pretty_print().as_bytes())
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
