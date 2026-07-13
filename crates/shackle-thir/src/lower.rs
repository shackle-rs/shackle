//! Functionality for converting HIR nodes into THIR nodes.
//!
//! The following is performed during lowering:
//! - Assignment items are moved into declarations/constraints
//! - Destructuring declarations are rewritten as separate declarations
//! - Destructuring in generators is rewritten into a where clause
//! - Type alias items removed as they have been resolved
//! - 2D array literals are re-written using `mzn_array_kd` calls
//! - Indexed array literals are re-written using `mzn_indexed_array` calls
//! - Array access and slicing is re-written using calls to `[]`
//! - Tuple/record access into arrays of structs are rewritten using a
//!   comprehension accessing the inner value

use derive_more::From;
use rustc_hash::FxHashMap;
use shackle_hir::{
	Item, PatternTy, TypeResult,
	constants::IdentifierRegistry,
	counts::EntityCounts,
	ids::{EntityRef, ExpressionRef, NodeRef, PatternRef},
	run_hir_phase,
};
use shackle_ty::{Ty, TyData};
use shackle_utils::maybe_grow_stack;

use super::{source::Origin, *};
use crate::{Db, db::Intermediate};
#[derive(Copy, Clone, Debug, PartialEq, Eq, From)]
enum DeclOrConstraint<'db> {
	Declaration(DeclarationId<'db>),
	Constraint(ConstraintId<'db>),
}

impl<'db> From<DeclOrConstraint<'db>> for LetItem<'db> {
	fn from(d: DeclOrConstraint<'db>) -> Self {
		match d {
			DeclOrConstraint::Constraint(c) => LetItem::Constraint(c),
			DeclOrConstraint::Declaration(d) => LetItem::Declaration(d),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredAnnotation<'db> {
	Items(Vec<DeclOrConstraint<'db>>),
	Expression(Expression<'db>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LoweredIdentifier<'db> {
	ResolvedIdentifier(ResolvedIdentifier<'db>),
	Callable(Callable<'db>),
}

/// Collects HIR items and lowers them to THIR
struct ItemCollector<'db> {
	db: &'db dyn Db,
	ids: &'db IdentifierRegistry<'db>,
	resolutions: FxHashMap<PatternRef<'db>, LoweredIdentifier<'db>>,
	model: Model<'db>,
	type_alias_expressions: FxHashMap<ExpressionRef<'db>, DeclarationId<'db>>,
	deferred: Vec<(FunctionId<'db>, Item<'db>)>,
}

impl<'db> ItemCollector<'db> {
	/// Create a new item collector
	fn new(
		db: &'db dyn Db,
		ids: &'db IdentifierRegistry<'db>,
		entity_counts: &EntityCounts,
	) -> Self {
		Self {
			db,
			ids,
			resolutions: FxHashMap::default(),
			model: Model::with_capacities(&entity_counts.into()),
			type_alias_expressions: FxHashMap::default(),
			deferred: Vec::new(),
		}
	}

	/// Collect an item
	fn collect_item(&mut self, item: Item<'db>) {
		log::debug!(
			"Lowering {:?} at {} to THIR",
			item.get_item_with_data_as_debug(self.db),
			Origin::from(item).pretty_print(self.db)
		);
		match item {
			Item::Annotation(a) => {
				let _ = self.collect_annotation(a);
			}
			Item::Assignment(a) => self.collect_assignment(a),
			Item::Constraint(c) => {
				let _ = self.collect_constraint(item, c.constraint(self.db), true);
			}
			Item::Declaration(d) => {
				let _ = self.collect_declaration(item, d.declaration(self.db), true);
			}
			Item::Enumeration(e) => {
				let _ = self.collect_enumeration(e);
			}
			Item::EnumAssignment(a) => self.collect_enumeration_assignment(a),
			Item::Function(f) => {
				let _ = self.collect_function(f);
			}
			Item::Output(o) => {
				let _ = self.collect_output(o);
			}
			Item::Solve(s) => self.collect_solve(s),
			Item::TypeAlias(t) => self.collect_type_alias(t),
			// Object lowering is not implemented yet. Models using classes are
			// rejected before this point by object validation.
			Item::Class(_) => {}
		}
	}

	/// Collect an annotation item
	fn collect_annotation(&mut self, it: shackle_hir::AnnotationItem<'db>) -> AnnotationId<'db> {
		let item: Item<'_> = it.into();
		let a = it.annotation(self.db);
		let types = item.types(self.db);
		let ty = &types[a.constructor_pattern()];
		match (&a.constructor, ty) {
			(shackle_hir::Constructor::Atom { pattern }, PatternTy::AnnotationAtom) => {
				let annotation = Annotation::new(
					a[*pattern]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				let idx = self
					.model
					.add_annotation(AnnotationItem::new(annotation, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *pattern),
					LoweredIdentifier::ResolvedIdentifier(idx.into()),
				);
				idx
			}
			(
				shackle_hir::Constructor::Function {
					constructor,
					destructor,
					parameters: params,
				},
				PatternTy::AnnotationConstructor(fn_entry),
			) => {
				let mut parameters = Vec::with_capacity(fn_entry.overload.params().len());
				for (param, ty) in params.iter().zip(fn_entry.overload.params()) {
					let mut collector = ExpressionCollector::new(self, a.data(), item, &types);
					let domain = collector.collect_domain(param.declared_type, *ty, false);
					let mut param_decl = Declaration::new(false, domain);
					// Ignore destructuring and recording resolution for now since these can't have bodies which refer
					// to parameters anyway
					if let Some(p) = param.pattern
						&& let Some(i) = a[p].identifier()
					{
						param_decl.set_name(i);
					}
					let idx = self
						.model
						.add_declaration(DeclarationItem::new(param_decl, item));
					parameters.push(idx);
				}
				let mut annotation = Annotation::new(
					a[*constructor]
						.identifier()
						.expect("Annotation must have identifier pattern"),
				);
				annotation.parameters = Some(parameters);
				let idx = self
					.model
					.add_annotation(AnnotationItem::new(annotation, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *constructor),
					LoweredIdentifier::Callable(Callable::Annotation(idx)),
				);
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, *destructor),
					LoweredIdentifier::Callable(Callable::AnnotationDestructure(idx)),
				);
				idx
			}
			_ => unreachable!(),
		}
	}

	/// Collect an assignment item
	fn collect_assignment(&mut self, it: shackle_hir::AssignmentItem<'db>) {
		let item: Item<'_> = it.into();
		let db = self.db;
		let a = it.assignment(db);
		let types = item.types(db);
		let res = types.name_resolution(a.assignee).unwrap();
		let decl = match &self.resolutions[&res] {
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Declaration(d)) => *d,
			_ => unreachable!(),
		};
		if self.model[decl].definition().is_some() {
			// Turn subsequent assignment items into equality constraints
			let mut collector = ExpressionCollector::new(self, a.data(), item, &types);
			let call = LookupCall {
				function: collector.parent.ids.functions.eq.into(),
				arguments: vec![
					collector.collect_expression(a.assignee),
					collector.collect_expression(a.definition),
				],
			};
			let constraint = Constraint::new(
				true,
				Expression::new(db, &collector.parent.model, item, call),
			);
			let _ = collector
				.parent
				.model
				.add_constraint(ConstraintItem::new(constraint, item));
		} else {
			let mut declaration = self.model[decl].clone();
			let mut collector = ExpressionCollector::new(self, a.data(), item, &types);
			let def = collector.collect_expression(a.definition);
			declaration.set_definition(def);
			self.model[decl] = declaration;
		}
	}

	/// Collect a constraint item
	fn collect_constraint(
		&mut self,
		item: Item<'db>,
		c: &shackle_hir::Constraint<'db>,
		top_level: bool,
	) -> ConstraintId<'db> {
		let db = self.db;
		let types = item.types(db);
		let mut collector = ExpressionCollector::new(self, item.data(db), item, &types);
		let mut constraint = Constraint::new(top_level, collector.collect_expression(c.expression));
		constraint.annotations_mut().extend(
			c.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		self.model
			.add_constraint(ConstraintItem::new(constraint, item))
	}

	/// Collect a declaration item
	fn collect_declaration(
		&mut self,
		item: Item<'db>,
		d: &shackle_hir::Declaration<'db>,
		top_level: bool,
	) -> Vec<DeclOrConstraint<'db>> {
		let db = self.db;
		let types = item.types(db);
		let ty = match &types[d.pattern] {
			PatternTy::Variable(ty) => *ty,
			PatternTy::Destructuring(ty) => *ty,
			_ => unreachable!(),
		};
		let data = item.data(db);
		let mut collector = ExpressionCollector::new(self, data, item, &types);
		let domain = collector.collect_domain(d.declared_type, ty, false);
		let mut decl = Declaration::new(top_level, domain);
		if let Some(def) = d.definition {
			decl.set_definition(collector.collect_expression(def));
		}
		let idx = collector
			.parent
			.model
			.add_declaration(DeclarationItem::new(decl, item));
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
	fn collect_enumeration(&mut self, it: shackle_hir::EnumerationItem<'db>) -> EnumerationId<'db> {
		let item: Item<'_> = it.into();
		let e = it.enumeration(self.db);
		let db = self.db;
		let types = item.types(db);
		let ty = &types[e.pattern];
		match ty {
			PatternTy::Enum(ty) => match ty.lookup(self.db) {
				TyData::Set(VarType::Par, OptType::NonOpt, element) => {
					match element.lookup(self.db) {
						TyData::Enum(_, _, t) => {
							let mut enumeration = Enumeration::new(*t);
							{
								let mut collector =
									ExpressionCollector::new(self, e.data(), item, &types);
								enumeration.annotations_mut().extend(
									e.annotations
										.iter()
										.map(|ann| collector.collect_expression(*ann)),
								);
							}
							if let Some(def) = &e.definition {
								enumeration.set_definition(
									def.iter()
										.map(|c| self.collect_enum_case(c, e.data(), item, &types)),
								)
							}
							let idx = self
								.model
								.add_enumeration(EnumerationItem::new(enumeration, item));
							let _ = self.resolutions.insert(
								PatternRef::new(self.db, item, e.pattern),
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
	fn collect_enumeration_assignment(&mut self, it: shackle_hir::EnumAssignmentItem<'db>) {
		let item: Item<'_> = it.into();
		let a = it.enum_assignment(self.db);
		let types = item.types(self.db);
		let res = types.name_resolution(a.assignee).unwrap();
		let idx = match &self.resolutions[&res] {
			LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Enumeration(e)) => *e,
			_ => unreachable!(),
		};
		let def = a
			.definition
			.iter()
			.map(|c| self.collect_enum_case(c, a.data(), item, &types))
			.collect::<Vec<_>>();
		self.model[idx].set_definition(def);
		self.add_enum_resolutions(idx, item, a.definition.iter());
	}

	fn add_enum_resolutions<'a>(
		&mut self,
		idx: EnumerationId<'db>,
		item: Item<'db>,
		ecs: impl Iterator<Item = &'a shackle_hir::EnumConstructor<'db>>,
	) where
		'db: 'a,
	{
		for (i, ec) in ecs.enumerate() {
			match ec {
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Atom { pattern }) => {
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *pattern),
						LoweredIdentifier::ResolvedIdentifier(
							EnumMemberId::new(idx, i as u32).into(),
						),
					);
				}
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Function {
					constructor,
					destructor,
					..
				}) => {
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *constructor),
						LoweredIdentifier::Callable(Callable::EnumConstructor(EnumMemberId::new(
							idx, i as u32,
						))),
					);
					let _ = self.resolutions.insert(
						PatternRef::new(self.db, item, *destructor),
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
		c: &shackle_hir::EnumConstructor<'db>,
		data: &shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &TypeResult<'db>,
	) -> Constructor<'db> {
		let (name, params) = match (c, &types[c.constructor_pattern()]) {
			(
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Atom { pattern }),
				_,
			) => {
				return Constructor {
					name: data[*pattern].identifier(),
					parameters: None,
				};
			}
			(
				shackle_hir::EnumConstructor::Named(shackle_hir::Constructor::Function {
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
				shackle_hir::EnumConstructor::Anonymous { parameters, .. },
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
						self.model
							.add_declaration(DeclarationItem::new(declaration, item))
					})
					.collect(),
			),
		}
	}

	/// Collect a function item
	fn collect_function(&mut self, it: shackle_hir::FunctionItem<'db>) -> FunctionId<'db> {
		let item: Item<'_> = it.into();
		let f = it.function(self.db);
		let types = item.types(self.db);
		let mut collector = ExpressionCollector::new(self, f.data(), item, &types);
		let res = PatternRef::new(collector.parent.db, item, f.pattern);
		match &types[f.pattern] {
			PatternTy::Function(fn_entry) => {
				let domain =
					collector.collect_domain(f.return_type, fn_entry.overload.return_type(), false);
				let name = f[f.pattern].identifier().unwrap();
				let mut function = Function::new(name.into(), domain);
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
							.collect_fn_param(param, *ty, f.data(), item, &types)
					})
					.collect::<Vec<_>>();
				function.set_parameters(parameters);

				let idx = self.model.add_function(FunctionItem::new(function, item));
				let _ = self
					.resolutions
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
		param: &shackle_hir::Parameter<'db>,
		ty: Ty<'db>,
		data: &shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &TypeResult<'db>,
	) -> DeclarationId<'db> {
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
		self.model
			.add_declaration(DeclarationItem::new(declaration, item))
	}

	/// Collect an output item
	fn collect_output(&mut self, it: shackle_hir::OutputItem<'db>) -> OutputId<'db> {
		let item: Item<'_> = it.into();
		let o = it.output(self.db);
		let types = item.types(self.db);
		let mut collector = ExpressionCollector::new(self, o.data(), item, &types);
		let mut output = Output::new(collector.collect_expression(o.expression));
		if let Some(s) = o.section {
			output.set_section(collector.collect_expression(s));
		}
		self.model.add_output(OutputItem::new(output, item))
	}

	/// Collect solve item
	fn collect_solve(&mut self, it: shackle_hir::SolveItem<'db>) {
		let item: Item<'_> = it.into();
		let s = it.solve(self.db);
		let types = item.types(self.db);
		let mut optimise = |pattern: shackle_hir::PatternId<'db>,
		                    objective: shackle_hir::ExpressionId<'db>,
		                    is_maximize: bool| match &types[pattern] {
			PatternTy::Variable(ty) => {
				let objective_origin =
					EntityRef::new(self.db, item, shackle_hir::ids::EntityId::from(objective));
				let mut collector = ExpressionCollector::new(self, s.data(), item, &types);
				let mut declaration = Declaration::new(
					true,
					Domain::unbounded(collector.parent.db, objective_origin, *ty),
				);
				if let Some(name) = s[pattern].identifier() {
					declaration.set_name(name);
				}
				let obj = collector.collect_expression(objective);
				declaration.set_definition(obj);
				let idx = self
					.model
					.add_declaration(DeclarationItem::new(declaration, item));
				let _ = self.resolutions.insert(
					PatternRef::new(self.db, item, pattern),
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
			shackle_hir::Goal::Maximize { pattern, objective } => {
				optimise(*pattern, *objective, true)
			}
			shackle_hir::Goal::Minimize { pattern, objective } => {
				optimise(*pattern, *objective, false)
			}
			shackle_hir::Goal::Satisfy => Solve::satisfy(),
		};
		let mut collector = ExpressionCollector::new(self, s.data(), item, &types);
		si.annotations_mut().extend(
			s.annotations
				.iter()
				.map(|ann| collector.collect_expression(*ann)),
		);
		let _ = self.model.set_solve(SolveItem::new(si, item));
	}

	fn collect_type_alias(&mut self, it: shackle_hir::TypeAliasItem<'db>) {
		let item: Item<'_> = it.into();
		let ta = it.type_alias(self.db);
		let types = item.types(self.db);
		let data = item.data(self.db);
		for e in shackle_hir::Type::expressions(ta.aliased_type, ta.data()) {
			if let Some(res) = types.name_resolution(e) {
				let res_types = res.item(self.db).types(self.db);
				if matches!(
					&res_types[res.pattern(self.db)],
					PatternTy::TypeAlias { .. }
				) {
					// Skip type aliases inside other type aliases (already will be processed)
					continue;
				}
			}
			// Create a declaration with the value of each expression used in a type alias
			let expression =
				ExpressionCollector::new(self, data, item, &types).collect_expression(e);
			let decl = Declaration::from_expression(self.db, true, expression);
			let idx = self.model.add_declaration(DeclarationItem::new(
				decl,
				EntityRef::new(self.db, item, shackle_hir::ids::EntityId::from(e)),
			));
			let _ = self
				.type_alias_expressions
				.insert(ExpressionRef::new(self.db, item, e), idx);
		}
	}

	/// Collect deferred function bodies
	fn collect_deferred(&mut self) {
		for (func, item) in self.deferred.clone().into_iter() {
			let types = item.types(self.db);
			let data = item.data(self.db);
			match item {
				Item::Function(f) => {
					let mut function = self.model[func].clone();
					let param_decls = function.parameters().to_owned();
					let mut decls = Vec::new();
					let mut collector = ExpressionCollector::new(self, data, item, &types);
					let ff = f.function(collector.parent.db);
					for (decl, param) in param_decls.into_iter().zip(ff.parameters.iter()) {
						if let Some(p) = param.pattern {
							let dsts = collector.collect_destructuring(decl, false, p);
							decls.extend(dsts);
						}
					}
					let body = ff.body.unwrap();
					let collected_body = collector.collect_expression(body);
					let e = if decls.is_empty() {
						collected_body
					} else {
						let origin = EntityRef::new(
							collector.parent.db,
							item,
							shackle_hir::ids::EntityId::from(body),
						);
						Expression::new(
							collector.parent.db,
							&collector.parent.model,
							origin,
							Let {
								items: decls.into_iter().map(LetItem::Declaration).collect(),
								in_expression: Box::new(collected_body),
							},
						)
					};
					function.set_body(e);
					collector.parent.model[func] = function;
				}
				_ => unreachable!(),
			}
		}
	}

	/// Finish lowering
	fn finish(self) -> Model<'db> {
		self.model
	}
}

struct ExpressionCollector<'db, 'a, 'b, 'c> {
	parent: &'a mut ItemCollector<'db>,
	data: &'b shackle_hir::ItemData<'db>,
	item: Item<'db>,
	types: &'c TypeResult<'db>,
}

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	fn new(
		parent: &'a mut ItemCollector<'db>,
		data: &'b shackle_hir::ItemData<'db>,
		item: Item<'db>,
		types: &'c TypeResult<'db>,
	) -> Self {
		Self {
			parent,
			data,
			types,
			item,
		}
	}

	fn introduce_declaration(
		&mut self,
		top_level: bool,
		origin: impl Into<Origin<'db>>,
		f: impl FnOnce(&mut Self) -> Expression<'db>,
	) -> DeclarationId<'db> {
		let origin: Origin = origin.into();
		let def = f(self);
		let decl = Declaration::from_expression(self.parent.db, top_level, def);
		self.parent
			.model
			.add_declaration(DeclarationItem::new(decl, origin))
	}

	/// Collect an expression
	fn collect_expression(&mut self, idx: shackle_hir::ExpressionId<'db>) -> Expression<'db> {
		maybe_grow_stack(|| self.collect_expression_inner(idx))
	}

	fn collect_expression_inner(&mut self, idx: shackle_hir::ExpressionId<'db>) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let mut result = match &self.data[idx] {
			shackle_hir::Expression::Absent => alloc_expression(Absent, self, origin),
			shackle_hir::Expression::ArrayAccess(aa) => {
				let is_slice = match self.types[aa.indices].lookup(db) {
					TyData::Tuple(_, fs) => fs.iter().any(|f| f.is_set(db)),
					TyData::Set(_, _, _) => true,
					_ => false,
				};
				if is_slice {
					self.collect_slice(aa.collection, aa.indices, origin)
				} else {
					let c = self.collect_expression(aa.collection);
					let i = self.collect_expression(aa.indices);
					self.collect_array_access(c, i, origin)
				}
			}
			shackle_hir::Expression::ArrayComprehension(c) => {
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
			shackle_hir::Expression::ArrayLiteral(al) => alloc_expression(
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
			shackle_hir::Expression::ArrayLiteral2D(al) => {
				let mut idx_array = |dim: &shackle_hir::MaybeIndexSet<'db>| match dim {
					shackle_hir::MaybeIndexSet::Indexed(es) => alloc_expression(
						ArrayLiteral(es.iter().map(|e| self.collect_expression(*e)).collect()),
						self,
						origin,
					),
					shackle_hir::MaybeIndexSet::NonIndexed(c) => alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.set2array.into(),
							arguments: vec![if *c > 0 {
								alloc_expression(
									LookupCall {
										function: self.parent.ids.functions.dot_dot.into(),
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
						function: self.parent.ids.functions.mzn_array_2d_literal.into(),
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
			shackle_hir::Expression::IndexedArrayLiteral(al) => {
				if al.indices.len() == 1 {
					alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.mzn_start_indexed_array.into(),
							arguments: vec![
								self.collect_expression(al.indices[0]),
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
				} else {
					alloc_expression(
						LookupCall {
							function: self.parent.ids.builtins.mzn_indexed_array.into(),
							arguments: vec![alloc_expression(
								ArrayLiteral(
									al.indices
										.iter()
										.zip(al.members.iter())
										.map(|(i, e)| {
											alloc_expression(
												TupleLiteral(vec![
													self.collect_expression(*i),
													self.collect_expression(*e),
												]),
												self,
												origin,
											)
										})
										.collect(),
								),
								self,
								origin,
							)],
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::BooleanLiteral(b) => alloc_expression(*b, self, origin),
			shackle_hir::Expression::Call(c) => {
				let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function]
				{
					let res = self.types.name_resolution(c.function).unwrap_or_else(|| {
						panic!(
							"No name resolution in types for {:?} at {:?}",
							c.function,
							ExpressionRef::new(self.parent.db, self.item, c.function)
								.source_span(self.parent.db)
						);
					});
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						let f = ExpressionRef::new(self.parent.db, self.item, c.function);
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							res.into_entity(self.parent.db).source_span(self.parent.db),
							f,
							f.source_span(self.parent.db),
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
			shackle_hir::Expression::Case(c) => {
				let scrutinee_origin = ExpressionRef::new(self.parent.db, self.item, c.expression)
					.into_entity(self.parent.db);
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
										let pattern_origin = PatternRef::new(
											self.parent.db,
											self.item,
											case.pattern,
										)
										.into_entity(self.parent.db);
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
			shackle_hir::Expression::FloatLiteral(f) => alloc_expression(*f, self, origin),
			shackle_hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(idx).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					let e = ExpressionRef::new(db, self.item, idx);
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						res.into_entity(self.parent.db).source_span(self.parent.db),
						e,
						e.source_span(self.parent.db),
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
					assert_eq!(expr.ty().make_par(db), ty);
					alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.fix.into(),
							arguments: vec![expr],
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::IfThenElse(ite) => alloc_expression(
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
			shackle_hir::Expression::Infinity => alloc_expression(Infinity, self, origin),
			shackle_hir::Expression::IntegerLiteral(i) => alloc_expression(*i, self, origin),
			shackle_hir::Expression::Lambda(l) => {
				let fn_type = match ty.lookup(db) {
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
						let body_entity = ExpressionRef::new(db, self.item, l.body).into_entity(db);
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
				let f = self
					.parent
					.model
					.add_function(FunctionItem::new(function, origin));
				alloc_expression(Lambda(f), self, origin)
			}
			shackle_hir::Expression::Let(l) => alloc_expression(
				Let {
					items: l
						.items
						.iter()
						.flat_map(|i| match i {
							shackle_hir::LetItem::Constraint(c) => {
								let constraint =
									self.parent.collect_constraint(self.item, c, false);
								vec![LetItem::Constraint(constraint)]
							}
							shackle_hir::LetItem::Declaration(d) => self
								.parent
								.collect_declaration(self.item, d, false)
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
			shackle_hir::Expression::RecordAccess(ra) => {
				let record = self.collect_expression(ra.record);
				if self.types[ra.record].is_array(self.parent.db) {
					// Lift to comprehension
					let record_ty = record.ty().elem_ty(self.parent.db).unwrap();
					let declaration = Declaration::new(
						false,
						Domain::unbounded(self.parent.db, origin, record_ty),
					);
					let idx = self
						.parent
						.model
						.add_declaration(DeclarationItem::new(declaration, origin));
					let g = Generator::Iterator {
						declarations: vec![idx],
						collection: record,
						where_clause: None,
					};
					alloc_expression(
						ArrayComprehension {
							generators: vec![g],
							template: Box::new(alloc_expression(
								RecordAccess {
									record: Box::new(alloc_expression(idx, self, origin)),
									field: self.data[ra.field].identifier().unwrap(),
								},
								self,
								origin,
							)),
							indices: None,
						},
						self,
						origin,
					)
				} else {
					alloc_expression(
						RecordAccess {
							record: Box::new(self.collect_expression(ra.record)),
							field: self.data[ra.field].identifier().unwrap(),
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::RecordLiteral(rl) => alloc_expression(
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
			shackle_hir::Expression::SetComprehension(c) => {
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
			shackle_hir::Expression::SetLiteral(sl) => alloc_expression(
				SetLiteral(
					sl.members
						.iter()
						.map(|m| self.collect_expression(*m))
						.collect(),
				),
				self,
				origin,
			),
			shackle_hir::Expression::Slice(_) => {
				unreachable!("Slice used outside of array access")
			}
			shackle_hir::Expression::StringLiteral(sl) => {
				alloc_expression(sl.clone(), self, origin)
			}
			shackle_hir::Expression::TupleAccess(ta) => {
				let tuple = self.collect_expression(ta.tuple);
				if self.types[ta.tuple].is_array(self.parent.db) {
					// Lift to comprehension
					let tuple_ty = tuple.ty().elem_ty(self.parent.db).unwrap();
					let declaration = Declaration::new(
						false,
						Domain::unbounded(self.parent.db, origin, tuple_ty),
					);
					let idx = self
						.parent
						.model
						.add_declaration(DeclarationItem::new(declaration, origin));
					let g = Generator::Iterator {
						declarations: vec![idx],
						collection: tuple,
						where_clause: None,
					};
					alloc_expression(
						ArrayComprehension {
							generators: vec![g],
							template: Box::new(alloc_expression(
								TupleAccess {
									tuple: Box::new(alloc_expression(idx, self, origin)),
									field: IntegerLiteral(
										self.data[ta.field].integer_value().unwrap(),
									),
								},
								self,
								origin,
							)),
							indices: None,
						},
						self,
						origin,
					)
				} else {
					alloc_expression(
						TupleAccess {
							tuple: Box::new(tuple),
							field: IntegerLiteral(self.data[ta.field].integer_value().unwrap()),
						},
						self,
						origin,
					)
				}
			}
			shackle_hir::Expression::TupleLiteral(tl) => alloc_expression(
				TupleLiteral(
					tl.fields
						.iter()
						.map(|f| self.collect_expression(*f))
						.collect(),
				),
				self,
				origin,
			),
			shackle_hir::Expression::Missing => unreachable!("Missing expression"),
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
			result.ty().pretty_print(db),
			ty.pretty_print(db),
			NodeRef::from(origin).source_span(db)
		);
		result
	}

	fn collect_declaration_annotation(
		&mut self,
		decl: DeclarationId<'db>,
		ann: shackle_hir::ExpressionId<'db>,
	) -> LoweredAnnotation<'db> {
		// Declarations can have annotations which point to functions using ::annotated_expression.
		// These need to be desugared into constraints.
		match &self.data[ann] {
			shackle_hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(ann).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					let e = ExpressionRef::new(self.parent.db, self.item, ann);
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						NodeRef::from(res.into_entity(self.parent.db)).source_span(self.parent.db),
						e,
						e.source_span(self.parent.db),
					)
				});
				if let LoweredIdentifier::Callable(function) = ident.clone() {
					let origin = ExpressionRef::new(self.parent.db, self.item, ann)
						.into_entity(self.parent.db);
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
							function: self.parent.ids.functions.annotate.into(),
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
						.add_constraint(ConstraintItem::new(constraint, origin));

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}
			}
			shackle_hir::Expression::Call(c) => {
				let origin =
					ExpressionRef::new(self.parent.db, self.item, ann).into_entity(self.parent.db);
				let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function]
				{
					let res = self.types.name_resolution(c.function).unwrap();
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						let e = ExpressionRef::new(self.parent.db, self.item, c.function);
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							NodeRef::from(res.into_entity(self.parent.db))
								.source_span(self.parent.db),
							e,
							e.source_span(self.parent.db),
						)
					});
					match ident {
						LoweredIdentifier::Callable(c) => c.clone(),
						_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
					}
				} else {
					Callable::Expression(Box::new(self.collect_expression(c.function)))
				};

				if let Callable::Function(f) = &function
					&& self.parent.model[*f].parameters().len() > c.arguments.len()
				{
					// Add the annotated declaration identifier as first argument
					let mut arguments = Vec::with_capacity(c.arguments.len() + 1);
					arguments.push(alloc_expression(
						ResolvedIdentifier::Declaration(decl),
						self,
						origin,
					));
					arguments.extend(c.arguments.iter().map(|arg| self.collect_expression(*arg)));

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
							function: self.parent.ids.functions.annotate.into(),
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
						.add_constraint(ConstraintItem::new(constraint, origin));

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
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

	/// Rewrite index slicing into a call
	///
	/// Turns all indices into sets to match the slicing builtin function, and then coerces to the correct output index set.
	fn collect_slice(
		&mut self,
		collection: shackle_hir::ExpressionId<'db>,
		indices: shackle_hir::ExpressionId<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let collection_entity =
			ExpressionRef::new(self.parent.db, self.item, collection).into_entity(self.parent.db);
		let indices_entity =
			ExpressionRef::new(self.parent.db, self.item, indices).into_entity(self.parent.db);

		let mut decls = Vec::new();
		let collection_decl = if matches!(
			&self.data[collection],
			shackle_hir::Expression::Identifier(_)
		) {
			let expr = self.collect_expression(collection);
			match &*expr {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(decl)) => *decl,
				_ => unreachable!(),
			}
		} else {
			// Add declaration to store collection
			let origin = collection_entity;
			let decl = self.introduce_declaration(false, origin, |collector| {
				collector.collect_expression(collection)
			});
			decls.push(decl);
			decl
		};
		let mut index_sets_for_infinite_slice = None;
		let array_dims = self.types[collection].dims(self.parent.db).unwrap();
		let mut slices = Vec::with_capacity(array_dims);
		match self.types[indices].lookup(self.parent.db) {
			TyData::Tuple(_, fs) => {
				if let shackle_hir::Expression::TupleLiteral(tl) = &self.data[indices] {
					for (i, (ty, e)) in fs.iter().zip(tl.fields.iter()).enumerate() {
						let index_entity = ExpressionRef::new(self.parent.db, self.item, *e)
							.into_entity(self.parent.db);
						let mut is_set = true;
						let decl = self.introduce_declaration(false, index_entity, |collector| {
							if let shackle_hir::Expression::Slice(s) = &collector.data[*e] {
								// Rewrite infinite slice .. into `'..'(index_set_mofn(c))`
								if index_sets_for_infinite_slice.is_none() {
									let decl = collector.introduce_declaration(
										false,
										origin,
										|collector| {
											alloc_expression(
												LookupCall {
													function: self
														.parent
														.ids
														.functions
														.index_sets
														.into(),
													arguments: vec![alloc_expression(
														collection_decl,
														collector,
														collection_entity,
													)],
												},
												collector,
												origin,
											)
										},
									);
									decls.push(decl);
									index_sets_for_infinite_slice = Some(decl);
								}
								alloc_expression(
									LookupCall {
										function: (*s).into(),
										arguments: vec![alloc_expression(
											TupleAccess {
												tuple: Box::new(alloc_expression(
													index_sets_for_infinite_slice.unwrap(),
													collector,
													index_entity,
												)),
												field: IntegerLiteral(i as i64 + 1),
											},
											collector,
											index_entity,
										)],
									},
									collector,
									index_entity,
								)
							} else if ty.is_set(collector.parent.db) {
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
						let is_set = f.is_set(self.parent.db);
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
					if let shackle_hir::Expression::Slice(s) = &collector.data[indices] {
						// Rewrite infinite slice .. into `'..'(index_set(c))`
						alloc_expression(
							LookupCall {
								function: (*s).into(),
								arguments: vec![alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.index_set.into(),
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
		let slice_tuple = alloc_expression(
			TupleLiteral(
				slices
					.iter()
					.map(|(decl, _, origin)| alloc_expression(*decl, self, *origin))
					.collect(),
			),
			self,
			indices_entity,
		);
		let arguments = slices
			.iter()
			.filter_map(|(decl, is_slice, origin)| {
				if *is_slice {
					Some(alloc_expression(*decl, self, *origin))
				} else {
					None
				}
			})
			.chain([alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.mzn_slice.into(),
					arguments: vec![collection_ident, slice_tuple],
				},
				self,
				origin,
			)])
			.collect::<Vec<_>>();
		alloc_expression(
			Let {
				items: decls.into_iter().map(LetItem::Declaration).collect(),
				in_expression: Box::new(alloc_expression(
					LookupCall {
						function: Identifier::new(
							self.parent.db,
							format!("array{}d", arguments.len() - 1),
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

	fn collect_array_access(
		&mut self,
		collection: Expression<'db>,
		indices: Expression<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		maybe_grow_stack(|| {
			let origin = origin.into();
			alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.array_access.into(),
					arguments: vec![collection, indices],
				},
				self,
				origin,
			)
		})
	}

	fn collect_generator(
		&mut self,
		generator: &shackle_hir::Generator<'db>,
		generators: &mut Vec<Generator<'db>>,
	) {
		let pattern_to_where = |c: &mut Self,
		                        decl: DeclarationId<'db>,
		                        p: shackle_hir::PatternId<'db>,
		                        origin: Origin<'db>| {
			// Turn destructuring into where clause of case matching pattern
			let pattern = c.collect_pattern(p);
			alloc_expression(
				Case {
					scrutinee: Box::new(alloc_expression(decl, c, origin)),
					branches: vec![
						CaseBranch::new(pattern, alloc_expression(BooleanLiteral(true), c, origin)),
						CaseBranch::new(
							Pattern::anonymous(
								match &c.types[p] {
									PatternTy::Destructuring(ty) => *ty,
									_ => unreachable!(),
								},
								origin,
							),
							alloc_expression(BooleanLiteral(false), c, origin),
						),
					],
				},
				c,
				origin,
			)
		};

		match generator {
			shackle_hir::Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				let mut assignments = Vec::new();
				let mut where_clauses = Vec::new();
				let declarations = patterns
					.iter()
					.map(|p| {
						let origin = PatternRef::new(self.parent.db, self.item, *p)
							.into_entity(self.parent.db);
						let ty = match &self.types[*p] {
							PatternTy::Variable(ty) | PatternTy::Destructuring(ty) => *ty,
							_ => unreachable!(),
						};
						let declaration =
							Declaration::new(false, Domain::unbounded(self.parent.db, origin, ty));
						let decl = self
							.parent
							.model
							.add_declaration(DeclarationItem::new(declaration, origin));
						let asgs = self.collect_destructuring(decl, false, *p);
						if !asgs.is_empty() && shackle_hir::Pattern::is_refutable(*p, self.data) {
							where_clauses.push(pattern_to_where(self, decl, *p, origin.into()));
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
					let origin = EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(patterns[0]),
					);
					if where_clauses.len() == 1 {
						generators.push(Generator::Iterator {
							declarations,
							collection,
							where_clause: Some(where_clauses.pop().unwrap()),
						});
					} else {
						let call = alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.forall.into(),
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
			shackle_hir::Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let def = ExpressionCollector::new(self.parent, self.data, self.item, self.types)
					.collect_expression(*value);
				let assignment = Declaration::from_expression(self.parent.db, false, def);
				let idx = self.parent.model.add_declaration(DeclarationItem::new(
					assignment,
					EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(*pattern),
					),
				));
				let mut asgs = self.collect_destructuring(idx, false, *pattern);
				generators.push(Generator::Assignment {
					assignment: idx,
					where_clause: where_clause.map(|w| self.collect_expression(w)),
				});
				if !asgs.is_empty() {
					if shackle_hir::Pattern::is_refutable(*pattern, self.data) {
						let w = pattern_to_where(
							self,
							idx,
							*pattern,
							EntityRef::new(
								self.parent.db,
								self.item,
								shackle_hir::ids::EntityId::from(*pattern),
							)
							.into(),
						);
						let last = asgs.pop().unwrap();
						generators.extend(asgs.iter().map(|asg| Generator::Assignment {
							assignment: *asg,
							where_clause: None,
						}));
						generators.push(Generator::Assignment {
							assignment: last,
							where_clause: Some(w),
						});
					} else {
						generators.extend(asgs.iter().map(|asg| Generator::Assignment {
							assignment: *asg,
							where_clause: None,
						}));
					}
				}
			}
		}
	}

	fn collect_default_else(&mut self, ty: Ty<'db>, origin: Origin<'db>) -> Expression<'db> {
		let db = self.parent.db;
		match ty.lookup(db) {
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
			TyData::String(_) => {
				alloc_expression(StringLiteral::new(self.parent.db, ""), self, origin)
			}
			TyData::Annotation(_) => {
				alloc_expression(self.parent.ids.annotations.empty_annotation, self, origin)
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
	fn collect_domain(
		&mut self,
		t: shackle_hir::TypeId<'db>,
		ty: Ty<'db>,
		is_type_alias: bool,
	) -> Domain<'db> {
		let db = self.parent.db;
		let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(t));
		match (&self.data[t], ty.lookup(db)) {
			(shackle_hir::Type::Bounded { domain, .. }, _) => {
				if let Some(res) = self.types.name_resolution(*domain) {
					let res_item = res.item(db);
					let res_types = res_item.types(db);
					let res_data = res_item.data(db);
					match &res_types[res.pattern(db)] {
						// Identifier is actually a type, not a domain expression
						PatternTy::TyVar(_) => {
							return Domain::unbounded(self.parent.db, origin, ty);
						}
						PatternTy::TypeAlias { .. } => match res.item(db) {
							Item::TypeAlias(ta) => {
								let mut c = ExpressionCollector::new(
									self.parent,
									res_data,
									res.item(db),
									&res_types,
								);
								return c.collect_domain(ta.type_alias(db).aliased_type, ty, true);
							}
							_ => unreachable!(),
						},
						_ => (),
					}
				}
				if is_type_alias {
					// Replace expressions with identifiers pointing to declarations for those expressions
					let er = ExpressionRef::new(db, self.item, *domain);
					let origin =
						EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(*domain));
					Domain::bounded(
						db,
						origin,
						ty.inst(db).unwrap(),
						ty.opt(db).unwrap(),
						alloc_expression(self.parent.type_alias_expressions[&er], self, origin),
					)
				} else {
					let e = self.collect_expression(*domain);
					Domain::bounded(db, origin, ty.inst(db).unwrap(), ty.opt(db).unwrap(), e)
				}
			}
			(
				shackle_hir::Type::Array {
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
				*opt,
				self.collect_domain(*dimensions, *d, is_type_alias),
				self.collect_domain(*element, *el, is_type_alias),
			),
			(shackle_hir::Type::Set { element, .. }, TyData::Set(inst, opt, e)) => Domain::set(
				db,
				origin,
				*inst,
				*opt,
				self.collect_domain(*element, *e, is_type_alias),
			),
			(shackle_hir::Type::Tuple { fields, .. }, TyData::Tuple(opt, fs)) => Domain::tuple(
				db,
				origin,
				*opt,
				fields
					.iter()
					.zip(fs.iter())
					.map(|(f, ty)| self.collect_domain(*f, *ty, is_type_alias)),
			),
			(shackle_hir::Type::Record { fields, .. }, TyData::Record(opt, fs)) => Domain::record(
				db,
				origin,
				*opt,
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
		root_decl: DeclarationId<'db>,
		top_level: bool,
		pattern: shackle_hir::PatternId<'db>,
	) -> Vec<DeclarationId<'db>> {
		let mut destructuring = Vec::new();
		let mut todo = vec![(0, pattern)];
		while let Some((i, p)) = todo.pop() {
			match &self.data[p] {
				shackle_hir::Pattern::Tuple { fields } => {
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
				shackle_hir::Pattern::Record { fields } => {
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
				shackle_hir::Pattern::Call {
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
				shackle_hir::Pattern::Identifier(name) => {
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
							let _ = self.parent.resolutions.insert(
								PatternRef::new(self.parent.db, self.item, pattern),
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
			let origin = EntityRef::new(
				self.parent.db,
				self.item,
				shackle_hir::ids::EntityId::from(item.pattern),
			);
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
				let _ = self.parent.resolutions.insert(
					PatternRef::new(self.parent.db, self.item, item.pattern),
					LoweredIdentifier::ResolvedIdentifier(decl.into()),
				);
			}
			let _ = decl_map.insert(idx + 1, decl);
			decls.push(decl);
		}
		decls
	}

	/// Lower an HIR pattern into a THIR pattern
	fn collect_pattern(&mut self, pattern: shackle_hir::PatternId<'db>) -> Pattern<'db> {
		let db = self.parent.db;
		let origin = EntityRef::new(db, self.item, shackle_hir::ids::EntityId::from(pattern));
		let ty = match &self.types[pattern] {
			PatternTy::Destructuring(ty) => *ty,
			PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
				return Pattern::anonymous(*ty, origin);
			}
			_ => unreachable!(),
		};
		match &self.data[pattern] {
			shackle_hir::Pattern::Absent => {
				Pattern::expression(alloc_expression(Absent, self, origin), origin)
			}
			shackle_hir::Pattern::Anonymous => Pattern::anonymous(ty, origin),
			shackle_hir::Pattern::Boolean(b) => {
				Pattern::expression(alloc_expression(*b, self, origin), origin)
			}
			shackle_hir::Pattern::Call {
				function,
				arguments,
			} => {
				let args = arguments
					.iter()
					.map(|a| self.collect_pattern(*a))
					.collect::<Vec<_>>();
				let pat = self.types.pattern_resolution(*function).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::Callable(Callable::Annotation(ann)) => {
						Pattern::annotation_constructor(db, &self.parent.model, origin, *ann, args)
					}
					LoweredIdentifier::Callable(Callable::EnumConstructor(member)) => {
						Pattern::enum_constructor(db, &self.parent.model, origin, *member, args)
					}
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Float { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Identifier(_) => {
				let pat = self.types.pattern_resolution(pattern).unwrap();
				let res = &self.parent.resolutions[&pat];
				match res {
					LoweredIdentifier::ResolvedIdentifier(ResolvedIdentifier::Annotation(a)) => {
						Pattern::expression(alloc_expression(*a, self, origin), origin)
					}
					LoweredIdentifier::ResolvedIdentifier(
						ResolvedIdentifier::EnumerationMember(m),
					) => Pattern::expression(alloc_expression(*m, self, origin), origin),
					_ => unreachable!(),
				}
			}
			shackle_hir::Pattern::Infinity { negated } => {
				let v = alloc_expression(Infinity, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Integer { negated, value } => {
				let v = alloc_expression(*value, self, origin);
				Pattern::expression(
					if *negated {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.minus.into(),
								arguments: vec![v],
							},
							self,
							origin,
						)
					} else {
						v
					},
					origin,
				)
			}
			shackle_hir::Pattern::Missing => unreachable!(),
			shackle_hir::Pattern::Record { fields } => {
				let fields = fields
					.iter()
					.map(|(i, p)| (*i, self.collect_pattern(*p)))
					.collect::<Vec<_>>();
				Pattern::record(db, &self.parent.model, origin, fields)
			}
			shackle_hir::Pattern::String(s) => {
				Pattern::expression(alloc_expression(s.clone(), self, origin), origin)
			}
			shackle_hir::Pattern::Tuple { fields } => {
				let fields = fields
					.iter()
					.map(|f| self.collect_pattern(*f))
					.collect::<Vec<_>>();
				Pattern::tuple(db, &self.parent.model, origin, fields)
			}
		}
	}
}

fn alloc_expression<'db>(
	data: impl ExpressionBuilder<'db>,
	collector: &ExpressionCollector<'db, '_, '_, '_>,
	origin: impl Into<Origin<'db>>,
) -> Expression<'db> {
	Expression::new(collector.parent.db, &collector.parent.model, origin, data)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DestructuringEntry<'db> {
	parent: usize, // 0 means no parent, otherwise = index of parent + 1
	kind: Destructuring<'db>,
	pattern: shackle_hir::PatternId<'db>,
	name: Option<Identifier<'db>>,
	create: bool,
}

impl<'db> DestructuringEntry<'db> {
	fn new(parent: usize, kind: Destructuring<'db>, pattern: shackle_hir::PatternId<'db>) -> Self {
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
enum Destructuring<'db> {
	TupleAccess(IntegerLiteral),
	RecordAccess(Identifier<'db>),
	Enumeration(EnumMemberId<'db>),
	Annotation(AnnotationId<'db>),
}

/// Lower the HIR program into THIR
pub fn lower_model<'db>(db: &'db dyn Db) -> Intermediate<Model<'db>> {
	log::info!("Lowering model to THIR");
	let hir = run_hir_phase(db);
	let ids = IdentifierRegistry::lookup(db);
	let counts = EntityCounts::lookup(db);
	let mut collector = ItemCollector::new(db, ids, counts);
	for item in hir.items.iter() {
		collector.collect_item(*item);
	}
	collector.collect_deferred();
	Intermediate::new(collector.finish())
}
