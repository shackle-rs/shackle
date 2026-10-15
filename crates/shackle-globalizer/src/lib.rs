//! Globalizer: suggests global constraints that can replace groups of
//! constraints in a model, following Leo, Mears, Tack and Garcia de la Banda,
//! "Globalizing Constraint Models" (CP 2013). It works on the THIR directly
//! after type checking:
//!
//! 1. Split the constraints into *parts* (conjunctions, nested `forall`s), with
//!    the user's functions inlined.
//! 2. Enumerate *groups* of up to `max_group` parts that share variables, with
//!    an optional *context* (one more part that is not replaced), and ways to
//!    *unroll* their loops: a loop variable becomes a *leader* that is fixed to
//!    the first or the last value of its set.
//! 3. Sample solutions of each group with `minizinc`, and evaluate every
//!    candidate global on them in the same run (forward check). A pair of
//!    parts where one implies the other on the samples is left out.
//! 4. Sample solutions of each surviving candidate and check that they satisfy
//!    the group (reverse check).
//!
//! An initial pass first runs these steps with two kinds of candidates only:
//! `true` (the group is redundant, and is left out of the second pass) and
//! `binaries_represent_int` (a 0/1 array that represents an integer array,
//! which the second pass can use as a *viewpoint*).
//!
//! Results are based on sampling for each data file, so they are suggestions,
//! not proofs.

mod arguments;
mod candidates;
mod check;
mod groups;
mod parts;
mod run;
mod submodel;
mod viewpoints;

use std::{
	collections::BTreeSet,
	fmt::{self, Display},
	path::PathBuf,
	sync::atomic::Ordering,
	time::Duration,
};

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use shackle_diagnostics::{Error, Result};
use shackle_fmt::{MiniZincFormatOptions, format_str};
use shackle_hir::{
	Db,
	input::{CompilerSettings, ModelFile, shackle_share_directory},
	run_hir_phase,
};
use shackle_thir::{
	DeclarationId, Expression, ItemId, Model,
	compat::OldMiniZincPrinter,
	pretty_print::{Printer, print_item},
	source::Origin,
	transform::inlining::inline_user_functions,
	traverse::Visitor,
};

use self::{
	candidates::{Candidate, Kind},
	check::Checker,
	groups::groups,
	parts::{CallCollector, DeclCollector, Part, decl_refs, normalise},
	run::Runner,
	viewpoints::Viewpoint,
};

/// Options for [`globalize`]
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
	/// Skip a global if a group gives more argument lists for it
	pub max_args: usize,
	/// Abandon a pair of constraints if one implies the other on the samples
	pub implies_check: bool,
	/// Number of worker threads that run `minizinc`
	pub jobs: usize,
	/// First find redundant groups and 0/1 arrays that represent integers
	pub initial_pass: bool,
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
			max_args: 50_000,
			implies_check: true,
			jobs: std::thread::available_parallelism().map_or(1, |n| n.get()),
			initial_pass: true,
		}
	}
}

/// A suggested replacement for a group of constraints
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
	/// Source locations of the replaced constraints (`file|l1|c1|l2|c2`)
	pub locations: Vec<String>,
	/// Location of the constraint that is needed for the replacement to hold
	pub context: Option<String>,
	/// The global constraint call that can replace them
	pub replacement: String,
}

impl Display for Finding {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{} [ {}] {}",
			self.locations.join(";"),
			self.context
				.as_ref()
				.map(|c| format!("{c} "))
				.unwrap_or_default(),
			self.replacement
		)
	}
}

/// Result of [`globalize`]
#[derive(Clone, Debug)]
pub struct Globalized {
	/// New integer arrays (with their channelling constraint) that some
	/// findings use, found by the initial pass
	pub viewpoints: Vec<String>,
	/// Suggestions, sorted by location (redundant groups have `true`)
	pub findings: Vec<Finding>,
	/// Number of `minizinc` runs
	pub calls: usize,
}

/// Which candidates a pass checks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
	/// Only redundancy (`true`) and viewpoints (`binaries_represent_int`)
	Initial,
	/// All other candidates
	Full,
}

impl Pass {
	fn checks(self, kind: Kind) -> bool {
		(kind != Kind::Normal) == (self == Pass::Initial)
	}
}

/// A finding before formatting
#[derive(Clone, Debug)]
struct RawFinding {
	group: Vec<usize>,
	context: Option<usize>,
	replacement: String,
	candidate: Candidate,
}

/// Which findings to keep: drop a finding whose replacement is also found for
/// a smaller group (a subset of the parts, or the same parts without a
/// context), and a finding with a context whose replacement is found for a
/// group that also contains the context
fn shadow(found: &[RawFinding]) -> Vec<bool> {
	found
		.iter()
		.map(|f| {
			let (group, context, replacement) = (&f.group, &f.context, &f.replacement);
			!found.iter().any(|other| {
				let (g, c, r) = (&other.group, &other.context, &other.replacement);
				r == replacement
					&& ((g.iter().all(|p| group.contains(p))
						&& (c.is_none() || c == context)
						&& (g.len() < group.len() || (c.is_none() && context.is_some())))
						|| (c.is_none()
							&& context.is_some_and(|ctx| g.contains(&ctx))
							&& group.iter().all(|p| g.contains(p))))
			})
		})
		.collect()
}

/// `seed` extended with everything `next` gives for its elements, until
/// nothing is new (without duplicates, in the order first found)
fn closure<T: PartialEq, I: IntoIterator<Item = T>>(
	seed: impl IntoIterator<Item = T>,
	mut next: impl FnMut(&T) -> I,
) -> Vec<T> {
	let mut all = Vec::new();
	let add = |all: &mut Vec<T>, t: T| {
		if !all.contains(&t) {
			all.push(t);
		}
	};
	for t in seed {
		add(&mut all, t);
	}
	let mut i = 0;
	while i < all.len() {
		for t in next(&all[i]) {
			add(&mut all, t);
		}
		i += 1;
	}
	all
}

/// Format a source location as `file|l1|c1|l2|c2` (1-based, inclusive end)
fn location<'db>(db: &'db dyn Db, origin: Origin<'db>) -> String {
	let (src, span) = origin.source_span(db);
	let position = |offset: usize| {
		let before = &src.contents()[..offset];
		let line = before.matches('\n').count() + 1;
		let line_start = before.rfind('\n').map_or(0, |i| i + 1);
		let col = before[line_start..].chars().count() + 1;
		(line, col)
	};
	let (l1, c1) = position(span.offset());
	let (l2, c2) = position(span.offset() + span.len().saturating_sub(1));
	format!("{}|{l1}|{c1}|{l2}|{c2}", src.name())
}

/// Remove the redundant parentheses of printed THIR (on one line)
fn tidy(text: String) -> String {
	let options = MiniZincFormatOptions {
		line_width: usize::MAX,
		keep_parentheses: false,
		..Default::default()
	};
	format_str(format!("constraint {text};"), &options)
		.ok()
		.and_then(|f| {
			f.trim()
				.strip_prefix("constraint ")?
				.strip_suffix(';')
				.map(str::to_owned)
		})
		.unwrap_or(text)
}

struct Globalizer<'a, 'db> {
	db: &'db dyn Db,
	model: &'a Model<'db>,
	printer: OldMiniZincPrinter<'db>,
	parts: Vec<Part<'a, 'db>>,
	/// Printed declarations, functions etc. for the submodels, with the
	/// declaration if it is a variable (only printed when a group needs it)
	items: Vec<(Option<DeclarationId<'db>>, String)>,
	/// The declarations each user declaration refers to
	decl_refs: FxHashMap<DeclarationId<'db>, Vec<DeclarationId<'db>>>,
	/// Declarations that the items printed in every submodel (transitively)
	/// refer to
	always_needed: FxHashSet<DeclarationId<'db>>,
	options: &'a GlobalizeOptions,
	checker: Checker<'a>,
	/// Integer viewpoints found by the initial pass
	viewpoints: Vec<Viewpoint<'db>>,
}

impl<'a, 'db> Globalizer<'a, 'db> {
	fn print(&self, e: &Expression<'db>) -> String {
		self.printer.print_expression(self.db, self.model, e)
	}

	fn name(&self, d: DeclarationId<'db>) -> String {
		self.printer.print_declaration_id(self.db, self.model, d)
	}

	/// Print the jobs for all groups and contexts, except those in `skip` (a
	/// group without context in `skip` stands for all its contexts), and
	/// check them
	fn run(
		&self,
		pass: Pass,
		skip: &FxHashSet<(Vec<usize>, Option<usize>)>,
	) -> Result<Vec<RawFinding>> {
		let decls = self
			.parts
			.iter()
			.map(|p| p.decls(self.model))
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
		let found = self.checker.check_all(self.options.jobs.max(1), |send| {
			for group in groups(&vars, self.options.max_group) {
				if skip.contains(&(group.clone(), None)) {
					continue;
				}
				let variants = self.variants(&group);
				let group_vars = group.iter().flat_map(|p| &vars[*p]).collect::<Vec<_>>();
				// No context, or one other part that shares variables with the
				// group
				let contexts = std::iter::once(None).chain(
					(0..self.parts.len())
						.filter(|c| {
							!group.contains(c) && vars[*c].iter().any(|v| group_vars.contains(&v))
						})
						.map(Some),
				);
				for context in contexts {
					if skip.contains(&(group.clone(), context)) {
						continue;
					}
					let group_decls = group
						.iter()
						.chain(&context)
						.flat_map(|p| decls[*p].iter().copied())
						.unique()
						.collect::<Vec<_>>();
					if let Some(job) = self.job(&group, context, &variants, &group_decls, pass)
						&& !send(job)
					{
						return;
					}
				}
			}
		})?;
		let keep = shadow(&found);
		Ok(found
			.into_iter()
			.zip(keep)
			.filter_map(|(f, keep)| keep.then_some(f))
			.collect())
	}

	fn finding(&self, raw: &RawFinding) -> Finding {
		let mut locations = raw
			.group
			.iter()
			.map(|p| location(self.db, self.parts[*p].origin))
			.collect::<Vec<_>>();
		locations.dedup();
		Finding {
			locations,
			context: raw.context.map(|c| location(self.db, self.parts[c].origin)),
			replacement: raw.replacement.clone(),
		}
	}
}

/// Suggest global constraints that can replace groups of constraints in
/// `model`, checked by sampling solutions on each data file
pub fn globalize(
	model: &shackle::Model,
	data: &[PathBuf],
	options: &GlobalizeOptions,
) -> Result<Globalized> {
	let db = model.db();
	let errors = run_hir_phase(db)
		.errors
		.into_iter()
		.cloned()
		.collect::<Vec<_>>();
	if !errors.is_empty() {
		return Err(Error::try_from(errors).unwrap());
	}
	with_globalizer(db, data, options, |globalizer| {
		let initial = if options.initial_pass {
			globalizer.run(Pass::Initial, &FxHashSet::default())?
		} else {
			Vec::new()
		};
		let redundant = initial
			.iter()
			.filter(|f| f.candidate.kind == Kind::Redundant)
			.collect::<Vec<_>>();
		let skip = redundant
			.iter()
			.map(|f| (f.group.clone(), f.context))
			.collect();
		globalizer.viewpoints = globalizer.viewpoints(&initial);
		let full = globalizer.run(Pass::Full, &skip)?;
		let findings = redundant
			.into_iter()
			.chain(&full)
			.map(|f| globalizer.finding(f))
			.collect::<BTreeSet<_>>();
		Ok(Globalized {
			viewpoints: globalizer
				.viewpoints
				.iter()
				.map(|vp| vp.text.clone())
				.collect(),
			findings: findings.into_iter().collect(),
			calls: globalizer.checker.runner.calls.load(Ordering::Relaxed),
		})
	})
}

/// Whether a model file is the user's (not part of a standard library).
/// Files found through the search directories (`-I`) count as the user's.
fn is_user_file(db: &dyn Db, file: ModelFile) -> bool {
	let ModelFile::Named(file) = file else {
		return true;
	};
	let path = file.path(db);
	let libraries = [
		shackle_share_directory(db).clone(),
		CompilerSettings::get(db)
			.minizinc_stdlib_directory(db)
			.clone(),
	];
	!libraries.iter().flatten().any(|lib| path.starts_with(lib))
}

/// Set up a [`Globalizer`] for the model in `db` and pass it to `f`
fn with_globalizer<R>(
	db: &dyn Db,
	data: &[PathBuf],
	options: &GlobalizeOptions,
	f: impl FnOnce(&mut Globalizer<'_, '_>) -> R,
) -> R {
	let is_user_item = |o: Origin<'_>| o.node().is_some_and(|n| is_user_file(db, n.model_file(db)));
	let model = shackle_thir::lower::lower_model(db).take();
	// Inline the user's functions, so their constraints are split into parts
	let user_functions = model
		.top_level_functions()
		.filter(|(_, f)| f.body().is_some() && is_user_item(f.origin()))
		.map(|(f, _)| f)
		.collect::<FxHashSet<_>>();
	let model = inline_user_functions(db, model, user_functions);
	let user_items = model
		.top_level_items()
		.filter(|item| {
			!matches!(
				item,
				ItemId::Constraint(_) | ItemId::Output(_) | ItemId::Solve
			) && is_user_item(model.item_origin(*item))
		})
		.collect::<Vec<_>>();
	// After inlining, a user function is only printed if it is still called
	// (e.g. recursively), so it adds no unrelated variables to the submodels
	let mut calls = CallCollector::default();
	for (_, c) in model.top_level_constraints() {
		if is_user_item(c.origin()) {
			calls.visit_expression(&model, c.expression());
		}
	}
	for item in &user_items {
		if !matches!(item, ItemId::Function(_)) {
			calls.visit_item(&model, *item);
		}
	}
	// Library functions never call user functions, so they are not expanded
	let called = closure(calls.functions, |f| {
		if !is_user_item(model[*f].origin()) {
			return Vec::new();
		}
		let mut calls = CallCollector::default();
		calls.visit_function(&model, *f);
		calls.functions
	});

	// The THIR is printed as written: without transforms, so not totalised
	let printer = OldMiniZincPrinter::new(db, false, true);
	let mut items = Vec::new();
	let mut always = DeclCollector::default();
	for item in user_items {
		let var = match item {
			ItemId::Declaration(d) if !model[d].ty().known_par(db) => Some(d),
			ItemId::Function(f) if model[f].body().is_some() && !called.contains(&f) => continue,
			_ => None,
		};
		if var.is_none() {
			always.visit_item(&model, item);
		}
		items.push((var, print_item(&printer, db, &model, item)));
	}
	let decl_refs = model
		.top_level_declarations()
		.filter(|(_, d)| is_user_item(d.origin()))
		.map(|(d, _)| (d, decl_refs(&model, d)))
		.collect::<FxHashMap<_, _>>();
	let always_needed = closure(always.decls, |d| {
		decl_refs.get(d).cloned().unwrap_or_default()
	})
	.into_iter()
	.collect();

	let mut globalizer = Globalizer {
		db,
		model: &model,
		printer,
		parts: normalise(db, &model, is_user_item),
		items,
		decl_refs,
		always_needed,
		options,
		checker: Checker {
			runner: Runner::new(options),
			data: if data.is_empty() {
				vec![None]
			} else {
				data.iter().cloned().map(Some).collect()
			},
		},
		viewpoints: Vec::new(),
	};
	f(&mut globalizer)
}

#[cfg(test)]
mod tests;
