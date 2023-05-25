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
pub mod value;

use db::{CompilerDatabase, Inputs};
use diagnostics::{InternalError, ShackleError};
use file::InputFile;
use rustc_hash::FxHashMap;
use serde_json::Map;
use ty::{Ty, TyData};
use value::Enum;

use std::{
	collections::BTreeMap, fmt::Display, io::Write, path::PathBuf, sync::Arc, time::Duration,
};

use crate::{
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
	value::Value,
};

// Export OptType enumeration used in [`Type`]
pub use ty::OptType;

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

		let input_types = self
			.db
			.input_type_map()
			.iter()
			.map(|(ident, ty)| match Type::from_compiler(&self.db, *ty) {
				Ok(nty) => Ok((ident.0.value(&self.db), nty)),
				Err(e) => Err(e),
			})
			.collect::<Result<FxHashMap<_, _>, _>>()?;

		let output_types = self
			.db
			.output_type_map()
			.iter()
			.map(|(ident, ty)| match Type::from_compiler(&self.db, *ty) {
				Ok(nty) => Ok((ident.0.value(&self.db), nty)),
				Err(e) => Err(e),
			})
			.collect::<Result<FxHashMap<_, _>, _>>()?;

		Ok(Program {
			db: self.db,
			slv: slv.clone(),
			code: prg_model,
			_input_types: input_types,
			_input_data: FxHashMap::default(),
			_output_types: output_types,
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
	// FIXME: CompilerDatabase should (probably) not be part of Program anymore
	db: CompilerDatabase,
	code: Arc<thir::Model>,
	slv: Solver,
	// Model instance data
	_input_types: FxHashMap<String, Type>,
	_input_data: FxHashMap<String, Value>,

	_output_types: FxHashMap<String, Type>,
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

/// An type of the input or output of a Shackle model
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
	/// Boolean scalar
	Boolean(OptType),
	/// Integer scalar
	Integer(OptType),
	/// Float scalar
	Float(OptType),
	/// Enumerated type scalar
	Enum(OptType, Option<Arc<Enum>>),

	/// String scalar
	String(OptType),
	/// Annotation scalar
	Annotation(OptType),

	/// Array type
	Array {
		/// Whether the array is optional
		opt: OptType,
		/// Type used for indexing
		dim: Box<[Type]>,
		/// Type of the element
		element: Box<Type>,
	},
	/// Set type
	Set(OptType, Box<Type>),
	/// Tuple type
	Tuple(OptType, Box<[Type]>),
	/// Record type
	Record(OptType, Box<[(String, Type)]>),
}

impl Type {
	fn from_compiler(db: &CompilerDatabase, value: Ty) -> Result<Self, ShackleError> {
		let data = value.lookup(db);
		match data {
			TyData::Boolean(_, opt) => Ok(Type::Boolean(opt)),
			TyData::Integer(_, opt) => Ok(Type::Integer(opt)),
			TyData::Float(_, opt) => Ok(Type::Float(opt)),
			TyData::Enum(_, opt, _) => Ok(Type::Enum(opt, None)), // TODO: Fix None
			TyData::String(opt) => Ok(Type::String(opt)),
			TyData::Annotation(opt) => Ok(Type::Annotation(opt)),
			TyData::Array { opt, dim, element } => {
				let elem = Type::from_compiler(db, element)?;
				let index_conv = |nty| -> Result<Type, ShackleError> {
					match nty {
						TyData::Integer(ty::VarType::Par, OptType::NonOpt) => {
							Ok(Type::Integer(OptType::NonOpt))
						}
						TyData::Enum(ty::VarType::Par, OptType::NonOpt, _) => {
							Ok(Type::Enum(OptType::NonOpt, None)) // TODO: Fix None
						}
						_ => Err(InternalError::new(format!(
							"Unexpected index set type on user facing type {:?}",
							nty
						))
						.into()),
					}
				};
				let ndim = match dim.lookup(db) {
					TyData::Tuple(OptType::NonOpt, li) => li
						.iter()
						.map(|ty| index_conv(ty.lookup(db)))
						.collect::<Result<Vec<_>, _>>()?,
					x => {
						let nty = index_conv(x)?;
						vec![nty]
					}
				};
				Ok(Type::Array {
					opt,
					dim: ndim.into_boxed_slice(),
					element: Box::new(elem),
				})
			}
			TyData::Set(_, opt, elem) => {
				Ok(Type::Set(opt, Box::new(Type::from_compiler(db, elem)?)))
			}
			TyData::Tuple(opt, li) => Ok(Type::Tuple(
				opt,
				li.iter()
					.map(|ty| Type::from_compiler(db, *ty))
					.collect::<Result<Vec<_>, _>>()?
					.into_boxed_slice(),
			)),
			TyData::Record(opt, li) => Ok(Type::Record(
				opt,
				li.iter()
					.map(|(name, ty)| match Type::from_compiler(db, *ty) {
						Ok(nty) => Ok((name.value(db), nty)),
						Err(e) => Err(e),
					})
					.collect::<Result<Vec<_>, _>>()?
					.into_boxed_slice(),
			)),
			_ => Err(InternalError::new(format!(
				"Unable to create user facing type from {:?}",
				data
			))
			.into()),
		}
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
mod tests {}
