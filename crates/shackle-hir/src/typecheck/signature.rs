//! Types of signatures - the type required when referring to an item
//!
//! E.g.
//! - Function parameter/return type
//! - Variable declaration LHS types
use shackle_diagnostics::{Error, SyntaxError, TypeInferenceFailure, TypeMismatch, Warning};
use shackle_ty::{
	ClassRef, EnumRef, FunctionEntry, FunctionType, OverloadedFunction, PolymorphicFunctionType,
	Ty, TyData, TyVar, TyVarRef, registry::TypeRegistry,
};
use shackle_utils::hash::Map;

use crate::{
	ClassMember, Constructor, ConstructorParameter, Db, EnumConstructor, EnumConstructorEntry,
	Expression, ExpressionId, Goal, Identifier, Item, ItemData, Pattern, PatternId, PatternTy,
	Type, TypeCompletionMode, TypeContext, TypeId, Typer,
	class_analysis::{class_pattern_for, var_actual_set_classes},
	constants::IdentifierRegistry,
	diagnostics::Diagnostics,
	ids::{ExpressionRef, NodeRef, PatternRef, TypeRef},
};

/// Collected types for an item signature
///
/// Obtained via `HasSignature::signature()`
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update, Default)]
pub struct SignatureTypes<'db> {
	/// Types of declarations
	pub patterns: Map<PatternId<'db>, PatternTy<'db>>,
	/// Types of expressions
	pub expressions: Map<ExpressionId<'db>, Ty<'db>>,
	/// Identifier resolution
	pub identifier_resolution: Map<ExpressionId<'db>, PatternRef<'db>>,
	/// Pattern resolution
	pub pattern_resolution: Map<PatternId<'db>, PatternRef<'db>>,
	/// Types computed for declared types
	pub types: Map<TypeId<'db>, Ty<'db>>,
}

#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
struct SignatureTypesResult<'db> {
	/// The signature of this item
	signature: SignatureTypes<'db>,

	/// Errors produced during signature typechecking.
	///
	/// We can't directly accumulate these since the `item_body_types` query can be cyclic.
	/// Instead we have to collect them and accumulate them at the end.
	errors: Diagnostics,
}

impl<'db> Item<'db> {
	/// Get the signature types for this item
	pub fn signature(&self, db: &'db dyn Db) -> &'db SignatureTypes<'db> {
		&item_signature(db, *self).as_ref().unwrap().signature
	}

	/// Get signature types for this item (callable from a cylic query)
	pub(super) fn possibly_cyclic_signature(
		&self,
		db: &'db dyn Db,
	) -> Option<&'db SignatureTypes<'db>> {
		item_signature(db, *self)
			.as_ref()
			.map(|result| &result.signature)
	}
}

/// Get signature types without accumulating errors into the database
#[salsa::tracked(returns(ref), cycle_initial=unknown_item_signature)]
pub(super) fn item_signature<'db>(
	db: &'db dyn Db,
	item: Item<'db>,
) -> Option<SignatureTypesResult<'db>> {
	let mut ctx = SignatureTypeContext::new(item);
	ctx.type_item(db, item);
	Some(ctx.finish())
}

/// When the `item_signature` query is a cycle, use an empty item signature.
///
/// Function signatures can require cyclic queries, for example if a function refers
/// to another overload of itself in its parameter/return types. The overloading
/// resolution will try to look up the signature of all overloads, including itself,
/// but since it can't call itself, we can just use an empty signature.
fn unknown_item_signature<'db>(
	_db: &'db dyn Db,
	_id: salsa::Id,
	_item: Item<'db>,
) -> Option<SignatureTypesResult<'db>> {
	None
}

/// Accumulate signature typechecking diagnostics for this item
#[salsa::tracked]
pub(super) fn accumulate_item_signature_diagnostics<'db>(db: &'db dyn Db, item: Item<'db>) {
	item_signature(db, item)
		.as_ref()
		.unwrap()
		.errors
		.accumulate(db);
}

/// Context for typing an item signature
struct SignatureTypeContext<'db> {
	item: Item<'db>,
	data: SignatureTypes<'db>,
	diagnostics: Diagnostics,
}

impl<'db> SignatureTypeContext<'db> {
	/// Create a new signature type context
	pub(crate) fn new(item: Item<'db>) -> Self {
		Self {
			item,
			data: SignatureTypes::default(),
			diagnostics: Diagnostics::default(),
		}
	}

	/// Compute the signature of the given item
	fn type_item(&mut self, db: &'db dyn Db, item: Item<'db>) {
		let data = item.data(db);
		match item {
			Item::Annotation(a) => {
				let it = a.annotation(db);
				match &it.constructor {
					Constructor::Atom { pattern } => {
						self.add_declaration(db, *pattern, PatternTy::AnnotationAtom);
					}
					Constructor::Function {
						constructor,
						destructor,
						parameters,
					} => {
						let params = parameters
							.iter()
							.map(|p| {
								let mut had_error = false;
								for t in Type::any_types(p.declared_type, data) {
									let (src, span) = TypeRef::new(db, item, t).source_span(db);
									self.add_diagnostic(
										db,
										item,
										TypeInferenceFailure {
											src,
											span,
											msg: "Incomplete parameter types are not allowed"
												.to_owned(),
										},
									);
									had_error = true;
								}
								let ty = if had_error {
									TypeRegistry::lookup(db).error
								} else {
									Typer::new(db, self, item, data)
										.complete_type(
											p.declared_type,
											None,
											TypeCompletionMode::AnnotationParameter,
										)
										.ty
								};
								if let Some(pat) = p.pattern {
									self.add_declaration(db, pat, PatternTy::Argument(ty));
								}
								ty
							})
							.collect::<Box<_>>();
						let ann = TypeRegistry::lookup(db).ann;
						let dtor = FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: if params.len() == 1 {
									params[0]
								} else {
									Ty::tuple(db, params.iter().copied())
								},
								params: Box::new([ann]),
							}),
						};
						self.add_declaration(
							db,
							*destructor,
							PatternTy::AnnotationDestructure(Box::new(dtor)),
						);
						let ctor = FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: ann,
								params,
							}),
						};
						self.add_declaration(
							db,
							*constructor,
							PatternTy::AnnotationConstructor(Box::new(ctor)),
						);
					}
				}
			}
			Item::Function(f) => {
				let it = f.function(db);
				// Set as computing so if there's a call to a function with this name we can break the cycle
				// (since if the call is actually not referring to this overload, it should work)
				self.add_declaration(db, it.pattern, PatternTy::Computing);
				let ids = IdentifierRegistry::lookup(db);
				let ty_params = it
					.type_inst_vars
					.iter()
					.map(|tv| {
						let ty_var = TyVarRef::new(
							PatternRef::new(db, item, tv.name).identifier(db).unwrap(),
						);
						let type_var = TyVar {
							ty_var,
							varifiable: tv.is_varifiable,
							enumerable: tv.is_enum,
							indexable: tv.is_indexable,
						};
						self.add_declaration(db, tv.name, PatternTy::TyVar(type_var));
						ty_var
					})
					.collect::<Box<[_]>>();
				let mut var_partial = false;
				let params = it
					.parameters
					.iter()
					.enumerate()
					.map(|(i, p)| {
						let mut had_error = false;
						let annotated_expression = p
							.annotations
							.iter()
							.find(|ann| match &data[**ann] {
								Expression::Identifier(i) => {
									*i == ids.annotations.annotated_expression
								}
								_ => false,
							})
							.copied();
						if i > 0
							&& let Some(ann) = annotated_expression
						{
							let (src, span) = ExpressionRef::new(db, item, ann).source_span(db);
							self.add_diagnostic(db,
									item,
									SyntaxError {
										src,
										span,
										msg: "'annotated_expression' only allowed on first function parameter.".to_owned(),
									},
								);
						}
						for t in Type::any_types(p.declared_type, data) {
							let (src, span) = TypeRef::new(db, item, t).source_span(db);
							self.add_diagnostic(
								db,
								item,
								TypeInferenceFailure {
									src,
									span,
									msg: "Incomplete parameter types are not allowed".to_owned(),
								},
							);
							had_error = true;
						}
						let mut typer = Typer::new(db, self, item, data);
						let ty = if had_error {
							TypeRegistry::lookup(db).error
						} else {
							let result = typer.complete_type(
								p.declared_type,
								None,
								TypeCompletionMode::Default,
							);
							var_partial |= result.has_var_bounded;
							result.ty
						};
						if let Some(pat) = p.pattern {
							let _ = typer.collect_pattern(None, false, pat, ty, true);
						}
						ty
					})
					.collect();
				if ty_params.is_empty() {
					let f = FunctionType {
						return_type: TypeRegistry::lookup(db).error,
						params,
					};
					self.add_declaration(
						db,
						it.pattern,
						PatternTy::Function(Box::new(FunctionEntry {
							has_body: it.body.is_some(),
							overload: OverloadedFunction::Function(f),
						})),
					);
				} else {
					let p = PolymorphicFunctionType {
						return_type: TypeRegistry::lookup(db).error,
						ty_params,
						params,
					};
					self.add_declaration(
						db,
						it.pattern,
						PatternTy::Function(Box::new(FunctionEntry {
							has_body: it.body.is_some(),
							overload: OverloadedFunction::PolymorphicFunction(p),
						})),
					);
				}

				let mut had_error = false;
				for t in Type::any_types(it.return_type, data)
					.chain(Type::anonymous_ty_vars(it.return_type, data))
				{
					let (src, span) = TypeRef::new(db, item, t).source_span(db);
					self.add_diagnostic(
						db,
						item,
						TypeInferenceFailure {
							src,
							span,
							msg: "Incomplete return type not allowed".to_owned(),
						},
					);
					had_error = true;
				}
				let return_type = if had_error {
					TypeRegistry::lookup(db).error
				} else {
					let result = Typer::new(db, self, item, data).complete_type(
						it.return_type,
						None,
						TypeCompletionMode::Default,
					);
					var_partial |= result.has_var_bounded;
					if var_partial && result.ty.known_par(db) {
						let (src, span) = TypeRef::new(db, item, it.return_type).source_span(db);
						self.add_diagnostic(
							db,
							item,
							TypeMismatch {
								src,
								span,
								msg: "Var bounded arguments require the return type to be var"
									.to_owned(),
							},
						);
					}
					result.ty
				};

				let d = self.data.patterns.get_mut(&it.pattern).unwrap();
				match d {
					PatternTy::Function(function) => match function.as_mut() {
						FunctionEntry {
							overload: OverloadedFunction::Function(f),
							..
						} => {
							f.return_type = return_type;
						}
						FunctionEntry {
							overload: OverloadedFunction::PolymorphicFunction(p),
							..
						} => {
							p.return_type = return_type;
						}
					},
					_ => unreachable!(),
				}
			}
			Item::Declaration(d) => {
				let it = d.declaration(db);
				let ids = IdentifierRegistry::lookup(db);
				let output_only = it
					.annotations
					.iter()
					.find(|ann| match &data[**ann] {
						Expression::Identifier(i) => *i == ids.annotations.output_only,
						_ => false,
					})
					.copied();
				for p in Pattern::identifiers(it.pattern, data) {
					self.add_declaration(db, p, PatternTy::Computing);
				}
				let mut typer = Typer::new(db, self, item, data);
				let ty = if data[it.declared_type].is_complete(data) {
					// Use LHS type only
					let expected = typer
						.complete_type(it.declared_type, None, TypeCompletionMode::Default)
						.ty;
					typer.collect_pattern(None, false, it.pattern, expected, false)
				} else if output_only.is_some() {
					typer.collect_output_declaration(it)
				} else {
					typer.collect_declaration(it).ty
				};

				if it.definition.is_none()
					&& (ty.contains_var(db) && ty.contains_par(db) || ty.contains_function(db))
				{
					let (src, span) = NodeRef::from(item).source_span(db);
					self.add_diagnostic(
						db,
						item,
						SyntaxError {
							src,
							span,
							msg: "declaration must have a right-hand side.".to_owned(),
						},
					);
				}

				if let Some(ann) = output_only {
					if it.definition.is_none() {
						let (src, span) = ExpressionRef::new(db, item, ann).source_span(db);
						self.add_diagnostic(
							db,
							item,
							SyntaxError {
								src,
								span,
								msg: "'output_only' declarations must have a right-hand side."
									.to_owned(),
							},
						);
					}
					if !ty.known_par(db) {
						let (src, span) = ExpressionRef::new(db, item, ann).source_span(db);
						self.add_diagnostic(
							db,
							item,
							TypeMismatch {
								src,
								span,
								msg: "'output_only' declarations must be par.".to_owned(),
							},
						);
					}
				}
			}
			Item::Enumeration(e) => {
				let it = e.enumeration(db);
				let ty = Ty::par_enum(
					db,
					EnumRef::new(
						PatternRef::new(db, item, it.pattern)
							.identifier(db)
							.unwrap(),
					),
				);
				self.add_declaration(
					db,
					it.pattern,
					PatternTy::Enum(Ty::par_set(db, ty).unwrap()),
				);
				if let Some(cases) = &it.definition {
					self.add_enum_cases(db, item, data, ty, cases);
				}
			}
			Item::EnumAssignment(e) => {
				let it = e.enum_assignment(db);
				let set_ty = Typer::new(db, self, item, data).collect_expression(it.assignee);
				let ty = match set_ty.lookup(db) {
					TyData::Set(_, _, e) => e,
					_ => unreachable!(),
				};
				self.add_enum_cases(db, item, data, *ty, &it.definition);
			}
			Item::Solve(s) => {
				let it = s.solve(db);
				match &it.goal {
					Goal::Maximize { pattern, objective }
					| Goal::Minimize { pattern, objective } => {
						self.add_declaration(db, *pattern, PatternTy::Computing);
						let actual =
							Typer::new(db, self, item, data).collect_expression(*objective);
						if !actual.is_subtype_of(db, TypeRegistry::lookup(db).var_float) {
							let (src, span) =
								ExpressionRef::new(db, item, *objective).source_span(db);
							self.add_diagnostic(
								db,
								item,
								TypeMismatch {
									src,
									span,
									msg: format!(
										"Objective must be numeric, but got '{}'",
										actual.pretty_print(db)
									),
								},
							);
						}
						self.add_declaration(db, *pattern, PatternTy::Variable(actual));
					}
					_ => (),
				}
			}
			Item::TypeAlias(t) => {
				let it = t.type_alias(db);
				self.add_declaration(db, it.name, PatternTy::Computing);
				let result = Typer::new(db, self, item, data).complete_type(
					it.aliased_type,
					None,
					TypeCompletionMode::Default,
				);
				self.add_declaration(
					db,
					it.name,
					PatternTy::TypeAlias {
						ty: result.ty,
						has_bounded: result.has_bounded,
						has_unbounded: result.has_unbounded,
					},
				);
			}
			Item::Class(c) => {
				let it = c.class(db);
				let tys = TypeRegistry::lookup(db);
				let class_pattern = PatternRef::new(db, item, it.pattern);
				let name = class_pattern
					.identifier(db)
					.expect("Class declaration must have identifier pattern");

				// Whether this class's *actual set* is var, i.e. its existence is a
				// solver decision, in which case its defining set is `var set of C`.
				// This is computed from the AST and the global scope alone, so
				// consulting it here does not pull signature typing into a cycle.
				// Note this is var-*actual-set*, not var-reached: a singular
				// `var new C` has var storage but a par actual set, because the
				// object always exists.
				let is_var_actual = var_actual_set_classes(db).contains(&class_pattern);
				let defining_set_ty = |class_ty: Ty<'db>| {
					let par = Ty::par_set(db, class_ty).unwrap();
					if is_var_actual {
						par.make_var(db).unwrap_or(par)
					} else {
						par
					}
				};

				self.add_declaration(db, it.pattern, PatternTy::Computing);

				// Type the superclass first: a class type carries its superclass, so
				// the class's own type is only fixed once the superclass is known.
				let mut superclass = None;
				let mut input_fields = Vec::new();
				let mut storage_fields = Vec::new();
				if let Some(base) = it.extends {
					let base_ty = Typer::new(db, self, item, data).collect_expression(base);
					if let Some(base_class) = base_ty.class_type(db) {
						superclass = Some(base_ty);
						let Some(base_pattern) = class_pattern_for(db, base_class) else {
							unreachable!("Class type must have a declaring item")
						};
						match self.type_pattern(db, base_pattern) {
							PatternTy::ClassDecl {
								input_record_ty,
								storage_record_ty,
								..
							} => {
								// The superclass's record types already include what it
								// inherits in turn, so the chain is walked implicitly.
								if let Some(fields) = input_record_ty.record_fields(db) {
									input_fields.extend(
										fields.into_iter().map(|(i, ty)| (Identifier(i), ty)),
									);
								}
								if let Some(fields) = storage_record_ty.record_fields(db) {
									storage_fields.extend(
										fields.into_iter().map(|(i, ty)| (Identifier(i), ty)),
									);
								}
							}
							_ => unreachable!("Class type must resolve to a class declaration"),
						}
					} else {
						let (src, span) = ExpressionRef::new(db, item, base).source_span(db);
						self.add_diagnostic(
							db,
							item,
							TypeMismatch {
								src,
								span,
								msg: format!(
									"Expected class, but got '{}'",
									base_ty.pretty_print(db)
								),
							},
						);
					}
				}

				let class_ty = Ty::class(db, ClassRef::new(name, superclass));

				// Publish the class before typing its attributes, so that an
				// attribute whose type refers to the class being declared (directly
				// or through a cycle of classes) can resolve it. The record types are
				// not known yet; the declaration is replaced once they are.
				self.add_declaration(
					db,
					it.pattern,
					PatternTy::ClassDecl {
						attributes: Vec::new(),
						defining_set_ty: defining_set_ty(class_ty),
						input_record_ty: tys.error,
						storage_record_ty: tys.error,
						var_storage_record_ty: tys.error,
					},
				);
				self.add_declaration(db, it.this_pattern, PatternTy::Variable(class_ty));

				let mut typer = Typer::new(db, self, item, data);
				for ann in it.annotations.iter() {
					let _ = typer.typecheck_expression(*ann, tys.ann);
				}

				let mut attributes = Vec::new();
				for member in it.items.iter() {
					match member {
						ClassMember::Constraint(constraint) => {
							let mut typer = Typer::new(db, self, item, data);
							for ann in constraint.annotations.iter() {
								let _ = typer.typecheck_expression(*ann, tys.ann);
							}
							let _ = typer.typecheck_expression(constraint.expression, tys.var_bool);
						}
						ClassMember::Declaration(d) => {
							let mut typer = Typer::new(db, self, item, data);
							let field_name = PatternRef::new(db, item, d.pattern)
								.identifier(db)
								.expect("Class attribute must have identifier pattern");
							let ty = typer.collect_declaration(d).ty;
							// A computed attribute (`int: y = x + 1`) is part of the
							// object's storage but is never supplied by the caller, so
							// it stays out of the input record while remaining in the
							// storage record. Without this, `new A: a = (x: 1)` would be
							// rejected for not supplying the `y` the caller must not give.
							if d.definition.is_none()
								&& let Some(field_ty) =
									typer.class_type_to_input_record_type(d.declared_type)
							{
								input_fields.push((field_name, field_ty));
							}
							attributes.push((field_name, ty));
							storage_fields.push((field_name, ty));
						}
					}
				}

				// The varified storage record, used when this class is reached via a
				// `var new` introduction path, so that the lowered per-class object
				// arrays can be declared with a var element type from the start.
				let var_storage_record_ty = Ty::record(
					db,
					storage_fields.iter().map(|(name, ty)| {
						let varified = ty.make_var(db).or_else(|| {
							// An array has no var form as a whole; var storage for an
							// array attribute is an array of var elements.
							match ty.lookup(db) {
								TyData::Array { opt, dim, element } => element
									.make_var(db)
									.and_then(|el| Ty::array(db, *dim, el))
									.map(|t| t.with_opt(db, *opt)),
								_ => None,
							}
						});
						(*name, varified.unwrap_or(*ty))
					}),
				);
				self.add_declaration(
					db,
					it.pattern,
					PatternTy::ClassDecl {
						attributes,
						defining_set_ty: defining_set_ty(class_ty),
						input_record_ty: Ty::record(db, input_fields),
						storage_record_ty: Ty::record(db, storage_fields),
						var_storage_record_ty,
					},
				);
			}
			_ => unreachable!("Item {:?} does not have signature", item),
		}
	}

	fn add_enum_cases(
		&mut self,
		db: &'db dyn Db,
		item: Item<'db>,
		data: &ItemData<'db>,
		ty: Ty<'db>,
		cases: &[EnumConstructor<'db>],
	) {
		let get_param_types = |ctx: &mut SignatureTypeContext<'db>,
		                       parameters: &[ConstructorParameter<'db>]| {
			let param_types = {
				let mut typer = Typer::new(db, ctx, item, data);
				parameters
					.iter()
					.map(|p| {
						typer
							.complete_type(
								p.declared_type,
								None,
								TypeCompletionMode::EnumerationParameter,
							)
							.ty
					})
					.collect::<Box<[_]>>()
			};

			let mut had_error = false;
			for (p, t) in parameters.iter().zip(param_types.iter()) {
				if t.contains_error(db) {
					had_error = true;
				}
				if !t.known_par(db) || !t.known_enumerable(db) {
					let (src, span) = TypeRef::new(db, item, p.declared_type).source_span(db);
					ctx.add_diagnostic(
						db,
						item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected par enumerable constructor parameter, but got '{}'",
								t.pretty_print(db)
							),
						},
					);
					had_error = true;
				}
			}

			(had_error, param_types)
		};

		for case in cases.iter() {
			match case {
				EnumConstructor::Named(Constructor::Atom { pattern }) => {
					self.add_declaration(db, *pattern, PatternTy::EnumAtom(ty));
				}
				EnumConstructor::Named(Constructor::Function {
					constructor,
					destructor,
					parameters,
				}) => {
					let (had_error, param_types) = get_param_types(self, parameters);
					let is_single = param_types.len() == 1;
					let mut constructors = Vec::with_capacity(6);
					let mut destructors = Vec::with_capacity(6);

					let mut add_ctor = |e: Ty<'db>, ps: Box<[Ty<'db>]>, l: bool| {
						destructors.push(FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: if is_single {
									ps[0]
								} else {
									Ty::tuple(db, ps.iter().copied())
								},
								params: Box::new([e]),
							}),
						});
						constructors.push(EnumConstructorEntry {
							constructor: FunctionEntry {
								has_body: false,
								overload: OverloadedFunction::Function(FunctionType {
									return_type: e,
									params: ps,
								}),
							},
							is_lifted: l,
						});
					};

					// C(a, b, ..) -> E
					add_ctor(ty, param_types.clone(), false);
					if !had_error {
						// C(var a, var b, ..) -> var E
						add_ctor(
							ty.make_var(db).unwrap(),
							param_types
								.iter()
								.map(|t| t.make_var(db).unwrap())
								.collect::<Box<_>>(),
							false,
						);

						// C(opt a, opt b, ..) -> opt E
						add_ctor(
							ty.make_opt(db),
							param_types
								.iter()
								.map(|t| t.make_opt(db))
								.collect::<Box<_>>(),
							true,
						);
						// C(var opt a, var opt b, ..) -> var opt E
						add_ctor(
							ty.make_var(db).unwrap().make_opt(db),
							param_types
								.iter()
								.map(|t| t.make_var(db).unwrap().make_opt(db))
								.collect(),
							true,
						);
						// C(set of a, set of b, ..) -> set of E
						add_ctor(
							Ty::par_set(db, ty).unwrap(),
							param_types
								.iter()
								.map(|t| Ty::par_set(db, *t).unwrap())
								.collect(),
							true,
						);
						// C(var set of a, var set of b, ..) -> var set of E
						add_ctor(
							Ty::par_set(db, ty).unwrap().make_var(db).unwrap(),
							param_types
								.iter()
								.map(|t| Ty::par_set(db, *t).unwrap().make_var(db).unwrap())
								.collect(),
							true,
						);
					}

					self.add_declaration(
						db,
						*constructor,
						PatternTy::EnumConstructor(constructors.into_boxed_slice()),
					);
					self.add_declaration(
						db,
						*destructor,
						PatternTy::EnumDestructure(destructors.into_boxed_slice()),
					);
				}
				EnumConstructor::Anonymous {
					pattern,
					parameters,
				} => {
					let (_, param_tys) = get_param_types(self, parameters);
					self.add_declaration(
						db,
						*pattern,
						PatternTy::AnonymousEnumConstructor(Box::new(FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: ty,
								params: param_tys,
							}),
						})),
					);
				}
			}
		}
	}

	/// Get results of typing
	fn finish(self) -> SignatureTypesResult<'db> {
		SignatureTypesResult {
			signature: self.data,
			errors: self.diagnostics,
		}
	}
}

impl<'db> TypeContext<'db> for SignatureTypeContext<'db> {
	fn add_declaration(
		&mut self,
		_db: &'db dyn Db,
		pattern: PatternId<'db>,
		declaration: PatternTy<'db>,
	) {
		let old = self.data.patterns.insert(pattern, declaration);
		// A class is published provisionally before its attributes are typed, so that
		// an attribute type may refer to the class being declared; the provisional
		// declaration is then replaced with the complete one.
		assert!(
			matches!(
				old,
				None | Some(PatternTy::Computing | PatternTy::ClassDecl { .. })
			),
			"Tried to add declaration for {:?} twice",
			pattern
		);
	}

	fn add_expression(&mut self, _db: &'db dyn Db, expression: ExpressionId<'db>, ty: Ty<'db>) {
		let old = self.data.expressions.insert(expression, ty);
		assert!(
			old.is_none(),
			"Tried to add type for expression {:?} twice",
			expression
		);
	}

	fn add_identifier_resolution(
		&mut self,
		_db: &'db dyn Db,
		expression: ExpressionId<'db>,
		resolution: PatternRef<'db>,
	) {
		let old = self
			.data
			.identifier_resolution
			.insert(expression, resolution);
		assert!(
			old.is_none(),
			"Tried to add identifier resolution for {:?} twice",
			expression
		);
	}

	fn add_pattern_resolution(
		&mut self,
		_db: &'db dyn Db,
		pattern: PatternId<'db>,
		resolution: PatternRef<'db>,
	) {
		let old = self.data.pattern_resolution.insert(pattern, resolution);
		assert!(
			old.is_none(),
			"Tried to add pattern resolution for {:?} twice",
			pattern
		);
	}

	fn add_diagnostic(&mut self, _db: &'db dyn Db, item: Item<'db>, e: impl Into<Error>) {
		let error = e.into();
		assert_eq!(item, self.item, "Got error '{}' for wrong item", error);
		self.diagnostics.add_error(error);
	}

	fn add_warning(&mut self, _db: &'db dyn Db, item: Item<'db>, e: impl Into<Warning>) {
		let warning = e.into();
		assert_eq!(item, self.item, "Got warning '{}' for wrong item", warning);
		self.diagnostics.add_warning(warning);
	}

	fn add_type(&mut self, _db: &'db dyn Db, declared_type: TypeId<'db>, ty: Ty<'db>) {
		let _ = self.data.types.insert(declared_type, ty);
	}

	fn get_type(&self, db: &'db dyn Db, declared_type: TypeId<'db>) -> Ty<'db> {
		// A type which failed to complete is never recorded; an error was already
		// reported for it, so report the error type here rather than panicking.
		self.data
			.types
			.get(&declared_type)
			.copied()
			.unwrap_or_else(|| TypeRegistry::lookup(db).error)
	}

	fn type_pattern(&mut self, db: &'db dyn Db, pattern: PatternRef<'db>) -> PatternTy<'db> {
		let item = pattern.item(db);
		let pat = pattern.pattern(db);
		if item == self.item {
			return self.data.patterns[&pat].clone();
		}
		// Use item_signature so we can recover from cycles
		let Some(result) = item.possibly_cyclic_signature(db) else {
			return PatternTy::Computing;
		};
		result.patterns[&pat].clone()
	}
}
