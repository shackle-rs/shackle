//! Lowering of array literals
//!

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect 1D array literal
	pub(super) fn collect_array_literal(
		&mut self,
		al: &shackle_hir::ArrayLiteral<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		alloc_expression(
			ArrayLiteral(
				al.members
					.iter()
					.map(|m| self.collect_expression(*m))
					.collect(),
			),
			self,
			origin,
		)
	}

	/// Collect 2D array literal
	pub(super) fn collect_array_literal_2d(
		&mut self,
		al: &shackle_hir::ArrayLiteral2D<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin = origin.into();
		// Desugar 2D array literal into array2d call
		let mut idx_array = |dim: &shackle_hir::MaybeIndexSet<'db>| match dim {
			shackle_hir::MaybeIndexSet::Indexed(es) => alloc_expression(
				ArrayLiteral(es.iter().map(|e| self.collect_expression(*e)).collect()),
				self,
				origin,
			),
			shackle_hir::MaybeIndexSet::NonIndexed(c) => alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.set2array.into(),
					arguments: vec![if *c > 0 {
						alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.dot_dot.into(),
								arguments: vec![
									alloc_expression(IntegerLiteral(1), self, origin),
									alloc_expression(IntegerLiteral(*c as i64), self, origin),
								],
							},
							self,
							origin,
						)
					} else {
						alloc_expression(SetLiteral(Vec::new()), self, origin)
					}],
				},
				self,
				origin,
			),
		};
		let rows = idx_array(&al.rows);
		let columns = idx_array(&al.columns);
		alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.mzn_array_2d_literal.into(),
				arguments: vec![
					rows,
					columns,
					alloc_expression(
						ArrayLiteral(
							al.members
								.iter()
								.map(|e| self.collect_expression(*e))
								.collect(),
						),
						self,
						origin,
					),
				],
			},
			self,
			origin,
		)
	}

	/// Collect indexed array literal
	pub(super) fn collect_indexed_array_literal(
		&mut self,
		al: &shackle_hir::IndexedArrayLiteral<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin = origin.into();
		// Desugar indexed array literal into arrayNd call
		if al.indices.len() == 1 {
			alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.mzn_start_indexed_array.into(),
					arguments: vec![
						self.collect_expression(al.indices[0]),
						alloc_expression(
							ArrayLiteral(
								al.members
									.iter()
									.map(|e| self.collect_expression(*e))
									.collect(),
							),
							self,
							origin,
						),
					],
				},
				self,
				origin,
			)
		} else {
			alloc_expression(
				LookupCall {
					function: self.parent.ids.builtins.mzn_indexed_array.into(),
					arguments: vec![alloc_expression(
						ArrayLiteral(
							al.indices
								.iter()
								.zip(al.members.iter())
								.map(|(i, e)| {
									alloc_expression(
										TupleLiteral(vec![
											self.collect_expression(*i),
											self.collect_expression(*e),
										]),
										self,
										origin,
									)
								})
								.collect(),
						),
						self,
						origin,
					)],
				},
				self,
				origin,
			)
		}
	}
}
