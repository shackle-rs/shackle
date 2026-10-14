//! Globalizer: suggests global constraints that can replace groups of
//! constraints in a model.
//!
//! This is a port of the Haskell [globalizer](https://github.com/MiniZinc/Globalizer)
//! (Leo, Mears, Tack, Garcia de la Banda, "Globalizing Constraint Models",
//! CP 2013). It works on the THIR directly after type checking:
//!
//! 1. Split the constraints into *parts* (conjunctions, nested `forall`s).
//! 2. Enumerate *groups* of up to `max_group` parts that share variables.
//! 3. Sample solutions of each group with `minizinc`, and evaluate every
//!    candidate global on them in the same run (forward check).
//! 4. Sample solutions of each surviving candidate and check that they satisfy
//!    the group (reverse check).
//!
//! Results are based on sampling for each data file, so they are suggestions,
//! not proofs.

mod candidates;
mod run;

use std::{
	collections::BTreeSet,
	fmt::{self, Display, Write},
	path::{Path, PathBuf},
	sync::atomic::{AtomicUsize, Ordering},
	time::Duration,
};

use rustc_hash::FxHashSet;
use serde_json::Value;
use shackle_hir::{Db, constants::IdentifierRegistry, input::InputFiles};
use shackle_thir::{
	Callable, DeclarationId, Expression, ExpressionData, Generator, ItemId, Model,
	ResolvedIdentifier,
	compat::OldMiniZincPrinter,
	pretty_print::{Printer, print_item},
	source::Origin,
	traverse::{Visitor, visit_expression},
};
use shackle_ty::VarType;

use self::{
	candidates::{Arg, ArgType, Candidate, candidates},
	run::{Outcome, Runner, Solution},
};
use crate::Result;

/// Options for [`crate::Model::globalize`]
#[derive(Clone, Debug)]
pub struct GlobalizeOptions {
	/// MiniZinc solver used to sample solutions
	pub solver: String,
	/// Maximum number of constraint parts in a group
	pub max_group: usize,
	/// Number of solutions sampled per check
	pub samples: usize,
	/// Time limit for each `minizinc` run
	pub time_limit: Duration,
	/// Random seed passed to the solver
	pub seed: u64,
	/// Only consider globals whose name contains one of these strings
	pub include: Vec<String>,
	/// Never consider globals whose name contains one of these strings
	pub exclude: Vec<String>,
}

impl Default for GlobalizeOptions {
	fn default() -> Self {
		Self {
			solver: "gecode".to_owned(),
			max_group: 2,
			samples: 30,
			time_limit: Duration::from_secs(1),
			seed: 0,
			include: Vec::new(),
			exclude: Vec::new(),
		}
	}
}

/// A suggested replacement for a group of constraints
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
	/// Source locations of the replaced constraints (`file|l1|c1|l2|c2`)
	pub locations: Vec<String>,
	/// The global constraint call that can replace them
	pub replacement: String,
}

impl Display for Finding {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} {}", self.locations.join(";"), self.replacement)
	}
}

/// Result of [`crate::Model::globalize`]
#[derive(Clone, Debug)]
pub struct Globalized {
	/// Suggestions, sorted by location
	pub findings: Vec<Finding>,
	/// Number of `minizinc` runs
	pub calls: usize,
}

/// A constraint after normalisation: `forall(generators)(body)`
struct Part<'a, 'db> {
	origin: Origin<'db>,
	generators: Vec<&'a Generator<'db>>,
	body: &'a Expression<'db>,
}

/// Split the top-level constraints of `model` into parts
fn normalise<'a, 'db>(
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
	// Depth-first, so parts keep their source order
	todo.reverse();
	while let Some((origin, generators, e)) = todo.pop() {
		let mut children = Vec::new();
		match &**e {
			ExpressionData::BooleanLiteral(b) if b.0 => continue,
			ExpressionData::Call(c) => {
				if let Callable::Function(f) = &c.function {
					let name = model[*f].name();
					if name == ids.functions.and && c.arguments.len() == 2 {
						for a in &c.arguments {
							children.push((a.origin(), generators.clone(), a));
						}
					} else if name == ids.functions.forall && c.arguments.len() == 1 {
						match &*c.arguments[0] {
							ExpressionData::ArrayComprehension(ac) if ac.indices.is_none() => {
								let mut gens = generators.clone();
								gens.extend(ac.generators.iter());
								children.push((origin, gens, &*ac.template));
							}
							ExpressionData::ArrayLiteral(al) => {
								for a in al.iter() {
									children.push((a.origin(), generators.clone(), a));
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
				generators,
				body: e,
			});
		} else {
			todo.extend(children.into_iter().rev());
		}
	}
	parts
}

/// Collects the top-level declarations an expression refers to
struct DeclCollector<'db> {
	decls: Vec<DeclarationId<'db>>,
}

impl<'a, 'db> Visitor<'a, 'db> for DeclCollector<'db> {
	fn visit_identifier(&mut self, model: &'a Model<'db>, identifier: &'a ResolvedIdentifier<'db>) {
		if let ResolvedIdentifier::Declaration(d) = identifier
			&& model[*d].top_level()
			&& !self.decls.contains(d)
		{
			self.decls.push(*d);
		}
	}
}

fn part_decls<'db>(model: &Model<'db>, part: &Part<'_, 'db>) -> Vec<DeclarationId<'db>> {
	let mut collector = DeclCollector { decls: Vec::new() };
	for g in &part.generators {
		match g {
			Generator::Iterator {
				collection,
				where_clause,
				..
			} => {
				visit_expression(&mut collector, model, collection);
				if let Some(w) = where_clause {
					visit_expression(&mut collector, model, w);
				}
			}
			Generator::Assignment {
				assignment,
				where_clause,
			} => {
				if let Some(d) = model[*assignment].definition() {
					visit_expression(&mut collector, model, d);
				}
				if let Some(w) = where_clause {
					visit_expression(&mut collector, model, w);
				}
			}
		}
	}
	visit_expression(&mut collector, model, part.body);
	collector.decls
}

/// Enumerate the groups of up to `max` parts (as sorted indices) whose
/// variables connect them.
fn groups<T: PartialEq>(vars: &[Vec<T>], max: usize) -> Vec<Vec<usize>> {
	use itertools::Itertools;
	let shares = |a: usize, b: usize| vars[a].iter().any(|v| vars[b].contains(v));
	let mut result = Vec::new();
	for size in 1..=max.min(vars.len()) {
		for group in (0..vars.len()).combinations(size) {
			if group.iter().any(|i| vars[*i].is_empty()) {
				continue;
			}
			// Flood fill from the first part
			let mut reached = vec![group[0]];
			let mut i = 0;
			while i < reached.len() {
				for &other in &group {
					if !reached.contains(&other) && shares(reached[i], other) {
						reached.push(other);
					}
				}
				i += 1;
			}
			if reached.len() == group.len() {
				result.push(group);
			}
		}
	}
	result
}

/// Format a source location as `file|l1|c1|l2|c2` (1-based, inclusive end)
fn location<'db>(db: &'db dyn Db, origin: Origin<'db>) -> String {
	let (src, span) = origin.source_span(db);
	let position = |offset: usize| {
		let before = &src.contents()[..offset];
		let line = before.matches('\n').count() + 1;
		let col = before.len() - before.rfind('\n').map_or(0, |i| i + 1) + 1;
		(line, col)
	};
	let (l1, c1) = position(span.offset());
	let (l2, c2) = position(span.offset() + span.len().saturating_sub(1));
	format!("{}|{l1}|{c1}|{l2}|{c2}", src.name())
}

struct Globalizer<'a, 'db> {
	db: &'db dyn Db,
	model: &'a Model<'db>,
	printer: OldMiniZincPrinter<'db>,
	parts: Vec<Part<'a, 'db>>,
	/// Printed declarations, functions etc. for the submodels, with the
	/// declaration if it is a variable (only printed when a group needs it)
	items: Vec<(Option<DeclarationId<'db>>, String)>,
	data: Vec<Option<&'a Path>>,
	options: &'a GlobalizeOptions,
	runner: Runner,
}

impl<'a, 'db> Globalizer<'a, 'db> {
	fn print_part(&self, part: &Part<'a, 'db>) -> String {
		let body = self
			.printer
			.print_expression(self.db, self.model, part.body);
		if part.generators.is_empty() {
			return body;
		}
		let gens = part
			.generators
			.iter()
			.map(|g| self.printer.print_generator(self.db, self.model, g))
			.collect::<Vec<_>>()
			.join(", ");
		format!("forall({gens})({body})")
	}

	/// The model without constraints, solve and output items. Of the variables
	/// it only declares those `decls` (transitively) depend on, so the solver
	/// does not enumerate unrelated variables.
	fn preamble(&self, decls: &[DeclarationId<'db>]) -> String {
		let mut needed = decls.to_vec();
		let mut i = 0;
		while i < needed.len() {
			let mut collector = DeclCollector { decls: Vec::new() };
			collector.visit_declaration(self.model, needed[i]);
			for d in collector.decls {
				if !needed.contains(&d) {
					needed.push(d);
				}
			}
			i += 1;
		}
		let mut preamble = "include \"globals.mzn\";\n".to_owned();
		for (decl, text) in &self.items {
			if decl.is_none_or(|d| needed.contains(&d)) {
				writeln!(preamble, "{text};").unwrap();
			}
		}
		preamble
	}

	fn name(&self, d: DeclarationId<'db>) -> String {
		self.printer.print_declaration_id(self.db, self.model, d)
	}

	/// Candidate arguments from the declarations used by a group
	fn arguments(&self, decls: &[DeclarationId<'db>]) -> Vec<Arg> {
		let db = self.db;
		let mut args = Vec::new();
		let mut scalars = Vec::new();
		for &d in decls {
			let ty = self.model[d].ty();
			let elem = ty.elem_ty(db).filter(|_| ty.is_array(db)).unwrap_or(ty);
			if !elem.is_int(db) || ty.opt(db) != Some(shackle_ty::OptType::NonOpt) {
				continue;
			}
			let var = elem.inst(db) == Some(VarType::Var);
			let dims = ty.dims(db).unwrap_or(0);
			if dims == 0 && var {
				scalars.push(self.name(d));
			}
			args.push(Arg {
				text: self.name(d),
				var,
				ty: ArgType::Int(dims),
			});
		}
		// Scalar variables of the group as one array (e.g. `[a, b, c]`). Two
		// variables are left out: such a global is just a binary constraint.
		if scalars.len() > 2 {
			args.push(Arg {
				text: format!("[{}]", scalars.join(", ")),
				var: true,
				ty: ArgType::Int(1),
			});
		}
		args
	}

	/// Search annotation that picks random values for the group's variables
	fn search(&self, decls: &[DeclarationId<'db>], restart: bool) -> String {
		let db = self.db;
		let vars = decls
			.iter()
			.filter(|d| {
				let ty = self.model[**d].ty();
				let elem = ty.elem_ty(db).filter(|_| ty.is_array(db)).unwrap_or(ty);
				elem.is_int(db)
					&& elem.inst(db) == Some(VarType::Var)
					&& elem.opt(db) == Some(shackle_ty::OptType::NonOpt)
			})
			.map(|d| {
				if self.model[*d].ty().is_array(db) {
					format!("array1d({})", self.name(*d))
				} else {
					format!("[{}]", self.name(*d))
				}
			})
			.collect::<Vec<_>>();
		let mut solve = "solve".to_owned();
		if !vars.is_empty() {
			write!(
				solve,
				" :: int_search({}, input_order, indomain_random)",
				vars.join(" ++ ")
			)
			.unwrap();
		}
		if restart {
			solve.push_str(" :: restart_luby(1)");
		}
		solve.push_str(" satisfy;\n");
		solve
	}

	/// Sample solutions of `body`. Returns `None` if the sample is unusable:
	/// an error, no solutions, or fewer than requested without a complete
	/// search.
	fn sample(
		&self,
		body: &str,
		decls: &[DeclarationId<'db>],
		data: Option<&Path>,
	) -> Result<Option<Vec<Solution>>> {
		let n = self.options.samples;
		// Restarts give diverse samples, but can never prove completeness
		let first = self
			.runner
			.run(&format!("{body}{}", self.search(decls, true)), data, n)?;
		let usable = |o: Outcome| {
			if let Some(err) = &o.error {
				log::info!("globalizer: skipping submodel: {err}");
				return None;
			}
			(o.solutions.len() >= n || (o.complete && !o.solutions.is_empty()))
				.then_some(o.solutions)
		};
		if first.error.is_some() || first.solutions.len() >= n {
			return Ok(usable(first));
		}
		let second = self
			.runner
			.run(&format!("{body}{}", self.search(decls, false)), data, n)?;
		Ok(usable(second))
	}

	/// Candidates that hold on all samples of the group (forward check)
	fn forward(&self, group: &[usize], decls: &[DeclarationId<'db>]) -> Result<Vec<Candidate>> {
		let mut cands = candidates(
			&self.arguments(decls),
			&self.options.include,
			&self.options.exclude,
		);
		// A group that already is the global is vacuous
		if let [single] = group
			&& let ExpressionData::Call(c) = &**self.parts[*single].body
			&& let Callable::Function(f) = &c.function
		{
			let name = self.model[*f].name().as_identifier(self.db).lookup(self.db);
			cands.retain(|c| c.name != name);
		}
		for data in &self.data {
			if cands.is_empty() {
				break;
			}
			let mut body = self.preamble(decls);
			for p in group {
				writeln!(body, "constraint {};", self.print_part(&self.parts[*p])).unwrap();
			}
			writeln!(
				body,
				"array[1..{}] of var bool: GZ_R :: output = [{}];",
				cands.len(),
				cands
					.iter()
					.map(Candidate::checked)
					.collect::<Vec<_>>()
					.join(", ")
			)
			.unwrap();
			let Some(solutions) = self.sample(&body, decls, *data)? else {
				return Ok(Vec::new());
			};
			let mut k = 0;
			cands.retain(|_| {
				k += 1;
				solutions
					.iter()
					.all(|s| s["GZ_R"].get(k - 1) == Some(&Value::Bool(true)))
			});
		}
		Ok(cands)
	}

	/// Whether all samples of the candidate satisfy the group (reverse check)
	fn reverse(
		&self,
		group: &[usize],
		decls: &[DeclarationId<'db>],
		cand: &Candidate,
	) -> Result<bool> {
		for data in &self.data {
			let mut body = self.preamble(decls);
			writeln!(body, "constraint {};", cand.call()).unwrap();
			let conj = group
				.iter()
				.map(|p| format!("({})", self.print_part(&self.parts[*p])))
				.collect::<Vec<_>>()
				.join(" /\\ ");
			writeln!(body, "var bool: GZ_G :: output = {conj};").unwrap();
			let Some(solutions) = self.sample(&body, decls, *data)? else {
				return Ok(false);
			};
			if !solutions.iter().all(|s| s["GZ_G"] == Value::Bool(true)) {
				return Ok(false);
			}
		}
		Ok(true)
	}

	fn run(&self) -> Result<Vec<Finding>> {
		let decls = self
			.parts
			.iter()
			.map(|p| part_decls(self.model, p))
			.collect::<Vec<_>>();
		let vars = decls
			.iter()
			.map(|ds| {
				ds.iter()
					.copied()
					.filter(|d| !self.model[*d].ty().known_par(self.db))
					.collect::<Vec<_>>()
			})
			.collect::<Vec<_>>();
		let mut findings = BTreeSet::new();
		for group in groups(&vars, self.options.max_group) {
			let mut group_decls = Vec::new();
			for d in group.iter().flat_map(|p| &decls[*p]) {
				if !group_decls.contains(d) {
					group_decls.push(*d);
				}
			}
			let mut locations = group
				.iter()
				.map(|p| location(self.db, self.parts[*p].origin))
				.collect::<Vec<_>>();
			locations.dedup();
			for cand in self.forward(&group, &group_decls)? {
				if self.reverse(&group, &group_decls, &cand)? {
					findings.insert(Finding {
						locations: locations.clone(),
						replacement: cand.call(),
					});
				}
			}
		}
		Ok(findings.into_iter().collect())
	}
}

/// Run the globalizer on the (type checked) model in `db`
pub(crate) fn globalize(
	db: &dyn Db,
	data: &[PathBuf],
	options: &GlobalizeOptions,
) -> Result<Globalized> {
	with_globalizer(db, data, options, |globalizer| {
		let findings = globalizer.run()?;
		Ok(Globalized {
			findings,
			calls: globalizer.runner.calls.load(Ordering::Relaxed),
		})
	})
}

/// Set up a [`Globalizer`] for the model in `db` and pass it to `f`
fn with_globalizer<R>(
	db: &dyn Db,
	data: &[PathBuf],
	options: &GlobalizeOptions,
	f: impl FnOnce(&Globalizer<'_, '_>) -> R,
) -> R {
	let model = shackle_thir::lower::lower_model(db).take();
	let user_files = FxHashSet::from_iter(InputFiles::get(db).files(db));
	let is_user_item = |o: Origin<'_>| {
		o.node()
			.is_some_and(|n| user_files.contains(&n.model_file(db)))
	};

	// The THIR is printed before any transforms
	let printer = OldMiniZincPrinter::new(db, &[], true);
	let mut items = Vec::new();
	for item in model.top_level_items() {
		if matches!(
			item,
			ItemId::Constraint(_) | ItemId::Output(_) | ItemId::Solve
		) || !is_user_item(model.item_origin(item))
		{
			continue;
		}
		let var = match item {
			ItemId::Declaration(d) if !model[d].ty().known_par(db) => Some(d),
			_ => None,
		};
		items.push((var, print_item(&printer, db, &model, item)));
	}

	let globalizer = Globalizer {
		db,
		model: &model,
		printer,
		parts: normalise(db, &model, is_user_item),
		items,
		data: if data.is_empty() {
			vec![None]
		} else {
			data.iter().map(|d| Some(d.as_path())).collect()
		},
		options,
		runner: Runner {
			solver: options.solver.clone(),
			time_limit: options.time_limit,
			seed: options.seed,
			calls: AtomicUsize::new(0),
		},
	};
	f(&globalizer)
}

#[cfg(test)]
mod tests;
