//! Turn domains into constraints/assertions
//! - Rewrite par domains into assertions
//! - Rewrite function parameter domains into constraints
//! - Unpack struct variables
//! - Rewrite struct domains into constraints
//! - Add index set checks
//!
//! This is done before type specialisation as the messages it produces need to call specialised
//! versions of `show` for erased types.
//!

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::{Identifier, IntegerLiteral, StringLiteral, constants::IdentifierRegistry};

use crate::{
	ArrayComprehension, ArrayLiteral, Constraint, Db, Declaration, DeclarationId, Domain,
	DomainData, Expression, FunctionId, Generator, Item, Let, LetItem, LookupCall, Marker, Model,
	RecordAccess, RecordLiteral, TupleAccess, TupleLiteral,
	source::Origin,
	traverse::{
		Folder, ReplacementMap, add_declaration, add_function, fold_domain, fold_function_body,
		fold_let,
	},
};

/// Whether erasing enums would silently lose this domain's finiteness: it
/// contains an `Unbounded` enum-typed leaf as a var scalar or as a var set's
/// element. Enum-typed domains are expected to carry their universe as a
/// `Bounded` expression — enum erasure replaces the type with plain `int`
/// without materialising any bound, so such a leaf on a declaration without a
/// definition becomes an unbounded `var int` / non-finite `var set of int` in
/// the emitted model. Used in debug assertions when unpacking free struct
/// variables (the position where every field domain ends up on an unassigned
/// fresh declaration).
#[cfg(debug_assertions)]
fn domain_loses_enum_bound<'db, T: Marker>(
	db: &'db dyn Db,
	domain: &Domain<'db, T>,
	elem_of_var_set: bool,
) -> bool {
	use shackle_ty::TyData;
	match &**domain {
		DomainData::Unbounded => {
			matches!(domain.ty().lookup(db), TyData::Enum(_, _, _))
				&& (elem_of_var_set || domain.ty().inst(db) == Some(shackle_hir::VarType::Var))
		}
		DomainData::Set(element, _) => domain_loses_enum_bound(
			db,
			element,
			domain.ty().inst(db) == Some(shackle_hir::VarType::Var),
		),
		DomainData::Array(_, element) => domain_loses_enum_bound(db, element, false),
		DomainData::Tuple(fields) => fields.iter().any(|f| domain_loses_enum_bound(db, f, false)),
		DomainData::Record(fields) => fields
			.iter()
			.any(|(_, f)| domain_loses_enum_bound(db, f, false)),
		DomainData::Bounded(_) => false,
	}
}

enum DomainConstraint<'db, T: Marker> {
	Array(
		Origin<'db>,
		Option<IndexSet<'db, T>>,
		Option<Box<DomainConstraint<'db, T>>>,
	),
	Tuple(Origin<'db>, Vec<(IntegerLiteral, DomainConstraint<'db, T>)>),
	Record(
		Origin<'db>,
		Vec<(Identifier<'db>, DomainConstraint<'db, T>)>,
	),
	Bound(DeclarationId<'db, T>),
	Set {
		domain: Option<DeclarationId<'db, T>>,
		card: DeclarationId<'db, T>,
	},
}

enum IndexSet<'db, T: Marker> {
	OneDimensional(DeclarationId<'db, T>),
	MutliDimensional(Vec<(IntegerLiteral, DeclarationId<'db, T>)>),
}

struct DomainRewriter<'db, Dst: Marker, Src: Marker = ()> {
	replacement_map: ReplacementMap<'db, Dst, Src>,
	model: Model<'db, Dst>,
	ids: &'db IdentifierRegistry<'db>,
	domain_constraints: FxHashMap<DeclarationId<'db, Src>, DomainConstraint<'db, Dst>>,
	return_constraints: FxHashMap<FunctionId<'db, Src>, DomainConstraint<'db, Dst>>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for DomainRewriter<'db, Dst, Src> {
	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn add_variable_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		d: DeclarationId<'db, Src>,
	) {
		if model[d].definition().is_none() && !model[d].ty().known_par(db) {
			if model[d].domain().walk().any(|d| {
				matches!(
					&**d,
					DomainData::Array { .. } | DomainData::Tuple(_) | DomainData::Record(_)
				)
			}) {
				// Struct has no RHS, so unpack
				let definition = self.unpack_struct(db, model, model[d].domain());
				let idx = add_declaration(self, db, model, d);
				self.model[idx].set_definition(definition);
				let unbounded =
					Domain::unbounded(db, self.model[idx].origin(), self.model[idx].ty());
				self.model[idx].set_domain(unbounded);
				return;
			}

			if model[d].top_level()
				&& let DomainData::Set(_, Some(e)) = &**model[d].domain()
			{
				// Extract the cardinality bound into a constraint; the folded
				// declaration keeps only the element domain.
				let card = self.fold_expression(db, model, e);
				let card_decl = Declaration::from_expression(db, true, card);
				let card_idx = self.model.add_declaration(Item::new(card_decl, e.origin()));
				let decl_id = add_declaration(self, db, model, d);
				let name = Expression::new(
					db,
					&self.model,
					model[d].origin(),
					StringLiteral::new(
						db,
						model[d]
							.name()
							.map(|n| n.pretty_print(db))
							.unwrap_or_else(|| "<unnamed>".to_owned()),
					),
				);
				let card_constraint = Expression::new(
					db,
					&self.model,
					e.origin(),
					LookupCall {
						function: self.ids.functions.mzn_card_constraint.into(),
						arguments: vec![
							name,
							Expression::new(db, &self.model, self.model[decl_id].origin(), decl_id),
							Expression::new(db, &self.model, e.origin(), card_idx),
						],
					},
				);
				let constraint = Constraint::new(true, card_constraint);
				let _ = self
					.model
					.add_constraint(Item::new(constraint, model[d].origin()));
				return;
			}
		} else if model[d].top_level() {
			// Ignore local declarations, they're processed separately
			if let Some(dom) =
				self.collect_domain_constraints(db, model, model[d].domain(), &mut Vec::new(), true)
			{
				// Has a right-hand side already, or is par, so create constraints for domains
				let name = Expression::new(
					db,
					&self.model,
					model[d].origin(),
					StringLiteral::new(
						db,
						model[d]
							.name()
							.map(|n| n.pretty_print(db))
							.unwrap_or_else(|| "<unnamed>".to_owned()),
					),
				);
				let idx = add_declaration(self, db, model, d);
				let variable = Expression::new(db, &self.model, model[d].origin(), idx);
				let constraint =
					Constraint::new(true, self.make_domain_constraint(db, name, variable, dom));
				let _ = self
					.model
					.add_constraint(Item::new(constraint, model[d].origin()));
				let unbounded =
					Domain::unbounded(db, self.model[idx].origin(), self.model[idx].ty());
				self.model[idx].set_domain(unbounded);
				return;
			}
		}
		let _ = add_declaration(self, db, model, d);
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		let function = &model[f];
		for p in function.parameters() {
			if let Some(dom) = self.collect_domain_constraints(
				db,
				model,
				model[*p].domain(),
				&mut Vec::new(),
				true,
			) {
				let _ = self.domain_constraints.insert(*p, dom);
			}
		}
		if let Some(dom) =
			self.collect_domain_constraints(db, model, function.domain(), &mut Vec::new(), true)
		{
			let _ = self.return_constraints.insert(f, dom);
		}
		let idx = add_function(self, db, model, f);
		let params = self.model[idx].parameters().to_vec();
		for p in params {
			let origin = self.model[p].domain().origin();
			let ty = self.model[p].domain().ty();
			self.model[p].set_domain(Domain::unbounded(db, origin, ty));
		}
		let unbounded_return = Domain::unbounded(
			db,
			self.model[idx].domain().origin(),
			self.model[idx].domain().ty(),
		);
		self.model[idx].set_domain(unbounded_return);
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		fold_function_body(self, db, model, f);
		let folded = self.fold_function_id(db, model, f);
		let mut let_items = Vec::new();
		for param in model[f].parameters() {
			if let Some(dom) = self.domain_constraints.remove(param) {
				let idx = self.fold_declaration_id(db, model, *param);
				let origin = model[*param].origin();
				let variable = Expression::new(db, &self.model, origin, idx);
				let name = Expression::new(
					db,
					&self.model,
					origin,
					StringLiteral::new(
						db,
						model[*param]
							.name()
							.map(|n| n.pretty_print(db))
							.unwrap_or_else(|| "<unnamed>".to_owned()),
					),
				);
				let constraint =
					Constraint::new(false, self.make_domain_constraint(db, name, variable, dom));
				let_items.push(LetItem::Constraint(
					self.model.add_constraint(Item::new(constraint, origin)),
				));
			}
		}
		let body = if let Some(dom) = self.return_constraints.remove(&f) {
			let return_value = self.model[folded]
				.take_body()
				.expect("Domain constraints cannot be added to function without body");
			let origin = return_value.origin();
			let decl = Declaration::from_expression(db, false, return_value);
			let idx = self.model.add_declaration(Item::new(decl, origin));
			let variable = Expression::new(db, &self.model, origin, idx);
			let name = Expression::new(
				db,
				&self.model,
				origin,
				StringLiteral::from(self.ids.literals.return_value),
			);
			let constraint = Constraint::new(
				false,
				self.make_domain_constraint(db, name, variable.clone(), dom),
			);
			let_items.push(LetItem::Declaration(idx));
			let_items.push(LetItem::Constraint(
				self.model.add_constraint(Item::new(constraint, origin)),
			));
			Some(variable)
		} else {
			None
		};
		if !let_items.is_empty() {
			let body = body.unwrap_or_else(|| {
				self.model[folded]
					.take_body()
					.expect("Domain constraints cannot be added to function without body")
			});
			let new_body = Expression::new(
				db,
				&self.model,
				body.origin(),
				Let {
					items: let_items,
					in_expression: Box::new(body),
				},
			);
			self.model[folded].set_body(new_body);
		}
	}

	fn fold_let(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		l: &Let<'db, Src>,
	) -> Let<'db, Dst> {
		let mut folded = fold_let(self, db, model, l);
		let folded_items = std::mem::take(&mut folded.items);
		folded.items.reserve(folded_items.len());
		for (folded_item, item) in folded_items.into_iter().zip(l.items.iter()) {
			if let (LetItem::Declaration(f), LetItem::Declaration(d)) = (folded_item, item) {
				let domain_constraint = model[*d].definition().and_then(|_| {
					self.collect_domain_constraints(
						db,
						model,
						model[*d].domain(),
						&mut folded.items,
						false,
					)
				});
				if let Some(dom) = domain_constraint {
					let name = Expression::new(
						db,
						&self.model,
						model[*d].origin(),
						StringLiteral::new(
							db,
							model[*d]
								.name()
								.map(|n| n.pretty_print(db))
								.unwrap_or_else(|| "<unnamed>".to_owned()),
						),
					);
					let variable = Expression::new(db, &self.model, model[*d].origin(), f);
					let constraint = Constraint::new(
						false,
						self.make_domain_constraint(db, name, variable, dom),
					);
					let idx = self
						.model
						.add_constraint(Item::new(constraint, model[*d].origin()));
					folded.items.push(folded_item);
					folded.items.push(LetItem::Constraint(idx));
					let unbounded =
						Domain::unbounded(db, self.model[f].origin(), self.model[f].ty());
					self.model[f].set_domain(unbounded);
					continue;
				}
				if model[*d].definition().is_none()
					&& let DomainData::Set(_, Some(e)) = &**model[*d].domain()
				{
					// A local set variable with a cardinality bound: extract the
					// bound into a constraint alongside the declaration.
					let card = self.fold_expression(db, model, e);
					let card_decl = Declaration::from_expression(db, false, card);
					let card_idx = self.model.add_declaration(Item::new(card_decl, e.origin()));
					folded.items.push(LetItem::Declaration(card_idx));
					let name = Expression::new(
						db,
						&self.model,
						model[*d].origin(),
						StringLiteral::new(
							db,
							model[*d]
								.name()
								.map(|n| n.pretty_print(db))
								.unwrap_or_else(|| "<unnamed>".to_owned()),
						),
					);
					let card_constraint = Expression::new(
						db,
						&self.model,
						e.origin(),
						LookupCall {
							function: self.ids.functions.mzn_card_constraint.into(),
							arguments: vec![
								name,
								Expression::new(db, &self.model, self.model[f].origin(), f),
								Expression::new(db, &self.model, e.origin(), card_idx),
							],
						},
					);
					let constraint = Constraint::new(false, card_constraint);
					let constraint_id = self
						.model
						.add_constraint(Item::new(constraint, model[*d].origin()));
					folded.items.push(folded_item);
					folded.items.push(LetItem::Constraint(constraint_id));
					continue;
				}
			}
			folded.items.push(folded_item);
		}
		folded
	}

	fn fold_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
	) -> Domain<'db, Dst> {
		// Cardinality bounds are rewritten into constraints at the declaration
		// sites, so folded set domains never keep one.
		if let DomainData::Set(elem, _) = &**domain {
			let elem = self.fold_domain(db, model, elem);
			Domain::set(
				db,
				domain.origin(),
				domain.ty().inst(db).unwrap(),
				domain.ty().opt(db).unwrap(),
				elem,
			)
		} else {
			fold_domain(self, db, model, domain)
		}
	}
}

impl<'db, Dst: Marker, Src: Marker> DomainRewriter<'db, Dst, Src> {
	fn collect_domain_constraints(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
		decls: &mut Vec<LetItem<'db, Dst>>,
		top_level: bool,
	) -> Option<DomainConstraint<'db, Dst>> {
		match &**domain {
			DomainData::Unbounded => None,
			DomainData::Bounded(e) => {
				let folded = self.fold_expression(db, model, e);
				let decl = Declaration::from_expression(db, top_level, folded);
				let idx = self.model.add_declaration(Item::new(decl, domain.origin()));
				if !top_level {
					decls.push(LetItem::Declaration(idx));
				}
				Some(DomainConstraint::Bound(idx))
			}
			DomainData::Array(dim, elem) => {
				let idx_set = match &***dim {
					DomainData::Bounded(d) => {
						let folded = self.fold_expression(db, model, d);
						let decl = Declaration::from_expression(db, top_level, folded);
						let idx = self.model.add_declaration(Item::new(decl, domain.origin()));
						if !top_level {
							decls.push(LetItem::Declaration(idx));
						}
						Some(IndexSet::OneDimensional(idx))
					}
					DomainData::Tuple(fields) => {
						let dims = fields
							.iter()
							.enumerate()
							.filter_map(|(i, f)| match &**f {
								DomainData::Bounded(d) => {
									let folded = self.fold_expression(db, model, d);
									let decl = Declaration::from_expression(db, top_level, folded);
									let idx = self
										.model
										.add_declaration(Item::new(decl, domain.origin()));
									if !top_level {
										decls.push(LetItem::Declaration(idx));
									}
									Some((IntegerLiteral(i as i64 + 1), idx))
								}
								_ => None,
							})
							.collect::<Vec<_>>();
						if dims.is_empty() {
							None
						} else {
							Some(IndexSet::MutliDimensional(dims))
						}
					}
					_ => None,
				};
				let elem_dom = self.collect_domain_constraints(db, model, elem, decls, top_level);
				if idx_set.is_none() && elem_dom.is_none() {
					None
				} else {
					Some(DomainConstraint::Array(
						domain.origin(),
						idx_set,
						elem_dom.map(Box::new),
					))
				}
			}
			DomainData::Set(elem, card) => {
				if let Some(c) = card {
					let folded = self.fold_expression(db, model, c);
					let decl = Declaration::from_expression(db, top_level, folded);
					let idx = self.model.add_declaration(Item::new(decl, c.origin()));
					if !top_level {
						decls.push(LetItem::Declaration(idx));
					}
					let dom_constr =
						self.collect_domain_constraints(db, model, elem, decls, top_level);
					Some(DomainConstraint::Set {
						domain: dom_constr.map(|d| match d {
							DomainConstraint::Bound(e) => e,
							_ => unreachable!(),
						}),
						card: idx,
					})
				} else {
					self.collect_domain_constraints(db, model, elem, decls, top_level)
				}
			}
			DomainData::Tuple(fields) => {
				let d = fields
					.iter()
					.enumerate()
					.filter_map(|(i, f)| {
						Some((
							IntegerLiteral(i as i64 + 1),
							self.collect_domain_constraints(db, model, f, decls, top_level)?,
						))
					})
					.collect::<Vec<_>>();
				if d.is_empty() {
					None
				} else {
					Some(DomainConstraint::Tuple(domain.origin(), d))
				}
			}
			DomainData::Record(fields) => {
				let d = fields
					.iter()
					.filter_map(|(i, f)| {
						Some((
							*i,
							self.collect_domain_constraints(db, model, f, decls, top_level)?,
						))
					})
					.collect::<Vec<_>>();
				if d.is_empty() {
					None
				} else {
					Some(DomainConstraint::Record(domain.origin(), d))
				}
			}
		}
	}

	fn make_domain_constraint(
		&mut self,
		db: &'db dyn Db,
		name: Expression<'db, Dst>,
		variable: Expression<'db, Dst>,
		domain_constraint: DomainConstraint<'db, Dst>,
	) -> Expression<'db, Dst> {
		match domain_constraint {
			DomainConstraint::Bound(e) => {
				let dom_origin = self.model[e].origin();
				let domain = Expression::new(db, &self.model, dom_origin, e);
				Expression::new(
					db,
					&self.model,
					dom_origin,
					LookupCall {
						function: self.ids.functions.mzn_domain_constraint.into(),
						arguments: vec![name, variable, domain],
					},
				)
			}
			DomainConstraint::Set { domain, card } => {
				let card_constraint = Expression::new(
					db,
					&self.model,
					self.model[card].origin(),
					LookupCall {
						function: self.ids.functions.mzn_card_constraint.into(),
						arguments: vec![
							name.clone(),
							variable.clone(),
							Expression::new(db, &self.model, self.model[card].origin(), card),
						],
					},
				);
				if let Some(e) = domain {
					let dom_origin = self.model[e].origin();
					let domain = Expression::new(db, &self.model, dom_origin, e);
					let dom_constraint = Expression::new(
						db,
						&self.model,
						dom_origin,
						LookupCall {
							function: self.ids.functions.mzn_domain_constraint.into(),
							arguments: vec![name, variable, domain],
						},
					);
					Expression::new(
						db,
						&self.model,
						dom_origin,
						LookupCall {
							function: self.ids.functions.and.into(),
							arguments: vec![card_constraint, dom_constraint],
						},
					)
				} else {
					card_constraint
				}
			}
			DomainConstraint::Array(origin, idx_set, dom) => {
				let mut constraints = Vec::with_capacity(2);
				match idx_set {
					Some(IndexSet::OneDimensional(e)) => {
						let origin = self.model[e].origin();
						let actual = Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.functions.index_set.into(),
								arguments: vec![variable.clone()],
							},
						);
						constraints.push(Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.functions.mzn_check_index_set.into(),
								arguments: vec![
									name.clone(),
									actual,
									Expression::new(db, &self.model, origin, e),
								],
							},
						));
					}
					Some(IndexSet::MutliDimensional(idx_sets)) => {
						let origin = variable.origin();
						let actual_index_sets = Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.functions.index_sets.into(),
								arguments: vec![variable.clone()],
							},
						);
						let dims = variable.ty().dims(db).unwrap();
						let decl = Declaration::from_expression(db, false, actual_index_sets);
						let idx = self.model.add_declaration(Item::new(decl, origin));
						let idx_expr = Expression::new(db, &self.model, origin, idx);
						let mut checks = Vec::with_capacity(dims);
						for (field, e) in idx_sets {
							let origin = self.model[e].origin();
							let actual = Expression::new(
								db,
								&self.model,
								idx_expr.origin(),
								TupleAccess {
									tuple: Box::new(idx_expr.clone()),
									field,
								},
							);
							checks.push(Expression::new(
								db,
								&self.model,
								origin,
								LookupCall {
									function: self.ids.functions.mzn_check_index_set.into(),
									arguments: vec![
										name.clone(),
										Expression::new(db, &self.model, origin, field),
										Expression::new(
											db,
											&self.model,
											origin,
											IntegerLiteral(dims as i64),
										),
										actual,
										Expression::new(db, &self.model, origin, e),
									],
								},
							));
						}
						constraints.push(Expression::new(
							db,
							&self.model,
							origin,
							Let {
								items: vec![LetItem::Declaration(idx)],
								in_expression: Box::new(Expression::new(
									db,
									&self.model,
									origin,
									LookupCall {
										function: self.ids.builtins.forall.into(),
										arguments: vec![Expression::new(
											db,
											&self.model,
											origin,
											ArrayLiteral(checks),
										)],
									},
								)),
							},
						));
					}
					_ => (),
				}
				if let Some(dom) = dom {
					let index_sets = Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.index_sets.into(),
							arguments: vec![variable.clone()],
						},
					);
					let index_sets_decl = Declaration::from_expression(db, false, index_sets);
					let dim_count = variable.ty().dims(db).unwrap();
					let index_sets_decl_idx = self
						.model
						.add_declaration(Item::new(index_sets_decl, origin));
					let index_sets_expr =
						Expression::new(db, &self.model, origin, index_sets_decl_idx);
					let mut generators = Vec::with_capacity(dim_count + 1);
					generators.push(Generator::Assignment {
						assignment: index_sets_decl_idx,
						where_clause: None,
					});
					let mut indices = Vec::with_capacity(dim_count);
					for i in 1..=dim_count {
						let index_set = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(index_sets_expr.clone()),
								field: IntegerLiteral(i as i64),
							},
						);
						let decl = Declaration::new(
							false,
							Domain::unbounded(db, origin, index_set.ty().elem_ty(db).unwrap()),
						);
						let idx = self.model.add_declaration(Item::new(decl, origin));
						let index = Expression::new(db, &self.model, origin, idx);
						indices.push(index);
						generators.push(Generator::Iterator {
							declarations: vec![idx],
							collection: index_set,
							where_clause: None,
						});
					}

					let indices_expr = if indices.len() == 1 {
						indices.pop().unwrap()
					} else {
						Expression::new(db, &self.model, origin, TupleLiteral(indices))
					};

					let name = Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.mzn_show_array_access.into(),
							arguments: vec![name, indices_expr.clone()],
						},
					);

					let element = Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.array_access.into(),
							arguments: vec![variable, indices_expr],
						},
					);
					let template = self.make_domain_constraint(db, name, element, *dom);

					let comprehension = Expression::new(
						db,
						&self.model,
						origin,
						ArrayComprehension {
							generators,
							indices: None,
							template: Box::new(template),
						},
					);
					constraints.push(Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.forall.into(),
							arguments: vec![comprehension],
						},
					));
				}
				if constraints.len() == 1 {
					constraints.pop().unwrap()
				} else {
					Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.forall.into(),
							arguments: vec![Expression::new(
								db,
								&self.model,
								origin,
								ArrayLiteral(constraints),
							)],
						},
					)
				}
			}
			DomainConstraint::Tuple(origin, fields) => {
				let mut constraints = Vec::with_capacity(fields.capacity());
				for (field, d) in fields {
					let accessor = Expression::new(
						db,
						&self.model,
						origin,
						TupleAccess {
							tuple: Box::new(variable.clone()),
							field,
						},
					);
					let field_expr = Expression::new(db, &self.model, name.origin(), field);
					let name = Expression::new(
						db,
						&self.model,
						name.origin(),
						LookupCall {
							function: self.ids.functions.mzn_show_tuple_access.into(),
							arguments: vec![name.clone(), field_expr],
						},
					);
					constraints.push(self.make_domain_constraint(db, name, accessor, d));
				}

				if constraints.len() == 1 {
					constraints.pop().unwrap()
				} else {
					Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.forall.into(),
							arguments: vec![Expression::new(
								db,
								&self.model,
								origin,
								ArrayLiteral(constraints),
							)],
						},
					)
				}
			}
			DomainConstraint::Record(origin, fields) => {
				let mut constraints = Vec::with_capacity(fields.capacity());
				for (field, d) in fields {
					let accessor = Expression::new(
						db,
						&self.model,
						origin,
						RecordAccess {
							record: Box::new(variable.clone()),
							field,
						},
					);
					let field_expr = Expression::new(
						db,
						&self.model,
						name.origin(),
						StringLiteral::new(db, field.pretty_print(db)),
					);
					let name = Expression::new(
						db,
						&self.model,
						name.origin(),
						LookupCall {
							function: self.ids.functions.mzn_show_record_access.into(),
							arguments: vec![name.clone(), field_expr],
						},
					);
					constraints.push(self.make_domain_constraint(db, name, accessor, d));
				}

				if constraints.len() == 1 {
					constraints.pop().unwrap()
				} else {
					Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.functions.forall.into(),
							arguments: vec![Expression::new(
								db,
								&self.model,
								origin,
								ArrayLiteral(constraints),
							)],
						},
					)
				}
			}
		}
	}

	fn unpack_struct(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
	) -> Expression<'db, Dst> {
		// Stores the declarations for evaluating the domain expressions
		let mut outer_let_items = Vec::new();
		// Stores the variable declarations (can't escape comprehensions)
		let mut inner_let_items = Vec::new();
		let in_expression = self.unpack_struct_inner(
			db,
			model,
			domain,
			&mut outer_let_items,
			&mut inner_let_items,
		);
		outer_let_items.extend(inner_let_items);
		if outer_let_items.is_empty() {
			in_expression
		} else {
			Expression::new(
				db,
				&self.model,
				in_expression.origin(),
				Let {
					items: outer_let_items,
					in_expression: Box::new(in_expression),
				},
			)
		}
	}

	fn unpack_struct_inner(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
		outer_let_items: &mut Vec<LetItem<'db, Dst>>,
		inner_let_items: &mut Vec<LetItem<'db, Dst>>,
	) -> Expression<'db, Dst> {
		match &**domain {
			DomainData::Array(dim, elem) => {
				self.unpack_array(db, model, domain.origin(), dim, elem, outer_let_items)
			}
			DomainData::Tuple(fields) => {
				let tuple = TupleLiteral(
					fields
						.iter()
						.map(|field| {
							self.unpack_struct_inner(
								db,
								model,
								field,
								outer_let_items,
								inner_let_items,
							)
						})
						.collect(),
				);
				Expression::new(db, &self.model, domain.origin(), tuple)
			}
			DomainData::Record(fields) => {
				let record = RecordLiteral(
					fields
						.iter()
						.map(|(ident, field)| {
							(
								*ident,
								self.unpack_struct_inner(
									db,
									model,
									field,
									outer_let_items,
									inner_let_items,
								),
							)
						})
						.collect(),
				);
				Expression::new(db, &self.model, domain.origin(), record)
			}
			DomainData::Set(_, Some(e)) => {
				// A set variable with a cardinality bound: declare the variable
				// with the element domain only and constrain its cardinality.
				let folded_domain = self.fold_domain(db, model, domain);
				#[cfg(debug_assertions)]
				debug_assert!(
					!domain_loses_enum_bound(db, &folded_domain, false),
					"unpacked free set variable's domain contains an unbounded enum \
					 position — enum erasure would lose its finiteness (at {})",
					domain.origin().pretty_print(db)
				);
				let dom_decl = Declaration::new(false, folded_domain);
				let dom_idx = self
					.model
					.add_declaration(Item::new(dom_decl, domain.origin()));
				let dom_expr = Expression::new(db, &self.model, domain.origin(), dom_idx);
				inner_let_items.push(LetItem::Declaration(dom_idx));

				let card = self.fold_expression(db, model, e);
				let card_decl = Declaration::from_expression(db, false, card);
				let card_idx = self.model.add_declaration(Item::new(card_decl, e.origin()));
				inner_let_items.push(LetItem::Declaration(card_idx));

				let name = Expression::new(
					db,
					&self.model,
					domain.origin(),
					StringLiteral::new(db, "<unnamed>"),
				);
				let card_constraint = Expression::new(
					db,
					&self.model,
					e.origin(),
					LookupCall {
						function: self.ids.functions.mzn_card_constraint.into(),
						arguments: vec![
							name,
							dom_expr.clone(),
							Expression::new(db, &self.model, e.origin(), card_idx),
						],
					},
				);
				let constraint = Constraint::new(false, card_constraint);
				let constraint_idx = self
					.model
					.add_constraint(Item::new(constraint, domain.origin()));
				inner_let_items.push(LetItem::Constraint(constraint_idx));
				dom_expr
			}
			_ => {
				let collected_domain = self.collect_domain(db, model, domain, outer_let_items);
				#[cfg(debug_assertions)]
				debug_assert!(
					!domain_loses_enum_bound(db, &collected_domain, false),
					"unpacked free struct field's domain contains an unbounded enum \
					 position — enum erasure would lose its finiteness (at {})",
					domain.origin().pretty_print(db)
				);
				let decl = Declaration::new(false, collected_domain);
				let idx = self.model.add_declaration(Item::new(decl, domain.origin()));
				let dom_expr = Expression::new(db, &self.model, domain.origin(), idx);
				inner_let_items.push(LetItem::Declaration(idx));
				dom_expr
			}
		}
	}

	fn collect_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
		let_items: &mut Vec<LetItem<'db, Dst>>,
	) -> Domain<'db, Dst> {
		let origin = domain.origin();
		let ty = domain.ty();
		match &**domain {
			DomainData::Bounded(e) => {
				let inst = ty.inst(db).unwrap();
				let opt = ty.opt(db).unwrap();
				let dom_decl =
					Declaration::from_expression(db, false, self.fold_expression(db, model, e));
				let dom_idx = self
					.model
					.add_declaration(Item::new(dom_decl, domain.origin()));
				let_items.push(LetItem::Declaration(dom_idx));
				Domain::bounded(
					db,
					origin,
					inst,
					opt,
					Expression::new(db, &self.model, origin, dom_idx),
				)
			}
			DomainData::Set(d, _) => {
				let inst = ty.inst(db).unwrap();
				let opt = ty.opt(db).unwrap();
				let element = self.collect_domain(db, model, d, let_items);
				Domain::set(db, origin, inst, opt, element)
			}
			_ => self.fold_domain(db, model, domain),
		}
	}

	fn unpack_array(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		origin: Origin<'db>,
		dims: &Domain<'db, Src>,
		elem: &Domain<'db, Src>,
		outer_let_items: &mut Vec<LetItem<'db, Dst>>,
	) -> Expression<'db, Dst> {
		let dims = match &**dims {
			DomainData::Bounded(e) => {
				vec![self.fold_expression(db, model, e)]
			}
			DomainData::Tuple(fs) => fs
				.iter()
				.map(|f| match &**f {
					DomainData::Bounded(e) => self.fold_expression(db, model, e),
					_ => unreachable!(),
				})
				.collect(),
			_ => unreachable!(),
		};

		let count = dims.len();
		let index_sets = Expression::new(db, &self.model, origin, TupleLiteral(dims));
		let index_sets_decl = Declaration::from_expression(db, false, index_sets);
		let index_sets_idx = self
			.model
			.add_declaration(Item::new(index_sets_decl, origin));
		let index_sets_expr = Expression::new(db, &self.model, origin, index_sets_idx);
		outer_let_items.push(LetItem::Declaration(index_sets_idx));

		let mut generators = Vec::with_capacity(count);
		for i in 1..=count {
			let collection = Expression::new(
				db,
				&self.model,
				origin,
				TupleAccess {
					tuple: Box::new(index_sets_expr.clone()),
					field: IntegerLiteral(i as i64),
				},
			);
			let gen_decl = Declaration::new(
				false,
				Domain::unbounded(db, origin, collection.ty().elem_ty(db).unwrap()),
			);
			let gen_idx = self.model.add_declaration(Item::new(gen_decl, origin));
			generators.push(Generator::Iterator {
				declarations: vec![gen_idx],
				collection,
				where_clause: None,
			});
		}

		let mut inner_let_items = Vec::new();
		let mut in_expression =
			self.unpack_struct_inner(db, model, elem, outer_let_items, &mut inner_let_items);
		let fresh_bool = in_expression.ty().is_bool(db);
		if fresh_bool {
			// Fresh bools can't return a bool here since that would make the let its own mixed boolean context
			in_expression = Expression::new(
				db,
				&self.model,
				in_expression.origin(),
				TupleLiteral(vec![in_expression]),
			);
		}
		let template = if inner_let_items.is_empty() {
			in_expression
		} else {
			Expression::new(
				db,
				&self.model,
				origin,
				Let {
					items: inner_let_items,
					in_expression: Box::new(in_expression),
				},
			)
		};
		let mut array = Expression::new(
			db,
			&self.model,
			origin,
			ArrayComprehension {
				generators,
				indices: None,
				template: Box::new(template),
			},
		);
		if fresh_bool {
			// Convert back to bool outside to not cause boolean context to transfer inside
			array = Expression::new(
				db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.functions.mzn_unwrap_bool_tuple.into(),
					arguments: vec![array],
				},
			);
		}
		Expression::new(
			db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.builtins.mzn_array_kd.into(),
				arguments: vec![index_sets_expr, array],
			},
		)
	}
}

/// Rewrite domains
pub fn rewrite_domains<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Rewriting domains into constraints and unpacking structured variables");
	let mut d = DomainRewriter {
		ids: IdentifierRegistry::lookup(db),
		model: Model::with_capacities(&model.item_counts()),
		replacement_map: ReplacementMap::default(),
		domain_constraints: FxHashMap::default(),
		return_constraints: FxHashMap::default(),
	};
	d.add_model(db, &model);
	Ok(d.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::rewrite_domains;
	use crate::transform::tests::check;

	#[test]
	fn test_rewrite_struct_domains() {
		check(
			rewrite_domains,
			r#"
				tuple(var 1..3, var 2..4): x;
                record(var 1..3: a, var 2..4: b): y;
                array [1..2] of tuple(var 1..3, record(var 2..4: a)): z;
                array [1..2, 2..3] of tuple(var 1..3, var 2..4): w;
                any: v = let {
                    tuple(var 1..2): m;
                    record(var 1..3: n): o;
                } in m.1;
				array [1..2] of var opt 1..2: p;
			"#,
			expect!([r#"
    tuple(var int, var int): x = let {
      set of int: _DECL_1 = '..'(1, 3);
      set of int: _DECL_2 = '..'(2, 4);
      var _DECL_1: _DECL_3;
      var _DECL_2: _DECL_4;
    } in (_DECL_3, _DECL_4);
    record(var int: a, var int: b): y = let {
      set of int: _DECL_5 = '..'(1, 3);
      set of int: _DECL_6 = '..'(2, 4);
      var _DECL_5: _DECL_7;
      var _DECL_6: _DECL_8;
    } in (a: _DECL_7, b: _DECL_8);
    array [int] of tuple(var int, record(var int: a)): z = let {
      tuple(set of int): _DECL_9 = ('..'(1, 2),);
      set of int: _DECL_10 = '..'(1, 3);
      set of int: _DECL_11 = '..'(2, 4);
    } in mzn_array_kd(_DECL_9, [let {
      var _DECL_10: _DECL_13;
      var _DECL_11: _DECL_14;
    } in (_DECL_13, (a: _DECL_14)) | _DECL_12 in (_DECL_9).1]);
    array [int, int] of tuple(var int, var int): w = let {
      tuple(set of int, set of int): _DECL_15 = ('..'(1, 2), '..'(2, 3));
      set of int: _DECL_16 = '..'(1, 3);
      set of int: _DECL_17 = '..'(2, 4);
    } in mzn_array_kd(_DECL_15, [let {
      var _DECL_16: _DECL_20;
      var _DECL_17: _DECL_21;
    } in (_DECL_20, _DECL_21) | _DECL_18 in (_DECL_15).1, _DECL_19 in (_DECL_15).2]);
    var int: v = let {
      tuple(var int): m = let {
      set of int: _DECL_22 = '..'(1, 2);
      var _DECL_22: _DECL_23;
    } in (_DECL_23,);
      record(var int: n): o = let {
      set of int: _DECL_24 = '..'(1, 3);
      var _DECL_24: _DECL_25;
    } in (n: _DECL_25);
    } in (m).1;
    array [int] of var opt int: p = let {
      tuple(set of int): _DECL_26 = ('..'(1, 2),);
      set of int: _DECL_27 = '..'(1, 2);
    } in mzn_array_kd(_DECL_26, [let {
      var opt _DECL_27: _DECL_29;
    } in _DECL_29 | _DECL_28 in (_DECL_26).1]);
"#]),
		)
	}

	#[test]
	fn test_rewrite_struct_domains_par() {
		check(
			rewrite_domains,
			r#"
				tuple(1..3, 2..4): x;
                record(1..3: a, 2..4: b): y;
                array [1..2] of tuple(1..3, record(2..4: a)): z;
                array [1..2, 2..3] of tuple(1..3, 2..4): w;
				array [1..2] of opt 1..2: p;
			"#,
			expect!([r#"
    set of int: _DECL_1 = '..'(1, 3);
    set of int: _DECL_2 = '..'(2, 4);
    tuple(int, int): x;
    constraint forall([mzn_domain_constraint(mzn_show_tuple_access("x", 1), (x).1, _DECL_1), mzn_domain_constraint(mzn_show_tuple_access("x", 2), (x).2, _DECL_2)]);
    set of int: _DECL_3 = '..'(1, 3);
    set of int: _DECL_4 = '..'(2, 4);
    record(int: a, int: b): y;
    constraint forall([mzn_domain_constraint(mzn_show_record_access("y", "a"), (y).a, _DECL_3), mzn_domain_constraint(mzn_show_record_access("y", "b"), (y).b, _DECL_4)]);
    set of int: _DECL_5 = '..'(1, 2);
    set of int: _DECL_6 = '..'(1, 3);
    set of int: _DECL_7 = '..'(2, 4);
    array [int] of tuple(int, record(int: a)): z;
    constraint forall([mzn_check_index_set("z", index_set(z), _DECL_5), forall([forall([mzn_domain_constraint(mzn_show_tuple_access(mzn_show_array_access("z", _DECL_9), 1), ('[]'(z, _DECL_9)).1, _DECL_6), mzn_domain_constraint(mzn_show_record_access(mzn_show_tuple_access(mzn_show_array_access("z", _DECL_9), 2), "a"), (('[]'(z, _DECL_9)).2).a, _DECL_7)]) | _DECL_8 = index_sets(z), _DECL_9 in (_DECL_8).1])]);
    set of int: _DECL_10 = '..'(1, 2);
    set of int: _DECL_11 = '..'(2, 3);
    set of int: _DECL_12 = '..'(1, 3);
    set of int: _DECL_13 = '..'(2, 4);
    array [int, int] of tuple(int, int): w;
    constraint forall([let {
      tuple(set of int, set of int): _DECL_14 = index_sets(w);
    } in forall([mzn_check_index_set("w", 1, 2, (_DECL_14).1, _DECL_10), mzn_check_index_set("w", 2, 2, (_DECL_14).2, _DECL_11)]), forall([forall([mzn_domain_constraint(mzn_show_tuple_access(mzn_show_array_access("w", (_DECL_16, _DECL_17)), 1), ('[]'(w, (_DECL_16, _DECL_17))).1, _DECL_12), mzn_domain_constraint(mzn_show_tuple_access(mzn_show_array_access("w", (_DECL_16, _DECL_17)), 2), ('[]'(w, (_DECL_16, _DECL_17))).2, _DECL_13)]) | _DECL_15 = index_sets(w), _DECL_16 in (_DECL_15).1, _DECL_17 in (_DECL_15).2])]);
    set of int: _DECL_18 = '..'(1, 2);
    set of int: _DECL_19 = '..'(1, 2);
    array [int] of opt int: p;
    constraint forall([mzn_check_index_set("p", index_set(p), _DECL_18), forall([mzn_domain_constraint(mzn_show_array_access("p", _DECL_21), '[]'(p, _DECL_21), _DECL_19) | _DECL_20 = index_sets(p), _DECL_21 in (_DECL_20).1])]);
"#]),
		)
	}

	#[test]
	fn test_domain_constraints() {
		check(
			rewrite_domains,
			r#"
				tuple(var 1..3, var 2..4): x = (1, 2);
				tuple(1..3, 2..4): y;
                record(1..2: a): z;
                predicate foo(var 1..3: x) = true;
                array [int] of 1..2: a;
                array [1..2] of int: b;
                array [1..2] of 1..2: c;
				array [1..2, 1..2] of int: d;
			"#,
			expect!([r#"
    set of int: _DECL_1 = '..'(1, 3);
    set of int: _DECL_2 = '..'(2, 4);
    tuple(var int, var int): x = (1, 2);
    constraint forall([mzn_domain_constraint(mzn_show_tuple_access("x", 1), (x).1, _DECL_1), mzn_domain_constraint(mzn_show_tuple_access("x", 2), (x).2, _DECL_2)]);
    set of int: _DECL_3 = '..'(1, 3);
    set of int: _DECL_4 = '..'(2, 4);
    tuple(int, int): y;
    constraint forall([mzn_domain_constraint(mzn_show_tuple_access("y", 1), (y).1, _DECL_3), mzn_domain_constraint(mzn_show_tuple_access("y", 2), (y).2, _DECL_4)]);
    set of int: _DECL_5 = '..'(1, 2);
    record(int: a): z;
    constraint mzn_domain_constraint(mzn_show_record_access("z", "a"), (z).a, _DECL_5);
    set of int: _DECL_6 = '..'(1, 3);
    function var bool: foo(var int: x) = let {
      constraint mzn_domain_constraint("x", x, _DECL_6);
    } in true;
    set of int: _DECL_7 = '..'(1, 2);
    array [int] of int: a;
    constraint forall([mzn_domain_constraint(mzn_show_array_access("a", _DECL_9), '[]'(a, _DECL_9), _DECL_7) | _DECL_8 = index_sets(a), _DECL_9 in (_DECL_8).1]);
    set of int: _DECL_10 = '..'(1, 2);
    array [int] of int: b;
    constraint mzn_check_index_set("b", index_set(b), _DECL_10);
    set of int: _DECL_11 = '..'(1, 2);
    set of int: _DECL_12 = '..'(1, 2);
    array [int] of int: c;
    constraint forall([mzn_check_index_set("c", index_set(c), _DECL_11), forall([mzn_domain_constraint(mzn_show_array_access("c", _DECL_14), '[]'(c, _DECL_14), _DECL_12) | _DECL_13 = index_sets(c), _DECL_14 in (_DECL_13).1])]);
    set of int: _DECL_15 = '..'(1, 2);
    set of int: _DECL_16 = '..'(1, 2);
    array [int, int] of int: d;
    constraint let {
      tuple(set of int, set of int): _DECL_17 = index_sets(d);
    } in forall([mzn_check_index_set("d", 1, 2, (_DECL_17).1, _DECL_15), mzn_check_index_set("d", 2, 2, (_DECL_17).2, _DECL_16)]);
"#]),
		)
	}

	#[test]
	fn test_non_domain_constraint() {
		check(
			rewrite_domains,
			r#"
				var 1..3: a;
				var set of 1..3: b;
				array [1..2] of var 1..3: c;
				array [1..2] of var set of 1..3: d;
				any: e = let {
					var 1..3: x;
				} in x;
			"#,
			expect!([r#"
    var '..'(1, 3): a;
    var set of '..'(1, 3): b;
    array [int] of var int: c = let {
      tuple(set of int): _DECL_1 = ('..'(1, 2),);
      set of int: _DECL_2 = '..'(1, 3);
    } in mzn_array_kd(_DECL_1, [let {
      var _DECL_2: _DECL_4;
    } in _DECL_4 | _DECL_3 in (_DECL_1).1]);
    array [int] of var set of int: d = let {
      tuple(set of int): _DECL_5 = ('..'(1, 2),);
      set of int: _DECL_6 = '..'(1, 3);
    } in mzn_array_kd(_DECL_5, [let {
      var set of _DECL_6: _DECL_8;
    } in _DECL_8 | _DECL_7 in (_DECL_5).1]);
    var int: e = let {
      var '..'(1, 3): x;
    } in x;
"#]),
		)
	}

	#[test]
	fn test_return_type_inst() {
		check(
			rewrite_domains,
			r#"
				function 1..2: foo(1..3: x) = x;
				function var 1..2: bar(var 1..3: x) = x;
			"#,
			expect!([r#"
    set of int: _DECL_1 = '..'(1, 3);
    set of int: _DECL_2 = '..'(1, 2);
    function int: foo(int: x) = let {
      constraint mzn_domain_constraint("x", x, _DECL_1);
      int: _DECL_3 = x;
      constraint mzn_domain_constraint("<return value>", _DECL_3, _DECL_2);
    } in _DECL_3;
    set of int: _DECL_4 = '..'(1, 3);
    set of int: _DECL_5 = '..'(1, 2);
    function var int: bar(var int: x) = let {
      constraint mzn_domain_constraint("x", x, _DECL_4);
      var int: _DECL_6 = x;
      constraint mzn_domain_constraint("<return value>", _DECL_6, _DECL_5);
    } in _DECL_6;
"#]),
		)
	}

	#[test]
	fn test_array_domain() {
		check(
			rewrite_domains,
			r#"
				array [1..3] of var int: x;
				array [1..3] of var 1..3: y;
				array [1..3] of tuple(var 1..2, var 3..4): z;
			"#,
			expect!([r#"
    array [int] of var int: x = let {
      tuple(set of int): _DECL_1 = ('..'(1, 3),);
    } in mzn_array_kd(_DECL_1, [let {
      var int: _DECL_3;
    } in _DECL_3 | _DECL_2 in (_DECL_1).1]);
    array [int] of var int: y = let {
      tuple(set of int): _DECL_4 = ('..'(1, 3),);
      set of int: _DECL_5 = '..'(1, 3);
    } in mzn_array_kd(_DECL_4, [let {
      var _DECL_5: _DECL_7;
    } in _DECL_7 | _DECL_6 in (_DECL_4).1]);
    array [int] of tuple(var int, var int): z = let {
      tuple(set of int): _DECL_8 = ('..'(1, 3),);
      set of int: _DECL_9 = '..'(1, 2);
      set of int: _DECL_10 = '..'(3, 4);
    } in mzn_array_kd(_DECL_8, [let {
      var _DECL_9: _DECL_12;
      var _DECL_10: _DECL_13;
    } in (_DECL_12, _DECL_13) | _DECL_11 in (_DECL_8).1]);
"#]),
		)
	}

	#[test]
	fn test_bool_vars() {
		check(
			rewrite_domains,
			r#"
				tuple(var bool): a;
				tuple(var bool, var bool): b;
				array [1..3] of var bool: c;
			"#,
			expect!([r#"
    tuple(var bool): a = let {
      var bool: _DECL_1;
    } in (_DECL_1,);
    tuple(var bool, var bool): b = let {
      var bool: _DECL_2;
      var bool: _DECL_3;
    } in (_DECL_2, _DECL_3);
    array [int] of var bool: c = let {
      tuple(set of int): _DECL_4 = ('..'(1, 3),);
    } in mzn_array_kd(_DECL_4, mzn_unwrap_bool_tuple([let {
      var bool: _DECL_6;
    } in (_DECL_6,) | _DECL_5 in (_DECL_4).1]));
"#]),
		)
	}

	#[test]
	fn test_set_card() {
		check(
			rewrite_domains,
			r#"
				var set(2..4) of 1..10: x;
				constraint let {
					var set(3..5) of 1..12: y;
				} in true;
				tuple(set(1..4) of 4..6): z1;
				tuple(var set(1..7) of 2..9): z2;
			"#,
			expect!([r#"
    set of int: _DECL_1 = '..'(2, 4);
    var set of '..'(1, 10): x;
    constraint mzn_card_constraint("x", x, _DECL_1);
    constraint let {
      set of int: _DECL_2 = '..'(3, 5);
      var set of '..'(1, 12): y;
      constraint mzn_card_constraint("y", y, _DECL_2);
    } in true;
    set of int: _DECL_3 = '..'(1, 4);
    set of int: _DECL_4 = '..'(4, 6);
    tuple(set of int): z1;
    constraint '/\'(mzn_card_constraint(mzn_show_tuple_access("z1", 1), (z1).1, _DECL_3), mzn_domain_constraint(mzn_show_tuple_access("z1", 1), (z1).1, _DECL_4));
    tuple(var set of int): z2 = let {
      var set of '..'(2, 9): _DECL_5;
      set of int: _DECL_6 = '..'(1, 7);
      constraint mzn_card_constraint("<unnamed>", _DECL_5, _DECL_6);
    } in (_DECL_5,);
"#]),
		)
	}
}
