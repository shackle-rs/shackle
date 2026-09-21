//! Command line interface for Shackle

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

use std::{
	ffi::{OsStr, OsString},
	fs::File,
	io::BufReader,
	ops::Deref,
	panic,
	path::PathBuf,
};

use clap::{Args, Parser, Subcommand, ValueEnum, crate_version};
use env_logger::{Builder, fmt::TimestampPrecision};
#[cfg(test)]
use expect_test as _;
use humantime::Duration;
use log::warn;
use miette::{IntoDiagnostic, Report, Result};
use shackle::{CompileTarget, Error, Message, Model, Solver, Status, error::InternalError};
use shackle_fmt::{MiniZincFormatOptions, check_format, format_files};
#[cfg(test)]
use tempfile as _;

/// The main function is the entry point for the `shackle` executable.
///
/// It parses the command-line arguments using a Clap parser, processes the
/// arguments, and then dispatches to called operation.
fn main() -> Result<()> {
	// Parse command line arguments
	// TODO: Should this use Miette?
	// let cli = Cli::try_parse().into_diagnostic()?;
	let cli = Cli::parse();

	// Initialise logger based on how many times the user used the "verbose"
	// flag
	let mut logger = Builder::new();
	logger
		.format_target(false)
		.format_module_path(cli.verbose >= 2)
		.filter_level(log::LevelFilter::Warn)
		.format_timestamp(match cli.verbose {
			0 => None,
			1 => Some(TimestampPrecision::Seconds),
			_ => Some(TimestampPrecision::Millis),
		})
		.parse_default_env();
	match cli.verbose {
		0 => (),
		1 => {
			logger.filter_level(log::LevelFilter::Info);
		}
		2 => {
			logger.filter_level(log::LevelFilter::Debug);
		}
		_ => {
			logger.filter_level(log::LevelFilter::Trace);
		}
	};
	logger.init();

	log::warn!(
		"Shackle is an unfinished product not ready to be used for any purpose apart from its own development."
	);

	// Dispatch to the correct subcommand
	match panic::catch_unwind(|| match cli.subcmd {
		SubCommand::Compile(c) => c.dispatch(),
		SubCommand::Solve(s) => s.dispatch(),
		SubCommand::Check(c) => c.dispatch(),
		SubCommand::Format(f) => f.dispatch(),
	}) {
		Err(_) => Err(InternalError::new("Panic occurred during execution").into()),
		Ok(res) => res,
	}
}

/// A command line interface to the shackle constraint modelling and rewriting
/// library.
#[derive(Parser)]
#[command(
    name = "shackle",
	version = crate_version!(),
)]
struct Cli {
	/// A level of verbosity, and can be used multiple times
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbose: u8,
	#[command(subcommand)]
	subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
	/// Compile a model to the intermediate form accepted by the interpreter and
	/// output the model to a file
	Compile(Box<Compile>),
	Solve(Box<Solve>),
	Check(Box<Check>),
	Format(Box<Format>),
}

/// Solve the given model instance using the given solver
#[derive(Args)]
struct Solve {
	#[arg(long)]
	statistics: bool,
	#[arg(long)]
	time_limit: Option<Duration>,
	#[command(flatten)]
	base: Compile,
	/// Additional arguments passed to MiniZinc (must follow `--`)
	#[arg(last = true)]
	minizinc_args: Vec<OsString>,
}

impl Solve {
	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, data) = self.base.sort_files()?;
		let slv = self.base.solver()?;

		// Construct model, typecheck, and compile into program
		let mut model = Model::from_file(model);
		model.set_target(self.base.target.into());
		let mut program = model.compile(&slv)?;

		program.add_data_files(data.iter().map(|f| f.deref()))?;

		// Set program options
		if let Some(time_limit) = self.time_limit {
			program = program.with_time_limit(time_limit.into());
		}
		program = program
			.with_statistics(self.statistics)
			.with_minizinc_args(self.minizinc_args.clone());

		// Run resulting program and show results
		let display_fn = |x: &Message| {
			print!("{}", x);
			Ok(())
		};
		let status = program.run(display_fn)?;
		match status {
			Status::Infeasible => println!("=====UNSATISFIABLE====="),
			Status::Satisfied => {}
			Status::Optimal | Status::AllSolutions => println!("=========="),
			Status::Unknown => println!("=====UNKNOWN====="),
		}

		// Compilation succeeded
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn compile_args(files: &[&str]) -> Compile {
		Compile {
			solver: "gecode".to_owned(),
			files: files.iter().map(PathBuf::from).collect(),
			target: CompileTargetFlag::MiniZinc,
		}
	}

	#[test]
	fn sort_files_splits_model_from_data_files() {
		let compile = compile_args(&["instance.dzn", "model.mzn", "values.json"]);

		let (model, data) = compile.sort_files().unwrap();

		assert_eq!(model, PathBuf::from("model.mzn"));
		assert_eq!(
			data,
			[PathBuf::from("instance.dzn"), PathBuf::from("values.json")]
		);
	}

	#[test]
	fn sort_files_accepts_eprime_models() {
		let compile = compile_args(&["model.eprime", "instance.dzn"]);

		let (model, data) = compile.sort_files().unwrap();

		assert_eq!(model, PathBuf::from("model.eprime"));
		assert_eq!(data, [PathBuf::from("instance.dzn")]);
	}

	#[test]
	fn sort_files_rejects_multiple_model_files() {
		let compile = compile_args(&["first.mzn", "second.eprime"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("detected multiple model files"));
		assert!(err.contains("first.mzn"));
		assert!(err.contains("second.eprime"));
	}

	#[test]
	fn sort_files_rejects_unsupported_file_types() {
		let compile = compile_args(&["model.mzn", "notes.txt"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("unsupported file type"));
		assert!(err.contains("notes.txt"));
	}

	#[test]
	fn sort_files_requires_a_model_file() {
		let compile = compile_args(&["instance.dzn", "values.json"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("no model file detected"));
	}

	#[test]
	fn solver_resolves_registered_solver_tags() {
		let compile = Compile {
			solver: "test-solver".to_owned(),
			files: vec![PathBuf::from("model.mzn")],
			target: CompileTargetFlag::MiniZinc,
		};

		assert!(compile.solver().is_ok());
	}

	#[test]
	fn boolean_flags_convert_to_bool() {
		assert!(bool::from(BooleanFlag::On));
		assert!(!bool::from(BooleanFlag::Off));
	}

	#[test]
	fn solve_forwards_arguments_after_double_dash() {
		let cli = Cli::try_parse_from([
			"shackle",
			"solve",
			"model.mzn",
			"--",
			"--all-solutions",
			"--num-solutions",
			"0",
		])
		.unwrap();

		let SubCommand::Solve(solve) = cli.subcmd else {
			panic!("expected solve subcommand");
		};
		assert_eq!(
			solve.minizinc_args,
			["--all-solutions", "--num-solutions", "0"].map(OsString::from)
		);
	}
}

/// Check model files for correctness
#[derive(Args)]
struct Check {
	/// Check whether all the data required to create a complete problem
	/// instance is provided
	#[arg(long)]
	check_complete: bool,
	#[command(flatten)]
	base: Compile,
}

impl Check {
	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, data) = self.base.sort_files()?;

		let slv = self.base.solver()?;
		let model = Model::from_file(model);
		let errors = model.check(&slv, &data, self.check_complete);

		if errors.is_empty() {
			Ok(())
		} else {
			Err(Error::try_from(errors).unwrap().into())
		}
	}
}

/// Compile the given model to a shackle intermediate format
#[derive(Args)]
pub struct Compile {
	#[arg(long, default_value = "gecode")]
	solver: String,
	#[arg(required = true)]
	files: Vec<PathBuf>,
	/// Compilation target
	#[arg(long, default_value = "microzinc")]
	target: CompileTargetFlag,
}

impl Compile {
	/// Sort through the files in the command line arguments and split the model
	/// from the data
	pub fn sort_files(&self) -> Result<(PathBuf, Vec<PathBuf>)> {
		let mut model_file: Option<PathBuf> = None;
		let mut data = Vec::with_capacity(self.files.len() - 1);
		for f in self.files.iter() {
			match f.extension().and_then(OsStr::to_str) {
				Some("mzn") | Some("eprime") => {
					if let Some(other) = model_file {
						return Err(Report::msg(format!(
							"detected multiple model files: `{}' and `{}'",
							other.display(),
							f.display()
						)));
					} else {
						model_file = Some(f.clone())
					}
				}
				Some("json") | Some("dzn") => data.push(f.clone()),
				_ => {
					return Err(Report::msg(format!(
						"file `{}' has an unsupported file type",
						f.to_str().unwrap_or_default()
					)));
				}
			}
		}
		if let Some(f) = model_file {
			Ok((f, data))
		} else {
			Err(Report::msg("no model file detected"))
		}
	}

	/// Resolve shackle [`Solver`] from the solver command line flag
	pub fn solver(&self) -> Result<Solver> {
		match shackle::Solver::lookup(self.solver.as_str()) {
			Some(slv) => Ok(slv),
			None => Err(Report::msg(format!(
				"no solver has been registered with the tag `{}'",
				self.solver,
			))),
		}
	}

	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, _data) = self.sort_files()?;

		let filename = model.with_extension("shackle.mzn");

		let slv = self.solver()?;
		let mut model = Model::from_file(model);
		model.set_target(self.target.into());
		let prg = model.compile(&slv)?;

		let mut file = File::create(filename).into_diagnostic()?;
		prg.write(&mut file).into_diagnostic()
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum BooleanFlag {
	/// Enabled
	#[value(alias("true"), alias("1"))]
	On,
	/// Disabled
	#[value(alias("false"), alias("0"))]
	Off,
}

impl From<BooleanFlag> for bool {
	fn from(value: BooleanFlag) -> Self {
		match value {
			BooleanFlag::On => true,
			BooleanFlag::Off => false,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CompileTargetFlag {
	/// Compile to MicroZinc
	#[value(alias("uzn"), alias("microzinc"))]
	MicroZinc,
	/// Compile to MiniZinc
	#[value(alias("mzn"), alias("minizinc"))]
	MiniZinc,
}

impl From<CompileTargetFlag> for CompileTarget {
	fn from(value: CompileTargetFlag) -> Self {
		match value {
			CompileTargetFlag::MicroZinc => CompileTarget::MicroZinc,
			CompileTargetFlag::MiniZinc => CompileTarget::MiniZinc,
		}
	}
}

/// Format MiniZinc code
#[derive(Args)]
struct Format {
	/// Files or directories to format.
	#[arg(required = true, value_hint = clap::ValueHint::AnyPath)]
	files: Vec<PathBuf>,

	/// Check formatting only without modifying files.
	#[arg(long)]
	check: bool,

	#[arg(long, value_hint = clap::ValueHint::FilePath)]
	config: Option<PathBuf>,

	/// Target maximum line length
	#[arg(long)]
	line_width: Option<usize>,
	/// Whether to indent using tabs
	#[arg(long)]
	use_tabs: Option<BooleanFlag>,
	/// Size of indent
	#[arg(long)]
	indent_size: Option<usize>,

	/// Keep parentheses (except double parentheses)
	#[arg(long)]
	keep_parentheses: Option<BooleanFlag>,
}

impl Format {
	/// The dispatch method checks the validity of the user input and runs the formatter
	pub fn dispatch(&self) -> Result<()> {
		let mut options = if let Some(config) = &self.config {
			let f = File::open(config).into_diagnostic()?;
			let reader = BufReader::new(f);
			serde_json::from_reader(reader).into_diagnostic()?
		} else {
			MiniZincFormatOptions::default()
		};

		if let Some(line_width) = self.line_width {
			options.line_width = line_width;
		}
		if let Some(use_tabs) = self.use_tabs {
			options.use_tabs = use_tabs.into();
		}
		if let Some(indent_size) = self.indent_size {
			options.indent_size = indent_size;
		}
		if let Some(keep_parentheses) = self.keep_parentheses {
			options.keep_parentheses = keep_parentheses.into();
		}

		if self.check {
			check_format(self.files.iter(), &options)?;
		} else {
			format_files(self.files.iter(), &options)?;
		}
		Ok(())
	}
}
