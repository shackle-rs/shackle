use rustc_hash::FxHashMap;
use std::{collections::hash_map::Entry, fmt::Write, sync::Arc};

use crate::{
	constants::{IdentifierRegistry, TypeRegistry},
	diagnostics::{
		AmbiguousCall, BranchMismatch, IllegalType, InvalidArrayLiteral, InvalidFieldAccess,
		NoMatchingFunction, SyntaxError, TypeInferenceFailure, TypeMismatch, UndefinedIdentifier,
	},
	hir::{
		db::Hir,
		ids::{EntityRef, ExpressionRef, ItemRef, NodeRef, PatternRef},
		ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, Call, Case, Declaration,
		Expression, Generator, Identifier, IfThenElse, IndexedArrayLiteral, ItemData, Lambda, Let,
		LetItem, MaybeIndexSet, Pattern, PrimitiveType, RecordAccess, RecordLiteral,
		SetComprehension, SetLiteral, TupleAccess, TupleLiteral, Type,
	},
	ty::{
		FunctionEntry, FunctionResolutionError, FunctionType, InstantiationError, OptType, Ty,
		TyData, VarType,
	},
	utils::{arena::ArenaIndex, maybe_grow_stack},
	Error,
};

use super::{PatternTy, TypeContext};

/// Mode for completing types
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeCompletionMode {
	/// Give an error if non-primitive types are used
	AnnotationParameter,
	/// Give an error if non-primitive types are used
	Operation,
	/// Give an error if unbounded types are used
	EnumerationParameter,
	/// Allow all types
	Default,
}

/// Result of completing a type
pub struct TypeCompletionResult {
	/// The computed type
	pub ty: Ty,
	/// Whether this type contains a bound
	pub has_bounded: bool,
	/// Whether this type contains an unbounded type
	pub has_unbounded: bool,
}

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
	types: Arc<TypeRegistry>,
	identifiers: Arc<IdentifierRegistry>,
	ctx: &'a mut T,
	item: ItemRef,
	data: &'a ItemData,
	in_output_item: bool,
}

impl<'a, T: TypeContext> Typer<'a, T> {
	/// Create a new typer
	pub fn new(db: &'a dyn Hir, ctx: &'a mut T, item: ItemRef, data: &'a ItemData) -> Self {
		Typer {
			db,
			types: db.type_registry(),
			identifiers: db.identifier_registry(),
			ctx,
			item,
			data,
			in_output_item: false,
		}
	}

	/// Collect the type of an expression and check that it is a subtype of the expected type.
	pub fn typecheck_expression(&mut self, expr: ArenaIndex<Expression>, expected: Ty) -> Ty {
		let db = self.db;
		let actual = self.collect_expression(expr);
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
		actual
	}

	/// Collect the type of an output expression and check that it is a subtype of the expected type.
	pub fn typecheck_output(&mut self, expr: ArenaIndex<Expression>, expected: Ty) {
		let prev = self.in_output_item;
		self.in_output_item = true;
		self.typecheck_expression(expr, expected);
		self.in_output_item = prev;
	}

	/// Get the type of this expression
	pub fn collect_expression(&mut self, expr: ArenaIndex<Expression>) -> Ty {
		maybe_grow_stack(|| self.collect_expression_inner(expr))
	}

	fn collect_expression_inner(&mut self, expr: ArenaIndex<Expression>) -> Ty {
		let db = self.db;
		let result = match &self.data[expr] {
			Expression::Absent => self.types.bottom.make_opt(db.upcast()),
			Expression::BooleanLiteral(_) => self.types.par_bool,
			Expression::IntegerLiteral(_) => self.types.par_int,
			Expression::FloatLiteral(_) => self.types.par_float,
			Expression::StringLiteral(_) => self.types.string,
			Expression::Infinity => self.types.par_int,
			Expression::Identifier(i) => self.collect_identifier(expr, i, None),
			Expression::Call(c) => self.collect_call(expr, c, None),
			Expression::ArrayLiteral(al) => self.collect_array_literal(expr, al),
			Expression::ArrayLiteral2D(al) => self.collect_array_literal_2d(expr, al),
			Expression::IndexedArrayLiteral(al) => self.collect_indexed_array_literal(expr, al),
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
			Expression::Lambda(l) => self.collect_lambda(l),
			Expression::Slice(_) => self.types.set_of_bottom,
			Expression::Missing => self.types.error,
		};
		self.ctx
			.add_expression(ExpressionRef::new(self.item, expr), result);
		self.collect_annotations(expr, result);
		result
	}

	fn collect_annotations(&mut self, expr: ArenaIndex<Expression>, ty: Ty) {
		let db = self.db;
		for ann in self
			.data
			.annotations
			.get(expr)
			.iter()
			.flat_map(|anns| anns.iter())
		{
			self.typecheck_expression(*ann, self.types.ann);
			// If annotation is shackle_type("...") then treat as sanity check for type
			if let Expression::Call(c) = &self.data[*ann] {
				if let Expression::Identifier(i) = &self.data[c.function] {
					if *i == self.identifiers.shackle_type {
						if let Expression::StringLiteral(sl) = &self.data[c.arguments[0]] {
							let expected = sl.value(db);
							let actual = ty.pretty_print(db.upcast());
							if actual != expected {
								let (src, span) =
									NodeRef::from(EntityRef::new(db, self.item, expr))
										.source_span(db);
								self.ctx.add_diagnostic(
									self.item,
									TypeMismatch {
										src,
										span,
										msg: format!(
									"shackle_type: expected computed type '{}' but got '{}'",
									expected,
									actual
								),
									},
								);
							}
						}
					}
				}
			}
		}
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
			match self.ctx.type_pattern(db, p) {
				PatternTy::Variable(ty) => {
					if self.in_output_item && p.item() != self.item {
						return ty.make_par(db.upcast());
					}
					return ty;
				}
				PatternTy::Argument(ty) | PatternTy::Enum(ty) | PatternTy::EnumAtom(ty) => {
					return ty
				}
				PatternTy::AnnotationAtom => return self.types.ann,
				PatternTy::TypeAlias { .. } => {
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
				PatternTy::EnumConstructor { .. } | PatternTy::AnnotationConstructor(_) => (),
				PatternTy::Computing => {
					// Error will be emitted during topological sorting
					return self.types.error;
				}
				pattern_ty => {
					unreachable!(
						"Matched variable in scope, but not a variable or type alias ({:?})",
						pattern_ty
					)
				}
			}
		}

		if let Some(ty) = is_annotation_for {
			// This is an annotation, so look for any matching functions with ::annotated_expression
			let patterns = self.find_function(expr, *i);
			let fn_match = patterns
				.iter()
				.find_map(|p| match self.ctx.type_pattern(db, *p) {
					PatternTy::Function(function) => {
						FunctionEntry::match_fn(db.upcast(), [(*p, *function)], &[ty]).ok()
					}
					_ => None,
				});
			if let Some((p, fe, t)) = fn_match {
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
							let ret = fe.overload.instantiate(db.upcast(), &t).return_type;
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
			.map(|e| self.collect_expression(*e))
			.collect::<Vec<_>>();

		match self.data[c.function] {
			Expression::Identifier(i) => {
				let (op, ret) = self.resolve_overloading(c.function, i, &args, is_annotation_for);
				self.collect_annotations(c.function, op);
				ret
			}
			_ => {
				let ty = self.collect_expression(c.function);
				if let TyData::Function(OptType::NonOpt, f) = ty.lookup(db.upcast()) {
					if f.matches(db.upcast(), &args).is_err() {
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
						return f.return_type;
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
			al.members.iter().map(|e| self.collect_expression(*e)),
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

	fn collect_array_literal_2d(
		&mut self,
		expr: ArenaIndex<Expression>,
		al: &ArrayLiteral2D,
	) -> Ty {
		let db = self.db;
		let mut idx_ty = |dim: &MaybeIndexSet| match dim {
			MaybeIndexSet::Indexed(indices) => Ty::most_specific_supertype(
				db.upcast(),
				indices.iter().map(|e| self.collect_expression(*e)),
			)
			.unwrap_or_else(|| {
				let (src, span) =
					NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					InvalidArrayLiteral {
						src,
						span,
						msg: "Non-uniform array indices".to_owned(),
					},
				);
				self.types.error
			}),
			MaybeIndexSet::NonIndexed(len) => {
				if *len > 0 {
					self.types.par_int
				} else {
					self.types.bottom
				}
			}
		};
		let dim_ty = Ty::tuple(db.upcast(), [idx_ty(&al.rows), idx_ty(&al.columns)]);
		let el_ty = if al.members.is_empty() {
			self.types.bottom
		} else {
			Ty::most_specific_supertype(
				db.upcast(),
				al.members.iter().map(|e| self.collect_expression(*e)),
			)
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
			})
		};
		Ty::array(db.upcast(), dim_ty, el_ty).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim_ty.pretty_print_as_dims(db.upcast()),
						el_ty.pretty_print(db.upcast())
					),
				},
			);
			self.types.error
		})
	}

	fn collect_indexed_array_literal(
		&mut self,
		expr: ArenaIndex<Expression>,
		al: &IndexedArrayLiteral,
	) -> Ty {
		let db = self.db;
		let dim_ty = Ty::most_specific_supertype(
			db.upcast(),
			al.indices.iter().map(|e| self.collect_expression(*e)),
		)
		.unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				InvalidArrayLiteral {
					src,
					span,
					msg: "Non-uniform array indices".to_owned(),
				},
			);
			self.types.error
		});
		let el_ty = if al.members.is_empty() {
			self.types.bottom
		} else {
			Ty::most_specific_supertype(
				db.upcast(),
				al.members.iter().map(|e| self.collect_expression(*e)),
			)
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
			})
		};
		Ty::array(db.upcast(), dim_ty, el_ty).unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim_ty.pretty_print_as_dims(db.upcast()),
						el_ty.pretty_print(db.upcast())
					),
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
			sl.members.iter().map(|e| self.collect_expression(*e)),
		)
		.unwrap_or_else(|| {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				InvalidArrayLiteral {
					src,
					span,
					msg: "Non-uniform set literal".to_owned(),
				},
			);
			self.types.error
		});
		match ty.inst(db.upcast()) {
			Some(VarType::Var) => {
				let ty = ty.make_par(db.upcast());
				Ty::par_set(db.upcast(), ty)
					.and_then(|t| t.make_var(db.upcast()))
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
				self.types.error
			}
		}
	}

	fn collect_tuple_literal(&mut self, tl: &TupleLiteral) -> Ty {
		let db = self.db;
		Ty::tuple(
			db.upcast(),
			tl.fields.iter().map(|f| self.collect_expression(*f)),
		)
	}

	fn collect_record_literal(&mut self, rl: &RecordLiteral) -> Ty {
		let db = self.db;
		let mut fields = FxHashMap::default();
		for (i, f) in rl.fields.iter() {
			let ident = self.data[*i]
				.identifier()
				.expect("Record field name not an identifier");
			match fields.entry(ident) {
				Entry::Vacant(e) => {
					e.insert(self.collect_expression(*f));
				}
				Entry::Occupied(_) => {
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
			}
		}
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
			lift_to_opt |= self.collect_generator(expr, g);
		}
		let el = self.collect_expression(c.template);
		let element = if lift_to_opt {
			el.make_opt(db.upcast())
				.make_var(db.upcast())
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
			.map(|i| self.collect_expression(i))
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
			is_var |= self.collect_generator(expr, g);
		}
		let el = self.collect_expression(c.template);
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

		let element = el.make_par(db.upcast()).make_occurs(db.upcast());
		Ty::par_set(db.upcast(), element)
			.and_then(|ty| {
				if is_var {
					ty.make_var(db.upcast())
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

	fn collect_generator(&mut self, expr: ArenaIndex<Expression>, g: &Generator) -> bool {
		let db = self.db;
		let mut is_var = false;
		let where_clause = match g {
			Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				let collection_ty = self.collect_expression(*collection);
				let gen_el = match collection_ty.lookup(db.upcast()) {
					TyData::Array {
						opt: OptType::NonOpt,
						element,
						..
					}
					| TyData::Set(VarType::Par, OptType::NonOpt, element) => element,
					TyData::Set(VarType::Var, OptType::NonOpt, element) => {
						is_var = true;
						element
					}
					TyData::Error => self.types.error,
					_ => {
						let (src, span) = NodeRef::from(EntityRef::new(db, self.item, *collection))
							.source_span(db);
						self.ctx.add_diagnostic(
							self.item,
							TypeMismatch {
								src,
								span,
								msg: format!(
									"Expected set or array type, but got {}",
									collection_ty.pretty_print(db.upcast())
								),
							},
						);
						self.types.error
					}
				};
				for p in patterns.iter() {
					self.collect_pattern(Some(expr), false, *p, gen_el, false);
				}
				*where_clause
			}
			Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let ty = self.collect_expression(*value);
				self.collect_pattern(Some(expr), false, *pattern, ty, false);
				*where_clause
			}
		};
		if let Some(w) = where_clause {
			let ty = self.collect_expression(w);
			if !ty.is_subtype_of(db.upcast(), self.types.var_bool) {
				let (src, span) = NodeRef::from(EntityRef::new(db, self.item, w)).source_span(db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected boolean where clause, but got {}",
							ty.pretty_print(db.upcast())
						),
					},
				);
			}
			if let Some(VarType::Var) = ty.inst(db.upcast()) {
				is_var = true;
			}
		}
		is_var
	}

	fn collect_array_access(&mut self, expr: ArenaIndex<Expression>, aa: &ArrayAccess) -> Ty {
		let db = self.db;
		let collection = self.collect_expression(aa.collection);
		let indices = self.collect_expression(aa.indices);

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
				dim.make_opt(db.upcast())
					.make_var(db.upcast())
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
						"Expected '{}', but got '{}'",
						dim.pretty_print_as_dims(db.upcast()),
						index.pretty_print(db.upcast())
					),
				}
				.into());
			}

			match index.opt(db.upcast()) {
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
			match index.inst(db.upcast()) {
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
									make_var |= v;
									make_opt |= o;
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
							make_var |= v;
							make_opt |= o;
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
				el.make_var(db.upcast()).unwrap_or_else(|| {
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
				result.make_opt(db.upcast())
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
				result.make_opt(db.upcast())
			} else {
				result
			}
		}
	}

	fn collect_tuple_access(&mut self, expr: ArenaIndex<Expression>, ta: &TupleAccess) -> Ty {
		let db = self.db;
		let tuple = self.collect_expression(ta.tuple);
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
					ty.make_opt(db.upcast())
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
						el.make_opt(db.upcast())
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
					self.types.error
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
		let record = self.collect_expression(ra.record);
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
					ty.make_opt(db.upcast())
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
						el.make_opt(db.upcast())
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
					self.types.error
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
			.map(|b| self.collect_expression(b.condition))
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
			.map(|e| (e, self.collect_expression(e)))
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
								original_span: first_span,
							},
						);
					}
				}
				self.types.error
			});
		if ty.contains_function(db.upcast()) {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				TypeInferenceFailure {
					src,
					span,
					msg:
						"Function types cannot be used in the results of if-then-else expressions."
							.to_owned(),
				},
			);
			return self.types.error;
		}
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
			.flat_map(|t| t.inst(db.upcast()))
			.max()
			.unwrap_or(VarType::Par)
		{
			// Var condition means var result
			ty.make_var(db.upcast()).unwrap_or_else(|| {
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
		let scrutinee = self.collect_expression(c.expression);
		for case in c.cases.iter() {
			self.collect_pattern(Some(expr), true, case.pattern, scrutinee, false);
		}
		let cases = c
			.cases
			.iter()
			.map(|case| (case.value, self.collect_expression(case.value)))
			.collect::<Vec<_>>();
		let ty = Ty::most_specific_supertype(db.upcast(), cases.iter().map(|(_, ty)| *ty))
			.unwrap_or_else(|| {
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
								original_span: first_span,
							},
						);
					}
				}
				self.types.error
			});
		if ty.contains_function(db.upcast()) {
			let (src, span) = NodeRef::from(EntityRef::new(db, self.item, expr)).source_span(db);
			self.ctx.add_diagnostic(
				self.item,
				TypeInferenceFailure {
					src,
					span,
					msg: "Function types cannot be used in the results of case expressions."
						.to_owned(),
				},
			);
			return self.types.error;
		}
		if let Some(VarType::Var) = scrutinee.inst(db.upcast()) {
			ty.make_var(db.upcast()).unwrap_or_else(|| {
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

	fn collect_let(&mut self, l: &Let) -> Ty {
		let db = self.db;
		let mut is_var = false;
		for item in l.items.iter() {
			match item {
				LetItem::Constraint(c) => {
					for ann in c.annotations.iter() {
						self.typecheck_expression(*ann, self.types.ann);
					}
					let ty = self.typecheck_expression(c.expression, self.types.var_bool);
					if ty == self.types.var_bool {
						is_var = true;
					}
				}
				LetItem::Declaration(d) => {
					let ty = self.collect_declaration(d);
					if (ty.contains_par(db.upcast()) || ty.contains_function(db.upcast()))
						&& d.definition.is_none()
					{
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
					if !ty.known_par(db.upcast()) {
						is_var = true;
					}
				}
			}
		}
		let ty = self.collect_expression(l.in_expression);
		if ty == self.types.par_bool && is_var {
			// Becomes var because any var partiality bubbles up to this point
			self.types.var_bool
		} else {
			ty
		}
	}

	/// Type check a declaration
	pub fn collect_declaration(&mut self, d: &Declaration) -> Ty {
		for p in Pattern::identifiers(d.pattern, self.data) {
			self.ctx
				.add_declaration(PatternRef::new(self.item, p), PatternTy::Computing);
		}
		let ty = if let Some(e) = d.definition {
			let actual = self.collect_expression(e);
			let expected = self
				.complete_type(d.declared_type, Some(actual), TypeCompletionMode::Default)
				.ty;
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
			self.complete_type(d.declared_type, None, TypeCompletionMode::Default)
				.ty
		};
		self.collect_pattern(None, false, d.pattern, ty, false);
		for ann in d.annotations.iter() {
			// Handle identifiers/calls which lead to ::annotated_expression functions

			self.typecheck_expression(*ann, self.types.ann);
		}
		ty
	}

	/// Type check a declaration in output mode
	pub fn collect_output_declaration(&mut self, d: &Declaration) -> Ty {
		let prev = self.in_output_item;
		self.in_output_item = true;
		let ty = self.collect_declaration(d);
		self.in_output_item = prev;
		ty
	}

	/// Typecheck an annotation for a declaration (since these may be calls to annotations using ::annotated_expression)
	pub fn typecheck_declaration_annotation(
		&mut self,
		ann: ArenaIndex<Expression>,
		declaration_ty: Ty,
	) {
		let db = self.db;
		let actual = match &self.data[ann] {
			Expression::Identifier(i) => self.collect_identifier(ann, i, Some(declaration_ty)),
			Expression::Call(c) => self.collect_call(ann, c, Some(declaration_ty)),
			_ => self.collect_expression(ann),
		};
		if !actual.is_subtype_of(db.upcast(), self.types.ann) {
			let (src, span) =
				NodeRef::from(EntityRef::new(db, self.item, ann)).source_span(self.db);
			self.ctx.add_diagnostic(
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						self.types.ann.pretty_print(db.upcast()),
						actual.pretty_print(db.upcast())
					),
				},
			);
		}
		self.ctx
			.add_expression(ExpressionRef::new(self.item, ann), actual);
		self.collect_annotations(ann, actual);
	}

	fn collect_lambda(&mut self, l: &Lambda) -> Ty {
		let db = self.db;
		for p in l
			.parameters
			.iter()
			.filter_map(|param| param.pattern)
			.flat_map(|p| Pattern::identifiers(p, self.data))
		{
			self.ctx
				.add_declaration(PatternRef::new(self.item, p), PatternTy::Computing);
		}
		let params = l
			.parameters
			.iter()
			.map(|param| {
				let ty = self
					.complete_type(param.declared_type, None, TypeCompletionMode::Default)
					.ty;
				if let Some(p) = param.pattern {
					self.collect_pattern(None, false, p, ty, true);
				}
				ty
			})
			.collect();
		let body = self.collect_expression(l.body);
		let return_type = if let Some(r) = l.return_type {
			let ret = self
				.complete_type(r, Some(body), TypeCompletionMode::Default)
				.ty;
			if !body.is_subtype_of(db.upcast(), ret) {
				let (src, span) =
					NodeRef::from(EntityRef::new(self.db, self.item, l.body)).source_span(self.db);
				self.ctx.add_diagnostic(
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected '{}' but got '{}'",
							ret.pretty_print(db.upcast()),
							body.pretty_print(db.upcast()),
						),
					},
				);
			}
			ret
		} else {
			body
		};

		Ty::function(
			db.upcast(),
			FunctionType {
				return_type,
				params,
			},
		)
	}

	/// Resolve overloading for the function `expr` that is the identifier `i`.
	///
	/// Returns a tuple of the type of the operation, and the return type
	fn resolve_overloading(
		&mut self,
		expr: ArenaIndex<Expression>,
		i: Identifier,
		args: &[Ty],
		is_annotation_for: Option<Ty>,
	) -> (Ty, Ty) {
		let db = self.db;
		let error = (self.types.error, self.types.error);
		if args.iter().any(|t| t.contains_error(db.upcast())) {
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return error;
		}

		// If there's a variable in scope which is a function, use it
		if let Some(p) = self.find_variable(expr, i) {
			let d = self.ctx.type_pattern(db, p);
			let f = match d {
				PatternTy::Variable(t) | PatternTy::Argument(t) => {
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
					return error;
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
					return error;
				}
				let ret = f.return_type;
				let op = Ty::function(db.upcast(), f);
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), op);
				self.ctx
					.add_identifier_resolution(ExpressionRef::new(self.item, expr), p);
				return (op, ret);
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
			return error;
		}

		let mut overloads = Vec::with_capacity(patterns.len());
		for p in patterns.iter() {
			match self.ctx.type_pattern(db, *p) {
				PatternTy::Function(function)
				| PatternTy::AnnotationConstructor(function)
				| PatternTy::AnnotationDestructure(function) => overloads.push((*p, *function.clone())),
				PatternTy::EnumConstructor(ec) => {
					overloads.extend(ec.iter().map(|ec| (*p, ec.constructor.clone())))
				}
				PatternTy::EnumDestructure(fs) => {
					overloads.extend(fs.iter().map(|f| (*p, f.clone())))
				}
				PatternTy::Computing => (),
				_ => unreachable!(),
			}
		}

		if overloads.is_empty() {
			self.ctx
				.add_expression(ExpressionRef::new(self.item, expr), self.types.error);
			return error;
		}

		let fn_result = FunctionEntry::match_fn(db.upcast(), overloads, args).or_else(|e| {
			if let Some(ty) = is_annotation_for {
				// Also try matching ::annotated_expression functions
				let mut new_args = Vec::with_capacity(args.len() + 1);
				new_args.push(ty);
				new_args.extend(args.iter().copied());

				let mut new_overloads = Vec::new();
				for p in patterns.iter() {
					if let PatternTy::Function(function) = self.ctx.type_pattern(db, *p) {
						if let crate::hir::ids::LocalItemRef::Function(f) =
							&p.item().local_item_ref(db)
						{
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
					}
				}
				return FunctionEntry::match_fn(db.upcast(), new_overloads, &new_args);
			}
			Err(e)
		});

		match fn_result {
			Ok((pattern, fe, tvs)) => {
				let instantiation = fe.overload.instantiate(db.upcast(), &tvs);
				let ret = instantiation.return_type;
				let op = Ty::function(db.upcast(), instantiation);
				self.ctx
					.add_expression(ExpressionRef::new(self.item, expr), op);
				self.ctx
					.add_identifier_resolution(ExpressionRef::new(self.item, expr), pattern);
				(op, ret)
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
				error
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
							if types.is_empty() {
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
				error
			}
		}
	}

	/// Collect the type of a pattern
	pub fn collect_pattern(
		&mut self,
		scope: Option<ArenaIndex<Expression>>,
		resolves_atoms: bool,
		pat: ArenaIndex<Pattern>,
		expected: Ty,
		is_argument: bool,
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
				let res = if resolves_atoms {
					// If this is an enum atom, then add a resolution to it
					(|| {
						let p = self.find_variable(scope?, *i)?;
						match self.ctx.type_pattern(db, p) {
							PatternTy::EnumAtom(ty) => {
								self.ctx
									.add_pattern_resolution(PatternRef::new(self.item, pat), p);
								Some(ty)
							}
							PatternTy::AnnotationAtom => {
								self.ctx
									.add_pattern_resolution(PatternRef::new(self.item, pat), p);
								Some(self.types.ann)
							}
							_ => None,
						}
					})()
				} else {
					None
				};
				if let Some(ty) = res {
					ty
				} else {
					// This pattern declares a new variable
					self.ctx.add_declaration(
						PatternRef::new(self.item, pat),
						if is_argument {
							PatternTy::Argument(expected)
						} else {
							PatternTy::Variable(expected)
						},
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
					let (ctor_pat, cs) = fns
						.iter()
						.find_map(|f| match self.ctx.type_pattern(db, *f) {
							PatternTy::EnumConstructor(ec) => Some((
								*f,
								Vec::from(ec)
									.into_iter()
									.filter_map(|ec| {
										if ec.is_lifted {
											None
										} else {
											Some(ec.constructor)
										}
									})
									.collect::<Box<_>>(),
							)),
							PatternTy::AnnotationConstructor(fe) => {
								Some((*f, Box::new([(*fe).clone()])))
							}
							_ => None,
						})
						.or_else(|| {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, self.item, pat)).source_span(db);
							self.ctx.add_diagnostic(
								self.item,
								TypeMismatch {
									src,
									span,
									msg: "Expected enum or annotation constructor in pattern call"
										.to_owned(),
								},
							);
							None
						})?;

					// Find the enum constructor via its return type
					// If this type is opt, make it non opt as if this call pattern is matched, the value occurs
					let non_opt = expected.make_occurs(db.upcast());
					let c = cs
						.iter()
						.find(|c| non_opt.is_subtype_of(db.upcast(), c.overload.return_type()))
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
						self.collect_pattern(scope, resolves_atoms, *p, t, is_argument);
					}
					let fn_pat = PatternRef::new(self.item, *function);
					self.ctx.add_pattern_resolution(fn_pat, ctor_pat);
					let ctor_type = c.overload.clone().into_function().unwrap();
					let dtor_type = FunctionType {
						params: Box::new([ctor_type.return_type]),
						return_type: if ctor_type.params.len() == 1 {
							ctor_type.params[0]
						} else {
							Ty::tuple(db.upcast(), ctor_type.params.iter().copied())
						},
					};
					self.ctx.add_declaration(
						fn_pat,
						PatternTy::DestructuringFn {
							constructor: Ty::function(db.upcast(), ctor_type),
							destructor: Ty::function(db.upcast(), dtor_type),
						},
					);
					Some(c.overload.return_type())
				})();

				if let Some(ty) = res {
					ty
				} else {
					// Continue collection
					for p in arguments.iter() {
						self.collect_pattern(
							scope,
							resolves_atoms,
							*p,
							self.types.error,
							is_argument,
						);
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
						.map(|(p, e)| {
							self.collect_pattern(scope, resolves_atoms, *p, e, is_argument)
						}),
				),
				_ => Ty::tuple(
					db.upcast(),
					fields.iter().map(|p| {
						self.collect_pattern(
							scope,
							resolves_atoms,
							*p,
							self.types.error,
							is_argument,
						)
					}),
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
									resolves_atoms,
									*p,
									map.get(&i.0).copied().unwrap_or(self.types.error),
									is_argument,
								),
							)
						}),
					)
				}
				_ => Ty::record(
					db.upcast(),
					fields.iter().map(|(i, p)| {
						(
							*i,
							self.collect_pattern(
								scope,
								resolves_atoms,
								*p,
								self.types.error,
								is_argument,
							),
						)
					}),
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
	pub fn complete_type(
		&mut self,
		t: ArenaIndex<Type>,
		ty: Option<Ty>,
		mode: TypeCompletionMode,
	) -> TypeCompletionResult {
		let mut has_bounded = false;
		let mut has_unbounded = false;
		let ty = self.complete_type_inner(t, ty, mode, &mut has_bounded, &mut has_unbounded);
		TypeCompletionResult {
			ty,
			has_bounded,
			has_unbounded,
		}
	}

	fn complete_type_inner(
		&mut self,
		t: ArenaIndex<Type>,
		ty: Option<Ty>,
		mode: TypeCompletionMode,
		has_bounded: &mut bool,
		has_unbounded: &mut bool,
	) -> Ty {
		let db = self.db;

		let mut set_bounded = |typer: &mut Self, domain: ArenaIndex<Expression>| {
			*has_bounded = true;
			match mode {
				TypeCompletionMode::AnnotationParameter => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, typer.item, domain)).source_span(db);
					typer.ctx.add_diagnostic(
						typer.item,
						TypeMismatch {
							src,
							span,
							msg: "Bounded domains are not supported in annotation parameters."
								.to_owned(),
						},
					);
				}
				TypeCompletionMode::Operation => {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, typer.item, domain)).source_span(db);
					typer.ctx.add_diagnostic(
						typer.item,
						TypeMismatch {
							src,
							span,
							msg: "Bounded domains are not \
supported in operation types."
								.to_owned(),
						},
					);
				}
				_ => (),
			}
		};

		let mut set_unbounded = |typer: &mut Self| {
			*has_unbounded = true;
			if let TypeCompletionMode::EnumerationParameter = mode {
				let (src, span) = NodeRef::from(EntityRef::new(db, typer.item, t)).source_span(db);
				typer.ctx.add_diagnostic(
					typer.item,
					TypeMismatch {
						src,
						span,
						msg: "Unbounded enumeration constructor \
						parameters are not supported"
							.to_owned(),
					},
				);
			}
		};

		match &self.data[t] {
			Type::Primitive {
				inst,
				opt,
				primitive_type,
			} => {
				set_unbounded(self);
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
							match self.ctx.type_pattern(db, p) {
								PatternTy::TypeAlias {
									ty,
									has_bounded: b,
									has_unbounded: ub,
								} => {
									if b {
										set_bounded(self, *domain);
									}
									if ub {
										set_unbounded(self);
									}
									ty
								}
								PatternTy::Variable(ty) | PatternTy::Argument(ty) => match ty
									.lookup(db.upcast())
								{
									TyData::Set(VarType::Par, OptType::NonOpt, inner) => {
										set_bounded(self, *domain);
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
								PatternTy::Enum(ty) => match ty.lookup(db.upcast()) {
									TyData::Set(VarType::Par, OptType::NonOpt, inner) => {
										// Don't set has_bounded or has_unbounded as enums are accepted
										// everywhere
										self.ctx.add_expression(domain_ref, ty);
										inner
									}
									TyData::Error => self.types.error,
									_ => unreachable!(),
								},
								PatternTy::TyVar(t) => {
									*has_unbounded = true;
									Ty::type_inst_var(db.upcast(), t)
								}
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
						let ty = self.collect_expression(*domain);
						match ty.lookup(db.upcast()) {
							TyData::Set(VarType::Par, OptType::NonOpt, e) => {
								set_bounded(self, *domain);
								e
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
				let dim =
					self.complete_type_inner(*dimensions, d_ty, mode, has_bounded, has_unbounded);
				let element =
					self.complete_type_inner(*element, e_ty, mode, has_bounded, has_unbounded);
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
				let el = self.complete_type_inner(*element, e_ty, mode, has_bounded, has_unbounded);
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
						.map(|(f, f_ty)| {
							self.complete_type_inner(*f, f_ty, mode, has_bounded, has_unbounded)
						}),
				)
				.with_opt(db.upcast(), *opt),
				_ => Ty::tuple(
					db.upcast(),
					fields.iter().map(|f| {
						self.complete_type_inner(*f, None, mode, has_bounded, has_unbounded)
					}),
				)
				.with_opt(db.upcast(), *opt),
			},
			Type::Record { opt, fields: fs } => {
				let mut fields = FxHashMap::default();
				for (p, t) in fs.iter() {
					let i = self.data[*p]
						.identifier()
						.expect("Record field not an identifier");
					match fields.entry(i) {
						Entry::Vacant(e) => {
							e.insert(*t);
						}
						Entry::Occupied(_) => {
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
					}
				}
				Ty::record(
					db.upcast(),
					fields.into_iter().map(|(i, f)| {
						(
							i,
							self.complete_type_inner(
								f,
								ty.and_then(|ty| match ty.lookup(db.upcast()) {
									TyData::Record(_, fs) => {
										fs.iter().find(|(i2, _)| i.0 == *i2).map(|(_, t)| *t)
									}
									_ => None,
								}),
								mode,
								has_bounded,
								has_unbounded,
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
			} => {
				if let TypeCompletionMode::AnnotationParameter = mode {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeMismatch {
							src,
							span,
							msg: "Operation types are are not supported in \
									annotation item parameters"
								.to_owned(),
						},
					);
				}
				match ty.map(|ty| ty.lookup(db.upcast())) {
					Some(TyData::Function(
						_,
						FunctionType {
							return_type: r,
							params: ps,
						},
					)) => Ty::function(
						db.upcast(),
						FunctionType {
							return_type: self.complete_type_inner(
								*return_type,
								Some(r),
								TypeCompletionMode::Operation,
								has_bounded,
								has_unbounded,
							),
							params: parameter_types
								.iter()
								.zip(ps.iter().map(|p| Some(*p)).chain(std::iter::repeat(None)))
								.map(|(p, p_ty)| {
									self.complete_type_inner(
										*p,
										p_ty,
										TypeCompletionMode::Operation,
										has_bounded,
										has_unbounded,
									)
								})
								.collect(),
						},
					)
					.with_opt(db.upcast(), *opt),
					_ => Ty::function(
						db.upcast(),
						FunctionType {
							return_type: self.complete_type_inner(
								*return_type,
								None,
								TypeCompletionMode::Operation,
								has_bounded,
								has_unbounded,
							),
							params: parameter_types
								.iter()
								.map(|p| {
									self.complete_type_inner(
										*p,
										None,
										TypeCompletionMode::Operation,
										has_bounded,
										has_unbounded,
									)
								})
								.collect(),
						},
					)
					.with_opt(db.upcast(), *opt),
				}
			}
			Type::AnonymousTypeInstVar { inst, opt, pattern } => {
				*has_unbounded = true;
				let mut ty = Ty::type_inst_var(
					db.upcast(),
					match self
						.ctx
						.type_pattern(db, PatternRef::new(self.item, *pattern))
					{
						PatternTy::TyVar(tv) => tv,
						_ => unimplemented!(),
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
			Type::Any => {
				*has_unbounded = true;
				ty.and_then(|ty| {
					if ty.contains_bottom(db.upcast()) {
						// Not allowed to use bottom type for any
						None
					} else {
						Some(ty)
					}
				})
				.unwrap_or_else(|| {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, self.item, t)).source_span(db);
					self.ctx.add_diagnostic(
						self.item,
						TypeInferenceFailure {
							src,
							span,
							msg: "Unable to infer type".to_owned(),
						},
					);
					self.types.error
				})
			}
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
