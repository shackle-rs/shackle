//! Printing groups as MiniZinc submodels

use std::fmt::Write;

use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use shackle_hir::constants::IdentifierRegistry;
use shackle_thir::{Callable, DeclarationId, Expression, ExpressionData, IntegerLiteral};

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
		// Leaders keep the loop variable names (a loop variable never has a
		// top-level, enum or annotation name, see `inline_user_functions`)
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
		// The variables of each element type, as one array
		let mut vars = FxHashMap::<Elem, Vec<String>>::default();
		for d in decls {
			if let Some((true, _, elem)) = self.arg_decl(*d) {
				let name = self.name(*d);
				vars.entry(elem)
					.or_default()
					.push(if self.model[*d].ty().is_array(self.db) {
						format!("array1d({name})")
					} else {
						format!("[{name}]")
					});
			}
		}
		let search =
			|kind: &str, vars: &str| format!("{kind}({vars}, input_order, indomain_random)");
		let searches = [Elem::Int, Elem::OptInt, Elem::Bool, Elem::Set]
			.into_iter()
			.filter_map(|elem| Some((elem, vars.get(&elem)?.join(" ++ "))))
			.flat_map(|(elem, vars)| match elem {
				Elem::Int => vec![search("int_search", &vars)],
				// Whether each value occurs, then its value
				Elem::OptInt => vec![
					search("bool_search", &format!("[occurs(gz_e) | gz_e in {vars}]")),
					search("int_search", &vars),
				],
				Elem::Bool => vec![search("bool_search", &vars)],
				// Whether each possible member is in the set
				Elem::Set => vec![search(
					"bool_search",
					&format!("[gz_e in gz_s | gz_s in {vars}, gz_e in ub(gz_s)]"),
				)],
			})
			.collect::<Vec<_>>();
		let solve = match &searches[..] {
			[] => "solve".to_owned(),
			[one] => format!("solve :: {one}"),
			_ => format!("solve :: seq_search([{}])", searches.join(", ")),
		};
		let plain = format!("{solve} satisfy;\n");
		[plain, format!("{solve} :: restart_luby(1) satisfy;\n")]
	}

	/// The candidate names (see `Global::uses`) of the functions that a
	/// constraint is a call of, or that a comparison compares, with `element`
	/// for an access with a variable index (e.g. `=` and `max` for
	/// `m = max(x)`, and `=` and `element` for `y = x[i]`)
	fn uses(&self, e: &Expression<'db>) -> Vec<&str> {
		let ids = IdentifierRegistry::lookup(self.db);
		let zero =
			|m: &Expression<'db>| matches!(&**m, ExpressionData::IntegerLiteral(IntegerLiteral(0)));
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
			// Other stdlib names of the same globals (`all_different(x, S)` is
			// the one with exceptions, `{0}` gives `alldifferent_except_0`)
			Some(match name.as_identifier(self.db).lookup(self.db) {
				"all_different" | "alldifferent" | "alldifferent_except"
					if c.arguments.len() == 2 =>
				{
					match &*c.arguments[1] {
						ExpressionData::SetLiteral(members) if members.is_empty() => {
							"all_different"
						}
						ExpressionData::SetLiteral(members) if matches!(&members[..], [m] if zero(m)) => {
							"alldifferent_except_0"
						}
						_ => "all_different_except",
					}
				}
				"alldifferent" => "all_different",
				"all_different_except_0" => "alldifferent_except_0",
				"atmost1" => "at_most1",
				"strict_lex2" => "lex2_strict",
				"lex_lt" => "lex_less",
				"lex_leq" => "lex_lesseq",
				// With the arguments swapped
				"lex_greater" => "lex_less",
				"lex_greatereq" => "lex_lesseq",
				name => name,
			})
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
		let mut cands = candidates(&self.arguments(u), select, options.max_args);
		if cands.is_empty() {
			return None;
		}
		// For each leader: the leaders its set depends on
		let depends = (0..u.leaders())
			.map(|m| {
				let deps = refs(self.model, true, u.leader_loop(m).exprs());
				(0..u.leaders())
					.filter(|k| *k != m && deps.contains(&u.leader_loop(*k).decl))
					.collect::<Vec<_>>()
			})
			.collect::<Vec<_>>();
		// The replacement must loop over every leader: only the first and last
		// iteration of a leader are checked, so a call outside its loop would
		// also replace the iterations in between. Such a call is checked by
		// the variant without that leader.
		cands.retain(|c| {
			closure(c.leaders.iter().copied(), |m| depends[*m].iter().copied()).len() == u.leaders()
		});
		if cands.is_empty() {
			return None;
		}
		Some(Plan {
			leaders: u
				.instantiations()
				.iter()
				.map(|last| self.print_leaders(u, last))
				.collect(),
			group: self.print_group(u),
			candidates: cands,
			wrap: (0..u.leaders())
				.map(|m| self.print_loop(u.leader_loop(m)))
				.collect(),
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
