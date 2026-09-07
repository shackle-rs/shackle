use std::{
	path::{MAIN_SEPARATOR_STR, Path, PathBuf},
	str::FromStr,
};

use lsp_types::{Position, Uri};
use miette::{SourceCode, SpanContents};
use shackle_hir::{db::Db, ids::NodeRef};

pub(crate) fn span_contents_to_range(r: &dyn SpanContents) -> lsp_types::Range {
	let mut range = lsp_types::Range::default();
	range.start.line = r.line() as u32;
	range.start.character = r.column() as u32;
	range.end.line = range.start.line;
	range.end.character = range.start.character;

	let mut iter = r.data().iter().copied().peekable();
	while let Some(char) = iter.next() {
		if matches!(char, b'\r' | b'\n') {
			range.end.line += 1;
			range.end.character = 0;
			if char == b'\r' {
				let _ = iter.next_if_eq(&b'\n');
			}
		} else {
			range.end.character += 1;
		}
	}
	range
}

pub(crate) fn node_ref_to_location<'db, T: Into<NodeRef<'db>>>(
	db: &'db dyn Db,
	node: T,
) -> Option<lsp_types::Location> {
	let (src, span) = node.into().source_span(db);
	let span_contents = src.read_span(&span, 0, 0).ok()?;
	let uri = path_to_uri(src.path()?);
	let range = span_contents_to_range(&*span_contents);
	Some(lsp_types::Location { uri, range })
}

pub(crate) fn uri_to_path(uri: &Uri) -> PathBuf {
	assert_eq!(
		uri.scheme()
			.expect("Not a file path")
			.as_str()
			.to_lowercase(),
		"file"
	);
	let mut p = PathBuf::new();
	if let Some(auth) = uri.authority() {
		let h = auth.host().as_str();
		if h != "localhost" && !h.is_empty() {
			p.push(format!(
				"{}{}{}{}",
				MAIN_SEPARATOR_STR, MAIN_SEPARATOR_STR, h, MAIN_SEPARATOR_STR
			));
		}
	}
	// `segments()` strips the leading `/` of an absolute path, so the root has to
	// be put back. Windows drive letters are handled by the loop below instead.
	let uri_path = uri.path();
	let mut segments = uri_path.segments().peekable();
	if p.as_os_str().is_empty()
		&& uri_path.is_absolute()
		&& !segments.peek().is_some_and(|s| s.as_str().ends_with(":"))
	{
		p.push(MAIN_SEPARATOR_STR);
	}
	for segment in segments {
		let s = segment.decode().into_string_lossy().to_string();
		if s.ends_with(":") {
			p.push(format!("{}{}", s, MAIN_SEPARATOR_STR));
		} else {
			p.push(s);
		}
	}
	p
}

/// Whether a byte can appear literally in the path of a URI, i.e. RFC 3986
/// `pchar` plus the `/` separator.
fn is_uri_path_byte(b: u8) -> bool {
	b.is_ascii_alphanumeric()
		|| matches!(
			b,
			b'-' | b'.'
				| b'_' | b'~'
				| b'!' | b'$'
				| b'&' | b'\''
				| b'(' | b')'
				| b'*' | b'+'
				| b',' | b';'
				| b'=' | b':'
				| b'@' | b'/'
		)
}

pub(crate) fn path_to_uri(path: &Path) -> Uri {
	// Note that a bare path parses as a valid relative URI reference, so the
	// `file://` scheme has to be added unconditionally rather than as a fallback.
	let p = path.to_string_lossy().replace("\\", "/");
	let mut url = String::from("file://");
	if !p.starts_with("/") {
		url.push('/');
	}
	for b in p.bytes() {
		if is_uri_path_byte(b) {
			url.push(b as char);
		} else {
			url.push_str(&format!("%{:02X}", b));
		}
	}
	Uri::from_str(&url).expect("percent encoded file URI is always valid")
}

pub(crate) fn position_to_byte_offset(s: &str, position: Position) -> Option<usize> {
	let mut line = 0;
	let mut col = 0;

	for (byte_idx, ch) in s.char_indices() {
		if line == position.line && col == position.character {
			return Some(byte_idx);
		}

		if ch == '\n' {
			line += 1;
			col = 0;
		} else {
			col += 1;
		}
	}

	// Handle position at end of string
	if line == position.line && col == position.character {
		return Some(s.len());
	}

	None
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use lsp_types::Uri;

	use super::{path_to_uri, uri_to_path};

	#[test]
	fn test_uri_path_round_trip() {
		for uri in [
			"file:///test.mzn",
			"file:///Users/x/model.mzn",
			"file:///Users/x/with%20space/model.mzn",
			"file:///Users/x/@scope/model.mzn",
		] {
			let path = uri_to_path(&Uri::from_str(uri).unwrap());
			assert!(path.is_absolute(), "{:?} should be absolute", path);
			assert_eq!(path_to_uri(&path).as_str(), uri);
		}
	}
}
