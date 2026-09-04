//! Type specialisation
//!
//! Creates concrete versions of polymorphic functions.
//! This enables type erasure while ensuring we call the right versions of functions involving e.g. enums.
//! We also create special versions of `show` if called on a type which will be erased later.
//! Array access also has to be specialised (and var index access to arrays of structs has to be decomposed).

use std::collections::hash_map::Entry;

use rustc_hash::{FxHashMap, FxHashSet};
use shackle_diagnostics::{Result, TypeSpecialisationRecursionLimit};
use shackle_hir::constants::IdentifierRegistry;
use shackle_ty::{FunctionType, PolymorphicFunctionType, Ty, TyData, TyParamInstantiations};
use shackle_utils::maybe_grow_stack;

use crate::{
	ArrayComprehension, ArrayLiteral, Call, Callable, Db, Declaration, DeclarationId, Domain,
	DomainData, Expression, ExpressionBuilder, Function, FunctionId, Generator, Identifier,
	IntegerLiteral, Item, ItemId, Marker, Model, OverloadMap, RecordAccess, RecordLiteral,
	StringLiteral, TupleAccess, TupleLiteral,
	pretty_print::PrettyPrinter,
	source::Origin,
	traverse::{Folder, ReplacementMap, add_function, fold_call, fold_declaration_id, fold_domain},
};

struct SpecialisedFunction<'db, Dst: Marker> {
	original: FunctionId<'db>,
	ty_vars: TyParamInstantiations<'db>,
	parameters: FxHashMap<DeclarationId<'db>, DeclarationId<'db, Dst>>,
	depth: u16,
}

struct TypeSpecialiser<'a, 'db, Dst: Marker> {
	specialised_model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst>,
	concrete: FxHashMap<(FunctionId<'db>, FunctionType<'db>), FunctionId<'db, Dst>>,
	specialised: Vec<(FunctionId<'db, Dst>, SpecialisedFunction<'db, Dst>)>,
	todo: Vec<SpecialisedFunction<'db, Dst>>,
	ids: &'db IdentifierRegistry<'db>,
	position: FxHashMap<FunctionId<'db>, ItemId<'db, Dst>>,
	count: usize,
	reached_recursion_limit: Option<FunctionId<'db>>,
	original_functions: OverloadMap<'a, 'db>,
	to_remove: FxHashSet<FunctionId<'db, Dst>>,
}

impl<'a, 'db, Dst: Marker> Folder<'_, 'db, Dst> for TypeSpecialiser<'a, 'db, Dst> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.specialised_model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst> {
		&mut self.replacement_map
	}

	fn add_model(&mut self, db: &'db dyn Db, model: &Model<'db>) {
		// Add items to the destination model
		for item in model.top_level_items() {
			self.add_item(db, model, item);
		}

		// Add bodies to non-specialised functions
		for (f, i) in model.all_functions() {
			if !i.is_polymorphic() && i.body().is_some() {
				self.fold_function_body(db, model, f);
			}
		}

		// Add bodies to specialised functions
		while let Some((f, mut s)) = self.specialised.pop() {
			if model[s.original].name() == self.ids.functions.array_access {
				// Create specialised decomposition of array access for var access of struct types
				if let Some(body) = self.decompose_array_access(db, model, f) {
					self.specialised_model[f].set_body(body);
					self.specialised_model[f].validate(db);
					continue;
				}
			}
			if (model[s.original].name() == self.ids.functions.show
				|| model[s.original].name() == self.ids.functions.show_json)
				&& model[s.original]
					.annotations()
					.has(model, self.ids.annotations.mzn_internal_generated)
			{
				// Create specialised show function for types which will be erased, except show on direct enum which will be generated later.
				let p = self.specialised_model[f].parameter(0);
				let ty = self.specialised_model[p].ty();
				if ty.contains_erased_type(db) {
					if !ty.is_enum(db) {
						let body = if model[s.original].name() == self.ids.functions.show {
							self.generate_show(db, model, p, ty)
						} else {
							self.generate_show_json(db, model, p, ty)
						};
						self.specialised_model[f].set_body(body);
						self.specialised_model[f].validate(db);
					}
					// Show for enum will be generated in enum erasure, just leave it without a body for now.
					continue;
				}
			}
			if (model[s.original].name() == self.ids.functions.eq
				|| model[s.original].name() == self.ids.functions.le
				|| model[s.original].name() == self.ids.functions.lt)
				&& model[s.original]
					.annotations()
					.has(model, self.ids.annotations.mzn_internal_generated)
				&& model[s.original].return_type().inst(db) == Some(shackle_ty::VarType::Var)
			{
				let lhs = self.specialised_model[f].parameter(0);
				let rhs = self.specialised_model[f].parameter(1);
				let body = if model[s.original].name() == self.ids.functions.lt {
					self.generate_lt_or_le(db, model, model[s.original].origin(), lhs, rhs, true)
				} else if model[s.original].name() == self.ids.functions.le {
					self.generate_lt_or_le(db, model, model[s.original].origin(), lhs, rhs, false)
				} else {
					self.generate_eq(db, model, model[s.original].origin(), lhs, rhs)
				};
				self.specialised_model[f].set_body(body);
				self.specialised_model[f].validate(db);
				continue;
			}

			if let Some(b) = model[s.original].body() {
				log::debug!(
					"Adding specialised body to {} (call depth {})",
					PrettyPrinter::new(db, &self.specialised_model)
						.pretty_print_signature(f.into()),
					s.depth
				);
				if s.depth > 1000 {
					log::debug!(
						"Reached maximum depth for {}",
						model[s.original].name().pretty_print(db)
					);
					self.reached_recursion_limit = Some(s.original);
					return;
				}
				s.depth += 1;
				self.todo.push(s);
				let body = self.fold_expression(db, model, b);
				let _ = self.todo.pop();
				self.specialised_model[f].set_body(body);
				self.specialised_model[f].validate(db);
				continue;
			}
		}

		assert!(self.todo.is_empty());
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db>, f: FunctionId<'db>) {
		if model[f].is_polymorphic()
			&& let Some(last) = self.specialised_model.all_items().last()
		{
			let _ = self.position.insert(f, last);
		}

		let idx = add_function(self, db, model, f);

		if model[f].is_polymorphic() && model[f].body().is_some()
			|| model[f]
				.annotations()
				.has(model, self.ids.annotations.mzn_unreachable)
		{
			// Remove non-builtin polymorphic and unreachable functions
			let _ = self.to_remove.insert(idx);
		}
	}

	fn fold_declaration_id(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		d: DeclarationId<'db>,
	) -> DeclarationId<'db, Dst> {
		if let Some(sf) = self.todo.last()
			&& let Some(result) = sf.parameters.get(&d)
		{
			// Map to specialised parameter
			return *result;
		}
		fold_declaration_id(self, db, model, d)
	}

	fn fold_call(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		call: &Call<'db>,
	) -> Call<'db, Dst> {
		if let Callable::Function(f) = &call.function {
			let arguments = call
				.arguments
				.iter()
				.map(|arg| self.fold_expression(db, model, arg))
				.collect::<Vec<_>>();
			// Match the new argument types in the old model to find the most specific overload
			let arg_tys = arguments.iter().map(|e| e.ty()).collect::<Vec<_>>();
			return Call {
				function: Callable::Function(self.instantiate(db, model, *f, &arg_tys)),
				arguments,
			};
		}
		fold_call(self, db, model, call)
	}

	fn fold_domain(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		domain: &Domain<'db>,
	) -> Domain<'db, Dst> {
		maybe_grow_stack(|| {
			if let Some(s) = self.todo.last() {
				// Instantiate type-inst vars in param/return types
				if let DomainData::Unbounded = &**domain {
					return Domain::unbounded(
						db,
						domain.origin(),
						domain.ty().instantiate_ty_vars(db, &s.ty_vars),
					);
				}
			}
			fold_domain(self, db, model, domain)
		})
	}
}

impl<'a, 'db, Dst: Marker> TypeSpecialiser<'a, 'db, Dst> {
	// Get or create the specialised version of a polymorphic function with the given argument types
	fn instantiate(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		function: FunctionId<'db>,
		args: &[Ty<'db>],
	) -> FunctionId<'db, Dst> {
		let name = model[function].name();
		let lookup = self
			.original_functions
			.rematch_fn(db, function, args)
			.unwrap_or_else(|e| panic!("{}", e.pretty_print(db)));
		let f = lookup.function;

		// Also instantiate subtyped polymorphic functions so we can dispatch to them
		let fns = self.original_functions.get(&name).unwrap().clone();
		for idx in fns {
			if f == idx
				|| !model[idx].is_polymorphic()
				|| model[idx].parameters().len() != args.len()
				|| model[idx]
					.annotations()
					.has(model, self.ids.annotations.mzn_unreachable)
			{
				continue;
			}
			(|ts: &mut Self| {
				let mut ty_vars = FxHashMap::default();
				let fe = model[idx].function_entry(model);
				let mut add_instantiation = |tv, ty| {
					match ty_vars.entry(tv) {
						Entry::Occupied(mut e) => {
							let p = e.get_mut();
							let Some(st) = Ty::most_specific_supertype(db, [*p, ty]) else {
								return false;
							};
							*p = st;
						}
						Entry::Vacant(e) => {
							let _ = e.insert(ty);
						}
					}
					true
				};
				for (arg, param) in args.iter().zip(fe.params().iter()) {
					if !PolymorphicFunctionType::collect_instantiations(
						db,
						&mut add_instantiation,
						*arg,
						*param,
					) {
						return;
					}
				}
				let ft = fe.instantiate(db, &ty_vars);
				if ft
					.params
					.iter()
					.zip(args.iter())
					.all(|(p, a)| p.is_subtype_of(db, *a))
				{
					let matched = ts
						.original_functions
						.rematch_fn(db, f, &ft.params)
						.unwrap()
						.function;
					if matched == idx {
						log::debug!(
							"Instantiating {} with {} has subtype {} with args {}",
							PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
							args.iter()
								.map(|ty| ty.pretty_print(db))
								.collect::<Vec<_>>()
								.join(", "),
							PrettyPrinter::new(db, model).pretty_print_signature(idx.into()),
							ft.params
								.iter()
								.map(|ty| ty.pretty_print(db))
								.collect::<Vec<_>>()
								.join(", ")
						);
						let _ = ts.instantiate_inner(db, model, idx, &ft.params);
					}
				}
			})(self);
		}
		self.instantiate_inner(db, model, f, args)
	}

	fn instantiate_inner(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		f: FunctionId<'db>,
		args: &[Ty<'db>],
	) -> FunctionId<'db, Dst> {
		assert!(
			!model[f]
				.annotations()
				.has(model, self.ids.annotations.mzn_unreachable),
			"Tried to instantiate unreachable internal function {} with args {}",
			PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
			args.iter()
				.map(|ty| ty.pretty_print(db))
				.collect::<Vec<_>>()
				.join(", ")
		);

		let needs_instantiation = model[f].is_polymorphic() && model[f].body().is_some();
		if !needs_instantiation {
			return self.fold_function_id(db, model, f);
		}

		assert!(model[f].is_polymorphic());
		assert!(model[f].top_level());

		let (ty_vars, function_type) = model[f]
			.function_entry(model)
			.instantiate_ty_params(db, args)
			.unwrap();

		let key = (f, function_type);

		log::debug!(
			"Instantiating {} with {}",
			PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
			ty_vars
				.iter()
				.map(|(tv, ty)| format!("{} = {}", tv.pretty_print(db), ty.pretty_print(db)))
				.collect::<Vec<_>>()
				.join(", ")
		);

		let concrete = self.concrete.get(&key);
		if let Some(concrete) = concrete {
			// Already instantiated this version
			log::debug!("Already exists");
			return *concrete;
		}

		let fn_match = model
			.rematch_fn(db, f, args)
			.unwrap_or_else(|e| panic!("{}", e.pretty_print(db)));
		if !fn_match.fn_entry.is_polymorphic() {
			// Already have existing concrete version, no need to create
			return self.fold_function_id(db, model, fn_match.function);
		}

		// Create specialised version of polymorphic function
		self.todo.push(SpecialisedFunction {
			original: f,
			ty_vars,
			parameters: FxHashMap::default(),
			depth: self.todo.last().map(|t| t.depth).unwrap_or_default(),
		});
		let mut function = Function::new(
			model[f].name(),
			self.fold_domain(db, model, model[f].domain()),
		);
		function.set_specialised(Some(f.into()));
		function.annotations_mut().extend(
			model[f]
				.annotations()
				.iter()
				.map(|ann| self.fold_expression(db, model, ann)),
		);
		function.set_parameters(model[f].parameters().iter().map(|p| {
			self.add_parameter_declaration(db, model, *p);
			self.fold_declaration_id(db, model, *p)
		}));

		let mut specialised = self.todo.pop().unwrap();
		specialised.parameters = model[f]
			.parameters()
			.iter()
			.copied()
			.zip(function.parameters().iter().copied())
			.collect();
		let position = || {
			if function.name() == self.ids.functions.show
				|| function.name() == self.ids.functions.show_json
				|| function.name() == self.ids.functions.show_dzn
			{
				// Show involving enums must appear after the definition of the enum
				let param_ty = &self.specialised_model[function.parameter(0)].ty();
				let needs_enums = param_ty
					.walk(db)
					.filter_map(|ty| ty.enum_ty(db))
					.collect::<FxHashSet<_>>();
				if !needs_enums.is_empty() {
					let enum_tys = self
						.specialised_model
						.enumerations()
						.map(|(idx, e)| (idx, e.enum_type()))
						.collect::<Vec<_>>();
					for (idx, e) in enum_tys.into_iter().rev() {
						if needs_enums.contains(&e) {
							return Some(ItemId::from(idx));
						}
					}
				}
			}
			self.position.get(&f).copied()
		};
		let idx = if let Some(p) = position() {
			self.specialised_model
				.add_function_after(Item::new(function, model[f].origin()), p)
		} else {
			self.specialised_model
				.prepend_function(Item::new(function, model[f].origin()))
		};
		let _ = self.concrete.insert(key, idx);
		self.specialised.push((idx, specialised));
		log::debug!(
			"Created {}",
			PrettyPrinter::new(db, &self.specialised_model).pretty_print_signature(idx.into())
		);
		self.count += 1;
		idx
	}

	fn expr(
		&self,
		db: &'db dyn Db,
		origin: impl Into<Origin<'db>>,
		e: impl ExpressionBuilder<'db, Dst>,
	) -> Expression<'db, Dst> {
		Expression::new(db, &self.specialised_model, origin, e)
	}

	/// Lookup call by name from original model and instantiate
	fn call(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		origin: impl Into<Origin<'db>>,
		name: Identifier<'db>,
		args: Vec<Expression<'db, Dst>>,
	) -> Expression<'db, Dst> {
		let arg_tys = args.iter().map(|arg| arg.ty()).collect::<Vec<_>>();
		let function = model
			.lookup_function(db, name.into(), &arg_tys)
			.unwrap()
			.function;
		let idx = self.instantiate(db, model, function, &arg_tys);
		self.expr(
			db,
			origin,
			Call {
				function: Callable::Function(idx),
				arguments: args,
			},
		)
	}

	// Generate specialised body for show needed if type will be erased
	fn generate_show(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		arg: DeclarationId<'db, Dst>,
		ty: Ty<'db>,
	) -> Expression<'db, Dst> {
		log::debug!("Generating specialised show for {}", ty.pretty_print(db));

		let origin = self.specialised_model[arg].origin();
		match ty.lookup(db) {
			TyData::Tuple(_, fs) => {
				// concat(["(", show(x.1), ", ", show(x.2), ")"])
				let mut fields = Vec::with_capacity(2 * fs.len() + 1);
				fields.push(self.expr(db, origin, StringLiteral::new(db, "(")));
				for i in 1..=fs.len() {
					if i > 1 {
						fields.push(self.expr(db, origin, StringLiteral::new(db, ", ")));
					}
					let show = self.call(
						db,
						model,
						origin,
						self.ids.functions.show,
						vec![self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(self.expr(db, origin, arg)),
								field: IntegerLiteral(i as i64),
							},
						)],
					);
					fields.push(show);
				}
				fields.push(self.expr(db, origin, StringLiteral::new(db, ")")));
				self.call(
					db,
					model,
					origin,
					self.ids.functions.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				)
			}
			TyData::Record(_, fs) => {
				// concat(["(", "foo", ": ", show(x.foo), ", ", "bar", ": ", show(x.bar), ")"])
				let mut fields = Vec::with_capacity(fs.len() * 4 + 1);
				fields.push(self.expr(db, origin, StringLiteral::new(db, "(")));
				let mut first = true;
				for (i, _) in fs.iter() {
					if first {
						first = false;
					} else {
						fields.push(self.expr(db, origin, StringLiteral::new(db, ", ")));
					}
					fields.push(self.expr(db, origin, StringLiteral::from(*i)));
					fields.push(self.expr(db, origin, StringLiteral::new(db, ": ")));
					let show = self.call(
						db,
						model,
						origin,
						self.ids.functions.show,
						vec![self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(self.expr(db, origin, arg)),
								field: Identifier(*i),
							},
						)],
					);
					fields.push(show);
				}
				fields.push(self.expr(db, origin, StringLiteral::new(db, ")")));
				self.call(
					db,
					model,
					origin,
					self.ids.functions.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				)
			}
			_ => unreachable!(
				"Unexpected type for specialised show: {}",
				ty.pretty_print(db)
			),
		}
	}

	// Generate specialised body for showJSON needed if type will be erased
	fn generate_show_json(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		arg: DeclarationId<'db, Dst>,
		ty: Ty<'db>,
	) -> Expression<'db, Dst> {
		log::debug!(
			"Generating specialised showJSON for {}",
			ty.pretty_print(db)
		);

		let origin = self.specialised_model[arg].origin();
		let call = |ts: &mut Self, name: Identifier<'db>, args: Vec<Expression<'db, Dst>>| {
			let arg_tys = args.iter().map(|arg| arg.ty()).collect::<Vec<_>>();
			let function = model
				.lookup_function(db, name.into(), &arg_tys)
				.unwrap()
				.function;
			let idx = ts.instantiate(db, model, function, &arg_tys);
			Call {
				function: Callable::Function(idx),
				arguments: args,
			}
		};

		match ty.lookup(db) {
			TyData::Tuple(_, fs) => {
				// concat(["[", showJSON(x.1), ", ", showJSON(x.2), "]"])
				let mut fields = Vec::with_capacity(2 * fs.len() + 1);
				fields.push(self.expr(db, origin, StringLiteral::new(db, "[")));
				for i in 1..=fs.len() {
					if i > 1 {
						fields.push(self.expr(db, origin, StringLiteral::new(db, ", ")));
					}
					let show = call(
						self,
						self.ids.functions.show_json,
						vec![self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(self.expr(db, origin, arg)),
								field: IntegerLiteral(i as i64),
							},
						)],
					);
					fields.push(self.expr(db, origin, show));
				}
				fields.push(self.expr(db, origin, StringLiteral::new(db, "]")));
				let concat = call(
					self,
					self.ids.functions.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				);
				self.expr(db, origin, concat)
			}
			TyData::Record(_, fs) => {
				// concat(["{", "foo", showJSON(": "), showJSON(x.foo), ", ", showJSON("bar"), ": ", showJSON(x.bar), "}"])
				let mut fields = Vec::with_capacity(fs.len() * 4 + 1);
				fields.push(self.expr(db, origin, StringLiteral::new(db, "{")));
				let mut first = true;
				for (i, _) in fs.iter() {
					if first {
						first = false;
					} else {
						fields.push(self.expr(db, origin, StringLiteral::new(db, ", ")));
					}
					let show_name = call(
						self,
						self.ids.functions.show_json,
						vec![self.expr(db, origin, StringLiteral::from(*i))],
					);
					fields.push(self.expr(db, origin, show_name));
					fields.push(self.expr(db, origin, StringLiteral::new(db, ": ")));
					let show_value = call(
						self,
						self.ids.functions.show_json,
						vec![self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(self.expr(db, origin, arg)),
								field: Identifier(*i),
							},
						)],
					);
					fields.push(self.expr(db, origin, show_value));
				}
				fields.push(self.expr(db, origin, StringLiteral::new(db, "}")));
				let concat = call(
					self,
					self.ids.functions.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				);
				self.expr(db, origin, concat)
			}
			_ => unreachable!(
				"Unexpected type for specialised show: {}",
				ty.pretty_print(db)
			),
		}
	}

	fn generate_eq(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		origin: Origin<'db>,
		lhs: DeclarationId<'db, Dst>,
		rhs: DeclarationId<'db, Dst>,
	) -> Expression<'db, Dst> {
		log::debug!(
			"Generating specialised {} = {} function",
			self.specialised_model[lhs].ty().pretty_print(db),
			self.specialised_model[rhs].ty().pretty_print(db)
		);
		let mut eqs = Vec::new();
		let mut todo = vec![(self.expr(db, origin, lhs), self.expr(db, origin, rhs))];
		while let Some((lhs, rhs)) = todo.pop() {
			let ty = lhs.ty();
			match ty.lookup(db) {
				TyData::Tuple(_, fs) => {
					for i in 1_i64..=(fs.len() as i64) {
						let new_lhs = self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(lhs.clone()),
								field: IntegerLiteral(i),
							},
						);
						let new_rhs = self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(rhs.clone()),
								field: IntegerLiteral(i),
							},
						);
						todo.push((new_lhs, new_rhs));
					}
				}
				TyData::Record(_, fs) => {
					for i in fs.iter().map(|(i, _)| *i) {
						let new_lhs = self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(lhs.clone()),
								field: Identifier(i),
							},
						);
						let new_rhs = self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(rhs.clone()),
								field: Identifier(i),
							},
						);
						todo.push((new_lhs, new_rhs));
					}
				}
				_ => {
					eqs.push(self.call(db, model, origin, self.ids.functions.eq, vec![lhs, rhs]));
				}
			}
		}
		self.call(
			db,
			model,
			origin,
			self.ids.functions.forall,
			vec![self.expr(db, origin, ArrayLiteral(eqs))],
		)
	}

	fn generate_lt_or_le(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		origin: Origin<'db>,
		lhs: DeclarationId<'db, Dst>,
		rhs: DeclarationId<'db, Dst>,
		strict: bool,
	) -> Expression<'db, Dst> {
		log::debug!(
			"Generating specialised {} < {} function",
			self.specialised_model[lhs].ty().pretty_print(db),
			self.specialised_model[rhs].ty().pretty_print(db)
		);
		// TODO: Generate lex_less[eq] constraint if possible

		let mut parts = Vec::new();
		let mut todo = vec![(self.expr(db, origin, lhs), self.expr(db, origin, rhs))];
		while let Some((lhs, rhs)) = todo.pop() {
			let ty = lhs.ty();
			match ty.lookup(db) {
				TyData::Tuple(_, fs) => {
					for i in 1_i64..=(fs.len() as i64) {
						let new_lhs = self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(lhs.clone()),
								field: IntegerLiteral(i),
							},
						);
						let new_rhs = self.expr(
							db,
							origin,
							TupleAccess {
								tuple: Box::new(rhs.clone()),
								field: IntegerLiteral(i),
							},
						);
						todo.push((new_lhs, new_rhs));
					}
				}
				TyData::Record(_, fs) => {
					for i in fs.iter().map(|(i, _)| *i) {
						let new_lhs = self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(lhs.clone()),
								field: Identifier(i),
							},
						);
						let new_rhs = self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(rhs.clone()),
								field: Identifier(i),
							},
						);
						todo.push((new_lhs, new_rhs));
					}
				}
				_ => {
					parts.push((
						self.call(
							db,
							model,
							origin,
							self.ids.functions.le,
							vec![lhs.clone(), rhs.clone()],
						),
						self.call(db, model, origin, self.ids.functions.lt, vec![lhs, rhs]),
					));
				}
			}
		}

		let mut iter = parts.into_iter();
		let (last_le, last_lt) = iter.next().unwrap();
		let mut expr = if strict { last_lt } else { last_le };
		for (le, lt) in iter {
			let or = self.call(db, model, origin, self.ids.functions.or, vec![lt, expr]);
			expr = self.call(db, model, origin, self.ids.functions.and, vec![le, or]);
		}

		expr
	}

	fn decompose_array_access(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db>,
		f: FunctionId<'db, Dst>,
	) -> Option<Expression<'db, Dst>> {
		let array = self.specialised_model[f].parameter(0);
		let indices = self.specialised_model[f].parameter(1);
		if self.specialised_model[indices].ty().contains_var(db) {
			let elem = self.specialised_model[array].ty().elem_ty(db).unwrap();
			if elem.is_tuple(db) || elem.is_record(db) {
				log::debug!(
					"Creating specialised array access for {}",
					PrettyPrinter::new(db, &self.specialised_model)
						.pretty_print_signature(f.into())
				);

				// Decompose access to array of structured types
				let origin = self.specialised_model[f].origin();
				let c_origin = self.specialised_model[array].origin();
				let c_ident = Expression::new(db, &self.specialised_model, c_origin, array);
				let i_origin = self.specialised_model[indices].origin();
				let i_ident = Expression::new(db, &self.specialised_model, i_origin, indices);

				if let Some(fields) = elem.record_fields(db) {
					let mut decomposed = Vec::with_capacity(fields.len());
					for (k, _) in fields {
						let field = Identifier::from(k);
						let decl = Declaration::new(false, Domain::unbounded(db, c_origin, elem));
						let decl_idx = self
							.specialised_model
							.add_declaration(Item::new(decl, c_origin));
						let generators = vec![Generator::Iterator {
							declarations: vec![decl_idx],
							collection: c_ident.clone(),
							where_clause: None,
						}];
						let comprehension = Expression::new(
							db,
							&self.specialised_model,
							origin,
							ArrayComprehension {
								generators,
								indices: None,
								template: Box::new(Expression::new(
									db,
									&self.specialised_model,
									origin,
									RecordAccess {
										record: Box::new(Expression::new(
											db,
											&self.specialised_model,
											c_origin,
											decl_idx,
										)),
										field,
									},
								)),
							},
						);
						let array = self.call(
							db,
							model,
							origin,
							self.ids.functions.array_xd,
							vec![c_ident.clone(), comprehension],
						);
						let inner = self.call(
							db,
							model,
							origin,
							self.ids.functions.array_access,
							vec![array, i_ident.clone()],
						);
						decomposed.push((field, inner));
					}
					return Some(Expression::new(
						db,
						&self.specialised_model,
						origin,
						RecordLiteral(decomposed),
					));
				}
				let fields = elem.field_len(db).unwrap();
				let mut decomposed = Vec::with_capacity(fields);
				for i in 1..=(fields as i64) {
					let decl = Declaration::new(false, Domain::unbounded(db, c_origin, elem));
					let decl_idx = self
						.specialised_model
						.add_declaration(Item::new(decl, c_origin));
					let generators = vec![Generator::Iterator {
						declarations: vec![decl_idx],
						collection: c_ident.clone(),
						where_clause: None,
					}];
					let comprehension = Expression::new(
						db,
						&self.specialised_model,
						origin,
						ArrayComprehension {
							generators,
							indices: None,
							template: Box::new(Expression::new(
								db,
								&self.specialised_model,
								origin,
								TupleAccess {
									tuple: Box::new(Expression::new(
										db,
										&self.specialised_model,
										c_origin,
										decl_idx,
									)),
									field: IntegerLiteral(i),
								},
							)),
						},
					);
					let array = self.call(
						db,
						model,
						origin,
						self.ids.functions.array_xd,
						vec![c_ident.clone(), comprehension],
					);
					let inner = self.call(
						db,
						model,
						origin,
						self.ids.functions.array_access,
						vec![array, i_ident.clone()],
					);
					decomposed.push(inner);
				}
				return Some(Expression::new(
					db,
					&self.specialised_model,
					origin,
					TupleLiteral(decomposed),
				));
			}
		}
		None
	}
}

struct RemoveUnreachableFunctions<'db, Dst: Marker> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst>,
	to_remove: FxHashSet<FunctionId<'db>>,
}

impl<'db, Dst: Marker> Folder<'_, 'db, Dst> for RemoveUnreachableFunctions<'db, Dst> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &'_ Model<'db, ()>, f: FunctionId<'db, ()>) {
		if self.to_remove.contains(&f) {
			return;
		}
		let _ = add_function(self, db, model, f);
	}
}

/// Type specialise a model
pub fn type_specialise<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Performing type specialisation");
	let ids = IdentifierRegistry::lookup(db);
	let mut ts = TypeSpecialiser {
		replacement_map: ReplacementMap::default(),
		specialised_model: Model::with_capacities(&model.item_counts()),
		concrete: FxHashMap::default(),
		specialised: Vec::new(),
		todo: Vec::new(),
		ids,
		position: FxHashMap::default(),
		count: 0,
		reached_recursion_limit: None,
		original_functions: model.overload_map(),
		to_remove: FxHashSet::default(),
	};
	ts.add_model(db, &model);
	log::info!("Created {} specialised functions", ts.count);
	if let Some(f) = ts.reached_recursion_limit {
		let (src, span) = model[f].origin().source_span(db);
		return Err(TypeSpecialisationRecursionLimit {
			name: model[f].name().pretty_print(db),
			src,
			span,
		}
		.into());
	}
	let mut ruf = RemoveUnreachableFunctions {
		model: Model::with_capacities(&ts.specialised_model.item_counts()),
		replacement_map: ReplacementMap::default(),
		to_remove: ts.to_remove,
	};
	ruf.add_model(db, &ts.specialised_model);
	Ok(ruf.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::type_specialise;
	use crate::transform::{
		name_mangle::mangle_names,
		tests::{check, check_no_stdlib},
		transformer,
	};

	#[test]
	fn test_type_specialisation_basic_1() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
					function any $T: foo(any $T: x) = x;
					predicate bar(var bool: p) = foo(p);
					constraint bar(true);
					any: y = foo(10);
					"#,
			expect!([r#"
    function var bool: 'foo<var bool>'(var bool: x) = x;
    function int: 'foo<int>'(int: x) = x;
    function var bool: bar(var bool: p) = 'foo<var bool>'(p);
    constraint bar(true);
    int: y = 'foo<int>'(10);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_basic_2() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			test foo(any $T: x) = true;
			any: a = foo((1, 2));
			any: b = foo((p: 1, q: 2));
			"#,
			expect!([r#"
    function bool: 'foo<record(int: p, int: q)>'(record(int: p, int: q): x) = true;
    function bool: 'foo<tuple(int, int)>'(tuple(int, int): x) = true;
    bool: a = 'foo<tuple(int, int)>'((1, 2));
    bool: b = 'foo<record(int: p, int: q)>'((p: 1, q: 2));
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_overloading() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			test foo(any $T: x) = bar(x);
			test bar(any $T: x) = true;
			test bar(int: x) = false;
			any: a = foo(1);
			"#,
			expect!([r#"
    function bool: foo(int: x) = bar(x);
    function bool: bar(int: x) = false;
    bool: a = foo(1);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_show() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			annotation mzn_internal_generated;
			test occurs(opt $T: x);
			function $T: deopt(opt $T: x);
			function string: concat(array [$T] of string: x);
			function string: join(string: s, array [$T] of string: x);
			function string: show($T: x) :: mzn_internal_generated = "";
			function string: show(opt $T: x) = if occurs(x) then show(deopt(x)) else "<>" endif;
			function string: show(array [$X] of $T: x) = concat(["[", join(",", [show(x_i) | x_i in x]), "]"]);
			output [show((a: 1, b: 2))];
			array [int] of tuple(opt int, bool): x;
			output [show(x)];
			"#,
			expect!([r#"
    annotation mzn_internal_generated;
    function bool: occurs(opt $T: x);
    function $T: deopt(opt $T: x);
    function string: concat(array [$T] of string: x);
    function string: join(string: s, array [$T] of string: x);
    function string: 'show<bool>'(bool: x) :: (mzn_internal_generated) = "";
    function string: 'show<int>'(int: x) :: (mzn_internal_generated) = "";
    function string: 'show<tuple(opt int, bool)>'(tuple(opt int, bool): x) :: (mzn_internal_generated) = concat(["(", 'show<opt int>'((x).1), ", ", 'show<bool>'((x).2), ")"]);
    function string: 'show<record(int: a, int: b)>'(record(int: a, int: b): x) :: (mzn_internal_generated) = concat(["(", "a", ": ", 'show<int>'((x).a), ", ", "b", ": ", 'show<int>'((x).b), ")"]);
    function string: 'show<opt int>'(opt int: x) = if occurs(x) then 'show<int>'(deopt(x)) else "<>" endif;
    function string: 'show<array [int] of tuple(opt int, bool)>'(array [int] of tuple(opt int, bool): x) = concat(["[", join(",", ['show<tuple(opt int, bool)>'(x_i) | x_i in x]), "]"]);
    output ['show<record(int: a, int: b)>'((a: 1, b: 2))];
    array [int] of tuple(opt int, bool): x;
    output ['show<array [int] of tuple(opt int, bool)>'(x)];
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_compare() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
				annotation mzn_internal_generated;
				predicate forall(array [int] of var bool);
				predicate '/\'(var bool: x, var bool: y);
				predicate '\/'(var bool: x, var bool: y);
				predicate '<='(any $T: x, any $T: y) :: mzn_internal_generated = true;
				predicate '<'(any $T: x, any $T: y) :: mzn_internal_generated = true;
				predicate '='(any $T: x, any $T: y) :: mzn_internal_generated = true;
				record(var int: a, var int: b): x;
				record(var int: a, var int: b): y;
				constraint x < y;
				constraint x = y;
			"#,
			expect!([r#"
    annotation mzn_internal_generated;
    predicate forall(array [int] of var bool: _DECL_1);
    function var bool: '/\'(var bool: x, var bool: y);
    function var bool: '\/'(var bool: x, var bool: y);
    function var bool: '<='(var int: x, var int: y) :: (mzn_internal_generated) = '<='(x, y);
    function var bool: '<<var int, var int>'(var int: x, var int: y) :: (mzn_internal_generated) = '<<var int, var int>'(x, y);
    function var bool: '<<record(var int: a, var int: b), record(var int: a, var int: b)>'(record(var int: a, var int: b): x, record(var int: a, var int: b): y) :: (mzn_internal_generated) = '/\'('<='((x).a, (y).a), '\/'('<<var int, var int>'((x).a, (y).a), '<<var int, var int>'((x).b, (y).b)));
    function var bool: '=<var int, var int>'(var int: x, var int: y) :: (mzn_internal_generated) = forall(['=<var int, var int>'(x, y)]);
    function var bool: '=<record(var int: a, var int: b), record(var int: a, var int: b)>'(record(var int: a, var int: b): x, record(var int: a, var int: b): y) :: (mzn_internal_generated) = forall(['=<var int, var int>'((x).b, (y).b), '=<var int, var int>'((x).a, (y).a)]);
    record(var int: a, var int: b): x;
    record(var int: a, var int: b): y;
    constraint '<<record(var int: a, var int: b), record(var int: a, var int: b)>'(x, y);
    constraint '=<record(var int: a, var int: b), record(var int: a, var int: b)>'(x, y);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_equivalent() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			test foo(var $T: v) = true;
			var int: x;
			int: y;
			any: a = foo(x);
			any: b = foo(y);
			"#,
			expect!([r#"
    function bool: foo(var int: v) = true;
    var int: x;
    int: y;
    bool: a = foo(x);
    bool: b = foo(y);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_enum_show() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			annotation mzn_internal_generated;
			function string: show($T: x) :: mzn_internal_generated = "";
			enum Foo;
			Foo: x;
			output [show(x)];
			"#,
			expect!([r#"
    annotation mzn_internal_generated;
    enum Foo;
    function string: show(Foo: x) :: (mzn_internal_generated);
    Foo: x;
    output [show(x)];
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_enum_show_2() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			annotation mzn_internal_generated;
			function string: show($T: x) :: mzn_internal_generated = "";
			function string: foo($$E: x) = show(x);
			enum Foo;
			Foo: x;
			output [foo(x)];
			"#,
			expect!([r#"
    annotation mzn_internal_generated;
    function string: foo(Foo: x) = show(x);
    enum Foo;
    function string: show(Foo: x) :: (mzn_internal_generated);
    Foo: x;
    output [foo(x)];
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_recursive() {
		check_no_stdlib(
			type_specialise,
			r#"
			test foo($T: x) = foo((1, x));
			any: f = foo(1);
			"#,
			expect!("Function instantiation error"),
		)
	}

	#[test]
	fn test_type_specialisation_nested() {
		check_no_stdlib(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			function $T: foo($T: x) = bar(x);
			function $T: bar($T: x) = x;
			function float: bar(float: x) = 2.4;
			any: a = foo(1);
			any: b = foo(1.5);
		"#,
			expect!([r#"
    function float: 'foo<float>'(float: x) = 'bar<float>'(x);
    function int: 'foo<int>'(int: x) = 'bar<int>'(x);
    function int: 'bar<int>'(int: x) = x;
    function float: 'bar<float>'(float: x) = 2.4;
    int: a = 'foo<int>'(1);
    float: b = 'foo<float>'(1.5);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_specialise_array_access_1() {
		check(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			any: x = [1, 2, 3];
			any: v = x[1];
			var 1..3: i;
			any: u = x[i];
		"#,
			expect!([r#"
    array [int] of int: x = [1, 2, 3];
    int: v = '[]<array [int] of int, int>'(x, 1);
    var '..<int, int>'(1, 3): i;
    var int: u = '[]<array [int] of var int, var int>'(x, i);
"#]),
		)
	}

	#[test]
	fn test_specialise_array_access_2() {
		check(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			enum Foo = {A, B, C};
			array [Foo] of var 1..3: x;
			any: v = x[A];
			var Foo: i;
			any: u = x[i];
		"#,
			expect!([r#"
    enum Foo = { A } ++ { B } ++ { C };
    array [Foo] of var '..<int, int>'(1, 3): x;
    var int: v = '[]<array [Foo] of var int, Foo>'(x, A);
    var Foo: i;
    var int: u = '[]<array [Foo] of var int, var Foo>'(x, i);
"#]),
		)
	}

	#[test]
	fn test_specialise_array_access_3() {
		check(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			array [1..3] of tuple(int, int): x;
			var 1..3: i;
			any: v = x[i];
		"#,
			expect!([r#"
    array ['..<int, int>'(1, 3)] of tuple(int, int): x;
    var '..<int, int>'(1, 3): i;
    tuple(var int, var int): v = '[]<array [int] of tuple(var int, var int), var int>'(x, i);
"#]),
		)
	}

	#[test]
	fn test_specialise_array_access_4() {
		check(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			array [1..3] of record(int: foo, int: bar): x;
			var 1..3: i;
			any: v = x[i];
		"#,
			expect!([r#"
    array ['..<int, int>'(1, 3)] of record(int: bar, int: foo): x;
    var '..<int, int>'(1, 3): i;
    record(var int: bar, var int: foo): v = '[]<array [int] of record(var int: bar, var int: foo), var int>'(x, i);
"#]),
		)
	}

	#[test]
	fn test_specialise_dispatch() {
		check(
			transformer(vec![type_specialise, mangle_names]),
			r#"
			function int: foo(var opt $$E: x) = 1;
			function int: foo(var $$E: x) = 2;
			function int: foo($$E: x) = 3;
			function int: foo(opt $$E: x) = 4;
			var opt int: x;
			any: y = foo(x);
		"#,
			expect!([r#"
    function int: 'foo<var opt int>'(var opt int: x) = 1;
    function int: 'foo<var int>'(var int: x) = 2;
    function int: 'foo<int>'(int: x) = 3;
    function int: 'foo<opt int>'(opt int: x) = 4;
    var opt int: x;
    int: y = 'foo<var opt int>'(x);
"#]),
		)
	}
}
