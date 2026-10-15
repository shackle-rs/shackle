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

/// The element type of an argument (integers include enums)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Elem {
	Int,
	OptInt,
	Bool,
	Set,
}

/// The type of a parameter: number of array dimensions (0 for a scalar) and
/// element type
type Param = (usize, Elem);

const INT: Param = (0, Elem::Int);
const INTS: Param = (1, Elem::Int);
const INTS2: Param = (2, Elem::Int);
const INTS3: Param = (3, Elem::Int);
const OPT_INT: Param = (0, Elem::OptInt);
const OPT_INTS: Param = (1, Elem::OptInt);
const BOOL: Param = (0, Elem::Bool);
const BOOLS: Param = (1, Elem::Bool);
const BOOLS2: Param = (2, Elem::Bool);
const SET: Param = (0, Elem::Set);
const SETS: Param = (1, Elem::Set);

/// A possible argument for a candidate global
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Arg {
	/// MiniZinc text of the argument
	pub(super) text: String,
	/// Whether the argument contains decision variables
	pub(super) var: bool,
	/// Number of array dimensions (0 for a scalar)
	pub(super) dims: usize,
	pub(super) elem: Elem,
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
	/// The largest number of true values of the Boolean argument in the
	/// given slot over all samples
	MostTrue(usize),
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

/// `global_cardinality_closed` with an explicit cover (`{2}`), for filled
/// counts: they count every value of the array
const COVERED_GCC: &str = "global_cardinality_closed({0}, {2}) = {1}";
const COVERED_GCC_TEST: &str = "global_cardinality_closed({0}, {2}, {1})";

impl Candidate {
	/// Fill in the arguments (`{i}`) and a loop variable (`{v}`) that no
	/// argument uses
	fn instantiate(&self, template: &str) -> String {
		let used = |name: &str| {
			self.args.iter().any(|a| {
				a.split(|c: char| !c.is_alphanumeric() && c != '_')
					.any(|w| w == name)
			})
		};
		let v = ["i", "j", "k", "l"]
			.into_iter()
			.map(str::to_owned)
			.chain((1..).map(|n| format!("i{n}")))
			.find(|v| !used(v))
			.unwrap();
		self.args
			.iter()
			.enumerate()
			.fold(template.replace("{v}", &v), |s, (i, a)| {
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
			Blank::MostTrue(i) => format!("[sum({})]", self.args[i]),
			Blank::Counts(i) => {
				let x = &self.args[i];
				format!("[lb_array({x})] ++ [count({x}, v) | v in lb_array({x})..ub_array({x})]")
			}
		})
	}

	/// Fill the blank from the feature values of all samples. Fails if the
	/// samples do not agree (except for the largest number of true values).
	pub(super) fn fill(&mut self, values: &[Vec<i64>]) -> bool {
		let Some((slot, blank)) = self.blank else {
			return true;
		};
		let Some(first) = values.first() else {
			return false;
		};
		if let Blank::MostTrue(_) = blank {
			let Some(most) = values.iter().filter_map(|v| v.first()).max() else {
				return false;
			};
			self.args[slot] = most.to_string();
			self.blank = None;
			self.filled = Some(slot);
			return true;
		}
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
				self.test = Some(COVERED_GCC_TEST);
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
	/// The function that a group calls if it already is this global
	/// (`element` for an access with a variable index)
	uses: &'static str,
	kind: Kind,
	/// The type of each argument
	signature: &'static [Param],
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
predicate gz_value_precede(int: s, int: t, array [int] of var int: x) =
  forall (i in index_set(x)) (
    x[i] = t -> exists (j in index_set(x) where j < i) (x[j] = s));
predicate gz_sliding_among(int: lo, int: hi, int: n, array [int] of var int: x, set of int: v) =
  forall (i in min(index_set(x))..max(index_set(x)) - n + 1) (
    lo <= sum (j in i..i + n - 1) (x[j] in v) /\ sum (j in i..i + n - 1) (x[j] in v) <= hi);
"#;

const fn global(name: &'static str, signature: &'static [Param]) -> Global {
	Global {
		name,
		uses: name,
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
		..global(name, &[INTS3])
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
		..global("all_different", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_equal", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		// 0 must be possible, else this is all_different
		guard: |a| {
			let x = &a[0].text;
			Some(format!("length({x}) > 2 /\\ 0 in dom_array({x})"))
		},
		dominated_by: Some(("all_different", 0)),
		..global("alldifferent_except_0", &[INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: |a| Some(format!("length({}) > 1", a[1].text)),
		template: Some("arg_max({1}) = {0}"),
		..global("arg_max", &[INT, INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: |a| Some(format!("length({}) > 1", a[1].text)),
		template: Some("arg_min({1}) = {0}"),
		..global("arg_min", &[INT, INTS])
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
		..global("bin_packing", &[INT, INTS, INTS])
	},
	Global {
		check: |a| !a[0].var && a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		..global("bin_packing_capa", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var && a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		template: Some("bin_packing_load({1}, {2}) = {0}"),
		test: Some("bin_packing_load({0}, {1}, {2})"),
		..global("bin_packing_load", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		test: Some("gz_circuit({0})"),
		..global("circuit", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[2].text)),
		template: Some("count({v} in index_set({0})) ({0}[{v}] = {1}) = {2}"),
		uses: "count",
		..global("count", &[INTS, INT, INT])
	},
	// The other comparisons, where they are not trivially true
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} < length({})", a[2].text, a[0].text)),
		template: Some("count({v} in index_set({0})) ({0}[{v}] = {1}) <= {2}"),
		uses: "count",
		..global("count_leq", &[INTS, INT, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} <= length({})", a[2].text, a[0].text)),
		template: Some("count({v} in index_set({0})) ({0}[{v}] = {1}) < {2}"),
		uses: "count",
		..global("count_lt", &[INTS, INT, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[2].text)),
		template: Some("count({v} in index_set({0})) ({0}[{v}] = {1}) >= {2}"),
		uses: "count",
		..global("count_geq", &[INTS, INT, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} >= 0", a[2].text)),
		template: Some("count({v} in index_set({0})) ({0}[{v}] = {1}) > {2}"),
		uses: "count",
		..global("count_gt", &[INTS, INT, INT])
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
		..global("cumulative", &[INTS, INTS, INTS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("decreasing", &[INTS])
	},
	Global {
		check: |a| (a[0].var && a[1].var) || (a[2].var && a[3].var),
		guard: same_length,
		..global("diffn", &[INTS, INTS, INTS, INTS])
	},
	Global {
		guard: same_length,
		..global("disjunctive", &[INTS, INTS])
	},
	// `x[i]` with a variable index
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[1].text)),
		template: Some("{1}[{0}] = {2}"),
		..global("element", &[INT, INTS, INT])
	},
	Global {
		guard: |a| same_length(&a[..2]),
		template: Some("distribute({1}, {2}) = {0}"),
		test: Some("distribute({0}, {1}, {2})"),
		..global("distribute", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		blank: Some((1, Blank::Counts(0))),
		template: Some("global_cardinality({0}, set2array(index_set({1}))) = {1}"),
		test: Some("global_cardinality({0}, set2array(index_set({1})), {1})"),
		dominated_by: Some(("global_cardinality_closed", 0)),
		..global("global_cardinality", &[INTS, INTS])
	},
	// No other values occur
	Global {
		check: |a| a[0].var,
		template: Some("global_cardinality_closed({0}, set2array(index_set({1}))) = {1}"),
		test: Some("global_cardinality_closed({0}, set2array(index_set({1})), {1})"),
		..global("global_cardinality_closed", &[INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("increasing", &[INTS])
	},
	Global {
		guard: same_length,
		template: Some("inverse({0}) = {1}"),
		test: Some("inverse({0}, {1})"),
		..global("inverse", &[INTS, INTS])
	},
	Global {
		guard: lex_guard,
		..global("lex_less", &[INTS, INTS])
	},
	Global {
		guard: lex_guard,
		..global("lex_lesseq", &[INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		..global("lex2", &[INTS2])
	},
	Global {
		check: |a| a[0].var,
		..global("lex_chain_less", &[INTS2])
	},
	Global {
		check: |a| a[0].var,
		..global("lex_chain_lesseq", &[INTS2])
	},
	Global {
		blank: Some((0, Blank::Max(1))),
		template: Some("max({1}) = {0}"),
		uses: "max",
		..global("maximum", &[INT, INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("member", &[INTS, INT])
	},
	Global {
		template: Some("min({1}) = {0}"),
		uses: "min",
		..global("minimum", &[INT, INTS])
	},
	Global {
		extra: |a| Some(format!("{} <= length({})", a[0].text, a[1].text)),
		dominated_by: Some(("all_different", 1)),
		template: Some("nvalue({1}) = {0}"),
		test: Some("nvalue({0}, {1})"),
		..global("nvalue", &[INT, INTS])
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
		..global("sliding_sum", &[INT, INT, INT, INTS])
	},
	Global {
		guard: same_length,
		template: Some("sort({0}) = {1}"),
		test: Some("sort({0}, {1})"),
		..global("sort", &[INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		..global("lex2_strict", &[INTS2])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("strictly_decreasing", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("strictly_increasing", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		test: Some("gz_subcircuit({0})"),
		// Both hold if every permutation has at most one cycle (e.g. 3 values)
		dominated_by: Some(("all_different", 0)),
		..global("subcircuit", &[INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("symmetric_all_different", &[INTS])
	},
	// The first index of `v` in `x` (`v` must occur)
	Global {
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		template: Some("arg_val({0}, {1}) = {2}"),
		// The function forces `v` to occur in a reified context too
		test: Some(
			"{2} in index_set({0}) /\\ {0}[{2}] = {1} /\\ forall ({v} in index_set({0}) where {v} < {2}) ({0}[{v}] != {1})",
		),
		uses: "arg_val",
		..global("arg_val", &[INTS, INT, INT])
	},
	// The first index of `v` in `x`, absent if `v` does not occur
	Global {
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		template: Some("arg_val_weak({0}, {1}) = {2}"),
		..global("arg_val_weak", &[INTS, INT, OPT_INT])
	},
	Global {
		check: |a| a[1].var,
		guard: same_length,
		template: Some("arg_sort({0}) = {1}"),
		test: Some("arg_sort({0}, {1})"),
		..global("arg_sort", &[INTS, INTS])
	},
	Global {
		check: |a| (a[0].var && a[1].var) || (a[2].var && a[3].var),
		guard: same_length,
		..global("diffn_nonstrict", &[INTS, INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		guard: |a| {
			Some(format!(
				"index_set_1of2({0}) = index_set_1of2({1}) /\\ index_set_2of2({0}) = index_set_2of2({1})",
				a[0].text, a[1].text
			))
		},
		..global("diffn_k", &[INTS2, INTS2])
	},
	Global {
		check: |a| a[0].var,
		guard: |a| {
			Some(format!(
				"index_set_1of2({0}) = index_set_1of2({1}) /\\ index_set_2of2({0}) = index_set_2of2({1})",
				a[0].text, a[1].text
			))
		},
		..global("diffn_nonstrict_k", &[INTS2, INTS2])
	},
	Global {
		guard: same_length,
		..global("disjunctive_strict", &[INTS, INTS])
	},
	// Counts between bounds
	Global {
		check: |a| a[0].var && !a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		template: Some("global_cardinality({0}, set2array(index_set({1})), {1}, {2})"),
		uses: "global_cardinality",
		dominated_by: Some(("global_cardinality_closed", 0)),
		..global("global_cardinality_bounds", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var && !a[1].var && !a[2].var,
		guard: |a| Some(same_index_set(a[1], a[2])),
		template: Some("global_cardinality_closed({0}, set2array(index_set({1})), {1}, {2})"),
		uses: "global_cardinality_closed",
		..global("global_cardinality_closed_bounds", &[INTS, INTS, INTS])
	},
	Global {
		guard: same_length,
		..global("inverse_in_range", &[INTS, INTS])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && a[2].var,
		guard: |a| {
			Some(format!(
				"{} /\\ {}",
				same_index_set(a[0], a[2]),
				same_index_set(a[1], a[2])
			))
		},
		..global("knapsack", &[INTS, INTS, INTS, INT, INT])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && !a[2].var && a[3].var && !a[4].var,
		guard: |a| {
			Some(format!(
				"{l} <= {u} /\\ 1 < {n} /\\ {n} <= length({vs}) /\\ card(dom_array({vs})) > 2",
				l = a[0].text,
				u = a[1].text,
				n = a[2].text,
				vs = a[3].text
			))
		},
		test: Some("gz_sliding_among({0}, {1}, {2}, {3}, {4})"),
		..global("sliding_among", &[INT, INT, INT, INTS, SET])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		guard: |a| {
			Some(format!(
				"index_set_2of2({}) = index_set({})",
				a[1].text, a[0].text
			))
		},
		..global("table", &[INTS, INTS2])
	},
	// Boolean arguments
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_equal", &[BOOLS])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		guard: |a| {
			Some(format!(
				"index_set_2of2({}) = index_set({})",
				a[1].text, a[0].text
			))
		},
		..global("table", &[BOOLS, BOOLS2])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[1].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}]) = {1}"),
		uses: "count",
		..global("count", &[BOOLS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} < length({})", a[1].text, a[0].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}]) <= {1}"),
		uses: "count",
		// At most k (e.g. at most one) from the samples
		blank: Some((1, Blank::MostTrue(0))),
		..global("count_leq", &[BOOLS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} <= length({})", a[1].text, a[0].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}]) < {1}"),
		uses: "count",
		..global("count_lt", &[BOOLS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[1].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}]) >= {1}"),
		uses: "count",
		..global("count_geq", &[BOOLS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} >= 0", a[1].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}]) > {1}"),
		uses: "count",
		..global("count_gt", &[BOOLS, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[1].text)),
		template: Some("{1}[{0}] = {2}"),
		..global("element", &[INT, BOOLS, BOOL])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("decreasing", &[BOOLS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		..global("increasing", &[BOOLS])
	},
	Global {
		guard: lex_guard,
		..global("lex_less", &[BOOLS, BOOLS])
	},
	Global {
		guard: lex_guard,
		..global("lex_lesseq", &[BOOLS, BOOLS])
	},
	Global {
		check: |a| a[0].var,
		..global("lex_chain_less", &[BOOLS2])
	},
	Global {
		check: |a| a[0].var,
		..global("lex_chain_lesseq", &[BOOLS2])
	},
	// Set arguments
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_equal", &[SETS])
	},
	Global {
		..global("inverse_set", &[SETS, SETS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_different", &[SETS])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		extra: |a| Some(format!("length({}) > 2", a[0].text)),
		dominated_by: Some(("all_different", 0)),
		..global("all_different_except", &[INTS, SET])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_disjoint", &[SETS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[2].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}] in {1}) = {2}"),
		uses: "count",
		// For a 0/1 array this is a count of one value
		guard: |a| Some(format!("card(dom_array({})) > 2", a[0].text)),
		..global("among", &[INTS, SET, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} < length({})", a[2].text, a[0].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}] in {1}) <= {2}"),
		uses: "count",
		// For a 0/1 array this is a count of one value
		guard: |a| Some(format!("card(dom_array({})) > 2", a[0].text)),
		..global("among_leq", &[INTS, SET, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} <= length({})", a[2].text, a[0].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}] in {1}) < {2}"),
		uses: "count",
		// For a 0/1 array this is a count of one value
		guard: |a| Some(format!("card(dom_array({})) > 2", a[0].text)),
		..global("among_lt", &[INTS, SET, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} > 0", a[2].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}] in {1}) >= {2}"),
		uses: "count",
		// For a 0/1 array this is a count of one value
		guard: |a| Some(format!("card(dom_array({})) > 2", a[0].text)),
		..global("among_geq", &[INTS, SET, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("{} >= 0", a[2].text)),
		template: Some("count ({v} in index_set({0})) ({0}[{v}] in {1}) > {2}"),
		uses: "count",
		// For a 0/1 array this is a count of one value
		guard: |a| Some(format!("card(dom_array({})) > 2", a[0].text)),
		..global("among_gt", &[INTS, SET, INT])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("at_most1", &[SETS])
	},
	Global {
		..global("disjoint", &[SET, SET])
	},
	Global {
		..global("int_set_channel", &[INTS, SETS])
	},
	Global {
		..global("link_set_to_booleans", &[SET, BOOLS])
	},
	Global {
		check: |a| a[0].var,
		..global("partition_set", &[SETS, SET])
	},
	Global {
		template: Some("range({0}, {1}) = {2}"),
		test: Some("range({0}, {1}, {2})"),
		..global("range", &[INTS, SET, SET])
	},
	Global {
		template: Some("roots({0}, {2}) = {1}"),
		test: Some("roots({0}, {1}, {2})"),
		..global("roots", &[INTS, SET, SET])
	},
	// Opt arguments (absent values are left out)
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		..global("all_different", &[OPT_INTS])
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
		..global("cumulative", &[OPT_INTS, INTS, INTS, INT])
	},
	Global {
		guard: same_length,
		..global("disjunctive", &[OPT_INTS, INTS])
	},
	Global {
		guard: same_length,
		..global("disjunctive_strict", &[OPT_INTS, INTS])
	},
	// The optional task is scheduled exactly when one of the tasks is
	Global {
		check: |a| a[0].var,
		guard: |a| same_length(&a[2..]),
		..global("alternative", &[OPT_INT, INT, OPT_INTS, INTS])
	},
	// The optional task spans the tasks that are scheduled
	Global {
		check: |a| a[0].var,
		guard: |a| same_length(&a[2..]),
		..global("span", &[OPT_INT, INT, OPT_INTS, INTS])
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
		test: Some("gz_value_precede({0}, {1}, {2})"),
		..global("value_precede", &[INT, INT, INTS])
	},
	Global {
		check: |a| !a[0].var && a[1].var,
		guard: |a| Some(format!("length({}) > 1", a[0].text)),
		test: Some(
			"forall ({v} in index_set({0}) where {v} < max(index_set({0}))) (gz_value_precede({0}[{v}], {0}[{v} + 1], {1}))",
		),
		..global("value_precede_chain", &[INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: |a| Some(format!("length({}) > 1", a[0].text)),
		test: Some("forall ({v} in 1..ub_array({0}) - 1) (gz_value_precede({v}, {v} + 1, {0}))"),
		..global("seq_precede_chain", &[INTS])
	},
];

/// Whether `name` passes the include/exclude substring filters
pub(super) fn selected(name: &str, include: &[String], exclude: &[String]) -> bool {
	(include.is_empty() || include.iter().any(|f| name.contains(f.as_str())))
		&& !exclude.iter().any(|f| name.contains(f.as_str()))
}

/// All candidate calls of the globals that `select` (name, the function it
/// uses, kind) accepts that can be built from `args`. A global with more than
/// `max_args` argument lists is skipped.
pub(super) fn candidates(
	args: &[Arg],
	select: impl Fn(&str, &str, Kind) -> bool,
	max_args: usize,
) -> Vec<Candidate> {
	let blank_arg = Arg {
		text: String::new(),
		var: false,
		dims: 0,
		elem: Elem::Int,
		leaders: Vec::new(),
	};
	let mut result = Vec::new();
	for global in GLOBALS {
		if !select(global.name, global.uses, global.kind) {
			continue;
		}
		let slots = global
			.signature
			.iter()
			.enumerate()
			.map(|(i, (dims, elem))| {
				let mut slot = args
					.iter()
					.filter(|a| a.dims == *dims && a.elem == *elem)
					.collect::<Vec<_>>();
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
		typed(text, var, dims, Elem::Int)
	}

	fn typed(text: &str, var: bool, dims: usize, elem: Elem) -> Arg {
		Arg {
			text: text.to_owned(),
			var,
			dims,
			elem,
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
		candidates(
			args,
			|name, _, _| selected(name, &include, &exclude),
			max_args,
		)
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
		assert_eq!(
			names(&["all_diff"], &[]),
			["all_different", "symmetric_all_different"]
		);
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
		assert!(
			checked(&args, "count")
				.contains(&"(c > 0) /\\ (count(i in index_set(x)) (x[i] = v) = c)".to_owned())
		);
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
			typed("o", true, 1, Elem::OptInt),
			typed("t", true, 0, Elem::Bool),
			typed("bs", true, 1, Elem::Bool),
			typed("bm", true, 2, Elem::Bool),
			typed("s", true, 0, Elem::Set),
			typed("ss", true, 1, Elem::Set),
		];
		// The first argument list of each global
		let calls = filtered(&args, &[], &[], usize::MAX)
			.into_iter()
			.dedup_by(|a, b| a.name == b.name && a.blank.is_some() == b.blank.is_some())
			.map(|c| {
				if c.blank.is_some() {
					format!("{} (blank)", c.name)
				} else {
					c.call()
				}
			})
			.collect::<Vec<_>>();
		expect_test::expect![[r#"
    [
        "all_different(x)",
        "all_equal(x)",
        "alldifferent_except_0(x)",
        "arg_max(x) = a",
        "arg_min(x) = a",
        "bin_packing(b, x, p)",
        "bin_packing_capa(p, x, q)",
        "bin_packing_load(y, p) = x",
        "circuit(x)",
        "count(i in index_set(x)) (x[i] = a) = b",
        "count(i in index_set(x)) (x[i] = a) <= b",
        "count(i in index_set(x)) (x[i] = a) < b",
        "count(i in index_set(x)) (x[i] = a) >= b",
        "count(i in index_set(x)) (x[i] = a) > b",
        "cumulative(x, p, q, a)",
        "decreasing(x)",
        "diffn(x, y, p, q)",
        "disjunctive(x, p)",
        "x[a] = b",
        "distribute(p, y) = x",
        "global_cardinality(x, set2array(index_set(p))) = p",
        "global_cardinality (blank)",
        "global_cardinality(y, set2array(index_set(x))) = x",
        "global_cardinality (blank)",
        "global_cardinality_closed(x, set2array(index_set(p))) = p",
        "increasing(x)",
        "inverse(x) = p",
        "lex_less(x, p)",
        "lex_lesseq(x, p)",
        "lex2(m)",
        "lex_chain_less(m)",
        "lex_chain_lesseq(m)",
        "max(x) = a",
        "maximum (blank)",
        "member(x, a)",
        "min(x) = a",
        "nvalue(x) = a",
        "sort(x) = p",
        "lex2_strict(m)",
        "strictly_decreasing(x)",
        "strictly_increasing(x)",
        "subcircuit(x)",
        "symmetric_all_different(x)",
        "arg_val(x, a) = b",
        "arg_sort(x) = y",
        "diffn_nonstrict(x, y, p, q)",
        "disjunctive_strict(x, p)",
        "global_cardinality(x, set2array(index_set(p)), p, q)",
        "global_cardinality_closed(x, set2array(index_set(p)), p, q)",
        "inverse_in_range(x, p)",
        "knapsack(p, q, x, a, b)",
        "all_equal(bs)",
        "count (i in index_set(bs)) (bs[i]) = a",
        "count (i in index_set(bs)) (bs[i]) <= a",
        "count_leq (blank)",
        "count (i in index_set(bs)) (bs[i]) < a",
        "count (i in index_set(bs)) (bs[i]) >= a",
        "count (i in index_set(bs)) (bs[i]) > a",
        "bs[a] = t",
        "decreasing(bs)",
        "increasing(bs)",
        "lex_chain_less(bm)",
        "lex_chain_lesseq(bm)",
        "all_equal(ss)",
        "all_different(ss)",
        "all_disjoint(ss)",
        "count (i in index_set(x)) (x[i] in s) = a",
        "count (i in index_set(x)) (x[i] in s) <= a",
        "count (i in index_set(x)) (x[i] in s) < a",
        "count (i in index_set(x)) (x[i] in s) >= a",
        "count (i in index_set(x)) (x[i] in s) > a",
        "at_most1(ss)",
        "int_set_channel(x, ss)",
        "link_set_to_booleans(s, bs)",
        "partition_set(ss, s)",
        "all_different(o)",
        "cumulative(o, p, q, a)",
        "disjunctive(o, x)",
        "disjunctive_strict(o, x)",
        "true",
        "value_precede_chain(p, x)",
        "seq_precede_chain(x)",
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
			"global_cardinality_closed(x, [0, 1, 2]) = [2, 0, 1]"
		);
		assert_eq!(
			from_zero.dominated_by,
			Some(("global_cardinality_closed", 0))
		);
		assert_eq!(
			from_zero.checked(),
			"(global_cardinality_closed(x, [0, 1, 2], [2, 0, 1]))"
		);
		// Values that never occur at the ends are left out
		let mut trimmed = gcc.clone();
		assert!(trimmed.fill(&[vec![0, 0, 1, 1, 0]]));
		assert_eq!(
			trimmed.call(),
			"global_cardinality_closed(x, [1, 2]) = [1, 1]"
		);
		assert_eq!(trimmed.dominated_by, Some(("all_different", 0)));
		let mut ones = gcc.clone();
		assert!(ones.fill(&[vec![1, 1, 1]]));
		assert_eq!(ones.dominated_by, Some(("all_different", 0)));
		assert!(!gcc.clone().fill(&[vec![1, 0, 0]]));
		assert!(gcc.fill(&[vec![1, 2, 1], vec![1, 2, 1]]));
		assert!(gcc.filled == Some(1) && gcc.blank.is_none());
		assert_eq!(gcc.call(), "global_cardinality_closed(x, [1, 2]) = [2, 1]");

		let mut max = filtered(&args, &["maximum"], &[], usize::MAX);
		assert_eq!(max[0].feature().unwrap(), "[max(x)]");
		assert!(max[0].fill(&[vec![7], vec![7]]));
		assert_eq!(max[0].call(), "max(x) = 7");
	}
}
