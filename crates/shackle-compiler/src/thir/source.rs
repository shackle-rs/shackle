//! Source mapping between THIR and HIR nodes.
//!
//! Tracks desugarings performed when lowering HIR to THIR.

use miette::SourceSpan;

use super::db::Thir;
use crate::{
	file::SourceFile,
	hir::ids::{EntityRef, ItemRef, NodeRef},
};

/// The HIR node which produced a THIR node
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Origin {
	/// Comes from a real HIR node
	HirNode(NodeRef),
	/// Is introduced, and does not have a location
	Introduced(&'static str),
}

impl From<NodeRef> for Origin {
	fn from(node: NodeRef) -> Self {
		Self::HirNode(node)
	}
}

impl From<ItemRef> for Origin {
	fn from(item: ItemRef) -> Self {
		NodeRef::from(item).into()
	}
}

impl From<EntityRef> for Origin {
	fn from(entity: EntityRef) -> Self {
		NodeRef::from(entity).into()
	}
}

impl Origin {
	/// Get the underlying HIR node
	pub fn node(&self) -> Option<NodeRef> {
		match self {
			Origin::HirNode(node) => Some(*node),
			_ => None,
		}
	}

	/// Get the source file and span of this origin
	pub fn source_span(&self, db: &dyn Thir) -> (SourceFile, SourceSpan) {
		match self {
			Origin::HirNode(node) => node.source_span(db.upcast()),
			Origin::Introduced(name) => (
				SourceFile::introduced(name),
				SourceSpan::new(0.into(), 0.into()),
			),
		}
	}

	/// Debug print this origin
	pub fn debug_print(&self, db: &dyn Thir) -> String {
		let (src, span) = self.source_span(db);
		let name = src.name().unwrap_or_else(|| "<unnamed file>".to_owned());
		format!("{}[{}:{}]", name, span.offset(), span.len(),)
	}
}
