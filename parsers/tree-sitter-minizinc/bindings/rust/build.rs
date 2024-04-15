use std::{collections::HashMap, fmt::Write};

fn main() {
	let src_dir = std::path::Path::new("src");

	let mut c_config = cc::Build::new();
	c_config.std("c11").include(src_dir);

	#[cfg(target_env = "msvc")]
	c_config.flag("-utf-8");

	let parser_path = src_dir.join("parser.c");
	c_config.file(&parser_path);
	println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

	// NOTE: if your language uses an external scanner, uncomment this block:
	/*
	let scanner_path = src_dir.join("scanner.c");
	c_config.file(&scanner_path);
	println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());
	*/

	c_config.compile("tree-sitter-minizinc");

	// Extract precedences from grammar

	let grammar_path = src_dir.join("grammar.json");
	println!("cargo:rerun-if-changed={}", grammar_path.to_str().unwrap());
	let prec_code = get_precendences(&grammar_path).unwrap();
	let out_dir = std::env::var_os("OUT_DIR").unwrap();
	let dest_path = std::path::Path::new(&out_dir).join("precedence.rs");
	std::fs::write(dest_path, prec_code).unwrap();
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Prec {
	Left(i64),
	#[allow(clippy::enum_variant_names)]
	Prec(i64),
	Right(i64),
}

fn get_precendences(
	grammar_path: impl AsRef<std::path::Path>,
) -> Result<String, Box<dyn std::error::Error>> {
	let file = std::fs::File::open(grammar_path)?;
	let reader = std::io::BufReader::new(file);
	let grammar = serde_json::from_reader::<_, serde_json::Value>(reader)?;
	let mut precedences = HashMap::new();
	let mut operator_precedences: HashMap<_, HashMap<_, _>> = HashMap::new();
	let mut todo = grammar["rules"]
		.as_object()
		.unwrap()
		.into_iter()
		.map(|(k, v)| (k, v, Prec::Prec(0)))
		.collect::<Vec<_>>();
	while let Some((name, rule, mut prec)) = todo.pop() {
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
			"STRING" => {
				operator_precedences
					.entry(name.clone())
					.or_default()
					.insert(rule["value"].as_str().unwrap().to_owned(), prec);
				continue;
			}
			_ => (),
		}
		if matches!(
			name.as_str(),
			"infix_operator" | "prefix_operator" | "postfix_operator"
		) {
			if let Some(c) = rule.get("content") {
				todo.push((name, c, prec.clone()));
			}
			if let Some(m) = rule.get("members") {
				todo.extend(
					m.as_array()
						.unwrap()
						.iter()
						.map(|v| (name, v, prec.clone())),
				);
			}
		} else {
			precedences.insert(name.clone(), prec);
		}
	}

	let mut buf = "impl Precedence {".to_owned();
	for (k, v) in precedences {
		let prec = match v {
			Prec::Left(i) => {
				format!("Precedence::Left({})", i)
			}
			Prec::Prec(i) => {
				format!("Precedence::Prec({})", i)
			}
			Prec::Right(i) => {
				format!("Precedence::Right({})", i)
			}
		};
		writeln!(&mut buf, "\t/// Get precedence for `{}`", k)?;
		writeln!(&mut buf, "\tpub fn {}() -> Precedence {{ {} }}", k, prec)?;
	}

	for (k, v) in operator_precedences {
		writeln!(&mut buf, "\t/// Get precedence for the given `{}`", k)?;
		writeln!(&mut buf, "\tpub fn {}(operator: &str) -> Precedence {{", k)?;
		writeln!(&mut buf, "\t\tmatch operator {{")?;
		for (op, prec) in v {
			let prec = match prec {
				Prec::Left(i) => {
					format!("Precedence::Left({})", i)
				}
				Prec::Prec(i) => {
					format!("Precedence::Prec({})", i)
				}
				Prec::Right(i) => {
					format!("Precedence::Right({})", i)
				}
			};
			writeln!(&mut buf, "\t\t\t{:?} => {},", op, prec)?;
		}
		writeln!(&mut buf, "\t\t\tx => panic!(\"Unknown operator {{}}\", x),")?;
		writeln!(&mut buf, "\t\t}}")?;
		writeln!(&mut buf, "\t}}")?;
	}
	writeln!(&mut buf, "}}")?;
	Ok(buf)
}
