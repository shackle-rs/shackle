use std::path::{Path, PathBuf};

use lsp_types::notification::Notification;
use miette::{Diagnostic, Severity, SourceCode, SourceSpan};
use shackle_hir::{Db, all_errors, all_warnings};

use crate::utils::{path_to_uri, source_span_to_range};

const ERROR: lsp_types::DiagnosticSeverity = lsp_types::DiagnosticSeverity::ERROR;
const WARNING: lsp_types::DiagnosticSeverity = lsp_types::DiagnosticSeverity::WARNING;

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

/// Convert a diagnostic's span into an LSP range.
///
/// Only the erased `SourceCode` is available here, and reading a span with no
/// context returns just the span itself, so the text preceding it — which is
/// what the UTF-16 columns are measured over — has to be requested separately.
fn span_to_range(sc: &dyn SourceCode, span: &SourceSpan) -> Option<lsp_types::Range> {
	let prefix = sc
		.read_span(&(0, span.offset() + span.len()).into(), 0, 0)
		.ok()?;
	let text = std::str::from_utf8(prefix.data()).ok()?;
	Some(source_span_to_range(text, span))
}

pub(crate) fn diagnostics_notification(db: &dyn Db, path: &Path) -> lsp_server::Notification {
	let base = source_file_base();
	let base = base.as_deref();
	let mut diagnostics = Vec::new();
	for d in all_errors(db) {
		let _ = collect_diagnostic(path, base, ERROR, d, &mut diagnostics);
	}
	for d in all_warnings(db) {
		let _ = collect_diagnostic(path, base, WARNING, d, &mut diagnostics);
	}
	publish(path, diagnostics)
}

/// Clear any diagnostics previously published for `path`.
///
/// Clients keep showing them until the server says otherwise, so a closed file
/// would otherwise keep its markers for the rest of the session.
pub(crate) fn clear_notification(path: &Path) -> lsp_server::Notification {
	publish(path, Vec::new())
}

fn publish(path: &Path, diagnostics: Vec<lsp_types::Diagnostic>) -> lsp_server::Notification {
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
	default_severity: lsp_types::DiagnosticSeverity,
	d: &dyn Diagnostic,
	out: &mut Vec<lsp_types::Diagnostic>,
) -> Option<()> {
	// Some diagnostics carry no source span at all — a missing standard library
	// being the important one. Reporting them against the start of the file is
	// the only way they reach the editor, and without them the user sees just
	// the errors they cause downstream.
	let located = d
		.source_code()
		.zip(d.labels())
		.and_then(|(sc, mut labels)| Some((sc, labels.next()?)));

	let (range, label, related_info) = match located {
		Some((sc, first)) => {
			let name = sc.read_span(first.inner(), 0, 0).ok()?;
			if !span_name_is(name.name()?, path, base) {
				return None;
			}
			let uri = path_to_uri(path);
			let related_info: Vec<_> = d
				.labels()
				.into_iter()
				.flatten()
				.skip(1)
				.filter_map(|l| {
					let label = l.label()?;
					let range = span_to_range(sc, l.inner())?;
					Some(lsp_types::DiagnosticRelatedInformation {
						location: lsp_types::Location {
							range,
							uri: uri.clone(),
						},
						message: label.to_owned(),
					})
				})
				.collect();
			(
				span_to_range(sc, first.inner())?,
				first.label().map(|l| l.to_owned()),
				related_info,
			)
		}
		None => (lsp_types::Range::default(), None, Vec::new()),
	};

	out.push(lsp_types::Diagnostic {
		code: d
			.code()
			.map(|c| lsp_types::NumberOrString::String(c.to_string())),
		// Most diagnostics do not declare a severity, and an omitted one makes
		// clients render everything as an error; which accumulator it came from
		// is the better default.
		severity: Some(
			d.severity()
				.map(|s| match s {
					Severity::Error => ERROR,
					Severity::Warning => WARNING,
					Severity::Advice => lsp_types::DiagnosticSeverity::HINT,
				})
				.unwrap_or(default_severity),
		),
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
				.chain(label)
				.chain(d.help().map(|h| h.to_string()))
				.collect::<Vec<_>>()
				.join("\n")
		),
		..Default::default()
	});
	if let Some(related) = d.related() {
		for d in related {
			let _ = collect_diagnostic(path, base, default_severity, d, out);
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
