//! HIR representation of types written in a model (not computed types).
//!

use crate::arena::ArenaIndex;

use super::{Expression, Pattern};

pub use crate::syntax::ast::{OptType, PrimitiveType, VarType};

/// Type of an expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
	/// Primitive unbounded type
	Primitive {
		/// Inst
		inst: VarType,
		/// Optionality
		opt: OptType,
		/// The base type
		primitive_type: PrimitiveType,
	},
	/// Bounded type or type-inst alias
	Bounded {
		/// Inst
		inst: Option<VarType>,
		/// Optionality
		opt: Option<OptType>,
		/// The domain
		domain: ArenaIndex<Expression>,
	},
	/// Array type
	Array {
		/// Optionality
		opt: OptType,
		/// Type of dimensions
		dimensions: ArenaIndex<Type>,
		/// Type of element
		element: ArenaIndex<Type>,
	},
	/// Set type
	Set {
		/// Inst
		inst: VarType,
		/// Optionality
		opt: OptType,
		/// Type of element
		element: ArenaIndex<Type>,
	},
	/// Tuple type
	Tuple {
		/// Optionality
		opt: OptType,
		/// Tuple field types
		fields: Box<[ArenaIndex<Type>]>,
	},
	/// Record type
	Record {
		/// Optionality
		opt: OptType,
		/// Record field types
		fields: Box<[(ArenaIndex<Pattern>, ArenaIndex<Type>)]>,
	},
	/// Operation (function) type
	Operation {
		/// Optionality
		opt: OptType,
		/// Return type
		return_type: ArenaIndex<Type>,
		/// Parameter types
		parameter_types: Box<[ArenaIndex<Type>]>,
	},
	/// Type inferred from RHS
	Any,

	/// Sentinel indicating an error during lowering
	Missing,
}
