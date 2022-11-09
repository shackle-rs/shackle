//! AST representation of items

use super::{
	helpers::*, Anonymous, AstNode, Children, Expression, Identifier, Pattern, StringLiteral, Type,
};

ast_enum!(
	/// Item
	Item,
	"include" => Include,
	"declaration" => Declaration,
	"enumeration" => Enumeration,
	"assignment" => Assignment,
	"constraint" => Constraint,
	"goal" => Solve,
	"output" => Output,
	"function_item" => Function,
	"predicate" => Predicate,
	"annotation" => Annotation,
	"type_alias" => TypeAlias,
);

ast_node!(
	/// Include item
	Include,
	file
);

impl Include {
	/// Get the included file
	pub fn file(&self) -> StringLiteral {
		child_with_field_name(self, "file")
	}
}

ast_node!(
	/// Variable declaration item
	Declaration,
	pattern,
	declared_type,
	definition,
	annotations
);

impl Declaration {
	/// Get the pattern of the declaration
	pub fn pattern(&self) -> Pattern {
		child_with_field_name(self, "name")
	}

	/// The type of the declaration
	pub fn declared_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the right hand side of this declaration if there is one
	pub fn definition(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "definition")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Enum declaration item
	Enumeration,
	id,
	cases,
	annotations
);

impl Enumeration {
	/// Get the variable being declared
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the definition of this enumeration
	pub fn cases(&self) -> Children<'_, EnumerationCase> {
		children_with_field_name(self, "case")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_enum!(
	/// Enum definition cases
	EnumerationCase,
	"enumeration_members" => Members(EnumerationMembers),
	"anonymous_enumeration" => Anonymous(AnonymousEnumeration),
	"enumeration_constructor" => Constructor(EnumerationConstructor)
);

ast_node!(
	/// Enum definition using set of identifiers
	EnumerationMembers,
	members
);

impl EnumerationMembers {
	/// Get the members of this enum case
	pub fn members(&self) -> Children<'_, Identifier> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Enum definition using anonymous enum
	AnonymousEnumeration,
	parameters
);

impl AnonymousEnumeration {
	/// Get the callee (will be _)
	pub fn anonymous(&self) -> Anonymous {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'_, Type> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Enum definition using enum constructor call
	EnumerationConstructor,
	id,
	parameters
);

impl EnumerationConstructor {
	/// Get the id of the call
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameter types
	pub fn parameters(&self) -> Children<'_, Type> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// Assignment item
	Assignment,
	assignee,
	definition
);

impl Assignment {
	/// Get the variable being assigned to
	pub fn assignee(&self) -> Expression {
		child_with_field_name(self, "name")
	}

	/// Get the right hand side of this assignment
	pub fn definition(&self) -> Expression {
		child_with_field_name(self, "definition")
	}
}

ast_node!(
	/// Constraint item
	Constraint,
	expression,
	annotations
);

impl Constraint {
	/// Get the value of the constraint
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Solve item
	Solve,
	goal,
	annotations
);

impl Solve {
	/// Get the goal of the solve item
	pub fn goal(&self) -> Goal {
		let tree = self.cst_node().cst();
		let node = self.cst_node().as_ref();
		match node.child_by_field_name("strategy").unwrap().kind() {
			"satisfy" => Goal::Satisfy,
			"maximize" => Goal::Maximize(Expression::new(
				tree.node(node.child_by_field_name("objective").unwrap()),
			)),
			"minimize" => Goal::Minimize(Expression::new(
				tree.node(node.child_by_field_name("objective").unwrap()),
			)),
			_ => unreachable!(),
		}
	}
	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

/// Solve goal
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Goal {
	/// Satisfaction problem
	Satisfy,
	/// Maximize the given objective
	Maximize(Expression),
	/// Minimize the given objective
	Minimize(Expression),
}

impl Goal {
	/// Return whether the solve goal is satisfaction
	pub fn is_satisfy(&self) -> bool {
		if let Goal::Satisfy = *self {
			true
		} else {
			false
		}
	}

	/// Return whether the solve goal is maximization
	pub fn is_maximize(&self) -> bool {
		if let Goal::Maximize(_) = *self {
			true
		} else {
			false
		}
	}

	/// Return whether the solve goal is minimization
	pub fn is_minimize(&self) -> bool {
		if let Goal::Minimize(_) = *self {
			true
		} else {
			false
		}
	}

	/// Get the objective value if there is one
	pub fn objective(&self) -> Option<&Expression> {
		match *self {
			Goal::Maximize(ref obj) => Some(obj),
			Goal::Minimize(ref obj) => Some(obj),
			_ => None,
		}
	}
}

ast_node!(
	/// Output item
	Output,
	expression,
	section
);

impl Output {
	/// Get the value of the output item
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}
	/// The output section (from the annotation)
	pub fn section(&self) -> Option<StringLiteral> {
		optional_child_with_field_name(self, "section")
	}
}

ast_node!(
	/// Function item
	Function,
	return_type,
	id,
	parameters,
	body,
	annotations
);

impl Function {
	/// Get the declared return type of this function
	pub fn return_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the name of this function
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this function
	pub fn parameters(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this function if there is one
	pub fn body(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Predicate item
	Predicate,
	declared_type,
	id,
	parameters,
	body,
	annotations
);

impl Predicate {
	/// Get the type of this predicate
	pub fn declared_type(&self) -> PredicateType {
		match self
			.cst_node()
			.as_ref()
			.child_by_field_name("type")
			.unwrap()
			.kind()
		{
			"predicate" => PredicateType::Predicate,
			"test" => PredicateType::Test,
			_ => unreachable!(),
		}
	}

	/// Get the name of this predicate
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters of this predicate
	pub fn parameters(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}

	/// Get the body of this predicate if there is one
	pub fn body(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "body")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

/// Return type of predicate
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredicateType {
	/// `var bool` function
	Predicate,
	/// `par bool` function
	Test,
}

impl PredicateType {
	/// Return whether this is a predicate
	pub fn is_predicate(&self) -> bool {
		if let PredicateType::Predicate = *self {
			true
		} else {
			false
		}
	}

	/// Return whether this is a test
	pub fn is_test(&self) -> bool {
		if let PredicateType::Test = *self {
			true
		} else {
			false
		}
	}
}

ast_node!(
	/// Annotation item
	Annotation,
	id,
	parameters
);

impl Annotation {
	/// Get the name of this annotation
	pub fn id(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the parameters if this is an annotation constructor, or return `None`
	/// if this is an atomic annotation.
	pub fn parameters(&self) -> Option<AnnotationParameters> {
		optional_child_with_field_name(self, "parameters")
	}
}

ast_node!(
	/// Annotation constructor function parameters
	AnnotationParameters,
	iter
);

impl AnnotationParameters {
	/// Get the parameters
	pub fn iter(&self) -> Children<'_, Parameter> {
		children_with_field_name(self, "parameter")
	}
}

ast_node!(
	/// A function parameter
	Parameter,
	declared_type,
	pattern,
	annotations
);

impl Parameter {
	/// Get the type of this parameter
	pub fn declared_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// Get the pattern of this parameter if there is one
	pub fn pattern(&self) -> Option<Pattern> {
		optional_child_with_field_name(self, "name")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

ast_node!(
	/// Type alias item
	TypeAlias,
	name,
	aliased_type,
	annotations
);

impl TypeAlias {
	/// The name of this type alias
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// The type this is an alias for
	pub fn aliased_type(&self) -> Type {
		child_with_field_name(self, "type")
	}

	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_include() {
		let model = parse_model(r#"include "foo.mzn";"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let include = items.first().unwrap().cast_ref::<Include>().unwrap();
		let file = include.file();
		assert_eq!(file.value(), "foo.mzn");
	}

	#[test]
	fn test_declaration() {
		let model = parse_model("int: x = 3;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let declaration = items.first().unwrap().cast_ref::<Declaration>().unwrap();
		assert_eq!(
			declaration.pattern().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let base = declaration.declared_type().cast::<TypeBase>().unwrap();
		assert!(base.var_type().is_none());
		let pt = base
			.domain()
			.cast_ref::<UnboundedDomain>()
			.unwrap()
			.primitive_type();
		assert!(pt.is_int());
		let def = declaration
			.definition()
			.unwrap()
			.cast::<IntegerLiteral>()
			.unwrap();
		assert_eq!(def.value(), 3);
	}

	#[test]
	fn test_enumeration() {
		let model = parse_model("enum Foo = {A, B, C};");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let enumeration = items.first().unwrap().cast_ref::<Enumeration>().unwrap();
		assert_eq!(enumeration.id().name(), "Foo");
		let cases: Vec<_> = enumeration.cases().collect();
		assert_eq!(cases.len(), 1);
		let definition = cases
			.first()
			.unwrap()
			.cast_ref::<EnumerationMembers>()
			.unwrap();
		let members: Vec<_> = definition.members().collect();
		assert_eq!(members.len(), 3);
		assert_eq!(members[0].name(), "A");
		assert_eq!(members[1].name(), "B");
		assert_eq!(members[2].name(), "C");
	}

	#[test]
	fn test_assignment() {
		let model = parse_model("x = 1;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let assignment = items.first().unwrap().cast_ref::<Assignment>().unwrap();
		assert_eq!(
			assignment.assignee().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let rhs = assignment.definition().cast::<IntegerLiteral>().unwrap();
		assert_eq!(rhs.value(), 1);
	}

	#[test]
	fn test_constraint() {
		let model = parse_model("constraint x > 1;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let constraint = items.first().unwrap().cast_ref::<Constraint>().unwrap();
		let op = constraint.expression().cast::<InfixOperator>().unwrap();
		assert_eq!(op.operator().name(), ">");
		let lhs = op.left().cast::<Identifier>().unwrap();
		assert_eq!(lhs.name(), "x");
		let rhs = op.right().cast::<IntegerLiteral>().unwrap();
		assert_eq!(rhs.value(), 1);
	}

	#[test]
	fn test_solve() {
		let model = parse_model("solve minimize x;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let solve = items.first().unwrap().cast_ref::<Solve>().unwrap();
		let goal = solve.goal();
		assert!(goal.is_minimize());
		let objective = goal.objective().unwrap().cast_ref::<Identifier>().unwrap();
		assert_eq!(objective.name(), "x");
	}

	#[test]
	fn test_output() {
		let model = parse_model(r#"output ["foo"];"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let output = items.first().unwrap().cast_ref::<Output>().unwrap();
		let value = output.expression().cast::<ArrayLiteral>().unwrap();
		let members: Vec<_> = value.members().collect();
		assert_eq!(members.len(), 1);
		let string = members
			.first()
			.unwrap()
			.value()
			.cast::<StringLiteral>()
			.unwrap();
		assert_eq!(string.value(), "foo");
	}

	#[test]
	fn test_function() {
		let model = parse_model("function int: foo(int: x) = x + 1;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);

		let function = items.first().unwrap().cast_ref::<Function>().unwrap();
		assert_eq!(function.id().name(), "foo");

		let params: Vec<_> = function.parameters().collect();
		assert_eq!(params.len(), 1);
		let param = params.first().unwrap();
		assert_eq!(
			param
				.pattern()
				.unwrap()
				.cast::<Identifier>()
				.unwrap()
				.name(),
			"x"
		);
		let param_type = param.declared_type().cast::<TypeBase>().unwrap();
		assert!(param_type.var_type().is_none());
		let domain = param_type.domain().cast::<UnboundedDomain>().unwrap();
		assert!(domain.primitive_type().is_int());

		let body = function.body().unwrap().cast::<InfixOperator>().unwrap();
		assert_eq!(body.operator().name(), "+");
		let lhs = body.left().cast::<Identifier>().unwrap();
		assert_eq!(lhs.name(), "x");
		let rhs = body.right().cast::<IntegerLiteral>().unwrap();
		assert_eq!(rhs.value(), 1);
	}

	#[test]
	fn test_type_alias() {
		let model = parse_model("type Foo = set of int");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let alias = items.first().unwrap().cast_ref::<TypeAlias>().unwrap();
		assert_eq!("Foo", alias.name().name());
		let st = alias.aliased_type().cast::<SetType>().unwrap();
		assert_eq!(st.var_type(), VarType::Par);
		assert_eq!(st.opt_type(), OptType::NonOpt);
		let et = st.element_type().cast::<TypeBase>().unwrap();
		assert!(et.var_type().is_none());
		assert!(et.opt_type().is_none());
		let d = et.domain().cast::<UnboundedDomain>().unwrap();
		assert_eq!(d.primitive_type(), PrimitiveType::Int);
	}
}
