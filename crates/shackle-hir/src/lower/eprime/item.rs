use std::iter;

use shackle_diagnostics::SourceSpan;
use shackle_syntax::{ast::AstNode, eprime};
use shackle_utils::hash::Map;

use crate::{
	Db,
	constants::IdentifierRegistry,
	diagnostics::Diagnostics,
	input::{ModelFile, ModelFileContents},
	lower::eprime::ExpressionCollector,
	source::Origin,
	*,
};

/// Collects AST items into an HIR model
#[derive(Debug)]
pub struct ItemCollector<'db, 'tree, 'a> {
	db: &'db dyn Db,
	identifiers: &'db IdentifierRegistry<'db>,
	items: Vec<Item<'db>>,
	diagnostics: Diagnostics,
	file: ModelFile,
	branching_annotations: Option<eprime::MatrixLiteral<'tree>>, // Used to store branching annotations
	goal: eprime::Goal<'tree>,                                   // Used to store goal of solve
	text: ModelFileContents<'a, 'db>,
}

impl<'db: 'a, 'tree, 'a> ItemCollector<'db, 'tree, 'a> {
	/// Create a new item collector
	pub fn new(db: &'db dyn Db, file: ModelFile) -> Self {
		let identifiers = IdentifierRegistry::lookup(db);

		Self {
			db,
			identifiers,
			items: Vec::new(),
			diagnostics: Diagnostics::default(),
			file,
			branching_annotations: None,
			goal: eprime::Goal::Satisfy,
			text: file.contents(db),
		}
	}

	/// Lower an AST item to HIR
	pub fn collect_item(&mut self, item: &eprime::Item<'tree>) {
		log::debug!("Lowering {} to HIR", item.cst_kind());
		match item.clone() {
			eprime::Item::Constraint(c) => self.collect_constraint(&c),
			eprime::Item::ConstDefinition(_) => (),
			eprime::Item::DecisionDeclaration(d) => self.collect_decision_declaration(&d),
			eprime::Item::ParamDeclaration(p) => self.collect_param_declaration(&p),
			eprime::Item::DomainAlias(d) => self.collect_domain_alias(&d),
			eprime::Item::Solve(o) => {
				self.goal = o.goal().clone();
			}
			eprime::Item::Branching(b) => {
				self.branching_annotations = Some(b.branching_array());
			}
			eprime::Item::Heuristic(_) => (), // Currently not supported
			eprime::Item::Output(i) => self.collect_output(&i),
		}
	}

	/// Finish lowering
	pub fn finish(self) -> (Model<'db>, Diagnostics) {
		(Model::new(self.db, self.file, self.items), self.diagnostics)
	}

	/// Checks if a solve item exists, if not, adds satisfy solve
	/// TODO: Broken SourceMap
	pub fn add_solve(&mut self) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);

		let annotations: Box<[ExpressionId<'db>]> = match &self.branching_annotations {
			Some(b) => {
				let arguments = Box::new([
					ctx.collect_matrix_literal(b, false),
					ctx.alloc_expression(b, Identifier::new(self.db, "input_order")),
					ctx.alloc_expression(b, Identifier::new(self.db, "indomain_min")),
				]);
				let function = ctx.alloc_expression(b, Identifier::new(self.db, "int_search"));
				Box::new([ctx.alloc_expression(
					b,
					Call {
						kind: CallKind::Synthetic,
						function,
						arguments,
						named_arguments: Box::new([]),
					},
				)])
			}
			None => Box::new([]),
		};
		let goal = match &self.goal {
			eprime::Goal::Satisfy => Goal::Satisfy,
			eprime::Goal::Minimising(e) => Goal::Minimize {
				pattern: ctx
					.alloc_pattern(e, Pattern::Identifier(self.identifiers.names.objective)),
				objective: ctx.collect_expression(e),
			},
			eprime::Goal::Maximising(e) => Goal::Maximize {
				pattern: ctx
					.alloc_pattern(e, Pattern::Identifier(self.identifiers.names.objective)),
				objective: ctx.collect_expression(e),
			},
		};
		let (item, source_map) = ctx.finish(Solve { goal, annotations });
		self.items.insert(
			self.items.len().saturating_sub(1),
			SolveItem::new(
				self.db,
				item,
				source_map,
				Origin::new(self.file, SourceSpan::new(0.into(), 0)),
			)
			.into(),
		);
	}

	/// Collect a constant definition, if the constant has an index set coerce it into an array
	fn collect_const_definition(
		&mut self,
		c: &eprime::ConstDefinition<'tree>,
		idx: Option<&Vec<eprime::Domain<'tree>>>,
	) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let assignee = ctx.collect_expression(&c.name());
		let mut definition = ctx.collect_expression(&c.definition());
		if let Some(indexes) = idx {
			if indexes.len() > 6 {
				let _ = ctx.add_array_over_dims_diagnostic(c);
			}
			let mut arguments: Vec<_> = indexes
				.iter()
				.map(|d| {
					ctx.collect_domain_expressions(d, VarType::Par)
						.into_expression(&mut ctx, c)
				})
				.collect();
			arguments.push(definition);
			let function = ctx.ident_exp(c, format!("array{}d", indexes.len()));
			definition = ctx.alloc_expression(
				c,
				Call {
					kind: CallKind::Synthetic,
					function,
					arguments: arguments.into_boxed_slice(),
					named_arguments: Box::new([]),
				},
			);
		};
		let (data, sm) = ctx.finish(Assignment {
			assignee,
			definition,
		});
		self.items
			.push(AssignmentItem::new(self.db, data, sm, Origin::new(self.file, c.span())).into());
	}

	fn collect_param_declaration(&mut self, p: &eprime::ParamDeclaration<'tree>) {
		self.collect_declarations(p.names(), &Some(p.domain()), false, None, VarType::Par);

		// Collect where expressions as constraints
		for w in p.wheres() {
			self.collect_constraint_expression(&w);
		}
	}

	fn collect_decision_declaration(&mut self, d: &eprime::DecisionDeclaration<'tree>) {
		self.collect_declarations(d.names(), &Some(d.domain()), false, None, VarType::Var);
	}

	fn collect_domain_alias(&mut self, d: &eprime::DomainAlias<'tree>) {
		// As per the specification domain alias function more as a declaration where the aliased
		// type is the definition as well as the declared type.
		// This approach is inefficient as domain is collected twice
		self.collect_declarations(
			iter::once(d.name()),
			&Some(d.definition()),
			true,
			None,
			VarType::Par,
		);
	}

	fn collect_declarations<I: Iterator<Item = eprime::Identifier<'tree>>>(
		&mut self,
		names: I,
		domain: &Option<eprime::Domain<'tree>>,
		domain_is_definition: bool, // Used for domain alias
		definition: Option<eprime::Expression<'tree>>,
		var_type: VarType,
	) {
		for name in names {
			let mut ctx = ExpressionCollector::new(
				self.db,
				self.file,
				self.text.as_ref(),
				&mut self.diagnostics,
			);
			let declared_type = domain
				.as_ref()
				.map(|d| ctx.collect_domain(d, var_type))
				.unwrap_or_else(|| ctx.alloc_type(&name, Type::Any));
			let pattern = ctx.alloc_ident_pattern(&name, name.clone());

			// If the domain is a domain alias create set type and assign definition
			let (definition, declared_type) = if domain_is_definition {
				let d = domain.as_ref().unwrap();
				(
					Some(
						ctx.collect_domain_expressions(d, VarType::Par)
							.into_expression(&mut ctx, d),
					),
					ctx.alloc_type(
						d,
						Type::Set {
							inst: VarType::Par,
							opt: OptType::NonOpt,
							cardinality: None,
							element: declared_type,
						},
					),
				)
			} else {
				(
					// If the definition isn't a domain see if it is an expression
					definition.as_ref().map(|d| ctx.collect_expression(d)),
					declared_type,
				)
			};
			let (data, sm) = ctx.finish(Declaration {
				declared_type,
				pattern,
				definition,
				annotations: Box::new([]),
			});
			self.items.push(
				DeclarationItem::new(self.db, data, sm, None, Origin::new(self.file, name.span()))
					.into(),
			);
		}
	}

	fn collect_constraint(&mut self, c: &eprime::Constraint<'tree>) {
		for expr in c.expressions() {
			self.collect_constraint_expression(&expr);
		}
	}

	fn collect_constraint_expression(&mut self, expr: &eprime::Expression<'tree>) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let expression = ctx.collect_expression(expr);
		let (data, sm) = ctx.finish(Constraint {
			annotations: Box::new([]),
			expression,
		});
		self.items.push(
			ConstraintItem::new(self.db, data, sm, Origin::new(self.file, expr.span())).into(),
		);
	}

	fn collect_output(&mut self, i: &eprime::Output<'tree>) {
		let mut ctx = ExpressionCollector::new(
			self.db,
			self.file,
			self.text.as_ref(),
			&mut self.diagnostics,
		);
		let expression = ctx.collect_expression(&i.expression());
		let (data, source_map) = ctx.finish(Output {
			section: None,
			expression,
		});
		self.items.push(
			OutputItem::new(self.db, data, source_map, Origin::new(self.file, i.span())).into(),
		);
	}

	/// Preprocess the model to collect parameter index sets, and ensure constants are declared
	pub fn preprocess(&mut self, items: impl Iterator<Item = eprime::Item<'tree>>) {
		let mut parameter_identifiers = Vec::new();
		let mut parameter_index_set_map = Map::default();
		for item in items {
			match item {
				eprime::Item::ParamDeclaration(p) => {
					for name in p.names() {
						let n = name.name(&self.file.contents(self.db)).to_owned();
						parameter_identifiers.push(n.clone());
						if let eprime::Domain::MatrixDomain(m) = p.domain() {
							let _ = parameter_index_set_map.insert(n, m.indexes().collect());
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
					let name_str = &name.name(&self.file.contents(self.db)).to_owned();
					if !parameter_identifiers.contains(name_str) {
						self.collect_declarations(
							iter::once(name),
							&c.domain(),
							false,
							Some(c.definition()),
							VarType::Par,
						);
					} else {
						self.collect_const_definition(&c, parameter_index_set_map.get(name_str));
					}
				}
				_ => {}
			}
		}
	}
}
