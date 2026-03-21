use crate::{
	value::{define_var_ref, DataView},
	Value,
};

#[derive(Debug)]
pub struct IntVar {
	ident: u64,
	_domain: Option<Value>,
	alias: Option<Value>,
}

impl PartialEq for IntVar {
	fn eq(&self, other: &Self) -> bool {
		self.ident == other.ident
			|| if let Some(alias) = self.alias.as_ref() {
				*alias.as_int_var() == *other
			} else {
				false
			}
	}
}
impl Eq for IntVar {}

define_var_ref!(IntVar, IntVarRef);

impl Value {
	pub(crate) fn as_int_var(&self) -> IntVarRef {
		let DataView::IntVar(iv) = self.deref() else {
			panic!("Expected int var, found {:?}", self)
		};
		iv
	}
}
