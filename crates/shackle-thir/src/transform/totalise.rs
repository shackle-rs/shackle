//! Totalisation
//!
//! Rewrites model such that all expressions are total

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::{BooleanLiteral, StringLiteral, constants::IdentifierRegistry};
use shackle_ty::{OptType, Ty, registry::TypeRegistry};
use shackle_utils::{arena::ArenaMap, hash::Set, maybe_grow_stack};

use crate::{
	ArrayComprehension, ArrayLiteral, Branch, Call, Callable, Constraint, Db, Declaration,
	DeclarationId, Domain, DomainData, DummyValue, Expression, ExpressionBuilder, ExpressionData,
	Function, FunctionId, FunctionItem, FunctionName, Generator, IfThenElse, IntegerLiteral, Item,
	Let, LetItem, LookupCall, LookupIdentifier, Marker, Model, ResolvedIdentifier, SetLiteral,
	TupleAccess, TupleLiteral,
	analyse::{Mode, ModeAnalysis, Totality, TotalityResult, analyse_totality},
	pretty_print::PrettyPrinter,
	source::Origin,
	traverse::{
		Folder, ReplacementMap, add_model, fold_declaration_id, fold_expression, fold_function_id,
	},
};

struct Totaliser<'a, 'db, Dst: Marker> {
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
	totalised_model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst>,
	totality: ArenaMap<FunctionItem<'db>, TotalityResult>,
	modes: &'a ModeAnalysis<'a, 'db>,
	root_fn_map: FxHashMap<FunctionId<'db>, FunctionId<'db, Dst>>,
	root_fn_decl_map: FxHashMap<DeclarationId<'db>, DeclarationId<'db, Dst>>,
	missing_reif_generated: Set<FunctionId<'db, Dst>>,
	in_root_fns: bool,
}

struct BoundExpression<'db, Dst: Marker> {
	declaration: DeclarationId<'db, Dst>,
	ident: Expression<'db, Dst>,
}

struct BoundPartialExpression<'db, Dst: Marker> {
	declaration: DeclarationId<'db, Dst>,
	ident: Expression<'db, Dst>,
	definedness: Expression<'db, Dst>,
	value: Expression<'db, Dst>,
}

impl<'a, 'db, Dst: Marker> Folder<'_, 'db, Dst> for Totaliser<'a, 'db, Dst> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.totalised_model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, ()> {
		&mut self.replacement_map
	}

	fn add_model(&mut self, db: &'db dyn Db, model: &Model<'db>) {
		add_model(self, db, model);
		// Add bodies for root versions of functions
		log::debug!("Adding bodies for root versions of functions");
		self.in_root_fns = true;
		for (f, i) in model.all_functions() {
			if i.body().is_some()
				&& self.root_fn_map.contains_key(&f)
				&& !self.already_has_root_version(f)
			{
				self.fold_function_body(db, model, f);
			}
		}
		// Add bodies for missign reif versions which abort
		for f in self.missing_reif_generated.iter() {
			let origin = self.totalised_model[*f].origin();
			let body = Expression::new(
				db,
				&self.totalised_model,
				origin,
				LookupCall {
					function: self.ids.functions.abort.into(),
					arguments: vec![Expression::new(
						db,
						&self.totalised_model,
						origin,
						StringLiteral::new(
							db,
							format!(
								"{} required, but not supported by solver.",
								self.totalised_model[*f].name().pretty_print(db)
							),
						),
					)],
				},
			);
			self.totalised_model[*f].set_body(body);
		}
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
		if model[f].name() == self.ids.functions.mzn_default_partial {
			// `default` calls rewritten so function no longer needed
			return;
		}
		if model[f].name() == self.ids.functions.mzn_in_root_context {
			// Root context already detected, so function no longer needed
			return;
		}

		let function_totality = self.totality[f];

		log::debug!(
			"{} is {:?}",
			PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
			function_totality
		);

		let orig_return = model[f].return_type();
		let return_type = if self.is_boolean_ty(orig_return) {
			// Booleans remain boolean
			orig_return
		} else {
			match function_totality.totality {
				Totality::Total => orig_return,
				Totality::ParPartial => Ty::tuple(db, [self.tys.par_bool, orig_return]),
				Totality::VarPartial => Ty::tuple(db, [self.tys.var_bool, orig_return]),
			}
		};

		if orig_return == self.tys.var_bool
			&& model[f].body().is_none()
			&& !model[f].is_polymorphic()
		{
			let reif_name = model[f].name().reif(db);
			if model.all_functions().all(|(_, func)| {
				func.name() != reif_name || func.mangled_param_tys() != model[f].mangled_param_tys()
			}) {
				// Add reif version that aborts if none exists
				let reif_idx = self.add_fn_decl(db, model, f, reif_name, orig_return);

				// These annotations don't apply to the reif version, so remove them
				let mut annotations =
					std::mem::take(self.totalised_model[reif_idx].annotations_mut());
				let _ = annotations.remove(
					&self.totalised_model,
					self.ids.annotations.promise_commutative,
				);
				let _ = annotations.remove(
					&self.totalised_model,
					self.ids.annotations.promise_ctx_monotone,
				);
				let _ = annotations.remove(
					&self.totalised_model,
					self.ids.annotations.promise_ctx_antitone,
				);
				let _ = std::mem::replace(
					self.totalised_model[reif_idx].annotations_mut(),
					annotations,
				);

				let r_param = Declaration::new(
					false,
					Domain::unbounded(db, model[f].origin(), self.tys.var_bool),
				);
				let r_param_idx = self
					.totalised_model
					.add_declaration(Item::new(r_param, model[f].origin()));
				self.totalised_model[reif_idx].add_parameter(r_param_idx);

				let _ = self.missing_reif_generated.insert(reif_idx);
			}
		}

		if function_totality.needs_root
			|| (model[f].body().is_some() && self.is_boolean_ty(orig_return))
		{
			let root_name = model[f].name().root(db);

			// Add root version
			assert!(
				model
					.all_functions()
					.all(|(_, func)| func.name() != root_name
						|| func.mangled_param_tys() != model[f].mangled_param_tys()),
				"Root version of {} already exists",
				PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
			);

			let root_idx = self.add_fn_decl(db, model, f, root_name, orig_return);
			let old = self.root_fn_map.insert(f, root_idx);
			assert!(
				old.is_none(),
				"Tried to add another root version of {} to root fn map",
				PrettyPrinter::new(db, model).pretty_print_signature(f.into())
			);
			for (idx, root) in model[f]
				.parameters()
				.iter()
				.zip(self.totalised_model[root_idx].parameters())
			{
				let _ = self.root_fn_decl_map.insert(*idx, *root);
			}
		}

		// Added after, so replacement map will contain totalised versions
		let _ = self.add_fn_decl(db, model, f, model[f].name(), return_type);
	}

	fn fold_function_id(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		f: FunctionId<'db>,
	) -> FunctionId<'db, Dst> {
		if self.in_root_fns {
			// Note that when totalising a call, we check for the root version there instead.
			self.root_fn_map[&f]
		} else {
			fold_function_id(self, db, model, f)
		}
	}

	fn fold_function_body(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
		log::debug!("Adding body for {}", {
			let idx = self.fold_function_id(db, model, f);
			PrettyPrinter::new(db, &self.totalised_model).pretty_print_signature(idx.into())
		});
		let body = model[f].body().unwrap();
		let folded = self.fold_expression(db, model, body);
		let body = self.wrap_root_ctx_bool(db, body, folded);
		let dst = self.fold_function_id(db, model, f);
		self.totalised_model[dst].set_body(body);
		self.totalised_model[dst].validate(db);
	}

	fn fold_declaration_id(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		d: DeclarationId<'db>,
	) -> DeclarationId<'db, Dst> {
		if self.in_root_fns
			&& let Some(idx) = self.root_fn_decl_map.get(&d)
		{
			return *idx;
		}
		fold_declaration_id(self, db, model, d)
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		expression: &Expression<'db>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			let origin = expression.origin();
			let mut e = match &**expression {
				ExpressionData::ArrayLiteral(al) => {
					self.totalise_collection_literal(db, model, al.iter(), origin, ArrayLiteral)
				}
				ExpressionData::SetLiteral(sl) => {
					self.totalise_collection_literal(db, model, sl.iter(), origin, SetLiteral)
				}
				ExpressionData::TupleLiteral(tl) => {
					self.totalise_collection_literal(db, model, tl.iter(), origin, TupleLiteral)
				}
				ExpressionData::ArrayComprehension(c) => {
					self.totalise_comprehension(db, model, c, origin)
				}
				ExpressionData::TupleAccess(ta) => {
					self.totalise_tuple_access(db, model, ta, origin)
				}
				ExpressionData::IfThenElse(ite) => {
					self.totalise_if_then_else(db, model, ite, origin, expression)
				}
				ExpressionData::Case(_c) => todo!(),
				ExpressionData::Call(c) => self.totalise_call(db, model, c, origin, expression),
				ExpressionData::Let(l) => self.totalise_let(db, model, l, expression),
				ExpressionData::Lambda(_l) => todo!(),
				_ => return fold_expression(self, db, model, expression),
			};
			e.annotations_mut().extend(
				expression
					.annotations()
					.iter()
					.map(|ann| self.fold_expression(db, model, ann)),
			);
			e
		})
	}
}

impl<'a, 'db, Dst: Marker> Totaliser<'a, 'db, Dst> {
	fn get_mode(&self, e: &Expression<'db>) -> Mode {
		if self.in_root_fns {
			self.modes.get_in_root_fn(e)
		} else {
			self.modes.get(e)
		}
	}

	fn is_boolean_ty(&self, ty: Ty<'db>) -> bool {
		ty == self.tys.par_bool || ty == self.tys.var_bool
	}

	fn add_fn_decl(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		f: FunctionId<'db>,
		name: FunctionName<'db>,
		return_type: Ty<'db>,
	) -> FunctionId<'db, Dst> {
		let ff = &model[f];
		let mut function = Function::new(name, Domain::unbounded(db, ff.origin(), return_type));
		let remove_promise_total = name.is_root(db) && self.is_boolean_ty(return_type);
		function
			.annotations_mut()
			.extend(ff.annotations().iter().filter_map(|ann| {
				if remove_promise_total
					&& let ExpressionData::Identifier(ResolvedIdentifier::Annotation(ann)) = &**ann
					&& model[*ann].name.unwrap() == self.ids.annotations.promise_total
				{
					return None;
				}
				Some(self.fold_expression(db, model, ann))
			}));
		function.set_parameters(ff.parameters().iter().map(|p| {
			self.add_parameter_declaration(db, model, *p);
			self.fold_declaration_id(db, model, *p)
		}));
		function.set_type_inst_vars(ff.type_inst_vars().iter().cloned());
		function.set_specialised(ff.specialised_from());
		if let Some(tys) = ff.mangled_param_tys() {
			function.set_mangled_param_tys(tys.to_vec());
		}
		let idx = self
			.totalised_model
			.add_function(Item::new(function, ff.origin()));
		self.replacement_map().insert_function(f, idx);

		log::debug!(
			"Added {}",
			PrettyPrinter::new(db, &self.totalised_model).pretty_print_signature(idx.into()),
		);

		idx
	}

	fn totalise_collection_literal<'b, T: ExpressionBuilder<'db, Dst>>(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		values: impl Iterator<Item = &'b Expression<'db>>,
		origin: Origin<'db>,
		create: impl FnOnce(Vec<Expression<'db, Dst>>) -> T,
	) -> Expression<'db, Dst>
	where
		'db: 'b,
	{
		let mut is_partial = false;
		let members = values
			.map(|v| {
				let folded = self.fold_expression(db, model, v);
				let wrapped = self.wrap_root_ctx_bool(db, v, folded);
				let total = self.is_total(db, model, v, &wrapped);
				if !total {
					is_partial = true;
				}
				(total, wrapped)
			})
			.collect::<Vec<_>>();
		if is_partial {
			let mut items = Vec::with_capacity(members.len());
			let mut definedness = Vec::new();
			let mut values = Vec::with_capacity(members.len());
			for (t, v) in members {
				let o = v.origin();
				let decl = Declaration::from_expression(db, false, v);
				let idx = self.totalised_model.add_declaration(Item::new(decl, o));
				items.push(LetItem::Declaration(idx));
				let ident = Expression::new(
					db,
					&self.totalised_model,
					o,
					ResolvedIdentifier::Declaration(idx),
				);
				if t {
					values.push(ident);
				} else {
					definedness.push(self.tuple_access(db, o, ident.clone(), 1));
					values.push(self.tuple_access(db, o, ident, 2));
				}
			}
			let def = self.forall_call(
				db,
				Expression::new(db, &self.totalised_model, origin, ArrayLiteral(definedness)),
			);
			let al = Expression::new(db, &self.totalised_model, origin, create(values));
			let in_expression = Expression::new(
				db,
				&self.totalised_model,
				origin,
				TupleLiteral(vec![def, al]),
			);
			self.total_let(
				db,
				origin,
				Let {
					items,
					in_expression: Box::new(in_expression),
				},
			)
		} else {
			Expression::new(
				db,
				&self.totalised_model,
				origin,
				create(members.into_iter().map(|(_, e)| e).collect()),
			)
		}
	}

	fn totalise_comprehension(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		c: &ArrayComprehension<'db>,
		origin: Origin<'db>,
	) -> Expression<'db, Dst> {
		self.totalise_comprehension_inner(db, model, c, origin, 0, None, Vec::new())
	}

	#[allow(
		clippy::too_many_arguments,
		reason = "Helper carries recursive comprehension state"
	)]
	fn totalise_comprehension_inner(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		c: &ArrayComprehension<'db>,
		origin: Origin<'db>,
		mut gen_idx: usize,
		out_item: Option<DeclarationId<'db, Dst>>,
		mut out_generators: Vec<Generator<'db, Dst>>,
	) -> Expression<'db, Dst> {
		let mut var_where_clauses = Vec::new();
		let mut inner = None;
		while gen_idx < c.generators.len() {
			let generator = &c.generators[gen_idx];
			gen_idx += 1;
			match generator {
				Generator::Assignment {
					assignment,
					where_clause,
				} => {
					let orig_rhs = model[*assignment].definition().unwrap();
					let rhs = self.fold_expression(db, model, orig_rhs);
					let total = self.is_total(db, model, orig_rhs, &rhs);
					let is_var_where_clause = model[*assignment]
						.annotations()
						.has(model, self.ids.annotations.mzn_var_where_clause);
					let o = model[*assignment].origin();
					let mut asg_decl = Declaration::from_expression(db, false, rhs);
					if let Some(name) = model[*assignment].name() {
						asg_decl.set_name(name);
					}
					let idx = self.totalised_model.add_declaration(Item::new(asg_decl, o));
					if !total {
						// Make sure identifier refers to actual value
						let ident = Expression::new(
							db,
							&self.totalised_model,
							o,
							ResolvedIdentifier::Declaration(idx),
						);

						let value_decl = Declaration::from_expression(
							db,
							false,
							self.tuple_access(db, o, ident.clone(), 2),
						);
						let value_idx = self
							.totalised_model
							.add_declaration(Item::new(value_decl, o));
						self.replacement_map()
							.insert_declaration(*assignment, value_idx);

						let inner_generator = Generator::Assignment {
							assignment: value_idx,
							where_clause: where_clause
								.as_ref()
								.map(|w| self.fold_expression(db, model, w)),
						};

						inner = Some(self.totalise_comprehension_inner(
							db,
							model,
							c,
							origin,
							gen_idx,
							Some(idx),
							vec![inner_generator],
						));

						break;
					}
					self.replacement_map.insert_declaration(*assignment, idx);
					if is_var_where_clause {
						var_where_clauses.push(Expression::new(
							db,
							&self.totalised_model,
							o,
							ResolvedIdentifier::Declaration(idx),
						));
					}
					out_generators.push(Generator::Assignment {
						assignment: idx,
						where_clause: where_clause
							.as_ref()
							.map(|w| self.fold_expression(db, model, w)),
					});
				}
				Generator::Iterator {
					declarations,
					collection,
					where_clause,
				} => {
					let folded = self.fold_expression(db, model, collection);
					let partial = !self.is_total(db, model, collection, &folded);
					let o = collection.origin();
					let indices = declarations
						.iter()
						.map(|d| {
							self.add_iterator_declaration(db, model, *d);
							self.fold_declaration_id(db, model, *d)
						})
						.collect::<Vec<_>>();
					if partial {
						let decl = Declaration::from_expression(db, false, folded);
						let idx = self.totalised_model.add_declaration(Item::new(decl, o));

						let ident = Expression::new(
							db,
							&self.totalised_model,
							o,
							ResolvedIdentifier::Declaration(idx),
						);

						let inner_generator = Generator::Iterator {
							declarations: indices,
							collection: self.tuple_access(db, o, ident.clone(), 2),
							where_clause: where_clause
								.as_ref()
								.map(|w| self.fold_expression(db, model, w)),
						};

						inner = Some(self.totalise_comprehension_inner(
							db,
							model,
							c,
							origin,
							gen_idx,
							Some(idx),
							vec![inner_generator],
						));
						break;
					}
					out_generators.push(Generator::Iterator {
						declarations: indices,
						collection: folded,
						where_clause: where_clause
							.as_ref()
							.map(|w| self.fold_expression(db, model, w)),
					});
				}
			}
		}

		if out_generators.is_empty() {
			assert!(out_item.is_none());
			return inner.unwrap();
		}

		let mut definedness = out_item
			.iter()
			.map(|idx| {
				let o = self.totalised_model[*idx].origin();
				self.tuple_access(
					db,
					o,
					Expression::new(
						db,
						&self.totalised_model,
						o,
						ResolvedIdentifier::Declaration(*idx),
					),
					1,
				)
			})
			.collect::<Vec<_>>();

		let (template, template_partial, needs_flatten) = match inner {
			Some(v) => (v, true, true),
			None => {
				let folded = self.fold_expression(db, model, &c.template);
				let wrapped = self.wrap_root_ctx_bool(db, &c.template, folded);
				let partial = !self.is_total(db, model, &c.template, &wrapped);
				(wrapped, partial, false)
			}
		};

		if template_partial {
			let t = if var_where_clauses.is_empty() {
				template
			} else {
				let o = template.origin();
				let decl = Declaration::from_expression(db, false, template);
				let idx = self.totalised_model.add_declaration(Item::new(decl, o));
				let ident = Expression::new(
					db,
					&self.totalised_model,
					o,
					ResolvedIdentifier::Declaration(idx),
				);

				var_where_clauses.push(self.tuple_access(db, o, ident.clone(), 1));

				self.total_let(
					db,
					o,
					Let {
						items: vec![LetItem::Declaration(idx)],
						in_expression: Box::new(Expression::new(
							db,
							&self.totalised_model,
							o,
							TupleLiteral(vec![
								self.exists_call(
									db,
									Expression::new(
										db,
										&self.totalised_model,
										o,
										ArrayLiteral(var_where_clauses),
									),
								),
								self.tuple_access(db, o, ident, 2),
							]),
						)),
					},
				)
			};

			let comprehension = Expression::new(
				db,
				&self.totalised_model,
				origin,
				ArrayComprehension {
					generators: out_generators,
					indices: None,
					template: Box::new(t),
				},
			);
			let elem_ty = comprehension.ty().elem_ty(db).unwrap();
			let mut items = out_item
				.iter()
				.map(|idx| LetItem::Declaration(*idx))
				.collect::<Vec<_>>();
			let o = c.template.origin();
			let decl = Declaration::from_expression(db, false, comprehension);
			let idx = self.totalised_model.add_declaration(Item::new(decl, o));
			items.push(LetItem::Declaration(idx));

			// Comprehension is defined if all elements are defined
			let def_iter_decl = Declaration::new(false, Domain::unbounded(db, o, elem_ty));
			let def_iter_idx = self
				.totalised_model
				.add_declaration(Item::new(def_iter_decl, o));
			let elements_defined = Expression::new(
				db,
				&self.totalised_model,
				o,
				ArrayComprehension {
					generators: vec![Generator::Iterator {
						declarations: vec![def_iter_idx],
						collection: Expression::new(
							db,
							&self.totalised_model,
							o,
							ResolvedIdentifier::Declaration(idx),
						),
						where_clause: None,
					}],
					indices: None,
					template: Box::new(self.tuple_access(
						db,
						o,
						Expression::new(
							db,
							&self.totalised_model,
							o,
							ResolvedIdentifier::Declaration(def_iter_idx),
						),
						1,
					)),
				},
			);
			definedness.push(self.forall_call(db, elements_defined));

			// Extract values of comprehension
			let val_iter_decl = Declaration::new(false, Domain::unbounded(db, o, elem_ty));
			let val_iter_idx = self
				.totalised_model
				.add_declaration(Item::new(val_iter_decl, o));

			let val_extract_decl = Declaration::from_expression(
				db,
				false,
				self.tuple_access(
					db,
					o,
					Expression::new(
						db,
						&self.totalised_model,
						o,
						ResolvedIdentifier::Declaration(val_iter_idx),
					),
					2,
				),
			);
			let val_extract_ty = val_extract_decl.ty();
			let val_extract_idx = self
				.totalised_model
				.add_declaration(Item::new(val_extract_decl, o));

			let mut ident = Expression::new(
				db,
				&self.totalised_model,
				o,
				ResolvedIdentifier::Declaration(val_extract_idx),
			);

			let mut generators = vec![
				Generator::Iterator {
					declarations: vec![val_iter_idx],
					collection: Expression::new(
						db,
						&self.totalised_model,
						o,
						ResolvedIdentifier::Declaration(idx),
					),
					where_clause: None,
				},
				Generator::Assignment {
					assignment: val_extract_idx,
					where_clause: None,
				},
			];

			if needs_flatten {
				// Was rewritten into nested comprehension, so now needs to be flattened out again
				let val_flat_decl = Declaration::new(
					false,
					Domain::unbounded(db, o, val_extract_ty.elem_ty(db).unwrap()),
				);
				let val_flat_idx = self
					.totalised_model
					.add_declaration(Item::new(val_flat_decl, o));
				generators.push(Generator::Iterator {
					declarations: vec![val_flat_idx],
					collection: ident,
					where_clause: None,
				});
				ident = Expression::new(
					db,
					&self.totalised_model,
					o,
					ResolvedIdentifier::Declaration(val_flat_idx),
				);
			}

			let element_values = Expression::new(
				db,
				&self.totalised_model,
				o,
				ArrayComprehension {
					generators,
					indices: None,
					template: Box::new(ident),
				},
			);

			return self.total_let(
				db,
				origin,
				Let {
					items,
					in_expression: Box::new(Expression::new(
						db,
						&self.totalised_model,
						origin,
						TupleLiteral(vec![
							self.forall_call(
								db,
								Expression::new(
									db,
									&self.totalised_model,
									origin,
									ArrayLiteral(definedness),
								),
							),
							element_values,
						]),
					)),
				},
			);
		}

		// Inner/template is total, so
		let comprehension = Expression::new(
			db,
			&self.totalised_model,
			origin,
			ArrayComprehension {
				generators: out_generators,
				indices: None,
				template: Box::new(template),
			},
		);
		if definedness.is_empty() {
			return comprehension;
		}

		let items = out_item
			.iter()
			.map(|idx| LetItem::Declaration(*idx))
			.collect::<Vec<_>>();

		self.total_let(
			db,
			origin,
			Let {
				items,
				in_expression: Box::new(Expression::new(
					db,
					&self.totalised_model,
					origin,
					TupleLiteral(vec![
						self.forall_call(
							db,
							Expression::new(
								db,
								&self.totalised_model,
								origin,
								ArrayLiteral(definedness),
							),
						),
						comprehension,
					]),
				)),
			},
		)
	}

	fn totalise_tuple_access(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		ta: &TupleAccess<'db>,
		origin: Origin<'db>,
	) -> Expression<'db, Dst> {
		let tuple = self.fold_expression(db, model, &ta.tuple);
		if !self.is_total(db, model, &ta.tuple, &tuple) {
			let decl = Declaration::from_expression(db, false, tuple);
			let idx = self
				.totalised_model
				.add_declaration(Item::new(decl, origin));
			let ident = Expression::new(
				db,
				&self.totalised_model,
				origin,
				ResolvedIdentifier::Declaration(idx),
			);
			let definedness = self.tuple_access(db, origin, ident.clone(), 1);
			let value = self.tuple_access(
				db,
				origin,
				self.tuple_access(db, origin, ident, 2),
				ta.field.0,
			);

			if self.is_boolean_ty(value.ty()) {
				// Capture partiality in boolean
				return self.total_let(
					db,
					origin,
					Let {
						items: vec![LetItem::Declaration(idx)],
						in_expression: Box::new(Expression::new(
							db,
							&self.totalised_model,
							origin,
							LookupCall {
								function: self.ids.functions.and.into(),
								arguments: vec![definedness, value],
							},
						)),
					},
				);
			}

			return self.total_let(
				db,
				origin,
				Let {
					items: vec![LetItem::Declaration(idx)],
					in_expression: Box::new(Expression::new(
						db,
						&self.totalised_model,
						origin,
						TupleLiteral(vec![definedness, value]),
					)),
				},
			);
		}
		self.tuple_access(db, origin, tuple, ta.field.0)
	}

	fn totalise_if_then_else(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		ite: &IfThenElse<'db>,
		origin: Origin<'db>,
		expression: &Expression<'db>,
	) -> Expression<'db, Dst> {
		let is_root = self.get_mode(expression).is_root();
		let mut is_partial = false;

		let folded_else = self.fold_expression(db, model, &ite.else_result);
		let mut er = self.wrap_root_ctx_bool(db, &ite.else_result, folded_else);
		let mut else_total = self.is_total(db, model, &ite.else_result, &er);

		let mut bs = Vec::with_capacity(ite.branches.len() + 1);
		for b in ite.branches.iter() {
			let condition = self.fold_expression(db, model, &b.condition);
			let folded = self.fold_expression(db, model, &b.result);
			let result = self.wrap_root_ctx_bool(db, &b.result, folded);
			let total = self.is_total(db, model, &b.result, &result);
			if self.is_true(&condition) {
				if bs.is_empty() {
					return result;
				}
				er = result;
				else_total = total;
				break;
			}
			if !total {
				is_partial = true;
			}
			bs.push((condition, total, result));
		}
		if !else_total {
			is_partial = true;
		}

		// Unify types of partial/non-partial branches
		let (branches, else_result) = if is_partial {
			(
				bs.into_iter()
					.map(|(condition, total, r)| {
						let result = if total {
							Expression::new(
								db,
								&self.totalised_model,
								r.origin(),
								TupleLiteral(vec![
									Expression::new(
										db,
										&self.totalised_model,
										r.origin(),
										BooleanLiteral(true),
									),
									r,
								]),
							)
						} else {
							r
						};
						Branch { condition, result }
					})
					.collect::<Vec<_>>(),
				if else_total {
					Expression::new(
						db,
						&self.totalised_model,
						er.origin(),
						TupleLiteral(vec![
							Expression::new(
								db,
								&self.totalised_model,
								er.origin(),
								BooleanLiteral(true),
							),
							er,
						]),
					)
				} else {
					er
				},
			)
		} else {
			(
				bs.into_iter()
					.map(|(condition, _, result)| Branch { condition, result })
					.collect::<Vec<_>>(),
				er,
			)
		};

		// Partition into var and par groups
		let mut groups = Vec::new();
		for branch in branches {
			let var_condition = branch.condition.ty() == self.tys.var_bool;
			if groups
				.last()
				.is_none_or(|(was_var, _)| *was_var != var_condition)
			{
				groups.push((var_condition, vec![branch]));
			} else {
				groups.last_mut().unwrap().1.push(branch);
			}
		}

		let mut result = else_result;
		for (var_condition, branches) in groups.into_iter().rev() {
			if var_condition {
				let ty = branches.first().unwrap().result.ty();
				let mut items = Vec::new();
				let mut conditions = Vec::with_capacity(branches.len());
				let mut result_idents = Vec::with_capacity(branches.len());
				for branch in branches {
					let o = branch.result.origin();
					let decl = Declaration::from_expression(db, false, branch.result);
					let idx = self.totalised_model.add_declaration(Item::new(decl, o));
					items.push(LetItem::Declaration(idx));
					conditions.push(branch.condition);
					result_idents.push(Expression::new(
						db,
						&self.totalised_model,
						o,
						ResolvedIdentifier::Declaration(idx),
					));
				}
				let else_o = result.origin();
				let else_decl = Declaration::from_expression(db, false, result);
				let else_idx = self
					.totalised_model
					.add_declaration(Item::new(else_decl, else_o));
				items.push(LetItem::Declaration(else_idx));
				conditions.push(Expression::new(
					db,
					&self.totalised_model,
					else_o,
					BooleanLiteral(true),
				));
				result_idents.push(Expression::new(
					db,
					&self.totalised_model,
					else_o,
					ResolvedIdentifier::Declaration(else_idx),
				));

				let conditions_decl = Declaration::from_expression(
					db,
					false,
					Expression::new(db, &self.totalised_model, origin, ArrayLiteral(conditions)),
				);
				let conditions_idx = self
					.totalised_model
					.add_declaration(Item::new(conditions_decl, origin));
				let conditions_e = Expression::new(
					db,
					&self.totalised_model,
					origin,
					ResolvedIdentifier::Declaration(conditions_idx),
				);
				items.push(LetItem::Declaration(conditions_idx));

				let mut in_expression =
					self.decompose_tuple_ite(db, model, ty, is_root, conditions_e, result_idents);
				if is_root && self.is_boolean_ty(ty) {
					let constraint = Constraint::new(false, in_expression);
					let c = self
						.totalised_model
						.add_constraint(Item::new(constraint, origin));
					items.push(LetItem::Constraint(c));
					in_expression = Expression::new(
						db,
						&self.totalised_model,
						origin,
						LookupIdentifier(self.ids.names.root),
					);
				}
				result = self.total_let(
					db,
					origin,
					Let {
						items,
						in_expression: Box::new(in_expression),
					},
				);
			} else {
				result = Expression::new(
					db,
					&self.totalised_model,
					origin,
					IfThenElse {
						branches,
						else_result: Box::new(result),
					},
				)
			}
		}

		if !self.is_total(db, model, expression, &result) && is_root {
			let decl = Declaration::from_expression(db, false, result);
			let decl_idx = self
				.totalised_model
				.add_declaration(Item::new(decl, origin));
			let ident = Expression::new(
				db,
				&self.totalised_model,
				origin,
				ResolvedIdentifier::Declaration(decl_idx),
			);

			let constraint =
				Constraint::new(false, self.tuple_access(db, origin, ident.clone(), 1));
			let constraint_idx = self
				.totalised_model
				.add_constraint(Item::new(constraint, origin));

			result = self.total_let(
				db,
				origin,
				Let {
					items: vec![
						LetItem::Declaration(decl_idx),
						LetItem::Constraint(constraint_idx),
					],
					in_expression: Box::new(self.tuple_access(db, origin, ident, 2)),
				},
			)
		}

		result
	}

	fn decompose_tuple_ite(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		ty: Ty<'db>,
		is_root: bool,
		conditions: Expression<'db, Dst>,
		results: Vec<Expression<'db, Dst>>,
	) -> Expression<'db, Dst> {
		let origin = conditions.origin();
		if ty.is_tuple(db) {
			let fields = ty
				.fields(db)
				.unwrap()
				.into_iter()
				.enumerate()
				.map(|(i, inner_ty)| {
					let inner_results = results
						.iter()
						.map(|r| self.tuple_access(db, origin, r.clone(), i as i64 + 1))
						.collect();
					maybe_grow_stack(|| {
						self.decompose_tuple_ite(
							db,
							model,
							inner_ty,
							is_root && !self.is_boolean_ty(inner_ty),
							conditions.clone(),
							inner_results,
						)
					})
				})
				.collect();
			return Expression::new(db, &self.totalised_model, origin, TupleLiteral(fields));
		}

		self.ite_call(
			db,
			model,
			origin,
			is_root,
			conditions,
			Expression::new(db, &self.totalised_model, origin, ArrayLiteral(results)),
		)
	}

	fn totalise_let(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		l: &Let<'db>,
		e: &Expression<'db>,
	) -> Expression<'db, Dst> {
		let mut definedness = Vec::new();
		let mut items = Vec::new();
		for i in l.items.iter() {
			match i {
				LetItem::Constraint(c) => {
					if self.get_mode(model[*c].expression()).is_root() {
						self.add_constraint(db, model, *c);
						items.push(LetItem::Constraint(self.fold_constraint_id(db, model, *c)));
					} else {
						// Turn into a declaration, and add to definedness
						let expression = self.fold_expression(db, model, model[*c].expression());
						let mut declaration = Declaration::from_expression(db, false, expression);
						declaration.annotations_mut().extend(
							model[*c]
								.annotations()
								.iter()
								.map(|ann| self.fold_expression(db, model, ann)),
						);
						let idx = self
							.totalised_model
							.add_declaration(Item::new(declaration, model[*c].origin()));
						let ident = Expression::new(
							db,
							&self.totalised_model,
							model[*c].origin(),
							ResolvedIdentifier::Declaration(idx),
						);
						definedness.push(ident);
						items.push(LetItem::Declaration(idx));
					}
				}
				LetItem::Declaration(d) => {
					let decl = &model[*d];
					let mut declaration = Declaration::new(
						false,
						self.totalise_domain(
							db,
							model,
							decl.domain(),
							&mut items,
							&mut definedness,
						),
					);
					if let Some(name) = decl.name() {
						declaration.set_name(name);
					}
					declaration.annotations_mut().extend(
						decl.annotations()
							.iter()
							.map(|ann| self.fold_expression(db, model, ann)),
					);
					let mut partial = false;
					if let Some(definition) = decl.definition() {
						let def = self.fold_expression(db, model, definition);
						if !self.is_total(db, model, definition, &def) {
							declaration.set_domain(Domain::unbounded(
								db,
								decl.domain().origin(),
								def.ty(),
							));
							partial = true;
						}
						declaration.set_definition(def);
						declaration.validate(db);
					} else if self.get_mode(e) != Mode::NonRoot {
						// Mark as promise_total
						let origin = e.origin();
						let identifier = model
							.lookup_identifier(db, self.ids.annotations.promise_total)
							.unwrap();
						let ann = self.fold_identifier(db, model, &identifier);
						declaration.annotations_mut().push(Expression::new(
							db,
							&self.totalised_model,
							origin,
							ann,
						));
					}

					let idx = self
						.totalised_model
						.add_declaration(Item::new(declaration, model[*d].origin()));
					items.push(LetItem::Declaration(idx));
					let ident = Expression::new(
						db,
						&self.totalised_model,
						model[*d].origin(),
						ResolvedIdentifier::Declaration(idx),
					);
					if partial {
						definedness.push(self.tuple_access(
							db,
							model[*d].origin(),
							ident.clone(),
							1,
						));
						// Ensure references to the variable now get the just the value
						let value_idx = self.totalised_model.add_declaration(Item::new(
							Declaration::from_expression(
								db,
								false,
								self.tuple_access(db, model[*d].origin(), ident, 2),
							),
							model[*d].origin(),
						));
						self.replacement_map().insert_declaration(*d, value_idx);
						items.push(LetItem::Declaration(value_idx));
					} else {
						self.replacement_map().insert_declaration(*d, idx);
					}
				}
			}
		}
		let mut in_expression = self.fold_expression(db, model, &l.in_expression);
		// Parameter constraints guard unchecked operations in the result (for
		// example, an array access). A false guard makes the value irrelevant,
		// but evaluating it eagerly can still panic before partiality is captured.
		let par_definedness = definedness
			.iter()
			.filter(|d| d.ty() == self.tys.par_bool)
			.cloned()
			.collect::<Vec<_>>();
		if !par_definedness.is_empty() {
			let condition = self.forall_call(
				db,
				Expression::new(
					db,
					&self.totalised_model,
					e.origin(),
					ArrayLiteral(par_definedness),
				),
			);
			let dummy = DummyValue(in_expression.ty()).build_no_bottom(
				db,
				&mut self.totalised_model,
				e.origin(),
			);
			in_expression = Expression::new(
				db,
				&self.totalised_model,
				e.origin(),
				IfThenElse {
					branches: vec![Branch {
						condition,
						result: in_expression,
					}],
					else_result: Box::new(dummy),
				},
			);
		}
		if !self.is_total(db, model, &l.in_expression, &in_expression) {
			// In expression is partial
			let o = in_expression.origin();
			let declaration = Declaration::from_expression(db, false, in_expression);
			let idx = self
				.totalised_model
				.add_declaration(Item::new(declaration, o));
			let ident = Expression::new(
				db,
				&self.totalised_model,
				o,
				ResolvedIdentifier::Declaration(idx),
			);
			definedness.push(self.tuple_access(db, o, ident.clone(), 1));
			items.push(LetItem::Declaration(idx));
			in_expression = self.tuple_access(db, o, ident, 2);
		}

		if self.is_boolean_ty(in_expression.ty()) {
			// Capture partiality here
			if !self.is_true(&in_expression) {
				definedness.push(in_expression);
			}
			let parts = std::mem::take(&mut definedness);
			if self.get_mode(&l.in_expression).is_root() {
				for def in parts {
					let o = def.origin();
					let constraint = Constraint::new(false, def);
					let constraint_idx = self
						.totalised_model
						.add_constraint(Item::new(constraint, o));
					items.push(LetItem::Constraint(constraint_idx));
				}
				in_expression = Expression::new(
					db,
					&self.totalised_model,
					l.in_expression.origin(),
					LookupIdentifier(self.ids.names.root),
				);
			} else {
				if parts.len() > 1 {
					in_expression = self.forall_call(
						db,
						Expression::new(
							db,
							&self.totalised_model,
							l.in_expression.origin(),
							ArrayLiteral(parts),
						),
					);
				} else {
					in_expression = parts.into_iter().next().unwrap_or_else(|| {
						Expression::new(
							db,
							&self.totalised_model,
							l.in_expression.origin(),
							BooleanLiteral(true),
						)
					});
				}
			}
		}

		let folded = Let {
			items,
			in_expression: Box::new(if definedness.is_empty() {
				in_expression
			} else {
				let def = self.forall_call(
					db,
					Expression::new(
						db,
						&self.totalised_model,
						l.in_expression.origin(),
						ArrayLiteral(definedness),
					),
				);
				Expression::new(
					db,
					&self.totalised_model,
					l.in_expression.origin(),
					TupleLiteral(vec![def, in_expression]),
				)
			}),
		};

		self.total_let(db, e.origin(), folded)
	}

	/// Preserve par results when binding totalised var values in a let.
	fn total_let(
		&mut self,
		db: &'db dyn Db,
		origin: Origin<'db>,
		l: Let<'db, Dst>,
	) -> Expression<'db, Dst> {
		if !l.in_expression.ty().contains_var(db)
			&& l.in_expression.ty() != self.tys.ann
			&& !l.items.iter().any(|item| {
				matches!(item,
				LetItem::Constraint(c) if self.totalised_model[*c].expression().ty() == self.tys.var_bool)
			}) {
			for item in &l.items {
				if let LetItem::Declaration(d) = item
					&& self.totalised_model[*d].definition().is_some()
					&& self.totalised_model[*d].ty().contains_var(db)
					&& !self.totalised_model[*d]
						.annotations()
						.has(&self.totalised_model, self.ids.annotations.promise_total)
				{
					let ann = Expression::new(
						db,
						&self.totalised_model,
						self.totalised_model[*d].origin(),
						LookupIdentifier(self.ids.annotations.promise_total),
					);
					self.totalised_model[*d].annotations_mut().push(ann);
				}
			}
		}
		Expression::new(db, &self.totalised_model, origin, l)
	}

	fn bind_expression(
		&mut self,
		db: &'db dyn Db,
		origin: Origin<'db>,
		expression: Expression<'db, Dst>,
	) -> BoundExpression<'db, Dst> {
		let decl = Declaration::from_expression(db, false, expression);
		let idx = self
			.totalised_model
			.add_declaration(Item::new(decl, origin));
		let ident = Expression::new(
			db,
			&self.totalised_model,
			origin,
			ResolvedIdentifier::Declaration(idx),
		);
		BoundExpression {
			declaration: idx,
			ident,
		}
	}

	fn tuple_access(
		&self,
		db: &'db dyn Db,
		origin: Origin<'db>,
		tuple: Expression<'db, Dst>,
		field: i64,
	) -> Expression<'db, Dst> {
		Expression::new(
			db,
			&self.totalised_model,
			origin,
			TupleAccess {
				tuple: Box::new(tuple),
				field: IntegerLiteral(field),
			},
		)
	}

	fn bind_partial_expression(
		&mut self,
		db: &'db dyn Db,
		origin: Origin<'db>,
		expression: Expression<'db, Dst>,
	) -> BoundPartialExpression<'db, Dst> {
		let bound = self.bind_expression(db, origin, expression);
		let definedness = self.tuple_access(db, origin, bound.ident.clone(), 1);
		let value = self.tuple_access(db, origin, bound.ident.clone(), 2);
		BoundPartialExpression {
			declaration: bound.declaration,
			ident: bound.ident,
			definedness,
			value,
		}
	}

	fn totalise_default(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		origin: Origin<'db>,
		e: &Expression<'db>,
		val: &Expression<'db>,
		def: &Expression<'db>,
	) -> Expression<'db, Dst> {
		let totalised_val = self.fold_expression(db, model, val);
		let val_is_total = self.is_total(db, model, val, &totalised_val);
		if val_is_total {
			// LHS is already total, so just return it
			return totalised_val;
		}

		let BoundPartialExpression {
			declaration: val_idx,
			ident: val_ident,
			definedness: val_defined,
			value: val_value,
		} = self.bind_partial_expression(db, val.origin(), totalised_val);

		let totalised_def = self.fold_expression(db, model, def);
		let def_is_total = self.is_total(db, model, def, &totalised_def);
		let BoundExpression {
			declaration: def_idx,
			ident: def_ident,
		} = self.bind_expression(db, def.origin(), totalised_def);

		let mut items = vec![LetItem::Declaration(val_idx), LetItem::Declaration(def_idx)];
		let val_is_par = val_defined.ty() == self.tys.par_bool;
		assert!(val_is_par || val_defined.ty() == self.tys.var_bool);

		let in_expression = match (def_is_total, val_is_par) {
			// RHS is total and condition is par bool
			(true, true) => Expression::new(
				db,
				&self.totalised_model,
				origin,
				IfThenElse {
					branches: vec![Branch {
						condition: val_defined,
						result: val_value,
					}],
					else_result: Box::new(def_ident),
				},
			),
			// RHS is total and condition is var bool
			(true, false) => self.decompose_tuple_ite(
				db,
				model,
				e.ty(),
				self.get_mode(e).is_root(),
				Expression::new(
					db,
					&self.totalised_model,
					origin,
					ArrayLiteral(vec![
						val_defined,
						Expression::new(db, &self.totalised_model, origin, BooleanLiteral(true)),
					]),
				),
				vec![val_value, def_ident],
			),
			// RHS is partial and condition is par bool
			(false, true) => {
				let ite = Expression::new(
					db,
					&self.totalised_model,
					origin,
					IfThenElse {
						branches: vec![Branch {
							condition: val_defined,
							result: val_ident,
						}],
						else_result: Box::new(def_ident),
					},
				);

				if self.get_mode(e).is_root() {
					let BoundPartialExpression {
						declaration: result_declaration,
						definedness: result_definedness,
						value: result_value,
						..
					} = self.bind_partial_expression(db, origin, ite);
					items.push(LetItem::Declaration(result_declaration));
					let constraint = Constraint::new(false, result_definedness);
					let constraint_idx = self
						.totalised_model
						.add_constraint(Item::new(constraint, origin));
					items.push(LetItem::Constraint(constraint_idx));
					result_value
				} else {
					ite
				}
			}
			// RHS is partial and condition is var bool
			(false, false) => {
				let ite = self.decompose_tuple_ite(
					db,
					model,
					def_ident.ty(),
					self.get_mode(e).is_root(),
					Expression::new(
						db,
						&self.totalised_model,
						origin,
						ArrayLiteral(vec![
							val_defined.clone(),
							Expression::new(
								db,
								&self.totalised_model,
								origin,
								BooleanLiteral(true),
							),
						]),
					),
					vec![val_ident, def_ident],
				);

				if self.get_mode(e).is_root() {
					let BoundPartialExpression {
						declaration: result_declaration,
						definedness: result_definedness,
						value: result_value,
						..
					} = self.bind_partial_expression(db, origin, ite);
					items.push(LetItem::Declaration(result_declaration));
					let constraint = Constraint::new(false, result_definedness);
					let constraint_idx = self
						.totalised_model
						.add_constraint(Item::new(constraint, origin));
					items.push(LetItem::Constraint(constraint_idx));
					result_value
				} else {
					ite
				}
			}
		};

		self.total_let(
			db,
			origin,
			Let {
				items,
				in_expression: Box::new(in_expression),
			},
		)
	}

	fn totalise_call(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		c: &Call<'db>,
		origin: Origin<'db>,
		e: &Expression<'db>,
	) -> Expression<'db, Dst> {
		if c.matches_builtin(model, self.ids.functions.mzn_default_partial)
			&& c.arguments.len() == 2
		{
			return self.totalise_default(db, model, origin, e, &c.arguments[0], &c.arguments[1]);
		}

		if c.matches_builtin(model, self.ids.functions.mzn_in_root_context) {
			return Expression::new(
				db,
				&self.totalised_model,
				origin,
				BooleanLiteral(self.in_root_fns),
			);
		}

		let mut definedness = Vec::new();
		let mut items = Vec::new();
		let mut is_partial = false;
		let function = match &c.function {
			Callable::Annotation(a) => Callable::Annotation(self.fold_annotation_id(db, model, *a)),
			Callable::Function(f) => Callable::Function((|| {
				if self.get_mode(e).is_root()
					&& let Some(idx) = self.root_fn_map.get(f)
				{
					return *idx;
				}
				fold_function_id(self, db, model, *f)
			})()),
			e => unreachable!("Unexpected {:?}", e),
		};
		let totalised_function = match (&c.function, function) {
			(Callable::Expression(f1), Callable::Expression(f2)) if f1.ty() != f2.ty() => {
				let BoundExpression {
					declaration: idx,
					ident,
				} = self.bind_expression(db, f1.origin(), *f2);
				items.push(LetItem::Declaration(idx));
				definedness.push(self.tuple_access(db, f1.origin(), ident.clone(), 1));
				Callable::Expression(Box::new(self.tuple_access(db, f1.origin(), ident, 2)))
			}
			(_, f) => f,
		};

		if c.matches_builtin(model, self.ids.functions.fix)
			|| c.matches_builtin(model, self.ids.functions.is_fixed)
		{
			// Pass the totalised argument to the call
			assert!(c.arguments.len() == 1);
			let call = Call {
				function: totalised_function,
				arguments: vec![self.fold_expression(db, model, &c.arguments[0])],
			};
			return Expression::new(db, &self.totalised_model, origin, call);
		}

		let arguments = c
			.arguments
			.iter()
			.map(|arg| {
				let v = self.fold_expression(db, model, arg);
				let total = self.is_total(db, model, arg, &v);
				if !total {
					is_partial = true;
				}
				(total, v)
			})
			.collect::<Vec<_>>();
		let discard_arg_totality = is_partial
			&& !e.ty().contains_var(db)
			&& c.arguments.iter().any(|arg| arg.ty().contains_var(db));
		if discard_arg_totality {
			log::debug!(
				"Argument totality ignored for call at {}",
				origin.pretty_print(db)
			);
		}
		let totalised_args = if is_partial {
			let mut totalised = Vec::with_capacity(arguments.len());
			for (b, arg) in arguments {
				if b {
					totalised.push(arg);
				} else if discard_arg_totality {
					totalised.push(Expression::new(
						db,
						&self.totalised_model,
						arg.origin(),
						TupleAccess {
							tuple: Box::new(arg),
							field: IntegerLiteral(2),
						},
					));
				} else {
					let o = arg.origin();
					let BoundPartialExpression {
						declaration: idx,
						definedness: arg_defined,
						value: arg_value,
						..
					} = self.bind_partial_expression(db, o, arg);
					items.push(LetItem::Declaration(idx));
					definedness.push(arg_defined);
					totalised.push(arg_value);
				}
			}
			totalised
		} else {
			arguments.into_iter().map(|(_, v)| v).collect()
		};

		let mut val = Expression::new(
			db,
			&self.totalised_model,
			origin,
			Call {
				function: totalised_function,
				arguments: totalised_args,
			},
		);

		if definedness.is_empty() {
			return val;
		}

		if self.is_boolean_ty(val.ty()) {
			// Capture partiality in boolean
			definedness.push(val);
			return self.total_let(
				db,
				origin,
				Let {
					items,
					in_expression: Box::new(self.forall_call(
						db,
						Expression::new(
							db,
							&self.totalised_model,
							origin,
							ArrayLiteral(definedness),
						),
					)),
				},
			);
		}

		if !self.is_total(db, model, e, &val) {
			// Call returns partial value
			let o = e.origin();
			let BoundPartialExpression {
				declaration: idx,
				definedness: call_defined,
				value: call_value,
				..
			} = self.bind_partial_expression(db, o, val);
			items.push(LetItem::Declaration(idx));
			definedness.push(call_defined);
			val = call_value;
		}

		let def = self.forall_call(
			db,
			Expression::new(db, &self.totalised_model, origin, ArrayLiteral(definedness)),
		);
		self.total_let(
			db,
			origin,
			Let {
				items,
				in_expression: Box::new(Expression::new(
					db,
					&self.totalised_model,
					origin,
					TupleLiteral(vec![def, val]),
				)),
			},
		)
	}

	fn totalise_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		domain: &Domain<'db>,
		items: &mut Vec<LetItem<'db, Dst>>,
		definedness: &mut Vec<Expression<'db, Dst>>,
	) -> Domain<'db, Dst> {
		maybe_grow_stack(|| {
			let origin = domain.origin();
			match &**domain {
				DomainData::Array(dim, elem) => Domain::array(
					db,
					origin,
					OptType::NonOpt,
					self.totalise_domain(db, model, dim, items, definedness),
					self.totalise_domain(db, model, elem, items, definedness),
				),
				DomainData::Set(elem, card) => Domain::set_with_card(
					db,
					origin,
					domain.ty().inst(db).unwrap(),
					OptType::NonOpt,
					card.as_ref().map(|c| self.fold_expression(db, model, c)),
					self.totalise_domain(db, model, elem, items, definedness),
				),
				DomainData::Bounded(e) => {
					let folded = self.fold_expression(db, model, e);
					if self.is_total(db, model, e, &folded) {
						return Domain::bounded(
							db,
							origin,
							domain.ty().inst(db).unwrap(),
							OptType::NonOpt,
							folded,
						);
					}

					let o = e.origin();
					let BoundPartialExpression {
						declaration: idx,
						definedness: bounded_defined,
						value: bounded_value,
						..
					} = self.bind_partial_expression(db, o, folded);
					items.push(LetItem::Declaration(idx));
					definedness.push(bounded_defined);
					Domain::bounded(
						db,
						origin,
						domain.ty().inst(db).unwrap(),
						OptType::NonOpt,
						bounded_value,
					)
				}
				// All other domains are always unbounded (rewritten in an earlier pass)
				_ => self.fold_domain(db, model, domain),
			}
		})
	}

	fn is_total(
		&self,
		db: &'db dyn Db,
		model: &Model<'db>,
		original: &Expression,
		folded: &Expression<'db, Dst>,
	) -> bool {
		if folded.ty().is_subtype_of(db, original.ty()) {
			return true;
		}
		let field_tys = folded.ty().fields(db).unwrap_or_else(|| {
			panic!(
				"Expected totalised type to be tuple, but got {} (original {}) at {}\n\nBefore totalisation:\n{}\n\nAfter totalisation:\n{}",
				folded.ty().pretty_print(db),
				original.ty().pretty_print(db),
				original.origin().pretty_print(db),
				PrettyPrinter::new(db, model).pretty_print_expression(original),
				PrettyPrinter::new(db, &self.totalised_model).pretty_print_expression(folded)
			)
		});
		assert!(
			field_tys.len() == 2
				&& self.is_boolean_ty(field_tys[0])
				&& field_tys[1].is_subtype_of(db, original.ty()),
			"Totalisation of {} with type {} gave {} with incorrect type {}",
			PrettyPrinter::new(db, model).pretty_print_expression(original),
			original.ty().pretty_print(db),
			PrettyPrinter::new(db, &self.totalised_model).pretty_print_expression(folded),
			folded.ty().pretty_print(db)
		);
		false
	}

	/// Whether there is a user-defined root version already
	fn already_has_root_version(&self, f: FunctionId) -> bool {
		if let Some(r) = self.root_fn_map.get(&f) {
			return self.totalised_model[*r].body().is_some();
		}
		false
	}

	fn forall_call(&self, db: &'db dyn Db, arg: Expression<'db, Dst>) -> Expression<'db, Dst> {
		let origin = arg.origin();
		Expression::new(
			db,
			&self.totalised_model,
			origin,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![arg],
			},
		)
	}

	fn exists_call(&self, db: &'db dyn Db, arg: Expression<'db, Dst>) -> Expression<'db, Dst> {
		let origin = arg.origin();
		Expression::new(
			db,
			&self.totalised_model,
			origin,
			LookupCall {
				function: self.ids.functions.exists.into(),
				arguments: vec![arg],
			},
		)
	}

	fn ite_call(
		&self,
		db: &'db dyn Db,
		model: &Model<'db>,
		origin: Origin<'db>,
		is_root: bool,
		conditions: Expression<'db, Dst>,
		results: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		let f = model
			.lookup_function(
				db,
				self.ids.functions.if_then_else.into(),
				&[conditions.ty(), results.ty()],
			)
			.unwrap()
			.function;
		if is_root && let Some(idx) = self.root_fn_map.get(&f) {
			return Expression::new(
				db,
				&self.totalised_model,
				origin,
				Call {
					function: Callable::Function(*idx),
					arguments: vec![conditions, results],
				},
			);
		}
		Expression::new(
			db,
			&self.totalised_model,
			origin,
			LookupCall {
				function: self.ids.functions.if_then_else.into(),
				arguments: vec![conditions, results],
			},
		)
	}

	fn is_true(&self, e: &Expression<'db, Dst>) -> bool {
		let mut todo = vec![e];
		while let Some(e) = todo.pop() {
			match &**e {
				ExpressionData::BooleanLiteral(b) if b.0 => {
					return true;
				}
				ExpressionData::Call(Call {
					function: Callable::Function(f),
					arguments,
				}) if self.totalised_model[*f].name() == self.ids.functions.or
					&& arguments.len() == 2 =>
				{
					todo.push(&arguments[0]);
					todo.push(&arguments[1]);
				}
				_ => (),
			}
		}
		false
	}

	fn wrap_root_ctx_bool(
		&mut self,
		db: &'db dyn Db,
		orig: &Expression<'db>,
		folded: Expression<'db, Dst>,
	) -> Expression<'db, Dst> {
		if !self.get_mode(orig).is_root() || !self.is_boolean_ty(folded.ty()) {
			return folded;
		}
		let origin = orig.origin();
		if let ExpressionData::BooleanLiteral(BooleanLiteral(true)) = &*folded {
			return Expression::new(
				db,
				&self.totalised_model,
				origin,
				LookupIdentifier(self.ids.names.root),
			);
		}
		let constraint_idx = self
			.totalised_model
			.add_constraint(Item::new(Constraint::new(false, folded), origin));
		self.total_let(
			db,
			origin,
			Let {
				items: vec![LetItem::Constraint(constraint_idx)],
				in_expression: Box::new(Expression::new(
					db,
					&self.totalised_model,
					origin,
					LookupIdentifier(self.ids.names.root),
				)),
			},
		)
	}
}

/// Totalise a model
pub fn totalise<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Performing totalisation");
	let modes = ModeAnalysis::analyse(db, &model);
	let mut totaliser = Totaliser {
		ids: IdentifierRegistry::lookup(db),
		tys: TypeRegistry::lookup(db),
		replacement_map: ReplacementMap::default(),
		totalised_model: Model::with_capacities(&model.item_counts()),
		modes: &modes,
		totality: analyse_totality(db, &model, &modes),
		root_fn_map: FxHashMap::default(),
		root_fn_decl_map: FxHashMap::default(),
		missing_reif_generated: Set::default(),
		in_root_fns: false,
	};
	totaliser.add_model(db, &model);
	log::info!("Finished totalisation");
	Ok(totaliser.totalised_model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check_no_stdlib};

	#[test]
	fn test_totalise_par_result_with_var_binding() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				annotation promise_total;
				test forall(array [int] of bool);
				test '/\'(bool: x, bool: y);
				function var int: foo(var int: x) = let {
					constraint false;
				} in x;
				var int: x;
				bool: b = (foo(x), true).2;
			"#,
			expect![[r#"
    annotation promise_total;
    function bool: forall(array [int] of bool: _DECL_1);
    function bool: '/\'(bool: x, bool: y);
    function var int: foo_root(var int: x) = let {
      constraint false;
    } in x;
    function tuple(bool, var int): foo(var int: x) = let {
      bool: _DECL_11 = false;
    } in (forall([_DECL_11]), if forall([_DECL_11]) then x else 0 endif);
    var int: x;
    bool: b = let {
      tuple(bool, tuple(var int, bool)): _DECL_9 :: (promise_total) = let {
      tuple(bool, var int): _DECL_7 = foo(x);
      bool: _DECL_8 = true;
    } in (forall([(_DECL_7).1]), ((_DECL_7).2, _DECL_8));
    } in '/\'((_DECL_9).1, ((_DECL_9).2).2);
    solve satisfy;
"#]],
		);
	}

	#[test]
	fn test_totalise_par_let() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				test forall(array [int] of bool);
				bool: x = let {
                    constraint false;
                } in true;
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    bool: x = let {
      bool: _DECL_2 = false;
    } in forall([_DECL_2, if forall([_DECL_2]) then true else false endif]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_par_fn() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				test '='(bool: x, bool: y);
				test forall(array [int] of bool: x);
                function int: foo() = let {
                    constraint false;
                } in 1;
                bool: x = let {
                    int: a = foo();
                } in true;
                int: z = foo();
			"#,
			expect!([r#"
    function bool: '='(bool: x, bool: y);
    function bool: forall(array [int] of bool: x);
    function int: foo_root() = let {
      constraint false;
    } in 1;
    function tuple(bool, int): foo() = let {
      bool: _DECL_8 = false;
    } in (forall([_DECL_8]), if forall([_DECL_8]) then 1 else 0 endif);
    bool: x = let {
      tuple(bool, int): a = foo();
      int: _DECL_5 = (a).2;
    } in forall([(a).1, if forall([(a).1]) then true else false endif]);
    int: z = foo_root();
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_ite() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				function bool: '='(bool: x, bool: y);
				test forall(array [int] of bool: x);
                predicate if_then_else(array [int] of var bool: c, array [int] of var bool: x);
                predicate if_then_else_reif(array [int] of var bool: c, array [int] of var bool: x, var bool: r);
                function var int: if_then_else(array [int] of var bool: c, array [int] of var int: x);
                function var int: foo(var bool: b) =
                    if b then
                        let {
                            constraint false;
                        } in 1
                    else
                        2
                    endif;
                function int: bar(var bool: b) =
                    if true then
                        let {
                            constraint false;
                        } in 1
                    else
                        2
                    endif;
			"#,
			expect!([r#"
    function bool: '='(bool: x, bool: y);
    function bool: forall(array [int] of bool: x);
    predicate if_then_else(array [int] of var bool: c, array [int] of var bool: x);
    predicate if_then_else_reif(array [int] of var bool: c, array [int] of var bool: x, var bool: r);
    function var int: if_then_else(array [int] of var bool: c, array [int] of var int: x);
    function var int: foo_root(var bool: b) = let {
      tuple(var bool, var int): _DECL_24 = let {
      tuple(bool, int): _DECL_21 = let {
      bool: _DECL_20 = false;
    } in (forall([_DECL_20]), if forall([_DECL_20]) then 1 else 0 endif);
      tuple(bool, int): _DECL_22 = (true, 2);
      array [int] of var bool: _DECL_23 = [b, true];
    } in (if_then_else(_DECL_23, [(_DECL_21).1, (_DECL_22).1]), if_then_else(_DECL_23, [(_DECL_21).2, (_DECL_22).2]));
      constraint (_DECL_24).1;
    } in (_DECL_24).2;
    function tuple(var bool, var int): foo(var bool: b) = let {
      tuple(bool, int): _DECL_16 = let {
      bool: _DECL_15 = false;
    } in (forall([_DECL_15]), if forall([_DECL_15]) then 1 else 0 endif);
      tuple(bool, int): _DECL_17 = (true, 2);
      array [int] of var bool: _DECL_18 = [b, true];
    } in (if_then_else(_DECL_18, [(_DECL_16).1, (_DECL_17).1]), if_then_else(_DECL_18, [(_DECL_16).2, (_DECL_17).2]));
    function int: bar_root(var bool: b) = let {
      constraint false;
    } in 1;
    function tuple(bool, int): bar(var bool: b) = let {
      bool: _DECL_19 = false;
    } in (forall([_DECL_19]), if forall([_DECL_19]) then 1 else 0 endif);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_comp() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				function bool: '='(bool: x, bool: y);
                predicate forall(array [int] of var bool: x);
                predicate forall_reif(array [int] of var bool, var bool);
                test forall(array [int] of bool);
                function array [int] of int: foo() =
                    [1 | i in {1}, j in {2}, k in {3}];
                function array [int] of int: bar() =
                    [let { constraint false } in 1 | i in {1}, j in {2}, k in {3}];
                function set of int: iter() = let {
                    constraint false;
                } in {1};
                function array [int] of int: qux() =
                    [1 | i in {1}, j in iter(), k in {3}];
			"#,
			expect!([r#"
    function bool: '='(bool: x, bool: y);
    predicate forall(array [int] of var bool: x);
    predicate forall_reif(array [int] of var bool: _DECL_4, var bool: _DECL_5);
    function bool: forall(array [int] of bool: _DECL_6);
    function array [int] of int: foo() = [1 | i in {1}, j in {2}, k in {3}];
    function array [int] of int: bar_root() = [let {
      constraint false;
    } in 1 | i in {1}, j in {2}, k in {3}];
    function tuple(bool, array [int] of int): bar() = let {
      array [int] of tuple(bool, int): _DECL_14 = [let {
      bool: _DECL_13 = false;
    } in (forall([_DECL_13]), if forall([_DECL_13]) then 1 else 0 endif) | i in {1}, j in {2}, k in {3}];
    } in (forall([forall([(_DECL_15).1 | _DECL_15 in _DECL_14])]), [_DECL_17 | _DECL_16 in _DECL_14, _DECL_17 = (_DECL_16).2]);
    function set of int: iter_root() = let {
      constraint false;
    } in {1};
    function tuple(bool, set of int): iter() = let {
      bool: _DECL_18 = false;
    } in (forall([_DECL_18]), if forall([_DECL_18]) then {1} else let {
      set of int: _DECL_19 = {};
    } in _DECL_19 endif);
    function array [int] of int: qux_root() = [1 | i in {1}, j in iter_root(), k in {3}];
    function tuple(bool, array [int] of int): qux() = let {
      array [int] of tuple(bool, array [int] of int): _DECL_24 = [let {
      tuple(bool, set of int): _DECL_22 = iter();
    } in (forall([(_DECL_22).1]), [1 | j in (_DECL_22).2, k in {3}]) | i in {1}];
    } in (forall([forall([(_DECL_25).1 | _DECL_25 in _DECL_24])]), [_DECL_28 | _DECL_26 in _DECL_24, _DECL_27 = (_DECL_26).2, _DECL_28 in _DECL_27]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_var_comp() {
		check_no_stdlib(
			vec![
				Transform::DesugarComprehension,
				Transform::EraseOpt,
				Transform::Totalise,
			],
			r#"
				bool: mzn_root = true;
				annotation mzn_var_where_clause;
				function opt $T: val2opt($T: x);
				function bool: '='(bool: x, bool: y);
                predicate forall(array [int] of var bool: x);
                predicate forall_reif(array [int] of var bool: x, var bool: b);
                test forall(array [int] of bool: x);
                predicate exists(array [int] of var bool: x);
                predicate exists_reif(array [int] of var bool: x, var bool: b);
				predicate if_then_else(array [int] of var bool: c, array [int] of bool: r);
				predicate if_then_else_reif(array [int] of var bool: c, array [int] of bool: r, var bool: b);
				function var int: if_then_else(array [int] of var bool: c, array [int] of int: r);
                predicate bar(int: x) = false;
                function set of int: qux() = let { constraint false } in {3, 4};
                function array [int] of var opt int: foo() =
                    [1 | i in {1, 2} where bar(i), j in qux()];
			"#,
			expect!([r#"
    bool: mzn_root = true;
    annotation mzn_var_where_clause;
    function bool: '='(bool: x, bool: y);
    predicate forall(array [int] of var bool: x);
    predicate forall_reif(array [int] of var bool: x, var bool: b);
    function bool: forall(array [int] of bool: x);
    predicate exists(array [int] of var bool: x);
    predicate exists_reif(array [int] of var bool: x, var bool: b);
    predicate if_then_else(array [int] of var bool: c, array [int] of bool: r);
    predicate if_then_else_reif(array [int] of var bool: c, array [int] of bool: r, var bool: b);
    function var int: if_then_else(array [int] of var bool: c, array [int] of int: r);
    function var bool: bar_root(int: x) = let {
      constraint false;
    } in mzn_root;
    function var bool: bar(int: x) = false;
    function set of int: qux_root() = let {
      constraint false;
    } in {3, 4};
    function tuple(bool, set of int): qux() = let {
      bool: _DECL_20 = false;
    } in (forall([_DECL_20]), if forall([_DECL_20]) then {3, 4} else let {
      set of int: _DECL_21 = {};
    } in _DECL_21 endif);
    function array [int] of tuple(var bool, var int): foo_root() = [let {
      tuple(bool, int): _DECL_40 = (true, 1);
      tuple(bool, int): _DECL_41 = let {
      tuple(bool, int): _DECL_39 = (false, 0);
    } in _DECL_39;
      array [int] of var bool: _DECL_42 = [_DECL_37, true];
    } in (if_then_else(_DECL_42, [(_DECL_40).1, (_DECL_41).1]), if_then_else(_DECL_42, [(_DECL_40).2, (_DECL_41).2])) | i in {1, 2}, _DECL_37 = bar(i), j in qux_root()];
    function tuple(var bool, array [int] of tuple(var bool, var int)): foo() = let {
      array [int] of tuple(var bool, array [int] of tuple(var bool, var int)): _DECL_31 = [let {
      tuple(bool, array [int] of tuple(var bool, var int)): _DECL_30 = let {
      tuple(bool, set of int): _DECL_25 = qux();
    } in (forall([(_DECL_25).1]), [let {
      tuple(bool, int): _DECL_27 = (true, 1);
      tuple(bool, int): _DECL_28 = let {
      tuple(bool, int): _DECL_26 = (false, 0);
    } in _DECL_26;
      array [int] of var bool: _DECL_29 = [_DECL_23, true];
    } in (if_then_else(_DECL_29, [(_DECL_27).1, (_DECL_28).1]), if_then_else(_DECL_29, [(_DECL_27).2, (_DECL_28).2])) | j in (_DECL_25).2]);
    } in (exists([_DECL_23, (_DECL_30).1]), (_DECL_30).2) | i in {1, 2}, _DECL_23 = bar(i)];
    } in (forall([forall([(_DECL_32).1 | _DECL_32 in _DECL_31])]), [_DECL_35 | _DECL_33 in _DECL_31, _DECL_34 = (_DECL_33).2, _DECL_35 in _DECL_34]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_bool_fns() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				bool: mzn_root = true;
				function bool: '='(bool: x, bool: y);
    			function bool: forall(array [int] of bool: x);
				function int: foo(int: x) = let {
					constraint false;
				} in 1;
                test bar(int: x) = let {
					int: f = foo(x);
				} in false;
				constraint bar(1);
			"#,
			expect!([r#"
    bool: mzn_root = true;
    function bool: '='(bool: x, bool: y);
    function bool: forall(array [int] of bool: x);
    function int: foo_root(int: x) = let {
      constraint false;
    } in 1;
    function tuple(bool, int): foo(int: x) = let {
      bool: _DECL_9 = false;
    } in (forall([_DECL_9]), if forall([_DECL_9]) then 1 else 0 endif);
    function bool: bar_root(int: x) = let {
      constraint let {
      int: f = foo_root(x);
      constraint false;
    } in mzn_root;
    } in mzn_root;
    function bool: bar(int: x) = let {
      tuple(bool, int): f = foo(x);
      int: _DECL_11 = (f).2;
    } in forall([(f).1, if forall([(f).1]) then false else false endif]);
    constraint bar_root(1);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_promise_total_bool_fn() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				annotation promise_total;
				bool: mzn_root = true;

                test foo(int: x) :: promise_total = false;
                constraint foo(1);
            "#,
			expect!([r#"
    annotation promise_total;
    bool: mzn_root = true;
    function bool: foo_root(int: x) = let {
      constraint false;
    } in mzn_root;
    function bool: foo(int: x) :: (promise_total) = false;
    constraint foo_root(1);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_predicate_with_ite() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				bool: mzn_root = true;
				function bool: '='(bool: x, bool: y);
                predicate forall(array [int] of var bool: x);
                predicate forall_reif(array [int] of var bool: x, var bool: r);
                test forall(array [int] of bool: x);
				function var int: qux(var int: x) = let {
					constraint false;
				} in x;
				predicate bar(var int: x);
				predicate bar_reif(var int: x, var bool: b);
                predicate foo(var int: x) =
					if true then let {
						var int: x = qux(3);
					} in bar(x)
					else true endif;
				var int: v;
				constraint foo(v);
			"#,
			expect!([r#"
    bool: mzn_root = true;
    function bool: '='(bool: x, bool: y);
    predicate forall(array [int] of var bool: x);
    predicate forall_reif(array [int] of var bool: x, var bool: r);
    function bool: forall(array [int] of bool: x);
    function var int: qux_root(var int: x) = let {
      constraint false;
    } in x;
    function tuple(bool, var int): qux(var int: x) = let {
      bool: _DECL_16 = false;
    } in (forall([_DECL_16]), if forall([_DECL_16]) then x else 0 endif);
    predicate bar(var int: x);
    predicate bar_reif(var int: x, var bool: b);
    function var bool: foo_root(var int: x) = let {
      constraint let {
      constraint let {
      var int: x = qux_root(3);
      constraint bar(x);
    } in mzn_root;
    } in mzn_root;
    } in mzn_root;
    function var bool: foo(var int: x) = let {
      tuple(bool, var int): x = qux(3);
      var int: _DECL_18 = (x).2;
    } in forall([(x).1, if forall([(x).1]) then bar(_DECL_18) else false endif]);
    var int: v;
    constraint foo_root(v);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_abort() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				bool: mzn_root = true;
				test '='(bool: x, bool: y);
                test forall(array [int] of bool: x);
                test abort(string: msg);
				test bar(int: x);
				function int: foo(int: x) = let {
					constraint if bar(x) then abort("foo") endif;
				} in x;
				int: a = foo(2);
			"#,
			expect!([r#"
    bool: mzn_root = true;
    function bool: '='(bool: x, bool: y);
    function bool: forall(array [int] of bool: x);
    function bool: abort(string: msg);
    function bool: bar(int: x);
    function int: foo(int: x) = let {
      constraint if bar(x) then let {
      constraint abort("foo");
    } in mzn_root else mzn_root endif;
    } in x;
    int: a = foo(2);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_default() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
                test forall(array [int] of bool);
                function int: mzn_default_partial(int: x, int: def);
				function int: foo() = mzn_default_partial(1, 2);
				function int: bar(bool: b) = mzn_default_partial(let { constraint b } in 1, 2);
				function int: qux(bool: b, bool: c) = mzn_default_partial(let { constraint b } in 1, let { constraint c } in 2);
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    function int: foo() = 1;
    function int: bar(bool: b) = let {
      tuple(bool, int): _DECL_8 = let {
      bool: _DECL_7 = b;
    } in (forall([_DECL_7]), if forall([_DECL_7]) then 1 else 0 endif);
      int: _DECL_9 = 2;
    } in if (_DECL_8).1 then (_DECL_8).2 else _DECL_9 endif;
    function int: qux_root(bool: b, bool: c) = let {
      tuple(bool, int): _DECL_15 = let {
      bool: _DECL_14 = b;
    } in (forall([_DECL_14]), if forall([_DECL_14]) then 1 else 0 endif);
      tuple(bool, int): _DECL_17 = let {
      bool: _DECL_16 = c;
    } in (forall([_DECL_16]), if forall([_DECL_16]) then 2 else 0 endif);
      tuple(bool, int): _DECL_18 = if (_DECL_15).1 then _DECL_15 else _DECL_17 endif;
      constraint (_DECL_18).1;
    } in (_DECL_18).2;
    function tuple(bool, int): qux(bool: b, bool: c) = let {
      tuple(bool, int): _DECL_11 = let {
      bool: _DECL_10 = b;
    } in (forall([_DECL_10]), if forall([_DECL_10]) then 1 else 0 endif);
      tuple(bool, int): _DECL_13 = let {
      bool: _DECL_12 = c;
    } in (forall([_DECL_12]), if forall([_DECL_12]) then 2 else 0 endif);
    } in if (_DECL_11).1 then _DECL_11 else _DECL_13 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_mzn_in_root_context() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				annotation promise_total;
				bool: mzn_root = true;
                predicate forall(array [int] of var bool);
                predicate forall_reif(array [int] of var bool, var bool);

				test mzn_in_root_context();
				predicate bar(var int: x);
				predicate bar_reif(var int: x, var bool: b);
				function var bool: foo(var int: x) =
					if mzn_in_root_context() then
						let {
							constraint bar(x);
						} in true
					else
						foo_t(x)
					endif;
				function var bool: foo_t(var int: x) :: promise_total =
					let {
						var bool: b;
						constraint bar_reif(x, b);
					} in b;
			"#,
			expect!([r#"
    annotation promise_total;
    bool: mzn_root = true;
    predicate forall(array [int] of var bool: _DECL_2);
    predicate forall_reif(array [int] of var bool: _DECL_3, var bool: _DECL_4);
    predicate bar(var int: x);
    predicate bar_reif(var int: x, var bool: b);
    function var bool: foo_root(var int: x) = let {
      constraint let {
      constraint let {
      constraint bar(x);
    } in mzn_root;
    } in mzn_root;
    } in mzn_root;
    function var bool: foo(var int: x) = if false then let {
      var bool: _DECL_12 = bar(x);
    } in _DECL_12 else foo_t(x) endif;
    function var bool: foo_t_root(var int: x) = let {
      constraint let {
      var bool: b :: (promise_total);
      constraint bar_reif(x, b);
      constraint b;
    } in mzn_root;
    } in mzn_root;
    function var bool: foo_t(var int: x) :: (promise_total) = let {
      var bool: b :: (promise_total);
      constraint bar_reif(x, b);
    } in b;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_free_var() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				annotation promise_total;
				bool: mzn_root = true;
                array [int] of var int: x = [let { var {1, 2, 3}: x_i; } in x_i | i in {1, 2, 3}];
				var bool: b = let { var int: y; } in true;
				predicate foo() =
					let {
						array [int] of tuple(var bool): p = [let { var bool: b } in (b,) | i in {1, 2, 3}];
						array [int] of var bool: q = [p_i.1 | p_i in p];
					} in true;
			"#,
			expect!([r#"
    annotation promise_total;
    bool: mzn_root = true;
    array [int] of var int: x = [let {
      var {1, 2, 3}: x_i :: (promise_total);
    } in x_i | i in {1, 2, 3}];
    var bool: b = let {
      var int: y;
    } in true;
    function var bool: foo_root() = let {
      constraint let {
      array [int] of tuple(var bool): p :: (promise_total) = [let {
      var bool: b :: (promise_total);
    } in (b,) | i in {1, 2, 3}];
      array [int] of var bool: q :: (promise_total) = [(p_i).1 | p_i in p];
    } in mzn_root;
    } in mzn_root;
    function var bool: foo() = let {
      array [int] of tuple(var bool): p :: (promise_total) = [let {
      var bool: b;
    } in (b,) | i in {1, 2, 3}];
      array [int] of var bool: q :: (promise_total) = [(p_i).1 | p_i in p];
    } in true;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_root_bool_fn() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				bool: mzn_root = true;
				test abort(string: msg);
				predicate foo(var int: x);
				predicate bar(var int: x) = if b then foo(x) else true endif;
				var int: x;
				bool: b;
				constraint bar(x);
				constraint if b then foo(x) else true endif;
			"#,
			expect!([r#"
    bool: mzn_root = true;
    function bool: abort(string: msg);
    function var bool: foo_reif(var int: x, var bool: _DECL_4) = abort("foo_reif required, but not supported by solver.");
    predicate foo(var int: x);
    bool: b;
    function var bool: bar_root(var int: x) = let {
      constraint if b then let {
      constraint foo(x);
    } in mzn_root else mzn_root endif;
    } in mzn_root;
    function var bool: bar(var int: x) = if b then foo(x) else true endif;
    var int: x;
    constraint bar_root(x);
    constraint if b then let {
      constraint foo(x);
    } in mzn_root else mzn_root endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_if_then_else_root() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				annotation promise_total;
				bool: mzn_root = true;
				test abort(string: msg);
				predicate fzn_if_then_else_var_bool(array [int] of var bool: c, array [int] of var bool: x, var bool: y);
				predicate fzn_if_then_else_var_int(array [int] of var bool: c, array [int] of var int: x, var int: y);
				function var bool: if_then_else(array [int] of var bool: c, array [int] of var bool: x) :: promise_total =
					let {
						var bool: y;
						constraint fzn_if_then_else_var_bool(c, x, y);
					} in y;
				function var int: if_then_else(array [int] of var bool: c, array [int] of var int: x) :: promise_total =
					let {
						var int: y;
						constraint fzn_if_then_else_var_int(c, x, y);
					} in y;
				var bool: b;
				var bool: x;
				var bool: y;
				constraint if b then x else y endif;
				var int: p;
				var int: q;
				any: r = if b then p else q endif;
			"#,
			expect!([r#"
    annotation promise_total;
    bool: mzn_root = true;
    function bool: abort(string: msg);
    function var bool: fzn_if_then_else_var_bool_reif(array [int] of var bool: c, array [int] of var bool: x, var bool: y, var bool: _DECL_6) = abort("fzn_if_then_else_var_bool_reif required, but not supported by solver.");
    predicate fzn_if_then_else_var_bool(array [int] of var bool: c, array [int] of var bool: x, var bool: y);
    function var bool: fzn_if_then_else_var_int_reif(array [int] of var bool: c, array [int] of var int: x, var int: y, var bool: _DECL_13) = abort("fzn_if_then_else_var_int_reif required, but not supported by solver.");
    predicate fzn_if_then_else_var_int(array [int] of var bool: c, array [int] of var int: x, var int: y);
    function var bool: if_then_else_root(array [int] of var bool: c, array [int] of var bool: x) = let {
      constraint let {
      var bool: y :: (promise_total);
      constraint fzn_if_then_else_var_bool(c, x, y);
      constraint y;
    } in mzn_root;
    } in mzn_root;
    function var bool: if_then_else(array [int] of var bool: c, array [int] of var bool: x) :: (promise_total) = let {
      var bool: y :: (promise_total);
      constraint fzn_if_then_else_var_bool(c, x, y);
    } in y;
    function var int: if_then_else(array [int] of var bool: c, array [int] of var int: x) :: (promise_total) = let {
      var int: y :: (promise_total);
      constraint fzn_if_then_else_var_int(c, x, y);
    } in y;
    var bool: b;
    var bool: x;
    var bool: y;
    constraint let {
      var bool: _DECL_26 = x;
      var bool: _DECL_27 = y;
      array [int] of var bool: _DECL_28 = [b, true];
      constraint if_then_else_root(_DECL_28, [_DECL_26, _DECL_27]);
    } in mzn_root;
    var int: p;
    var int: q;
    var int: r = let {
      var int: _DECL_31 = p;
      var int: _DECL_32 = q;
      array [int] of var bool: _DECL_33 = [b, true];
    } in if_then_else(_DECL_33, [_DECL_31, _DECL_32]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_nested_par_let() {
		check_no_stdlib(
			Transform::Totalise,
			r#"
				test forall(array [int] of bool);
				int: x = let {
					bool: b = let {
						int: y = let {
							constraint false;
							constraint false;
						} in 1;
						constraint false;
					} in true;
				} in 5;
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    int: x = let {
      bool: b = let {
      tuple(bool, int): y = let {
      bool: _DECL_2 = false;
      bool: _DECL_3 = false;
    } in (forall([_DECL_2, _DECL_3]), if forall([_DECL_2, _DECL_3]) then 1 else 0 endif);
      int: _DECL_5 = (y).2;
      bool: _DECL_6 = false;
    } in forall([(y).1, _DECL_6, if forall([(y).1, _DECL_6]) then true else false endif]);
    } in 5;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_opt_bool() {
		check_no_stdlib(
			vec![
				Transform::TopDownType,
				Transform::EraseOpt,
				Transform::Totalise,
			],
			r#"
				function any $T: mzn_default_partial(any $T: x, any $T: y);
				test forall(array [int] of bool);
				function opt $T: val2opt($T: x);
				opt bool: b = let {
					constraint false;
				} in <>;
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    tuple(bool, bool): b = let {
      tuple(bool, tuple(bool, bool)): _DECL_4 = let {
      bool: _DECL_2 = false;
    } in (forall([_DECL_2]), if forall([_DECL_2]) then let {
      tuple(bool, bool): _DECL_3 = (false, false);
    } in _DECL_3 else (false, false) endif);
      tuple(bool, bool): _DECL_5 = (true, false);
    } in if (_DECL_4).1 then (_DECL_4).2 else _DECL_5 endif;
    solve satisfy;
"#]),
		)
	}
}
