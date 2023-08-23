//! Shackle library

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

pub mod constants;
pub mod db;
pub mod diagnostics;
pub mod file;
pub mod hir;
pub mod mir;
pub mod syntax;
pub mod thir;
pub mod ty;
pub mod utils;

mod data;
mod legacy;
mod value;

use data::{
	dzn::{parse_dzn, typecheck_dzn},
	serde::SerdeFileVisitor,
	ParserVal,
};
use db::{CompilerDatabase, Inputs};
use diagnostics::{FileError, IdentifierAlreadyDefined, InternalError, ShackleError};
use file::{InputFile, SourceFile};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserializer;
use syntax::ast::{AstNode, Identifier};
use ty::{Ty, TyData};

use std::{
	ffi::OsStr,
	fmt::Display,
	io::Write,
	path::{Path, PathBuf},
	sync::Arc,
	time::Duration,
};

use crate::{
	diagnostics::UndefinedIdentifier,
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

// Export OptType enumeration used in [`Type`]
pub use ty::OptType;

/// Shackle error type
pub type Error = ShackleError;
/// Result type for Shackle operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use diagnostics::Warning;
pub use value::{Enum, Value};

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
	Tuple(OptType, Arc<[Type]>),
	/// Record type
	Record(OptType, Arc<Vec<(Arc<str>, Type)>>),
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
					.into_boxed_slice()
					.into(),
			)),
			TyData::Record(opt, li) => Ok(Type::Record(
				opt,
				li.iter()
					.map(|(name, ty)| match Type::from_compiler(db, *ty) {
						Ok(nty) => Ok((name.value(db).into(), nty)),
						Err(e) => Err(e),
					})
					.collect::<Result<Vec<_>, _>>()?
					.into(),
			)),
			_ => Err(InternalError::new(format!(
				"Unable to create user facing type from {:?}",
				data
			))
			.into()),
		}
	}

	fn is_opt(&self) -> bool {
		matches!(
			self,
			Type::Boolean(OptType::Opt)
				| Type::Integer(OptType::Opt)
				| Type::Float(OptType::Opt)
				| Type::Enum(OptType::Opt, _)
				| Type::String(OptType::Opt)
				| Type::Array {
					opt: OptType::Opt,
					dim: _,
					element: _
				} | Type::Set(OptType::Opt, _)
				| Type::Tuple(OptType::Opt, _)
				| Type::Record(OptType::Opt, _)
		)
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let opt_str = |opt| if opt == &OptType::Opt { "opt " } else { "" };
		match self {
			Type::Boolean(opt) => write!(f, "{}bool", opt_str(opt)),
			Type::Integer(opt) => write!(f, "{}int", opt_str(opt)),
			Type::Float(opt) => write!(f, "{}float", opt_str(opt)),
			Type::Enum(_opt, _e) => todo!(),
			Type::String(opt) => write!(f, "{}string", opt_str(opt)),
			Type::Annotation(opt) => write!(f, "{}ann", opt_str(opt)),
			Type::Array { opt, dim, element } => {
				write!(
					f,
					"{}array[{}] of {}",
					opt_str(opt),
					dim.iter().format(", "),
					element
				)
			}
			Type::Set(opt, element) => write!(f, "{}set of {}", opt_str(opt), element),
			Type::Tuple(opt, members) => {
				write!(f, "{}tuple({})", opt_str(opt), members.iter().format(", "))
			}
			Type::Record(opt, members) => {
				let mty = members
					.iter()
					.format_with(", ", |(k, ty), f| f(&format_args!("{}: {}", ty, k)));
				write!(f, "{}record({})", opt_str(opt), mty)
			}
		}
	}
}

/// Intermediate messages emitted by shackle in processing and solving a program
#[derive(Debug)]
pub enum Message<'a> {
	/// (Intermediate) solution emitted in the process
	Solution(FxHashMap<&'a str, Value>),
	/// Statistical information of the shackle or solving process
	Statistic(Vec<(&'a str, serde_json::Value)>),
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
				for (name, val) in map {
					writeln!(f, "%%%mzn-stat: {}={}", name, val)?;
				}
				writeln!(f, "%%%mzn-stat-end")
			}
			Message::Trace(msg) => writeln!(f, "% mzn-trace: {}", msg),
			Message::Warning(msg) => writeln!(f, "% WARNING: {}", msg),
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

	/// Add and parse data to be used by the program.
	pub fn add_data_files<'a>(
		&mut self,
		files: impl Iterator<Item = &'a Path>,
	) -> Result<(), ShackleError> {
		// First parse all files:
		// - most values will be simple values that can be directly assigned
		// - some values will be values of enumerated types, possible part of tuples, records, or indices.
		// - files can also contain the constructors for enumerated types.
		let mut data = Vec::new();
		let mut names = FxHashSet::default();
		for f in files {
			let src = SourceFile::try_from(f)?;
			match f.extension().and_then(OsStr::to_str) {
				Some("dzn") => {
					// Parse the DZN file
					let assignments = parse_dzn(&src)?;
					data.reserve(assignments.len());
					names.reserve(assignments.len());
					// Match the parser
					for asg in assignments {
						let ident = asg.assignee().cast::<Identifier>().unwrap();
						if let Some((k, ty)) = self._input_types.get_key_value::<str>(&ident.name())
						{
							let val = typecheck_dzn(&src, &ident.name(), &asg.definition(), ty)?;
							data.push((k, ty, val));
							// Identifier already seen
							if names.contains(k) || self._input_data.contains_key(k) {
								return Err(IdentifierAlreadyDefined {
									src,
									span: asg.cst_node().as_ref().byte_range().into(),
									identifier: k.to_string(),
								}
								.into());
							}
							names.insert(k);
						} else {
							// Unknown identifier
							return Err(UndefinedIdentifier {
								src,
								span: ident.cst_node().as_ref().byte_range().into(),
								identifier: ident.name().to_string(),
							}
							.into());
						}
					}
				}
				Some("json") => {
					let assignments = serde_json::Deserializer::from_str(src.contents())
						.deserialize_map(SerdeFileVisitor(&self._input_types))
						.map_err(|_| InternalError::new("TODO: JSON parsing error"))?;

					data.reserve(assignments.len());
					names.reserve(assignments.len());
					for asg in assignments {
						// Identifier already seen
						if names.contains(asg.0) || self._input_data.contains_key(asg.0) {
							return Err(IdentifierAlreadyDefined {
								src,
								span: (0, 0).into(), // TODO: actual byte range
								identifier: asg.0.to_string(),
							}
							.into());
						}
						names.insert(asg.0);
						data.push(asg);
					}
				}
				_ => {
					return Err(FileError {
						file: f.into(),
						message: format!(
							"Attempting to read data file using unknown extension \"{}\"",
							f.display()
						),
						other: vec![],
					}
					.into());
				}
			};
		}
		// Topologically sort the constructors to allow us to resolve the dependencies
		// data.sort_by(|_a, _b| todo!());

		// Itererate between initializing the enumerated types and creating the final values for the interpreter
		for (key, ty, val) in data {
			if let ParserVal::EnumCtor(_) = val {
				todo!()
			} else {
				let _none = self
					._input_data
					.insert(key.to_string(), val.resolve_value(ty)?);
				debug_assert_eq!(_none, None);
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {}
