//! Source mapping from HIR nodes to their locations
//!

use std::ops::Index;

use shackle_diagnostics::{SourceFile, SourceSpan};
use shackle_syntax::ast::AstNode;
pub use shackle_syntax::cst::Point;
use shackle_utils::{TypedIndex, arena::ArenaMap};

use crate::{
	Db, Expression, ExpressionId, Item, Pattern, PatternId, Type, TypeId,
	ids::{EntityId, EntityRef, ExpressionRef},
	input::ModelFile,
};

/// Map for storing entity origins
#[derive(Clone, Debug, Default, PartialEq, Eq, TypedIndex, salsa::SalsaValue)]
pub struct SourceMap<'db> {
	#[index(ExpressionId<'db>)]
	expressions: ArenaMap<Expression<'db>, Origin>,
	#[index(TypeId<'db>)]
	types: ArenaMap<Type<'db>, Origin>,
	#[index(PatternId<'db>)]
	patterns: ArenaMap<Pattern<'db>, Origin>,
}

impl<'db> SourceMap<'db> {
	/// Insert an origin for an entity into the map
	pub fn insert<'tree>(
		&mut self,
		_db: &'db dyn Db,
		file: ModelFile,
		entity: impl Into<EntityId<'db>>,
		ast: &impl AstNode<'tree>,
	) {
		let entity_id: EntityId<'db> = entity.into();
		let span = ast.cst_node().span();
		let origin = Origin::new(file, span);
		match entity_id {
			EntityId::Expression(x) => self.expressions.insert(x, origin),
			EntityId::Type(x) => self.types.insert(x, origin),
			EntityId::Pattern(x) => self.patterns.insert(x, origin),
		}
	}
}

impl<'db> Index<EntityId<'db>> for SourceMap<'db> {
	type Output = Origin;

	fn index(&self, index: EntityId<'db>) -> &Self::Output {
		match index {
			EntityId::Expression(x) => &self.expressions[x],
			EntityId::Type(x) => &self.types[x],
			EntityId::Pattern(x) => &self.patterns[x],
		}
	}
}

#[salsa::tracked]
fn sorted_leaf_nodes<'db>(db: &'db dyn Db, item: Item<'db>) -> Vec<(EntityId<'db>, usize, usize)> {
	let mut nodes = Vec::new();
	let sm = item.sources(db);
	for (e, origin) in sm.expressions.iter() {
		if item.data(db)[e].is_leaf() {
			nodes.push((e.into(), origin.span.offset(), origin.span.len()));
		}
	}
	for (t, origin) in sm.types.iter() {
		if item.data(db)[t].is_leaf() {
			nodes.push((t.into(), origin.span.offset(), origin.span.len()));
		}
	}
	for (p, origin) in sm.patterns.iter() {
		if item.data(db)[p].is_leaf() {
			nodes.push((p.into(), origin.span.offset(), origin.span.len()));
		}
	}
	nodes.sort_by_key(|(_, offset, _)| *offset);
	nodes
}

#[salsa::tracked]
fn expression_nodes<'db>(
	db: &'db dyn Db,
	item: Item<'db>,
) -> Vec<(ExpressionId<'db>, usize, usize)> {
	let mut nodes = Vec::new();
	let sm = item.sources(db);
	for (e, origin) in sm.expressions.iter() {
		nodes.push((e, origin.span.offset(), origin.span.len()));
	}
	nodes.sort_by(|(_, o1, l1), (_, o2, l2)| o1.cmp(o2).then(l2.cmp(l1)));
	nodes
}

/// Find the item that contains the given position
pub fn find_item<'db>(db: &'db dyn Db, file: ModelFile, byte_offset: usize) -> Option<Item<'db>> {
	let items = file.hir(db).items(db);
	if items.is_empty() {
		return None;
	}
	let item_idx =
		match items.binary_search_by_key(&byte_offset, |item| item.origin(db).span.offset()) {
			Ok(i) => i,
			Err(0) => return None,
			Err(i) => i - 1,
		};
	let item = items[item_idx];
	let item_span = item.origin(db).span;
	if byte_offset >= item_span.offset() + item_span.len() {
		return None;
	}
	Some(item)
}

/// Get the leaf nodes in the given model
#[salsa::tracked]
pub fn model_leaves<'db>(db: &'db dyn Db, model: ModelFile) -> Vec<EntityRef<'db>> {
	let mut result = Vec::new();
	for item in model.hir(db).items(db).iter() {
		for (leaf, _, _) in sorted_leaf_nodes(db, *item) {
			result.push(EntityRef::new(db, *item, leaf));
		}
	}
	result
}

/// Find a leaf HIR node from a given location in the source.
pub fn find_leaf<'db>(
	db: &'db dyn Db,
	file: ModelFile,
	byte_offset: usize,
) -> Option<EntityRef<'db>> {
	let item = find_item(db, file, byte_offset)?;
	let leaf_nodes = sorted_leaf_nodes(db, item);
	let leaf_end = leaf_nodes.partition_point(|(_, offset, _)| *offset <= byte_offset);
	let offset = leaf_nodes.get(leaf_end.checked_sub(1)?)?.1;
	let leaf_start = leaf_nodes.partition_point(|(_, o, _)| *o < offset);
	// Prefer the shortest containing origin, then an expression if the origins are identical.
	let (e, _, _) = leaf_nodes[leaf_start..leaf_end]
		.iter()
		.copied()
		.filter(|(_, offset, len)| byte_offset < offset + len)
		.min_by_key(|(entity, _, len)| (*len, !matches!(*entity, EntityId::Expression(_))))?;
	Some(EntityRef::new(db, item, e))
}

/// Find an expression from a given location in the source (not necessarily a leaf).
pub fn find_expression<'db>(
	db: &'db dyn Db,
	file: ModelFile,
	byte_offset: usize,
) -> Option<ExpressionRef<'db>> {
	let item = find_item(db, file, byte_offset)?;
	let expression_nodes = expression_nodes(db, item);
	let mut expr_idx =
		match expression_nodes.binary_search_by_key(&byte_offset, |(_, offset, _)| *offset) {
			Ok(i) => i,
			Err(0) => return None,
			Err(i) => i - 1,
		};
	while expr_idx < expression_nodes.len() - 1
		&& expression_nodes[expr_idx + 1].1 == expression_nodes[expr_idx].1
		&& byte_offset < expression_nodes[expr_idx + 1].1 + expression_nodes[expr_idx + 1].2
	{
		expr_idx += 1;
	}
	while expr_idx > 0 && byte_offset >= expression_nodes[expr_idx].1 + expression_nodes[expr_idx].2
	{
		expr_idx -= 1;
	}

	let (e, offset, len) = expression_nodes[expr_idx];

	if byte_offset >= offset + len {
		// Not actually within this expression
		return None;
	}

	Some(ExpressionRef::new(db, item, e))
}

/// Origin of an HIR node.
#[derive(Clone, PartialEq, Eq, salsa::SalsaValue, Hash)]
pub struct Origin {
	/// The file this construct is from
	pub file: ModelFile,

	/// The location of this construct
	pub span: SourceSpan,
}

impl Origin {
	/// Create a new origin with the given file and span
	pub fn new(file: ModelFile, span: SourceSpan) -> Self {
		Self { file, span }
	}

	/// Get the source and span of this location
	pub fn source_span(&self, db: &dyn Db) -> (SourceFile, SourceSpan) {
		(self.file.source_file(db), self.span)
	}

	/// Pretty-print this origin for diagnostics
	pub fn pretty_print(&self, db: &dyn Db) -> String {
		let (source, span) = self.source_span(db);
		let range = span_to_range(span, source.contents());
		format!("{}:{}", source.name(), range)
	}
}

impl std::fmt::Debug for Origin {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		crate::db::with_attached_database(|db| match self.file {
			ModelFile::Named(n) => {
				let range = span_to_range(self.span, n.contents(db).as_ref());
				write!(f, "{}:{}", n.path(db).display(), range)
			}
			ModelFile::Inline(i) => {
				let range = span_to_range(self.span, i.contents(db));
				write!(
					f,
					"{}:{}",
					i.name(db).as_deref().unwrap_or("<unnamed file>"),
					range
				)
			}
		})
		.unwrap_or_else(|| f.debug_struct("Origin").finish())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RowColSpan {
	from_line: usize,
	from_char: usize,
	to_line: usize,
	to_char: usize,
}

impl std::fmt::Display for RowColSpan {
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

fn span_to_range(span: SourceSpan, src: &str) -> RowColSpan {
	let mut from_line = 0_usize;
	let mut from_char = 0_usize;
	let mut to_line = 0_usize;
	let mut to_char = 0_usize;
	let mut iter = src[0..span.offset() + span.len()]
		.chars()
		.enumerate()
		.peekable();
	while let Some((i, char)) = iter.next() {
		if matches!(char, '\r' | '\n') {
			if i < span.offset() {
				from_line += 1;
				from_char = 0;
			}
			to_line += 1;
			to_char = 0;
			if char == '\r' {
				let _ = iter.next_if(|(_, c)| *c == '\n');
			}
		} else {
			if i < span.offset() {
				from_char += 1;
			}
			to_char += 1;
		}
	}
	RowColSpan {
		from_line,
		from_char,
		to_line,
		to_char,
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase,
		ids::EntityId,
		input::{CompilerSettings, InlineModelFile, InputFiles, ModelFile},
		source::{expression_nodes, find_expression, find_item, find_leaf, sorted_leaf_nodes},
	};

	fn setup_test(model: &str) -> (CompilerDatabase, ModelFile) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model]);
		(db, model)
	}

	fn check_leaf_nodes(item: &str, expected: Expect) {
		let (db, model) = setup_test(item);
		let it = model.hir(&db).items(&db)[0];
		let sm = it.sources(&db);
		let mut actual = String::new();
		for (leaf, o, l) in sorted_leaf_nodes(&db, it) {
			let (source, span) = sm[*leaf].source_span(&db);
			actual.push_str(&source.contents()[span.offset()..span.offset() + span.len()]);
			actual.push_str(&format!(" [{}, {}]\n", o, l));
		}
		expected.assert_eq(&actual);
	}

	fn check_expression_nodes(item: &str, expected: Expect) {
		let (db, model) = setup_test(item);
		let it = model.hir(&db).items(&db)[0];
		let sm = it.sources(&db);
		let mut actual = String::new();
		for (expr, o, l) in expression_nodes(&db, it) {
			let (source, span) = sm[*expr].source_span(&db);
			actual.push_str(&source.contents()[span.offset()..span.offset() + span.len()]);
			actual.push_str(&format!(" [{}, {}]\n", o, l));
		}
		expected.assert_eq(&actual);
	}

	fn check_find_item(model: &str, byte_offset: usize, expected: Expect) {
		let (db, model) = setup_test(model);
		let actual = find_item(&db, model, byte_offset)
			.map(|i| {
				let (source, span) = i.origin(&db).source_span(&db);
				source.contents()[span.offset()..span.offset() + span.len()].to_owned()
			})
			.unwrap_or_else(|| "<not found>".to_owned());
		expected.assert_eq(&actual);
	}

	fn check_find_leaf(model: &str, byte_offset: usize, expected: Expect) {
		let (db, model) = setup_test(model);
		let actual = find_leaf(&db, model, byte_offset)
			.map(|n| {
				let (source, span) = n.source_span(&db);
				source.contents()[span.offset()..span.offset() + span.len()].to_owned()
			})
			.unwrap_or_else(|| "<not found>".to_owned());
		expected.assert_eq(&actual);
	}

	fn check_find_expression(model: &str, byte_offset: usize, expected: Expect) {
		let (db, model) = setup_test(model);
		let actual = find_expression(&db, model, byte_offset)
			.map(|n| {
				let (source, span) = n.source_span(&db);
				source.contents()[span.offset()..span.offset() + span.len()].to_owned()
			})
			.unwrap_or_else(|| "<not found>".to_owned());
		expected.assert_eq(&actual);
	}

	#[test]
	fn test_leaf_nodes() {
		check_leaf_nodes(
			r#"var int: foo = a + 123;"#,
			expect!([r#"
    var int [0, 7]
    foo [9, 3]
    a [15, 1]
    + [17, 1]
    123 [19, 3]
"#]),
		);
	}

	#[test]
	fn test_expression_nodes() {
		check_expression_nodes(
			r#"var int: foo = a + 123 + b;"#,
			expect!([r#"
    a + 123 + b [15, 11]
    a + 123 [15, 7]
    a [15, 1]
    + [17, 1]
    123 [19, 3]
    + [23, 1]
    b [25, 1]
"#]),
		);
	}

	#[test]
	fn test_find_item() {
		let model = "var int: x; var int: y; var int: z;";
		check_find_item(model, 5, expect!["var int: x"]);
		check_find_item(model, 15, expect!["var int: y"]);
		check_find_item(model, 30, expect!["var int: z"]);
	}

	#[test]
	fn test_find_leaf() {
		let model = "var int: foo; var int: y = foo + 1;";
		check_find_leaf(model, 28, expect!["foo"]);
		check_find_leaf(model, 33, expect!["1"]);
		check_find_leaf(model, 3, expect!["var int"]);
	}

	#[test]
	fn test_find_leaf_2() {
		let model = "any: x = let { int: foo = 1; } in foo;";
		check_find_leaf(model, 1, expect!["any"]);
		check_find_leaf(model, 5, expect!["x"]);
		check_find_leaf(model, 9, expect!["<not found>"]);
		check_find_leaf(model, 15, expect!["int"]);
	}

	#[test]
	fn test_find_leaf_prefers_shortest_span() {
		let model = "int: foo; int: bar; solve minimize foo + bar;";
		check_find_leaf(model, 35, expect!["foo"]);
		check_find_leaf(model, 36, expect!["foo"]);
		check_find_leaf(model, 37, expect!["foo"]);
		check_find_leaf(model, 38, expect!["foo + bar"]);
	}

	#[test]
	fn test_find_leaf_prefers_expression_over_pattern() {
		let (db, model) = setup_test("int: foo; solve minimize foo;");
		assert!(matches!(
			find_leaf(&db, model, 26).map(|entity| entity.entity(&db)),
			Some(EntityId::Expression(_))
		));
	}

	#[test]
	fn test_find_expression() {
		let model = "var int: foo = a + 123 + b;";
		check_find_expression(model, 15, expect!["a"]);
		check_find_expression(model, 16, expect!["a + 123"]);
		check_find_expression(model, 19, expect!["123"]);
		check_find_expression(model, 22, expect!["a + 123 + b"]);
		check_find_expression(model, 25, expect!["b"]);
	}
}
