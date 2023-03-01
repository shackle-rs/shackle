//! Type specialisation
//!
//! Creates concrete versions of polymorphic functions.
//! This enables type erasure while ensuring we call the right versions of functions involving e.g. enums.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ty::{Ty, TyParamInstantiations};

use super::{
	add_function, db::Thir, fold_call, fold_declaration_id, fold_domain, fold_function,
	fold_function_body, fold_function_id, visit_call, visit_function, Call, Callable,
	DeclarationId, Domain, DomainData, Folder, Function, FunctionId, LookupCall, Model,
	ReplacementMap, Visitor,
};

/// Collects (possibly nested) concrete instantiations of polymorphic functions.
///
/// Does not collect instantiations of builtin polymorphic functions as these can't be monomorphised,
/// and also doesn't collect instantiations for which an overload has been written by the user
#[derive(Debug, Default)]
struct ConcreteCalls {
	concrete: FxHashSet<Vec<Ty>>,
	children: Vec<(FunctionId, Vec<Ty>)>,
}

struct ConcreteCallCollector<'a> {
	db: &'a dyn Thir,
	polymorphic: FxHashMap<FunctionId, ConcreteCalls>,
	non_polymorphic: FxHashSet<FunctionId>,
	current: Option<Vec<(FunctionId, Vec<Ty>)>>,
}

impl<'a> Visitor for ConcreteCallCollector<'a> {
	fn visit_function(&mut self, model: &Model, function: FunctionId) {
		// Don't enter function bodies, we will enter them manually when handling calls
		visit_function(self, model, function, false);
	}

	fn visit_call(&mut self, model: &Model, call: &Call) {
		if let Callable::Function(f) = &call.function {
			if model[*f].is_polymorphic() {
				if model[*f].body().is_some() {
					let arg_tys = call.arguments.iter().map(|e| e.ty()).collect::<Vec<_>>();
					if !self.polymorphic.contains_key(f) {
						let mut entry = ConcreteCalls::default();
						if let Some(body) = model[*f].body() {
							let prev = self.current.replace(Vec::new());
							self.visit_expression(model, body);
							let c = if let Some(p) = prev {
								self.current.replace(p)
							} else {
								self.current.take()
							}
							.unwrap();
							entry.children = c;
						}
						self.polymorphic.insert(*f, entry);
					}
					if let Some(c) = self.current.as_mut() {
						if arg_tys
							.iter()
							.any(|ty| ty.contains_type_inst_var(self.db.upcast()))
						{
							// Polymorphic call depends on instantiation of parent.
							// This will get handled later.
							c.push((*f, arg_tys));
							visit_call(self, model, call);
							return;
						}
					}
					self.polymorphic
						.get_mut(f)
						.unwrap()
						.concrete
						.insert(arg_tys);
				}
			} else {
				if self.non_polymorphic.insert(*f) {
					if let Some(body) = model[*f].body() {
						let prev = self.current.take();
						self.visit_expression(model, body);
						self.current = prev;
					}
				}
			}
		}
		visit_call(self, model, call);
	}
}

impl<'a> ConcreteCallCollector<'a> {
	/// Get concrete instantiations for reachable polymorphic calls
	pub fn run(
		mut self,
		db: &dyn Thir,
		model: &Model,
	) -> FxHashMap<FunctionId, Vec<SpecialisedFunction>> {
		// Collect calls
		self.visit_model(model);

		// Get concrete instantiations for nested polymorphic calls
		let mut done = false;
		let fns = self.polymorphic.keys().copied().collect::<Vec<_>>();
		while !done {
			done = true;
			for f in fns.iter() {
				let mut e = self.polymorphic.remove(f).unwrap();
				if !e.concrete.is_empty() && !e.children.is_empty() {
					done = false;
					for concrete in e.concrete.iter() {
						let ty_vars = model[*f]
							.function_entry(model)
							.overload
							.instantiate_ty_params(db.upcast(), &concrete)
							.unwrap();
						for (child, args) in e.children.drain(..) {
							if child == *f {
								continue;
							}
							let child_args = args
								.iter()
								.map(|ty| ty.instantiate_ty_vars(db.upcast(), &ty_vars))
								.collect();
							self.polymorphic
								.get_mut(&child)
								.unwrap()
								.concrete
								.insert(child_args);
						}
					}
				}
				self.polymorphic.insert(*f, e);
			}
		}
		self.polymorphic
			.into_iter()
			.filter_map(|(f, e)| {
				let instantiations = e
					.concrete
					.iter()
					.filter_map(|args| {
						let lookup = model.lookup_function(db, model[f].name(), &args).unwrap();
						if lookup.function != f {
							// Already exists
							return None;
						}
						let ty_vars = model[f]
							.function_entry(model)
							.overload
							.instantiate_ty_params(db.upcast(), &args)
							.unwrap();
						Some(SpecialisedFunction::new(ty_vars))
					})
					.collect::<Vec<_>>();
				if instantiations.is_empty() {
					None
				} else {
					Some((f, instantiations))
				}
			})
			.collect()
	}
}

struct SpecialisedFunction {
	specialised_function: Option<FunctionId>,
	ty_vars: TyParamInstantiations,
	param_map: FxHashMap<DeclarationId, DeclarationId>,
}

impl SpecialisedFunction {
	fn new(ty_vars: TyParamInstantiations) -> Self {
		Self {
			specialised_function: None,
			ty_vars,
			param_map: FxHashMap::default(),
		}
	}
}

/// Clones a model, while specialising functions, and re-matching calls.
///
/// - When we encounter a polymorphic function, we create copies of it for each concrete instantiation
/// - Calls are looked up again during this, which redirects them to the new specialised versions
struct Specialiser {
	specialised_model: Model,
	replacement_map: ReplacementMap,
	concrete_calls: FxHashMap<FunctionId, Vec<SpecialisedFunction>>,
	current: Option<(FunctionId, usize)>,
}

impl Folder for Specialiser {
	fn model(&mut self) -> &mut Model {
		&mut self.specialised_model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		if model[f].is_polymorphic() {
			// Create specialised versions of function
			for i in 0..self
				.concrete_calls
				.get(&f)
				.map(|ccs| ccs.len())
				.unwrap_or(0)
			{
				self.current = Some((f, i));
				let idx = add_function(self, db, model, f);
				self.concrete_calls.get_mut(&f).unwrap()[i].specialised_function = Some(idx);
			}
		}
		self.current = None;
		// Fold original function
		add_function(self, db, model, f);
	}

	fn fold_function(&mut self, db: &dyn Thir, model: &Model, f: &Function) -> Function {
		let function = fold_function(self, db, model, f);
		if let Some((fi, i)) = self.current.as_ref() {
			// Record the parameter declaration map so we can substitute in the specialised body
			self.concrete_calls.get_mut(fi).unwrap()[*i]
				.param_map
				.extend(
					f.parameters()
						.iter()
						.copied()
						.zip(function.parameters().iter().copied()),
				);
		}
		function
	}

	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		if model[f].is_polymorphic() {
			// Create specialised bodies
			for i in 0..self
				.concrete_calls
				.get(&f)
				.map(|ccs| ccs.len())
				.unwrap_or(0)
			{
				self.current = Some((f, i));
				fold_function_body(self, db, model, f);
			}
			self.current = None;
		}
		// Fold original body
		fold_function_body(self, db, model, f);
	}

	fn fold_function_id(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) -> FunctionId {
		if let Some((fi, i)) = self.current.as_ref() {
			if f == *fi {
				return self.concrete_calls[fi][*i].specialised_function.unwrap();
			}
		}
		fold_function_id(self, db, model, f)
	}

	fn fold_declaration_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		d: DeclarationId,
	) -> DeclarationId {
		if let Some((f, idx)) = self.current.as_ref() {
			if let Some(result) = self.concrete_calls[f][*idx].param_map.get(&d) {
				// Map to specialised parameter
				return *result;
			}
		}
		fold_declaration_id(self, db, model, d)
	}

	fn fold_call(&mut self, db: &dyn Thir, model: &Model, call: &Call) -> Call {
		if let Callable::Function(f) = &call.function {
			if model[*f].is_polymorphic() {
				// Lookup polymorphic calls again to make them point to new specialised versions (or existing overloads)
				return LookupCall {
					function: model[*f].name(),
					arguments: call
						.arguments
						.iter()
						.map(|arg| self.fold_expression(db, model, arg))
						.collect(),
				}
				.resolve(db, &self.specialised_model)
				.0;
			}
		}
		fold_call(self, db, model, call)
	}

	fn fold_domain(&mut self, db: &dyn Thir, model: &Model, domain: &Domain) -> Domain {
		if let Some((f, i)) = self.current.as_ref() {
			// Instantiate type-inst vars in param/return types
			if let DomainData::Unbounded = &**domain {
				return Domain::unbounded(
					domain.origin(),
					domain
						.ty()
						.instantiate_ty_vars(db.upcast(), &self.concrete_calls[f][*i].ty_vars),
				);
			}
		}
		fold_domain(self, db, model, domain)
	}
}

impl Specialiser {
	/// Type specialise a model
	pub fn run(mut self, db: &dyn Thir, model: &Model) -> Model {
		self.add_model(db, model);
		self.specialised_model
	}
}

/// Type specialise a model
pub fn type_specialise(db: &dyn Thir, model: &Model) -> Model {
	let collector = ConcreteCallCollector {
		db,
		current: None,
		non_polymorphic: FxHashSet::default(),
		polymorphic: FxHashMap::default(),
	};
	let specialiser = Specialiser {
		concrete_calls: collector.run(db, model),
		current: None,
		replacement_map: ReplacementMap::default(),
		specialised_model: Model::default(),
	};
	specialiser.run(db, model)
}

#[cfg(test)]
mod test {
	use std::sync::Arc;

	use expect_test::{expect, Expect};

	use crate::{
		db::{CompilerDatabase, Inputs},
		file::InputFile,
		thir::{db::Thir, pretty_print::PrettyPrinter},
	};

	use super::type_specialise;

	fn check_type_specialisation(source: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		db.set_ignore_stdlib(true);
		db.set_input_files(Arc::new(vec![InputFile::ModelString(source.to_owned())]));
		let model = db.model_thir();
		let result = type_specialise(&db, &model);
		let pretty = PrettyPrinter::new(&db, &result).pretty_print();
		expected.assert_eq(&pretty);
	}

	#[test]
	fn test_type_specialisation_basic_1() {
		check_type_specialisation(
			r#"
					function any $T: foo(any $T: x) = x;
					predicate bar(var bool: p) = foo(p);
					constraint bar(true);
					any: y = foo(10);
					"#,
			expect!([r#"
    function var bool: foo(var bool: x) = x;
    function int: foo(int: x) = x;
    function any $T: foo(any $T: x) = x;
    function var bool: bar(var bool: p) = foo(p);
    constraint bar(true);
    int: y = foo(10);
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_basic_2() {
		check_type_specialisation(
			r#"
			test foo(any $T: x) = true;
			any: a = foo((1, 2));
			any: b = foo((p: 1, q: 2));
			"#,
			expect!([r#"
    function bool: foo(record(int: p, int: q): x) = true;
    function bool: foo(tuple(int, int): x) = true;
    function bool: foo(any $T: x) = true;
    bool: a = foo((1, 2));
    bool: b = foo((p: 1, q: 2));
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_type_specialisation_overloading() {
		check_type_specialisation(
			r#"
			test foo(any $T: x) = bar(x);
			test bar(any $T: x) = true;
			test bar(int: x) = false;
			any: a = foo(1);
			"#,
			expect!([r#"
    function bool: foo(int: x) = bar(x);
    function bool: foo(any $T: x) = bar(x);
    function bool: bar(any $T: x) = true;
    function bool: bar(int: x) = false;
    bool: a = foo(1);
    solve satisfy;
"#]),
		)
	}
}
