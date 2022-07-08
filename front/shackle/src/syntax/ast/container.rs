//! AST Representation for containers

use super::{helpers::*, Identifier};
use super::{AstNode, Children, Expression, Pattern};

ast_node!(
	/// Tuple literal
	TupleLiteral,
	members,
);

impl TupleLiteral {
	/// Get the values in this tuple literal
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Record literal
	RecordLiteral,
	members,
);

impl RecordLiteral {
	/// Get the values in this record literal
	pub fn members(&self) -> Children<'_, RecordLiteralMember> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Record literal key-value pair
	RecordLiteralMember,
	name,
	value
);

impl RecordLiteralMember {
	/// Get the name of this member
	pub fn name(&self) -> Identifier {
		child_with_field_name(self, "name")
	}

	/// Get the value of this member
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}
}

ast_node!(
	/// Set literal
	SetLiteral,
	members
);

impl SetLiteral {
	/// Get the values in this set literal
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array literal
	ArrayLiteral,
	members
);

impl ArrayLiteral {
	/// Get the members of this array literal
	pub fn members(&self) -> Children<'_, ArrayLiteralMember> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array literal member (indices if present and value)
	ArrayLiteralMember,
	indices,
	value
);

impl ArrayLiteralMember {
	/// Get the indices for this member
	pub fn indices(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// Get the value of this member
	pub fn value(&self) -> Expression {
		child_with_field_name(self, "value")
	}
}

ast_node!(
	/// 2D array literal
	ArrayLiteral2D,
	column_indices,
	rows
);

impl ArrayLiteral2D {
	/// Get the column indices if any
	pub fn column_indices(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "column_index")
	}

	/// Get the rows in this 2D array literal
	pub fn rows(&self) -> Children<'_, ArrayLiteral2DRow> {
		children_with_field_name(self, "row")
	}
}

ast_node!(
	/// 2D array literal row
	ArrayLiteral2DRow,
	index,
	members
);

impl ArrayLiteral2DRow {
	/// Get the row index if present
	pub fn index(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// Get the values in this 2D array literal row
	pub fn members(&self) -> Children<'_, Expression> {
		children_with_field_name(self, "member")
	}
}

ast_node!(
	/// Array access
	ArrayAccess,
	collection,
	indices
);

impl ArrayAccess {
	/// The array being indexed
	pub fn collection(&self) -> Expression {
		child_with_field_name(self, "collection")
	}

	/// Get the indices
	pub fn indices(&self) -> Children<'_, ArrayIndex> {
		children_with_field_name(self, "index")
	}
}

ast_enum!(
	/// Array index (could be `..` or an expression)
	ArrayIndex,
	".." | "<.." | "<..<" => IndexSlice,
	_ => Expression
);

ast_node!(
	/// Array index slice
	IndexSlice,
	operator,
);

impl IndexSlice {
	/// Get the operator
	pub fn operator(&self) -> &str {
		let node = self.cst_node().as_ref();
		node.kind()
	}
}

ast_node!(
	/// Array comprehension
	ArrayComprehension,
	indices,
	template,
	generators
);

impl ArrayComprehension {
	/// The indices for the body of this comprehension
	pub fn indices(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "index")
	}

	/// The body of this comprehension
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}

	/// The generators for this comprehension
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}
}

ast_node!(
	/// Set comprehension
	SetComprehension,
	template,
	generators
);

impl SetComprehension {
	/// The body of this comprehension
	pub fn template(&self) -> Expression {
		child_with_field_name(self, "template")
	}
	/// The generators for this comprehension
	pub fn generators(&self) -> Children<'_, Generator> {
		children_with_field_name(self, "generator")
	}
}

ast_node!(
	/// Generator for a comprehension
	Generator,
	patterns,
	collection,
	where_clause
);

impl Generator {
	/// Patterns (variable names)
	pub fn patterns(&self) -> Children<'_, Pattern> {
		children_with_field_name(self, "name")
	}

	/// Expression being iterated over
	pub fn collection(&self) -> Expression {
		child_with_field_name(self, "collection")
	}

	/// Where clause constraining interation
	pub fn where_clause(&self) -> Option<Expression> {
		optional_child_with_field_name(self, "where")
	}
}

#[cfg(test)]
mod test {
	use crate::syntax::ast::helpers::test::*;
	use crate::syntax::ast::*;

	#[test]
	fn test_tuple_literal() {
		let model = parse_model(
			r#"
		x = (1, 2);
		y = (1, (2, 3));
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_tl = x.definition().cast::<TupleLiteral>().unwrap();
			let x_members: Vec<_> = x_tl.members().collect();
			assert_eq!(x_members.len(), 2);
			assert_eq!(
				x_members[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			assert_eq!(
				x_members[1].cast_ref::<IntegerLiteral>().unwrap().value(),
				2
			);
		}

		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_tl = y.definition().cast::<TupleLiteral>().unwrap();
			let y_members: Vec<_> = y_tl.members().collect();
			assert_eq!(y_members.len(), 2);
			assert_eq!(
				y_members[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			let y2_members: Vec<_> = y_members[1]
				.cast_ref::<TupleLiteral>()
				.unwrap()
				.members()
				.collect();
			assert_eq!(y2_members.len(), 2);
			assert_eq!(
				y2_members[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				2
			);
			assert_eq!(
				y2_members[1].cast_ref::<IntegerLiteral>().unwrap().value(),
				3
			);
		}
	}

	#[test]
	fn test_record_literal() {
		let model = parse_model(
			r#"
		x = (a: 1, b: 2);
		y = (a: 1, b: (c: 2, d: 3));
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 2);

		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_tl = x.definition().cast::<RecordLiteral>().unwrap();
			let x_members: Vec<_> = x_tl.members().collect();
			assert_eq!(x_members.len(), 2);
			assert_eq!(x_members[0].name().name(), "a");
			assert_eq!(
				x_members[0]
					.value()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(x_members[1].name().name(), "b");
			assert_eq!(
				x_members[1]
					.value()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				2
			);
		}
		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_tl = y.definition().cast::<RecordLiteral>().unwrap();
			let y_members: Vec<_> = y_tl.members().collect();
			assert_eq!(y_members.len(), 2);
			assert_eq!(y_members[0].name().name(), "a");
			assert_eq!(
				y_members[0]
					.value()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(y_members[1].name().name(), "b");
			let y2_members: Vec<_> = y_members[1]
				.value()
				.cast_ref::<RecordLiteral>()
				.unwrap()
				.members()
				.collect();
			assert_eq!(y2_members.len(), 2);
			assert_eq!(y2_members[0].name().name(), "c");
			assert_eq!(
				y2_members[0]
					.value()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				2
			);
			assert_eq!(y2_members[1].name().name(), "d");
			assert_eq!(
				y2_members[1]
					.value()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				3
			);
		}
	}

	#[test]
	fn test_set_literal() {
		let model = parse_model("x = {1, 2};");

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 1);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_sl = x.definition().cast::<SetLiteral>().unwrap();

			let x_members: Vec<_> = x_sl.members().collect();
			assert_eq!(x_members.len(), 2);

			assert_eq!(
				x_members[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			assert_eq!(
				x_members[1].cast_ref::<IntegerLiteral>().unwrap().value(),
				2
			);
		}
	}

	#[test]
	fn test_array_literal() {
		let model = parse_model(
			r#"
		x = [1, 3];
		y = [2: 1, 3];
		z = [0: 1, 1: 3];
		w = [(1, 1): 1, (1, 2): 3];
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 4);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_al = x.definition().cast::<ArrayLiteral>().unwrap();
			let x_members: Vec<_> = x_al.members().collect();
			assert_eq!(x_members.len(), 2);
			assert_eq!(
				x_members[0]
					.value()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(
				x_members[1]
					.value()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				3
			);
		}
		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_al = y.definition().cast::<ArrayLiteral>().unwrap();
			let y_members: Vec<_> = y_al.members().collect();
			assert_eq!(y_members.len(), 2);
			let y0 = &y_members[0];
			assert_eq!(
				y0.indices()
					.unwrap()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				2
			);
			assert_eq!(y0.value().cast::<IntegerLiteral>().unwrap().value(), 1);
			let y1 = &y_members[1];
			assert!(y1.indices().is_none());
			assert_eq!(y1.value().cast::<IntegerLiteral>().unwrap().value(), 3);
		}
		{
			let z = items[2].cast_ref::<Assignment>().unwrap();
			assert_eq!(z.assignee().cast::<Identifier>().unwrap().name(), "z");
			let z_al = z.definition().cast::<ArrayLiteral>().unwrap();
			let z_members: Vec<_> = z_al.members().collect();
			assert_eq!(z_members.len(), 2);
			let z0 = &z_members[0];
			assert_eq!(
				z0.indices()
					.unwrap()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				0
			);
			assert_eq!(z0.value().cast::<IntegerLiteral>().unwrap().value(), 1);
			let z1 = &z_members[1];
			assert_eq!(
				z1.indices()
					.unwrap()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(z1.value().cast::<IntegerLiteral>().unwrap().value(), 3);
		}
		{
			let w = items[3].cast_ref::<Assignment>().unwrap();
			assert_eq!(w.assignee().cast::<Identifier>().unwrap().name(), "w");
			let w_al = w.definition().cast::<ArrayLiteral>().unwrap();
			let w_members: Vec<_> = w_al.members().collect();
			assert_eq!(w_members.len(), 2);
			let w0 = &w_members[0];
			let w0_indices: Vec<_> = w0
				.indices()
				.unwrap()
				.cast::<TupleLiteral>()
				.unwrap()
				.members()
				.collect();
			assert_eq!(w0_indices.len(), 2);
			assert_eq!(
				w0_indices[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			assert_eq!(
				w0_indices[1].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			assert_eq!(w0.value().cast::<IntegerLiteral>().unwrap().value(), 1);
			let w1 = &w_members[1];
			let w1_indices: Vec<_> = w1
				.indices()
				.unwrap()
				.cast::<TupleLiteral>()
				.unwrap()
				.members()
				.collect();
			assert_eq!(w1_indices.len(), 2);
			assert_eq!(
				w1_indices[0].cast_ref::<IntegerLiteral>().unwrap().value(),
				1
			);
			assert_eq!(
				w1_indices[1].cast_ref::<IntegerLiteral>().unwrap().value(),
				2
			);
			assert_eq!(w1.value().cast::<IntegerLiteral>().unwrap().value(), 3);
		}
	}

	#[test]
	fn test_2d_array_literal() {
		let model = parse_model(
			r#"
		x = [| 1, 2
		     | 3, 4 |];
		y = [| 1: 2:
		     | 1, 2 |];
		z = [|    1: 2: |
		     | 1: 1, 2 |];
		w = [| 1: 1, 2 |];
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 4);

		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_al = x.definition().cast::<ArrayLiteral2D>().unwrap();
			let x_rows: Vec<_> = x_al.rows().collect();
			assert_eq!(x_rows.len(), 2);

			let x0: Vec<_> = x_rows[0].members().collect();
			assert_eq!(x0.len(), 2);
			assert_eq!(x0[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(x0[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);

			let x1: Vec<_> = x_rows[1].members().collect();
			assert_eq!(x1.len(), 2);
			assert_eq!(x1[0].cast_ref::<IntegerLiteral>().unwrap().value(), 3);
			assert_eq!(x1[1].cast_ref::<IntegerLiteral>().unwrap().value(), 4);
		}

		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_al = y.definition().cast::<ArrayLiteral2D>().unwrap();

			let y_cols: Vec<_> = y_al.column_indices().collect();
			assert_eq!(y_cols.len(), 2);
			assert_eq!(y_cols[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(y_cols[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);

			let y_rows: Vec<_> = y_al.rows().collect();
			assert_eq!(y_rows.len(), 1);

			let y0: Vec<_> = y_rows[0].members().collect();
			assert_eq!(y0.len(), 2);
			assert_eq!(y0[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(y0[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);
		}
		{
			let z = items[2].cast_ref::<Assignment>().unwrap();
			assert_eq!(z.assignee().cast::<Identifier>().unwrap().name(), "z");
			let z_al = z.definition().cast::<ArrayLiteral2D>().unwrap();

			let z_cols: Vec<_> = z_al.column_indices().collect();
			assert_eq!(z_cols.len(), 2);
			assert_eq!(z_cols[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(z_cols[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);

			let z_rows: Vec<_> = z_al.rows().collect();
			assert_eq!(z_rows.len(), 1);

			assert_eq!(
				z_rows[0]
					.index()
					.unwrap()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			let z0: Vec<_> = z_rows[0].members().collect();
			assert_eq!(z0.len(), 2);
			assert_eq!(z0[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(z0[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);
		}
		{
			let w = items[3].cast_ref::<Assignment>().unwrap();
			assert_eq!(w.assignee().cast::<Identifier>().unwrap().name(), "w");
			let w_al = w.definition().cast::<ArrayLiteral2D>().unwrap();

			assert!(w_al.column_indices().next().is_none());

			let w_rows: Vec<_> = w_al.rows().collect();
			assert_eq!(w_rows.len(), 1);

			assert_eq!(
				w_rows[0]
					.index()
					.unwrap()
					.cast::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			let w0: Vec<_> = w_rows[0].members().collect();
			assert_eq!(w0.len(), 2);
			assert_eq!(w0[0].cast_ref::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(w0[1].cast_ref::<IntegerLiteral>().unwrap().value(), 2);
		}
	}

	#[test]
	fn test_array_access() {
		let model = parse_model(
			r#"
		x = foo[1];
		y = foo[1, 2];
		z = foo[1, .., 3..];
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);

		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_aa = x.definition().cast::<ArrayAccess>().unwrap();
			assert_eq!(
				x_aa.collection().cast::<Identifier>().unwrap().name(),
				"foo"
			);
			let x_idxs: Vec<_> = x_aa.indices().collect();
			assert_eq!(x_idxs.len(), 1);
			assert_eq!(
				x_idxs[0]
					.cast_ref::<Expression>()
					.unwrap()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
		}
		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_aa = y.definition().cast::<ArrayAccess>().unwrap();
			assert_eq!(
				y_aa.collection().cast::<Identifier>().unwrap().name(),
				"foo"
			);
			let y_idxs: Vec<_> = y_aa.indices().collect();
			assert_eq!(y_idxs.len(), 2);
			assert_eq!(
				y_idxs[0]
					.cast_ref::<Expression>()
					.unwrap()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(
				y_idxs[1]
					.cast_ref::<Expression>()
					.unwrap()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				2
			);
		}
		{
			let z = items[2].cast_ref::<Assignment>().unwrap();
			assert_eq!(z.assignee().cast::<Identifier>().unwrap().name(), "z");
			let z_aa = z.definition().cast::<ArrayAccess>().unwrap();
			assert_eq!(
				z_aa.collection().cast::<Identifier>().unwrap().name(),
				"foo"
			);
			let z_idxs: Vec<_> = z_aa.indices().collect();
			assert_eq!(z_idxs.len(), 3);
			assert_eq!(
				z_idxs[0]
					.cast_ref::<Expression>()
					.unwrap()
					.cast_ref::<IntegerLiteral>()
					.unwrap()
					.value(),
				1
			);
			assert_eq!(z_idxs[1].cast_ref::<IndexSlice>().unwrap().operator(), "..");
			let z_op = z_idxs[2]
				.cast_ref::<Expression>()
				.unwrap()
				.cast_ref::<PostfixOperator>()
				.unwrap();
			assert_eq!(z_op.operand().cast::<IntegerLiteral>().unwrap().value(), 3);
			assert_eq!(z_op.operator(), "..");
		}
	}

	#[test]
	fn test_array_comprehension() {
		let model = parse_model(
			r#"
		x = [1 | i in s];
		y = [i: v | i in 1..3, j in s where i < j];
		z = [(i, j): v | i, j in s];
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_c = x.definition().cast::<ArrayComprehension>().unwrap();
			assert!(x_c.indices().is_none());
			assert_eq!(x_c.template().cast::<IntegerLiteral>().unwrap().value(), 1);
			let x_gs: Vec<_> = x_c.generators().collect();
			assert_eq!(x_gs.len(), 1);
			let x_g0_ps: Vec<_> = x_gs[0].patterns().collect();
			assert_eq!(x_g0_ps.len(), 1);
			assert_eq!(x_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(
				x_gs[0].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			assert!(x_gs[0].where_clause().is_none());
		}
		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_c = y.definition().cast::<ArrayComprehension>().unwrap();
			assert_eq!(
				y_c.indices().unwrap().cast::<Identifier>().unwrap().name(),
				"i"
			);
			assert_eq!(y_c.template().cast::<Identifier>().unwrap().name(), "v");
			let y_gs: Vec<_> = y_c.generators().collect();
			assert_eq!(y_gs.len(), 2);
			let y_g0_ps: Vec<_> = y_gs[0].patterns().collect();
			assert_eq!(y_g0_ps.len(), 1);
			assert_eq!(y_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			let y_g0_c = y_gs[0].collection().cast::<InfixOperator>().unwrap();
			assert_eq!(y_g0_c.left().cast::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(y_g0_c.operator(), "..");
			assert_eq!(y_g0_c.right().cast::<IntegerLiteral>().unwrap().value(), 3);
			assert!(y_gs[0].where_clause().is_none());
			let y_g1_ps: Vec<_> = y_gs[1].patterns().collect();
			assert_eq!(y_g1_ps.len(), 1);
			assert_eq!(y_g1_ps[0].cast_ref::<Identifier>().unwrap().name(), "j");
			assert_eq!(
				y_gs[1].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			let y_g1_w = y_gs[1]
				.where_clause()
				.unwrap()
				.cast::<InfixOperator>()
				.unwrap();
			assert_eq!(y_g1_w.left().cast::<Identifier>().unwrap().name(), "i");
			assert_eq!(y_g1_w.operator(), "<");
			assert_eq!(y_g1_w.right().cast::<Identifier>().unwrap().name(), "j");
		}
		{
			let z = items[2].cast_ref::<Assignment>().unwrap();
			assert_eq!(z.assignee().cast::<Identifier>().unwrap().name(), "z");
			let z_c = z.definition().cast::<ArrayComprehension>().unwrap();
			let z_idxs: Vec<_> = z_c
				.indices()
				.unwrap()
				.cast::<TupleLiteral>()
				.unwrap()
				.members()
				.collect();
			assert_eq!(z_idxs.len(), 2);
			assert_eq!(z_idxs[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(z_idxs[1].cast_ref::<Identifier>().unwrap().name(), "j");

			assert_eq!(z_c.template().cast::<Identifier>().unwrap().name(), "v");
			let z_gs: Vec<_> = z_c.generators().collect();
			assert_eq!(z_gs.len(), 1);
			let z_g0_ps: Vec<_> = z_gs[0].patterns().collect();
			assert_eq!(z_g0_ps.len(), 2);
			assert_eq!(z_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(z_g0_ps[1].cast_ref::<Identifier>().unwrap().name(), "j");
			assert_eq!(
				z_gs[0].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			assert!(z_gs[0].where_clause().is_none());
		}
	}

	#[test]
	fn test_set_comprehension() {
		let model = parse_model(
			r#"
		x = {v | i in s};
		y = {v | i in 1..3, j in s where i < j};
		z = {v | i, j in s};
		"#,
		);

		let items: Vec<_> = model.items().collect();
		assert_eq!(items.len(), 3);
		{
			let x = items[0].cast_ref::<Assignment>().unwrap();
			assert_eq!(x.assignee().cast::<Identifier>().unwrap().name(), "x");
			let x_c = x.definition().cast::<SetComprehension>().unwrap();
			assert_eq!(x_c.template().cast::<Identifier>().unwrap().name(), "v");
			let x_gs: Vec<_> = x_c.generators().collect();
			assert_eq!(x_gs.len(), 1);
			let x_g0_ps: Vec<_> = x_gs[0].patterns().collect();
			assert_eq!(x_g0_ps.len(), 1);
			assert_eq!(x_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(
				x_gs[0].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			assert!(x_gs[0].where_clause().is_none());
		}
		{
			let y = items[1].cast_ref::<Assignment>().unwrap();
			assert_eq!(y.assignee().cast::<Identifier>().unwrap().name(), "y");
			let y_c = y.definition().cast::<SetComprehension>().unwrap();
			assert_eq!(y_c.template().cast::<Identifier>().unwrap().name(), "v");
			let y_gs: Vec<_> = y_c.generators().collect();
			assert_eq!(y_gs.len(), 2);
			let y_g0_ps: Vec<_> = y_gs[0].patterns().collect();
			assert_eq!(y_g0_ps.len(), 1);
			assert_eq!(y_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			let y_g0_c = y_gs[0].collection().cast::<InfixOperator>().unwrap();
			assert_eq!(y_g0_c.left().cast::<IntegerLiteral>().unwrap().value(), 1);
			assert_eq!(y_g0_c.operator(), "..");
			assert_eq!(y_g0_c.right().cast::<IntegerLiteral>().unwrap().value(), 3);
			assert!(y_gs[0].where_clause().is_none());
			let y_g1_ps: Vec<_> = y_gs[1].patterns().collect();
			assert_eq!(y_g1_ps.len(), 1);
			assert_eq!(y_g1_ps[0].cast_ref::<Identifier>().unwrap().name(), "j");
			assert_eq!(
				y_gs[1].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			let y_g1_w = y_gs[1]
				.where_clause()
				.unwrap()
				.cast::<InfixOperator>()
				.unwrap();
			assert_eq!(y_g1_w.left().cast::<Identifier>().unwrap().name(), "i");
			assert_eq!(y_g1_w.operator(), "<");
			assert_eq!(y_g1_w.right().cast::<Identifier>().unwrap().name(), "j");
		}
		{
			let z = items[2].cast_ref::<Assignment>().unwrap();
			assert_eq!(z.assignee().cast::<Identifier>().unwrap().name(), "z");
			let z_c = z.definition().cast::<SetComprehension>().unwrap();
			assert_eq!(z_c.template().cast::<Identifier>().unwrap().name(), "v");
			let z_gs: Vec<_> = z_c.generators().collect();
			assert_eq!(z_gs.len(), 1);
			let z_g0_ps: Vec<_> = z_gs[0].patterns().collect();
			assert_eq!(z_g0_ps.len(), 2);
			assert_eq!(z_g0_ps[0].cast_ref::<Identifier>().unwrap().name(), "i");
			assert_eq!(z_g0_ps[1].cast_ref::<Identifier>().unwrap().name(), "j");
			assert_eq!(
				z_gs[0].collection().cast::<Identifier>().unwrap().name(),
				"s"
			);
			assert!(z_gs[0].where_clause().is_none());
		}
	}
}
