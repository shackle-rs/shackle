//! Mid-level IR

pub mod ty;

use ty::Ty;

use crate::{
	hir::{BooleanLiteral, FloatLiteral, Identifier, IntegerLiteral, StringLiteral},
	thir::source::Origin,
	utils::arena::{Arena, ArenaIndex},
};

/// A mid-level IR program (MicroZinc)
pub struct Model {
	_entrypoint: Expression,
	_annotations: Arena<Annotation>,
	_functions: Arena<Function>,
}

/// An annotation item
pub struct Annotation {
	_name: Identifier,
	_parameter_count: u16,
}

/// A constraint item
pub struct Constraint {
	_expression: Expression,
	_annotations: Vec<AnnotationRef>,
}

/// An annotation
pub enum AnnotationRef {
	/// Identifier for declaration with a RHS expression
	Identifier(Identifier),
	/// Direct reference to annotation definition
	Reference(AnnotationId),
}

/// The ID of an annotation item
pub type AnnotationId = ArenaIndex<Annotation>;

/// A declaration item
pub struct Declaration {
	_ty: Ty,
	_domain: Option<Domain>,
	_name: Identifier,
	_definition: Option<Expression>,
}

/// A domain
pub enum Domain {
	/// Identifier for declaration with a RHS expression
	Identifier(Identifier),
	/// Fully evaluated set domain
	Set(Set),
}

/// A function item
pub struct Function {
	_name: Identifier,
	_parameters: Vec<Identifier>,
	_body: Option<Expression>,
}

/// An expression
pub struct Expression {
	_data: ExpressionData,
	_ty: Ty,
	_origin: Origin,
}

/// The expression data
pub enum ExpressionData {
	/// A let expression
	Let(Let),
	/// A call
	Call(Call),
	/// An if-then-else expression
	IfThenElse(IfThenElse),
	/// A comprehension
	Comprehension(Comprehension),
	/// A value
	Value(ValueData),
	/// A root-level forall
	Forall(Comprehension),
}

/// A let expression
pub struct Let {
	_items: Vec<LetItem>,
	_result: Option<Identifier>,
}

/// An item in a let expression
pub enum LetItem {
	/// A constraint
	Constraint(Constraint),
	/// A declaration
	Declaration(Declaration),
}

/// A tuple literal
pub struct Tuple {
	/// Tuple members
	pub members: Vec<Value>,
}
/// An array literal
pub struct Array {
	/// Array literal members
	pub members: Vec<Value>,
}
/// A set literal
pub struct Set {
	/// Set literal members
	pub members: Vec<Value>,
}
/// An array access
pub struct ArrayAccess {
	/// The array being indexed
	pub array: Identifier,
	/// The indices being used to index the array
	pub indices: Vec<Literal>,
}

/// A tuple field access
pub struct TupleAccess {
	/// The tuple being accessed
	pub tuple: Identifier,
	/// The field
	pub field: IntegerLiteral,
}

/// A call
pub struct Call {
	/// The function being called
	pub function: Identifier,
	/// The arguments
	pub arguments: Vec<Value>,
}

/// An if-then-else expression
///
/// This only has an if-then and else branch, so may need to be nested
pub struct IfThenElse {
	/// The (par) condition
	pub condition: Value,
	/// The value if the condition holds
	pub then: Box<Expression>,
	/// The value if the condition doesn't hold
	pub else_expression: Box<Expression>,
}

/// A comprehension
pub struct Comprehension {
	/// The indices of the generated expression
	pub indices: Option<Box<Expression>>,
	/// The generated expression
	pub expression: Box<Expression>,
	/// The generators
	pub generators: Vec<Generator>,
}

/// A generator in a comprehension
pub enum Generator {
	/// An iterator such as `i, j in foo where bar`
	Iterator {
		/// The names of the iterators
		names: Vec<Identifier>,
		/// The collection being iterated over
		collection: Expression,
		/// The where clause
		where_clause: Option<Expression>,
	},
	/// An assignment such as `i = foo where bar`
	Assignment {
		/// The name of the assignment
		name: Identifier,
		/// The value of the assignment
		definition: Expression,
		/// The where clause
		where_clause: Option<Expression>,
	},
}

/// A literal
pub struct Literal {
	_data: LiteralData,
	_ty: Ty,
	_origin: Origin,
}

/// The literal data
pub enum LiteralData {
	/// Bottom (cannot be evaluated)
	Bottom,
	/// A boolean
	Boolean(BooleanLiteral),
	/// An integer
	Integer(IntegerLiteral),
	/// A floating point value
	Float(FloatLiteral),
	/// A string
	String(StringLiteral),
	/// Infinity
	Infinity,
	/// An identifier
	Identifier(Identifier),
}

/// A value
pub struct Value {
	_data: ValueData,
	_ty: Ty,
	_origin: Origin,
}

/// The value data
pub enum ValueData {
	/// A literal
	Literal(LiteralData),
	/// A tuple
	Tuple(Tuple),
	/// A set
	Set(Set),
	/// An array
	Array(Array),
	/// An array access
	ArrayAccess(ArrayAccess),
	/// A tuple access
	TupleAccess(TupleAccess),
}
