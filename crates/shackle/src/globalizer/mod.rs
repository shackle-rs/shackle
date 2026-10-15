//! Globalizer: suggests global constraints that can replace groups of
//! constraints in a model.
//!
//! This is a port of the Haskell [globalizer](https://github.com/MiniZinc/Globalizer)
//! (Leo, Mears, Tack, Garcia de la Banda, "Globalizing Constraint Models",
//! CP 2013). It works on the THIR directly after type checking:
//!
//! 1. Split the constraints into *parts* (conjunctions, nested `forall`s).
//! 2. Enumerate *groups* of up to `max_group` parts that share variables, and
//!    ways to *unroll* their loops: a loop variable becomes a *leader* that is
//!    fixed to the first or the last value of its set.
//! 3. Sample solutions of each group with `minizinc`, and evaluate every
//!    candidate global on them in the same run (forward check).
//! 4. Sample solutions of each surviving candidate and check that they satisfy
//!    the group (reverse check).
//!
//! Results are based on sampling for each data file, so they are suggestions,
//! not proofs.

mod candidates;
mod check;
mod parts;
mod run;

use std::{
	collections::BTreeSet,
	fmt::{self, Display, Write},
	path::PathBuf,
	sync::{
		Mutex,
		atomic::{AtomicUsize, Ordering},
		mpsc,
	},
	time::Duration,
};

use itertools::Itertools;
use rustc_hash::FxHashSet;
use shackle_fmt::{MiniZincFormatOptions, format_str};
use shackle_hir::{Db, constants::IdentifierRegistry, input::InputFiles};
use shackle_thir::{
	Callable, DeclarationId, Expression, ExpressionData, ItemId, Model, ResolvedIdentifier,
	compat::OldMiniZincPrinter,
	pretty_print::{Printer, print_item},
	source::Origin,
	transform::inlining::inline_user_functions,
	traverse::{Visitor, visit_call, visit_expression},
};
use shackle_ty::VarType;

use self::{
	candidates::{Arg, ArgType, Candidate, candidates},
	check::{Checker, Job, Plan},
	parts::{Loop, LoopKind, Part, decl_refs, is_par_int, normalise, refs},
	run::Runner,
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

/// Result of [`crate::Model::globalize`]
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

/// Enumerate the groups of up to `max` parts (as sorted indices) whose
/// variables connect them.
fn groups<T: PartialEq>(vars: &[Vec<T>], max: usize) -> Vec<Vec<usize>> {
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

/// Which candidates a pass checks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
	/// Only `true` (redundant groups) and `binaries_represent_int`
	Initial,
	/// All other candidates
	Full,
}

impl Pass {
	fn checks(self, name: &str) -> bool {
		let initial = name == "true" || name.starts_with("binaries_represent_int");
		initial == (self == Pass::Initial)
	}
}

/// A 2D integer array that represents a 3D 0/1 array `array`, which has
/// exactly one 1 along `axis`
#[derive(Clone, Debug)]
struct Viewpoint<'db> {
	array: DeclarationId<'db>,
	axis: usize,
	name: String,
	/// Declaration and channelling constraint (as suggested to the user)
	text: String,
	/// Definition as a function of the 0/1 array that does not constrain it,
	/// so it does not change the samples of a group
	definition: String,
	/// The channelling constraint
	channel: String,
}

impl<'db> Viewpoint<'db> {
	fn new(array: DeclarationId<'db>, x: &str, axis: usize) -> Self {
		let name = format!("{x}_3{}", ["a", "b", "c"][axis]);
		let set = |k: usize| format!("index_set_{}of3({x})", k + 1);
		let others = (0..3).filter(|k| *k != axis).collect::<Vec<_>>();
		let idx = ["i", "j", "k"];
		let declaration = format!(
			"array [{}, {}] of var {}: {name};",
			set(others[0]),
			set(others[1]),
			set(axis)
		);
		let channel = format!(
			"constraint forall (i in {}, j in {}, k in {}) ({x}[i, j, k] = 1 <-> {name}[{}, {}] = {});",
			set(0),
			set(1),
			set(2),
			idx[others[0]],
			idx[others[1]],
			idx[axis],
		);
		let definition = format!(
			"array [{b}, {c}] of var int: {name} = array2d({b}, {c}, [sum ({} in {}) ({} * {x}[i, j, k]) | {} in {b}, {} in {c}]);",
			idx[axis],
			set(axis),
			idx[axis],
			idx[others[0]],
			idx[others[1]],
			b = set(others[0]),
			c = set(others[1]),
		);
		Viewpoint {
			array,
			axis,
			text: format!("{declaration} {channel}"),
			name,
			definition,
			channel,
		}
	}
}

/// How a group is unrolled. Leader `m` is the loop `fixed[0][m]` of the
/// first part, and `fixed[k][m]` is the matching loop of the `k`-th part.
/// Each leader is set to the first or the last value of its set.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Variant {
	fixed: Vec<Vec<usize>>,
}

/// One unrolled group: the parts of a group with a variant
struct Unrolled<'g, 'a, 'db> {
	parts: Vec<&'g Part<'a, 'db>>,
	variant: &'g Variant,
	/// Another constraint that is part of the submodel, but not replaced
	context: Option<&'g Part<'a, 'db>>,
	/// Top-level declarations the parts and the context use
	decls: Vec<DeclarationId<'db>>,
}

impl<'a, 'db> Unrolled<'_, 'a, 'db> {
	fn leaders(&self) -> usize {
		self.variant.fixed[0].len()
	}

	/// The leader that a loop declaration of the group is fixed to
	fn leader_of(&self, d: DeclarationId<'db>) -> Option<usize> {
		self.parts
			.iter()
			.zip(&self.variant.fixed)
			.find_map(|(p, f)| f.iter().position(|l| p.loops[*l].decl == d))
	}

	fn leader_loop(&self, m: usize) -> &Loop<'a, 'db> {
		&self.parts[0].loops[self.variant.fixed[0][m]]
	}
}

/// Finds array accesses `x[i, j]` whose indices are leaders or par integers
struct AccessCollector<'u, 'g, 'a, 'db> {
	db: &'db dyn Db,
	unrolled: &'u Unrolled<'g, 'a, 'db>,
	array_access: shackle_hir::Identifier<'db>,
	/// For each array: the allowed indices (declaration, leader) per position
	accesses: Vec<(DeclarationId<'db>, Vec<Vec<IndexChoice<'db>>>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct IndexChoice<'db> {
	decl: DeclarationId<'db>,
	leader: Option<usize>,
}

impl<'a, 'db> Visitor<'a, 'db> for AccessCollector<'_, '_, '_, 'db> {
	fn visit_call(&mut self, model: &'a Model<'db>, call: &'a shackle_thir::Call<'db>) {
		if let Callable::Function(f) = &call.function
			&& model[*f].name() == self.array_access
			&& call.arguments.len() == 2
			&& let ExpressionData::Identifier(ResolvedIdentifier::Declaration(x)) =
				&*call.arguments[0]
			&& model[*x].top_level()
		{
			let indices = match &*call.arguments[1] {
				ExpressionData::TupleLiteral(t) => t.iter().collect::<Vec<_>>(),
				_ => vec![&call.arguments[1]],
			};
			let pos = match self.accesses.iter().position(|(d, _)| d == x) {
				Some(pos) => pos,
				None => {
					self.accesses.push((*x, vec![Vec::new(); indices.len()]));
					self.accesses.len() - 1
				}
			};
			let choices = &mut self.accesses[pos].1;
			for (i, idx) in indices.iter().enumerate() {
				if let ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) = &***idx {
					let leader = self.unrolled.leader_of(*d);
					let choice = IndexChoice { decl: *d, leader };
					if (leader.is_some() || is_par_int(self.db, &model[*d]))
						&& i < choices.len()
						&& !choices[i].contains(&choice)
					{
						choices[i].push(choice);
					}
				}
			}
		}
		visit_call(self, model, call);
	}
}

struct Globalizer<'a, 'db> {
	db: &'db dyn Db,
	model: &'a Model<'db>,
	printer: OldMiniZincPrinter<'db>,
	parts: Vec<Part<'a, 'db>>,
	/// Printed declarations, functions etc. for the submodels, with the
	/// declaration if it is a variable (only printed when a group needs it)
	items: Vec<(Option<DeclarationId<'db>>, String)>,
	options: &'a GlobalizeOptions,
	checker: Checker,
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

	fn print_loop(&self, l: &Loop<'a, 'db>) -> String {
		let mut s = match l.kind {
			LoopKind::Iter(c) => format!("{} in {}", self.name(l.decl), self.print(c)),
			LoopKind::Assign(e) => format!("{} = {}", self.name(l.decl), self.print(e)),
		};
		if let Some(w) = l.where_clause {
			write!(s, " where {}", self.print(w)).unwrap();
		}
		s
	}

	fn print_part(&self, part: &Part<'a, 'db>) -> String {
		self.print_unrolled(part, &[])
	}

	/// Print a part with the loops at `fixed` taken out. Their `where` clauses
	/// become a condition.
	fn print_unrolled(&self, part: &Part<'a, 'db>, fixed: &[usize]) -> String {
		let body = self.print(part.body);
		let loops = (0..part.loops.len())
			.filter(|i| !fixed.contains(i))
			.map(|i| self.print_loop(&part.loops[i]))
			.collect::<Vec<_>>();
		let inner = if loops.is_empty() {
			body
		} else {
			format!("forall({})({body})", loops.join(", "))
		};
		let guards = fixed
			.iter()
			.filter_map(|i| part.loops[*i].where_clause)
			.map(|w| format!("({})", self.print(w)))
			.collect::<Vec<_>>();
		if guards.is_empty() {
			inner
		} else {
			format!("({}) -> ({inner})", guards.join(" /\\ "))
		}
	}

	/// The ways to unroll a group (always including no unrolling)
	fn variants(&self, group: &[usize]) -> Vec<Variant> {
		let first = &self.parts[group[0]];
		let mut result = Vec::new();
		for subset in (0..first.loops.len()).powerset() {
			if !first.can_fix(self.db, self.model, &subset) {
				continue;
			}
			let sets = subset
				.iter()
				.map(|l| self.loop_set(&first.loops[*l]))
				.collect::<Vec<_>>();
			// Every other part must unroll loops over the same sets
			let mut options = vec![vec![subset.clone()]];
			for p in &group[1..] {
				let part = &self.parts[*p];
				let matches = (0..part.loops.len())
					.permutations(subset.len())
					.filter(|ls| {
						ls.iter().zip(&sets).all(|(l, s)| {
							matches!(part.loops[*l].kind, LoopKind::Iter(_))
								&& self.loop_set(&part.loops[*l]) == *s
						}) && part.can_fix(self.db, self.model, ls)
					})
					.collect::<Vec<_>>();
				options.push(matches);
			}
			for fixed in options.into_iter().multi_cartesian_product() {
				result.push(Variant { fixed });
			}
		}
		result
	}

	fn loop_set(&self, l: &Loop<'a, 'db>) -> String {
		match l.kind {
			LoopKind::Iter(c) => self.print(c),
			LoopKind::Assign(_) => String::new(),
		}
	}

	/// Declarations of the leaders for one instantiation (`last[m]`: leader
	/// `m` takes the last value of its set instead of the first)
	fn print_leaders(&self, u: &Unrolled<'_, 'a, 'db>, last: &[bool]) -> String {
		// ponytail: leaders keep the loop variable names, so a top-level
		// declaration with the same name clashes; rename if that shows up.
		let mut s = String::new();
		for (m, last) in last.iter().enumerate() {
			let l = u.leader_loop(m);
			writeln!(
				s,
				"any: {} = {}({});",
				self.name(l.decl),
				if *last { "max" } else { "min" },
				self.loop_set(l)
			)
			.unwrap();
		}
		for (part, fixed) in u.parts.iter().zip(&u.variant.fixed).skip(1) {
			for (m, l) in fixed.iter().enumerate() {
				let (name, leader) = (
					self.name(part.loops[*l].decl),
					self.name(u.leader_loop(m).decl),
				);
				if name != leader {
					writeln!(s, "any: {name} = {leader};").unwrap();
				}
			}
		}
		// An instantiation outside the `where` clauses says nothing about the
		// group: it makes the submodel unsatisfiable, and is skipped
		for (part, fixed) in u.parts.iter().zip(&u.variant.fixed) {
			for w in fixed.iter().filter_map(|l| part.loops[*l].where_clause) {
				writeln!(s, "constraint {};", self.print(w)).unwrap();
			}
		}
		s
	}

	fn print_group(&self, u: &Unrolled<'_, 'a, 'db>) -> Vec<String> {
		u.parts
			.iter()
			.zip(&u.variant.fixed)
			.map(|(p, f)| self.print_unrolled(p, f))
			.collect()
	}

	/// All instantiations of the leaders
	fn instantiations(u: &Unrolled<'_, 'a, 'db>) -> Vec<Vec<bool>> {
		(0..1usize << u.leaders())
			.map(|mask| (0..u.leaders()).map(|m| mask & (1 << m) != 0).collect())
			.collect()
	}

	/// The model without constraints, solve and output items. Of the variables
	/// it only declares those `decls` (transitively) depend on, so the solver
	/// does not enumerate unrelated variables.
	fn preamble(&self, decls: &[DeclarationId<'db>]) -> String {
		let mut needed = decls.to_vec();
		let mut i = 0;
		while i < needed.len() {
			for d in decl_refs(self.model, needed[i]) {
				if !needed.contains(&d) {
					needed.push(d);
				}
			}
			i += 1;
		}
		let mut preamble = format!("include \"globals.mzn\";{}", candidates::LIBRARY);
		for (decl, text) in &self.items {
			if decl.is_none_or(|d| needed.contains(&d)) {
				writeln!(preamble, "{text};").unwrap();
			}
		}
		for vp in &self.viewpoints {
			if needed.contains(&vp.array) {
				writeln!(preamble, "{}", vp.definition).unwrap();
			}
		}
		preamble
	}

	/// Candidate arguments: the group's int declarations, their slices and
	/// accesses with leader or par indices, and its scalar variables as one
	/// array
	fn arguments(&self, u: &Unrolled<'_, 'a, 'db>) -> Vec<Arg> {
		let db = self.db;
		let int_ty = |d: DeclarationId<'db>| {
			let ty = self.model[d].ty();
			let elem = ty.elem_ty(db).filter(|_| ty.is_array(db)).unwrap_or(ty);
			(elem.is_int(db) && ty.opt(db) == Some(shackle_ty::OptType::NonOpt)).then(|| {
				(
					elem.inst(db) == Some(VarType::Var),
					ty.dims(db).unwrap_or(0),
				)
			})
		};
		// The literal 0 (e.g. a lower bound)
		let mut args = vec![Arg {
			text: "0".to_owned(),
			var: false,
			ty: ArgType::Int(0),
			leaders: Vec::new(),
		}];
		let mut scalars = Vec::new();
		for &d in &u.decls {
			let Some((var, dims)) = int_ty(d) else {
				continue;
			};
			if dims == 0 && var {
				scalars.push(self.name(d));
			}
			args.push(Arg {
				text: self.name(d),
				var,
				ty: ArgType::Int(dims),
				leaders: Vec::new(),
			});
		}
		for vp in self
			.viewpoints
			.iter()
			.filter(|vp| u.decls.contains(&vp.array))
		{
			args.push(Arg {
				text: vp.name.clone(),
				var: true,
				ty: ArgType::Int(2),
				leaders: Vec::new(),
			});
		}
		// Scalar variables of the group as one array (e.g. `[a, b, c]`). Two
		// variables are left out: such a global is just a binary constraint.
		scalars.sort();
		if scalars.len() > 2 {
			args.push(Arg {
				text: format!("[{}]", scalars.join(", ")),
				var: true,
				ty: ArgType::Int(1),
				leaders: Vec::new(),
			});
		}

		let mut collector = AccessCollector {
			db,
			unrolled: u,
			array_access: IdentifierRegistry::lookup(db).functions.array_access,
			accesses: Vec::new(),
		};
		for part in &u.parts {
			for e in part.loops.iter().flat_map(|l| l.exprs()).chain([part.body]) {
				visit_expression(&mut collector, self.model, e);
			}
		}
		// Accesses of `array` with one index (or a slice `..`, as `None`) per
		// position
		let mut slices = |array: String, var: bool, positions: Vec<&Vec<IndexChoice<'db>>>| {
			let choices = positions.iter().map(|cs| {
				std::iter::once(None)
					.chain(cs.iter().map(Some))
					.collect::<Vec<_>>()
			});
			for chosen in choices.multi_cartesian_product() {
				if chosen.iter().all(Option::is_none) {
					continue;
				}
				let indices = chosen
					.iter()
					.map(|c| match c {
						None => "..".to_owned(),
						Some(c) => match c.leader {
							Some(m) => self.name(u.leader_loop(m).decl),
							None => self.name(c.decl),
						},
					})
					.collect::<Vec<_>>();
				let text = format!("{array}[{}]", indices.join(", "));
				if args.iter().any(|a| a.text == text) {
					continue;
				}
				args.push(Arg {
					text,
					var,
					ty: ArgType::Int(chosen.iter().filter(|c| c.is_none()).count()),
					leaders: chosen.iter().flatten().filter_map(|c| c.leader).collect(),
				});
			}
		};
		for (x, positions) in &collector.accesses {
			let Some((var, dims)) = int_ty(*x) else {
				continue;
			};
			if dims != positions.len() {
				continue;
			}
			slices(self.name(*x), var, positions.iter().collect());
			// The viewpoint has the indices of the other two axes
			for vp in self.viewpoints.iter().filter(|vp| vp.array == *x) {
				let other = (0..3)
					.filter(|k| *k != vp.axis)
					.map(|k| &positions[k])
					.collect();
				slices(vp.name.clone(), true, other);
			}
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

	/// Print an unrolled group with its candidates
	fn plan(&self, u: &Unrolled<'_, 'a, 'db>, pass: Pass) -> Plan {
		let options = self.options;
		// The name filters of the user only apply to the full pass
		let (include, exclude) = match pass {
			Pass::Initial => (&[][..], &[][..]),
			Pass::Full => (&options.include[..], &options.exclude[..]),
		};
		let mut cands = candidates(&self.arguments(u), include, exclude, options.max_args);
		cands.retain(|c| pass.checks(c.name));
		// A group that already is the global is vacuous
		if let [single] = &u.parts[..]
			&& let ExpressionData::Call(c) = &**single.body
			&& let Callable::Function(f) = &c.function
		{
			let name = self.model[*f].name().as_identifier(self.db).lookup(self.db);
			cands.retain(|c| c.name != name);
		}
		// Leaders only check the first and last iteration: too weak to call the
		// whole constraint redundant
		if u.leaders() > 0 {
			cands.retain(|c| c.name != "true");
		}
		let wrap = (0..u.leaders())
			.map(|m| {
				let deps = refs(self.model, true, u.leader_loop(m).exprs());
				let depends = (0..u.leaders())
					.filter(|k| *k != m && deps.contains(&u.leader_loop(*k).decl))
					.collect();
				(self.print_loop(u.leader_loop(m)), depends)
			})
			.collect();
		Plan {
			preamble: self.preamble(&u.decls),
			search: [self.search(&u.decls, false), self.search(&u.decls, true)],
			leaders: Self::instantiations(u)
				.iter()
				.map(|last| self.print_leaders(u, last))
				.collect(),
			group: self.print_group(u),
			context: u.context.map(|c| self.print_part(c)),
			candidates: cands,
			wrap,
			channels: self
				.viewpoints
				.iter()
				.filter(|vp| u.decls.contains(&vp.array))
				.map(|vp| (vp.name.clone(), vp.channel.clone()))
				.collect(),
		}
	}

	/// All variants of a group with a context, printed
	fn job(
		&self,
		group: &[usize],
		context: Option<usize>,
		decls: &[DeclarationId<'db>],
		pass: Pass,
	) -> Job {
		let plans = self
			.variants(group)
			.iter()
			.map(|variant| {
				let u = Unrolled {
					parts: group.iter().map(|p| &self.parts[*p]).collect(),
					variant,
					context: context.map(|c| &self.parts[c]),
					decls: decls.to_vec(),
				};
				self.plan(&u, pass)
			})
			.collect();
		Job {
			group: group.to_vec(),
			context,
			plans,
		}
	}

	/// Print the jobs on this thread (it owns the compiler database), and
	/// check them on `jobs` worker threads. Groups in `skip` are left out.
	fn run(&self, pass: Pass, skip: &[Vec<usize>]) -> Result<Vec<RawFinding>> {
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
		let workers = self.options.jobs.max(1);
		let checker = &self.checker;
		let found = Mutex::new(Vec::new());
		let failure = Mutex::new(None);
		let (send, receive) = mpsc::sync_channel::<Job>(workers);
		let receive = Mutex::new(receive);
		std::thread::scope(|scope| {
			for _ in 0..workers {
				scope.spawn(|| {
					loop {
						let Ok(job) = receive.lock().unwrap().recv() else {
							break;
						};
						match checker.check_job(&job) {
							Ok(replacements) => {
								found.lock().unwrap().extend(replacements.into_iter().map(
									|(replacement, candidate)| RawFinding {
										group: job.group.clone(),
										context: job.context,
										replacement,
										candidate,
									},
								))
							}
							Err(err) => *failure.lock().unwrap() = Some(err),
						}
					}
				});
			}
			for group in groups(&vars, self.options.max_group) {
				if skip.contains(&group) {
					continue;
				}
				let group_vars = group.iter().flat_map(|p| &vars[*p]).collect::<Vec<_>>();
				// No context, or one other part that shares variables with the group
				let contexts = std::iter::once(None).chain(
					(0..self.parts.len())
						.filter(|c| {
							!group.contains(c) && vars[*c].iter().any(|v| group_vars.contains(&v))
						})
						.map(Some),
				);
				for context in contexts {
					let mut group_decls = Vec::new();
					for d in group.iter().chain(&context).flat_map(|p| &decls[*p]) {
						if !group_decls.contains(d) {
							group_decls.push(*d);
						}
					}
					let _ = send.send(self.job(&group, context, &group_decls, pass));
				}
			}
			// Let the workers stop when the queue is empty
			drop(send);
		});
		if let Some(err) = failure.into_inner().unwrap() {
			return Err(err);
		}
		let found = found.into_inner().unwrap();
		Ok(shadow(&found).into_iter().cloned().collect())
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

	/// The viewpoints for the `binaries_represent_int` findings
	fn viewpoints(&self, found: &[RawFinding]) -> Vec<Viewpoint<'db>> {
		let mut viewpoints = Vec::<Viewpoint<'db>>::new();
		for raw in found {
			let c = &raw.candidate;
			let Some(axis) = c
				.name
				.strip_prefix("binaries_represent_int_3")
				.and_then(|a| "ABC".find(a))
			else {
				continue;
			};
			let array = self
				.model
				.top_level_declarations()
				.map(|(d, _)| d)
				.find(|d| self.name(*d) == c.args[0]);
			if let Some(array) = array
				&& !viewpoints
					.iter()
					.any(|vp| vp.array == array && vp.axis == axis)
			{
				viewpoints.push(Viewpoint::new(array, &c.args[0], axis));
			}
		}
		// The workers find them in any order
		viewpoints.sort_by(|a, b| a.name.cmp(&b.name));
		viewpoints
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

/// Drop findings whose replacement is also found for a smaller group (a
/// subset of the parts, or the same parts without a context), and findings
/// with a context whose replacement is found for a group that also contains
/// the context
fn shadow(found: &[RawFinding]) -> Vec<&RawFinding> {
	found
		.iter()
		.filter(|f| {
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

/// Run the globalizer on the (type checked) model in `db`
pub(crate) fn globalize(
	db: &dyn Db,
	data: &[PathBuf],
	options: &GlobalizeOptions,
) -> Result<Globalized> {
	with_globalizer(db, data, options, |globalizer| {
		// The initial pass finds redundant groups (left out of the full pass)
		// and 0/1 arrays that represent integers (viewpoints for the full pass)
		let initial = if options.initial_pass {
			globalizer.run(Pass::Initial, &[])?
		} else {
			Vec::new()
		};
		let redundant = initial
			.iter()
			.filter(|f| f.candidate.name == "true")
			.collect::<Vec<_>>();
		let skip = redundant
			.iter()
			.filter(|f| f.context.is_none())
			.map(|f| f.group.clone())
			.collect::<Vec<_>>();
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

/// Set up a [`Globalizer`] for the model in `db` and pass it to `f`
fn with_globalizer<R>(
	db: &dyn Db,
	data: &[PathBuf],
	options: &GlobalizeOptions,
	f: impl FnOnce(&mut Globalizer<'_, '_>) -> R,
) -> R {
	let model = shackle_thir::lower::lower_model(db).take();
	let user_files = FxHashSet::from_iter(InputFiles::get(db).files(db));
	let is_user_item = |o: Origin<'_>| {
		o.node()
			.is_some_and(|n| user_files.contains(&n.model_file(db)))
	};
	// Inline the user's predicates and functions, so their constraints can be
	// split into parts
	let user_functions = model
		.top_level_functions()
		.filter(|(_, f)| f.body().is_some() && is_user_item(f.origin()))
		.map(|(f, _)| f)
		.collect::<FxHashSet<_>>();
	let model = inline_user_functions(db, &model, user_functions);

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

	let mut globalizer = Globalizer {
		db,
		model: &model,
		printer,
		parts: normalise(db, &model, is_user_item),
		items,
		options,
		checker: Checker {
			runner: Runner {
				solver: options.solver.clone(),
				time_limit: options.time_limit,
				seed: options.seed,
				calls: AtomicUsize::new(0),
				cache: Default::default(),
			},
			data: if data.is_empty() {
				vec![None]
			} else {
				data.iter().cloned().map(Some).collect()
			},
			options: options.clone(),
		},
		viewpoints: Vec::new(),
	};
	f(&mut globalizer)
}

#[cfg(test)]
mod tests;
