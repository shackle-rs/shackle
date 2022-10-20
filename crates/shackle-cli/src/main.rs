//! Command line interface for Shackle

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

use clap::{crate_version, Args, Parser, Subcommand};
use env_logger::{fmt::TimestampPrecision, Builder};
use humantime::Duration;
use miette::{Report, Result};
use shackle::error::InternalError;
use shackle::{Message, Model, Solver, Status};

use std::ffi::OsStr;
use std::panic;
use std::path::PathBuf;

/// The main function is the entry point for the `shackle` executable.
///
/// It parses the command-line arguments using a Clap parser, processes the arguments, and then
/// dispatches to called operation.
fn main() -> Result<()> {
	// Parse command line arguments
	let opts: Opts = Opts::parse();

	// Initialise logger based on how many times the user used the "verbose" flag
	let mut logger = Builder::new();
	logger
		.format_target(false)
		.format_module_path(opts.verbose >= 2)
		.filter_level(log::LevelFilter::Warn)
		.format_timestamp(match opts.verbose {
			0 => None,
			1 => Some(TimestampPrecision::Seconds),
			_ => Some(TimestampPrecision::Millis),
		})
		.parse_default_env();
	match opts.verbose {
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

	log::warn!("Shackle is an unfinished product not ready to be used for any purpose apart from its own development.");

	// Dispatch to the correct subcommand
	match panic::catch_unwind(|| match opts.subcmd {
		SubCommand::Compile(c) => c.dispatch(),
		SubCommand::Solve(s) => s.dispatch(),
		_ => unimplemented!(),
	}) {
		Err(_) => Err(InternalError::new("Panic occurred during execution").into()),
		Ok(res) => res,
	}
}

/// A command line interface to the shackle constraint modelling and rewriting library.
#[derive(Parser, Debug)]
#[clap(
    name = "shackle",
	version = crate_version!(),
)]
struct Opts {
	/// A level of verbosity, and can be used multiple times
	#[clap(short, long, action = clap::ArgAction::Count)]
	verbose: u8,
	#[clap(subcommand)]
	subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
	Compile(Compile),
	Solve(Box<Solve>),
	Check(Check),
}

/// Solve the given model instance using the given solver
#[derive(Args, Debug)]
struct Solve {
	#[clap(long, default_value = "gecode")]
	solver: String,
	input: PathBuf,
	#[clap(long)]
	statistics: bool,
	#[clap(long)]
	time_limit: Option<Duration>,
}

impl Solve {
	/// The dispatch method checks the validity of the user input and then call the corresponding
	/// functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		match self.input.extension().and_then(OsStr::to_str) {
			Some("mzn") => {}
			Some("eprime") => {}
			_ => {
				return Err(Report::msg(format!(
					"File {:?} has an unsupported file type",
					self.input
				)));
			}
		}

		// Lookup Solver definition
		let slv = Solver::lookup(self.solver.as_str()).unwrap();

		// Construct model, typecheck, and compile into program
		let model = Model::from_file(self.input.clone());
		let mut program = model.compile(&slv)?;

		// Set program options
		if let Some(time_limit) = self.time_limit {
			program = program.with_time_limit(time_limit.into());
		}
		program = program.with_statistics(self.statistics);

		// Run resulting program and show results
		let display_fn = |x: &Message| {
			print!("{}", x);
			true
		};
		let status = program.run(&display_fn);
		match status {
			Status::Infeasible => println!("=====UNSATISFIABLE====="),
			Status::Satisfied => {}
			Status::Optimal | Status::AllSolutions => println!("=========="),
			Status::Unknown => println!("=====UNKNOWN====="),
			Status::Err(err) => return Err(err.into()),
		}

		// Compilation succeeded
		Ok(())
	}
}

/// Check model files for correctness
#[derive(Args, Debug)]
struct Check {
	input: PathBuf,
}

/// Compile the given model to a shackle intermediate format
#[derive(Args, Debug)]
pub struct Compile {
	#[clap(required = true)]
	input: PathBuf,
}

impl Compile {
	/// The dispatch method checks the validity of the user input and then call the corresponding
	/// functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		Ok(())
	}
}
