//! Source mapping between THIR and HIR nodes.
//!
//! Tracks desugarings performed when lowering HIR to THIR.

use std::ops::Deref;

use crate::hir::ids::{EntityRef, ItemRef, NodeRef};

/// The HIR node which produced a THIR node
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
	desugar_kind: DesugarKind,
	hir_node: NodeRef,
}

impl From<NodeRef> for Origin {
	fn from(node: NodeRef) -> Self {
		Self {
			desugar_kind: DesugarKind::None,
			hir_node: node,
		}
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

impl Deref for Origin {
	type Target = NodeRef;
	fn deref(&self) -> &Self::Target {
		&self.hir_node
	}
}

impl Origin {
	/// Create a copy of this origin with the given desugaring
	pub fn with_desugaring(self, kind: DesugarKind) -> Self {
		Self {
			desugar_kind: kind,
			hir_node: self.hir_node,
		}
	}
}

/// Desugaring from lowering HIR to THIR (or from THIR to THIR)
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum DesugarKind {
	/// Not desugared, direct lowering
	None,
	/// Desugaring infinite array slicing
	ArraySlice,
	/// Destructuring into separate items
	Destructuring,
	/// Desugared from objective value
	Objective,
}
