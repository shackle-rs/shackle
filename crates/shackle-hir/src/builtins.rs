//! Utility for finding builtin functions

use crate::{Db, FunctionItem, Item, toposort::topological_sort};

/// Get the builtin functions in the program
#[salsa::tracked]
pub fn builtins<'db>(db: &'db dyn Db) -> Vec<FunctionItem<'db>> {
	topological_sort(db)
		.iter()
		.filter_map(|i| {
			let Item::Function(f) = i else {
				return None;
			};
			if f.function(db).body.is_some() {
				None
			} else {
				Some(*f)
			}
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use expect_test::expect;
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase,
		builtins::builtins,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};

	#[test]
	fn test_list_builtins() {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let code = r#"
            function int: foo(int: x);
            function bool: bar(string: x);
            function var int: qux(var int: x) = x;
        "#;
		let model = InlineModelFile::new(&db, code.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		let actual = builtins(&db)
			.iter()
			.map(|builtin| builtin.pretty_print_signature(&db))
			.collect::<Vec<_>>()
			.join("\n");
		expect![[r#"
    function int: foo(int: x)
    test bar(string: x)"#]]
		.assert_eq(&actual);
	}
}
