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
	Expression, FunctionEntry, ItemData, Pattern, Ty, TyData, TyVar, TypeRegistry,
};

mod body;
mod signature;
mod toposort;
mod typer;

pub use self::body::*;
pub use self::signature::*;
pub use self::toposort::*;
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

	/// Get the pattern this pattern (e.g. enum atom/constructor) resolves to
	pub fn pattern_resolution(&self, index: ArenaIndex<Pattern>) -> Option<PatternRef> {
		if let Some(t) = self.body.pattern_resolution.get(&index) {
			return Some(*t);
		}
		if let Some(b) = &self.signature {
			if let Some(t) = b.pattern_resolution.get(&PatternRef::new(self.item, index)) {
				return Some(*t);
			}
		}
		None
	}

	/// Get the declaration for a pattern
	pub fn get_pattern(&self, pattern: ArenaIndex<Pattern>) -> Option<&PatternTy> {
		if let Some(d) = self.body.patterns.get(pattern) {
			return Some(d);
		}
		if let Some(b) = &self.signature {
			if let Some(d) = b.patterns.get(&PatternRef::new(self.item, pattern)) {
				return Some(d);
			}
		}
		None
	}

	/// Get the type of an expression
	pub fn get_expression(&self, expression: ArenaIndex<Expression>) -> Option<Ty> {
		if let Some(t) = self.body.expressions.get(expression) {
			return Some(*t);
		}
		if let Some(b) = &self.signature {
			if let Some(t) = b
				.expressions
				.get(&ExpressionRef::new(self.item, expression))
			{
				return Some(*t);
			}
		}
		None
	}

	/// Pretty print the type of an expression
	pub fn pretty_print_expression_ty(
		&self,
		db: &dyn Hir,
		data: &ItemData,
		expression: ArenaIndex<Expression>,
	) -> Option<String> {
		let ty = self.get_expression(expression)?;
		if let Expression::Identifier(i) = data[expression] {
			if let TyData::Function(opt, function) = ty.lookup(db) {
				// Pretty print functions using item-like syntax if possible
				return Some(
					opt.pretty_print()
						.into_iter()
						.chain([function.pretty_print_item(db, i)])
						.collect::<Vec<_>>()
						.join(" "),
				);
			}
		}
		Some(ty.pretty_print(db))
	}

	/// Pretty print the type of a pattern
	pub fn pretty_print_pattern_ty(
		&self,
		db: &dyn Hir,
		data: &ItemData,
		pattern: ArenaIndex<Pattern>,
	) -> Option<String> {
		let decl = self.get_pattern(pattern)?;
		match decl {
			PatternTy::Variable(ty) | PatternTy::Destructuring(ty) => {
				if let Pattern::Identifier(i) = data[pattern] {
					if let TyData::Function(opt, function) = ty.lookup(db) {
						// Pretty print functions using item-like syntax if possible
						return Some(
							opt.pretty_print()
								.into_iter()
								.chain([function.pretty_print_item(db, i)])
								.collect::<Vec<_>>()
								.join(" "),
						);
					}
					return Some(format!("{}: {}", ty.pretty_print(db), i.lookup(db)));
				}
				Some(ty.pretty_print(db))
			}
			PatternTy::EnumAtom(ty) => Some(format!(
				"{}: {}",
				ty.pretty_print(db),
				data[pattern].identifier()?.lookup(db)
			)),
			PatternTy::Function(f) => Some(
				f.overload
					.pretty_print_item(db, data[pattern].identifier()?),
			),
			PatternTy::EnumConstructor(e) => Some(
				e.first()?
					.overload
					.pretty_print_item(db, data[pattern].identifier()?),
			),
			PatternTy::TyVar(t) => Some(t.ty_var.name(db)),
			PatternTy::TypeAlias(ty) => Some(format!(
				"type {} = {}",
				data[pattern].identifier()?.lookup(db),
				ty.pretty_print(db)
			)),
			_ => None,
		}
	}
}

impl Index<ArenaIndex<Pattern>> for TypeResult {
	type Output = PatternTy;
	fn index(&self, index: ArenaIndex<Pattern>) -> &Self::Output {
		self.get_pattern(index).expect("No declaration for pattern")
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
		unreachable!("No type for expression")
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
		for (i, p) in self
			.body
			.pattern_resolution
			.iter()
			.map(|(e, t)| (*e, t))
			.chain(self.signature.iter().flat_map(|ts| {
				ts.pattern_resolution.iter().filter_map(|(k, v)| {
					if k.item() == self.item {
						Some((k.pattern(), v))
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
	fn add_declaration(&mut self, pattern: PatternRef, declaration: PatternTy);
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
	) -> PatternTy;
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

/// Type of a pattern (usually a declaration)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternTy {
	/// Pattern is a variable declaration
	Variable(Ty),
	/// Pattern is a function (with a flag indicating if the return type is known yet)
	Function(Box<FunctionEntry>),
	/// Pattern is a type-inst variable
	TyVar(TyVar),
	/// Pattern is a type-inst alias
	TypeAlias(Ty),
	/// Enum constructor
	EnumConstructor(Box<[FunctionEntry]>),
	/// Enum atom
	EnumAtom(Ty),
	/// Destructuring pattern
	Destructuring(Ty),
	/// Currently computing
	Computing,
}

impl<'a> DebugPrint<'a> for PatternTy {
	type Database = dyn Hir + 'a;

	fn debug_print(&self, db: &Self::Database) -> String {
		match self {
			PatternTy::Variable(ty) => format!("Variable({})", ty.pretty_print(db)),
			PatternTy::Function(function) => {
				format!("Function({})", function.overload.pretty_print(db))
			}
			PatternTy::TyVar(t) => format!("TyVar({})", t.ty_var.name(db)),
			PatternTy::TypeAlias(ty) => format!("TypeAlias({})", ty.pretty_print(db)),
			PatternTy::EnumConstructor(fs) => {
				format!(
					"EnumConstructor({})",
					fs.iter()
						.map(|f| f.overload.pretty_print(db))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			PatternTy::EnumAtom(ty) => format!("EnumAtom({})", ty.pretty_print(db)),
			PatternTy::Destructuring(ty) => format!("Destructuring({})", ty.pretty_print(db)),
			PatternTy::Computing => "{computing}".to_owned(),
		}
	}
}
