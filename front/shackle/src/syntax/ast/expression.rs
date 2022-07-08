//! AST representation of expressions

use crate::syntax::cst::CstNode;

use super::helpers::*;
use super::{
	Absent, ArrayAccess, ArrayComprehension, ArrayLiteral, ArrayLiteral2D, AstNode, BooleanLiteral,
	Children, Constraint, Declaration, FloatLiteral, Generator, Infinity, IntegerLiteral, Pattern,
	RecordLiteral, SetComprehension, SetLiteral, StringLiteral, TupleLiteral,
};

ast_enum!(
	/// Expression
	Expression,
	"integer_literal" => IntegerLiteral,
	"float_literal" => FloatLiteral,
	"tuple_literal" => TupleLiteral,
	"record_literal" => RecordLiteral,
	"set_literal" => SetLiteral,
	"boolean_literal" => BooleanLiteral,
	"string_literal" => StringLiteral,
	"identifier" | "quoted_identifier" => Identifier,
	"absent" => Absent,
	"infinity" => Infinity,
	"anonymous" => Anonymous,
	"array_literal" => ArrayLiteral,
	"array_literal_2d" => ArrayLiteral2D,
	"indexed_access" => ArrayAccess,
	"array_comprehension" => ArrayComprehension,
	"set_comprehension" => SetComprehension,
	"if_then_else" => IfThenElse,
	"call" => Call,
	"prefix_operator" => PrefixOperator,
	"infix_operator" => InfixOperator,
	"postfix_operator" => PostfixOperator,
	"generator_call" => GeneratorCall,
	"string_interpolation" => StringInterpolation,
	"case_expression" => Case,
	"let_expression" => Let,
	"tuple_access" => TupleAccess,
	"record_access" => RecordAccess,
	"annotated_expression" => AnnotatedExpression,
	"parenthesised_expression" => "expression" // Turn parenthesised_expression into Expression node
);

ast_node!(
	/// An annotated expression
	AnnotatedExpression,
	annotations,
	expression
);

impl AnnotatedExpression {
	/// The annotations
	pub fn annotations(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "annotation")
	}
	/// The expression which was annotated
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}
}

ast_enum!(
	/// An identifier (quoted or normal)
	Identifier,
	"identifier" => UnquotedIdentifier,
	"quoted_identifier" => QuotedIdentifier
);

impl Identifier {
	/// Get the name of this identifier
	pub fn name(&self) -> &str {
		match *self {
			Identifier::QuotedIdentifier(ref i) => i.name(),
			Identifier::UnquotedIdentifier(ref i) => i.name(),
		}
	}
}

ast_node!(
	/// Identifierentifier
	UnquotedIdentifier,
	name
);

impl UnquotedIdentifier {
	/// Get the name of this identifier
	pub fn name(&self) -> &str {
		self.cst_text()
	}
}

ast_node!(
	/// Quoted identifier
	QuotedIdentifier,
	name
);

impl QuotedIdentifier {
	/// Get the name of this identifier without the enclosing quotes
	pub fn name(&self) -> &str {
		let text = self.cst_text();
		&text[1..text.len() - 1]
	}
}

ast_node!(
	/// Anonymous variable `_`
	Anonymous,
);

ast_node!(
	/// If-then-else
	IfThenElse,
	branches,
	else_result
);

impl IfThenElse {
	/// If-then and elseif-then pairs
	pub fn branches(&self) -> Branches<'_> {
		Branches {
			conditions: children_with_field_name(self, "condition"),
			results: children_with_field_name(self, "result"),
		}
	}

	/// Else expression
	pub fn else_result(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "else")
	}
}

/// Iterator over the branches of an `IfThenElse`

#[derive(Clone, Debug)]
pub struct Branches<'a> {
	conditions: Children<'a, Expression>,
	results: Children<'a, Expression>,
}

impl Iterator for Branches<'_> {
	type Item = Branch;
	fn next(&mut self) -> Option<Branch> {
		match (self.conditions.next(), self.results.next()) {
			(Some(condition), Some(result)) => Some(Branch { condition, result }),
			(None, None) => None,
			_ => unreachable!("Mismatch in size of conditions and results for if-then-else"),
		}
	}
}

/// A branch of an `IfThenElse`
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Branch {
	/// The boolean condition
	pub condition: Expression,
	/// The result if the condition holds
	pub result: Expression,
}

ast_node!(
	/// Function call
	Call,
	function,
	arguments
);

impl Call {
	/// Get the expression being called
	/// Will usually be an identifier
	pub fn function(&self) -> Expression {
		child_with_field_name(self, "function")
	}

	/// Get the call arguments.
	pub fn arguments(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "argument")
	}
}

ast_node!(
	/// Prefix (unary) operator
	PrefixOperator,
	operator,
	operand
);

impl PrefixOperator {
	/// Get the operator
	pub fn operator(&self) -> &str {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("operator").unwrap().kind()
	}

	/// Get the operand
	pub fn operand(&self) -> Expression {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// Infix (binary) operator
	InfixOperator,
	left,
	operator,
	right
);

impl InfixOperator {
	/// Get the left hand side
	pub fn operator(&self) -> &str {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("operator").unwrap().kind()
	}

	/// Get the left hand side
	pub fn left(&self) -> Expression {
		child_with_field_name(self, "left")
	}

	/// Get the left hand side
	pub fn right(&self) -> Expression {
		child_with_field_name(self, "right")
	}
}

ast_node!(
	/// Postfix operator
	PostfixOperator,
	operand,
	operator,
);

impl PostfixOperator {
	/// Get the operator
	pub fn operator(&self) -> &str {
		let node = self.cst_node().as_ref();
		node.child_by_field_name("operator").unwrap().kind()
	}

	/// Get the operand
	pub fn operand(&self) -> Expression {
		child_with_field_name(self, "operand")
	}
}

ast_node!(
	/// Call using generator syntax
	GeneratorCall,
	function,
	generators,
	template
);

impl GeneratorCall {
	/// Get the expression being called
	/// Should always be an `Identifier` for now but for lambdas would be something else
	pub fn function(&self) -> Expression {
		child_with_field_name(self, "function")
	}

	/// The generators for this call
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}

	/// The body of this call
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}
}

ast_node!(
	/// String interpolation
	StringInterpolation,
	contents
);

impl StringInterpolation {
	/// Get the contents of this string interpolation
	pub fn contents(&self) -> Children<'_, InterpolationItem> {
		children_with_field_name(self, "item")
	}
}

/// An element in a string interpolation
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum InterpolationItem {
	/// String content
	String(String),
	/// An expression
	Expression(Expression),
}

impl InterpolationItem {
	/// Return whether this interpolation item is a string
	pub fn is_string(&self) -> bool {
		if let InterpolationItem::String(_) = *self {
			true
		} else {
			false
		}
	}
	/// Return whether this interpolation item is an expression
	pub fn is_expression(&self) -> bool {
		if let InterpolationItem::Expression(_) = *self {
			true
		} else {
			false
		}
	}

	/// Get the string if this is one
	pub fn string(&self) -> Option<&str> {
		match *self {
			InterpolationItem::String(ref s) => Some(s),
			_ => None,
		}
	}

	/// Get the expression if this is one
	pub fn expression(&self) -> Option<&Expression> {
		match *self {
			InterpolationItem::Expression(ref e) => Some(e),
			_ => None,
		}
	}
}

impl From<CstNode> for InterpolationItem {
	fn from(syntax: CstNode) -> Self {
		let tree = syntax.cst();
		let c = syntax.as_ref();
		match c.kind() {
			"string" => InterpolationItem::String(decode_string(&tree.node(*c))),
			"expression" => {
				InterpolationItem::Expression(Expression::new(tree.node(c.child(0).unwrap())))
			}
			_ => unreachable!(),
		}
	}
}

ast_node!(
	/// Let expression
	Let,
	items,
	in_expression
);

impl Let {
	/// The items of the let expression
	pub fn items(&self) -> Children<'_, LetItem> {
		children_with_field_name(self, "item")
	}

	/// The value of the let expression
	pub fn in_expression(&self) -> Expression {
		child_with_field_name(self, "in")
	}
}

ast_node!(
	/// Case pattern match
	Case,
	expression,
	cases,
);

impl Case {
	/// The expression being matched
	pub fn expression(&self) -> Expression {
		child_with_field_name(self, "expression")
	}

	/// The cases
	pub fn cases(&self) -> Children<'_, CaseItem> {
		children_with_field_name(self, "case")
	}
}

ast_node!(
	/// Case pattern case
	CaseItem,
	pattern,
	value
);

impl CaseItem {
	/// The pattern to match
	pub fn pattern(&self) -> Pattern {
		child_with_field_name(self, "pattern")
	}

	/// The value if this case holds
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}
}

ast_enum!(
	/// Item in a let expression
	LetItem,
	"declaration" => Declaration,
	"constraint" => Constraint
);

ast_node!(
	/// Tuple access
	TupleAccess,
	tuple,
	field
);

impl TupleAccess {
	/// The tuple being accessed
	pub fn tuple(&self) -> Expression {
		child_with_field_name(self, "tuple")
	}

	/// The field being accessed
	pub fn field(&self) -> IntegerLiteral {
		child_with_field_name(self, "field")
	}
}

ast_node!(
	/// Record access
	RecordAccess,
	record,
	field
);

impl RecordAccess {
	/// The record being accessed
	pub fn record(&self) -> Expression {
		child_with_field_name(self, "record")
	}

	/// The field being accessed
	pub fn field(&self) -> Identifier {
		child_with_field_name(self, "field")
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_annotated_expression() {
		let model = parse_model(
			r#"
		x = foo :: bar :: qux;
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let ae = x.definition().cast::<AnnotatedExpression>().unwrap();
			let anns: Vec<_> = ae.annotations().collect();
			assert_eq!(anns.len(), 2);
			assert_eq!(anns[0].cast_ref::<Identifier>().unwrap().name(), "bar");
			assert_eq!(anns[1].cast_ref::<Identifier>().unwrap().name(), "qux");
			assert_eq!(ae.expression().cast::<Identifier>().unwrap().name(), "foo");
		}
	}

	#[test]
	fn test_identifier() {
		let model = parse_model(
			r#"
		bool: x;
		bool: 'hello world';
		bool: ✔️;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);
		let cases = items
			.into_iter()
			.map(|i| {
				i.cast::<Declaration>()
					.unwrap()
					.pattern()
					.cast::<Identifier>()
					.unwrap()
			})
			.zip(["x", "hello world", "✔️"]);
		for (item, expected) in cases {
			assert_eq!(item.name(), expected);
		}
	}

	#[test]
	fn test_if_then_else() {
		let model = parse_model(
			r#"
		x = if a then b else c endif;
		y = if a then b elseif c then d else e endif;
		z = if a then b endif;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);
		{
			let asg = items[0].cast_ref::<Assignment>().unwrap();
			let ite = asg.definition().cast::<IfThenElse>().unwrap();
			let branches: Vec<_> = ite.branches().collect();
			assert_eq!(branches.len(), 1);
			assert_eq!(
				branches[0]
					.condition
					.cast_ref::<Identifier>()
					.unwrap()
					.name(),
				"a"
			);
			assert_eq!(
				branches[0].result.cast_ref::<Identifier>().unwrap().name(),
				"b"
			);
			assert_eq!(
				ite.else_result()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"c"
			);
		}
		{
			let asg = items[1].cast_ref::<Assignment>().unwrap();
			let ite = asg.definition().cast::<IfThenElse>().unwrap();
			let branches: Vec<_> = ite.branches().collect();
			assert_eq!(branches.len(), 2);
			assert_eq!(
				branches[0]
					.condition
					.cast_ref::<Identifier>()
					.unwrap()
					.name(),
				"a"
			);
			assert_eq!(
				branches[0].result.cast_ref::<Identifier>().unwrap().name(),
				"b"
			);
			assert_eq!(
				branches[1]
					.condition
					.cast_ref::<Identifier>()
					.unwrap()
					.name(),
				"c"
			);
			assert_eq!(
				branches[1].result.cast_ref::<Identifier>().unwrap().name(),
				"d"
			);
			assert_eq!(
				ite.else_result()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"e"
			);
		}
		{
			let asg = items[2].cast_ref::<Assignment>().unwrap();
			let ite = asg.definition().cast::<IfThenElse>().unwrap();
			let branches: Vec<_> = ite.branches().collect();
			assert_eq!(branches.len(), 1);
			assert_eq!(
				branches[0]
					.condition
					.cast_ref::<Identifier>()
					.unwrap()
					.name(),
				"a"
			);
			assert_eq!(
				branches[0].result.cast_ref::<Identifier>().unwrap().name(),
				"b"
			);
			assert!(ite.else_result().is_none());
		}
	}

	#[test]
	fn test_call() {
		let model = parse_model(
			r#"
		x = foo();
		y = foo(one, two);
		z = foo(bar)(qux);
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);
		{
			let call = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<Call>()
				.unwrap();
			assert_eq!(call.function().cast::<Identifier>().unwrap().name(), "foo");
			assert_eq!(call.arguments().count(), 0);
		}
		{
			let call = items[1]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<Call>()
				.unwrap();
			assert_eq!(call.function().cast::<Identifier>().unwrap().name(), "foo");
			let args: Vec<_> = call.arguments().collect();
			assert_eq!(args[0].cast_ref::<Identifier>().unwrap().name(), "one");
			assert_eq!(args[1].cast_ref::<Identifier>().unwrap().name(), "two");
		}
		{
			let outer = items[2]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<Call>()
				.unwrap();
			let outer_args: Vec<_> = outer.arguments().collect();
			assert_eq!(outer_args.len(), 1);
			assert_eq!(
				outer_args[0].cast_ref::<Identifier>().unwrap().name(),
				"qux"
			);
			let inner = outer.function().cast::<Call>().unwrap();
			assert_eq!(inner.function().cast::<Identifier>().unwrap().name(), "foo");
			let inner_args: Vec<_> = inner.arguments().collect();
			assert_eq!(inner_args.len(), 1);
			assert_eq!(
				inner_args[0].cast_ref::<Identifier>().unwrap().name(),
				"bar"
			);
		}
	}

	#[test]
	fn test_prefix_operator() {
		let model = parse_model("x = -a;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let op = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<PrefixOperator>()
				.unwrap();
			assert_eq!(op.operator(), "-");
			assert_eq!(op.operand().cast::<Identifier>().unwrap().name(), "a");
		}
	}

	#[test]
	fn test_infix_operator() {
		let model = parse_model(
			r#"
		x = a + b;
		y = a + b * c;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let op = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<InfixOperator>()
				.unwrap();
			assert_eq!(op.left().cast::<Identifier>().unwrap().name(), "a");
			assert_eq!(op.operator(), "+");
			assert_eq!(op.right().cast::<Identifier>().unwrap().name(), "b");
		}
		{
			let sum = items[1]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<InfixOperator>()
				.unwrap();
			assert_eq!(sum.left().cast::<Identifier>().unwrap().name(), "a");
			assert_eq!(sum.operator(), "+");
			let product = sum.right().cast::<InfixOperator>().unwrap();
			assert_eq!(product.left().cast::<Identifier>().unwrap().name(), "b");
			assert_eq!(product.operator(), "*");
			assert_eq!(product.right().cast::<Identifier>().unwrap().name(), "c");
		}
	}

	#[test]
	fn test_postfix_operator() {
		let model = parse_model("x = a..;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let op = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<PostfixOperator>()
				.unwrap();
			assert_eq!(op.operand().cast::<Identifier>().unwrap().name(), "a");
			assert_eq!(op.operator(), "..");
		}
	}

	#[test]
	fn test_generator_call() {
		let model = parse_model(
			r#"
			constraint forall (i in s) (true);
			constraint exists (i, j in s, k in t where p) (true);
			"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let call = items[0]
				.cast_ref::<Constraint>()
				.unwrap()
				.expression()
				.cast::<GeneratorCall>()
				.unwrap();
			assert_eq!(
				call.function().cast::<Identifier>().unwrap().name(),
				"forall"
			);
			let gs: Vec<_> = call.generators().collect();
			assert_eq!(gs.len(), 1);
			let g1ps: Vec<_> = gs[0].patterns().collect();
			assert_eq!(g1ps.len(), 1);
			assert_eq!(g1ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(gs[0].collection().cast::<Identifier>().unwrap().name(), "s");
			assert!(gs[0].where_clause().is_none());
			assert!(call.template().cast::<BooleanLiteral>().unwrap().value());
		}
		{
			let call = items[1]
				.cast_ref::<Constraint>()
				.unwrap()
				.expression()
				.cast::<GeneratorCall>()
				.unwrap();
			assert_eq!(
				call.function().cast::<Identifier>().unwrap().name(),
				"exists"
			);
			let gs: Vec<_> = call.generators().collect();
			assert_eq!(gs.len(), 2);
			let g1ps: Vec<_> = gs[0].patterns().collect();
			assert_eq!(g1ps.len(), 2);
			assert_eq!(g1ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(g1ps[1].cast_ref::<Identifier>().unwrap().name(), "j");
			assert_eq!(gs[0].collection().cast::<Identifier>().unwrap().name(), "s");
			assert!(gs[0].where_clause().is_none());
			let g2ps: Vec<_> = gs[1].patterns().collect();
			assert_eq!(g2ps.len(), 1);
			assert_eq!(g2ps[0].cast_ref::<Identifier>().unwrap().name(), "k");
			assert_eq!(gs[1].collection().cast::<Identifier>().unwrap().name(), "t");
			assert_eq!(
				gs[1]
					.where_clause()
					.unwrap()
					.cast::<Identifier>()
					.unwrap()
					.name(),
				"p"
			);
			assert!(call.template().cast::<BooleanLiteral>().unwrap().value());
		}
	}

	#[test]
	fn test_string_interpolation() {
		let model = parse_model(r#"x = "foo\(y)bar";"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let s = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<StringInterpolation>()
				.unwrap();
			let cs: Vec<_> = s.contents().collect();
			assert_eq!(cs.len(), 3);
			assert_eq!(cs[0].string().unwrap(), "foo");
			assert_eq!(
				cs[1]
					.expression()
					.unwrap()
					.cast_ref::<Identifier>()
					.unwrap()
					.name(),
				"y"
			);
			assert_eq!(cs[2].string().unwrap(), "bar");
		}
	}

	#[test]
	fn test_let() {
		let model = parse_model(
			r#"
			constraint let {
				var int: x;
				constraint false;
			} in true;
			"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let l = items[0]
				.cast_ref::<Constraint>()
				.unwrap()
				.expression()
				.cast::<Let>()
				.unwrap();
			let is: Vec<_> = l.items().collect();
			assert_eq!(is.len(), 2);
			let d = is[0].cast_ref::<Declaration>().unwrap();
			assert_eq!(d.pattern().cast::<Identifier>().unwrap().name(), "x");
			let c = is[1].cast_ref::<Constraint>().unwrap();
			assert!(!c.expression().cast::<BooleanLiteral>().unwrap().value());
			assert!(l.in_expression().cast::<BooleanLiteral>().unwrap().value());
		}
	}

	#[test]
	fn test_case() {
		let model = parse_model(
			r#"
			x = case a of 
					Foo(b) => true,
					_ => false
				endcase;
			"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let c = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<Case>()
				.unwrap();
			assert_eq!(c.expression().cast::<Identifier>().unwrap().name(), "a");
			let cs: Vec<_> = c.cases().collect();
			assert_eq!(cs.len(), 2);
			let p1 = cs[0].pattern().cast::<PatternCall>().unwrap();
			assert_eq!(p1.identifier().name(), "Foo");
			let p1as: Vec<_> = p1.arguments().collect();
			assert_eq!(p1as.len(), 1);
			assert_eq!(p1as[0].cast_ref::<Identifier>().unwrap().name(), "b");
			assert!(cs[0].value().cast::<BooleanLiteral>().unwrap().value());
			assert!(cs[1].pattern().cast::<Anonymous>().is_some());
			assert!(!cs[1].value().cast::<BooleanLiteral>().unwrap().value());
		}
	}

	#[test]
	fn test_tuple_access() {
		let model = parse_model("x = foo.1;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let t = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<TupleAccess>()
				.unwrap();
			assert_eq!(t.tuple().cast::<Identifier>().unwrap().name(), "foo");
			assert_eq!(t.field().value(), 1);
		}
	}

	#[test]
	fn test_record_access() {
		let model = parse_model("x = foo.bar;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let r = items[0]
				.cast_ref::<Assignment>()
				.unwrap()
				.definition()
				.cast::<RecordAccess>()
				.unwrap();
			assert_eq!(r.record().cast::<Identifier>().unwrap().name(), "foo");
			assert_eq!(r.field().name(), "bar");
		}
	}
}
