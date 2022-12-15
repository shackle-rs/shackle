//! Functionality for converting HIR nodes into THIR nodes.
//!
//! The following is performed during lowering:
//! - Assignment items are moved into declarations/constraints
//! - Destructuring declarations are rewritten as separate declarations
//! - Destructuring in generators is rewritten into a where clause
//! - Type alias items removed as they have been resolved
//! - Array slicing is re-written using calls to `slice_Xd`
//!

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	arena::ArenaIndex,
	hir::{
		self,
		ids::{EntityRef, ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		IdentifierRegistry, PatternTy, TypeResult,
	},
	ty::{OptType, Ty, TyData, VarType},
};

use super::{
	db::Thir,
	source::{DesugarKind, Origin},
	*,
};

/// Collects HIR items and lowers them to THIR
struct ItemCollector<'a> {
	db: &'a dyn Thir,
	ids: &'a IdentifierRegistry,
	resolutions: FxHashMap<PatternRef, ResolvedIdentifier>,
	model: Model,
	type_alias_expressions: FxHashMap<ExpressionRef, DeclarationId>,
	deferred: Vec<(FunctionId, ItemRef)>,
}

impl<'a> ItemCollector<'a> {
	/// Create a new item collector
	pub fn new(db: &'a dyn Thir, ids: &'a IdentifierRegistry) -> Self {
		Self {
			db,
			ids,
			resolutions: FxHashMap::default(),
			model: Model::default(),
			type_alias_expressions: FxHashMap::default(),
			deferred: Vec::new(),
		}
	}

	/// Collect an item
	pub fn collect_item(&mut self, item: ItemRef) {
		let model = item.model(self.db.upcast());
		let local_item = item.local_item_ref(self.db.upcast());
		match local_item {
			LocalItemRef::Annotation(a) => {
				self.collect_annotation(item, &model[a]);
			}
			LocalItemRef::Assignment(a) => self.collect_assignment(item, &model[a]),
			LocalItemRef::Constraint(c) => {
				self.collect_constraint(item, &model[c], &model[c].data, true);
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
		a: &hir::Item<hir::Annotation>,
	) -> AnnotationId {
		let types = self.db.lookup_item_types(item);
		let ty = &types[a.constructor_pattern()];
		match (&a.constructor, ty) {
			(hir::Constructor::Atom { pattern }, PatternTy::AnnotationAtom) => {
				let idx = self.model.add_annotation(AnnotationItem::new(
					a.data[*pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
					item,
				));
				self.resolutions.insert(
					PatternRef::new(item, *pattern),
					ResolvedIdentifier::Annotation(idx),
				);
				idx
			}
			(
				hir::Constructor::Function {
					constructor,
					destructor,
					parameters: params,
				},
				PatternTy::AnnotationConstructor(fn_entry),
			) => {
				let mut parameters = Vec::with_capacity(fn_entry.overload.params().len());
				for (param, ty) in params.iter().zip(fn_entry.overload.params()) {
					let param_decl = self
						.model
						.add_declaration(DeclarationItem::new(false, item));
					let mut collector =
						ExpressionCollector::new(self, &a.data, item, param_decl, &types);
					let domain = collector.collect_domain(param.declared_type, *ty, false);
					self.model[param_decl].set_domain(domain);
					// Ignore destructuring and recording resolution for now since these can't have bodies which refer
					// to parameters anyway
					if let Some(p) = param.pattern {
						if let Some(i) = a.data[p].identifier() {
							self.model[param_decl].set_name(i);
						}
					}
					parameters.push(param_decl);
				}
				let mut annotation = AnnotationItem::new(
					a.data[*constructor]
						.identifier()
						.expect("Annotation must have identifier pattern"),
					item,
				);
				annotation.parameters = Some(parameters);
				let idx = self.model.add_annotation(annotation);
				self.resolutions.insert(
					PatternRef::new(item, *constructor),
					ResolvedIdentifier::Annotation(idx),
				);
				self.resolutions.insert(
					PatternRef::new(item, *destructor),
					ResolvedIdentifier::AnnotationDestructure(idx),
				);
				idx
			}
			_ => unreachable!(),
		}
	}

	/// Collect an assignment item
	pub fn collect_assignment(&mut self, item: ItemRef, a: &hir::Item<hir::Assignment>) {
		let db = self.db;
		let types = db.lookup_item_types(item);
		let res = types.name_resolution(a.assignee).unwrap();
		let decl = match &self.resolutions[&res] {
			ResolvedIdentifier::Declaration(d) => *d,
			_ => unreachable!(),
		};
		if self.model[decl].definition().is_some() {
			// Turn subsequent assignment items into equality constraints
			let constraint = self.model.add_constraint(ConstraintItem::new(true, item));
			let mut collector = ExpressionCollector::new(self, &a.data, item, constraint, &types);
			let lhs = collector.collect_expression(a.assignee);
			let rhs = collector.collect_expression(a.definition);
			let call = ExpressionBuilder::new(db, &self.model)
				.lookup_call(
					self.ids.eq,
					[lhs, rhs],
					self.model[constraint].expressions(),
				)
				.finish(self.model[constraint].expressions_mut(), item);
			self.model[constraint].set_expression(call);
		} else {
			let mut collector = ExpressionCollector::new(self, &a.data, item, decl, &types);
			let def = collector.collect_expression(a.definition);
			self.model[decl].set_definition(def);
		}
	}

	/// Collect a constraint item
	pub fn collect_constraint(
		&mut self,
		item: ItemRef,
		c: &hir::Constraint,
		data: &hir::ItemData,
		top_level: bool,
	) -> ConstraintId {
		let types = self.db.lookup_item_types(item);
		let constraint = self
			.model
			.add_constraint(ConstraintItem::new(top_level, item));
		let mut collector = ExpressionCollector::new(self, data, item, constraint, &types);
		let value = collector.collect_expression(c.expression);
		collector.parent.model[constraint].set_expression(value);
		for ann in c.annotations.iter() {
			let a = collector.collect_expression(*ann);
			collector.parent.model[constraint].add_annotation(a);
		}
		constraint
	}

	/// Collect a declaration item
	pub fn collect_declaration(
		&mut self,
		item: ItemRef,
		d: &hir::Declaration,
		data: &hir::ItemData,
		top_level: bool,
	) -> Vec<DeclarationId> {
		let types = self.db.lookup_item_types(item);

		let ty = match &types[d.pattern] {
			PatternTy::Variable(ty) => *ty,
			PatternTy::Destructuring(ty) => *ty,
			_ => unreachable!(),
		};
		let decl = self
			.model
			.add_declaration(DeclarationItem::new(top_level, item));
		let mut collector = ExpressionCollector::new(self, data, item, decl, &types);
		let domain = collector.collect_domain(d.declared_type, ty, false);
		collector.parent.model[decl].set_domain(domain);
		let decls = collector.collect_destructuring(decl, top_level, d.pattern);
		for ann in d.annotations.iter() {
			let a = collector.collect_expression(*ann);
			collector.parent.model[decl].add_annotation(a);
		}
		if let Some(def) = d.definition {
			let e = collector.collect_expression(def);
			collector.parent.model[decl].set_definition(e);
		}
		[decl].into_iter().chain(decls).collect()
	}

	/// Collect an enumeration item
	pub fn collect_enumeration(
		&mut self,
		item: ItemRef,
		e: &hir::Item<hir::Enumeration>,
	) -> EnumerationId {
		let types = self.db.lookup_item_types(item);
		let ty = &types[e.pattern];
		match ty {
			PatternTy::Variable(ty) => match ty.lookup(self.db.upcast()) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(self.db.upcast()) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = EnumerationItem::new(t, item);
							if let Some(def) = &e.definition {
								enumeration.set_definition(
									def.iter()
										.map(|c| self.collect_enum_case(c, &e.data, item, &types)),
								)
							}
							let idx = self.model.add_enumeration(enumeration);
							self.resolutions.insert(
								PatternRef::new(item, e.pattern),
								ResolvedIdentifier::Enumeration(idx),
							);
							self.add_enum_resolutions(
								idx,
								item,
								e.definition.iter().flat_map(|cs| cs.iter()),
							);
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
		a: &hir::Item<hir::EnumAssignment>,
	) {
		let types = self.db.lookup_item_types(item);
		let res = types.name_resolution(a.assignee).unwrap();
		let idx = match &self.resolutions[&res] {
			ResolvedIdentifier::Enumeration(e) => *e,
			_ => unreachable!(),
		};
		let def = a
			.definition
			.iter()
			.map(|c| self.collect_enum_case(c, &a.data, item, &types))
			.collect::<Vec<_>>();
		self.model[idx].set_definition(def);
		self.add_enum_resolutions(idx, item, a.definition.iter());
	}

	fn add_enum_resolutions<'i>(
		&mut self,
		idx: EnumerationId,
		item: ItemRef,
		ecs: impl Iterator<Item = &'i hir::EnumConstructor>,
	) {
		for (i, ec) in ecs.enumerate() {
			match ec {
				hir::EnumConstructor::Named(hir::Constructor::Atom { pattern }) => {
					self.resolutions.insert(
						PatternRef::new(item, *pattern),
						ResolvedIdentifier::EnumerationMember(EnumMemberId::new(idx, i as u32)),
					);
				}
				hir::EnumConstructor::Named(hir::Constructor::Function {
					constructor,
					destructor,
					..
				}) => {
					self.resolutions.insert(
						PatternRef::new(item, *constructor),
						ResolvedIdentifier::EnumerationMember(EnumMemberId::new(idx, i as u32)),
					);
					self.resolutions.insert(
						PatternRef::new(item, *destructor),
						ResolvedIdentifier::EnumerationDestructure(EnumMemberId::new(
							idx, i as u32,
						)),
					);
				}
				_ => (),
			}
		}
	}

	fn collect_enum_case(
		&mut self,
		c: &hir::EnumConstructor,
		data: &hir::ItemData,
		item: ItemRef,
		types: &TypeResult,
	) -> Constructor {
		let (name, params) = match (c, &types[c.constructor_pattern()]) {
			(crate::hir::EnumConstructor::Named(crate::hir::Constructor::Atom { pattern }), _) => {
				return Constructor {
					name: data[*pattern].identifier(),
					parameters: None,
				}
			}
			(
				crate::hir::EnumConstructor::Named(crate::hir::Constructor::Function {
					constructor,
					parameters,
					..
				}),
				PatternTy::EnumConstructor(ecs),
			) => (
				data[*constructor].identifier(),
				ecs[0]
					.overload
					.params()
					.iter()
					.zip(parameters.iter())
					.map(|(ty, t)| (*ty, t.declared_type))
					.collect::<Vec<_>>(),
			),
			(
				crate::hir::EnumConstructor::Anonymous { parameters, .. },
				PatternTy::AnonymousEnumConstructor(f),
			) => (
				None,
				f.overload
					.params()
					.iter()
					.zip(parameters.iter())
					.map(|(ty, t)| (*ty, t.declared_type))
					.collect::<Vec<_>>(),
			),
			_ => unreachable!(),
		};

		Constructor {
			name,
			parameters: Some(
				params
					.iter()
					.map(|(ty, t)| {
						let declaration = self
							.model
							.add_declaration(DeclarationItem::new(false, item));
						let mut collector =
							ExpressionCollector::new(self, data, item, declaration, types);
						let domain = collector.collect_domain(*t, *ty, false);
						self.model[declaration].set_domain(domain);
						declaration
					})
					.collect(),
			),
		}
	}

	/// Collect a function item
	pub fn collect_function(&mut self, item: ItemRef, f: &hir::Item<hir::Function>) -> FunctionId {
		let types = self.db.lookup_item_types(item);
		let function = self.model.add_function(FunctionItem::new(
			f.data[f.pattern].identifier().unwrap(),
			item,
		));
		let mut collector = ExpressionCollector::new(self, &f.data, item, function, &types);
		for ann in f.annotations.iter() {
			let a = collector.collect_expression(*ann);
			collector.parent.model[function].add_annotation(a);
		}
		let res = PatternRef::new(item, f.pattern);
		collector
			.parent
			.resolutions
			.insert(res, ResolvedIdentifier::Function(function));
		match &types[f.pattern] {
			PatternTy::Function(fn_entry) => {
				let domain =
					collector.collect_domain(f.return_type, fn_entry.overload.return_type(), false);
				collector.parent.model[function].set_domain(domain);

				collector.parent.model[function].set_type_inst_vars(f.type_inst_vars.iter().map(
					|t| match &types[t.name] {
						PatternTy::TyVar(tv) => tv.clone(),
						_ => unreachable!(),
					},
				));

				let parameters = f
					.parameters
					.iter()
					.zip(fn_entry.overload.params())
					.map(|(param, ty)| {
						collector
							.parent
							.collect_fn_param(param, *ty, &f.data, item, &types)
					})
					.collect::<Vec<_>>();
				collector.parent.model[function].set_parameters(parameters);
				if f.body.is_some() {
					self.deferred.push((function, item));
				}
				function
			}
			_ => unreachable!(),
		}
	}

	fn collect_fn_param(
		&mut self,
		param: &crate::hir::Parameter,
		ty: Ty,
		data: &hir::ItemData,
		item: ItemRef,
		types: &TypeResult,
	) -> DeclarationId {
		let declaration = self
			.model
			.add_declaration(DeclarationItem::new(false, item));
		let mut collector = ExpressionCollector::new(self, data, item, declaration, types);
		let domain = collector.collect_domain(param.declared_type, ty, false);
		collector.parent.model[declaration].set_domain(domain);
		for ann in param.annotations.iter() {
			let a = collector.collect_expression(*ann);
			collector.parent.model[declaration].add_annotation(a);
		}
		declaration
	}

	/// Collect an output item
	pub fn collect_output(&mut self, item: ItemRef, o: &hir::Item<hir::Output>) -> OutputId {
		let types = self.db.lookup_item_types(item);
		let output = self.model.add_output(OutputItem::new(item));
		let mut collector = ExpressionCollector::new(self, &o.data, item, output, &types);
		let e = collector.collect_expression(o.expression);
		collector.parent.model[output].set_expression(e);
		if let Some(s) = o.section {
			let section = collector.collect_expression(s);
			collector.parent.model[output].set_section(section);
		}
		output
	}

	/// Collect solve item
	pub fn collect_solve(&mut self, item: ItemRef, s: &hir::Item<hir::Solve>) {
		let types = self.db.lookup_item_types(item);
		let mut optimise = |pattern: ArenaIndex<hir::Pattern>,
		                    objective: ArenaIndex<hir::Expression>,
		                    is_maximize: bool| match &types[pattern] {
			PatternTy::Variable(ty) => {
				let declaration = self.model.add_declaration(DeclarationItem::new(
					true,
					Origin::from(item).with_desugaring(DesugarKind::Objective),
				));
				self.model[declaration].set_domain(Domain::unbounded(*ty));
				if let Some(name) = s.data[pattern].identifier() {
					self.model[declaration].set_name(name);
				}
				let mut collector =
					ExpressionCollector::new(self, &s.data, item, declaration, &types);
				let obj = collector.collect_expression(objective);
				self.model[declaration].set_definition(obj);
				self.resolutions.insert(
					PatternRef::new(item, pattern),
					ResolvedIdentifier::Declaration(declaration),
				);
				if is_maximize {
					SolveItem::maximize(declaration, item)
				} else {
					SolveItem::minimize(declaration, item)
				}
			}
			_ => unreachable!(),
		};
		let si = match &s.goal {
			hir::Goal::Maximize { pattern, objective } => optimise(*pattern, *objective, true),
			hir::Goal::Minimize { pattern, objective } => optimise(*pattern, *objective, false),
			hir::Goal::Satisfy => SolveItem::satisfy(item),
		};
		let solve = self.model.set_solve(si);
		let mut collector = ExpressionCollector::new(self, &s.data, item, solve, &types);
		for ann in s.annotations.iter() {
			let a = collector.collect_expression(*ann);
			collector
				.parent
				.model
				.solve_mut()
				.unwrap()
				.add_annotation(a);
		}
	}

	fn collect_type_alias(&mut self, item: ItemRef, ta: &hir::Item<hir::TypeAlias>) {
		let types = self.db.lookup_item_types(item);
		for e in hir::Type::expressions(ta.aliased_type, &ta.data) {
			if let Some(res) = types.name_resolution(e) {
				let res_types = self.db.lookup_item_types(res.item());
				if matches!(&res_types[res.pattern()], PatternTy::TypeAlias(_)) {
					// Skip type aliases inside other type aliases (already will be processed)
					continue;
				}
			}
			// Create a declaration with the value of each expression used in a type alias
			let decl = self.model.add_declaration(DeclarationItem::new(
				true,
				EntityRef::new(self.db.upcast(), item, e),
			));
			let expression =
				ExpressionCollector::new(self, &ta.data, item, decl, &types).collect_expression(e);
			self.model[decl].set_definition(expression);
			self.type_alias_expressions
				.insert(ExpressionRef::new(item, e), decl);
		}
	}

	/// Collect deferred function bodies
	pub fn collect_deferred(&mut self) {
		for (func, item) in self.deferred.clone().into_iter() {
			let types = self.db.lookup_item_types(item);
			let model = item.model(self.db.upcast());
			let local_item = item.local_item_ref(self.db.upcast());
			match local_item {
				LocalItemRef::Function(f) => {
					let mut decls = Vec::new();
					let param_decls = self.model[func].parameters().to_owned();
					for (decl, param) in param_decls.into_iter().zip(model[f].parameters.iter()) {
						if let Some(p) = param.pattern {
							let dsts =
								ExpressionCollector::new(self, &model[f].data, item, decl, &types)
									.collect_destructuring(decl, false, p);
							decls.extend(dsts);
						}
					}
					let body = model[f].body.unwrap();
					let mut collector =
						ExpressionCollector::new(self, &model[f].data, item, func, &types);
					let collected_body = collector.collect_expression(body);
					let e = if decls.is_empty() {
						collected_body
					} else {
						let origin = EntityRef::new(collector.parent.db.upcast(), item, body);
						collector
							.builder()
							.let_expression(
								decls.into_iter().map(LetItem::Declaration),
								collected_body,
								collector.expressions(),
							)
							.finish(collector.expressions_mut(), origin)
					};
					self.model[func].set_body(e);
				}
				_ => unreachable!(),
			}
		}
	}

	/// Finish lowering
	pub fn finish(self) -> Model {
		self.model
	}
}

struct ExpressionCollector<'a, 'b> {
	parent: &'a mut ItemCollector<'b>,
	data: &'a hir::ItemData,
	item: ItemRef,
	thir_item: ItemId,
	types: &'a TypeResult,
}

impl<'a, 'b> ExpressionCollector<'a, 'b> {
	fn new(
		parent: &'a mut ItemCollector<'b>,
		data: &'a crate::hir::ItemData,
		item: ItemRef,
		thir_item: impl Into<ItemId>,
		types: &'a TypeResult,
	) -> Self {
		Self {
			parent,
			data,
			item,
			thir_item: thir_item.into(),
			types,
		}
	}

	fn builder(&self) -> ExpressionBuilder<'_> {
		ExpressionBuilder::new(self.parent.db, &self.parent.model)
	}

	fn introduce_declaration(
		&mut self,
		top_level: bool,
		origin: impl Into<Origin>,
		f: impl FnOnce(&mut ExpressionCollector<'_, '_>, DeclarationId) -> ExpressionId,
	) -> DeclarationId {
		let decl = DeclarationItem::new(top_level, origin);
		let idx = self.parent.model.add_declaration(decl);
		let mut collector =
			ExpressionCollector::new(self.parent, self.data, self.item, idx, self.types);
		let def = f(&mut collector, idx);
		collector.parent.model[idx].set_definition(def);
		idx
	}

	fn expressions(&self) -> &'_ ExpressionAllocator {
		self.parent.model.expressions(self.thir_item)
	}

	fn expressions_mut(&mut self) -> &'_ mut ExpressionAllocator {
		self.parent.model.expressions_mut(self.thir_item)
	}

	/// Collect an expression
	pub fn collect_expression(&mut self, idx: ArenaIndex<hir::Expression>) -> ExpressionId {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = EntityRef::new(db.upcast(), self.item, idx);
		let annotations = self
			.data
			.annotations(idx)
			.map(|ann| self.collect_expression(ann))
			.collect::<Vec<_>>();
		let result = match &self.data[idx] {
			hir::Expression::Absent => self
				.builder()
				.absent()
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::ArrayAccess(aa) => {
				let is_slice = match self.types[aa.indices].lookup(db.upcast()) {
					TyData::Tuple(_, fs) => fs.iter().any(|f| f.is_set(db.upcast())),
					TyData::Set(_, _, _) => true,
					_ => false,
				};
				if is_slice {
					self.collect_slice(aa.collection, aa.indices, origin)
						.with_annotations(annotations)
						.finish(self.expressions_mut(), origin)
				} else {
					let collection = self.collect_expression(aa.collection);
					let indices = self.collect_expression(aa.indices);
					self.builder()
						.array_access(collection, indices, self.expressions())
						.with_annotations(annotations)
						.finish(self.expressions_mut(), origin)
				}
			}
			hir::Expression::ArrayComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				let template = self.collect_expression(c.template);
				if let Some(indices) = c.indices {
					let indices = self.collect_expression(indices);
					self.builder()
						.indexed_array_comprehension(
							generators,
							indices,
							template,
							self.expressions(),
						)
						.with_annotations(annotations)
						.finish(self.expressions_mut(), origin)
				} else {
					self.builder()
						.array_comprehension(generators, template, self.expressions())
						.with_annotations(annotations)
						.finish(self.expressions_mut(), origin)
				}
			}
			hir::Expression::ArrayLiteral(al) => {
				let members = al
					.members
					.iter()
					.map(|m| self.collect_expression(*m))
					.collect::<Vec<_>>();
				self.builder()
					.array(members, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::BooleanLiteral(b) => self
				.builder()
				.boolean(*b)
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::Call(c) => {
				let function = self.collect_expression(c.function);
				let arguments = c
					.arguments
					.iter()
					.map(|arg| self.collect_expression(*arg))
					.collect::<Vec<_>>();
				self.builder()
					.call(function, arguments, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::Case(c) => {
				let scrutinee_origin =
					EntityRef::new(self.parent.db.upcast(), self.item, c.expression);
				let scrutinee =
					self.introduce_declaration(false, scrutinee_origin, |collector, _| {
						collector.collect_expression(c.expression)
					});
				let branches = c
					.cases
					.iter()
					.map(|case| {
						let pattern_origin =
							EntityRef::new(self.parent.db.upcast(), self.item, case.pattern);
						let pattern = self.collect_pattern(case.pattern);
						let decls = self.collect_destructuring(scrutinee, false, case.pattern);
						let result = self.collect_expression(case.value);
						if decls.is_empty() {
							CaseBranch::new(pattern, result)
						} else {
							CaseBranch::new(
								pattern,
								self.builder()
									.let_expression(
										decls.into_iter().map(LetItem::Declaration),
										result,
										self.expressions(),
									)
									.finish(self.expressions_mut(), pattern_origin),
							)
						}
					})
					.collect::<Vec<_>>();
				let scrutinee_ident = self
					.builder()
					.identifier(scrutinee)
					.finish(self.expressions_mut(), scrutinee_origin);
				let case_expression = self
					.builder()
					.case(scrutinee_ident, branches, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin);
				self.builder()
					.let_expression(
						[LetItem::Declaration(scrutinee)],
						case_expression,
						self.expressions(),
					)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::FloatLiteral(f) => self
				.builder()
				.float(*f)
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(idx).unwrap();
				let ident = ExpressionData::Identifier(
					self.parent
						.resolutions
						.get(&res)
						.unwrap_or_else(|| {
							panic!(
								"Did not lower {:?} at {:?} used by {:?} at {:?}",
								res,
								NodeRef::from(res.into_entity(self.parent.db.upcast()))
									.source_span(self.parent.db.upcast()),
								ExpressionRef::new(self.item, idx),
								NodeRef::from(EntityRef::new(
									self.parent.db.upcast(),
									self.item,
									idx
								))
								.source_span(self.parent.db.upcast()),
							)
						})
						.clone(),
				);
				let allocator = self.expressions_mut();
				let idx = allocator.new_unchecked(origin, ty, ident);
				for ann in annotations {
					allocator.annotate_expression(idx, ann);
				}
				idx
			}
			hir::Expression::IfThenElse(ite) => {
				let branches = ite
					.branches
					.iter()
					.map(|b| {
						Branch::new(
							self.collect_expression(b.condition),
							self.collect_expression(b.result),
						)
					})
					.collect::<Vec<_>>();
				let else_result = if let Some(e) = ite.else_result {
					self.collect_expression(e)
				} else {
					self.collect_default_else(ty, origin.into())
				};
				self.builder()
					.if_then_else(branches, else_result, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::Infinity => self
				.builder()
				.infinity()
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::IntegerLiteral(i) => self
				.builder()
				.integer(*i)
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::Lambda(l) => {
				let fn_type = match ty.lookup(db.upcast()) {
					TyData::Function(_, f) => f,
					_ => unreachable!(),
				};
				let return_type = l
					.return_type
					.map(|r| self.collect_domain(r, fn_type.return_type, false))
					.unwrap_or_else(|| Domain::unbounded(fn_type.return_type));
				let mut decls = Vec::new();
				let parameters = l
					.parameters
					.iter()
					.zip(fn_type.params.iter())
					.map(|(param, ty)| {
						let decl = self
							.parent
							.collect_fn_param(param, *ty, self.data, self.item, self.types);
						if let Some(p) = param.pattern {
							decls.extend(self.collect_destructuring(decl, false, p));
						}
						decl
					})
					.collect::<Vec<_>>();
				let body = self.collect_expression(l.body);
				let b = if decls.is_empty() {
					body
				} else {
					let body_entity = EntityRef::new(self.parent.db.upcast(), self.item, l.body);
					self.builder()
						.let_expression(
							decls.into_iter().map(LetItem::Declaration),
							body,
							self.expressions(),
						)
						.finish(self.expressions_mut(), body_entity)
				};
				self.builder()
					.lambda(return_type, parameters, b)
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::Let(l) => {
				let items = l
					.items
					.iter()
					.flat_map(|i| match i {
						hir::LetItem::Constraint(c) => {
							let constraint = self
								.parent
								.collect_constraint(self.item, c, self.data, false);
							vec![LetItem::Constraint(constraint)]
						}
						hir::LetItem::Declaration(d) => self
							.parent
							.collect_declaration(self.item, d, self.data, false)
							.into_iter()
							.map(LetItem::Declaration)
							.collect::<Vec<_>>(),
					})
					.collect::<Vec<_>>();
				let in_expression = self.collect_expression(l.in_expression);
				self.builder()
					.let_expression(items, in_expression, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::RecordAccess(ra) => {
				let record = self.collect_expression(ra.record);
				self.builder()
					.record_access(record, ra.field, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::RecordLiteral(rl) => {
				let fields = rl
					.fields
					.iter()
					.map(|(i, v)| {
						(
							self.data[*i].identifier().unwrap(),
							self.collect_expression(*v),
						)
					})
					.collect::<Vec<_>>();
				self.builder()
					.record(fields, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::SetComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				let template = self.collect_expression(c.template);
				self.builder()
					.set_comprehension(generators, template, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::SetLiteral(sl) => {
				let members = sl
					.members
					.iter()
					.map(|m| self.collect_expression(*m))
					.collect::<Vec<_>>();
				self.builder()
					.set(members, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::Slice(_) => {
				unreachable!("Slice used outside of array access")
			}
			hir::Expression::StringLiteral(sl) => self
				.builder()
				.string(sl.clone())
				.with_annotations(annotations)
				.finish(self.expressions_mut(), origin),
			hir::Expression::TupleAccess(ta) => {
				let tuple = self.collect_expression(ta.tuple);
				self.builder()
					.tuple_access(tuple, ta.field, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			hir::Expression::TupleLiteral(tl) => {
				let fields = tl
					.fields
					.iter()
					.map(|f| self.collect_expression(*f))
					.collect::<Vec<_>>();
				self.builder()
					.tuple(fields, self.expressions())
					.with_annotations(annotations)
					.finish(self.expressions_mut(), origin)
			}
			_ => unimplemented!("{:?}", &self.data[idx]),
		};
		let result_ty = self.expressions()[result].ty();
		assert_eq!(
			result_ty,
			ty,
			"Type by construction ({}) disagrees with typechecker ({}) at {:?}",
			result_ty.pretty_print(db.upcast()),
			ty.pretty_print(db.upcast()),
			NodeRef::from(origin).source_span(db.upcast())
		);
		result
	}

	/// Rewrite index slicing into slice_Xd() call
	fn collect_slice(
		&mut self,
		collection: ArenaIndex<hir::Expression>,
		indices: ArenaIndex<hir::Expression>,
		origin: impl Into<Origin>,
	) -> ExpressionBuilderWithData {
		let collection_entity = EntityRef::new(self.parent.db.upcast(), self.item, collection);
		let indices_entity = EntityRef::new(self.parent.db.upcast(), self.item, indices);

		let mut decls = Vec::new();
		let collection_decl = if matches!(&self.data[collection], hir::Expression::Identifier(_)) {
			let idx = self.collect_expression(collection);
			match &*self.expressions()[idx] {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(decl)) => *decl,
				_ => unreachable!(),
			}
		} else {
			// Add declaration to store collection
			let origin = EntityRef::new(self.parent.db.upcast(), self.item, collection);
			let decl = self.introduce_declaration(false, origin, |collector, _| {
				collector.collect_expression(collection)
			});
			decls.push(decl);
			decl
		};
		let array_dims = self.types[collection]
			.dims(self.parent.db.upcast())
			.unwrap();
		let mut slices = Vec::with_capacity(array_dims);
		match self.types[indices].lookup(self.parent.db.upcast()) {
			TyData::Tuple(_, fs) => {
				if let hir::Expression::TupleLiteral(tl) = &self.data[indices] {
					for (i, (ty, e)) in fs.iter().zip(tl.fields.iter()).enumerate() {
						let index_entity = EntityRef::new(self.parent.db.upcast(), self.item, *e);
						let decl =
							self.introduce_declaration(false, index_entity, |collector, decl| {
								if let hir::Expression::Slice(s) = &collector.data[*e] {
									// Rewrite infinite slice .. into `'..'(index_set_mofn(c))`
									let collection_ident = collector
										.builder()
										.identifier(collection_decl)
										.finish(collector.expressions_mut(), collection_entity);
									let index_set = collector
										.builder()
										.lookup_call(
											Identifier::new(
												format!("index_set_{}of{}", i + 1, array_dims),
												collector.parent.db.upcast(),
											),
											[collection_ident],
											collector.expressions(),
										)
										.finish(collector.expressions_mut(), index_entity);
									let call = collector
										.builder()
										.lookup_call(*s, [index_set], collector.expressions())
										.finish(collector.expressions_mut(), index_entity);
									slices.push((decl, true, index_entity));
									call
								} else if ty.is_set(collector.parent.db.upcast()) {
									// Slice
									slices.push((decl, true, index_entity));
									collector.collect_expression(*e)
								} else {
									// Rewrite index as slice of {i}
									let index = collector.collect_expression(*e);
									let set = collector
										.builder()
										.set([index], collector.expressions())
										.finish(collector.expressions_mut(), index_entity);
									slices.push((decl, false, index_entity));
									set
								}
							});
						decls.push(decl);
					}
				} else {
					// Expression which evaluates to a tuple
					let indices_decl =
						self.introduce_declaration(false, indices_entity, |collector, _| {
							collector.collect_expression(indices)
						});
					decls.push(indices_decl);
					for (i, f) in fs.iter().enumerate() {
						// Create declaration for each index
						let is_set = f.is_set(self.parent.db.upcast());
						let accessor =
							self.introduce_declaration(false, indices_entity, |collector, decl| {
								let ident = collector
									.builder()
									.identifier(indices_decl)
									.finish(collector.expressions_mut(), indices_entity);
								let ta = collector
									.builder()
									.tuple_access(
										ident,
										IntegerLiteral(i as i64 + 1),
										collector.expressions(),
									)
									.finish(collector.expressions_mut(), indices_entity);
								if is_set {
									slices.push((decl, true, indices_entity));
									ta
								} else {
									// Rewrite as {i}
									let sl = collector
										.builder()
										.set([ta], collector.expressions())
										.finish(collector.expressions_mut(), indices_entity);
									slices.push((decl, false, indices_entity));
									sl
								}
							});
						decls.push(accessor);
					}
				}
			}
			_ => {
				// 1D slicing, so must be a set index
				let decl = self.introduce_declaration(false, indices_entity, |collector, decl| {
					if let hir::Expression::Slice(s) = &collector.data[indices] {
						// Rewrite infinite slice .. into `'..'(index_set(c))`
						let collection_ident = collector
							.builder()
							.identifier(collection_decl)
							.finish(collector.expressions_mut(), collection_entity);
						let index_set = collector
							.builder()
							.lookup_call(
								collector.parent.ids.index_set,
								[collection_ident],
								collector.expressions(),
							)
							.finish(collector.expressions_mut(), indices_entity);
						let call = collector
							.builder()
							.lookup_call(*s, [index_set], collector.expressions())
							.finish(collector.expressions_mut(), indices_entity);
						slices.push((decl, true, indices_entity));
						call
					} else {
						let rhs = collector.collect_expression(indices);
						slices.push((decl, true, indices_entity));
						rhs
					}
				});
				decls.push(decl);
			}
		}

		let collection_ident = self
			.builder()
			.identifier(collection_decl)
			.finish(self.expressions_mut(), collection_entity);
		let slice_elements = slices
			.iter()
			.map(|(decl, _, origin)| {
				let ident = self
					.builder()
					.identifier(*decl)
					.finish(self.expressions_mut(), *origin);
				self.builder()
					.lookup_call(self.parent.ids.erase_enum, [ident], self.expressions())
					.finish(self.expressions_mut(), *origin)
			})
			.collect::<Vec<_>>();
		let slice_array = self
			.builder()
			.array(slice_elements, self.expressions())
			.finish(self.expressions_mut(), indices_entity);
		let dims_elements = slices
			.iter()
			.filter_map(|(decl, is_slice, origin)| {
				if *is_slice {
					Some(
						self.builder()
							.identifier(*decl)
							.finish(self.expressions_mut(), *origin),
					)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		let call = self
			.builder()
			.lookup_call(
				Identifier::new(
					format!("slice_{}d", dims_elements.len()),
					self.parent.db.upcast(),
				),
				[collection_ident, slice_array]
					.into_iter()
					.chain(dims_elements),
				self.expressions(),
			)
			.finish(self.expressions_mut(), origin);
		self.builder().let_expression(
			decls.into_iter().map(LetItem::Declaration),
			call,
			self.expressions(),
		)
	}

	fn collect_generator(&mut self, generator: &hir::Generator, generators: &mut Vec<Generator>) {
		match generator {
			hir::Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				let mut assignments = Vec::new();
				let mut where_clauses = Vec::new();
				let declarations = patterns
					.iter()
					.map(|p| {
						let origin = EntityRef::new(self.parent.db.upcast(), self.item, *p);
						let ty = match &self.types[*p] {
							PatternTy::Variable(ty) | PatternTy::Destructuring(ty) => *ty,
							_ => unreachable!(),
						};
						let mut declaration = DeclarationItem::new(false, origin);
						declaration.set_domain(Domain::unbounded(ty));
						let decl = self.parent.model.add_declaration(declaration);
						let asgs = self.collect_destructuring(decl, false, *p);
						if !asgs.is_empty() {
							// Turn destructuring into where clause of case matching pattern
							let pattern = self.collect_pattern(*p);
							let ident = self
								.builder()
								.identifier(decl)
								.finish(self.expressions_mut(), origin);
							let branches = [
								CaseBranch::new(
									pattern,
									self.builder()
										.boolean(BooleanLiteral(true))
										.finish(self.expressions_mut(), origin),
								),
								CaseBranch::new(
									Pattern::Anonymous(match &self.types[*p] {
										PatternTy::Destructuring(ty) => *ty,
										_ => unreachable!(),
									}),
									self.builder()
										.boolean(BooleanLiteral(false))
										.finish(self.expressions_mut(), origin),
								),
							];
							let case = self
								.builder()
								.case(ident, branches, self.expressions())
								.finish(self.expressions_mut(), origin);
							where_clauses.push(case);
						}
						assignments.extend(asgs);
						decl
					})
					.collect();
				let collection = self.collect_expression(*collection);
				let where_clause = where_clause.map(|w| self.collect_expression(w));
				if assignments.is_empty() {
					generators.push(Generator::Iterator {
						declarations,
						collection,
						where_clause,
					});
				} else {
					// Add destructuring assignments and new where clause
					let origin = EntityRef::new(self.parent.db.upcast(), self.item, patterns[0]);
					if where_clauses.len() == 1 {
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(where_clauses[0]),
						});
					} else {
						let arguments = self
							.builder()
							.array(where_clauses, self.expressions())
							.finish(self.expressions_mut(), origin);
						let call = self
							.builder()
							.lookup_call(self.parent.ids.forall, [arguments], self.expressions())
							.finish(self.expressions_mut(), origin);
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(call),
						});
					}
					let last = assignments.len() - 1;
					for (i, assignment) in assignments.into_iter().enumerate() {
						generators.push(Generator::Assignment {
							assignment,
							where_clause: if i == last { where_clause } else { None },
						});
					}
				}
			}
			hir::Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let assignment = self.parent.model.add_declaration(DeclarationItem::new(
					false,
					EntityRef::new(self.parent.db.upcast(), self.item, *pattern),
				));
				let def = ExpressionCollector::new(
					self.parent,
					self.data,
					self.item,
					assignment,
					self.types,
				)
				.collect_expression(*value);
				self.parent.model[assignment].set_definition(def);
				if let Some(name) = self.data[*pattern].identifier() {
					self.parent.model[assignment].set_name(name);
					self.parent.resolutions.insert(
						PatternRef::new(self.item, *pattern),
						ResolvedIdentifier::Declaration(assignment),
					);
				}
				generators.push(Generator::Assignment {
					assignment,
					where_clause: where_clause.map(|w| self.collect_expression(w)),
				});
			}
		}
	}

	fn collect_default_else(&mut self, ty: Ty, origin: Origin) -> ExpressionId {
		let db = self.parent.db;
		match ty.lookup(db.upcast()) {
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
			| TyData::TyVar(_, Some(OptType::Opt), _) => self
				.builder()
				.absent()
				.finish(self.expressions_mut(), origin),
			TyData::Boolean(_, _) => self
				.builder()
				.boolean(BooleanLiteral(true))
				.finish(self.expressions_mut(), origin),
			TyData::String(_) => self
				.builder()
				.string(StringLiteral::new("", db.upcast()))
				.finish(self.expressions_mut(), origin),
			TyData::Annotation(_) => self
				.builder()
				.lookup_identifier(self.parent.ids.empty_annotation)
				.finish(self.expressions_mut(), origin),
			TyData::Array { .. } => self
				.builder()
				.array([], self.expressions())
				.finish(self.expressions_mut(), origin),
			TyData::Set(_, _, _) => self
				.builder()
				.set([], self.expressions())
				.finish(self.expressions_mut(), origin),
			TyData::Tuple(_, fs) => {
				let fields = fs
					.iter()
					.map(|f| self.collect_default_else(*f, origin))
					.collect::<Vec<_>>();
				self.builder()
					.tuple(fields, self.expressions())
					.finish(self.expressions_mut(), origin)
			}
			TyData::Record(_, fs) => {
				let fields = fs
					.iter()
					.map(|(i, t)| (Identifier(*i), self.collect_default_else(*t, origin)))
					.collect::<Vec<_>>();
				self.builder()
					.record(fields, self.expressions())
					.finish(self.expressions_mut(), origin)
			}
			_ => unreachable!("No default value for this type"),
		}
	}

	// Collect a domain from a user ascribed type
	fn collect_domain(&mut self, t: ArenaIndex<hir::Type>, ty: Ty, is_type_alias: bool) -> Domain {
		let db = self.parent.db;
		match (&self.data[t], ty.lookup(db.upcast())) {
			(hir::Type::Bounded { domain, .. }, _) => {
				if let Some(res) = self.types.name_resolution(*domain) {
					let res_types = db.lookup_item_types(res.item());
					match &res_types[res.pattern()] {
						// Identifier is actually a type, not a domain expression
						PatternTy::TyVar(_) => return Domain::unbounded(ty),
						PatternTy::TypeAlias(_) => {
							let model = res.item().model(db.upcast());
							match res.item().local_item_ref(db.upcast()) {
								LocalItemRef::TypeAlias(ta) => {
									let mut c = ExpressionCollector::new(
										self.parent,
										&model[ta].data,
										res.item(),
										self.thir_item,
										&res_types,
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
					let er = ExpressionRef::new(self.item, *domain);
					let origin = EntityRef::new(db.upcast(), self.item, *domain);
					let ident = self
						.builder()
						.identifier(self.parent.type_alias_expressions[&er])
						.finish(self.expressions_mut(), origin);
					Domain::bounded(
						db,
						self.expressions(),
						ty.inst(db.upcast()).unwrap(),
						ty.opt(db.upcast()).unwrap(),
						ident,
					)
				} else {
					let e = self.collect_expression(*domain);
					Domain::bounded(
						db,
						self.expressions(),
						ty.inst(db.upcast()).unwrap(),
						ty.opt(db.upcast()).unwrap(),
						e,
					)
				}
			}
			(
				hir::Type::Array {
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
				Domain::array(db, dim, el)
			}
			(hir::Type::Set { element, .. }, TyData::Set(inst, opt, e)) => Domain::set(
				db,
				inst,
				opt,
				self.collect_domain(*element, e, is_type_alias),
			),
			(hir::Type::Tuple { fields, .. }, TyData::Tuple(_, fs)) => {
				let domains = fields
					.iter()
					.zip(fs.iter())
					.map(|(f, ty)| self.collect_domain(*f, *ty, is_type_alias))
					.collect::<Vec<_>>();
				Domain::tuple(db, domains)
			}
			(hir::Type::Record { fields, .. }, TyData::Record(_, fs)) => {
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
				Domain::record(db, domains)
			}
			_ => Domain::unbounded(ty),
		}
	}

	/// Create declarations which perform destructuring according to the given pattern
	fn collect_destructuring(
		&mut self,
		root_decl: DeclarationId,
		top_level: bool,
		pattern: ArenaIndex<hir::Pattern>,
	) -> Vec<DeclarationId> {
		let mut destructuring = Vec::new();
		let mut todo = vec![(0, pattern)];
		while let Some((i, p)) = todo.pop() {
			match &self.data[p] {
				hir::Pattern::Tuple { fields } => {
					for (idx, field) in fields.iter().enumerate() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				hir::Pattern::Record { fields } => {
					for (ident, field) in fields.iter() {
						// Destructuring returns the field inside
						destructuring.push(DestructuringEntry::new(
							i,
							Destructuring::RecordAccess(*ident),
							*field,
						));
						todo.push((destructuring.len(), *field));
					}
				}
				hir::Pattern::Call {
					function,
					arguments,
				} => {
					let destructuring_pattern = if arguments.len() == 1 {
						// If we have a single arg, destructuring will return the inside directly
						arguments[0]
					} else {
						// Destructuring returns a tuple
						p
					};
					let pat = self.types.pattern_resolution(*function).unwrap();
					let res = &self.parent.resolutions[&pat];
					match res {
						ResolvedIdentifier::Annotation(ann) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Annotation(*ann),
								destructuring_pattern,
							));
						}
						ResolvedIdentifier::EnumerationMember(member) => {
							let kind = match &self.types[p] {
								PatternTy::Destructuring(ty) => {
									EnumConstructorKind::from_ty(self.parent.db, *ty)
								}
								_ => unreachable!(),
							};
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Enumeration(*member, kind),
								destructuring_pattern,
							));
						}
						_ => unreachable!(),
					};
					let j = destructuring.len();
					if arguments.len() == 1 {
						todo.push((j, arguments[0]));
					} else {
						for (idx, field) in arguments.iter().enumerate() {
							// Destructuring the tuple returns the field inside
							destructuring.push(DestructuringEntry::new(
								j,
								Destructuring::TupleAccess(IntegerLiteral(idx as i64 + 1)),
								*field,
							));
							todo.push((destructuring.len(), *field));
						}
					}
				}
				hir::Pattern::Identifier(name) => {
					if let PatternTy::Variable(_) = &self.types[p] {
						if i > 0 {
							destructuring[i - 1].name = Some(*name);
							// Mark used destructurings as to be created
							let mut c = i;
							loop {
								if c == 0 {
									break;
								}
								let item = &mut destructuring[c - 1];
								if item.create {
									break;
								}
								item.create = true;
								c = item.parent;
							}
						} else {
							self.parent.model[root_decl].set_name(*name);
							self.parent.resolutions.insert(
								PatternRef::new(self.item, pattern),
								ResolvedIdentifier::Declaration(root_decl),
							);
						}
					}
				}
				_ => (),
			}
		}
		let mut decls = Vec::new();
		let mut decl_map = FxHashMap::default();
		for (idx, item) in destructuring
			.into_iter()
			.enumerate()
			.filter(|(_, item)| item.create)
		{
			let origin = EntityRef::new(self.parent.db.upcast(), self.item, item.pattern);
			let decl = self.introduce_declaration(top_level, origin, |collector, _| {
				let ident = collector
					.builder()
					.identifier(if item.parent == 0 {
						root_decl
					} else {
						decl_map[&item.parent]
					})
					.finish(collector.expressions_mut(), origin);
				match item.kind {
					Destructuring::Annotation(a) => {
						let dtor = collector
							.builder()
							.annotation_destructure(a)
							.finish(collector.expressions_mut(), origin);
						let call = collector
							.builder()
							.call(dtor, [ident], collector.expressions())
							.finish(collector.expressions_mut(), origin);
						call
					}
					Destructuring::Enumeration(e, k) => {
						let dtor = collector
							.builder()
							.enum_destructure(e, k)
							.finish(collector.expressions_mut(), origin);
						let call = collector
							.builder()
							.call(dtor, [ident], collector.expressions())
							.finish(collector.expressions_mut(), origin);
						call
					}
					Destructuring::RecordAccess(f) => collector
						.builder()
						.record_access(ident, f, collector.expressions())
						.finish(collector.expressions_mut(), origin),
					Destructuring::TupleAccess(f) => collector
						.builder()
						.tuple_access(ident, f, collector.expressions())
						.finish(collector.expressions_mut(), origin),
				}
			});
			if let Some(name) = item.name {
				eprintln!("{:?}", PatternRef::new(self.item, item.pattern));
				self.parent.model[decl].set_name(name);
				self.parent.resolutions.insert(
					PatternRef::new(self.item, item.pattern),
					ResolvedIdentifier::Declaration(decl),
				);
			}
			decl_map.insert(idx + 1, decl);
			decls.push(decl);
		}
		decls
	}

	/// Lower an HIR pattern into a THIR pattern
	fn collect_pattern(&mut self, pattern: ArenaIndex<hir::Pattern>) -> Pattern {
		let db = self.parent.db;
		let origin = EntityRef::new(db.upcast(), self.item, pattern);
		let ty = match &self.types[pattern] {
			PatternTy::Destructuring(ty) => *ty,
			PatternTy::Variable(ty) => return Pattern::Anonymous(*ty),
			_ => unreachable!(),
		};
		match &self.data[pattern] {
			hir::Pattern::Absent => Pattern::Expression(
				self.builder()
					.absent()
					.finish(self.expressions_mut(), origin),
			),
			hir::Pattern::Anonymous => Pattern::Anonymous(ty),
			hir::Pattern::Boolean(b) => Pattern::Expression(
				self.builder()
					.boolean(*b)
					.finish(self.expressions_mut(), origin),
			),
			hir::Pattern::Call {
				function,
				arguments,
			} => {
				let args = arguments.iter().map(|a| self.collect_pattern(*a)).collect();
				let pat = self.types.pattern_resolution(*function).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					ResolvedIdentifier::Annotation(ann) => {
						Pattern::AnnotationConstructor { item: *ann, args }
					}
					ResolvedIdentifier::EnumerationMember(member) => Pattern::EnumConstructor {
						member: *member,
						kind: EnumConstructorKind::from_ty(self.parent.db, ty),
						args,
					},
					_ => unreachable!(),
				}
			}
			hir::Pattern::Float { negated, value } => {
				let v = self
					.builder()
					.float(*value)
					.finish(self.expressions_mut(), origin);
				Pattern::Expression(if *negated {
					self.builder()
						.lookup_call(self.parent.ids.minus, [v], self.expressions())
						.finish(self.expressions_mut(), origin)
				} else {
					v
				})
			}
			hir::Pattern::Identifier(_) => {
				let pat = self.types.pattern_resolution(pattern).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					ResolvedIdentifier::Annotation(a) => Pattern::Expression(
						self.builder()
							.annotation_constructor(*a)
							.finish(self.expressions_mut(), origin),
					),
					ResolvedIdentifier::EnumerationMember(m) => Pattern::Expression(
						self.builder()
							.enum_atom(*m)
							.finish(self.expressions_mut(), origin),
					),
					_ => unreachable!(),
				}
			}
			hir::Pattern::Infinity { negated } => {
				let v = self
					.builder()
					.infinity()
					.finish(self.expressions_mut(), origin);
				Pattern::Expression(if *negated {
					self.builder()
						.lookup_call(self.parent.ids.minus, [v], self.expressions())
						.finish(self.expressions_mut(), origin)
				} else {
					v
				})
			}
			hir::Pattern::Integer { negated, value } => {
				let v = self
					.builder()
					.integer(*value)
					.finish(self.expressions_mut(), origin);
				Pattern::Expression(if *negated {
					self.builder()
						.lookup_call(self.parent.ids.minus, [v], self.expressions())
						.finish(self.expressions_mut(), origin)
				} else {
					v
				})
			}
			hir::Pattern::Missing => unreachable!(),
			hir::Pattern::Record { fields } => Pattern::Record(
				fields
					.iter()
					.map(|(i, p)| (*i, self.collect_pattern(*p)))
					.collect(),
			),
			hir::Pattern::String(s) => Pattern::Expression(
				self.builder()
					.string(s.clone())
					.finish(self.expressions_mut(), origin),
			),
			hir::Pattern::Tuple { fields } => {
				Pattern::Tuple(fields.iter().map(|f| self.collect_pattern(*f)).collect())
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DestructuringEntry {
	parent: usize, // 0 means no parent, otherwise = index of parent + 1
	kind: Destructuring,
	pattern: ArenaIndex<hir::Pattern>,
	name: Option<Identifier>,
	create: bool,
}

impl DestructuringEntry {
	fn new(parent: usize, kind: Destructuring, pattern: ArenaIndex<hir::Pattern>) -> Self {
		Self {
			parent,
			kind,
			pattern,
			name: None,
			create: false,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Destructuring {
	TupleAccess(IntegerLiteral),
	RecordAccess(Identifier),
	Enumeration(EnumMemberId, EnumConstructorKind),
	Annotation(AnnotationId),
}

/// Lower a model to THIR
pub fn lower_model(db: &dyn Thir) -> Arc<Model> {
	let ids = db.identifier_registry();
	let mut collector = ItemCollector::new(db, &ids);
	let items = db.lookup_topological_sorted_items();
	for item in items.iter() {
		collector.collect_item(*item);
	}
	collector.collect_deferred();
	Arc::new(collector.finish())
}
