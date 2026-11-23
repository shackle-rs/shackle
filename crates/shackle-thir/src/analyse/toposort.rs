//! Topological sorting of top-level items
//!
//! Although HIR is toposorted, we need to sort the THIR again after transforms to ensure the MIR gets a valid order.
//! Only sorts declarations and constraints

use shackle_utils::hash::Set;

use crate::{
	Callable, ItemId, Marker, Model, ResolvedIdentifier,
	traverse::{Visitor, visit_item},
};

/// Topologically sorted items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopoSorted<'db, T: Marker> {
	/// Top-level items in order
	pub items: Vec<ItemId<'db, T>>,
}

impl<'db, T: Marker> TopoSorted<'db, T> {
	/// Get the topologically sorted items
	pub fn get(model: &Model<'db, T>) -> Self {
		let mut sorter = TopoSorter::default();
		sorter.visit_model(model);
		assert_eq!(sorter.items.len(), model.top_level_items().count());
		Self {
			items: sorter.items,
		}
	}
}

#[derive(Debug, Default)]
struct TopoSorter<'db, T: Marker> {
	items: Vec<ItemId<'db, T>>,
	seen: Set<ItemId<'db, T>>,
}

impl<'db, T: Marker> Visitor<'_, 'db, T> for TopoSorter<'db, T> {
	fn visit_model(&mut self, model: &Model<'db, T>) {
		for item in model.top_level_items().collect::<Vec<_>>() {
			if self.seen.insert(item) {
				visit_item(self, model, item);
				self.items.push(item);
			}
		}
	}

	fn visit_identifier(&mut self, model: &Model<'db, T>, identifier: &ResolvedIdentifier<'db, T>) {
		if let ResolvedIdentifier::Declaration(d) = identifier {
			let decl = &model[*d];
			if self.seen.insert((*d).into()) {
				self.visit_declaration(model, *d);
				if decl.top_level() {
					self.items.push((*d).into());
				}
			}
		}
	}

	fn visit_callable(&mut self, model: &Model<'db, T>, callable: &Callable<'db, T>) {
		if let Callable::Function(f) = callable
			&& self.seen.insert((*f).into())
		{
			self.visit_function(model, *f);
			self.items.push((*f).into());
		}
	}
}

#[cfg(test)]
mod tests {
	use std::fmt::Write;

	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use super::*;
	use crate::{
		lower::lower_model, pretty_print::PrettyPrinter,
		transform::domain_constraint::rewrite_domains,
	};

	fn check(program: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, program.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = rewrite_domains(&db, lower_model(&db).take()).unwrap();
		let sorted = TopoSorted::get(&model);
		let mut actual = String::new();
		let printer = PrettyPrinter::new(&db, &model);
		for item in sorted.items.iter() {
			writeln!(&mut actual, "{};", printer.pretty_print_item(*item)).unwrap();
		}
		expected.assert_eq(&actual);
	}

	#[test]
	fn test_toposoft_domain_vars() {
		check(
			r#"
			test mzn_domain_constraint(string, int, set of int);
            test bar() = foo(10);
            constraint bar();
            test foo({1, n}: x) = true;
            int: n = 2;
        "#,
			expect![[r#"
    function bool: mzn_domain_constraint(string: _DECL_1, int: _DECL_2, set of int: _DECL_3);
    int: n = 2;
    set of int: _DECL_5 = {1, n};
    function bool: foo(int: x) = let {
      constraint mzn_domain_constraint("x", x, _DECL_5);
    } in true;
    function bool: bar() = foo(10);
    constraint bar();
"#]],
		)
	}
}
