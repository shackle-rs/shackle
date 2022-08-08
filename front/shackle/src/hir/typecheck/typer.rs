use rustc_hash::FxHashMap;
use std::{fmt::Write, sync::Arc};

use crate::{
	arena::ArenaIndex,
	error::{
		AmbiguousCall, IllegalType, InvalidArrayLiteral, InvalidFieldAccess, NoMatchingFunction,
		TypeInferenceFailure, TypeMismatch, UndefinedIdentifier,
	},
	hir::{
		db::Hir,
		ids::{EntityRef, ExpressionRef, ItemRef, NodeRef, PatternRef},
		ArrayAccess, ArrayComprehension, ArrayLiteral, Call, Case, Declaration, Expression,
		FunctionEntry, FunctionResolutionError, FunctionType, Identifier, IfThenElse,
		InstantiationError, ItemData, Let, LetItem, OptType, Pattern, PrimitiveType, RecordAccess,
		RecordLiteral, SetComprehension, SetLiteral, TupleAccess, TupleLiteral, Ty, TyData, TyVar,
		TyVarRef, Type, TypeRegistry, VarType,
	},
	Error,
};

use super::{PatternTy, TypeContext};

/// Computes types of expressions and patterns in an item
pub struct Typer<'a, T> {
	db: &'a dyn Hir,
	types: &'a TypeRegistry,
	ctx: &'a mut T,
	item: ItemRef,
	data: &'a ItemData,
}

impl<'a, T: TypeContext> Typer<'a, T> {
	/// Create a new typer
	pub fn new(
		db: &'a dyn Hir,
		types: &'a TypeRegistry,
		ctx: &'a mut T,
		item: ItemRef,
		data: &'a ItemData,
	) -> Self {
		Typer {
			db,
			types,
			ctx,
			item,
			data,
		}
	}

	/// Collect the type of an expression and check that it is a subtype of the expected type.
	pub fn typecheck_expression(&mut self, expr: ArenaIndex<Expression>, expected: Ty) {
		let db = self.db;
		let actual = self.collect_expression(expr);
		if !actual.is_subtype_of(self.db, expected) {
			let (src, span) =
				NodeRef::from(EntityRef::new(self.db, self.item, expr)).source_span(self.db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						expected.pretty_print(db),
						actual.pretty_print(db)
					),
				},
			);
		}
	}

	/// Get the type of this expression
	pub fn collect_expression(&mut self, expr: ArenaIndex<Expression>) -> Ty {
		let db = self.db;
		for ann in self
			.data
			.annotations
			.get(expr)
			.iter()
			.flat_map(|anns| anns.iter())
		{
			self.typecheck_expression(*ann, self.types.ann);
		}
		let result = match &self.data[expr] {
			Expression::Absent => self.types.bottom.with_opt(db, OptType::Opt),
			Expression::BooleanLiteral(_) => self.types.par_bool,
			Expression::IntegerLiteral(_) => self.types.par_int,
			Expression::FloatLiteral(_) => self.types.par_float,
			Expression::StringLiteral(_) => self.types.string,
			Expression::Infinity => self.types.par_int,
			Expression::Identifier(i) => self.collect_identifier(expr, i),
			Expression::Call(c) => self.collect_call(expr, c),
			Expression::ArrayLiteral(al) => self.collect_array_literal(expr, al),
			Expression::SetLiteral(sl) => self.collect_set_literal(expr, sl),
			Expression::TupleLiteral(tl) => self.collect_tuple_literal(tl),
			Expression::RecordLiteral(rl) => self.collect_record_literal(rl),
			Expression::ArrayComprehension(c) => self.collect_array_comprehension(expr, c),
			Expression::SetComprehension(c) => self.collect_set_comprehension(expr, c),
			Expression::ArrayAccess(aa) => self.collect_array_access(expr, aa),
			Expression::TupleAccess(ta) => self.collect_tuple_access(expr, ta),
			Expression::RecordAccess(ra) => self.collect_record_access(expr, ra),
			Expression::IfThenElse(ite) => self.collect_if_then_else(expr, ite),
			Expression::Case(c) => self.collect_case(expr, c),
			Expression::Let(l) => self.collect_let(l),
			Expression::Slice(_) => self.types.set_of_bottom,
			Expression::Missing => self.types.error,
		};
		self.ctx
			.add_expression(ExpressionRef::new(self.item, expr), result);
		result
	}

	fn collect_identifier(&mut self, expr: ArenaIndex<Expression>, i: &Identifier) -> Ty {
		let db = self.db;
		if let Some(p) = self.find_variable(expr, *i) {
			let expression = ExpressionRef::new(self.item, expr);
			self.ctx.add_identifier_resolution(expression, p);
			match self.ctx.type_pattern(db, self.types, p) {
				PatternTy::Variable(ty) | PatternTy::EnumAtom(ty) | PatternTy::TypeAlias(ty) => {
					return ty
				}
				PatternTy::EnumConstructor(_) => (),
				PatternTy::Computing => {
					// Error will be emitted during topological sorting
					return self.types.error;
				}
				_ => {
					unreachable!("Matched variable in scope, but not a variable or type alias")
				}
			}
		}
		let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
		self.ctx.add_diagnostic(
			self.item,
			UndefinedIdentifier {
				identifier: i.lookup(db),
				src,
				span,
			},
		);
		self.types.error
	}

	fn collect_call(&mut self, expr: ArenaIndex<Expression>, c: &Call) -> Ty {
		let db = self.db;
		let args = c
			.arguments
			.iter()
			.map(|e| self.collect_expression(*e))
			.collect::<Vec<_>>();
		match self.data[c.function] {
			Expression::Identifier(i) => self.resolve_overloading(c.function, i, &args),
			_ => {
				let ty = self.collect_expression(c.function);
				if let TyData::Function(OptType::NonOpt, f) = ty.lookup(db) {
					if let Err(_) = f.matches(db, &args) {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							NoMatchingFunction {
								src,
								span,
								msg: format!(
									"Cannot call function with signature '{}' with arguments {}",
									f.pretty_print(db),
									args.iter()
										.map(|a| format!("'{}'", a.pretty_print(db)))
										.collect::<Vec<_>>()
										.join(", ")
								),
							},
						);
						return self.types.error;
					} else {
						return ty;
					}
				}

				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!("Type '{}' is not callable", ty.pretty_print(db)),
					},
				);
				self.types.error
			}
		}
	}

	fn collect_array_literal(&mut self, expr: ArenaIndex<Expression>, al: &ArrayLiteral) -> Ty {
		let db = self.db;
		if al.members.is_empty() {
			return Ty::array(db, self.types.par_int, self.types.bottom).unwrap();
		}
		let ty =
			Ty::most_specific_supertype(db, al.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						InvalidArrayLiteral {
							src,
							span,
							msg: "Non-uniform array literal".to_owned(),
						},
					);
					self.types.error
				});
		Ty::array(db, self.types.par_int, ty).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!("array [int] of {}", ty.pretty_print(db)),
				},
			);
			self.types.error
		})
	}

	fn collect_set_literal(&mut self, expr: ArenaIndex<Expression>, sl: &SetLiteral) -> Ty {
		let db = self.db;
		if sl.members.is_empty() {
			return Ty::par_set(db, self.types.bottom).unwrap();
		}
		let ty =
			Ty::most_specific_supertype(db, sl.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						InvalidArrayLiteral {
							src,
							span,
							msg: "Non-uniform array literal".to_owned(),
						},
					);
					self.types.error
				});
		match ty.inst(db) {
			Some(VarType::Var) => {
				let ty = ty.with_inst(db, VarType::Par).unwrap();
				Ty::par_set(db, ty)
					.and_then(|t| t.with_inst(db, VarType::Var))
					.unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: format!("var set of {}", ty.pretty_print(db)),
							},
						);
						self.types.error
					})
			}
			Some(VarType::Par) => Ty::par_set(db, ty).unwrap_or_else(|| {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					IllegalType {
						src,
						span,
						ty: format!("set of {}", ty.pretty_print(db)),
					},
				);
				self.types.error
			}),
			None => {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeInferenceFailure {
						src,
						span,
						msg: "Cannot determine inst for set literal".to_owned(),
					},
				);
				return self.types.error;
			}
		}
	}

	fn collect_tuple_literal(&mut self, tl: &TupleLiteral) -> Ty {
		let db = self.db;
		Ty::tuple(db, tl.fields.iter().map(|f| self.collect_expression(*f)))
	}

	fn collect_record_literal(&mut self, rl: &RecordLiteral) -> Ty {
		let db = self.db;
		Ty::record(
			db,
			rl.fields.iter().map(|(i, f)| {
				(
					self.data[*i]
						.identifier()
						.expect("Record field name not an identifier"),
					self.collect_expression(*f),
				)
			}),
		)
	}

	fn collect_array_comprehension(
		&mut self,
		expr: ArenaIndex<Expression>,
		c: &ArrayComprehension,
	) -> Ty {
		let db = self.db;
		let mut lift_to_opt = false;
		for g in c.generators.iter() {
			let collection = self.collect_expression(g.collection);
			let gen_el = match collection.lookup(db) {
				TyData::Array {
					opt: OptType::NonOpt,
					element,
					..
				} => element,
				TyData::Set(VarType::Par, OptType::NonOpt, element) => element,
				TyData::Set(VarType::Var, OptType::NonOpt, element) => {
					lift_to_opt = true;
					element
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db)
							),
						},
					);
					self.types.error
				}
			};
			for p in g.patterns.iter() {
				self.collect_pattern(Some(expr), *p, gen_el);
			}
			if let Some(w) = g.where_clause {
				let where_clause = self.collect_expression(w);
				if let Some(VarType::Var) = where_clause.inst(db) {
					lift_to_opt = true;
				}
				if !where_clause.is_subtype_of(db, self.types.var_bool) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db)
							),
						},
					);
				}
			}
		}
		let el = self.collect_expression(c.template);
		let element = if lift_to_opt {
			el.with_opt(db, OptType::Opt)
				.with_inst(db, VarType::Var)
				.unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("array [..] of var opt {}", el.pretty_print(db)),
						},
					);
					self.types.error
				})
		} else {
			el
		};
		let dim = c
			.indices
			.map(|i| self.collect_expression(i))
			.unwrap_or(self.types.par_int);
		Ty::array(db, dim, element).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim.pretty_print(db),
						element.pretty_print(db)
					),
				},
			);
			self.types.error
		})
	}

	fn collect_set_comprehension(
		&mut self,
		expr: ArenaIndex<Expression>,
		c: &SetComprehension,
	) -> Ty {
		let db = self.db;
		let mut is_var = false;
		for g in c.generators.iter() {
			let collection = self.collect_expression(g.collection);
			let gen_el = match collection.lookup(db) {
				TyData::Array {
					opt: OptType::NonOpt,
					element,
					..
				} => match element.inst(db) {
					Some(VarType::Var) => {
						is_var = true;
						element.with_inst(db, VarType::Par).unwrap()
					}
					Some(VarType::Par) => element,
					None => self.types.error,
				},
				TyData::Set(VarType::Par, OptType::NonOpt, element) => element,
				TyData::Set(VarType::Var, OptType::NonOpt, element) => {
					is_var = true;
					element
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db)
							),
						},
					);
					self.types.error
				}
			};
			for p in g.patterns.iter() {
				self.collect_pattern(Some(expr), *p, gen_el);
			}
			if let Some(w) = g.where_clause {
				let where_clause = self.collect_expression(w);
				if let Some(VarType::Var) = where_clause.inst(db) {
					is_var = true;
				}
				if !where_clause.is_subtype_of(db, self.types.var_bool) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db)
							),
						},
					);
				}
			}
		}
		let el = self.collect_expression(c.template);
		if !is_var {
			// Inst determined by el inst
			match el.inst(db) {
				Some(VarType::Var) => is_var = true,
				Some(VarType::Par) => (),
				None => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeInferenceFailure {
							src,
							span,
							msg: format!(
								"Could not determine inst for type {}",
								el.pretty_print(db)
							),
						},
					);
					return self.types.error;
				}
			}
		};

		let element = el.with_inst(db, VarType::Par).unwrap();
		Ty::par_set(db, element)
			.and_then(|ty| {
				if is_var {
					ty.with_inst(db, VarType::Var)
				} else {
					Some(ty)
				}
			})
			.unwrap_or_else(|| {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					IllegalType {
						src,
						span,
						ty: format!(
							"{}set of {}",
							if is_var { "var " } else { "" },
							element.pretty_print(db)
						),
					},
				);
				self.types.error
			})
	}

	fn collect_array_access(&mut self, expr: ArenaIndex<Expression>, aa: &ArrayAccess) -> Ty {
		let db = self.db;
		let collection = self.collect_expression(aa.collection);
		let indices = self.collect_expression(aa.indices);

		let process_index = |index: Ty, dim: Ty| -> Result<_, Error> {
			let mut make_var = false;
			let mut make_opt = false;
			if let TyData::Set(i1, o1, t) = index.lookup(db) {
				if !t.is_subtype_of(db, dim) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeMismatch {
						src,
						span,
						msg: format!(
							"Cannot slice index of type {} using {}",
							dim.pretty_print(db),
							index.pretty_print(db)
						),
					}
					.into());
				}
				if i1 == VarType::Var {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeMismatch {
						src,
						span,
						msg: "Slicing using variable range not supported".to_owned(),
					}
					.into());
				}
				if o1 == OptType::Opt {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeMismatch {
						src,
						span,
						msg: "Slicing using optional range not supported".to_owned(),
					}
					.into());
				}
				return Ok((make_var, make_opt, true));
			}

			if !index.is_subtype_of(
				db,
				dim.with_inst(db, VarType::Var).unwrap_or_else(|| {
					panic!(
						"Array dimension {} should be varifiable",
						dim.pretty_print(db),
					)
				}),
			) {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
				return Err(TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}', but got {}",
						dim.pretty_print(db),
						index.pretty_print(db)
					),
				}
				.into());
			}

			match indices.opt(db) {
				Some(OptType::Opt) => {
					make_opt = true;
				}
				None => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeInferenceFailure {
						src,
						span,
						msg: "Failed to determine optionality of array access
		due to unknown optionality of index."
							.to_owned(),
					}
					.into());
				}
				_ => (),
			}
			match indices.inst(db) {
				Some(VarType::Var) => {
					make_var = true;
				}
				None => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeInferenceFailure {
						src,
						span,
						msg: "Failed to determine inst of array access
		due to unknown inst of index."
							.to_owned(),
					}
					.into());
				}
				_ => (),
			}
			Ok((make_var, make_opt, false))
		};

		let mut slices = Vec::new();
		let mut make_var = false;
		let mut make_opt = false;
		let el = match collection.lookup(db) {
			TyData::Array { opt, dim, element } => {
				make_opt = make_opt || opt == OptType::Opt;
				match (indices.lookup(db), dim.lookup(db)) {
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
						make_opt = make_opt || o1 == OptType::Opt || o2 == OptType::Opt;
						if f1.len() != f2.len() {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, aa.indices))
									.source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								TypeMismatch {
									src,
									span,
									msg: format!(
										"Cannot index into {}D array using {} {}",
										f2.len(),
										f1.len(),
										if f1.len() > 1 { "indices" } else { "index" }
									),
								},
							);
							return self.types.error;
						}
						for (i, d) in f1.iter().zip(f2.iter()) {
							match process_index(*i, *d) {
								Ok((v, o, s)) => {
									make_var = make_var | v;
									make_opt = make_opt | o;
									if s {
										slices.push(*d);
									}
								}
								Err(e) => {
									self.ctx.add_diagnostic(self.item, e);
									return self.types.error;
								}
							}
						}
					}
					_ => match process_index(indices, dim) {
						Ok((v, o, s)) => {
							make_var = make_var | v;
							make_opt = make_opt | o;
							if s {
								slices.push(dim);
							}
						}
						Err(e) => {
							self.ctx.add_diagnostic(self.item, e);
							return self.types.error;
						}
					},
				}
				element
			}
			TyData::Error => return self.types.error,
			_ => {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, aa.collection)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected array type, but got '{}'",
							collection.pretty_print(db)
						),
					},
				);
				return self.types.error;
			}
		};

		if slices.is_empty() {
			let result = if make_var {
				el.with_inst(db, VarType::Var).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("var {}", el.pretty_print(db)),
						},
					);
					self.types.error
				})
			} else {
				el
			};
			if make_opt {
				result.with_opt(db, OptType::Opt)
			} else {
				result
			}
		} else {
			if make_var {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: "Slicing involving var index unsupported".to_owned(),
					},
				);
				return self.types.error;
			}

			let result = Ty::array(
				db,
				if slices.len() > 1 {
					Ty::tuple(db, slices)
				} else {
					slices[0]
				},
				el,
			)
			.unwrap();
			if make_opt {
				result.with_opt(db, OptType::Opt)
			} else {
				result
			}
		}
	}

	fn collect_tuple_access(&mut self, expr: ArenaIndex<Expression>, ta: &TupleAccess) -> Ty {
		let db = self.db;
		let tuple = self.collect_expression(ta.tuple);
		match tuple.lookup(db) {
			TyData::Tuple(opt, fields) => {
				let i = ta.field.0;
				if i < 1 || i > fields.len() as i64 {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						InvalidFieldAccess {
							src,
							span,
							msg: format!("No such field {} for '{}'", i, tuple.pretty_print(db)),
						},
					);
					return self.types.error;
				}
				let ty = fields[(i - 1) as usize];
				if let OptType::Opt = opt {
					ty.with_opt(db, OptType::Opt)
				} else {
					ty
				}
			}
			TyData::Array {
				opt: o1,
				dim,
				element,
			} => match element.lookup(db) {
				TyData::Tuple(o2, fields) => {
					let i = ta.field.0;
					if i < 1 || i > fields.len() as i64 {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							InvalidFieldAccess {
								src,
								span,
								msg: format!(
									"No such field {} for '{}'",
									i,
									element.pretty_print(db)
								),
							},
						);
						return self.types.error;
					}
					let el = fields[(i - 1) as usize];
					let ty = if let OptType::Opt = o1.max(o2) {
						el.with_opt(db, OptType::Opt)
					} else {
						el
					};
					Ty::array(db, dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print(db),
							ty.pretty_print(db)
						)
					})
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected array of tuple type, but got '{}'",
								tuple.pretty_print(db)
							),
						},
					);
					return self.types.error;
				}
			},
			TyData::Error => self.types.error,
			_ => {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!("Expected tuple type, but got '{}'", tuple.pretty_print(db)),
					},
				);
				self.types.error
			}
		}
	}

	fn collect_record_access(&mut self, expr: ArenaIndex<Expression>, ra: &RecordAccess) -> Ty {
		let db = self.db;
		let record = self.collect_expression(ra.record);
		match record.lookup(db) {
			TyData::Record(opt, fields) => {
				let ty = fields
					.iter()
					.find(|(i, _)| *i == ra.field)
					.map(|(_, ty)| *ty)
					.unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							InvalidFieldAccess {
								src,
								span,
								msg: format!(
									"No such field {} for '{}'",
									ra.field.lookup(db),
									record.pretty_print(db)
								),
							},
						);
						self.types.error
					});
				if let OptType::Opt = opt {
					ty.with_opt(db, OptType::Opt)
				} else {
					ty
				}
			}
			TyData::Array {
				opt: o1,
				dim,
				element,
			} => match element.lookup(db) {
				TyData::Record(o2, fields) => {
					let el = fields
						.iter()
						.find(|(i, _)| *i == ra.field)
						.map(|(_, ty)| *ty)
						.unwrap_or_else(|| {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								InvalidFieldAccess {
									src,
									span,
									msg: format!(
										"No such field {} for '{}'",
										ra.field.lookup(db),
										element.pretty_print(db)
									),
								},
							);
							self.types.error
						});
					let ty = if let OptType::Opt = o1.max(o2) {
						el.with_opt(db, OptType::Opt)
					} else {
						el
					};
					Ty::array(db, dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print(db),
							ty.pretty_print(db)
						)
					})
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected array of record type, but got '{}'",
								record.pretty_print(db)
							),
						},
					);
					return self.types.error;
				}
			},
			TyData::Error => self.types.error,
			_ => {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected record type, but got '{}'",
							record.pretty_print(db)
						),
					},
				);
				self.types.error
			}
		}
	}

	fn collect_if_then_else(&mut self, expr: ArenaIndex<Expression>, ite: &IfThenElse) -> Ty {
		let db = self.db;
		let condition_types = ite
			.branches
			.iter()
			.map(|b| self.collect_expression(b.condition))
			.collect::<Vec<_>>();
		let result_types = ite
			.branches
			.iter()
			.map(|b| b.result)
			.chain(ite.else_result)
			.map(|e| self.collect_expression(e));
		let super_type = Ty::most_specific_supertype(db, result_types);
		if !condition_types
			.iter()
			.all(|t| t.is_subtype_of(db, self.types.var_bool))
		{
			return self.types.error;
		}
		match super_type {
			Some(ty) => {
				if let VarType::Var = condition_types
					.iter()
					.map(|t| t.inst(db).unwrap())
					.max()
					.unwrap()
				{
					// Var condition means var result
					ty.with_inst(db, VarType::Var).unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: format!("var {}", ty.pretty_print(db)),
							},
						);
						self.types.error
					})
				} else {
					ty
				}
			}
			None => {
				return self.types.error;
			}
		}
	}

	fn collect_case(&mut self, expr: ArenaIndex<Expression>, c: &Case) -> Ty {
		let scrutinee = self.collect_expression(c.expression);
		for case in c.cases.iter() {
			self.collect_pattern(Some(expr), case.pattern, scrutinee);
		}
		Ty::most_specific_supertype(
			self.db,
			c.cases
				.iter()
				.map(|case| self.collect_expression(case.value)),
		)
		.unwrap_or_else(|| {
			let (src, span) =
				NodeRef::from(EntityRef::new(self.db, self.item, expr)).source_span(self.db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: "Case expression has incompatible arm types.".to_owned(),
				},
			);
			self.types.error
		})
	}

	fn collect_let(&mut self, l: &Let) -> Ty {
		let db = self.db;
		for item in l.items.iter() {
			match item {
				LetItem::Constraint(c) => {
					for ann in c.annotations.iter() {
						self.typecheck_expression(*ann, self.types.ann);
					}
					self.typecheck_expression(c.expression, self.types.var_bool);
				}
				LetItem::Declaration(d) => {
					let ty = self.collect_declaration(d);
					if ty.known_par(db) && d.definition.is_none() {
						let (src, span) =
							NodeRef::from(EntityRef::new(self.db, self.item, d.pattern))
								.source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							TypeMismatch {
								src,
								span,
								msg: "Local parameter declaration must have a right-hand side"
									.to_owned(),
							},
						);
					}
				}
			}
		}
		self.collect_expression(l.in_expression)
	}

	/// Type check a declaration
	pub fn collect_declaration(&mut self, d: &Declaration) -> Ty {
		for p in Pattern::identifiers(d.pattern, self.data) {
			self.ctx
				.add_declaration(PatternRef::new(self.item, p), PatternTy::Computing);
		}
		let ty = if let Some(e) = d.definition {
			let actual = self.collect_expression(e);
			let expected = self.complete_type(d.declared_type, Some(actual));
			if !actual.is_subtype_of(self.db, expected) {
				let (src, span) =
					NodeRef::from(EntityRef::new(self.db, self.item, e)).source_span(self.db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected '{}' but got '{}'",
							expected.pretty_print(self.db),
							actual.pretty_print(self.db)
						),
					},
				);
			}
			expected
		} else {
			self.complete_type(d.declared_type, None)
		};
		self.collect_pattern(None, d.pattern, ty);
		ty
	}

	fn resolve_overloading(
		&mut self,
		expr: ArenaIndex<Expression>,
		i: Identifier,
		args: &[Ty],
	) -> Ty {
		let db = self.db;
		if args.iter().any(|t| t.contains_error(db)) {
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return self.types.error;
		}

		// If there's a variable in scope which is a function, use it
		if let Some(p) = self.find_variable(expr, i) {
			let d = self.ctx.type_pattern(db, self.types, p);
			let f = match d {
				PatternTy::Variable(t) => {
					if let TyData::Function(OptType::NonOpt, f) = t.lookup(db) {
						Some(f)
					} else {
						None
					}
				}
				_ => None,
			};
			if let Some(f) = f {
				if f.contains_error(db) {
					return self.types.error;
				}
				if let Err(e) = f.matches(db, args) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					let mut msg = format!(
						"Cannot call function with signature '{}'",
						f.pretty_print(db)
					);
					match e {
						InstantiationError::ArgumentCountMismatch { expected, actual } => {
							writeln!(
								&mut msg,
								"  {} arguments required, {} given",
								expected, actual
							)
							.unwrap();
						}
						InstantiationError::ArgumentMismatch {
							index,
							expected,
							actual,
						} => {
							writeln!(
								&mut msg,
								"  argument {} expected '{}', but '{}' given",
								index + 1,
								expected.pretty_print(db),
								actual.pretty_print(db)
							)
							.unwrap();
						}
						_ => unreachable!("Polymorphic function expressions not allowed"),
					}
					self.ctx
						.add_diagnostic(self.item, TypeMismatch { src, span, msg });
					return self.types.error;
				}
				let ret = f.return_type;
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), Ty::function(db, f));
				self.ctx
					.add_identifier_resolution(ExpressionRef::new(self.item, expr), p);
				return ret;
			}
		}

		// Otherwise resolve overloaded function items
		let patterns = self.find_function(expr, i);
		if patterns.is_empty() {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				NoMatchingFunction {
					src,
					span,
					msg: format!("No function with name '{}' could be found.", i.lookup(db)),
				},
			);
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return self.types.error;
		}

		let mut overloads = Vec::new();
		for p in patterns.iter() {
			match self.ctx.type_pattern(db, self.types, *p) {
				PatternTy::Function(function) => overloads.push((*p, *function.clone())),
				PatternTy::EnumConstructor(functions) => {
					overloads.extend(functions.iter().map(|function| (*p, function.clone())))
				}
				PatternTy::Computing => (),
				_ => unreachable!(),
			}
		}

		if overloads.is_empty() {
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return self.types.error;
		}

		match FunctionEntry::match_fn(db, overloads, args) {
			Ok((pattern, _, instantiation)) => {
				let ret = instantiation.return_type;
				let ty = Ty::function(db, instantiation);
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), ty);
				self.ctx
					.add_identifier_resolution(ExpressionRef::new(self.item, expr), pattern);
				ret
			}
			Err(FunctionResolutionError::AmbiguousOverloading(ps)) => {
				let mut msg = format!(
					"Call with argument types {} is ambiguous.\n",
					args.iter()
						.map(|t| format!("'{}'", t.pretty_print(db)))
						.collect::<Vec<_>>()
						.join(", ")
				);
				writeln!(
					&mut msg,
					"Could not choose an overload from the candidate functions:"
				)
				.unwrap();
				for (_, f) in ps.iter() {
					writeln!(&mut msg, "  {}", f.overload.pretty_print_item(db, i)).unwrap();
				}
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx
					.add_diagnostic(self.item, AmbiguousCall { src, span, msg });
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
				self.types.error
			}
			Err(FunctionResolutionError::NoMatchingFunction(es)) => {
				let mut msg = String::new();
				if args.is_empty() {
					writeln!(
						&mut msg,
						"No function '{}' could be found taking no arguments.",
						i.lookup(db)
					)
					.unwrap();
				} else {
					writeln!(
						&mut msg,
						"No function '{}' matching argument types {} could be found.",
						i.lookup(db),
						args.iter()
							.map(|t| format!("'{}'", t.pretty_print(db)))
							.collect::<Vec<_>>()
							.join(", ")
					)
					.unwrap();
				}
				writeln!(&mut msg, "The following overloads could not be used:").unwrap();
				for (_, f, e) in es.iter() {
					writeln!(&mut msg, "  {}", f.overload.pretty_print_item(db, i)).unwrap();
					match e {
						InstantiationError::ArgumentCountMismatch { expected, actual } => {
							writeln!(
								&mut msg,
								"    {} arguments required, {} given",
								expected, actual
							)
							.unwrap();
						}
						InstantiationError::ArgumentMismatch {
							index,
							expected,
							actual,
						} => {
							writeln!(
								&mut msg,
								"    argument {} expected '{}', but '{}' given",
								index + 1,
								expected.pretty_print(db),
								actual.pretty_print(db)
							)
							.unwrap();
						}
						InstantiationError::IncompatibleTypeInstVariable { ty_var, types } => {
							if types.len() == 0 {
								// Should not be possible currently
								writeln!(
									&mut msg,
									"    Type-inst parameter '{}' not instantiated",
									ty_var.name(db)
								)
								.unwrap();
							} else {
								let tys = types
									.iter()
									.map(|t| format!("'{}'", t.pretty_print(db)))
									.collect::<Vec<_>>()
									.join(", ");
								writeln!(
                                    &mut msg,
                                    "    Type-inst parameter '{}' instantiated with incompatible types {}",
                                    ty_var.name(db),
                                    tys
                                )
                                .unwrap();
							}
						}
					}
				}

				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx
					.add_diagnostic(self.item, NoMatchingFunction { src, span, msg });
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
				self.types.error
			}
		}
	}

	/// Collect the type of a pattern
	pub fn collect_pattern(
		&mut self,
		scope: Option<ArenaIndex<Expression>>,
		pat: ArenaIndex<Pattern>,
		expected: Ty,
	) -> Ty {
		let db = self.db;
		let actual = match &self.data[pat] {
			Pattern::Absent => self.types.opt_bottom,
			Pattern::Boolean(_) => self.types.par_bool,
			Pattern::Infinity { .. } | Pattern::Integer { .. } => self.types.par_int,
			Pattern::Float { .. } => self.types.par_float,
			Pattern::String(_) => self.types.string,
			Pattern::Anonymous => expected,
			Pattern::Missing => self.types.error,
			Pattern::Identifier(i) => {
				// If this is an enum atom, then add a resolution to it
				let res = (|| {
					let p = self.find_variable(scope?, *i)?;
					if let PatternTy::EnumAtom(ty) = self.ctx.type_pattern(db, self.types, p) {
						self.ctx
							.add_pattern_resolution(PatternRef::new(self.item, pat), p);
						return Some(ty);
					}
					None
				})();
				if let Some(ty) = res {
					ty
				} else {
					// This pattern is irrefutable and declares a new variable
					self.ctx.add_declaration(
						PatternRef::new(self.item, pat),
						PatternTy::Variable(expected),
					);
					return expected;
				}
			}
			Pattern::Call {
				function,
				arguments,
			} => {
				let res = (|| {
					let name = self.data[*function].identifier().unwrap();
					let fns = self.find_function(scope?, name);
					let cs = fns
						.iter()
						.find_map(|f| {
							if let PatternTy::EnumConstructor(cs) =
								self.ctx.type_pattern(db, self.types, *f)
							{
								self.ctx
									.add_pattern_resolution(PatternRef::new(self.item, pat), *f);
								Some(cs)
							} else {
								None
							}
						})
						.or_else(|| {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								TypeMismatch {
									src,
									span,
									msg: "Expected enum constructor in pattern call".to_owned(),
								},
							);
							None
						})?;

					// Find the enum constructor via its return type
					let c = cs
						.iter()
						.find(|c| expected.is_subtype_of(db, c.overload.return_type()))
						.or_else(|| {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								NoMatchingFunction {
									src,
									span,
									msg: format!(
										"No enum constructor '{}' found for type '{}'",
										name.lookup(db),
										expected.pretty_print(db)
									),
								},
							);
							None
						})?;

					if c.overload.params().len() != arguments.len() {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							NoMatchingFunction {
								src,
								span,
								msg: "Wrong number of arguments for enum constructor".to_owned(),
							},
						);
					}

					for (p, t) in arguments.iter().zip(
						c.overload
							.params()
							.iter()
							.copied()
							.chain(std::iter::repeat(self.types.error)),
					) {
						self.collect_pattern(scope, *p, t);
					}
					let fn_type = c.overload.clone().into_function().unwrap();
					self.ctx.add_declaration(
						PatternRef::new(self.item, *function),
						PatternTy::Destructuring(Ty::function(db, fn_type)),
					);
					Some(c.overload.return_type())
				})();

				if let Some(ty) = res {
					ty
				} else {
					// Continue collection
					for p in arguments.iter() {
						self.collect_pattern(scope, *p, self.types.error);
					}
					self.types.error
				}
			}
			Pattern::Tuple { fields } => match expected.lookup(db) {
				TyData::Tuple(_, fs) => Ty::tuple(
					db,
					fields
						.iter()
						.zip(
							fs.iter()
								.copied()
								.chain(std::iter::repeat(self.types.error)),
						)
						.map(|(p, e)| self.collect_pattern(scope, *p, e)),
				),
				_ => Ty::tuple(
					db,
					fields
						.iter()
						.map(|p| self.collect_pattern(scope, *p, self.types.error)),
				),
			},
			Pattern::Record { fields } => match expected.lookup(db) {
				TyData::Record(_, fs) => {
					let mut map = FxHashMap::default();
					for (i, f) in fs.iter() {
						map.insert(*i, *f);
					}
					Ty::record(
						db,
						fields.iter().map(|(i, p)| {
							(
								*i,
								self.collect_pattern(
									scope,
									*p,
									map.get(i).copied().unwrap_or(self.types.error),
								),
							)
						}),
					)
				}
				_ => Ty::record(
					db,
					fields
						.iter()
						.map(|(i, p)| (*i, self.collect_pattern(scope, *p, self.types.error))),
				),
			},
		};
		self.ctx.add_declaration(
			PatternRef::new(self.item, pat),
			PatternTy::Destructuring(actual),
		);
		if !actual.is_subtype_of(db, expected) {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						expected.pretty_print(db),
						actual.pretty_print(db),
					),
				},
			);
			return self.types.error;
		}
		actual
	}

	/// Collect an ascribed type `t`, filling in `Any` types with using `ty` if present.
	pub fn complete_type(&mut self, t: ArenaIndex<Type>, ty: Option<Ty>) -> Ty {
		let db = self.db;
		match &self.data[t] {
			Type::Primitive {
				inst,
				opt,
				primitive_type,
			} => {
				let ty = match primitive_type {
					PrimitiveType::Ann => Ty::ann(db),
					PrimitiveType::Bool => Ty::par_bool(db),
					PrimitiveType::Float => Ty::par_float(db),
					PrimitiveType::Int => Ty::par_int(db),
					PrimitiveType::String => Ty::string(db),
				};
				ty.with_inst(db, *inst)
					.unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: inst
									.pretty_print()
									.into_iter()
									.chain([ty.pretty_print(db)])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					})
					.with_opt(db, *opt)
			}
			Type::Bounded { inst, opt, domain } => {
				let mut ty = match &self.data[*domain] {
					Expression::Identifier(i) => {
						if let Some(p) = self.find_variable(*domain, *i) {
							self.ctx.add_identifier_resolution(
								ExpressionRef::new(self.item, *domain),
								p,
							);
							match self.ctx.type_pattern(db, self.types, p) {
								PatternTy::TypeAlias(ty) => ty,
								PatternTy::Variable(ty) => match ty.lookup(db) {
									TyData::Set(VarType::Par, OptType::NonOpt, inner) => inner,
									TyData::Error => self.types.error,
									_ => {
										let (src, span) =
											NodeRef::from(EntityRef::new(db, self.item, *domain))
												.source_span(db);
										self.ctx.add_diagnostic(
											self.item,
											TypeMismatch {
												src,
												span,
												msg: format!(
													"Expected a 'par set' but got {}",
													ty.pretty_print(db)
												),
											},
										);
										return self.types.error;
									}
								},
								PatternTy::TyVar(t) => Ty::type_inst_var(db, t),
								PatternTy::Computing => {
									// Error will be emitted during topological sorting
									return self.types.error;
								}
								_ => {
									let (src, span) =
										NodeRef::from(EntityRef::new(db, self.item, t))
											.source_span(db);
									self.ctx.add_diagnostic(
										self.item,
										TypeMismatch {
											src,
											span,
											msg: "Expected a domain or type alias.".to_owned(),
										},
									);
									return self.types.error;
								}
							}
						} else {
							let (src, span) = NodeRef::from(EntityRef::new(db, self.item, *domain))
								.source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								UndefinedIdentifier {
									identifier: i.lookup(db),
									src,
									span,
								},
							);
							return self.types.error;
						}
					}
					_ => {
						let ty = self.collect_expression(*domain);
						match ty.lookup(db) {
							TyData::Set(VarType::Par, OptType::NonOpt, e) => e,
							TyData::Error => self.types.error,
							_ => {
								let (src, span) =
									NodeRef::from(EntityRef::new(db, self.item, *domain))
										.source_span(db);
								self.ctx.add_diagnostic(
									self.item,
									TypeMismatch {
										src,
										span,
										msg: format!(
											"Expected a 'par set' but got {}",
											ty.pretty_print(db)
										),
									},
								);
								return self.types.error;
							}
						}
					}
				};
				if let Some(inst) = inst {
					ty = ty.with_inst(db, *inst).unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, *domain)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: inst
									.pretty_print()
									.into_iter()
									.chain([ty.pretty_print(db)])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					});
				}
				if let Some(opt) = opt {
					ty = ty.with_opt(db, *opt)
				}
				ty
			}
			Type::Array {
				opt,
				dimensions,
				element,
			} => {
				let (d_ty, e_ty) = match ty.map(|ty| ty.lookup(db)) {
					Some(TyData::Array { dim, element, .. }) => (Some(dim), Some(element)),
					_ => (None, None),
				};
				let dim = self.complete_type(*dimensions, d_ty);
				let element = self.complete_type(*element, e_ty);
				let ty = Ty::array(db, dim, element).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!(
								"array [{}] of {}",
								dim.pretty_print(db),
								element.pretty_print(db)
							),
						},
					);
					self.types.error
				});
				ty.with_opt(db, *opt)
			}
			Type::Set { inst, opt, element } => {
				let e_ty = match ty.map(|ty| ty.lookup(db)) {
					Some(TyData::Set(_, _, element)) => Some(element),
					_ => None,
				};
				let el = self.complete_type(*element, e_ty);
				let ty = Ty::par_set(db, el).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("set of {}", el.pretty_print(db),),
						},
					);
					self.types.error
				});
				ty.with_inst(db, *inst)
					.unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: inst
									.pretty_print()
									.into_iter()
									.chain([ty.pretty_print(db)])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					})
					.with_opt(db, *opt)
			}
			Type::Tuple { opt, fields } => match ty.map(|ty| ty.lookup(db)) {
				Some(TyData::Tuple(_, fs)) => Ty::tuple(
					db,
					fields
						.iter()
						.zip(fs.iter().map(|f| Some(*f)).chain(std::iter::repeat(None)))
						.map(|(f, f_ty)| self.complete_type(*f, f_ty)),
				)
				.with_opt(db, *opt),
				_ => Ty::tuple(db, fields.iter().map(|f| self.complete_type(*f, None)))
					.with_opt(db, *opt),
			},
			Type::Record { opt, fields } => match ty.map(|ty| ty.lookup(db)) {
				Some(TyData::Record(_, fs)) => Ty::record(
					db,
					fields.iter().map(|(i1, t)| {
						let i = self.data[*i1].identifier().unwrap();
						let ty = fs.iter().find(|(i2, _)| i == *i2).map(|(_, t)| *t);
						(i, self.complete_type(*t, ty))
					}),
				)
				.with_opt(db, *opt),
				_ => Ty::record(
					db,
					fields.iter().map(|(i, f)| {
						(
							self.data[*i].identifier().unwrap(),
							self.complete_type(*f, None),
						)
					}),
				)
				.with_opt(db, *opt),
			},
			Type::Operation {
				opt,
				return_type,
				parameter_types,
			} => match ty.map(|ty| ty.lookup(db)) {
				Some(TyData::Function(
					_,
					FunctionType {
						return_type: r,
						params: ps,
					},
				)) => Ty::function(
					db,
					FunctionType {
						return_type: self.complete_type(*return_type, Some(r)),
						params: parameter_types
							.iter()
							.zip(ps.iter().map(|p| Some(*p)).chain(std::iter::repeat(None)))
							.map(|(p, p_ty)| self.complete_type(*p, p_ty))
							.collect(),
					},
				)
				.with_opt(db, *opt),
				_ => Ty::function(
					db,
					FunctionType {
						return_type: self.complete_type(*return_type, None),
						params: parameter_types
							.iter()
							.map(|p| self.complete_type(*p, None))
							.collect(),
					},
				)
				.with_opt(db, *opt),
			},
			Type::AnonymousTypeInstVar {
				inst,
				opt,
				pattern,
				varifiable,
				enumerable,
				indexable,
			} => {
				let mut ty = Ty::type_inst_var(
					db,
					TyVar {
						ty_var: TyVarRef(PatternRef::new(self.item, *pattern)),
						varifiable: *varifiable,
						enumerable: *enumerable,
						indexable: *indexable,
					},
				);
				if let Some(inst) = inst {
					ty = ty.with_inst(db, *inst).unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: inst
									.pretty_print()
									.into_iter()
									.chain([ty.pretty_print(db)])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					});
				}
				if let Some(opt) = opt {
					ty = ty.with_opt(db, *opt);
				}
				ty
			}
			Type::Any => ty.unwrap_or_else(|| {
				let (src, span) = NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeInferenceFailure {
						src,
						span,
						msg: "Unable to infer type".to_owned(),
					},
				);
				self.types.error
			}),
			Type::Missing => self.types.error,
		}
	}

	fn find_variable(
		&self,
		expression: ArenaIndex<Expression>,
		identifier: Identifier,
	) -> Option<PatternRef> {
		let scope = self.db.lookup_item_scope(self.item);
		scope.find_variable(self.db, expression, identifier)
	}

	fn find_function(
		&self,
		expression: ArenaIndex<Expression>,
		identifier: Identifier,
	) -> Arc<Vec<PatternRef>> {
		let scope = self.db.lookup_item_scope(self.item);
		scope.find_function(self.db, expression, identifier)
	}
}
