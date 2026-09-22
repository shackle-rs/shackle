use std::{collections::HashMap, fmt::Write};

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

	c_config.compile("tree-sitter-minizinc");

	// Extract precedences from grammar

	let grammar_path = src_dir.join("grammar.json");
	println!("cargo:rerun-if-changed={}", grammar_path.to_str().unwrap());
	let prec_code = get_precendences(&grammar_path).unwrap();
	let out_dir = std::env::var_os("OUT_DIR").unwrap();
	let dest_path = std::path::Path::new(&out_dir).join("precedence.rs");
	std::fs::write(dest_path, prec_code).unwrap();
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Prec {
	Left(i64),
	#[allow(clippy::enum_variant_names)]
	Prec(i64),
	Right(i64),
}

/// `nonAssoc` in grammar.js encodes non-associativity by adding this to the
/// level, because tree-sitter's DSL has no non-associative precedence. Testing
/// for the exact offset rather than "not a multiple of ten" keeps a deliberate
/// `prec.left(PREC.x + 1)` from being read as non-associative.
const NON_ASSOC_OFFSET: i64 = 5;

fn render_prec(prec: &Prec) -> String {
	match prec {
		Prec::Left(i) if i % 10 == NON_ASSOC_OFFSET => {
			format!("Precedence::NonAssoc({})", i - NON_ASSOC_OFFSET)
		}
		Prec::Left(i) => format!("Precedence::Left({})", i),
		Prec::Prec(i) => format!("Precedence::Prec({})", i),
		Prec::Right(i) => format!("Precedence::Right({})", i),
	}
}

fn get_precendences(
	grammar_path: impl AsRef<std::path::Path>,
) -> Result<String, Box<dyn std::error::Error>> {
	let file = std::fs::File::open(grammar_path)?;
	let reader = std::io::BufReader::new(file);
	let grammar = serde_json::from_reader::<_, serde_json::Value>(reader)?;
	let mut precedences = HashMap::new();
	let mut operator_precedences: HashMap<_, HashMap<_, _>> = HashMap::new();
	// Operators that are a rule reference rather than a literal string (the
	// backtick operators) have no fixed text to match on, so their precedence
	// becomes the fallback arm of the generated lookup.
	let mut operator_fallbacks: HashMap<String, Prec> = HashMap::new();
	// `_concatenation` is a separate rule so that type domains can exclude a
	// top-level `++`, but it is aliased to `infix_operator` in the tree, so its
	// operator has to end up in that rule's table.
	let infix = "infix_operator".to_owned();
	let mut todo = grammar["rules"]
		.as_object()
		.unwrap()
		.into_iter()
		.map(|(k, v)| {
			(
				if k == "_concatenation" { &infix } else { k },
				v,
				Prec::Prec(0),
				false,
			)
		})
		.collect::<Vec<_>>();
	while let Some((name, rule, mut prec, mut is_operator)) = todo.pop() {
		if rule["type"] == "FIELD" {
			is_operator = rule["name"] == "operator";
		}
		match rule["type"].as_str().unwrap() {
			"PREC" => {
				prec = Prec::Prec(rule["value"].as_i64().unwrap());
			}
			"PREC_LEFT" => {
				prec = Prec::Left(rule["value"].as_i64().unwrap());
			}
			"PREC_RIGHT" => {
				prec = Prec::Right(rule["value"].as_i64().unwrap());
			}
			"PREC_DYNAMIC" => {
				prec = Prec::Prec(rule["value"].as_i64().unwrap());
			}
			"STRING" => {
				operator_precedences
					.entry(name.clone())
					.or_default()
					.insert(rule["value"].as_str().unwrap().to_owned(), prec);
				continue;
			}
			"SYMBOL" if is_operator => {
				operator_fallbacks.insert(name.clone(), prec);
				continue;
			}
			_ => (),
		}
		if matches!(
			name.as_str(),
			"infix_operator" | "prefix_operator" | "postfix_operator"
		) {
			if let Some(c) = rule.get("content") {
				todo.push((name, c, prec.clone(), is_operator));
			}
			if let Some(m) = rule.get("members") {
				todo.extend(
					m.as_array()
						.unwrap()
						.iter()
						.map(|v| (name, v, prec.clone(), is_operator)),
				);
			}
		} else {
			precedences.insert(name.clone(), prec);
		}
	}

	let mut buf = "impl Precedence {".to_owned();
	for (k, v) in precedences {
		let prec = render_prec(&v);
		writeln!(&mut buf, "\t/// Get precedence for `{}`", k)?;
		writeln!(&mut buf, "\tpub fn {}() -> Precedence {{ {} }}", k, prec)?;
	}

	for (k, v) in operator_precedences.iter() {
		writeln!(&mut buf, "\t/// Get precedence for the given `{}`", k)?;
		writeln!(&mut buf, "\tpub fn {}(operator: &str) -> Precedence {{", k)?;
		writeln!(&mut buf, "\t\tmatch operator {{")?;
		for (op, prec) in v {
			let prec = render_prec(prec);
			writeln!(&mut buf, "\t\t\t{:?} => {},", op, prec)?;
		}
		match operator_fallbacks.get(k) {
			Some(prec) => {
				let prec = render_prec(prec);
				writeln!(&mut buf, "\t\t\t_ => {},", prec)?;
			}
			None => {
				writeln!(&mut buf, "\t\t\tx => panic!(\"Unknown operator {{}}\", x),")?;
			}
		}
		writeln!(&mut buf, "\t\t}}")?;
		writeln!(&mut buf, "\t}}")?;
	}
	writeln!(&mut buf, "}}")?;

	for (k, v) in operator_precedences.iter() {
		writeln!(
			&mut buf,
			"/// Whether or not this operator is {} `{}`",
			if ['a', 'e', 'i', 'o', 'u']
				.into_iter()
				.any(|v| k.starts_with(v))
			{
				"an"
			} else {
				"a"
			},
			k
		)?;
		writeln!(&mut buf, "pub fn is_{}(op: &str) -> bool {{", k)?;
		writeln!(
			&mut buf,
			"\tmatches!(op, {})",
			v.keys()
				.map(|op| format!("{:?}", op))
				.collect::<Vec<_>>()
				.join(" | ")
		)?;
		writeln!(&mut buf, "}}")?;
	}

	Ok(buf)
}
