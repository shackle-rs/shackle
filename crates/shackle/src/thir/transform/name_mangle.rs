//! Perform name mangling on overloaded/specialised functions

use rustc_hash::FxHashMap;

use crate::thir::{db::Thir, FunctionId, FunctionName, Model};

/// Mangle names of overloaded/specialised functions
pub fn mangle_names(db: &dyn Thir, model: &Model) -> Model {
	let mut model = model.clone();
	let mut overloaded: FxHashMap<_, Vec<FunctionId>> = FxHashMap::default();
	for (idx, function) in model.top_level_functions() {
		if function.body().is_some() {
			if let FunctionName::Named(ident) = function.name() {
				overloaded.entry(ident).or_default().push(idx);
			}
		}
	}
	for (name, functions) in overloaded {
		for function in functions {
			let mangled = FunctionName::Named(name).mangled(
				db,
				model[function].parameters().iter().map(|p| model[*p].ty()),
			);
			model[function].set_name(mangled);
		}
	}
	model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check_no_stdlib;

	use super::mangle_names;

	#[test]
	fn test_name_mangling() {
		check_no_stdlib(
			mangle_names,
			r#"
                function int: builtin(int: x);
                function int: builtin(string: x);
                function int: foo(int: x) = 1;
                function int: bar(int: x) = 1;
                function int: bar(string: x) = 1;
            "#,
			expect!([r#"
    function int: builtin(int: x);
    function int: builtin(string: x);
    function int: 'foo<int>'(int: x) = 1;
    function int: 'bar<int>'(int: x) = 1;
    function int: 'bar<string>'(string: x) = 1;
    solve satisfy;
"#]),
		);
	}
}
