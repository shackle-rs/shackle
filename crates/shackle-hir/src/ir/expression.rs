//! HIR representation of expressions.
//!
//! See also the `container` and `primitive` modules.

use std::fmt;

use derive_more::{From, TryUnwrap, Unwrap};
use shackle_utils::arena::ArenaIndex;

use super::{
	ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, BooleanLiteral, Constraint,
	Declaration, FloatLiteral, Generator, Identifier, IndexedArrayLiteral, IntegerLiteral,
	ItemData, MaybeIndexSet, Parameter, RecordLiteral, SetComprehension, SetLiteral, StringLiteral,
	TupleLiteral, Type,
};
use crate::{PatternId, TypeId};

/// The local ID of an expresion (used to index into the containing item)
pub type ExpressionId<'db> = ArenaIndex<Expression<'db>>;

/// An expression
#[derive(Clone, From, Hash, PartialEq, Eq, salsa::Update, Unwrap, TryUnwrap)]
#[unwrap(ref)]
#[try_unwrap(ref)]
pub enum Expression<'db> {
	/// Integer literal
	#[from]
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	#[from]
	FloatLiteral(FloatLiteral),
	/// Set literal
	#[from]
	SetLiteral(SetLiteral<'db>),
	/// Bool literal
	#[from]
	BooleanLiteral(BooleanLiteral),
	/// String literal
	#[from]
	StringLiteral(StringLiteral<'db>),
	/// Identifier
	#[from]
	Identifier(Identifier<'db>),
	/// Absent `<>`
	Absent,
	/// Infinity
	Infinity,
	/// Tuple literal
	#[from]
	TupleLiteral(TupleLiteral<'db>),
	/// Record literal
	#[from]
	RecordLiteral(RecordLiteral<'db>),
	/// Array literal
	#[from]
	ArrayLiteral(ArrayLiteral<'db>),
	/// 2D array literal
	#[from]
	ArrayLiteral2D(ArrayLiteral2D<'db>),
	/// Indexed array literal
	#[from]
	IndexedArrayLiteral(IndexedArrayLiteral<'db>),
	/// Array access
	#[from]
	ArrayAccess(ArrayAccess<'db>),
	/// Array comprehension
	#[from]
	ArrayComprehension(ArrayComprehension<'db>),
	/// Set comprehension
	#[from]
	SetComprehension(SetComprehension<'db>),
	/// If-then-else
	#[from]
	IfThenElse(IfThenElse<'db>),
	/// Function call
	#[from]
	Call(Call<'db>),
	/// Case expression
	#[from]
	Case(Case<'db>),
	/// Let expression
	#[from]
	Let(Let<'db>),
	/// Tuple access
	#[from]
	TupleAccess(TupleAccess<'db>),
	/// Record access
	#[from]
	RecordAccess(RecordAccess<'db>),
	/// Lambda function
	#[from]
	Lambda(Lambda<'db>),
	/// Slice from array access
	Slice(Identifier<'db>),

	/// Sentinel for errors during lowering
	Missing,
}

impl<'db> Expression<'db> {
	/// Whether this is a leaf expression (has no subexpressions)
	pub fn is_leaf(&self) -> bool {
		match self {
			Expression::IntegerLiteral(_)
			| Expression::FloatLiteral(_)
			| Expression::Absent
			| Expression::Identifier(_)
			| Expression::BooleanLiteral(_)
			| Expression::StringLiteral(_)
			| Expression::Infinity
			| Expression::Slice(_)
			| Expression::Missing => true,
			Expression::SetLiteral(sl) => sl.members.is_empty(),
			Expression::TupleLiteral(tl) => tl.fields.is_empty(),
			Expression::RecordLiteral(rl) => rl.fields.is_empty(),
			Expression::ArrayLiteral(al) => al.members.is_empty(),
			Expression::ArrayLiteral2D(al) => {
				al.members.is_empty()
					&& matches!(al.rows, MaybeIndexSet::NonIndexed(_))
					&& matches!(al.columns, MaybeIndexSet::NonIndexed(_))
			}
			Expression::IndexedArrayLiteral(al) => al.members.is_empty() && al.indices.is_empty(),
			Expression::ArrayAccess(_)
			| Expression::ArrayComprehension(_)
			| Expression::SetComprehension(_)
			| Expression::IfThenElse(_)
			| Expression::Call(_)
			| Expression::Case(_)
			| Expression::Let(_)
			| Expression::TupleAccess(_)
			| Expression::RecordAccess(_)
			| Expression::Lambda(_) => false,
		}
	}

	/// Walk over the subexpressions contained in this expression
	pub fn walk<'a>(
		e: ExpressionId<'db>,
		data: &'a ItemData<'db>,
	) -> impl 'a + Iterator<Item = ExpressionId<'db>> {
		let mut todo = vec![e];
		std::iter::from_fn(move || {
			let e = todo.pop()?;
			if let Some(anns) = data.annotations.get(e) {
				todo.extend(anns.iter().copied());
			}
			match &data[e] {
				Expression::Absent
				| Expression::BooleanLiteral(_)
				| Expression::FloatLiteral(_)
				| Expression::Identifier(_)
				| Expression::Infinity
				| Expression::IntegerLiteral(_)
				| Expression::Missing
				| Expression::Slice(_)
				| Expression::StringLiteral(_) => (),
				Expression::ArrayAccess(aa) => {
					todo.push(aa.collection);
					todo.push(aa.indices);
				}
				Expression::ArrayComprehension(c) => {
					for Generator::Iterator {
						collection: v,
						where_clause,
						..
					}
					| Generator::Assignment {
						value: v,
						where_clause,
						..
					} in c.generators.iter()
					{
						todo.push(*v);
						todo.extend(*where_clause);
					}
					todo.extend(c.indices);
					todo.push(c.template);
				}
				Expression::ArrayLiteral(al) => {
					todo.extend(al.members.iter().copied());
				}
				Expression::ArrayLiteral2D(al) => {
					if let MaybeIndexSet::Indexed(s) = &al.rows {
						todo.extend(s.iter().copied());
					}
					if let MaybeIndexSet::Indexed(s) = &al.columns {
						todo.extend(s.iter().copied());
					}
					todo.extend(al.members.iter().copied());
				}
				Expression::IndexedArrayLiteral(al) => {
					todo.extend(al.indices.iter().copied());
					todo.extend(al.members.iter().copied());
				}
				Expression::Call(c) => {
					todo.push(c.function);
					todo.extend(c.arguments.iter().copied());
					todo.extend(c.named_arguments.iter().map(|(_, e)| *e));
				}
				Expression::Case(c) => {
					todo.push(c.expression);
					todo.extend(c.cases.iter().map(|c| c.value));
				}
				Expression::IfThenElse(ite) => {
					todo.extend(ite.branches.iter().flat_map(|b| [b.condition, b.result]));
					todo.extend(ite.else_result);
				}
				Expression::Lambda(l) => {
					for p in l.parameters.iter() {
						todo.extend(p.annotations.iter().copied());
						todo.extend(Type::expressions(p.declared_type, data));
					}
					todo.push(l.body);
				}
				Expression::Let(l) => {
					for i in l.items.iter() {
						match i {
							LetItem::Constraint(c) => {
								todo.extend(c.annotations.iter().copied());
								todo.push(c.expression);
							}
							LetItem::Declaration(d) => {
								todo.extend(Type::expressions(d.declared_type, data));
								todo.extend(d.annotations.iter().copied());
								todo.extend(d.definition);
							}
						}
					}
					todo.push(l.in_expression);
				}
				Expression::RecordAccess(ra) => {
					todo.push(ra.record);
				}
				Expression::RecordLiteral(rl) => {
					todo.extend(rl.fields.iter().map(|(_, e)| *e));
				}
				Expression::SetComprehension(c) => {
					for Generator::Iterator {
						collection: v,
						where_clause,
						..
					}
					| Generator::Assignment {
						value: v,
						where_clause,
						..
					} in c.generators.iter()
					{
						todo.push(*v);
						todo.extend(*where_clause);
					}
					todo.push(c.template);
				}
				Expression::SetLiteral(sl) => {
					todo.extend(sl.members.iter().copied());
				}
				Expression::TupleAccess(ta) => {
					todo.push(ta.tuple);
				}
				Expression::TupleLiteral(tl) => {
					todo.extend(tl.fields.iter().copied());
				}
			}
			Some(e)
		})
	}
}

impl<'db> fmt::Debug for Expression<'db> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Expression::IntegerLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::FloatLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::SetLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::BooleanLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::StringLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::Identifier(x) => fmt::Debug::fmt(x, f),
			Expression::Absent => f.write_str("Absent"),
			Expression::Infinity => f.write_str("Infinity"),
			Expression::TupleLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::RecordLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayLiteral2D(x) => fmt::Debug::fmt(x, f),
			Expression::IndexedArrayLiteral(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayAccess(x) => fmt::Debug::fmt(x, f),
			Expression::ArrayComprehension(x) => fmt::Debug::fmt(x, f),
			Expression::SetComprehension(x) => fmt::Debug::fmt(x, f),
			Expression::IfThenElse(x) => fmt::Debug::fmt(x, f),
			Expression::Call(x) => fmt::Debug::fmt(x, f),
			Expression::Case(x) => fmt::Debug::fmt(x, f),
			Expression::Let(x) => fmt::Debug::fmt(x, f),
			Expression::TupleAccess(x) => fmt::Debug::fmt(x, f),
			Expression::RecordAccess(x) => fmt::Debug::fmt(x, f),
			Expression::Lambda(x) => fmt::Debug::fmt(x, f),
			Expression::Slice(x) => fmt::Debug::fmt(x, f),
			Expression::Missing => f.write_str("Missing"),
		}
	}
}

/// If-then-else
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct IfThenElse<'db> {
	/// The if-then and elseif-then branches
	pub branches: Box<[Branch<'db>]>,
	/// The else result
	pub else_result: Option<ExpressionId<'db>>,
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Branch<'db> {
	/// The boolean condition
	pub condition: ExpressionId<'db>,
	/// The result if the condition holds
	pub result: ExpressionId<'db>,
}

/// Function call
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Call<'db> {
	/// The source construct from which this call was lowered
	pub kind: CallKind,
	/// Function being called
	pub function: ExpressionId<'db>,
	/// Positional call arguments
	pub arguments: Box<[ExpressionId<'db>]>,
	/// Named call arguments
	pub named_arguments: Box<[(PatternId<'db>, ExpressionId<'db>)]>,
}

/// The source construct from which a function call was lowered.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub enum CallKind {
	/// An explicit function call, such as `foo(x)`.
	SourceCall,
	/// A prefix, infix, or postfix operator.
	Operator,
	/// A call using generator or quantification syntax.
	GeneratorCall,
	/// A call introduced by another lowering transformation.
	Synthetic,
}

/// Case expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Case<'db> {
	/// Expression being matched
	pub expression: ExpressionId<'db>,
	/// Cases being matched
	pub cases: Box<[CaseItem<'db>]>,
}

/// Case item
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct CaseItem<'db> {
	/// Pattern being matched
	pub pattern: PatternId<'db>,
	/// Value if matched
	pub value: ExpressionId<'db>,
}

/// Let expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Let<'db> {
	/// Items in this let expression
	pub items: Box<[LetItem<'db>]>,
	/// Value of the let expression
	pub in_expression: ExpressionId<'db>,
}

/// Item in a let expression
#[derive(Clone, Debug, From, Hash, PartialEq, Eq, salsa::Update, Unwrap, TryUnwrap)]
#[unwrap(ref)]
pub enum LetItem<'db> {
	/// A declaration
	Declaration(Declaration<'db>),
	/// A constraint
	Constraint(Constraint<'db>),
}

/// Tuple access expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct TupleAccess<'db> {
	/// Tuple being accessed
	pub tuple: ExpressionId<'db>,
	/// Field being accessed (always an integer)
	pub field: PatternId<'db>,
}

/// Record access expression
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct RecordAccess<'db> {
	/// Record being accessed
	pub record: ExpressionId<'db>,
	/// Field being accessed (always an identifier)
	pub field: PatternId<'db>,
}
/// Lambda function
#[derive(Clone, Debug, Hash, PartialEq, Eq, salsa::Update)]
pub struct Lambda<'db> {
	/// Return type if given
	pub return_type: Option<TypeId<'db>>,
	/// Parameters
	pub parameters: Box<[Parameter<'db>]>,
	/// Function body
	pub body: ExpressionId<'db>,
}
