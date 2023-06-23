use std::rc::Rc;

#[allow(variant_size_differences)]
pub enum Ty {
	Bottom { dim: u8, is_set: bool },
	Bool { dim: u8, is_var: bool, is_set: bool },
	Int { dim: u8, is_var: bool, is_set: bool },
	Float { dim: u8, is_var: bool, is_set: bool },
	String { dim: u8 },
	Ann { dim: u8 },
	Tuple(Rc<[Ty]>),
}
