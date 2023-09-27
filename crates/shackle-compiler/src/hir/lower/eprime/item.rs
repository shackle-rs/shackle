use crate::{
	constants::IdentifierRegistry,
	file::ModelRef,
	hir::{
		db::Hir,
		ids::ItemRef,
		lower::eprime::ExpressionCollector,
		source::{Origin, SourceMap},
		*,
	},
	syntax::eprime,
	Error,
};

pub struct ItemCollector<'a> {
	db: &'a dyn Hir,
	identifiers: &'a IdentifierRegistry,
	model: Model,
	source_map: SourceMap,
	diagnostics: Vec<Error>,
	owner: ModelRef,
}

impl ItemCollector<'_> {
	/// Create a new item collector
	pub fn new<'a>(
		db: &'a dyn Hir,
		identifiers: &'a IdentifierRegistry,
		owner: ModelRef,
	) -> ItemCollector<'a> {
		ItemCollector {
			db,
			identifiers,
			model: Model::default(),
			source_map: SourceMap::default(),
			diagnostics: Vec::new(),
			owner,
		}
	}

	/// Lower an AST item to HIR
	pub fn collect_item(&mut self, item: eprime::Item) {
		let (it, sm) = match item.clone() {
			// eprime::Item::Branching(b) => ,
			// eprime::Item::Constraint(c) => ,
			eprime::Item::ConstDefinition(c) => self.collect_const_definition(c),
			eprime::Item::DomainAlias(d) => self.collect_domain_alias(d),
			// eprime::Item::DecisionDeclaration(d) => ,
			// eprime::Item::Heuristic(h) => ,
			// eprime::Item::Objective(o) => ,
			// eprime::Item::ParamDeclaration(p) =>,
			_ => unimplemented!("Item not implemented"),
		};
		self.source_map.insert(it.into(), Origin::new(&item));
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	/// Finish lowering
	pub fn finish(self) -> (Model, SourceMap, Vec<Error>) {
		(self.model, self.source_map, self.diagnostics)
	}

	fn collect_const_definition(
		&mut self,
		c: eprime::ConstDefinition,
	) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let assignee = ctx.collect_identifier_expression(c.name());
		let definition = ctx.collect_expression(c.definition());
		let (data, source_map) = ctx.finish();
		let index = self.model.assignments.insert(Item::new(
			Assignment {
				assignee,
				definition,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	fn collect_constraint(&mut self, c: eprime::Constraint) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		unimplemented!("Constraint not implemented")
		// let expressions = c
		// 	.expressions();
		// 	// .map(|e| ctx.collect_expression(e))
		// 	// .collect()/;
		// for expr in expressions {
		// 	let expression = ctx.collect_expression(expr);
		// 	let (data, source_map) = ctx.finish();
		// 	let index = self.model.constraints.insert(Item::new(
		// 		Constraint {
		// 			annotations: Box::new([]),
		// 			expression,
		// 		},
		// 		data,
		// 	));
		// 	let it = ItemRef::new(self.db, self.owner, index);
		// 	self.source_map.insert(it.into(), Origin::new(&c));
		// 	self.source_map.add_from_item_data(self.db, it, &source_map);
		// }
	}

	fn collect_domain_alias(&mut self, d: eprime::DomainAlias) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, self.identifiers, &mut self.diagnostics);
		let name = ctx.collect_identifier_pattern(d.name());
		let aliased_type = ctx.collect_domain(d.definition());
		let (data, source_map) = ctx.finish();
		let index = self.model.type_aliases.insert(Item::new(
			TypeAlias {
				name,
				aliased_type,
				annotations: Box::new([]),
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}
}
