//! Browser-safe Shackle transpilation entry point.

use std::{
	collections::HashMap,
	path::{Component, Path, PathBuf},
	sync::Arc,
};

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use shackle_diagnostics::{FileError, Result};
use shackle_fmt::{MiniZincFormatOptions, format_str};
use shackle_hir::{
	CompilerDatabase, Db,
	db::{FileHandler, Setter},
	input::{CompilerSettings, InputFiles, ModelFile, NamedModelFile},
	run_hir_phase,
};
use shackle_syntax as _;
use shackle_thir::{compat::OldMiniZincPrinter, transform::Transformer};
use wasm_bindgen::prelude::*;

include!(concat!(env!("OUT_DIR"), "/packed_files.rs"));

const WORKSPACE: &str = "/workspace";
const SHACKLE_STDLIB: &str = "/shackle/share/minizinc";
const MINIZINC_STDLIB: &str = "/minizinc/share/minizinc";

/// Input accepted by [`transpile`]. Only `.mzn` files are compiler inputs;
/// other Playground files are deliberately retained but ignored by Shackle.
#[derive(Deserialize)]
struct Request {
	entry: String,
	files: HashMap<String, String>,
	#[serde(default)]
	target: Target,
}

/// The form of the generated model.
#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Target {
	/// Preserve Shackle constructs so the native MiniZinc compiler can run it.
	#[default]
	MiniZinc,
	/// Lower the model through Shackle's complete THIR transformation pipeline.
	MicroZinc,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Response {
	Success(Success),
	Failure(Failure),
}
#[derive(Debug, Serialize)]
struct Success {
	ok: bool,
	model: String,
	warnings: Vec<DiagnosticRecord>,
}
#[derive(Debug, Serialize)]
struct Failure {
	ok: bool,
	diagnostics: Vec<DiagnosticRecord>,
	warnings: Vec<DiagnosticRecord>,
}

/// A renderer-neutral compiler diagnostic for the JavaScript boundary.
#[derive(Debug, Serialize)]
struct DiagnosticRecord {
	severity: &'static str,
	message: String,
	filename: Option<String>,
	line: Option<usize>,
	column: Option<usize>,
	length: Option<usize>,
}

#[wasm_bindgen(start)]
/// Install useful panic reporting for unexpected internal failures.
pub fn initialise() {
	console_error_panic_hook::set_once();
}

#[wasm_bindgen]
/// Transpile a selected Playground model. Ordinary compiler errors are values,
/// so JavaScript exceptions indicate only bridge/internal failures.
pub fn transpile(request: JsValue) -> Result<JsValue, JsValue> {
	let request: Request = serde_wasm_bindgen::from_value(request)
		.map_err(|error| JsValue::from_str(&format!("invalid shackle request: {error}")))?;
	let response = transpile_request(request);
	serde_wasm_bindgen::to_value(&response).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn transpile_request(request: Request) -> Response {
	let Some(entry) = workspace_path(&request.entry) else {
		return Response::Failure(Failure {
			ok: false,
			diagnostics: vec![plain_error("entry escapes the Playground workspace")],
			warnings: vec![],
		});
	};
	let mut project = HashMap::new();
	for (name, text) in request.files {
		if let Some(path) = workspace_path(&name) {
			let _ = project.insert(path, text);
		}
	}
	if !project.contains_key(&entry) {
		return Response::Failure(Failure {
			ok: false,
			diagnostics: vec![plain_error("selected model is not present in files")],
			warnings: vec![],
		});
	}
	let handler = Arc::new(PackedFileHandler::new(project));
	let mut db = CompilerDatabase::with_file_handler(handler);
	let settings = CompilerSettings::get(&db);
	let _ = settings
		.set_stdlib_directory(&mut db)
		.to(Some(PathBuf::from(SHACKLE_STDLIB)));
	let _ = settings
		.set_minizinc_stdlib_directory(&mut db)
		.to(Some(PathBuf::from(MINIZINC_STDLIB)));
	let _ = settings
		.set_search_directories(&mut db)
		.to(vec![PathBuf::from(WORKSPACE)]);
	let input: ModelFile = NamedModelFile::new(&db, entry).into();
	let _ = InputFiles::get(&db).set_files(&mut db).to(vec![input]);
	let hir = run_hir_phase(&db);
	let warnings = hir
		.warnings
		.iter()
		.map(|warning| diagnostic(*warning, "warning"))
		.collect();
	if !hir.errors.is_empty() {
		return Response::Failure(Failure {
			ok: false,
			diagnostics: hir
				.errors
				.iter()
				.map(|error| diagnostic(*error, "error"))
				.collect(),
			warnings,
		});
	}
	let print_input_files_only = match request.target {
		Target::MiniZinc => {
			Transformer::set_transforms(&mut db, []);
			true
		}
		Target::MicroZinc => false,
	};
	if let Err(error) = Transformer::run(&db) {
		return Response::Failure(Failure {
			ok: false,
			diagnostics: vec![diagnostic(&error, "error")],
			warnings,
		});
	}
	let emitted = OldMiniZincPrinter::run(&db, print_input_files_only);
	let model = match format_str(
		emitted,
		&MiniZincFormatOptions {
			keep_parentheses: false,
			..Default::default()
		},
	) {
		Ok(model) => model,
		Err(error) => {
			return Response::Failure(Failure {
				ok: false,
				diagnostics: vec![plain_error(format!(
					"failed to format transpiled MiniZinc: {error}"
				))],
				warnings,
			});
		}
	};
	Response::Success(Success {
		ok: true,
		model,
		warnings,
	})
}

fn plain_error(message: impl Into<String>) -> DiagnosticRecord {
	DiagnosticRecord {
		severity: "error",
		message: message.into(),
		filename: None,
		line: None,
		column: None,
		length: None,
	}
}

fn diagnostic(value: &dyn Diagnostic, severity: &'static str) -> DiagnosticRecord {
	let mut filename = None;
	let mut line = None;
	let mut column = None;
	let mut length = None;
	if let (Some(source), Some(label)) = (
		value.source_code(),
		value.labels().and_then(|mut labels| labels.next()),
	) {
		let span = *label.inner();
		length = Some(span.len());
		if let Ok(contents) = source.read_span(&span, 0, 0) {
			filename = contents.name().map(display_path);
			line = Some(contents.line() + 1);
			column = Some(contents.column() + 1);
		}
	}
	DiagnosticRecord {
		severity,
		message: value.to_string(),
		filename,
		line,
		column,
		length,
	}
}

fn display_path(path: &str) -> String {
	path.strip_prefix("/workspace/").unwrap_or(path).to_owned()
}

fn workspace_path(name: &str) -> Option<PathBuf> {
	let candidate = Path::new(WORKSPACE).join(name);
	normalize(&candidate).filter(|path| path.starts_with(WORKSPACE))
}

fn normalize(path: &Path) -> Option<PathBuf> {
	let mut result = PathBuf::new();
	for component in path.components() {
		match component {
			Component::RootDir => result.push("/"),
			Component::CurDir => {}
			Component::Normal(name) => result.push(name),
			Component::ParentDir => {
				if !result.pop() {
					return None;
				}
			}
			Component::Prefix(_) => return None,
		}
	}
	Some(result)
}

#[derive(Debug)]
struct PackedFileHandler {
	files: HashMap<PathBuf, &'static str>,
	project: HashMap<PathBuf, String>,
}
impl PackedFileHandler {
	fn new(project: HashMap<PathBuf, String>) -> Self {
		Self {
			files: PACKED_FILES
				.iter()
				.map(|(path, text)| (PathBuf::from(path), *text))
				.collect(),
			project,
		}
	}
	fn path(path: &Path) -> Option<PathBuf> {
		normalize(path)
	}
}
impl FileHandler for PackedFileHandler {
	fn read_file(&self, path: &Path) -> Result<String> {
		let Some(path) = Self::path(path) else {
			return Err(FileError {
				file: path.to_owned(),
				message: "path escapes virtual filesystem".to_owned(),
				other: vec![],
			}
			.into());
		};
		self.project
			.get(&path)
			.cloned()
			.or_else(|| self.files.get(&path).map(|text| (*text).to_owned()))
			.ok_or_else(|| {
				FileError {
					file: path,
					message: "file is not in the virtual filesystem".to_owned(),
					other: vec![],
				}
				.into()
			})
	}
	fn is_dir(&self, path: &Path) -> bool {
		let Some(path) = Self::path(path) else {
			return false;
		};
		self.project
			.keys()
			.chain(self.files.keys())
			.any(|file| file.starts_with(&path) && file != &path)
	}
	fn is_file(&self, path: &Path) -> bool {
		let Some(path) = Self::path(path) else {
			return false;
		};
		self.project.contains_key(&path) || self.files.contains_key(&path)
	}
	fn on_resolved_includes(&self, _db: &dyn Db, _files: &[ModelFile]) {}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn flattens_project_include_without_compatibility_appendix() {
		let response = transpile_request(Request {
			entry: "main.mzn".to_owned(),
			target: Target::MiniZinc,
			files: HashMap::from([
				(
					"main.mzn".to_owned(),
					"include \"part.mzn\";\nint: x = 1;\nsolve satisfy;".to_owned(),
				),
				("part.mzn".to_owned(), "int: y = 2;".to_owned()),
			]),
		});
		let Response::Success(result) = response else {
			panic!("expected valid embedded-library transpilation: {response:?}")
		};
		assert!(result.model.contains("int: y = 2"));
		assert!(!result.model.contains("include \"part.mzn\""));
		assert!(!result.model.contains("compat.mzn"));
	}

	#[test]
	fn microzinc_runs_the_default_transforms_and_prints_the_full_model() {
		let response = transpile_request(Request {
			entry: "main.mzn".to_owned(),
			target: Target::MicroZinc,
			files: HashMap::from([(
				"main.mzn".to_owned(),
				"var 1..2: x;\nsolve satisfy;".to_owned(),
			)]),
		});
		let Response::Success(result) = response else {
			panic!("expected valid MicroZinc transpilation: {response:?}")
		};
		assert!(result.model.contains("shackle_mzn_absent_zero"));
	}

	#[test]
	fn prevents_workspace_escape() {
		assert!(workspace_path("../outside.mzn").is_none());
	}

	#[test]
	fn transpiled_format_options_drop_redundant_parentheses() {
		let formatted = format_str(
			"int: x = (1 + 2);",
			&MiniZincFormatOptions {
				keep_parentheses: false,
				..Default::default()
			},
		)
		.unwrap();
		assert!(!formatted.contains("(1 + 2)"));
	}
}
