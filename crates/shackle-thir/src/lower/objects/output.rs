//! Output of objects and references.
//!
//! Inside the lowering an object is an identity, a member of `<C>_potential`.
//! Solutions show it by its string id instead (see `ids.rs`): a reference is
//! rendered as the id of the object it refers to, and the objects introduced by
//! a `new` root are rendered as records `(id: .., <field>: .., ...)`, with owned
//! children nested inside their parent's record.
//!
//! An identity-valued declaration that would be output is renamed
//! `<name>_ref`, marked `::no_output`, and replaced in the output by an
//! `::output_only` declaration carrying the user's name whose value is the
//! rendering. Strings cannot be `var`, so the rendering only exists in the
//! output model, where every value is fixed.

use rustc_hash::FxHashMap;
use shackle_hir::{Identifier, ids::PatternRef};
use shackle_ty::{EnumRef, Ty, TyData};

use crate::{lower::ItemCollector, source::Origin, *};

/// How a top-level declaration whose type mentions a class refers to objects.
#[derive(Clone, Copy)]
pub(in crate::lower) enum OutputCandidateKind<'db> {
	/// A `new` root of `class`, whose value is the identity (or identity set) of
	/// the objects it introduces. Those are rendered in full.
	Root { class: PatternRef<'db> },
	/// Any other declaration: the objects it refers to are rendered as ids.
	Reference,
}

/// Whether a type mentions a class anywhere, e.g. `set of A` or `array [1..3]
/// of opt A`.
pub(in crate::lower) fn ty_mentions_class<'db>(db: &'db dyn Db, ty: Ty<'db>) -> bool {
	match ty.lookup(db) {
		TyData::Class(..) => true,
		TyData::Array { dim, element, .. } => {
			ty_mentions_class(db, *dim) || ty_mentions_class(db, *element)
		}
		TyData::Set(_, _, element) => ty_mentions_class(db, *element),
		TyData::Tuple(_, fields) => fields.iter().any(|field| ty_mentions_class(db, *field)),
		TyData::Record(_, fields) => fields
			.iter()
			.any(|(_, field)| ty_mentions_class(db, *field)),
		_ => false,
	}
}

impl<'db> ItemCollector<'db> {
	/// Replace every identity-valued declaration that would be output by an
	/// `::output_only` rendering of the objects it refers to.
	///
	/// Runs at the end of `finish()`: whether a declaration is output depends
	/// on its definition, which an assignment item may supply after the
	/// declaration was collected, and rendering reads the final `<C>_objects`
	/// and `<C>_ids`.
	pub(in crate::lower) fn emit_object_outputs(&mut self) {
		let candidates = std::mem::take(&mut self.objects.output_candidates);
		if candidates.is_empty() {
			return;
		}
		let potential_classes = self.potential_classes();

		for (decl, kind) in candidates {
			if !self.is_output_declaration(decl) {
				continue;
			}
			// A declaration read from data has given its name to the data
			// declaration, so its rendering is named `<name>_output`.
			let data_input_name = self.objects.data_input_names.get(&decl).copied();
			let Some(name) = data_input_name.or_else(|| self.model[decl].name()) else {
				continue;
			};
			let origin = self.model[decl].origin();
			let value = Expression::new(
				self.db,
				&self.model,
				origin,
				ResolvedIdentifier::Declaration(decl),
			);
			let value = self.output_value(value);
			let rendering = match kind {
				OutputCandidateKind::Root { class } => {
					self.render_introduced_objects(origin, &potential_classes, class, value)
				}
				OutputCandidateKind::Reference => {
					self.render_references(origin, &potential_classes, value)
				}
			};

			let mut output =
				Declaration::new(true, Domain::unbounded(self.db, origin, rendering.ty()));
			output.set_name(if data_input_name.is_some() {
				Identifier::new(self.db, format!("{}_output", name.lookup(self.db)))
			} else {
				name
			});
			output.set_definition(rendering);
			output.annotations_mut().push(Expression::new(
				self.db,
				&self.model,
				origin,
				self.ids.annotations.output_only,
			));
			output.annotations_mut().push(Expression::new(
				self.db,
				&self.model,
				origin,
				self.ids.annotations.output,
			));

			let renamed = Identifier::new(self.db, format!("{}_ref", name.lookup(self.db)));
			let no_output =
				Expression::new(self.db, &self.model, origin, self.ids.annotations.no_output);
			let mut internal = self.model[decl].clone();
			internal.set_name(renamed);
			let _ = internal
				.annotations_mut()
				.remove(&self.model, self.ids.annotations.output);
			internal.annotations_mut().push(no_output);
			self.model[decl] = internal;

			let _ = self
				.model
				.add_declaration(DeclarationItem::new(output, origin));
		}
	}

	/// The argument of a `show` call with every object reference inside it
	/// replaced by the id of the object, so `show` prints ids (quoted, as
	/// strings are). `None` if the argument refers to no object, or is var:
	/// only fixed values have ids to show.
	pub(in crate::lower) fn show_argument_with_ids(
		&mut self,
		argument: &Expression<'db>,
	) -> Option<Expression<'db>> {
		if !argument.ty().known_par(self.db) {
			return None;
		}
		let relabelled = self.output_value(argument.clone());
		let potential_classes = self.potential_classes();
		let origin = argument.origin();
		let rendered = self.render_references(origin, &potential_classes, relabelled);
		(rendered.ty() != argument.ty()).then_some(rendered)
	}

	/// The class whose objects each `<C>_potential` enum enumerates.
	fn potential_classes(&self) -> FxHashMap<EnumRef<'db>, PatternRef<'db>> {
		let mut classes: Vec<PatternRef<'db>> = self.objects.class_map.keys().copied().collect();
		self.objects.plan.in_class_order(&mut classes);
		classes
			.into_iter()
			.map(|class| {
				let info = self.objects.class_map[&class];
				(self.model[info.class_enum].enum_type(), class)
			})
			.collect()
	}

	/// Whether a declaration is part of the output: annotated `::output`, or
	/// an unannotated `var` declaration without a definition.
	fn is_output_declaration(&self, decl: DeclarationId<'db>) -> bool {
		let declaration = &self.model[decl];
		let annotations = declaration.annotations();
		if annotations.has(&self.model, self.ids.annotations.output_only) {
			return false;
		}
		if annotations.has(&self.model, self.ids.annotations.output) {
			return true;
		}
		if annotations.has(&self.model, self.ids.annotations.no_output) {
			return false;
		}
		declaration.definition().is_none() && !declaration.ty().known_par(self.db)
	}

	/// `expr` with the class types in its type relabelled to their potential
	/// enums. The relabel is only a change of type: MiniZinc types the
	/// printed expression from its declaration, whose values already range
	/// over `<C>_potential`.
	pub(in crate::lower) fn with_potential_enum_types(
		&self,
		expr: Expression<'db>,
	) -> Expression<'db> {
		if !ty_mentions_class(self.db, expr.ty()) {
			return expr;
		}
		let ty = self.substitute_class_with_potential_enum(expr.ty());
		let origin = expr.origin();
		let mut relabelled = Expression::new_unchecked(ty, (*expr).clone(), origin);
		relabelled
			.annotations_mut()
			.extend(expr.annotations().iter().cloned());
		relabelled
	}

	/// The value of `expr` as seen in the output model: its class types
	/// relabelled to their potential enums, and fixed if it is var.
	fn output_value(&self, expr: Expression<'db>) -> Expression<'db> {
		let expr = self.with_potential_enum_types(expr);
		if expr.ty().known_par(self.db) {
			return expr;
		}
		let origin = expr.origin();
		Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.fix.into(),
				arguments: vec![expr],
			},
		)
	}

	/// Render the objects a `new` root of `class` introduces, given its (par)
	/// identity value: a record for a singular root, and an array of records
	/// for a collection. An optional object is an array of zero or one
	/// records, as MiniZinc has no `opt record`.
	fn render_introduced_objects(
		&mut self,
		origin: Origin<'db>,
		potential_classes: &FxHashMap<EnumRef<'db>, PatternRef<'db>>,
		class: PatternRef<'db>,
		value: Expression<'db>,
	) -> Expression<'db> {
		match value.ty().lookup(self.db) {
			TyData::Enum(_, OptType::NonOpt, _) => {
				self.render_object(origin, potential_classes, class, value)
			}
			TyData::Enum(_, OptType::Opt, _) => {
				let present =
					Expression::new(self.db, &self.model, origin, ArrayLiteral(vec![value]));
				self.map_members(origin, present, true, |this, member| {
					this.render_object(origin, potential_classes, class, member)
				})
			}
			TyData::Set(..) => self.map_members(origin, value, false, |this, member| {
				this.render_object(origin, potential_classes, class, member)
			}),
			_ => self.render_references(origin, potential_classes, value),
		}
	}

	/// Render the object `identity` of `class` as the record `(id: <id>,
	/// <field>: <value>, ...)` of all its storage fields. Owned children are
	/// rendered in full, references as ids.
	fn render_object(
		&mut self,
		origin: Origin<'db>,
		potential_classes: &FxHashMap<EnumRef<'db>, PatternRef<'db>>,
		class: PatternRef<'db>,
		identity: Expression<'db>,
	) -> Expression<'db> {
		let info = self.objects.class_map[&class];
		let owned = self.owned_child_classes(class);
		let fields = self.model[info.class_objects]
			.ty()
			.elem_ty(self.db)
			.and_then(|record| record.record_fields(self.db))
			.unwrap_or_default();

		let ids = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(info.class_ids),
		);
		let id = Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![ids, identity.clone()],
			},
		);
		// `<C>_objects` is int-indexed by the identity's global ordinal.
		let objects = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(info.class_objects),
		);
		let ordinal = Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.enum2int.into(),
				arguments: vec![identity],
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

		let mut members = vec![(self.ids.names.object_id, id)];
		for (field_name, _) in fields {
			let field = Identifier(field_name);
			let value = Expression::new(
				self.db,
				&self.model,
				origin,
				RecordAccess {
					record: Box::new(object.clone()),
					field,
				},
			);
			let value = self.output_value(value);
			let rendered = match owned.get(&field) {
				Some(child) => {
					self.render_introduced_objects(origin, potential_classes, *child, value)
				}
				None => self.render_references(origin, potential_classes, value),
			};
			members.push((field, rendered));
		}
		Expression::new(self.db, &self.model, origin, RecordLiteral(members))
	}

	/// The attributes of `class` (including inherited ones) that introduce
	/// objects, mapped to the class they introduce.
	fn owned_child_classes(
		&self,
		class: PatternRef<'db>,
	) -> FxHashMap<Identifier<'db>, PatternRef<'db>> {
		self.class_storage_field_decls(class.item(self.db))
			.into_iter()
			.filter_map(|field| {
				let data = field.owner.data(self.db);
				let class_domain = data[field.declared_type].get_new_class(data)?;
				let child = field.owner.types(self.db).name_resolution(class_domain)?;
				Some((field.ident, child))
			})
			.collect()
	}

	/// Render every object reference inside the (par) `value` as the id of the
	/// object it refers to. Sets of references become arrays of ids, as
	/// MiniZinc has no `set of string`.
	fn render_references(
		&mut self,
		origin: Origin<'db>,
		potential_classes: &FxHashMap<EnumRef<'db>, PatternRef<'db>>,
		value: Expression<'db>,
	) -> Expression<'db> {
		let db = self.db;
		let mentions_class = |ty: Ty<'db>| {
			let mut stack = vec![ty];
			while let Some(ty) = stack.pop() {
				match ty.lookup(db) {
					TyData::Enum(_, _, e) if potential_classes.contains_key(e) => return true,
					TyData::Array { element, .. } => stack.push(*element),
					TyData::Set(_, _, element) => stack.push(*element),
					TyData::Tuple(_, fields) => stack.extend(fields.iter().copied()),
					TyData::Record(_, fields) => stack.extend(fields.iter().map(|(_, ty)| *ty)),
					_ => (),
				}
			}
			false
		};
		if !mentions_class(value.ty()) {
			return value;
		}
		match value.ty().lookup(db) {
			TyData::Enum(_, OptType::NonOpt, e) => {
				let class = potential_classes[e];
				self.object_id(origin, class, value)
			}
			TyData::Enum(_, OptType::Opt, e) => {
				let class = potential_classes[e];
				let occurs = Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.occurs.into(),
						arguments: vec![value.clone()],
					},
				);
				let deopt = Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.deopt.into(),
						arguments: vec![value],
					},
				);
				let id = self.object_id(origin, class, deopt);
				let absent = Expression::new(db, &self.model, origin, Absent);
				Expression::new(
					db,
					&self.model,
					origin,
					IfThenElse {
						branches: vec![Branch::new(occurs, id)],
						else_result: Box::new(absent),
					},
				)
			}
			TyData::Set(..) => self.map_members(origin, value, false, |this, member| {
				this.render_references(origin, potential_classes, member)
			}),
			TyData::Array { .. } => {
				// `arrayXd` keeps the index sets of the original array.
				let elements = self.map_members(origin, value.clone(), false, |this, member| {
					this.render_references(origin, potential_classes, member)
				});
				Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.array_xd.into(),
						arguments: vec![value, elements],
					},
				)
			}
			TyData::Tuple(_, fields) => {
				let fields = (1..=fields.len())
					.map(|i| {
						let field = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(value.clone()),
								field: IntegerLiteral(i as i64),
							},
						);
						self.render_references(origin, potential_classes, field)
					})
					.collect();
				Expression::new(db, &self.model, origin, TupleLiteral(fields))
			}
			TyData::Record(_, fields) => {
				let fields = fields
					.iter()
					.map(|(name, _)| {
						let field = Identifier(*name);
						let access = Expression::new(
							db,
							&self.model,
							origin,
							RecordAccess {
								record: Box::new(value.clone()),
								field,
							},
						);
						(
							field,
							self.render_references(origin, potential_classes, access),
						)
					})
					.collect();
				Expression::new(db, &self.model, origin, RecordLiteral(fields))
			}
			_ => value,
		}
	}

	/// `<C>_ids[<identity>]`
	fn object_id(
		&self,
		origin: Origin<'db>,
		class: PatternRef<'db>,
		identity: Expression<'db>,
	) -> Expression<'db> {
		let ids = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(self.objects.class_map[&class].class_ids),
		);
		Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.array_access.into(),
				arguments: vec![ids, identity],
			},
		)
	}

	/// `[<f(x)> | x in <collection>]`, or with `only_occurring`,
	/// `[<f(deopt(x))> | x in <collection> where occurs(x)]`.
	fn map_members(
		&mut self,
		origin: Origin<'db>,
		collection: Expression<'db>,
		only_occurring: bool,
		f: impl FnOnce(&mut Self, Expression<'db>) -> Expression<'db>,
	) -> Expression<'db> {
		let member_ty = collection
			.ty()
			.elem_ty(self.db)
			.expect("collection has an element type");
		let member_decl = self.model.add_declaration(DeclarationItem::new(
			Declaration::new(false, Domain::unbounded(self.db, origin, member_ty)),
			origin,
		));
		let member = Expression::new(self.db, &self.model, origin, member_decl);
		let (member, where_clause) = if only_occurring {
			let occurs = Expression::new(
				self.db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.functions.occurs.into(),
					arguments: vec![member.clone()],
				},
			);
			let deopt = Expression::new(
				self.db,
				&self.model,
				origin,
				LookupCall {
					function: self.ids.functions.deopt.into(),
					arguments: vec![member],
				},
			);
			(deopt, Some(occurs))
		} else {
			(member, None)
		};
		let template = f(self, member);
		Expression::new(
			self.db,
			&self.model,
			origin,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![member_decl],
					collection,
					where_clause,
				}],
				template,
			),
		)
	}
}
