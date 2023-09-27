//! Eprime Domain Expressions

use super::{Children, Expression};
use crate::syntax::ast::{
	ast_enum, ast_node, children_with_field_name, optional_child_with_field_name, AstNode,
};

ast_enum!(
	Domain,
	"boolean_domain" => BooleanDomain,
	"integer_domain" => IntegerDomain,
);

ast_node!(
	/// Boolean domain
	BooleanDomain,
);

impl BooleanDomain {}

ast_node!(
	/// Integer domain
	IntegerDomain,
	range_members,
);

impl IntegerDomain {
	/// Get the range expressions of domain
	pub fn range_members(&self) -> Children<'_, RangeMember> {
		children_with_field_name(self, "member")
	}
}

ast_enum!(
	RangeMember,
	".." => RangeLiteral,
	_ => Expression,
);

ast_node!(
	/// Range literal
	RangeLiteral,
	min,
	max,
);

impl RangeLiteral {
	/// Get the minimum value of this range
	pub fn min(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "min")
	}

	/// Get the maximum value of this range
	pub fn max(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "max")
	}
}

#[cfg(test)]
mod test {}
