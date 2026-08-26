//! Erase enums, transforming them into ints
//! - Generate enum value definitions and constructor functions
//! - Replace enum types with integer types
//!
//! Since this transform generates optional types and var set comprehensions, it must be run before
//! option type erasure and comprehension desugaring comprehensions.

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::{IntegerLiteral, StringLiteral, VarType, constants::IdentifierRegistry};
use shackle_ty::{EnumRef, registry::TypeRegistry};
use shackle_utils::{arena::ArenaMap, maybe_grow_stack};

use super::top_down_type::add_coercion;
use crate::{
	ArrayLiteral, Callable, Db, Declaration, DeclarationId, Domain, DomainData, EnumMemberId,
	EnumerationId, EnumerationItem, Expression, ExpressionData, Function, FunctionId, FunctionName,
	Item, ItemId, Let, LetItem, LookupCall, Marker, Model, ResolvedIdentifier, TupleLiteral,
	traverse::{
		Folder, ReplacementMap, add_function, add_item, fold_domain, fold_expression,
		fold_function, fold_function_body, fold_identifier,
	},
};
struct EnumEraser<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
	enum_definitions: Vec<Expression<'db, Dst>>,
	identifier_replacement: FxHashMap<ResolvedIdentifier<'db, Src>, DeclarationId<'db, Dst>>,
	mzn_enum_for_item: ArenaMap<EnumerationItem<'db, Src>, DeclarationId<'db, Dst>>,
	enum_id_for_ty: FxHashMap<EnumRef<'db>, i64>,
	defining_set_for_ty: FxHashMap<EnumRef<'db>, DeclarationId<'db, Dst>>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for EnumEraser<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_item(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, item: ItemId<'db, Src>) {
		if let ItemId::Enumeration(e) = item {
			// Erase enum items
			self.erase_enum(db, model, e);
		} else {
			add_item(self, db, model, item);
		}
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		if model[f].name() == self.ids.functions.enum2int
			|| model[f].name() == self.ids.functions.to_enum_internal
			|| model[f].name() == self.ids.functions.index2int
			|| model[f].name() == self.ids.functions.enum_of
		{
			// Remove unnecessary functions
			return;
		}
		let _ = add_function(self, db, model, f);
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		if model[f].name() == self.ids.functions.enum2int
			|| model[f].name() == self.ids.functions.to_enum_internal
			|| model[f].name() == self.ids.functions.index2int
			|| model[f].name() == self.ids.functions.enum_of
		{
			// Remove unnecessary functions
			return;
		}
		fold_function_body(self, db, model, f);
	}

	fn fold_function(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: &Function<'db, Src>,
	) -> Function<'db, Dst> {
		let mut folded = fold_function(self, db, model, f);
		if f.name() == self.ids.functions.show && f.body().is_none() {
			let p = &model[f.parameter(0)];
			if let Some(enum_ty) = p.ty().enum_ty(db) {
				let origin = p.origin();
				let arg = Expression::new(db, &self.model, origin, folded.parameter(0));
				let index = self.enum_id_for_ty[&enum_ty];
				let enum_id = Expression::new(db, &self.model, origin, IntegerLiteral(index));
				let enums = Expression::new(
					db,
					&self.model,
					origin,
					ArrayLiteral(self.enum_definitions[..index as usize].to_vec()),
				);
				let body = Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.mzn_show_enum.into(),
						arguments: vec![enums, enum_id, arg],
					},
				);
				folded.set_body(body);
			}
		}
		if f.name() == self.ids.functions.show_json && f.body().is_none() {
			let p = &model[f.parameter(0)];
			if let Some(enum_ty) = p.ty().enum_ty(db) {
				let origin = p.origin();
				let arg = Expression::new(db, &self.model, origin, folded.parameter(0));
				let index = self.enum_id_for_ty[&enum_ty];
				let enum_id = Expression::new(db, &self.model, origin, IntegerLiteral(index));
				let enums = Expression::new(
					db,
					&self.model,
					origin,
					ArrayLiteral(self.enum_definitions[..index as usize].to_vec()),
				);
				let body = Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.mzn_show_enum.into(),
						arguments: vec![enums, enum_id, arg],
					},
				);
				folded.set_body(body);
			}
		}
		folded
	}

	fn fold_identifier(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		identifier: &ResolvedIdentifier<'db, Src>,
	) -> ResolvedIdentifier<'db, Dst> {
		if let Some(i) = self.identifier_replacement.get(identifier) {
			(*i).into()
		} else {
			fold_identifier(self, db, model, identifier)
		}
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		expression: &Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			if let ExpressionData::Call(c) = &**expression {
				match &c.function {
					Callable::EnumConstructor(e) => {
						log::debug!(
							"Erasing constructor at {}",
							expression.origin().pretty_print(db)
						);
						let mzn_enum = self.mzn_enum_for_item[e.enumeration_id()];
						let member_id = e.member_index() as i64 + 1;
						let origin = model[e.enumeration_id()].origin();
						let arguments = if model[*e].parameters.is_none() {
							vec![
								Expression::new(db, &self.model, origin, mzn_enum),
								Expression::new(db, &self.model, origin, IntegerLiteral(member_id)),
							]
						} else {
							let al = ArrayLiteral(
								c.arguments
									.iter()
									.map(|arg| self.fold_expression(db, model, arg))
									.collect(),
							);
							vec![
								Expression::new(db, &self.model, origin, mzn_enum),
								Expression::new(db, &self.model, origin, IntegerLiteral(member_id)),
								Expression::new(db, &self.model, origin, al),
							]
						};
						return Expression::new(
							db,
							&self.model,
							expression.origin(),
							LookupCall {
								function: self.ids.functions.mzn_construct_enum.into(),
								arguments,
							},
						);
					}
					Callable::EnumDestructor(e) => {
						log::debug!(
							"Erasing destructor at {}",
							expression.origin().pretty_print(db)
						);
						let mzn_enum = self.mzn_enum_for_item[e.enumeration_id()];
						let member_id = e.member_index() as i64 + 1;
						let origin = model[e.enumeration_id()].origin();
						let arguments = vec![
							Expression::new(db, &self.model, origin, mzn_enum),
							Expression::new(db, &self.model, origin, IntegerLiteral(member_id)),
							self.fold_expression(db, model, &c.arguments[0]),
						];
						// mzn_destruct_enum returns a list of values, which we need to convert to a tuple
						let array = Expression::new(
							db,
							&self.model,
							origin,
							LookupCall {
								function: self.ids.functions.mzn_destruct_enum.into(),
								arguments,
							},
						);

						let len = model[e.enumeration_id()].definition().as_ref().unwrap()
							[e.member_index() as usize]
							.parameters
							.as_ref()
							.unwrap()
							.len();
						if len == 1 {
							return Expression::new(
								db,
								&self.model,
								expression.origin(),
								LookupCall {
									function: self.ids.builtins.mzn_element_internal.into(),
									arguments: vec![
										array,
										Expression::new(db, &self.model, origin, IntegerLiteral(1)),
									],
								},
							);
						}

						let array_decl = Declaration::from_expression(db, false, array);
						let array_decl_idx =
							self.model.add_declaration(Item::new(array_decl, origin));
						let array_decl_ident =
							Expression::new(db, &self.model, origin, array_decl_idx);
						let in_expression = Expression::new(
							db,
							&self.model,
							origin,
							ArrayLiteral(
								(1..len + 1)
									.map(|i| {
										Expression::new(
											db,
											&self.model,
											origin,
											LookupCall {
												function: self
													.ids
													.builtins
													.mzn_element_internal
													.into(),
												arguments: vec![
													array_decl_ident.clone(),
													Expression::new(
														db,
														&self.model,
														origin,
														IntegerLiteral(i as i64),
													),
												],
											},
										)
									})
									.collect::<Vec<_>>(),
							),
						);

						return Expression::new(
							db,
							&self.model,
							expression.origin(),
							Let {
								items: vec![LetItem::Declaration(array_decl_idx)],
								in_expression: Box::new(in_expression),
							},
						);
					}
					Callable::Function(f) => {
						if model[*f].name() == self.ids.functions.enum2int
							|| model[*f].name() == self.ids.functions.index2int
						{
							log::debug!(
								"Erasing enum to integer coercion at {}",
								expression.origin().pretty_print(db)
							);
							return self.fold_expression(db, model, &c.arguments[0]);
						} else if model[*f].name() == self.ids.functions.to_enum_internal {
							log::debug!(
								"Erasing integer to enum coercion at {}",
								expression.origin().pretty_print(db)
							);
							return self.fold_expression(db, model, &c.arguments[1]);
						} else if model[*f].name() == self.ids.functions.enum_of {
							log::debug!(
								"Erasing enum_of call at {}",
								expression.origin().pretty_print(db)
							);
							if let Some(e) = c.arguments[0].ty().enum_ty(db) {
								return Expression::new(
									db,
									&self.model,
									expression.origin(),
									self.defining_set_for_ty[&e],
								);
							}
							return Expression::new(
								db,
								&self.model,
								expression.origin(),
								LookupCall {
									function: self.ids.functions.mzn_infinite_range.into(),
									arguments: vec![],
								},
							);
						}
					}
					_ => (),
				}
			}
			fold_expression(self, db, model, expression)
		})
	}

	fn fold_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
	) -> Domain<'db, Dst> {
		maybe_grow_stack(|| {
			let mut folded = fold_domain(self, db, model, domain);
			if folded.ty().enum_ty(db).is_some() {
				log::debug!(
					"Erasing enum domain to int at {}",
					domain.origin().pretty_print(db)
				);
				// Erase enum types into ints
				let ty = if let Some(VarType::Var) = folded.ty().inst(db) {
					self.tys.var_int
				} else {
					self.tys.par_int
				};
				if let Some(opt) = folded.ty().opt(db) {
					folded.set_ty_unchecked(ty.with_opt(db, opt));
				} else {
					folded.set_ty_unchecked(ty);
				}
			}
			folded
		})
	}
}

impl<'db, Src: Marker, Dst: Marker> EnumEraser<'db, Dst, Src> {
	fn erase_enum(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		idx: EnumerationId<'db, Src>,
	) {
		let enumeration = &model[idx];
		let origin = enumeration.origin();
		let enum_id = self.enum_definitions.len() as i64 + 1;
		let _ = self.enum_id_for_ty.insert(enumeration.enum_type(), enum_id);

		let enum_rhs = if let Some(definition) = enumeration.definition() {
			let mut enum_def = Vec::with_capacity(definition.len());
			for constructor in definition.iter() {
				if let Some(params) = &constructor.parameters {
					let name = if let Some(ident) = constructor.name {
						FunctionName::new(ident)
					} else {
						FunctionName::anonymous()
					};
					let ctor_params = params
						.iter()
						.map(|p| {
							let tl = vec![
								Expression::new(
									db,
									&self.model,
									origin,
									IntegerLiteral(
										model[*p]
											.ty()
											.enum_ty(db)
											.map(|e| self.enum_id_for_ty[&e])
											.unwrap_or(0),
									),
								),
								match &**model[*p].domain() {
									DomainData::Bounded(e) => self.fold_expression(db, model, e),
									_ => unreachable!(),
								},
							];
							Expression::new(db, &self.model, origin, TupleLiteral(tl))
						})
						.collect();
					enum_def.push(Expression::new(
						db,
						&self.model,
						origin,
						TupleLiteral(vec![
							Expression::new(
								db,
								&self.model,
								origin,
								StringLiteral::new(db, name.pretty_print(db)),
							),
							Expression::new(db, &self.model, origin, ArrayLiteral(ctor_params)),
						]),
					));
				} else {
					let name = constructor.name.unwrap();
					let empty_array =
						Expression::new(db, &self.model, origin, ArrayLiteral(vec![]));
					let ctor_params = add_coercion(
						db,
						&mut self.model,
						self.tys.array_of_tuple_int_set_of_int,
						empty_array,
					);
					enum_def.push(Expression::new(
						db,
						&self.model,
						origin,
						TupleLiteral(vec![
							Expression::new(db, &self.model, origin, StringLiteral::from(name)),
							ctor_params,
						]),
					));
				}
			}
			Expression::new(db, &self.model, origin, ArrayLiteral(enum_def))
		} else {
			// Create declaration for enum data input
			let mut enum_declaration = Declaration::new(
				true,
				Domain::unbounded(db, origin, self.tys.array_of_string),
			);
			enum_declaration.set_name(enumeration.enum_type().name().into());
			let enum_declaration_idx = self
				.model
				.add_declaration(Item::new(enum_declaration, origin));
			Expression::new(db, &self.model, origin, enum_declaration_idx)
		};

		// Create declaration to hold definition of enum
		//   MznEnum: mzn_enum = mzn_get_enum(enum_rhs);
		let mut mzn_enum = Declaration::new(true, Domain::unbounded(db, origin, self.tys.mzn_enum));
		mzn_enum.set_definition(Expression::new(
			db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.mzn_get_enum.into(),
				arguments: vec![enum_rhs],
			},
		));
		let mzn_enum_idx = self.model.add_declaration(Item::new(mzn_enum, origin));
		self.mzn_enum_for_item.insert(idx, mzn_enum_idx);
		self.enum_definitions
			.push(Expression::new(db, &self.model, origin, mzn_enum_idx));

		// Create declaration for enum defining set
		//   set of int: Foo = mzn_defining_set(mzn_enum);
		let mut defining_set_declaration =
			Declaration::new(true, Domain::unbounded(db, origin, self.tys.set_of_int));
		defining_set_declaration.annotations_mut().extend(
			enumeration
				.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
		defining_set_declaration.set_definition(Expression::new(
			db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.mzn_defining_set.into(),
				arguments: vec![Expression::new(db, &self.model, origin, mzn_enum_idx)],
			},
		));
		let defining_set = self
			.model
			.add_declaration(Item::new(defining_set_declaration, origin));
		let _ = self
			.identifier_replacement
			.insert(ResolvedIdentifier::Enumeration(idx), defining_set);
		let _ = self
			.defining_set_for_ty
			.insert(enumeration.enum_type(), defining_set);

		// Create declarations for atoms
		//   set of int: A = mzn_construct_enum(mzn_enum, i);
		if let Some(definition) = enumeration.definition() {
			for (i, constructor) in definition.iter().enumerate() {
				if constructor.parameters.is_some() {
					continue;
				}
				let mut atom =
					Declaration::new(true, Domain::unbounded(db, origin, self.tys.par_int));
				atom.set_name(constructor.name.unwrap());
				atom.set_definition(Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.mzn_construct_enum.into(),
						arguments: vec![
							Expression::new(db, &self.model, origin, mzn_enum_idx),
							Expression::new(db, &self.model, origin, IntegerLiteral(i as i64 + 1)),
						],
					},
				));
				let member_idx = self.model.add_declaration(Item::new(atom, origin));

				let _ = self.identifier_replacement.insert(
					ResolvedIdentifier::EnumerationMember(EnumMemberId::new(idx, i as u32)),
					member_idx,
				);
			}
		}
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_enum<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Erasing enums into ints");
	let mut c = EnumEraser {
		model: Model::with_capacities(&model.item_counts()),
		replacement_map: ReplacementMap::default(),
		ids: IdentifierRegistry::lookup(db),
		tys: TypeRegistry::lookup(db),
		enum_definitions: Vec::with_capacity(model.enumerations_len() as usize),
		mzn_enum_for_item: ArenaMap::with_capacity(model.enumerations_len()),
		enum_id_for_ty: FxHashMap::default(),
		identifier_replacement: FxHashMap::default(),
		defining_set_for_ty: FxHashMap::default(),
	};
	c.add_model(db, &model);
	Ok(c.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::erase_enum;
	use crate::transform::{tests::check, transformer, type_specialise::type_specialise};

	#[test]
	fn test_enum_type_erasure() {
		check(
			transformer(vec![type_specialise, erase_enum]),
			r#"
                enum Foo = {A, B, C} ++ D(Bar);
				enum Bar = {E, F};
				any: x = B;
				any: y = D(E);
            "#,
			expect!([r#"
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_1 = mzn_get_enum([("E", let {
      array [int] of tuple(int, set of int): _DECL_2 = [];
    } in _DECL_2), ("F", let {
      array [int] of tuple(int, set of int): _DECL_3 = [];
    } in _DECL_3)]);
    set of int: _DECL_4 = mzn_defining_set(_DECL_1);
    int: E = mzn_construct_enum(_DECL_1, 1);
    int: F = mzn_construct_enum(_DECL_1, 2);
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_5 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_6 = [];
    } in _DECL_6), ("B", let {
      array [int] of tuple(int, set of int): _DECL_7 = [];
    } in _DECL_7), ("C", let {
      array [int] of tuple(int, set of int): _DECL_8 = [];
    } in _DECL_8), ("D", [(1, _DECL_4)])]);
    set of int: _DECL_9 = mzn_defining_set(_DECL_5);
    int: A = mzn_construct_enum(_DECL_5, 1);
    int: B = mzn_construct_enum(_DECL_5, 2);
    int: C = mzn_construct_enum(_DECL_5, 3);
    int: x = B;
    int: y = mzn_construct_enum(_DECL_5, 4, [E]);
"#]),
		);
	}

	#[test]
	fn test_enum_show() {
		check(
			transformer(vec![type_specialise, erase_enum]),
			r#"
                enum Foo = {A, B, C} ++ D(Bar);
				enum Bar = {E, F};
				function string: show(Foo: x);
				function string: show(Bar: x);
            "#,
			expect!([r#"
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_1 = mzn_get_enum([("E", let {
      array [int] of tuple(int, set of int): _DECL_2 = [];
    } in _DECL_2), ("F", let {
      array [int] of tuple(int, set of int): _DECL_3 = [];
    } in _DECL_3)]);
    set of int: _DECL_4 = mzn_defining_set(_DECL_1);
    int: E = mzn_construct_enum(_DECL_1, 1);
    int: F = mzn_construct_enum(_DECL_1, 2);
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_5 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_6 = [];
    } in _DECL_6), ("B", let {
      array [int] of tuple(int, set of int): _DECL_7 = [];
    } in _DECL_7), ("C", let {
      array [int] of tuple(int, set of int): _DECL_8 = [];
    } in _DECL_8), ("D", [(1, _DECL_4)])]);
    set of int: _DECL_9 = mzn_defining_set(_DECL_5);
    int: A = mzn_construct_enum(_DECL_5, 1);
    int: B = mzn_construct_enum(_DECL_5, 2);
    int: C = mzn_construct_enum(_DECL_5, 3);
    function string: show(_DECL_9: x) = mzn_show_enum([_DECL_1, _DECL_5], 2, x);
    function string: show(_DECL_4: x) = mzn_show_enum([_DECL_1], 1, x);
"#]),
		);
	}

	#[test]
	fn test_erase_to_enum() {
		check(
			transformer(vec![type_specialise, erase_enum]),
			r#"
                enum Foo = {A};
				Foo: x = to_enum(Foo, 1);
				int: y = enum2int(A);
            "#,
			expect!([r#"
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_1 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_2 = [];
    } in _DECL_2)]);
    set of int: _DECL_3 = mzn_defining_set(_DECL_1);
    int: A = mzn_construct_enum(_DECL_1, 1);
    _DECL_3: x = to_enum(_DECL_3, 1);
    int: y = A;
"#]),
		);
	}

	#[test]
	fn test_erase_enum_of() {
		check(
			transformer(vec![type_specialise, erase_enum]),
			r#"
                enum Foo = {A};
				any: x = enum_of(A);
            "#,
			expect!([r#"
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_1 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_2 = [];
    } in _DECL_2)]);
    set of int: _DECL_3 = mzn_defining_set(_DECL_1);
    int: A = mzn_construct_enum(_DECL_1, 1);
    set of int: x = _DECL_3;
"#]),
		);
	}
}
