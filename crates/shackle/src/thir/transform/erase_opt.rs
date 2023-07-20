//! Erase option types
//! - Replace a non optional literal `x` with `(true, x)` if needed to coerce to optional
//! - Replace `<>` with `(false, ⊥)`
//! - Replace `opt T` with `tuple(bool, T)`
//! - Make `occurs(x)` return `x.1` and `deopt(x)` return `x.2`
//!
//! Does not handle records, so records must be erased into tuple first

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::{IdentifierRegistry, TypeRegistry},
	hir::{BooleanLiteral, IntegerLiteral, OptType, VarType},
	refmap::RefMap,
	thir::{
		db::Thir,
		traverse::{
			fold_domain, fold_expression, visit_declaration, visit_expression, visit_function,
			Folder, ReplacementMap, Visitor,
		},
		ArrayComprehension, Bottom, Callable, Declaration, DeclarationId, Domain,
		EnumConstructorKind, Expression, ExpressionData, FunctionId, Generator, Item, Let, LetItem,
		LookupCall, Marker, Model, TupleAccess, TupleLiteral,
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

struct OptEraser<'a, Dst, Src = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
	ids: Arc<IdentifierRegistry>,
	tys: Arc<TypeRegistry>,
	top_down_types: TopDownTyper<'a, Src>,
	needs_opt_erase: FxHashMap<(Ty, Ty), bool>,
}

impl<Dst: Marker, Src: Marker> Folder<Dst, Src> for OptEraser<'_, Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn add_model(&mut self, db: &dyn Thir, model: &Model<Src>) {
		// Add items to the destination model
		for item in model.top_level_items() {
			self.add_item(db, model, item);
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
			let mut folded = fold_domain(self, db, model, domain);
			let deopt = folded.ty().make_occurs(db.upcast());
			folded.set_ty_unchecked(deopt);
			return Domain::tuple(
				db,
				origin,
				OptType::NonOpt,
				[Domain::unbounded(db, origin, occurs), folded],
			);
		}
		fold_domain(self, db, model, domain)
	}
}

impl<Src: Marker, Dst: Marker> OptEraser<'_, Dst, Src> {
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
}

/// Erase types which are not present in MicroZinc
pub fn erase_opt(db: &dyn Thir, model: &Model) -> Model {
	let mut top_down_types = TopDownTyper {
		db,
		types: RefMap::default(),
	};
	top_down_types.visit_model(model);
	let mut c = OptEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		ids: db.identifier_registry(),
		tys: db.type_registry(),
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

	use super::erase_opt;

	#[test]
	fn test_option_type_erasure() {
		check_no_stdlib(
			erase_opt,
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
			erase_opt,
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
