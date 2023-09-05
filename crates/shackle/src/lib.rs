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
	dzn::{collect_dzn_value, parse_dzn},
	serde::SerdeFileVisitor,
};
use db::{CompilerDatabase, Inputs, InternedString, Interner};
use diagnostics::{FileError, IdentifierAlreadyDefined, ShackleError};
use file::{InputFile, SourceFile};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserializer;
use syntax::ast::{AstNode, Identifier};
use thir::db::ModelIoInterface;
use ty::{Ty, TyData};
use value::EnumInner;

use std::{
	ffi::OsStr,
	fmt::Display,
	io::Write,
	ops::Deref,
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

		let ModelIoInterface {
			input,
			output,
			enums,
		} = (*self.db.model_io_interface()).clone();
		let legacy_enums = enums
			.iter()
			.filter_map(|(_, e)| {
				if e.state.lock().unwrap().deref() == &EnumInner::NoDefinition {
					Some(e.clone())
				} else {
					None
				}
			})
			.collect();

		Ok(Program {
			db: self.db,
			slv: slv.clone(),
			code: prg_model,
			input_types: input,
			input_data: FxHashMap::default(),
			enum_types: enums,
			legacy_enums,
			output_types: output,
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
	input_types: FxHashMap<Arc<str>, Type>,
	input_data: FxHashMap<Arc<str>, Value>,
	enum_types: FxHashMap<Arc<str>, Arc<Enum>>,

	// LEGACY: names of the enumerated types that have to be given to the legacy interpreter
	legacy_enums: Vec<Arc<Enum>>,

	output_types: FxHashMap<Arc<str>, Type>,
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
	Enum(OptType, Arc<Enum>),

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
	Record(OptType, Arc<[(Arc<str>, Type)]>),
}

impl Type {
	fn from_compiler<S: FnMut(InternedString) -> Arc<str>>(
		db: &dyn Interner,
		str_interner: &mut S,
		type_map: &mut FxHashMap<Ty, Type>,
		enum_map: &FxHashMap<Arc<str>, Arc<Enum>>,
		value: Ty,
	) -> Self {
		let data = value.lookup(db);
		match data {
			TyData::Boolean(_, opt) => Type::Boolean(opt),
			TyData::Integer(_, opt) => Type::Integer(opt),
			TyData::Float(_, opt) => Type::Float(opt),
			TyData::Enum(_, opt, e) => Type::Enum(opt, enum_map[&str_interner(e.name(db))].clone()),
			TyData::String(opt) => Type::String(opt),
			TyData::Annotation(opt) => Type::Annotation(opt),
			TyData::Array { opt, dim, element } => {
				let elem = Type::from_compiler(db, str_interner, type_map, enum_map, element);
				let mut index_conv = |nty| -> Type {
					match nty {
						TyData::Integer(ty::VarType::Par, OptType::NonOpt) => {
							Type::Integer(OptType::NonOpt)
						}
						TyData::Enum(ty::VarType::Par, OptType::NonOpt, e) => {
							Type::Enum(OptType::NonOpt, enum_map[&str_interner(e.name(db))].clone())
						}
						_ => {
							unreachable!("invalid index set type {:?}", nty)
						}
					}
				};
				let ndim = match dim.lookup(db) {
					TyData::Tuple(OptType::NonOpt, li) => {
						li.iter().map(|ty| index_conv(ty.lookup(db))).collect()
					}
					x => {
						vec![index_conv(x)]
					}
				};
				Type::Array {
					opt,
					dim: ndim.into_boxed_slice(),
					element: Box::new(elem),
				}
			}
			TyData::Set(_, opt, elem) => Type::Set(
				opt,
				Box::new(Type::from_compiler(
					db,
					str_interner,
					type_map,
					enum_map,
					elem,
				)),
			),
			TyData::Tuple(opt, li) => {
				let tmp = if opt == OptType::NonOpt {
					value
				} else {
					todo!()
				};
				let Type::Tuple(_, li) = (if let Some(x) = type_map.get(&tmp) {
					x
				} else {
					let mut v = Vec::with_capacity(li.len());
					for ty in li.iter() {
						v.push(Type::from_compiler(
							db,
							str_interner,
							type_map,
							enum_map,
							*ty,
						))
					}
					type_map.insert(tmp, Type::Tuple(opt, v.into_boxed_slice().into()));
					&type_map[&tmp]
				}) else {
					unreachable!()
				};
				Type::Tuple(opt, li.clone())
			}
			TyData::Record(opt, li) => {
				let tmp = if opt == OptType::NonOpt {
					value
				} else {
					todo!()
				};
				let Type::Record(_, li) = (if let Some(x) = type_map.get(&tmp) {
					x
				} else {
					let mut v = Vec::with_capacity(li.len());
					for (name, ty) in li.iter() {
						v.push((
							str_interner(*name),
							Type::from_compiler(db, str_interner, type_map, enum_map, *ty),
						))
					}
					type_map.insert(tmp, Type::Record(opt, v.into_boxed_slice().into()));
					&type_map[&tmp]
				}) else {
					unreachable!()
				};
				Type::Record(opt, li.clone())
			}
			_ => unreachable!("invalid user facing type {:?}", data),
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
			Type::Enum(opt, e) => write!(f, "{}{}", opt_str(opt), e.name()),
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
						if let Some((k, ty)) = self.input_types.get_key_value::<str>(&ident.name())
						{
							let val = collect_dzn_value(&src, &asg.definition(), ty)?;
							data.push((k, ty, val));
							// Identifier already seen
							if names.contains(k) || self.input_data.contains_key(k) {
								return Err(IdentifierAlreadyDefined {
									src,
									span: asg.cst_node().as_ref().byte_range().into(),
									identifier: k.to_string(),
								}
								.into());
							}
							names.insert(k);
						} else if let Some((k, e)) =
							self.enum_types.get_key_value::<str>(&ident.name())
						{
							let mut inner = e.state.lock().unwrap();
							if matches!(*inner, EnumInner::NoDefinition) {
								(*inner).collect_definition(&src, &asg.definition())?
							} else {
								return Err(IdentifierAlreadyDefined {
									src,
									span: asg.cst_node().as_ref().byte_range().into(),
									identifier: k.to_string(),
								}
								.into());
							}
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
						.deserialize_map(SerdeFileVisitor {
							input_types: &self.input_types,
							enum_types: &self.enum_types,
						})
						.map_err(|err| ShackleError::from_serde_json(err, &src))?;

					data.reserve(assignments.len());
					names.reserve(assignments.len());
					for asg in assignments {
						// Identifier already seen
						if names.contains(asg.0) || self.input_data.contains_key(asg.0) {
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
			let _none = self.input_data.insert(key.clone(), val.resolve_value(ty)?);
			debug_assert_eq!(_none, None);
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {}
