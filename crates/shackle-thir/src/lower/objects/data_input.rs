//! Objects and references given in data files.
//!
//! A `new` root or a reference that the model leaves uninitialised is read
//! from a data file (`.dzn` or `.json`) under the user's own name. Its data
//! form differs from the form an inline model literal takes: every object
//! record carries an `id: string`, nested records included, and a reference is
//! written as the id of the object it refers to (a set of references as an
//! array of ids, as MiniZinc has no `set of string`).
//!
//! The reconstruction resolves those ids through `mzn_object_from_id` against
//! `<C>_data_objects`, the objects of `C` that came from data: data can only
//! refer to objects given in the data. The ids themselves become the objects'
//! entries in `<C>_ids`, and every class hierarchy with data objects checks
//! that its ids are distinct.

use rustc_hash::FxHashMap;
use shackle_hir::{
	Item, StringLiteral,
	class_analysis::{LocalDomainSource, OccurrenceId, class_pattern_for},
	ids::PatternRef,
};
use shackle_ty::{Ty, TyData};

use super::{StorageFieldDecl, ty_mentions_class};
use crate::{lower::ItemCollector, source::Origin, *};

impl<'db> ItemCollector<'db> {
	/// The data form of `inline`, the input record type of `class`: `id:
	/// string` added, references replaced by ids, and the records of owned
	/// children converted recursively.
	pub(in crate::lower) fn data_input_record_ty(
		&self,
		class: PatternRef<'db>,
		inline: Ty<'db>,
	) -> Ty<'db> {
		let db = self.db;
		let TyData::Record(opt, fields) = inline.lookup(db) else {
			unreachable!("object input type is a record")
		};
		let owned = self.owned_child_fields(class);
		let mut data_fields = vec![(self.ids.names.object_id.0, Ty::string(db))];
		for (name, ty) in fields.iter() {
			let ty = match owned.iter().find(|(field, _)| field.0 == *name) {
				Some((_, child)) => self.map_input_records(*ty, *child),
				None => self.references_as_ids_ty(*ty),
			};
			data_fields.push((*name, ty));
		}
		Ty::record(db, data_fields).with_opt(db, *opt)
	}

	/// `ty` (a record, or an array or optional record of an owned child's
	/// inline input) with each child record in its data form.
	fn map_input_records(&self, ty: Ty<'db>, child: PatternRef<'db>) -> Ty<'db> {
		match ty.lookup(self.db) {
			TyData::Record(..) => self.data_input_record_ty(child, ty),
			TyData::Array { opt, dim, element } => {
				Ty::array(self.db, *dim, self.map_input_records(*element, child))
					.expect("array of object records")
					.with_opt(self.db, *opt)
			}
			_ => ty,
		}
	}

	/// `ty` with every reference replaced by the id of the object it refers
	/// to: a reference becomes a `string`, and a set of references an array of
	/// strings.
	pub(in crate::lower) fn references_as_ids_ty(&self, ty: Ty<'db>) -> Ty<'db> {
		let db = self.db;
		if !ty_mentions_class(db, ty) {
			return ty;
		}
		match ty.lookup(db) {
			TyData::Class(_, opt, _) => Ty::string(db).with_opt(db, *opt),
			TyData::Set(_, _, element) => {
				Ty::array(db, Ty::par_int(db), self.references_as_ids_ty(*element))
					.expect("array of ids")
			}
			TyData::Array { opt, dim, element } => {
				Ty::array(db, *dim, self.references_as_ids_ty(*element))
					.expect("array of ids")
					.with_opt(db, *opt)
			}
			TyData::Tuple(opt, fields) => Ty::tuple(
				db,
				fields
					.iter()
					.map(|field| self.references_as_ids_ty(*field))
					.collect::<Vec<_>>(),
			)
			.with_opt(db, *opt),
			TyData::Record(opt, fields) => Ty::record(
				db,
				fields
					.iter()
					.map(|(name, field)| (*name, self.references_as_ids_ty(*field)))
					.collect::<Vec<_>>(),
			)
			.with_opt(db, *opt),
			_ => ty,
		}
	}

	/// Whether `ty` can be read from data with its references given as ids.
	/// Arrays indexed by a class cannot, as their index sets are not data.
	pub(in crate::lower) fn has_data_form(&self, ty: Ty<'db>) -> bool {
		match ty.lookup(self.db) {
			TyData::Array { dim, element, .. } => {
				!ty_mentions_class(self.db, *dim) && self.has_data_form(*element)
			}
			TyData::Set(_, _, element) => self.has_data_form(*element),
			TyData::Tuple(_, fields) => fields.iter().all(|field| self.has_data_form(*field)),
			TyData::Record(_, fields) => fields.iter().all(|(_, field)| self.has_data_form(*field)),
			_ => true,
		}
	}

	/// Whether the input record `input` gives the reference attribute `field`,
	/// stored as `storage_ty`, in its data form, as ids to resolve.
	pub(in crate::lower) fn is_data_reference_field(
		&self,
		field: &StorageFieldDecl<'db>,
		storage_ty: Ty<'db>,
		input: &Expression<'db>,
	) -> bool {
		let data = field.owner.data(self.db);
		if data[field.declared_type].get_new_class(data).is_some()
			|| !ty_mentions_class(self.db, storage_ty)
		{
			return false;
		}
		// Storage read from elsewhere (a var root's free storage) already
		// holds identities.
		let ids_ty = self.references_as_ids_ty(storage_ty.make_par(self.db));
		input
			.ty()
			.record_fields(self.db)
			.and_then(|fields| fields.into_iter().find(|(name, _)| *name == field.ident.0))
			.is_some_and(|(_, ty)| ty == ids_ty)
	}

	/// `value`, in the data form of `ty` (see `references_as_ids_ty`), with
	/// every id resolved to the object it names.
	pub(in crate::lower) fn resolve_data_references(
		&mut self,
		origin: Origin<'db>,
		value: Expression<'db>,
		ty: Ty<'db>,
	) -> Expression<'db> {
		let db = self.db;
		if !ty_mentions_class(db, ty) {
			return value;
		}
		match ty.lookup(db) {
			TyData::Class(_, OptType::NonOpt, class) => {
				let class = class_pattern_for(db, *class).expect("class item for class type");
				self.object_from_id(origin, class, value)
			}
			TyData::Class(_, OptType::Opt, class) => {
				let class = class_pattern_for(db, *class).expect("class item for class type");
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
				let object = self.object_from_id(origin, class, deopt);
				let absent = Expression::new(db, &self.model, origin, Absent);
				Expression::new(
					db,
					&self.model,
					origin,
					IfThenElse {
						branches: vec![Branch::new(occurs, object)],
						else_result: Box::new(absent),
					},
				)
			}
			TyData::Set(_, _, element) => {
				let element = *element;
				let (generator, member) = self.member_generator(origin, value);
				let template = self.resolve_data_references(origin, member, element);
				Expression::new(
					db,
					&self.model,
					origin,
					SetComprehension {
						template: Box::new(template),
						generators: vec![generator],
					},
				)
			}
			TyData::Array { element, .. } => {
				let element = *element;
				let (generator, member) = self.member_generator(origin, value.clone());
				let template = self.resolve_data_references(origin, member, element);
				let resolved = Expression::new(
					db,
					&self.model,
					origin,
					ArrayComprehension::new([generator], template),
				);
				// `arrayXd` keeps the index sets of the data array.
				Expression::new(
					db,
					&self.model,
					origin,
					LookupCall {
						function: self.ids.functions.array_xd.into(),
						arguments: vec![value, resolved],
					},
				)
			}
			TyData::Tuple(_, fields) => {
				let fields = fields.to_vec();
				let fields = fields
					.into_iter()
					.enumerate()
					.map(|(i, field_ty)| {
						let field = Expression::new(
							db,
							&self.model,
							origin,
							TupleAccess {
								tuple: Box::new(value.clone()),
								field: IntegerLiteral(i as i64 + 1),
							},
						);
						self.resolve_data_references(origin, field, field_ty)
					})
					.collect();
				Expression::new(db, &self.model, origin, TupleLiteral(fields))
			}
			TyData::Record(_, fields) => {
				let fields = fields.to_vec();
				let fields = fields
					.into_iter()
					.map(|(name, field_ty)| {
						let field = Identifier(name);
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
							self.resolve_data_references(origin, access, field_ty),
						)
					})
					.collect();
				Expression::new(db, &self.model, origin, RecordLiteral(fields))
			}
			_ => value,
		}
	}

	/// `x in <collection>`, and `x`.
	fn member_generator(
		&mut self,
		origin: Origin<'db>,
		collection: Expression<'db>,
	) -> (Generator<'db>, Expression<'db>) {
		let member_ty = collection
			.ty()
			.elem_ty(self.db)
			.expect("collection has an element type");
		let member_decl = self.model.add_declaration(DeclarationItem::new(
			Declaration::new(false, Domain::unbounded(self.db, origin, member_ty)),
			origin,
		));
		let member = Expression::new(self.db, &self.model, origin, member_decl);
		(
			Generator::Iterator {
				declarations: vec![member_decl],
				collection,
				where_clause: None,
			},
			member,
		)
	}

	/// `mzn_object_from_id(<C>_ids, <C>_data_objects, <id>, "<C>")`
	fn object_from_id(
		&mut self,
		origin: Origin<'db>,
		class: PatternRef<'db>,
		id: Expression<'db>,
	) -> Expression<'db> {
		let ids = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(self.objects.class_map[&class].class_ids),
		);
		let data_objects_decl = self.class_data_objects(class);
		let data_objects = Expression::new(
			self.db,
			&self.model,
			origin,
			ResolvedIdentifier::Declaration(data_objects_decl),
		);
		let class_name = Expression::new(
			self.db,
			&self.model,
			origin,
			StringLiteral::new(self.db, class.identifier(self.db).unwrap().lookup(self.db)),
		);
		Expression::new(
			self.db,
			&self.model,
			origin,
			LookupCall {
				function: self.ids.functions.mzn_object_from_id.into(),
				arguments: vec![ids, data_objects, id, class_name],
			},
		)
	}

	/// `set of <C>_potential: <C>_data_objects`, the objects of `class` that
	/// came from data. Declared the first time data refers to `class`, and
	/// defined in `finish()` once every constructor is known.
	fn class_data_objects(&mut self, class: PatternRef<'db>) -> DeclarationId<'db> {
		if let Some(decl) = self.objects.class_data_objects.get(&class) {
			return *decl;
		}
		let item = class.item(self.db);
		let class_enum = self.objects.class_map[&class].class_enum;
		let ty = Ty::par_set(
			self.db,
			Ty::par_enum(self.db, self.model[class_enum].enum_type()),
		)
		.expect("set of potential objects");
		let mut declaration = Declaration::new(true, Domain::unbounded(self.db, item, ty));
		declaration.set_name(Identifier::new(
			self.db,
			format!(
				"{}_data_objects",
				class.identifier(self.db).unwrap().lookup(self.db)
			),
		));
		let decl = self
			.model
			.add_declaration(DeclarationItem::new(declaration, item));
		let _ = self.objects.class_data_objects.insert(class, decl);
		decl
	}

	/// The attributes of `class` (including inherited ones) that introduce
	/// objects, with the class they introduce, in declaration order.
	pub(in crate::lower) fn owned_child_fields(
		&self,
		class: PatternRef<'db>,
	) -> Vec<(Identifier<'db>, PatternRef<'db>)> {
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

	/// Record the data ids of the objects of `class` that the data record
	/// `record` holds, at `path` below the data root `root_pattern`, and those
	/// of its owned children. `generators` range over the records of the
	/// occurrence in slot order: the reconstruction numbers the objects of a
	/// nested occurrence in the order of this walk (parents first, then their
	/// children in data order).
	pub(in crate::lower) fn register_data_id_blocks(
		&mut self,
		origin: Origin<'db>,
		root_pattern: PatternRef<'db>,
		class: PatternRef<'db>,
		path: Vec<Identifier<'db>>,
		generators: Vec<Generator<'db>>,
		record: Expression<'db>,
	) {
		let occurrence = if path.is_empty() {
			Some(self.top_level_occurrence(root_pattern))
		} else {
			self.maybe_nested_occurrence(root_pattern, &path)
		};
		let Some(occurrence) = occurrence else {
			return;
		};
		let id = Expression::new(
			self.db,
			&self.model,
			origin,
			RecordAccess {
				record: Box::new(record.clone()),
				field: self.ids.names.object_id,
			},
		);
		let block = if generators.is_empty() {
			Expression::new(self.db, &self.model, origin, ArrayLiteral(vec![id]))
		} else {
			Expression::new(
				self.db,
				&self.model,
				origin,
				ArrayComprehension::new(generators.clone(), id),
			)
		};
		let _ = self.objects.data_id_blocks.insert(occurrence, block);

		for (field, child) in self.owned_child_fields(class) {
			// A var-existence child is a decision, not data, so its field is
			// missing from the record; its objects keep their generated ids.
			if !self.record_ty_has_field(&record, field) {
				continue;
			}
			let children = Expression::new(
				self.db,
				&self.model,
				origin,
				RecordAccess {
					record: Box::new(record.clone()),
					field,
				},
			);
			let mut child_path = path.clone();
			child_path.push(field);
			if children.ty().record_fields(self.db).is_some() {
				self.register_data_id_blocks(
					origin,
					root_pattern,
					child,
					child_path,
					generators.clone(),
					children,
				);
			} else {
				// `set of new` children, and an `opt new` child as its list of
				// zero or one records.
				let (generator, member) = self.member_generator(origin, children);
				let mut child_generators = generators.clone();
				child_generators.push(generator);
				self.register_data_id_blocks(
					origin,
					root_pattern,
					child,
					child_path,
					child_generators,
					member,
				);
			}
		}
	}

	/// The ids of the objects of a constructor whose occurrence has data
	/// objects: `data`, the data ids in slot order, followed by generated ids
	/// for the slots that no data object fills. Only a singular nested child
	/// has such slots: its constructor has one slot per potential object of
	/// the parent class, whose other introductions give it children that never
	/// exist.
	pub(in crate::lower) fn data_occurrence_ids_block(
		&mut self,
		item: Item<'db>,
		occurrence: OccurrenceId,
		domain: Expression<'db>,
		data: Expression<'db>,
		unfilled: impl FnOnce(&mut Self, Expression<'db>, Expression<'db>) -> Expression<'db>,
	) -> Expression<'db> {
		if self.occurrence_local_domain_source(occurrence) != LocalDomainSource::OnePerParent {
			return data;
		}
		let count = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.length.into(),
				arguments: vec![data.clone()],
			},
		);
		let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		let first = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.plus.into(),
				arguments: vec![count, one],
			},
		);
		let last = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.card.into(),
				arguments: vec![domain],
			},
		);
		let generated = unfilled(self, first, last);
		Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.plus_plus.into(),
				arguments: vec![data, generated],
			},
		)
	}

	/// Define `<C>_data_objects` for every class that data refers to, and check
	/// that the ids of every class hierarchy with data objects are distinct.
	/// Runs in `finish()` after `define_class_ids`.
	pub(in crate::lower) fn define_class_data_objects(
		&mut self,
		occurrence_of: &FxHashMap<(PatternRef<'db>, usize), OccurrenceId>,
	) {
		let mut data_classes: Vec<PatternRef<'db>> = occurrence_of
			.iter()
			.filter(|(_, occurrence)| self.objects.data_id_blocks.contains_key(occurrence))
			.map(|((class, _), _)| *class)
			.collect();
		self.objects.plan.in_class_order(&mut data_classes);
		data_classes.dedup();

		let mut referenced: Vec<(PatternRef<'db>, DeclarationId<'db>)> = self
			.objects
			.class_data_objects
			.iter()
			.map(|(class, decl)| (*class, *decl))
			.collect();
		referenced.sort_by_key(|(class, _)| self.objects.plan.class_rank(*class));
		for (class, decl) in referenced {
			let item = class.item(self.db);
			let class_enum = self.objects.class_map[&class].class_enum;
			let constructor_count = self.model[class_enum]
				.definition()
				.map(|constructors| constructors.len())
				.unwrap_or(0);
			let mut objects: Option<Expression<'db>> = None;
			for constructor_index in 0..constructor_count {
				let occurrence = occurrence_of[&(class, constructor_index)];
				let Some(data) = self.objects.data_id_blocks.get(&occurrence).cloned() else {
					continue;
				};
				let image = if self.occurrence_local_domain_source(occurrence)
					== LocalDomainSource::OnePerParent
				{
					// Only the first `length(data)` slots hold data objects.
					let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
					let count = Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.length.into(),
							arguments: vec![data],
						},
					);
					let slots = Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.dot_dot.into(),
							arguments: vec![one, count],
						},
					);
					Expression::new(
						self.db,
						&self.model,
						item,
						Call {
							function: Callable::EnumConstructor(EnumMemberId::new(
								class_enum,
								constructor_index as u32,
							)),
							arguments: vec![slots],
						},
					)
				} else {
					self.class_enum_constructor_image_set(item, class_enum, constructor_index)
						.expect("object constructor has a bounded parameter domain")
				};
				objects = Some(match objects {
					None => image,
					Some(objects) => Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.union.into(),
							arguments: vec![objects, image],
						},
					),
				});
			}
			let definition = objects.unwrap_or_else(|| {
				Expression::new_unchecked(self.model[decl].ty(), SetLiteral(vec![]), item)
			});
			self.model[decl].set_definition(definition);
		}

		// Ids are unique per class hierarchy, and the table of a hierarchy's
		// root class holds every object of the hierarchy. Generated ids are
		// distinct by construction, so only hierarchies with data are checked.
		let mut roots: Vec<PatternRef<'db>> = data_classes
			.into_iter()
			.map(|class| self.hierarchy_root(class))
			.collect();
		self.objects.plan.in_class_order(&mut roots);
		roots.dedup();
		for root in roots {
			let item = root.item(self.db);
			let ids = Expression::new(
				self.db,
				&self.model,
				item,
				ResolvedIdentifier::Declaration(self.objects.class_map[&root].class_ids),
			);
			let class_name = Expression::new(
				self.db,
				&self.model,
				item,
				StringLiteral::new(self.db, root.identifier(self.db).unwrap().lookup(self.db)),
			);
			let distinct = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.mzn_object_ids_distinct.into(),
					arguments: vec![ids, class_name],
				},
			);
			let _ = self
				.model
				.add_constraint(ConstraintItem::new(Constraint::new(true, distinct), item));
		}
	}

	/// The class at the top of `class`'s (single) inheritance chain.
	fn hierarchy_root(&self, class: PatternRef<'db>) -> PatternRef<'db> {
		let mut class = class;
		loop {
			let item = class.item(self.db);
			let Item::Class(class_item) = item else {
				unreachable!("class pattern belongs to a class item")
			};
			let Some(base) = class_item
				.class(self.db)
				.extends
				.and_then(|base| item.types(self.db).name_resolution(base))
			else {
				return class;
			};
			class = base;
		}
	}
}
