/// Types of signatures - the type required when referring to an item
///
/// E.g.
/// - Function parameter/return type
/// - Variable declaration LHS types
use rustc_hash::FxHashMap;

use crate::{
	diagnostics::{SyntaxError, TypeInferenceFailure, TypeMismatch},
	hir::{
		db::Hir,
		ids::{EntityRef, ExpressionRef, ItemRef, LocalItemRef, NodeRef, PatternRef},
		Constructor, ConstructorParameter, EnumConstructor, Goal, ItemData, Pattern, Type,
	},
	ty::{
		EnumRef, FunctionEntry, FunctionType, OptType, OverloadedFunction, PolymorphicFunctionType,
		Ty, TyData, TyVar, TyVarRef, VarType,
	},
	Error,
};

use super::{EnumConstructorEntry, NameResolution, PatternTy, TypeContext, Typer};

/// Collected types for an item signature
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureTypes {
	/// Types of declarations
	pub patterns: FxHashMap<PatternRef, PatternTy>,
	/// Types of expressions
	pub expressions: FxHashMap<ExpressionRef, Ty>,
	/// Identifier resolution
	pub identifier_resolution: FxHashMap<ExpressionRef, NameResolution>,
	/// Pattern resolution
	pub pattern_resolution: FxHashMap<PatternRef, NameResolution>,
}

/// Context for typing an item signature
pub struct SignatureTypeContext {
	starting_item: ItemRef,
	data: SignatureTypes,
	diagnostics: Vec<Error>,
}

impl SignatureTypeContext {
	/// Create a new signature type context
	pub fn new(item: ItemRef) -> Self {
		Self {
			starting_item: item,
			data: SignatureTypes {
				patterns: FxHashMap::default(),
				expressions: FxHashMap::default(),
				identifier_resolution: FxHashMap::default(),
				pattern_resolution: FxHashMap::default(),
			},
			diagnostics: Vec::new(),
		}
	}

	/// Compute the signature of the given item
	pub fn type_item(&mut self, db: &dyn Hir, item: ItemRef) {
		let model = &*item.model(db);
		let it = item.local_item_ref(db);
		let data = it.data(model);
		match it {
			LocalItemRef::Annotation(a) => {
				let it = &model[a];
				match &it.constructor {
					Constructor::Atom { pattern } => {
						self.add_declaration(
							PatternRef::new(item, *pattern),
							PatternTy::AnnotationAtom,
						);
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
								for t in Type::any_types(p.declared_type, &it.data) {
									let (src, span) =
										NodeRef::from(EntityRef::new(db, item, t)).source_span(db);
									self.add_diagnostic(
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
									db.type_registry().error
								} else {
									Typer::new(db, self, item, data)
										.complete_type(p.declared_type, None)
								};
								if let Some(pat) = p.pattern {
									self.add_declaration(
										PatternRef::new(item, pat),
										PatternTy::Argument(ty),
									);
								}
								ty
							})
							.collect::<Box<_>>();
						let ann = db.type_registry().ann;
						let dtor = FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: if params.len() == 1 {
									params[0]
								} else {
									Ty::tuple(db.upcast(), params.iter().copied())
								},
								params: Box::new([ann]),
							}),
						};
						self.add_declaration(
							PatternRef::new(item, *destructor),
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
							PatternRef::new(item, *constructor),
							PatternTy::AnnotationConstructor(Box::new(ctor)),
						);
					}
				}
			}
			LocalItemRef::Function(f) => {
				let it = &model[f];
				// Set as computing so if there's a call to a function with this name we can break the cycle
				// (since if the call is actually not referring to this overload, it should work)
				self.add_declaration(PatternRef::new(item, it.pattern), PatternTy::Computing);
				let ty_params = it
					.type_inst_vars
					.iter()
					.map(|tv| {
						let ty_var = TyVarRef::new(db, PatternRef::new(item, tv.name));
						let type_var = TyVar {
							ty_var,
							varifiable: tv.is_varifiable,
							enumerable: tv.is_enum,
							indexable: tv.is_indexable,
						};
						self.add_declaration(
							PatternRef::new(item, tv.name),
							PatternTy::TyVar(type_var),
						);
						ty_var
					})
					.collect::<Box<[_]>>();
				let params = it
					.parameters
					.iter()
					.enumerate()
					.map(|(i, p)| {
						let mut had_error = false;
						let annotated_expression = p
							.annotations
							.iter()
							.find(|ann| match &it.data[**ann] {
								crate::hir::Expression::Identifier(i) => {
									i.lookup(db) == "annotated_expression"
								}
								_ => false,
							})
							.copied();
						if i > 0 {
							if let Some(ann) = annotated_expression {
								let (src, span) =
									NodeRef::from(EntityRef::new(db, item, ann)).source_span(db);
								self.add_diagnostic(
									item,
									SyntaxError {
										src,
										span,
										msg: "'annotated_expression' only allowed on first function parameter.".to_owned(),
										other: Vec::new(),
									},
								);
							}
						}
						for t in Type::any_types(p.declared_type, &it.data) {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, item, t)).source_span(db);
							self.add_diagnostic(
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
							db.type_registry().error
						} else {
							typer.complete_type(p.declared_type, None)
						};
						if let Some(pat) = p.pattern {
							typer.collect_pattern(None, false, pat, ty, true);
						}
						ty
					})
					.collect();
				let pattern = PatternRef::new(item, it.pattern);
				if ty_params.is_empty() {
					let f = FunctionType {
						return_type: db.type_registry().error,
						params,
					};
					self.add_declaration(
						pattern,
						PatternTy::Function(Box::new(FunctionEntry {
							has_body: it.body.is_some(),
							overload: OverloadedFunction::Function(f),
						})),
					);
				} else {
					let p = PolymorphicFunctionType {
						return_type: db.type_registry().error,
						ty_params,
						params,
					};
					self.add_declaration(
						pattern,
						PatternTy::Function(Box::new(FunctionEntry {
							has_body: it.body.is_some(),
							overload: OverloadedFunction::PolymorphicFunction(p),
						})),
					);
				}

				let mut had_error = false;
				for t in Type::any_types(it.return_type, &it.data)
					.chain(Type::anonymous_ty_vars(it.return_type, &it.data))
				{
					let (src, span) = NodeRef::from(EntityRef::new(db, item, t)).source_span(db);
					self.add_diagnostic(
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
					db.type_registry().error
				} else {
					Typer::new(db, self, item, data).complete_type(it.return_type, None)
				};

				let d = self.data.patterns.get_mut(&pattern).unwrap();
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
			LocalItemRef::Declaration(d) => {
				let it = &model[d];
				for p in Pattern::identifiers(it.pattern, data) {
					self.add_declaration(PatternRef::new(item, p), PatternTy::Computing);
				}
				let mut typer = Typer::new(db, self, item, data);
				if data[it.declared_type].is_complete(data) {
					// Use LHS type only
					let expected = typer.complete_type(it.declared_type, None);
					typer.collect_pattern(None, false, it.pattern, expected, false);
				} else {
					typer.collect_declaration(it);
				}
			}
			LocalItemRef::Enumeration(e) => {
				let it = &model[e];
				let ty = Ty::par_enum(
					db.upcast(),
					EnumRef::new(db, PatternRef::new(item, it.pattern)),
				);
				self.add_declaration(
					PatternRef::new(item, it.pattern),
					PatternTy::Enum(Ty::par_set(db.upcast(), ty).unwrap()),
				);
				if let Some(cases) = &it.definition {
					self.add_enum_cases(db, item, data, ty, cases);
				}
			}
			LocalItemRef::EnumAssignment(e) => {
				let it = &model[e];
				let set_ty = Typer::new(db, self, item, data).collect_expression(it.assignee);
				let ty = match set_ty.lookup(db.upcast()) {
					TyData::Set(_, _, e) => e,
					_ => unreachable!(),
				};
				self.add_enum_cases(db, item, data, ty, &it.definition);
			}
			LocalItemRef::Solve(s) => {
				let it = &model[s];
				match &it.goal {
					Goal::Maximize { pattern, objective }
					| Goal::Minimize { pattern, objective } => {
						self.add_declaration(PatternRef::new(item, *pattern), PatternTy::Computing);
						let actual =
							Typer::new(db, self, item, data).collect_expression(*objective);
						if !actual.is_subtype_of(db.upcast(), db.type_registry().var_float) {
							let (src, span) =
								NodeRef::from(EntityRef::new(db, item, *objective)).source_span(db);
							self.add_diagnostic(
								item,
								TypeMismatch {
									src,
									span,
									msg: format!(
										"Objective must be numeric, but got '{}'",
										actual.pretty_print(db.upcast())
									),
								},
							);
						}
						self.add_declaration(
							PatternRef::new(item, *pattern),
							PatternTy::Variable(actual),
						);
					}
					_ => (),
				}
			}
			LocalItemRef::TypeAlias(t) => {
				let it = &model[t];
				let pat = PatternRef::new(item, it.name);
				self.add_declaration(pat, PatternTy::Computing);
				let ty = Typer::new(db, self, item, data).complete_type(it.aliased_type, None);
				self.add_declaration(pat, PatternTy::TypeAlias(ty));
			}
			_ => unreachable!("Item {:?} does not have signature", it),
		}
	}

	fn add_enum_cases(
		&mut self,
		db: &dyn Hir,
		item: ItemRef,
		data: &ItemData,
		ty: Ty,
		cases: &[EnumConstructor],
	) {
		let get_param_types = |ctx: &mut SignatureTypeContext,
		                       parameters: &[ConstructorParameter]| {
			let param_types = {
				let mut typer = Typer::new(db, ctx, item, data);
				parameters
					.iter()
					.map(|p| typer.complete_type(p.declared_type, None))
					.collect::<Box<[_]>>()
			};

			let mut had_error = false;
			for (p, t) in parameters.iter().zip(param_types.iter()) {
				if t.contains_error(db.upcast()) {
					had_error = true;
				}
				if !t.known_par(db.upcast()) || !t.known_enumerable(db.upcast()) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, item, p.declared_type)).source_span(db);
					ctx.add_diagnostic(
						item,
						TypeMismatch {
							src,
							span,
							msg: format!(
								"Expected par enumerable constructor parameter, but got '{}'",
								t.pretty_print(db.upcast())
							),
						},
					);
					had_error = true;
				}

				for unbounded in Type::primitives(p.declared_type, data) {
					let (src, span) =
						NodeRef::from(EntityRef::new(db, item, unbounded)).source_span(db);
					ctx.add_diagnostic(
						item,
						TypeInferenceFailure {
							src,
							span,
							msg: "Unbounded enum constructor parameters not supported".to_owned(),
						},
					);
				}
			}

			(had_error, param_types)
		};

		for case in cases.iter() {
			match case {
				EnumConstructor::Named(Constructor::Atom { pattern }) => {
					self.add_declaration(PatternRef::new(item, *pattern), PatternTy::EnumAtom(ty));
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

					let mut add_ctor = |e: Ty, ps: Box<[Ty]>, l: bool| {
						destructors.push(FunctionEntry {
							has_body: false,
							overload: OverloadedFunction::Function(FunctionType {
								return_type: if is_single {
									ps[0]
								} else {
									Ty::tuple(db.upcast(), ps.iter().copied())
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
							ty.with_inst(db.upcast(), VarType::Var).unwrap(),
							param_types
								.iter()
								.map(|t| t.with_inst(db.upcast(), VarType::Var).unwrap())
								.collect::<Box<_>>(),
							false,
						);

						// C(opt a, opt b, ..) -> opt E
						add_ctor(
							ty.with_opt(db.upcast(), OptType::Opt),
							param_types
								.iter()
								.map(|t| t.with_opt(db.upcast(), OptType::Opt))
								.collect::<Box<_>>(),
							true,
						);
						// C(var opt a, var opt b, ..) -> var opt E
						add_ctor(
							ty.with_inst(db.upcast(), VarType::Var)
								.unwrap()
								.with_opt(db.upcast(), OptType::Opt),
							param_types
								.iter()
								.map(|t| {
									t.with_inst(db.upcast(), VarType::Var)
										.unwrap()
										.with_opt(db.upcast(), OptType::Opt)
								})
								.collect(),
							true,
						);
						// C(set of a, set of b, ..) -> set of E
						add_ctor(
							Ty::par_set(db.upcast(), ty).unwrap(),
							param_types
								.iter()
								.map(|t| Ty::par_set(db.upcast(), *t).unwrap())
								.collect(),
							true,
						);
						// C(var set of a, var set of b, ..) -> var set of E
						add_ctor(
							Ty::par_set(db.upcast(), ty)
								.unwrap()
								.with_inst(db.upcast(), VarType::Var)
								.unwrap(),
							param_types
								.iter()
								.map(|t| {
									Ty::par_set(db.upcast(), *t)
										.unwrap()
										.with_inst(db.upcast(), VarType::Var)
										.unwrap()
								})
								.collect(),
							true,
						);
					}

					self.add_declaration(
						PatternRef::new(item, *constructor),
						PatternTy::EnumConstructor(constructors.into_boxed_slice()),
					);
					self.add_declaration(
						PatternRef::new(item, *destructor),
						PatternTy::EnumDestructure(destructors.into_boxed_slice()),
					);
				}
				EnumConstructor::Anonymous {
					pattern,
					parameters,
				} => {
					let (_, param_tys) = get_param_types(self, parameters);
					self.add_declaration(
						PatternRef::new(item, *pattern),
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
	pub fn finish(self) -> (SignatureTypes, Vec<Error>) {
		(self.data, self.diagnostics)
	}
}

impl TypeContext for SignatureTypeContext {
	fn add_declaration(&mut self, pattern: PatternRef, declaration: PatternTy) {
		let old = self.data.patterns.insert(pattern, declaration);
		assert!(
			matches!(old, None | Some(PatternTy::Computing)),
			"Tried to add declaration for {:?} twice",
			pattern
		);
	}
	fn add_expression(&mut self, expression: ExpressionRef, ty: Ty) {
		let old = self.data.expressions.insert(expression, ty);
		assert!(
			old.is_none(),
			"Tried to add type for expression {:?} twice",
			expression
		);
	}
	fn add_identifier_resolution(&mut self, expression: ExpressionRef, resolution: NameResolution) {
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
	fn add_pattern_resolution(&mut self, pattern: PatternRef, resolution: NameResolution) {
		let old = self.data.pattern_resolution.insert(pattern, resolution);
		assert!(
			old.is_none(),
			"Tried to add pattern resolution for {:?} twice",
			pattern
		);
	}
	fn add_diagnostic(&mut self, item: ItemRef, e: impl Into<Error>) {
		// Suppress errors from other items
		if item == self.starting_item {
			self.diagnostics.push(e.into());
		}
	}

	fn type_pattern(&mut self, db: &dyn Hir, pattern: PatternRef) -> PatternTy {
		// When computing signatures, we always type everything required
		// So other signatures get typed as well
		if let Some(d) = self.data.patterns.get(&pattern).cloned() {
			return d;
		}
		self.type_item(db, pattern.item());
		self.data.patterns[&pattern].clone()
	}
}
