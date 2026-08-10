//! Lowering array access and slicing
//!
//! Array access and slicing are rewritten into calls to `[]`

use shackle_hir::ids::ExpressionRef;
use shackle_ty::TyData;

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect an array access or slice and rewrite as call to `[]`
	///
	pub(in crate::lower) fn collect_array_access(
		&mut self,
		aa: &shackle_hir::ArrayAccess<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin = origin.into();
		let is_slice = match self.types[aa.indices].lookup(self.parent.db) {
			TyData::Tuple(_, fs) => fs.iter().any(|f| f.is_set(self.parent.db)),
			TyData::Set(_, _, _) => true,
			_ => false,
		};
		if is_slice {
			self.collect_slice(aa.collection, aa.indices, origin)
		} else {
			let c = self.collect_expression(aa.collection);
			let i = self.collect_expression(aa.indices);
			self.introduce_array_access(c, i, origin)
		}
	}

	/// Introduce a call to `[]`
	pub(in crate::lower) fn introduce_array_access(
		&mut self,
		collection: Expression<'db>,
		indices: Expression<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin = origin.into();
		alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.array_access.into(),
				arguments: vec![collection, indices],
			},
			self,
			origin,
		)
	}

	/// Rewrite index slicing into a call
	///
	/// Turns all indices into sets to match the slicing builtin function, and then coerces to the correct output index set.
	pub(super) fn collect_slice(
		&mut self,
		collection: shackle_hir::ExpressionId<'db>,
		indices: shackle_hir::ExpressionId<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let collection_entity =
			ExpressionRef::new(self.parent.db, self.item, collection).into_entity(self.parent.db);
		let indices_entity =
			ExpressionRef::new(self.parent.db, self.item, indices).into_entity(self.parent.db);

		let mut decls = Vec::new();
		let collection_decl = if matches!(
			&self.data[collection],
			shackle_hir::Expression::Identifier(_)
		) {
			let expr = self.collect_expression(collection);
			match &*expr {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(decl)) => *decl,
				_ => unreachable!(),
			}
		} else {
			// Add declaration to store collection
			let origin = collection_entity;
			let decl = self.introduce_declaration(false, origin, |collector| {
				collector.collect_expression(collection)
			});
			decls.push(decl);
			decl
		};
		let mut index_sets_for_infinite_slice = None;
		let array_dims = self.types[collection].dims(self.parent.db).unwrap();
		let mut slices = Vec::with_capacity(array_dims);
		match self.types[indices].lookup(self.parent.db) {
			TyData::Tuple(_, fs) => {
				if let shackle_hir::Expression::TupleLiteral(tl) = &self.data[indices] {
					for (i, (ty, e)) in fs.iter().zip(tl.fields.iter()).enumerate() {
						let index_entity = ExpressionRef::new(self.parent.db, self.item, *e)
							.into_entity(self.parent.db);
						let mut is_set = true;
						let decl = self.introduce_declaration(false, index_entity, |collector| {
							if let shackle_hir::Expression::Slice(s) = &collector.data[*e] {
								// Rewrite infinite slice .. into `'..'(index_set_mofn(c))`
								if index_sets_for_infinite_slice.is_none() {
									let decl = collector.introduce_declaration(
										false,
										origin,
										|collector| {
											alloc_expression(
												LookupCall {
													function: self
														.parent
														.ids
														.functions
														.index_sets
														.into(),
													arguments: vec![alloc_expression(
														collection_decl,
														collector,
														collection_entity,
													)],
												},
												collector,
												origin,
											)
										},
									);
									decls.push(decl);
									index_sets_for_infinite_slice = Some(decl);
								}
								alloc_expression(
									LookupCall {
										function: (*s).into(),
										arguments: vec![alloc_expression(
											TupleAccess {
												tuple: Box::new(alloc_expression(
													index_sets_for_infinite_slice.unwrap(),
													collector,
													index_entity,
												)),
												field: IntegerLiteral(i as i64 + 1),
											},
											collector,
											index_entity,
										)],
									},
									collector,
									index_entity,
								)
							} else if ty.is_set(collector.parent.db) {
								// Slice
								collector.collect_expression(*e)
							} else {
								// Rewrite index as slice of {i}
								is_set = false;
								alloc_expression(
									SetLiteral(vec![collector.collect_expression(*e)]),
									collector,
									index_entity,
								)
							}
						});
						slices.push((decl, is_set, index_entity));
						decls.push(decl);
					}
				} else {
					// Expression which evaluates to a tuple
					let indices_decl =
						self.introduce_declaration(false, indices_entity, |collector| {
							collector.collect_expression(indices)
						});
					decls.push(indices_decl);
					for (i, f) in fs.iter().enumerate() {
						// Create declaration for each index
						let is_set = f.is_set(self.parent.db);
						let accessor =
							self.introduce_declaration(false, indices_entity, |collector| {
								let ta = alloc_expression(
									TupleAccess {
										tuple: Box::new(alloc_expression(
											indices_decl,
											collector,
											indices_entity,
										)),
										field: IntegerLiteral(i as i64 + 1),
									},
									collector,
									indices_entity,
								);
								if is_set {
									ta
								} else {
									// Rewrite as {i}
									alloc_expression(
										SetLiteral(vec![ta]),
										collector,
										indices_entity,
									)
								}
							});

						slices.push((accessor, is_set, indices_entity));
						decls.push(accessor);
					}
				}
			}
			_ => {
				// 1D slicing, so must be a set index
				let decl = self.introduce_declaration(false, indices_entity, |collector| {
					if let shackle_hir::Expression::Slice(s) = &collector.data[indices] {
						// Rewrite infinite slice .. into `'..'(index_set(c))`
						alloc_expression(
							LookupCall {
								function: (*s).into(),
								arguments: vec![alloc_expression(
									LookupCall {
										function: collector.parent.ids.functions.index_set.into(),
										arguments: vec![alloc_expression(
											collection_decl,
											collector,
											collection_entity,
										)],
									},
									collector,
									indices_entity,
								)],
							},
							collector,
							indices_entity,
						)
					} else {
						collector.collect_expression(indices)
					}
				});
				slices.push((decl, true, indices_entity));
				decls.push(decl);
			}
		}
		let collection_ident = alloc_expression(collection_decl, self, collection_entity);
		let slice_tuple = alloc_expression(
			TupleLiteral(
				slices
					.iter()
					.map(|(decl, _, origin)| alloc_expression(*decl, self, *origin))
					.collect(),
			),
			self,
			indices_entity,
		);
		let arguments = slices
			.iter()
			.filter_map(|(decl, is_slice, origin)| {
				if *is_slice {
					Some(alloc_expression(*decl, self, *origin))
				} else {
					None
				}
			})
			.chain([alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.mzn_slice.into(),
					arguments: vec![collection_ident, slice_tuple],
				},
				self,
				origin,
			)])
			.collect::<Vec<_>>();
		alloc_expression(
			Let {
				items: decls.into_iter().map(LetItem::Declaration).collect(),
				in_expression: Box::new(alloc_expression(
					LookupCall {
						function: Identifier::new(
							self.parent.db,
							format!("array{}d", arguments.len() - 1),
						)
						.into(),
						arguments,
					},
					self,
					origin,
				)),
			},
			self,
			origin,
		)
	}
}
