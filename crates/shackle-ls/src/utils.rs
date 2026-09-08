use std::{
	path::{MAIN_SEPARATOR_STR, Path, PathBuf},
	str::FromStr,
	sync::OnceLock,
};

use lsp_types::{Position, PositionEncodingKind, Uri};
use miette::SourceSpan;
use shackle_hir::{db::Db, ids::NodeRef};

/// The unit `Position::character` is counted in.
///
/// Negotiated once during initialisation. UTF-16 is what the protocol falls
/// back to, and the only encoding some clients accept; UTF-8 lets the byte
/// offsets the compiler works in pass through untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PositionEncoding {
	Utf8,
	Utf16,
}

impl PositionEncoding {
	fn width(self, ch: char) -> u32 {
		match self {
			PositionEncoding::Utf8 => ch.len_utf8() as u32,
			PositionEncoding::Utf16 => ch.len_utf16() as u32,
		}
	}
}

impl From<PositionEncoding> for PositionEncodingKind {
	fn from(encoding: PositionEncoding) -> Self {
		match encoding {
			PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
			PositionEncoding::Utf16 => PositionEncodingKind::UTF16,
		}
	}
}

static POSITION_ENCODING: OnceLock<PositionEncoding> = OnceLock::new();

/// Record the encoding negotiated with the client.
pub(crate) fn set_position_encoding(encoding: PositionEncoding) {
	let _ = POSITION_ENCODING.set(encoding);
}

fn position_encoding() -> PositionEncoding {
	*POSITION_ENCODING.get().unwrap_or(&PositionEncoding::Utf16)
}

/// Convert a byte span in `text` into an LSP range.
///
/// Offsets that fall outside `text`, or inside a character, are clamped.
pub(crate) fn source_span_to_range(text: &str, span: &SourceSpan) -> lsp_types::Range {
	source_span_to_range_in(text, span, position_encoding())
}

fn source_span_to_range_in(
	text: &str,
	span: &SourceSpan,
	encoding: PositionEncoding,
) -> lsp_types::Range {
	let start_offset = span.offset();
	let end_offset = start_offset + span.len();
	let mut position = Position::default();
	let mut start = None;
	let mut end = None;
	let mut chars = text.char_indices().peekable();
	while let Some((offset, ch)) = chars.next() {
		if start.is_none() && offset >= start_offset {
			start = Some(position);
		}
		if end.is_none() && offset >= end_offset {
			end = Some(position);
			break;
		}
		if matches!(ch, '\n' | '\r') {
			if ch == '\r' {
				let _ = chars.next_if(|(_, c)| *c == '\n');
			}
			position.line += 1;
			position.character = 0;
		} else {
			position.character += encoding.width(ch);
		}
	}
	lsp_types::Range {
		start: start.unwrap_or(position),
		end: end.unwrap_or(position),
	}
}

pub(crate) fn node_ref_to_location<'db, T: Into<NodeRef<'db>>>(
	db: &'db dyn Db,
	node: T,
) -> Option<lsp_types::Location> {
	let (src, span) = node.into().source_span(db);
	let uri = path_to_uri(src.path()?);
	let range = source_span_to_range(src.contents(), &span);
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

/// Convert an LSP position into a byte offset into `s`.
///
/// The inverse of [`source_span_to_range`].
pub(crate) fn position_to_byte_offset(s: &str, position: Position) -> Option<usize> {
	position_to_byte_offset_in(s, position, position_encoding())
}

fn position_to_byte_offset_in(
	s: &str,
	position: Position,
	encoding: PositionEncoding,
) -> Option<usize> {
	let mut line = 0;
	let mut character = 0;

	let mut chars = s.char_indices().peekable();
	while let Some((byte_idx, ch)) = chars.next() {
		if line == position.line {
			if matches!(ch, '\n' | '\r') {
				// The spec requires a character past the end of the line to clamp
				// to the line length.
				return Some(byte_idx);
			}
			if position.character < character + encoding.width(ch) {
				// Covers both an exact hit and a position inside a character,
				// which clamps to where that character starts.
				return Some(byte_idx);
			}
		}

		if matches!(ch, '\n' | '\r') {
			if ch == '\r' {
				let _ = chars.next_if(|(_, c)| *c == '\n');
			}
			line += 1;
			character = 0;
		} else {
			character += encoding.width(ch);
		}
	}

	// Anything at or past the end of the document clamps to the end.
	(line <= position.line).then_some(s.len())
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use lsp_types::{Position, Uri};

	use super::{
		PositionEncoding, path_to_uri, position_to_byte_offset_in, source_span_to_range_in,
		uri_to_path,
	};

	// `é` is one UTF-16 code unit but two bytes; `𝄞` is two units, four bytes.
	const TEXT: &str = "int: café = 1;\nconstraint 𝄞 > 0;\n";

	#[test]
	fn test_source_span_to_range_utf16() {
		let r = source_span_to_range_in(TEXT, &(5, 5).into(), PositionEncoding::Utf16);
		assert_eq!((r.start.line, r.start.character), (0, 5));
		assert_eq!((r.end.line, r.end.character), (0, 9));

		let r = source_span_to_range_in(TEXT, &(27, 4).into(), PositionEncoding::Utf16);
		assert_eq!((r.start.line, r.start.character), (1, 11));
		assert_eq!((r.end.line, r.end.character), (1, 13));
	}

	#[test]
	fn test_source_span_to_range_utf8() {
		// In UTF-8 the columns are the byte offsets the compiler already uses.
		let r = source_span_to_range_in(TEXT, &(5, 5).into(), PositionEncoding::Utf8);
		assert_eq!((r.start.line, r.start.character), (0, 5));
		assert_eq!((r.end.line, r.end.character), (0, 10));

		let r = source_span_to_range_in(TEXT, &(27, 4).into(), PositionEncoding::Utf8);
		assert_eq!((r.start.line, r.start.character), (1, 11));
		assert_eq!((r.end.line, r.end.character), (1, 15));
	}

	#[test]
	fn test_position_to_byte_offset_round_trips() {
		for (encoding, cases) in [
			(
				PositionEncoding::Utf16,
				[(0, 5, 5), (0, 9, 10), (1, 11, 27), (1, 13, 31)],
			),
			(
				PositionEncoding::Utf8,
				[(0, 5, 5), (0, 10, 10), (1, 11, 27), (1, 15, 31)],
			),
		] {
			for (line, character, offset) in cases {
				assert_eq!(
					position_to_byte_offset_in(TEXT, Position::new(line, character), encoding),
					Some(offset),
					"{:?} at {}:{}",
					encoding,
					line,
					character
				);
			}
		}
		// A CR-only file still advances lines.
		assert_eq!(
			position_to_byte_offset_in("a\rb", Position::new(1, 0), PositionEncoding::Utf16),
			Some(2)
		);
	}

	#[test]
	fn test_position_to_byte_offset_clamps() {
		let e = PositionEncoding::Utf16;
		// Past the end of a line clamps to the line end, not an error.
		assert_eq!(
			position_to_byte_offset_in(TEXT, Position::new(0, 99), e),
			Some(15)
		);
		// Inside a surrogate pair clamps to the start of the character.
		assert_eq!(
			position_to_byte_offset_in(TEXT, Position::new(1, 12), e),
			Some(27)
		);
		// Past the end of the document clamps to the end.
		assert_eq!(
			position_to_byte_offset_in(TEXT, Position::new(99, 0), e),
			Some(TEXT.len())
		);
	}

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
