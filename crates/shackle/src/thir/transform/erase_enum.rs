//! Erase enums, transforming them into ints
//! - Generate enum value definitions and constructor functions
//! - Replace enum types with integer types
//!
//! Since this transform generates optional types and var set comprehensions, it must be run before
//! option type erasure and comprehension desugaring comprehensions.

use crate::{
	arena::ArenaMap,
	constants::{IdentifierRegistry, TypeRegistry},
	hir::{IntegerLiteral, OptType, VarType},
	thir::{
		db::Thir,
		source::Origin,
		traverse::{add_item, fold_call, fold_domain, fold_identifier, Folder, ReplacementMap},
		ArrayAccess, ArrayLiteral, Call, Callable, Declaration, DeclarationId, Domain, DomainData,
		EnumConstructorKind, EnumerationId, EnumerationItem, Expression, Function, FunctionId,
		FunctionName, Generator, Item, ItemId, Let, LetItem, LookupCall, Marker, Model,
		ResolvedIdentifier, SetComprehension, TupleLiteral,
	},
};
use std::sync::Arc;

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

struct EnumEraser<Dst, Src = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	tys: Arc<TypeRegistry>,
	enum_map: ArenaMap<EnumerationItem<Src>, ErasedEnum<Dst>>,
}

impl<Dst: Marker, Src: Marker> Folder<Dst, Src> for EnumEraser<Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn add_item(&mut self, db: &dyn Thir, model: &Model<Src>, item: ItemId<Src>) {
		if let ItemId::Enumeration(e) = item {
			// Erase enum items
			self.erase_enum(db, model, e);
		} else {
			add_item(self, db, model, item);
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

	fn fold_domain(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
	) -> Domain<Dst> {
		let mut folded = fold_domain(self, db, model, domain);
		if folded.ty().enum_ty(db.upcast()).is_some() {
			// Erase enum types into ints
			if let Some(VarType::Var) = folded.ty().inst(db.upcast()) {
				folded.set_ty_unchecked(self.tys.var_int);
			} else {
				folded.set_ty_unchecked(self.tys.par_int);
			}
		}
		folded
	}
}

impl<Src: Marker, Dst: Marker> EnumEraser<Dst, Src> {
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
		let mut decl = Declaration::new(false, Domain::unbounded(db, origin, inner.ty()));
		decl.set_definition(inner);
		let d = self.model.add_declaration(Item::new(decl, origin));

		Expression::new(
			db,
			&self.model,
			origin,
			Let {
				items: vec![LetItem::Declaration(d)],
				in_expression: Box::new(if arg_count == 1 {
					Expression::new(
						db,
						&self.model,
						origin,
						ArrayAccess {
							collection: Box::new(Expression::new(db, &self.model, origin, d)),
							indices: Box::new(Expression::new(
								db,
								&self.model,
								origin,
								IntegerLiteral(1),
							)),
						},
					)
				} else {
					let range = 1i64..=(arg_count as i64);
					Expression::new(
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
					)
				}),
			},
		)
	}

	fn erase_enum(&mut self, db: &dyn Thir, model: &Model<Src>, idx: EnumerationId<Src>) {
		let enumeration = &model[idx];
		let origin = enumeration.origin();
		let mut declaration = Declaration::new(
			true,
			Domain::set(
				db,
				origin,
				VarType::Par,
				OptType::NonOpt,
				Domain::unbounded(db, origin, self.tys.par_int),
			),
		);
		declaration.set_name(enumeration.enum_type().name(db.upcast()).into());
		declaration.annotations_mut().extend(
			enumeration
				.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
		let defining_set = self.model.add_declaration(Item::new(declaration, origin));
		let members = if let Some(definition) = enumeration.definition() {
			let mut prev_cards = Vec::new();
			let mut atom_card = 1i64;
			let mut members = Vec::with_capacity(definition.len());
			let enum_dom = Expression::new(db, &self.model, origin, defining_set);
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
					let mut par_ctor = Function::new(
						name,
						Domain::bounded(
							db,
							origin,
							VarType::Par,
							OptType::NonOpt,
							enum_dom.clone(),
						),
					);
					par_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Par, OptType::NonOpt, arg.clone()),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					par_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &par_ctor));
					let par_ctor_idx = self.model.add_function(Item::new(par_ctor, origin));

					let mut var_ctor = Function::new(
						name,
						Domain::bounded(
							db,
							origin,
							VarType::Var,
							OptType::NonOpt,
							enum_dom.clone(),
						),
					);
					var_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Var, OptType::NonOpt, arg.clone()),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					var_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &var_ctor));
					let var_ctor_idx = self.model.add_function(Item::new(var_ctor, origin));

					let mut opt_ctor = Function::new(
						name,
						Domain::bounded(db, origin, VarType::Par, OptType::Opt, enum_dom.clone()),
					);
					opt_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Par, OptType::Opt, arg.clone()),
						);
						self.model.add_declaration(Item::new(d, origin))
					}));
					opt_ctor.set_body(self.enum_ctor(db, origin, prev_cards.clone(), &opt_ctor));

					let mut var_opt_ctor = Function::new(
						name,
						Domain::bounded(db, origin, VarType::Var, OptType::Opt, enum_dom.clone()),
					);
					var_opt_ctor.set_parameters(args.iter().map(|arg| {
						let d = Declaration::new(
							false,
							Domain::bounded(db, origin, VarType::Var, OptType::Opt, arg.clone()),
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
							Domain::bounded(
								db,
								origin,
								VarType::Par,
								OptType::NonOpt,
								enum_dom.clone(),
							),
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
					let decls = args
						.iter()
						.map(|_| {
							let d = Declaration::new(
								false,
								Domain::unbounded(db, origin, self.tys.par_int),
							);
							self.model.add_declaration(Item::new(d, origin))
						})
						.collect::<Vec<_>>();
					set_ctor.set_body(Expression::new(
						db,
						&self.model,
						origin,
						SetComprehension {
							generators: decls
								.iter()
								.zip(set_ctor.parameters())
								.map(|(d, c)| Generator::Iterator {
									declarations: vec![*d],
									collection: Expression::new(db, &self.model, origin, *c),
									where_clause: None,
								})
								.collect(),
							template: Box::new(Expression::new(
								db,
								&self.model,
								origin,
								Call {
									function: Callable::Function(par_ctor_idx),
									arguments: decls
										.iter()
										.map(|d| Expression::new(db, &self.model, origin, *d))
										.collect(),
								},
							)),
						},
					));

					let mut var_set_ctor = Function::new(
						name,
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
								enum_dom.clone(),
							),
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
					let decls = args
						.iter()
						.map(|_| {
							let d = Declaration::new(
								false,
								Domain::unbounded(db, origin, self.tys.par_int),
							);
							self.model.add_declaration(Item::new(d, origin))
						})
						.collect::<Vec<_>>();
					var_set_ctor.set_body(Expression::new(
						db,
						&self.model,
						origin,
						SetComprehension {
							generators: decls
								.iter()
								.zip(set_ctor.parameters())
								.map(|(d, c)| Generator::Iterator {
									declarations: vec![*d],
									collection: Expression::new(db, &self.model, origin, *c),
									where_clause: None,
								})
								.collect(),
							template: Box::new(Expression::new(
								db,
								&self.model,
								origin,
								Call {
									function: Callable::Function(par_ctor_idx),
									arguments: decls
										.iter()
										.map(|d| Expression::new(db, &self.model, origin, *d))
										.collect(),
								},
							)),
						},
					));

					let constructors = EnumConstructorFunctions {
						par: par_ctor_idx,
						var: var_ctor_idx,
						opt: self.model.add_function(Item::new(opt_ctor, origin)),
						var_opt: self.model.add_function(Item::new(var_opt_ctor, origin)),
						set: self.model.add_function(Item::new(set_ctor, origin)),
						var_set: self.model.add_function(Item::new(var_set_ctor, origin)),
					};

					// Create destructor functions
					let inverse = name.inversed(db);
					let inverse_dom = Expression::new(db, &self.model, origin, defining_set);
					let doms = params
						.iter()
						.map(|p| self.fold_domain(db, model, model[*p].domain()))
						.collect::<Vec<_>>();

					let mut par_dtor: Function<Dst> = Function::new(
						inverse,
						if doms.len() == 1 {
							doms[0].clone()
						} else {
							Domain::tuple(db, origin, OptType::NonOpt, doms.clone())
						},
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
							doms.iter().map(|dom| {
								let mut d = dom.clone();
								d.set_ty_unchecked(dom.ty().make_var(db.upcast()).unwrap());
								d
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
							doms.iter().map(|dom| {
								let mut d = dom.clone();
								d.set_ty_unchecked(d.ty().make_opt(db.upcast()));
								d
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
							doms.iter().map(|dom| {
								let mut d = dom.clone();
								d.set_ty_unchecked(
									dom.ty()
										.make_var(db.upcast())
										.unwrap()
										.make_opt(db.upcast()),
								);
								d
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
							doms.iter().map(|dom| {
								Domain::set(db, origin, VarType::Par, OptType::NonOpt, dom.clone())
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
							doms.iter().map(|dom| {
								Domain::set(db, origin, VarType::Var, OptType::NonOpt, dom.clone())
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
					let mut atom =
						Declaration::new(true, Domain::unbounded(db, origin, self.tys.par_int));
					if let Some(name) = constructor.name {
						atom.set_name(name);
					}
					let card_expression =
						Expression::new(db, &self.model, origin, IntegerLiteral(atom_card));
					if prev_cards.is_empty() {
						// No previous parts, so value is a just a literal
						atom.set_definition(card_expression);
					} else {
						// Value is cardinality of previous parts, plus current value
						let cards = Expression::new(
							db,
							&self.model,
							origin,
							ArrayLiteral(
								prev_cards
									.iter()
									.cloned()
									.chain([card_expression])
									.collect(),
							),
						);
						let value = Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.sum.into(),
								arguments: vec![cards],
							},
						);
						atom.set_definition(value);
					}
					let idx = self.model.add_declaration(Item::new(atom, origin));
					members.push(ErasedEnumMember::Atomic(idx));
					atom_card += 1;
				}
			}

			let max = if prev_cards.is_empty() {
				Expression::new(db, &self.model, origin, IntegerLiteral(atom_card - 1))
			} else if prev_cards.len() > 1 || atom_card > 1 {
				let cards = Expression::new(
					db,
					&self.model,
					origin,
					ArrayLiteral(
						prev_cards
							.iter()
							.cloned()
							.chain(if atom_card > 1 {
								Some(Expression::new(
									db,
									&self.model,
									origin,
									IntegerLiteral(atom_card - 1),
								))
							} else {
								None
							})
							.collect(),
					),
				);
				Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.sum.into(),
						arguments: vec![cards],
					},
				)
			} else {
				prev_cards[0].clone()
			};

			let rhs = Expression::new(
				db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.dot_dot.into(),
					arguments: vec![
						Expression::new(db, &self.model, origin, IntegerLiteral(1)),
						max,
					],
				},
			);
			self.model[defining_set].set_definition(rhs);
			members
		} else {
			Vec::new()
		};

		self.enum_map.insert(
			idx,
			ErasedEnum {
				defining_set,
				members,
			},
		);
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_enum(db: &dyn Thir, model: &Model) -> Model {
	let mut c = EnumEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
		tys: db.type_registry(),
		enum_map: ArenaMap::with_capacity(model.enumerations_len() as usize),
	};
	c.add_model(db, model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check;

	use super::erase_enum;

	#[test]
	fn test_enum_type_erasure() {
		check(
			erase_enum,
			r#"
                enum Foo = {A, B, C} ++ D(Bar);
				enum Bar = {E, F};
				any: x = B;
				any: y = D(E);
            "#,
			expect!([r#"
    set of int: Bar = '..'(1, 2);
    int: E = 1;
    int: F = 2;
    set of int: Foo = '..'(1, card(Bar));
    int: A = 1;
    int: B = 2;
    int: C = 3;
    function Foo: D(Bar: _DECL_2293) = mzn_enum_constructor([], [(Bar, _DECL_2293)]);
    function var Foo: D(var Bar: _DECL_2294) = mzn_enum_constructor([], [(Bar, _DECL_2294)]);
    function opt Foo: D(opt Bar: _DECL_2295) = mzn_enum_constructor([], [(Bar, _DECL_2295)]);
    function var opt Foo: D(var opt Bar: _DECL_2296) = mzn_enum_constructor([], [(Bar, _DECL_2296)]);
    function set of Foo: D(set of Bar: _DECL_2297) = {D(_DECL_2298) | _DECL_2298 in _DECL_2297};
    function var set of Foo: D(var set of Bar: _DECL_2299) = {D(_DECL_2300) | _DECL_2300 in _DECL_2297};
    function Bar: D⁻¹(Foo: _DECL_2301) = let {
      array [int] of int: _DECL_2302 = mzn_enum_destructor([], [Foo], _DECL_2301);
    } in (_DECL_2302)[1];
    function tuple(var Bar): D⁻¹(var Foo: _DECL_2303) = let {
      array [int] of var int: _DECL_2304 = mzn_enum_destructor([], [Foo], _DECL_2303);
    } in (_DECL_2304)[1];
    function tuple(opt Bar): D⁻¹(opt Foo: _DECL_2305) = let {
      array [int] of opt int: _DECL_2306 = mzn_enum_destructor([], [Foo], _DECL_2305);
    } in (_DECL_2306)[1];
    function tuple(var opt Bar): D⁻¹(var opt Foo: _DECL_2307) = let {
      array [int] of var opt int: _DECL_2308 = mzn_enum_destructor([], [Foo], _DECL_2307);
    } in (_DECL_2308)[1];
    function tuple(set of Bar): D⁻¹(set of Foo: _DECL_2309);
    function tuple(var set of Bar): D⁻¹(var set of Foo: _DECL_2310);
    int: x = B;
    int: y = D(E);
"#]),
		);
	}
}
