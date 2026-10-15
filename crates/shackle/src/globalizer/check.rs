//! Checking candidates by sampling. This stage works on printed submodels
//! only (no compiler database), so jobs can run on several threads.

use std::{fmt::Write, path::PathBuf};

use itertools::Itertools;
use serde_json::Value;

use super::{
	GlobalizeOptions,
	candidates::Candidate,
	run::{Outcome, Runner, Solution},
	tidy,
};
use crate::Result;

/// One unrolled group (a variant), printed
#[derive(Debug)]
pub(super) struct Plan {
	/// Includes, helper library and the declarations the group needs
	pub(super) preamble: String,
	/// Solve items: plain search, and search with restarts
	pub(super) search: [String; 2],
	/// Leader declarations, one for each instantiation (one empty string if
	/// there are no leaders)
	pub(super) leaders: Vec<String>,
	/// The unrolled parts of the group
	pub(super) group: Vec<String>,
	/// The context constraint
	pub(super) context: Option<String>,
	pub(super) candidates: Vec<Candidate>,
	/// For each leader: its generator, and the leaders its set depends on
	pub(super) wrap: Vec<(String, Vec<usize>)>,
}

/// All variants of one group with one context
#[derive(Debug)]
pub(super) struct Job {
	pub(super) group: Vec<usize>,
	pub(super) context: Option<usize>,
	pub(super) plans: Vec<Plan>,
}

/// Result of sampling a submodel
pub(super) enum Sample {
	Solutions(Vec<Solution>),
	/// No solutions (or none found within the time limit)
	Unsat,
	/// `minizinc` failed
	Error,
}

/// Runs the checks (shared by all worker threads)
#[derive(Debug)]
pub(super) struct Checker {
	pub(super) runner: Runner,
	/// The data files, or one `None` if there are none
	pub(super) data: Vec<Option<PathBuf>>,
	pub(super) options: GlobalizeOptions,
}

impl Checker {
	/// Sample solutions of `body`
	pub(super) fn sample(&self, body: &str, plan: &Plan, data: &Option<PathBuf>) -> Result<Sample> {
		let n = self.options.samples;
		let data = data.as_deref();
		let check = |o: Outcome| match o.error {
			Some(err) => {
				log::info!("globalizer: submodel failed: {err}");
				Sample::Error
			}
			None if o.solutions.is_empty() => Sample::Unsat,
			None => Sample::Solutions(o.solutions),
		};
		// A plain search proves completeness if there are few solutions
		let plain = self
			.runner
			.run(&format!("{body}{}", plan.search[0]), data, n)?;
		if plain.complete || plain.error.is_some() || plain.solutions.is_empty() {
			return Ok(check(plain));
		}
		// Restarts give more diverse samples, but can never prove completeness
		let restart = self
			.runner
			.run(&format!("{body}{}", plan.search[1]), data, n)?;
		if restart.error.is_none() && restart.solutions.len() >= n {
			Ok(check(restart))
		} else {
			Ok(check(plain))
		}
	}

	/// Keep the candidates that hold on all samples of `base`, and fill their
	/// blanks. Returns `None` if the submodel has no samples.
	pub(super) fn check_candidates(
		&self,
		base: &str,
		plan: &Plan,
		data: &Option<PathBuf>,
		mut cands: Vec<Candidate>,
	) -> Result<Option<Vec<Candidate>>> {
		let mut body = base.to_owned();
		writeln!(
			body,
			"array[1..{}] of var bool: GZ_R :: output = [{}];",
			cands.len(),
			cands.iter().map(Candidate::checked).join(", ")
		)
		.unwrap();
		for (k, c) in cands.iter().enumerate() {
			if let Some(feature) = c.feature() {
				writeln!(
					body,
					"array[int] of var int: GZ_F{k} :: output = {feature};"
				)
				.unwrap();
			}
		}
		let solutions = match self.sample(&body, plan, data)? {
			Sample::Solutions(s) => s,
			Sample::Unsat => return Ok(None),
			// One bad candidate (e.g. a failed assertion) fails the whole run:
			// split the candidates to find it
			Sample::Error if cands.len() > 1 => {
				let second = cands.split_off(cands.len() / 2);
				let Some(mut first) = self.check_candidates(base, plan, data, cands)? else {
					return Ok(None);
				};
				let Some(second) = self.check_candidates(base, plan, data, second)? else {
					return Ok(None);
				};
				first.extend(second);
				return Ok(Some(first));
			}
			Sample::Error => return Ok(Some(Vec::new())),
		};
		let mut k = 0;
		cands.retain_mut(|c| {
			k += 1;
			let holds = solutions
				.iter()
				.all(|s| s["GZ_R"].get(k - 1) == Some(&Value::Bool(true)));
			let features = solutions
				.iter()
				.map(|s| {
					s.get(&format!("GZ_F{}", k - 1))
						.and_then(Value::as_array)
						.map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
						.unwrap_or_default()
				})
				.collect::<Vec<_>>();
			holds && c.fill(&features)
		});
		Ok(Some(cands))
	}

	/// Whether `a` implies `b` on the samples of `a`
	pub(super) fn implies(
		&self,
		base: &str,
		plan: &Plan,
		data: &Option<PathBuf>,
		a: &str,
		b: &str,
	) -> Result<bool> {
		let body = format!("{base}constraint {a};\nvar bool: GZ_I :: output = {b};\n");
		Ok(match self.sample(&body, plan, data)? {
			Sample::Solutions(s) => s.iter().all(|s| s["GZ_I"] == Value::Bool(true)),
			_ => false,
		})
	}

	/// Candidates that hold on all samples of the group (forward check)
	fn forward(&self, plan: &Plan) -> Result<Vec<Candidate>> {
		let mut cands = plan.candidates.clone();
		let constraints = plan.group.iter().chain(&plan.context).collect::<Vec<_>>();
		let mut tested = false;
		for data in &self.data {
			for leaders in &plan.leaders {
				if cands.is_empty() {
					return Ok(cands);
				}
				let mut base = format!("{}{leaders}", plan.preamble);
				// Abandon a pair if one constraint implies the other
				if let [a, b] = &constraints[..]
					&& self.options.implies_check
					&& (self.implies(&base, plan, data, a, b)?
						|| self.implies(&base, plan, data, b, a)?)
				{
					return Ok(Vec::new());
				}
				for c in &constraints {
					writeln!(base, "constraint {c};").unwrap();
				}
				// Without samples, skip the instantiation (outside the `where`
				// clauses of the leaders)
				let Some(kept) = self.check_candidates(&base, plan, data, cands.clone())? else {
					continue;
				};
				cands = kept;
				tested = true;
			}
		}
		Ok(if tested { cands } else { Vec::new() })
	}

	/// Drop candidates that hold without the group (on the samples of the
	/// declarations and the context). Such a candidate can only pass the
	/// reverse check if the group is redundant, and then only `true` is
	/// reported. This saves a reverse check for each of them.
	fn vacuous(&self, plan: &Plan, cands: Vec<Candidate>) -> Result<Vec<Candidate>> {
		let (trivial, mut cands): (Vec<_>, Vec<_>) =
			cands.into_iter().partition(|c| c.name == "true");
		if cands.is_empty() {
			return Ok(trivial);
		}
		let mut base = format!("{}{}", plan.preamble, plan.leaders[0]);
		if let Some(c) = &plan.context {
			writeln!(base, "constraint {c};").unwrap();
		}
		if let Some(implied) = self.check_candidates(&base, plan, &self.data[0], cands.clone())? {
			cands.retain(|c| !implied.contains(c));
		}
		cands.extend(trivial);
		Ok(cands)
	}

	/// Whether all samples of the candidate (with the context) satisfy the
	/// group (reverse check)
	fn reverse(&self, plan: &Plan, cand: &Candidate) -> Result<bool> {
		let conj = plan.group.iter().map(|c| format!("({c})")).join(" /\\ ");
		let mut tested = false;
		for data in &self.data {
			for leaders in &plan.leaders {
				let mut body = format!("{}{leaders}", plan.preamble);
				writeln!(body, "constraint {};", cand.call()).unwrap();
				if let Some(c) = &plan.context {
					writeln!(body, "constraint {c};").unwrap();
				}
				writeln!(body, "var bool: GZ_G :: output = {conj};").unwrap();
				match self.sample(&body, plan, data)? {
					Sample::Solutions(solutions) => {
						if !solutions.iter().all(|s| s["GZ_G"] == Value::Bool(true)) {
							return Ok(false);
						}
						tested = true;
					}
					// Outside the `where` clauses of the leaders
					Sample::Unsat => {}
					Sample::Error => return Ok(false),
				}
			}
		}
		Ok(tested)
	}

	/// The replacement text: the call inside a `forall` over the leaders it
	/// uses (and the leaders their sets depend on)
	pub(super) fn replacement(plan: &Plan, cand: &Candidate) -> String {
		let mut used = cand.leaders.clone();
		let mut i = 0;
		while i < used.len() {
			for m in &plan.wrap[used[i]].1 {
				if !used.contains(m) {
					used.push(*m);
				}
			}
			i += 1;
		}
		if used.is_empty() {
			return tidy(cand.call());
		}
		used.sort_unstable();
		let gens = used.iter().map(|m| plan.wrap[*m].0.as_str()).join(", ");
		tidy(format!("forall({gens})({})", cand.call()))
	}

	/// The replacements found for one group with one context
	pub(super) fn check_job(&self, job: &Job) -> Result<Vec<String>> {
		let mut found = Vec::new();
		for plan in &job.plans {
			for cand in self.vacuous(plan, self.forward(plan)?)? {
				if self.reverse(plan, &cand)? {
					found.push((Self::replacement(plan, &cand), cand));
				}
			}
		}
		// A redundant group needs no global
		if found.iter().any(|(_, c)| c.name == "true") {
			found.retain(|(_, c)| c.name == "true");
		}
		// `all_different(x)` is the clearest of its equivalent forms
		let different = found
			.iter()
			.filter(|(_, c)| c.name == "all_different")
			.map(|(_, c)| c.args[0].clone())
			.collect::<Vec<_>>();
		found.retain(|(_, c)| match c.name {
			"nvalue" => !different.contains(&c.args[1]),
			"global_cardinality" if c.filled => {
				!(different.contains(&c.args[0])
					&& c.args[1]
						.trim_matches(['[', ']'])
						.split(", ")
						.all(|n| n == "1"))
			}
			_ => true,
		});
		// Prefer arguments of the model over values filled from the samples
		let given = found
			.iter()
			.filter(|(_, c)| !c.filled)
			.map(|(_, c)| c.clone())
			.collect::<Vec<_>>();
		found.retain(|(_, c)| {
			!c.filled
				|| !given.iter().any(|g| {
					g.name == c.name
						&& g.args.len() == c.args.len()
						&& (0..g.args.len())
							.filter(|i| g.args[*i] != c.args[*i])
							.count()
							<= 1
				})
		});
		Ok(found.into_iter().map(|(r, _)| r).unique().collect())
	}
}
