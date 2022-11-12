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

use crate::db::InternedString;
use crate::hir::db::Hir;

/// Trait for pretty printing for debugging with a Salsa database
pub trait DebugPrint<'a> {
	/// Type of database (e.g. `dyn Hir`)
	type Database: ?Sized + 'a;
	/// Pretty print to a string
	fn debug_print(&self, db: &Self::Database) -> String;
}

/// Replace debug printed `InternedString`s with their values
pub fn debug_print_strings(db: &dyn Hir, s: &str) -> String {
	// Replace interned strings with values
	let mut o = String::new();
	for (i, x) in s.split("InternedString(").enumerate() {
		if i > 0 {
			if let Some(idx) = x.find(')') {
				let s = InternedString::from_intern_id((x[..idx]).parse::<u32>().unwrap().into())
					.value(db.upcast());
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

/// Get levenshtein distance between two strings
pub fn levenshtein_distance(s: &str, t: &str) -> usize {
	let n = t.len();
	let mut dp0 = (0..=n).collect::<Vec<_>>();
	let mut dp1 = vec![0usize; n + 1];
	for (i, s_i) in s.chars().enumerate() {
		dp1[0] = i + 1;
		for (j, t_j) in t.chars().enumerate() {
			let del = dp0[j + 1] + 1;
			let ins = dp1[j] + 1;
			let sub = if s_i == t_j { dp0[j] } else { dp0[j] + 1 };
			dp1[j + 1] = del.min(ins.min(sub));
		}
		std::mem::swap(&mut dp0, &mut dp1);
	}
	*dp0.last().unwrap()
}
