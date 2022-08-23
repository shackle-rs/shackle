//! Shackle library

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

pub mod arena;
pub mod db;
pub mod error;
pub mod file;
pub mod hir;
pub mod syntax;
pub mod thir;
pub mod ty;
pub mod utils;

use db::Inputs;
use error::{MultipleErrors, ShackleError};
use file::InputFile;

use std::{
	env,
	path::{Path, PathBuf},
	sync::Arc,
	time::Instant,
};

use crate::{
	hir::db::Hir,
	thir::{db::Thir, pretty_print::PrettyPrinter},
};

/// Shackle error type
pub type Error = ShackleError;
/// Result type for Shackle operations
pub type Result<T> = std::result::Result<T, Error>;

/// Parses a list of MiniZinc files given located using the Paths in the vector
pub fn parse_files(paths: Vec<&Path>) -> Result<()> {
	let now = Instant::now();
	let mut db = db::CompilerDatabase::new();
	db.set_input_files(Arc::new(
		paths
			.into_iter()
			.map(|p| InputFile::Path(p.to_owned()))
			.collect(),
	));

	let mut search_dirs = Vec::new();
	let stdlib_dir = env::var("MZN_STDLIB_DIR");
	match stdlib_dir {
		Ok(v) => search_dirs.push(PathBuf::from(v)),
		_ => {}
	}
	db.set_search_directories(Arc::new(search_dirs));
	let mut errors = (*db.all_diagnostics()).clone();
	eprintln!("Done in {}ms", now.elapsed().as_millis());
	if errors.is_empty() {
		// Can print THIR if there were no errors
		println!(
			"{}",
			PrettyPrinter::new(&db, &db.model_thir()).pretty_print()
		);
		Ok(())
	} else if errors.len() == 1 {
		Err(errors.pop().unwrap())
	} else {
		Err(MultipleErrors { errors }.into())
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
