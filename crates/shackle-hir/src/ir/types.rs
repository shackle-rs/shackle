//! HIR representation of types written in a model (not computed types).
//!
//! See the `typecheck` module for computing types.

pub use shackle_syntax::minizinc::{OptType, PrimitiveType, VarType};
use shackle_utils::arena::ArenaIndex;

use super::ItemData;
use crate::{ExpressionId, PatternId};

/// The local ID of a type (used to index into the containing item)
pub type TypeId<'db> = ArenaIndex<Type<'db>>;

/// Type of an expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum Type<'db> {
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
		domain: ExpressionId<'db>,
	},
	/// Type which introduces new objects of a class
	New {
		/// Inst
		inst: VarType,
		/// Optionality
		opt: OptType,
		/// The class whose objects are introduced
		domain: ExpressionId<'db>,
	},
	/// Array type
	Array {
		/// Optionality
		opt: OptType,
		/// Type of dimensions
		dimensions: TypeId<'db>,
		/// Type of element
		element: TypeId<'db>,
	},
	/// Set type
	Set {
		/// Inst
		inst: VarType,
		/// Optionality
		opt: OptType,
		/// Cardinality of the set, if constrained
		cardinality: Option<ExpressionId<'db>>,
		/// Type of element
		element: TypeId<'db>,
	},
	/// Tuple type
	Tuple {
		/// Optionality
		opt: OptType,
		/// Tuple field types
		fields: Box<[TypeId<'db>]>,
	},
	/// Record type
	Record {
		/// Optionality
		opt: OptType,
		/// Record field types
		fields: Box<[(PatternId<'db>, TypeId<'db>)]>,
	},
	/// Operation (function) type
	Operation {
		/// Optionality
		opt: OptType,
		/// Return type
		return_type: TypeId<'db>,
		/// Parameter types
		parameter_types: Box<[TypeId<'db>]>,
	},
	/// Anonymous type-inst var `_`
	AnonymousTypeInstVar {
		/// Inst to apply
		inst: Option<VarType>,
		/// Optionality to apply
		opt: Option<OptType>,
		/// The pattern for this type-inst var
		pattern: PatternId<'db>,
	},
	/// Type inferred from RHS
	Any,

	/// Sentinel indicating an error during lowering
	Missing,
}

impl<'db> Type<'db> {
	/// Whether or not this type is a leaf type (i.e. contains no other nodes)
	pub fn is_leaf(&self) -> bool {
		match self {
			Type::Primitive { .. } | Type::Any | Type::Missing => true,
			Type::Bounded { .. }
			| Type::New { .. }
			| Type::Array { .. }
			| Type::Set { .. }
			| Type::Operation { .. }
			| Type::AnonymousTypeInstVar { .. } => false,
			Type::Tuple { fields, .. } => fields.is_empty(),
			Type::Record { fields, .. } => fields.is_empty(),
		}
	}

	/// Whether or not this type is completely known.
	///
	/// Returns `false` if the type contains an `Any` and `true` otherwise.
	pub fn is_complete(&self, data: &ItemData) -> bool {
		match self {
			Type::Any => false,
			Type::Primitive { .. }
			| Type::Bounded { .. }
			| Type::New { .. }
			| Type::AnonymousTypeInstVar { .. }
			| Type::Missing => true,
			Type::Array {
				dimensions,
				element,
				..
			} => data[*dimensions].is_complete(data) && data[*element].is_complete(data),
			Type::Set { element, .. } => data[*element].is_complete(data),
			Type::Tuple { fields, .. } => fields.iter().all(|f| data[*f].is_complete(data)),
			Type::Record { fields, .. } => fields.iter().all(|(_, f)| data[*f].is_complete(data)),
			Type::Operation {
				return_type,
				parameter_types,
				..
			} => {
				data[*return_type].is_complete(data)
					&& parameter_types.iter().all(|p| data[*p].is_complete(data))
			}
		}
	}

	/// Return the `any` types in the given type.
	pub fn any_types<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = TypeId<'db>> {
		Type::walk(t, data).filter(|t| matches!(data[*t], Type::Any))
	}

	/// Return the anonymous type-inst variables in the given type.
	pub fn anonymous_ty_vars<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = TypeId<'db>> {
		Type::walk(t, data).filter(|t| matches!(data[*t], Type::AnonymousTypeInstVar { .. }))
	}

	/// Return the operation types in the given type.
	pub fn operations<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = TypeId<'db>> {
		Type::walk(t, data).filter(|t| matches!(data[*t], Type::Operation { .. }))
	}

	/// Get the unbounded primitive types in this type
	pub fn primitives<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = TypeId<'db>> {
		Type::walk(t, data).filter(|t| matches!(data[*t], Type::Primitive { .. }))
	}

	/// Get the expressions (bounds) contained in this type
	pub fn expressions<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = ExpressionId<'db>> {
		Type::walk(t, data).filter_map(|t| {
			if let Type::Bounded { domain, .. } | Type::New { domain, .. } = data[t] {
				Some(domain)
			} else {
				None
			}
		})
	}

	/// Walk over the types contained in this type
	pub fn walk<'a>(
		t: TypeId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = TypeId<'db>> {
		let mut todo = vec![t];
		std::iter::from_fn(move || {
			let t = todo.pop()?;
			match &data[t] {
				Type::Array {
					dimensions,
					element,
					..
				} => {
					todo.push(*dimensions);
					todo.push(*element)
				}
				Type::Set { element, .. } => todo.push(*element),
				Type::Tuple { fields, .. } => todo.extend(fields.iter().copied()),
				Type::Record { fields, .. } => todo.extend(fields.iter().map(|(_, f)| *f)),
				Type::Operation {
					return_type,
					parameter_types,
					..
				} => {
					todo.push(*return_type);
					todo.extend(parameter_types.iter().copied());
				}
				_ => (),
			}
			Some(t)
		})
	}

	/// Whether this type introduces new objects, possibly inside an array or set
	pub fn is_new(&self, data: &ItemData<'db>) -> bool {
		match self {
			Type::New { .. } => true,
			Type::Array { element, .. } | Type::Set { element, .. } => data[*element].is_new(data),
			_ => false,
		}
	}

	/// Get the class whose objects this type introduces, looking through arrays and sets
	pub fn get_new_class(&self, data: &ItemData<'db>) -> Option<ExpressionId<'db>> {
		match self {
			Type::New { domain, .. } => Some(*domain),
			Type::Array { element, .. } | Type::Set { element, .. } => {
				data[*element].get_new_class(data)
			}
			_ => None,
		}
	}
}
