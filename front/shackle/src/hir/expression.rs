//! HIR representation of expressions.
//!
//! See also the `container` and `primitive` modules.

use std::fmt;

use crate::{arena::ArenaIndex, utils::impl_enum_from};

use super::{
	db::{Hir, HirString, HirStringData},
	ArrayAccess, ArrayComprehension, ArrayLiteral, BoolLiteral, Constraint, Declaration,
	FloatLiteral, IntegerLiteral, Pattern, RecordLiteral, SetComprehension, SetLiteral,
	StringLiteral, TupleLiteral,
};

/// An expression
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Expression {
	/// Integer literal
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	FloatLiteral(FloatLiteral),
	/// Set literal
	SetLiteral(SetLiteral),
	/// Bool literal
	BoolLiteral(BoolLiteral),
	/// String literal
	StringLiteral(StringLiteral),
	/// Identifier
	Identifier(Identifier),
	/// Absent `<>`
	Absent,
	/// Infinity
	Infinity,
	/// Anonymous variable `_`
	Anonymous,
	/// Tuple literal
	TupleLiteral(TupleLiteral),
	/// Record literal
	RecordLiteral(RecordLiteral),
	/// Array literal
	ArrayLiteral(ArrayLiteral),
	/// Array access
	ArrayAccess(ArrayAccess),
	/// Array comprehension
	ArrayComprehension(ArrayComprehension),
	/// Set comprehension
	SetComprehension(SetComprehension),
	/// If-then-else
	IfThenElse(IfThenElse),
	/// Function call
	Call(Call),
	/// Case expression
	Case(Case),
	/// Let expression
	Let(Let),
	/// Tuple access
	TupleAccess(TupleAccess),
	/// Record access
	RecordAccess(RecordAccess),

	/// Sentinel for errors during lowering
	Missing,
}

impl fmt::Debug for Expression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Expression::IntegerLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::FloatLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::SetLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::BoolLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::StringLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::Identifier(x) => fmt::Debug::fmt(x, f),
			Expression::Absent => f.write_str("Absent"),
			Expression::Infinity => f.write_str("Infinity"),
			Expression::Anonymous => f.write_str("Anonymous"),
			Expression::TupleLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::RecordLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayAccess(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayComprehension(x) => fmt::Debug::fmt(x, f),
			Expression::SetComprehension(x) => fmt::Debug::fmt(x, f),
			Expression::IfThenElse(x) => fmt::Debug::fmt(x, f),
			Expression::Call(x) => fmt::Debug::fmt(x, f),
			Expression::Case(x) => fmt::Debug::fmt(x, f),
			Expression::Let(x) => fmt::Debug::fmt(x, f),
			Expression::TupleAccess(x) => fmt::Debug::fmt(x, f),
			Expression::RecordAccess(x) => fmt::Debug::fmt(x, f),
			Expression::Missing => f.write_str("Missing"),
		}
	}
}

impl_enum_from!(Expression::IntegerLiteral);
impl_enum_from!(Expression::FloatLiteral);
impl_enum_from!(Expression::SetLiteral);
impl_enum_from!(Expression::BoolLiteral);
impl_enum_from!(Expression::StringLiteral);
impl_enum_from!(Expression::Identifier);
impl_enum_from!(Expression::ArrayLiteral);
impl_enum_from!(Expression::ArrayAccess);
impl_enum_from!(Expression::ArrayComprehension);
impl_enum_from!(Expression::SetComprehension);
impl_enum_from!(Expression::IfThenElse);
impl_enum_from!(Expression::Call);
impl_enum_from!(Expression::Case);
impl_enum_from!(Expression::Let);
impl_enum_from!(Expression::TupleLiteral);
impl_enum_from!(Expression::RecordLiteral);
impl_enum_from!(Expression::TupleAccess);
impl_enum_from!(Expression::RecordAccess);

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
}

/// Anonymous variable `_`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Anonymous;

/// If-then-else
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IfThenElse {
	/// The if-then and elseif-then branches
	pub branches: Box<[Branch]>,
	/// The else result
	pub else_result: Option<ArenaIndex<Expression>>,
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Branch {
	/// The boolean condition
	pub condition: ArenaIndex<Expression>,
	/// The result if the condition holds
	pub result: ArenaIndex<Expression>,
}

/// Function call
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call {
	/// Function being called
	pub function: ArenaIndex<Expression>,
	/// Call arguments
	pub arguments: Box<[ArenaIndex<Expression>]>,
}

/// Case expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Case {
	/// Expression being matched
	pub expression: ArenaIndex<Expression>,
	/// Cases being matched
	pub cases: Box<[CaseItem]>,
}

/// Case item
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CaseItem {
	/// Pattern being matched
	pub pattern: ArenaIndex<Pattern>,
	/// Value if matched
	pub value: ArenaIndex<Expression>,
}

/// Let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Let {
	/// Items in this let expression
	pub items: Box<[LetItem]>,
	/// Value of the let expression
	pub in_expression: ArenaIndex<Expression>,
}

/// Item in a let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum LetItem {
	/// A declaration
	Declaration(Declaration),
	/// A constraint
	Constraint(Constraint),
}

impl_enum_from!(LetItem::Declaration);
impl_enum_from!(LetItem::Constraint);

/// Tuple access expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TupleAccess {
	/// Tuple being accessed
	pub tuple: ArenaIndex<Expression>,
	/// Field being accessed
	pub field: IntegerLiteral,
}

/// Record access expression
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RecordAccess {
	/// Record being accessed
	pub record: ArenaIndex<Expression>,
	/// Field being accessed
	pub field: Identifier,
}
