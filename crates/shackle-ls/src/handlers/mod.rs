mod completions;
mod goto_definition;
mod hover;
mod references;
mod semantic_tokens;
mod vfs;
mod view_ast;
mod view_cst;
mod view_hir;
mod view_pretty_print;
mod view_scope;

pub use self::completions::*;
pub use self::goto_definition::*;
pub use self::hover::*;
pub use self::references::*;
pub use self::semantic_tokens::*;
pub use self::vfs::*;
pub use self::view_ast::*;
pub use self::view_cst::*;
pub use self::view_hir::*;
pub use self::view_pretty_print::*;
pub use self::view_scope::*;

#[cfg(test)]
pub mod test {
	use expect_test::Expect;
	use lsp_server::ResponseError;
	use shackle::{
		db::{CompilerDatabase, FileReader, Inputs},
		diagnostics::FileError,
		file::{FileHandler, InputFile},
	};
	use std::{
		ops::Deref,
		panic::RefUnwindSafe,
		path::{Path, PathBuf},
		str::FromStr,
		sync::Arc,
	};

	use crate::{db::LanguageServerContext, dispatch::RequestHandler};

	struct MockFileHandler(String);

	impl FileHandler for MockFileHandler {
		fn durable(&self) -> bool {
			true
		}

		fn read_file(&self, path: &Path) -> Result<Arc<String>, FileError> {
			if path == PathBuf::from_str("test.mzn").unwrap() {
				return Ok(Arc::new(self.0.clone()));
			}
			std::fs::read_to_string(path)
				.map(Arc::new)
				.map_err(|err| FileError {
					file: path.to_path_buf(),
					message: err.to_string(),
					other: Vec::new(),
				})
		}

		fn snapshot(&self) -> Box<dyn FileHandler + RefUnwindSafe> {
			unimplemented!()
		}
	}

	struct MockDatabase(CompilerDatabase);

	impl Deref for MockDatabase {
		type Target = CompilerDatabase;
		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	impl LanguageServerContext for MockDatabase {
		fn set_active_file_from_document(
			&mut self,
			_doc: &lsp_types::TextDocumentIdentifier,
		) -> Result<shackle::file::ModelRef, lsp_server::ResponseError> {
			Ok(self.input_models()[0])
		}
	}

	pub fn run_handler<H, R, T>(
		model: &str,
		no_stdlib: bool,
		params: R::Params,
	) -> Result<R::Result, ResponseError>
	where
		H: RequestHandler<R, T>,
		R: lsp_types::request::Request,
	{
		let mut db = MockDatabase(CompilerDatabase::with_file_handler(Box::new(
			MockFileHandler(model.to_string()),
		)));
		db.0.set_ignore_stdlib(no_stdlib);
		db.0.set_input_files(Arc::new(vec![InputFile::Path(
			PathBuf::from_str("test.mzn").unwrap(),
		)]));
		H::prepare(&mut db, params).and_then(|t| H::execute(&db, t))
	}

	/// Test an LSP handler
	pub fn test_handler<H, R, T>(model: &str, no_stdlib: bool, params: R::Params, expected: Expect)
	where
		H: RequestHandler<R, T>,
		R: lsp_types::request::Request,
	{
		let actual = run_handler::<H, R, T>(model, no_stdlib, params);
		expected.assert_eq(&serde_json::to_string_pretty(&actual).unwrap());
	}

	/// Test an LSP handler which returns a string
	pub fn test_handler_display<H, R, T>(
		model: &str,
		no_stdlib: bool,
		params: R::Params,
		expected: Expect,
	) where
		H: RequestHandler<R, T>,
		R: lsp_types::request::Request,
		R::Result: std::fmt::Display,
	{
		let actual = run_handler::<H, R, T>(model, no_stdlib, params);
		if let Ok(s) = actual {
			expected.assert_eq(&s.to_string());
		} else {
			expected.assert_eq(&serde_json::to_string_pretty(&actual).unwrap());
		}
	}
}
