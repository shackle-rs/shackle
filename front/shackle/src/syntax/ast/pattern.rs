//! AST representation of destructuring patterns

use super::{
	helpers::*, Absent, Anonymous, AstNode, BoolLiteral, Children, FloatLiteral, Identifier,
	Infinity, IntegerLiteral, StringLiteral,
};

ast_enum!(
	/// A pattern for (future) destructuring.
	Pattern,
	"identifier" | "quoted_identifier" => Identifier,
	"anonymous" => Anonymous,
	"absent" => Absent,
	"boolean_literal" => BoolLiteral,
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
