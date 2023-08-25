//! Erase option types
//! - Replace a non optional literal `x` with `(true, x)` if needed to coerce to optional
//! - Replace `<>` with `(false, ⊥)`
//! - Replace `opt T` with `tuple(bool, T)`
//! - Make `occurs(x)` return `x.1` and `deopt(x)` return `x.2`
//!
//! Does not handle records, so records must be erased into tuples first

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::{IdentifierRegistry, TypeRegistry},
	hir::{BooleanLiteral, IntegerLiteral, OptType, VarType},
	thir::{
		db::Thir,
		source::Origin,
		traverse::{
			add_function, fold_domain, fold_expression, fold_function_body, Folder, ReplacementMap,
		},
		ArrayComprehension, Call, Callable, Constraint, Declaration, Domain, DomainData,
		DummyValue, Expression, ExpressionData, FunctionId, Generator, Item, Let, LetItem,
		LookupCall, Marker, Model, TupleAccess, TupleLiteral,
	},
	ty::{Ty, TyData},
	utils::maybe_grow_stack,
};

struct OptEraser<Dst: Marker, Src: Marker = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	tys: Arc<TypeRegistry>,
	needs_opt_erase: FxHashMap<(Ty, Ty), bool>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for OptEraser<Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f].name() == self.ids.mzn_construct_opt
			|| model[f].name() == self.ids.mzn_destruct_opt
		{
			// Remove mzn_construct_opt/mzn_destruct_opt
			return;
		}
		add_function(self, db, model, f);
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model<Src>, f: FunctionId<Src>) {
		if model[f].name() == self.ids.mzn_construct_opt
			|| model[f].name() == self.ids.mzn_destruct_opt
		{
			// Remove mzn_construct_opt/mzn_destruct_opt
			return;
		}
		fold_function_body(self, db, model, f)
	}

	fn fold_declaration(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		d: &Declaration<Src>,
	) -> Declaration<Dst> {
		let mut declaration =
			Declaration::new(d.top_level(), self.fold_domain(db, model, d.domain()));
		if let Some(name) = d.name() {
			declaration.set_name(name);
		}
		declaration.annotations_mut().extend(
			d.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
		if let Some(def) = d.definition() {
			let folded = self.fold_expression(db, model, def);
			let erased = self.erase_opt(db, d.ty(), def.ty(), folded);
			declaration.set_definition(erased);
			declaration.validate(db);
		} else if let DomainData::Bounded(e) = &**d.domain() {
			if d.ty().inst(db.upcast()) == Some(VarType::Var)
				&& d.ty().opt(db.upcast()) == Some(OptType::Opt)
			{
				// Cannot leave domain in tuple type-inst
				let dom = self.fold_expression(db, model, e);
				let dom_decl = Declaration::from_expression(db, false, dom);
				let dom_idx = self.model.add_declaration(Item::new(dom_decl, e.origin()));
				let opt_var = self.create_opt_var(
					db,
					e.origin(),
					Expression::new(db, &self.model, e.origin(), dom_idx),
				);
				declaration.set_definition(Expression::new(
					db,
					&self.model,
					e.origin(),
					Let {
						items: vec![LetItem::Declaration(dom_idx)],
						in_expression: Box::new(opt_var),
					},
				));
			}
		}
		declaration
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
			let deopt = domain.ty().make_occurs(db.upcast());
			return Domain::tuple(
				db,
				origin,
				OptType::NonOpt,
				[
					Domain::unbounded(db, origin, occurs),
					Domain::unbounded(db, origin, deopt),
				],
			);
		}
		fold_domain(self, db, model, domain)
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model<Src>, call: &Call<Src>) -> Call<Dst> {
		let fe = call.function_type(db, model);
		let call = Call {
			function: self.fold_callable(db, model, &call.function),
			arguments: call
				.arguments
				.iter()
				.zip(fe.params.iter())
				.map(|(arg, param_ty)| {
					let folded = self.fold_expression(db, model, arg);
					self.erase_opt(db, *param_ty, arg.ty(), folded)
				})
				.collect(),
		};
		call
	}

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		expression: &Expression<Src>,
	) -> Expression<Dst> {
		maybe_grow_stack(|| {
			if let ExpressionData::Call(c) = &**expression {
				// Remove calls to mzn_construct_opt/mzn_destruct_opt
				if let Callable::Function(f) = &c.function {
					if model[*f].name() == self.ids.mzn_construct_opt
						|| model[*f].name() == self.ids.mzn_destruct_opt
					{
						return self.fold_expression(db, model, &c.arguments[0]);
					}
				}
			}
			fold_expression(self, db, model, expression)
		})
	}
}

impl<Src: Marker, Dst: Marker> OptEraser<Dst, Src> {
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
		mut e: Expression<Dst>,
	) -> Expression<Dst> {
		let origin = e.origin();
		if top_down_ty.opt(db.upcast()) == Some(OptType::Opt)
			&& bottom_up_ty.opt(db.upcast()) == Some(OptType::NonOpt)
		{
			// Known to occur, transform `x` into `(true, x)`
			let bool_true = Expression::new(db, &self.model, origin, BooleanLiteral(true));
			return Expression::new(db, &self.model, origin, TupleLiteral(vec![bool_true, e]));
		}
		if let ExpressionData::Absent = &*e {
			// Transform `<>` into `(false, ...)`
			let bool_false = Expression::new(db, &self.model, origin, BooleanLiteral(false));
			let bottom = Expression::new(
				db,
				&self.model,
				origin,
				DummyValue(top_down_ty.with_opt(db.upcast(), OptType::NonOpt)),
			);
			let mut tl = Expression::new(
				db,
				&self.model,
				origin,
				TupleLiteral(vec![bool_false, bottom]),
			);
			tl.annotations_mut().extend(e.annotations_mut().drain(..));
			return tl;
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
				TyData::Tuple(_, b_fs) => {
					// Create tuple literal then erase optionality on fields
					let fields = top_down_ty
						.fields(db.upcast())
						.unwrap()
						.into_iter()
						.zip(b_fs.iter())
						.enumerate()
						.map(|(i, (t, b))| {
							self.erase_opt(
								db,
								t,
								*b,
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

	fn create_opt_var(
		&mut self,
		db: &dyn Thir,
		origin: Origin,
		domain: Expression<Dst>,
	) -> Expression<Dst> {
		let occurs_decl = Declaration::new(false, Domain::unbounded(db, origin, self.tys.var_bool));
		let deopt_decl = Declaration::new(
			false,
			Domain::bounded(
				db,
				origin,
				VarType::Var,
				OptType::NonOpt,
				Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.mzn_opt_domain.into(),
						arguments: vec![domain.clone()],
					},
				),
			),
		);
		let tuple_ty = Ty::tuple(db.upcast(), [occurs_decl.ty(), deopt_decl.ty()]);
		let occurs = self.model.add_declaration(Item::new(occurs_decl, origin));
		let deopt = self.model.add_declaration(Item::new(deopt_decl, origin));

		Expression::new(
			db,
			&self.model,
			origin,
			TupleLiteral(vec![
				Expression::new(db, &self.model, origin, occurs),
				Expression::new(db, &self.model, origin, deopt),
			]),
		);

		let mut tuple_decl = Declaration::new(false, Domain::unbounded(db, origin, tuple_ty));
		tuple_decl.set_definition(Expression::new(
			db,
			&self.model,
			origin,
			TupleLiteral(vec![
				Expression::new(db, &self.model, origin, occurs),
				Expression::new(db, &self.model, origin, deopt),
			]),
		));
		let tuple = self.model.add_declaration(Item::new(tuple_decl, origin));

		let channel = Constraint::new(
			false,
			Expression::new(
				db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.mzn_opt_channel.into(),
					arguments: vec![Expression::new(db, &self.model, origin, tuple), domain],
				},
			),
		);

		let constraint = self.model.add_constraint(Item::new(channel, origin));
		Expression::new(
			db,
			&self.model,
			origin,
			Let {
				items: vec![
					LetItem::Declaration(occurs),
					LetItem::Declaration(deopt),
					LetItem::Declaration(tuple),
					LetItem::Constraint(constraint),
				],
				in_expression: Box::new(Expression::new(db, &self.model, origin, tuple)),
			},
		)
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_opt(db: &dyn Thir, model: Model) -> Model {
	log::info!("Erasing option types");
	let mut c = OptEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
		tys: db.type_registry(),
		needs_opt_erase: FxHashMap::default(),
	};
	c.add_model(db, &model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::{test::check_no_stdlib, top_down_type, transformer};

	use super::erase_opt;

	#[test]
	fn test_option_type_erasure() {
		check_no_stdlib(
			transformer(vec![top_down_type, erase_opt]),
			r#"
				function set of int: mzn_opt_domain(set of int: x);
				predicate mzn_opt_channel(var opt int: x, set of int: s);
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
    function set of int: mzn_opt_domain(set of int: x);
    function var bool: mzn_opt_channel(tuple(var bool, var int): x, set of int: s);
    tuple(bool, int): x = (true, 2);
    tuple(bool, bool): y = (false, false);
    tuple(var bool, var int): a = let {
      set of int: _DECL_6 = {1, 2, 3};
    } in let {
      var bool: _DECL_7;
      var mzn_opt_domain(_DECL_6): _DECL_8;
      tuple(var bool, var int): _DECL_9 = (_DECL_7, _DECL_8);
      constraint mzn_opt_channel(_DECL_9, _DECL_6);
    } in _DECL_9;
    tuple(bool, int): b = if true then let {
      tuple(bool, int): _DECL_11 = (true, 1);
    } in _DECL_11 else let {
      tuple(bool, int): _DECL_12 = (false, 0);
    } in _DECL_12 endif;
    array [int] of tuple(bool, int): c = [let {
      tuple(bool, int): _DECL_14 = (true, 1);
    } in _DECL_14, let {
      tuple(bool, int): _DECL_15 = (false, 0);
    } in _DECL_15];
    tuple(int, tuple(bool, int)): d;
    tuple(tuple(bool, int), tuple(bool, int)): e = ((true, d.1), d.2);
    function tuple(bool, int): foo(tuple(bool, int): x) = let {
      tuple(bool, int): _DECL_22 = (true, 1);
    } in _DECL_22;
    tuple(bool, int): f = foo(let {
      tuple(bool, int): _DECL_20 = (true, 1);
    } in _DECL_20);
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_option_type_erasure_2() {
		check_no_stdlib(
			transformer(vec![top_down_type, erase_opt]),
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
      var bar(foo(let {
      array [int] of tuple(var bool, var int): _DECL_8 = arrayXd(x, [(true, _DECL_7) | _DECL_7 in x]);
    } in _DECL_8), foo(let {
      array [int] of tuple(var bool, var int): _DECL_10 = arrayXd(x, [(true, _DECL_9) | _DECL_9 in x]);
    } in _DECL_10)): r;
    } in r;
    solve satisfy;
"#]),
		);
	}
}
