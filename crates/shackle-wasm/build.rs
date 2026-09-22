#![allow(missing_docs, reason = "Cargo build scripts are not public API")]

use std::{
	env, fs,
	path::{Path, PathBuf},
};

fn collect(root: &Path, virtual_root: &str, files: &mut Vec<(String, PathBuf)>) {
	collect_below(root, root, virtual_root, files);
}

fn collect_below(
	root: &Path,
	directory: &Path,
	virtual_root: &str,
	files: &mut Vec<(String, PathBuf)>,
) {
	let entries = fs::read_dir(directory)
		.unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
	for entry in entries {
		let entry = entry.unwrap_or_else(|error| panic!("cannot read library entry: {error}"));
		let path = entry.path();
		if path.is_dir() {
			collect_below(root, &path, virtual_root, files);
		} else if path.is_file() {
			let relative = path.strip_prefix(root).expect("walked below root");
			files.push((
				format!(
					"{virtual_root}/{}",
					relative.to_string_lossy().replace('\\', "/")
				),
				path,
			));
		}
	}
}

fn main() {
	println!("cargo:rerun-if-env-changed=MZN_STDLIB_DIR");
	let upstream = env::var_os("MZN_STDLIB_DIR")
		.map(PathBuf::from)
		.expect("shackle-wasm requires MZN_STDLIB_DIR to name MiniZinc's share/minizinc directory");
	for required in ["std/stdlib.mzn", "std/solver_redefinitions.mzn"] {
		if !upstream.join(required).is_file() {
			panic!(
				"MZN_STDLIB_DIR={} is not a MiniZinc share/minizinc directory: missing {required}",
				upstream.display()
			);
		}
	}
	let shackle =
		PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../share/minizinc");
	let mut files = Vec::new();
	collect(&shackle, "/shackle/share/minizinc", &mut files);
	collect(&upstream, "/minizinc/share/minizinc", &mut files);
	files.sort_by(|a, b| a.0.cmp(&b.0));
	let mut generated = String::from(
		"/// Files embedded into the browser-only virtual filesystem.\npub static PACKED_FILES: &[(&str, &str)] = &[\n",
	);
	for (virtual_path, path) in files {
		println!("cargo:rerun-if-changed={}", path.display());
		generated.push_str(&format!(
			"    ({virtual_path:?}, include_str!({:?})),\n",
			path
		));
	}
	generated.push_str("];\n");
	fs::write(
		PathBuf::from(env::var("OUT_DIR").unwrap()).join("packed_files.rs"),
		generated,
	)
	.unwrap();
}
