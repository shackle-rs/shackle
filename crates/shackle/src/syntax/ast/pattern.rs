//! AST representation of destructuring patterns

use super::{
	helpers::*, Absent, Anonymous, AstNode, BooleanLiteral, Children, FloatLiteral, Identifier,
	Infinity, IntegerLiteral, StringLiteral,
};

ast_enum!(
	/// A pattern for (future) destructuring.
	Pattern,
	"identifier" | "quoted_identifier" => Identifier,
	"anonymous" => Anonymous,
	"absent" => Absent,
	"boolean_literal" => BooleanLiteral,
	"string_literal" => StringLiteral,
	"pattern_numeric_literal" => PatternNumericLiteral,
	"pattern_call" => Call(PatternCall),
	"pattern_tuple" => Tuple(PatternTuple),
	"pattern_record" => Record(PatternRecord)
);

ast_node!(
	/// A pattern that matches a numeric literal
	///
	/// Note that we have to deal with possible negation here because numeric
	/// literals are always positive.
	PatternNumericLiteral,
	negated,
	value
);

impl PatternNumericLiteral {
	/// Whether this literal is negative
	pub fn negated(&self) -> bool {
		self.cst_node()
			.as_ref()
			.child_by_field_name("negative")
			.is_some()
	}

	/// The underlying literal
	pub fn value(&self) -> NumericLiteral {
		child_with_field_name(self, "value")
	}
}

ast_enum!(
	/// A numeric literal
	NumericLiteral,
	"integer_literal" => IntegerLiteral,
	"float_literal" => FloatLiteral,
	"infinity" => Infinity
);

ast_node!(
	/// A pattern that matches a call
	PatternCall,
	identifier,
	arguments
);

impl PatternCall {
	/// Get the name of the function
	pub fn identifier(&self) -> Identifier {
		child_with_field_name(self, "identifier")
	}
	/// Get the arguments to this call pattern
	pub fn arguments(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "argument")
	}
}

ast_node!(
	/// A pattern that matches a tuple
	PatternTuple,
	fields
);

impl PatternTuple {
	/// Get the fields of this tuple pattern
	pub fn fields(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// A pattern that matches a record
	PatternRecord,
	fields
);

impl PatternRecord {
	/// Get the fields of this tuple pattern
	pub fn fields(&self) -> Children<'_, PatternRecordField> {
		children_with_field_name(self, "field")
	}
}

ast_node!(
	/// Field in a record pattern
	PatternRecordField,
	name,
	value
);

impl PatternRecordField {
	/// The field name being matched
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// The pattern of the field being matched
	pub fn value(&self) -> Pattern {
		child_with_field_name(self, "value")
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_patterns() {
		let model = parse_model(
			r#"
		any: (a: (p, q), b: r) = foo;
		any: v = case x of
			A => 1,
			B(x) => 2,
			C(x, D(y)) => 3,
			true => 4,
			123 => 5,
			-5.5 => 6,
			infinity => 7,
			"foo" => 8,
			<> => 9,
			_ => 10,
		endcase;
		"#,
		);
		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		let destructuring = items[0].cast_ref::<Declaration>().unwrap().pattern();
		let rfs: Vec<_> = destructuring
			.cast::<PatternRecord>()
			.unwrap()
			.fields()
			.collect();
		assert_eq!(rfs.len(), 2);
		assert_eq!(rfs[0].name().name(), "a");
		let tfs: Vec<_> = rfs[0]
			.value()
			.cast::<PatternTuple>()
			.unwrap()
			.fields()
			.collect();
		assert_eq!(tfs.len(), 2);
		assert_eq!(tfs[0].cast_ref::<Identifier>().unwrap().name(), "p");
		assert_eq!(tfs[1].cast_ref::<Identifier>().unwrap().name(), "q");
		assert_eq!(rfs[1].name().name(), "b");
		assert_eq!(rfs[1].value().cast::<Identifier>().unwrap().name(), "r");
		let cases: Vec<_> = items[1]
			.cast_ref::<Declaration>()
			.unwrap()
			.definition()
			.unwrap()
			.cast::<Case>()
			.unwrap()
			.cases()
			.collect();
		assert_eq!(cases.len(), 10);
		assert_eq!(cases[0].pattern().cast::<Identifier>().unwrap().name(), "A");
		{
			let call = cases[1].pattern().cast::<PatternCall>().unwrap();
			assert_eq!(call.identifier().name(), "B");
			let call_args: Vec<_> = call.arguments().collect();
			assert_eq!(call_args.len(), 1);
			assert_eq!(call_args[0].cast_ref::<Identifier>().unwrap().name(), "x");
		}
		{
			let call = cases[2].pattern().cast::<PatternCall>().unwrap();
			assert_eq!(call.identifier().name(), "C");
			let call_args: Vec<_> = call.arguments().collect();
			assert_eq!(call_args.len(), 2);
			assert_eq!(call_args[0].cast_ref::<Identifier>().unwrap().name(), "x");
			let inner_call = call_args[1].cast_ref::<PatternCall>().unwrap();
			assert_eq!(inner_call.identifier().name(), "D");
			let inner_call_args: Vec<_> = inner_call.arguments().collect();
			assert_eq!(inner_call_args.len(), 1);
			assert_eq!(
				inner_call_args[0].cast_ref::<Identifier>().unwrap().name(),
				"y"
			);
		}
		assert!(cases[3].pattern().cast::<BooleanLiteral>().unwrap().value());
		{
			let number = cases[4].pattern().cast::<PatternNumericLiteral>().unwrap();
			assert!(!number.negated());
			assert_eq!(
				number.value().cast::<IntegerLiteral>().unwrap().value(),
				123
			);
		}
		{
			let number = cases[5].pattern().cast::<PatternNumericLiteral>().unwrap();
			assert!(number.negated());
			assert_eq!(number.value().cast::<FloatLiteral>().unwrap().value(), 5.5);
		}
		{
			let number = cases[6].pattern().cast::<PatternNumericLiteral>().unwrap();
			assert!(!number.negated());
			let _ = number.value().cast::<Infinity>().unwrap();
		}

		assert_eq!(
			cases[7].pattern().cast::<StringLiteral>().unwrap().value(),
			"foo"
		);
		let _ = cases[8].pattern().cast::<Absent>().unwrap();
		let _ = cases[9].pattern().cast::<Anonymous>().unwrap();
	}
}
