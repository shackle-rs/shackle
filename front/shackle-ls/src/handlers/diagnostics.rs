use lsp_server::Connection;
use lsp_types::notification::Notification;
use lsp_types::Url;
use miette::{Diagnostic, Severity};
use shackle::db::*;
use shackle::file::InputFile;
use shackle::hir::db::Hir;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use threadpool::ThreadPool;

use crate::utils::span_contents_to_range;

pub fn publish_diagnostics(
	db: &mut CompilerDatabase,
	path: &Path,
	pool: &ThreadPool,
	sender: &Connection,
) {
	db.set_input_files(Arc::new(vec![InputFile::Path(path.to_owned())]));
	let path = path.to_owned();
	let sender = sender.sender.clone();
	let snapshot = db.snapshot();
	pool.execute(move || {
		let all_diagnostics = snapshot.all_diagnostics();
		let mut diagnostics = Vec::new();
		for d in all_diagnostics.iter() {
			collect_diagnostic(&path, d, &mut diagnostics);
		}
		sender
			.send(lsp_server::Message::Notification(
				lsp_server::Notification {
					method: lsp_types::notification::PublishDiagnostics::METHOD.to_owned(),
					params: serde_json::to_value(lsp_types::PublishDiagnosticsParams {
						uri: Url::from_file_path(path).unwrap(),
						diagnostics,
						version: None,
					})
					.unwrap(),
				},
			))
			.unwrap();
	});
}

fn collect_diagnostic(
	path: &Path,
	d: &dyn Diagnostic,
	out: &mut Vec<lsp_types::Diagnostic>,
) -> Option<()> {
	let sc = d.source_code()?;
	let mut ls = d.labels()?;
	let first = ls.next()?;
	let span = sc.read_span(first.inner(), 0, 0).ok()?;
	let range = span_contents_to_range(span.as_ref());
	let name = span.name()?;
	let p = PathBuf::from_str(name).ok()?.canonicalize().ok()?;
	if p.as_path() != path {
		return None;
	}
	let uri = Url::from_file_path(path).ok()?;
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
			collect_diagnostic(path, d, out);
		}
	}
	Some(())
}
