//! Storage layout for class objects.
//!
//! Each class stores its objects as an array of records. This module decides
//! that record's shape — which fields it carries, and each field's domain,
//! substituting `Class<X>` with the child's `<X>_potential` identity — and
//! lowers `opt new` inputs into the representation storage expects.

use rustc_hash::{FxHashMap, FxHashSet};
use shackle_hir::{
	ClassMember, Item, PatternTy, class_analysis::class_pattern_for, ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use crate::{
	lower::{ItemCollector, expression::ExpressionCollector},
	source::Origin,
	*,
};

impl<'db> ItemCollector<'db> {
	/// Replace any `Class<X>` element inside `ty` with `<X>_potential` (the
	/// par enum representing that class's potential identity universe).
	/// Used when building storage records so that `var set of new B: bs`
	/// lowers to `var set of B_potential` rather than `var set of B` — the
	/// latter would create a circular type definition once `B` is itself
	/// derived as `array_union(...)` over the very `bs` field.
	pub(in crate::lower) fn substitute_class_with_potential_enum(&self, ty: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Class(inst, opt, class_ref) => {
				let Some(class_pattern) = class_pattern_for(db, *class_ref) else {
					return ty;
				};
				let Some(class_map_info) = self.objects.class_map.get(&class_pattern) else {
					// Class hasn't been registered yet. This only happens while
					// predeclaring a class that participates in a reference
					// cycle (`Seat` ↔ `Handrail`): items are predeclared in
					// topological order, but a cycle has no valid order, so
					// whichever class comes first sees the other unregistered.
					// The unsubstituted `Class<X>` must not survive into item
					// collection — expression lowering freezes decl types at
					// build time and a class-typed storage field defeats the
					// potential-enum arm of `lowered_ty_matches` — so
					// `repair_predeclared_class_objects_domains` rebuilds every
					// `<C>_objects` domain once all classes are registered.
					return ty;
				};
				let enum_ref = self.model[class_map_info.class_enum].enum_type();
				// Preserve the source `inst`: `var Class<B>` (field reached
				// through a `var new` path) must lower to `var B_potential`,
				// not `par B_potential`. Without this,
				// `class_storage_fields_for_domain` returns par-typed fields
				// for a var-reached class's storage record, the per-class
				// `*_objects` decl is initially declared par-typed at
				// `predeclare_class` time, constraint-emission captures that
				// par type in let-decomposition decls, and the later
				// `finish()` update of the decl to var-typed leaves the
				// captured decls stale (par-vs-var mismatch in lowered MZN).
				let par_enum = Ty::par_enum(db, enum_ref);
				par_enum
					.with_inst(db, *inst)
					.unwrap_or(par_enum)
					.with_opt(db, *opt)
			}
			TyData::Set(inst, opt, element) => {
				let new_element = self.substitute_class_with_potential_enum(*element);
				let par_set = Ty::par_set(db, new_element).unwrap_or(*element);
				par_set
					.with_inst(db, *inst)
					.unwrap_or(par_set)
					.with_opt(db, *opt)
			}
			TyData::Array { opt, dim, element } => {
				let new_element = self.substitute_class_with_potential_enum(*element);
				Ty::array(db, *dim, new_element)
					.unwrap_or(ty)
					.with_opt(db, *opt)
			}
			TyData::Tuple(opt, fields) => {
				let new_fields = fields
					.iter()
					.map(|f| self.substitute_class_with_potential_enum(*f))
					.collect::<Vec<_>>();
				Ty::tuple(db, new_fields).with_opt(db, *opt)
			}
			TyData::Record(opt, fields) => {
				let new_fields = fields
					.iter()
					.map(|(name, f)| (*name, self.substitute_class_with_potential_enum(*f)))
					.collect::<Vec<_>>();
				Ty::record(db, new_fields).with_opt(db, *opt)
			}
			_ => ty,
		}
	}

	/// Domain-level counterpart of `substitute_class_with_potential_enum`:
	/// build a `Domain` for `ty` in which every `Class<X>` element becomes a
	/// `Bounded` domain referencing the `<X>_potential` enum item. Enum-typed
	/// domains are expected to carry their universe as a `Bounded` expression
	/// throughout shackle — type erasure replaces enum types with plain `int`
	/// without materialising any bound, so an `Unbounded` potential-enum
	/// position would lose its finiteness there (surfacing as e.g. an
	/// unbounded `var set of int` storage slot, which MiniZinc rejects).
	///
	/// Classes not yet registered (reference cycles during predeclare) fall
	/// back to the unbounded class-typed form, exactly like the type
	/// substitution — `repair_predeclared_class_objects_domains` rebuilds
	/// those domains through this same builder once registration completes.
	pub(in crate::lower) fn substitute_class_with_potential_enum_domain(
		&self,
		ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		match ty.lookup(db) {
			TyData::Class(inst, opt, class_ref) => {
				let Some(class_pattern) = class_pattern_for(db, *class_ref) else {
					return Domain::unbounded(db, origin, ty);
				};
				let Some(class_map_info) = self.objects.class_map.get(&class_pattern) else {
					return Domain::unbounded(db, origin, ty);
				};
				let potential = Expression::new(db, &self.model, origin, class_map_info.class_enum);
				Domain::bounded(db, origin, *inst, *opt, potential)
			}
			TyData::Enum(inst, opt, enum_ref) => {
				// A `<X>_potential` enum the caller substituted in already
				// (storage element types are often built from pre-substituted
				// record types): recover its enum item to serve as the bound.
				// Non-potential enums are left alone — their declarations carry
				// `Bounded` domains from HIR lowering.
				let Some(class_enum) = self
					.objects
					.class_map
					.values()
					.map(|info| info.class_enum)
					.find(|e| self.model[*e].enum_type() == *enum_ref)
				else {
					return Domain::unbounded(db, origin, ty);
				};
				let potential = Expression::new(db, &self.model, origin, class_enum);
				Domain::bounded(db, origin, *inst, *opt, potential)
			}
			TyData::Set(inst, opt, element) => {
				let element = self.substitute_class_with_potential_enum_domain(*element, origin);
				Domain::set(db, origin, *inst, *opt, element)
			}
			TyData::Array { opt, dim, element } => {
				let element = self.substitute_class_with_potential_enum_domain(*element, origin);
				Domain::array(
					db,
					origin,
					*opt,
					Domain::unbounded(db, origin, *dim),
					element,
				)
			}
			TyData::Tuple(opt, fields) => Domain::tuple(
				db,
				origin,
				*opt,
				fields
					.iter()
					.map(|f| self.substitute_class_with_potential_enum_domain(*f, origin))
					.collect::<Vec<_>>(),
			),
			TyData::Record(opt, fields) => Domain::record(
				db,
				origin,
				*opt,
				fields
					.iter()
					.map(|(name, f)| {
						(
							Identifier(*name),
							self.substitute_class_with_potential_enum_domain(*f, origin),
						)
					})
					.collect::<Vec<_>>(),
			),
			_ => Domain::unbounded(db, origin, ty),
		}
	}

	pub(in crate::lower) fn class_storage_fields(
		&self,
		class_pattern: PatternRef<'db>,
	) -> Vec<(Identifier<'db>, Ty<'db>)> {
		let types = class_pattern.item(self.db).types(self.db);
		match &types[class_pattern.pattern(self.db)] {
			PatternTy::ClassDecl {
				storage_record_ty,
				var_storage_record_ty,
				..
			} => {
				// For classes reached via any `var new` introduction path,
				// use the varified storage record so `*_objects` arrays are
				// declared with var-typed fields from the start. Otherwise
				// use the par-typed record from the class declaration.
				let record_ty = if self
					.objects
					.plan
					.var_reached_classes
					.contains(&class_pattern)
				{
					*var_storage_record_ty
				} else {
					*storage_record_ty
				};
				record_ty
					.record_fields(self.db)
					.unwrap_or_default()
					.into_iter()
					.map(|(field, ty)| (Identifier(field), ty))
					.collect()
			}
			_ => unreachable!(),
		}
	}

	/// Same as `class_storage_fields` but with each `Class<X>` element
	/// substituted with `<X>_potential` (par enum). Use at the points that
	/// emit the storage record domain — both `*_storage` declarations and
	/// per-class `*_objects` arrays — so the lowered model references the
	/// child class's potential universe rather than the child class's
	/// actual-set (which would create a circular type definition since
	/// the actual-set is derived from these very fields).
	pub(in crate::lower) fn class_storage_fields_for_domain(
		&self,
		class_pattern: PatternRef<'db>,
	) -> Vec<(Identifier<'db>, Ty<'db>)> {
		self.class_storage_fields(class_pattern)
			.into_iter()
			.map(|(field, ty)| (field, self.substitute_class_with_potential_enum(ty)))
			.collect()
	}

	// Recursively gather `(class_item, declared_type, original_ty)` per field
	// identifier for every storage field of `class_pattern`, walking
	// superclass items first so that inherited fields appear before own
	// fields. Used to build per-field domains for the `_storage` record that
	// preserve declared bounds (`var 5..10: i`, `var set of 0..3: s`, etc.).
	pub(in crate::lower) fn collect_class_field_descriptors(
		&self,
		class_pattern: PatternRef<'db>,
		descriptors: &mut FxHashMap<
			Identifier<'db>,
			(Item<'db>, shackle_hir::TypeId<'db>, Ty<'db>),
		>,
	) {
		let class_item = class_pattern.item(self.db);
		let Item::Class(class_item_ref) = class_item else {
			return;
		};
		let class = class_item_ref.class(self.db);
		let class_types = class_item.types(self.db);
		if let Some(base) = class.extends.and_then(|b| class_types.name_resolution(b)) {
			self.collect_class_field_descriptors(base, descriptors);
		}
		for field_item in class.items.iter() {
			let ClassMember::Declaration(d) = field_item else {
				continue;
			};
			for pattern in shackle_hir::Pattern::identifiers(d.pattern, class.data()) {
				let pat_ref = PatternRef::new(self.db, class_item, pattern);
				let Some(ident) = pat_ref.identifier(self.db) else {
					continue;
				};
				let original_ty = match &class_types[pattern] {
					PatternTy::Variable(ty) => *ty,
					_ => continue,
				};
				let _ = descriptors.insert(ident, (class_item, d.declared_type, original_ty));
			}
		}
	}

	// Wrap `build_class_storage_record_domain` for the `array [...] of
	// record(...)` shape that every `*_objects` / `*_storage` declaration
	// uses. The index domain is intentionally left unbounded — callers
	// that need a tighter (par-enum or singleton ordinal) index domain
	// should still call `build_class_storage_record_domain` directly and
	// assemble the `Domain::array` themselves.
	pub(in crate::lower) fn build_class_storage_array_domain(
		&mut self,
		class_pattern: PatternRef<'db>,
		array_ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		let (Some(elem_ty), Some(dim_ty)) = (array_ty.elem_ty(db), array_ty.dim_ty(db)) else {
			return Domain::unbounded(db, origin, array_ty);
		};
		let opt = array_ty.opt(db).unwrap_or(OptType::NonOpt);
		let elem_dom = self.build_class_storage_record_domain(class_pattern, elem_ty, origin);
		let dim_dom = Domain::unbounded(db, origin, dim_ty);
		Domain::array(db, origin, opt, dim_dom, elem_dom)
	}

	// Build a record `Domain` for the `_storage` element with declared bounds
	// preserved per field. Scalar fields like `var 5..10: i` come back as
	// `Domain::bounded(5..10)` instead of `Domain::unbounded`, which keeps the
	// downstream FlatZinc within solver-supported domain widths (gecode
	// otherwise errors during SOLVING on unbounded `var int` / `var set of int`
	// introduced by the per-field symmetry-breaking `= lb(...)` constraints).
	// Class-typed fields keep their unbounded form — bounds for those live in
	// the substituted potential-enum element type.
	pub(in crate::lower) fn build_class_storage_record_domain(
		&mut self,
		class_pattern: PatternRef<'db>,
		storage_elem_ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Domain<'db> {
		let origin = origin.into();
		let db = self.db;
		let Some(record_fields) = storage_elem_ty.record_fields(db) else {
			return Domain::unbounded(db, origin, storage_elem_ty);
		};
		let opt = storage_elem_ty.opt(db).unwrap_or(OptType::NonOpt);
		let mut descriptors: FxHashMap<
			Identifier<'db>,
			(Item<'db>, shackle_hir::TypeId<'db>, Ty<'db>),
		> = FxHashMap::default();
		self.collect_class_field_descriptors(class_pattern, &mut descriptors);
		let mut field_domains: Vec<(Identifier<'db>, Domain<'db>)> =
			Vec::with_capacity(record_fields.len());
		let storage_field_decls = self.class_storage_field_decls(class_pattern.item(db));
		for (raw_ident, field_ty) in record_fields {
			let ident = Identifier(raw_ident);
			// A relocated defined field's declared domain must not bind in
			// the shared element record — it is re-imposed as a realised-set
			// class invariant instead. The per-field decision is the same at
			// every site (owner-keyed predicate), so the `_storage`,
			// predeclare, contribution, and `finish()` domains all relax
			// together.
			let relocated = storage_field_decls
				.iter()
				.find(|d| d.ident == ident)
				.map(|d| self.field_relocates_declared_domain(d))
				.unwrap_or(false);
			// Set-cardinality relocation: the card bound must not bind on
			// unrealised slots, whose canonical pin/witness value is `{}`. It
			// is re-imposed on realised objects by
			// `emit_nested_set_cardinality_class_invariants`.
			let card_relocated = storage_field_decls
				.iter()
				.find(|d| d.ident == ident)
				.map(|d| self.field_relocates_set_card(d))
				.unwrap_or(false);
			let domain = match descriptors.get(&ident).copied() {
				Some(_) if relocated => {
					self.substitute_class_with_potential_enum_domain(field_ty, origin)
				}
				Some((class_item, declared_type, _original_ty)) if card_relocated => {
					self.card_stripped_set_field_domain(class_item, declared_type, field_ty, origin)
				}
				Some((class_item, declared_type, original_ty)) => {
					let substituted = self.substitute_class_with_potential_enum(original_ty);
					if substituted != original_ty {
						self.class_storage_field_domain(class_item, declared_type, field_ty, origin)
					} else if self.field_domain_references_attribute(class_item, declared_type) {
						// A scalar field whose declared domain references another
						// attribute (`var 1..z: s`, with `z` computed) can't carry
						// that per-object bound in the shared record element type —
						// it would need `z` in scope, which isn't meaningful for a
						// record type. Leave the declared bound off; the tight
						// per-object bound is enforced in the reconstruction
						// comprehension's fresh `let {var 1..z: ..} in ..` decl.
						self.substitute_class_with_potential_enum_domain(field_ty, origin)
					} else {
						let Item::Class(class_item_ref) = class_item else {
							unreachable!()
						};
						let class = class_item_ref.class(db);
						let class_types = class_item.types(db);
						let mut inner =
							ExpressionCollector::new(self, class.data(), class_item, &class_types);
						inner.collect_domain(declared_type, field_ty, false)
					}
				}
				None => self.substitute_class_with_potential_enum_domain(field_ty, origin),
			};
			field_domains.push((ident, domain));
		}
		Domain::record(db, origin, opt, field_domains)
	}

	/// Whether a field's declared type carries a `Bounded` domain expression —
	/// directly (`var 1..z: s`) or nested inside array dimensions/elements or
	/// set elements (`array [1..l] of int: xs`, the par-ragged shape) — that
	/// references another class attribute. Such a per-object bound cannot
	/// live in the shared storage record element type (the sibling isn't in
	/// scope there): the element type goes unbounded and the value's own
	/// index sets / the reconstruction comprehension carry the per-object
	/// shape instead. Var-reached classes never get here for these shapes —
	/// validation rejects attribute-referencing domains on them outright.
	pub(in crate::lower) fn field_domain_references_attribute(
		&self,
		class_item: Item<'db>,
		declared_type: shackle_hir::TypeId<'db>,
	) -> bool {
		let db = self.db;
		let field_patterns: Vec<PatternRef<'db>> = self
			.class_storage_field_decls(class_item)
			.into_iter()
			.map(|f| f.pattern)
			.collect();
		let class_types = class_item.types(db);
		let Item::Class(class_item_ref) = class_item else {
			return false;
		};
		let class_data = class_item_ref.class(db).data();
		for type_node in shackle_hir::Type::walk(declared_type, class_data) {
			let shackle_hir::Type::Bounded { domain, .. } = &class_data[type_node] else {
				continue;
			};
			for sub in shackle_hir::Expression::walk(*domain, class_data) {
				if matches!(&class_data[sub], shackle_hir::Expression::Identifier(_))
					&& let Some(res) = class_types.name_resolution(sub)
					&& field_patterns.contains(&res)
				{
					return true;
				}
			}
		}
		false
	}

	/// Does this object input record type contain an optional-child slot
	/// (`opt record`, produced by an `opt new C` attribute) at any depth?
	/// MiniZinc forbids `opt record` declarations and values, so such a slot
	/// must be lowered to a non-opt 0-or-1-length list before it can appear in
	/// a `_inputs` array. Used to gate the (snapshot-affecting) normalisation
	/// so classes with no optional child are left byte-identical.
	pub(in crate::lower) fn input_ty_needs_opt_new_lowering(&self, ty: Ty<'db>) -> bool {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Record(opt, fields) => {
				*opt == OptType::Opt
					|| fields
						.iter()
						.any(|(_, f)| self.input_ty_needs_opt_new_lowering(*f))
			}
			TyData::Array { element, .. } => self.input_ty_needs_opt_new_lowering(*element),
			_ => false,
		}
	}

	/// Lower an object input record TYPE by replacing each optional-child
	/// slot (`opt record`, from `opt new C`) with a non-opt 0-or-1-length list
	/// (`array [int] of record`). Recurses through records / lists so nested
	/// optional children are normalised too. The 0/1-length list feeds the
	/// same flattened-collection machinery a `set(0..1) of new C` field uses;
	/// see `lower_opt_new_input_value` for the matching value transform.
	pub(in crate::lower) fn lower_opt_new_input_ty(&self, ty: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		match ty.lookup(db) {
			TyData::Record(opt, fields) => {
				let lowered = Ty::record(
					db,
					fields
						.iter()
						.map(|(name, f)| (*name, self.lower_opt_new_input_ty(*f)))
						.collect::<Vec<_>>(),
				);
				if *opt == OptType::Opt {
					Ty::array(db, Ty::par_int(db), lowered).expect("opt-new child list type")
				} else {
					lowered
				}
			}
			TyData::Array { opt, dim, element } => {
				Ty::array(db, *dim, self.lower_opt_new_input_ty(*element))
					.map(|t| t.with_opt(db, *opt))
					.unwrap_or(ty)
			}
			_ => ty,
		}
	}

	/// Lower an object input record VALUE to match `lower_opt_new_input_ty`.
	/// A present optional child `(f: (…))` becomes `(f: [(…)])`; an absent one
	/// `(f: <>)` becomes `(f: [])` (an empty list typed as the child input
	/// record list). Recurses through record / list literals. Because MiniZinc
	/// forbids `opt record` values, only STATICALLY present/absent literal
	/// inputs are handled here; a non-literal optional input is left unchanged
	/// (rejected earlier by validation).
	pub(in crate::lower) fn lower_opt_new_input_value(
		&mut self,
		item: Item<'db>,
		value: Expression<'db>,
		orig_ty: Ty<'db>,
	) -> Expression<'db> {
		let db = self.db;
		match orig_ty.lookup(db) {
			TyData::Record(OptType::Opt, fields) => {
				let inner_ty = Ty::record(db, fields.to_vec());
				match &*value {
					ExpressionData::Absent => {
						let list_ty =
							Ty::array(db, Ty::par_int(db), self.lower_opt_new_input_ty(inner_ty))
								.expect("opt-new child list type");
						Expression::new_unchecked(list_ty, ArrayLiteral(vec![]), value.origin())
					}
					_ => {
						let normalized = self.lower_opt_new_input_value(item, value, inner_ty);
						Expression::new(self.db, &self.model, item, ArrayLiteral(vec![normalized]))
					}
				}
			}
			TyData::Record(_, fields) => {
				if !self.input_ty_needs_opt_new_lowering(orig_ty) {
					return value;
				}
				match &*value {
					ExpressionData::RecordLiteral(rl) => {
						let fields = fields.clone();
						let new_fields = rl
							.iter()
							.map(|(ident, fv)| {
								let fty = fields
									.iter()
									.find(|(n, _)| Identifier(*n) == *ident)
									.map(|(_, t)| *t)
									.expect("input record field type");
								(
									*ident,
									self.lower_opt_new_input_value(item, fv.clone(), fty),
								)
							})
							.collect::<Vec<_>>();
						Expression::new(self.db, &self.model, item, RecordLiteral(new_fields))
					}
					_ => value,
				}
			}
			TyData::Array { element, .. } => {
				if !self.input_ty_needs_opt_new_lowering(orig_ty) {
					return value;
				}
				let element = *element;
				match &*value {
					ExpressionData::ArrayLiteral(al) => {
						let new_elems = al
							.iter()
							.map(|e| self.lower_opt_new_input_value(item, e.clone(), element))
							.collect::<Vec<_>>();
						Expression::new(self.db, &self.model, item, ArrayLiteral(new_elems))
					}
					_ => value,
				}
			}
			_ => value,
		}
	}

	/// Lower a top-level collection input VALUE (the RHS of a
	/// `set of new C: cs = [..]` / `array [d] of new C: pool = [..]` root) whose
	/// member records carry optional children. The collection is an array *of*
	/// input records, so the element records must be lowered one level down:
	/// `element_record_ty` is the per-member input record type (`opt new`
	/// children still `opt record`), which we wrap in an array type so
	/// `lower_opt_new_input_value`'s array arm maps the transform over each
	/// member. Passing the element record type directly (as if the value were a
	/// single record) would fall through unchanged — the value is an array, not
	/// a record. Non-literal collections are left unchanged (rejected earlier).
	pub(in crate::lower) fn lower_opt_new_input_collection_value(
		&mut self,
		item: Item<'db>,
		value: Expression<'db>,
		element_record_ty: Ty<'db>,
	) -> Expression<'db> {
		let db = self.db;
		let collection_ty = Ty::array(db, Ty::par_int(db), element_record_ty)
			.expect("opt-new input collection type");
		self.lower_opt_new_input_value(item, value, collection_ty)
	}

	/// Restrict a class's full storage record type to the fields that are
	/// *free decisions* in the `_storage` array. Two kinds of field are dropped
	/// because they are instead *defined* per object in the reconstruction
	/// comprehension (as generator aliases), not stored freely:
	///
	/// - **Computed attributes** (`d.definition.is_some()`, e.g.
	///   `array[int] of var 1..x: y = f(x)`). A free `_storage` decision for
	///   such a field would be an invalid MiniZinc declaration anyway — you
	///   cannot allocate an unknown-length `array[int] of var int` or a free
	///   unbounded `var set of int`; only the alias form (`y = f(x)`) is legal.
	/// - **Domain-dependent fields** (`field_domain_references_attribute`, e.g.
	///   `var 1..z: s` with `z` a sibling). Their per-object bound can't live in
	///   the shared record element type; they are minted per object in the
	///   comprehension via `let { var 1..z: .. } in ..`.
	///
	/// Class-typed fields are *kept* — under a var-new root every attribute is a
	/// free decision and class-typed fields carry their `<X>_potential` identity
	/// in storage (bounded by the per-parent slice constraint).
	pub(in crate::lower) fn free_storage_record_ty(
		&self,
		class_pattern: PatternRef<'db>,
		storage_record_ty: Ty<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let Some(record_fields) = storage_record_ty.record_fields(db) else {
			return storage_record_ty;
		};
		let excluded: FxHashSet<Identifier<'db>> = self
			.class_storage_field_decls(class_pattern.item(db))
			.into_iter()
			.filter(|d| {
				d.definition.is_some()
					|| self.field_domain_references_attribute(d.owner, d.declared_type)
			})
			.map(|d| d.ident)
			.collect();
		if excluded.is_empty() {
			return storage_record_ty;
		}
		let opt = storage_record_ty.opt(db).unwrap_or(OptType::NonOpt);
		Ty::record(
			db,
			record_fields
				.into_iter()
				.filter(|(name, _)| !excluded.contains(&Identifier(*name)))
				.collect::<Vec<_>>(),
		)
		.with_opt(db, opt)
	}

	// Domain for a class-typed storage field (the `substituted != original`
	// case). The base type is the `Class<X>`→`<X>_potential`-substituted,
	// otherwise-unbounded form. If the field is a *reference* set field
	// `set(<card>) of <Class>` (no `new`), carry its cardinality bound into the
	// type natively as `var set(<card>) of <X>_potential` instead of dropping
	// it — the native `set(c) of S` syntax desugars to a `card(...) in <card>`
	// constraint in the target MiniZinc. Fresh `set of new` collections keep
	// the unbounded form and instead get their bound from the nested
	// cardinality invariant / slice loop. Only a constant bound belongs in the
	// shared element type; a per-object bound would still need the `forall`.
	pub(in crate::lower) fn class_storage_field_domain(
		&mut self,
		class_item: Item<'db>,
		declared_type: shackle_hir::TypeId<'db>,
		field_ty: Ty<'db>,
		origin: Origin<'db>,
	) -> Domain<'db> {
		let db = self.db;
		let subst = self.substitute_class_with_potential_enum(field_ty);
		let Item::Class(class_item_ref) = class_item else {
			return self.substitute_class_with_potential_enum_domain(field_ty, origin);
		};
		let class_data = class_item_ref.class(db).data();
		// A single-dimension array-of-class-reference field
		// (`array [1..2] of B`). The declared per-object dimensions live in the
		// AST declared type — the resolved `Ty` erases `1..2` to `int` — so a
		// bare `Domain::unbounded(subst)` gives the free `_storage` decision an
		// `array [int] of var <B>_potential` element with an unknown index set,
		// which MiniZinc cannot allocate. Preserve the declared dimensions and
		// give each slot the substituted `<B>_potential` identity domain, exactly
		// a scalar class-reference field's storage (`var <B>_potential`) per
		// array position. Only single-dim, scalar-class-element arrays are
		// supported here; other array-of-class shapes stay rejected by
		// object validation.
		if let shackle_hir::Type::Array { dimensions, .. } = &class_data[declared_type] {
			let dimensions = *dimensions;
			if let (Some(dim_ty), Some(elem_ty)) = (field_ty.dim_ty(db), field_ty.elem_ty(db)) {
				let elem_subst = self.substitute_class_with_potential_enum(elem_ty);
				if !dim_ty.is_tuple(db) && matches!(elem_subst.lookup(db), TyData::Enum(_, _, _)) {
					let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
					let class_types = class_item.types(db);
					let dim_dom = {
						let mut inner = ExpressionCollector::new(
							self,
							class_item_ref.class(db).data(),
							class_item,
							&class_types,
						);
						inner.collect_element_domain(dimensions, dim_ty, false)
					};
					let elem_dom =
						self.substitute_class_with_potential_enum_domain(elem_ty, origin);
					return Domain::array(db, origin, opt, dim_dom, elem_dom);
				}
			}
		}
		let card_idx = match &class_data[declared_type] {
			shackle_hir::Type::Set {
				cardinality: Some(card),
				..
			} if class_data[declared_type]
				.get_new_class(class_data)
				.is_none() =>
			{
				*card
			}
			_ => return self.substitute_class_with_potential_enum_domain(field_ty, origin),
		};
		let class_types = class_item.types(db);
		let card_expr = {
			let mut inner = ExpressionCollector::new(
				self,
				class_item_ref.class(db).data(),
				class_item,
				&class_types,
			);
			inner.collect_expression(card_idx)
		};
		let elem_ty = field_ty.elem_ty(db).unwrap_or(field_ty);
		let inst = subst.inst(db).unwrap_or(VarType::Var);
		let opt = subst.opt(db).unwrap_or(OptType::NonOpt);
		Domain::set_with_card(
			db,
			origin,
			inst,
			opt,
			Some(card_expr),
			self.substitute_class_with_potential_enum_domain(elem_ty, origin),
		)
	}
}
