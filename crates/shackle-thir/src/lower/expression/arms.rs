//! The three largest arms of `collect_expression_inner`.
//!
//! Calls, identifier references and record accesses each carry enough logic —
//! cross-class identity coercion on `=`/`!=`, class projection of a root, and
//! reconstruction-engine reads of an object's field — to warrant their own file.

use rustc_hash::FxHashMap;
use shackle_hir::{
	class_analysis::class_pattern_for,
	ids::{EntityRef, ExpressionRef, NodeRef, PatternRef},
};
use shackle_ty::{Ty, TyData};

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

	/// Lower an identifier reference, projecting a class-typed root into the
	/// identity universe expected at this position.
	pub(super) fn collect_identifier_expression(
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

	/// Lower a record field access.
	///
	/// Accessing a field of an array of records is rewritten into a
	/// comprehension over the inner value; reads of a class-typed object's
	/// field go through the reconstruction engine.
	pub(super) fn collect_record_access(
		&mut self,
		ra: &shackle_hir::RecordAccess<'db>,
		idx: shackle_hir::ExpressionId<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		let ty = self.types[idx];
		let origin = ExpressionRef::new(db, self.item, idx).into_entity(db);
		let record = self.collect_expression(ra.record);
		if self.types[ra.record].is_array(self.parent.db) {
			// Lift to comprehension
			let record_ty = record.ty().elem_ty(self.parent.db).unwrap();
			let declaration =
				Declaration::new(false, Domain::unbounded(self.parent.db, origin, record_ty));
			let idx = self
				.parent
				.model
				.add_declaration(DeclarationItem::new(declaration, origin));
			let g = Generator::Iterator {
				declarations: vec![idx],
				collection: record,
				where_clause: None,
			};
			alloc_expression(
				ArrayComprehension {
					generators: vec![g],
					template: Box::new(alloc_expression(
						RecordAccess {
							record: Box::new(alloc_expression(idx, self, origin)),
							field: self.data[ra.field].identifier().unwrap(),
						},
						self,
						origin,
					)),
					indices: None,
				},
				self,
				origin,
			)
		} else {
			let field_ident = self.data[ra.field].identifier().unwrap();
			let static_class = record
				.ty()
				.class_type(self.parent.db)
				.or_else(|| self.types[ra.record].class_type(self.parent.db));
			if let Some(class_ref) = static_class {
				let class_pattern = class_pattern_for(self.parent.db, class_ref)
					.expect("class item for class type");
				let class_objects = self.parent.objects.class_map[&class_pattern].class_objects;
				let class_objects_expr = alloc_expression(class_objects, self, origin);
				if record.ty().opt(self.parent.db) == Some(OptType::Opt) {
					// Optional-occurrence receiver: indexing
					// `<C>_objects` by `enum2int(<var opt …>)` would
					// pass an `opt int` into integer array access,
					// which MiniZinc rejects. Project through
					// `deopt(.)` and guard the whole access with
					// `occurs(.)` so an absent receiver yields `<>`.
					let deopt_record = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.deopt.into(),
							arguments: vec![record.clone()],
						},
						self,
						origin,
					);
					let object_index = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.enum2int.into(),
							arguments: vec![deopt_record],
						},
						self,
						origin,
					);
					let object_record =
						self.collect_array_access(class_objects_expr, object_index, origin);
					let field_access = alloc_expression(
						RecordAccess {
							record: Box::new(object_record),
							field: field_ident,
						},
						self,
						origin,
					);
					let occurs = alloc_expression(
						LookupCall {
							function: self.parent.ids.functions.occurs.into(),
							arguments: vec![record],
						},
						self,
						origin,
					);
					let absent = alloc_expression(Absent, self, origin);
					let guarded = IfThenElse {
						branches: vec![Branch::new(occurs, field_access)],
						else_result: Box::new(absent),
					};
					let inferred = alloc_expression(guarded.clone(), self, origin);
					return if inferred.ty() == ty {
						inferred
					} else {
						Expression::new_unchecked(ty, guarded, origin)
					};
				}
				// Reading a single-dimension array-typed attribute
				// through a class-reference receiver. A
				// `<C>_objects[i].arr` access where `i` is a var object
				// identity is a var index into an
				// array-of-records-that-contain-arrays, which MiniZinc
				// rejects ("array access using a variable is not
				// supported for arrays which contain other arrays").
				// The column-projection decomposition of the `'[]'`
				// specialisation avoids this, but only fires on a var
				// THIR index — and a class reference to a par-actual
				// class is relabelled to a PAR potential-enum read, so
				// its index looks par here even though the emitted decl
				// is `var <C>` (var in MiniZinc). Force the index to
				// its var form when the field is a UNIFORM array column
				// and the receiver is NOT provably par, so the
				// decomposition fires for a genuine-var receiver (a
				// `var <C>` reference, or a projected nested identity)
				// while a provably-par receiver (a single-potential
				// root, or a `p in <par set>` generator) keeps the
				// direct access it already lowers correctly. A RAGGED
				// array field (`array [1..l]` with `l` a sibling
				// attribute) is EXCLUDED: its per-object index set
				// makes the single-representative-index-set column
				// projection wrong, and a genuinely-var-receiver ragged
				// read is a type error anyway
				// (`varify_array_class_attribute`), so only
				// effectively-par ragged reads reach here and they
				// lower fine unforced.
				let field_is_array_column = class_objects_expr
					.ty()
					.elem_ty(self.parent.db)
					.and_then(|e| e.record_fields(self.parent.db))
					.map(|fields| {
						fields.iter().any(|(n, fty)| {
							Identifier(*n) == field_ident
								&& matches!(
									fty.lookup(self.parent.db),
									TyData::Array { dim, .. }
										if !dim.is_tuple(self.parent.db)
								)
						})
					})
					.unwrap_or(false);
				let field_is_ragged = field_is_array_column
					&& self
						.parent
						.class_storage_field_decls(class_pattern.item(self.parent.db))
						.into_iter()
						.find(|d| d.ident == field_ident)
						.map(|d| {
							self.parent
								.field_domain_references_attribute(d.owner, d.declared_type)
						})
						.unwrap_or(false);
				// A receiver identifier resolving to a par declaration
				// (a pinned single-potential root, or a par-set
				// generator) is a genuinely-par index — the direct
				// access lowers fine and forcing decomposition would
				// only churn the output.
				let receiver_provably_par = matches!(
					&*record,
					ExpressionData::Identifier(ResolvedIdentifier::Declaration(d))
						if self.parent.model[*d].ty().known_par(self.parent.db)
				);
				let force_column_projection =
					field_is_array_column && !field_is_ragged && !receiver_provably_par;
				let object_index = alloc_expression(
					LookupCall {
						function: self.parent.ids.functions.enum2int.into(),
						arguments: vec![record],
					},
					self,
					origin,
				);
				let object_index = if force_column_projection {
					match object_index.ty().make_var(self.parent.db) {
						Some(var_ty) if var_ty != object_index.ty() => {
							let origin = object_index.origin();
							Expression::new_unchecked(var_ty, (*object_index).clone(), origin)
						}
						_ => object_index,
					}
				} else {
					object_index
				};
				let object_record =
					self.collect_array_access(class_objects_expr, object_index, origin);
				let field_access = RecordAccess {
					record: Box::new(object_record),
					field: field_ident,
				};
				// The projected field may be par where the HIR expected
				// var (a par storage field read through a var context)
				// or var where the HIR kept the attribute par (a
				// varified storage field read through an unvarified
				// context like a class constraint's `this`). Both flow
				// through unchanged: par is a subtype of var, and a
				// par relabel of a genuine var projection would not
				// survive a transform fold. Calls over the value
				// re-dispatch by name.
				alloc_expression(field_access, self, origin)
			} else {
				alloc_expression(
					RecordAccess {
						record: Box::new(self.collect_expression(ra.record)),
						field: field_ident,
					},
					self,
					origin,
				)
			}
		}
	}
}
