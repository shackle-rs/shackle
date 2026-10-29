//! The structure of a root's inputs: the part of the data that decides which
//! objects exist.
//!
//! The number of objects a root creates, and with it the class's universe and
//! actual set, only depends on its owned children. Its reference fields may
//! refer to any object, including the ones the same data creates
//! (`(leaves: [(v: 1), (v: 2)], favourite: min(Leaf))`), so sizing the objects
//! from the whole `<C>_<x>_inputs` makes the model cyclic. The sizes are read
//! from `<C>_<x>_structure` instead, a copy of the inputs reduced to the owned
//! children: each record keeps the fields that are a `set of new` or an `opt
//! new` (an array of 0 or 1 records in the inputs), or that lead to one, and a
//! record without such fields becomes `true`. It is only emitted when a size is
//! read from the data, as an array literal's length is known.
//!
//! The copy is made syntactically, through array and record literals,
//! comprehensions, if-then-else and let. Any other expression is projected as
//! a whole, so the structure depends on everything it refers to.

use rustc_hash::FxHashSet;
use shackle_hir::{Item, ids::PatternRef};
use shackle_ty::{Ty, TyData};

use crate::{lower::ItemCollector, *};

/// The structure of one inputs declaration
pub(in crate::lower) enum InputsStructure<'db> {
	/// Not needed yet: its type and value
	Pending(Ty<'db>, Expression<'db>),
	/// `<C>_<x>_structure`
	Declared(DeclarationId<'db>),
}

impl<'db> ItemCollector<'db> {
	/// Record the structure of the inputs declaration `inputs` of a root of
	/// `class`, given `value`, a separately collected copy of its definition
	pub(in crate::lower) fn register_inputs_structure(
		&mut self,
		item: Item<'db>,
		inputs: DeclarationId<'db>,
		class: PatternRef<'db>,
		value: Expression<'db>,
	) {
		// Typed like the inputs, as the value may hold an empty array
		let ty = self.structure_ty(class, self.model[inputs].ty());
		let structure = self.project_structure(item, class, value);
		let _ = self
			.objects
			.inputs_structures
			.insert(inputs, InputsStructure::Pending(ty, structure));
	}

	/// The structure of `inputs`, to size objects by, if it is an inputs
	/// declaration written in the model
	pub(in crate::lower) fn inputs_structure(
		&mut self,
		item: Item<'db>,
		inputs: &Expression<'db>,
	) -> Option<Expression<'db>> {
		let ExpressionData::Identifier(ResolvedIdentifier::Declaration(inputs)) = &**inputs else {
			return None;
		};
		let declaration = match self.objects.inputs_structures.remove(inputs)? {
			InputsStructure::Declared(declaration) => declaration,
			InputsStructure::Pending(ty, value) => {
				let name = self.model[*inputs]
					.name()
					.expect("inputs declarations are named")
					.lookup(self.db);
				let name = name.strip_suffix("_inputs").unwrap_or(name);
				let mut declaration = Declaration::new(true, Domain::unbounded(self.db, item, ty));
				declaration.set_name(Identifier::new(self.db, format!("{name}_structure")));
				declaration.set_definition(value);
				self.model
					.add_declaration(DeclarationItem::new(declaration, item))
			}
		};
		let _ = self
			.objects
			.inputs_structures
			.insert(*inputs, InputsStructure::Declared(declaration));
		Some(Expression::new(self.db, &self.model, item, declaration))
	}

	/// The owned children of `class` that its structure keeps: the `set of
	/// new` fields, and the other owned children whose class has a structure
	fn structure_fields(&self, class: PatternRef<'db>) -> Vec<(Identifier<'db>, PatternRef<'db>)> {
		self.structure_fields_visiting(class, &mut FxHashSet::default())
	}

	fn structure_fields_visiting(
		&self,
		class: PatternRef<'db>,
		visiting: &mut FxHashSet<PatternRef<'db>>,
	) -> Vec<(Identifier<'db>, PatternRef<'db>)> {
		if !visiting.insert(class) {
			return Vec::new();
		}
		let collections = self
			.class_storage_field_decls(class.item(self.db))
			.into_iter()
			.filter(|field| {
				matches!(
					field.owner.data(self.db)[field.declared_type],
					shackle_hir::Type::Set { .. }
						| shackle_hir::Type::New {
							opt: OptType::Opt,
							..
						}
				)
			})
			.map(|field| field.ident)
			.collect::<Vec<_>>();
		let fields = self
			.owned_child_fields(class)
			.into_iter()
			.filter(|(field, child)| {
				collections.contains(field)
					|| !self.structure_fields_visiting(*child, visiting).is_empty()
			})
			.collect();
		let _ = visiting.remove(&class);
		fields
	}

	/// The type of the structure of a value of type `ty`, a `class` input
	/// record or an array of them
	fn structure_ty(&self, class: PatternRef<'db>, ty: Ty<'db>) -> Ty<'db> {
		match ty.lookup(self.db) {
			TyData::Array { dim, element, .. } => {
				Ty::array(self.db, *dim, self.structure_ty(class, *element))
					.expect("an array of structures")
			}
			TyData::Record(_, fields) => {
				let fields = self
					.structure_fields(class)
					.into_iter()
					.filter_map(|(field, child)| {
						let (_, ty) = fields.iter().find(|(name, _)| *name == field.0)?;
						Some((field.0, self.structure_ty(child, *ty)))
					})
					.collect::<Vec<_>>();
				if fields.is_empty() {
					Ty::par_bool(self.db)
				} else {
					Ty::record(self.db, fields)
				}
			}
			// The element of an empty array, `[]`
			_ => ty,
		}
	}

	/// The structure of `value`, a `class` input record or an array of them
	fn project_structure(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		value: Expression<'db>,
	) -> Expression<'db> {
		if matches!(value.ty().lookup(self.db), TyData::Array { .. }) {
			self.project_structure_array(item, class, value)
		} else {
			self.project_structure_record(item, class, value)
		}
	}

	fn project_structure_array(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		value: Expression<'db>,
	) -> Expression<'db> {
		let origin = value.origin();
		match &*value {
			// Typed explicitly, as the inputs' empty array is
			ExpressionData::ArrayLiteral(members) if members.is_empty() => {
				Expression::new_unchecked(
					self.structure_ty(class, value.ty()),
					ArrayLiteral(Vec::new()),
					origin,
				)
			}
			ExpressionData::ArrayLiteral(members) => {
				let members = members
					.iter()
					.map(|member| self.project_structure_record(item, class, member.clone()))
					.collect();
				Expression::new(self.db, &self.model, origin, ArrayLiteral(members))
			}
			ExpressionData::ArrayComprehension(c) => {
				let template = self.project_structure_record(item, class, (*c.template).clone());
				Expression::new(
					self.db,
					&self.model,
					origin,
					ArrayComprehension {
						template: Box::new(template),
						indices: c.indices.clone(),
						generators: c.generators.clone(),
					},
				)
			}
			ExpressionData::IfThenElse(_) | ExpressionData::Let(_) => {
				self.project_structure_branches(item, class, value, Self::project_structure_array)
			}
			_ => {
				let element_ty = value.ty().elem_ty(self.db).expect("an array of records");
				let mut element =
					Declaration::new(false, Domain::unbounded(self.db, item, element_ty));
				element.set_name(Identifier::new(self.db, "o"));
				let element = self
					.model
					.add_declaration(DeclarationItem::new(element, item));
				let template = Expression::new(self.db, &self.model, origin, element);
				let template = self.project_structure_record(item, class, template);
				Expression::new(
					self.db,
					&self.model,
					origin,
					ArrayComprehension::new(
						[Generator::Iterator {
							declarations: vec![element],
							collection: value,
							where_clause: None,
						}],
						template,
					),
				)
			}
		}
	}

	fn project_structure_record(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		value: Expression<'db>,
	) -> Expression<'db> {
		let origin = value.origin();
		let fields = self
			.structure_fields(class)
			.into_iter()
			.filter(|(field, _)| self.record_expr_has_field(&value, *field))
			.collect::<Vec<_>>();
		if fields.is_empty() {
			return Expression::new(self.db, &self.model, origin, BooleanLiteral(true));
		}
		match &*value {
			ExpressionData::RecordLiteral(members) => {
				let members = members.as_hash_map();
				let members = fields
					.iter()
					.map(|(field, child)| {
						let member = members[field].clone();
						(*field, self.project_structure(item, *child, member))
					})
					.collect();
				Expression::new(self.db, &self.model, origin, RecordLiteral(members))
			}
			ExpressionData::IfThenElse(_) | ExpressionData::Let(_) => {
				self.project_structure_branches(item, class, value, Self::project_structure_record)
			}
			ExpressionData::Identifier(_) => {
				let members = fields
					.iter()
					.map(|(field, child)| {
						let access = Expression::new(
							self.db,
							&self.model,
							origin,
							RecordAccess {
								record: Box::new(value.clone()),
								field: *field,
							},
						);
						(*field, self.project_structure(item, *child, access))
					})
					.collect();
				Expression::new(self.db, &self.model, origin, RecordLiteral(members))
			}
			_ => {
				// Bound once, as each field reads it
				let mut record =
					Declaration::new(false, Domain::unbounded(self.db, item, value.ty()));
				record.set_name(Identifier::new(self.db, "r"));
				record.set_definition(value);
				let record = self
					.model
					.add_declaration(DeclarationItem::new(record, item));
				let identifier = Expression::new(self.db, &self.model, origin, record);
				let in_expression = self.project_structure_record(item, class, identifier);
				Expression::new(
					self.db,
					&self.model,
					origin,
					Let {
						items: vec![LetItem::Declaration(record)],
						in_expression: Box::new(in_expression),
					},
				)
			}
		}
	}

	/// Project the results of the if-then-else or let `value` with `project`
	fn project_structure_branches(
		&mut self,
		item: Item<'db>,
		class: PatternRef<'db>,
		value: Expression<'db>,
		project: fn(&mut Self, Item<'db>, PatternRef<'db>, Expression<'db>) -> Expression<'db>,
	) -> Expression<'db> {
		let origin = value.origin();
		match &*value {
			ExpressionData::IfThenElse(ite) => {
				let branches = ite
					.branches
					.iter()
					.map(|branch| Branch {
						condition: branch.condition.clone(),
						result: project(self, item, class, branch.result.clone()),
					})
					.collect();
				let else_result = project(self, item, class, (*ite.else_result).clone());
				Expression::new(
					self.db,
					&self.model,
					origin,
					IfThenElse {
						branches,
						else_result: Box::new(else_result),
					},
				)
			}
			ExpressionData::Let(l) => {
				let in_expression = project(self, item, class, (*l.in_expression).clone());
				Expression::new(
					self.db,
					&self.model,
					origin,
					Let {
						items: l.items.clone(),
						in_expression: Box::new(in_expression),
					},
				)
			}
			_ => unreachable!("an if-then-else or let"),
		}
	}
}
