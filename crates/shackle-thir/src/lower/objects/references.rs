//! Constraining references to existing objects.
//!
//! A reference to class `C` is lowered with the domain `<C>_potential`, every
//! object that could exist, because the actual set `<C>` can be a var set,
//! which is not a valid domain. When the existence of `C`'s objects is a
//! decision, that domain lets a reference point at an object that does not
//! exist, so each reference is also constrained to the actual set. For a class
//! whose objects all exist, the potential objects are the actual ones and no
//! constraint is needed.

use rustc_hash::FxHashMap;
use shackle_hir::ids::PatternRef;
use shackle_ty::{EnumRef, TyData};

use super::OutputCandidateKind;
use crate::{lower::ItemCollector, source::Origin, *};

impl<'db> ItemCollector<'db> {
	/// Constrain every top-level reference declaration, and every reference
	/// attribute of an existing object, to refer to existing objects.
	///
	/// Runs in `finish()`, once every class's actual set and `<C>_objects`
	/// are defined, and before `emit_object_outputs` consumes the candidates.
	pub(in crate::lower) fn emit_reference_existence_constraints(&mut self) {
		let actual_sets = self.var_actual_sets();
		if actual_sets.is_empty() {
			return;
		}

		let references = self
			.objects
			.output_candidates
			.iter()
			.filter(|(_, kind)| matches!(kind, OutputCandidateKind::Reference))
			.map(|(decl, _)| *decl)
			.collect::<Vec<_>>();
		for decl in references {
			let origin = self.model[decl].origin();
			let value = Expression::new(
				self.db,
				&self.model,
				origin,
				ResolvedIdentifier::Declaration(decl),
			);
			let value = self.with_potential_enum_types(value);
			if let Some(condition) = self.refers_to_existing(origin, &actual_sets, value) {
				let _ = self.model.add_constraint(ConstraintItem::new(
					Constraint::new(true, condition),
					origin,
				));
			}
		}

		let mut classes: Vec<PatternRef<'db>> = self.objects.class_map.keys().copied().collect();
		self.objects.plan.in_class_order(&mut classes);
		for class in classes {
			self.constrain_reference_attributes(class, &actual_sets);
		}
	}

	/// The actual set of every class whose existence is a decision, keyed by
	/// the class's potential enum.
	fn var_actual_sets(&self) -> FxHashMap<EnumRef<'db>, DeclarationId<'db>> {
		let mut classes: Vec<PatternRef<'db>> = self
			.objects
			.plan
			.var_actual_set_classes
			.iter()
			.copied()
			.collect();
		self.objects.plan.in_class_order(&mut classes);
		classes
			.into_iter()
			.filter_map(|class| {
				let info = self.objects.class_map.get(&class)?;
				Some((self.model[info.class_enum].enum_type(), info.class_set))
			})
			.collect()
	}

	/// `forall(this in <C>)(<attribute refers to existing objects>)` for each
	/// reference attribute `class` declares itself. An inherited attribute is
	/// constrained by the class declaring it: the superclass's storage also
	/// holds its subclasses' objects. Owned `new` attributes need no
	/// constraint, as their children exist exactly when they are introduced.
	fn constrain_reference_attributes(
		&mut self,
		class: PatternRef<'db>,
		actual_sets: &FxHashMap<EnumRef<'db>, DeclarationId<'db>>,
	) {
		let class_item = class.item(self.db);
		let origin: Origin<'db> = class_item.into();
		let info = self.objects.class_map[&class];
		let field_tys = self.model[info.class_objects]
			.ty()
			.elem_ty(self.db)
			.and_then(|record| record.record_fields(self.db))
			.unwrap_or_default();
		let reference_fields = self
			.class_storage_field_decls(class_item)
			.into_iter()
			.filter(|field| {
				let data = field.owner.data(self.db);
				field.owner == class_item && data[field.declared_type].get_new_class(data).is_none()
			})
			.filter(|field| field_tys.iter().any(|(name, _)| *name == field.ident.0))
			.map(|field| field.ident)
			.collect::<Vec<_>>();
		if reference_fields.is_empty() {
			return;
		}

		let this_ty = self.model[info.class_set]
			.ty()
			.elem_ty(self.db)
			.expect("class set has an element type")
			.make_par(self.db);
		for field in reference_fields {
			let this_decl = self.model.add_declaration(DeclarationItem::new(
				Declaration::new(false, Domain::unbounded(self.db, origin, this_ty)),
				origin,
			));
			let value = self.attribute_of(origin, class, this_decl, field);
			let Some(condition) = self.refers_to_existing(origin, actual_sets, value) else {
				continue;
			};
			let class_set = Expression::new(
				self.db,
				&self.model,
				origin,
				ResolvedIdentifier::Declaration(info.class_set),
			);
			let conditions = Expression::new(
				self.db,
				&self.model,
				origin,
				ArrayComprehension::new(
					[Generator::Iterator {
						declarations: vec![this_decl],
						collection: class_set,
						where_clause: None,
					}],
					condition,
				),
			);
			let forall = Expression::new(
				self.db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.functions.forall.into(),
					arguments: vec![conditions],
				},
			);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(Constraint::new(true, forall), origin));
		}
	}

	/// `<C>_objects[enum2int(this)].<field>`
	fn attribute_of(
		&self,
		origin: Origin<'db>,
		class: PatternRef<'db>,
		this: DeclarationId<'db>,
		field: Identifier<'db>,
	) -> Expression<'db> {
		let info = self.objects.class_map[&class];
		let objects = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(info.class_objects),
		);
		let this = Expression::new(self.db, &self.model, origin, this);
		let ordinal = Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.enum2int.into(),
				arguments: vec![this],
			},
		);
		let object = Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![objects, ordinal],
			},
		);
		Expression::new(
			self.db,
			&self.model,
			origin,
			RecordAccess {
				record: Box::new(object),
				field,
			},
		)
	}

	/// The condition that every reference inside `value` refers to an existing
	/// object, or `None` if it refers to no class whose existence is a
	/// decision.
	fn refers_to_existing(
		&mut self,
		origin: Origin<'db>,
		actual_sets: &FxHashMap<EnumRef<'db>, DeclarationId<'db>>,
		value: Expression<'db>,
	) -> Option<Expression<'db>> {
		let db = self.db;
		let call = |this: &Self, function: Identifier<'db>, arguments| {
			Expression::new(
				db,
				&this.model,
				origin,
				LookupCall {
					function: function.into(),
					arguments,
				},
			)
		};
		let actual_set = |this: &Self, set: DeclarationId<'db>| {
			Expression::new(
				db,
				&this.model,
				origin,
				ResolvedIdentifier::Declaration(set),
			)
		};
		match value.ty().lookup(db) {
			TyData::Enum(_, OptType::NonOpt, e) => {
				let set = actual_set(self, *actual_sets.get(e)?);
				Some(call(self, self.ids.functions.in_, vec![value, set]))
			}
			TyData::Enum(_, OptType::Opt, e) => {
				let set = actual_set(self, *actual_sets.get(e)?);
				let occurs = call(self, self.ids.functions.occurs, vec![value.clone()]);
				let deopt = call(self, self.ids.functions.deopt, vec![value]);
				let member = call(self, self.ids.functions.in_, vec![deopt, set]);
				Some(call(self, self.ids.functions.implies, vec![occurs, member]))
			}
			TyData::Set(_, _, element) => {
				let TyData::Enum(_, _, e) = element.lookup(db) else {
					return None;
				};
				let set = actual_set(self, *actual_sets.get(e)?);
				// Class sets are compared as sets of ordinals, as elsewhere in
				// the object lowering.
				let value = call(self, self.ids.functions.enum2int, vec![value]);
				let set = call(self, self.ids.functions.enum2int, vec![set]);
				Some(call(self, self.ids.functions.subset, vec![value, set]))
			}
			TyData::Array { element, .. } => {
				let element_decl = self.model.add_declaration(DeclarationItem::new(
					Declaration::new(false, Domain::unbounded(db, origin, *element)),
					origin,
				));
				let element = Expression::new(db, &self.model, origin, element_decl);
				let condition = self.refers_to_existing(origin, actual_sets, element)?;
				let conditions = Expression::new(
					db,
					&self.model,
					origin,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![element_decl],
							collection: value,
							where_clause: None,
						}],
						condition,
					),
				);
				Some(call(self, self.ids.functions.forall, vec![conditions]))
			}
			TyData::Tuple(_, fields) => {
				let conditions = (1..=fields.len())
					.filter_map(|i| {
						let field = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(value.clone()),
								field: IntegerLiteral(i as i64),
							},
						);
						self.refers_to_existing(origin, actual_sets, field)
					})
					.collect::<Vec<_>>();
				self.conjunction(origin, conditions)
			}
			TyData::Record(_, fields) => {
				let conditions = fields
					.iter()
					.filter_map(|(name, _)| {
						let field = Expression::new(
							db,
							&self.model,
							origin,
							RecordAccess {
								record: Box::new(value.clone()),
								field: Identifier(*name),
							},
						);
						self.refers_to_existing(origin, actual_sets, field)
					})
					.collect::<Vec<_>>();
				self.conjunction(origin, conditions)
			}
			_ => None,
		}
	}

	/// `forall([<conditions>])`, or `None` for no conditions.
	fn conjunction(
		&self,
		origin: Origin<'db>,
		conditions: Vec<Expression<'db>>,
	) -> Option<Expression<'db>> {
		if conditions.is_empty() {
			return None;
		}
		let conditions = Expression::new(self.db, &self.model, origin, ArrayLiteral(conditions));
		Some(Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.forall.into(),
				arguments: vec![conditions],
			},
		))
	}
}
