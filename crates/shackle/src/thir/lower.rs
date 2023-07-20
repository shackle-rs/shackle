//! Functionality for converting HIR nodes into THIR nodes.
//!
//! The following is performed during lowering:
//! - Assignment items are moved into declarations/constraints
//! - Destructuring declarations are rewritten as separate declarations
//! - Destructuring in generators is rewritten into a where clause
//! - Type alias items removed as they have been resolved
//! - 2D array literals are re-written using `array2d` calls
//! - Indexed array literals are re-written using `arrayNd` calls
//! - Array slicing is re-written using calls to `slice_Xd`
//!

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	arena::ArenaIndex,
	constants::IdentifierRegistry,
	hir::{
		self,
		ids::{EntityRef, ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		PatternTy, TypeResult,
	},
	ty::{OptType, Ty, TyData, VarType},
	utils::impl_enum_from,
};

use super::{
	db::Thir,
	source::{DesugarKind, Origin},
	*,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DeclOrConstraint {
	Declaration(DeclarationId),
	Constraint(ConstraintId),
}

impl_enum_from!(DeclOrConstraint::Declaration(DeclarationId));
impl_enum_from!(DeclOrConstraint::Constraint(ConstraintId));

impl From<DeclOrConstraint> for LetItem {
	fn from(d: DeclOrConstraint) -> Self {
		match d {
			DeclOrConstraint::Constraint(c) => LetItem::Constraint(c),
			DeclOrConstraint::Declaration(d) => LetItem::Declaration(d),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredAnnotation {
	Items(Vec<DeclOrConstraint>),
	Expression(Expression),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredIdentifier {
	ResolvedIdentifier(ResolvedIdentifier),
	Callable(Callable),
}

/// Collects HIR items and lowers them to THIR
struct ItemCollector<'a> {
	db: &'a dyn Thir,
	ids: &'a IdentifierRegistry,
	resolutions: FxHashMap<PatternRef, LoweredIdentifier>,
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
				let annotation = Annotation::new(
					a.data[*pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				let idx = self.model.add_annotation(Item::new(annotation, item));
				self.resolutions.insert(
					PatternRef::new(item, *pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
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
					let mut collector = ExpressionCollector::new(self, &a.data, item, &types);
					let domain = collector.collect_domain(param.declared_type, *ty, false);
					let mut param_decl = Declaration::new(false, domain);
					// Ignore destructuring and recording resolution for now since these can't have bodies which refer
					// to parameters anyway
					if let Some(p) = param.pattern {
						if let Some(i) = a.data[p].identifier() {
							param_decl.set_name(i);
						}
					}
					let idx = self.model.add_declaration(Item::new(param_decl, item));
					parameters.push(idx);
				}
				let mut annotation = Annotation::new(
					a.data[*constructor]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				annotation.parameters = Some(parameters);
				let idx = self.model.add_annotation(Item::new(annotation, item));
				self.resolutions.insert(
					PatternRef::new(item, *constructor),
					LoweredIdentifier::Callable(Callable::Annotation(idx)),
				);
				self.resolutions.insert(
					PatternRef::new(item, *destructor),
					LoweredIdentifier::Callable(Callable::AnnotationDestructure(idx)),
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
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Declaration(d)) => *d,
			_ => unreachable!(),
		};
		if self.model[decl].definition().is_some() {
			// Turn subsequent assignment items into equality constraints
			let mut collector = ExpressionCollector::new(self, &a.data, item, &types);
			let call = LookupCall {
				function: collector.parent.ids.eq.into(),
				arguments: vec![
					collector.collect_expression(a.assignee),
					collector.collect_expression(a.definition),
				],
			};
			let constraint = Constraint::new(
				true,
				Expression::new(db, &collector.parent.model, item, call),
			);
			self.model.add_constraint(Item::new(constraint, item));
		} else {
			let mut declaration = self.model[decl].clone();
			let mut collector = ExpressionCollector::new(self, &a.data, item, &types);
			let def = collector.collect_expression(a.definition);
			declaration.set_definition(def);
			self.model[decl] = declaration;
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
		let mut collector = ExpressionCollector::new(self, data, item, &types);
		let mut constraint = Constraint::new(top_level, collector.collect_expression(c.expression));
		constraint.annotations_mut().extend(
			c.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model.add_constraint(Item::new(constraint, item))
	}

	/// Collect a declaration item
	pub fn collect_declaration(
		&mut self,
		item: ItemRef,
		d: &hir::Declaration,
		data: &hir::ItemData,
		top_level: bool,
	) -> Vec<DeclOrConstraint> {
		let types = self.db.lookup_item_types(item);

		let ty = match &types[d.pattern] {
			PatternTy::Variable(ty) => *ty,
			PatternTy::Destructuring(ty) => *ty,
			_ => unreachable!(),
		};
		let mut collector = ExpressionCollector::new(self, data, item, &types);
		let domain = collector.collect_domain(d.declared_type, ty, false);
		let mut decl = Declaration::new(top_level, domain);
		if let Some(def) = d.definition {
			decl.set_definition(collector.collect_expression(def));
		}
		let idx = collector
			.parent
			.model
			.add_declaration(Item::new(decl, item));
		let decls = collector.collect_destructuring(idx, top_level, d.pattern);
		let mut ids = vec![idx.into()];
		collector.parent.model[idx]
			.annotations_mut()
			.reserve(d.annotations.len());
		for ann in d.annotations.iter().copied() {
			match collector.collect_declaration_annotation(idx, ann) {
				LoweredAnnotation::Expression(e) => {
					collector.parent.model[idx].annotations_mut().push(e)
				}
				LoweredAnnotation::Items(items) => ids.extend(items),
			}
		}
		ids.extend(decls.into_iter().map(DeclOrConstraint::Declaration));
		ids
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
			PatternTy::Enum(ty) => match ty.lookup(self.db.upcast()) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(self.db.upcast()) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = Enumeration::new(t);
							{
								let mut collector =
									ExpressionCollector::new(self, &e.data, item, &types);
								enumeration.annotations_mut().extend(
									e.annotations
										.iter()
										.map(|ann| collector.collect_expression(*ann)),
								);
							}
							if let Some(def) = &e.definition {
								enumeration.set_definition(
									def.iter()
										.map(|c| self.collect_enum_case(c, &e.data, item, &types)),
								)
							}
							let idx = self.model.add_enumeration(Item::new(enumeration, item));
							self.resolutions.insert(
								PatternRef::new(item, e.pattern),
								LoweredIdentifier::ResolvedIdentifier(idx.into()),
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
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Enumeration(e)) => *e,
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
						LoweredIdentifier::ResolvedIdentifier(
							EnumMemberId::new(idx, i as u32).into(),
						),
					);
				}
				hir::EnumConstructor::Named(hir::Constructor::Function {
					constructor,
					destructor,
					..
				}) => {
					self.resolutions.insert(
						PatternRef::new(item, *constructor),
						LoweredIdentifier::Callable(Callable::EnumConstructor(EnumMemberId::new(
							idx, i as u32,
						))),
					);
					self.resolutions.insert(
						PatternRef::new(item, *destructor),
						LoweredIdentifier::Callable(Callable::EnumDestructor(EnumMemberId::new(
							idx, i as u32,
						))),
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
						let mut collector = ExpressionCollector::new(self, data, item, types);
						let domain = collector.collect_domain(*t, *ty, false);
						let declaration = Declaration::new(false, domain);
						self.model.add_declaration(Item::new(declaration, item))
					})
					.collect(),
			),
		}
	}

	/// Collect a function item
	pub fn collect_function(&mut self, item: ItemRef, f: &hir::Item<hir::Function>) -> FunctionId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &f.data, item, &types);
		let res = PatternRef::new(item, f.pattern);
		match &types[f.pattern] {
			PatternTy::Function(fn_entry) => {
				let domain =
					collector.collect_domain(f.return_type, fn_entry.overload.return_type(), false);
				let mut function =
					Function::new(f.data[f.pattern].identifier().unwrap().into(), domain);
				function.annotations_mut().extend(
					f.annotations
						.iter()
						.map(|ann| collector.collect_expression(*ann)),
				);
				function.set_type_inst_vars(f.type_inst_vars.iter().map(|t| {
					match &types[t.name] {
						PatternTy::TyVar(tv) => tv.clone(),
						_ => unreachable!(),
					}
				}));

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
				function.set_parameters(parameters);

				let idx = self.model.add_function(Item::new(function, item));
				self.resolutions
					.insert(res, LoweredIdentifier::Callable(Callable::Function(idx)));
				if f.body.is_some() {
					self.deferred.push((idx, item));
				}
				idx
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
		let mut collector = ExpressionCollector::new(self, data, item, types);
		let domain = collector.collect_domain(param.declared_type, ty, false);
		let mut declaration = Declaration::new(false, domain);
		if let Some(p) = param.pattern.and_then(|p| data[p].identifier()) {
			declaration.set_name(p);
		}
		declaration.annotations_mut().extend(
			param
				.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model.add_declaration(Item::new(declaration, item))
	}

	/// Collect an output item
	pub fn collect_output(&mut self, item: ItemRef, o: &hir::Item<hir::Output>) -> OutputId {
		let types = self.db.lookup_item_types(item);
		let mut collector = ExpressionCollector::new(self, &o.data, item, &types);
		let mut output = Output::new(collector.collect_expression(o.expression));
		if let Some(s) = o.section {
			output.set_section(collector.collect_expression(s));
		}
		self.model.add_output(Item::new(output, item))
	}

	/// Collect solve item
	pub fn collect_solve(&mut self, item: ItemRef, s: &hir::Item<hir::Solve>) {
		let types = self.db.lookup_item_types(item);
		let mut optimise = |pattern: ArenaIndex<hir::Pattern>,
		                    objective: ArenaIndex<hir::Expression>,
		                    is_maximize: bool| match &types[pattern] {
			PatternTy::Variable(ty) => {
				let objective_origin = EntityRef::new(self.db.upcast(), item, objective);
				let mut collector = ExpressionCollector::new(self, &s.data, item, &types);
				let mut declaration = Declaration::new(
					true,
					Domain::unbounded(collector.parent.db, objective_origin, *ty),
				);
				if let Some(name) = s.data[pattern].identifier() {
					declaration.set_name(name);
				}
				let obj = collector.collect_expression(objective);
				declaration.set_definition(obj);
				let idx = self.model.add_declaration(Item::new(
					declaration,
					Origin::from(item).with_desugaring(DesugarKind::Objective),
				));
				self.resolutions.insert(
					PatternRef::new(item, pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
				);
				if is_maximize {
					Solve::maximize(idx)
				} else {
					Solve::minimize(idx)
				}
			}
			_ => unreachable!(),
		};
		let mut si = match &s.goal {
			hir::Goal::Maximize { pattern, objective } => optimise(*pattern, *objective, true),
			hir::Goal::Minimize { pattern, objective } => optimise(*pattern, *objective, false),
			hir::Goal::Satisfy => Solve::satisfy(),
		};
		let mut collector = ExpressionCollector::new(self, &s.data, item, &types);
		si.annotations_mut().extend(
			s.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model.set_solve(Item::new(si, item));
	}

	fn collect_type_alias(&mut self, item: ItemRef, ta: &hir::Item<hir::TypeAlias>) {
		let types = self.db.lookup_item_types(item);
		for e in hir::Type::expressions(ta.aliased_type, &ta.data) {
			if let Some(res) = types.name_resolution(e) {
				let res_types = self.db.lookup_item_types(res.item());
				if matches!(&res_types[res.pattern()], PatternTy::TypeAlias { .. }) {
					// Skip type aliases inside other type aliases (already will be processed)
					continue;
				}
			}
			// Create a declaration with the value of each expression used in a type alias
			let expression =
				ExpressionCollector::new(self, &ta.data, item, &types).collect_expression(e);
			let mut decl = Declaration::new(
				true,
				Domain::unbounded(self.db, expression.origin(), expression.ty()),
			);
			decl.set_definition(expression);
			let idx = self
				.model
				.add_declaration(Item::new(decl, EntityRef::new(self.db.upcast(), item, e)));
			self.type_alias_expressions
				.insert(ExpressionRef::new(item, e), idx);
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
					let mut function = self.model[func].clone();
					let param_decls = function.parameters().to_owned();
					let mut decls = Vec::new();
					let mut collector =
						ExpressionCollector::new(self, &model[f].data, item, &types);
					for (decl, param) in param_decls.into_iter().zip(model[f].parameters.iter()) {
						if let Some(p) = param.pattern {
							let dsts = collector.collect_destructuring(decl, false, p);
							decls.extend(dsts);
						}
					}
					let body = model[f].body.unwrap();
					let collected_body = collector.collect_expression(body);
					let e = if decls.is_empty() {
						collected_body
					} else {
						let origin = EntityRef::new(collector.parent.db.upcast(), item, body);
						Expression::new(
							self.db,
							&self.model,
							origin,
							Let {
								items: decls.into_iter().map(LetItem::Declaration).collect(),
								in_expression: Box::new(collected_body),
							},
						)
					};
					function.set_body(e);
					self.model[func] = function;
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
	types: &'a TypeResult,
}

impl<'a, 'b> ExpressionCollector<'a, 'b> {
	fn new(
		parent: &'a mut ItemCollector<'b>,
		data: &'a crate::hir::ItemData,
		item: ItemRef,
		types: &'a TypeResult,
	) -> Self {
		Self {
			parent,
			data,
			item,
			types,
		}
	}

	fn introduce_declaration(
		&mut self,
		top_level: bool,
		origin: impl Into<Origin>,
		f: impl FnOnce(&mut ExpressionCollector<'_, '_>) -> Expression,
	) -> DeclarationId {
		let origin: Origin = origin.into();
		let mut collector = ExpressionCollector::new(self.parent, self.data, self.item, self.types);
		let def = f(&mut collector);
		let mut decl = Declaration::new(
			top_level,
			Domain::unbounded(self.parent.db, origin, def.ty()),
		);
		decl.set_definition(def);
		self.parent.model.add_declaration(Item::new(decl, origin))
	}

	/// Collect an expression
	pub fn collect_expression(&mut self, idx: ArenaIndex<hir::Expression>) -> Expression {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = EntityRef::new(db.upcast(), self.item, idx);
		let mut result = match &self.data[idx] {
			hir::Expression::Absent => alloc_expression(Absent, self, origin),
			hir::Expression::ArrayAccess(aa) => {
				let is_slice = match self.types[aa.indices].lookup(db.upcast()) {
					TyData::Tuple(_, fs) => fs.iter().any(|f| f.is_set(db.upcast())),
					TyData::Set(_, _, _) => true,
					_ => false,
				};
				if is_slice {
					self.collect_slice(aa.collection, aa.indices, origin)
				} else {
					alloc_expression(
						ArrayAccess {
							collection: Box::new(self.collect_expression(aa.collection)),
							indices: Box::new(self.collect_expression(aa.indices)),
						},
						self,
						origin,
					)
				}
			}
			hir::Expression::ArrayComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				alloc_expression(
					ArrayComprehension {
						generators,
						template: Box::new(self.collect_expression(c.template)),
						indices: c
							.indices
							.map(|indices| Box::new(self.collect_expression(indices))),
					},
					self,
					origin,
				)
			}
			hir::Expression::ArrayLiteral(al) => alloc_expression(
				ArrayLiteral(
					al.members
						.iter()
						.map(|m| self.collect_expression(*m))
						.collect(),
				),
				self,
				origin,
			),
			// Desugar 2D array literal into array2d call
			hir::Expression::ArrayLiteral2D(al) => {
				let mut idx_array = |dim: &hir::MaybeIndexSet| match dim {
					hir::MaybeIndexSet::Indexed(es) => alloc_expression(
						ArrayLiteral(es.iter().map(|e| self.collect_expression(*e)).collect()),
						self,
						origin,
					),
					hir::MaybeIndexSet::NonIndexed(c) => alloc_expression(
						LookupCall {
							function: self.parent.ids.set2array.into(),
							arguments: vec![if *c > 0 {
								alloc_expression(
									LookupCall {
										function: self.parent.ids.dot_dot.into(),
										arguments: vec![
											alloc_expression(IntegerLiteral(1), self, origin),
											alloc_expression(
												IntegerLiteral(*c as i64),
												self,
												origin,
											),
										],
									},
									self,
									origin,
								)
							} else {
								alloc_expression(SetLiteral(Vec::new()), self, origin)
							}],
						},
						self,
						origin,
					),
				};
				let rows = idx_array(&al.rows);
				let columns = idx_array(&al.columns);
				alloc_expression(
					LookupCall {
						function: self.parent.ids.array2d.into(),
						arguments: vec![
							rows,
							columns,
							alloc_expression(
								ArrayLiteral(
									al.members
										.iter()
										.map(|e| self.collect_expression(*e))
										.collect(),
								),
								self,
								origin,
							),
						],
					},
					self,
					origin,
				)
			}
			// Desugar indexed array literal into arrayNd call
			hir::Expression::IndexedArrayLiteral(al) => alloc_expression(
				LookupCall {
					function: self.parent.ids.array_nd.into(),
					arguments: vec![
						if al.indices.len() == 1 {
							self.collect_expression(al.indices[0])
						} else {
							alloc_expression(
								ArrayLiteral(
									al.indices
										.iter()
										.map(|e| self.collect_expression(*e))
										.collect(),
								),
								self,
								origin,
							)
						},
						alloc_expression(
							ArrayLiteral(
								al.members
									.iter()
									.map(|e| self.collect_expression(*e))
									.collect(),
							),
							self,
							origin,
						),
					],
				},
				self,
				origin,
			),
			hir::Expression::BooleanLiteral(b) => alloc_expression(*b, self, origin),
			hir::Expression::Call(c) => {
				let function = if let hir::Expression::Identifier(_) = self.data[c.function] {
					let res = self.types.name_resolution(c.function).unwrap();
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							NodeRef::from(res.into_entity(self.parent.db.upcast()))
								.source_span(self.parent.db.upcast()),
							ExpressionRef::new(self.item, c.function),
							NodeRef::from(EntityRef::new(
								self.parent.db.upcast(),
								self.item,
								c.function
							))
							.source_span(self.parent.db.upcast()),
						)
					});
					match ident {
						LoweredIdentifier::Callable(c) => c.clone(),
						_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
					}
				} else {
					Callable::Expression(Box::new(self.collect_expression(c.function)))
				};
				alloc_expression(
					Call {
						function,
						arguments: c
							.arguments
							.iter()
							.map(|arg| self.collect_expression(*arg))
							.collect(),
					},
					self,
					origin,
				)
			}
			hir::Expression::Case(c) => {
				let scrutinee_origin =
					EntityRef::new(self.parent.db.upcast(), self.item, c.expression);
				let scrutinee = self.introduce_declaration(false, scrutinee_origin, |collector| {
					collector.collect_expression(c.expression)
				});
				alloc_expression(
					Let {
						items: vec![LetItem::Declaration(scrutinee)],
						in_expression: Box::new(alloc_expression(
							Case {
								scrutinee: Box::new(alloc_expression(scrutinee, self, origin)),
								branches: c
									.cases
									.iter()
									.map(|case| {
										let pattern_origin = EntityRef::new(
											self.parent.db.upcast(),
											self.item,
											case.pattern,
										);
										let pattern = self.collect_pattern(case.pattern);
										let decls = self.collect_destructuring(
											scrutinee,
											false,
											case.pattern,
										);
										let result = self.collect_expression(case.value);
										if decls.is_empty() {
											CaseBranch::new(pattern, result)
										} else {
											CaseBranch::new(
												pattern,
												alloc_expression(
													Let {
														items: decls
															.into_iter()
															.map(LetItem::Declaration)
															.collect(),
														in_expression: Box::new(result),
													},
													self,
													pattern_origin,
												),
											)
										}
									})
									.collect(),
							},
							self,
							origin,
						)),
					},
					self,
					origin,
				)
			}
			hir::Expression::FloatLiteral(f) => alloc_expression(*f, self, origin),
			hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(idx).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						NodeRef::from(res.into_entity(self.parent.db.upcast()))
							.source_span(self.parent.db.upcast()),
						ExpressionRef::new(self.item, idx),
						NodeRef::from(EntityRef::new(self.parent.db.upcast(), self.item, idx))
							.source_span(self.parent.db.upcast()),
					)
				});
				let expr = alloc_expression(
					match ident {
						LoweredIdentifier::ResolvedIdentifier(i) => i.clone(),
						_ => unreachable!(),
					},
					self,
					origin,
				);

				if expr.ty() == ty {
					expr
				} else {
					// Need to insert call to fix()
					assert_eq!(expr.ty().make_par(db.upcast()), ty);
					alloc_expression(
						LookupCall {
							function: self.parent.ids.fix.into(),
							arguments: vec![expr],
						},
						self,
						origin,
					)
				}
			}
			hir::Expression::IfThenElse(ite) => alloc_expression(
				IfThenElse {
					branches: ite
						.branches
						.iter()
						.map(|b| {
							Branch::new(
								self.collect_expression(b.condition),
								self.collect_expression(b.result),
							)
						})
						.collect(),
					else_result: Box::new(
						ite.else_result
							.map(|e| self.collect_expression(e))
							.unwrap_or_else(|| self.collect_default_else(ty, origin.into())),
					),
				},
				self,
				origin,
			),
			hir::Expression::Infinity => alloc_expression(Infinity, self, origin),
			hir::Expression::IntegerLiteral(i) => alloc_expression(*i, self, origin),
			hir::Expression::Lambda(l) => {
				let fn_type = match ty.lookup(db.upcast()) {
					TyData::Function(_, f) => f,
					_ => unreachable!(),
				};
				let return_type = l
					.return_type
					.map(|r| self.collect_domain(r, fn_type.return_type, false))
					.unwrap_or_else(|| {
						Domain::unbounded(self.parent.db, origin, fn_type.return_type)
					});
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
				let function = Function::lambda(
					return_type,
					parameters,
					if decls.is_empty() {
						body
					} else {
						let body_entity =
							EntityRef::new(self.parent.db.upcast(), self.item, l.body);
						alloc_expression(
							Let {
								items: decls.into_iter().map(LetItem::Declaration).collect(),
								in_expression: Box::new(body),
							},
							self,
							body_entity,
						)
					},
				);
				let f = self.parent.model.add_function(Item::new(function, origin));
				alloc_expression(Lambda(f), self, origin)
			}
			hir::Expression::Let(l) => alloc_expression(
				Let {
					items: l
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
								.map(|d| d.into())
								.collect::<Vec<_>>(),
						})
						.collect(),
					in_expression: Box::new(self.collect_expression(l.in_expression)),
				},
				self,
				origin,
			),
			hir::Expression::RecordAccess(ra) => alloc_expression(
				RecordAccess {
					record: Box::new(self.collect_expression(ra.record)),
					field: ra.field,
				},
				self,
				origin,
			),
			hir::Expression::RecordLiteral(rl) => alloc_expression(
				RecordLiteral(
					rl.fields
						.iter()
						.map(|(i, v)| {
							(
								self.data[*i].identifier().unwrap(),
								self.collect_expression(*v),
							)
						})
						.collect(),
				),
				self,
				origin,
			),
			hir::Expression::SetComprehension(c) => {
				let mut generators = Vec::with_capacity(c.generators.len());
				for g in c.generators.iter() {
					self.collect_generator(g, &mut generators);
				}
				alloc_expression(
					SetComprehension {
						generators,
						template: Box::new(self.collect_expression(c.template)),
					},
					self,
					origin,
				)
			}
			hir::Expression::SetLiteral(sl) => alloc_expression(
				SetLiteral(
					sl.members
						.iter()
						.map(|m| self.collect_expression(*m))
						.collect(),
				),
				self,
				origin,
			),
			hir::Expression::Slice(_) => {
				unreachable!("Slice used outside of array access")
			}
			hir::Expression::StringLiteral(sl) => alloc_expression(sl.clone(), self, origin),
			hir::Expression::TupleAccess(ta) => alloc_expression(
				TupleAccess {
					tuple: Box::new(self.collect_expression(ta.tuple)),
					field: ta.field,
				},
				self,
				origin,
			),
			hir::Expression::TupleLiteral(tl) => alloc_expression(
				TupleLiteral(
					tl.fields
						.iter()
						.map(|f| self.collect_expression(*f))
						.collect(),
				),
				self,
				origin,
			),
			hir::Expression::Missing => unreachable!("Missing expression"),
		};
		result.annotations_mut().extend(
			self.data
				.annotations(idx)
				.map(|ann| self.collect_expression(ann)),
		);
		assert_eq!(
			result.ty(),
			ty,
			"Type by construction ({}) disagrees with typechecker ({}) at {:?}",
			result.ty().pretty_print(db.upcast()),
			ty.pretty_print(db.upcast()),
			NodeRef::from(origin).source_span(db.upcast())
		);
		result
	}

	fn collect_declaration_annotation(
		&mut self,
		decl: DeclarationId,
		ann: ArenaIndex<hir::Expression>,
	) -> LoweredAnnotation {
		// Declarations can have annotations which point to functions using ::annotated_expression.
		// These need to be desugared into constraints.
		match &self.data[ann] {
			hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(ann).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						NodeRef::from(res.into_entity(self.parent.db.upcast()))
							.source_span(self.parent.db.upcast()),
						ExpressionRef::new(self.item, ann),
						NodeRef::from(EntityRef::new(self.parent.db.upcast(), self.item, ann))
							.source_span(self.parent.db.upcast()),
					)
				});
				if let LoweredIdentifier::Callable(function) = ident.clone() {
					let origin = EntityRef::new(self.parent.db.upcast(), self.item, ann);
					let ann_decl = self.introduce_declaration(
						self.parent.model[decl].top_level(),
						origin,
						|collector| {
							// Call annotation function using the annotated declaration
							let arguments = vec![alloc_expression(
								ResolvedIdentifier::Declaration(decl),
								collector,
								origin,
							)];
							alloc_expression(
								Call {
									function: function.clone(),
									arguments,
								},
								collector,
								origin,
							)
						},
					);

					let annotate = alloc_expression(
						LookupCall {
							function: self.parent.ids.annotate.into(),
							arguments: vec![
								alloc_expression(
									ResolvedIdentifier::Declaration(decl),
									self,
									origin,
								),
								alloc_expression(
									ResolvedIdentifier::Declaration(ann_decl),
									self,
									origin,
								),
							],
						},
						self,
						origin,
					);
					let constraint = Constraint::new(self.parent.model[decl].top_level(), annotate);
					let c_idx = self
						.parent
						.model
						.add_constraint(Item::new(constraint, origin));

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}
			}
			hir::Expression::Call(c) => {
				let origin = EntityRef::new(self.parent.db.upcast(), self.item, ann);
				let function = if let hir::Expression::Identifier(_) = self.data[c.function] {
					let res = self.types.name_resolution(c.function).unwrap();
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							NodeRef::from(res.into_entity(self.parent.db.upcast()))
								.source_span(self.parent.db.upcast()),
							ExpressionRef::new(self.item, c.function),
							NodeRef::from(EntityRef::new(
								self.parent.db.upcast(),
								self.item,
								c.function
							))
							.source_span(self.parent.db.upcast()),
						)
					});
					match ident {
						LoweredIdentifier::Callable(c) => c.clone(),
						_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
					}
				} else {
					Callable::Expression(Box::new(self.collect_expression(c.function)))
				};

				if let Callable::Function(f) = &function {
					if self.parent.model[*f].parameters().len() > c.arguments.len() {
						// Add the annotated declaration identifier as first argument
						let mut arguments = Vec::with_capacity(c.arguments.len() + 1);
						arguments.push(alloc_expression(
							ResolvedIdentifier::Declaration(decl),
							self,
							origin,
						));
						arguments
							.extend(c.arguments.iter().map(|arg| self.collect_expression(*arg)));

						let ann_decl = self.introduce_declaration(
							self.parent.model[decl].top_level(),
							origin,
							|collector| {
								alloc_expression(
									Call {
										function,
										arguments,
									},
									collector,
									origin,
								)
							},
						);

						let annotate = alloc_expression(
							LookupCall {
								function: self.parent.ids.annotate.into(),
								arguments: vec![
									alloc_expression(
										ResolvedIdentifier::Declaration(decl),
										self,
										origin,
									),
									alloc_expression(
										ResolvedIdentifier::Declaration(ann_decl),
										self,
										origin,
									),
								],
							},
							self,
							origin,
						);
						let constraint =
							Constraint::new(self.parent.model[decl].top_level(), annotate);
						let c_idx = self
							.parent
							.model
							.add_constraint(Item::new(constraint, origin));

						return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
					}
				}

				// Return as is
				return LoweredAnnotation::Expression(alloc_expression(
					Call {
						function,
						arguments: c
							.arguments
							.iter()
							.map(|arg| self.collect_expression(*arg))
							.collect(),
					},
					self,
					origin,
				));
			}
			_ => (),
		}
		LoweredAnnotation::Expression(self.collect_expression(ann))
	}

	/// Rewrite index slicing into slice_Xd() call
	fn collect_slice(
		&mut self,
		collection: ArenaIndex<hir::Expression>,
		indices: ArenaIndex<hir::Expression>,
		origin: impl Into<Origin>,
	) -> Expression {
		let origin: Origin = origin.into();
		let collection_entity = EntityRef::new(self.parent.db.upcast(), self.item, collection);
		let indices_entity = EntityRef::new(self.parent.db.upcast(), self.item, indices);

		let mut decls = Vec::new();
		let collection_decl = if matches!(&self.data[collection], hir::Expression::Identifier(_)) {
			let expr = self.collect_expression(collection);
			match &*expr {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(decl)) => *decl,
				_ => unreachable!(),
			}
		} else {
			// Add declaration to store collection
			let origin = EntityRef::new(self.parent.db.upcast(), self.item, collection);
			let decl = self.introduce_declaration(false, origin, |collector| {
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
						let mut is_set = true;
						let decl = self.introduce_declaration(false, index_entity, |collector| {
							if let hir::Expression::Slice(s) = &collector.data[*e] {
								// Rewrite infinite slice .. into `'..'(index_set_mofn(c))`
								alloc_expression(
									LookupCall {
										function: (*s).into(),
										arguments: vec![alloc_expression(
											LookupCall {
												function: Identifier::new(
													format!("index_set_{}of{}", i + 1, array_dims),
													collector.parent.db.upcast(),
												)
												.into(),
												arguments: vec![alloc_expression(
													collection_decl,
													collector,
													collection_entity,
												)],
											},
											collector,
											index_entity,
										)],
									},
									collector,
									index_entity,
								)
							} else if ty.is_set(collector.parent.db.upcast()) {
								// Slice
								collector.collect_expression(*e)
							} else {
								// Rewrite index as slice of {i}
								is_set = false;
								alloc_expression(
									SetLiteral(vec![collector.collect_expression(*e)]),
									collector,
									index_entity,
								)
							}
						});
						slices.push((decl, is_set, index_entity));
						decls.push(decl);
					}
				} else {
					// Expression which evaluates to a tuple
					let indices_decl =
						self.introduce_declaration(false, indices_entity, |collector| {
							collector.collect_expression(indices)
						});
					decls.push(indices_decl);
					for (i, f) in fs.iter().enumerate() {
						// Create declaration for each index
						let is_set = f.is_set(self.parent.db.upcast());
						let accessor =
							self.introduce_declaration(false, indices_entity, |collector| {
								let ta = alloc_expression(
									TupleAccess {
										tuple: Box::new(alloc_expression(
											indices_decl,
											collector,
											indices_entity,
										)),
										field: IntegerLiteral(i as i64 + 1),
									},
									collector,
									indices_entity,
								);
								if is_set {
									ta
								} else {
									// Rewrite as {i}
									alloc_expression(
										SetLiteral(vec![ta]),
										collector,
										indices_entity,
									)
								}
							});

						slices.push((accessor, is_set, indices_entity));
						decls.push(accessor);
					}
				}
			}
			_ => {
				// 1D slicing, so must be a set index
				let decl = self.introduce_declaration(false, indices_entity, |collector| {
					if let hir::Expression::Slice(s) = &collector.data[indices] {
						// Rewrite infinite slice .. into `'..'(index_set(c))`
						alloc_expression(
							LookupCall {
								function: (*s).into(),
								arguments: vec![alloc_expression(
									LookupCall {
										function: collector.parent.ids.index_set.into(),
										arguments: vec![alloc_expression(
											collection_decl,
											collector,
											collection_entity,
										)],
									},
									collector,
									indices_entity,
								)],
							},
							collector,
							indices_entity,
						)
					} else {
						collector.collect_expression(indices)
					}
				});
				slices.push((decl, true, indices_entity));
				decls.push(decl);
			}
		}
		let collection_ident = alloc_expression(collection_decl, self, collection_entity);
		let slice_array = alloc_expression(
			ArrayLiteral(
				slices
					.iter()
					.map(|(decl, _, origin)| {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.erase_enum.into(),
								arguments: vec![alloc_expression(*decl, self, *origin)],
							},
							self,
							*origin,
						)
					})
					.collect(),
			),
			self,
			indices_entity,
		);
		let mut arguments = vec![collection_ident, slice_array];
		arguments.extend(slices.iter().filter_map(|(decl, is_slice, origin)| {
			if *is_slice {
				Some(alloc_expression(*decl, self, *origin))
			} else {
				None
			}
		}));
		alloc_expression(
			Let {
				items: decls.into_iter().map(LetItem::Declaration).collect(),
				in_expression: Box::new(alloc_expression(
					LookupCall {
						function: Identifier::new(
							format!("slice_{}d", arguments.len() - 2),
							self.parent.db.upcast(),
						)
						.into(),
						arguments,
					},
					self,
					origin,
				)),
			},
			self,
			origin,
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
						let declaration =
							Declaration::new(false, Domain::unbounded(self.parent.db, origin, ty));
						let decl = self
							.parent
							.model
							.add_declaration(Item::new(declaration, origin));
						let asgs = self.collect_destructuring(decl, false, *p);
						if !asgs.is_empty() {
							// Turn destructuring into where clause of case matching pattern
							let pattern = self.collect_pattern(*p);
							let case = alloc_expression(
								Case {
									scrutinee: Box::new(alloc_expression(decl, self, origin)),
									branches: vec![
										CaseBranch::new(
											pattern,
											alloc_expression(BooleanLiteral(true), self, origin),
										),
										CaseBranch::new(
											Pattern::new(
												PatternData::Anonymous(match &self.types[*p] {
													PatternTy::Destructuring(ty) => *ty,
													_ => unreachable!(),
												}),
												origin,
											),
											alloc_expression(BooleanLiteral(false), self, origin),
										),
									],
								},
								self,
								origin,
							);
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
							where_clause: Some(where_clauses.pop().unwrap()),
						});
					} else {
						let call = alloc_expression(
							LookupCall {
								function: self.parent.ids.forall.into(),
								arguments: vec![alloc_expression(
									ArrayLiteral(where_clauses),
									self,
									origin,
								)],
							},
							self,
							origin,
						);
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(call),
						});
					}
					let mut iter = assignments.into_iter();
					let mut assignment = iter.next().unwrap();
					for next in iter {
						generators.push(Generator::Assignment {
							assignment,
							where_clause: None,
						});
						assignment = next;
					}
					generators.push(Generator::Assignment {
						assignment,
						where_clause,
					});
				}
			}
			hir::Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let def = ExpressionCollector::new(self.parent, self.data, self.item, self.types)
					.collect_expression(*value);
				let mut assignment = Declaration::new(
					false,
					Domain::unbounded(self.parent.db, def.origin(), def.ty()),
				);
				assignment.set_definition(def);
				if let Some(name) = self.data[*pattern].identifier() {
					assignment.set_name(name);
				}
				let idx = self.parent.model.add_declaration(Item::new(
					assignment,
					EntityRef::new(self.parent.db.upcast(), self.item, *pattern),
				));
				self.parent.resolutions.insert(
					PatternRef::new(self.item, *pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
				);
				generators.push(Generator::Assignment {
					assignment: idx,
					where_clause: where_clause.map(|w| self.collect_expression(w)),
				});
			}
		}
	}

	fn collect_default_else(&mut self, ty: Ty, origin: Origin) -> Expression {
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
			| TyData::TyVar(_, Some(OptType::Opt), _) => alloc_expression(Absent, self, origin),
			TyData::Boolean(_, _) => alloc_expression(BooleanLiteral(true), self, origin),
			TyData::String(_) => alloc_expression(
				StringLiteral::new("", self.parent.db.upcast()),
				self,
				origin,
			),
			TyData::Annotation(_) => {
				alloc_expression(self.parent.ids.empty_annotation, self, origin)
			}
			TyData::Array { .. } => alloc_expression(ArrayLiteral::default(), self, origin),
			TyData::Set(_, _, _) => alloc_expression(SetLiteral::default(), self, origin),
			TyData::Tuple(_, fs) => alloc_expression(
				TupleLiteral(
					fs.iter()
						.map(|f| self.collect_default_else(*f, origin))
						.collect(),
				),
				self,
				origin,
			),
			TyData::Record(_, fs) => alloc_expression(
				RecordLiteral(
					fs.iter()
						.map(|(i, t)| (Identifier(*i), self.collect_default_else(*t, origin)))
						.collect(),
				),
				self,
				origin,
			),
			_ => unreachable!("No default value for this type"),
		}
	}

	// Collect a domain from a user ascribed type
	fn collect_domain(&mut self, t: ArenaIndex<hir::Type>, ty: Ty, is_type_alias: bool) -> Domain {
		let db = self.parent.db;
		let origin = EntityRef::new(db.upcast(), self.item, t);
		match (&self.data[t], ty.lookup(db.upcast())) {
			(hir::Type::Bounded { domain, .. }, _) => {
				if let Some(res) = self.types.name_resolution(*domain) {
					let res_types = db.lookup_item_types(res.item());
					match &res_types[res.pattern()] {
						// Identifier is actually a type, not a domain expression
						PatternTy::TyVar(_) => {
							return Domain::unbounded(self.parent.db, origin, ty);
						}
						PatternTy::TypeAlias { .. } => {
							let model = res.item().model(db.upcast());
							match res.item().local_item_ref(db.upcast()) {
								LocalItemRef::TypeAlias(ta) => {
									let mut c = ExpressionCollector::new(
										self.parent,
										&model[ta].data,
										res.item(),
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
					Domain::bounded(
						db,
						origin,
						ty.inst(db.upcast()).unwrap(),
						ty.opt(db.upcast()).unwrap(),
						alloc_expression(self.parent.type_alias_expressions[&er], self, origin),
					)
				} else {
					let e = self.collect_expression(*domain);
					Domain::bounded(
						db,
						origin,
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
					opt,
					dim: d,
					element: el,
				},
			) => Domain::array(
				db,
				origin,
				opt,
				self.collect_domain(*dimensions, d, is_type_alias),
				self.collect_domain(*element, el, is_type_alias),
			),
			(hir::Type::Set { element, .. }, TyData::Set(inst, opt, e)) => Domain::set(
				db,
				origin,
				inst,
				opt,
				self.collect_domain(*element, e, is_type_alias),
			),
			(hir::Type::Tuple { fields, .. }, TyData::Tuple(opt, fs)) => Domain::tuple(
				db,
				origin,
				opt,
				fields
					.iter()
					.zip(fs.iter())
					.map(|(f, ty)| self.collect_domain(*f, *ty, is_type_alias)),
			),
			(hir::Type::Record { fields, .. }, TyData::Record(opt, fs)) => Domain::record(
				db,
				origin,
				opt,
				fs.iter().map(|(i, ty)| {
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
				}),
			),
			_ => Domain::unbounded(self.parent.db, origin, ty),
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
						LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Annotation(*ann),
								destructuring_pattern,
							));
						}
						LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
							destructuring.push(DestructuringEntry::new(
								i,
								Destructuring::Enumeration(*member),
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
					if matches!(
						&self.types[p],
						PatternTy::Variable(_) | PatternTy::Argument(_)
					) {
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
								LoweredIdentifier::ResolvedIdentifier(root_decl.into()),
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
			let decl = self.introduce_declaration(top_level, origin, |collector| {
				let ident = alloc_expression(
					if item.parent == 0 {
						root_decl
					} else {
						decl_map[&item.parent]
					},
					collector,
					origin,
				);
				match item.kind {
					Destructuring::Annotation(a) => alloc_expression(
						Call {
							function: Callable::AnnotationDestructure(a),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::Enumeration(e) => alloc_expression(
						Call {
							function: Callable::EnumDestructor(e),
							arguments: vec![ident],
						},
						collector,
						origin,
					),
					Destructuring::RecordAccess(f) => alloc_expression(
						RecordAccess {
							record: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
					Destructuring::TupleAccess(f) => alloc_expression(
						TupleAccess {
							tuple: Box::new(ident),
							field: f,
						},
						collector,
						origin,
					),
				}
			});
			if let Some(name) = item.name {
				self.parent.model[decl].set_name(name);
				self.parent.resolutions.insert(
					PatternRef::new(self.item, item.pattern),
					LoweredIdentifier::ResolvedIdentifier(decl.into()),
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
			PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
				return Pattern::new(PatternData::Anonymous(*ty), origin);
			}
			_ => unreachable!(),
		};
		Pattern::new(
			match &self.data[pattern] {
				hir::Pattern::Absent => {
					PatternData::Expression(Box::new(alloc_expression(Absent, self, origin)))
				}
				hir::Pattern::Anonymous => PatternData::Anonymous(ty),
				hir::Pattern::Boolean(b) => {
					PatternData::Expression(Box::new(alloc_expression(*b, self, origin)))
				}
				hir::Pattern::Call {
					function,
					arguments,
				} => {
					let args = arguments.iter().map(|a| self.collect_pattern(*a)).collect();
					let pat = self.types.pattern_resolution(*function).unwrap();
					let res = &self.parent.resolutions[&pat];
					match res {
						LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
							PatternData::AnnotationConstructor { item: *ann, args }
						}
						LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
							PatternData::EnumConstructor {
								member: *member,
								args,
							}
						}
						_ => unreachable!(),
					}
				}
				hir::Pattern::Float { negated, value } => {
					let v = alloc_expression(*value, self, origin);
					PatternData::Expression(Box::new(if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					}))
				}
				hir::Pattern::Identifier(_) => {
					let pat = self.types.pattern_resolution(pattern).unwrap();
					let res = &self.parent.resolutions[&pat];
					match res {
						LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Annotation(
							a,
						)) => PatternData::Expression(Box::new(alloc_expression(*a, self, origin))),
						LoweredIdentifier::ResolvedIdentifier(
							ResolvedIdentifier::EnumerationMember(m),
						) => PatternData::Expression(Box::new(alloc_expression(*m, self, origin))),
						_ => unreachable!(),
					}
				}
				hir::Pattern::Infinity { negated } => {
					let v = alloc_expression(Infinity, self, origin);
					PatternData::Expression(Box::new(if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					}))
				}
				hir::Pattern::Integer { negated, value } => {
					let v = alloc_expression(*value, self, origin);
					PatternData::Expression(Box::new(if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					}))
				}
				hir::Pattern::Missing => unreachable!(),
				hir::Pattern::Record { fields } => PatternData::Record(
					fields
						.iter()
						.map(|(i, p)| (*i, self.collect_pattern(*p)))
						.collect(),
				),
				hir::Pattern::String(s) => {
					PatternData::Expression(Box::new(alloc_expression(s.clone(), self, origin)))
				}
				hir::Pattern::Tuple { fields } => {
					PatternData::Tuple(fields.iter().map(|f| self.collect_pattern(*f)).collect())
				}
			},
			origin,
		)
	}
}

fn alloc_expression(
	data: impl ExpressionBuilder,
	collector: &ExpressionCollector<'_, '_>,
	origin: impl Into<Origin>,
) -> Expression {
	Expression::new(collector.parent.db, &collector.parent.model, origin, data)
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
	Enumeration(EnumMemberId),
	Annotation(AnnotationId),
}

/// Lower a model to THIR
pub fn lower_model(db: &dyn Thir) -> Arc<Model> {
	for e in db.all_errors().iter() {
		eprintln!("{:#?}", e);
	}
	assert!(db.all_errors().is_empty());
	let ids = db.identifier_registry();
	let mut collector = ItemCollector::new(db, &ids);
	let items = db.lookup_topological_sorted_items();
	for item in items.iter() {
		collector.collect_item(*item);
	}
	collector.collect_deferred();
	Arc::new(collector.finish())
}
