//! Normalisation of constraints into parts: `forall(loops)(body)` where the
//! body is not a conjunction, and every loop declares one variable.

use shackle_hir::{Db, constants::IdentifierRegistry};
use shackle_thir::{
	Callable, Declaration, DeclarationId, Expression, ExpressionData, FunctionId, Generator, Model,
	ResolvedIdentifier,
	source::Origin,
	traverse::{Visitor, visit_callable, visit_expression},
};

/// Collects the functions that are called
#[derive(Default)]
pub(super) struct CallCollector<'db> {
	pub(super) functions: Vec<FunctionId<'db>>,
}

/// Collects the declarations an expression refers to
#[derive(Default)]
pub(super) struct DeclCollector<'db> {
	/// Also collect local declarations (e.g. loop variables)
	pub(super) local: bool,
	pub(super) decls: Vec<DeclarationId<'db>>,
}

/// One loop variable of a part. A `where` clause belongs to the last
/// variable of its generator.
#[derive(Clone, Copy)]
pub(super) struct Loop<'a, 'db> {
	pub(super) decl: DeclarationId<'db>,
	pub(super) kind: LoopKind<'a, 'db>,
	pub(super) where_clause: Option<&'a Expression<'db>>,
}

/// What a loop variable ranges over
#[derive(Clone, Copy)]
pub(super) enum LoopKind<'a, 'db> {
	/// `i in collection`
	Iter(&'a Expression<'db>),
	/// `i = definition`
	Assign(&'a Expression<'db>),
}

/// A constraint after normalisation: `forall(loops)(body)`
pub(super) struct Part<'a, 'db> {
	pub(super) origin: Origin<'db>,
	pub(super) loops: Vec<Loop<'a, 'db>>,
	pub(super) body: &'a Expression<'db>,
}

/// The declarations a declaration item refers to (domain and definition)
pub(super) fn decl_refs<'db>(model: &Model<'db>, d: DeclarationId<'db>) -> Vec<DeclarationId<'db>> {
	let mut collector = DeclCollector {
		local: false,
		decls: Vec::new(),
	};
	collector.visit_declaration(model, d);
	collector.decls
}

/// Whether a declaration is a top-level par integer (allowed as an index)
pub(super) fn is_par_int(db: &dyn Db, decl: &Declaration<'_>) -> bool {
	let ty = decl.ty();
	decl.top_level() && ty.known_par(db) && (ty.is_int(db) || ty.is_enum(db))
}

fn loops<'a, 'db>(model: &'a Model<'db>, g: &'a Generator<'db>) -> Vec<Loop<'a, 'db>> {
	match g {
		Generator::Iterator {
			declarations,
			collection,
			where_clause,
		} => declarations
			.iter()
			.enumerate()
			.map(|(i, d)| Loop {
				decl: *d,
				kind: LoopKind::Iter(collection),
				where_clause: where_clause
					.as_ref()
					.filter(|_| i + 1 == declarations.len()),
			})
			.collect(),
		Generator::Assignment {
			assignment,
			where_clause,
		} => vec![Loop {
			decl: *assignment,
			kind: LoopKind::Assign(
				model[*assignment]
					.definition()
					.expect("generator assignment has a definition"),
			),
			where_clause: where_clause.as_ref(),
		}],
	}
}

/// Split the top-level constraints of `model` into parts
pub(super) fn normalise<'a, 'db>(
	db: &'db dyn Db,
	model: &'a Model<'db>,
	is_user_item: impl Fn(Origin<'db>) -> bool,
) -> Vec<Part<'a, 'db>> {
	let ids = IdentifierRegistry::lookup(db);
	let mut parts = Vec::new();
	let mut todo = Vec::new();
	for (_, c) in model.top_level_constraints() {
		if is_user_item(c.origin()) {
			todo.push((c.origin(), Vec::new(), c.expression()));
		}
	}
	// A part of an inlined function keeps the location of the call
	let inside = |child: Origin<'db>, parent: Origin<'db>| {
		let ((cf, cs), (pf, ps)) = (child.source_span(db), parent.source_span(db));
		cf == pf && ps.offset() <= cs.offset() && cs.offset() + cs.len() <= ps.offset() + ps.len()
	};
	let child_origin = |child: &Expression<'db>, parent: Origin<'db>| {
		if inside(child.origin(), parent) {
			child.origin()
		} else {
			parent
		}
	};
	// Depth-first, so parts keep their source order
	todo.reverse();
	while let Some((origin, outer, e)) = todo.pop() {
		let mut children = Vec::new();
		match &**e {
			ExpressionData::BooleanLiteral(b) if b.0 => continue,
			ExpressionData::Call(c) => {
				if let Callable::Function(f) = &c.function {
					let name = model[*f].name();
					if name == ids.functions.and && c.arguments.len() == 2 {
						for a in &c.arguments {
							children.push((child_origin(a, origin), outer.clone(), a));
						}
					} else if name == ids.functions.forall && c.arguments.len() == 1 {
						match &*c.arguments[0] {
							ExpressionData::ArrayComprehension(ac) if ac.indices.is_none() => {
								let mut inner = outer.clone();
								for g in &ac.generators {
									inner.extend(loops(model, g));
								}
								children.push((origin, inner, &*ac.template));
							}
							ExpressionData::ArrayLiteral(al) => {
								for a in al.iter() {
									children.push((child_origin(a, origin), outer.clone(), a));
								}
							}
							_ => {}
						}
					}
				}
			}
			_ => {}
		}
		if children.is_empty() {
			parts.push(Part {
				origin,
				loops: outer,
				body: e,
			});
		} else {
			todo.extend(children.into_iter().rev());
		}
	}
	parts
}

/// The declarations the expressions refer to
pub(super) fn refs<'a, 'db>(
	model: &'a Model<'db>,
	local: bool,
	exprs: impl IntoIterator<Item = &'a Expression<'db>>,
) -> Vec<DeclarationId<'db>> {
	let mut collector = DeclCollector {
		local,
		decls: Vec::new(),
	};
	for e in exprs {
		visit_expression(&mut collector, model, e);
	}
	collector.decls
}

impl<'a, 'db> Visitor<'a, 'db> for CallCollector<'db> {
	fn visit_callable(&mut self, model: &'a Model<'db>, callable: &'a Callable<'db>) {
		if let Callable::Function(f) = callable
			&& !self.functions.contains(f)
		{
			self.functions.push(*f);
		}
		visit_callable(self, model, callable);
	}
}

impl<'a, 'db> Visitor<'a, 'db> for DeclCollector<'db> {
	fn visit_identifier(&mut self, model: &'a Model<'db>, identifier: &'a ResolvedIdentifier<'db>) {
		if let ResolvedIdentifier::Declaration(d) = identifier
			&& (self.local || model[*d].top_level())
			&& !self.decls.contains(d)
		{
			self.decls.push(*d);
		}
	}
}

impl<'a, 'db> Loop<'a, 'db> {
	/// The expressions of the loop header
	pub(super) fn exprs(&self) -> impl Iterator<Item = &'a Expression<'db>> {
		let main = match self.kind {
			LoopKind::Iter(e) | LoopKind::Assign(e) => e,
		};
		std::iter::once(main).chain(self.where_clause)
	}
}

impl<'a, 'db> Part<'a, 'db> {
	/// Whether the loops at `fixed` can be leaders: they iterate over sets, and
	/// their headers only use top-level declarations and other leaders.
	pub(super) fn can_fix(&self, db: &'db dyn Db, model: &'a Model<'db>, fixed: &[usize]) -> bool {
		let fixed_decls = fixed
			.iter()
			.map(|i| self.loops[*i].decl)
			.collect::<Vec<_>>();
		let loop_decls = self.loops.iter().map(|l| l.decl).collect::<Vec<_>>();
		fixed.iter().all(|i| {
			let l = &self.loops[*i];
			matches!(l.kind, LoopKind::Iter(c) if c.ty().is_set(db))
				&& refs(model, true, l.exprs())
					.iter()
					.all(|d| !loop_decls.contains(d) || fixed_decls.contains(d))
		})
	}

	/// The top-level declarations the part refers to
	pub(super) fn decls(&self, model: &'a Model<'db>) -> Vec<DeclarationId<'db>> {
		refs(
			model,
			false,
			self.loops.iter().flat_map(|l| l.exprs()).chain([self.body]),
		)
	}
}
