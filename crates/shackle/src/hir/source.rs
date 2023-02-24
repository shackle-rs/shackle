//! Source mapping between HIR and AST nodes.
//!

use std::fmt::Write;

use miette::SourceSpan;
use rustc_hash::FxHashMap;
use tree_sitter::Node;
pub use tree_sitter::Point;

use crate::{
	file::{FileRef, SourceFile},
	syntax::ast::*,
	utils::{debug_print_strings, DebugPrint},
};

use super::{
	db::Hir,
	ids::{EntityRef, ExpressionRef, ItemRef, LocalEntityRef, NodeRef},
	ItemDataSourceMap, Type,
};

/// Source mapping between HIR and AST nodes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceMap {
	map: FxHashMap<NodeRef, Origin>,
	reverse: FxHashMap<usize, NodeRef>,
}

impl<'a> DebugPrint<'a> for SourceMap {
	type Database = dyn Hir + 'a;
	fn debug_print(&self, db: &Self::Database) -> String {
		let mut w = String::new();
		writeln!(&mut w, "Source map:").unwrap();
		for (k, v) in self.map.iter() {
			writeln!(&mut w, "  {:?}: {:?}", k, v).unwrap();
		}
		debug_print_strings(db, &w)
	}
}

impl SourceMap {
	/// Insert into the source map
	pub fn insert(&mut self, node: NodeRef, origin: Origin) {
		self.map.insert(node, origin.clone());
		self.reverse.insert(origin.node_id, node);
	}

	/// Get the origin of the given node
	pub fn get_origin(&self, node: NodeRef) -> Option<&Origin> {
		self.map.get(&node)
	}

	/// Get the reference to the HIR node from the CST node ID
	pub fn find_node<'a>(&self, cst_node: impl Into<Node<'a>>) -> Option<NodeRef> {
		self.reverse.get(&cst_node.into().id()).copied()
	}

	/// Add entries for item data source map
	pub fn add_from_item_data(&mut self, db: &dyn Hir, item: ItemRef, sm: &ItemDataSourceMap) {
		for (k, v) in sm.expression_source.iter() {
			self.insert(EntityRef::new(db, item, k).into(), v.clone());
		}
		for (k, v) in sm.pattern_source.iter() {
			self.insert(EntityRef::new(db, item, k).into(), v.clone());
		}
		for (k, v) in sm.type_source.iter() {
			self.insert(EntityRef::new(db, item, k).into(), v.clone());
		}
	}
}

/// Find an HIR node from a location.
pub fn find_node(db: &dyn Hir, file: FileRef, start: Point, end: Point) -> Option<NodeRef> {
	let cst = db.cst(file).ok()?;
	let root = cst.root_node();
	let mut node = root.descendant_for_point_range(start, end)?;
	let source_map = db.lookup_source_map(file.into());
	loop {
		match source_map.find_node(node) {
			Some(r) => return Some(r),
			None => node = node.parent()?,
		}
	}
}

/// Find an HIR expression from a location.
pub fn find_expression(
	db: &dyn Hir,
	file: FileRef,
	start: Point,
	end: Point,
) -> Option<ExpressionRef> {
	find_node(db, file, start, end).and_then(|n| match n {
		NodeRef::Entity(e) => {
			let item = e.item(db);
			let entity = e.entity(db);
			match entity {
				LocalEntityRef::Expression(e) => Some(ExpressionRef::new(item, e)),
				LocalEntityRef::Type(t) => {
					let model = db.lookup_model(file.into());
					match &item.local_item_ref(db).data(model.as_ref())[t] {
						Type::Bounded { domain, .. } => Some(ExpressionRef::new(item, *domain)),
						_ => None,
					}
				}
				_ => None,
			}
		}
		_ => None,
	})
}

/// Origin of an HIR node.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
	file: FileRef,
	range: std::ops::Range<usize>,
	node_id: usize,
}

impl Origin {
	/// Create an origin.
	pub fn new<T: AstNode>(node: &T) -> Self {
		let node = node.cst_node();
		Self {
			file: node.cst().file(),
			range: node.as_ref().byte_range(),
			node_id: node.as_ref().id(),
		}
	}

	/// Get the source and span
	pub fn source_span(&self, db: &dyn Hir) -> (SourceFile, SourceSpan) {
		(
			SourceFile::new(self.file, db.upcast()),
			self.range.clone().into(),
		)
	}
}

impl<T: AstNode> From<&T> for Origin {
	fn from(node: &T) -> Self {
		Self::new(node)
	}
}
