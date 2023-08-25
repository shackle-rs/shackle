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

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::IdentifierRegistry,
	hir::{Identifier, IntegerLiteral, OptType, StringLiteral, VarType},
	thir::{
		db::Thir,
		source::Origin,
		traverse::{
			add_declaration, add_function, fold_function_body, fold_let, Folder, ReplacementMap,
		},
		ArrayAccess, ArrayComprehension, ArrayLiteral, Constraint, Declaration, DeclarationId,
		Domain, DomainData, Expression, FunctionId, Generator, Item, Let, LetItem, LookupCall,
		Marker, Model, RecordAccess, RecordLiteral, TupleAccess, TupleLiteral,
	},
};

enum DomainConstraint<T: Marker> {
	Array(
		Origin,
		Option<IndexSet<T>>,
		Option<Box<DomainConstraint<T>>>,
	),
	Tuple(Origin, Vec<(IntegerLiteral, DomainConstraint<T>)>),
	Record(Origin, Vec<(Identifier, DomainConstraint<T>)>),
	Bound(DeclarationId<T>),
}

enum IndexSet<T: Marker> {
	OneDimensional(DeclarationId<T>),
	MutliDimensional(Vec<(IntegerLiteral, DeclarationId<T>)>),
}

struct DomainRewriter<Dst: Marker, Src: Marker = ()> {
	replacement_map: ReplacementMap<Dst, Src>,
	model: Model<Dst>,
	ids: Arc<IdentifierRegistry>,
	domain_constraints: FxHashMap<DeclarationId<Src>, DomainConstraint<Dst>>,
	return_constraints: FxHashMap<FunctionId<Src>, DomainConstraint<Dst>>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for DomainRewriter<Dst, Src> {
	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn add_variable_declaration(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		d: DeclarationId<Src>,
	) {
		if model[d].definition().is_none() && !model[d].ty().known_par(db.upcast()) {
			let array_of_opt = if let DomainData::Array(_, elem) = &**model[d].domain() {
				elem.ty().inst(db.upcast()) == Some(VarType::Var)
					&& elem.ty().opt(db.upcast()) == Some(OptType::Opt)
			} else {
				false
			};
			if array_of_opt
				|| model[d]
					.domain()
					.walk()
					.any(|d| matches!(&**d, DomainData::Tuple(_) | DomainData::Record(_)))
			{
				// Struct has no RHS, so unpack
				let definition = self.unpack_struct(db, model, model[d].domain());
				let idx = add_declaration(self, db, model, d);
				self.model[idx].set_definition(definition);
				let unbounded =
					Domain::unbounded(db, self.model[idx].origin(), self.model[idx].ty());
				self.model[idx].set_domain(unbounded);
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
						model[d]
							.name()
							.map(|n| n.pretty_print(db.upcast()))
							.unwrap_or_else(|| "<unnamed>".to_owned()),
						db.upcast(),
					),
				);
				let idx = add_declaration(self, db, model, d);
				let variable = Expression::new(db, &self.model, model[d].origin(), idx);
				let constraint =
					Constraint::new(true, self.make_domain_constraint(db, name, variable, dom));
				self.model
					.add_constraint(Item::new(constraint, model[d].origin()));
				let unbounded =
					Domain::unbounded(db, self.model[idx].origin(), self.model[idx].ty());
				self.model[idx].set_domain(unbounded);
				return;
			}
		}
		add_declaration(self, db, model, d);
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		let function = &model[f];
		for p in function.parameters() {
			if let Some(dom) = self.collect_domain_constraints(
				db,
				model,
				model[*p].domain(),
				&mut Vec::new(),
				true,
			) {
				self.domain_constraints.insert(*p, dom);
			}
		}
		if let Some(dom) =
			self.collect_domain_constraints(db, model, function.domain(), &mut Vec::new(), true)
		{
			self.return_constraints.insert(f, dom);
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

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
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
						model[*param]
							.name()
							.map(|n| n.pretty_print(db.upcast()))
							.unwrap_or_else(|| "<unnamed>".to_owned()),
						db.upcast(),
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
				StringLiteral::from(self.ids.return_value),
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

	fn fold_let(&mut self, db: &dyn Thir, model: &Model<Src>, l: &Let<Src>) -> Let<Dst> {
		let mut folded = fold_let(self, db, model, l);
		let folded_items = std::mem::take(&mut folded.items);
		folded.items.reserve(folded_items.len());
		for (folded_item, item) in folded_items.into_iter().zip(l.items.iter()) {
			if let (LetItem::Declaration(f), LetItem::Declaration(d)) = (folded_item, item) {
				let domain_constraint = if model[*d].definition().is_some() {
					self.collect_domain_constraints(
						db,
						model,
						model[*d].domain(),
						&mut folded.items,
						false,
					)
				} else {
					None
				};
				if let Some(dom) = domain_constraint {
					let name = Expression::new(
						db,
						&self.model,
						model[*d].origin(),
						StringLiteral::new(
							model[*d]
								.name()
								.map(|n| n.pretty_print(db.upcast()))
								.unwrap_or_else(|| "<unnamed>".to_owned()),
							db.upcast(),
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
			}
			folded.items.push(folded_item);
		}
		folded
	}
}

impl<Dst: Marker, Src: Marker> DomainRewriter<Dst, Src> {
	fn collect_domain_constraints(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
		decls: &mut Vec<LetItem<Dst>>,
		top_level: bool,
	) -> Option<DomainConstraint<Dst>> {
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
			DomainData::Set(elem) => {
				self.collect_domain_constraints(db, model, elem, decls, top_level)
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
		db: &dyn Thir,
		name: Expression<Dst>,
		variable: Expression<Dst>,
		domain_constraint: DomainConstraint<Dst>,
	) -> Expression<Dst> {
		match domain_constraint {
			DomainConstraint::Bound(e) => {
				let dom_origin = self.model[e].origin();
				let domain = Expression::new(db, &self.model, dom_origin, e);
				Expression::new(
					db,
					&self.model,
					dom_origin,
					LookupCall {
						function: self.ids.mzn_domain_constraint.into(),
						arguments: vec![name, variable, domain],
					},
				)
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
								function: self.ids.index_set.into(),
								arguments: vec![variable.clone()],
							},
						);
						constraints.push(Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.mzn_check_index_set.into(),
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
								function: self.ids.index_sets.into(),
								arguments: vec![variable.clone()],
							},
						);
						let dims = variable.ty().dims(db.upcast()).unwrap();
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
									function: self.ids.mzn_check_index_set.into(),
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
										function: self.ids.forall.into(),
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
							function: self.ids.index_sets.into(),
							arguments: vec![variable.clone()],
						},
					);
					let index_sets_decl = Declaration::from_expression(db, false, index_sets);
					let dim_count = variable.ty().dims(db.upcast()).unwrap();
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
							Domain::unbounded(
								db,
								origin,
								index_set.ty().elem_ty(db.upcast()).unwrap(),
							),
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
							function: self.ids.mzn_show_array_access.into(),
							arguments: vec![name, indices_expr.clone()],
						},
					);

					let element = Expression::new(
						db,
						&self.model,
						origin,
						ArrayAccess {
							collection: Box::new(variable),
							indices: Box::new(indices_expr),
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
							function: self.ids.forall.into(),
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
							function: self.ids.forall.into(),
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
							function: self.ids.mzn_show_tuple_access.into(),
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
							function: self.ids.forall.into(),
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
						StringLiteral::new(field.pretty_print(db.upcast()), db.upcast()),
					);
					let name = Expression::new(
						db,
						&self.model,
						name.origin(),
						LookupCall {
							function: self.ids.mzn_show_record_access.into(),
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
							function: self.ids.forall.into(),
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
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
	) -> Expression<Dst> {
		let mut let_items = Vec::new();
		let in_expression = self.unpack_struct_inner(db, model, domain, &mut let_items);
		if let_items.is_empty() {
			in_expression
		} else {
			Expression::new(
				db,
				&self.model,
				in_expression.origin(),
				Let {
					items: let_items,
					in_expression: Box::new(in_expression),
				},
			)
		}
	}

	fn unpack_struct_inner(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
		let_items: &mut Vec<LetItem<Dst>>,
	) -> Expression<Dst> {
		match &**domain {
			DomainData::Array(dim, elem)
				if matches!(
					&***elem,
					DomainData::Tuple(_) | DomainData::Record(_) | DomainData::Array(_, _)
				) || elem.ty().inst(db.upcast()) == Some(VarType::Var)
					&& elem.ty().opt(db.upcast()) == Some(OptType::Opt) =>
			{
				self.unpack_array(db, model, domain.origin(), dim, elem)
			}
			DomainData::Tuple(fields) => {
				let tuple = TupleLiteral(
					fields
						.iter()
						.map(|field| self.unpack_struct_inner(db, model, field, let_items))
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
								self.unpack_struct_inner(db, model, field, let_items),
							)
						})
						.collect(),
				);
				Expression::new(db, &self.model, domain.origin(), record)
			}
			_ => {
				let dom_decl = Declaration::new(false, self.fold_domain(db, model, domain));
				let dom_idx = self
					.model
					.add_declaration(Item::new(dom_decl, domain.origin()));
				let dom_expr = Expression::new(db, &self.model, domain.origin(), dom_idx);
				let_items.push(LetItem::Declaration(dom_idx));
				dom_expr
			}
		}
	}

	fn unpack_array(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		origin: Origin,
		dims: &Domain<Src>,
		elem: &Domain<Src>,
	) -> Expression<Dst> {
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
				Domain::unbounded(db, origin, collection.ty().elem_ty(db.upcast()).unwrap()),
			);
			let gen_idx = self.model.add_declaration(Item::new(gen_decl, origin));
			generators.push(Generator::Iterator {
				declarations: vec![gen_idx],
				collection,
				where_clause: None,
			});
		}

		let template = self.unpack_struct(db, model, elem);
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
		Expression::new(
			db,
			&self.model,
			origin,
			Let {
				items: vec![LetItem::Declaration(index_sets_idx)],
				in_expression: Box::new(Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.mzn_array_kd.into(),
						arguments: vec![index_sets_expr, comprehension],
					},
				)),
			},
		)
	}
}

/// Rewrite domains
pub fn rewrite_domains(db: &dyn Thir, model: Model) -> Model {
	log::info!("Rewriting domains into constraints and unpacking structured variables");
	let mut d = DomainRewriter {
		ids: db.identifier_registry(),
		model: Model::with_capacities(&model.entity_counts()),
		replacement_map: ReplacementMap::default(),
		domain_constraints: FxHashMap::default(),
		return_constraints: FxHashMap::default(),
	};
	d.add_model(db, &model);
	d.model
}

#[cfg(test)]
mod test {
	use crate::thir::transform::test::check;
	use expect_test::expect;

	use super::rewrite_domains;

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
      var '..'(1, 3): _DECL_1;
      var '..'(2, 4): _DECL_2;
    } in (_DECL_1, _DECL_2);
    record(var int: a, var int: b): y = let {
      var '..'(1, 3): _DECL_3;
      var '..'(2, 4): _DECL_4;
    } in (a: _DECL_3, b: _DECL_4);
    array [int] of tuple(var int, record(var int: a)): z = let {
      tuple(set of int): _DECL_5 = ('..'(1, 2),);
    } in mzn_array_kd(_DECL_5, [let {
      var '..'(1, 3): _DECL_7;
      var '..'(2, 4): _DECL_8;
    } in (_DECL_7, (a: _DECL_8)) | _DECL_6 in _DECL_5.1]);
    array [int, int] of tuple(var int, var int): w = let {
      tuple(set of int, set of int): _DECL_9 = ('..'(1, 2), '..'(2, 3));
    } in mzn_array_kd(_DECL_9, [let {
      var '..'(1, 3): _DECL_12;
      var '..'(2, 4): _DECL_13;
    } in (_DECL_12, _DECL_13) | _DECL_10 in _DECL_9.1, _DECL_11 in _DECL_9.2]);
    var int: v = let {
      tuple(var int): m = let {
      var '..'(1, 2): _DECL_14;
    } in (_DECL_14,);
      record(var int: n): o = let {
      var '..'(1, 3): _DECL_15;
    } in (n: _DECL_15);
    } in m.1;
    array [int] of var opt int: p = let {
      tuple(set of int): _DECL_16 = ('..'(1, 2),);
    } in mzn_array_kd(_DECL_16, [let {
      var opt '..'(1, 2): _DECL_18;
    } in _DECL_18 | _DECL_17 in _DECL_16.1]);
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
    constraint forall([mzn_domain_constraint(mzn_show_tuple_access("x", 1), x.1, _DECL_1), mzn_domain_constraint(mzn_show_tuple_access("x", 2), x.2, _DECL_2)]);
    set of int: _DECL_3 = '..'(1, 3);
    set of int: _DECL_4 = '..'(2, 4);
    tuple(int, int): y;
    constraint forall([mzn_domain_constraint(mzn_show_tuple_access("y", 1), y.1, _DECL_3), mzn_domain_constraint(mzn_show_tuple_access("y", 2), y.2, _DECL_4)]);
    set of int: _DECL_5 = '..'(1, 2);
    record(int: a): z;
    constraint mzn_domain_constraint(mzn_show_record_access("z", "a"), z.a, _DECL_5);
    set of int: _DECL_6 = '..'(1, 3);
    function var bool: foo(var int: x) = let {
      constraint mzn_domain_constraint("x", x, _DECL_6);
    } in true;
    set of int: _DECL_7 = '..'(1, 2);
    array [int] of int: a;
    constraint forall([mzn_domain_constraint(mzn_show_array_access("a", _DECL_9), (a)[_DECL_9], _DECL_7) | _DECL_8 = index_sets(a), _DECL_9 in _DECL_8.1]);
    set of int: _DECL_10 = '..'(1, 2);
    array [int] of int: b;
    constraint mzn_check_index_set("b", index_set(b), _DECL_10);
    set of int: _DECL_11 = '..'(1, 2);
    set of int: _DECL_12 = '..'(1, 2);
    array [int] of int: c;
    constraint forall([mzn_check_index_set("c", index_set(c), _DECL_11), forall([mzn_domain_constraint(mzn_show_array_access("c", _DECL_14), (c)[_DECL_14], _DECL_12) | _DECL_13 = index_sets(c), _DECL_14 in _DECL_13.1])]);
    set of int: _DECL_15 = '..'(1, 2);
    set of int: _DECL_16 = '..'(1, 2);
    array [int, int] of int: d;
    constraint let {
      tuple(set of int, set of int): _DECL_17 = index_sets(d);
    } in forall([mzn_check_index_set("d", 1, 2, _DECL_17.1, _DECL_15), mzn_check_index_set("d", 2, 2, _DECL_17.2, _DECL_16)]);
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
    array ['..'(1, 2)] of var '..'(1, 3): c;
    array ['..'(1, 2)] of var set of '..'(1, 3): d;
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
}
