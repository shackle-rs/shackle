//! Functionality for converting HIR nodes into THIR nodes.
//!
//! The following is performed during lowering:
//! - Assignment items are moved into declarations/constraints
//! - Destructuring declarations are rewritten as separate declarations
//! - Destructuring in generators is rewritten into a where clause
//! - Type alias items removed as they have been resolved
//!

use std::{
	ops::{Deref, DerefMut},
	sync::Arc,
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
	hir::{
		ids::{EntityRef, ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef, TypeRef},
		PatternTy, TypeResult,
	},
	ty::{OptType, OverloadedFunction, Ty, TyData, TypeRegistry, VarType},
};

use super::{
	db::Thir,
	source::{DesugarKind, Origin},
	*,
};

/// Collects HIR items and lowers them to THIR
pub struct ItemCollector<'a> {
	db: &'a dyn Thir,
	tys: &'a TypeRegistry,
	resolutions: FxHashMap<PatternRef, ResolvedIdentifier>,
	model: Model,
	type_alias_domains: FxHashMap<TypeRef, DeclarationId>,
	visited: FxHashSet<ItemRef>,
}

impl<'a> ItemCollector<'a> {
	/// Create a new item collector
	pub fn new(db: &'a dyn Thir, tys: &'a TypeRegistry) -> Self {
		Self {
			db,
			tys,
			resolutions: FxHashMap::default(),
			model: Model::default(),
			type_alias_domains: FxHashMap::default(),
			visited: FxHashSet::default(),
		}
	}

	/// Collect an item
	pub fn collect_item(&mut self, item: ItemRef) {
		if !self.visited.insert(item) {
			return;
		}
		let model = item.model(self.db.upcast());
		let local_item = item.local_item_ref(self.db.upcast());
		match local_item {
			LocalItemRef::Annotation(a) => {
				self.collect_annotation(item, &model[a]);
			}
			LocalItemRef::Assignment(a) => self.collect_assignment(item, &model[a]),
			LocalItemRef::Constraint(c) => {
				self.collect_constraint(item, &model[c], true);
			}
			LocalItemRef::Declaration(d) => {
				self.collect_declaration(item, &model[d], &model[d].data, true);
			}
			LocalItemRef::Enumeration(e) => {
				self.collect_enumeration(item, &model[e]);
			}
			LocalItemRef::EnumAssignment(a) => self.collect_enumeration_assignment(item, &model[a]),
			LocalItemRef::Function(f) => {
				self.collect_function(item, &model[f]);
			}
			LocalItemRef::Output(o) => {
				self.collect_output(item, &model[o]);
			}
			LocalItemRef::Solve(s) => self.collect_solve(item, &model[s]),
			LocalItemRef::TypeAlias(t) => self.collect_type_alias(item, &model[t]),
		}
	}

	/// Collect an annotation item
	pub fn collect_annotation(
		&mut self,
		item: ItemRef,
		a: &crate::hir::Item<crate::hir::Annotation>,
	) -> AnnotationId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &a.data, item, types.clone());
		let ty = &types[a.pattern];
		match ty {
			PatternTy::AnnotationAtom => {
				let annotation = AnnotationItem::new(
					a.data[a.pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
					item,
				);
				let idx = self.model.add_annotation(annotation);
				self.resolutions.insert(
					PatternRef::new(item, a.pattern),
					ResolvedIdentifier::Annotation(idx),
				);
				idx
			}
			PatternTy::AnnotationConstructor(fn_entry) => {
				let mut parameters = Vec::with_capacity(fn_entry.overload.params().len());
				for (param, ty) in a
					.parameters
					.as_ref()
					.unwrap()
					.iter()
					.zip(fn_entry.overload.params())
				{
					let domain = collector.collect_domain(param.declared_type, *ty, false);
					let mut declaration = DeclarationItem::new(&domain, false, item);
					// Ignore destructuring and recording resolution for now since these can't have bodies which refer
					// to parameters anyway
					if let Some(p) = param.pattern {
						declaration.name = a.data[p].identifier();
					}
					parameters.push(collector.model.add_declaration(declaration));
				}
				let mut annotation = AnnotationItem::new(
					a.data[a.pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
					item,
				);
				annotation.parameters = Some(parameters);
				let idx = self.model.add_annotation(annotation);
				self.resolutions.insert(
					PatternRef::new(item, a.pattern),
					ResolvedIdentifier::Annotation(idx),
				);
				idx
			}
			_ => unreachable!(),
		}
	}

	/// Collect an assignment item
	pub fn collect_assignment(
		&mut self,
		item: ItemRef,
		a: &crate::hir::Item<crate::hir::Assignment>,
	) {
		let db = self.db;
		let types = db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &a.data, item, types.clone());
		let res = types.name_resolution(a.assignee).unwrap();
		let def = collector.collect_expression(a.definition);
		if !collector.resolutions.contains_key(&res) {
			collector.collect_item(res.item());
		}
		let decl = match &collector.resolutions[&res] {
			ResolvedIdentifier::Declaration(d) => *d,
			_ => unreachable!(),
		};
		if collector.model[decl].definition.is_some() {
			// Turn subsequent assignment items into equality constraints
			let ids = db.identifier_registry();
			let fn_lookup = lookup_function(
				db,
				&collector.model,
				ids.eq,
				&[types[a.assignee], types[a.definition]],
			)
			.unwrap();
			let assignee = collector.collect_expression(a.assignee);
			let call = fn_lookup.into_call(db, item).with_args([assignee, def]);
			let constraint = ConstraintItem::new(&*call, true, item);
			collector.model.add_constraint(constraint);
		} else {
			collector.model[decl].set_definition(&*def);
		}
	}

	/// Collect a constraint item
	pub fn collect_constraint(
		&mut self,
		item: ItemRef,
		c: &crate::hir::Item<crate::hir::Constraint>,
		top_level: bool,
	) -> ConstraintId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &c.data, item, types);
		let mut constraint = ConstraintItem::new(
			&*collector.collect_expression(c.expression),
			top_level,
			item,
		);
		for ann in c.annotations.iter() {
			constraint.add_annotation(&*collector.collect_expression(*ann));
		}
		self.model.add_constraint(constraint)
	}

	/// Collect a declaration item
	pub fn collect_declaration(
		&mut self,
		item: ItemRef,
		d: &crate::hir::Declaration,
		data: &crate::hir::ItemData,
		top_level: bool,
	) -> Vec<DeclarationId> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, data, item, types.clone());
		let ty = &types[d.pattern];
		match ty {
			PatternTy::Variable(ty) => {
				let domain = collector.collect_domain(d.declared_type, *ty, false);
				let mut declaration = DeclarationItem::new(&domain, top_level, item);
				for ann in d.annotations.iter() {
					declaration.add_annotation(&*collector.collect_expression(*ann));
				}
				if let Some(def) = d.definition {
					declaration.set_definition(&*collector.collect_expression(def));
				}
				declaration.name = data[d.pattern].identifier();
				let decl = self.model.add_declaration(declaration);
				self.resolutions.insert(
					PatternRef::new(item, d.pattern),
					ResolvedIdentifier::Declaration(decl),
				);
				vec![decl]
			}
			PatternTy::Destructuring(ty) => {
				let domain = collector.collect_domain(d.declared_type, *ty, false);
				let mut declaration = DeclarationItem::new(
					&domain,
					top_level,
					Origin::from(item).with_desugaring(DesugarKind::Destructuring),
				);
				for ann in d.annotations.iter() {
					declaration.add_annotation(&*collector.collect_expression(*ann));
				}
				if let Some(def) = d.definition {
					declaration.set_definition(&*collector.collect_expression(def));
				}
				let decl = collector.model.add_declaration(declaration);
				let mut decls = vec![decl];
				let mut dc = DestructuringCollector {
					parent: &mut collector,
					decls: &mut decls,
					top_level,
				};
				dc.collect_destructuring(
					IdentifierBuilder::new(
						*ty,
						ResolvedIdentifier::Declaration(decl),
						EntityRef::new(dc.db.upcast(), item, d.pattern),
					),
					d.pattern,
				);
				decls
			}
			_ => unreachable!(),
		}
	}

	/// Collect an enumeration item
	pub fn collect_enumeration(
		&mut self,
		item: ItemRef,
		e: &crate::hir::Item<crate::hir::Enumeration>,
	) -> EnumerationId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &e.data, item, types.clone());
		let ty = &types[e.pattern];
		match ty {
			PatternTy::Variable(ty) => match ty.lookup(collector.db.upcast()) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(collector.db.upcast()) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = EnumerationItem::new(t, item);
							if let Some(def) = &e.definition {
								enumeration.definition = Some(
									def.iter().map(|c| collector.collect_enum_case(c)).collect(),
								)
							}
							let idx = self.model.add_enumeration(enumeration);
							self.resolutions.insert(
								PatternRef::new(item, e.pattern),
								ResolvedIdentifier::Enumeration(idx),
							);
							for (i, c) in e.definition.iter().flat_map(|d| d.iter()).enumerate() {
								self.resolutions.insert(
									PatternRef::new(item, c.pattern),
									ResolvedIdentifier::EnumerationMember(idx, i),
								);
							}
							idx
						}
						_ => unreachable!(),
					}
				}
				_ => unreachable!(),
			},
			_ => unreachable!(),
		}
	}

	/// Collect an enum assignment item
	pub fn collect_enumeration_assignment(
		&mut self,
		item: ItemRef,
		a: &crate::hir::Item<crate::hir::EnumAssignment>,
	) {
		let types = self.db.lookup_item_types(item);
		let res = types.name_resolution(a.assignee).unwrap();
		if !self.resolutions.contains_key(&res) {
			self.collect_item(res.item());
		}
		let mut collector = ExpressionCollector::new(self, &a.data, item, types);
		let e = match &collector.resolutions[&res] {
			ResolvedIdentifier::Enumeration(e) => *e,
			_ => unreachable!(),
		};
		collector.model[e].definition = Some(
			a.definition
				.iter()
				.map(|c| collector.collect_enum_case(c))
				.collect(),
		);
		for (i, c) in a.definition.iter().enumerate() {
			self.resolutions.insert(
				PatternRef::new(item, c.pattern),
				ResolvedIdentifier::EnumerationMember(e, i),
			);
		}
	}

	/// Collect a function item
	pub fn collect_function(
		&mut self,
		item: ItemRef,
		f: &crate::hir::Item<crate::hir::Function>,
	) -> FunctionId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &f.data, item, types.clone());
		let res = PatternRef::new(item, f.pattern);
		match &types[f.pattern] {
			PatternTy::Function(fn_entry) => {
				let domain =
					collector.collect_domain(f.return_type, fn_entry.overload.return_type(), false);

				let func =
					FunctionItem::new(f.data[f.pattern].identifier().unwrap(), &domain, item);
				let fn_idx = collector.model.add_function(func);
				collector
					.resolutions
					.insert(res, ResolvedIdentifier::Function(fn_idx));
				for ann in f.annotations.iter() {
					let a = collector.collect_expression(*ann);
					collector.model[fn_idx].add_annotation(&*a);
				}
				if let OverloadedFunction::PolymorphicFunction(pf) = &fn_entry.overload {
					collector.model[fn_idx]
						.type_inst_vars
						.extend(pf.ty_params.iter().copied());
				}
				let mut decls = Vec::new();

				let mut dc = DestructuringCollector {
					parent: &mut collector,
					decls: &mut decls,
					top_level: false,
				};
				for (param, ty) in f.parameters.iter().zip(fn_entry.overload.params()) {
					let domain = dc.collect_domain(param.declared_type, *ty, false);
					let mut declaration = DeclarationItem::new(&domain, false, item);
					if let Some(p) = param.pattern {
						declaration.name = f.data[p].identifier();
					}
					for ann in param.annotations.iter() {
						declaration.add_annotation(&*dc.collect_expression(*ann));
					}
					let idx = dc.model.add_declaration(declaration);
					dc.model[fn_idx].parameters.push(idx);
					if let Some(p) = param.pattern {
						match &types[p] {
							PatternTy::Variable(_) => {
								dc.resolutions.insert(
									PatternRef::new(item, p),
									ResolvedIdentifier::Declaration(idx),
								);
							}
							PatternTy::Destructuring(_) => {
								let ident = IdentifierBuilder::new(
									*ty,
									ResolvedIdentifier::Declaration(idx),
									Origin::from(
										PatternRef::new(item, p).into_entity(dc.db.upcast()),
									)
									.with_desugaring(DesugarKind::Destructuring),
								);
								dc.collect_destructuring(ident, p);
							}
							_ => unreachable!(),
						}
					}
				}
				if let Some(e) = f.body {
					let body = dc.collect_expression(e);
					if dc.decls.is_empty() {
						dc.model[fn_idx].set_body(&*body);
					} else {
						let builder = LetBuilder::new(
							types[e],
							Origin::from(ExpressionRef::new(item, e).into_entity(dc.db.upcast()))
								.with_desugaring(DesugarKind::Destructuring),
						)
						.with_items(dc.decls.iter().copied().map(LetItem::Declaration))
						.with_in(body);
						dc.model[fn_idx].set_body(&*builder);
					}
				}
				fn_idx
			}
			_ => unreachable!(),
		}
	}

	/// Collect an output item
	pub fn collect_output(
		&mut self,
		item: ItemRef,
		o: &crate::hir::Item<crate::hir::Output>,
	) -> OutputId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &o.data, item, types);
		let e = collector.collect_expression(o.expression);
		let mut output = OutputItem::new(&*e, item);
		if let Some(s) = o.section {
			let section = collector.collect_expression(s);
			output.set_section(&*section);
		}
		self.model.add_output(output)
	}

	/// Collect solve item
	pub fn collect_solve(&mut self, item: ItemRef, s: &crate::hir::Item<crate::hir::Solve>) {
		let mut solve = SolveItem::new(item);
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &s.data, item, types.clone());
		for ann in s.annotations.iter() {
			let a = collector.collect_expression(*ann);
			solve.add_annotation(&*a);
		}
		match &s.goal {
			crate::hir::Goal::Maximize { pattern, objective } => match &types[*pattern] {
				PatternTy::Variable(ty) => {
					let mut declaration = DeclarationItem::new(
						&DomainBuilder::unbounded(*ty),
						true,
						Origin::from(item).with_desugaring(DesugarKind::Objective),
					);
					declaration.name = s.data[*pattern].identifier();
					declaration.set_definition(&*collector.collect_expression(*objective));
					let decl = collector.model.add_declaration(declaration);
					collector.resolutions.insert(
						PatternRef::new(item, *pattern),
						ResolvedIdentifier::Declaration(decl),
					);
					solve.set_maximize(decl);
				}
				_ => unreachable!(),
			},
			crate::hir::Goal::Minimize { pattern, objective } => match &types[*pattern] {
				PatternTy::Variable(ty) => {
					let mut declaration = DeclarationItem::new(
						&DomainBuilder::unbounded(*ty),
						true,
						Origin::from(item).with_desugaring(DesugarKind::Objective),
					);
					declaration.name = s.data[*pattern].identifier();
					declaration.set_definition(&*collector.collect_expression(*objective));
					let decl = collector.model.add_declaration(declaration);
					collector.resolutions.insert(
						PatternRef::new(item, *pattern),
						ResolvedIdentifier::Declaration(decl),
					);
					solve.set_minimize(decl);
				}
				_ => unreachable!(),
			},
			crate::hir::Goal::Satisfy => (),
		}
		collector.model.set_solve(solve);
	}

	/// Collect type alias item
	pub fn collect_type_alias(
		&mut self,
		item: ItemRef,
		t: &crate::hir::Item<crate::hir::TypeAlias>,
	) {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &t.data, item, types.clone());
		match &types[t.name] {
			PatternTy::TypeAlias(ty) => {
				collector.collect_domain(t.aliased_type, *ty, true);
			}
			_ => unreachable!(),
		}
	}

	/// Finish lowering
	pub fn finish(self) -> Model {
		self.model
	}
}

/// Collects HIR expressions and generates builders for THIR
pub struct ExpressionCollector<'a, 'b> {
	parent: &'a mut ItemCollector<'b>,
	data: &'a crate::hir::ItemData,
	item: ItemRef,
	types: TypeResult,
}

impl<'a, 'b> Deref for ExpressionCollector<'a, 'b> {
	type Target = ItemCollector<'b>;
	fn deref(&self) -> &Self::Target {
		self.parent
	}
}

impl<'a, 'b> DerefMut for ExpressionCollector<'a, 'b> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.parent
	}
}

impl<'a, 'b> ExpressionCollector<'a, 'b> {
	fn new(
		parent: &'a mut ItemCollector<'b>,
		data: &'a crate::hir::ItemData,
		item: ItemRef,
		types: TypeResult,
	) -> Self {
		Self {
			parent,
			data,
			item,
			types,
		}
	}

	/// Collect an expression
	pub fn collect_expression(
		&mut self,
		idx: ArenaIndex<crate::hir::Expression>,
	) -> Box<dyn ExpressionBuilder> {
		let ty = self.types[idx];
		let origin: Origin = EntityRef::new(self.db.upcast(), self.item, idx).into();
		let result: Box<dyn ExpressionBuilder> = match &self.data[idx] {
			crate::hir::Expression::Absent => AbsentBuilder::new(ty, origin).with_annotations(
				self.data
					.annotations(idx)
					.map(|ann| self.collect_expression(ann)),
			),
			crate::hir::Expression::ArrayAccess(aa) => {
				let collection = self.collect_expression(aa.collection);
				let has_slices = match &self.data[aa.indices] {
					crate::hir::Expression::Slice(_) => true,
					crate::hir::Expression::TupleLiteral(tl) => tl
						.fields
						.iter()
						.any(|f| matches!(&self.data[*f], crate::hir::Expression::Slice(_))),
					_ => false,
				};
				if has_slices {
					// Rewrite a[..] into let { any: c = a } in c['..'(index_set(c))]
					let collection_ty = self.types[aa.collection];
					let collection_origin =
						Origin::from(EntityRef::new(self.db.upcast(), self.item, aa.collection))
							.with_desugaring(DesugarKind::ArraySlice);
					let indices_origin =
						Origin::from(EntityRef::new(self.db.upcast(), self.item, aa.indices))
							.with_desugaring(DesugarKind::ArraySlice);
					let mut declaration = DeclarationItem::new(
						&DomainBuilder::unbounded(collection_ty),
						false,
						collection_origin,
					);
					declaration.set_definition(&*collection);
					let decl = self.model.add_declaration(declaration);
					let indices: Box<dyn ExpressionBuilder> = match &self.data[aa.indices] {
						crate::hir::Expression::Slice(s) => {
							// 1D array, so use `index_set` function
							let index_set = lookup_function(
								self.db,
								&self.model,
								self.db.identifier_registry().index_set,
								&[collection_ty],
							)
							.unwrap();
							let slice = lookup_function(
								self.db,
								&self.model,
								*s,
								&[index_set.fn_type.return_type],
							)
							.unwrap();
							slice.into_call(self.db, indices_origin).with_arg(
								index_set.into_call(self.db, indices_origin).with_arg(
									IdentifierBuilder::new(
										self.types[aa.collection],
										ResolvedIdentifier::Declaration(decl),
										collection_origin,
									),
								),
							)
						}
						crate::hir::Expression::TupleLiteral(tl) => {
							// Multidim array slice
							let dims = tl.fields.len();
							let (tys, indices): (Vec<_>, Vec<_>) = tl
								.fields
								.iter()
								.enumerate()
								.map(|(i, f)| -> (Ty, Box<dyn ExpressionBuilder>) {
									if let crate::hir::Expression::Slice(s) = &self.data[*f] {
										let origin = Origin::from(EntityRef::new(
											self.db.upcast(),
											self.item,
											*f,
										))
										.with_desugaring(DesugarKind::ArraySlice);
										let index_set_mofn = Identifier::new(
											format!("index_set_{}of{}", i + 1, dims),
											self.db.upcast(),
										);
										let index_set_fn = lookup_function(
											self.db,
											&self.model,
											index_set_mofn,
											&[collection_ty],
										)
										.unwrap();
										let slice_fn = lookup_function(
											self.db,
											&self.model,
											*s,
											&[index_set_fn.fn_type.return_type],
										)
										.unwrap();
										let ty = slice_fn.fn_type.return_type;
										let call = slice_fn.into_call(self.db, origin).with_arg(
											index_set_fn.into_call(self.db, origin).with_arg(
												IdentifierBuilder::new(
													self.types[aa.collection],
													ResolvedIdentifier::Declaration(decl),
													collection_origin,
												),
											),
										);
										(ty, call)
									} else {
										(self.types[*f], self.collect_expression(*f))
									}
								})
								.unzip();
							TupleLiteralBuilder::new(
								Ty::tuple(self.db.upcast(), tys),
								indices_origin,
							)
							.with_members(indices)
						}
						_ => unreachable!(),
					};
					LetBuilder::new(ty, origin)
						.with_item(LetItem::Declaration(decl))
						.with_in(
							ArrayAccessBuilder::new(
								ty,
								IdentifierBuilder::new(
									self.types[aa.collection],
									ResolvedIdentifier::Declaration(decl),
									collection_origin,
								),
								indices,
								origin,
							)
							.with_annotations(
								self.data
									.annotations(idx)
									.map(|ann| self.collect_expression(ann)),
							),
						)
				} else {
					ArrayAccessBuilder::new(
						ty,
						collection,
						self.collect_expression(aa.indices),
						origin,
					)
					.with_annotations(
						self.data
							.annotations(idx)
							.map(|ann| self.collect_expression(ann)),
					)
				}
			}
			crate::hir::Expression::ArrayComprehension(c) => {
				ArrayComprehensionBuilder::new(ty, origin)
					.with_generators(c.generators.iter().map(|g| self.collect_generator(g)))
					.with_indices(c.indices.map(|i| self.collect_expression(i)))
					.with_template(self.collect_expression(c.template))
					.with_annotations(
						self.data
							.annotations(idx)
							.map(|ann| self.collect_expression(ann)),
					)
			}
			crate::hir::Expression::ArrayLiteral(al) => ArrayLiteralBuilder::new(ty, origin)
				.with_members(al.members.iter().map(|m| self.collect_expression(*m)))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::BooleanLiteral(b) => BooleanLiteralBuilder::new(ty, *b, origin)
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::Call(c) => {
				CallBuilder::new(ty, self.collect_expression(c.function), origin)
					.with_args(c.arguments.iter().map(|arg| self.collect_expression(*arg)))
					.with_annotations(
						self.data
							.annotations(idx)
							.map(|ann| self.collect_expression(ann)),
					)
			}
			crate::hir::Expression::Case(_c) => {
				unimplemented!()
			}
			crate::hir::Expression::FloatLiteral(f) => FloatLiteralBuilder::new(ty, *f, origin)
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(idx).unwrap();
				if !self.resolutions.contains_key(&res) {
					self.collect_item(res.item());
					assert!(
						self.resolutions.contains_key(&res),
						"Collected item at {:?} but did not resolve identifier owned by item",
						NodeRef::from(res.into_entity(self.db.upcast()))
							.source_span(self.db.upcast())
					);
				}
				IdentifierBuilder::new(ty, self.resolutions[&res], origin).with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
			}
			crate::hir::Expression::IfThenElse(ite) => IfThenElseBuilder::new(ty, origin)
				.with_branches(ite.branches.iter().map(|b| {
					(
						self.collect_expression(b.condition),
						self.collect_expression(b.result),
					)
				}))
				.with_else(if let Some(e) = ite.else_result {
					self.collect_expression(e)
				} else {
					self.collect_default_else(ty, origin).1
				})
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::Infinity => InfinityBuilder::new(ty, origin).with_annotations(
				self.data
					.annotations(idx)
					.map(|ann| self.collect_expression(ann)),
			),
			crate::hir::Expression::IntegerLiteral(i) => IntegerLiteralBuilder::new(ty, *i, origin)
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::Let(l) => LetBuilder::new(ty, origin)
				.with_items(l.items.iter().flat_map(|i| match i {
					crate::hir::LetItem::Constraint(c) => {
						let e = ConstraintItem::new(
							&*self.collect_expression(c.expression),
							false,
							origin,
						);
						vec![LetItem::Constraint(self.model.add_constraint(e))]
					}
					crate::hir::LetItem::Declaration(d) => {
						let item = self.item;
						let data = self.data;
						self.collect_declaration(item, d, data, false)
							.into_iter()
							.map(LetItem::Declaration)
							.collect::<Vec<_>>()
					}
				}))
				.with_in(self.collect_expression(l.in_expression))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::RecordAccess(ra) => {
				let record = self.collect_expression(ra.record);
				RecordAccessBuilder::new(ty, record, ra.field, origin).with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
			}
			crate::hir::Expression::RecordLiteral(rl) => RecordLiteralBuilder::new(ty, origin)
				.with_members(rl.fields.iter().map(|(i, v)| {
					(
						self.data[*i].identifier().unwrap(),
						self.collect_expression(*v),
					)
				}))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::SetComprehension(c) => SetComprehensionBuilder::new(ty, origin)
				.with_generators(c.generators.iter().map(|g| self.collect_generator(g)))
				.with_template(self.collect_expression(c.template))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::SetLiteral(sl) => SetLiteralBuilder::new(ty, origin)
				.with_members(sl.members.iter().map(|m| self.collect_expression(*m)))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			crate::hir::Expression::Slice(_) => {
				unreachable!("Slice used outside of array access")
			}
			crate::hir::Expression::StringLiteral(sl) => {
				StringLiteralBuilder::new(ty, sl.clone(), origin).with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
			}
			crate::hir::Expression::TupleAccess(ta) => {
				let tuple = self.collect_expression(ta.tuple);
				TupleAccessBuilder::new(ty, tuple, ta.field, origin).with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
			}
			crate::hir::Expression::TupleLiteral(tl) => TupleLiteralBuilder::new(ty, origin)
				.with_members(tl.fields.iter().map(|m| self.collect_expression(*m)))
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				),
			_ => unimplemented!("{:?}", &self.data[idx]),
		};
		result
	}

	fn collect_generator(&mut self, g: &crate::hir::Generator) -> GeneratorBuilder {
		let mut b = GeneratorBuilder::new(self.collect_expression(g.collection));
		for p in g.patterns.iter() {
			let decl = match &self.types[*p] {
				PatternTy::Variable(ty) => {
					let mut declaration = DeclarationItem::new(
						&DomainBuilder::unbounded(*ty),
						false,
						EntityRef::new(self.db.upcast(), self.item, *p),
					);
					declaration.name = self.data[*p].identifier();
					let decl = self.model.add_declaration(declaration);
					let item = self.item;
					self.resolutions.insert(
						PatternRef::new(item, *p),
						ResolvedIdentifier::Declaration(decl),
					);
					decl
				}
				// TODO: Rewrite into where clause
				PatternTy::Destructuring(_) => unimplemented!(),
				_ => unreachable!(),
			};
			b = b.with_declaration(decl)
		}
		if let Some(w) = g.where_clause {
			b = b.with_where(self.collect_expression(w));
		}
		b
	}

	fn collect_default_else(&self, ty: Ty, origin: Origin) -> (Ty, Box<dyn ExpressionBuilder>) {
		match ty.lookup(self.db.upcast()) {
			TyData::Boolean(_, OptType::Opt)
			| TyData::Integer(_, OptType::Opt)
			| TyData::Float(_, OptType::Opt)
			| TyData::Enum(_, OptType::Opt, _)
			| TyData::Bottom(OptType::Opt)
			| TyData::Array {
				opt: OptType::Opt, ..
			}
			| TyData::Set(_, OptType::Opt, _)
			| TyData::Tuple(OptType::Opt, _)
			| TyData::Record(OptType::Opt, _)
			| TyData::Function(OptType::Opt, _)
			| TyData::TyVar(_, Some(OptType::Opt), _) => {
				(self.tys.bottom, AbsentBuilder::new(self.tys.bottom, origin))
			}
			TyData::Boolean(_, _) => (
				self.tys.par_bool,
				BooleanLiteralBuilder::new(self.tys.par_bool, BooleanLiteral(true), origin),
			),
			TyData::String(_) => (
				self.tys.string,
				StringLiteralBuilder::new(
					self.tys.string,
					StringLiteral::new("", self.db.upcast()),
					origin,
				),
			),
			TyData::Annotation(_) => {
				unimplemented!()
				// IdentifierBuilder::new(ty, ResolvedIdentifier::Declaration(()), origin)
			}
			TyData::Array { .. } => (
				self.tys.array_of_bottom,
				ArrayLiteralBuilder::new(self.tys.array_of_bottom, origin),
			),
			TyData::Set(_, _, _) => (
				self.tys.set_of_bottom,
				SetLiteralBuilder::new(self.tys.set_of_bottom, origin),
			),
			TyData::Tuple(_, fs) => {
				let mut tys = Vec::with_capacity(fs.len());
				let mut es = Vec::with_capacity(fs.len());
				for f in fs.iter() {
					let (t, e) = self.collect_default_else(*f, origin);
					tys.push(t);
					es.push(e);
				}
				let tt = Ty::tuple(self.db.upcast(), tys);
				let mut builder = TupleLiteralBuilder::new(tt, origin);
				for e in es {
					builder = builder.with_member(e);
				}
				(tt, builder)
			}
			TyData::Record(_, fs) => {
				let mut tys = Vec::with_capacity(fs.len());
				let mut es = Vec::with_capacity(fs.len());
				for (i, f) in fs.iter() {
					let (t, e) = self.collect_default_else(*f, origin);
					tys.push(t);
					es.push((*i, e));
				}
				let tt = Ty::tuple(self.db.upcast(), tys);
				let mut builder = RecordLiteralBuilder::new(tt, origin);
				for (i, e) in es {
					builder = builder.with_member(Identifier(i), e);
				}
				(tt, builder)
			}
			_ => unreachable!("No default value for this type"),
		}
	}

	fn collect_domain(
		&mut self,
		t: ArenaIndex<crate::hir::Type>,
		ty: Ty,
		is_type_alias: bool,
	) -> DomainBuilder {
		match (&self.data[t], ty.lookup(self.db.upcast())) {
			(crate::hir::Type::Bounded { domain, .. }, _) => {
				if let Some(res) = self.types.name_resolution(*domain) {
					let res_types = self.db.lookup_item_types(res.item());
					match &res_types[res.pattern()] {
						// Identifier is actually a type, not a domain expression
						PatternTy::TyVar(_) => return DomainBuilder::unbounded(ty),
						PatternTy::TypeAlias(_) => {
							let model = res.item().model(self.db.upcast());
							match res.item().local_item_ref(self.db.upcast()) {
								LocalItemRef::TypeAlias(ta) => {
									let mut c = ExpressionCollector::new(
										self.parent,
										&model[ta].data,
										res.item(),
										res_types,
									);
									return c.collect_domain(model[ta].aliased_type, ty, true);
								}
								_ => unreachable!(),
							}
						}
						_ => (),
					}
				}
				if is_type_alias {
					// Replace expressions with identifiers pointing to declarations for those expressions
					let tr = TypeRef::new(self.item, t);
					let dom_ty = self.types[*domain];
					let origin: Origin =
						EntityRef::new(self.db.upcast(), self.item, *domain).into();
					// Note: unable to keep Entry until insert, since "self" is mutably borrowed to create the value
					#[allow(clippy::map_entry)]
					if !self.type_alias_domains.contains_key(&tr) {
						let mut declaration =
							DeclarationItem::new(&DomainBuilder::unbounded(dom_ty), true, origin);
						declaration.set_definition(&*self.collect_expression(*domain));
						let decl = self.model.add_declaration(declaration);
						self.type_alias_domains.insert(tr, decl);
					}
					DomainBuilder::bounded(
						ty,
						IdentifierBuilder::new(
							dom_ty,
							ResolvedIdentifier::Declaration(self.type_alias_domains[&tr]),
							origin,
						),
					)
				} else {
					DomainBuilder::bounded(ty, self.collect_expression(*domain))
				}
			}
			(
				crate::hir::Type::Array {
					dimensions,
					element,
					..
				},
				TyData::Array {
					dim: d,
					element: el,
					..
				},
			) => {
				let dim = self.collect_domain(*dimensions, d, is_type_alias);
				let el = self.collect_domain(*element, el, is_type_alias);
				DomainBuilder::array(ty, dim, el)
			}
			(crate::hir::Type::Set { element, .. }, TyData::Set(_, _, e)) => {
				DomainBuilder::set(ty, self.collect_domain(*element, e, is_type_alias))
			}
			(crate::hir::Type::Tuple { fields, .. }, TyData::Tuple(_, fs)) => {
				let domains = fields
					.iter()
					.zip(fs.iter())
					.map(|(f, ty)| self.collect_domain(*f, *ty, is_type_alias))
					.collect::<Vec<_>>();
				DomainBuilder::tuple(ty, domains)
			}
			(crate::hir::Type::Record { fields, .. }, TyData::Record(_, fs)) => {
				let domains = fs
					.iter()
					.map(|(i, ty)| {
						let ident = Identifier(*i);
						(
							ident,
							self.collect_domain(
								fields
									.iter()
									.find_map(|(p, t)| {
										if self.data[*p].identifier().unwrap() == ident {
											Some(*t)
										} else {
											None
										}
									})
									.unwrap(),
								*ty,
								is_type_alias,
							),
						)
					})
					.collect::<Vec<_>>();
				DomainBuilder::record(ty, domains)
			}
			_ => DomainBuilder::unbounded(ty),
		}
	}

	fn collect_enum_case(&mut self, c: &crate::hir::Constructor) -> Constructor {
		let tys = match &self.types[c.pattern] {
			PatternTy::EnumAtom(_) => {
				return Constructor {
					name: self.data[c.pattern].identifier(),
					parameters: None,
				}
			}
			PatternTy::EnumConstructor(fs) => fs[0].overload.params().to_owned(),
			_ => unreachable!(),
		};
		Constructor {
			name: self.data[c.pattern].identifier(),
			parameters: Some(
				tys.iter()
					.zip(c.parameters())
					.map(|(ty, t)| {
						let domain = self.collect_domain(t.declared_type, *ty, false);
						let declaration = DeclarationItem::new(&domain, false, self.item);
						self.model.add_declaration(declaration)
					})
					.collect(),
			),
		}
	}
}

struct DestructuringCollector<'a, 'b, 'c> {
	parent: &'a mut ExpressionCollector<'b, 'c>,
	decls: &'a mut Vec<DeclarationId>,
	top_level: bool,
}

impl<'a, 'b, 'c> Deref for DestructuringCollector<'a, 'b, 'c> {
	type Target = ExpressionCollector<'b, 'c>;
	fn deref(&self) -> &Self::Target {
		self.parent
	}
}

impl<'a, 'b, 'c> DerefMut for DestructuringCollector<'a, 'b, 'c> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.parent
	}
}

impl<'a, 'b, 'c> DestructuringCollector<'a, 'b, 'c> {
	fn collect_destructuring(
		&mut self,
		definition: Box<dyn ExpressionBuilder>,
		pattern: ArenaIndex<crate::hir::Pattern>,
	) {
		match (&self.data[pattern], self.types[pattern].clone()) {
			(crate::hir::Pattern::Identifier(i), PatternTy::Variable(ty)) => {
				let mut declaration = DeclarationItem::new(
					&DomainBuilder::unbounded(ty),
					self.top_level,
					Origin::from(self.item).with_desugaring(DesugarKind::Destructuring),
				);
				declaration.set_definition(&*definition);
				declaration.name = Some(*i);
				let decl = self.model.add_declaration(declaration);
				let pat = PatternRef::new(self.item, pattern);
				self.resolutions
					.insert(pat, ResolvedIdentifier::Declaration(decl));
				self.decls.push(decl);
			}
			(crate::hir::Pattern::Tuple { fields }, PatternTy::Destructuring(ty)) => {
				for (i, f) in fields.iter().enumerate() {
					let def = TupleAccessBuilder::new(
						ty,
						definition.clone(),
						IntegerLiteral((i + 1) as i64),
						Origin::from(EntityRef::new(self.db.upcast(), self.item, *f))
							.with_desugaring(DesugarKind::Destructuring),
					);
					self.collect_destructuring(def, *f)
				}
			}
			(crate::hir::Pattern::Record { fields }, PatternTy::Destructuring(ty)) => {
				for (i, f) in fields.iter() {
					let def = RecordAccessBuilder::new(
						ty,
						definition.clone(),
						*i,
						Origin::from(EntityRef::new(self.db.upcast(), self.item, *f))
							.with_desugaring(DesugarKind::Destructuring),
					);
					self.collect_destructuring(def, *f)
				}
			}
			_ => unreachable!(),
		}
	}
}

/// Lower a model to THIR
pub fn lower_model(db: &dyn Thir) -> Arc<Model> {
	let tys = db.type_registry();
	let mut collector = ItemCollector::new(db, &tys);
	let items = db.lookup_topological_sorted_items();
	for item in items.iter() {
		collector.collect_item(*item);
	}
	Arc::new(collector.finish())
}
