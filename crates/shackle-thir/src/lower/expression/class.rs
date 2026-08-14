//! Class-identity projection for expressions.
//!
//! A class-typed value denotes an identity in some class's potential universe.
//! When a value flows into a position expecting a different class — a `Sub`-typed
//! root used where a `Super` is expected, or a reference read at a non-root
//! position — its ordinal must be projected into the expected universe. These
//! helpers perform that projection and the postcondition checks that a lowered
//! expression's type still matches what the typechecker inferred.

use shackle_hir::{
	class_analysis::{OccurrenceId, class_pattern_for},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Whether `actual` is a valid lowered form of the typechecker's
	/// `expected` type: the lowering substitutes `Class<C>` with the
	/// `<C>_potential` enum (and par-ifies singular fresh identities), so the
	/// constructed expression's type legitimately differs from the HIR type
	/// at exactly those points.
	pub(super) fn lowered_ty_matches(&self, actual: Ty<'db>, expected: Ty<'db>) -> bool {
		if actual == expected {
			return true;
		}
		let class_pattern = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(self.parent.db, class).expect("class item for class type")
		};
		match (
			actual.lookup(self.parent.db),
			expected.lookup(self.parent.db),
		) {
			(
				TyData::Class(actual_inst, actual_opt, actual_class),
				TyData::Class(expected_inst, expected_opt, expected_class),
			) if actual_opt == expected_opt
				&& class_pattern(*actual_class) == class_pattern(*expected_class)
				&& (actual_inst == expected_inst
					|| (*actual_inst == VarType::Par && *expected_inst == VarType::Var)) =>
			{
				// par-class is a valid lowering of var-class: the identity is
				// par because the singular fresh introduction collapses it,
				// while the HIR type kept var to drive attribute varification
				// through the field-access cascade.
				true
			}
			(
				TyData::Set(actual_inst, actual_opt, actual_element),
				TyData::Set(expected_inst, expected_opt, expected_element),
			) if actual_inst == expected_inst
				&& actual_opt == expected_opt
				&& self.lowered_ty_matches(*actual_element, *expected_element) =>
			{
				true
			}
			(
				TyData::Array {
					opt: actual_opt,
					dim: actual_dim,
					element: actual_element,
				},
				TyData::Array {
					opt: expected_opt,
					dim: expected_dim,
					element: expected_element,
				},
			) if actual_opt == expected_opt
				&& actual_dim == expected_dim
				&& self.lowered_ty_matches(*actual_element, *expected_element) =>
			{
				// An `array [..] of <C>` attribute lowers its element to the
				// substituted potential enum (`array [..] of var <C>_potential`),
				// so a sibling/field read of the whole array carries the enum
				// element while the HIR keeps the class element. Recurse on the
				// element exactly as the Set arm does — the class/enum element
				// arms below absorb the `<C>_potential`↔`Class<C>` equivalence.
				// The dimension type is object-independent (a plain index set),
				// so it must match exactly.
				true
			}
			(
				TyData::Enum(actual_inst, actual_opt, actual_enum),
				TyData::Class(expected_inst, expected_opt, expected_class),
			) if actual_opt == expected_opt
				&& (actual_inst == expected_inst
					|| (*actual_inst == VarType::Par && *expected_inst == VarType::Var)) =>
			{
				// `<C>_potential` is the lowered form of `Class<C>`, with the
				// same par-for-var allowance as the Class/Class arm above (a
				// singular fresh introduction collapses the identity to par).
				//
				// Matched only at the SAME level: a set/array wrapper is the
				// business of the arms above, which recurse into the element and
				// compare inst and opt as they go.
				self.parent.model
					[self.parent.objects.class_map[&class_pattern(*expected_class)].class_enum]
					.enum_type() == *actual_enum
			}
			(
				TyData::Set(actual_inst, actual_opt, actual_element),
				TyData::Set(expected_inst, expected_opt, expected_element),
			) if actual_inst == expected_inst && actual_opt == expected_opt => {
				let Some(actual_enum) = actual_element.enum_ty(self.parent.db) else {
					return false;
				};
				let Some(expected_class) = expected_element.class_type(self.parent.db) else {
					return false;
				};
				self.parent.model
					[self.parent.objects.class_map[&class_pattern(expected_class)].class_enum]
					.enum_type() == actual_enum
			}
			_ => false,
		}
	}

	/// Whether `actual` is the same lowered shape as `expected` modulo inst
	/// and opt at every level (and the class/potential-enum identification):
	/// the loosest form of the postcondition, for shapes where a var-set
	/// comprehension lift or storage varification legitimately changed both.
	pub(super) fn lowered_shape_matches(&self, actual: Ty<'db>, expected: Ty<'db>) -> bool {
		let db = self.parent.db;
		let class_enum = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(db, class)
				.and_then(|p| self.parent.objects.class_map.get(&p))
				.map(|info| self.parent.model[info.class_enum].enum_type())
		};
		match (actual.lookup(db), expected.lookup(db)) {
			(TyData::Boolean(_, _), TyData::Boolean(_, _))
			| (TyData::Integer(_, _), TyData::Integer(_, _))
			| (TyData::Float(_, _), TyData::Float(_, _))
			| (TyData::String(_), TyData::String(_))
			| (TyData::Bottom(_), TyData::Bottom(_)) => true,
			(TyData::Enum(_, _, a), TyData::Enum(_, _, e)) => a == e,
			(TyData::Class(_, _, a), TyData::Class(_, _, e)) => {
				class_pattern_for(db, *a) == class_pattern_for(db, *e)
			}
			(TyData::Enum(_, _, a), TyData::Class(_, _, e)) => class_enum(*e) == Some(*a),
			(TyData::Class(_, _, a), TyData::Enum(_, _, e)) => class_enum(*a) == Some(*e),
			(TyData::Set(_, _, a), TyData::Set(_, _, e)) => self.lowered_shape_matches(*a, *e),
			(
				TyData::Array {
					dim: ad,
					element: ae,
					..
				},
				TyData::Array {
					dim: ed,
					element: ee,
					..
				},
			) => self.lowered_shape_matches(*ad, *ed) && self.lowered_shape_matches(*ae, *ee),
			(TyData::Tuple(_, afs), TyData::Tuple(_, efs)) => {
				afs.len() == efs.len()
					&& afs
						.iter()
						.zip(efs.iter())
						.all(|(a, e)| self.lowered_shape_matches(*a, *e))
			}
			(TyData::Record(_, afs), TyData::Record(_, efs)) => {
				afs.len() == efs.len()
					&& afs
						.iter()
						.zip(efs.iter())
						.all(|((an, a), (en, e))| an == en && self.lowered_shape_matches(*a, *e))
			}
			_ => false,
		}
	}

	pub(super) fn project_class_identity(
		&mut self,
		expr: Expression<'db>,
		source_occurrence: OccurrenceId,
		source_class: PatternRef<'db>,
		target_class: PatternRef<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let source_contribution = self
			.parent
			.occurrence_contribution(source_occurrence, source_class);
		let target_contribution = self
			.parent
			.occurrence_contribution(source_occurrence, target_class);
		let target_member = EnumMemberId::new(
			self.parent.objects.class_map[&target_class].class_enum,
			target_contribution.constructor_index as u32,
		);
		let source_constructor_index = source_contribution.constructor_index;
		let global_ordinal = alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.enum2int.into(),
				arguments: vec![expr],
			},
			self,
			origin,
		);
		let local_ordinal = if source_constructor_index == 0 {
			global_ordinal
		} else {
			let previous_end = self.parent.objects.contribution_end_map
				[&(source_class, source_constructor_index - 1)];
			let previous_end_expr =
				alloc_expression(ResolvedIdentifier::Declaration(previous_end), self, origin);
			let zero_based = alloc_expression(
				LookupCall {
					function: self.parent.ids.functions.minus.into(),
					arguments: vec![global_ordinal, previous_end_expr],
				},
				self,
				origin,
			);
			alloc_expression(
				LookupCall {
					function: self.parent.ids.builtins.plus.into(),
					arguments: vec![
						alloc_expression(IntegerLiteral(1), self, origin),
						zero_based,
					],
				},
				self,
				origin,
			)
		};
		alloc_expression(
			Call {
				function: Callable::EnumConstructor(target_member),
				arguments: vec![local_ordinal],
			},
			self,
			origin,
		)
	}

	/// The join constructor index for projecting a NON-root reference of
	/// `source_class` into `join_class`'s identity universe, or `None` when
	/// no closed-form projection exists (kept a clean type error).
	///
	/// A root operand carries a static occurrence, so `project_class_identity`
	/// can correct its ordinal per contribution. A *reference* (`var Sub: r`)
	/// holds a runtime `Sub_potential` value, so the projection must be a total
	/// map over the whole potential enum. That map is a closed form ONLY when
	/// `source_class` has a SINGLE contribution across all occurrences: then
	/// `Sub_potential` has one constructor, `enum2int(r)` is already the
	/// contribution-local 1-based ordinal (`contribution_local_ordinal` is the
	/// identity for constructor 0), and the join image is
	/// `Join_occ_ct(enum2int(r))` where `ct` is the constructor the SAME
	/// occurrence contributes to the join (its superclass image, whose slot i
	/// coincides with the direct slot i — the 1:1 mapping
	/// `superclass_projection_contribution_expr` relies on). A
	/// multi-contribution source would need a piecewise per-constructor offset
	/// map; that stays a clean type error.
	pub(super) fn reference_projection_join_constructor(
		&self,
		source_class: PatternRef<'db>,
		join_class: PatternRef<'db>,
	) -> Option<usize> {
		let mut single: Option<OccurrenceId> = None;
		for occ_contribs in self.parent.objects.plan.contributions_in_occurrence_order() {
			for contribution in occ_contribs
				.iter()
				.filter(|c| c.target_class == source_class)
			{
				// More than one contribution to the source class → its potential
				// enum has multiple constructors, so no single-constructor closed
				// form. Also require constructor 0 (a lone contribution always is),
				// so `enum2int` is the contribution-local ordinal.
				if single.replace(contribution.occurrence).is_some()
					|| contribution.constructor_index != 0
				{
					return None;
				}
			}
		}
		let occurrence = single?;
		self.parent.objects.plan.contributions_by_occurrence[&occurrence]
			.iter()
			.find(|c| c.target_class == join_class)
			.map(|c| c.constructor_index)
	}

	/// Relabel a class-labeled call operand to its potential-enum lowering.
	///
	/// Function resolution and type specialisation instantiate generic
	/// parameters from the argument types, and the standard library is typed
	/// over enums — a `var Class<B>` operand meeting a `var set of
	/// B_potential` operand fails to instantiate `in(var $$E, var set of
	/// $$E)`. The runtime value of a class-labeled expression already IS the
	/// potential-enum identity, so the relabel is cosmetic and makes every
	/// call see consistent enum labels.
	pub(super) fn relabel_class_operand(&mut self, expr: Expression<'db>) -> Expression<'db> {
		if !expr
			.ty()
			.walk(self.parent.db)
			.any(|t| t.class_type(self.parent.db).is_some())
		{
			return expr;
		}
		let enum_ty = self.parent.substitute_class_with_potential_enum(expr.ty());
		if enum_ty == expr.ty() {
			return expr;
		}
		let mut relabeled = Expression::new_unchecked(enum_ty, (*expr).clone(), expr.origin());
		relabeled
			.annotations_mut()
			.extend(expr.annotations().iter().cloned());
		relabeled
	}

	/// Project a NON-root reference `expr : var Sub` into `join_class`'s
	/// identity universe as `Join_occ_ct(enum2int(expr))`. The caller resolved
	/// `join_constructor` via `reference_projection_join_constructor`, which
	/// guarantees the single-contribution closed form.
	pub(super) fn project_reference_identity(
		&mut self,
		expr: Expression<'db>,
		join_class: PatternRef<'db>,
		join_constructor: usize,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin: Origin = origin.into();
		let global_ordinal = alloc_expression(
			LookupCall {
				function: self.parent.ids.functions.enum2int.into(),
				arguments: vec![expr],
			},
			self,
			origin,
		);
		let join_member = EnumMemberId::new(
			self.parent.objects.class_map[&join_class].class_enum,
			join_constructor as u32,
		);
		alloc_expression(
			Call {
				function: Callable::EnumConstructor(join_member),
				arguments: vec![global_ordinal],
			},
			self,
			origin,
		)
	}
}
