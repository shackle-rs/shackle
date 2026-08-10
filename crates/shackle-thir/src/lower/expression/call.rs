//! Lowering of calls to THIR
//!
//! This needs special handling for calls to class comparison operators

use rustc_hash::FxHashMap;
use shackle_hir::{
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef, PatternRef},
};
use shackle_ty::Ty;

use crate::{
	lower::{
		LoweredIdentifier,
		expression::{ExpressionCollector, alloc_expression},
	},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Lower a call, including the cross-class identity coercion applied to
	/// `=`/`!=` when the two operands' class universes differ.
	pub(super) fn collect_call_expression(
		&mut self,
		c: &shackle_hir::Call<'db>,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let function = if let shackle_hir::Expression::Identifier(_) = self.data[c.function] {
			let res = self.types.name_resolution(c.function).unwrap_or_else(|| {
				panic!(
					"No name resolution in types for {:?} at {:?}",
					c.function,
					ExpressionRef::new(self.parent.db, self.item, c.function)
						.source_span(self.parent.db)
				);
			});
			let ident = self.parent.resolutions.get(&res).unwrap_or_else(|| {
				let f = ExpressionRef::new(self.parent.db, self.item, c.function);
				panic!(
					"Did not lower {:?} at {:?} used by {:?} at {:?}",
					res,
					res.into_entity(self.parent.db).source_span(self.parent.db),
					f,
					f.source_span(self.parent.db),
				)
			});
			match ident {
				LoweredIdentifier::Callable(c) => c.clone(),
				_ => Callable::Expression(Box::new(self.collect_expression(c.function))),
			}
		} else {
			Callable::Expression(Box::new(self.collect_expression(c.function)))
		};
		// Cross-class identity coercion for the equality operators.
		// `a = b` / `a != b` with `a : var C1`, `b : var C2` where one
		// class is a subclass of the other lowers each operand into its
		// OWN potential universe (`C1_potential` vs `C2_potential`),
		// which MiniZinc rejects as an enum mismatch. The typer unifies
		// both operands to the join class; each subtype operand is
		// projected into the join's identity universe with
		// `project_class_identity` (the ordinal correction), while an
		// operand already OF the join class keeps its natural enum
		// lowering — so both operands become the same
		// `<Join>_potential` value. Projection needs a top-level
		// occurrence, so it applies to root identifiers (`= s1`); a
		// subtype operand that is not a projectable root is left as-is
		// (still a clean type error, not a crash). Gated on `=`/`!=`
		// (whose two operands share the type variable); every other
		// call keeps natural-type collection so a function with
		// genuinely distinct class parameters is never mis-projected.
		// NOTE: do NOT use `collect_expression_as` here — its
		// class-target relabel would flip the already-join operand to a
		// `Class<Join>` type, re-introducing a Class-vs-enum mismatch
		// against the projected (enum) operand.
		let eq_db = self.parent.db;
		let is_equality_op = matches!(
			&self.data[c.function],
			shackle_hir::Expression::Identifier(id)
				if *id == self.parent.ids.functions.eq
					|| *id == self.parent.ids.builtins.ne
					|| *id == self.parent.ids.builtins.lt
					|| *id == self.parent.ids.builtins.le
					|| *id == self.parent.ids.functions.gt
					|| *id == self.parent.ids.functions.ge
		);
		let class_pattern = |class: shackle_ty::ClassRef<'db>| {
			class_pattern_for(eq_db, class).expect("class item for class type")
		};
		// Resolve the join class and the per-operand occurrence to
		// project through. The coercion is ALL-OR-NOTHING: it applies
		// only when every operand whose class differs from the join is
		// a root identity with a top-level occurrence (the ordinal
		// correction `project_class_identity` needs) or a
		// single-contribution reference. If any cross-class operand
		// cannot be projected, leave every operand at its natural
		// lowering — a clean MiniZinc type error rather than a
		// mid-lowering THIR panic from partially-coerced operands.
		let join_class: Option<PatternRef<'db>> = if is_equality_op && c.arguments.len() == 2 {
			let arg_classes: Vec<Option<PatternRef<'db>>> = c
				.arguments
				.iter()
				.map(|arg| self.types[*arg].class_type(eq_db).map(class_pattern))
				.collect();
			let all_class = arg_classes.iter().all(|c| c.is_some());
			let distinct = match (arg_classes.first(), arg_classes.get(1)) {
				(Some(a), Some(b)) => a != b,
				_ => false,
			};
			let join = if all_class && distinct {
				Ty::most_specific_supertype(eq_db, c.arguments.iter().map(|arg| self.types[*arg]))
					.and_then(|j| j.class_type(eq_db))
					.map(class_pattern)
			} else {
				None
			};
			join.filter(|jc| {
				c.arguments
					.iter()
					.zip(arg_classes.iter())
					.all(|(arg, sc)| match sc {
						Some(source_class) if source_class != jc => {
							let is_root = self
								.types
								.name_resolution(*arg)
								.map(|res| {
									self.parent
										.objects
										.plan
										.top_level_occurrences
										.contains_key(&res)
								})
								.unwrap_or(false);
							is_root
								|| self
									.reference_projection_join_constructor(*source_class, *jc)
									.is_some()
						}
						_ => true,
					})
			})
		} else {
			None
		};
		// With no projection available and no objects introduced
		// anywhere — neither operand's class universe has a single
		// constructor — both reference domains are empty, so the
		// identities can only be compared as bare ordinals. That
		// keeps the comparison well-typed (the potential enums are
		// distinct types) and is vacuously correct over empty
		// domains.
		let ordinal_compare = join_class.is_none() && is_equality_op && c.arguments.len() == 2 && {
			let classes = c
				.arguments
				.iter()
				.map(|arg| self.types[*arg].class_type(eq_db).map(class_pattern))
				.collect::<Vec<_>>();
			classes.iter().all(|class| class.is_some())
				&& classes[0] != classes[1]
				&& classes.iter().all(|class| {
					class
						.and_then(|p| self.parent.objects.class_map.get(&p))
						.is_some_and(|info| {
							self.parent.model[info.class_enum]
								.definition()
								.is_none_or(|d| d.is_empty())
						})
				})
		};
		let mut arguments = c
			.arguments
			.iter()
			.map(|arg| {
				let expr = self.collect_expression(*arg);
				let Some(join_class) = join_class else {
					let relabeled = self.relabel_class_operand(expr);
					if ordinal_compare && relabeled.ty().enum_ty(eq_db).is_some() {
						let arg_origin = EntityRef::new(
							eq_db,
							self.item,
							shackle_hir::ids::EntityId::from(*arg),
						);
						return alloc_expression(
							LookupCall {
								function: self.parent.ids.functions.enum2int.into(),
								arguments: vec![relabeled],
							},
							self,
							arg_origin,
						);
					}
					return relabeled;
				};
				let arg_origin =
					EntityRef::new(eq_db, self.item, shackle_hir::ids::EntityId::from(*arg));
				// Project a subtype root operand into the join's identity
				// universe (ordinal correction); the result is a
				// `<Join>_potential` enum value.
				let projected = match self.types[*arg].class_type(eq_db).map(class_pattern) {
					Some(source_class) if source_class != join_class => {
						// A root operand projects through its static
						// occurrence; a non-root reference (no occurrence)
						// projects via the single-contribution closed form.
						match self.types.name_resolution(*arg).and_then(|res| {
							self.parent
								.objects
								.plan
								.top_level_occurrences
								.get(&res)
								.copied()
						}) {
							Some(occurrence) => self.project_class_identity(
								expr,
								occurrence,
								source_class,
								join_class,
								arg_origin,
							),
							None => {
								let ct = self
									.reference_projection_join_constructor(source_class, join_class)
									.expect("join filter guarantees a projectable reference");
								self.project_reference_identity(expr, join_class, ct, arg_origin)
							}
						}
					}
					_ => expr,
				};
				// Both operands must share the `<Join>_potential` enum
				// type. `project_class_identity` already yields that
				// enum, but an operand of the join class that lowered as
				// a genuine `var Class<Join>` (a par-actual reference)
				// must be RELABELLED to `var <Join>_potential`, or `=`
				// sees a Class-vs-enum mismatch. The relabel is cosmetic:
				// MiniZinc re-types the pretty-printed identifier from
				// its `var <Join>` declaration, whose values already
				// range over `<Join>_potential`.
				if projected.ty().class_type(eq_db).is_some() {
					let enum_ty = self
						.parent
						.substitute_class_with_potential_enum(projected.ty());
					let mut relabeled = Expression::new_unchecked(
						enum_ty,
						(*projected).clone(),
						projected.origin(),
					);
					relabeled
						.annotations_mut()
						.extend(projected.annotations().iter().cloned());
					relabeled
				} else {
					projected
				}
			})
			.collect::<Vec<_>>();

		let params = match &function {
			Callable::Function(f) => Some(self.parent.model[*f].parameters()),
			Callable::Annotation(a) => self.parent.model[*a].parameters.as_ref().map(|v| &v[..]),
			Callable::EnumConstructor(e) => self.parent.model[e.enumeration_id()]
				.definition()
				.unwrap()[e.member_index() as usize]
				.parameters
				.as_ref()
				.map(|v| &v[..]),
			_ => None,
		};

		if let Some(params) = params
			&& params.len() > arguments.len()
		{
			// Need to fill in default and named arguments
			let params = params[arguments.len()..].to_vec();
			let mut named = c
				.named_arguments
				.iter()
				.map(|(name, arg)| {
					(
						self.data[*name].identifier().unwrap(),
						self.collect_expression(*arg),
					)
				})
				.collect::<FxHashMap<_, _>>();

			for param in params {
				let param_name = self.parent.model[param].name().unwrap();
				if let Some(arg) = named.remove(&param_name) {
					arguments.push(arg);
				} else {
					let default = self.parent.param_defaults[&param].clone();
					arguments.push(default);
				}
			}
		}

		// The HIR-resolved function item may no longer match the
		// lowered argument types: varified storage widens par HIR
		// operands to var, and class operands are relabeled to their
		// potential enums. Re-dispatch by name so the call binds the
		// overload for the actual argument types (a `LookupCall`
		// resolves straight back to a `Call`, so a still-matching
		// resolution is unchanged).
		let needs_redispatch = match &function {
			Callable::Function(f) => {
				let params = self.parent.model[*f].parameters();
				params.len() != arguments.len()
					|| !arguments
						.iter()
						.zip(params.iter())
						.all(|(arg, p)| arg.ty().is_subtype_of(db, self.parent.model[*p].ty()))
			}
			_ => false,
		};
		if needs_redispatch {
			let Callable::Function(f) = &function else {
				unreachable!()
			};
			let name = self.parent.model[*f].name();
			alloc_expression(
				LookupCall {
					function: name,
					arguments,
				},
				self,
				origin,
			)
		} else {
			alloc_expression(
				Call {
					function,
					arguments,
				},
				self,
				origin,
			)
		}
	}
}
