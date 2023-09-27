//! AST Representation for containers

use super::{Children, Expression};
use crate::syntax::ast::{
	ast_node, children_with_field_name, optional_child_with_field_name, AstNode,
};

ast_node!(
	// Matrix Literal
	MatrixLiteral,
	members,
	index
);

impl MatrixLiteral {
	// Get the members of this matrix literal
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}

	// Get the index of this matrix literal
	pub fn index(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}
}
