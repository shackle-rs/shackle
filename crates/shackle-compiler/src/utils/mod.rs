//! Miscellaneous utilities

pub mod arena;
pub mod refmap;

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
	($enum:ident<$($ls:lifetime,)* $($ts:ident $(: $fb:path)?),+>::$type:ident) => {
		impl_enum_from!($enum<$($ts $(: $fb)?),+>::$type($type));
	};
	($enum:ident<$($ls:lifetime,)* $($ts:ident $(: $fb:path)?),+>::$variant:ident($type:ty)) => {
		impl<$($ls,)* $($ts $(: $fb)?),+> std::convert::From<$type> for $enum<$($ls,)* $($ts),+> {
			fn from(v: $type) -> Self {
				Self::$variant(v)
			}
		}
	};
}

macro_rules! impl_index {
	($type:ident[$self:ident, $index:ident: $index_type:ty] -> $output:ty {$value:expr}) => {
		impl std::ops::Index<$index_type> for $type {
			type Output = $output;
			fn index(&$self, $index: $index_type) -> &Self::Output {
				&$value
			}
		}

		impl std::ops::IndexMut<$index_type> for $type {
			fn index_mut(&mut $self, $index: $index_type) -> &mut Self::Output {
				&mut $value
			}
		}
	};

	($type:ident<$($tp:ident $(: $fb:path)?),+>[$self:ident, $index:ident: $index_type:ty] -> $output:ty {$value:expr}) => {
		impl<$($tp $(: $fb)?),+> std::ops::Index<$index_type> for $type<$($tp),+> {
			type Output = $output;
			fn index(&$self, $index: $index_type) -> &Self::Output {
				&$value
			}
		}

		impl<$($tp $(: $fb)?),+> std::ops::IndexMut<$index_type> for $type<$($tp),+> {
			fn index_mut(&mut $self, $index: $index_type) -> &mut Self::Output {
				&mut $value
			}
		}
	};
}

use std::fmt::Write;

pub(crate) use impl_enum_from;
pub(crate) use impl_index;
use salsa::InternKey;

use crate::{
	db::InternedString,
	hir::db::Hir,
	syntax::minizinc::{parse_float_literal, parse_integer_literal},
};

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

/// Pretty print an identifier.
///
/// Either returns the string as is, if it is already a valid identifier,
/// otherwise, encloses it in quotes.
///
/// Panics if the given name contains a quote.
pub fn pretty_print_identifier(name: &str) -> String {
	assert!(
		!name.contains('\''),
		"Identifier names cannot contain single quotes"
	);
	if matches!(
		name,
		"ann"
			| "annotation"
			| "any" | "array"
			| "bool" | "case"
			| "constraint"
			| "default" | "diff"
			| "div" | "else"
			| "elseif" | "endif"
			| "enum" | "false"
			| "float" | "function"
			| "if" | "in"
			| "include" | "int"
			| "intersect"
			| "let" | "list"
			| "maximize" | "minimize"
			| "mod" | "not"
			| "of" | "op"
			| "opt" | "output"
			| "par" | "predicate"
			| "record" | "satisfy"
			| "set" | "solve"
			| "string" | "subset"
			| "superset" | "symdiff"
			| "test" | "then"
			| "true" | "tuple"
			| "type" | "union"
			| "var" | "where"
			| "xor"
	) {
		// Identifiers which are keywords need quoting
		return format!("'{}'", name);
	}

	for c in name.chars() {
		if matches!(
			c,
			'"' | '\''
				| '.' | '-' | '['
				| ']' | '^' | ','
				| ';' | ':' | '('
				| ')' | '{' | '}'
				| '&' | '|' | '$'
				| '∞' | '%' | '<'
				| '>' | '⟷' | '⇔'
				| '→' | '⇒' | '←'
				| '⇐' | '/' | '∨'
				| '⊻' | '∧' | '='
				| '!' | '≠' | '≤'
				| '≥' | '∈' | '⊆'
				| '⊇' | '∪' | '∩'
				| '+' | '*' | '~'
		) || c.is_whitespace()
		{
			// Operators in identifiers need quoting
			return format!("'{}'", name);
		}
	}

	if parse_integer_literal(name).is_ok() || parse_float_literal(name).is_ok() {
		// Identifiers which are numeric literals need quoting
		return format!("'{}'", name);
	}

	name.to_owned()
}

/// Grow the stack if necessary to run the given function.
///
/// Useful for recursive calls which may overrun the stack otherwise.
#[inline]
pub fn maybe_grow_stack<R>(f: impl FnOnce() -> R) -> R {
	stacker::maybe_grow(64 * 1024, 1024 * 1024, f)
}

#[cfg(test)]
mod test {
	use super::pretty_print_identifier;

	#[test]
	fn pretty_print_ident() {
		assert_eq!(pretty_print_identifier("x"), "x");
		assert_eq!(pretty_print_identifier("-"), "'-'");
		assert_eq!(pretty_print_identifier("a b"), "'a b'");
		assert_eq!(pretty_print_identifier("😃"), "😃");
		assert_eq!(pretty_print_identifier("123"), "'123'");
		assert_eq!(pretty_print_identifier("1E24"), "'1E24'");
	}
}
