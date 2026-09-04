//! Lowering of annotations
//!
//! These need special handling for calls to functions that use :: annotated_expression

use shackle_hir::ids::{ExpressionRef, NodeRef};

use crate::{
	lower::{
		DeclOrConstraint, LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect an annotation value, converting it to an ann if it is a string
	pub(in crate::lower) fn collect_annotation_value(
		&mut self,
		ann: shackle_hir::ExpressionId<'db>,
		for_constraint: bool,
	) -> Expression<'db> {
		let result = self.collect_expression(ann);
		if result.ty() == self.parent.tys.string {
			return Expression::new(
				self.parent.db,
				&self.parent.model,
				result.origin(),
				LookupAnnotation {
					name: if for_constraint {
						self.parent.ids.annotations.constraint_name
					} else {
						self.parent.ids.annotations.expression_name
					},
					arguments: vec![result],
				},
			);
		}
		result
	}

	/// Return the function of an annotation call which requires the annotated expression
	pub(in crate::lower) fn get_annotated_expression_call(
		&self,
		ann: shackle_hir::ExpressionId<'db>,
	) -> Option<Callable<'db>> {
		match &self.data[ann] {
			shackle_hir::Expression::Identifier(_) => {
				let res = self.types.name_resolution(ann).unwrap();
				let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
					let e = ExpressionRef::new(self.parent.db, self.item, ann);
					panic!(
						"Did not lower {:?} at {:?} used by {:?} at {:?}",
						res,
						NodeRef::from(res.into_entity(self.parent.db)).source_span(self.parent.db),
						e,
						e.source_span(self.parent.db),
					)
				});
				if let LoweredIdentifier::Callable(c) = ident {
					return Some(c.clone());
				}
			}
			shackle_hir::Expression::Call(c) => {
				if let shackle_hir::Expression::Identifier(_) = self.data[c.function] {
					let res = self.types.name_resolution(c.function).unwrap();
					let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
						let e = ExpressionRef::new(self.parent.db, self.item, c.function);
						panic!(
							"Did not lower {:?} at {:?} used by {:?} at {:?}",
							res,
							NodeRef::from(res.into_entity(self.parent.db))
								.source_span(self.parent.db),
							e,
							e.source_span(self.parent.db),
						)
					});
					match ident {
						LoweredIdentifier::Callable(callable @ Callable::Function(f))
							if self.parent.model[*f].parameters().len() > c.arguments.len() =>
						{
							return Some(callable.clone());
						}

						LoweredIdentifier::Callable(callable @ Callable::Annotation(a))
							if self.parent.model[*a].parameters.as_ref().unwrap().len()
								> c.arguments.len() =>
						{
							return Some(callable.clone());
						}
						_ => (),
					}
				}
			}
			_ => (),
		}
		None
	}

	/// Collect an annotation on an expression which is a call requiring the annotated expression
	pub(in crate::lower) fn collect_expression_annotated_expression_call(
		&mut self,
		expression: Expression<'db>,
		ann: shackle_hir::ExpressionId<'db>,
		function: Callable<'db>,
	) -> Expression<'db> {
		let origin = ExpressionRef::new(self.parent.db, self.item, ann).into_entity(self.parent.db);
		let arguments = match &self.data[ann] {
			shackle_hir::Expression::Identifier(_) => vec![expression],
			shackle_hir::Expression::Call(c) => {
				let mut args = Vec::with_capacity(c.arguments.len() + 1);
				args.push(expression);
				args.extend(c.arguments.iter().map(|arg| self.collect_expression(*arg)));
				args
			}
			_ => unreachable!(),
		};

		return Expression::new(
			self.parent.db,
			&self.parent.model,
			origin,
			Call {
				function,
				arguments,
			},
		);
	}

	/// Collect an annotation on a declaration which is a call requiring the annotated expression
	pub(in crate::lower) fn collect_declaration_annotated_expression_call(
		&mut self,
		decl: DeclarationId<'db>,
		ann: shackle_hir::ExpressionId<'db>,
		function: Callable<'db>,
	) -> Vec<DeclOrConstraint<'db>> {
		let origin = ExpressionRef::new(self.parent.db, self.item, ann).into_entity(self.parent.db);
		let ann_decl =
			self.introduce_declaration(self.parent.model[decl].top_level(), origin, |collector| {
				// Call annotation function using the annotated declaration
				let arguments = match &self.data[ann] {
					shackle_hir::Expression::Identifier(_) => vec![alloc_expression(
						ResolvedIdentifier::Declaration(decl),
						collector,
						origin,
					)],
					shackle_hir::Expression::Call(c) => {
						let mut args = Vec::with_capacity(c.arguments.len() + 1);
						args.push(alloc_expression(
							ResolvedIdentifier::Declaration(decl),
							collector,
							origin,
						));
						args.extend(
							c.arguments
								.iter()
								.map(|arg| collector.collect_expression(*arg)),
						);
						args
					}
					_ => unreachable!(),
				};

				alloc_expression(
					Call {
						function,
						arguments,
					},
					collector,
					origin,
				)
			});

		let annotate = alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.annotate.into(),
				arguments: vec![
					alloc_expression(ResolvedIdentifier::Declaration(decl), self, origin),
					alloc_expression(ResolvedIdentifier::Declaration(ann_decl), self, origin),
				],
			},
			self,
			origin,
		);
		let constraint = Constraint::new(self.parent.model[decl].top_level(), annotate);
		let c_idx = self
			.parent
			.model
			.add_constraint(ConstraintItem::new(constraint, origin));
		vec![ann_decl.into(), c_idx.into()]
	}
}
