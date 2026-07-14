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
	DeclarationId, Domain, DomainData, Expression, ExpressionBuilder, ExpressionData, Function,
	FunctionId, FunctionItem, FunctionName, Generator, IfThenElse, IntegerLiteral, Item, Let,
	LetItem, LookupCall, Marker, Model, ResolvedIdentifier, SetLiteral, TupleAccess, TupleLiteral,
	analyse::{Mode, ModeAnalysis, Totality, TotalityResult, analyse_totality},
	pretty_print::PrettyPrinter,
	source::Origin,
	traverse::{
		Folder, ReplacementMap, add_model, fold_declaration_id, fold_expression,
		fold_function_body, fold_function_id,
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
					function: self.ids.builtins.abort.into(),
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
		if model[f].name() == self.ids.builtins.mzn_default_partial {
			// `default` calls rewritten so function no longer needed
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

		if function_totality.needs_root {
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
		fold_function_body(self, db, model, f);
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
				ExpressionData::Let(l) => {
					let folded = self.fold_let(db, model, l);
					Expression::new(db, &self.totalised_model, expression.origin(), folded)
				}
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

	fn fold_let(&mut self, db: &'db dyn Db, model: &Model<'db>, l: &Let<'db>) -> Let<'db, Dst> {
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
					BooleanLiteral(true),
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

		Let {
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
		}
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
		function.annotations_mut().extend(
			ff.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
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
				let total = self.is_total(db, model, v, &folded);
				if !total {
					is_partial = true;
				}
				(total, folded)
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
			Expression::new(
				db,
				&self.totalised_model,
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
					let idx = self.totalised_model.add_declaration(Item::new(
						Declaration::from_expression(db, false, rhs),
						o,
					));
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
				let partial = !self.is_total(db, model, &c.template, &folded);
				(folded, partial, false)
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

				Expression::new(
					db,
					&self.totalised_model,
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

			return Expression::new(
				db,
				&self.totalised_model,
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

		Expression::new(
			db,
			&self.totalised_model,
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
				return Expression::new(
					db,
					&self.totalised_model,
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

			return Expression::new(
				db,
				&self.totalised_model,
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

		let mut er = self.fold_expression(db, model, &ite.else_result);
		let mut else_total = self.is_total(db, model, &ite.else_result, &er);

		let mut bs = Vec::with_capacity(ite.branches.len() + 1);
		for b in ite.branches.iter() {
			let condition = self.fold_expression(db, model, &b.condition);
			let result = self.fold_expression(db, model, &b.result);
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

				let in_expression =
					self.decompose_tuple_ite(db, ty, origin, conditions_e, result_idents);
				result = Expression::new(
					db,
					&self.totalised_model,
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

			result = Expression::new(
				db,
				&self.totalised_model,
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
		ty: Ty<'db>,
		origin: Origin<'db>,
		conditions: Expression<'db, Dst>,
		results: Vec<Expression<'db, Dst>>,
	) -> Expression<'db, Dst> {
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
							inner_ty,
							origin,
							conditions.clone(),
							inner_results,
						)
					})
				})
				.collect();
			return Expression::new(db, &self.totalised_model, origin, TupleLiteral(fields));
		}

		Expression::new(
			db,
			&self.totalised_model,
			origin,
			LookupCall {
				function: self.ids.functions.if_then_else.into(),
				arguments: vec![
					conditions,
					Expression::new(db, &self.totalised_model, origin, ArrayLiteral(results)),
				],
			},
		)
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
			// RHS is total, so result is always defined
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
			(true, false) => Expression::new(
				db,
				&self.totalised_model,
				origin,
				LookupCall {
					function: self.ids.functions.if_then_else.into(),
					arguments: vec![
						Expression::new(
							db,
							&self.totalised_model,
							origin,
							ArrayLiteral(vec![
								val_defined,
								Expression::new(
									db,
									&self.totalised_model,
									origin,
									BooleanLiteral(true),
								),
							]),
						),
						Expression::new(
							db,
							&self.totalised_model,
							origin,
							ArrayLiteral(vec![val_value, def_ident]),
						),
					],
				},
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
				let def_defined = self.tuple_access(db, val.origin(), def_ident.clone(), 1);
				let def_value = self.tuple_access(db, val.origin(), def_ident, 2);

				let ite = Expression::new(
					db,
					&self.totalised_model,
					origin,
					LookupCall {
						function: self.ids.functions.if_then_else.into(),
						arguments: vec![
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
							Expression::new(
								db,
								&self.totalised_model,
								origin,
								ArrayLiteral(vec![val_value, def_value]),
							),
						],
					},
				);

				let defined = self.exists_call(
					db,
					Expression::new(
						db,
						&self.totalised_model,
						origin,
						ArrayLiteral(vec![val_defined, def_defined]),
					),
				);

				if self.get_mode(e).is_root() {
					let constraint = Constraint::new(false, defined);
					let constraint_idx = self
						.totalised_model
						.add_constraint(Item::new(constraint, origin));
					items.push(LetItem::Constraint(constraint_idx));
					ite
				} else {
					Expression::new(
						db,
						&self.totalised_model,
						origin,
						TupleLiteral(vec![defined, ite]),
					)
				}
			}
		};

		Expression::new(
			db,
			&self.totalised_model,
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
		if c.matches_builtin(model, self.ids.builtins.mzn_default_partial) && c.arguments.len() == 2
		{
			return self.totalise_default(db, model, origin, e, &c.arguments[0], &c.arguments[1]);
		}

		if c.matches_builtin(model, self.ids.builtins.mzn_in_root_context) {
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

		if c.matches_builtin(model, self.ids.builtins.fix)
			|| c.matches_builtin(model, self.ids.builtins.is_fixed)
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
			return Expression::new(
				db,
				&self.totalised_model,
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
		Expression::new(
			db,
			&self.totalised_model,
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
				function: self.ids.builtins.forall.into(),
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
				function: self.ids.builtins.exists.into(),
				arguments: vec![arg],
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
				}) if self.totalised_model[*f].name() == self.ids.builtins.or
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

	use super::totalise;
	use crate::transform::{
		comprehension::desugar_comprehension, erase_opt::erase_opt, tests::check_no_stdlib,
		transformer,
	};

	#[test]
	fn test_totalise_par_let() {
		check_no_stdlib(
			totalise,
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
    } in _DECL_2;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_par_fn() {
		check_no_stdlib(
			totalise,
			r#"
				test forall(array [int] of bool);
                function int: foo() = let {
                    constraint false;
                } in 1;
                bool: x = let {
                    int: a = foo();
                } in true;
                int: z = foo();
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    function int: foo_root() = let {
      constraint false;
    } in 1;
    function tuple(bool, int): foo() = let {
      bool: _DECL_6 = false;
    } in (forall([_DECL_6]), 1);
    bool: x = let {
      tuple(bool, int): a = foo();
      int: _DECL_3 = (a).2;
    } in (a).1;
    int: z = foo_root();
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_ite() {
		check_no_stdlib(
			totalise,
			r#"
				test forall(array [int] of bool);
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
    function bool: forall(array [int] of bool: _DECL_1);
    predicate if_then_else(array [int] of var bool: c, array [int] of var bool: x);
    predicate if_then_else_reif(array [int] of var bool: c, array [int] of var bool: x, var bool: r);
    function var int: if_then_else(array [int] of var bool: c, array [int] of var int: x);
    function var int: foo_root(var bool: b) = let {
      tuple(var bool, var int): _DECL_22 = let {
      tuple(bool, int): _DECL_19 = let {
      bool: _DECL_18 = false;
    } in (forall([_DECL_18]), 1);
      tuple(bool, int): _DECL_20 = (true, 2);
      array [int] of var bool: _DECL_21 = [b, true];
    } in (if_then_else(_DECL_21, [(_DECL_19).1, (_DECL_20).1]), if_then_else(_DECL_21, [(_DECL_19).2, (_DECL_20).2]));
      constraint (_DECL_22).1;
    } in (_DECL_22).2;
    function tuple(var bool, var int): foo(var bool: b) = let {
      tuple(bool, int): _DECL_14 = let {
      bool: _DECL_13 = false;
    } in (forall([_DECL_13]), 1);
      tuple(bool, int): _DECL_15 = (true, 2);
      array [int] of var bool: _DECL_16 = [b, true];
    } in (if_then_else(_DECL_16, [(_DECL_14).1, (_DECL_15).1]), if_then_else(_DECL_16, [(_DECL_14).2, (_DECL_15).2]));
    function int: bar_root(var bool: b) = let {
      constraint false;
    } in 1;
    function tuple(bool, int): bar(var bool: b) = let {
      bool: _DECL_17 = false;
    } in (forall([_DECL_17]), 1);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_comp() {
		check_no_stdlib(
			totalise,
			r#"
                predicate forall(array [int] of var bool);
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
    predicate forall(array [int] of var bool: _DECL_1);
    predicate forall_reif(array [int] of var bool: _DECL_2, var bool: _DECL_3);
    function bool: forall(array [int] of bool: _DECL_4);
    function array [int] of int: foo() = [1 | i in {1}, j in {2}, k in {3}];
    function array [int] of int: bar_root() = [let {
      constraint false;
    } in 1 | i in {1}, j in {2}, k in {3}];
    function tuple(bool, array [int] of int): bar() = let {
      array [int] of tuple(bool, int): _DECL_12 = [let {
      bool: _DECL_11 = false;
    } in (forall([_DECL_11]), 1) | i in {1}, j in {2}, k in {3}];
    } in (forall([forall([(_DECL_13).1 | _DECL_13 in _DECL_12])]), [_DECL_15 | _DECL_14 in _DECL_12, _DECL_15 = (_DECL_14).2]);
    function set of int: iter_root() = let {
      constraint false;
    } in {1};
    function tuple(bool, set of int): iter() = let {
      bool: _DECL_16 = false;
    } in (forall([_DECL_16]), {1});
    function array [int] of int: qux_root() = [1 | i in {1}, j in iter_root(), k in {3}];
    function tuple(bool, array [int] of int): qux() = let {
      array [int] of tuple(bool, array [int] of int): _DECL_21 = [let {
      tuple(bool, set of int): _DECL_19 = iter();
    } in (forall([(_DECL_19).1]), [1 | j in (_DECL_19).2, k in {3}]) | i in {1}];
    } in (forall([forall([(_DECL_22).1 | _DECL_22 in _DECL_21])]), [_DECL_25 | _DECL_23 in _DECL_21, _DECL_24 = (_DECL_23).2, _DECL_25 in _DECL_24]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_var_comp() {
		check_no_stdlib(
			transformer(vec![desugar_comprehension, erase_opt, totalise]),
			r#"
				annotation mzn_var_where_clause;
				function opt $T: val2opt($T: x);
                predicate forall(array [int] of var bool);
                predicate forall_reif(array [int] of var bool, var bool);
                test forall(array [int] of bool);
                predicate exists(array [int] of var bool);
                predicate exists_reif(array [int] of var bool);
				predicate if_then_else(array [int] of var bool, array [int] of bool);
				predicate if_then_else_reif(array [int] of var bool, array [int] of bool, var bool);
				function var int: if_then_else(array [int] of var bool, array [int] of int);
                predicate bar(int: x) = false;
                function set of int: qux() = let { constraint false } in {3, 4};
                function array [int] of var opt int: foo() =
                    [1 | i in {1, 2} where bar(i), j in qux()];
			"#,
			expect!([r#"
    annotation mzn_var_where_clause;
    predicate forall(array [int] of var bool: _DECL_1);
    predicate forall_reif(array [int] of var bool: _DECL_2, var bool: _DECL_3);
    function bool: forall(array [int] of bool: _DECL_4);
    predicate exists(array [int] of var bool: _DECL_5);
    predicate exists_reif(array [int] of var bool: _DECL_6);
    predicate if_then_else(array [int] of var bool: _DECL_7, array [int] of bool: _DECL_8);
    predicate if_then_else_reif(array [int] of var bool: _DECL_9, array [int] of bool: _DECL_10, var bool: _DECL_11);
    function var int: if_then_else(array [int] of var bool: _DECL_12, array [int] of int: _DECL_13);
    function var bool: bar(int: x) = false;
    function set of int: qux_root() = let {
      constraint false;
    } in {3, 4};
    function tuple(bool, set of int): qux() = let {
      bool: _DECL_15 = false;
    } in (forall([_DECL_15]), {3, 4});
    function array [int] of tuple(var bool, var int): foo_root() = [let {
      tuple(bool, int): _DECL_34 = (true, 1);
      tuple(bool, int): _DECL_35 = let {
      tuple(bool, int): _DECL_33 = (false, 0);
    } in _DECL_33;
      array [int] of var bool: _DECL_36 = [_DECL_31, true];
    } in (if_then_else(_DECL_36, [(_DECL_34).1, (_DECL_35).1]), if_then_else(_DECL_36, [(_DECL_34).2, (_DECL_35).2])) | i in {1, 2}, _DECL_31 = bar(i), j in qux_root()];
    function tuple(var bool, array [int] of tuple(var bool, var int)): foo() = let {
      array [int] of tuple(var bool, array [int] of tuple(var bool, var int)): _DECL_25 = [let {
      tuple(bool, array [int] of tuple(var bool, var int)): _DECL_24 = let {
      tuple(bool, set of int): _DECL_19 = qux();
    } in (forall([(_DECL_19).1]), [let {
      tuple(bool, int): _DECL_21 = (true, 1);
      tuple(bool, int): _DECL_22 = let {
      tuple(bool, int): _DECL_20 = (false, 0);
    } in _DECL_20;
      array [int] of var bool: _DECL_23 = [_DECL_17, true];
    } in (if_then_else(_DECL_23, [(_DECL_21).1, (_DECL_22).1]), if_then_else(_DECL_23, [(_DECL_21).2, (_DECL_22).2])) | j in (_DECL_19).2]);
    } in (exists([_DECL_17, (_DECL_24).1]), (_DECL_24).2) | i in {1, 2}, _DECL_17 = bar(i)];
    } in (forall([forall([(_DECL_26).1 | _DECL_26 in _DECL_25])]), [_DECL_29 | _DECL_27 in _DECL_25, _DECL_28 = (_DECL_27).2, _DECL_29 in _DECL_28]);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_bool_fns() {
		check_no_stdlib(
			totalise,
			r#"
    			function bool: forall(array [int] of bool: _DECL_2);
				function int: foo(int: x) = let {
					constraint false;
				} in 1;
                test bar(int: x) = let {
					int: f = foo(x);
				} in false;
				constraint bar(1);
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_2);
    function int: foo_root(int: x) = let {
      constraint false;
    } in 1;
    function tuple(bool, int): foo(int: x) = let {
      bool: _DECL_5 = false;
    } in (forall([_DECL_5]), 1);
    function bool: bar(int: x) = let {
      tuple(bool, int): f = foo(x);
      int: _DECL_7 = (f).2;
    } in forall([(f).1, false]);
    constraint bar(1);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_predicate_with_ite() {
		check_no_stdlib(
			totalise,
			r#"
                predicate forall(array [int] of var bool);
                predicate forall_reif(array [int] of var bool, var bool);
                test forall(array [int] of bool);
				function var int: qux(var int: x) = let {
					constraint false;
				} in x;
				predicate bar(var int: x);
				predicate bar_reif(var int: x, var bool: b);
                predicate foo(var int: x) =
					if true then let {
						var int: x = qux(3);
					} in bar(x)
					else true endif;;
				var int: v;
				constraint foo(v);
			"#,
			expect!([r#"
    predicate forall(array [int] of var bool: _DECL_1);
    predicate forall_reif(array [int] of var bool: _DECL_2, var bool: _DECL_3);
    function bool: forall(array [int] of bool: _DECL_4);
    function var int: qux_root(var int: x) = let {
      constraint false;
    } in x;
    function tuple(bool, var int): qux(var int: x) = let {
      bool: _DECL_12 = false;
    } in (forall([_DECL_12]), x);
    predicate bar(var int: x);
    predicate bar_reif(var int: x, var bool: b);
    function var bool: foo(var int: x) = let {
      tuple(bool, var int): x = qux(3);
      var int: _DECL_14 = (x).2;
    } in forall([(x).1, bar(_DECL_14)]);
    var int: v;
    constraint foo(v);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_abort() {
		check_no_stdlib(
			totalise,
			r#"
                test forall(array [int] of bool);
                test abort(string: msg);
				test bar(int: x);
				function int: foo(int: x) = let {
					constraint if bar(x) then abort("foo") endif;
				} in x;
				int: a = foo(2);
			"#,
			expect!([r#"
    function bool: forall(array [int] of bool: _DECL_1);
    function bool: abort(string: msg);
    function bool: bar(int: x);
    function int: foo(int: x) = let {
      constraint if bar(x) then abort("foo") else true endif;
    } in x;
    int: a = foo(2);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_totalise_default() {
		check_no_stdlib(
			totalise,
			r#"
                test forall(array [int] of bool);
                function var int: mzn_default_partial(var int: x, var int: def);
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
    } in (forall([_DECL_7]), 1);
      int: _DECL_9 = 2;
    } in if (_DECL_8).1 then (_DECL_8).2 else _DECL_9 endif;
    function int: qux_root(bool: b, bool: c) = let {
      tuple(bool, int): _DECL_15 = let {
      bool: _DECL_14 = b;
    } in (forall([_DECL_14]), 1);
      tuple(bool, int): _DECL_17 = let {
      bool: _DECL_16 = c;
    } in (forall([_DECL_16]), 2);
      tuple(bool, int): _DECL_18 = if (_DECL_15).1 then _DECL_15 else _DECL_17 endif;
      constraint (_DECL_18).1;
    } in (_DECL_18).2;
    function tuple(bool, int): qux(bool: b, bool: c) = let {
      tuple(bool, int): _DECL_11 = let {
      bool: _DECL_10 = b;
    } in (forall([_DECL_10]), 1);
      tuple(bool, int): _DECL_13 = let {
      bool: _DECL_12 = c;
    } in (forall([_DECL_12]), 2);
    } in if (_DECL_11).1 then _DECL_11 else _DECL_13 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_mzn_in_root_context() {
		check_no_stdlib(
			totalise,
			r#"
                predicate forall(array [int] of var bool);
                predicate forall_reif(array [int] of var bool, var bool);
				annotation promise_total;
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
    predicate forall(array [int] of var bool: _DECL_1);
    predicate forall_reif(array [int] of var bool: _DECL_2, var bool: _DECL_3);
    annotation promise_total;
    function bool: mzn_in_root_context();
    predicate bar(var int: x);
    predicate bar_reif(var int: x, var bool: b);
    function var bool: foo_root(var int: x) = let {
      constraint bar(x);
    } in true;
    function var bool: foo(var int: x) = if false then let {
      var bool: _DECL_10 = bar(x);
    } in _DECL_10 else foo_t(x) endif;
    function var bool: foo_t(var int: x) :: (promise_total) = let {
      var bool: b;
      constraint bar_reif(x, b);
    } in b;
    solve satisfy;
"#]),
		)
	}
}
