//! Perform name mangling on overloaded/specialised functions
//!
//! Must be done before type erasure to give correct mangled names.
//!
//! This doesn't actually modify the `name` of a function item, instead we store
//! the value so that we can print with the correct name. This ensures that
//! function matching using the original names still works after this.

use rustc_hash::FxHashMap;

use crate::thir::{db::Thir, FunctionId, FunctionName, Model};

/// Mangle names of overloaded/specialised functions
pub fn mangle_names(_db: &dyn Thir, mut model: Model) -> Model {
	log::info!("Mangling overloaded function names");

	let mut overloaded: FxHashMap<_, Vec<FunctionId>> = FxHashMap::default();
	for (idx, function) in model.top_level_functions() {
		if let FunctionName::Named(ident) = function.name() {
			overloaded.entry(ident).or_default().push(idx);
		}
	}
	for (_, functions) in overloaded {
		if functions.len() > 1 {
			for function in functions {
				let tys = model[function]
					.parameters()
					.iter()
					.map(|p| model[*p].ty())
					.collect();
				model[function].set_mangled_param_tys(tys);
			}
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
				function int: mixed(string: x);
				function int: mixed(int: x) = 1;
				function int: mixed(float: x) = 1;
                function int: foo(int: x) = 1;
                function int: bar(int: x) = 1;
                function int: bar(string: x) = 1;
            "#,
			expect!([r#"
    function int: 'builtin<int>'(int: x);
    function int: 'builtin<string>'(string: x);
    function int: 'mixed<string>'(string: x);
    function int: 'mixed<int>'(int: x) = 1;
    function int: 'mixed<float>'(float: x) = 1;
    function int: foo(int: x) = 1;
    function int: 'bar<int>'(int: x) = 1;
    function int: 'bar<string>'(string: x) = 1;
    solve satisfy;
"#]),
		);
	}
}
