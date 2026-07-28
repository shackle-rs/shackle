//! Final validation step for HIR representation.
//!
//! This module contains miscellaneous validation steps which require the whole
//! program HIR, and can't be done on a per-item basis.
//!
//! - Check for illegal overloading/duplicate definitions
//! - Check for multiple definitions of variables
//! - Check for multiple solve items

use std::collections::hash_map::Entry;

use shackle_diagnostics::{
	AdditionalSolveItem, DuplicateAssignment, MultipleAssignments, MultipleSolveItems,
};
use shackle_utils::hash::Map;

use crate::{
	Db, GlobalScope, Item,
	diagnostics::Errors,
	ids::{ExpressionRef, NodeRef},
	lower::lower_models,
	overloading::validate_overloading,
};

/// Validate HIR
pub fn validate_hir<'db>(db: &'db dyn Db) {
	log::info!("Validating HIR");
	for (name, _) in GlobalScope::functions(db) {
		validate_overloading(db, name);
	}

	// Check for multiple assignments to variables
	let mut assignments: Map<_, Vec<NodeRef<'db>>> = Map::default();
	let mut multiple_asgs = Vec::new();
	for model in lower_models(db).iter() {
		for it in model.items(db).iter() {
			match it {
				Item::Assignment(item) => {
					let a = item.assignment(db);
					let types = it.types(db);
					if let Some(p) = types.name_resolution(a.assignee) {
						match assignments.entry(p) {
							Entry::Occupied(mut e) => {
								let asgs = e.get_mut();
								if asgs.len() == 1 {
									multiple_asgs.push(p);
								}
								asgs.push((*it).into());
							}
							Entry::Vacant(e) => {
								let mut v = Vec::new();
								let resolved_item = p.item(db);
								if let Item::Declaration(d) = resolved_item
									&& let Some(def) = d.declaration(db).definition
								{
									v.push(
										ExpressionRef::new(db, resolved_item, def)
											.into_entity(db)
											.into(),
									);
									multiple_asgs.push(p);
								}
								v.push((*it).into());
								let _ = e.insert(v);
							}
						}
					}
				}
				Item::EnumAssignment(item) => {
					let a = item.enum_assignment(db);
					let types = it.types(db);
					if let Some(p) = types.name_resolution(a.assignee) {
						match assignments.entry(p) {
							Entry::Occupied(mut e) => {
								let asgs = e.get_mut();
								if asgs.len() == 1 {
									multiple_asgs.push(p);
								}
								asgs.push((*it).into());
							}
							Entry::Vacant(e) => {
								let mut v = Vec::new();
								let resolved_item = p.item(db);
								if let Item::Enumeration(e) = resolved_item
									&& e.enumeration(db).definition.is_some()
								{
									v.push(p.into_entity(db).into());
									multiple_asgs.push(p);
								}
								v.push((*it).into());
								let _ = e.insert(v);
							}
						}
					}
				}
				_ => (),
			}
		}
	}
	for p in multiple_asgs.into_iter() {
		let asgs = &assignments[&p];
		let variable = p.identifier(db).unwrap().pretty_print(db);
		let mut asgs = asgs.iter();
		let (src, span) = asgs.next().unwrap().source_span(db);
		let others = asgs
			.map(|i| {
				let (src, span) = i.source_span(db);
				DuplicateAssignment { src, span }
			})
			.collect();
		Errors::add(
			db,
			MultipleAssignments {
				src,
				span,
				variable,
				others,
			},
		)
	}

	// Check for multiple solve items
	let mut solve_items = Vec::new();
	for m in lower_models(db).iter() {
		for it in m.items(db).iter() {
			if let Item::Solve(_) = it {
				solve_items.push(*it);
			}
		}
	}
	if solve_items.len() > 1 {
		let mut iter = solve_items.into_iter();
		let first = iter.next().unwrap();
		let (src, span) = NodeRef::from(first).source_span(db);
		Errors::add(
			db,
			MultipleSolveItems {
				src,
				span,
				others: iter
					.map(|i| {
						let (src, span) = NodeRef::from(i).source_span(db);
						AdditionalSolveItem { src, span }
					})
					.collect(),
			},
		);
	}
}
