//! Erase option types
//! - Replace a non optional literal `x` with `(true, x)` if needed to coerce to optional
//! - Replace `<>` with `(false, ⊥)`
//! - Replace `opt T` with `tuple(bool, T)`
//! - Make `occurs(x)` return `x.1` and `deopt(x)` return `x.2`
//!
//! Does not handle records, so records must be erased into tuples first

use shackle_diagnostics::Result;
use shackle_hir::{BooleanLiteral, OptType, VarType, constants::IdentifierRegistry};
use shackle_ty::{Ty, registry::TypeRegistry};
use shackle_utils::maybe_grow_stack;

use crate::{
	Constraint, Db, Declaration, Domain, DomainData, DummyValue, Expression, ExpressionData,
	FunctionId, Item, Let, LetItem, LookupCall, LookupIdentifier, Marker, Model, TupleLiteral,
	source::Origin,
	traverse::{
		Folder, ReplacementMap, add_function, fold_domain, fold_expression, fold_function_body,
	},
};

struct OptEraser<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for OptEraser<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		if model[f].name() == self.ids.functions.mzn_construct_opt
			|| model[f].name() == self.ids.functions.mzn_destruct_opt
			|| model[f].name() == self.ids.functions.val2opt
		{
			// Remove mzn_construct_opt/mzn_destruct_opt/val2opt
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
		if model[f].name() == self.ids.functions.mzn_construct_opt
			|| model[f].name() == self.ids.functions.mzn_destruct_opt
			|| model[f].name() == self.ids.functions.val2opt
		{
			// Remove mzn_construct_opt/mzn_destruct_opt/val2opt
			return;
		}
		fold_function_body(self, db, model, f)
	}

	fn fold_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		d: &Declaration<'db, Src>,
	) -> Declaration<'db, Dst> {
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
			if matches!(&**def, ExpressionData::Absent) {
				// Transform `<>` into `(false, ...)`
				let origin = def.origin();
				let bool_false = Expression::new(db, &self.model, origin, BooleanLiteral(false));
				let bottom = Expression::new(
					db,
					&self.model,
					origin,
					DummyValue(declaration.ty().fields(db).unwrap()[1]),
				);
				declaration.set_definition(Expression::new(
					db,
					&self.model,
					origin,
					TupleLiteral(vec![bool_false, bottom]),
				));
			} else {
				declaration.set_definition(self.fold_expression(db, model, def));
			}
			declaration.validate(db);
		} else if let DomainData::Bounded(e) = &**d.domain()
			&& d.ty().inst(db) == Some(VarType::Var)
			&& d.ty().opt(db) == Some(OptType::Opt)
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
		declaration
	}

	fn fold_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
	) -> Domain<'db, Dst> {
		maybe_grow_stack(|| {
			let origin = domain.origin();
			if let Some(OptType::Opt) = domain.ty().opt(db) {
				// Convert into tuple of occurs boolean and non-optional value
				let occurs = if let Some(VarType::Var) = domain.ty().inst(db) {
					self.tys.var_bool
				} else {
					self.tys.par_bool
				};
				let deopt = domain.ty().make_occurs(db);
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
		})
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		expression: &Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		let origin = expression.origin();
		let mut folded = maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::Absent => {
					unreachable!("<> should have been replaced in a let declaration coercion")
				}
				ExpressionData::Call(c) if c.matches_builtin(model, self.ids.functions.val2opt) => {
					// Known to occur, transform `x` into `(true, x)`
					log::debug!("Erasing val2opt at {}", origin.pretty_print(db));
					assert_eq!(c.arguments.len(), 1, "val2opt should have one argument");
					let bool_true = Expression::new(db, &self.model, origin, BooleanLiteral(true));
					let value = self.fold_expression(db, model, &c.arguments[0]);
					Expression::new(
						db,
						&self.model,
						origin,
						TupleLiteral(vec![bool_true, value]),
					)
				}
				ExpressionData::Call(c)
					if c.matches_builtin(model, self.ids.functions.mzn_construct_opt)
						|| c.matches_builtin(model, self.ids.functions.mzn_destruct_opt) =>
				{
					// Remove calls to mzn_construct_opt/mzn_destruct_opt
					self.fold_expression(db, model, &c.arguments[0])
				}
				_ => fold_expression(self, db, model, expression),
			}
		});
		if expression.ty() == self.tys.par_opt_bool || expression.ty() == self.tys.var_opt_bool {
			// Needed so we can implement partial semantics during totalisation
			folded.annotations_mut().push(Expression::new(
				db,
				&self.model,
				expression.origin(),
				LookupIdentifier(self.ids.annotations.mzn_opt_bool),
			));
		}
		assert!(
			folded.ty().opt(db) != Some(OptType::Opt),
			"Did not erase opt for {:?} got {:?}) at {}",
			expression,
			folded,
			expression.origin().pretty_print(db)
		);
		folded
	}
}

impl<'db, Src: Marker, Dst: Marker> OptEraser<'db, Dst, Src> {
	fn create_opt_var(
		&mut self,
		db: &'db dyn Db,
		origin: Origin<'db>,
		domain: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
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
						function: self.ids.functions.mzn_opt_domain.into(),
						arguments: vec![domain.clone()],
					},
				),
			),
		);
		let tuple_ty = Ty::tuple(db, [occurs_decl.ty(), deopt_decl.ty()]);
		let occurs = self.model.add_declaration(Item::new(occurs_decl, origin));
		let deopt = self.model.add_declaration(Item::new(deopt_decl, origin));

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
					function: self.ids.functions.mzn_opt_channel.into(),
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
pub fn erase_opt<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Erasing option types");
	let mut c = OptEraser {
		model: Model::with_capacities(&model.item_counts()),
		replacement_map: ReplacementMap::default(),
		ids: IdentifierRegistry::lookup(db),
		tys: TypeRegistry::lookup(db),
	};
	c.add_model(db, &model);
	Ok(c.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check_no_stdlib};

	#[test]
	fn test_option_type_erasure() {
		check_no_stdlib(
			vec![Transform::TopDownType, Transform::EraseOpt],
			r#"
				annotation mzn_opt_bool;
				function var opt $T: val2opt(var $T: x);
				function opt $T: val2opt($T: x);
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
    annotation mzn_opt_bool;
    function set of int: mzn_opt_domain(set of int: x);
    predicate mzn_opt_channel(tuple(var bool, var int): x, set of int: s);
    tuple(bool, int): x = (true, 2);
    tuple(bool, bool): y = let {
      tuple(bool, bool): _DECL_5 = (false, false);
    } in _DECL_5 :: (mzn_opt_bool) :: (mzn_opt_bool);
    tuple(var bool, var int): a = let {
      set of int: _DECL_7 = {1, 2, 3};
    } in let {
      var bool: _DECL_8;
      var mzn_opt_domain(_DECL_7): _DECL_9;
      tuple(var bool, var int): _DECL_10 = (_DECL_8, _DECL_9);
      constraint mzn_opt_channel(_DECL_10, _DECL_7);
    } in _DECL_10;
    tuple(bool, int): b = if true then (true, 1) else let {
      tuple(bool, int): _DECL_12 = (false, 0);
    } in _DECL_12 endif;
    array [int] of tuple(bool, int): c = [(true, 1), let {
      tuple(bool, int): _DECL_14 = (false, 0);
    } in _DECL_14];
    tuple(int, tuple(bool, int)): d;
    tuple(tuple(bool, int), tuple(bool, int)): e = let {
      tuple(int, tuple(bool, int)): _DECL_17 = d;
    } in ((true, (_DECL_17).1), (_DECL_17).2);
    function tuple(bool, int): foo(tuple(bool, int): x) = (true, 1);
    tuple(bool, int): f = foo((true, 1));
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_option_type_erasure_2() {
		check_no_stdlib(
			vec![Transform::TopDownType, Transform::EraseOpt],
			r#"
			function var opt $T: val2opt(var $T: x);
			function opt $T: val2opt($T: x);
			function array [int] of var opt int: arrayXd(array [int] of var int, array [int] of var opt int);
			function int: foo(array [int] of var opt int: x);
			function set of int: bar(int: a, int: b);
			function var int: qux(array [int] of var int: x) = let {
				var bar(foo(x), foo(x)): r;
			} in r;
			"#,
			expect!([r#"
    function array [int] of tuple(var bool, var int): arrayXd(array [int] of var int: _DECL_1, array [int] of tuple(var bool, var int): _DECL_2);
    function int: foo(array [int] of tuple(var bool, var int): x);
    function set of int: bar(int: a, int: b);
    function var int: qux(array [int] of var int: x) = let {
      var bar(foo(let {
      array [int] of var int: _DECL_7 = x;
    } in arrayXd(_DECL_7, [(true, _DECL_8) | _DECL_8 in _DECL_7])), foo(let {
      array [int] of var int: _DECL_9 = x;
    } in arrayXd(_DECL_9, [(true, _DECL_10) | _DECL_10 in _DECL_9]))): r;
    } in r;
    solve satisfy;
"#]),
		);
	}
}
