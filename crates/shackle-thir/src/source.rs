//! Source mapping between THIR and HIR nodes.
//!
//! Tracks desugarings performed when lowering HIR to THIR.

use miette::SourceSpan;
use shackle_diagnostics::SourceFile;
use shackle_hir::{
	Item,
	db::with_attached_database,
	ids::{EntityRef, NodeRef},
};

use crate::Db;

/// The HIR node which produced a THIR node
#[derive(Copy, Clone, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum Origin<'db> {
	/// Comes from a real HIR node
	HirNode(NodeRef<'db>),
	/// Is introduced, and does not have a location
	Introduced(&'static str),
}

impl<'db> From<NodeRef<'db>> for Origin<'db> {
	fn from(node: NodeRef<'db>) -> Self {
		Self::HirNode(node)
	}
}

impl<'db> From<Item<'db>> for Origin<'db> {
	fn from(item: Item<'db>) -> Self {
		NodeRef::from(item).into()
	}
}

impl<'db> From<EntityRef<'db>> for Origin<'db> {
	fn from(entity: EntityRef<'db>) -> Self {
		NodeRef::from(entity).into()
	}
}

impl<'db> std::fmt::Debug for Origin<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		with_attached_database(|db| write!(f, "{}", self.pretty_print(db)))
			.unwrap_or_else(|| f.debug_struct("Origin").finish())
	}
}

impl<'db> Origin<'db> {
	/// Get the underlying HIR node
	pub fn node(&self) -> Option<NodeRef<'_>> {
		match self {
			Origin::HirNode(node) => Some(*node),
			_ => None,
		}
	}

	/// Get the source file and span of this origin
	pub fn source_span(&self, db: &'db dyn Db) -> (SourceFile, SourceSpan) {
		match self {
			Origin::HirNode(node) => node.source_span(db),
			Origin::Introduced(name) => {
				(SourceFile::introduced(name), SourceSpan::new(0.into(), 0))
			}
		}
	}

	/// Pretty print this origin
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		format!("{}:{}", self.name(db), self.line_char(db))
	}

	/// Get the name of the source file of this origin
	pub fn name(&self, db: &'db dyn Db) -> String {
		let (src, _) = self.source_span(db);
		src.name()
	}

	/// Get the line and character span of this origin
	pub fn line_char(&self, db: &'db dyn Db) -> LineCharSpan {
		let (src, span) = self.source_span(db);
		let mut lc = LineCharSpan::default();
		let mut iter = src.contents()[0..span.offset() + span.len()]
			.chars()
			.enumerate()
			.peekable();
		while let Some((i, char)) = iter.next() {
			if matches!(char, '\r' | '\n') {
				if i < span.offset() {
					lc.from_line += 1;
					lc.from_char = 0;
				}
				lc.to_line += 1;
				lc.to_char = 0;
				if char == '\r' {
					let _ = iter.next_if(|(_, c)| *c == '\n');
				}
			} else {
				if i < span.offset() {
					lc.from_char += 1;
				}
				lc.to_char += 1;
			}
		}
		lc
	}
}

/// A span in terms of line and character numbers
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
pub struct LineCharSpan {
	/// The line number where the span starts
	pub from_line: u32,
	/// The character number where the span starts
	pub from_char: u32,
	/// The line number where the span ends
	pub to_line: u32,
	/// The character number where the span ends
	pub to_char: u32,
}

impl std::fmt::Display for LineCharSpan {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}", self.from_line + 1, self.from_char + 1)?;
		if self.from_line != self.to_line {
			write!(f, "-{}.{}", self.to_line + 1, self.to_char + 1)?;
		} else if self.from_char != self.to_char {
			write!(f, "-{}", self.to_char + 1)?;
		}
		Ok(())
	}
}
