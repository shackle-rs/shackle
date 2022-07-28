//! Final validation step for HIR representation.
//!
//! This module contains validation steps which require the whole program HIR,
//! and can't be done on a per-item basis.
//!
//! - Check for illegal overloading/duplicate definitions
//! - Check for multiple definitions of variables
//! - Check for cyclic definitions of variables
//! - Check for multiple solve items

use std::sync::Arc;

use crate::{
	error::{AdditionalSolveItem, MultipleSolveItems},
	hir::ids::{ItemRef, NodeRef},
	Error,
};

use super::{db::Hir, DeclarationType};

/// Validate HIR
pub fn validate_hir(db: &dyn Hir) -> Arc<Vec<Error>> {
	let mut diagnostics = Vec::new();
	// Validate overloading
	let global_scope = db.lookup_global_scope();
	for (_, ps) in global_scope.functions(0) {
		let mut overloads = ps
			.iter()
			.map(|p| {
				let signature = db.lookup_item_signature(p.item());
				match &signature.patterns[p] {
					DeclarationType::Function(f) => *f.clone(),
					_ => unreachable!(),
				}
			})
			.collect::<Vec<_>>();
	}
	// Check for multiple assignments to variables

	// Check for cyclic definitions of variables

	// Check for multiple solve items
	let mut solve_items = Vec::new();
	for m in db.resolve_includes().unwrap().iter() {
		let model = db.lookup_model(*m);
		for (i, s) in model.solves.iter() {
			let item_ref = ItemRef::new(db, *m, i);
			solve_items.push(item_ref);
		}
	}
	if solve_items.len() > 1 {
		let mut iter = solve_items.into_iter();
		let first = iter.next().unwrap();
		let (src, span) = NodeRef::from(first).source_span(db);
		diagnostics.push(
			MultipleSolveItems {
				src,
				span,
				others: iter
					.map(|i| {
						let (src, span) = NodeRef::from(i).source_span(db);
						AdditionalSolveItem { src, span }
					})
					.collect(),
			}
			.into(),
		);
	}
	Arc::new(diagnostics)
}
