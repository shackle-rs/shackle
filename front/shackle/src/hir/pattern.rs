//! Destructuring/case matching patterns
//!

use crate::{arena::ArenaIndex, utils::impl_enum_from};

use super::{
	db::{Hir, HirString, HirStringData},
	BooleanLiteral, Expression, FloatLiteral, IntegerLiteral, ItemData, StringLiteral,
};

/// A pattern for destructuring
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Pattern {
	/// A single identifier
	Identifier(Identifier),
	/// Don't care wildcard
	Anonymous,
	/// Absent literal
	Absent,
	/// Boolean literal
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
	String(StringLiteral),
	/// Enum constructor pattern
	Call {
		/// Callee identifier
		function: ArenaIndex<Expression>,
		/// Call arguments
		arguments: Box<[ArenaIndex<Pattern>]>,
	},
	/// Tuple pattern
	Tuple {
		/// Tuple fields
		fields: Box<[ArenaIndex<Pattern>]>,
	},
	/// Record pattern
	Record {
		/// Record fields (pairs of field name, field value pattern)
		fields: Box<[(ArenaIndex<Expression>, ArenaIndex<Pattern>)]>,
	},
	/// Indicates an error
	Missing,
}

impl_enum_from!(Pattern::Identifier);
impl_enum_from!(Pattern::Boolean(BooleanLiteral));
impl_enum_from!(Pattern::String(StringLiteral));

impl Pattern {
	/// Get the identifier if this is one
	pub fn identifier(&self) -> Option<Identifier> {
		match *self {
			Pattern::Identifier(i) => Some(i),
			_ => None,
		}
	}

	/// Get the identifiers in this pattern
	pub fn identifiers(
		pattern: ArenaIndex<Pattern>,
		data: &ItemData,
	) -> impl '_ + Iterator<Item = ArenaIndex<Pattern>> {
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
}

/// Identifier
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Identifier(pub HirString);

impl Identifier {
	/// Create a new identifier with the given value
	pub fn new<T: Into<HirStringData>>(v: T, db: &dyn Hir) -> Self {
		Self(db.intern_string(v.into()))
	}

	/// Get the name of this identifier
	pub fn lookup(&self, db: &dyn Hir) -> String {
		db.lookup_intern_string(self.0).0
	}

	/// Pretty print this identifier (adding quotes if needed)
	///
	/// TODO: Don't quote UTF-8
	pub fn pretty_print(&self, db: &dyn Hir) -> String {
		let ident = self.lookup(db);
		let name = ident.as_str();
		if matches!(
			name,
			"ann"
				| "annotation" | "any"
				| "array" | "bool"
				| "case" | "constraint"
				| "diff" | "div" | "else"
				| "elseif" | "endif"
				| "enum" | "false"
				| "float" | "function"
				| "if" | "in" | "include"
				| "int" | "intersect"
				| "let" | "list" | "maximize"
				| "minimize" | "mod"
				| "not" | "of" | "op"
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
			return format!("'{}'", name);
		}
		for c in name.chars() {
			if matches!(
				c,
				'"' | '\''
					| '.' | '-' | '[' | ']'
					| '^' | ',' | ';' | ':'
					| '(' | ')' | '{' | '}'
					| '&' | '|' | '$' | '∞'
					| '%' | '<' | '>' | '⟷'
					| '⇔' | '→' | '⇒' | '←'
					| '⇐' | '/' | '∨' | '⊻'
					| '∧' | '=' | '!' | '≠'
					| '≤' | '≥' | '∈' | '⊆'
					| '⊇' | '∪' | '∩' | '+'
					| '*',
			) || c.is_whitespace()
			{
				return format!("'{}'", name);
			}
		}
		ident
	}
}
