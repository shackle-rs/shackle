//! Lowering of array access.
//!
//! Has special handling for record access of class types.

use shackle_hir::{class_analysis::class_pattern_for, ids::ExpressionRef};
use shackle_ty::TyData;

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
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
						self.introduce_array_access(class_objects_expr, object_index, origin);
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
					if let Some(fixed) = self.fix_for_output(&inferred, ty, origin) {
						return fixed;
					}
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
					self.introduce_array_access(class_objects_expr, object_index, origin);
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
				//
				// The exception is an output context, where the typer
				// par-ified the projection because it reads a solved
				// value — there the var storage read has to be fixed so
				// the two agree, and so the generators beneath it stay
				// par (see `collect_record_access` in the HIR typer).
				let projected = alloc_expression(field_access, self, origin);
				self.fix_for_output(&projected, ty, origin)
					.unwrap_or(projected)
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
