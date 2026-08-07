//! Symmetry-breaking defaults for unrealised potential objects.
//!
//! Every potential object that is not realised still occupies a storage slot,
//! so its fields are pinned to a canonical in-domain default — otherwise the
//! solver is free to choose them arbitrarily and the model gains symmetric
//! solutions. Structured fields are pinned leaf-wise; singular `new` fields are
//! channelled to their static child identity instead.

use rustc_hash::FxHashSet;
use shackle_hir::{Item, ids::PatternRef};
use shackle_ty::{Ty, TyData};

use super::{FieldIntroduction, FieldIntroductionKind, PinLeafStep};
use crate::{lower::ItemCollector, *};

impl<'db> ItemCollector<'db> {
	/// Channel a singular `new`/`opt new` storage field to its STATIC per-slot
	/// identity: `forall(p)(<guard> -> <parent>[p].<field> = <Child>_occ_k(p))`.
	///
	/// The var storage field is a *free* `(var) (opt) <Child>_potential`
	/// decision read through from `_storage` — nothing else ties its value to
	/// the slot's own potential identity, so without this pin the field could
	/// point at a sibling slot's identity while the derived actual set claims
	/// the static one (and two parents could alias one child). The guard is
	/// `occurs(<field>)` for opt fields (an absent field pins nothing), the
	/// parent slot's realisation for non-opt fields of var-actual parents
	/// (an unrealised slot's field is symmetry-pinned to `lb`, which need not
	/// be the static identity), and nothing otherwise. Par fields are skipped:
	/// their values are minted statically by the reconstruction engine.
	pub(in crate::lower) fn emit_singular_field_identity_pin(
		&mut self,
		item: Item<'db>,
		intro: &FieldIntroduction<'db>,
		field_ty: Ty<'db>,
		parent_expr: &Expression<'db>,
		parent_index_ty: Ty<'db>,
		child_enum_member: EnumMemberId<'db>,
	) {
		if field_ty.inst(self.db) != Some(VarType::Var) {
			return;
		}
		let FieldIntroductionKind::Singular { opt, .. } = &intro.kind else {
			return;
		};
		let parent_index_set = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![parent_expr.clone()],
			},
		);
		let p_decl = Declaration::new(false, Domain::unbounded(self.db, item, parent_index_ty));
		let p_idx = self
			.model
			.add_declaration(DeclarationItem::new(p_decl, item));
		let p_expr = Expression::new(self.db, &self.model, item, p_idx);
		let parent_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![parent_expr.clone(), p_expr.clone()],
			},
		);
		let field_at_p = Expression::new(
			self.db,
			&self.model,
			item,
			RecordAccess {
				record: Box::new(parent_at_p),
				field: intro.attribute,
			},
		);
		let ordinal = self.contribution_local_ordinal(
			item,
			intro.parent_class,
			intro.parent_contribution_index,
			parent_index_ty,
			p_expr.clone(),
		);
		let identity = Expression::new(
			self.db,
			&self.model,
			item,
			Call {
				function: Callable::EnumConstructor(child_enum_member),
				arguments: vec![ordinal],
			},
		);
		let field_eq_identity = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.eq.into(),
				arguments: vec![field_at_p.clone(), identity],
			},
		);
		let guard = if *opt {
			Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.occurs.into(),
					arguments: vec![field_at_p],
				},
			))
		} else if self
			.object_lowering
			.var_actual_set_classes
			.contains(&intro.parent_class)
		{
			let parent_identity = self.contribution_slot_identity(
				item,
				intro.parent_class,
				intro.parent_contribution_index,
				parent_index_ty,
				p_expr,
			);
			let parent_set_expr = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(self.class_map[&intro.parent_class].class_set),
			);
			Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.in_.into(),
					arguments: vec![parent_identity, parent_set_expr],
				},
			))
		} else {
			None
		};
		let template = match guard {
			Some(guard) => Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.implies.into(),
					arguments: vec![guard, field_eq_identity],
				},
			),
			None => field_eq_identity,
		};
		let compr = Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![p_idx],
					collection: parent_index_set,
					where_clause: None,
				}],
				template,
			),
		);
		let forall = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![compr],
			},
		);
		let _ = self
			.model
			.add_constraint(ConstraintItem::new(Constraint::new(true, forall), item));
	}

	/// Build the symmetry-breaking default expression for a storage-field
	/// access. Returns `None` when the field type has no canonical default
	/// (e.g. functions). Numeric fields route through `mzn_safe_default`
	/// (`lb` of an unbounded var is `-infinity`, and `field = -infinity` is
	/// invalid — the helper picks the first finite bound, falling back to
	/// `0`); bools, enums, `<Class>_potential` refs, and var sets use `lb`
	/// directly (always a valid finite default — `false` / first member /
	/// `{}`); `<>` for any opt field; field-wise recursion for records and
	/// tuples.
	pub(in crate::lower) fn build_field_default_expr(
		&mut self,
		item: Item<'db>,
		field_access_expr: Expression<'db>,
	) -> Option<Expression<'db>> {
		let db = self.db;
		let ty = field_access_expr.ty();
		if ty.opt(db) == Some(OptType::Opt) {
			return Some(Expression::new(self.db, &self.model, item, Absent));
		}
		match ty.lookup(db) {
			TyData::Integer(_, _) | TyData::Float(_, _) => Some(Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.mzn_safe_default.into(),
					arguments: vec![field_access_expr],
				},
			)),
			TyData::Boolean(_, _) | TyData::Enum(_, _, _) | TyData::Set(_, _, _) => {
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.lb.into(),
						arguments: vec![field_access_expr],
					},
				))
			}
			TyData::Record(_, fs) => {
				let fs = fs.clone();
				let mut record_fields: Vec<(Identifier<'db>, Expression<'db>)> =
					Vec::with_capacity(fs.len());
				for (field_id, _) in fs.iter() {
					let field_ident = Identifier(*field_id);
					let inner_access = Expression::new(
						self.db,
						&self.model,
						item,
						RecordAccess {
							record: Box::new(field_access_expr.clone()),
							field: field_ident,
						},
					);
					let inner_default = self.build_field_default_expr(item, inner_access)?;
					record_fields.push((field_ident, inner_default));
				}
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					RecordLiteral(record_fields),
				))
			}
			TyData::Tuple(_, fs) => {
				let len = fs.len();
				let mut tuple_fields: Vec<Expression<'db>> = Vec::with_capacity(len);
				for i in 0..len {
					let inner_access = Expression::new(
						self.db,
						&self.model,
						item,
						TupleAccess {
							tuple: Box::new(field_access_expr.clone()),
							field: IntegerLiteral((i + 1) as i64),
						},
					);
					let inner_default = self.build_field_default_expr(item, inner_access)?;
					tuple_fields.push(inner_default);
				}
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					TupleLiteral(tuple_fields),
				))
			}
			TyData::Array { .. } => {
				// Pin an array field element-wise, re-indexed to the field's
				// own index sets:
				// `f = arrayXd(f, [<default>(f[j]) | j in index_set(f)])`.
				let j_decl =
					Declaration::new(false, Domain::unbounded(self.db, item, Ty::par_int(db)));
				let j_idx = self
					.model
					.add_declaration(DeclarationItem::new(j_decl, item));
				let j_expr = Expression::new(self.db, &self.model, item, j_idx);
				let f_at_j = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![field_access_expr.clone(), j_expr],
					},
				);
				let inner_default = self.build_field_default_expr(item, f_at_j)?;
				let index_set = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.index_set.into(),
						arguments: vec![field_access_expr.clone()],
					},
				);
				let compr = Expression::new(
					self.db,
					&self.model,
					item,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![j_idx],
							collection: index_set,
							where_clause: None,
						}],
						inner_default,
					),
				);
				Some(Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_xd.into(),
						arguments: vec![field_access_expr, compr],
					},
				))
			}
			_ => None,
		}
	}

	/// Emit one symmetry-breaking constraint per defaultable storage field of
	/// `class_pattern`:
	///
	/// ```ignore
	/// constraint forall(x in <C>_potential)
	///                  (x in <C> \/ <C>_objects[x].<f> = <default>(x));
	/// ```
	///
	/// This is the stdlib-thinner rewrite of
	/// `forall(x in <C>_potential diff <C>)(...)`: keeping the iterator par
	/// avoids dispatching a var-set generator and instead half-reifies the
	/// membership through a disjunction. Skips fields without a canonical
	/// default (see `build_field_default_expr`).
	///
	/// We don't short-circuit on potential cardinality at THIR time because
	/// the cardinality of e.g. `A_occ_0(1..n)` is a runtime parameter — even
	/// when there is only one constructor. Instead we skip whenever the
	/// class actual set is statically pinned: par-typed AND defined. That
	/// covers singular `var new C` and `var opt new C` (definition is the
	/// full potential constructor) and singular field-only chains that fell
	/// back to defining `<C>` as the full enum — in both shapes no potential
	/// can ever be unused so the recipe is vacuous. The `array_union(...)`
	/// definition path widens `<C>` to var and stays in the emit set.
	pub(in crate::lower) fn emit_unused_potential_default_constraints(
		&mut self,
		class_pattern: PatternRef<'db>,
	) {
		let Some(class_info) = self.class_map.get(&class_pattern).copied() else {
			return;
		};
		let class_enum = class_info.class_enum;
		let class_set = class_info.class_set;
		let class_objects = class_info.class_objects;

		let class_set_decl = &self.model[class_set];
		let class_set_is_var = class_set_decl.ty().inst(self.db) == Some(VarType::Var);
		let class_set_defined = class_set_decl.definition().is_some();
		if !class_set_is_var && class_set_defined {
			return;
		}

		// Skip if the stdlib isn't loaded: the recipe needs `lb`,
		// `enum2int`, `forall`, etc. The cases that survive the gate
		// above without stdlib (e.g. `array of var new`) realise every
		// potential by construction, so the constraint is vacuous and
		// skipping is safe.
		let par_int_ty = Ty::par_int(self.db);
		let par_enum_ty = Ty::par_enum(self.db, self.model[class_enum].enum_type());
		if self
			.model
			.lookup_function(self.db, self.ids.functions.lb.into(), &[par_enum_ty])
			.is_err() || self
			.model
			.lookup_function(self.db, self.ids.functions.enum2int.into(), &[par_enum_ty])
			.is_err() || self
			.model
			.lookup_function(
				self.db,
				self.ids.functions.forall.into(),
				&[Ty::array(self.db, par_int_ty, Ty::par_bool(self.db)).unwrap()],
			)
			.is_err()
		{
			return;
		}

		let item = class_pattern.item(self.db);
		let fields = self.class_storage_fields_for_domain(class_pattern);

		// Skip the pin for defined fields (computed attributes and
		// domain-dependent fields) when every contribution to this class leaves
		// them functionally determined (alias-defined, or read through from a
		// determined contribution). A determined field's unrealised-slot value
		// is a function of its pinned free siblings, so skipping loses no
		// symmetry breaking — while pinning it to `mzn_safe_default` (its own
		// flatten-time `lb`) is inconsistent whenever the defining RHS evaluated
		// at the siblings' pinned defaults differs from that `lb` (any
		// non-monotone RHS), which forces unrealised potentials into the class
		// set and silently removes solutions. Where some contribution still
		// fresh-mints a defined field, the pin stays load-bearing and is kept.
		let skip_defined_fields = self
			.class_contributions_all_determined
			.get(&class_pattern)
			.copied()
			.unwrap_or(false);
		let defined_fields: FxHashSet<Identifier<'db>> = if skip_defined_fields {
			self.class_storage_field_decls(class_pattern.item(self.db))
				.into_iter()
				.filter(|d| {
					d.definition.is_some()
						|| self.field_domain_references_attribute(d.owner, d.declared_type)
				})
				.map(|d| d.ident)
				.collect()
		} else {
			Default::default()
		};

		let enum_ref = self.model[class_enum].enum_type();
		let x_ty = Ty::par_enum(self.db, enum_ref);
		// `<C>_objects` may be indexed by `int` (top-level storage) or by
		// `<C>_potential` (field-only chains whose contribution decl was
		// declared with the enum-typed dimension). Read the array's
		// dimension type and only emit `enum2int(x)` when we need a par
		// int index — passing enum2int into an enum-typed dimension would
		// itself fail to dispatch.
		let class_objects_index_is_int = match self.model[class_objects].ty().lookup(self.db) {
			TyData::Array { dim, .. } => *dim != x_ty,
			_ => return,
		};

		for (field_ident, field_ty) in fields {
			if defined_fields.contains(&field_ident) {
				continue;
			}
			// Structured (tuple/record) fields are pinned LEAF-WISE — one
			// forall per scalar leaf, `f.1 = <default>(f.1)` — instead of one
			// whole-value equality `f = (<default>, ...)`. The whole-value
			// form cannot be evaluated by the target MiniZinc when the field
			// reads a var-tuple-containing record through the generic `'[]'`
			// helper inside the reified disjunction (an upstream limitation);
			// component pins evaluate fine and pin exactly the same values.
			// An opt structured field stays a single leaf (`f = <>`).
			for leaf_path in Self::pin_leaf_paths(self.db, field_ty) {
				let mut x_decl = Declaration::new(false, Domain::unbounded(self.db, item, x_ty));
				x_decl.set_name(Identifier::new(self.db, "x"));
				let x_decl_idx = self
					.model
					.add_declaration(DeclarationItem::new(x_decl, item));
				let x_expr = Expression::new(self.db, &self.model, item, x_decl_idx);

				let class_objects_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_objects),
				);
				let x_index = if class_objects_index_is_int {
					Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.enum2int.into(),
							arguments: vec![x_expr.clone()],
						},
					)
				} else {
					x_expr.clone()
				};
				let object_at_x = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.array_access.into(),
						arguments: vec![class_objects_expr, x_index],
					},
				);
				let field_at_x = Expression::new(
					self.db,
					&self.model,
					item,
					RecordAccess {
						record: Box::new(object_at_x),
						field: field_ident,
					},
				);
				let mut leaf_at_x = field_at_x;
				for step in leaf_path.iter() {
					leaf_at_x = match step {
						PinLeafStep::Tuple(i) => Expression::new(
							self.db,
							&self.model,
							item,
							TupleAccess {
								tuple: Box::new(leaf_at_x),
								field: IntegerLiteral(*i),
							},
						),
						PinLeafStep::Record(ident) => Expression::new(
							self.db,
							&self.model,
							item,
							RecordAccess {
								record: Box::new(leaf_at_x),
								field: *ident,
							},
						),
					};
				}
				let Some(default_expr) = self.build_field_default_expr(item, leaf_at_x.clone())
				else {
					continue;
				};
				let eq_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.eq.into(),
						arguments: vec![leaf_at_x, default_expr],
					},
				);

				let class_set_expr = Expression::new(
					self.db,
					&self.model,
					item,
					ResolvedIdentifier::Declaration(class_set),
				);
				let in_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.in_.into(),
						arguments: vec![x_expr, class_set_expr],
					},
				);

				let disj_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.or.into(),
						arguments: vec![in_call, eq_call],
					},
				);

				let enum_set_expr = Expression::new(self.db, &self.model, item, class_enum);
				let compr = Expression::new(
					self.db,
					&self.model,
					item,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![x_decl_idx],
							collection: enum_set_expr,
							where_clause: None,
						}],
						disj_call,
					),
				);
				let forall_call = Expression::new(
					self.db,
					&self.model,
					item,
					LookupCall {
						function: self.ids.functions.forall.into(),
						arguments: vec![compr],
					},
				);

				let _ = self.model.add_constraint(ConstraintItem::new(
					Constraint::new(true, forall_call),
					item,
				));
			}
		}
	}

	/// The scalar-leaf access paths of a storage field type for the
	/// unused-potential pins: a non-structured (or opt) type is a single
	/// leaf at the empty path; tuple/record types expand field-wise. See
	/// the leaf-wise pin note in
	/// `emit_unused_potential_default_constraints`.
	pub(in crate::lower) fn pin_leaf_paths(
		db: &'db dyn Db,
		ty: Ty<'db>,
	) -> Vec<Vec<PinLeafStep<'db>>> {
		let mut out = Vec::new();
		let mut todo: Vec<(Vec<PinLeafStep<'db>>, Ty<'db>)> = vec![(Vec::new(), ty)];
		while let Some((path, t)) = todo.pop() {
			if t.opt(db) == Some(OptType::Opt) {
				out.push(path);
				continue;
			}
			match t.lookup(db) {
				TyData::Tuple(_, fs) => {
					for (i, f) in fs.iter().enumerate() {
						let mut p = path.clone();
						p.push(PinLeafStep::Tuple((i + 1) as i64));
						todo.push((p, *f));
					}
				}
				TyData::Record(_, fs) => {
					for (ident, f) in fs.iter() {
						let mut p = path.clone();
						p.push(PinLeafStep::Record(Identifier(*ident)));
						todo.push((p, *f));
					}
				}
				_ => out.push(path),
			}
		}
		out
	}
}
