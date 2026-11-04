//! String ids of objects.
//!
//! Every potential object has a string id, which identifies it in data files
//! and solutions. `<C>_ids` (declared in `predeclare_class`) maps each member of
//! `<C>_potential` to its id. The ids are generated from the occurrence that
//! introduces the object (see [`OccurrenceIdPrefix`]), so a superclass
//! projection, which shares its occurrence with the direct contribution, gives
//! an object the same id in every table of its hierarchy.

use rustc_hash::FxHashMap;
use shackle_hir::{Item, StringLiteral, class_analysis::OccurrenceId, ids::PatternRef};
use shackle_ty::Ty;

use super::OccurrenceIdPrefix;
use crate::{lower::ItemCollector, *};

impl<'db> ItemCollector<'db> {
	/// Define every class's `<C>_ids` as the concatenation of one block of ids
	/// per constructor of `<C>_potential`, in constructor order.
	///
	/// Runs in `finish()` once every constructor has been added, including the
	/// empty definitions of classes without introductions.
	pub(in crate::lower) fn define_class_ids(&mut self) {
		let mut occurrence_of: FxHashMap<(PatternRef<'db>, usize), OccurrenceId> =
			FxHashMap::default();
		for contributions in self.objects.plan.contributions_in_occurrence_order() {
			for contribution in contributions {
				let _ = occurrence_of.insert(
					(contribution.target_class, contribution.constructor_index),
					contribution.occurrence,
				);
			}
		}

		let mut classes: Vec<PatternRef<'db>> = self.objects.class_map.keys().copied().collect();
		self.objects.plan.in_class_order(&mut classes);
		for class in classes {
			let info = self.objects.class_map[&class];
			let item = class.item(self.db);
			let constructor_count = self.model[info.class_enum]
				.definition()
				.map(|constructors| constructors.len())
				.unwrap_or(0);
			let mut ids: Option<Expression<'db>> = None;
			for constructor_index in 0..constructor_count {
				let domain = self
					.class_enum_constructor_domain(info.class_enum, constructor_index)
					.expect("object constructor has a bounded parameter domain");
				let occurrence = occurrence_of[&(class, constructor_index)];
				let prefix = self.objects.plan.occurrence_id_prefixes[&occurrence].clone();
				let present = self.objects.opt_child_presence.get(&occurrence).copied();
				let block = match self.objects.data_id_blocks.get(&occurrence).cloned() {
					Some(given) => {
						let data = self.data_object_ids(item, given.clone(), &prefix, present);
						self.data_occurrence_ids_block(
							item,
							occurrence,
							domain,
							given,
							data,
							|this, first, last| this.indexed_ids(item, &prefix.prefix, first, last),
						)
					}
					None => self.occurrence_ids_block(item, domain, &prefix, present),
				};
				ids = Some(match ids {
					None => block,
					Some(ids) => Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.plus_plus.into(),
							arguments: vec![ids, block],
						},
					),
				});
			}
			let definition = match ids {
				Some(ids) => {
					let potential = Expression::new(self.db, &self.model, item, info.class_enum);
					Expression::new(
						self.db,
						&self.model,
						item,
						LookupCall {
							function: self.ids.functions.array1d.into(),
							arguments: vec![potential, ids],
						},
					)
				}
				None => Expression::new_unchecked(
					self.model[info.class_ids].ty(),
					ArrayLiteral(vec![]),
					item,
				),
			};
			self.model[info.class_ids].set_definition(definition);
		}
		self.define_class_data_objects(&occurrence_of);
	}

	/// The ids of the potential objects of one constructor, whose parameter
	/// domain is `domain`: `[<prefix> ++ "[" ++ show(i) ++ "]" | i in
	/// 1..card(<domain>)]`, or `[<prefix>]` for the single object of a
	/// singular chain. The objects of an optional child occurrence, whose
	/// parents have `present` children each, take their parents' positions.
	fn occurrence_ids_block(
		&mut self,
		item: Item<'db>,
		domain: Expression<'db>,
		prefix: &OccurrenceIdPrefix,
		present: Option<DeclarationId<'db>>,
	) -> Expression<'db> {
		// The single object of a singular root: `{1}`.
		if prefix.singular
			&& matches!(&*domain, ExpressionData::SetLiteral(members) if members.len() == 1)
		{
			let name = self.string_literal(item, &prefix.prefix);
			return Expression::new(self.db, &self.model, item, ArrayLiteral(vec![name]));
		}

		let count = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.card.into(),
				arguments: vec![domain],
			},
		);
		let indexed = match present {
			Some(present) => self.present_parent_ids(item, &prefix.prefix, present),
			None => {
				let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
				self.indexed_ids(item, &prefix.prefix, one, count.clone())
			}
		};
		if !prefix.singular {
			return indexed;
		}

		// A singular chain has one potential object, unless the parent class has
		// potential objects from other introductions as well, whose slots in
		// this chain are never realised. Those still need distinct ids.
		let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
		let is_single = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.eq.into(),
				arguments: vec![count, one],
			},
		);
		let name = self.string_literal(item, &prefix.prefix);
		let single = Expression::new(self.db, &self.model, item, ArrayLiteral(vec![name]));
		Expression::new(
			self.db,
			&self.model,
			item,
			IfThenElse {
				branches: vec![Branch::new(is_single, single)],
				else_result: Box::new(indexed),
			},
		)
	}

	/// The ids of the data objects of an occurrence, from `given`, their ids in
	/// the data in slot order (`opt string`): `mzn_object_data_ids(<given>,
	/// <generated>)`, so an object given without an id gets the id generated for
	/// its slot.
	fn data_object_ids(
		&mut self,
		item: Item<'db>,
		given: Expression<'db>,
		prefix: &OccurrenceIdPrefix,
		present: Option<DeclarationId<'db>>,
	) -> Expression<'db> {
		let generated = if prefix.singular {
			// A singular chain has a single data object.
			let name = self.string_literal(item, &prefix.prefix);
			Expression::new(self.db, &self.model, item, ArrayLiteral(vec![name]))
		} else if let Some(present) = present {
			// Optional children take their parents' positions
			self.present_parent_ids(item, &prefix.prefix, present)
		} else {
			let one = Expression::new(self.db, &self.model, item, IntegerLiteral(1));
			let count = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.length.into(),
					arguments: vec![given.clone()],
				},
			);
			self.indexed_ids(item, &prefix.prefix, one, count)
		};
		Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.mzn_object_data_ids.into(),
				arguments: vec![given, generated],
			},
		)
	}

	/// `[<prefix> ++ "[" ++ show(p) ++ "]" | p in <first>..<last>]`
	fn indexed_ids(
		&mut self,
		item: Item<'db>,
		prefix: &str,
		first: Expression<'db>,
		last: Expression<'db>,
	) -> Expression<'db> {
		let positions = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.dot_dot.into(),
				arguments: vec![first, last],
			},
		);
		self.position_ids(item, prefix, positions, None)
	}

	/// The ids of the optional children of an occurrence, which take their
	/// parents' positions: `[<prefix> ++ "[" ++ show(p) ++ "]" | p in
	/// index_set(<present>) where <present>[p] > 0]`, where `<present>` is the
	/// number of children of each parent
	fn present_parent_ids(
		&mut self,
		item: Item<'db>,
		prefix: &str,
		present: DeclarationId<'db>,
	) -> Expression<'db> {
		let present = Expression::new(self.db, &self.model, item, present);
		let positions = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.index_set.into(),
				arguments: vec![present.clone()],
			},
		);
		self.position_ids(item, prefix, positions, Some(present))
	}

	/// `[<prefix> ++ "[" ++ show(p) ++ "]" | p in <positions>]`, only for the
	/// positions `p` with `<present>[p] > 0` if given
	fn position_ids(
		&mut self,
		item: Item<'db>,
		prefix: &str,
		positions: Expression<'db>,
		present: Option<Expression<'db>>,
	) -> Expression<'db> {
		let mut position = Declaration::new(
			false,
			Domain::unbounded(self.db, item, Ty::par_int(self.db)),
		);
		position.set_name(Identifier::new(self.db, "p"));
		let position_decl = self
			.model
			.add_declaration(DeclarationItem::new(position, item));
		let position = Expression::new(self.db, &self.model, item, position_decl);
		let where_clause = present.map(|present| {
			let count = Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.array_access.into(),
					arguments: vec![present, position.clone()],
				},
			);
			let zero = Expression::new(self.db, &self.model, item, IntegerLiteral(0));
			Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.gt.into(),
					arguments: vec![count, zero],
				},
			)
		});
		let shown_position = Expression::new(
			self.db,
			&self.model,
			item,
			LookupCall {
				function: self.ids.functions.show.into(),
				arguments: vec![position],
			},
		);
		let id = [
			self.string_literal(item, &format!("{prefix}[")),
			shown_position,
			self.string_literal(item, "]"),
		]
		.into_iter()
		.reduce(|acc, part| {
			Expression::new(
				self.db,
				&self.model,
				item,
				LookupCall {
					function: self.ids.functions.plus_plus.into(),
					arguments: vec![acc, part],
				},
			)
		})
		.expect("id has parts");
		Expression::new(
			self.db,
			&self.model,
			item,
			ArrayComprehension::new(
				[Generator::Iterator {
					declarations: vec![position_decl],
					collection: positions,
					where_clause,
				}],
				id,
			),
		)
	}

	fn string_literal(&self, item: Item<'db>, value: &str) -> Expression<'db> {
		Expression::new(
			self.db,
			&self.model,
			item,
			StringLiteral::new(self.db, value),
		)
	}
}
