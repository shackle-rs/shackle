use std::{collections::hash_map::Entry, fmt::Write};

use shackle_diagnostics::{
	AmbiguousCall, BranchMismatch, Error, IllegalOutputType, IllegalType, InvalidArrayLiteral,
	InvalidCall, InvalidDeclaration, InvalidFieldAccess, NoMatchingFunction, SyntaxError,
	TypeInferenceFailure, TypeMismatch, UndefinedIdentifier, UnsupportedObjectFeature,
};
use shackle_ty::{
	ClassRef, FunctionResolutionError, FunctionType, InstantiationError, OptType, Ty, TyData,
	VarType, match_fn, registry::TypeRegistry,
};
use shackle_utils::{
	hash::{Map, Set},
	maybe_grow_stack,
};

use super::{PatternTy, TypeContext};
use crate::{
	ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, Call, Case, ClassMember, Db,
	Declaration, Expression, ExpressionId, Generator, Identifier, IfThenElse, IndexedArrayLiteral,
	Item, ItemData, Lambda, Let, LetItem, MaybeIndexSet, Pattern, PatternId, PrimitiveType,
	RecordAccess, RecordLiteral, SetComprehension, SetLiteral, TupleAccess, TupleLiteral, Type,
	TypeId,
	class_analysis::{class_item_for, class_pattern_for, var_reached_classes},
	constants::IdentifierRegistry,
	ids::{ExpressionRef, PatternRef, TypeRef},
	overloading::{NamedArgumentResoler, OverloadEliminationReason},
};

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
#[derive(Debug)]
pub struct TypeCompletionResult<'db> {
	/// The computed type
	pub ty: Ty<'db>,
	/// Whether this type contains a bound
	pub has_bounded: bool,
	/// Whether this type contains a var bound
	pub has_var_bounded: bool,
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
#[derive(Debug)]
pub struct Typer<'ctx, 'db, T> {
	db: &'db dyn Db,
	types: &'db TypeRegistry<'db>,
	identifiers: &'db IdentifierRegistry<'db>,
	ctx: &'ctx mut T,
	item: Item<'db>,
	data: &'ctx ItemData<'db>,
	in_output_item: bool,
}

impl<'ctx, 'db, T: TypeContext<'db>> Typer<'ctx, 'db, T> {
	/// Create a new typer
	pub fn new(
		db: &'db dyn Db,
		ctx: &'ctx mut T,
		item: impl Into<Item<'db>>,
		data: &'ctx ItemData<'db>,
	) -> Self {
		Typer {
			db,
			types: TypeRegistry::lookup(db),
			identifiers: IdentifierRegistry::lookup(db),
			ctx,
			item: item.into(),
			data,
			in_output_item: false,
		}
	}

	/// Collect the type of an expression and check that it is a subtype of the expected type.
	pub fn typecheck_expression(&mut self, expr: ExpressionId<'db>, expected: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		let item = self.item;
		let actual = self.collect_expression(expr);
		if !actual.is_subtype_of(db, expected) {
			let (src, span) = ExpressionRef::new(db, item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				item,
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
		actual
	}

	/// Collect the type of an output expression and check that it is a subtype of the expected type.
	pub fn typecheck_output(&mut self, expr: ExpressionId<'db>, expected: Ty<'db>) {
		let prev = self.in_output_item;
		self.in_output_item = true;
		let _ = self.typecheck_expression(expr, expected);
		self.in_output_item = prev;
	}

	/// Get the type of this expression
	pub fn collect_expression(&mut self, expr: ExpressionId<'db>) -> Ty<'db> {
		maybe_grow_stack(|| self.collect_expression_inner(expr))
	}

	fn collect_expression_inner(&mut self, expr: ExpressionId<'db>) -> Ty<'db> {
		let db = self.db;
		let result = match &self.data[expr] {
			Expression::Absent => self.types.bottom.make_opt(db),
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
		if self.in_output_item && !result.known_par(db) {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				IllegalOutputType {
					src,
					span,
					ty: result.pretty_print(db),
				},
			);
		}
		self.ctx.add_expression(db, expr, result);
		self.collect_annotations(expr, result);
		result
	}

	fn collect_annotations(&mut self, expr: ExpressionId<'db>, ty: Ty<'db>) {
		let db = self.db;
		for ann in self
			.data
			.annotations
			.get(expr)
			.iter()
			.flat_map(|anns| anns.iter())
		{
			let _ = self.typecheck_expression(*ann, self.types.ann);
			// If annotation is shackle_type("...") then treat as sanity check for type
			if let Expression::Call(c) = &self.data[*ann]
				&& let Expression::Identifier(i) = &self.data[c.function]
				&& *i == self.identifiers.annotations.shackle_type
				&& let Expression::StringLiteral(sl) = &self.data[c.arguments[0]]
			{
				let expected = sl.value(db);
				let actual = ty.pretty_print(db);
				if actual != expected {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
						self.item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"shackle_type: expected computed type '{}' but got '{}'",
								expected, actual
							),
						},
					);
				}
			}
		}
	}

	fn collect_identifier(
		&mut self,
		expr: ExpressionId<'db>,
		i: &Identifier<'db>,
		is_annotation_for: Option<Ty<'db>>,
	) -> Ty<'db> {
		let db = self.db;
		if let Some(p) = self.find_variable(expr, *i) {
			self.ctx.add_identifier_resolution(db, expr, p);
			match self.ctx.type_pattern(db, p) {
				PatternTy::Variable(ty) => {
					if self.in_output_item && p.item(db) != self.item {
						return ty.make_par(db);
					}
					return self.varify_class_body_attribute(p, *i, ty);
				}
				PatternTy::Argument(ty) | PatternTy::Enum(ty) | PatternTy::EnumAtom(ty) => {
					return ty;
				}
				PatternTy::AnnotationAtom => return self.types.ann,
				PatternTy::TypeAlias { .. } => {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
				PatternTy::ClassDecl {
					defining_set_ty, ..
				} => {
					// In a value position, a class name denotes the set of its objects
					if self.in_output_item {
						// Same par-ification as the Variable arm above. Only the
						// SET is fixed — the objects' attributes may still live in var
						// storage, so `c.x` for `c in C` may remain a var read.
						return defining_set_ty.make_par(db);
					}
					return defining_set_ty;
				}
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
			let overloads = patterns
				.iter()
				.filter_map(|p| match self.ctx.type_pattern(db, *p) {
					PatternTy::Function(function) if function.has_annotated_expression => {
						Some((*p, function.overload))
					}
					_ => None,
				});
			if let Ok(r) = match_fn(db, overloads, &[ty]) {
				self.ctx.add_identifier_resolution(db, expr, r.function.0);
				return self.types.ann;
			}
		}

		let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
		self.ctx.add_diagnostic(
			db,
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
		expr: ExpressionId<'db>,
		c: &Call<'db>,
		is_annotation_for: Option<Ty<'db>>,
	) -> Ty<'db> {
		let db = self.db;
		let mut args = Vec::with_capacity(c.arguments.len() + c.named_arguments.len());
		for arg in c.arguments.iter() {
			args.push(self.collect_expression(*arg));
		}
		for (_, arg) in c.named_arguments.iter() {
			args.push(self.collect_expression(*arg));
		}
		let arg_names = c
			.named_arguments
			.iter()
			.map(|(name, _)| self.data[*name].identifier().unwrap())
			.collect::<Vec<_>>();
		match self.data[c.function] {
			Expression::Identifier(i) => {
				let (op, ret) =
					self.resolve_overloading(c.function, i, &args, &arg_names, is_annotation_for);
				self.collect_annotations(c.function, op);
				ret
			}
			_ => {
				let ty = self.collect_expression(c.function);
				if let TyData::Function(OptType::NonOpt, f) = ty.lookup(db) {
					if f.matches(db, &args).is_err() {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
						return f.return_type;
					}
				}

				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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

	fn collect_array_literal(
		&mut self,
		expr: ExpressionId<'db>,
		al: &ArrayLiteral<'db>,
	) -> Ty<'db> {
		let db = self.db;
		if al.members.is_empty() {
			return self.types.array_of_bottom;
		}
		let ty =
			Ty::most_specific_supertype(db, al.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				IllegalType {
					src,
					span,
					ty: format!("array [..] of {}", ty.pretty_print(db)),
				},
			);
			self.types.error
		})
	}

	fn collect_array_literal_2d(
		&mut self,
		expr: ExpressionId<'db>,
		al: &ArrayLiteral2D<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let mut idx_ty = |dim: &MaybeIndexSet<'db>| match dim {
			MaybeIndexSet::Indexed(indices) => {
				Ty::most_specific_supertype(db, indices.iter().map(|e| self.collect_expression(*e)))
					.unwrap_or_else(|| {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							InvalidArrayLiteral {
								src,
								span,
								msg: "Non-uniform array indices".to_owned(),
							},
						);
						self.types.error
					})
			}
			MaybeIndexSet::NonIndexed(len) => {
				if *len > 0 {
					self.types.par_int
				} else {
					self.types.bottom
				}
			}
		};
		let dim_ty = Ty::tuple(db, [idx_ty(&al.rows), idx_ty(&al.columns)]);
		let el_ty = if al.members.is_empty() {
			self.types.bottom
		} else {
			Ty::most_specific_supertype(db, al.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
		Ty::array(db, dim_ty, el_ty).unwrap_or_else(|| {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim_ty.pretty_print_as_dims(db),
						el_ty.pretty_print(db)
					),
				},
			);
			self.types.error
		})
	}

	fn collect_indexed_array_literal(
		&mut self,
		expr: ExpressionId<'db>,
		al: &IndexedArrayLiteral<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let dim_ty =
			Ty::most_specific_supertype(db, al.indices.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
			Ty::most_specific_supertype(db, al.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
		Ty::array(db, dim_ty, el_ty).unwrap_or_else(|| {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim_ty.pretty_print_as_dims(db),
						el_ty.pretty_print(db)
					),
				},
			);
			self.types.error
		})
	}

	fn collect_set_literal(&mut self, expr: ExpressionId<'db>, sl: &SetLiteral<'db>) -> Ty<'db> {
		let db = self.db;
		if sl.members.is_empty() {
			return Ty::par_set(db, self.types.bottom).unwrap();
		}
		let ty =
			Ty::most_specific_supertype(db, sl.members.iter().map(|e| self.collect_expression(*e)))
				.unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
						self.item,
						InvalidArrayLiteral {
							src,
							span,
							msg: "Non-uniform set literal".to_owned(),
						},
					);
					self.types.error
				});
		match ty.inst(db) {
			Some(VarType::Var) => {
				let ty = ty.make_par(db);
				Ty::par_set(db, ty)
					.and_then(|t| t.make_var(db))
					.unwrap_or_else(|| {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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

	fn collect_tuple_literal(&mut self, tl: &TupleLiteral<'db>) -> Ty<'db> {
		let db = self.db;
		Ty::tuple(db, tl.fields.iter().map(|f| self.collect_expression(*f)))
	}

	fn collect_record_literal(&mut self, rl: &RecordLiteral<'db>) -> Ty<'db> {
		let db = self.db;
		let mut fields = Map::default();
		for (i, f) in rl.fields.iter() {
			let ident = self.data[*i]
				.identifier()
				.expect("Record field name not an identifier");
			let field_ty = match fields.entry(ident) {
				Entry::Vacant(e) => *(e.insert(self.collect_expression(*f))),
				Entry::Occupied(_) => {
					let (src, span) = PatternRef::new(db, self.item, *i).source_span(db);
					self.ctx.add_diagnostic(
						db,
						self.item,
						SyntaxError {
							src,
							span,
							msg: format!(
								"Record literal contains duplicate field '{}'",
								ident.pretty_print(db)
							),
						},
					);
					self.types.error
				}
			};
			self.ctx
				.add_declaration(db, *i, PatternTy::RecordField(field_ty));
		}
		Ty::record(db, fields)
	}

	fn collect_array_comprehension(
		&mut self,
		expr: ExpressionId<'db>,
		c: &ArrayComprehension<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let mut lift_to_opt = false;
		for g in c.generators.iter() {
			lift_to_opt |= self.collect_generator(expr, g);
		}
		let el = self.collect_expression(c.template);
		let element = if lift_to_opt {
			el.make_opt(db).make_var(db).unwrap_or_else(|| {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				IllegalType {
					src,
					span,
					ty: format!(
						"array [{}] of {}",
						dim.pretty_print_as_dims(db),
						element.pretty_print(db)
					),
				},
			);
			self.types.error
		})
	}

	fn collect_set_comprehension(
		&mut self,
		expr: ExpressionId<'db>,
		c: &SetComprehension<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let mut is_var = false;
		for g in c.generators.iter() {
			is_var |= self.collect_generator(expr, g);
		}
		let el = self.collect_expression(c.template);
		if !is_var {
			// Inst determined by el inst
			match el.inst(db) {
				Some(VarType::Var) => is_var = true,
				Some(VarType::Par) => (),
				None => {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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

		let element = el.make_par(db).make_occurs(db);
		Ty::par_set(db, element)
			.and_then(|ty| if is_var { ty.make_var(db) } else { Some(ty) })
			.unwrap_or_else(|| {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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

	fn collect_generator(&mut self, expr: ExpressionId<'db>, g: &Generator<'db>) -> bool {
		let db = self.db;
		let mut is_var = false;
		let where_clause = match g {
			Generator::Iterator {
				patterns,
				collection,
				where_clause,
			} => {
				let collection_ty = self.collect_expression(*collection);
				let gen_el = match collection_ty.lookup(db) {
					TyData::Array {
						opt: OptType::NonOpt,
						element,
						..
					}
					| TyData::Set(VarType::Par, OptType::NonOpt, element) => *element,
					TyData::Set(VarType::Var, OptType::NonOpt, element) => {
						is_var = true;
						// A `var set of new C` means both var membership and var
						// attribute storage. Varify the binding for class elements so
						// that field access through the bound name sees var storage:
						// iterating `a.bs` where `a` is a `var new A` must bind a
						// `var` object.
						if let TyData::Class(VarType::Par, _, _) = element.lookup(db) {
							element.make_var(db).unwrap_or(*element)
						} else {
							*element
						}
					}
					TyData::Error => self.types.error,
					_ => {
						let (src, span) =
							ExpressionRef::new(db, self.item, *collection).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							TypeMismatch {
								src,
								span,
								msg: format!(
									"Expected set or array type, but got {}",
									collection_ty.pretty_print(db)
								),
							},
						);
						self.types.error
					}
				};
				for p in patterns.iter() {
					let _ = self.collect_pattern(Some(expr), false, *p, gen_el, false);
				}
				*where_clause
			}
			Generator::Assignment {
				pattern,
				value,
				where_clause,
			} => {
				let ty = self.collect_expression(*value);
				let _ = self.collect_pattern(Some(expr), false, *pattern, ty, false);
				*where_clause
			}
		};
		if let Some(w) = where_clause {
			let ty = self.collect_expression(w);
			if !ty.is_subtype_of(db, self.types.var_bool) {
				let (src, span) = ExpressionRef::new(db, self.item, w).source_span(db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected boolean where clause, but got {}",
							ty.pretty_print(db)
						),
					},
				);
			}
			if let Some(VarType::Var) = ty.inst(db) {
				is_var = true;
			}
		}
		is_var
	}

	fn collect_array_access(&mut self, expr: ExpressionId<'db>, aa: &ArrayAccess<'db>) -> Ty<'db> {
		let db = self.db;
		let collection = self.collect_expression(aa.collection);
		let indices = self.collect_expression(aa.indices);

		let process_index = |index: Ty<'db>, dim: Ty| -> Result<_, Error> {
			let mut make_var = false;
			let mut make_opt = false;
			if let TyData::Set(i1, o1, t) = index.lookup(db) {
				if !t.is_subtype_of(db, dim) {
					let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
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
				if *i1 == VarType::Var {
					let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
					return Err(TypeMismatch {
						src,
						span,
						msg: "Slicing using variable range not supported".to_owned(),
					}
					.into());
				}
				if *o1 == OptType::Opt {
					let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
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
				dim.make_opt(db).make_var(db).unwrap_or_else(|| {
					panic!(
						"Array dimension {} should be varifiable",
						dim.pretty_print(db),
					)
				}),
			) {
				let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
				return Err(TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}', but got '{}'",
						dim.pretty_print_as_dims(db),
						index.pretty_print(db)
					),
				}
				.into());
			}

			match index.opt(db) {
				Some(OptType::Opt) => {
					make_opt = true;
				}
				None => {
					let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
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
			match index.inst(db) {
				Some(VarType::Var) => {
					make_var = true;
				}
				None => {
					let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
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
				make_opt = make_opt || *opt == OptType::Opt;
				match (indices.lookup(db), dim.lookup(db)) {
					(TyData::Tuple(o1, f1), TyData::Tuple(o2, f2)) => {
						make_opt = make_opt || *o1 == OptType::Opt || *o2 == OptType::Opt;
						if f1.len() != f2.len() {
							let (src, span) =
								ExpressionRef::new(db, self.item, aa.indices).source_span(db);
							self.ctx.add_diagnostic(
								db,
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
									self.ctx.add_diagnostic(db, self.item, e);
									return self.types.error;
								}
							}
						}
					}
					_ => match process_index(indices, *dim) {
						Ok((v, o, s)) => {
							make_var |= v;
							make_opt |= o;
							if s {
								slices.push(*dim);
							}
						}
						Err(e) => {
							self.ctx.add_diagnostic(db, self.item, e);
							return self.types.error;
						}
					},
				}
				element
			}
			TyData::Error => return self.types.error,
			_ => {
				let (src, span) = ExpressionRef::new(db, self.item, aa.collection).source_span(db);
				self.ctx.add_diagnostic(
					db,
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
				el.make_var(db).unwrap_or_else(|| {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
				*el
			};
			if make_opt {
				result.make_opt(db)
			} else {
				result
			}
		} else {
			if make_var {
				let (src, span) = ExpressionRef::new(db, self.item, aa.indices).source_span(db);
				self.ctx.add_diagnostic(
					db,
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
				*el,
			)
			.unwrap();
			if make_opt {
				result.make_opt(db)
			} else {
				result
			}
		}
	}

	fn collect_tuple_access(&mut self, expr: ExpressionId<'db>, ta: &TupleAccess<'db>) -> Ty<'db> {
		let db = self.db;
		let tuple = self.collect_expression(ta.tuple);
		let ty = match tuple.lookup(db) {
			TyData::Tuple(opt, fields) => {
				let i = self.data[ta.field].integer_value().unwrap();
				if i < 1 || i > fields.len() as i64 {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
					ty.make_opt(db)
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
					let i = self.data[ta.field].integer_value().unwrap();
					if i < 1 || i > fields.len() as i64 {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
						el.make_opt(db)
					} else {
						el
					};
					Ty::array(db, *dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print_as_dims(db),
							ty.pretty_print(db)
						)
					})
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
					self.types.error
				}
			},
			TyData::Error => self.types.error,
			_ => {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!("Expected tuple type, but got '{}'", tuple.pretty_print(db)),
					},
				);
				self.types.error
			}
		};

		self.ctx
			.add_declaration(db, ta.field, PatternTy::TupleField(ty));
		ty
	}

	/// Find an attribute of a class, searching the superclass chain.
	///
	/// Returns the class which declares the attribute along with its type.
	fn lookup_class_attribute(
		&mut self,
		class: ClassRef<'db>,
		field: Identifier<'db>,
	) -> Option<(ClassRef<'db>, Ty<'db>)> {
		let db = self.db;
		for c in class.superclasses(db).collect::<Vec<_>>() {
			let pattern = class_pattern_for(db, c)?;
			let PatternTy::ClassDecl { attributes, .. } = self.ctx.type_pattern(db, pattern) else {
				continue;
			};
			if let Some((_, ty)) = attributes.iter().find(|(name, _)| *name == field) {
				return Some((c, *ty));
			}
		}
		None
	}

	/// Varify an array-typed class attribute read through a var receiver.
	///
	/// There is no `var array` type, so the array cannot be varified as a whole.
	/// When the attribute's index sets are the same for every object of the class
	/// — its declared dimensions mention neither another attribute nor `this` —
	/// the read is an array of var elements, so it varifies element-wise. A ragged
	/// attribute (`array [1..x] of int` with `x` an attribute) has no such form:
	/// its read-back would need a decision-dependent index set, so it stays
	/// rejected, as do multi-dimensional attributes.
	fn varify_array_class_attribute(
		&mut self,
		class: ClassRef<'db>,
		field: Identifier<'db>,
		ty: Ty<'db>,
	) -> Option<Ty<'db>> {
		let db = self.db;
		let TyData::Array {
			opt: OptType::NonOpt,
			dim,
			element,
		} = ty.lookup(db)
		else {
			return None;
		};
		let var_element = element.make_var(db)?;
		let class_item = class_item_for(db, class)?;
		let c = class_item.class(db);
		let data = c.data();
		let decl = c.items.iter().find_map(|member| match member {
			ClassMember::Declaration(d) if data[d.pattern].identifier() == Some(field) => Some(d),
			_ => None,
		})?;
		let Type::Array { dimensions, .. } = data[decl.declared_type] else {
			return None;
		};
		if matches!(data[dimensions], Type::Tuple { .. }) {
			return None;
		}
		// The names visible inside the class body: its own and inherited attributes,
		// plus `this`. A dimension mentioning any of them varies per object.
		let mut attribute_names = Set::default();
		let _ = attribute_names.insert(self.identifiers.names.this);
		for c in class.superclasses(db).collect::<Vec<_>>() {
			let Some(pattern) = class_pattern_for(db, c) else {
				continue;
			};
			if let PatternTy::ClassDecl { attributes, .. } = self.ctx.type_pattern(db, pattern) {
				attribute_names.extend(attributes.iter().map(|(name, _)| *name));
			}
		}
		for domain in Type::expressions(dimensions, data) {
			for e in Expression::walk(domain, data) {
				if let Expression::Identifier(identifier) = &data[e]
					&& attribute_names.contains(identifier)
				{
					return None;
				}
			}
		}
		Ty::array(db, *dim, var_element)
	}

	/// Varify a bare attribute name read inside the body of a var-reached class.
	///
	/// A record access varifies when its receiver is var, but inside a class
	/// body an attribute is named without a receiver, and `this` is par — the
	/// lowering iterates the realised object set, so the object handle really is
	/// a par index. The attribute is still projected out of the class's varified
	/// storage record, so the reference is var, exactly as `obj.x` would be.
	/// Without this the body types against the declared par types and resolves
	/// calls to par overloads that no longer apply once lowering hands them the
	/// var projection.
	fn varify_class_body_attribute(
		&mut self,
		pattern: PatternRef<'db>,
		field: Identifier<'db>,
		ty: Ty<'db>,
	) -> Ty<'db> {
		let db = self.db;
		// Attribute patterns live in a class item; anything else is a global.
		// For an inherited attribute that item is the declaring superclass, so
		// the var-reach question is asked of the class being typed, whose
		// storage the read actually goes through.
		let (Item::Class(_), Item::Class(c)) = (pattern.item(db), self.item) else {
			return ty;
		};
		let class = c.class(db);
		// `this` resolves here too, and stays par.
		if pattern == PatternRef::new(db, self.item, class.this_pattern) {
			return ty;
		}
		if !var_reached_classes(db).contains(&PatternRef::new(db, self.item, class.pattern)) {
			return ty;
		}
		let PatternTy::Variable(this_ty) = self
			.ctx
			.type_pattern(db, PatternRef::new(db, self.item, class.this_pattern))
		else {
			return ty;
		};
		let Some(class_ref) = this_ty.class_type(db) else {
			return ty;
		};
		let varified = if matches!(ty.lookup(db), TyData::Array { .. }) {
			self.varify_array_class_attribute(class_ref, field, ty)
		} else {
			ty.make_var(db)
		};
		// An unvarifiable attribute on a var-reached class is reported against
		// its declaration by `object_validation`, with a message that names the
		// class. Keep the declared type here rather than emitting a second,
		// vaguer error at every use.
		varified.unwrap_or(ty)
	}

	fn collect_record_access(
		&mut self,
		expr: ExpressionId<'db>,
		ra: &RecordAccess<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let record = self.collect_expression(ra.record);
		let Some(field) = self.data[ra.field].identifier() else {
			assert!(self.data[ra.field].is_missing());
			// Already would be reported as a syntax error
			return self.types.error;
		};
		let ty = match record.lookup(db) {
			TyData::Record(opt, fields) => {
				let ty = fields
					.iter()
					.find(|(i, _)| *i == field.0)
					.map(|(_, ty)| *ty)
					.unwrap_or_else(|| {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							InvalidFieldAccess {
								src,
								span,
								msg: format!(
									"No such field {} for '{}'",
									field.pretty_print(db),
									record.pretty_print(db)
								),
							},
						);
						self.types.error
					});
				if let OptType::Opt = opt {
					ty.make_opt(db)
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
						.find(|(i, _)| *i == field.0)
						.map(|(_, ty)| *ty)
						.unwrap_or_else(|| {
							let (src, span) =
								ExpressionRef::new(db, self.item, expr).source_span(db);
							self.ctx.add_diagnostic(
								db,
								self.item,
								InvalidFieldAccess {
									src,
									span,
									msg: format!(
										"No such field {} for '{}'",
										field.pretty_print(db),
										element.pretty_print(db)
									),
								},
							);
							self.types.error
						});
					let ty = if let OptType::Opt = o1.max(o2) {
						el.make_opt(db)
					} else {
						el
					};
					Ty::array(db, *dim, ty).unwrap_or_else(|| {
						panic!(
							"Could not create array [{}] of {}",
							dim.pretty_print_as_dims(db),
							ty.pretty_print(db)
						)
					})
				}
				TyData::Error => self.types.error,
				_ => {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
					self.types.error
				}
			},
			TyData::Class(var_type, opt_type, class_ref) => {
				let (var_type, opt_type, class_ref) = (*var_type, *opt_type, *class_ref);
				match self.lookup_class_attribute(class_ref, field) {
					Some((owner, mut ty)) => {
						if let OptType::Opt = opt_type {
							ty = ty.make_opt(db);
						}
						// A var-reached class has its whole storage record varified,
						// so every read of one of its attributes is var — even
						// through a par handle. A par `new C: p` declared alongside
						// a `var new C` elsewhere still projects out of that one var
						// storage array.
						// An output context is the exception: the projection reads a
						// SOLVED value, so it is par whatever the storage inst is,
						// and the lowering wraps it in `fix`. Par-ifying here is
						// what makes the rest of an output comprehension par too — a
						// par `o.children` yields par generator identities, so the
						// object-array accesses beneath it are par-indexed and
						// totalise emits the PAR `if_then_else`, which has a par
						// twin. Leaving it var is what left
						// `shackle_if_then_else<.. var ..>` in the output with no par
						// version to call.
						if self.in_output_item {
							ty = ty.make_par(db);
						}
						let reads_var_storage = !self.in_output_item
							&& (matches!(var_type, VarType::Var)
								|| class_pattern_for(db, class_ref)
									.is_some_and(|p| var_reached_classes(db).contains(&p)));
						if reads_var_storage {
							// An array attribute must go through the guarded
							// element-wise varification: a plain `make_var`
							// varifies the elements blindly, but a ragged
							// array (per-object length) must reject the read —
							// its index set would be decision-dependent.
							let varified = if matches!(ty.lookup(db), TyData::Array { .. }) {
								self.varify_array_class_attribute(owner, field, ty)
							} else {
								ty.make_var(db)
							};
							ty = varified.unwrap_or_else(|| {
								let (src, span) =
									ExpressionRef::new(db, self.item, expr).source_span(db);
								self.ctx.add_diagnostic(
									db,
									self.item,
									IllegalType {
										src,
										span,
										ty: format!("var {}", ty.pretty_print(db)),
									},
								);
								self.types.error
							});
						}
						ty
					}
					None => {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							InvalidFieldAccess {
								src,
								span,
								msg: format!(
									"No such field {} for '{}'",
									field.pretty_print(db),
									class_ref.pretty_print(db)
								),
							},
						);
						self.types.error
					}
				}
			}
			TyData::Error => self.types.error,
			_ => {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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
		};
		self.ctx
			.add_declaration(db, ra.field, PatternTy::RecordField(ty));
		ty
	}

	fn collect_if_then_else(&mut self, expr: ExpressionId<'db>, ite: &IfThenElse<'db>) -> Ty<'db> {
		let db = self.db;
		let condition_types = ite
			.branches
			.iter()
			.map(|b| self.collect_expression(b.condition))
			.collect::<Vec<_>>();
		for (t, b) in condition_types.iter().zip(ite.branches.iter()) {
			if !t.is_subtype_of(db, self.types.var_bool) {
				let (src, span) = ExpressionRef::new(db, self.item, b.condition).source_span(db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected boolean condition, but got '{}'",
							t.pretty_print(db)
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
		let ty = Ty::most_specific_supertype(db, result_types.iter().map(|(_, ty)| *ty))
			.unwrap_or_else(|| {
				let mut expr_tys = result_types.into_iter();
				let (first_expr, first_ty) = expr_tys.next().unwrap();
				let (_, first_span) = ExpressionRef::new(db, self.item, first_expr).source_span(db);
				for (expr, ty) in expr_tys {
					if Ty::most_specific_supertype(db, [first_ty, ty]).is_none() {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							BranchMismatch {
								msg: format!(
									"Mismatch in if-then-else branch types. Expected type compatible with '{}' but got '{}'",
									first_ty.pretty_print(db),
									ty.pretty_print(db)
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
		if ty.contains_function(db) {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
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
		if ite.else_result.is_none() && !ty.has_default_value(db) {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"If-then expression with branch type '{}' must have an else",
						ty.pretty_print(db)
					),
				},
			);
		}
		if let VarType::Var = condition_types
			.iter()
			.flat_map(|t| t.inst(db))
			.max()
			.unwrap_or(VarType::Par)
		{
			// Var condition means var result
			ty.make_var(db).unwrap_or_else(|| {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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

	fn collect_case(&mut self, expr: ExpressionId<'db>, c: &Case<'db>) -> Ty<'db> {
		let db = self.db;
		let scrutinee = self.collect_expression(c.expression);
		for case in c.cases.iter() {
			let _ = self.collect_pattern(Some(expr), true, case.pattern, scrutinee, false);
		}
		let cases = c
			.cases
			.iter()
			.map(|case| (case.value, self.collect_expression(case.value)))
			.collect::<Vec<_>>();
		let ty =
			Ty::most_specific_supertype(db, cases.iter().map(|(_, ty)| *ty)).unwrap_or_else(|| {
				let mut expr_tys = cases.into_iter();
				let (first_expr, first_ty) = expr_tys.next().unwrap();
				let (_, first_span) = ExpressionRef::new(db, self.item, first_expr).source_span(db);
				for (expr, ty) in expr_tys {
					if Ty::most_specific_supertype(db, [first_ty, ty]).is_none() {
						let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							BranchMismatch {
								msg: format!(
									"Mismatch in case arm types. Expected type compatible with '{}' but got '{}'",
									first_ty.pretty_print(db),
									ty.pretty_print(db)
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
		if ty.contains_function(db) {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
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
		if let Some(VarType::Var) = scrutinee.inst(db) {
			ty.make_var(db).unwrap_or_else(|| {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
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

	fn collect_let(&mut self, l: &Let<'db>) -> Ty<'db> {
		let db = self.db;
		let mut make_var = false;
		for item in l.items.iter() {
			match item {
				LetItem::Constraint(c) => {
					for ann in c.annotations.iter() {
						let _ = self.typecheck_expression(*ann, self.types.ann);
					}
					let ty = self.typecheck_expression(c.expression, self.types.var_bool);
					if ty == self.types.var_bool {
						// Var constraints make the return type var
						make_var = true;
					}
				}
				LetItem::Declaration(d) => {
					if self.data[d.declared_type].is_new(self.data) {
						let (src, span) =
							TypeRef::new(db, self.item, d.declared_type).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							SyntaxError {
								src,
								span,
								msg: "'new' declarations are not allowed in let expressions"
									.to_owned(),
							},
						);
					}
					let result = self.collect_declaration(d);
					if !result.ty.contains_error(db)
						&& (result.ty.contains_par(db) || result.ty.contains_function(db))
						&& d.definition.is_none()
					{
						let (src, span) =
							PatternRef::new(self.db, self.item, d.pattern).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							InvalidDeclaration {
								src,
								span,
								msg: if result.ty.contains_var(db) {
									"Local mixed var/par declaration must have a right-hand side"
										.to_owned()
								} else {
									"Local parameter declaration must have a right-hand side"
										.to_owned()
								},
							},
						);
					}
					if result
						.ty
						.walk(db)
						.any(|ty| ty != self.types.var_bool && ty.inst(db) == Some(VarType::Var))
						&& d.definition.is_some()
					{
						// Var declarations make the return type var
						make_var = true;
					}
				}
			}
		}
		let mut ty = self.collect_expression(l.in_expression);
		if make_var && !ty.contains_var(db) {
			ty = ty.make_var(db).unwrap_or_else(|| {
				let (src, span) =
					ExpressionRef::new(db, self.item, l.in_expression).source_span(db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					IllegalType {
						src,
						span,
						ty: format!("var {}", ty.pretty_print(db)),
					},
				);
				self.types.error
			});
		}
		ty
	}

	/// Type check a declaration
	pub fn collect_declaration(&mut self, d: &Declaration<'db>) -> TypeCompletionResult<'db> {
		for p in Pattern::identifiers(d.pattern, self.data) {
			self.ctx.add_declaration(self.db, p, PatternTy::Computing);
		}
		let result = if let Some(e) = d.definition {
			let actual = self.collect_expression(e);
			let lhs =
				self.complete_type(d.declared_type, Some(actual), TypeCompletionMode::Default);
			let expected = lhs.ty;
			if !actual.is_subtype_of(self.db, expected) {
				let (src, span) = (ExpressionRef::new(self.db, self.item, e)).source_span(self.db);
				self.ctx.add_diagnostic(
					self.db,
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
			lhs
		} else {
			self.complete_type(d.declared_type, None, TypeCompletionMode::Default)
		};
		let _ = self.collect_pattern(None, false, d.pattern, result.ty, false);
		for ann in d.annotations.iter() {
			// Handle identifiers/calls which lead to ::annotated_expression functions

			let _ = self.typecheck_expression(*ann, self.types.ann);
		}
		result
	}

	/// Type check a declaration in output mode
	pub fn collect_output_declaration(&mut self, d: &Declaration<'db>) -> Ty<'db> {
		let prev = self.in_output_item;
		self.in_output_item = true;
		let result = self.collect_declaration(d);
		self.in_output_item = prev;
		result.ty
	}

	/// Typecheck an annotation for a declaration (since these may be calls to annotations using `::annotated_expression`)
	pub fn typecheck_declaration_annotation(
		&mut self,
		ann: ExpressionId<'db>,
		declaration_ty: Ty<'db>,
	) {
		let db = self.db;
		let actual = match &self.data[ann] {
			Expression::Identifier(i) => self.collect_identifier(ann, i, Some(declaration_ty)),
			Expression::Call(c) => self.collect_call(ann, c, Some(declaration_ty)),
			_ => self.collect_expression(ann),
		};
		if !actual.is_subtype_of(db, self.types.ann) {
			let (src, span) = (ExpressionRef::new(db, self.item, ann)).source_span(self.db);
			self.ctx.add_diagnostic(
				db,
				self.item,
				TypeMismatch {
					src,
					span,
					msg: format!(
						"Expected '{}' but got '{}'",
						self.types.ann.pretty_print(db),
						actual.pretty_print(db)
					),
				},
			);
		}
		self.ctx.add_expression(db, ann, actual);
		self.collect_annotations(ann, actual);
	}

	fn collect_lambda(&mut self, l: &Lambda<'db>) -> Ty<'db> {
		let db = self.db;
		for p in l
			.parameters
			.iter()
			.filter_map(|param| param.pattern)
			.flat_map(|p| Pattern::identifiers(p, self.data))
		{
			self.ctx.add_declaration(db, p, PatternTy::Computing);
		}
		let params = l
			.parameters
			.iter()
			.map(|param| {
				let ty = self
					.complete_type(param.declared_type, None, TypeCompletionMode::Default)
					.ty;
				if let Some(p) = param.pattern {
					let _ = self.collect_pattern(None, false, p, ty, true);
				}
				ty
			})
			.collect();
		let body = self.collect_expression(l.body);
		let return_type = if let Some(r) = l.return_type {
			let ret = self
				.complete_type(r, Some(body), TypeCompletionMode::Default)
				.ty;
			if !body.is_subtype_of(db, ret) {
				let (src, span) =
					(ExpressionRef::new(self.db, self.item, l.body)).source_span(self.db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					TypeMismatch {
						src,
						span,
						msg: format!(
							"Expected '{}' but got '{}'",
							ret.pretty_print(db),
							body.pretty_print(db),
						),
					},
				);
			}
			ret
		} else {
			body
		};

		Ty::function(
			db,
			FunctionType {
				return_type,
				params,
			},
		)
	}

	/// Resolve overloading for the function `expr` that is the identifier `i`.
	///
	/// `arg_names` is a list of named arguments (the last arguments in `args`)
	///
	/// Returns a tuple of the type of the operation, and the return type
	fn resolve_overloading(
		&mut self,
		expr: ExpressionId<'db>,
		i: Identifier<'db>,
		args: &[Ty<'db>],
		arg_names: &[Identifier<'db>],
		is_annotation_for: Option<Ty<'db>>,
	) -> (Ty<'db>, Ty<'db>) {
		let db = self.db;
		let error = (self.types.error, self.types.error);
		if args.iter().any(|t| t.contains_error(db)) {
			self.ctx.add_expression(db, expr, self.types.error);
			return error;
		}

		// If there's a variable in scope which is a function, use it
		if let Some(p) = self.find_variable(expr, i) {
			let d = self.ctx.type_pattern(db, p);
			let f = match d {
				PatternTy::Variable(t) | PatternTy::Argument(t) => {
					if let TyData::Function(OptType::NonOpt, f) = t.lookup(db) {
						Some(f.clone())
					} else {
						None
					}
				}
				_ => None,
			};
			if let Some(f) = f {
				if f.contains_error(db) {
					return error;
				}
				if !arg_names.is_empty() {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
					self.ctx.add_diagnostic(
						db,
						self.item,
						TypeMismatch {
							src,
							span,
							msg: "Named arguments are not supported for function variables"
								.to_owned(),
						},
					);
					return error;
				}
				if let Err(e) = f.matches(db, args) {
					let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
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
						.add_diagnostic(db, self.item, TypeMismatch { src, span, msg });
					return error;
				}
				let ret = f.return_type;
				let op = Ty::function(db, f);
				self.ctx.add_expression(db, expr, op);
				self.ctx.add_identifier_resolution(db, expr, p);
				return (op, ret);
			}
		}

		// Otherwise resolve overloaded function items
		let patterns = self.find_function(expr, i);
		if patterns.is_empty() {
			let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
			self.ctx.add_diagnostic(
				db,
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
			self.ctx.add_expression(db, expr, self.types.error);
			return error;
		}

		let mut had_duplicate = false;
		let mut seen = Set::new();
		for name in arg_names.iter() {
			if !seen.insert(*name) {
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx.add_diagnostic(
					db,
					self.item,
					InvalidCall {
						src,
						span,
						msg: format!(
							"Name '{}' is used for multiple arguments in call",
							name.pretty_print(db)
						),
					},
				);
				had_duplicate = true;
			}
		}
		if had_duplicate {
			self.ctx.add_expression(db, expr, self.types.error);
			return error;
		}

		let n_positional = args.len() - arg_names.len();
		let mut resolver = NamedArgumentResoler::new(n_positional, arg_names);
		for p in patterns.iter() {
			match self.ctx.type_pattern(db, *p) {
				PatternTy::Function(function)
				| PatternTy::AnnotationConstructor(function)
				| PatternTy::AnnotationDestructure(function) => resolver.add_overload(*p, *function),
				PatternTy::EnumConstructor(ec) => {
					for entry in ec {
						resolver.add_overload(*p, entry.constructor);
					}
				}
				PatternTy::EnumDestructure(fs) => {
					for entry in fs {
						resolver.add_overload(*p, entry);
					}
				}
				PatternTy::Computing => (),
				_ => unreachable!(),
			}
		}
		let resolved = resolver.finish();

		let fn_result = match_fn(db, resolved.canidates, args).or_else(|e| {
			if let Some(ty) = is_annotation_for {
				// Also try matching ::annotated_expression functions
				let mut new_args = Vec::with_capacity(args.len() + 1);
				new_args.push(ty);
				new_args.extend(args.iter().copied());

				let mut new_resolver = NamedArgumentResoler::new(n_positional + 1, arg_names);
				for p in patterns.iter() {
					if let PatternTy::Function(function) = self.ctx.type_pattern(db, *p)
						&& function.has_annotated_expression
					{
						new_resolver.add_overload(*p, *function);
					}
				}

				return match_fn(db, new_resolver.finish().canidates, &new_args).map_err(|_| e);
			}
			Err(e)
		});

		match fn_result {
			Ok(res) => {
				let (pattern, entry) = res.function;
				let instantiation = entry.overload.instantiate(db, &res.ty_params);
				let ret = instantiation.return_type;
				let op = Ty::function(db, instantiation);
				self.ctx.add_expression(db, expr, op);
				self.ctx.add_identifier_resolution(db, expr, pattern);
				(op, ret)
			}
			Err(FunctionResolutionError::AmbiguousOverloading(ps)) => {
				let mut msg = format!(
					"Call {}({}) is ambiguous.\n",
					i.pretty_print(db),
					args.iter()
						.enumerate()
						.map(|(idx, t)| if idx < n_positional {
							t.pretty_print(db)
						} else {
							format!(
								"{}: {}",
								t.pretty_print(db),
								arg_names[idx - n_positional].pretty_print(db)
							)
						})
						.collect::<Vec<_>>()
						.join(", ")
				);

				writeln!(
					&mut msg,
					"Could not choose an overload from the candidate functions:"
				)
				.unwrap();
				for (pattern, entry) in ps.iter() {
					writeln!(
						&mut msg,
						"  {}",
						self.ctx
							.type_pattern(db, *pattern)
							.pretty_print(db, pattern.identifier(db))
							.unwrap_or_else(|| entry.overload.pretty_print_item(db, i))
					)
					.unwrap();
				}
				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx
					.add_diagnostic(db, self.item, AmbiguousCall { src, span, msg });
				self.ctx.add_expression(db, expr, self.types.error);
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
						"No function {}({}) could be found.",
						i.pretty_print(db),
						args.iter()
							.enumerate()
							.map(|(idx, t)| if idx < n_positional {
								t.pretty_print(db)
							} else {
								format!(
									"{}: {}",
									t.pretty_print(db),
									arg_names[idx - n_positional].pretty_print(db)
								)
							})
							.collect::<Vec<_>>()
							.join(", ")
					)
					.unwrap();
				}
				if !es.is_empty() || !resolved.eliminated.is_empty() {
					writeln!(&mut msg, "The following overloads could not be used:").unwrap();
					for ((pattern, entry), e) in es.iter() {
						writeln!(
							&mut msg,
							"  {}",
							self.ctx
								.type_pattern(db, *pattern)
								.pretty_print(db, pattern.identifier(db))
								.unwrap_or_else(|| entry.overload.pretty_print_item(db, i))
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
									if *index < n_positional {
										(index + 1).to_string()
									} else {
										format!(
											"'{}'",
											arg_names[*index - n_positional].pretty_print(db)
										)
									},
									expected.pretty_print(db),
									actual.pretty_print(db)
								)
								.unwrap();
							}
							InstantiationError::IncompatibleTypeInstVariable { ty_var, types } => {
								if types.is_empty() {
									// Should not be possible currently
									writeln!(
										&mut msg,
										"    Type-inst parameter '{}' not instantiated",
										ty_var.pretty_print(db)
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
										ty_var.pretty_print(db),
										tys
									)
									.unwrap();
								}
							}
						}
					}
					for eliminated in resolved.eliminated.iter() {
						writeln!(
							&mut msg,
							"  {}",
							eliminated.overload.1.overload.pretty_print_item(db, i)
						)
						.unwrap();
						match eliminated.reason {
							OverloadEliminationReason::PositionalNameConflict {
								position,
								name,
							} => {
								writeln!(
									&mut msg,
									"    Argument {} conflicts with named argument '{}'",
									position + 1,
									name.pretty_print(db)
								)
								.unwrap();
							}
							OverloadEliminationReason::MissingParameter { name } => {
								writeln!(
									&mut msg,
									"    Missing argument for parameter '{}'",
									name.pretty_print(db)
								)
								.unwrap();
							}
							OverloadEliminationReason::ArgumentCountMismatch {
								expected_min,
								expected_max,
								actual,
							} => {
								writeln!(
									&mut msg,
									"    {} arguments required, {} given",
									if actual <= expected_min {
										expected_min
									} else {
										expected_max
									},
									actual
								)
								.unwrap();
							}
						}
					}
				}

				let (src, span) = ExpressionRef::new(db, self.item, expr).source_span(db);
				self.ctx
					.add_diagnostic(db, self.item, NoMatchingFunction { src, span, msg });
				self.ctx.add_expression(db, expr, self.types.error);
				error
			}
		}
	}

	/// Collect the type of a pattern
	pub fn collect_pattern(
		&mut self,
		scope: Option<ExpressionId<'db>>,
		resolves_atoms: bool,
		pat: PatternId<'db>,
		expected: Ty<'db>,
		is_argument: bool,
	) -> Ty<'db> {
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
								self.ctx.add_pattern_resolution(db, pat, p);
								Some(ty)
							}
							PatternTy::AnnotationAtom => {
								self.ctx.add_pattern_resolution(db, pat, p);
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
						db,
						pat,
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
							let (src, span) = PatternRef::new(db, self.item, pat).source_span(db);
							self.ctx.add_diagnostic(
								db,
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
					let non_opt = expected.make_occurs(db);
					let c = cs
						.iter()
						.find(|c| non_opt.is_subtype_of(db, c.overload.return_type()))
						.or_else(|| {
							let (src, span) = PatternRef::new(db, self.item, pat).source_span(db);
							self.ctx.add_diagnostic(
								db,
								self.item,
								NoMatchingFunction {
									src,
									span,
									msg: format!(
										"No constructor '{}' found for type '{}'",
										name.pretty_print(db),
										expected.pretty_print(db)
									),
								},
							);
							None
						})?
						.clone();

					if c.overload.params().len() != arguments.len() {
						let (src, span) = PatternRef::new(db, self.item, pat).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
						let _ = self.collect_pattern(scope, resolves_atoms, *p, t, is_argument);
					}
					self.ctx.add_pattern_resolution(db, *function, ctor_pat);
					let ctor_type = c.overload.clone().into_function().unwrap();
					let dtor_type = FunctionType {
						params: Box::new([ctor_type.return_type]),
						return_type: if ctor_type.params.len() == 1 {
							ctor_type.params[0]
						} else {
							Ty::tuple(db, ctor_type.params.iter().copied())
						},
					};
					self.ctx.add_declaration(
						db,
						*function,
						PatternTy::DestructuringFn {
							constructor: Ty::function(db, ctor_type),
							destructor: Ty::function(db, dtor_type),
						},
					);
					Some(c.overload.return_type())
				})();

				if let Some(ty) = res {
					ty
				} else {
					// Continue collection
					for p in arguments.iter() {
						let _ = self.collect_pattern(
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
						.map(|(p, e)| {
							self.collect_pattern(scope, resolves_atoms, *p, e, is_argument)
						}),
				),
				_ => Ty::tuple(
					db,
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
			Pattern::Record { fields } => match expected.lookup(db) {
				TyData::Record(_, fs) => {
					let mut map = Map::default();
					for (i, f) in fs.iter() {
						let _ = map.insert(*i, *f);
					}
					Ty::record(
						db,
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
					db,
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
		self.ctx
			.add_declaration(db, pat, PatternTy::Destructuring(actual));
		if !actual.is_subtype_of(db, expected) {
			let (src, span) = PatternRef::new(db, self.item, pat).source_span(db);
			self.ctx.add_diagnostic(
				db,
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
	pub fn complete_type(
		&mut self,
		t: TypeId<'db>,
		ty: Option<Ty<'db>>,
		mode: TypeCompletionMode,
	) -> TypeCompletionResult<'db> {
		let mut has_bounded = false;
		let mut has_var_bounded = false;
		let mut has_unbounded = false;
		let ty = self.complete_type_inner(
			t,
			ty,
			mode,
			&mut has_bounded,
			&mut has_var_bounded,
			&mut has_unbounded,
		);
		TypeCompletionResult {
			ty,
			has_bounded,
			has_var_bounded,
			has_unbounded,
		}
	}

	fn complete_type_inner(
		&mut self,
		t: TypeId<'db>,
		ty: Option<Ty<'db>>,
		mode: TypeCompletionMode,
		has_bounded: &mut bool,
		has_var_bounded: &mut bool,
		has_unbounded: &mut bool,
	) -> Ty<'db> {
		let db = self.db;
		let mut is_bounded = false;
		let mut set_bounded = |typer: &mut Self, domain: ExpressionId<'db>| {
			*has_bounded = true;
			is_bounded = true;
			match mode {
				TypeCompletionMode::AnnotationParameter => {
					let (src, span) = (ExpressionRef::new(db, typer.item, domain)).source_span(db);
					typer.ctx.add_diagnostic(
						db,
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
					let (src, span) = ExpressionRef::new(db, typer.item, domain).source_span(db);
					typer.ctx.add_diagnostic(
						db,
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
				let (src, span) = TypeRef::new(db, typer.item, t).source_span(db);
				typer.ctx.add_diagnostic(
					db,
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

		let result = match &self.data[t] {
			Type::Primitive {
				inst,
				opt,
				primitive_type,
			} => {
				set_unbounded(self);
				let ty = match primitive_type {
					PrimitiveType::Ann => Ty::ann(db),
					PrimitiveType::Bool => Ty::par_bool(db),
					PrimitiveType::Float => Ty::par_float(db),
					PrimitiveType::Int => Ty::par_int(db),
					PrimitiveType::String => Ty::string(db),
				};
				ty.with_inst(db, *inst)
					.unwrap_or_else(|| {
						let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
							self.ctx.add_identifier_resolution(db, *domain, p);
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
								PatternTy::Variable(ty) | PatternTy::Argument(ty) => {
									match ty.lookup(db) {
										TyData::Set(VarType::Par, OptType::NonOpt, inner) => {
											set_bounded(self, *domain);
											self.ctx.add_expression(db, *domain, ty);
											*inner
										}
										TyData::Error => self.types.error,
										_ => {
											let (src, span) =
												ExpressionRef::new(db, self.item, *domain)
													.source_span(db);
											self.ctx.add_diagnostic(
												db,
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
								PatternTy::Enum(ty)
								| PatternTy::ClassDecl {
									defining_set_ty: ty,
									..
								} => match ty.lookup(db) {
									// A var-reached class has a `var set of C` defining
									// set, but in a domain or type position the class
									// denotes its par potential universe, not the var
									// actual set. Accept either inst and record the par
									// set; the var actual set is only used in value
									// positions such as `x in C`.
									TyData::Set(_, OptType::NonOpt, inner) => {
										// Don't set has_bounded or has_unbounded as enums are accepted
										// everywhere
										self.ctx.add_expression(db, *domain, ty.make_par(db));
										*inner
									}
									TyData::Error => self.types.error,
									_ => unreachable!(),
								},
								PatternTy::TyVar(t) => {
									*has_unbounded = true;
									Ty::type_inst_var(db, t)
								}
								PatternTy::Computing => {
									// Error will be emitted during topological sorting
									return self.types.error;
								}
								_ => {
									let (src, span) =
										TypeRef::new(db, self.item, t).source_span(db);
									self.ctx.add_diagnostic(
										db,
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
							let (src, span) =
								ExpressionRef::new(db, self.item, *domain).source_span(db);
							self.ctx.add_diagnostic(
								db,
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
						match ty.lookup(db) {
							TyData::Set(VarType::Par, OptType::NonOpt, e) => {
								set_bounded(self, *domain);
								*e
							}
							TyData::Error => self.types.error,
							_ => {
								let (src, span) =
									ExpressionRef::new(db, self.item, *domain).source_span(db);
								self.ctx.add_diagnostic(
									db,
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
							ExpressionRef::new(db, self.item, *domain).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
				dimension_pattern,
				dimensions,
				element,
			} => {
				let (d_ty, e_ty) = match ty.map(|ty| ty.lookup(db)) {
					Some(TyData::Array { dim, element, .. }) => (Some(*dim), Some(*element)),
					_ => (None, None),
				};
				let dim = self.complete_type_inner(
					*dimensions,
					d_ty,
					mode,
					has_bounded,
					has_var_bounded,
					has_unbounded,
				);
				if let Some(pattern) = dimension_pattern {
					let _ = self.collect_pattern(None, false, *pattern, dim, false);
				}
				let element = self.complete_type_inner(
					*element,
					e_ty,
					mode,
					has_bounded,
					has_var_bounded,
					has_unbounded,
				);
				let ty = Ty::array(db, dim, element).unwrap_or_else(|| {
					let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
					self.ctx.add_diagnostic(
						db,
						self.item,
						IllegalType {
							src,
							span,
							ty: format!(
								"array [{}] of {}",
								dim.pretty_print_as_dims(db),
								element.pretty_print(db)
							),
						},
					);
					self.types.error
				});
				ty.with_opt(db, *opt)
			}
			Type::Set {
				inst,
				opt,
				cardinality,
				element,
			} => {
				let e_ty = match ty.map(|ty| ty.lookup(db)) {
					Some(TyData::Set(_, _, element)) => Some(*element),
					_ => None,
				};
				let el = self.complete_type_inner(
					*element,
					e_ty,
					mode,
					&mut is_bounded,
					has_var_bounded,
					has_unbounded,
				);
				*has_bounded |= is_bounded;
				if let Some(card) = cardinality {
					let _ = self.typecheck_expression(*card, self.types.set_of_int);
				}
				let ty = Ty::par_set(db, el).unwrap_or_else(|| {
					let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
						let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
						.map(|(f, f_ty)| {
							self.complete_type_inner(
								*f,
								f_ty,
								mode,
								has_bounded,
								has_var_bounded,
								has_unbounded,
							)
						}),
				)
				.with_opt(db, *opt),
				_ => Ty::tuple(
					db,
					fields.iter().map(|f| {
						self.complete_type_inner(
							*f,
							None,
							mode,
							has_bounded,
							has_var_bounded,
							has_unbounded,
						)
					}),
				)
				.with_opt(db, *opt),
			},
			Type::Record { opt, fields: fs } => {
				let mut fields = Map::default();
				for (p, t) in fs.iter() {
					let i = self.data[*p]
						.identifier()
						.expect("Record field not an identifier");
					match fields.entry(i) {
						Entry::Vacant(e) => {
							let _ = e.insert((*p, *t));
						}
						Entry::Occupied(_) => {
							let (src, span) = PatternRef::new(db, self.item, *p).source_span(db);
							self.ctx.add_diagnostic(
								db,
								self.item,
								SyntaxError {
									src,
									span,
									msg: format!(
										"Record type contains duplicate field '{}'",
										i.pretty_print(db)
									),
								},
							);
						}
					}
				}
				Ty::record(
					db,
					fields.into_iter().map(|(i, (p, f))| {
						let field_ty = self.complete_type_inner(
							f,
							ty.and_then(|ty| match ty.lookup(db) {
								TyData::Record(_, fs) => {
									fs.iter().find(|(i2, _)| i.0 == *i2).map(|(_, t)| *t)
								}
								_ => None,
							}),
							mode,
							has_bounded,
							has_var_bounded,
							has_unbounded,
						);

						self.ctx
							.add_declaration(db, p, PatternTy::RecordField(field_ty));

						(i, field_ty)
					}),
				)
				.with_opt(db, *opt)
			}
			Type::Operation {
				opt,
				return_type,
				parameter_types,
			} => {
				if let TypeCompletionMode::AnnotationParameter = mode {
					let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
				match ty.map(|ty| ty.lookup(db)) {
					Some(TyData::Function(
						_,
						FunctionType {
							return_type: r,
							params: ps,
						},
					)) => Ty::function(
						db,
						FunctionType {
							return_type: self.complete_type_inner(
								*return_type,
								Some(*r),
								TypeCompletionMode::Operation,
								has_bounded,
								has_var_bounded,
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
										has_var_bounded,
										has_unbounded,
									)
								})
								.collect(),
						},
					)
					.with_opt(db, *opt),
					_ => Ty::function(
						db,
						FunctionType {
							return_type: self.complete_type_inner(
								*return_type,
								None,
								TypeCompletionMode::Operation,
								has_bounded,
								has_var_bounded,
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
										has_var_bounded,
										has_unbounded,
									)
								})
								.collect(),
						},
					)
					.with_opt(db, *opt),
				}
			}
			Type::AnonymousTypeInstVar { inst, opt, pattern } => {
				*has_unbounded = true;
				let mut ty = Ty::type_inst_var(
					db,
					match self
						.ctx
						.type_pattern(db, PatternRef::new(db, self.item, *pattern))
					{
						PatternTy::TyVar(tv) => tv,
						_ => unimplemented!(),
					},
				);
				if let Some(inst) = inst {
					ty = ty.with_inst(db, *inst).unwrap_or_else(|| {
						let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
						self.ctx.add_diagnostic(
							db,
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
			Type::Any => {
				*has_unbounded = true;
				ty.filter(|&ty| !ty.contains_bottom(db)).unwrap_or_else(|| {
					let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
					self.ctx.add_diagnostic(
						db,
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
			Type::New { inst, opt, domain } => {
				let mut ty = match &self.data[*domain] {
					Expression::Identifier(i) => {
						let Some(p) = self.find_variable(*domain, *i) else {
							let (src, span) =
								ExpressionRef::new(db, self.item, *domain).source_span(db);
							self.ctx.add_diagnostic(
								db,
								self.item,
								UndefinedIdentifier {
									identifier: i.pretty_print(db),
									src,
									span,
								},
							);
							return self.types.error;
						};
						self.ctx.add_identifier_resolution(db, *domain, p);
						match self.ctx.type_pattern(db, p) {
							PatternTy::ClassDecl {
								defining_set_ty, ..
							} => {
								self.ctx.add_expression(db, *domain, defining_set_ty);
								defining_set_ty
									.elem_ty(db)
									.expect("Class defining set must be a set type")
							}
							PatternTy::Computing => {
								// Error will be emitted during topological sorting
								self.ctx.add_expression(db, *domain, self.types.error);
								return self.types.error;
							}
							_ => {
								self.ctx.add_expression(db, *domain, self.types.error);
								let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
								self.ctx.add_diagnostic(
									db,
									self.item,
									TypeMismatch {
										src,
										span,
										msg: "Expected a class name.".to_owned(),
									},
								);
								return self.types.error;
							}
						}
					}
					_ => unreachable!("'new' domain must be an identifier"),
				};
				// `var new C` introduces fresh var attribute storage: the class type
				// carries the var signal, so attribute access varifies through the
				// field-access rules.
				if let VarType::Var = inst {
					ty = ty.make_var(db).unwrap_or_else(|| {
						let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
						self.ctx.add_diagnostic(
							db,
							self.item,
							IllegalType {
								src,
								span,
								ty: format!("var {}", ty.pretty_print(db)),
							},
						);
						self.types.error
					});
				}
				ty.with_opt(db, *opt)
			}
			Type::Missing => self.types.error,
		};

		if matches!(result.inst(db), Some(VarType::Var)) && is_bounded {
			*has_var_bounded = true;
		}

		self.ctx.add_type(db, t, result);
		result
	}

	/// Get the type of a class attribute as it appears in the class's input record,
	/// i.e. the type the caller supplies when constructing an object.
	///
	/// Returns `None` when the attribute is not part of the input record: var
	/// attributes are decided rather than supplied, and unsupported attribute
	/// shapes report an error and are dropped.
	pub fn class_type_to_input_record_type(&mut self, t: TypeId<'db>) -> Option<Ty<'db>> {
		let db = self.db;
		let ty = self.ctx.get_type(db, t);
		if ty.inst(db) == Some(VarType::Var) {
			return None;
		}
		match &self.data[t] {
			Type::Primitive { .. } | Type::Bounded { .. } => Some(ty),

			Type::Array {
				dimensions,
				element,
				..
			} => {
				let dim_ty = self.ctx.get_type(db, *dimensions);
				let element_ty = self.class_type_to_input_record_type(*element)?;
				Ty::array(db, dim_ty, element_ty)
			}

			Type::New { opt, .. } => {
				let class = ty.class_type(db)?;
				let pattern = class_pattern_for(db, class)?;
				// A `new` attribute whose introduced class lies on an ownership
				// cycle would make this record infinite: inlining the child's
				// input record grows at every signature fixpoint iteration
				// instead of converging. Drop the slot without a diagnostic;
				// object validation rejects the model with the real error.
				if crate::class_analysis::ownership_cyclic_classes(db).contains(&pattern) {
					return None;
				}
				match self.ctx.type_pattern(db, pattern) {
					PatternTy::ClassDecl {
						input_record_ty, ..
					} => Some(input_record_ty.with_opt(db, *opt)),
					_ => unreachable!("Class type must resolve to a class declaration"),
				}
			}

			Type::Set { element, .. } => {
				let element_ty = self.class_type_to_input_record_type(*element)?;
				if ty.class_type(db).is_some() {
					// The members of a `set of new C` are object identities, which
					// cannot be given in data. Keep the array-of-input-records form,
					// one slot per member; the actual set is derived from those inputs.
					Ty::array(db, self.types.par_int, element_ty)
				} else {
					// A plain par set (`set of 0..3`) is an ordinary value, so it maps
					// to a set-typed input slot and is stored directly. That keeps
					// `card`, membership and iteration working on read-back.
					Ty::par_set(db, element_ty)
				}
			}

			Type::Tuple { .. } | Type::Record { .. } => {
				// A structured attribute with no class reference anywhere in its
				// subtree is an ordinary value: the caller supplies the tuple or
				// record directly, so the input slot is the declared type itself.
				// Structured attributes which do contain object machinery are
				// rejected: occurrence expansion does not look inside tuple and
				// record types, so a `tuple(new A, int)` would silently skip the
				// introduction bookkeeping. A structured attribute with a var leaf is
				// excluded from the input record without a diagnostic, like an
				// explicitly var scalar attribute.
				if ty.walk(db).any(|leaf| leaf.class_type(db).is_some()) {
					let is_tuple = matches!(&self.data[t], Type::Tuple { .. });
					self.add_unsupported_class_attribute_diagnostic(
						t,
						if is_tuple {
							"tuple types containing class references or `new` \
							 introductions are not supported as class attributes yet"
						} else {
							"record types containing class references or `new` \
							 introductions are not supported as class attributes yet"
						},
					);
					None
				} else if !ty.known_par(db) {
					None
				} else {
					Some(ty)
				}
			}
			Type::Operation { .. } => {
				self.add_unsupported_class_attribute_diagnostic(
					t,
					"function-typed class attributes are not supported",
				);
				None
			}
			Type::AnonymousTypeInstVar { .. } => {
				self.add_unsupported_class_attribute_diagnostic(
					t,
					"anonymous type-inst variables are not supported as class attributes",
				);
				None
			}
			Type::Any => {
				self.add_unsupported_class_attribute_diagnostic(
					t,
					"`any` typed class attributes are not supported",
				);
				None
			}
			Type::Missing => None,
		}
	}

	/// Report a class attribute type which cannot be lowered into an input record.
	fn add_unsupported_class_attribute_diagnostic(&mut self, t: TypeId<'db>, msg: &str) {
		let db = self.db;
		let (src, span) = TypeRef::new(db, self.item, t).source_span(db);
		self.ctx.add_diagnostic(
			db,
			self.item,
			UnsupportedObjectFeature {
				src,
				span,
				msg: msg.to_owned(),
			},
		);
	}

	fn find_variable(
		&self,
		expression: ExpressionId<'db>,
		identifier: Identifier<'db>,
	) -> Option<PatternRef<'db>> {
		let scope = self.item.scope(self.db);
		scope.find_variable(self.db, expression, identifier)
	}

	fn find_function(
		&self,
		expression: ExpressionId<'db>,
		identifier: Identifier<'db>,
	) -> Vec<PatternRef<'db>> {
		let scope = self.item.scope(self.db);
		scope.find_function(self.db, expression, identifier)
	}
}
