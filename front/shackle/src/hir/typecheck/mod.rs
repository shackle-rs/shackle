//! Typing and name resolution of MiniZinc programs.
//!
//! Performs
//! - Name resolution
//! - Overloading resolution
//! - Field access resolution
//! - Computation of types of items and expressions
//! - Type correctness check

use std::{fmt::Write, ops::Index, sync::Arc};

use crate::{
	arena::{ArenaIndex, ArenaMap},
	utils::{debug_print_strings, DebugPrint},
	Error,
};

use super::{
	db::Hir,
	ids::{ExpressionRef, ItemRef, LocalItemRef, PatternRef},
	Expression, FunctionEntry, FunctionType, Pattern, Ty, TyVar, TypeRegistry,
};

mod body;
mod signature;
mod typer;

pub use self::body::*;
pub use self::signature::*;
pub use self::typer::*;

/// Collected types for an item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeResult {
	item: ItemRef,
	body: Arc<BodyTypes>,
	signature: Option<Arc<SignatureTypes>>,
}

impl TypeResult {
	/// Get the computed types for this item
	pub fn new(db: &dyn Hir, item: ItemRef) -> Self {
		let it = item.local_item_ref(db);
		match it {
			LocalItemRef::Assignment(_)
			| LocalItemRef::Constraint(_)
			| LocalItemRef::Output(_)
			| LocalItemRef::Solve(_) => TypeResult {
				item,
				body: db.lookup_item_body(item),
				signature: None,
			},
			_ => TypeResult {
				item,
				body: db.lookup_item_body(item),
				signature: Some(db.lookup_item_signature(item)),
			},
		}
	}

	/// Get the pattern this identifier expression resolves to
	pub fn name_resolution(&self, index: ArenaIndex<Expression>) -> Option<PatternRef> {
		if let Some(t) = self.body.identifier_resolution.get(&index) {
			return Some(*t);
		}
		if let Some(b) = &self.signature {
			if let Some(t) = b
				.identifier_resolution
				.get(&ExpressionRef::new(self.item, index))
			{
				return Some(*t);
			}
		}
		None
	}
}

impl Index<ArenaIndex<Pattern>> for TypeResult {
	type Output = DeclarationType;
	fn index(&self, index: ArenaIndex<Pattern>) -> &Self::Output {
		if let Some(d) = self.body.patterns.get(index) {
			return d;
		}
		if let Some(b) = &self.signature {
			if let Some(d) = b.patterns.get(&PatternRef::new(self.item, index)) {
				return d;
			}
		}
		unreachable!("No declaration for pattern {:?}", index)
	}
}

impl Index<ArenaIndex<Expression>> for TypeResult {
	type Output = Ty;
	fn index(&self, index: ArenaIndex<Expression>) -> &Self::Output {
		if let Some(t) = self.body.expressions.get(index) {
			return t;
		}
		if let Some(b) = &self.signature {
			if let Some(t) = b.expressions.get(&ExpressionRef::new(self.item, index)) {
				return t;
			}
		}
		unreachable!("No declaration for pattern {:?}", index)
	}
}

impl<'a> DebugPrint<'a> for TypeResult {
	type Database = dyn Hir + 'a;
	fn debug_print(&self, db: &Self::Database) -> String {
		let mut w = String::new();
		writeln!(&mut w, "Computed types:").unwrap();
		writeln!(&mut w, "  Declarations:").unwrap();
		for (i, t) in self
			.body
			.patterns
			.iter()
			.map(|(p, d)| (p, d))
			.chain(self.signature.iter().flat_map(|ts| {
				ts.patterns.iter().filter_map(|(p, d)| {
					if p.item() == self.item {
						Some((p.pattern(), d))
					} else {
						None
					}
				})
			}))
			.collect::<ArenaMap<_, _>>()
			.into_iter()
		{
			writeln!(&mut w, "    {:?}: {}", i, t.debug_print(db)).unwrap();
		}
		writeln!(&mut w, "  Expressions:").unwrap();
		for (i, e) in self
			.body
			.expressions
			.iter()
			.chain(self.signature.iter().flat_map(|ts| {
				ts.expressions.iter().filter_map(|(e, t)| {
					if e.item() == self.item {
						Some((e.expression(), t))
					} else {
						None
					}
				})
			}))
			.collect::<ArenaMap<_, _>>()
			.into_iter()
		{
			writeln!(&mut w, "    {:?}: {}", i, e.pretty_print(db)).unwrap();
		}
		writeln!(&mut w, "  Name resolution:").unwrap();
		for (i, p) in self
			.body
			.identifier_resolution
			.iter()
			.map(|(e, t)| (*e, t))
			.chain(self.signature.iter().flat_map(|ts| {
				ts.identifier_resolution.iter().filter_map(|(k, v)| {
					if k.item() == self.item {
						Some((k.expression(), v))
					} else {
						None
					}
				})
			}))
			.collect::<ArenaMap<_, _>>()
			.into_iter()
		{
			writeln!(&mut w, "    {:?}: {:?}", i, p).unwrap();
		}
		debug_print_strings(db, &w)
	}
}

/// Diagnostics collected when typing an item
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeDiagnostics(Arc<Vec<Error>>, Option<Arc<Vec<Error>>>);

impl TypeDiagnostics {
	/// Get the diagnostics for type-checking an item
	pub fn new(db: &dyn Hir, item: ItemRef) -> Self {
		let it = item.local_item_ref(db);
		match it {
			LocalItemRef::Assignment(_)
			| LocalItemRef::Constraint(_)
			| LocalItemRef::Output(_)
			| LocalItemRef::Solve(_) => TypeDiagnostics(db.lookup_item_body_diagnostics(item), None),
			_ => TypeDiagnostics(
				db.lookup_item_signature_diagnostics(item),
				Some(db.lookup_item_body_diagnostics(item)),
			),
		}
	}

	/// Iterate over the diagnostics
	pub fn iter(&self) -> impl Iterator<Item = &Error> {
		self.0.iter().chain(self.1.iter().flat_map(|es| es.iter()))
	}
}

/// Context for computation of types
pub trait TypeContext {
	/// Add a declaration for a pattern
	fn add_declaration(&mut self, pattern: PatternRef, declaration: DeclarationType);
	/// Add a type for an expression
	fn add_expression(&mut self, expression: ExpressionRef, ty: Ty);
	/// Add identifier resolution
	fn add_identifier_resolution(&mut self, expression: ExpressionRef, resolution: PatternRef);
	/// Add pattern resolution
	fn add_pattern_resolution(&mut self, pattern: PatternRef, resolution: PatternRef);
	/// Add an error
	fn add_diagnostic(&mut self, item: ItemRef, e: impl Into<Error>);

	/// Type a pattern (or lookup the type if already known)
	fn type_pattern(
		&mut self,
		db: &dyn Hir,
		types: &TypeRegistry,
		pattern: PatternRef,
	) -> DeclarationType;
}

/// Get the signature of an item (ignores RHS of items except for `any` declarations)
pub fn collect_item_signature(
	db: &dyn Hir,
	item: ItemRef,
) -> (Arc<SignatureTypes>, Arc<Vec<Error>>) {
	let types = TypeRegistry::new(db);
	let mut ctx = SignatureTypeContext::new(item);
	ctx.type_item(db, &types, item);
	let (s, e) = ctx.finish();
	(Arc::new(s), Arc::new(e))
}

/// Type-check expressions in an item (other than those used in the signature)
pub fn collect_item_body(db: &dyn Hir, item: ItemRef) -> (Arc<BodyTypes>, Arc<Vec<Error>>) {
	let types = TypeRegistry::new(db);
	let mut ctx = BodyTypeContext::new(item);
	ctx.type_item(db, &types);
	let (s, e) = ctx.finish();
	(Arc::new(s), Arc::new(e))
}

/// Type of a declaration
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeclarationType {
	/// Pattern is a variable declaration
	Variable(Ty),
	/// Pattern is a function (with a flag indicating if the return type is known yet)
	Function(Box<FunctionEntry>),
	/// Pattern is a type-inst variable
	TyVar(TyVar),
	/// Pattern is a type-inst alias
	TypeAlias(Ty),
	/// Enum constructor
	EnumConstructor(Box<[FunctionType]>),
	/// Enum atom
	EnumAtom(Ty),
	/// Currently computing
	Computing,
}

impl<'a> DebugPrint<'a> for DeclarationType {
	type Database = dyn Hir + 'a;

	fn debug_print(&self, db: &Self::Database) -> String {
		match self {
			DeclarationType::Variable(ty) => format!("Variable({})", ty.pretty_print(db)),
			DeclarationType::Function(function) => {
				format!("Function({})", function.overload.pretty_print(db))
			}
			DeclarationType::TyVar(t) => format!("TyVar({})", t.ty_var.name(db)),
			DeclarationType::TypeAlias(ty) => format!("TypeAlias({})", ty.pretty_print(db)),
			DeclarationType::EnumConstructor(fs) => {
				format!(
					"EnumConstructor({})",
					fs.iter()
						.map(|f| f.pretty_print(db))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			DeclarationType::EnumAtom(ty) => format!("EnumAtom({})", ty.pretty_print(db)),
			DeclarationType::Computing => "{computing}".to_owned(),
		}
	}
}
