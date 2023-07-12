//! Module containing mid-level IR type representation
use std::rc::Rc;

/// A mid-level IR type
#[allow(variant_size_differences)]
pub enum Ty {
	/// Type of bottom
	Bottom {
		/// Dimensions (0 if not an array)
		dim: u8,
		/// Whether this is a set
		is_set: bool,
	},
	/// Type of a boolean
	Bool {
		/// Dimensions (0 if not an array)
		dim: u8,
		/// Whether this is a decision variable
		is_var: bool,
		/// Whether this is a set
		is_set: bool,
	},
	/// Type of an integer
	Int {
		/// Dimensions (0 if not an array)
		dim: u8,
		/// Whether this is a decision variable
		is_var: bool,
		/// Whether this is a set
		is_set: bool,
	},
	/// Type of a float
	Float {
		/// Dimensions (0 if not an array)
		dim: u8,
		/// Whether this is a decision variable
		is_var: bool,
		/// Whether this is a set
		is_set: bool,
	},
	/// Type of a string
	String {
		/// Dimensions (0 if not an array)
		dim: u8,
	},
	/// Type of an annotation
	Ann {
		/// Dimensions (0 if not an array)
		dim: u8,
	},
	/// Type of a tuple
	Tuple {
		/// Dimensions (0 if not an array)
		dim: u8,
		/// Types of the fields
		fields: Rc<[Ty]>,
	},
}
