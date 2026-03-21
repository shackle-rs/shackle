use crate::{
	value::{define_var_ref, DataView},
	Value,
};

#[derive(Debug)]
pub struct FloatVar {
	ident: u64,
	_domain: Option<Value>,
	alias: Option<Value>,
}

impl PartialEq for FloatVar {
	fn eq(&self, other: &Self) -> bool {
		self.ident == other.ident
			|| if let Some(alias) = self.alias.as_ref() {
				*alias.as_float_var() == *other
			} else {
				false
			}
	}
}
impl Eq for FloatVar {}

define_var_ref!(FloatVar, FloatVarRef);

impl Value {
	pub(crate) fn as_float_var(&self) -> FloatVarRef {
		let DataView::FloatVar(fv) = self.deref() else {
			panic!("Expected float var, found {:?}", self)
		};
		fv
	}
}
