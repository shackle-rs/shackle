//! Checking candidates by sampling. This stage works on printed submodels
//! only (no compiler database), so jobs run on several worker threads.

use std::{
	fmt::Write,
	path::PathBuf,
	sync::{
		Arc, Mutex,
		atomic::{AtomicBool, Ordering},
		mpsc,
	},
};

use itertools::Itertools;
use serde_json::Value;
use shackle_diagnostics::{Error, Result};

use super::{
	GlobalizeOptions, RawFinding,
	candidates::{Candidate, Kind},
	run::{Outcome, Runner, Solution},
	tidy,
};

/// One unrolled group (a variant), printed
#[derive(Debug)]
pub(super) struct Plan {
	/// Leader declarations, one for each instantiation (one empty string if
	/// there are no leaders)
	pub(super) leaders: Vec<String>,
	/// The unrolled parts of the group
	pub(super) group: Vec<String>,
	pub(super) candidates: Vec<Candidate>,
	/// For each leader: its generator
	pub(super) wrap: Vec<String>,
}

/// All variants of one group with one context, printed
#[derive(Debug)]
pub(super) struct Job {
	pub(super) group: Vec<usize>,
	pub(super) context: Option<usize>,
	/// Includes, helper library and the declarations the group needs
	pub(super) preamble: String,
	/// Solve items: plain search, and search with restarts
	pub(super) search: Search,
	/// The context constraint
	pub(super) context_text: Option<String>,
	/// Viewpoints (name and channelling constraint) the candidates can use
	pub(super) channels: Vec<(String, String)>,
	pub(super) plans: Vec<Plan>,
}

/// Result of sampling a submodel
pub(super) enum Sample {
	/// The outcome of the run, which has solutions
	Solutions(Arc<Outcome>),
	/// No solutions
	Unsat,
	/// No solutions found within the time limit
	Unknown,
	/// `minizinc` failed
	Error,
}

impl From<Arc<Outcome>> for Sample {
	fn from(outcome: Arc<Outcome>) -> Self {
		match &outcome.error {
			Some(err) => {
				log::info!("globalizer: submodel failed: {err}");
				Sample::Error
			}
			None if outcome.solutions.is_empty() && outcome.complete => Sample::Unsat,
			None if outcome.solutions.is_empty() => Sample::Unknown,
			None => Sample::Solutions(outcome),
		}
	}
}

/// The replacements of one group to report, from all that were found (with
/// their candidates)
pub(super) fn rank(mut found: Vec<(String, Candidate)>) -> Vec<(String, Candidate)> {
	// A redundant group needs no global
	if found.iter().any(|(_, c)| c.kind == Kind::Redundant) {
		found.retain(|(_, c)| c.kind == Kind::Redundant);
	}
	// Drop a candidate that is an equivalent but less clear form of another
	// one that holds (e.g. `nvalue(n, x)` of `all_different(x)`)
	let held = found.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>();
	found.retain(|(_, c)| {
		!c.dominated_by.is_some_and(|(name, slot)| {
			held.iter()
				.any(|h| h.name == name && h.args.first() == c.args.get(slot))
		})
	});
	// Prefer arguments of the model over values filled from the samples:
	// drop a filled candidate if the same global holds with the same other
	// arguments (a filled blank can add arguments, e.g. a cover)
	let given = found
		.iter()
		.filter(|(_, c)| c.filled.is_none())
		.map(|(_, c)| c.clone())
		.collect::<Vec<_>>();
	found.retain(|(_, c)| {
		let Some(slot) = c.filled else {
			return true;
		};
		!given.iter().any(|g| {
			g.name == c.name
				&& (0..g.args.len()).all(|i| i == slot || c.args.get(i) == Some(&g.args[i]))
		})
	});
	found.into_iter().unique_by(|(r, _)| r.clone()).collect()
}

/// Whether the boolean output `name` is true in every solution
fn all_true(solutions: &[Solution], name: &str) -> bool {
	solutions
		.iter()
		.all(|s| s.get(name) == Some(&Value::Bool(true)))
}

/// Runs the checks (shared by all worker threads)
#[derive(Debug)]
pub(super) struct Checker<'a> {
	pub(super) runner: Runner<'a>,
	/// The data files, or one `None` if there are none
	pub(super) data: Vec<Option<PathBuf>>,
}

/// Solve items: plain search, and search with restarts
pub(super) type Search = [String; 2];

impl<'a> Checker<'a> {
	fn options(&self) -> &'a GlobalizeOptions {
		self.runner.options
	}

	/// Sample solutions of `body` (a submodel without solve item). Outcomes
	/// are cached, unless `cache` is false (for runs that are unlikely to
	/// repeat).
	pub(super) fn sample(
		&self,
		body: &str,
		search: &Search,
		data: &Option<PathBuf>,
		cache: bool,
	) -> Result<Sample> {
		let n = self.options().samples;
		let data = data.as_deref();
		// A plain search proves completeness if there are few solutions
		let plain = self
			.runner
			.run(&format!("{body}{}", search[0]), data, n, cache)?;
		if plain.complete || plain.error.is_some() {
			return Ok(Sample::from(plain));
		}
		// Restarts give more diverse samples, but cannot prove that a set of
		// solutions is complete. The plain samples often share the first
		// variables, so both are used.
		let restart = self
			.runner
			.run(&format!("{body}{}", search[1]), data, n, cache)?;
		if restart.error.is_some() {
			return Ok(Sample::from(plain));
		}
		if restart.complete && restart.solutions.is_empty() {
			return Ok(Sample::Unsat);
		}
		Ok(Sample::from(Arc::new(Outcome {
			solutions: plain
				.solutions
				.iter()
				.chain(&restart.solutions)
				.cloned()
				.collect(),
			complete: false,
			error: None,
		})))
	}

	/// The data files and leader instantiations of a plan, with the preamble
	/// and the leader declarations of each
	fn instances<'j>(
		&'j self,
		job: &'j Job,
		plan: &'j Plan,
	) -> impl Iterator<Item = (&'j Option<PathBuf>, String)> {
		self.data.iter().flat_map(move |data| {
			plan.leaders
				.iter()
				.map(move |leaders| (data, format!("{}{leaders}", job.preamble)))
		})
	}

	/// Keep the candidates that hold on all samples of `base`, and fill their
	/// blanks. Returns `None` if the submodel has no samples.
	pub(super) fn check_candidates(
		&self,
		base: &str,
		search: &Search,
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
			if let Some(witness) = &c.witness {
				writeln!(body, "var int: GZ_W{k} :: output = {witness};").unwrap();
			}
		}
		// The outputs differ for each list of candidates, so these runs are not
		// cached
		let outcome = match self.sample(&body, search, data, false)? {
			Sample::Solutions(outcome) => outcome,
			Sample::Unsat => return Ok(None),
			// The candidates cannot be checked on this instance, so none of
			// them is suggested: a time out can hide a counterexample
			Sample::Unknown => return Ok(Some(Vec::new())),
			// Without the candidates the run may still fail (e.g. a leader
			// outside its set): then the instance has no samples
			Sample::Error if matches!(self.sample(base, search, data, true)?, Sample::Error) => {
				return Ok(None);
			}
			// One bad candidate (e.g. a failed assertion) fails the whole run:
			// split the candidates to find it
			Sample::Error if cands.len() > 1 => {
				let second = cands.split_off(cands.len() / 2);
				let Some(mut first) = self.check_candidates(base, search, data, cands)? else {
					return Ok(None);
				};
				let Some(second) = self.check_candidates(base, search, data, second)? else {
					return Ok(None);
				};
				first.extend(second);
				return Ok(Some(first));
			}
			Sample::Error => return Ok(Some(Vec::new())),
		};
		let solutions = &outcome.solutions;
		let mut k = 0;
		cands.retain_mut(|c| {
			k += 1;
			let holds = solutions
				.iter()
				.all(|s| s.get("GZ_R").and_then(|r| r.get(k - 1)) == Some(&Value::Bool(true)));
			if !holds {
				return false;
			}
			// The witness shows that the candidate is not trivially true
			if c.witness.is_some() {
				let key = format!("GZ_W{}", k - 1);
				let witnessed = solutions
					.iter()
					.any(|s| s.get(&key).and_then(Value::as_i64).is_some_and(|w| w > 0));
				if !witnessed {
					return false;
				}
			}
			if c.blank.is_none() {
				return true;
			}
			let key = format!("GZ_F{}", k - 1);
			let features = solutions
				.iter()
				.map(|s| {
					s.get(&key)
						.and_then(Value::as_array)
						.map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
						.unwrap_or_default()
				})
				.collect::<Vec<_>>();
			c.fill(&features)
		});
		Ok(Some(cands))
	}

	/// Whether `a` implies `b` on the samples of `a`
	pub(super) fn implies(
		&self,
		base: &str,
		search: &Search,
		data: &Option<PathBuf>,
		a: &str,
		b: &str,
	) -> Result<bool> {
		let body = format!("{base}constraint {a};\nvar bool: GZ_I :: output = {b};\n");
		Ok(match self.sample(&body, search, data, true)? {
			Sample::Solutions(o) => all_true(&o.solutions, "GZ_I"),
			_ => false,
		})
	}

	/// Candidates that hold on all samples of the group (forward check)
	fn forward(&self, job: &Job, plan: &Plan) -> Result<Vec<Candidate>> {
		let mut cands = plan.candidates.clone();
		// `true` holds on any sample: only the reverse check matters
		if cands.iter().all(|c| c.kind == Kind::Redundant) {
			return Ok(cands);
		}
		let constraints = plan
			.group
			.iter()
			.chain(&job.context_text)
			.collect::<Vec<_>>();
		let Some(held) = self.holding(job, plan, &constraints, cands)? else {
			return Ok(Vec::new());
		};
		cands = held;
		// Abandon a pair if one constraint implies the other (checked last, as
		// usually no candidate is left)
		if let [a, b] = &plan.group[..]
			&& self.options().implies_check
			&& !cands.is_empty()
		{
			for (data, base) in self.instances(job, plan) {
				if self.implies(&base, &job.search, data, a, b)?
					|| self.implies(&base, &job.search, data, b, a)?
				{
					return Ok(Vec::new());
				}
			}
		}
		Ok(cands)
	}

	/// Drop candidates that hold without the group (on the samples of the
	/// declarations and the context) for every data file and instantiation
	/// with samples. Such a candidate can only pass the reverse check if the
	/// group is redundant, and then only `true` is reported. This saves a
	/// reverse check for each of them.
	fn vacuous(&self, job: &Job, plan: &Plan, cands: Vec<Candidate>) -> Result<Vec<Candidate>> {
		let (trivial, mut cands): (Vec<_>, Vec<_>) =
			cands.into_iter().partition(|c| c.kind == Kind::Redundant);
		let context = job.context_text.iter().collect::<Vec<_>>();
		if let Some(implied) = self.holding(job, plan, &context, cands.clone())? {
			cands.retain(|c| !implied.contains(c));
		}
		cands.extend(trivial);
		Ok(cands)
	}

	/// The candidates that hold on the samples with `constraints`, for every
	/// data file and instantiation that has samples (the others are outside
	/// the `where` clauses of the leaders). `None` if none has samples.
	fn holding(
		&self,
		job: &Job,
		plan: &Plan,
		constraints: &[&String],
		mut cands: Vec<Candidate>,
	) -> Result<Option<Vec<Candidate>>> {
		let mut sampled = false;
		for (data, mut body) in self.instances(job, plan) {
			if cands.is_empty() {
				break;
			}
			for c in constraints {
				writeln!(body, "constraint {c};").unwrap();
			}
			if let Some(held) = self.check_candidates(&body, &job.search, data, cands.clone())? {
				cands = held;
				sampled = true;
			}
		}
		Ok(sampled.then_some(cands))
	}

	/// Whether all samples of the candidate (with the context) satisfy the
	/// group (reverse check)
	fn reverse(&self, job: &Job, plan: &Plan, cand: &Candidate) -> Result<bool> {
		let conj = plan.group.iter().map(|c| format!("({c})")).join(" /\\ ");
		// A viewpoint comes with its channelling constraint
		let channels = job
			.channels
			.iter()
			.filter(|(name, _)| {
				cand.args.iter().any(|a| {
					a.strip_prefix(name.as_str())
						.is_some_and(|rest| rest.is_empty() || rest.starts_with('['))
				})
			})
			.map(|(_, channel)| channel.as_str())
			.join("\n");
		let mut tested = false;
		for (data, mut body) in self.instances(job, plan) {
			if let Some(c) = &job.context_text {
				writeln!(body, "constraint {c};").unwrap();
			}
			writeln!(body, "var bool: GZ_G :: output = {conj};").unwrap();
			// The channels must not give the group without the candidate
			if !channels.is_empty() {
				writeln!(body, "{channels}").unwrap();
				if let Sample::Solutions(o) = self.sample(&body, &job.search, data, true)?
					&& all_true(&o.solutions, "GZ_G")
				{
					return Ok(false);
				}
			}
			writeln!(body, "constraint {};", cand.call()).unwrap();
			match self.sample(&body, &job.search, data, true)? {
				Sample::Solutions(o) => {
					if !all_true(&o.solutions, "GZ_G") {
						return Ok(false);
					}
					tested = true;
				}
				// Outside the `where` clauses of the leaders
				Sample::Unsat => {}
				Sample::Unknown | Sample::Error => return Ok(false),
			}
		}
		Ok(tested)
	}

	/// The replacement text: the call inside a `forall` over the leaders (a
	/// candidate loops over all of them, see `plan`)
	pub(super) fn replacement(plan: &Plan, cand: &Candidate) -> String {
		if plan.wrap.is_empty() {
			return tidy(cand.call());
		}
		tidy(format!("forall({})({})", plan.wrap.join(", "), cand.call()))
	}

	/// The replacements (and candidates) found for one group with one context
	pub(super) fn check_job(&self, job: &Job) -> Result<Vec<(String, Candidate)>> {
		let mut found = Vec::new();
		for plan in &job.plans {
			let forward = self.forward(job, plan)?;
			for cand in self.vacuous(job, plan, forward)? {
				if self.reverse(job, plan, &cand)? {
					found.push((Self::replacement(plan, &cand), cand));
				}
			}
		}
		Ok(rank(found))
	}

	/// Check the jobs that `produce` sends, on `jobs` worker threads. The
	/// producer runs on this thread (it owns the compiler database). Its
	/// `send` returns `false` once checking has stopped (after an error).
	pub(super) fn check_all(
		&self,
		jobs: usize,
		produce: impl FnOnce(&mut dyn FnMut(Job) -> bool),
	) -> Result<Vec<RawFinding>> {
		let found = Mutex::new(Vec::new());
		let failure = Mutex::new(None::<Error>);
		let stop = AtomicBool::new(false);
		std::thread::scope(|scope| {
			let (send, receive) = mpsc::sync_channel::<Job>(jobs);
			// The workers own the receiver: if they all stop, `send` fails
			let receive = Arc::new(Mutex::new(receive));
			for _ in 0..jobs {
				let receive = Arc::clone(&receive);
				let (found, failure, stop) = (&found, &failure, &stop);
				let _ = scope.spawn(move || {
					while !stop.load(Ordering::Relaxed) {
						let Ok(job) = receive.lock().unwrap().recv() else {
							break;
						};
						match self.check_job(&job) {
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
							Err(err) => {
								stop.store(true, Ordering::Relaxed);
								let _ = failure.lock().unwrap().get_or_insert(err);
							}
						}
					}
				});
			}
			drop(receive);
			produce(&mut |job| !stop.load(Ordering::Relaxed) && send.send(job).is_ok());
		});
		match failure.into_inner().unwrap() {
			Some(err) => Err(err),
			None => Ok(found.into_inner().unwrap()),
		}
	}
}
