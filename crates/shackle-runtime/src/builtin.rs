use crate::{
	value::{DataView, SetView},
	Value,
};

pub fn int_set_union(a: Value, b: Value) -> Value {
	let DataView::IntSet(x) = a.deref() else {
		panic!("expected value to be an integer set literal")
	};
	let DataView::IntSet(y) = b.deref() else {
		panic!("expected value to be an integer set literal")
	};
	x.union(&y)
}
