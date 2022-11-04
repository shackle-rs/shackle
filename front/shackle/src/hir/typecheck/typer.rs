use rustc_hash::{FxHashMap, FxHashSet};
use std::{fmt::Write, sync::Arc};

use crate::{
	arena::ArenaIndex,
	error::{
		AmbiguousCall, BranchMismatch, IllegalType, InvalidArrayLiteral, InvalidFieldAccess,
		NoMatchingFunction, SyntaxError, TypeInferenceFailure, TypeMismatch, UndefinedIdentifier,
	},
	hir::{
		db::Hir,
		ids::{EntityRef, ExpressionRef, ItemRef, NodeRef, PatternRef},
		ArrayAccess, ArrayComprehension, ArrayLiteral, Call, Case, Declaration, Expression,
		Identifier, IdentifierRegistry, IfThenElse, ItemData, Let, LetItem, Pattern, PrimitiveType,
		RecordAccess, RecordLiteral, SetComprehension, SetLiteral, TupleAccess, TupleLiteral, Type,
	},
	ty::{
		FunctionEntry, FunctionResolutionError, FunctionType, InstantiationError, OptType, Ty,
		TyData, TyVar, TyVarRef, TypeRegistry, VarType,
	},
	Error,
};

use super::{PatternTy, TypeContext};

/// Computes types of expressions and patterns in an item.
///
/// The typer walks an expression tree and computes types of child nodes to
/// determine the types of parent nodes. The exception to this is when computing
/// the type of a `Call`, in which case we need to perform overloading
/// resolution (so we type the identifier being called at this point since we
/// have the arguments).
///
/// Errors have to be handled in a way so as to not require aborting compilation
/// entirely. To achieve this, the `TyData::Error` type is used to signal that
/// a type could not be computed. When creating an error type
/// (`self.types.error`) a diagnostic must be emitted. This sentinel then
/// bubbles up during type checking, but allows us to suppress further errors
/// which are just caused by the original error we already reported.
pub struct Typer<'a, T> {
	db: &'a dyn Hir,
	types: &'a TypeRegistry,
	identifiers: &'a IdentifierRegistry,
	ctx: &'a mut T,
	item: ItemRef,
	data: &'a ItemData,
}

impl<'a, T: TypeContext> Typer<'a, T> {
	/// Create a new typer
	pub fn new(
		db: &'a dyn Hir,
		types: &'a TypeRegistry,
		identifiers: &'a IdentifierRegistry,
		ctx: &'a mut T,
		item: ItemRef,
		data: &'a ItemData,
	) -> Self {
		Typer {
			db,
			types,
			identifiers,
			ctx,
			item,
			data,
		}
	}

	/// Collect the type of an expression and check that it is a subtype of the expected type.
	pub fn typecheck_expression(
		&mut self,
		expr: ArenaIndex<Expression>,
		expected: Ty,
		is_annotation_for: Option<Ty>,
	) {
		let db = self.db;
		let actual = self.collect_expression(expr, is_annotation_for);
		if !actual.is_subtype_of(self.db.upcast(), expected) {
			let (src, span) =
				NodeRef::from(EntityRef::new(self.db, self.item, expr)).source_span(self.db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						expected.pretty_print(db.upcast()),
						actual.pretty_print(db.upcast())
					),
				},
			);
		}
	}

	/// Get the type of this expression
	pub fn collect_expression(
		&mut self,
		expr: ArenaIndex<Expression>,
		is_annotation_for: Option<Ty>,
	) -> Ty {
		let db = self.db;
		let result = match &self.data[expr] {
			Expression::Absent => self.types.bottom.with_opt(db.upcast(), OptType::Opt),
			Expression::BooleanLiteral(_) => self.types.par_bool,
			Expression::IntegerLiteral(_) => self.types.par_int,
			Expression::FloatLiteral(_) => self.types.par_float,
			Expression::StringLiteral(_) => self.types.string,
			Expression::Infinity => self.types.par_int,
			Expression::Identifier(i) => self.collect_identifier(expr, i, is_annotation_for),
			Expression::Call(c) => self.collect_call(expr, c, is_annotation_for),
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
		for ann in self
			.data
			.annotations
			.get(expr)
			.iter()
			.flat_map(|anns| anns.iter())
		{
			self.typecheck_expression(*ann, self.types.ann, Some(result));
		}
		result
	}

	fn collect_identifier(
		&mut self,
		expr: ArenaIndex<Expression>,
		i: &Identifier,
		is_annotation_for: Option<Ty>,
	) -> Ty {
		let db = self.db;
		if let Some(p) = self.find_variable(expr, *i) {
			let expression = ExpressionRef::new(self.item, expr);
			self.ctx.add_identifier_resolution(expression, p);
			match self.ctx.type_pattern(db, self.types, self.identifiers, p) {
				PatternTy::Variable(ty) | PatternTy::EnumAtom(ty) => return ty,
				PatternTy::AnnotationAtom => return self.types.ann,
				PatternTy::TypeAlias(_) => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							msg: "Unexpected type alias".to_owned(),
							src,
							span,
						},
					);
					return self.types.error;
				}
				PatternTy::EnumConstructor(_) | PatternTy::AnnotationConstructor(_) => (),
				PatternTy::Computing => {
					// Error will be emitted during topological sorting
					return self.types.error;
				}
				_ => {
					unreachable!("Matched variable in scope, but not a variable or type alias")
				}
			}
		}

		if let Some(ty) = is_annotation_for {
			// This is an annotation, so look for any matching functions with ::annotated_expression
			let patterns = self.find_function(expr, *i);
			let fn_match = patterns.iter().find_map(|p| {
				match self.ctx.type_pattern(db, self.types, self.identifiers, *p) {
					PatternTy::Function(function) => {
						FunctionEntry::match_fn(db.upcast(), [(*p, *function.clone())], &[ty]).ok()
					}
					_ => None,
				}
			});
			if let Some((p, _, t)) = fn_match {
				match p.item().local_item_ref(db) {
					crate::hir::ids::LocalItemRef::Function(f) => {
						let fi = &p.item().model(db)[f];
						let has_annotated_expression =
							fi.parameters[0]
								.annotations
								.iter()
								.any(|ann| match &fi.data[*ann] {
									Expression::Identifier(i) => {
										*i == self.identifiers.annotated_expression
									}
									_ => false,
								});
						if has_annotated_expression {
							let ret = t.return_type;
							self.ctx
								.add_identifier_resolution(ExpressionRef::new(self.item, expr), p);
							return ret;
						}
					}
					_ => unreachable!(),
				}
			}
		}

		let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
		self.ctx.add_diagnostic(
			self.item,
			UndefinedIdentifier {
				identifier: i.pretty_print(db),
				src,
				span,
			},
		);
		self.types.error
	}

	fn collect_call(
		&mut self,
		expr: ArenaIndex<Expression>,
		c: &Call,
		is_annotation_for: Option<Ty>,
	) -> Ty {
		let db = self.db;
		let args = c
			.arguments
			.iter()
			.map(|e| self.collect_expression(*e, None))
			.collect::<Vec<_>>();

		match self.data[c.function] {
			Expression::Identifier(i) => {
				self.resolve_overloading(c.function, i, &args, is_annotation_for)
			}
			_ => {
				let ty = self.collect_expression(c.function, None);
				if let TyData::Function(OptType::NonOpt, f) = ty.lookup(db.upcast()) {
					if let Err(_) = f.matches(db.upcast(), &args) {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							NoMatchingFunction {
								src,
								span,
								msg: format!(
									"Cannot call function with signature '{}' with arguments {}",
									f.pretty_print(db.upcast()),
									args.iter()
										.map(|a| format!("'{}'", a.pretty_print(db.upcast())))
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
						msg: format!("Type '{}' is not callable", ty.pretty_print(db.upcast())),
					},
				);
				self.types.error
			}
		}
	}

	fn collect_array_literal(&mut self, expr: ArenaIndex<Expression>, al: &ArrayLiteral) -> Ty {
		let db = self.db;
		if al.members.is_empty() {
			return self.types.array_of_bottom;
		}
		let ty = Ty::most_specific_supertype(
			db.upcast(),
			al.members.iter().map(|e| self.collect_expression(*e, None)),
		)
		.unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
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
		Ty::array(db.upcast(), self.types.par_int, ty).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!("array [..] of {}", ty.pretty_print(db.upcast())),
				},
			);
			self.types.error
		})
	}

	fn collect_set_literal(&mut self, expr: ArenaIndex<Expression>, sl: &SetLiteral) -> Ty {
		let db = self.db;
		if sl.members.is_empty() {
			return Ty::par_set(db.upcast(), self.types.bottom).unwrap();
		}
		let ty = Ty::most_specific_supertype(
			db.upcast(),
			sl.members.iter().map(|e| self.collect_expression(*e, None)),
		)
		.unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
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
		match ty.inst(db.upcast()) {
			Some(VarType::Var) => {
				let ty = ty.with_inst(db.upcast(), VarType::Par).unwrap();
				Ty::par_set(db.upcast(), ty)
					.and_then(|t| t.with_inst(db.upcast(), VarType::Var))
					.unwrap_or_else(|| {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							IllegalType {
								src,
								span,
								ty: format!("var set of {}", ty.pretty_print(db.upcast())),
							},
						);
						self.types.error
					})
			}
			Some(VarType::Par) => Ty::par_set(db.upcast(), ty).unwrap_or_else(|| {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					IllegalType {
						src,
						span,
						ty: format!("set of {}", ty.pretty_print(db.upcast())),
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
		Ty::tuple(
			db.upcast(),
			tl.fields.iter().map(|f| self.collect_expression(*f, None)),
		)
	}

	fn collect_record_literal(&mut self, rl: &RecordLiteral) -> Ty {
		let db = self.db;
		let mut seen = FxHashSet::default();
		let mut fields = rl
			.fields
			.iter()
			.map(|(i, f)| {
				let ident = self.data[*i]
					.identifier()
					.expect("Record field name not an identifier");
				if seen.contains(&ident) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, *i)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						SyntaxError {
							src,
							span,
							msg: format!(
								"Record literal contains duplicate field '{}'",
								ident.pretty_print(db)
							),
							other: Vec::new(),
						},
					);
				}
				seen.insert(ident);
				(ident, self.collect_expression(*f, None))
			})
			.collect::<Vec<_>>();
		fields.sort_by_key(|(i, _)| i.lookup(db));
		fields.dedup_by_key(|(i, _)| *i);
		Ty::record(db.upcast(), fields)
	}

	fn collect_array_comprehension(
		&mut self,
		expr: ArenaIndex<Expression>,
		c: &ArrayComprehension,
	) -> Ty {
		let db = self.db;
		let mut lift_to_opt = false;
		for g in c.generators.iter() {
			let collection = self.collect_expression(g.collection, None);
			let gen_el = match collection.lookup(db.upcast()) {
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
								collection.pretty_print(db.upcast())
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
				let where_clause = self.collect_expression(w, None);
				if let Some(VarType::Var) = where_clause.inst(db.upcast()) {
					lift_to_opt = true;
				}
				if !where_clause.is_subtype_of(db.upcast(), self.types.var_bool) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db.upcast())
							),
						},
					);
				}
			}
		}
		let el = self.collect_expression(c.template, None);
		let element = if lift_to_opt {
			el.with_opt(db.upcast(), OptType::Opt)
				.with_inst(db.upcast(), VarType::Var)
				.unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("array [..] of var opt {}", el.pretty_print(db.upcast())),
						},
					);
					self.types.error
				})
		} else {
			el
		};
		let dim = c
			.indices
			.map(|i| self.collect_expression(i, None))
			.unwrap_or(self.types.par_int);
		Ty::array(db.upcast(), dim, element).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim.pretty_print_as_dims(db.upcast()),
						element.pretty_print(db.upcast())
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
			let collection = self.collect_expression(g.collection, None);
			let gen_el = match collection.lookup(db.upcast()) {
				TyData::Array {
					opt: OptType::NonOpt,
					element,
					..
				} => match element.inst(db.upcast()) {
					Some(VarType::Var) => {
						is_var = true;
						element.with_inst(db.upcast(), VarType::Par).unwrap()
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
								collection.pretty_print(db.upcast())
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
				let where_clause = self.collect_expression(w, None);
				if let Some(VarType::Var) = where_clause.inst(db.upcast()) {
					is_var = true;
				}
				if !where_clause.is_subtype_of(db.upcast(), self.types.var_bool) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, g.collection)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected set or array type, but got {}",
								collection.pretty_print(db.upcast())
							),
						},
					);
				}
			}
		}
		let el = self.collect_expression(c.template, None);
		if !is_var {
			// Inst determined by el inst
			match el.inst(db.upcast()) {
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
								el.pretty_print(db.upcast())
							),
						},
					);
					return self.types.error;
				}
			}
		};

		let element = el.with_inst(db.upcast(), VarType::Par).unwrap();
		Ty::par_set(db.upcast(), element)
			.and_then(|ty| {
				if is_var {
					ty.with_inst(db.upcast(), VarType::Var)
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
							element.pretty_print(db.upcast())
						),
					},
				);
				self.types.error
			})
	}

	fn collect_array_access(&mut self, expr: ArenaIndex<Expression>, aa: &ArrayAccess) -> Ty {
		let db = self.db;
		let collection = self.collect_expression(aa.collection, None);
		let indices = self.collect_expression(aa.indices, None);

		let process_index = |index: Ty, dim: Ty| -> Result<_, Error> {
			let mut make_var = false;
			let mut make_opt = false;
			if let TyData::Set(i1, o1, t) = index.lookup(db.upcast()) {
				if !t.is_subtype_of(db.upcast(), dim) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, aa.indices)).source_span(db);
					return Err(TypeMismatch {
						src,
						span,
						msg: format!(
							"Cannot slice index of type {} using {}",
							dim.pretty_print(db.upcast()),
							index.pretty_print(db.upcast())
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
				db.upcast(),
				dim.with_opt(db.upcast(), OptType::Opt)
					.with_inst(db.upcast(), VarType::Var)
					.unwrap_or_else(|| {
						panic!(
							"Array dimension {} should be varifiable",
							dim.pretty_print(db.upcast()),
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
						dim.pretty_print(db.upcast()),
						index.pretty_print(db.upcast())
					),
				}
				.into());
			}

			match indices.opt(db.upcast()) {
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
			match indices.inst(db.upcast()) {
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
		let el = match collection.lookup(db.upcast()) {
			TyData::Array { opt, dim, element } => {
				make_opt = make_opt || opt == OptType::Opt;
				match (indices.lookup(db.upcast()), dim.lookup(db.upcast())) {
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
							collection.pretty_print(db.upcast())
						),
					},
				);
				return self.types.error;
			}
		};

		if slices.is_empty() {
			let result = if make_var {
				el.with_inst(db.upcast(), VarType::Var).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("var {}", el.pretty_print(db.upcast())),
						},
					);
					self.types.error
				})
			} else {
				el
			};
			if make_opt {
				result.with_opt(db.upcast(), OptType::Opt)
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
				db.upcast(),
				if slices.len() > 1 {
					Ty::tuple(db.upcast(), slices)
				} else {
					slices[0]
				},
				el,
			)
			.unwrap();
			if make_opt {
				result.with_opt(db.upcast(), OptType::Opt)
			} else {
				result
			}
		}
	}

	fn collect_tuple_access(&mut self, expr: ArenaIndex<Expression>, ta: &TupleAccess) -> Ty {
		let db = self.db;
		let tuple = self.collect_expression(ta.tuple, None);
		match tuple.lookup(db.upcast()) {
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
							msg: format!(
								"No such field {} for '{}'",
								i,
								tuple.pretty_print(db.upcast())
							),
						},
					);
					return self.types.error;
				}
				let ty = fields[(i - 1) as usize];
				if let OptType::Opt = opt {
					ty.with_opt(db.upcast(), OptType::Opt)
				} else {
					ty
				}
			}
			TyData::Array {
				opt: o1,
				dim,
				element,
			} => match element.lookup(db.upcast()) {
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
									element.pretty_print(db.upcast())
								),
							},
						);
						return self.types.error;
					}
					let el = fields[(i - 1) as usize];
					let ty = if let OptType::Opt = o1.max(o2) {
						el.with_opt(db.upcast(), OptType::Opt)
					} else {
						el
					};
					Ty::array(db.upcast(), dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print_as_dims(db.upcast()),
							ty.pretty_print(db.upcast())
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
								tuple.pretty_print(db.upcast())
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
							"Expected tuple type, but got '{}'",
							tuple.pretty_print(db.upcast())
						),
					},
				);
				self.types.error
			}
		}
	}

	fn collect_record_access(&mut self, expr: ArenaIndex<Expression>, ra: &RecordAccess) -> Ty {
		let db = self.db;
		let record = self.collect_expression(ra.record, None);
		match record.lookup(db.upcast()) {
			TyData::Record(opt, fields) => {
				let ty = fields
					.iter()
					.find(|(i, _)| *i == ra.field.0)
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
									ra.field.pretty_print(db),
									record.pretty_print(db.upcast())
								),
							},
						);
						self.types.error
					});
				if let OptType::Opt = opt {
					ty.with_opt(db.upcast(), OptType::Opt)
				} else {
					ty
				}
			}
			TyData::Array {
				opt: o1,
				dim,
				element,
			} => match element.lookup(db.upcast()) {
				TyData::Record(o2, fields) => {
					let el = fields
						.iter()
						.find(|(i, _)| *i == ra.field.0)
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
										ra.field.pretty_print(db),
										element.pretty_print(db.upcast())
									),
								},
							);
							self.types.error
						});
					let ty = if let OptType::Opt = o1.max(o2) {
						el.with_opt(db.upcast(), OptType::Opt)
					} else {
						el
					};
					Ty::array(db.upcast(), dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print_as_dims(db.upcast()),
							ty.pretty_print(db.upcast())
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
								record.pretty_print(db.upcast())
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
							record.pretty_print(db.upcast())
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
			.map(|b| self.collect_expression(b.condition, None))
			.collect::<Vec<_>>();
		for (t, b) in condition_types.iter().zip(ite.branches.iter()) {
			if !t.is_subtype_of(db.upcast(), self.types.var_bool) {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, b.condition)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected boolean condition, but got '{}'",
							t.pretty_print(db.upcast())
						),
					},
				);
			}
		}
		let result_types = ite
			.branches
			.iter()
			.map(|b| b.result)
			.chain(ite.else_result)
			.map(|e| (e, self.collect_expression(e, None)))
			.collect::<Vec<_>>();
		let ty = Ty::most_specific_supertype(db.upcast(), result_types.iter().map(|(_, ty)| *ty))
			.unwrap_or_else(|| {
				let mut expr_tys = result_types.into_iter();
				let (first_expr, first_ty) = expr_tys.next().unwrap();
				let (_, first_span) =
					NodeRef::from(EntityRef::new(db, self.item, first_expr)).source_span(db);
				for (expr, ty) in expr_tys {
					if Ty::most_specific_supertype(db.upcast(), [first_ty, ty]).is_none() {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							BranchMismatch {
								msg: format!(
									"Mismatch in if-then-else branch types. Expected type compatible with '{}' but got '{}'",
									first_ty.pretty_print(db.upcast()),
									ty.pretty_print(db.upcast())
								),
								src,
								span,
								original_span: first_span.clone(),
							},
						);
					}
				}
				self.types.error
			});
		if ite.else_result.is_none() && !ty.has_default_value(db.upcast()) {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"If-then expression with branch type '{}' must have an else",
						ty.pretty_print(db.upcast())
					),
				},
			);
		}
		if let VarType::Var = condition_types
			.iter()
			.map(|t| t.inst(db.upcast()).unwrap())
			.max()
			.unwrap()
		{
			// Var condition means var result
			ty.with_inst(db.upcast(), VarType::Var).unwrap_or_else(|| {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					IllegalType {
						src,
						span,
						ty: format!("var {}", ty.pretty_print(db.upcast())),
					},
				);
				self.types.error
			})
		} else {
			ty
		}
	}

	fn collect_case(&mut self, expr: ArenaIndex<Expression>, c: &Case) -> Ty {
		let db = self.db;
		let scrutinee = self.collect_expression(c.expression, None);
		for case in c.cases.iter() {
			self.collect_pattern(Some(expr), case.pattern, scrutinee);
		}
		let cases = c
			.cases
			.iter()
			.map(|case| (case.value, self.collect_expression(case.value, None)))
			.collect::<Vec<_>>();
		Ty::most_specific_supertype(db.upcast(), cases.iter().map(|(_, ty)| *ty)).unwrap_or_else(
			|| {
				let mut expr_tys = cases.into_iter();
				let (first_expr, first_ty) = expr_tys.next().unwrap();
				let (_, first_span) =
					NodeRef::from(EntityRef::new(db, self.item, first_expr)).source_span(db);
				for (expr, ty) in expr_tys {
					if Ty::most_specific_supertype(db.upcast(), [first_ty, ty]).is_none() {
						let (src, span) =
							NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							BranchMismatch {
								msg: format!(
									"Mismatch in case arm types. Expected type compatible with '{}' but got '{}'",
									first_ty.pretty_print(db.upcast()),
									ty.pretty_print(db.upcast())
								),
								src,
								span,
								original_span: first_span.clone(),
							},
						);
					}
				}
				self.types.error
			},
		)
	}

	fn collect_let(&mut self, l: &Let) -> Ty {
		let db = self.db;
		for item in l.items.iter() {
			match item {
				LetItem::Constraint(c) => {
					for ann in c.annotations.iter() {
						self.typecheck_expression(*ann, self.types.ann, None);
					}
					self.typecheck_expression(c.expression, self.types.var_bool, None);
				}
				LetItem::Declaration(d) => {
					let ty = self.collect_declaration(d);
					if ty.contains_par(db.upcast()) && d.definition.is_none() {
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
		self.collect_expression(l.in_expression, None)
	}

	/// Type check a declaration
	pub fn collect_declaration(&mut self, d: &Declaration) -> Ty {
		for p in Pattern::identifiers(d.pattern, self.data) {
			self.ctx
				.add_declaration(PatternRef::new(self.item, p), PatternTy::Computing);
		}
		let ty = if let Some(e) = d.definition {
			let actual = self.collect_expression(e, None);
			let expected = self.complete_type(d.declared_type, Some(actual));
			if !actual.is_subtype_of(self.db.upcast(), expected) {
				let (src, span) =
					NodeRef::from(EntityRef::new(self.db, self.item, e)).source_span(self.db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected '{}' but got '{}'",
							expected.pretty_print(self.db.upcast()),
							actual.pretty_print(self.db.upcast())
						),
					},
				);
			}
			expected
		} else {
			self.complete_type(d.declared_type, None)
		};
		self.collect_pattern(None, d.pattern, ty);
		for ann in d.annotations.iter() {
			self.typecheck_expression(*ann, self.types.ann, Some(ty));
		}
		ty
	}

	fn resolve_overloading(
		&mut self,
		expr: ArenaIndex<Expression>,
		i: Identifier,
		args: &[Ty],
		is_annotation_for: Option<Ty>,
	) -> Ty {
		let db = self.db;
		if args.iter().any(|t| t.contains_error(db.upcast())) {
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return self.types.error;
		}

		// If there's a variable in scope which is a function, use it
		if let Some(p) = self.find_variable(expr, i) {
			let d = self.ctx.type_pattern(db, self.types, self.identifiers, p);
			let f = match d {
				PatternTy::Variable(t) => {
					if let TyData::Function(OptType::NonOpt, f) = t.lookup(db.upcast()) {
						Some(f)
					} else {
						None
					}
				}
				_ => None,
			};
			if let Some(f) = f {
				if f.contains_error(db.upcast()) {
					return self.types.error;
				}
				if let Err(e) = f.matches(db.upcast(), args) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
					let mut msg = format!(
						"Cannot call function with signature '{}'",
						f.pretty_print(db.upcast())
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
								expected.pretty_print(db.upcast()),
								actual.pretty_print(db.upcast())
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
				self.ctx.add_expression(
					ExpressionRef::new(self.item, expr),
					Ty::function(db.upcast(), f),
				);
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
					msg: format!(
						"No function with name '{}' could be found.",
						i.pretty_print(db)
					),
				},
			);
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return self.types.error;
		}

		let mut overloads = Vec::with_capacity(patterns.len());
		for p in patterns.iter() {
			match self.ctx.type_pattern(db, self.types, self.identifiers, *p) {
				PatternTy::Function(function) | PatternTy::AnnotationConstructor(function) => {
					overloads.push((*p, *function.clone()))
				}
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

		let fn_result = FunctionEntry::match_fn(db.upcast(), overloads, args).or_else(|e| {
			if let Some(ty) = is_annotation_for {
				// Also try matching ::annotated_expression functions
				let mut new_args = Vec::with_capacity(args.len() + 1);
				new_args.push(ty);
				new_args.extend(args.iter().copied());

				let mut new_overloads = Vec::new();
				for p in patterns.iter() {
					match self.ctx.type_pattern(db, self.types, self.identifiers, *p) {
						PatternTy::Function(function) => match &p.item().local_item_ref(db) {
							crate::hir::ids::LocalItemRef::Function(f) => {
								let fi = &p.item().model(db)[*f];
								if let Some(param) = fi.parameters.first() {
									let has_annotated_expression =
										param.annotations.iter().any(|ann| match &fi.data[*ann] {
											Expression::Identifier(i) => {
												*i == self.identifiers.annotated_expression
											}
											_ => false,
										});
									if has_annotated_expression {
										new_overloads.push((*p, *function.clone()));
									}
								}
							}
							_ => (),
						},
						_ => (),
					}
				}
				return FunctionEntry::match_fn(db.upcast(), new_overloads, &new_args);
			}
			Err(e)
		});

		match fn_result {
			Ok((pattern, _, instantiation)) => {
				let ret = instantiation.return_type;
				let ty = Ty::function(db.upcast(), instantiation);
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
						.map(|t| format!("'{}'", t.pretty_print(db.upcast())))
						.collect::<Vec<_>>()
						.join(", ")
				);
				writeln!(
					&mut msg,
					"Could not choose an overload from the candidate functions:"
				)
				.unwrap();
				for (_, f) in ps.iter() {
					writeln!(
						&mut msg,
						"  {}",
						f.overload.pretty_print_item(db.upcast(), i)
					)
					.unwrap();
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
						i.pretty_print(db)
					)
					.unwrap();
				} else {
					writeln!(
						&mut msg,
						"No function '{}' matching argument types {} could be found.",
						i.pretty_print(db),
						args.iter()
							.map(|t| format!("'{}'", t.pretty_print(db.upcast())))
							.collect::<Vec<_>>()
							.join(", ")
					)
					.unwrap();
				}
				writeln!(&mut msg, "The following overloads could not be used:").unwrap();
				for (_, f, e) in es.iter() {
					writeln!(
						&mut msg,
						"  {}",
						f.overload.pretty_print_item(db.upcast(), i)
					)
					.unwrap();
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
								expected.pretty_print(db.upcast()),
								actual.pretty_print(db.upcast())
							)
							.unwrap();
						}
						InstantiationError::IncompatibleTypeInstVariable { ty_var, types } => {
							if types.len() == 0 {
								// Should not be possible currently
								writeln!(
									&mut msg,
									"    Type-inst parameter '{}' not instantiated",
									ty_var.pretty_print(db.upcast())
								)
								.unwrap();
							} else {
								let tys = types
									.iter()
									.map(|t| format!("'{}'", t.pretty_print(db.upcast())))
									.collect::<Vec<_>>()
									.join(", ");
								writeln!(
                                    &mut msg,
                                    "    Type-inst parameter '{}' instantiated with incompatible types {}",
                                    ty_var.pretty_print(db.upcast()),
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
					match self.ctx.type_pattern(db, self.types, self.identifiers, p) {
						PatternTy::EnumAtom(ty) => {
							self.ctx
								.add_pattern_resolution(PatternRef::new(self.item, pat), p);
							return Some(ty);
						}
						PatternTy::AnnotationAtom => {
							self.ctx
								.add_pattern_resolution(PatternRef::new(self.item, pat), p);
							return Some(self.types.ann);
						}
						_ => None,
					}
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
					let cs =
						fns.iter()
							.find_map(|f| {
								match self.ctx.type_pattern(db, self.types, self.identifiers, *f) {
									PatternTy::EnumConstructor(cs) => {
										self.ctx.add_pattern_resolution(
											PatternRef::new(self.item, pat),
											*f,
										);
										Some(cs)
									}
									PatternTy::AnnotationConstructor(fe) => {
										self.ctx.add_pattern_resolution(
											PatternRef::new(self.item, pat),
											*f,
										);
										Some(Box::new([(*fe).clone()]))
									}
									_ => None,
								}
							})
							.or_else(|| {
								let (src, span) = NodeRef::from(EntityRef::new(db, self.item, pat))
									.source_span(db);
								self.ctx.add_diagnostic(
								self.item,
								TypeMismatch {
									src,
									span,
									msg: "Expected enum or annotation constructor in pattern call".to_owned(),
								},
							);
								None
							})?;

					// Find the enum constructor via its return type
					let c = cs
						.iter()
						.find(|c| expected.is_subtype_of(db.upcast(), c.overload.return_type()))
						.or_else(|| {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								NoMatchingFunction {
									src,
									span,
									msg: format!(
										"No constructor '{}' found for type '{}'",
										name.pretty_print(db),
										expected.pretty_print(db.upcast())
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
								msg: format!(
									"Constructor expected {} arguments, but got {}",
									c.overload.params().len(),
									arguments.len()
								),
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
						PatternTy::Destructuring(Ty::function(db.upcast(), fn_type)),
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
			Pattern::Tuple { fields } => match expected.lookup(db.upcast()) {
				TyData::Tuple(_, fs) => Ty::tuple(
					db.upcast(),
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
					db.upcast(),
					fields
						.iter()
						.map(|p| self.collect_pattern(scope, *p, self.types.error)),
				),
			},
			Pattern::Record { fields } => match expected.lookup(db.upcast()) {
				TyData::Record(_, fs) => {
					let mut map = FxHashMap::default();
					for (i, f) in fs.iter() {
						map.insert(*i, *f);
					}
					Ty::record(
						db.upcast(),
						fields.iter().map(|(i, p)| {
							(
								*i,
								self.collect_pattern(
									scope,
									*p,
									map.get(&i.0).copied().unwrap_or(self.types.error),
								),
							)
						}),
					)
				}
				_ => Ty::record(
					db.upcast(),
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
		if !actual.is_subtype_of(db.upcast(), expected) {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						expected.pretty_print(db.upcast()),
						actual.pretty_print(db.upcast()),
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
					PrimitiveType::Ann => Ty::ann(db.upcast()),
					PrimitiveType::Bool => Ty::par_bool(db.upcast()),
					PrimitiveType::Float => Ty::par_float(db.upcast()),
					PrimitiveType::Int => Ty::par_int(db.upcast()),
					PrimitiveType::String => Ty::string(db.upcast()),
				};
				ty.with_inst(db.upcast(), *inst)
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
									.chain([ty.pretty_print(db.upcast())])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					})
					.with_opt(db.upcast(), *opt)
			}
			Type::Bounded { inst, opt, domain } => {
				let mut ty = match &self.data[*domain] {
					Expression::Identifier(i) => {
						if let Some(p) = self.find_variable(*domain, *i) {
							let domain_ref = ExpressionRef::new(self.item, *domain);
							self.ctx.add_identifier_resolution(domain_ref, p);
							match self.ctx.type_pattern(db, self.types, self.identifiers, p) {
								PatternTy::TypeAlias(ty) => ty,
								PatternTy::Variable(ty) => match ty.lookup(db.upcast()) {
									TyData::Set(VarType::Par, OptType::NonOpt, inner) => {
										self.ctx.add_expression(domain_ref, ty);
										inner
									}
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
													ty.pretty_print(db.upcast())
												),
											},
										);
										return self.types.error;
									}
								},
								PatternTy::TyVar(t) => Ty::type_inst_var(db.upcast(), t),
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
									identifier: i.pretty_print(db),
									src,
									span,
								},
							);
							return self.types.error;
						}
					}
					_ => {
						let ty = self.collect_expression(*domain, None);
						match ty.lookup(db.upcast()) {
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
											ty.pretty_print(db.upcast())
										),
									},
								);
								return self.types.error;
							}
						}
					}
				};
				if let Some(inst) = inst {
					ty = ty.with_inst(db.upcast(), *inst).unwrap_or_else(|| {
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
									.chain([ty.pretty_print(db.upcast())])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					});
				}
				if let Some(opt) = opt {
					ty = ty.with_opt(db.upcast(), *opt)
				}
				ty
			}
			Type::Array {
				opt,
				dimensions,
				element,
			} => {
				let (d_ty, e_ty) = match ty.map(|ty| ty.lookup(db.upcast())) {
					Some(TyData::Array { dim, element, .. }) => (Some(dim), Some(element)),
					_ => (None, None),
				};
				let dim = self.complete_type(*dimensions, d_ty);
				let element = self.complete_type(*element, e_ty);
				let ty = Ty::array(db.upcast(), dim, element).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!(
								"array [{}] of {}",
								dim.pretty_print_as_dims(db.upcast()),
								element.pretty_print(db.upcast())
							),
						},
					);
					self.types.error
				});
				ty.with_opt(db.upcast(), *opt)
			}
			Type::Set { inst, opt, element } => {
				let e_ty = match ty.map(|ty| ty.lookup(db.upcast())) {
					Some(TyData::Set(_, _, element)) => Some(element),
					_ => None,
				};
				let el = self.complete_type(*element, e_ty);
				let ty = Ty::par_set(db.upcast(), el).unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						IllegalType {
							src,
							span,
							ty: format!("set of {}", el.pretty_print(db.upcast()),),
						},
					);
					self.types.error
				});
				ty.with_inst(db.upcast(), *inst)
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
									.chain([ty.pretty_print(db.upcast())])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					})
					.with_opt(db.upcast(), *opt)
			}
			Type::Tuple { opt, fields } => match ty.map(|ty| ty.lookup(db.upcast())) {
				Some(TyData::Tuple(_, fs)) => Ty::tuple(
					db.upcast(),
					fields
						.iter()
						.zip(fs.iter().map(|f| Some(*f)).chain(std::iter::repeat(None)))
						.map(|(f, f_ty)| self.complete_type(*f, f_ty)),
				)
				.with_opt(db.upcast(), *opt),
				_ => Ty::tuple(
					db.upcast(),
					fields.iter().map(|f| self.complete_type(*f, None)),
				)
				.with_opt(db.upcast(), *opt),
			},
			Type::Record { opt, fields } => {
				let mut seen = FxHashSet::default();
				let mut fields = fields
					.iter()
					.map(|(p, t)| {
						let i = self.data[*p]
							.identifier()
							.expect("Record field not an identifier");
						if seen.contains(&i) {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, *p)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								SyntaxError {
									src,
									span,
									msg: format!(
										"Record type contains duplicate field '{}'",
										i.pretty_print(db)
									),
									other: Vec::new(),
								},
							);
						}
						seen.insert(i);
						(i, *t)
					})
					.collect::<Vec<_>>();
				fields.sort_by_key(|(i, _)| i.lookup(db));
				fields.dedup_by_key(|(i, _)| *i);
				Ty::record(
					db.upcast(),
					fields.into_iter().map(|(i, f)| {
						(
							i,
							self.complete_type(
								f,
								ty.and_then(|ty| match ty.lookup(db.upcast()) {
									TyData::Record(_, fs) => {
										fs.iter().find(|(i2, _)| i.0 == *i2).map(|(_, t)| *t)
									}
									_ => None,
								}),
							),
						)
					}),
				)
				.with_opt(db.upcast(), *opt)
			}
			Type::Operation {
				opt,
				return_type,
				parameter_types,
			} => match ty.map(|ty| ty.lookup(db.upcast())) {
				Some(TyData::Function(
					_,
					FunctionType {
						return_type: r,
						params: ps,
					},
				)) => Ty::function(
					db.upcast(),
					FunctionType {
						return_type: self.complete_type(*return_type, Some(r)),
						params: parameter_types
							.iter()
							.zip(ps.iter().map(|p| Some(*p)).chain(std::iter::repeat(None)))
							.map(|(p, p_ty)| self.complete_type(*p, p_ty))
							.collect(),
					},
				)
				.with_opt(db.upcast(), *opt),
				_ => Ty::function(
					db.upcast(),
					FunctionType {
						return_type: self.complete_type(*return_type, None),
						params: parameter_types
							.iter()
							.map(|p| self.complete_type(*p, None))
							.collect(),
					},
				)
				.with_opt(db.upcast(), *opt),
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
					db.upcast(),
					TyVar {
						ty_var: TyVarRef::new(db, PatternRef::new(self.item, *pattern)),
						varifiable: *varifiable,
						enumerable: *enumerable,
						indexable: *indexable,
					},
				);
				if let Some(inst) = inst {
					ty = ty.with_inst(db.upcast(), *inst).unwrap_or_else(|| {
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
									.chain([ty.pretty_print(db.upcast())])
									.collect::<Vec<_>>()
									.join(" "),
							},
						);
						self.types.error
					});
				}
				if let Some(opt) = opt {
					ty = ty.with_opt(db.upcast(), *opt);
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
