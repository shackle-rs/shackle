//! Miscellaneous utilities

/// Implement `From<T>` for an enum `X` which has a variant `X::V(T)`.
macro_rules! impl_enum_from {
	($enum:ident::$type:ident) => {
		impl_enum_from!($enum::$type($type));
	};
	($enum:ident::$variant:ident($type:ty)) => {
		impl std::convert::From<$type> for $enum {
			fn from(v: $type) -> Self {
				Self::$variant(v)
			}
		}
	};
}

pub(crate) use impl_enum_from;
