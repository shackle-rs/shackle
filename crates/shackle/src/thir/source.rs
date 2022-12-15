//! Source mapping between THIR and HIR nodes.
//!
//! Tracks desugarings performed when lowering HIR to THIR.

use std::ops::{Deref, Index};

use rustc_hash::FxHashMap;

use crate::hir::ids::{EntityRef, ItemRef, NodeRef};

use super::ExpressionId;

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

/// Tracks HIR nodes which produced THIR nodes
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemOrigins {
	item: Origin,
	expressions: FxHashMap<ExpressionId, Origin>,
}

impl Index<ExpressionId> for ItemOrigins {
	type Output = Origin;
	fn index(&self, index: ExpressionId) -> &Self::Output {
		&self.expressions[&index]
	}
}

impl ItemOrigins {
	/// Create a new item origin table
	pub fn new(item: Origin) -> Self {
		Self {
			item,
			expressions: FxHashMap::default(),
		}
	}

	/// Insert an expression into the table
	pub fn insert(&mut self, expression: ExpressionId, origin: impl Into<Origin>) {
		self.expressions.insert(expression, origin.into());
	}
}
