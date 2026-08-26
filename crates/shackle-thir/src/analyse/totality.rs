//! Determines if a function is total
//!

use rustc_hash::FxHashMap;
use shackle_hir::constants::IdentifierRegistry;
use shackle_ty::registry::TypeRegistry;
use shackle_utils::{arena::ArenaMap, maybe_grow_stack};

use super::ModeAnalysis;
use crate::{
	ArrayComprehension, Call, Callable, ConstraintId, Db, Expression, FunctionId, FunctionItem,
	IfThenElse, Model,
	traverse::{
		Visitor, visit_array_comprehension, visit_call, visit_callable, visit_constraint,
		visit_expression, visit_if_then_else,
	},
};

/// Totality of a function
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Totality {
	/// Function is total
	Total,
	/// Function is partial but partiality is par
	ParPartial,
	/// Function is partial and partiality is var
	VarPartial,
}

/// Totality of a function and whether it needs a root version to be generated for it
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TotalityResult {
	/// THe totality of the function (determines the return type of the function)
	pub totality: Totality,
	/// Whether a root version of this function should be generated
	pub needs_root: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Dependency {
	in_var_ite: bool,
	in_bool_ctx: bool,
}

/// Get the totality of the functions in this model
pub fn analyse_totality<'db>(
	db: &'db dyn Db,
	model: &Model<'db>,
	modes: &ModeAnalysis<'_, 'db>,
) -> ArenaMap<FunctionItem<'db>, TotalityResult> {
	let ids = IdentifierRegistry::lookup(db);
	let tys = TypeRegistry::lookup(db);
	let mut todo = Vec::new();
	let mut result = ArenaMap::with_capacity(model.functions_len());
	let mut reverse_dependencies: FxHashMap<FunctionId, FxHashMap<FunctionId, Dependency>> =
		FxHashMap::default();
	for (idx, f) in model.all_functions() {
		result.insert(
			idx,
			TotalityResult {
				totality: Totality::Total,
				needs_root: false,
			},
		);
		let already_total = f.annotations().has(model, ids.annotations.promise_total)
			|| f.body().is_none()
			|| f.return_type() == tys.ann;
		if !already_total && let Some(body) = f.body() {
			let mut v = TotalityVisitor {
				db,
				modes,
				ids: IdentifierRegistry::lookup(db),
				tys: TypeRegistry::lookup(db),
				dependencies: FxHashMap::default(),
				totality: Totality::Total,
				needs_root: false,
				in_var_ite: false,
				in_bool_ctx: f.return_type() == tys.par_bool || f.return_type() == tys.var_bool,
			};
			v.visit_expression(model, body);
			if v.totality < Totality::VarPartial {
				for (f, dep) in v.dependencies {
					// Totality of this function needs to be updated when the totality of these dependencies change
					let _ = reverse_dependencies.entry(f).or_default().insert(idx, dep);
				}
			}
			if v.needs_root {
				result[idx].totality = v.totality;
				result[idx].needs_root = true;
				// Trigger updates of functions which depend on this
				todo.push((idx, v.totality));
			}
		}
	}

	while let Some((idx, mut totality)) = todo.pop() {
		if let Some(deps) = reverse_dependencies.get(&idx) {
			for (f, dep) in deps.iter() {
				let mut needs_update = false;
				if !dep.in_bool_ctx {
					if dep.in_var_ite && totality == Totality::ParPartial {
						totality = Totality::VarPartial;
					}
					if result[*f].totality < totality {
						result[*f].totality = totality;
						needs_update = true;
					}
				}
				if !result[*f].needs_root {
					result[*f].needs_root = true;
					needs_update = true;
				}
				if needs_update {
					todo.push((*f, totality));
				}
			}
		}
	}
	result
}

struct TotalityVisitor<'a, 'db> {
	db: &'db dyn Db,
	modes: &'a ModeAnalysis<'a, 'db>,
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
	dependencies: FxHashMap<FunctionId<'db>, Dependency>,
	totality: Totality,
	needs_root: bool,
	in_var_ite: bool,
	in_bool_ctx: bool,
}

impl<'a, 'db> Visitor<'a, 'db> for TotalityVisitor<'a, 'db> {
	fn visit_constraint(&mut self, model: &'a Model<'db>, c: ConstraintId<'db>) {
		if self.modes.get(model[c].expression()).is_root() {
			// If the constraint in the non-root version of the function is in root context,
			// this must actually be a statically true constraint
			return;
		}
		if model[c].expression().ty() == self.tys.var_bool {
			self.needs_root = true;
			if !self.in_bool_ctx {
				self.totality = Totality::VarPartial;
			}
		} else if self.totality == Totality::Total {
			self.needs_root = true;
			if !self.in_bool_ctx {
				self.totality = Totality::ParPartial;
			}
		}
		visit_constraint(self, model, c);
	}

	fn visit_array_comprehension(&mut self, model: &'a Model<'db>, c: &'a ArrayComprehension<'db>) {
		let var_condition = c.generators.iter().any(|g| {
			g.var_where(self.db)
				|| g.declarations().any(|d| {
					model[d]
						.annotations()
						.has(model, self.ids.annotations.mzn_var_where_clause)
				})
		});
		let prev_var_ite = self.in_var_ite;
		if var_condition {
			self.in_var_ite = true;
		}
		visit_array_comprehension(self, model, c);
		self.in_var_ite = prev_var_ite;
		if self.totality == Totality::ParPartial && var_condition {
			self.needs_root = true;
			if !self.in_bool_ctx {
				self.totality = Totality::VarPartial;
			}
		}
	}

	fn visit_if_then_else(&mut self, model: &'a Model<'db>, ite: &'a IfThenElse<'db>) {
		let var_condition = ite.has_var_condition(self.db);
		let prev_var_ite = self.in_var_ite;
		if var_condition {
			self.in_var_ite = true;
		}
		visit_if_then_else(self, model, ite);
		self.in_var_ite = prev_var_ite;
		if self.totality == Totality::ParPartial && var_condition {
			self.needs_root = true;
			if !self.in_bool_ctx {
				self.totality = Totality::VarPartial;
			}
		}
	}

	fn visit_callable(&mut self, model: &'a Model<'db>, callable: &'a Callable<'db>) {
		if let Callable::Function(f) = callable
			&& !self.in_bool_ctx
		{
			// This function is partial if the called function is partial
			let _ = self.dependencies.insert(
				*f,
				Dependency {
					in_var_ite: self.in_var_ite,
					in_bool_ctx: self.in_bool_ctx,
				},
			);
		}
		visit_callable(self, model, callable);
	}

	fn visit_call(&mut self, model: &'a Model<'db>, call: &'a Call<'db>) {
		if call.matches_builtin(model, self.ids.functions.mzn_default_partial)
			&& call.arguments.len() == 2
		{
			// Totality only depends on RHS
			self.visit_expression(model, &call.arguments[1]);
			return;
		}
		visit_call(self, model, call);
	}

	fn visit_expression(&mut self, model: &'a Model<'db>, expression: &'a Expression<'db>) {
		if self.totality < Totality::VarPartial {
			let is_boolean =
				expression.ty() == self.tys.par_bool || expression.ty() == self.tys.var_bool;
			if !is_boolean || self.modes.get_in_root_fn(expression) != self.modes.get(expression) {
				let prev_in_bool_ctx = self.in_bool_ctx;
				self.in_bool_ctx |= is_boolean;
				// If an expression is boolean, we still allow it to affect the totality result if
				// its context is different in the root version of the function as this indicates
				// that it would benefit from having a separate root version generated
				maybe_grow_stack(|| {
					visit_expression(self, model, expression);
				});
				self.in_bool_ctx = prev_in_bool_ctx;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use super::{Totality, analyse_totality};
	use crate::{analyse::ModeAnalysis, lower::lower_model, pretty_print::PrettyPrinter};

	fn check_totality(program: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, program.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = lower_model(&db).take();
		let modes = ModeAnalysis::analyse(&db, &model);
		let result = analyse_totality(&db, &model, &modes);
		let printer = PrettyPrinter::new(&db, &model);
		let mut pretty = String::new();
		for (f, _) in model.top_level_functions() {
			pretty.push_str(&printer.pretty_print_signature(f.into()));
			match result[f].totality {
				Totality::Total => pretty.push_str(" :: total"),
				Totality::ParPartial => pretty.push_str(" :: par_partial"),
				Totality::VarPartial => pretty.push_str(" :: var_partial"),
			}
			if result[f].needs_root {
				pretty.push_str(" :: needs_root");
			}
			pretty.push_str(";\n");
		}
		expected.assert_eq(&pretty);
	}

	#[test]
	fn test_totality_analysis_basic() {
		check_totality(
			r#"
            function int: foo(int: x) = x;
            function int: bar(int: x) = let {
                constraint false;
            } in 1;
            test qux(int: x) = let {
                constraint false;
            } in true;
            "#,
			expect![[r#"
    function int: foo(int: x) :: total;
    function int: bar(int: x) :: par_partial :: needs_root;
    function bool: qux(int: x) :: total :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_calls() {
		check_totality(
			r#"
            function int: foo(int: x) = bar(x);
            function int: bar(int: x) = let {
                constraint false;
            } in 1;
            "#,
			expect![[r#"
    function int: foo(int: x) :: par_partial :: needs_root;
    function int: bar(int: x) :: par_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_recursive() {
		check_totality(
			r#"
            function int: foo(int: x) = bar(x);
            function int: bar(int: x) = foo(x);

            function int: f(int: x) = let {
                any: a = g(x);
                constraint false;
            } in 1;
            function int: g(int: x) = let {
                any: a = f(x);
            } in 1;
            "#,
			expect![[r#"
    function int: foo(int: x) :: total;
    function int: bar(int: x) :: total;
    function int: f(int: x) :: par_partial :: needs_root;
    function int: g(int: x) :: par_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_comprehension() {
		check_totality(
			r#"
            function set of int: foo() = let {
				var bool: b;
				constraint b;
			} in {1, 3, 5};
			function array [int] of int: bar() = [1 | i in foo()];
            "#,
			expect![[r#"
    function set of int: foo() :: var_partial :: needs_root;
    function array [int] of int: bar() :: var_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_ite() {
		check_totality(
			r#"
            function int: foo(bool: b) =
				if b then
					let { constraint false; } in 1
				else
					2
				endif;
            function var int: bar(var bool: b) =
				if b then
					let { constraint false; } in 1
				else
					2
				endif;
            "#,
			expect![[r#"
    function int: foo(bool: b) :: par_partial :: needs_root;
    function var int: bar(var bool: b) :: var_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_ite_call() {
		check_totality(
			r#"
			function int: foo() = let {
				constraint false;
			} in 1;
            function var int: bar(var bool: b) =
				if b then
					foo()
				else
					2
				endif;
            "#,
			expect![[r#"
    function int: foo() :: par_partial :: needs_root;
    function var int: bar(var bool: b) :: var_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_abort() {
		check_totality(
			r#"
		    test: mzn_abort(string: msg);
			test bar(int: x);
			function int: foo(int: x) = let {
				constraint if bar(x) then true else mzn_abort("foo") endif;
			} in x;
            "#,
			expect![[r#"
    function bool: mzn_abort(string: msg) :: total;
    function bool: bar(int: x) :: total;
    function int: foo(int: x) :: par_partial :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_analysis_comprehension_bool() {
		check_totality(
			r#"
			function array [int] of bool: foo() =
				[let { constraint false; } in true | i in {1}]
            "#,
			expect![[r#"
    function array [int] of bool: foo() :: total :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_bool_coerce() {
		check_totality(
			r#"
				predicate bar(int: x);
				function var float: foo() = bar(
					let {
						constraint false;
					} in 1
				);
            "#,
			expect![[r#"
    predicate bar(int: x) :: total;
    function var float: foo() :: total :: needs_root;
"#]],
		);
	}

	#[test]
	fn test_totality_mzn_in_root_context() {
		check_totality(
			r#"
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
						foo_t(x).1
					endif;
				function tuple(var bool): foo_t(var int: x) :: promise_total =
					let {
						var bool: b;
						constraint bar_reif(x, b);
					} in (b,);
            "#,
			expect![[r#"
    function bool: mzn_in_root_context() :: total;
    predicate bar(var int: x) :: total;
    predicate bar_reif(var int: x, var bool: b) :: total;
    function var bool: foo(var int: x) :: total :: needs_root;
    function tuple(var bool): foo_t(var int: x) :: (promise_total) :: total;
"#]],
		);
	}
}
