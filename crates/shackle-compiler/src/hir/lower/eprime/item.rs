use std::{collections::HashMap, iter};

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

/// Collects AST items into an HIR model
pub struct ItemCollector<'a> {
	db: &'a dyn Hir,
	identifiers: &'a IdentifierRegistry,
	model: Model,
	source_map: SourceMap,
	diagnostics: Vec<Error>,
	owner: ModelRef,
	branching_annotations: Option<eprime::MatrixLiteral>, // Used to store branching annotations
	goal: eprime::Goal,                                   // Used to store goal of solve
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
			branching_annotations: None,
			goal: eprime::Goal::Satisfy,
		}
	}

	/// Lower an AST item to HIR
	pub fn collect_item(&mut self, item: eprime::Item) {
		let (it, sm) = match item.clone() {
			eprime::Item::Constraint(c) => return self.collect_constraint(c),
			eprime::Item::ConstDefinition(_) => return,
			eprime::Item::DecisionDeclaration(d) => return self.collect_decision_declaration(d),
			eprime::Item::ParamDeclaration(p) => return self.collect_param_declaration(p),
			eprime::Item::DomainAlias(d) => return self.collect_domain_alias(d),
			eprime::Item::Solve(o) => {
				self.goal = o.goal().clone();
				return;
			}
			eprime::Item::Branching(b) => {
				self.branching_annotations = Some(b.branching_array());
				return;
			}
			eprime::Item::Heuristic(_) => return, // Currently not supported
			eprime::Item::Output(i) => self.collect_output(i),
		};
		self.source_map.insert(it.into(), Origin::new(&item));
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	/// Finish lowering
	pub fn finish(self) -> (Model, SourceMap, Vec<Error>) {
		(self.model, self.source_map, self.diagnostics)
	}

	/// Checks if a solve item exists, if not, adds satisfy solve
	/// TODO: Broken SourceMap
	pub fn add_solve(&mut self) {
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);

		let annotations = match &self.branching_annotations {
			Some(b) => {
				let origin = Origin::new(b);
				let arguments = Box::new([
					ctx.collect_matrix_literal(b.clone(), false),
					ctx.alloc_expression(origin.clone(), Identifier::new("input_order", self.db)),
					ctx.alloc_expression(origin.clone(), Identifier::new("indomain_min", self.db)),
				]);
				let function =
					ctx.alloc_expression(origin.clone(), Identifier::new("int_search", self.db));
				Box::new([ctx.alloc_expression(
					origin.clone(),
					Call {
						function,
						arguments,
					},
				)])
			}
			None => Box::new([]) as Box<[ArenaIndex<Expression>]>,
		};
		let goal = match &self.goal {
			eprime::Goal::Satisfy => Goal::Satisfy,
			eprime::Goal::Minimising(e) => Goal::Minimize {
				pattern: ctx.alloc_pattern(
					Origin::new(e),
					Pattern::Identifier(self.identifiers.objective),
				),
				objective: ctx.collect_expression(e.clone()),
			},
			eprime::Goal::Maximising(e) => Goal::Maximize {
				pattern: ctx.alloc_pattern(
					Origin::new(e),
					Pattern::Identifier(self.identifiers.objective),
				),
				objective: ctx.collect_expression(e.clone()),
			},
		};
		let (data, _) = ctx.finish();
		let index = self
			.model
			.solves
			.insert(Item::new(Solve { goal, annotations }, data));
		self.model
			.items
			.insert(self.model.items.len().saturating_sub(1), index.into());
		// let it = ItemRef::new(self.db, self.owner, index);
		// self.source_map.insert(it.into(), Origin::new(&goal));
		// self.source_map.add_from_item_data(self.db, it, &sm);
	}

	/// Collect a constant definition, if the constant has an index set coerce it into an array
	fn collect_const_definition(
		&mut self,
		c: eprime::ConstDefinition,
		idx: Option<&Vec<eprime::Domain>>,
	) {
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let assignee = ctx.collect_expression(c.name());
		let mut definition = ctx.collect_expression(c.definition());
		if let Some(indexes) = idx {
			let origin = Origin::new(&c);
			if indexes.len() > 6 {
				ctx.add_array_over_dims_diagnostic(c.clone());
			}
			let mut arguments: Vec<ArenaIndex<Expression>> = indexes
				.iter()
				.map(|d| {
					ctx.collect_domain_expressions(d.clone(), VarType::Par)
						.into_expression(&mut ctx, origin.clone())
				})
				.collect();
			arguments.push(definition);
			let function = ctx.ident_exp(origin, format!("array{}d", indexes.len()));
			definition = ctx.alloc_expression(
				Origin::new(&c),
				Call {
					function,
					arguments: arguments.into_boxed_slice(),
				},
			);
		};
		let (data, sm) = ctx.finish();
		let index = self.model.assignments.insert(Item::new(
			Assignment {
				assignee,
				definition,
			},
			data,
		));
		self.model.items.push(index.into());
		let it = ItemRef::new(self.db, self.owner, index);
		self.source_map.insert(it.into(), Origin::new(&c));
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	fn collect_param_declaration(&mut self, p: eprime::ParamDeclaration) {
		self.collect_declarations(p.names(), Some(p.domain()), false, None, VarType::Par);

		// Collect where expressions as constraints
		for w in p.wheres() {
			self.collect_constraint_expression(w);
		}
	}

	fn collect_decision_declaration(&mut self, d: eprime::DecisionDeclaration) {
		self.collect_declarations(d.names(), Some(d.domain()), false, None, VarType::Var);
	}

	fn collect_domain_alias(&mut self, d: eprime::DomainAlias) {
		// As per the specification domain alias function more as a declaration where the aliased
		// type is the definition as well as the declared type.
		// This approach is inefficient as domain is collected twice
		self.collect_declarations(
			iter::once(d.name()),
			Some(d.definition()),
			true,
			None,
			VarType::Par,
		);
	}

	fn collect_declarations<I: Iterator<Item = eprime::Identifier>>(
		&mut self,
		names: I,
		domain: Option<eprime::Domain>,
		domain_is_definition: bool, // Used for domain alias
		definition: Option<eprime::Expression>,
		var_type: VarType,
	) {
		for name in names {
			let origin = Origin::new(&name);
			let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
			let declared_type = domain
				.as_ref()
				.map(|d| ctx.collect_domain(d.clone(), var_type))
				.unwrap_or_else(|| ctx.alloc_type(origin.clone(), Type::Any));
			let pattern = ctx.alloc_ident_pattern(origin.clone(), name.clone());

			// If the domain is a domain alias create set type and assign definition
			let (definition, declared_type) = if domain_is_definition {
				(
					Some(
						ctx.collect_domain_expressions(domain.clone().unwrap(), VarType::Par)
							.into_expression(&mut ctx, origin.clone()),
					),
					ctx.alloc_type(
						origin.clone(),
						Type::Set {
							inst: VarType::Par,
							opt: OptType::NonOpt,
							element: declared_type,
						},
					),
				)
			} else {
				(
					// If the definition isn't a domain see if it is an expression
					definition
						.as_ref()
						.map(|d| ctx.collect_expression(d.clone())),
					declared_type,
				)
			};
			let (data, sm) = ctx.finish();
			let index = self.model.declarations.insert(Item::new(
				Declaration {
					declared_type,
					pattern,
					definition,
					annotations: Box::new([]),
				},
				data,
			));
			self.model.items.push(index.into());
			let it = ItemRef::new(self.db, self.owner, index);
			self.source_map.insert(it.into(), origin);
			self.source_map.add_from_item_data(self.db, it, &sm);
		}
	}

	fn collect_constraint(&mut self, c: eprime::Constraint) {
		for expr in c.expressions() {
			self.collect_constraint_expression(expr);
		}
	}

	fn collect_constraint_expression(&mut self, expr: eprime::Expression) {
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let expression = ctx.collect_expression(expr.clone());
		let (data, sm) = ctx.finish();
		let index = self.model.constraints.insert(Item::new(
			Constraint {
				annotations: Box::new([]),
				expression,
			},
			data,
		));
		self.model.items.push(index.into());
		let it = ItemRef::new(self.db, self.owner, index);
		self.source_map.insert(it.into(), Origin::new(&expr));
		self.source_map.add_from_item_data(self.db, it, &sm);
	}

	fn collect_output(&mut self, i: eprime::Output) -> (ItemRef, ItemDataSourceMap) {
		let mut ctx = ExpressionCollector::new(self.db, &mut self.diagnostics);
		let expression = ctx.collect_expression(i.expression());
		let (data, source_map) = ctx.finish();
		let index = self.model.outputs.insert(Item::new(
			Output {
				section: None,
				expression,
			},
			data,
		));
		self.model.items.push(index.into());
		(ItemRef::new(self.db, self.owner, index), source_map)
	}

	/// Preprocess the model to collect parameter index sets, and ensure constants are declared
	pub fn preprocess(&mut self, items: impl Iterator<Item = eprime::Item>) {
		let mut parameter_identifiers = Vec::new();
		let mut parameter_index_set_map = HashMap::new();
		for item in items {
			match item {
				eprime::Item::ParamDeclaration(p) => {
					for name in p.names() {
						let n = name.name().to_string();
						parameter_identifiers.push(n.clone());
						if let eprime::Domain::MatrixDomain(m) = p.domain() {
							parameter_index_set_map.insert(n, m.indexes().collect());
						}
					}
				}
				eprime::Item::ConstDefinition(c) => {
					// If the constant definition isn't a parameter assignment give it a declaration
					// Otherwise give it an assignment
					let name = match c.name() {
						eprime::Expression::Identifier(i) => i,
						_ => continue,
					};
					let name_str = &name.name().to_string();
					if !parameter_identifiers.contains(name_str) {
						self.collect_declarations(
							iter::once(name),
							c.domain(),
							false,
							Some(c.definition()),
							VarType::Par,
						);
					} else {
						self.collect_const_definition(c, parameter_index_set_map.get(name_str));
					}
				}
				_ => {}
			}
		}
	}
}
