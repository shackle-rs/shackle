//! Erase option types, enums, records
//! - Replace a non optional literal `x` with `(true, x)` if needed to coerce to optional
//! - Replace `<>` with `(false, ⊥)`
//! - Replace `opt T` with `tuple(bool, T)`
//! - Make `occurs(x)` return `x.1` and `deopt(x)` return `x.2`
//! - Populate declarations/functions for known enum constructors
//! - Replace enum types with `int`
//! - Replace records with tuples

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	arena::ArenaMap,
	constants::{IdentifierRegistry, TypeRegistry},
	hir::{BooleanLiteral, IntegerLiteral, OptType, VarType},
	refmap::RefMap,
	thir::{
		db::Thir, fold_call, fold_domain, fold_expression, fold_identifier, source::Origin,
		visit_declaration, visit_expression, visit_function, ArrayAccess, ArrayComprehension,
		ArrayLiteral, Bottom, Call, Callable, Declaration, DeclarationId, Domain, DomainData,
		EnumConstructorKind, EnumerationId, EnumerationItem, Expression, ExpressionData, Folder,
		Function, FunctionId, FunctionName, Generator, Item, ItemId, Let, LetItem, LookupCall,
		Marker, Model, ReplacementMap, ResolvedIdentifier, TupleAccess, TupleLiteral, Visitor,
	},
	ty::{Ty, TyData},
};

struct TopDownTyper<'a, T> {
	db: &'a dyn Thir,
	types: RefMap<'a, Expression<T>, Ty>,
}

impl<'a, T: Marker> Visitor<'a, T> for TopDownTyper<'a, T> {
	fn visit_declaration(&mut self, model: &'a Model<T>, declaration: DeclarationId<T>) {
		if let Some(def) = model[declaration].definition() {
			self.insert(def, model[declaration].ty());
		}
		visit_declaration(self, model, declaration)
	}

	fn visit_function(&mut self, model: &'a Model<T>, function: FunctionId<T>) {
		if let Some(body) = model[function].body() {
			self.insert(body, model[function].domain().ty());
		}
		visit_function(self, model, function, true)
	}

	fn visit_expression(&mut self, model: &'a Model<T>, expression: &'a Expression<T>) {
		let ty = self.get(expression).unwrap_or_else(|| expression.ty());
		match &**expression {
			ExpressionData::ArrayLiteral(al) => self.extend(
				al.iter()
					.map(|e| (e, ty.elem_ty(self.db.upcast()).unwrap())),
			),
			ExpressionData::ArrayComprehension(c) => {
				self.insert(&*c.template, ty.elem_ty(self.db.upcast()).unwrap())
			}
			ExpressionData::SetLiteral(sl) => self.extend(sl.iter().map(|e| {
				(
					e,
					ty.elem_ty(self.db.upcast())
						.unwrap()
						.with_inst(self.db.upcast(), ty.inst(self.db.upcast()).unwrap())
						.unwrap(),
				)
			})),
			ExpressionData::SetComprehension(c) => self.insert(
				&*c.template,
				ty.elem_ty(self.db.upcast())
					.unwrap()
					.with_inst(self.db.upcast(), ty.inst(self.db.upcast()).unwrap())
					.unwrap(),
			),
			ExpressionData::IfThenElse(ite) => self.extend(
				ite.branches
					.iter()
					.map(|b| &b.result)
					.chain([&*ite.else_result])
					.map(|e| (e, ty)),
			),
			ExpressionData::Case(c) => self.extend(c.branches.iter().map(|b| (&b.result, ty))),
			ExpressionData::TupleLiteral(tl) => {
				self.extend(tl.iter().zip(ty.fields(self.db.upcast()).unwrap()))
			}
			ExpressionData::RecordLiteral(rl) => self.extend(
				rl.iter()
					.zip(ty.fields(self.db.upcast()).unwrap())
					.map(|((_, e), t)| (e, t)),
			),
			ExpressionData::Call(c) => {
				let params = match &c.function {
					Callable::Annotation(a) => model[*a]
						.parameters
						.as_ref()
						.unwrap()
						.iter()
						.map(|p| model[*p].ty())
						.collect::<Vec<_>>(),
					Callable::AnnotationDestructure(_) => vec![self.db.type_registry().ann],
					Callable::EnumConstructor(e) => model[*e]
						.parameters
						.as_ref()
						.unwrap()
						.iter()
						.map(|p| model[*p].ty())
						.collect::<Vec<_>>(),
					Callable::EnumDestructor(_) => {
						let (_, ty) = EnumConstructorKind::from_ty(self.db, c.arguments[0].ty());
						vec![ty]
					}
					Callable::Expression(e) => e.ty().function_params(self.db.upcast()).unwrap(),
					Callable::Function(f) => model[*f]
						.parameters()
						.iter()
						.map(|p| model[*p].ty())
						.collect::<Vec<_>>(),
				};
				self.extend(c.arguments.iter().zip(params))
			}
			_ => (),
		}
		visit_expression(self, model, expression)
	}
}

impl<'a, T: Marker> TopDownTyper<'a, T> {
	fn insert(&mut self, e: &'a Expression<T>, ty: Ty) {
		self.types.insert(e, ty);
	}

	fn extend(&mut self, iter: impl Iterator<Item = (&'a Expression<T>, Ty)>) {
		self.types.extend(iter);
	}

	fn get(&self, e: &'a Expression<T>) -> Option<Ty> {
		self.types.get(e).copied()
	}
}

struct ErasedEnum<Dst> {
	defining_set: DeclarationId<Dst>,
	members: Vec<ErasedEnumMember<Dst>>,
}

enum ErasedEnumMember<Dst> {
	Atomic(DeclarationId<Dst>),
	Constructor {
		constructors: EnumConstructorFunctions<Dst>,
		destructors: EnumConstructorFunctions<Dst>,
	},
}

struct EnumConstructorFunctions<Dst> {
	par: FunctionId<Dst>,
	var: FunctionId<Dst>,
	opt: FunctionId<Dst>,
	var_opt: FunctionId<Dst>,
	set: FunctionId<Dst>,
	var_set: FunctionId<Dst>,
}

impl<Dst> EnumConstructorFunctions<Dst> {
	fn get(&self, kind: EnumConstructorKind) -> FunctionId<Dst> {
		match kind {
			EnumConstructorKind::Par => self.par,
			EnumConstructorKind::Var => self.var,
			EnumConstructorKind::Opt => self.opt,
			EnumConstructorKind::VarOpt => self.var_opt,
			EnumConstructorKind::Set => self.set,
			EnumConstructorKind::VarSet => self.var_set,
		}
	}
}

struct TypeEraser<'a, Dst, Src = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	tys: Arc<TypeRegistry>,
	enum_map: ArenaMap<EnumerationItem<Src>, ErasedEnum<Dst>>,
	top_down_types: TopDownTyper<'a, Src>,
	needs_opt_erase: FxHashMap<(Ty, Ty), bool>,
}

impl<Dst: Marker, Src: Marker> Folder<Dst, Src> for TypeEraser<'_, Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn add_model(&mut self, db: &dyn Thir, model: &Model<Src>) {
		// Add items to the destination model
		for item in model.top_level_items() {
			if let ItemId::Enumeration(e) = item {
				// Erase enum items
				self.erase_enum(db, model, e);
			} else {
				self.add_item(db, model, item);
			}
		}
		// Now that all items have been added, we can process function bodies
		for (f, i) in model.all_functions() {
			if i.body().is_some() {
				self.fold_function_body(db, model, f);
			} else if i.name() == self.ids.occurs {
				// Add body to occurs which accesses boolean from tuple
				let idx = self.replacement_map.get_function(f).unwrap();
				let origin = self.model[idx].origin();
				let body = Expression::new(
					db,
					&self.model,
					origin,
					TupleAccess {
						tuple: Box::new(Expression::new(
							db,
							&self.model,
							origin,
							self.model[idx].parameter(0),
						)),
						field: IntegerLiteral(1),
					},
				);
				self.model[idx].set_body(body);
			} else if i.name() == self.ids.deopt {
				// Add body to deopt which accesses value from tuple
				let idx = self.replacement_map.get_function(f).unwrap();
				let origin = self.model[idx].origin();
				let body = Expression::new(
					db,
					&self.model,
					origin,
					TupleAccess {
						tuple: Box::new(Expression::new(
							db,
							&self.model,
							origin,
							self.model[idx].parameter(0),
						)),
						field: IntegerLiteral(2),
					},
				);
				self.model[idx].set_body(body);
			}
		}
	}

	fn fold_identifier(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		identifier: &ResolvedIdentifier<Src>,
	) -> ResolvedIdentifier<Dst> {
		match identifier {
			ResolvedIdentifier::Enumeration(e) => {
				// Erase enum into declaration
				ResolvedIdentifier::Declaration(self.enum_map[*e].defining_set)
			}
			ResolvedIdentifier::EnumerationMember(e) => {
				// Erase enum member into declaration
				let erased = &self.enum_map[e.enumeration_id()];
				let member = &erased.members[e.member_index() as usize];
				match member {
					ErasedEnumMember::Atomic(d) => ResolvedIdentifier::Declaration(*d),
					_ => unreachable!(),
				}
			}
			_ => fold_identifier(self, db, model, identifier),
		}
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model<Src>, call: &Call<Src>) -> Call<Dst> {
		// Erase enum constructor into function call
		match &call.function {
			Callable::EnumConstructor(e) => {
				let erased = &self.enum_map[e.enumeration_id()];
				let member = &erased.members[e.member_index() as usize];
				let function = match member {
					ErasedEnumMember::Constructor { constructors, .. } => {
						let kind = EnumConstructorKind::from_tys(
							db,
							call.arguments.iter().map(|arg| arg.ty()),
						);
						constructors.get(kind)
					}
					_ => unreachable!(),
				};
				Call {
					function: Callable::Function(function),
					arguments: call
						.arguments
						.iter()
						.map(|arg| self.fold_expression(db, model, arg))
						.collect(),
				}
			}
			Callable::EnumDestructor(e) => {
				let erased = &self.enum_map[e.enumeration_id()];
				let member = &erased.members[e.member_index() as usize];
				let function = match member {
					ErasedEnumMember::Constructor { destructors, .. } => {
						let (kind, _) = EnumConstructorKind::from_ty(db, call.arguments[0].ty());
						destructors.get(kind)
					}
					_ => unreachable!(),
				};
				Call {
					function: Callable::Function(function),
					arguments: call
						.arguments
						.iter()
						.map(|arg| self.fold_expression(db, model, arg))
						.collect(),
				}
			}
			_ => fold_call(self, db, model, call),
		}
	}

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		expression: &Expression<Src>,
	) -> Expression<Dst> {
		let origin = expression.origin();
		let folded = match &**expression {
			ExpressionData::Absent => {
				// Transform `<>` into `(false, ⊥)`
				let bool_false = Expression::new(db, &self.model, origin, BooleanLiteral(false));
				let bottom = Expression::new(db, &self.model, origin, Bottom);
				let mut e = Expression::new(
					db,
					&self.model,
					origin,
					TupleLiteral(vec![bool_false, bottom]),
				);
				e.annotations_mut().extend(
					expression
						.annotations()
						.iter()
						.map(|ann| self.fold_expression(db, model, ann)),
				);
				return e;
			}
			ExpressionData::RecordLiteral(rl) => {
				// Erase record literals into tuples
				let fields = rl
					.iter()
					.map(|(_, e)| self.fold_expression(db, model, e))
					.collect();
				let mut e = Expression::new(db, &self.model, origin, TupleLiteral(fields));
				e.annotations_mut().extend(
					expression
						.annotations()
						.iter()
						.map(|ann| self.fold_expression(db, model, ann)),
				);
				e
			}
			_ => fold_expression(self, db, model, expression),
		};
		self.erase_opt(
			db,
			self.top_down_types
				.get(expression)
				.unwrap_or_else(|| expression.ty()),
			expression.ty(),
			folded,
		)
	}

	fn fold_domain(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
	) -> Domain<Dst> {
		let origin = domain.origin();
		if let Some(OptType::Opt) = domain.ty().opt(db.upcast()) {
			// Convert into tuple of occurs boolean and non-optional value
			let occurs = if let Some(VarType::Var) = domain.ty().inst(db.upcast()) {
				self.tys.var_bool
			} else {
				self.tys.par_bool
			};
			let deopt = domain.ty().with_opt(db.upcast(), OptType::NonOpt);
			let mut folded = fold_domain(self, db, model, domain);
			folded.set_ty_unchecked(deopt);
			return Domain::tuple(
				db,
				origin,
				OptType::NonOpt,
				[Domain::unbounded(db, origin, occurs), folded],
			);
		}
		match &**domain {
			DomainData::Record(items) => {
				// Erase record types into tuples
				let fields = items
					.iter()
					.map(|(_, d)| self.fold_domain(db, model, d))
					.collect::<Vec<_>>();
				Domain::tuple(db, origin, OptType::NonOpt, fields)
			}
			_ => fold_domain(self, db, model, domain),
		}
	}
}

impl<Src: Marker, Dst: Marker> TypeEraser<'_, Dst, Src> {
	fn needs_opt_erase(&mut self, db: &dyn Thir, top_down: Ty, bottom_up: Ty) -> bool {
		if let Some(b) = self.needs_opt_erase.get(&(top_down, bottom_up)) {
			return *b;
		}
		let result = top_down.opt(db.upcast()) == Some(OptType::Opt)
			&& bottom_up.opt(db.upcast()) == Some(OptType::NonOpt)
			|| match (top_down.lookup(db.upcast()), bottom_up.lookup(db.upcast())) {
				(TyData::Array { element: t_ty, .. }, TyData::Array { element: b_ty, .. }) => {
					self.needs_opt_erase(db, t_ty, b_ty)
				}
				(TyData::Set(_, _, t_ty), TyData::Set(_, _, b_ty)) => {
					self.needs_opt_erase(db, t_ty, b_ty)
				}
				(TyData::Tuple(_, fs1), TyData::Tuple(_, fs2)) => fs1
					.iter()
					.zip(fs2.iter())
					.any(|(t_ty, b_ty)| self.needs_opt_erase(db, *t_ty, *b_ty)),
				(TyData::Record(_, fs1), TyData::Record(_, fs2)) => fs1
					.iter()
					.zip(fs2.iter())
					.any(|((_, t_ty), (_, b_ty))| self.needs_opt_erase(db, *t_ty, *b_ty)),
				_ => false,
			};
		self.needs_opt_erase.insert((top_down, bottom_up), result);
		result
	}

	fn erase_opt(
		&mut self,
		db: &dyn Thir,
		top_down_ty: Ty,
		bottom_up_ty: Ty,
		e: Expression<Dst>,
	) -> Expression<Dst> {
		let origin = e.origin();
		if top_down_ty.opt(db.upcast()) == Some(OptType::Opt)
			&& bottom_up_ty.opt(db.upcast()) == Some(OptType::NonOpt)
		{
			// Known to occur, transform `x` into `(true, x)`
			let bool_true = Expression::new(db, &self.model, origin, BooleanLiteral(true));
			return Expression::new(db, &self.model, origin, TupleLiteral(vec![bool_true, e]));
		}

		if self.needs_opt_erase(db, top_down_ty, bottom_up_ty) {
			// Needs to be reconstructed to erase optionality
			let (decl, ident) = if matches!(&*e, ExpressionData::Identifier(_)) {
				(None, e)
			} else {
				let mut declaration =
					Declaration::new(false, Domain::unbounded(db, origin, e.ty()));
				declaration.set_definition(e);
				let idx = self.model.add_declaration(Item::new(declaration, origin));
				(Some(idx), Expression::new(db, &self.model, origin, idx))
			};

			let erased = match bottom_up_ty.lookup(db.upcast()) {
				TyData::Array { .. } => {
					// Create comprehension then erase optionality on template
					let idx = self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::unbounded(db, origin, ident.ty().elem_ty(db.upcast()).unwrap()),
						),
						origin,
					));
					let template = self.erase_opt(
						db,
						top_down_ty.elem_ty(db.upcast()).unwrap(),
						bottom_up_ty.elem_ty(db.upcast()).unwrap(),
						Expression::new(db, &self.model, origin, idx),
					);
					Expression::new(
						db,
						&self.model,
						origin,
						LookupCall {
							function: self.ids.array_xd.into(),
							arguments: vec![
								ident.clone(),
								Expression::new(
									db,
									&self.model,
									origin,
									ArrayComprehension {
										generators: vec![Generator::Iterator {
											declarations: vec![idx],
											collection: ident,
											where_clause: None,
										}],
										indices: None,
										template: Box::new(template),
									},
								),
							],
						},
					)
				}
				TyData::Tuple(_, _) | TyData::Record(_, _) => {
					// Create tuple literal then erase optionality on fields
					let fields = top_down_ty
						.fields(db.upcast())
						.unwrap()
						.into_iter()
						.zip(bottom_up_ty.fields(db.upcast()).unwrap())
						.enumerate()
						.map(|(i, (t, b))| {
							self.erase_opt(
								db,
								t,
								b,
								Expression::new(
									db,
									&self.model,
									origin,
									TupleAccess {
										tuple: Box::new(ident.clone()),
										field: IntegerLiteral(i as i64 + 1),
									},
								),
							)
						})
						.collect();
					Expression::new(db, &self.model, origin, TupleLiteral(fields))
				}
				_ => unreachable!(),
			};

			if let Some(decl) = decl {
				return Expression::new(
					db,
					&self.model,
					origin,
					Let {
						items: vec![LetItem::Declaration(decl)],
						in_expression: Box::new(erased),
					},
				);
			} else {
				return erased;
			}
		}

		// No need to do anything
		e
	}

	fn enum_ctor(
		&self,
		db: &dyn Thir,
		origin: Origin,
		prev_cards: Vec<Expression<Dst>>,
		f: &Function<Dst>,
	) -> Expression<Dst> {
		Expression::new(
			db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.mzn_enum_constructor.into(),
				arguments: vec![
					Expression::new(db, &self.model, origin, ArrayLiteral(prev_cards)),
					Expression::new(
						db,
						&self.model,
						origin,
						ArrayLiteral(
							f.parameters()
								.iter()
								.map(|p| {
									let arg = match &**self.model[*p].domain() {
										DomainData::Bounded(e) => (**e).clone(),
										_ => unreachable!(),
									};
									Expression::new(
										db,
										&self.model,
										origin,
										TupleLiteral(vec![
											arg,
											Expression::new(db, &self.model, origin, *p),
										]),
									)
								})
								.collect(),
						),
					),
				],
			},
		)
	}

	fn enum_dtor(
		&mut self,
		db: &dyn Thir,
		origin: Origin,
		prev_cards: Vec<Expression<Dst>>,
		f: &Function<Dst>,
		arg_count: usize,
	) -> Expression<Dst> {
		let inner = Expression::new(
			db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.mzn_enum_destructor.into(),
				arguments: vec![
					Expression::new(db, &self.model, origin, ArrayLiteral(prev_cards)),
					Expression::new(
						db,
						&self.model,
						origin,
						ArrayLiteral(
							f.parameters()
								.iter()
								.map(|p| match &**self.model[*p].domain() {
									DomainData::Bounded(e) => (**e).clone(),
									_ => unreachable!(),
								})
								.collect(),
						),
					),
					Expression::new(db, &self.model, origin, f.parameter(0)),
				],
			},
		);
		let d = self.model.add_declaration(Item::new(
			Declaration::new(false, Domain::unbounded(db, origin, inner.ty())),
			origin,
		));

		let range = 1i64..=(arg_count as i64);

		Expression::new(
			db,
			&self.model,
			origin,
			Let {
				items: vec![LetItem::Declaration(d)],
				in_expression: Box::new(Expression::new(
					db,
					&self.model,
					origin,
					TupleLiteral(
						range
							.map(|i| {
								Expression::new(
									db,
									&self.model,
									origin,
									ArrayAccess {
										collection: Box::new(Expression::new(
											db,
											&self.model,
											origin,
											d,
										)),
										indices: Box::new(Expression::new(
											db,
											&self.model,
											origin,
											IntegerLiteral(i),
										)),
									},
								)
							})
							.collect(),
					),
				)),
			},
		)
	}

	fn erase_enum(&mut self, db: &dyn Thir, model: &Model<Src>, idx: EnumerationId<Src>) {
		let enumeration = &model[idx];
		let origin = enumeration.origin();
		let enum_ty = Ty::par_enum(db.upcast(), enumeration.enum_type());
		let mut declaration = Declaration::new(
			true,
			Domain::set(
				db,
				origin,
				VarType::Par,
				OptType::NonOpt,
				Domain::unbounded(db, origin, Ty::par_set(db.upcast(), enum_ty).unwrap()),
			),
		);
		declaration.annotations_mut().extend(
			enumeration
				.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
		let defining_set = self.model.add_declaration(Item::new(declaration, origin));
		if let Some(definition) = enumeration.definition() {
			let mut prev_cards = Vec::new();
			let mut atom_card = 1i64;
			let mut members = Vec::with_capacity(definition.len());
			for constructor in definition {
				if let Some(params) = &constructor.parameters {
					let name = if let Some(ident) = constructor.name {
						FunctionName::new(ident)
					} else {
						FunctionName::anonymous()
					};
					let args = params
						.iter()
						.map(|p| match &**model[*p].domain() {
							DomainData::Bounded(e) => self.fold_expression(db, model, e),
							_ => unreachable!(),
						})
						.collect::<Vec<_>>();

					// Create constructor functions
					let mut par_ctor = Function::new(name, Domain::unbounded(db, origin, enum_ty));
					par_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Par, OptType::NonOpt, arg.clone()),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					par_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &par_ctor));

					let mut var_ctor = Function::new(
						name,
						Domain::unbounded(db, origin, EnumConstructorKind::Var.lift(db, enum_ty)),
					);
					var_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Var, OptType::NonOpt, arg.clone()),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					var_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &var_ctor));

					let mut opt_ctor = Function::new(
						name,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							[
								Domain::unbounded(db, origin, self.tys.par_bool),
								Domain::unbounded(db, origin, enum_ty),
							],
						),
					);
					opt_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::tuple(
								db,
								origin,
								OptType::NonOpt,
								[
									Domain::unbounded(db, origin, self.tys.par_bool),
									Domain::bounded(
										db,
										origin,
										VarType::Par,
										OptType::NonOpt,
										arg.clone(),
									),
								],
							),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					opt_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &opt_ctor));

					let mut var_opt_ctor = Function::new(
						name,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							[
								Domain::unbounded(db, origin, self.tys.var_bool),
								Domain::unbounded(
									db,
									origin,
									enum_ty.with_inst(db.upcast(), VarType::Var).unwrap(),
								),
							],
						),
					);
					var_opt_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::tuple(
								db,
								origin,
								OptType::NonOpt,
								[
									Domain::unbounded(db, origin, self.tys.var_bool),
									Domain::bounded(
										db,
										origin,
										VarType::Var,
										OptType::NonOpt,
										arg.clone(),
									),
								],
							),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					var_opt_ctor.set_body(self.enum_ctor(
						db,
						origin,
						prev_cards.clone(),
						&var_opt_ctor,
					));

					let mut set_ctor = Function::new(
						name,
						Domain::set(
							db,
							origin,
							VarType::Par,
							OptType::NonOpt,
							Domain::unbounded(db, origin, enum_ty),
						),
					);
					set_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::set(
								db,
								origin,
								VarType::Par,
								OptType::NonOpt,
								Domain::bounded(
									db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									arg.clone(),
								),
							),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));

					let mut var_set_ctor = Function::new(
						name,
						Domain::set(
							db,
							origin,
							VarType::Var,
							OptType::NonOpt,
							Domain::unbounded(db, origin, enum_ty),
						),
					);
					var_set_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::set(
								db,
								origin,
								VarType::Var,
								OptType::NonOpt,
								Domain::bounded(
									db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									arg.clone(),
								),
							),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));

					let constructors = EnumConstructorFunctions {
						par: self.model.add_function(Item::new(par_ctor, origin)),
						var: self.model.add_function(Item::new(var_ctor, origin)),
						opt: self.model.add_function(Item::new(opt_ctor, origin)),
						var_opt: self.model.add_function(Item::new(var_opt_ctor, origin)),
						set: self.model.add_function(Item::new(set_ctor, origin)),
						var_set: self.model.add_function(Item::new(var_set_ctor, origin)),
					};

					// Create destructor functions
					let inverse = name.inversed(db);
					let inverse_dom = Expression::new(db, &self.model, origin, defining_set);
					let tys = params
						.iter()
						.map(|p| model[*p].domain().ty())
						.collect::<Vec<_>>();

					let mut par_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| Domain::unbounded(db, origin, *ty)),
						),
					);
					par_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::bounded(
								db,
								origin,
								VarType::Par,
								OptType::NonOpt,
								inverse_dom.clone(),
							),
						),
						origin,
					)));
					par_dtor.set_body(self.enum_dtor(
						db,
						origin,
						prev_cards.clone(),
						&par_dtor,
						params.len(),
					));

					let mut var_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| {
								Domain::unbounded(
									db,
									origin,
									ty.with_inst(db.upcast(), VarType::Var).unwrap(),
								)
							}),
						),
					);
					var_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::bounded(
								db,
								origin,
								VarType::Var,
								OptType::NonOpt,
								inverse_dom.clone(),
							),
						),
						origin,
					)));
					var_dtor.set_body(self.enum_dtor(
						db,
						origin,
						prev_cards.clone(),
						&var_dtor,
						params.len(),
					));

					let mut opt_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| {
								Domain::unbounded(
									db,
									origin,
									ty.with_opt(db.upcast(), OptType::Opt),
								)
							}),
						),
					);
					opt_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::bounded(
								db,
								origin,
								VarType::Par,
								OptType::Opt,
								inverse_dom.clone(),
							),
						),
						origin,
					)));
					opt_dtor.set_body(self.enum_dtor(
						db,
						origin,
						prev_cards.clone(),
						&opt_dtor,
						params.len(),
					));

					let mut var_opt_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| {
								Domain::unbounded(
									db,
									origin,
									ty.with_inst(db.upcast(), VarType::Var)
										.unwrap()
										.with_opt(db.upcast(), OptType::Opt),
								)
							}),
						),
					);
					var_opt_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::bounded(
								db,
								origin,
								VarType::Var,
								OptType::Opt,
								inverse_dom.clone(),
							),
						),
						origin,
					)));
					var_opt_dtor.set_body(self.enum_dtor(
						db,
						origin,
						prev_cards.clone(),
						&var_opt_dtor,
						params.len(),
					));

					let mut set_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| {
								Domain::set(
									db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									Domain::unbounded(db, origin, *ty),
								)
							}),
						),
					);
					set_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::set(
								db,
								origin,
								VarType::Par,
								OptType::NonOpt,
								Domain::bounded(
									db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									inverse_dom.clone(),
								),
							),
						),
						origin,
					)));

					let mut var_set_dtor = Function::new(
						inverse,
						Domain::tuple(
							db,
							origin,
							OptType::NonOpt,
							tys.iter().map(|ty| {
								Domain::set(
									db,
									origin,
									VarType::Var,
									OptType::NonOpt,
									Domain::unbounded(db, origin, *ty),
								)
							}),
						),
					);
					var_set_dtor.add_parameter(self.model.add_declaration(Item::new(
						Declaration::new(
							false,
							Domain::set(
								db,
								origin,
								VarType::Var,
								OptType::NonOpt,
								Domain::bounded(
									db,
									origin,
									VarType::Par,
									OptType::NonOpt,
									inverse_dom.clone(),
								),
							),
						),
						origin,
					)));

					let destructors = EnumConstructorFunctions {
						par: self.model.add_function(Item::new(par_dtor, origin)),
						var: self.model.add_function(Item::new(var_dtor, origin)),
						opt: self.model.add_function(Item::new(opt_dtor, origin)),
						var_opt: self.model.add_function(Item::new(var_opt_dtor, origin)),
						set: self.model.add_function(Item::new(set_dtor, origin)),
						var_set: self.model.add_function(Item::new(var_set_dtor, origin)),
					};

					// Update total cardinality
					let constructor_card = if args.len() == 1 {
						Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.card.into(),
								arguments: vec![args[0].clone()],
							},
						)
					} else {
						// Product constructor
						Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.product.into(),
								arguments: args
									.iter()
									.map(|arg| {
										Expression::new(
											db,
											&self.model,
											origin,
											LookupCall {
												function: self.ids.card.into(),
												arguments: vec![arg.clone()],
											},
										)
									})
									.collect(),
							},
						)
					};
					prev_cards.push(constructor_card);
					members.push(ErasedEnumMember::Constructor {
						constructors,
						destructors,
					});
					atom_card = 1;
				} else {
					let mut atom = Declaration::new(true, Domain::unbounded(db, origin, enum_ty));
					let card_expression =
						Expression::new(db, &self.model, origin, IntegerLiteral(atom_card));
					if prev_cards.is_empty() {
						// Value is cardinality of previous parts, plus current value
						let value = Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.sum.into(),
								arguments: prev_cards
									.iter()
									.cloned()
									.chain([card_expression])
									.collect(),
							},
						);
						atom.set_definition(value);
					} else {
						// No previous parts, so value is a just a literal
						atom.set_definition(card_expression);
					}
					let idx = self.model.add_declaration(Item::new(atom, origin));
					members.push(ErasedEnumMember::Atomic(idx));
					atom_card += 1;
				}
			}
		} else {
			self.enum_map.insert(
				idx,
				ErasedEnum {
					defining_set,
					members: Vec::new(),
				},
			);
		}
	}
}

/// Erase types which are not present in MicroZinc
pub fn type_erase(db: &dyn Thir, model: &Model) -> Model {
	let mut top_down_types = TopDownTyper {
		db,
		types: RefMap::default(),
	};
	top_down_types.visit_model(model);
	let mut c = TypeEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
		tys: db.type_registry(),
		enum_map: ArenaMap::with_capacity(model.enumerations_len() as usize),
		top_down_types,
		needs_opt_erase: FxHashMap::default(),
	};
	c.add_model(db, model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check_no_stdlib;

	use super::type_erase;

	#[test]
	fn test_record_type_erasure() {
		check_no_stdlib(
			type_erase,
			r#"
                record(int: foo, float: bar): x = (foo: 1, bar: 2.5);
            "#,
			expect!([r#"
    tuple(int, float): x = (1, 2.5);
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_option_type_erasure() {
		check_no_stdlib(
			type_erase,
			r#"
                opt int: x = 2;
				opt bool: y = <>;
				var opt {1, 2, 3}: a;
				opt int: b = if true then 1 else <> endif;
				array [int] of opt int: c = [1, <>];
				tuple(int, opt int): d;
				tuple(opt int, opt int): e = d;
				function opt int: foo(opt int: x) = 1;
				any: f = foo(1);
            "#,
			expect!([r#"
    tuple(bool, int): x = (true, 2);
    tuple(bool, bool): y = (false, ⊥);
    tuple(var bool, var {1, 2, 3}): a;
    tuple(bool, int): b = if true then (true, 1) else (false, ⊥) endif;
    array [int] of tuple(bool, int): c = [(true, 1), (false, ⊥)];
    tuple(int, tuple(bool, int)): d;
    tuple(tuple(bool, int), tuple(bool, int)): e = ((true, d.1), d.2);
    function tuple(bool, int): foo(tuple(bool, int): x) = (true, 1);
    tuple(bool, int): f = foo((true, 1));
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_option_type_erasure_2() {
		check_no_stdlib(
			type_erase,
			r#"
			function array [int] of tuple(bool, var int): arrayXd(array [int] of var int: a, array [int] of tuple(bool, var int): b);
			function int: foo(array [int] of var opt int: x);
			function set of int: bar(int: a, int: b);
			function var int: qux(array [int] of var int: x) = let {
				var bar(foo(x), foo(x)): r;
			} in r;
			"#,
			expect!([r#"
    function array [int] of tuple(bool, var int): arrayXd(array [int] of var int: a, array [int] of tuple(bool, var int): b);
    function int: foo(array [int] of tuple(var bool, var int): x);
    function set of int: bar(int: a, int: b);
    function var int: qux(array [int] of var int: x) = let {
      var bar(foo(arrayXd(x, [(true, _DECL_7) | _DECL_7 in x])), foo(arrayXd(x, [(true, _DECL_8) | _DECL_8 in x]))): r;
    } in r;
    solve satisfy;
"#]),
		);
	}
}
