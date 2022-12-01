//! THIR representation of expressions

use std::{fmt::Debug, ops::Deref};

use rustc_hash::FxHashMap;

use super::{
	source::Origin, AnnotationId, ConstraintId, DeclarationId, EnumerationId, FunctionId,
	Identifier, ItemData,
};
pub use crate::hir::{BooleanLiteral, FloatLiteral, IntegerLiteral, StringLiteral};
use crate::{
	arena::ArenaIndex,
	ty::{Ty, TyVarRef},
};

/// An expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
	ty: Ty,
	annotations: Vec<ArenaIndex<Expression>>,
	data: ExpressionData,
	origin: Origin,
}

impl Expression {
	/// Get the type of this expression
	pub fn ty(&self) -> Ty {
		self.ty
	}

	/// Get the annotations for this expression
	pub fn annotations(&self) -> &[ArenaIndex<Expression>] {
		&self.annotations
	}

	/// Get the origin of this expression
	pub fn origin(&self) -> Origin {
		self.origin
	}
}

impl Deref for Expression {
	type Target = ExpressionData;
	fn deref(&self) -> &Self::Target {
		&self.data
	}
}

/// A trait for building expression trees
pub trait ExpressionBuilder: Debug {
	/// Build the expression by adding it to the given `owner`
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression>;

	/// Clone the expression builder
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder>;
}

impl Clone for Box<dyn ExpressionBuilder> {
	fn clone(&self) -> Self {
		self.clone_dyn()
	}
}

/// Builder for `<>`
#[derive(Debug, Clone)]
pub struct AbsentBuilder {
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl AbsentBuilder {
	/// Create a new absent literal
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for AbsentBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::Absent,
			origin: self.origin,
			annotations,
		})
	}

	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for boolean literals
#[derive(Debug, Clone)]
pub struct BooleanLiteralBuilder {
	value: BooleanLiteral,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl BooleanLiteralBuilder {
	/// Create a new boolean
	pub fn new(ty: Ty, value: BooleanLiteral, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			value,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for BooleanLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::BooleanLiteral(self.value),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for integer literals
#[derive(Debug, Clone)]
pub struct IntegerLiteralBuilder {
	value: IntegerLiteral,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl IntegerLiteralBuilder {
	/// Create a new integer
	pub fn new(ty: Ty, value: IntegerLiteral, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			value,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for IntegerLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::IntegerLiteral(self.value),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for float literals
#[derive(Debug, Clone)]
pub struct FloatLiteralBuilder {
	value: FloatLiteral,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl FloatLiteralBuilder {
	/// Create a new float
	pub fn new(ty: Ty, value: FloatLiteral, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			value,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for FloatLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::FloatLiteral(self.value),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for string literals
#[derive(Debug, Clone)]
pub struct StringLiteralBuilder {
	value: StringLiteral,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl StringLiteralBuilder {
	/// Create a new string
	pub fn new(ty: Ty, value: StringLiteral, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			value,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for StringLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::StringLiteral(self.value.clone()),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for `infinity`
#[derive(Debug, Clone)]
pub struct InfinityBuilder {
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl InfinityBuilder {
	/// Create a new infinity
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for InfinityBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::Infinity,
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for identifiers
#[derive(Debug, Clone)]
pub struct IdentifierBuilder {
	value: ResolvedIdentifier,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl IdentifierBuilder {
	/// Create a new identifier
	pub fn new(ty: Ty, value: ResolvedIdentifier, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			value,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for IdentifierBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::Identifier(self.value.clone()),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for array literals
#[derive(Debug, Clone)]
pub struct ArrayLiteralBuilder {
	members: Vec<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl ArrayLiteralBuilder {
	/// Create a new array literal
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			members: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given member to this array literal
	pub fn with_member(mut self: Box<Self>, expression: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.members.push(expression);
		self
	}

	/// Add the given members to this array literal
	pub fn with_members(
		mut self: Box<Self>,
		expressions: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.members.extend(expressions);
		self
	}
}

impl ExpressionBuilder for ArrayLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let members = self.members.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::ArrayLiteral(members),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for set literals
#[derive(Debug, Clone)]
pub struct SetLiteralBuilder {
	members: Vec<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl SetLiteralBuilder {
	/// Create a new set literal
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			members: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given member to this set literal
	pub fn with_member(mut self: Box<Self>, expression: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.members.push(expression);
		self
	}

	/// Add the given members to this set literal
	pub fn with_members(
		mut self: Box<Self>,
		expression: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.members.extend(expression);
		self
	}
}

impl ExpressionBuilder for SetLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let members = self.members.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::SetLiteral(members),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for tuple literals
#[derive(Debug, Clone)]
pub struct TupleLiteralBuilder {
	members: Vec<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl TupleLiteralBuilder {
	/// Create a new tuple literal
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			members: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given member to this tuple literal
	pub fn with_member(mut self: Box<Self>, expression: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.members.push(expression);
		self
	}

	/// Add the given members to this tuple literal
	pub fn with_members(
		mut self: Box<Self>,
		expression: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.members.extend(expression);
		self
	}
}

impl ExpressionBuilder for TupleLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let members = self.members.iter().map(|e| e.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::TupleLiteral(members),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for record literals
#[derive(Debug, Clone)]
pub struct RecordLiteralBuilder {
	members: FxHashMap<Identifier, Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl RecordLiteralBuilder {
	/// Create a new record literal
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			members: FxHashMap::default(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given member to this record literal
	pub fn with_member(
		mut self: Box<Self>,
		identifier: Identifier,
		expression: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.members.insert(identifier, expression);
		self
	}

	/// Add the given members to this record literal
	pub fn with_members(
		mut self: Box<Self>,
		members: impl IntoIterator<Item = (Identifier, Box<dyn ExpressionBuilder>)>,
	) -> Box<Self> {
		self.members.extend(members);
		self
	}
}

impl ExpressionBuilder for RecordLiteralBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let members = self
			.members
			.iter()
			.map(|(i, e)| (*i, e.finish(owner)))
			.collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::RecordLiteral(members),
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for array comprehensions
#[derive(Debug, Clone)]
pub struct ArrayComprehensionBuilder {
	template: Option<Box<dyn ExpressionBuilder>>,
	indices: Option<Box<dyn ExpressionBuilder>>,
	generators: Vec<GeneratorBuilder>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl ArrayComprehensionBuilder {
	/// Create a new array comprehension
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			template: None,
			indices: None,
			generators: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Set the template for this array comprehension
	pub fn with_template(mut self: Box<Self>, template: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.template = Some(template);
		self
	}

	/// Set the indices for this array comprehension
	pub fn with_indices(
		mut self: Box<Self>,
		indices: Option<Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.indices = indices;
		self
	}

	/// Add the given generator to this comprehension
	pub fn with_generator(mut self: Box<Self>, generator: GeneratorBuilder) -> Box<Self> {
		self.generators.push(generator);
		self
	}

	/// Add the given generators to this comprehension
	pub fn with_generators(
		mut self: Box<Self>,
		generators: impl IntoIterator<Item = GeneratorBuilder>,
	) -> Box<Self> {
		self.generators.extend(generators);
		self
	}
}

impl ExpressionBuilder for ArrayComprehensionBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let indices = self.indices.as_ref().map(|e| e.finish(owner));
		let template = self
			.template
			.as_ref()
			.expect("No template for comprehension")
			.finish(owner);
		let generators = self
			.generators
			.iter()
			.map(|g| g.finish(owner))
			.collect::<Vec<_>>();
		assert!(
			!generators.is_empty(),
			"Cannot create array comprehension without generator"
		);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::ArrayComprehension {
				template,
				indices,
				generators,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for array comprehensions
#[derive(Debug, Clone)]
pub struct SetComprehensionBuilder {
	template: Option<Box<dyn ExpressionBuilder>>,
	generators: Vec<GeneratorBuilder>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl SetComprehensionBuilder {
	/// Create a new set comprehension
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			template: None,
			generators: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Set the template for this set comprehension
	pub fn with_template(mut self: Box<Self>, template: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.template = Some(template);
		self
	}

	/// Add the given generator to this comprehension
	pub fn with_generator(mut self: Box<Self>, generator: GeneratorBuilder) -> Box<Self> {
		self.generators.push(generator);
		self
	}

	/// Add the given generators to this comprehension
	pub fn with_generators(
		mut self: Box<Self>,
		generators: impl IntoIterator<Item = GeneratorBuilder>,
	) -> Box<Self> {
		self.generators.extend(generators);
		self
	}
}

impl ExpressionBuilder for SetComprehensionBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let template = self
			.template
			.as_ref()
			.expect("No template for comprehension")
			.finish(owner);
		let generators = self
			.generators
			.iter()
			.map(|g| g.finish(owner))
			.collect::<Vec<_>>();
		assert!(
			!generators.is_empty(),
			"Cannot create set comprehension without generator"
		);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::SetComprehension {
				template,
				generators,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for generators
#[derive(Debug, Clone)]
pub struct GeneratorBuilder {
	declarations: Vec<DeclarationId>,
	collection: Box<dyn ExpressionBuilder>,
	where_clause: Option<Box<dyn ExpressionBuilder>>,
}

impl GeneratorBuilder {
	/// Create a new generator
	pub fn new(collection: Box<dyn ExpressionBuilder>) -> Self {
		Self {
			declarations: Vec::new(),
			collection,
			where_clause: None,
		}
	}

	/// Add the given declaration to this generator
	pub fn with_declaration(mut self, declaration: DeclarationId) -> Self {
		self.declarations.push(declaration);
		self
	}

	/// Set the where clause for this generator
	pub fn with_where(mut self, where_clause: Box<dyn ExpressionBuilder>) -> Self {
		self.where_clause = Some(where_clause);
		self
	}

	/// Finish building the generator
	pub fn finish(&self, owner: &mut ItemData) -> Generator {
		let collection = self.collection.finish(owner);
		let where_clause = self.where_clause.as_ref().map(|e| e.finish(owner));
		Generator {
			declarations: self.declarations.clone(),
			collection,
			where_clause,
		}
	}
}

/// Builder for array access
#[derive(Debug, Clone)]
pub struct ArrayAccessBuilder {
	collection: Box<dyn ExpressionBuilder>,
	indices: Box<dyn ExpressionBuilder>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl ArrayAccessBuilder {
	/// Create a new array access
	pub fn new(
		ty: Ty,
		collection: Box<dyn ExpressionBuilder>,
		indices: Box<dyn ExpressionBuilder>,
		origin: impl Into<Origin>,
	) -> Box<Self> {
		Box::new(Self {
			collection,
			indices,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for ArrayAccessBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let collection = self.collection.finish(owner);
		let indices = self.indices.finish(owner);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::ArrayAccess {
				collection,
				indices,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for tuple access
#[derive(Debug, Clone)]
pub struct TupleAccessBuilder {
	tuple: Box<dyn ExpressionBuilder>,
	field: IntegerLiteral,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl TupleAccessBuilder {
	/// Create a new tuple access
	pub fn new(
		ty: Ty,
		tuple: Box<dyn ExpressionBuilder>,
		field: IntegerLiteral,
		origin: impl Into<Origin>,
	) -> Box<Self> {
		Box::new(Self {
			tuple,
			field,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for TupleAccessBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let tuple = self.tuple.finish(owner);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::TupleAccess {
				tuple,
				field: self.field,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for record access
#[derive(Debug, Clone)]
pub struct RecordAccessBuilder {
	record: Box<dyn ExpressionBuilder>,
	field: Identifier,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl RecordAccessBuilder {
	/// Create a new record access
	pub fn new(
		ty: Ty,
		record: Box<dyn ExpressionBuilder>,
		field: Identifier,
		origin: impl Into<Origin>,
	) -> Box<Self> {
		Box::new(Self {
			record,
			field,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}
}

impl ExpressionBuilder for RecordAccessBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let record = self.record.finish(owner);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::RecordAccess {
				record,
				field: self.field,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for if-then-else
#[derive(Debug, Clone)]
pub struct IfThenElseBuilder {
	branches: Vec<(Box<dyn ExpressionBuilder>, Box<dyn ExpressionBuilder>)>,
	else_result: Option<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl IfThenElseBuilder {
	/// Create a new if-then-else
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			branches: Vec::new(),
			else_result: None,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given branch to the if-then-else
	pub fn with_branch(
		mut self: Box<Self>,
		condition: Box<dyn ExpressionBuilder>,
		result: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.branches.push((condition, result));
		self
	}

	/// Add the given branches to the if-then-else
	pub fn with_branches(
		mut self: Box<Self>,
		branches: impl IntoIterator<Item = (Box<dyn ExpressionBuilder>, Box<dyn ExpressionBuilder>)>,
	) -> Box<Self> {
		self.branches.extend(branches);
		self
	}

	/// Set the else-
	pub fn with_else(mut self: Box<Self>, result: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.else_result = Some(result);
		self
	}
}

impl ExpressionBuilder for IfThenElseBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let branches = self
			.branches
			.iter()
			.map(|(c, r)| Branch {
				condition: c.finish(owner),
				result: r.finish(owner),
			})
			.collect::<Vec<_>>();
		assert!(
			!branches.is_empty(),
			"Cannot create if-then-else with no branches"
		);
		let else_result = self
			.else_result
			.as_ref()
			.expect("Cannot create if-then-else without else")
			.finish(owner);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::IfThenElse {
				branches,
				else_result,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for function calls
#[derive(Debug, Clone)]
pub struct CallBuilder {
	function: Box<dyn ExpressionBuilder>,
	arguments: Vec<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl CallBuilder {
	/// Create a new call
	pub fn new(
		ty: Ty,
		function: Box<dyn ExpressionBuilder>,
		origin: impl Into<Origin>,
	) -> Box<Self> {
		Box::new(Self {
			function,
			arguments: Vec::new(),
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add an argument
	pub fn with_arg(mut self: Box<Self>, arg: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.arguments.push(arg);
		self
	}

	/// Add arguments
	pub fn with_args(
		mut self: Box<Self>,
		args: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.arguments.extend(args);
		self
	}
}

impl ExpressionBuilder for CallBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let function = self.function.finish(owner);
		let arguments = self.arguments.iter().map(|a| a.finish(owner)).collect();
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::Call {
				function,
				arguments,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

/// Builder for let expressions
#[derive(Debug, Clone)]
pub struct LetBuilder {
	items: Vec<LetItem>,
	in_expression: Option<Box<dyn ExpressionBuilder>>,
	annotations: Vec<Box<dyn ExpressionBuilder>>,
	ty: Ty,
	origin: Origin,
}

impl LetBuilder {
	/// Create a new let expression
	pub fn new(ty: Ty, origin: impl Into<Origin>) -> Box<Self> {
		Box::new(Self {
			items: Vec::new(),
			in_expression: None,
			annotations: Vec::new(),
			ty,
			origin: origin.into(),
		})
	}

	/// Add the given annotation to this expression
	pub fn with_annotation(
		mut self: Box<Self>,
		annotation: Box<dyn ExpressionBuilder>,
	) -> Box<Self> {
		self.annotations.push(annotation);
		self
	}

	/// Add the given annotations to this expression
	pub fn with_annotations(
		mut self: Box<Self>,
		annotations: impl IntoIterator<Item = Box<dyn ExpressionBuilder>>,
	) -> Box<Self> {
		self.annotations.extend(annotations);
		self
	}

	/// Add the given item to this let expression
	pub fn with_item(mut self: Box<Self>, item: LetItem) -> Box<Self> {
		self.items.push(item);
		self
	}

	/// Add the given items to this let expression
	pub fn with_items(mut self: Box<Self>, items: impl IntoIterator<Item = LetItem>) -> Box<Self> {
		self.items.extend(items);
		self
	}

	/// Set the value of this let expression
	pub fn with_in(mut self: Box<Self>, in_expression: Box<dyn ExpressionBuilder>) -> Box<Self> {
		self.in_expression = Some(in_expression);
		self
	}
}

impl ExpressionBuilder for LetBuilder {
	fn finish(&self, owner: &mut ItemData) -> ArenaIndex<Expression> {
		let annotations = self.annotations.iter().map(|e| e.finish(owner)).collect();
		let in_expression = self
			.in_expression
			.as_ref()
			.expect("No 'in' expression for let")
			.finish(owner);
		owner.expressions.insert(Expression {
			ty: self.ty,
			data: ExpressionData::Let {
				items: self.items.clone(),
				in_expression,
			},
			origin: self.origin,
			annotations,
		})
	}
	fn clone_dyn(&self) -> Box<dyn ExpressionBuilder> {
		Box::new(self.clone())
	}
}

// /// Mutable access to an expression
// pub struct ExpressionMut<'a> {
// 	inner: &'a mut Expression,
// }

// impl<'a> ExpressionMut<'a> {}

/// An expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionData {
	/// Absent `<>`
	Absent,
	/// Bool literal
	BooleanLiteral(BooleanLiteral),
	/// Integer literal
	IntegerLiteral(IntegerLiteral),
	/// Float literal
	FloatLiteral(FloatLiteral),
	/// String literal
	StringLiteral(StringLiteral),
	/// Infinity
	Infinity,
	/// Identifier
	Identifier(ResolvedIdentifier),
	/// Array literal
	ArrayLiteral(Vec<ArenaIndex<Expression>>),
	/// Set literal
	SetLiteral(Vec<ArenaIndex<Expression>>),
	/// Tuple literal
	TupleLiteral(Vec<ArenaIndex<Expression>>),
	/// Record literal
	RecordLiteral(Vec<(Identifier, ArenaIndex<Expression>)>),
	/// Array comprehension
	ArrayComprehension {
		/// Value of the comprehension
		template: ArenaIndex<Expression>,
		/// The indices to generate
		indices: Option<ArenaIndex<Expression>>,
		/// Generators of the comprehension
		generators: Vec<Generator>,
	},
	/// Set comprehension
	SetComprehension {
		/// Value of the comprehension
		template: ArenaIndex<Expression>,
		/// Generators of the comprehension
		generators: Vec<Generator>,
	},
	/// Array access
	ArrayAccess {
		/// The array being indexed into
		collection: ArenaIndex<Expression>,
		/// The indices
		indices: ArenaIndex<Expression>,
	},
	/// Tuple access
	TupleAccess {
		/// Tuple being accessed
		tuple: ArenaIndex<Expression>,
		/// Field being accessed
		field: IntegerLiteral,
	},
	/// Record access
	RecordAccess {
		/// Record being accessed
		record: ArenaIndex<Expression>,
		/// Field being accessed
		field: Identifier,
	},
	/// If-then-else
	IfThenElse {
		/// The if-then and elseif-then branches
		branches: Vec<Branch>,
		/// The else result
		else_result: ArenaIndex<Expression>,
	},
	/// Function call
	Call {
		/// Function being called
		function: ArenaIndex<Expression>,
		/// Call arguments
		arguments: Vec<ArenaIndex<Expression>>,
	},
	/// Let expression
	Let {
		/// Items in this let expression
		items: Vec<LetItem>,
		/// Value of the let expression
		in_expression: ArenaIndex<Expression>,
	},
}

/// An identifier which resolves to a declaration
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedIdentifier {
	/// Identifier resolves to an annotation
	Annotation(AnnotationId),
	/// Identifier resolves to an annotation deconstructor
	AnnotationDeconstructor(AnnotationId),
	/// Identifier resolves to a declaration
	Declaration(DeclarationId),
	/// Identifier resolves to an enumeration
	Enumeration(EnumerationId),
	/// Identifier resolves to an enumeration member with the given index
	EnumerationMember(EnumerationId, usize),
	/// Identifier resolves to the deconstructor for an enumeration member with the given index
	EnumerationDeconstructor(EnumerationId, usize),
	/// Identifier resolves to a function
	Function(FunctionId),
	/// Identifier resolves to a type-inst variable
	TyVarRef(TyVarRef),
}

/// Comprehension generator
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generator {
	/// Generator declaration
	pub declarations: Vec<DeclarationId>,
	/// Expression being iterated over
	pub collection: ArenaIndex<Expression>,
	/// Where clause
	pub where_clause: Option<ArenaIndex<Expression>>,
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
	/// The boolean condition
	pub condition: ArenaIndex<Expression>,
	/// The result if the condition holds
	pub result: ArenaIndex<Expression>,
}

/// An item in a let expression
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetItem {
	/// A local constraint item
	Constraint(ConstraintId),
	/// A local declaration item
	Declaration(DeclarationId),
}
