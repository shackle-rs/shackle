mod completions;
mod format;
mod goto_definition;
mod hover;
mod inlay_hints;
mod references;
mod rename_symbol;
mod semantic_tokens;
mod vfs;
mod view_ast;
mod view_cst;
mod view_format_ir;
mod view_hir;
mod view_mir;
mod view_pretty_print;
mod view_scope;

pub(crate) use self::{
	completions::*, format::*, goto_definition::*, hover::*, inlay_hints::*, references::*,
	rename_symbol::*, semantic_tokens::*, vfs::*, view_ast::*, view_cst::*, view_format_ir::*,
	view_hir::*, view_mir::*, view_pretty_print::*, view_scope::*,
};

#[cfg(test)]
pub(crate) mod tests {
	use std::{
		ops::Deref,
		path::{Path, PathBuf},
		str::FromStr,
		sync::Arc,
	};

	use expect_test::Expect;
	use lsp_server::ResponseError;
	use shackle_diagnostics::{FileError, Result};
	use shackle_hir::{
		CompilerDatabase, Db,
		db::{FileHandler, Setter},
		input::{CompilerSettings, InputFiles, ModelFile, NamedModelFile},
	};

	use crate::{
		db::{LanguageServerContext, LanguageServerOptions},
		dispatch::RequestHandler,
	};

	struct MockFileHandler(String);

	impl FileHandler for MockFileHandler {
		fn read_file(&self, path: &Path) -> Result<String> {
			if path == PathBuf::from_str("test.mzn").unwrap() {
				return Ok(self.0.clone());
			}
			std::fs::read_to_string(path).map_err(|err| {
				FileError {
					file: path.to_path_buf(),
					message: err.to_string(),
					other: Vec::new(),
				}
				.into()
			})
		}

		fn on_resolved_includes(&self, _db: &dyn Db, _files: &[ModelFile]) {}
	}

	struct MockDatabase {
		db: CompilerDatabase,
		options: LanguageServerOptions,
	}

	impl Deref for MockDatabase {
		type Target = CompilerDatabase;

		fn deref(&self) -> &Self::Target {
			&self.db
		}
	}

	impl LanguageServerContext for MockDatabase {
		fn set_active_file_from_document(
			&mut self,
			_doc: &lsp_types::TextDocumentIdentifier,
		) -> Result<ModelFile, ResponseError> {
			let file = InputFiles::get(&self.db).files(&self.db)[0];
			Ok(file)
		}

		fn get_options(&self) -> &LanguageServerOptions {
			&self.options
		}
	}

	pub(crate) fn run_handler<H, R, T>(
		model: &str,
		no_stdlib: bool,
		params: R::Params,
	) -> Result<R::Result, ResponseError>
	where
		H: RequestHandler<R, T>,
		R: lsp_types::request::Request,
	{
		let mut db = MockDatabase {
			db: CompilerDatabase::with_file_handler(Arc::new(MockFileHandler(model.to_owned()))),
			options: LanguageServerOptions {
				workspace_uri: lsp_types::Uri::from_str("file:///").ok(),
			},
		};
		let _ = CompilerSettings::get(&db.db)
			.set_ignore_stdlib(&mut db.db)
			.to(no_stdlib);
		let file = NamedModelFile::new(&db.db, PathBuf::from_str("test.mzn").unwrap());
		let _ = InputFiles::get(&db.db)
			.set_files(&mut db.db)
			.to(vec![file.into()]);
		H::prepare(&mut db, params).and_then(|t| H::execute(&db, t))
	}

	/// Test an LSP handler
	pub(crate) fn test_handler<H, R, T>(
		model: &str,
		no_stdlib: bool,
		params: R::Params,
		expected: Expect,
	) where
		H: RequestHandler<R, T>,
		R: lsp_types::request::Request,
	{
		let actual = run_handler::<H, R, T>(model, no_stdlib, params);
		expected.assert_eq(&serde_json::to_string_pretty(&actual).unwrap());
	}

	/// Test an LSP handler which returns a string
	pub(crate) fn test_handler_display<H, R, T>(
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
