//! Dead code elimination
use shackle_diagnostics::Result;
use shackle_hir::{Db, Identifier, constants::IdentifierRegistry};
use shackle_utils::hash::{Map, Set};

use crate::{
	Callable, FunctionId, ItemId, Marker, Model, ResolvedIdentifier,
	traverse::{Folder, ReplacementMap, Visitor, visit_item},
};

struct ReachabilityVisitor<'db, T: Marker> {
	db: &'db dyn Db,
	reachable: Set<ItemId<'db, T>>,
	keep_overloads: bool,
	overloads: Map<Identifier<'db>, Vec<FunctionId<'db, T>>>,
}

impl<'db, T: Marker> ReachabilityVisitor<'db, T> {
	fn run(
		db: &'db dyn Db,
		model: Model<'db, T>,
		keep_overloads: bool,
		keep_known_ids: bool,
	) -> Set<ItemId<'db, T>> {
		let mut overloads: Map<_, Vec<_>> = Map::default();
		if keep_overloads {
			for (idx, f) in model.top_level_functions() {
				overloads
					.entry(f.name().as_identifier(db))
					.or_default()
					.push(idx);
			}
		}

		let mut visitor = Self {
			db,
			reachable: Set::default(),
			keep_overloads,
			overloads,
		};

		if keep_known_ids {
			let ids = IdentifierRegistry::lookup(db);
			for name in ids.annotations.all.iter() {
				if let Some(ident) = model.lookup_identifier(db, *name) {
					visitor.visit_identifier(&model, &ident);
				}
			}
			for name in ids.functions.all.iter().chain(ids.functions.all.iter()) {
				if let Some(fs) = visitor.overloads.get(name) {
					for f in fs.clone() {
						visitor.visit_item(&model, f.into());
					}
				}
			}
		}

		visitor.visit_model(&model);
		visitor.reachable
	}
}

impl<'a, 'db, T: Marker> Visitor<'a, 'db, T> for ReachabilityVisitor<'db, T> {
	fn visit_model(&mut self, model: &'a Model<'db, T>) {
		for item in model.top_level_items() {
			if matches!(
				item,
				ItemId::Constraint(_)
					| ItemId::Declaration(_)
					| ItemId::Enumeration(_)
					| ItemId::Output(_)
					| ItemId::Solve
			) {
				self.visit_item(model, item);
			}
		}
	}

	fn visit_item(&mut self, model: &Model<'db, T>, item: ItemId<'db, T>) {
		let inserted = self.reachable.insert(item);
		if inserted {
			visit_item(self, model, item);
		}
	}

	fn visit_identifier(
		&mut self,
		model: &'a Model<'db, T>,
		identifier: &'a ResolvedIdentifier<'db, T>,
	) {
		match identifier {
			ResolvedIdentifier::Annotation(a) => self.visit_item(model, (*a).into()),
			ResolvedIdentifier::Declaration(d) => self.visit_item(model, (*d).into()),
			ResolvedIdentifier::Enumeration(e) => self.visit_item(model, (*e).into()),
			ResolvedIdentifier::EnumerationMember(e) => {
				self.visit_item(model, e.enumeration_id().into())
			}
		}
	}

	fn visit_callable(&mut self, model: &'a Model<'db, T>, callable: &'a Callable<'db, T>) {
		match callable {
			Callable::Function(f) => {
				if self.keep_overloads {
					let overloads =
						self.overloads[&model[*f].name().as_identifier(self.db)].clone();
					for f in overloads {
						self.visit_item(model, f.into());
					}
				} else {
					self.visit_item(model, (*f).into());
				}
			}
			Callable::Annotation(a) | Callable::AnnotationDestructure(a) => {
				self.visit_item(model, (*a).into());
			}
			Callable::EnumConstructor(e) | Callable::EnumDestructor(e) => {
				self.visit_item(model, e.enumeration_id().into())
			}
			Callable::Expression(expression) => {
				self.visit_expression(model, expression);
			}
		}
	}
}

struct ReachabilityFolder<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	reachable: Set<ItemId<'db, Src>>,
	removed: usize,
}

impl<'a, 'db, Dst: Marker, Src: Marker> Folder<'a, 'db, Dst, Src>
	for ReachabilityFolder<'db, Dst, Src>
{
	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn add_model(&mut self, db: &'db dyn Db, model: &'a Model<'db, Src>) {
		// Add items to the destination model
		for item in model.top_level_items() {
			if self.reachable.contains(&item) {
				self.add_item(db, model, item);
			} else {
				self.removed += 1;
			}
		}
		// Now that all items have been added, we can process function bodies
		for (f, i) in model.all_functions() {
			if self.reachable.contains(&(f.into())) && i.body().is_some() {
				self.fold_function_body(db, model, f);
			}
		}
	}
}

/// Eliminates dead code from the model
///
/// Conservative version which keeps all overloads and known identifiers in case they're needed later on
pub fn eliminate_dead_code_conservative<'db, T: Marker>(
	db: &'db dyn Db,
	model: Model<'db>,
) -> Result<Model<'db, T>> {
	log::info!("Eliminating dead code (1st pass)");
	let reachable = ReachabilityVisitor::run(db, model.clone(), true, true);
	let mut folder = ReachabilityFolder {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		reachable,
		removed: 0,
	};
	folder.add_model(db, &model);
	log::info!("Removed {} unreachable items", folder.removed);
	Ok(folder.model)
}

/// Eliminates dead code from the model
pub fn eliminate_dead_code<'db, T: Marker>(
	db: &'db dyn Db,
	model: Model<'db>,
) -> Result<Model<'db, T>> {
	log::info!("Eliminating dead code (2nd pass)");
	let reachable = ReachabilityVisitor::run(db, model.clone(), false, false);
	let mut folder = ReachabilityFolder {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
		reachable,
		removed: 0,
	};
	folder.add_model(db, &model);
	log::info!("Removed {} unreachable items", folder.removed);
	Ok(folder.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{dead_code::eliminate_dead_code, tests::check_no_stdlib};

	#[test]
	fn test_dce() {
		check_no_stdlib(
			eliminate_dead_code,
			r#"
            test unused(int: x) = true;
            test used(int: x) = true;
            int: a = 3;
            constraint used(a);
        "#,
			expect![[r#"
    function bool: used(int: x) = true;
    int: a = 3;
    constraint used(a);
    solve satisfy;
"#]],
		);
	}
}
