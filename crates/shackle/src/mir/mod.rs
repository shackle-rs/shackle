//! Mid-level IR

pub mod ty;

use crate::{
	arena::{Arena, ArenaIndex},
	hir::{BooleanLiteral, FloatLiteral, Identifier, IntegerLiteral, StringLiteral},
	thir::source::Origin,
};

use ty::Ty;

pub struct Model {
	entrypoint: Expression,
	annotations: Arena<Annotation>,
	functions: Arena<Function>,
}

pub struct Annotation {
	name: Identifier,
	parameter_count: u16,
}

pub struct Constraint {
	expression: Expression,
	annotations: Vec<AnnotationRef>,
}

pub enum AnnotationRef {
	Identifier(Identifier),
	Reference(AnnotationId),
}

pub type AnnotationId = ArenaIndex<Annotation>;

pub struct Declaration {
	ty: Ty,
	domain: Option<Domain>,
	name: Identifier,
	definition: Option<Expression>,
}

pub enum Domain {
	Identifier(Identifier),
	Set(Set),
}

pub struct Function {
	name: Identifier,
	parameters: Vec<Identifier>,
	body: Option<Expression>,
}

pub struct Expression {
	data: ExpressionData,
	ty: Ty,
	origin: Origin,
}

pub enum ExpressionData {
	Let(Let),
	Call(Call),
	IfThenElse(IfThenElse),
	Comprehension(Comprehension),
	Value(ValueData),
	Forall(Comprehension),
}
pub struct Tuple {
	pub members: Vec<Value>,
}
pub struct Array {
	pub members: Vec<Value>,
}
pub struct Set {
	pub members: Vec<Value>,
}

pub struct ArrayAccess {
	pub array: Identifier,
	pub indices: Vec<Literal>,
}

pub struct TupleAccess {
	pub tuple: Identifier,
	pub field: IntegerLiteral,
}

pub struct Call {
	pub function: Identifier,
	pub arguments: Vec<Value>,
}

pub struct IfThenElse {
	pub condition: Value,
	pub then: Box<Expression>,
	pub else_expression: Box<Expression>,
}

pub struct Comprehension {
	pub indices: Option<Box<Expression>>,
	pub expression: Box<Expression>,
	pub generators: Vec<Generator>,
}

pub enum Generator {
	Iterator {
		names: Vec<Identifier>,
		collection: Expression,
		where_clause: Option<Expression>,
	},
	Assignment {
		name: Identifier,
		definition: Expression,
		where_clause: Option<Expression>,
	},
}

pub struct Literal {
	data: LiteralData,
	ty: Ty,
	origin: Origin,
}

pub enum LiteralData {
	Bottom,
	Boolean(BooleanLiteral),
	Integer(IntegerLiteral),
	Float(FloatLiteral),
	String(StringLiteral),
	Infinity,
	Identifier(Identifier),
}

pub struct Value {
	data: ValueData,
	ty: Ty,
	origin: Origin,
}

pub enum ValueData {
	Literal(LiteralData),
	Tuple(Tuple),
	Set(Set),
	Array(Array),
	ArrayAccess(ArrayAccess),
	TupleAccess(TupleAccess),
}

pub struct Let {
	items: Vec<LetItem>,
	result: Option<Identifier>,
}

pub enum LetItem {
	Constraint(Constraint),
	Declaration(Declaration),
}
