use clap::Args;
use miette::{Report, Result};

use std::ffi::OsStr;
use std::path::PathBuf;

/// Compile the given model to a shackle intermediate format
#[derive(Args, Debug)]
pub struct Compile {
	#[clap(required = true)]
	input: Vec<PathBuf>,
}

impl Compile {
	/// The dispatch method checks the validity of the user input and then call the corresponding
	/// functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		for i in &self.input {
			match i.extension().and_then(OsStr::to_str) {
				Some("mzn") => {},
				Some("eprime") => {},
				_ => {
					return Err(Report::msg(format!(
						"File {:?} has an unsupported file type",
						i
					)));
				}
			}
		}

		// Construct model, typecheck, and compile
		shackle::parse_files(self.input.iter().map(PathBuf::as_path).collect())?;
		// Output compiled model

		// Compilation succeeded
		Ok(())
	}
}
