use crate::{
	value::{define_var_ref, DataView},
	Value,
};

#[derive(Debug)]
pub struct IntSetVar {
	ident: u64,
	_domain: Option<Value>,
	_card: Option<Value>,
	alias: Option<Value>,
}

impl PartialEq for IntSetVar {
	fn eq(&self, other: &Self) -> bool {
		self.ident == other.ident
			|| if let Some(alias) = self.alias.as_ref() {
				*alias.as_int_set_var() == *other
			} else {
				false
			}
	}
}
impl Eq for IntSetVar {}

define_var_ref!(IntSetVar, IntSetVarRef);

impl Value {
	pub(crate) fn as_int_set_var(&self) -> IntSetVarRef {
		let DataView::IntSetVar(isv) = self.deref() else {
			panic!("Expected int set var, found {:?}", self)
		};
		isv
	}
}
