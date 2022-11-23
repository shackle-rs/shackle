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
use serde_json::Map;

use std::{
	collections::BTreeMap,
	fmt::{self, Display},
	io::Write,
	ops::{Index, RangeInclusive},
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
	Array(Vec<RangeInclusive<i64>>, Vec<Value>),
	/// A set of values
	/// All values are of the same type and only occur once
	Set(SetValue),
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
			Value::Boolean(v) => write!(f, "{}", v),
			Value::Integer(v) => write!(f, "{}", v),
			Value::Float(v) => write!(f, "{}", v),
			Value::String(v) => write!(f, "{:?}", v),
			Value::Enum(v) => write!(f, "{}", v),
			Value::Array(idx, v) => {
				let mut ii: Vec<i64> = idx.iter().map(|r| *r.start()).collect();
				let incr = |ii: &mut Vec<i64>| {
					let mut i = ii.len() - 1;
					ii[i] += 1;
					while !idx[i].contains(&ii[i]) && i > 0 {
						ii[i] = *idx[i].start();
						i = i.wrapping_sub(1);
						ii[i] += 1;
					}
				};
				let mut first = true;
				write!(f, "[")?;
				for x in v {
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
					incr(&mut ii);
				}
				write!(f, "]")
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

/// Different representations used to represent sets in [`Value`]
#[derive(Debug, Clone, PartialEq)]
pub enum SetValue {
	/// List of (unique) Value elements
	SetList(Vec<Value>),
	/// Sorted list of non-overlapping inclusive integer ranges
	IntRangeList(Vec<RangeInclusive<i64>>),
	/// Sorted list of non-overlapping inclusive floating point ranges
	FloatRangeList(Vec<RangeInclusive<f64>>),
}

impl Display for SetValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SetValue::SetList(v) => {
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
			SetValue::IntRangeList(ranges) => {
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
			SetValue::FloatRangeList(ranges) => {
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
impl Index<&str> for Record {
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
