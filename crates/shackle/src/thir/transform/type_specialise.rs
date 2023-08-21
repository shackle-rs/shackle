//! Type specialisation
//!
//! Creates concrete versions of polymorphic functions.
//! This enables type erasure while ensuring we call the right versions of functions involving e.g. enums.
//! We also create special versions of `show` if called on a type which will be erased later.

use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::{
	constants::IdentifierRegistry,
	thir::{
		db::Thir,
		pretty_print::PrettyPrinter,
		source::Origin,
		traverse::{
			add_function, fold_call, fold_declaration_id, fold_domain, Folder, ReplacementMap,
		},
		ArrayComprehension, ArrayLiteral, Branch, Call, Callable, Declaration, DeclarationId,
		Domain, DomainData, Expression, ExpressionBuilder, Function, FunctionId, Generator,
		Identifier, IfThenElse, IntegerLiteral, Item, ItemId, Marker, Model, OptType, RecordAccess,
		StringLiteral, TupleAccess,
	},
	ty::{FunctionType, Ty, TyData, TyParamInstantiations},
	utils::DebugPrint,
};

struct SpecialisedFunction<Dst> {
	original: FunctionId,
	ty_vars: TyParamInstantiations,
	parameters: FxHashMap<DeclarationId, DeclarationId<Dst>>,
}

struct TypeSpecialiser<Dst> {
	specialised_model: Model<Dst>,
	replacement_map: ReplacementMap<Dst>,
	concrete: FxHashMap<(FunctionId, FunctionType), FunctionId<Dst>>,
	specialised: Vec<(FunctionId<Dst>, SpecialisedFunction<Dst>)>,
	todo: Vec<SpecialisedFunction<Dst>>,
	ids: Arc<IdentifierRegistry>,
	position: FxHashMap<FunctionId, ItemId<Dst>>,
}

impl<Dst: Marker> Folder<'_, Dst> for TypeSpecialiser<Dst> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.specialised_model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst> {
		&mut self.replacement_map
	}

	fn add_model(&mut self, db: &dyn Thir, model: &Model) {
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
		while let Some((f, s)) = self.specialised.pop() {
			if let Some(b) = model[s.original].body() {
				self.todo.push(s);
				let body = self.fold_expression(db, model, b);
				self.todo.pop();
				self.specialised_model[f].set_body(body);
				continue;
			}
			if model[s.original].name() == self.ids.show {
				// Create specialised show function for types which will be erased, except show on direct enum which will be generated later
				let p = self.specialised_model[f].parameter(0);
				let ty = self.specialised_model[p].ty();
				if ty.contains_erased_type(db.upcast()) {
					if !ty.is_enum(db.upcast()) {
						let body = self.generate_show(db, model, p, ty);
						self.specialised_model[f].set_body(body);
						log::debug!(
							"Generated specialised show\n{}",
							PrettyPrinter::new(db, &self.specialised_model)
								.pretty_print_item(f.into())
						);
					}
					continue;
				}
			}
		}

		assert!(self.todo.is_empty());
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		if model[f].is_polymorphic() {
			if let Some(last) = self.specialised_model.all_items().last() {
				self.position.insert(f, last);
			}
		}
		if !model[f].is_polymorphic() || model[f].body().is_none() {
			// Remove non-builtin polymorphic functions
			add_function(self, db, model, f);
		}
	}

	fn fold_declaration_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		d: DeclarationId,
	) -> DeclarationId<Dst> {
		if let Some(sf) = self.todo.last() {
			if let Some(result) = sf.parameters.get(&d) {
				// Map to specialised parameter
				return *result;
			}
		}
		fold_declaration_id(self, db, model, d)
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model, call: &Call) -> Call<Dst> {
		if let Callable::Function(f) = &call.function {
			if self.needs_specialisation(db, model, *f, call.arguments.iter().map(|arg| arg.ty())) {
				let arguments = call
					.arguments
					.iter()
					.map(|arg| self.fold_expression(db, model, arg))
					.collect::<Vec<_>>();
				return Call {
					function: Callable::Function(self.instantiate(
						db,
						model,
						*f,
						&arguments.iter().map(|e| e.ty()).collect::<Vec<_>>(),
					)),
					arguments,
				};
			}
		}
		fold_call(self, db, model, call)
	}

	fn fold_domain(&mut self, db: &dyn Thir, model: &Model, domain: &Domain) -> Domain<Dst> {
		if let Some(s) = self.todo.last() {
			// Instantiate type-inst vars in param/return types
			if let DomainData::Unbounded = &**domain {
				return Domain::unbounded(
					db,
					domain.origin(),
					domain.ty().instantiate_ty_vars(db.upcast(), &s.ty_vars),
				);
			}
		}
		fold_domain(self, db, model, domain)
	}
}

impl<Dst: Marker> TypeSpecialiser<Dst> {
	fn needs_specialisation(
		&self,
		db: &dyn Thir,
		model: &Model,
		f: FunctionId,
		args: impl IntoIterator<Item = Ty>,
	) -> bool {
		model[f].is_polymorphic()
			&& (model[f].body().is_some()
				|| (model[f].name() == self.ids.show
					|| model[f].name() == self.ids.show_json
					|| model[f].name() == self.ids.show_dzn)
					&& args
						.into_iter()
						.next()
						.unwrap()
						.contains_erased_type(db.upcast()))
	}

	// Get or create the specialised version of a polymorphic function with the given argument types
	fn instantiate(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		f: FunctionId,
		args: &[Ty],
	) -> FunctionId<Dst> {
		assert!(model[f].is_polymorphic());
		assert!(model[f].top_level());

		let (ty_vars, function_type) = model[f]
			.function_entry(model)
			.overload
			.instantiate_ty_params(db.upcast(), args)
			.unwrap();

		let key = (f, function_type);

		let concrete = self.concrete.get(&key);
		if let Some(concrete) = concrete {
			// Already instantiated this version
			return *concrete;
		}

		log::debug!(
			"Instantiating {} with {}",
			PrettyPrinter::new(db, model).pretty_print_signature(f.into()),
			ty_vars
				.iter()
				.map(|(tv, ty)| format!(
					"{} = {}",
					tv.pretty_print(db.upcast()),
					ty.pretty_print(db.upcast())
				))
				.collect::<Vec<_>>()
				.join(", ")
		);

		let fn_match = model
			.lookup_function(db, model[f].name(), args)
			.unwrap_or_else(|e| panic!("{}", e.debug_print(db.upcast())));
		if !fn_match.fn_entry.overload.is_polymorphic() {
			// Already have existing concrete version, no need to create
			return self.fold_function_id(db, model, fn_match.function);
		}

		// Create specialised version of polymorphic function
		self.todo.push(SpecialisedFunction {
			original: f,
			ty_vars,
			parameters: FxHashMap::default(),
		});
		let mut function = Function::new(
			model[f].name(),
			self.fold_domain(db, model, model[f].domain()),
		);
		function.set_specialised(true);
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
		let idx = if let Some(p) = self.position.get(&f) {
			self.specialised_model
				.add_function_after(Item::new(function, model[f].origin()), *p)
		} else {
			self.specialised_model
				.prepend_function(Item::new(function, model[f].origin()))
		};
		self.concrete.insert(key, idx);
		self.specialised.push((idx, specialised));
		log::debug!(
			"Created {}",
			PrettyPrinter::new(db, &self.specialised_model).pretty_print_signature(idx.into())
		);
		idx
	}

	fn expr(
		&self,
		db: &dyn Thir,
		origin: impl Into<Origin>,
		e: impl ExpressionBuilder<Dst>,
	) -> Expression<Dst> {
		Expression::new(db, &self.specialised_model, origin, e)
	}

	// Generate specialised body for show needed if type will be erased
	fn generate_show(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		arg: DeclarationId<Dst>,
		ty: Ty,
	) -> Expression<Dst> {
		let origin = self.specialised_model[arg].origin();
		let call = |ts: &mut Self, name: Identifier, args: Vec<Expression<Dst>>| {
			let lookup = model
				.lookup_function(
					db,
					name.into(),
					&args.iter().map(|arg| arg.ty()).collect::<Vec<_>>(),
				)
				.unwrap();
			let idx = if ts.needs_specialisation(
				db,
				model,
				lookup.function,
				args.iter().map(|arg| arg.ty()),
			) {
				ts.instantiate(
					db,
					model,
					lookup.function,
					&args.iter().map(|arg| arg.ty()).collect::<Vec<_>>(),
				)
			} else {
				ts.fold_function_id(db, model, lookup.function)
			};
			Call {
				function: Callable::Function(idx),
				arguments: args,
			}
		};

		if let Some(OptType::Opt) = ty.opt(db.upcast()) {
			// if occurs(x) then show(deopt(x)) else "<>" endif
			let occurs = call(self, self.ids.occurs, vec![self.expr(db, origin, arg)]);
			let deopt = call(self, self.ids.deopt, vec![self.expr(db, origin, arg)]);
			let show = call(self, self.ids.show, vec![self.expr(db, origin, deopt)]);
			return self.expr(
				db,
				origin,
				IfThenElse {
					branches: vec![Branch {
						condition: self.expr(db, origin, occurs),
						result: self.expr(db, origin, show),
					}],
					else_result: Box::new(self.expr(
						db,
						origin,
						StringLiteral::new("<>", db.upcast()),
					)),
				},
			);
		}

		match ty.lookup(db.upcast()) {
			TyData::Array { element, .. } => {
				// concat(["[", join(", ", [show(x_i) | x_i in x]), "]"])
				let gen = Declaration::new(false, Domain::unbounded(db, origin, element));
				let x_i = self
					.specialised_model
					.add_declaration(Item::new(gen, origin));
				let show = call(self, self.ids.show, vec![self.expr(db, origin, x_i)]);
				let join = call(
					self,
					self.ids.join,
					vec![
						self.expr(db, origin, StringLiteral::new(", ", db.upcast())),
						self.expr(
							db,
							origin,
							ArrayComprehension {
								generators: vec![Generator::Iterator {
									declarations: vec![x_i],
									collection: self.expr(db, origin, arg),
									where_clause: None,
								}],
								indices: None,
								template: Box::new(self.expr(db, origin, show)),
							},
						),
					],
				);
				let concat = call(
					self,
					self.ids.concat,
					vec![self.expr(
						db,
						origin,
						ArrayLiteral(vec![
							self.expr(db, origin, StringLiteral::new("[", db.upcast())),
							self.expr(db, origin, join),
							self.expr(db, origin, StringLiteral::new("]", db.upcast())),
						]),
					)],
				);
				self.expr(db, origin, concat)
			}
			TyData::Set(_, _, element) => {
				// concat(["{", join(", ", [show(x_i) | x_i in x]), "}"])
				let gen = Declaration::new(false, Domain::unbounded(db, origin, element));
				let x_i = self
					.specialised_model
					.add_declaration(Item::new(gen, origin));
				let show = call(self, self.ids.show, vec![self.expr(db, origin, x_i)]);
				let join = call(
					self,
					self.ids.join,
					vec![
						self.expr(db, origin, StringLiteral::new(", ", db.upcast())),
						self.expr(
							db,
							origin,
							ArrayComprehension {
								generators: vec![Generator::Iterator {
									declarations: vec![x_i],
									collection: self.expr(db, origin, arg),
									where_clause: None,
								}],
								indices: None,
								template: Box::new(self.expr(db, origin, show)),
							},
						),
					],
				);
				let concat = call(
					self,
					self.ids.concat,
					vec![self.expr(
						db,
						origin,
						ArrayLiteral(vec![
							self.expr(db, origin, StringLiteral::new("{", db.upcast())),
							self.expr(db, origin, join),
							self.expr(db, origin, StringLiteral::new("}", db.upcast())),
						]),
					)],
				);
				self.expr(db, origin, concat)
			}
			TyData::Tuple(_, fs) => {
				// concat(["(", show(x.1), ", ", show(x.2), ")"])
				let mut fields = Vec::with_capacity(2 * fs.len() + 1);
				fields.push(self.expr(db, origin, StringLiteral::new("(", db.upcast())));
				for i in 1..=fs.len() {
					if i > 1 {
						fields.push(self.expr(db, origin, StringLiteral::new(", ", db.upcast())));
					}
					let show = call(
						self,
						self.ids.show,
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
				fields.push(self.expr(db, origin, StringLiteral::new(")", db.upcast())));
				let concat = call(
					self,
					self.ids.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				);
				self.expr(db, origin, concat)
			}
			TyData::Record(_, fs) => {
				// concat(["(", "foo", ": ", show(x.foo), ", ", "bar", ": ", show(x.bar), ")"])
				let mut fields = Vec::with_capacity(fs.len() * 4 + 1);
				fields.push(self.expr(db, origin, StringLiteral::new("(", db.upcast())));
				let mut first = true;
				for (i, _) in fs.iter() {
					if first {
						first = false;
					} else {
						fields.push(self.expr(db, origin, StringLiteral::new(", ", db.upcast())));
					}
					fields.push(self.expr(db, origin, StringLiteral::from(*i)));
					fields.push(self.expr(db, origin, StringLiteral::new(": ", db.upcast())));
					let show = call(
						self,
						self.ids.show,
						vec![self.expr(
							db,
							origin,
							RecordAccess {
								record: Box::new(self.expr(db, origin, arg)),
								field: Identifier(*i),
							},
						)],
					);
					fields.push(self.expr(db, origin, show));
				}
				fields.push(self.expr(db, origin, StringLiteral::new(")", db.upcast())));
				let concat = call(
					self,
					self.ids.concat,
					vec![self.expr(db, origin, ArrayLiteral(fields))],
				);
				self.expr(db, origin, concat)
			}
			_ => unreachable!(),
		}
	}
}

/// Type specialise a model
pub fn type_specialise(db: &dyn Thir, model: &Model) -> Model {
	let ids = db.identifier_registry();
	let mut ts = TypeSpecialiser {
		replacement_map: ReplacementMap::default(),
		specialised_model: Model::default(),
		concrete: FxHashMap::default(),
		specialised: Vec::new(),
		todo: Vec::new(),
		ids,
		position: FxHashMap::default(),
	};
	ts.add_model(db, model);
	ts.specialised_model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::{name_mangle::mangle_names, test::check_no_stdlib, transformer};

	use super::type_specialise;

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
			test occurs(opt $T: x);
			function $T: deopt(opt $T: x);
			function string: show(any $T: x);
			function string: show(array [$X] of any $T: x);
			function string: concat(array [$T] of string: x);
			function string: join(string: s, array [$T] of string: x);
			output [show((a: 1, b: 2))];
			array [int] of tuple(opt int, bool): x;
			output [show(x)];
			"#,
			expect!([r#"
    function bool: occurs(opt $T: x);
    function $T: deopt(opt $T: x);
    function string: 'show<opt int>'(opt int: x) = if occurs(x) then show(deopt(x)) else "<>" endif;
    function string: 'show<tuple(opt int, bool)>'(tuple(opt int, bool): x) = concat(["(", 'show<opt int>'(x.1), ", ", show(x.2), ")"]);
    function string: 'show<record(int: a, int: b)>'(record(int: a, int: b): x) = concat(["(", "a", ": ", show(x.a), ", ", "b", ": ", show(x.b), ")"]);
    function string: show(any $T: x);
    function string: 'show<array [int] of tuple(opt int, bool)>'(array [int] of tuple(opt int, bool): x) = concat(["[", join(", ", ['show<tuple(opt int, bool)>'(_DECL_11) | _DECL_11 in x]), "]"]);
    function string: show(array [$X] of any $T: x);
    function string: concat(array [$T] of string: x);
    function string: join(string: s, array [$T] of string: x);
    output ['show<record(int: a, int: b)>'((a: 1, b: 2))];
    array [int] of tuple(opt int, bool): x;
    output ['show<array [int] of tuple(opt int, bool)>'(x)];
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
}
