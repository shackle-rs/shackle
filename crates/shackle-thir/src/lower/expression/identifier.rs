//! Lowering of identifiers

use shackle_hir::{
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef, NodeRef},
};

use crate::{
	lower::{
		LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Lower an identifier reference, projecting a class-typed root into the
	/// identity universe expected at this position.
	pub(super) fn collect_identifier(
		&mut self,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let res = self.types.name_resolution(idx).unwrap();
		let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
			let e = ExpressionRef::new(db, self.item, idx);
			panic!(
				"Did not lower {:?} at {:?} used by {:?} at {:?}",
				res,
				res.into_entity(self.parent.db).source_span(self.parent.db),
				e,
				e.source_span(self.parent.db),
			)
		});
		let expr = alloc_expression(
			match ident {
				LoweredIdentifier::ResolvedIdentifier(i) => i.clone(),
				_ => unreachable!(),
			},
			self,
			origin,
		);

		if self.lowered_ty_matches(expr.ty(), ty) {
			expr
		} else if let (Some(source_class), Some(target_class)) =
			(expr.ty().class_type(db), ty.class_type(db))
		{
			let source_class =
				class_pattern_for(db, source_class).expect("class item for class type");
			let target_class =
				class_pattern_for(db, target_class).expect("class item for class type");
			let source_occurrence = self
				.parent
				.objects
				.plan
				.top_level_occurrences
				.get(&res)
				.copied();
			if let Some(occurrence) = source_occurrence
				&& source_class != target_class
				&& expr.ty().is_subtype_of(db, ty)
			{
				self.project_class_identity(expr, occurrence, source_class, target_class, origin)
			} else if source_class == target_class {
				// A same-class label mismatch: a class-labeled
				// reference where the enum lowering is expected.
				// RELABEL to the potential-enum form of the
				// expression's own type: the underlying value is
				// already a `<A>_potential` member, so only the label
				// changes. The enum label (never `Class<A>`) is what
				// the transform pipeline's function instantiation and
				// type propagation expect; the inst is left alone —
				// a var choice of object stays var.
				let relabel_ty = self.parent.substitute_class_with_potential_enum(expr.ty());
				let mut relabeled =
					Expression::new_unchecked(relabel_ty, (*expr).clone(), expr.origin());
				relabeled
					.annotations_mut()
					.extend(expr.annotations().iter().cloned());
				relabeled
			} else {
				// Cross-class coercion of a bare reference with no
				// top-level occurrence to project through. Reaching this
				// requires an *upcast reference* (`var Sub: r` read
				// where `var Sup` is expected) whose identity would need
				// an ordinal correction between the two potential
				// universes — which only `project_class_identity` can
				// supply, and it needs an occurrence. No such shape
				// exists today: upcast projection of a root goes through
				// `collect_expression_as`, and bare references are
				// same-class (handled above). Kept as a loud panic so a
				// future cross-class reference surfaces here rather than
				// silently emitting a mis-mapped identity.
				unreachable!(
					"class-typed identifier coercion: {:?} at {:?} expected {} but lowered as {}; source {:?} target {:?} top-level occurrence present: {}",
					res,
					NodeRef::from(EntityRef::new(
						self.parent.db,
						self.item,
						shackle_hir::ids::EntityId::from(idx)
					))
					.source_span(self.parent.db),
					ty.pretty_print(db),
					expr.ty().pretty_print(db),
					source_class.identifier(db),
					target_class.identifier(db),
					source_occurrence.is_some(),
				)
			}
		} else {
			assert!(
				self.lowered_ty_matches(expr.ty().make_par(db), ty),
				"identifier {:?} at {:?} expected {} but lowered as {}",
				res,
				NodeRef::from(EntityRef::new(
					self.parent.db,
					self.item,
					shackle_hir::ids::EntityId::from(idx)
				))
				.source_span(self.parent.db),
				ty.pretty_print(db),
				expr.ty().pretty_print(db),
			);
			// Lowered is var (a var-storage class field reached through
			// a var-new path) but the HIR-typer kept the reference par —
			// e.g. a bare attribute name in a class constraint, where
			// `this`'s class type is unvarified. Let the var-ness flow:
			// relabelling to the par HIR type would not survive a
			// transform fold (identifier types are re-derived from
			// their declarations), and `fix()` fails at runtime on a
			// genuine var decision. Calls over the widened value
			// re-dispatch by name to their var overloads.
			expr
		}
	}
}
