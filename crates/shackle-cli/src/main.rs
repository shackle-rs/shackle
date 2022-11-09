//! Command line interface for Shackle

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

use clap::{crate_version, Args, Parser, Subcommand};
use env_logger::{fmt::TimestampPrecision, Builder};
use miette::Result;
use shackle::error::InternalError;

use std::panic;
use std::path::PathBuf;

mod compile;

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
		SubCommand::Compile(s) => s.dispatch(),
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
	Compile(compile::Compile),
	Solve(Solve),
	Check(Check),
}

/// Solve the given model instance using the given solver
#[derive(Args, Debug)]
struct Solve {
	solver: String,
	input: Vec<PathBuf>,
}

/// Check model files for correctness
#[derive(Args, Debug)]
struct Check {
	input: Vec<PathBuf>,
}
