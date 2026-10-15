//! Table of global constraints the globalizer can suggest, with their argument
//! signatures and the filters that reject trivial or ill-typed argument lists.
//! MiniZinc evaluates the candidates on the samples, so most of them are the
//! stdlib globals themselves.

use itertools::Itertools;

/// What a candidate means when it holds
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Kind {
	/// A global that can replace the group
	Normal,
	/// `true`: the group is redundant
	Redundant,
	/// The 3D 0/1 argument has exactly one 1 along the given axis, so it
	/// represents a 2D integer array (a viewpoint)
	Viewpoint(usize),
}

/// A possible argument for a candidate global
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Arg {
	/// MiniZinc text of the argument
	pub(super) text: String,
	/// Whether the argument contains decision variables
	pub(super) var: bool,
	/// Number of array dimensions of the integer argument (0 for a scalar)
	pub(super) dims: usize,
	/// Leaders (indices into the variant's leaders) the argument uses
	pub(super) leaders: Vec<usize>,
}

/// An argument computed from the samples instead of taken from the model
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Blank {
	/// The maximum of the argument in the given slot
	Max(usize),
	/// The value counts of the argument in the given slot
	Counts(usize),
}

/// A global constraint call to check against the samples
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Candidate {
	pub(super) name: &'static str,
	pub(super) kind: Kind,
	/// Argument texts. A blank slot is empty until it is filled.
	pub(super) args: Vec<String>,
	template: Option<&'static str>,
	/// Expression used instead of the call on the samples
	test: Option<&'static str>,
	/// Par condition: if false the candidate is rejected without evaluating it
	guard: Option<String>,
	/// Side condition that must hold on every sample (makes the candidate
	/// non-trivial)
	extra: Option<String>,
	/// Unfilled blank: its slot and how to compute it
	pub(super) blank: Option<(usize, Blank)>,
	/// The slot of the blank, once it is filled
	pub(super) filled: Option<usize>,
	/// Leaders the arguments use (sorted)
	pub(super) leaders: Vec<usize>,
	/// Integer that must be positive on some sample
	pub(super) witness: Option<String>,
	/// This is a less clear form of another global (name), whose first
	/// argument is the argument in the given slot: report only the other one
	/// if both hold
	pub(super) dominated_by: Option<(&'static str, usize)>,
}

/// `global_cardinality` with an explicit cover (`{2}`), for filled counts
const COVERED_GCC: &str = "global_cardinality({0}, {2}, {1})";

impl Candidate {
	fn instantiate(&self, template: &str) -> String {
		self.args
			.iter()
			.enumerate()
			.fold(template.to_owned(), |s, (i, a)| {
				s.replace(&format!("{{{i}}}"), a)
			})
	}

	/// The global call, e.g. `all_different(x)`
	pub(super) fn call(&self) -> String {
		match self.template {
			Some(t) => self.instantiate(t),
			None => format!("{}({})", self.name, self.args.join(", ")),
		}
	}

	/// Boolean expression to evaluate on the samples. For an unfilled blank
	/// only the conditions are checked.
	pub(super) fn checked(&self) -> String {
		let mut body = self.extra.iter().cloned().collect::<Vec<_>>();
		if self.blank.is_none() {
			body.push(match self.test {
				Some(t) => self.instantiate(t),
				None => self.call(),
			});
		}
		let body = if body.is_empty() {
			"true".to_owned()
		} else {
			body.iter().map(|b| format!("({b})")).join(" /\\ ")
		};
		match &self.guard {
			Some(guard) => format!("if {guard} then {body} else false endif"),
			None => body,
		}
	}

	/// Integer array to output so the blank can be filled from the samples
	pub(super) fn feature(&self) -> Option<String> {
		let (_, blank) = self.blank?;
		Some(match blank {
			Blank::Max(i) => format!("[max({})]", self.args[i]),
			Blank::Counts(i) => {
				let x = &self.args[i];
				format!("[lb_array({x})] ++ [count({x}, v) | v in lb_array({x})..ub_array({x})]")
			}
		})
	}

	/// Fill the blank from the feature values of all samples. Fails if the
	/// samples do not agree.
	pub(super) fn fill(&mut self, values: &[Vec<i64>]) -> bool {
		let Some((slot, blank)) = self.blank else {
			return true;
		};
		let Some(first) = values.first() else {
			return false;
		};
		if values.iter().any(|v| v != first) {
			return false;
		}
		self.args[slot] = match (blank, &first[..]) {
			(Blank::Max(_), [m]) => m.to_string(),
			// At most 101 counts, so the suggested array stays readable
			(Blank::Counts(source), [lo, counts @ ..]) if counts.len() <= 101 => {
				// Values that never occur at the ends are left out (the reverse
				// check shows whether that is still equivalent)
				let start = counts.iter().position(|c| *c != 0).unwrap_or(0);
				let end = counts.iter().rposition(|c| *c != 0).map_or(0, |e| e + 1);
				let (lo, counts) = (&(lo + start as i64), &counts[start..end.max(start)]);
				if counts.is_empty() {
					return false;
				}
				if counts.iter().all(|c| *c == 1) {
					self.dominated_by = Some(("all_different", source));
				}
				let hi = lo + counts.len() as i64 - 1;
				self.args.push(format!("[{}]", (*lo..=hi).join(", ")));
				self.template = Some(COVERED_GCC);
				format!("[{}]", counts.iter().join(", "))
			}
			_ => return false,
		};
		self.blank = None;
		self.filled = Some(slot);
		true
	}
}

type Cond = fn(&[&Arg]) -> Option<String>;

struct Global {
	name: &'static str,
	kind: Kind,
	/// Number of array dimensions of each integer argument
	signature: &'static [usize],
	/// Rejects argument lists the global is not useful for
	check: fn(&[&Arg]) -> bool,
	guard: Cond,
	extra: Cond,
	/// Slot that may be computed from the samples instead
	blank: Option<(usize, Blank)>,
	/// Call text with `{i}` for argument `i` (default: `name(args)`)
	template: Option<&'static str>,
	/// Check on the samples if the global cannot be used in a reified context
	test: Option<&'static str>,
	/// Integer that must be positive on some sample, else the global is
	/// trivially true there
	witness: Cond,
	/// See [`Candidate::dominated_by`]
	dominated_by: Option<(&'static str, usize)>,
}

/// Checks used by `test` templates, included in every submodel
pub(super) const LIBRARY: &str = r#"
function var int: gz_succ(array [int] of var int: x, int: k, var int: s) =
  if k = 0 then s else gz_succ(x, k - 1, x[s]) endif;
predicate gz_circuit(array [int] of var int: x) =
  all_different(x) /\ forall (k in 1..length(x) - 1) (
    gz_succ(x, k, min(index_set(x))) != min(index_set(x)));
predicate gz_subcircuit(array [int] of var int: x) =
  let { var int: m = sum (i in index_set(x)) (x[i] != i) } in
  all_different(x) /\ forall (i in index_set(x), k in 1..length(x) - 1) (
    x[i] != i /\ gz_succ(x, k, i) = i -> k >= m);
"#;

const fn global(name: &'static str, signature: &'static [usize]) -> Global {
	Global {
		name,
		kind: Kind::Normal,
		signature,
		check: |_| true,
		guard: |_| None,
		extra: |_| None,
		blank: None,
		template: None,
		test: None,
		witness: |_| None,
		dominated_by: None,
	}
}

/// The 3D 0/1 argument has exactly one 1 along `axis` (`check` is the sum
/// along that axis)
const fn binaries(name: &'static str, axis: usize, check: &'static str) -> Global {
	Global {
		kind: Kind::Viewpoint(axis),
		check: |a| a[0].var,
		guard: |a| Some(format!("dom_array({}) subset 0..1", a[0].text)),
		template: Some(check),
		..global(name, &[3])
	}
}

/// Both arrays have the same length, of at least 2
fn lex_guard(a: &[&Arg]) -> Option<String> {
	Some(format!("{} /\\ length({}) > 1", same_length(a)?, a[0].text))
}

fn same_length(a: &[&Arg]) -> Option<String> {
	let lengths = a
		.iter()
		.map(|a| format!("length({})", a.text))
		.collect::<Vec<_>>();
	Some(
		lengths
			.windows(2)
			.map(|w| format!("{} = {}", w[0], w[1]))
			.join(" /\\ "),
	)
}

fn same_index_set(x: &Arg, y: &Arg) -> String {
	format!("index_set({}) = index_set({})", x.text, y.text)
}

const GLOBALS: &[Global] = &[
	// A 0/1 array whose sums along one axis are 1 represents a 2D integer
	// array (used by the initial pass to add a viewpoint)
	binaries(
		"binaries_represent_int_3A",
		0,
		"forall (j in index_set_2of3({0}), k in index_set_3of3({0})) (sum (i in index_set_1of3({0})) ({0}[i, j, k]) = 1)",
	),
	binaries(
		"binaries_represent_int_3B",
		1,
		"forall (i in index_set_1of3({0}), k in index_set_3of3({0})) (sum (j in index_set_2of3({0})) ({0}[i, j, k]) = 1)",
	),
	binaries(
		"binaries_represent_int_3C",
		2,
		"forall (i in index_set_1of3({0}), j in index_set_2of3({0})) (sum (k in index_set_3of3({0})) ({0}[i, j, k]) = 1)",
	),
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_different", &[1])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_equal", &[1])
	},
	Global {
		check: |a| a[0].var,
		// 0 must be possible, else this is all_different
		guard: |a| {
			let x = &a[0].text;
			Some(format!("length({x}) > 2 /\\ 0 in dom_array({x})"))
		},
		dominated_by: Some(("all_different", 0)),
		..global("alldifferent_except_0", &[1])
	},
	Global {
		check: |a| !a[0].var && a[1].var && !a[2].var,
		guard: |a| {
			Some(format!(
				"{} /\\ ub_array({b}) - lb_array({b}) <= 20",
				same_index_set(a[1], a[2]),
				b = a[1].text
			))
		},
		..global("bin_packing", &[0, 1, 1])
	},
	Global {
		check: |a| !a[0].var && a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		..global("bin_packing_capa", &[1, 1, 1])
	},
	Global {
		check: |a| a[0].var && a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		..global("bin_packing_load", &[1, 1, 1])
	},
	Global {
		check: |a| a[0].var,
		test: Some("gz_circuit({0})"),
		..global("circuit", &[1])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[2].text)),
		..global("count", &[1, 0, 0])
	},
	Global {
		check: |a| !a[1].var && !a[2].var,
		guard: |a| {
			Some(format!(
				"{} /\\ min({}) > 0 /\\ min({}) > 0",
				same_length(&a[..3])?,
				a[1].text,
				a[2].text
			))
		},
		extra: |a| Some(format!("length({}) > 1 /\\ {} > 0", a[0].text, a[3].text)),
		..global("cumulative", &[1, 1, 1, 0])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("decreasing", &[1])
	},
	Global {
		check: |a| (a[0].var && a[1].var) || (a[2].var && a[3].var),
		guard: same_length,
		..global("diffn", &[1, 1, 1, 1])
	},
	Global {
		guard: same_length,
		..global("disjunctive", &[1, 1])
	},
	Global {
		guard: |a| same_length(&a[..2]),
		..global("distribute", &[1, 1, 1])
	},
	Global {
		check: |a| a[0].var,
		blank: Some((1, Blank::Counts(0))),
		template: Some("global_cardinality({0}, set2array(index_set({1})), {1})"),
		..global("global_cardinality", &[1, 1])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("increasing", &[1])
	},
	Global {
		guard: same_length,
		..global("inverse", &[1, 1])
	},
	Global {
		guard: lex_guard,
		..global("lex_less", &[1, 1])
	},
	Global {
		guard: lex_guard,
		..global("lex_lesseq", &[1, 1])
	},
	Global {
		check: |a| a[0].var,
		..global("lex2", &[2])
	},
	Global {
		blank: Some((0, Blank::Max(1))),
		test: Some("{0} = max({1})"),
		..global("maximum", &[0, 1])
	},
	Global {
		test: Some("{0} = min({1})"),
		..global("minimum", &[0, 1])
	},
	Global {
		extra: |a| Some(format!("{} <= length({})", a[0].text, a[1].text)),
		dominated_by: Some(("all_different", 1)),
		..global("nvalue", &[0, 1])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && !a[2].var && a[3].var,
		guard: |a| {
			Some(format!(
				"{l} <= {u} /\\ 1 < {n} /\\ {n} <= length({vs})",
				l = a[0].text,
				u = a[1].text,
				n = a[2].text,
				vs = a[3].text
			))
		},
		..global("sliding_sum", &[0, 0, 0, 1])
	},
	Global {
		guard: same_length,
		..global("sort", &[1, 1])
	},
	Global {
		check: |a| a[0].var,
		..global("strict_lex2", &[2])
	},
	Global {
		check: |a| a[0].var,
		test: Some("gz_subcircuit({0})"),
		..global("subcircuit", &[1])
	},
	// The group is implied by the declarations (and the context)
	Global {
		kind: Kind::Redundant,
		template: Some("true"),
		..global("true", &[])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && a[2].var,
		// `t` must occur
		witness: |a| Some(format!("count({}, {})", a[2].text, a[1].text)),
		// Both values must be possible
		guard: |a| {
			Some(format!(
				"{s} in dom_array({x}) /\\ {t} in dom_array({x})",
				s = a[0].text,
				t = a[1].text,
				x = a[2].text
			))
		},
		test: Some(
			"forall (i in index_set({2})) ({2}[i] = {1} -> exists (j in index_set({2}) where j < i) ({2}[j] = {0}))",
		),
		..global("value_precede", &[0, 0, 1])
	},
];

/// Whether `name` passes the include/exclude substring filters
pub(super) fn selected(name: &str, include: &[String], exclude: &[String]) -> bool {
	(include.is_empty() || include.iter().any(|f| name.contains(f.as_str())))
		&& !exclude.iter().any(|f| name.contains(f.as_str()))
}

/// All candidate calls of the globals that `select` (name, kind) accepts that
/// can be built from `args`. A global with more than `max_args` argument
/// lists is skipped.
pub(super) fn candidates(
	args: &[Arg],
	select: impl Fn(&str, Kind) -> bool,
	max_args: usize,
) -> Vec<Candidate> {
	let blank_arg = Arg {
		text: String::new(),
		var: false,
		dims: 0,
		leaders: Vec::new(),
	};
	let mut result = Vec::new();
	for global in GLOBALS {
		if !select(global.name, global.kind) {
			continue;
		}
		let slots = global
			.signature
			.iter()
			.enumerate()
			.map(|(i, dims)| {
				let mut slot = args.iter().filter(|a| a.dims == *dims).collect::<Vec<_>>();
				if global.blank.is_some_and(|(b, _)| b == i) {
					slot.push(&blank_arg);
				}
				slot
			})
			.collect::<Vec<_>>();
		let size = slots.iter().map(Vec::len).product::<usize>();
		if size > max_args {
			log::info!(
				"globalizer: skipping {} ({size} argument lists)",
				global.name
			);
			continue;
		}
		let lists = if slots.is_empty() {
			vec![Vec::new()]
		} else {
			slots.into_iter().multi_cartesian_product().collect()
		};
		for chosen in lists {
			let blank = global
				.blank
				.filter(|(b, _)| std::ptr::eq(chosen[*b], &blank_arg));
			let given = chosen
				.iter()
				.filter(|a| !std::ptr::eq(**a, &blank_arg))
				.collect::<Vec<_>>();
			// Some argument must be a variable, and arguments must differ
			if !global.signature.is_empty() && !given.iter().any(|a| a.var)
				|| !given.iter().map(|a| &a.text).all_unique()
				|| !(global.check)(&chosen)
			{
				continue;
			}
			// The conditions cannot refer to an unfilled blank
			let condition = |c: Cond| if blank.is_some() { None } else { c(&chosen) };
			let mut leaders = chosen
				.iter()
				.flat_map(|a| a.leaders.iter().copied())
				.collect::<Vec<_>>();
			leaders.sort_unstable();
			leaders.dedup();
			result.push(Candidate {
				name: global.name,
				kind: global.kind,
				args: chosen.iter().map(|a| a.text.clone()).collect(),
				template: global.template,
				test: global.test,
				witness: condition(global.witness),
				guard: condition(global.guard),
				extra: condition(global.extra),
				dominated_by: global.dominated_by,
				blank,
				filled: None,
				leaders,
			});
		}
	}
	result
}

#[cfg(test)]
impl Candidate {
	/// A plain call `name(args)` for tests
	pub(super) fn raw(name: &'static str, args: &[&str]) -> Self {
		Candidate {
			name,
			kind: if name == "true" {
				Kind::Redundant
			} else {
				Kind::Normal
			},
			args: args.iter().map(|a| a.to_string()).collect(),
			template: None,
			test: None,
			guard: None,
			extra: None,
			blank: None,
			filled: None,
			leaders: Vec::new(),
			witness: None,
			dominated_by: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn arg(text: &str, var: bool, dims: usize) -> Arg {
		Arg {
			text: text.to_owned(),
			var,
			dims,
			leaders: Vec::new(),
		}
	}

	/// The candidates of the globals that pass the name filters
	fn filtered(
		args: &[Arg],
		include: &[&str],
		exclude: &[&str],
		max_args: usize,
	) -> Vec<Candidate> {
		let include = include.iter().map(|s| s.to_string()).collect::<Vec<_>>();
		let exclude = exclude.iter().map(|s| s.to_string()).collect::<Vec<_>>();
		candidates(args, |name, _| selected(name, &include, &exclude), max_args)
	}

	fn checked(args: &[Arg], name: &str) -> Vec<String> {
		filtered(args, &[name], &[], usize::MAX)
			.iter()
			.filter(|c| c.name == name)
			.map(Candidate::checked)
			.collect()
	}

	#[test]
	fn all_different_needs_a_var_array() {
		let args = [
			arg("x", true, 1),
			arg("p", false, 1),
			arg("y", true, 0),
			arg("m", true, 2),
		];
		assert_eq!(
			checked(&args, "all_different"),
			["(length(x) > 1) /\\ (all_different(x))"]
		);
	}

	#[test]
	fn increasing_and_decreasing_need_three_elements() {
		let args = [arg("x", true, 1), arg("p", false, 1)];
		assert_eq!(
			checked(&args, "increasing"),
			["(length(x) > 2) /\\ (increasing(x))"]
		);
		assert_eq!(
			checked(&args, "decreasing"),
			["(length(x) > 2) /\\ (decreasing(x))"]
		);
	}

	#[test]
	fn candidates_collect_leaders_of_their_arguments() {
		let mut row = arg("x[i, ..]", true, 1);
		row.leaders = vec![1, 0, 1];
		let found = filtered(&[row], &["all_different"], &[], usize::MAX);
		assert_eq!(found[0].leaders, [0, 1]);
	}

	#[test]
	fn filters_by_name() {
		let args = [arg("x", true, 1)];
		let names = |include: &[&str], exclude: &[&str]| {
			filtered(&args, include, exclude, usize::MAX)
				.into_iter()
				.map(|c| c.name)
				.collect::<Vec<_>>()
		};
		assert_eq!(names(&["all_diff"], &[]), ["all_different"]);
		assert!(names(&["lex2"], &[]).is_empty());
		assert!(!names(&[], &["all_diff"]).contains(&"all_different"));
		assert!(names(&[], &[]).contains(&"true"));
	}

	#[test]
	fn some_argument_must_be_a_variable() {
		let args = [arg("p", false, 1), arg("q", false, 1)];
		let found = filtered(&args, &[], &[], usize::MAX);
		assert_eq!(found.iter().map(|c| c.name).collect::<Vec<_>>(), ["true"]);
	}

	#[test]
	fn arguments_must_differ() {
		let args = [arg("x", true, 1)];
		assert!(checked(&args, "lex_lesseq").is_empty());
		let args = [arg("x", true, 1), arg("y", true, 1)];
		assert_eq!(
			checked(&args, "lex_lesseq"),
			[
				"if length(x) = length(y) /\\ length(x) > 1 then (lex_lesseq(x, y)) else false endif",
				"if length(y) = length(x) /\\ length(y) > 1 then (lex_lesseq(y, x)) else false endif",
			]
		);
	}

	#[test]
	fn guards_and_extras() {
		let args = [
			arg("0", false, 0),
			arg("u", false, 0),
			arg("b", false, 0),
			arg("x", true, 1),
		];
		assert!(checked(&args, "sliding_sum").contains(
			&"if 0 <= u /\\ 1 < b /\\ b <= length(x) then (sliding_sum(0, u, b, x)) else false endif"
				.to_owned()
		));
		let args = [arg("x", true, 1), arg("v", false, 0), arg("c", true, 0)];
		assert!(checked(&args, "count").contains(&"(c > 0) /\\ (count(x, v, c))".to_owned()));
		let args = [arg("x", true, 1)];
		assert_eq!(
			checked(&args, "alldifferent_except_0"),
			[
				"if length(x) > 2 /\\ 0 in dom_array(x) then (alldifferent_except_0(x)) else false endif"
			]
		);
	}

	#[test]
	fn signatures_of_every_global() {
		// One argument of every type, var and par
		let args = [
			arg("a", true, 0),
			arg("b", false, 0),
			arg("x", true, 1),
			arg("p", false, 1),
			arg("y", true, 1),
			arg("q", false, 1),
			arg("m", true, 2),
		];
		let mut calls = filtered(&args, &[], &[], usize::MAX)
			.into_iter()
			.map(|c| {
				if c.blank.is_some() {
					format!("{} (blank)", c.name)
				} else {
					c.call()
				}
			})
			.collect::<Vec<_>>();
		// The first argument list of each global
		calls.dedup_by(|a, b| a.split('(').next() == b.split('(').next());
		expect_test::expect![[r#"
    [
        "all_different(x)",
        "all_equal(x)",
        "alldifferent_except_0(x)",
        "bin_packing(b, x, p)",
        "bin_packing_capa(p, x, q)",
        "bin_packing_load(x, y, p)",
        "circuit(x)",
        "count(x, a, b)",
        "cumulative(x, p, q, a)",
        "decreasing(x)",
        "diffn(x, y, p, q)",
        "disjunctive(x, p)",
        "distribute(x, p, y)",
        "global_cardinality(x, set2array(index_set(p)), p)",
        "global_cardinality (blank)",
        "global_cardinality(y, set2array(index_set(x)), x)",
        "global_cardinality (blank)",
        "increasing(x)",
        "inverse(x, p)",
        "lex_less(x, p)",
        "lex_lesseq(x, p)",
        "lex2(m)",
        "maximum(a, x)",
        "maximum (blank)",
        "minimum(a, x)",
        "nvalue(a, x)",
        "sort(x, p)",
        "strict_lex2(m)",
        "subcircuit(x)",
        "true",
    ]
"#]]
		.assert_debug_eq(&calls);
	}

	#[test]
	fn max_args_skips_big_globals() {
		let args = (0..10)
			.map(|i| arg(&format!("x{i}"), true, 1))
			.collect::<Vec<_>>();
		let names = filtered(&args, &[], &[], 100)
			.into_iter()
			.map(|c| c.name)
			.unique()
			.collect::<Vec<_>>();
		// 10^2 lists for two arrays, 10^3 for three, 10^4 for four
		assert!(names.contains(&"inverse"));
		assert!(!names.contains(&"distribute"));
		assert!(!names.contains(&"diffn"));
	}

	#[test]
	fn blanks_are_filled_from_samples() {
		let args = [arg("x", true, 1)];
		let mut found = filtered(&args, &["global_card"], &[], usize::MAX);
		assert_eq!(found.len(), 1);
		let gcc = &mut found[0];
		assert_eq!(gcc.checked(), "true");
		assert_eq!(
			gcc.feature().unwrap(),
			"[lb_array(x)] ++ [count(x, v) | v in lb_array(x)..ub_array(x)]"
		);
		// Samples disagree
		assert!(!gcc.clone().fill(&[vec![1, 2, 0], vec![1, 1, 1]]));
		// Values from 0 get an explicit cover
		let mut from_zero = gcc.clone();
		assert!(from_zero.fill(&[vec![0, 2, 0, 1]]));
		assert_eq!(
			from_zero.call(),
			"global_cardinality(x, [0, 1, 2], [2, 0, 1])"
		);
		assert_eq!(from_zero.dominated_by, None);
		// Values that never occur at the ends are left out
		let mut trimmed = gcc.clone();
		assert!(trimmed.fill(&[vec![0, 0, 1, 1, 0]]));
		assert_eq!(trimmed.call(), "global_cardinality(x, [1, 2], [1, 1])");
		assert_eq!(trimmed.dominated_by, Some(("all_different", 0)));
		let mut ones = gcc.clone();
		assert!(ones.fill(&[vec![1, 1, 1]]));
		assert_eq!(ones.dominated_by, Some(("all_different", 0)));
		assert!(!gcc.clone().fill(&[vec![1, 0, 0]]));
		assert!(gcc.fill(&[vec![1, 2, 1], vec![1, 2, 1]]));
		assert!(gcc.filled == Some(1) && gcc.blank.is_none());
		assert_eq!(gcc.call(), "global_cardinality(x, [1, 2], [2, 1])");

		let mut max = filtered(&args, &["maximum"], &[], usize::MAX);
		assert_eq!(max[0].feature().unwrap(), "[max(x)]");
		assert!(max[0].fill(&[vec![7], vec![7]]));
		assert_eq!(max[0].call(), "maximum(7, x)");
	}
}
