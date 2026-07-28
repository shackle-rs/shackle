//! Top-down resolution of types.
//! - Assigns a real type to literals <>, [] and {} using a [Coercion] expression
//! - Makes coercions to opt explicit using a [Coercion] expression
//! - Makes value coercions explicit using bool2int, bool2int or int2float.

use shackle_diagnostics::Result;
use shackle_hir::{IntegerLiteral, constants::IdentifierRegistry};
use shackle_ty::{
	FunctionType, OptType, PolymorphicFunctionType, Ty, TyData, registry::TypeRegistry,
};
use shackle_utils::{maybe_grow_stack, refmap::RefMap};

use crate::{
	ArrayComprehension, Callable, Db, Declaration, DeclarationId, Domain, EnumConstructorKind,
	Expression, ExpressionData, FunctionId, Generator, Item, Let, LetItem, LookupCall, Marker,
	Model, RecordAccess, RecordLiteral, TupleAccess, TupleLiteral,
	traverse::{Folder, ReplacementMap, add_declaration, add_function, fold_expression},
};

/// Coerce an expression to the given type if required
pub fn add_coercion<'db, T: Marker>(
	db: &'db dyn Db,
	model: &mut Model<'db, T>,
	ty: Ty<'db>,
	expression: Expression<'db, T>,
) -> Expression<'db, T> {
	Coercer {
		db,
		model,
		ids: IdentifierRegistry::lookup(db),
		tys: TypeRegistry::lookup(db),
	}
	.coerce(ty, expression)
}

struct Coercer<'a, 'db, T: Marker> {
	db: &'db dyn Db,
	model: &'a mut Model<'db, T>,
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
}

impl<'a, 'db, T: Marker> Coercer<'a, 'db, T> {
	fn coerce(&mut self, ty: Ty<'db>, expression: Expression<'db, T>) -> Expression<'db, T> {
		let db = self.db;
		if expression.ty() == ty || expression.ty().make_par(db) == ty.make_par(db) {
			return expression;
		}

		let ids = self.ids;
		let tys = self.tys;
		let origin = expression.origin();

		if ty.opt(db) == Some(OptType::Opt) && expression.ty().opt(db) != Some(OptType::Opt) {
			log::debug!("Adding val2opt at {}", origin.pretty_print(db));
			let coerced = self.coerce(ty.make_occurs(db), expression);
			return Expression::new(
				db,
				self.model,
				origin,
				LookupCall {
					function: ids.functions.val2opt.into(),
					arguments: vec![coerced],
				},
			);
		}

		if expression.ty() == tys.array_of_bottom
			|| expression.ty() == tys.array_of_opt_bottom
			|| expression.ty() == tys.set_of_bottom
			|| expression.ty() == tys.opt_bottom
		{
			let mut decl = Declaration::new(false, Domain::unbounded(db, origin, ty));
			decl.set_definition(expression);
			let idx = self.model.add_declaration(Item::new(decl, origin));
			return Expression::new(
				db,
				self.model,
				origin,
				Let {
					items: vec![LetItem::Declaration(idx)],
					in_expression: Box::new(Expression::new(db, self.model, origin, idx)),
				},
			);
		}

		let (expr_ty, target_ty) = if ty.is_set(db) {
			(
				expression.ty().elem_ty(db).unwrap(),
				ty.elem_ty(db).unwrap(),
			)
		} else {
			(expression.ty(), ty)
		};

		let coerced = match target_ty.lookup(db) {
			TyData::Integer(_, _) => {
				assert!(
					expr_ty.is_bool(db),
					"Invalid coercion from {} to {}",
					expr_ty,
					target_ty
				);
				log::debug!(
					"Adding bool2int at {}",
					expression.origin().pretty_print(db)
				);
				Expression::new(
					db,
					self.model,
					origin,
					LookupCall {
						function: ids.functions.bool2int.into(),
						arguments: vec![expression],
					},
				)
			}
			TyData::Float(_, _) => {
				if expr_ty.is_bool(db) {
					log::debug!(
						"Adding bool2float at {}",
						expression.origin().pretty_print(db)
					);
					Expression::new(
						db,
						self.model,
						origin,
						LookupCall {
							function: ids.functions.bool2float.into(),
							arguments: vec![expression],
						},
					)
				} else {
					assert!(
						expr_ty.is_int(db),
						"Invalid coercion from {} to {}",
						expr_ty,
						target_ty
					);
					log::debug!(
						"Adding int2float at {}",
						expression.origin().pretty_print(db)
					);
					Expression::new(
						db,
						self.model,
						origin,
						LookupCall {
							function: ids.functions.int2float.into(),
							arguments: vec![expression],
						},
					)
				}
			}
			TyData::Array { element, .. } => {
				let expr_decl = Declaration::from_expression(db, false, expression);
				let idx = self.model.add_declaration(Item::new(expr_decl, origin));
				let expr_ident = Expression::new(db, self.model, origin, idx);
				let gen_decl = Declaration::new(
					false,
					Domain::unbounded(db, origin, expr_ty.elem_ty(db).unwrap()),
				);
				let gen_idx = self.model.add_declaration(Item::new(gen_decl, origin));
				let gen_ident = Expression::new(db, self.model, origin, gen_idx);
				let template = self.coerce(*element, gen_ident);
				let comp = Expression::new(
					db,
					self.model,
					origin,
					ArrayComprehension {
						indices: None,
						template: Box::new(template),
						generators: vec![Generator::Iterator {
							declarations: vec![gen_idx],
							collection: expr_ident.clone(),
							where_clause: None,
						}],
					},
				);
				let array = Expression::new(
					db,
					self.model,
					origin,
					LookupCall {
						function: ids.functions.array_xd.into(),
						arguments: vec![expr_ident, comp],
					},
				);
				Expression::new(
					db,
					self.model,
					origin,
					Let {
						items: vec![LetItem::Declaration(idx)],
						in_expression: Box::new(array),
					},
				)
			}
			TyData::Tuple(_, fs) => {
				let expr_decl = Declaration::from_expression(db, false, expression);
				let idx = self.model.add_declaration(Item::new(expr_decl, origin));
				let expr_ident = Expression::new(db, self.model, origin, idx);
				let fields = fs
					.iter()
					.enumerate()
					.map(|(i, t)| {
						let field = Expression::new(
							db,
							self.model,
							origin,
							TupleAccess {
								tuple: Box::new(expr_ident.clone()),
								field: IntegerLiteral((i + 1) as i64),
							},
						);
						self.coerce(*t, field)
					})
					.collect::<Vec<_>>();
				let tuple = Expression::new(db, self.model, origin, TupleLiteral(fields));
				Expression::new(
					db,
					self.model,
					origin,
					Let {
						items: vec![LetItem::Declaration(idx)],
						in_expression: Box::new(tuple),
					},
				)
			}
			TyData::Record(_, fs) => {
				let expr_decl = Declaration::from_expression(db, false, expression);
				let idx = self.model.add_declaration(Item::new(expr_decl, origin));
				let expr_ident = Expression::new(db, self.model, origin, idx);
				let fields = fs
					.iter()
					.map(|(i, t)| {
						let ident = (*i).into();
						let field = Expression::new(
							db,
							self.model,
							origin,
							RecordAccess {
								record: Box::new(expr_ident.clone()),
								field: ident,
							},
						);
						(ident, self.coerce(*t, field))
					})
					.collect::<Vec<_>>();
				let record = Expression::new(db, self.model, origin, RecordLiteral(fields));
				Expression::new(
					db,
					self.model,
					origin,
					Let {
						items: vec![LetItem::Declaration(idx)],
						in_expression: Box::new(record),
					},
				)
			}
			_ => expression,
		};

		assert!(
			coerced.ty().ty_var(db).is_some() || coerced.ty().is_subtype_of(db, ty),
			"Coercion from {} to {} resulted in type {}",
			expr_ty,
			ty,
			coerced.ty()
		);

		coerced
	}
}

fn replace_bottom<'db>(db: &'db dyn Db, ty: Ty<'db>) -> Ty<'db> {
	// Replace bottom with int when it doesn't matter what type is used
	match ty.lookup(db) {
		TyData::Bottom(opt) => Ty::par_int(db).with_opt(db, *opt),
		TyData::Array { opt, dim, element } => {
			Ty::array(db, replace_bottom(db, *dim), replace_bottom(db, *element))
				.unwrap()
				.with_opt(db, *opt)
		}
		TyData::Set(inst, opt, elem) => Ty::par_set(db, replace_bottom(db, *elem))
			.unwrap()
			.with_inst(db, *inst)
			.unwrap()
			.with_opt(db, *opt),
		TyData::Tuple(opt, fs) => {
			Ty::tuple(db, fs.iter().map(|f| replace_bottom(db, *f))).with_opt(db, *opt)
		}
		TyData::Record(opt, fs) => {
			Ty::record(db, fs.iter().map(|(i, f)| (*i, replace_bottom(db, *f)))).with_opt(db, *opt)
		}
		TyData::Function(opt, f) => Ty::function(
			db,
			FunctionType {
				params: f.params.iter().map(|p| replace_bottom(db, *p)).collect(),
				return_type: replace_bottom(db, f.return_type),
			},
		)
		.with_opt(db, *opt),
		_ => ty,
	}
}

#[derive(Default)]
struct TopDownTyper<'a, 'db, Dst: Marker, Src: Marker = ()> {
	types: RefMap<'a, Expression<'db, Src>, Ty<'db>>,
	result: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
}

impl<'a, 'db, Src: Marker, Dst: Marker> Folder<'a, 'db, Dst, Src>
	for TopDownTyper<'a, 'db, Dst, Src>
{
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.result
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &'a Model<'db, Src>,
		d: DeclarationId<'db, Src>,
	) {
		if let Some(def) = model[d].definition() {
			self.insert(db, def, model[d].ty());
		}
		let _ = add_declaration(self, db, model, d);
	}

	fn add_function(
		&mut self,
		db: &'db dyn Db,
		model: &'a Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		if let Some(body) = model[f].body() {
			self.insert(db, body, model[f].return_type());
		}
		let _ = add_function(self, db, model, f);
	}

	fn fold_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &'a Model<'db, Src>,
		d: &'a Declaration<'db, Src>,
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
			self.insert(db, def, d.ty());
			let _ = self.propagate_ty(db, model, def);
			let def = self.fold_expression(db, model, def);
			declaration.set_definition(def);
			declaration.validate(db);
		}
		declaration
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &'a Model<'db, Src>,
		expression: &'a Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			let enter_expression = self.propagate_ty(db, model, expression);
			let folded = fold_expression(self, db, model, expression);
			if !enter_expression && let Some(ty) = self.get(expression) {
				return add_coercion(db, &mut self.result, ty, folded);
			}
			folded
		})
	}
}

impl<'a, 'db, Src: Marker, Dst: Marker> TopDownTyper<'a, 'db, Dst, Src> {
	fn insert(&mut self, db: &'db dyn Db, e: &'a Expression<'db, Src>, ty: Ty<'db>) {
		assert!(
			e.ty().is_subtype_of(db, ty),
			"{} is not a subtype of {} at {}",
			e.ty().pretty_print(db),
			ty.pretty_print(db),
			e.origin().pretty_print(db)
		);
		let _ = self.types.insert(e, ty);
	}

	fn extend(
		&mut self,
		db: &'db dyn Db,
		iter: impl Iterator<Item = (&'a Expression<'db, Src>, Ty<'db>)>,
	) {
		for (e, ty) in iter {
			self.insert(db, e, ty);
		}
	}

	fn get(&self, e: &'a Expression<'db, Src>) -> Option<Ty<'db>> {
		self.types.get(e).copied()
	}

	fn propagate_ty(
		&mut self,
		db: &'db dyn Db,
		model: &'a Model<'db, Src>,
		expression: &'a Expression<'db, Src>,
	) -> bool {
		let ty = self.get(expression).unwrap_or_else(|| expression.ty());
		match &**expression {
			ExpressionData::ArrayLiteral(al) => {
				if al.is_empty() {
					return false;
				}
				self.extend(db, al.iter().map(|e| (e, ty.elem_ty(db).unwrap())))
			}
			ExpressionData::ArrayComprehension(c) => {
				self.insert(db, &*c.template, ty.elem_ty(db).unwrap())
			}
			ExpressionData::SetLiteral(sl) => {
				if sl.is_empty() {
					return false;
				}
				self.extend(
					db,
					sl.iter().map(|e| {
						(
							e,
							ty.elem_ty(db)
								.unwrap()
								.with_inst(db, ty.inst(db).unwrap())
								.unwrap(),
						)
					}),
				)
			}
			ExpressionData::SetComprehension(c) => self.insert(
				db,
				&*c.template,
				ty.elem_ty(db)
					.unwrap()
					.with_inst(db, ty.inst(db).unwrap())
					.unwrap(),
			),
			ExpressionData::IfThenElse(ite) => self.extend(
				db,
				ite.branches
					.iter()
					.map(|b| &b.result)
					.chain([&*ite.else_result])
					.map(|e| (e, ty)),
			),
			ExpressionData::Case(c) => self.extend(db, c.branches.iter().map(|b| (&b.result, ty))),
			ExpressionData::TupleLiteral(tl) => {
				self.extend(db, tl.iter().zip(ty.fields(db).unwrap()))
			}
			ExpressionData::Let(l) => self.insert(db, &l.in_expression, ty),
			ExpressionData::Call(c) => {
				let params = match &c.function {
					Callable::Annotation(a) => model[*a]
						.parameters
						.as_ref()
						.unwrap()
						.iter()
						.map(|p| model[*p].ty())
						.collect::<Vec<_>>(),
					Callable::AnnotationDestructure(_) => vec![TypeRegistry::lookup(db).ann],
					Callable::EnumConstructor(e) => model[*e]
						.parameters
						.as_ref()
						.unwrap()
						.iter()
						.map(|p| {
							EnumConstructorKind::from_tys(
								db,
								c.arguments.iter().map(|arg| arg.ty()),
							)
							.lift(db, model[*p].ty())
						})
						.collect::<Vec<_>>(),
					Callable::EnumDestructor(_) => {
						return false;
					}
					Callable::Expression(e) => e.ty().function_params(db).unwrap(),
					Callable::Function(f) => {
						if model[*f].is_polymorphic() {
							let bottom_up_tys =
								c.arguments.iter().map(|arg| arg.ty()).collect::<Vec<_>>();
							let overload = model[*f].function_entry(model);
							let mut ty_vars = overload
								.instantiate_ty_params(db, &bottom_up_tys)
								.unwrap()
								.0;
							if model[*f].return_type().contains_type_inst_var(db) {
								// Also instantiate with top-down return type
								let _ = PolymorphicFunctionType::collect_instantiations(
									db,
									&mut |tv, t| {
										let prev = ty_vars.get_mut(&tv).unwrap();
										*prev =
											Ty::most_specific_supertype(db, [*prev, t]).unwrap();
										true
									},
									ty,
									model[*f].return_type(),
								);
							}
							for t in ty_vars.values_mut() {
								// Any bottom left in the ty vars must not matter, so just change to int
								*t = replace_bottom(db, *t);
							}
							model[*f]
								.function_entry(model)
								.instantiate(db, &ty_vars)
								.params
								.to_vec()
						} else {
							model[*f]
								.parameters()
								.iter()
								.map(|p| model[*p].ty())
								.collect::<Vec<_>>()
						}
					}
				};
				self.extend(db, c.arguments.iter().zip(params));
				return false;
			}
			_ => return false,
		};
		true
	}
}

/// Compute real types for bottom types
pub fn top_down_type<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Computing top-down types");
	let mut tdt = TopDownTyper::default();
	tdt.add_model(db, &model);
	Ok(tdt.result)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::top_down_type;
	use crate::transform::tests::check_no_stdlib;

	#[test]
	fn test_top_down_type_bottom() {
		check_no_stdlib(
			top_down_type,
			r#"
                    function set of int: foo(opt int);
                    any: a = foo(<>);
                    any: b = if true then [1] else [] endif;
                    tuple(int, set of int): c = (1, {});
					"#,
			expect!([r#"
    function set of int: foo(opt int: _DECL_1);
    set of int: a = foo(let {
      opt int: _DECL_2 = <>;
    } in _DECL_2);
    array [int] of int: b = if true then [1] else let {
      array [int] of int: _DECL_4 = [];
    } in _DECL_4 endif;
    tuple(int, set of int): c = (1, let {
      set of int: _DECL_6 = {};
    } in _DECL_6);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_top_down_type_bottom_polymorphic() {
		check_no_stdlib(
			top_down_type,
			r#"
                    function any $T: foo(any $T, array [$X] of any $U);
                    opt int: x = foo(<>, []);
					"#,
			expect!([r#"
    function any $T: foo(any $T: _DECL_1, array [$X] of any $U: _DECL_2);
    opt int: x = foo(let {
      opt int: _DECL_3 = <>;
    } in _DECL_3, let {
      array [int] of int: _DECL_4 = [];
    } in _DECL_4);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_top_down_type_opt() {
		check_no_stdlib(
			top_down_type,
			r#"
					function opt $T: val2opt($T: x);
                    any: x = ([1, <>],);
                    function int: foo(opt int);
                    any: y = foo(3);
					opt int: z = let {
						any: b = 1;
					} in (<>, 1).1;
					"#,
			expect!([r#"
    function opt $T: val2opt($T: x);
    tuple(array [int] of opt int): x = ([val2opt(1), let {
      opt int: _DECL_2 = <>;
    } in _DECL_2],);
    function int: foo(opt int: _DECL_4);
    int: y = foo(val2opt(3));
    opt int: z = let {
      int: b = 1;
    } in let {
      opt int: _DECL_7 = ((<>, 1)).1;
    } in _DECL_7;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_top_down_type_array_opt() {
		check_no_stdlib(
			top_down_type,
			r#"
					function opt $T: val2opt($T: x);
					function array [int] of opt int: arrayXd(array [int] of int, array [int] of opt int);
                    any: x = [1];
                    any: y = if true then x else [<>] endif;
                    array [int] of opt int: z = [1];
					"#,
			expect!([r#"
    function opt $T: val2opt($T: x);
    function array [int] of opt int: arrayXd(array [int] of int: _DECL_2, array [int] of opt int: _DECL_3);
    array [int] of int: x = [1];
    array [int] of opt int: y = if true then let {
      array [int] of int: _DECL_5 = x;
    } in arrayXd(_DECL_5, [val2opt(_DECL_6) | _DECL_6 in _DECL_5]) else [let {
      opt int: _DECL_7 = <>;
    } in _DECL_7] endif;
    array [int] of opt int: z = [val2opt(1)];
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_coercions() {
		check_no_stdlib(
			top_down_type,
			r#"
				function int: bool2int(bool: b);
				function float: bool2float(bool: b);
				function float: int2float(int: i);
				float: f = let {
					int: i = true;
				} in i;
				float: g = true;
				"#,
			expect!([r#"
    function int: bool2int(bool: b);
    function float: bool2float(bool: b);
    function float: int2float(int: i);
    float: f = let {
      int: i = bool2int(true);
    } in int2float(i);
    float: g = bool2float(true);
    solve satisfy;
"#]),
		)
	}
}
