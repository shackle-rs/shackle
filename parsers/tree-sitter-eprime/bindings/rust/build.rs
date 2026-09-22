fn main() {
	let src_dir = std::path::Path::new("src");

	// `lib.rs` gates the query constants on these, so they have to be declared
	// here whether or not the corresponding query file exists.
	let queries_dir = std::path::Path::new("queries");
	for (file, cfg) in [
		("highlights.scm", "with_highlights_query"),
		("injections.scm", "with_injections_query"),
		("locals.scm", "with_locals_query"),
		("tags.scm", "with_tags_query"),
	] {
		println!("cargo::rustc-check-cfg=cfg({cfg})");
		if queries_dir.join(file).exists() {
			println!("cargo::rustc-cfg={cfg}");
		}
	}

	let mut c_config = cc::Build::new();
	c_config.std("c11").include(src_dir);
	configure_wasm_headers(&mut c_config);

	#[cfg(target_env = "msvc")]
	c_config.flag("-utf-8");

	let parser_path = src_dir.join("parser.c");
	c_config.file(&parser_path);
	println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

	let scanner_path = src_dir.join("scanner.c");
	if scanner_path.exists() {
		c_config.file(&scanner_path);
		println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());
	}

	c_config.compile("tree-sitter-eprime");
}

/// The `wasm32-unknown-unknown` target has no C sysroot. `tree-sitter-language`
/// exposes the headers it ships for this target through Cargo build metadata;
/// this crate supplies the remaining `stdbool.h` compatibility header.
fn configure_wasm_headers(c_config: &mut cc::Build) {
	let is_wasm = std::env::var("TARGET")
		.map(|target| target.starts_with("wasm32-unknown"))
		.unwrap_or(false);
	if !is_wasm {
		return;
	}

	let wasm_headers = std::env::var_os("DEP_TREE_SITTER_LANGUAGE_WASM_HEADERS")
		.expect("tree-sitter-language did not provide WASM headers");
	let compatibility_headers = std::path::Path::new("bindings/rust/wasm-include");
	c_config
		.include(compatibility_headers)
		.include(wasm_headers);
	println!(
		"cargo:rerun-if-changed={}",
		compatibility_headers.join("stdbool.h").display()
	);
}
