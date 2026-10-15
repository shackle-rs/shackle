//! Printing groups as MiniZinc submodels

use std::fmt::Write;

use itertools::Itertools;
use rustc_hash::FxHashSet;
use shackle_hir::constants::IdentifierRegistry;
use shackle_thir::{Callable, DeclarationId, Expression, ExpressionData};

use super::{
	Globalizer, Pass,
	candidates::{self, Elem, Kind, candidates, selected},
	check::{Job, Plan, Search},
	closure,
	groups::{Unrolled, Variant},
	parts::{Loop, LoopKind, Part, refs},
};

impl<'a, 'db> Globalizer<'a, 'db> {
	pub(super) fn print_loop(&self, l: &Loop<'a, 'db>) -> String {
		let mut s = match l.kind {
			LoopKind::Iter(c) => format!("{} in {}", self.name(l.decl), self.print(c)),
			LoopKind::Assign(e) => format!("{} = {}", self.name(l.decl), self.print(e)),
		};
		if let Some(w) = l.where_clause {
			write!(s, " where {}", self.print(w)).unwrap();
		}
		s
	}

	pub(super) fn print_part(&self, part: &Part<'a, 'db>) -> String {
		self.print_unrolled(part, &[])
	}

	/// Print a part with the loops at `fixed` taken out. Their `where` clauses
	/// become a condition.
	pub(super) fn print_unrolled(&self, part: &Part<'a, 'db>, fixed: &[usize]) -> String {
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

	pub(super) fn print_group(&self, u: &Unrolled<'_, 'a, 'db>) -> Vec<String> {
		u.parts
			.iter()
			.zip(&u.variant.fixed)
			.map(|(p, f)| self.print_unrolled(p, f))
			.collect()
	}

	/// Declarations of the leaders for one instantiation (see
	/// [`Unrolled::instantiations`])
	pub(super) fn print_leaders(&self, u: &Unrolled<'_, 'a, 'db>, last: &[bool]) -> String {
		// TODO: leaders keep the loop variable names, so a top-level
		// declaration with the same name clashes. Rename such leaders.
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
		// The matched loops of the other parts are aliases of the leaders (each
		// name once, see `Globalizer::variants`)
		let aliases = u
			.parts
			.iter()
			.zip(&u.variant.fixed)
			.skip(1)
			.flat_map(|(part, fixed)| {
				fixed.iter().enumerate().map(|(m, l)| {
					(
						self.name(part.loops[*l].decl),
						self.name(u.leader_loop(m).decl),
					)
				})
			})
			.filter(|(name, leader)| name != leader)
			.unique();
		for (name, leader) in aliases {
			writeln!(s, "any: {name} = {leader};").unwrap();
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

	/// The model without constraints, solve and output items. Of the variables
	/// it only declares those that `decls` and the par items (transitively)
	/// depend on, so the solver does not enumerate unrelated variables.
	pub(super) fn preamble(&self, decls: &[DeclarationId<'db>]) -> String {
		let mut needed = closure(decls.iter().copied(), |d| {
			self.decl_refs.get(d).cloned().unwrap_or_default()
		})
		.into_iter()
		.collect::<FxHashSet<_>>();
		needed.extend(&self.always_needed);
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

	/// Search annotation that picks random values for the group's variables
	/// (for a set: random members)
	pub(super) fn search(&self, decls: &[DeclarationId<'db>]) -> Search {
		let vars = |elem: Elem| {
			decls
				.iter()
				.filter(|d| self.arg_decl(**d) == Some((true, self.dims(**d), elem)))
				.map(|d| {
					if self.model[*d].ty().is_array(self.db) {
						format!("array1d({})", self.name(*d))
					} else {
						format!("[{}]", self.name(*d))
					}
				})
				.join(" ++ ")
		};
		let searches = [
			(vars(Elem::Int), "int_search"),
			(vars(Elem::OptInt), "int_search"),
			(vars(Elem::Bool), "bool_search"),
			(
				Some(vars(Elem::Set))
					.filter(|s| !s.is_empty())
					.map(|s| format!("[gz_e in gz_s | gz_s in {s}, gz_e in ub(gz_s)]"))
					.unwrap_or_default(),
				"bool_search",
			),
		]
		.into_iter()
		.filter(|(vars, _)| !vars.is_empty())
		.map(|(vars, search)| format!("{search}({vars}, input_order, indomain_random)"))
		.collect::<Vec<_>>();
		let solve = match &searches[..] {
			[] => "solve".to_owned(),
			[one] => format!("solve :: {one}"),
			_ => format!("solve :: seq_search([{}])", searches.join(", ")),
		};
		let plain = format!("{solve} satisfy;\n");
		[plain, format!("{solve} :: restart_luby(1) satisfy;\n")]
	}

	fn dims(&self, d: DeclarationId<'db>) -> usize {
		self.model[d].ty().dims(self.db).unwrap_or(0)
	}

	/// The functions that a constraint is a call of, or that a comparison
	/// compares, with `element` for an access with a variable index (e.g.
	/// `=` and `max` for `m = max(x)`, and `=` and `element` for `y = x[i]`)
	fn uses(&self, e: &Expression<'db>) -> Vec<&str> {
		let ids = IdentifierRegistry::lookup(self.db);
		let name = |e: &Expression<'db>| {
			let ExpressionData::Call(c) = &**e else {
				return None;
			};
			let Callable::Function(f) = &c.function else {
				return None;
			};
			let name = self.model[*f].name();
			if name == ids.functions.array_access {
				return (c.arguments.len() == 2 && !c.arguments[1].ty().known_par(self.db))
					.then_some("element");
			}
			Some(name.as_identifier(self.db).lookup(self.db))
		};
		let mut uses = name(e).into_iter().collect::<Vec<_>>();
		if let ExpressionData::Call(c) = &**e {
			uses.extend(c.arguments.iter().filter_map(name));
		}
		uses
	}

	/// Print an unrolled group with its candidates (`None` if there are no
	/// candidates)
	pub(super) fn plan(&self, u: &Unrolled<'_, 'a, 'db>, pass: Pass) -> Option<Plan> {
		let options = self.options;
		// A group that already is the global is vacuous
		let existing = match &u.parts[..] {
			[single] => self.uses(single.body),
			_ => Vec::new(),
		};
		let select = |name: &str, uses: &str, kind: Kind| {
			pass.checks(kind)
				// Leaders only check the first and last iteration: too weak to
				// call the whole constraint redundant
				&& !(kind == Kind::Redundant && u.leaders() > 0)
				&& !existing.contains(&uses)
				// The name filters of the user only apply to the full pass
				&& (pass == Pass::Initial || selected(name, &options.include, &options.exclude))
		};
		let cands = candidates(&self.arguments(u), select, options.max_args);
		if cands.is_empty() {
			return None;
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
		Some(Plan {
			leaders: u
				.instantiations()
				.iter()
				.map(|last| self.print_leaders(u, last))
				.collect(),
			group: self.print_group(u),
			candidates: cands,
			wrap,
		})
	}

	/// All variants of a group with a context, printed (`None` if none of
	/// them has candidates)
	pub(super) fn job(
		&self,
		group: &[usize],
		context: Option<usize>,
		variants: &[Variant],
		decls: &[DeclarationId<'db>],
		pass: Pass,
	) -> Option<Job> {
		let plans = variants
			.iter()
			.filter_map(|variant| {
				let u = Unrolled {
					parts: group.iter().map(|p| &self.parts[*p]).collect(),
					variant,
					decls,
				};
				self.plan(&u, pass)
			})
			.collect_vec();
		if plans.is_empty() {
			return None;
		}
		Some(Job {
			group: group.to_vec(),
			context,
			preamble: self.preamble(decls),
			search: self.search(decls),
			context_text: context.map(|c| self.print_part(&self.parts[c])),
			channels: self
				.viewpoints
				.iter()
				.filter(|vp| decls.contains(&vp.array))
				.map(|vp| (vp.name.clone(), vp.channel.clone()))
				.collect(),
			plans,
		})
	}
}
