use expect_test::expect;

use crate::hir::lower::test::check_lower_item_eprime;

#[test]
fn test_const_definition() {
	check_lower_item_eprime("letting one = 1", expect![[""]])
}
