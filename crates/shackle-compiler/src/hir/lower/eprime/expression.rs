use crate::{
	constants::IdentifierRegistry,
	hir::{db::Hir, source::Origin, *},
	syntax::{ast::AstNode, eprime},
	utils::arena::ArenaIndex,
	Error,
};

pub struct ExpressionCollector<'a> {
	db: &'a dyn Hir,
	identifiers: &'a IdentifierRegistry,
	data: ItemData,
	source_map: ItemDataSourceMap,
	diagnostics: &'a mut Vec<Error>,
}

impl ExpressionCollector<'_> {
	/// Create a new expression collector
	pub fn new<'a>(
		db: &'a dyn Hir,
		identifiers: &'a IdentifierRegistry,
		diagnostics: &'a mut Vec<Error>,
	) -> ExpressionCollector<'a> {
		ExpressionCollector {
			db,
			identifiers,
			data: ItemData::new(),
			source_map: ItemDataSourceMap::new(),
			diagnostics,
		}
	}

	pub fn collect_expression(&mut self, expression: eprime::Expression) -> ArenaIndex<Expression> {
		let origin = Origin::new(&expression);
		if expression.is_missing() {
			return self.alloc_expression(origin, Expression::Missing);
		}
		let collected: Expression = match expression {
			eprime::Expression::BooleanLiteral(b) => BooleanLiteral(b.value()).into(),
			// eprime::Expression::Call(c) => ,
			eprime::Expression::Identifier(i) => Identifier::new(i.name(), self.db).into(),
			// eprime::Expression::IndexedAccess(i) => ,
			// eprime::Expression::InfixOperator(o) => ,
			eprime::Expression::IntegerLiteral(i) => IntegerLiteral(i.value()).into(),
			// eprime::Expression::MatrixLiteral(m) => ,
			// eprime::Expression::PrefixOperator(o) => ,
			// eprime::Expression::Quantification(q) => ,
			// eprime::Expression::MatrixComprehension(m) => ,
			// eprime::Expression::AbsoluteOperator(a) => ,
			_ => unimplemented!("Expression not implemented"),
		};
		self.alloc_expression(origin, collected)
	}

	pub fn collect_domain(&mut self, t: eprime::Domain) -> ArenaIndex<Type> {
		unimplemented!("Domain not implemented");
	}

	/// Collect Identifier and return pattern type
	pub fn collect_identifier_pattern(&mut self, p: eprime::Identifier) -> ArenaIndex<Pattern> {
		let origin = Origin::new(&p);
		let identifier = Identifier::new(p.name(), self.db);
		self.alloc_pattern(origin, identifier)
	}

	/// Collect Identifier and return expression type
	pub fn collect_identifier_expression(
		&mut self,
		i: eprime::Identifier,
	) -> ArenaIndex<Expression> {
		let origin = Origin::new(&i);
		let identifier = Identifier::new(i.name(), self.db);
		self.alloc_expression(origin, identifier)
	}

	/// Get the collected expressions
	pub fn finish(mut self) -> (ItemData, ItemDataSourceMap) {
		self.data.shrink_to_fit();
		(self.data, self.source_map)
	}

	pub(super) fn alloc_expression<V: Into<Expression>>(
		&mut self,
		origin: Origin,
		v: V,
	) -> ArenaIndex<Expression> {
		let index = self.data.expressions.insert(v.into());
		self.source_map.expression_source.insert(index, origin);
		index
	}

	pub(super) fn alloc_type<V: Into<Type>>(&mut self, origin: Origin, v: V) -> ArenaIndex<Type> {
		let index = self.data.types.insert(v);
		self.source_map.type_source.insert(index, origin);
		index
	}

	pub(super) fn alloc_pattern<V: Into<Pattern>>(
		&mut self,
		origin: Origin,
		v: V,
	) -> ArenaIndex<Pattern> {
		let index = self.data.patterns.insert(v);
		self.source_map.pattern_source.insert(index, origin);
		index
	}
}
