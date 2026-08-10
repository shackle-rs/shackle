//! Lowering of annotations on declarations
//!
//! These need special handling for calls to functions that use :: annotated_expression

use shackle_hir::ids::{ExpressionRef, NodeRef};

use crate::{
	lower::{
		LoweredAnnotation, LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect an annotation on a declaration
	pub(in crate::lower) fn collect_declaration_annotation(
		&mut self,
		decl: DeclarationId<'db>,
		ann: shackle_hir::ExpressionId<'db>,
	) -> LoweredAnnotation<'db> {
		// Declarations can have annotations which point to functions using ::annotated_expression.
		// These need to be desugared into constraints.
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
				if let LoweredIdentifier::Callable(function) = ident.clone() {
					let origin = ExpressionRef::new(self.parent.db, self.item, ann)
						.into_entity(self.parent.db);
					let ann_decl = self.introduce_declaration(
						self.parent.model[decl].top_level(),
						origin,
						|collector| {
							// Call annotation function using the annotated declaration
							let arguments = vec![alloc_expression(
								ResolvedIdentifier::Declaration(decl),
								collector,
								origin,
							)];
							alloc_expression(
								Call {
									function: function.clone(),
									arguments,
								},
								collector,
								origin,
							)
						},
					);

					let annotate = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.annotate.into(),
							arguments: vec![
								alloc_expression(
									ResolvedIdentifier::Declaration(decl),
									self,
									origin,
								),
								alloc_expression(
									ResolvedIdentifier::Declaration(ann_decl),
									self,
									origin,
								),
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

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}
			}
			shackle_hir::Expression::Call(c) => {
				let origin =
					ExpressionRef::new(self.parent.db, self.item, ann).into_entity(self.parent.db);
				let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function]
				{
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
						LoweredIdentifier::Callable(c) => c.clone(),
						_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
					}
				} else {
					Callable::Expression(Box::new(self.collect_expression(c.function)))
				};

				if let Callable::Function(f) = &function
					&& self.parent.model[*f].parameters().len() > c.arguments.len()
				{
					// Add the annotated declaration identifier as first argument
					let mut arguments = Vec::with_capacity(c.arguments.len() + 1);
					arguments.push(alloc_expression(
						ResolvedIdentifier::Declaration(decl),
						self,
						origin,
					));
					arguments.extend(c.arguments.iter().map(|arg| self.collect_expression(*arg)));

					let ann_decl = self.introduce_declaration(
						self.parent.model[decl].top_level(),
						origin,
						|collector| {
							alloc_expression(
								Call {
									function,
									arguments,
								},
								collector,
								origin,
							)
						},
					);

					let annotate = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.annotate.into(),
							arguments: vec![
								alloc_expression(
									ResolvedIdentifier::Declaration(decl),
									self,
									origin,
								),
								alloc_expression(
									ResolvedIdentifier::Declaration(ann_decl),
									self,
									origin,
								),
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

					return LoweredAnnotation::Items(vec![ann_decl.into(), c_idx.into()]);
				}

				// Return as is
				return LoweredAnnotation::Expression(alloc_expression(
					Call {
						function,
						arguments: c
							.arguments
							.iter()
							.map(|arg| self.collect_expression(*arg))
							.collect(),
					},
					self,
					origin,
				));
			}
			_ => (),
		}
		LoweredAnnotation::Expression(self.collect_expression(ann))
	}
}
