//! Shackle user-facing library.

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

mod data;
mod legacy;
mod value;

use std::{
	ffi::{OsStr, OsString},
	fmt::Display,
	io::Write,
	ops::Deref,
	path::{Path, PathBuf},
	sync::Arc,
	time::Duration,
};

use data::{
	dzn::{collect_dzn_value, parse_dzn},
	serde::SerdeFileVisitor,
};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserializer;
// Result type for Shackle operations
pub use shackle_diagnostics::{Error, FileError, Result, SourceFile};
use shackle_fmt::{MiniZincFormatOptions, format_str};
use shackle_hir::{
	CompilerDatabase, Db,
	constants::IdentifierRegistry,
	db::Setter,
	input::{InlineModelFile, InputFiles, NamedModelFile},
	run_hir_phase,
};
use shackle_syntax::{InputLang, ast::AstNode, minizinc::Identifier};
use shackle_thir::{db::final_thir, lower::lower_model, pretty_print::PrettyPrinter};
// Export OptType enumeration used in [`Type`]
pub use shackle_ty::OptType;
use shackle_ty::{Ty, TyData};
use shackle_utils::InternedString;
use value::EnumInner;
pub use value::{Enum, Value};

/// Shackle errors
pub mod error {
	pub use shackle_diagnostics::{Result, error::*};
}

/// Shackle warnings
pub mod warning {
	pub use shackle_diagnostics::warning::*;
}

/// Structure used to build a shackle model
pub struct Model {
	db: CompilerDatabase,
}

impl Model {
	/// Create a Model from the file at the given path
	pub fn from_file(path: PathBuf) -> Model {
		let mut db = CompilerDatabase::default();
		let model_file = NamedModelFile::new(&db, path).into();
		InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		Model { db }
	}

	/// Create a Model from the given string
	pub fn from_string(m: String, l: InputLang) -> Model {
		let mut db = CompilerDatabase::default();
		let model_file = InlineModelFile::new(&db, m, l).into();
		InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		Model { db }
	}

	/// Check whether a model contains any (non-runtime) errors
	pub fn check(&self, _slv: &Solver, _data: &[PathBuf], _complete: bool) -> Vec<Error> {
		// TODO: Check data files
		run_hir_phase(&self.db)
			.errors
			.into_iter()
			.cloned()
			.collect()
	}

	/// Compile current model into a [`Program`] that can be used by the Shackle interpreter
	pub fn compile(self, slv: &Solver) -> Result<Program> {
		let errors = self.check(slv, &[], false);
		if !errors.is_empty() {
			return Err(Error::try_from(errors).unwrap());
		}
		let ModelIoInterface {
			input,
			output,
			enums,
		} = ModelIoInterface::new(&self.db);
		let legacy_enums = enums
			.values()
			.filter_map(|e| {
				if e.state.lock().unwrap().deref() == &EnumInner::NoDefinition {
					Some(e.clone())
				} else {
					None
				}
			})
			.collect();

		let hir_result = run_hir_phase(&self.db);
		if !hir_result.errors.is_empty() {
			return Err(Error::try_from(
				hir_result.errors.into_iter().cloned().collect::<Vec<_>>(),
			)
			.unwrap());
		}
		let _ = final_thir(&self.db)?;

		Ok(Program {
			db: self.db,
			slv: slv.clone(),
			input_types: input,
			input_data: FxHashMap::default(),
			enum_types: enums,
			legacy_enums,
			output_types: output,
			enable_stats: false,
			time_limit: None,
			minizinc_args: Vec::new(),
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
	minizinc_args: Vec<OsString>,
}

/// Status of running and solving a [`Program`]
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
	fn from_compiler<'db, S: FnMut(InternedString<'db>) -> Arc<str>>(
		db: &'db dyn Db,
		str_interner: &mut S,
		type_map: &mut FxHashMap<Ty<'db>, Type>,
		enum_map: &FxHashMap<Arc<str>, Arc<Enum>>,
		value: Ty<'db>,
	) -> Self {
		let data = value.lookup(db);
		match data {
			TyData::Boolean(_, opt) => Type::Boolean(*opt),
			TyData::Integer(_, opt) => Type::Integer(*opt),
			TyData::Float(_, opt) => Type::Float(*opt),
			TyData::Enum(_, opt, e) => Type::Enum(*opt, enum_map[&str_interner(e.name())].clone()),
			TyData::String(opt) => Type::String(*opt),
			TyData::Annotation(opt) => Type::Annotation(*opt),
			TyData::Array { opt, dim, element } => {
				let elem = Type::from_compiler(db, str_interner, type_map, enum_map, *element);
				let mut index_conv = |nty: &TyData<'db>| -> Type {
					match nty {
						TyData::Integer(_, _) => Type::Integer(OptType::NonOpt),
						TyData::Enum(_, _, e) => {
							Type::Enum(OptType::NonOpt, enum_map[&str_interner(e.name())].clone())
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
					opt: *opt,
					dim: ndim.into_boxed_slice(),
					element: Box::new(elem),
				}
			}
			TyData::Set(_, opt, elem) => Type::Set(
				*opt,
				Box::new(Type::from_compiler(
					db,
					str_interner,
					type_map,
					enum_map,
					*elem,
				)),
			),
			TyData::Tuple(opt, li) => {
				let tmp = if *opt == OptType::NonOpt {
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
					type_map.insert(tmp, Type::Tuple(*opt, v.into_boxed_slice().into()));
					&type_map[&tmp]
				}) else {
					unreachable!()
				};
				Type::Tuple(*opt, li.clone())
			}
			TyData::Record(opt, li) => {
				let tmp = if *opt == OptType::NonOpt {
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
					type_map.insert(tmp, Type::Record(*opt, v.into_boxed_slice().into()));
					&type_map[&tmp]
				}) else {
					unreachable!()
				};
				Type::Record(*opt, li.clone())
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
	Solution {
		/// Assignments of the values of the model's output variables
		assignments: FxHashMap<&'a str, Value>,
		/// Rendered contents of the model's output sections, in display order.
		///
		/// Empty when the model has no output items, in which case the
		/// assignments are shown instead.
		sections: Vec<(&'a str, String)>,
	},
	/// Statistical information of the shackle or solving process
	Statistic(Vec<(&'a str, serde_json::Value)>),
	/// Trace messages emitted during the shackle process
	Trace(&'a str),
	/// Warning messages emitted by shackle or the solver
	Warning(&'a str),
}

impl Display for Message<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Message::Solution {
				assignments,
				sections,
			} => {
				if sections.is_empty() {
					for (name, val) in assignments {
						writeln!(f, "{} = {};", name, val)?;
					}
				} else {
					// The rendered sections are printed verbatim, but the
					// solution separator must start on its own line
					let mut ends_in_newline = true;
					for (_, contents) in sections {
						if let Some(c) = contents.chars().next_back() {
							write!(f, "{}", contents)?;
							ends_in_newline = c == '\n';
						}
					}
					if !ends_in_newline {
						writeln!(f)?;
					}
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

	/// Add arguments to be passed directly to the MiniZinc command.
	pub fn with_minizinc_args(mut self, args: Vec<OsString>) -> Self {
		self.minizinc_args = args;
		self
	}

	/// Output the [`Program`] using the given output interface, using the [`Write`] trait
	pub fn write<W: Write>(&self, out: &mut W) -> Result<(), std::io::Error> {
		let model = final_thir(&self.db).unwrap();
		let printer = PrettyPrinter::new_compat(&self.db, model);
		let code = printer.pretty_print();
		let formatted = format_str(
			&code,
			&MiniZincFormatOptions {
				keep_parentheses: false,
				..Default::default()
			},
		)
		.unwrap();
		out.write_all(formatted.as_bytes())
	}

	/// Add and parse data to be used by the program.
	pub fn add_data_files<'a>(
		&mut self,
		files: impl Iterator<Item = &'a Path>,
	) -> Result<(), Error> {
		// First parse all files:
		// - most values will be simple values that can be directly assigned
		// - some values will be values of enumerated types, possible part of tuples, records, or indices.
		// - files can also contain the constructors for enumerated types.
		let mut data = Vec::new();
		let mut names = FxHashSet::default();
		for f in files {
			let src = SourceFile::new(
				f.to_path_buf(),
				std::fs::read_to_string(f).map_err(|err| FileError {
					file: f.to_path_buf(),
					message: err.to_string(),
					other: Vec::new(),
				})?,
			);
			match f.extension().and_then(OsStr::to_str) {
				Some("dzn") => {
					// Parse the DZN file
					let dzn = parse_dzn(&src)?;
					let assignments = dzn.items().collect::<Vec<_>>();
					data.reserve(assignments.len());
					names.reserve(assignments.len());
					// Match the parser
					for asg in assignments {
						let ident = asg.assignee().cast::<Identifier>().unwrap();
						if let Some((k, ty)) = self
							.input_types
							.get_key_value::<str>(&ident.name(src.contents()))
						{
							let val = collect_dzn_value(&src, &asg.definition(), ty)?;
							data.push((k, ty, val));
							// Identifier already seen
							if names.contains(k) || self.input_data.contains_key(k) {
								return Err(error::IdentifierAlreadyDefined {
									src: src.clone(),
									span: asg.cst_node().as_ref().byte_range().into(),
									identifier: k.to_string(),
								}
								.into());
							}
							names.insert(k);
						} else if let Some((k, e)) = self
							.enum_types
							.get_key_value::<str>(&ident.name(src.contents()))
						{
							let mut inner = e.state.lock().unwrap();
							if matches!(*inner, EnumInner::NoDefinition) {
								(*inner).collect_definition(&src, &asg.definition())?
							} else {
								return Err(error::IdentifierAlreadyDefined {
									src: src.clone(),
									span: asg.cst_node().as_ref().byte_range().into(),
									identifier: k.to_string(),
								}
								.into());
							}
						} else {
							// Unknown identifier
							return Err(error::UndefinedIdentifier {
								src: src.clone(),
								span: ident.cst_node().as_ref().byte_range().into(),
								identifier: ident.name(src.contents()).to_string(),
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
						.map_err(|err| Error::from_serde_json(err, &src))?;

					data.reserve(assignments.len());
					names.reserve(assignments.len());
					for asg in assignments {
						// Identifier already seen
						if names.contains(asg.0) || self.input_data.contains_key(asg.0) {
							return Err(error::IdentifierAlreadyDefined {
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
					return Err(error::FileError {
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

/// Get a mapping from input/output identifiers to their computed types or enumerated type declaration
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelIoInterface {
	pub input: FxHashMap<Arc<str>, crate::Type>,
	pub output: FxHashMap<Arc<str>, crate::Type>,
	pub enums: FxHashMap<Arc<str>, Arc<crate::Enum>>,
}

impl ModelIoInterface {
	fn new<'db>(db: &'db dyn Db) -> Self {
		let sh = lower_model(db);
		let val = sh.get();
		let model = val.as_ref();

		// Local interner
		let mut interner: FxHashMap<InternedString<'db>, Arc<str>> = FxHashMap::default();
		let mut resolve_name = |s| {
			interner
				.entry(s)
				.or_insert_with(|| Arc::from(s.lookup(db)))
				.clone()
		};
		let mut type_map: FxHashMap<Ty<'db>, Type> = FxHashMap::default();

		// Create a map of enumerations
		let mut enums = FxHashMap::default();
		for (_, e) in model.enumerations() {
			let name = resolve_name(e.enum_type().name());
			let Some(ctors) = e.definition() else {
				enums.insert(name.clone(), Arc::new(Enum::from_data(name)));
				continue;
			};
			// A constructor that takes an argument ranges over that argument's
			// domain, which is an arbitrary expression and so may depend on
			// data. Only the atomic members can be resolved from the model.
			let mut members = Vec::with_capacity(ctors.len());
			for ctor in ctors {
				match (ctor.name, &ctor.parameters) {
					(Some(n), None) => {
						members.push((resolve_name(n.0), Box::from([]), 1));
					}
					_ => {
						members.clear();
						break;
					}
				}
			}
			if members.is_empty() && !ctors.is_empty() {
				log::warn!(
					"TODO: the members of enumerated type {} are constructed from arguments that cannot be resolved from the model, so its values can only be shown by position",
					name
				);
				// TODO: evaluate the constructor argument domains once the data
				// they depend on is known
				enums.insert(name.clone(), Arc::new(Enum::model_defined(name, [])));
			} else {
				enums.insert(
					name.clone(),
					Arc::new(Enum::from_constructors(name, members)),
				);
			}
		}

		// Find the annotation identifiers
		let reg = IdentifierRegistry::lookup(db);
		let output_ann = reg.annotations.output;
		let no_output_ann = reg.annotations.no_output;

		// Determine input and output from declarations
		let mut input = FxHashMap::default();
		let mut output = FxHashMap::default();
		for (_, decl) in model.all_declarations() {
			// Determine whether declaration is part of input
			if decl.top_level() && decl.domain().ty().known_par(db) && decl.definition().is_none() {
				let name = resolve_name(decl.name().unwrap().0);
				if name.starts_with("mzn_") {
					// TODO: Don't add these since compat.mzn maps these to the old compiler parameters
					continue;
				}
				let ty = crate::Type::from_compiler(
					db,
					&mut resolve_name,
					&mut type_map,
					&enums,
					decl.domain().ty(),
				);
				input.insert(name, ty);
			}

			// Determine whether declaration is part of output
			let mut should_output = None;
			if decl.annotations().has(model, output_ann) {
				should_output = Some(true)
			} else if decl.annotations().has(model, no_output_ann) {
				should_output = Some(false)
			}
			if should_output == Some(true)
				|| (should_output.is_none()
					&& decl.top_level()
					&& !decl.domain().ty().known_par(db)
					&& decl.definition().is_none())
			{
				let name = resolve_name(decl.name().unwrap().0);
				let ty = crate::Type::from_compiler(
					db,
					&mut resolve_name,
					&mut type_map,
					&enums,
					decl.domain().ty(),
				);
				output.insert(name, ty);
			}
		}

		ModelIoInterface {
			input,
			output,
			enums,
		}
	}
}

#[cfg(test)]
mod tests {}
