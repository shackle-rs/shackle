//! Source mapping between HIR and AST nodes.
//!

use std::{fmt::Write, ops::Deref};

use rustc_hash::FxHashMap;
pub use tree_sitter::Point;

use crate::{
	file::FileRef,
	syntax::{
		ast::{self, AstNodeRef},
		cst::CstNode,
	},
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
	reverse: FxHashMap<CstNode, NodeRef>,
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
		self.map.insert(node.clone(), origin.clone());
		self.reverse
			.insert(origin.ast_node.cst_node().clone(), node);
	}

	/// Get the origin of the given node
	pub fn get_origin(&self, node: NodeRef) -> Option<&Origin> {
		self.map.get(&node)
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
		match source_map.reverse.get(&cst.node(node.clone())) {
			Some(r) => return Some(*r),
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

/// Type of desugaring that occurred.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DesugarKind {
	/// Indexed array literal
	IndexedArrayLiteral,
	/// 2D array literal
	ArrayLiteral2D,
	/// String interpolation
	StringInterpolation,
	/// Prefix operator
	PrefixOperator,
	/// Infix operator
	InfixOperator,
	/// Postfix operator
	PostfixOperator,
	/// Generator call
	GeneratorCall,
}

/// Origin of an HIR node.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Origin {
	desugar_kind: Option<DesugarKind>,
	ast_node: AstNodeRef,
}

impl Origin {
	/// Create an origin.
	pub fn new<T: Into<AstNodeRef>>(node: T, kind: Option<DesugarKind>) -> Self {
		Self {
			desugar_kind: kind,
			ast_node: node.into(),
		}
	}

	/// Clone this origin and assign the given desugaring kind.
	pub fn with_desugaring(&self, kind: DesugarKind) -> Self {
		Self {
			desugar_kind: Some(kind),
			ast_node: self.ast_node.clone(),
		}
	}
}

impl Deref for Origin {
	type Target = AstNodeRef;
	fn deref(&self) -> &Self::Target {
		&self.ast_node
	}
}

impl From<ast::Expression> for Origin {
	fn from(v: ast::Expression) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::Item> for Origin {
	fn from(v: ast::Item) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::Type> for Origin {
	fn from(v: ast::Type) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::Pattern> for Origin {
	fn from(v: ast::Pattern) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::LetItem> for Origin {
	fn from(v: ast::LetItem) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::Parameter> for Origin {
	fn from(v: ast::Parameter) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::Generator> for Origin {
	fn from(v: ast::Generator) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::IndexSlice> for Origin {
	fn from(v: ast::IndexSlice) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::EnumerationCase> for Origin {
	fn from(v: ast::EnumerationCase) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::TypeInstIdentifier> for Origin {
	fn from(v: ast::TypeInstIdentifier) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
impl From<ast::TypeInstEnumIdentifier> for Origin {
	fn from(v: ast::TypeInstEnumIdentifier) -> Self {
		Origin {
			desugar_kind: None,
			ast_node: v.into(),
		}
	}
}
