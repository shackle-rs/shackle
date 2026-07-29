//! Miscellaneous utilities

use salsa::Database as Db;
pub use shackle_utils_derive::TypedIndex;

pub mod arena;
pub mod hash;
pub mod refmap;

/// Get levenshtein distance between two strings
pub fn levenshtein_distance(s: &str, t: &str) -> usize {
	let n = t.len();
	let mut dp0 = (0..=n).collect::<Vec<_>>();
	let mut dp1 = vec![0_usize; n + 1];
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

/// Grow the stack if necessary to run the given function.
///
/// Useful for recursive calls which may overrun the stack otherwise.
#[inline]
pub fn maybe_grow_stack<R>(f: impl FnOnce() -> R) -> R {
	stacker::maybe_grow(64 * 1024, 1024 * 1024, f)
}

#[allow(
	missing_docs,
	clippy::missing_docs_in_private_items,
	reason = "Salsa generates code with missing docs"
)]
mod interned_string {
	use std::fmt::Display;

	use crate::Db;

	/// An interned string
	#[salsa::interned(debug)]
	pub struct InternedString {
		value: String,
	}

	impl<'db> InternedString<'db> {
		/// Get the string value
		pub fn lookup(&self, db: &'db dyn Db) -> &'db str {
			self.value(db)
		}
	}

	impl<'db> Display for InternedString<'db> {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			salsa::with_attached_database(|db| self.value(db).fmt(f))
				.unwrap_or_else(|| write!(f, "<interned string>"))
		}
	}
}

pub use interned_string::InternedString;
