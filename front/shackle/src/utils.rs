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

use std::fmt::Write;

pub(crate) use impl_enum_from;
use salsa::InternKey;

use crate::hir::db::{Hir, HirString};

/// Trait for pretty printing for debugging with a Salsa database
pub trait DebugPrint<'a> {
	/// Type of database (e.g. `dyn Hir`)
	type Database: ?Sized + 'a;
	/// Pretty print to a string
	fn debug_print(&self, db: &Self::Database) -> String;
}

/// Replace debug printed HirStrings with their values
pub fn debug_print_strings(db: &dyn Hir, s: &str) -> String {
	// Replace interned strings with values
	let mut o = String::new();
	for (i, x) in s.split("HirString(").enumerate() {
		if i > 0 {
			if let Some(idx) = x.find(')') {
				let s =
					HirString::from_intern_id((&x[..idx]).parse::<u32>().unwrap().into()).value(db);
				write!(&mut o, "{:?}", s).unwrap();
				o.push_str(&x[idx + 1..]);
			} else {
				o.push_str(x);
			}
		} else {
			o.push_str(x);
		}
	}
	o
}
