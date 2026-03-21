use crate::{
	value::{define_var_ref, DataView},
	Value,
};

#[derive(Debug)]
pub struct BoolVar {
	ident: u64,
	_domain: Option<bool>,
	alias: Option<Value>,
}

impl PartialEq for BoolVar {
	fn eq(&self, other: &Self) -> bool {
		self.ident == other.ident
			|| if let Some(alias) = self.alias.as_ref() {
				*alias.as_bool_var() == *other
			} else {
				false
			}
	}
}
impl Eq for BoolVar {}

define_var_ref!(BoolVar, BoolVarRef);

impl Value {
	pub(crate) fn as_bool_var(&self) -> BoolVarRef {
		let DataView::BoolVar(bv) = self.deref() else {
			panic!("Expected bool var, found {:?}", self)
		};
		bv
	}
}
