use std::path::{Path, PathBuf};

use lsp_types::notification::Notification;
use miette::{Diagnostic, Severity};
use shackle_hir::{Db, all_errors, all_warnings};

use crate::utils::{path_to_uri, span_contents_to_range};

/// Whether the span name of a diagnostic refers to `path`.
///
/// `SourceFile::name` is a display string: it strips the canonicalised working
/// directory, so a diagnostic for a file inside the directory the server was
/// started in is named relatively. Resolving it against the same base is what
/// makes the comparison against the absolute `path` meaningful.
fn span_name_is(name: &str, path: &Path, base: Option<&Path>) -> bool {
	let named = Path::new(name);
	if named.is_absolute() {
		named == path
	} else {
		base.is_some_and(|base| base.join(named) == path)
	}
}

fn source_file_base() -> Option<PathBuf> {
	std::env::current_dir().ok()?.canonicalize().ok()
}

pub(crate) fn diagnostics_notification(db: &dyn Db, path: &Path) -> lsp_server::Notification {
	let base = source_file_base();
	let base = base.as_deref();
	let mut diagnostics = Vec::new();
	for d in all_errors(db) {
		let _ = collect_diagnostic(path, base, d, &mut diagnostics);
	}
	for d in all_warnings(db) {
		let _ = collect_diagnostic(path, base, d, &mut diagnostics);
	}
	lsp_server::Notification {
		method: lsp_types::notification::PublishDiagnostics::METHOD.to_owned(),
		params: serde_json::to_value(lsp_types::PublishDiagnosticsParams {
			uri: path_to_uri(path),
			diagnostics,
			version: None,
		})
		.unwrap(),
	}
}

fn collect_diagnostic(
	path: &Path,
	base: Option<&Path>,
	d: &dyn Diagnostic,
	out: &mut Vec<lsp_types::Diagnostic>,
) -> Option<()> {
	let sc = d.source_code()?;
	let mut ls = d.labels()?;
	let first = ls.next()?;
	let span = sc.read_span(first.inner(), 0, 0).ok()?;
	let range = span_contents_to_range(span.as_ref());
	let name = span.name()?;
	if !span_name_is(name, path, base) {
		return None;
	}
	let uri = path_to_uri(path);
	let related_info: Vec<_> = ls
		.filter_map(|l| {
			let label = l.label()?;
			let r = sc.read_span(l.inner(), 0, 0).unwrap();
			let range = span_contents_to_range(r.as_ref());
			Some(lsp_types::DiagnosticRelatedInformation {
				location: lsp_types::Location {
					range,
					uri: uri.clone(),
				},
				message: label.to_owned(),
			})
		})
		.collect();
	out.push(lsp_types::Diagnostic {
		code: d
			.code()
			.map(|c| lsp_types::NumberOrString::String(c.to_string())),
		severity: d.severity().map(|s| match s {
			Severity::Error => lsp_types::DiagnosticSeverity::ERROR,
			Severity::Warning => lsp_types::DiagnosticSeverity::WARNING,
			Severity::Advice => lsp_types::DiagnosticSeverity::HINT,
		}),
		related_information: if related_info.is_empty() {
			None
		} else {
			Some(related_info)
		},
		range,
		source: Some("minizinc".to_owned()),
		message: format!(
			"{}\n",
			[d.to_string()]
				.into_iter()
				.chain(first.label().map(|l| l.to_owned()))
				.chain(d.help().map(|h| h.to_string()))
				.collect::<Vec<_>>()
				.join("\n")
		),
		..Default::default()
	});
	if let Some(related) = d.related() {
		for d in related {
			let _ = collect_diagnostic(path, base, d, out);
		}
	}
	Some(())
}

#[cfg(test)]
mod tests {
	use std::path::{Path, PathBuf};

	use super::span_name_is;

	#[test]
	fn test_span_name_is() {
		let base = PathBuf::from("/home/user/project");
		let file = Path::new("/home/user/project/model.mzn");

		// A file inside the working directory is named relatively.
		assert!(span_name_is("model.mzn", file, Some(&base)));
		assert!(span_name_is(
			"sub/model.mzn",
			Path::new("/home/user/project/sub/model.mzn"),
			Some(&base)
		));
		// One outside it keeps its absolute path.
		assert!(span_name_is(
			"/elsewhere/model.mzn",
			Path::new("/elsewhere/model.mzn"),
			Some(&base)
		));

		assert!(!span_name_is("other.mzn", file, Some(&base)));
		assert!(!span_name_is("/elsewhere/model.mzn", file, Some(&base)));
		assert!(!span_name_is("model.mzn", file, None));
	}
}
