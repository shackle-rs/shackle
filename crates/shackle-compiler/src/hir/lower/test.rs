use std::sync::Arc;

use expect_test::Expect;

use crate::{
	db::{CompilerDatabase, FileReader, Inputs},
	file::{InputFile, InputLang},
	hir::db::Hir,
	utils::DebugPrint,
};

pub fn check_lower_item_with_lang(language: InputLang, item: &str, expected: Expect) {
	let mut db = CompilerDatabase::default();
	db.set_ignore_stdlib(true);
	db.set_input_files(Arc::new(vec![InputFile::String(item.to_owned(), language)]));
	let model = db.input_models();
	let items = db.lookup_items(model[0]);
	let item = *items.last().unwrap();
	let debug_print = item.debug_print(&db);
	expected.assert_eq(&debug_print);
}

pub fn check_lower_item(item: &str, expected: Expect) {
	check_lower_item_with_lang(InputLang::MiniZinc, item, expected);
}

pub fn check_lower_item_eprime(item: &str, expected: Expect) {
	check_lower_item_with_lang(InputLang::EPrime, item, expected);
}
