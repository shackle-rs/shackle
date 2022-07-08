//! AST representation of primitive values

use super::AstNode;

use super::helpers::*;

ast_node!(
	/// Integer literal
	IntegerLiteral,
	value
);

impl IntegerLiteral {
	/// Get the value of this integer literal
	pub fn value(&self) -> i64 {
		self.cst_text().parse().unwrap()
	}
}

ast_node!(
	/// Float literal
	FloatLiteral,
	value
);

impl FloatLiteral {
	/// Get the value of this float literal
	pub fn value(&self) -> f64 {
		self.cst_text().parse().unwrap()
	}
}

ast_node!(
	/// Boolean literal
	BooleanLiteral,
	value
);

impl BooleanLiteral {
	/// Get the value of this boolean literal
	pub fn value(&self) -> bool {
		match self.cst_text() {
			"true" => true,
			"false" => false,
			_ => unreachable!(),
		}
	}
}

ast_node!(
	/// String literal (without interpolation)
	StringLiteral,
	value
);

impl StringLiteral {
	/// Get the value of this string literal
	pub fn value(&self) -> String {
		decode_string(self.cst_node())
	}
}

ast_node!(
	/// Absent literal `<>`
	Absent,
);

ast_node!(
	/// Infinity literal
	Infinity,
);

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_integer_literal() {
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
	fn test_float_literal() {
		let model = parse_model("x = 1.2;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let assignment = items.first().unwrap().cast_ref::<Assignment>().unwrap();
		assert_eq!(
			assignment.assignee().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let rhs = assignment.definition().cast::<FloatLiteral>().unwrap();
		assert_eq!(rhs.value(), 1.2);
	}

	#[test]
	fn test_string_literal() {
		let model = parse_model(r#"x = "foo";"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let assignment = items.first().unwrap().cast_ref::<Assignment>().unwrap();
		assert_eq!(
			assignment.assignee().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let rhs = assignment.definition().cast::<StringLiteral>().unwrap();
		assert_eq!(rhs.value(), "foo");
	}

	#[test]
	fn test_absent() {
		let model = parse_model("x = <>;");
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let assignment = items.first().unwrap().cast_ref::<Assignment>().unwrap();
		assert_eq!(
			assignment.assignee().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let _ = assignment.definition().cast::<Absent>().unwrap();
	}

	#[test]
	fn test_infinity() {
		let model = parse_model(r#"x = infinity;"#);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		let assignment = items.first().unwrap().cast_ref::<Assignment>().unwrap();
		assert_eq!(
			assignment.assignee().cast::<Identifier>().unwrap().name(),
			"x"
		);
		let _ = assignment.definition().cast::<Infinity>().unwrap();
	}
}
