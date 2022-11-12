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
		ids::{EntityRef, ExpressionRef, ItemRef, LocalItemRef, PatternRef, TypeRef},
		source::{Origin, SourceMap},
		PatternTy, TypeResult,
	},
	ty::{OptType, OverloadedFunction, Ty, TyData, TypeRegistry, VarType},
};

use super::{db::Thir, *};

/// Collects HIR items and lowers them to THIR
pub struct ItemCollector<'a> {
	db: &'a dyn Thir,
	tys: &'a TypeRegistry,
	resolutions: FxHashMap<PatternRef, ResolvedIdentifier>,
	annotations: Arena<Item<Annotation>>,
	constraints: Arena<Item<Constraint>>,
	declarations: Arena<Item<Declaration>>,
	enumerations: Arena<Item<Enumeration>>,
	functions: Arena<Item<Function>>,
	outputs: Arena<Item<Output>>,
	solve: Option<Item<Solve>>,
	type_alias_domains: FxHashMap<TypeRef, ArenaIndex<Item<Declaration>>>,
	top_level: Vec<ItemId>,
	visited: FxHashSet<ItemRef>,
}

impl<'a> ItemCollector<'a> {
	/// Create a new item collector
	pub fn new(db: &'a dyn Thir, tys: &'a TypeRegistry) -> Self {
		Self {
			db,
			tys,
			resolutions: FxHashMap::default(),
			annotations: Arena::new(),
			constraints: Arena::new(),
			declarations: Arena::new(),
			enumerations: Arena::new(),
			functions: Arena::new(),
			outputs: Arena::new(),
			solve: None,
			type_alias_domains: FxHashMap::default(),
			top_level: Vec::new(),
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
				let idx = self.collect_annotation(item, &model[a]);
				self.top_level.push(idx.into());
			}
			LocalItemRef::Assignment(a) => self.collect_assignment(item, &model[a]),
			LocalItemRef::Constraint(c) => {
				let idx = self.collect_constraint(item, &model[c]);
				self.top_level.push(idx.into());
			}
			LocalItemRef::Declaration(d) => {
				for idx in self.collect_declaration(item, &model[d], &model[d].data) {
					self.top_level.push(idx.into());
				}
			}
			LocalItemRef::Enumeration(e) => {
				let idx = self.collect_enumeration(item, &model[e]);
				self.top_level.push(idx.into());
			}
			LocalItemRef::EnumAssignment(a) => self.collect_enumeration_assignment(item, &model[a]),
			LocalItemRef::Function(f) => {
				let idx = self.collect_function(item, &model[f]);
				self.top_level.push(idx.into());
			}
			LocalItemRef::Output(o) => {
				let idx = self.collect_output(item, &model[o]);
				self.top_level.push(idx.into());
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
	) -> ArenaIndex<Item<Annotation>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &a.data, item, types.clone());
		let ty = &types[a.pattern];
		match ty {
			PatternTy::AnnotationAtom => {
				let annotation = <Item<Annotation>>::new(
					a.data[a.pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				let idx = self.annotations.insert(annotation);
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
					let mut declaration = <Item<Declaration>>::new(&domain);
					// Ignore destructuring and recording resolution for now since these can't have bodies which refer
					// to parameters anyway
					if let Some(p) = param.pattern {
						declaration.name = a.data[p].identifier();
					}
					parameters.push(collector.declarations.insert(declaration));
				}
				let mut annotation = <Item<Annotation>>::new(
					a.data[a.pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				annotation.parameters = Some(parameters);
				let idx = self.annotations.insert(annotation);
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
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &a.data, item, types.clone());
		let res = types.name_resolution(a.assignee).unwrap();
		let def = collector.collect_expression(a.definition);
		if !self.resolutions.contains_key(&res) {
			self.collect_item(res.item());
		}
		match &self.resolutions[&res] {
			ResolvedIdentifier::Declaration(d) => self.declarations[*d].set_definition(&*def),
			_ => unreachable!(),
		}
	}

	/// Collect a constraint item
	pub fn collect_constraint(
		&mut self,
		item: ItemRef,
		c: &crate::hir::Item<crate::hir::Constraint>,
	) -> ArenaIndex<Item<Constraint>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &c.data, item, types);
		let mut constraint = <Item<Constraint>>::new(&*collector.collect_expression(c.expression));
		for ann in c.annotations.iter() {
			constraint.add_annotation(&*collector.collect_expression(*ann));
		}
		self.constraints.insert(constraint)
	}

	/// Collect a declaration item
	pub fn collect_declaration(
		&mut self,
		item: ItemRef,
		d: &crate::hir::Declaration,
		data: &crate::hir::ItemData,
	) -> Vec<ArenaIndex<Item<Declaration>>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, data, item, types.clone());
		let ty = &types[d.pattern];
		match ty {
			PatternTy::Variable(ty) => {
				let domain = collector.collect_domain(d.declared_type, *ty, false);
				let mut declaration = <Item<Declaration>>::new(&domain);
				for ann in d.annotations.iter() {
					declaration.add_annotation(&*collector.collect_expression(*ann));
				}
				if let Some(def) = d.definition {
					declaration.set_definition(&*collector.collect_expression(def));
				}
				declaration.name = data[d.pattern].identifier();
				let decl = self.declarations.insert(declaration);
				self.resolutions.insert(
					PatternRef::new(item, d.pattern),
					ResolvedIdentifier::Declaration(decl),
				);
				vec![decl]
			}
			PatternTy::Destructuring(ty) => {
				let domain = collector.collect_domain(d.declared_type, *ty, false);
				let mut declaration = <Item<Declaration>>::new(&domain);
				for ann in d.annotations.iter() {
					declaration.add_annotation(&*collector.collect_expression(*ann));
				}
				if let Some(def) = d.definition {
					declaration.set_definition(&*collector.collect_expression(def));
				}
				let decl = self.declarations.insert(declaration);
				let origins = self.db.lookup_source_map(item.model_ref(self.db.upcast()));
				let origin = origins
					.get_origin(EntityRef::new(self.db.upcast(), item, d.pattern).into())
					.unwrap()
					.clone();
				let mut decls = vec![decl];
				self.collect_destructuring(
					&mut decls,
					IdentifierBuilder::new(*ty, ResolvedIdentifier::Declaration(decl), origin),
					d.pattern,
					item,
					data,
					&types,
					&origins,
				);
				decls
			}
			_ => unreachable!(),
		}
	}

	// FIXME: Refactor so this method has less arguments
	#[allow(clippy::too_many_arguments)]
	fn collect_destructuring(
		&mut self,
		decls: &mut Vec<ArenaIndex<Item<Declaration>>>,
		definition: Box<dyn ExpressionBuilder>,
		pattern: ArenaIndex<crate::hir::Pattern>,
		item: ItemRef,
		data: &crate::hir::ItemData,
		types: &TypeResult,
		origins: &SourceMap,
	) {
		let ty = &types[pattern];
		match (&data[pattern], ty) {
			(crate::hir::Pattern::Identifier(i), PatternTy::Variable(ty)) => {
				let mut declaration = <Item<Declaration>>::new(&DomainBuilder::unbounded(*ty));
				declaration.set_definition(&*definition);
				declaration.name = Some(*i);
				let decl = self.declarations.insert(declaration);
				decls.push(decl);
			}
			(crate::hir::Pattern::Tuple { fields }, PatternTy::Destructuring(ty)) => {
				for (i, f) in fields.iter().enumerate() {
					let def = TupleAccessBuilder::new(
						*ty,
						definition.clone(),
						IntegerLiteral(i as i64),
						origins
							.get_origin(EntityRef::new(self.db.upcast(), item, *f).into())
							.unwrap()
							.clone(),
					);
					self.collect_destructuring(decls, def, *f, item, data, types, origins)
				}
			}
			(crate::hir::Pattern::Record { fields }, PatternTy::Destructuring(ty)) => {
				for (i, f) in fields.iter() {
					let def = RecordAccessBuilder::new(
						*ty,
						definition.clone(),
						*i,
						origins
							.get_origin(EntityRef::new(self.db.upcast(), item, *f).into())
							.unwrap()
							.clone(),
					);
					self.collect_destructuring(decls, def, *f, item, data, types, origins)
				}
			}
			_ => unreachable!(),
		}
	}

	/// Collect an enumeration item
	pub fn collect_enumeration(
		&mut self,
		item: ItemRef,
		e: &crate::hir::Item<crate::hir::Enumeration>,
	) -> ArenaIndex<Item<Enumeration>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &e.data, item, types.clone());
		let ty = &types[e.pattern];
		match ty {
			PatternTy::Variable(ty) => match ty.lookup(collector.db.upcast()) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(collector.db.upcast()) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = <Item<Enumeration>>::new(t);
							if let Some(def) = &e.definition {
								enumeration.definition = Some(
									def.iter().map(|c| collector.collect_enum_case(c)).collect(),
								)
							}
							let idx = self.enumerations.insert(enumeration);
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
		collector.enumerations[e].definition = Some(
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

	/// Collect an function item
	pub fn collect_function(
		&mut self,
		item: ItemRef,
		f: &crate::hir::Item<crate::hir::Function>,
	) -> ArenaIndex<Item<Function>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &f.data, item, types.clone());
		let res = PatternRef::new(item, f.pattern);
		if !collector.resolutions.contains_key(&res) {
			match &types[f.pattern] {
				PatternTy::Function(fn_entry) => {
					let domain = collector.collect_domain(
						f.return_type,
						fn_entry.overload.return_type(),
						false,
					);
					let mut func =
						<Item<Function>>::new(f.data[f.pattern].identifier().unwrap(), &domain);
					for ann in f.annotations.iter() {
						func.add_annotation(&*collector.collect_expression(*ann));
					}
					if let OverloadedFunction::PolymorphicFunction(pf) = &fn_entry.overload {
						func.type_inst_vars.extend(pf.ty_params.iter().copied());
					}
					for (param, ty) in f.parameters.iter().zip(fn_entry.overload.params()) {
						let domain = collector.collect_domain(param.declared_type, *ty, false);
						let mut declaration = <Item<Declaration>>::new(&domain);
						if let Some(p) = param.pattern {
							declaration.name = f.data[p].identifier();
						}
						for ann in param.annotations.iter() {
							declaration.add_annotation(&*collector.collect_expression(*ann));
						}
						let idx = collector.declarations.insert(declaration);
						func.parameters.push(idx);
						if let Some(p) = param.pattern {
							collector.resolutions.insert(
								PatternRef::new(item, p),
								ResolvedIdentifier::Declaration(idx),
							);
						}
					}
					let idx = collector.functions.insert(func);
					collector
						.resolutions
						.insert(res, ResolvedIdentifier::Function(idx));
				}
				_ => unreachable!(),
			}
		}
		match collector.resolutions[&res] {
			ResolvedIdentifier::Function(i) => {
				if let Some(b) = f.body {
					let body = collector.collect_expression(b);

					let origins = collector
						.db
						.lookup_source_map(item.model_ref(collector.db.upcast()));
					let mut decls = Vec::new();
					for (param, idx) in f
						.parameters
						.iter()
						.zip(collector.functions[i].parameters.clone())
					{
						if let Some(p) = param.pattern {
							if f.data[p].identifier().is_none() {
								let decl = &collector.declarations[idx];
								let ident = IdentifierBuilder::new(
									decl.domain.ty(),
									ResolvedIdentifier::Declaration(idx),
									origins
										.get_origin(
											PatternRef::new(item, p)
												.into_entity(collector.db.upcast())
												.into(),
										)
										.unwrap()
										.clone(),
								);
								collector.collect_destructuring(
									&mut decls, ident, p, item, &f.data, &types, &origins,
								);
							}
						}
					}
					if decls.is_empty() {
						collector.functions[i].set_body(&*body);
					} else {
						let builder = LetBuilder::new(
							types[b],
							origins
								.get_origin(
									ExpressionRef::new(item, b)
										.into_entity(collector.db.upcast())
										.into(),
								)
								.unwrap()
								.clone(),
						)
						.with_items(decls.into_iter().map(LetItem::Declaration))
						.with_in(body);
						collector.functions[i].set_body(&*builder);
					}
				}
				i
			}
			_ => unreachable!(),
		}
	}

	/// Collect an output item
	pub fn collect_output(
		&mut self,
		item: ItemRef,
		o: &crate::hir::Item<crate::hir::Output>,
	) -> ArenaIndex<Item<Output>> {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &o.data, item, types);
		let e = collector.collect_expression(o.expression);
		let mut output = <Item<Output>>::new(&*e);
		if let Some(s) = o.section {
			let section = collector.collect_expression(s);
			output.set_section(&*section);
		}
		self.outputs.insert(output)
	}

	/// Collect solve item
	pub fn collect_solve(&mut self, item: ItemRef, s: &crate::hir::Item<crate::hir::Solve>) {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &s.data, item, types.clone());
		let mut solve = <Item<Solve>>::default();
		for ann in s.annotations.iter() {
			solve.add_annotation(&*collector.collect_expression(*ann));
		}
		match &s.goal {
			crate::hir::Goal::Maximize { pattern, objective } => match &types[*pattern] {
				PatternTy::Variable(ty) => {
					let mut declaration = <Item<Declaration>>::new(&DomainBuilder::unbounded(*ty));
					declaration.name = s.data[*pattern].identifier();
					declaration.set_definition(&*collector.collect_expression(*objective));
					let decl = self.declarations.insert(declaration);
					self.resolutions.insert(
						PatternRef::new(item, *pattern),
						ResolvedIdentifier::Declaration(decl),
					);
					self.top_level.push(decl.into());
					solve.set_maximize(decl);
				}
				_ => unreachable!(),
			},
			crate::hir::Goal::Minimize { pattern, objective } => match &types[*pattern] {
				PatternTy::Variable(ty) => {
					let mut declaration = <Item<Declaration>>::new(&DomainBuilder::unbounded(*ty));
					declaration.name = s.data[*pattern].identifier();
					declaration.set_definition(&*collector.collect_expression(*objective));
					let decl = self.declarations.insert(declaration);
					self.resolutions.insert(
						PatternRef::new(item, *pattern),
						ResolvedIdentifier::Declaration(decl),
					);
					self.top_level.push(decl.into());
					solve.set_minimize(decl);
				}
				_ => unreachable!(),
			},
			crate::hir::Goal::Satisfy => (),
		}
		self.solve = Some(solve);
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
		Model {
			annotations: self.annotations,
			constraints: self.constraints,
			declarations: self.declarations,
			enumerations: self.enumerations,
			functions: self.functions,
			outputs: self.outputs,
			solve: self.solve.unwrap_or_default(),
			top_level: self.top_level,
		}
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
		let origins = self
			.db
			.lookup_source_map(self.item.model_ref(self.db.upcast()));
		let origin = origins
			.get_origin(EntityRef::new(self.db.upcast(), self.item, idx).into())
			.unwrap()
			.clone();
		let result: Box<dyn ExpressionBuilder> = match &self.data[idx] {
			crate::hir::Expression::Absent => AbsentBuilder::new(ty, origin).with_annotations(
				self.data
					.annotations(idx)
					.map(|ann| self.collect_expression(ann)),
			),
			crate::hir::Expression::ArrayAccess(aa) => ArrayAccessBuilder::new(
				ty,
				self.collect_expression(aa.collection),
				self.collect_expression(aa.indices),
				origin,
			)
			.with_annotations(
				self.data
					.annotations(idx)
					.map(|ann| self.collect_expression(ann)),
			),
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
			crate::hir::Expression::Identifier(i) => {
				let res = self.types.name_resolution(idx).unwrap();
				if !self.resolutions.contains_key(&res) {
					self.collect_item(res.item());
				}
				if !self.resolutions.contains_key(&res) {
					eprintln!(
						"{} ({:?})",
						i.pretty_print(self.db.upcast()),
						origin.source_span(self.db.upcast())
					);
				}
				IdentifierBuilder::new(ty, self.resolutions[&res], origin).with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
			}
			crate::hir::Expression::IfThenElse(ite) => IfThenElseBuilder::new(ty, origin.clone())
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
						let e = <Item<Constraint>>::new(&*self.collect_expression(c.expression));
						vec![LetItem::Constraint(self.constraints.insert(e))]
					}
					crate::hir::LetItem::Declaration(d) => {
						let item = self.item;
						let data = self.data;
						self.collect_declaration(item, d, data)
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
			crate::hir::Expression::Slice(s) => {
				let (i, _, _) =
					lookup_function(self.db, *s, &[ty], &self.functions, &self.declarations)
						.unwrap();
				CallBuilder::new(
					ty,
					IdentifierBuilder::new(ty, ResolvedIdentifier::Function(i), origin.clone()),
					origin,
				)
				.with_annotations(
					self.data
						.annotations(idx)
						.map(|ann| self.collect_expression(ann)),
				)
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
					let mut declaration = <Item<Declaration>>::new(&DomainBuilder::unbounded(*ty));
					declaration.name = self.data[*p].identifier();
					let decl = self.declarations.insert(declaration);
					let item = self.item;
					self.resolutions.insert(
						PatternRef::new(item, *p),
						ResolvedIdentifier::Declaration(decl),
					);
					decl
				}
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
					let (t, e) = self.collect_default_else(*f, origin.clone());
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
					let (t, e) = self.collect_default_else(*f, origin.clone());
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
					let origins = self
						.db
						.lookup_source_map(self.item.model_ref(self.db.upcast()));
					let origin = origins
						.get_origin(EntityRef::new(self.db.upcast(), self.item, *domain).into())
						.unwrap()
						.clone();
					// Note: unable to keep Entry until insert, since "self" is mutably borrowed to create the value
					#[allow(clippy::map_entry)]
					if !self.type_alias_domains.contains_key(&tr) {
						let mut declaration =
							<Item<Declaration>>::new(&DomainBuilder::unbounded(dom_ty));
						declaration.set_definition(&*self.collect_expression(*domain));
						let decl = self.declarations.insert(declaration);
						self.top_level.push(ItemId::Declaration(decl));
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
						let declaration = <Item<Declaration>>::new(&domain);
						self.declarations.insert(declaration)
					})
					.collect(),
			),
		}
	}
}

/// Lower a model to THIR
pub fn lower_model(db: &dyn Thir) -> Arc<Model> {
	let tys = TypeRegistry::new(db.upcast());
	let mut collector = ItemCollector::new(db, &tys);
	let items = db.lookup_topological_sorted_items();
	for item in items.iter() {
		collector.collect_item(*item);
	}
	Arc::new(collector.finish())
}
