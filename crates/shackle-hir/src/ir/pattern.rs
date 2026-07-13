//! Destructuring/case matching patterns
//!
use derive_more::{From, Into};
use shackle_syntax::minizinc::pretty_print_identifier;
use shackle_utils::{InternedString, arena::ArenaIndex};

use super::{BooleanLiteral, FloatLiteral, IntegerLiteral, ItemData, StringLiteral};
use crate::Db;

/// The local ID of a pattern (used to index into the containing item)
pub type PatternId<'db> = ArenaIndex<Pattern<'db>>;

/// A pattern for destructuring
#[derive(Clone, Debug, From, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub enum Pattern<'db> {
	/// A single identifier
	#[from]
	Identifier(Identifier<'db>),
	/// Don't care wildcard
	Anonymous,
	/// Absent literal
	Absent,
	/// Boolean literal
	#[from]
	Boolean(BooleanLiteral),
	/// Float literal
	Float {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: FloatLiteral,
	},
	/// Integer literal
	Integer {
		/// Whether this has been negated
		negated: bool,
		/// The literal value
		value: IntegerLiteral,
	},
	/// Infinity
	Infinity {
		/// Whether this has been negated
		negated: bool,
	},
	/// String literal
	#[from]
	String(StringLiteral<'db>),
	/// Enum constructor pattern
	Call {
		/// Callee identifier
		function: PatternId<'db>,
		/// Call arguments
		arguments: Box<[PatternId<'db>]>,
	},
	/// Tuple pattern
	Tuple {
		/// Tuple fields
		fields: Box<[PatternId<'db>]>,
	},
	/// Record pattern
	Record {
		/// Record fields (pairs of field name, field value pattern)
		fields: Box<[(Identifier<'db>, PatternId<'db>)]>,
	},
	/// Indicates an error
	Missing,
}

impl<'db> Pattern<'db> {
	/// Get the identifier if this is one
	pub fn identifier(&self) -> Option<Identifier<'db>> {
		match *self {
			Pattern::Identifier(i) => Some(i),
			_ => None,
		}
	}

	/// Get the identifiers in this pattern
	pub fn identifiers<'a>(
		pattern: PatternId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = PatternId<'db>> {
		let mut todo = vec![pattern];
		std::iter::from_fn(move || {
			while let Some(p) = todo.pop() {
				match &data[p] {
					Pattern::Identifier(_) => return Some(p),
					Pattern::Call { arguments, .. } => todo.extend(arguments.iter().copied()),
					Pattern::Tuple { fields } => todo.extend(fields.iter().copied()),
					Pattern::Record { fields } => todo.extend(fields.iter().map(|(_, p)| *p)),
					_ => (),
				}
			}
			None
		})
	}

	/// Get this pattern as an integer value if it is one
	pub fn integer_value(&self) -> Option<i64> {
		match self {
			Pattern::Integer { negated, value } => Some(if *negated { -value.0 } else { value.0 }),
			_ => None,
		}
	}

	/// Get whether this pattern is a leaf (contains no sub-patterns)
	pub fn is_leaf(&self) -> bool {
		match self {
			Pattern::Identifier(_)
			| Pattern::Anonymous
			| Pattern::Absent
			| Pattern::Boolean(_)
			| Pattern::Float { .. }
			| Pattern::Integer { .. }
			| Pattern::Infinity { .. }
			| Pattern::String(_)
			| Pattern::Missing => true,
			Pattern::Call { .. } => false,
			Pattern::Tuple { fields } => fields.is_empty(),
			Pattern::Record { fields } => fields.is_empty(),
		}
	}

	/// Get whether this pattern can only possibly match a single value
	/// (i.e. no identifiers, no wildcards)
	pub fn is_singular(pattern: PatternId<'db>, data: &ItemData) -> bool {
		let mut todo = vec![pattern];
		while let Some(p) = todo.pop() {
			match &data[p] {
				Pattern::Identifier(_) | Pattern::Anonymous => return false,
				Pattern::Call { arguments, .. } => todo.extend(arguments.iter().copied()),
				Pattern::Tuple { fields } => todo.extend(fields.iter().copied()),
				Pattern::Record { fields } => todo.extend(fields.iter().map(|(_, p)| *p)),
				_ => (),
			}
		}
		true
	}

	/// Get whether this pattern is refutable (i.e. may not always match)
	pub fn is_refutable(pattern: PatternId<'db>, data: &ItemData) -> bool {
		let mut todo = vec![pattern];
		while let Some(p) = todo.pop() {
			match &data[p] {
				Pattern::Identifier(_) | Pattern::Anonymous => (),
				Pattern::Tuple { fields } => todo.extend(fields.iter().copied()),
				Pattern::Record { fields } => todo.extend(fields.iter().map(|(_, p)| *p)),
				_ => return true,
			}
		}
		false
	}

	/// True if this pattern is missing (i.e. an error)
	pub fn is_missing(&self) -> bool {
		matches!(self, Pattern::Missing)
	}
}

/// Identifier
#[derive(Copy, Clone, From, Into, Hash, PartialEq, Eq, salsa::SalsaValue)]
pub struct Identifier<'db>(pub InternedString<'db>);

impl<'db> Identifier<'db> {
	/// Create a new identifier with the given value
	pub fn new<T: AsRef<str>>(db: &'db dyn Db, v: T) -> Self {
		Self(InternedString::new(db, v.as_ref()))
	}

	/// Get the name of this identifier
	pub fn lookup(&self, db: &'db dyn Db) -> &'db str {
		self.0.lookup(db)
	}

	/// Append ⁻¹ to this identifier
	pub fn inversed(&self, db: &'db dyn Db) -> Self {
		let mut v = self.lookup(db).to_owned();
		v.push_str("⁻¹");
		Self::new(db, v)
	}

	/// Get this identifier but with `_root` appended
	pub fn root(&self, db: &'db dyn Db) -> Self {
		let mut v = self.lookup(db).to_owned();
		if v.ends_with("_root") {
			return *self;
		}
		v.push_str("_root");
		Self::new(db, v)
	}

	/// Whether or not this identifier ends with `_root`
	pub fn is_root(&self, db: &'db dyn Db) -> bool {
		self.lookup(db).ends_with("_root")
	}

	/// Get this identifier but with `_reif` appended
	pub fn reif(&self, db: &'db dyn Db) -> Self {
		let mut v = self.lookup(db).to_owned();
		if v.ends_with("_reif") {
			return *self;
		}
		v.push_str("_reif");
		Self::new(db, v)
	}

	/// Whether or not this identifier ends with `_reif`
	pub fn is_reif(&self, db: &'db dyn Db) -> bool {
		self.lookup(db).ends_with("_reif")
	}

	/// Get this identifier but with `_imp` appended
	pub fn imp(&self, db: &'db dyn Db) -> Self {
		let mut v = self.lookup(db).to_owned();
		if v.ends_with("_imp") {
			return *self;
		}
		v.push_str("_imp");
		Self::new(db, v)
	}

	/// Whether or not this identifier ends with `_imp`
	pub fn is_imp(&self, db: &'db dyn Db) -> bool {
		self.lookup(db).ends_with("_imp")
	}

	/// Whether this identifier matches a string
	pub fn is<T: AsRef<str>>(&self, db: &'db dyn Db, v: T) -> bool {
		self.0.lookup(db) == v.as_ref()
	}

	/// Pretty print this identifier (adding quotes if needed)
	pub fn pretty_print(&self, db: &'db dyn Db) -> String {
		let ident = self.lookup(db);
		pretty_print_identifier(ident)
	}
}

impl std::fmt::Debug for Identifier<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Identifier")
			.field(&format!("{}", self.0))
			.finish()
	}
}

impl std::fmt::Display for Identifier<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", pretty_print_identifier(&format!("{}", self.0)))
	}
}
