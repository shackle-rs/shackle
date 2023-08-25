//! Erase enums, transforming them into ints
//! - Generate enum value definitions and constructor functions
//! - Replace enum types with integer types
//!
//! Since this transform generates optional types and var set comprehensions, it must be run before
//! option type erasure and comprehension desugaring comprehensions.

use rustc_hash::FxHashMap;

use crate::{
	arena::ArenaMap,
	constants::{IdentifierRegistry, TypeRegistry},
	hir::{Identifier, IntegerLiteral, StringLiteral, VarType},
	thir::{
		db::Thir,
		traverse::{
			add_function, add_item, fold_call, fold_domain, fold_expression, fold_function,
			fold_function_body, fold_identifier, Folder, ReplacementMap,
		},
		ArrayLiteral, Call, Callable, Declaration, DeclarationId, Domain, DomainData, EnumMemberId,
		EnumerationId, EnumerationItem, Expression, ExpressionData, Function, FunctionId,
		FunctionName, Item, ItemId, LookupCall, Marker, Model, ResolvedIdentifier, TupleLiteral,
	},
	ty::EnumRef,
	utils::maybe_grow_stack,
};
use std::sync::Arc;

use super::top_down_type::add_coercion;

struct EnumEraser<Dst: Marker, Src: Marker = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	tys: Arc<TypeRegistry>,
	enum_definitions: Vec<Expression<Dst>>,
	identifier_replacement: FxHashMap<ResolvedIdentifier<Src>, DeclarationId<Dst>>,
	mzn_enum_for_item: ArenaMap<EnumerationItem<Src>, DeclarationId<Dst>>,
	enum_id_for_ty: FxHashMap<EnumRef, i64>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for EnumEraser<Dst, Src> {
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

	fn add_function(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f].name() == self.ids.erase_enum && model[f].body().is_some() {
			// Remove unnecessary functions
			return;
		}
		add_function(self, db, model, f);
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f].name() == self.ids.erase_enum && model[f].body().is_some() {
			// Remove unnecessary functions
			return;
		}
		fold_function_body(self, db, model, f);
	}

	fn fold_function(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		f: &Function<Src>,
	) -> Function<Dst> {
		let mut folded = fold_function(self, db, model, f);
		if f.name() == self.ids.show && f.body().is_none() {
			let p = &model[f.parameter(0)];
			if let Some(enum_ty) = p.ty().enum_ty(db.upcast()) {
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
						function: self.ids.mzn_show_enum.into(),
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
		db: &dyn Thir,
		model: &Model<Src>,
		identifier: &ResolvedIdentifier<Src>,
	) -> ResolvedIdentifier<Dst> {
		if let Some(i) = self.identifier_replacement.get(identifier) {
			(*i).into()
		} else {
			fold_identifier(self, db, model, identifier)
		}
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model<Src>, call: &Call<Src>) -> Call<Dst> {
		// Erase enum constructor into function call
		match &call.function {
			Callable::EnumConstructor(e) => {
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
						call.arguments
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
				LookupCall {
					function: self.ids.mzn_construct_enum.into(),
					arguments,
				}
				.resolve(db, &self.model)
				.0
			}
			Callable::EnumDestructor(e) => {
				let mzn_enum = self.mzn_enum_for_item[e.enumeration_id()];
				let member_id = e.member_index() as i64 + 1;
				let origin = model[e.enumeration_id()].origin();
				let arguments = vec![
					Expression::new(db, &self.model, origin, mzn_enum),
					Expression::new(db, &self.model, origin, IntegerLiteral(member_id)),
					self.fold_expression(db, model, &call.arguments[0]),
				];
				LookupCall {
					function: self.ids.mzn_destruct_enum.into(),
					arguments,
				}
				.resolve(db, &self.model)
				.0
			}
			Callable::Function(f)
				if model[*f].name() == self.ids.to_enum && model[*f].body().is_none() =>
			{
				LookupCall {
					function: self.ids.mzn_to_enum.into(),
					arguments: call
						.arguments
						.iter()
						.map(|arg| self.fold_expression(db, model, arg))
						.collect(),
				}
				.resolve(db, &self.model)
				.0
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
		maybe_grow_stack(|| {
			if let ExpressionData::Call(c) = &**expression {
				if let Callable::Function(f) = &c.function {
					if model[*f].name() == self.ids.erase_enum {
						return self.fold_expression(db, model, &c.arguments[0]);
					}
				}
			}
			fold_expression(self, db, model, expression)
		})
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
	fn erase_enum(&mut self, db: &dyn Thir, model: &Model<Src>, idx: EnumerationId<Src>) {
		let enumeration = &model[idx];
		let origin = enumeration.origin();
		let enum_id = self.enum_definitions.len() as i64 + 1;
		self.enum_id_for_ty.insert(enumeration.enum_type(), enum_id);

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
											.enum_ty(db.upcast())
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
								StringLiteral::new(name.pretty_print(db), db.upcast()),
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
				Domain::unbounded(db, origin, self.tys.mzn_enum_definition),
			);
			enum_declaration.set_name(Identifier::new(
				format!(
					"mzn_enum_{}",
					enumeration.enum_type().name(db.upcast()).value(db.upcast())
				),
				db.upcast(),
			));
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
				function: self.ids.mzn_get_enum.into(),
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
		defining_set_declaration.set_name(enumeration.enum_type().name(db.upcast()).into());
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
				function: self.ids.mzn_defining_set.into(),
				arguments: vec![Expression::new(db, &self.model, origin, mzn_enum_idx)],
			},
		));
		let defining_set = self
			.model
			.add_declaration(Item::new(defining_set_declaration, origin));
		self.identifier_replacement
			.insert(ResolvedIdentifier::Enumeration(idx), defining_set);

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
						function: self.ids.mzn_construct_enum.into(),
						arguments: vec![
							Expression::new(db, &self.model, origin, mzn_enum_idx),
							Expression::new(db, &self.model, origin, IntegerLiteral(i as i64 + 1)),
						],
					},
				));
				let member_idx = self.model.add_declaration(Item::new(atom, origin));

				self.identifier_replacement.insert(
					ResolvedIdentifier::EnumerationMember(EnumMemberId::new(idx, i as u32)),
					member_idx,
				);
			}
		}
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_enum(db: &dyn Thir, model: Model) -> Model {
	log::info!("Erasing enums into ints");
	let mut c = EnumEraser {
		model: Model::with_capacities(&model.entity_counts()),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
		tys: db.type_registry(),
		enum_definitions: Vec::with_capacity(model.enumerations_len() as usize),
		mzn_enum_for_item: ArenaMap::with_capacity(model.enumerations_len()),
		enum_id_for_ty: FxHashMap::default(),
		identifier_replacement: FxHashMap::default(),
	};
	c.add_model(db, &model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::{test::check, transformer, type_specialise::type_specialise};

	use super::erase_enum;

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
    set of int: Bar = mzn_defining_set(_DECL_1);
    int: E = mzn_construct_enum(_DECL_1, 1);
    int: F = mzn_construct_enum(_DECL_1, 2);
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_4 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_5 = [];
    } in _DECL_5), ("B", let {
      array [int] of tuple(int, set of int): _DECL_6 = [];
    } in _DECL_6), ("C", let {
      array [int] of tuple(int, set of int): _DECL_7 = [];
    } in _DECL_7), ("D", [(1, Bar)])]);
    set of int: Foo = mzn_defining_set(_DECL_4);
    int: A = mzn_construct_enum(_DECL_4, 1);
    int: B = mzn_construct_enum(_DECL_4, 2);
    int: C = mzn_construct_enum(_DECL_4, 3);
    int: x = B;
    int: y = mzn_construct_enum(_DECL_4, 4, [E]);
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
    set of int: Bar = mzn_defining_set(_DECL_1);
    int: E = mzn_construct_enum(_DECL_1, 1);
    int: F = mzn_construct_enum(_DECL_1, 2);
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_4 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_5 = [];
    } in _DECL_5), ("B", let {
      array [int] of tuple(int, set of int): _DECL_6 = [];
    } in _DECL_6), ("C", let {
      array [int] of tuple(int, set of int): _DECL_7 = [];
    } in _DECL_7), ("D", [(1, Bar)])]);
    set of int: Foo = mzn_defining_set(_DECL_4);
    int: A = mzn_construct_enum(_DECL_4, 1);
    int: B = mzn_construct_enum(_DECL_4, 2);
    int: C = mzn_construct_enum(_DECL_4, 3);
    function string: show(Foo: x) = mzn_show_enum([_DECL_1, _DECL_4], 2, x);
    function string: show(Bar: x) = mzn_show_enum([_DECL_1], 1, x);
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
				int: y = erase_enum(A);
            "#,
			expect!([r#"
    tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int)): _DECL_1 = mzn_get_enum([("A", let {
      array [int] of tuple(int, set of int): _DECL_2 = [];
    } in _DECL_2)]);
    set of int: Foo = mzn_defining_set(_DECL_1);
    int: A = mzn_construct_enum(_DECL_1, 1);
    Foo: x = mzn_to_enum(Foo, 1);
    int: y = A;
"#]),
		);
	}
}
