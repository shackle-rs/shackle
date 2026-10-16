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

/// Fill in the arguments (`{i}`) and fresh loop variable names (`{v}`, `{w}`,
/// `{u}`, distinct and not used by any argument), in one pass (so an argument
/// text is never rewritten)
fn instantiate(template: &str, args: &[impl AsRef<str>]) -> String {
	let used = |name: &str| {
		args.iter().any(|a| {
			a.as_ref()
				.split(|c: char| !c.is_alphanumeric() && c != '_')
				.any(|w| w == name)
		})
	};
	let mut fresh = ["i", "j", "k", "l"]
		.into_iter()
		.map(str::to_owned)
		.chain((1..).map(|n| format!("i{n}")))
		.filter(|v| !used(v));
	let loops: [String; 3] = std::array::from_fn(|_| fresh.next().unwrap());
	let mut result = String::with_capacity(template.len());
	let mut rest = template;
	while let Some(open) = rest.find('{') {
		result.push_str(&rest[..open]);
		rest = &rest[open..];
		let Some(close) = rest.find('}') else {
			break;
		};
		match &rest[1..close] {
			"v" => result.push_str(&loops[0]),
			"w" => result.push_str(&loops[1]),
			"u" => result.push_str(&loops[2]),
			i => match i.parse::<usize>().ok().and_then(|i| args.get(i)) {
				Some(arg) => result.push_str(arg.as_ref()),
				None => result.push_str(&rest[..=close]),
			},
		}
		rest = &rest[close + 1..];
	}
	result.push_str(rest);
	result
}

impl Candidate {
	fn instantiate(&self, template: &str) -> String {
		instantiate(template, &self.args)
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
			Blank::MostTrue(i) => format!("[sum({x}), length({x})]", x = self.args[i]),
			// At most 101 values, so the suggested arrays stay readable (else
			// nothing, and the blank is not filled)
			Blank::Counts(i) => instantiate(
				"if ub_array({0}) - lb_array({0}) <= 100 then [lb_array({0})] ++ [count({0}, {v}) | {v} in lb_array({0})..ub_array({0})] else [] endif",
				&[&self.args[i]],
			),
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
		let agree = values.iter().all(|v| v == first);
		self.args[slot] = match (blank, &first[..]) {
			// Fewer than the number of elements, else the bound is trivially true
			(Blank::MostTrue(_), [_, length]) => {
				if values.iter().any(|v| v.get(1) != Some(length)) {
					return false;
				}
				let most = values.iter().map(|v| v[0]).max().unwrap();
				if most >= *length {
					return false;
				}
				most.to_string()
			}
			(Blank::Max(_), [m]) if agree => m.to_string(),
			(Blank::Counts(source), [lo, counts @ ..]) if agree => {
				// Values that never occur at the ends are left out (the reverse
				// check shows whether that is still equivalent)
				let Some(start) = counts.iter().position(|c| *c != 0) else {
					return false;
				};
				let end = counts.iter().rposition(|c| *c != 0).unwrap() + 1;
				let (lo, counts) = (lo + start as i64, &counts[start..end]);
				if counts.iter().all(|c| *c == 1) {
					self.dominated_by = Some(("all_different", source));
				}
				let hi = lo + counts.len() as i64 - 1;
				self.args.push(format!("[{}]", (lo..=hi).join(", ")));
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
	/// Par condition (with `{i}` for argument `i`), see [`Candidate::guard`]
	guard: Option<&'static str>,
	/// Sample condition, see [`Candidate::extra`]
	extra: Option<&'static str>,
	/// Slot that may be computed from the samples instead
	blank: Option<(usize, Blank)>,
	/// Call text with `{i}` for argument `i` and `{v}`, `{w}`, `{u}` for loop
	/// variables (see `instantiate`; default: `name(args)`)
	template: Option<&'static str>,
	/// Expression checked on the samples instead of the call (e.g. the
	/// predicate form of a functional template, or a global that cannot be
	/// reified)
	test: Option<&'static str>,
	/// Integer that must be positive on some sample, else the global is
	/// trivially true there
	witness: Option<&'static str>,
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
		guard: None,
		extra: None,
		blank: None,
		template: None,
		test: None,
		witness: None,
		dominated_by: None,
	}
}

/// The 3D 0/1 argument has exactly one 1 along `axis` (`sums` checks the sum
/// along that axis)
const fn binaries(name: &'static str, axis: usize, sums: &'static str) -> Global {
	Global {
		kind: Kind::Viewpoint(axis),
		check: |a| a[0].var,
		guard: Some("dom_array({0}) subset 0..1"),
		template: Some(sums),
		..global(name, &[INTS3])
	}
}

/// A global whose first argument must be a variable
const fn on_vars(name: &'static str, signature: &'static [Param]) -> Global {
	Global {
		check: |a| a[0].var,
		..global(name, signature)
	}
}

/// A comparison of a count over the first argument (written as
/// `count (i in index_set(x)) (...)`)
const fn counting(
	name: &'static str,
	signature: &'static [Param],
	template: &'static str,
	extra: &'static str,
) -> Global {
	Global {
		extra: Some(extra),
		template: Some(template),
		uses: "count",
		..on_vars(name, signature)
	}
}

const LONGER_THAN_1: &str = "length({0}) > 1";
const LONGER_THAN_2: &str = "length({0}) > 2";
const SAME_LENGTH: &str = "length({0}) = length({1})";
const LEX: &str = "length({0}) = length({1}) /\\ length({0}) > 1";
/// The arrays have the same index sets (the globals assert this)
const SAME_INDEX: &str = "index_sets_agree({0}, {1})";
const SAME_INDEX_12: &str = "index_sets_agree({1}, {2})";
const SAME_INDEX_23: &str = "index_sets_agree({2}, {3})";
const SAME_INDEX_4: &str =
	"index_sets_agree({0}, {1}) /\\ index_sets_agree({0}, {2}) /\\ index_sets_agree({0}, {3})";
const CUMULATIVE: Global = Global {
	check: |a| !a[1].var && !a[2].var,
	guard: Some(
		"index_sets_agree({0}, {1}) /\\ index_sets_agree({0}, {2}) /\\ min({1}) > 0 /\\ min({2}) > 0",
	),
	extra: Some("length({0}) > 1 /\\ {3} > 0"),
	..global("cumulative", &[INTS, INTS, INTS, INT])
};
/// The rows of the par table have one value per variable
const TABLE: &str = "index_set_2of2({1}) = index_set({0})";
/// More than one column to order
const LEX_CHAIN: &str = "card(index_set_2of2({0})) > 1";
/// For a 0/1 array this is a count of one value
const AMONG: &str = "card(dom_array({0})) > 2";

const GLOBALS: &[Global] = &[
	// A 0/1 array whose sums along one axis are 1 represents a 2D integer
	// array (used by the initial pass to add a viewpoint)
	binaries(
		"binaries_represent_int_3A",
		0,
		"forall ({w} in index_set_2of3({0}), {u} in index_set_3of3({0})) (sum ({v} in index_set_1of3({0})) ({0}[{v}, {w}, {u}]) = 1)",
	),
	binaries(
		"binaries_represent_int_3B",
		1,
		"forall ({v} in index_set_1of3({0}), {u} in index_set_3of3({0})) (sum ({w} in index_set_2of3({0})) ({0}[{v}, {w}, {u}]) = 1)",
	),
	binaries(
		"binaries_represent_int_3C",
		2,
		"forall ({v} in index_set_1of3({0}), {w} in index_set_2of3({0})) (sum ({u} in index_set_3of3({0})) ({0}[{v}, {w}, {u}]) = 1)",
	),
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_different", &[INTS])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_equal", &[INTS])
	},
	Global {
		// 0 must be possible, else this is all_different
		guard: Some("length({0}) > 2 /\\ 0 in dom_array({0})"),
		dominated_by: Some(("all_different", 0)),
		..on_vars("alldifferent_except_0", &[INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: Some("length({1}) > 1"),
		template: Some("arg_max({1}) = {0}"),
		..global("arg_max", &[INT, INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: Some("length({1}) > 1"),
		template: Some("arg_min({1}) = {0}"),
		..global("arg_min", &[INT, INTS])
	},
	Global {
		check: |a| a[1].var,
		guard: Some(SAME_LENGTH),
		template: Some("arg_sort({0}) = {1}"),
		test: Some("arg_sort({0}, {1})"),
		..global("arg_sort", &[INTS, INTS])
	},
	// The first index of `v` in `x` (`v` must occur)
	Global {
		extra: Some(LONGER_THAN_1),
		template: Some("arg_val({0}, {1}) = {2}"),
		// The function forces `v` to occur in a reified context too
		test: Some(
			"{2} in index_set({0}) /\\ {0}[{2}] = {1} /\\ forall ({v} in index_set({0}) where {v} < {2}) ({0}[{v}] != {1})",
		),
		..global("arg_val", &[INTS, INT, INT])
	},
	// The first index of `v` in `x`, absent if `v` does not occur
	Global {
		extra: Some(LONGER_THAN_1),
		template: Some("arg_val_weak({0}, {1}) = {2}"),
		..global("arg_val_weak", &[INTS, INT, OPT_INT])
	},
	Global {
		check: |a| !a[0].var && a[1].var && !a[2].var,
		guard: Some("index_sets_agree({1}, {2}) /\\ ub_array({1}) - lb_array({1}) <= 20"),
		..global("bin_packing", &[INT, INTS, INTS])
	},
	Global {
		check: |a| !a[0].var && a[1].var && !a[2].var,
		guard: Some(SAME_INDEX_12),
		..global("bin_packing_capa", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var && a[1].var && !a[2].var,
		guard: Some(SAME_INDEX_12),
		template: Some("bin_packing_load({1}, {2}) = {0}"),
		test: Some("bin_packing_load({0}, {1}, {2})"),
		..global("bin_packing_load", &[INTS, INTS, INTS])
	},
	Global {
		test: Some("gz_circuit({0})"),
		..on_vars("circuit", &[INTS])
	},
	// Comparisons of a count, where they are not trivially true
	counting(
		"count",
		&[INTS, INT, INT],
		"count ({v} in index_set({0})) ({0}[{v}] = {1}) = {2}",
		"{2} > 0",
	),
	counting(
		"count_leq",
		&[INTS, INT, INT],
		"count ({v} in index_set({0})) ({0}[{v}] = {1}) <= {2}",
		"{2} < length({0})",
	),
	counting(
		"count_lt",
		&[INTS, INT, INT],
		"count ({v} in index_set({0})) ({0}[{v}] = {1}) < {2}",
		"{2} <= length({0})",
	),
	counting(
		"count_geq",
		&[INTS, INT, INT],
		"count ({v} in index_set({0})) ({0}[{v}] = {1}) >= {2}",
		"{2} > 0",
	),
	counting(
		"count_gt",
		&[INTS, INT, INT],
		"count ({v} in index_set({0})) ({0}[{v}] = {1}) > {2}",
		"{2} >= 0",
	),
	CUMULATIVE,
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("decreasing", &[INTS])
	},
	Global {
		check: |a| (a[0].var && a[1].var) || (a[2].var && a[3].var),
		guard: Some(SAME_INDEX_4),
		..global("diffn", &[INTS, INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		guard: Some(SAME_INDEX),
		..global("diffn_k", &[INTS2, INTS2])
	},
	Global {
		check: |a| (a[0].var && a[1].var) || (a[2].var && a[3].var),
		guard: Some(SAME_INDEX_4),
		..global("diffn_nonstrict", &[INTS, INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var,
		guard: Some(SAME_INDEX),
		..global("diffn_nonstrict_k", &[INTS2, INTS2])
	},
	Global {
		guard: Some(SAME_INDEX),
		..global("disjunctive", &[INTS, INTS])
	},
	Global {
		guard: Some(SAME_INDEX),
		..global("disjunctive_strict", &[INTS, INTS])
	},
	Global {
		guard: Some(SAME_INDEX),
		template: Some("distribute({1}, {2}) = {0}"),
		test: Some("distribute({0}, {1}, {2})"),
		..global("distribute", &[INTS, INTS, INTS])
	},
	// `x[i]` with a variable index
	Global {
		extra: Some("length({1}) > 1"),
		template: Some("{1}[{0}] = {2}"),
		..on_vars("element", &[INT, INTS, INT])
	},
	Global {
		blank: Some((1, Blank::Counts(0))),
		template: Some("global_cardinality({0}, set2array(index_set({1}))) = {1}"),
		test: Some("global_cardinality({0}, set2array(index_set({1})), {1})"),
		dominated_by: Some(("global_cardinality_closed", 0)),
		..on_vars("global_cardinality", &[INTS, INTS])
	},
	// No other values occur
	Global {
		template: Some("global_cardinality_closed({0}, set2array(index_set({1}))) = {1}"),
		test: Some("global_cardinality_closed({0}, set2array(index_set({1})), {1})"),
		..on_vars("global_cardinality_closed", &[INTS, INTS])
	},
	// Counts between bounds
	Global {
		check: |a| a[0].var && !a[1].var && !a[2].var,
		guard: Some(SAME_INDEX_12),
		template: Some("global_cardinality({0}, set2array(index_set({1})), {1}, {2})"),
		uses: "global_cardinality",
		dominated_by: Some(("global_cardinality_closed", 0)),
		..global("global_cardinality_bounds", &[INTS, INTS, INTS])
	},
	Global {
		check: |a| a[0].var && !a[1].var && !a[2].var,
		guard: Some(SAME_INDEX_12),
		template: Some("global_cardinality_closed({0}, set2array(index_set({1})), {1}, {2})"),
		uses: "global_cardinality_closed",
		..global("global_cardinality_closed_bounds", &[INTS, INTS, INTS])
	},
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("increasing", &[INTS])
	},
	Global {
		guard: Some(SAME_LENGTH),
		template: Some("inverse({0}) = {1}"),
		test: Some("inverse({0}, {1})"),
		..global("inverse", &[INTS, INTS])
	},
	// Also for arrays of different lengths
	global("inverse_in_range", &[INTS, INTS]),
	Global {
		check: |a| !a[0].var && !a[1].var && a[2].var,
		guard: Some("index_sets_agree({0}, {2}) /\\ index_sets_agree({1}, {2})"),
		..global("knapsack", &[INTS, INTS, INTS, INT, INT])
	},
	Global {
		guard: Some(LEX),
		..global("lex_less", &[INTS, INTS])
	},
	Global {
		guard: Some(LEX),
		..global("lex_lesseq", &[INTS, INTS])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("lex2", &[INTS2])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("lex2_strict", &[INTS2])
	},
	Global {
		extra: Some(LEX_CHAIN),
		..on_vars("lex_chain_less", &[INTS2])
	},
	Global {
		extra: Some(LEX_CHAIN),
		..on_vars("lex_chain_lesseq", &[INTS2])
	},
	Global {
		blank: Some((0, Blank::Max(1))),
		template: Some("max({1}) = {0}"),
		uses: "max",
		..global("maximum", &[INT, INTS])
	},
	Global {
		check: |a| a[1].var,
		extra: Some(LONGER_THAN_1),
		..global("member", &[INTS, INT])
	},
	Global {
		template: Some("min({1}) = {0}"),
		uses: "min",
		..global("minimum", &[INT, INTS])
	},
	Global {
		dominated_by: Some(("all_different", 1)),
		template: Some("nvalue({1}) = {0}"),
		test: Some("nvalue({0}, {1})"),
		..global("nvalue", &[INT, INTS])
	},
	Global {
		check: |a| a[0].var,
		extra: Some(LONGER_THAN_1),
		test: Some("forall ({v} in 1..ub_array({0}) - 1) (gz_value_precede({v}, {v} + 1, {0}))"),
		..global("seq_precede_chain", &[INTS])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && !a[2].var && a[3].var && !a[4].var,
		// For a 0/1 array this is a sliding_sum
		guard: Some("{0} <= {1} /\\ 1 < {2} /\\ {2} <= length({3}) /\\ card(dom_array({3})) > 2"),
		test: Some("gz_sliding_among({0}, {1}, {2}, {3}, {4})"),
		..global("sliding_among", &[INT, INT, INT, INTS, SET])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && !a[2].var && a[3].var,
		guard: Some("{0} <= {1} /\\ 1 < {2} /\\ {2} <= length({3})"),
		..global("sliding_sum", &[INT, INT, INT, INTS])
	},
	Global {
		guard: Some(SAME_LENGTH),
		template: Some("sort({0}) = {1}"),
		test: Some("sort({0}, {1})"),
		..global("sort", &[INTS, INTS])
	},
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("strictly_decreasing", &[INTS])
	},
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("strictly_increasing", &[INTS])
	},
	Global {
		test: Some("gz_subcircuit({0})"),
		// Both hold if every permutation has at most one cycle (e.g. 3 values)
		dominated_by: Some(("all_different", 0)),
		..on_vars("subcircuit", &[INTS])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("symmetric_all_different", &[INTS])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		guard: Some(TABLE),
		..global("table", &[INTS, INTS2])
	},
	Global {
		check: |a| !a[0].var && !a[1].var && a[2].var,
		// `t` must occur
		witness: Some("count({2}, {1})"),
		// Both values must be possible
		guard: Some("{0} in dom_array({2}) /\\ {1} in dom_array({2})"),
		test: Some("gz_value_precede({0}, {1}, {2})"),
		..global("value_precede", &[INT, INT, INTS])
	},
	Global {
		check: |a| !a[0].var && a[1].var,
		guard: Some(LONGER_THAN_1),
		test: Some(
			"forall ({v} in index_set({0}) where {v} < max(index_set({0}))) (gz_value_precede({0}[{v}], {0}[{v} + 1], {1}))",
		),
		..global("value_precede_chain", &[INTS, INTS])
	},
	// Boolean arguments
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_equal", &[BOOLS])
	},
	counting(
		"count",
		&[BOOLS, INT],
		"count ({v} in index_set({0})) ({0}[{v}]) = {1}",
		"{1} > 0",
	),
	// At most k (e.g. at most one) from the samples
	Global {
		blank: Some((1, Blank::MostTrue(0))),
		..counting(
			"count_leq",
			&[BOOLS, INT],
			"count ({v} in index_set({0})) ({0}[{v}]) <= {1}",
			"{1} < length({0})",
		)
	},
	counting(
		"count_lt",
		&[BOOLS, INT],
		"count ({v} in index_set({0})) ({0}[{v}]) < {1}",
		"{1} <= length({0})",
	),
	counting(
		"count_geq",
		&[BOOLS, INT],
		"count ({v} in index_set({0})) ({0}[{v}]) >= {1}",
		"{1} > 0",
	),
	counting(
		"count_gt",
		&[BOOLS, INT],
		"count ({v} in index_set({0})) ({0}[{v}]) > {1}",
		"{1} >= 0",
	),
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("decreasing", &[BOOLS])
	},
	Global {
		extra: Some("length({1}) > 1"),
		template: Some("{1}[{0}] = {2}"),
		..on_vars("element", &[INT, BOOLS, BOOL])
	},
	Global {
		extra: Some(LONGER_THAN_2),
		..on_vars("increasing", &[BOOLS])
	},
	Global {
		guard: Some(LEX),
		..global("lex_less", &[BOOLS, BOOLS])
	},
	Global {
		guard: Some(LEX),
		..global("lex_lesseq", &[BOOLS, BOOLS])
	},
	Global {
		extra: Some(LEX_CHAIN),
		..on_vars("lex_chain_less", &[BOOLS2])
	},
	Global {
		extra: Some(LEX_CHAIN),
		..on_vars("lex_chain_lesseq", &[BOOLS2])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		guard: Some(TABLE),
		..global("table", &[BOOLS, BOOLS2])
	},
	// Set arguments
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_different", &[SETS])
	},
	Global {
		check: |a| a[0].var && !a[1].var,
		extra: Some(LONGER_THAN_2),
		dominated_by: Some(("all_different", 0)),
		// Values in the set may occur more than once
		template: Some("all_different({0}, {1})"),
		..global("all_different_except", &[INTS, SET])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_disjoint", &[SETS])
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("all_equal", &[SETS])
	},
	Global {
		guard: Some(AMONG),
		..counting(
			"among",
			&[INTS, SET, INT],
			"count ({v} in index_set({0})) ({0}[{v}] in {1}) = {2}",
			"{2} > 0",
		)
	},
	Global {
		guard: Some(AMONG),
		..counting(
			"among_leq",
			&[INTS, SET, INT],
			"count ({v} in index_set({0})) ({0}[{v}] in {1}) <= {2}",
			"{2} < length({0})",
		)
	},
	Global {
		guard: Some(AMONG),
		..counting(
			"among_lt",
			&[INTS, SET, INT],
			"count ({v} in index_set({0})) ({0}[{v}] in {1}) < {2}",
			"{2} <= length({0})",
		)
	},
	Global {
		guard: Some(AMONG),
		..counting(
			"among_geq",
			&[INTS, SET, INT],
			"count ({v} in index_set({0})) ({0}[{v}] in {1}) >= {2}",
			"{2} > 0",
		)
	},
	Global {
		guard: Some(AMONG),
		..counting(
			"among_gt",
			&[INTS, SET, INT],
			"count ({v} in index_set({0})) ({0}[{v}] in {1}) > {2}",
			"{2} >= 0",
		)
	},
	Global {
		extra: Some(LONGER_THAN_1),
		..on_vars("at_most1", &[SETS])
	},
	global("disjoint", &[SET, SET]),
	global("int_set_channel", &[INTS, SETS]),
	global("inverse_set", &[SETS, SETS]),
	global("link_set_to_booleans", &[SET, BOOLS]),
	Global {
		check: |a| a[0].var && !a[1].var,
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
		extra: Some(LONGER_THAN_1),
		..on_vars("all_different", &[OPT_INTS])
	},
	// The optional task is scheduled exactly when one of the tasks is
	Global {
		guard: Some(SAME_INDEX_23),
		..on_vars("alternative", &[OPT_INT, INT, OPT_INTS, INTS])
	},
	Global {
		signature: &[OPT_INTS, INTS, INTS, INT],
		..CUMULATIVE
	},
	Global {
		guard: Some(SAME_INDEX),
		..global("disjunctive", &[OPT_INTS, INTS])
	},
	Global {
		guard: Some(SAME_INDEX),
		..global("disjunctive_strict", &[OPT_INTS, INTS])
	},
	// The optional task spans the tasks that are scheduled
	Global {
		guard: Some(SAME_INDEX_23),
		..on_vars("span", &[OPT_INT, INT, OPT_INTS, INTS])
	},
	// The group is implied by the declarations (and the context)
	Global {
		kind: Kind::Redundant,
		template: Some("true"),
		..global("true", &[])
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
		for chosen in slots.into_iter().multi_cartesian_product() {
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
			let texts = chosen.iter().map(|a| a.text.as_str()).collect::<Vec<_>>();
			let condition = |c: Option<&str>| {
				c.filter(|_| blank.is_none())
					.map(|c| instantiate(c, &texts))
			};
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

	#[test]
	fn instantiate_fills_arguments_once_and_picks_a_free_loop_variable() {
		// An argument with braces is not rewritten
		assert_eq!(instantiate("f({0}, {1})", &["{1}", "x"]), "f({1}, x)");
		assert_eq!(
			instantiate("count ({v} in S) ({0}[{v}])", &["i"]),
			"count (j in S) (i[j])"
		);
		// Distinct loop variables, none of them in an argument
		assert_eq!(
			instantiate("sum ({v} in S) ({0}[{v}, {w}, {u}])", &["x[i, ..]"]),
			"sum (j in S) (x[i, ..][j, k, l])"
		);
		// Unknown placeholders are kept
		assert_eq!(instantiate("{} {2}", &["a"]), "{} {2}");
	}

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
				.contains(&"(c > 0) /\\ (count (i in index_set(x)) (x[i] = v) = c)".to_owned())
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
        "arg_sort(x) = y",
        "arg_val(x, a) = b",
        "bin_packing(b, x, p)",
        "bin_packing_capa(p, x, q)",
        "bin_packing_load(y, p) = x",
        "circuit(x)",
        "count (i in index_set(x)) (x[i] = a) = b",
        "count (i in index_set(x)) (x[i] = a) <= b",
        "count (i in index_set(x)) (x[i] = a) < b",
        "count (i in index_set(x)) (x[i] = a) >= b",
        "count (i in index_set(x)) (x[i] = a) > b",
        "cumulative(x, p, q, a)",
        "decreasing(x)",
        "diffn(x, y, p, q)",
        "diffn_nonstrict(x, y, p, q)",
        "disjunctive(x, p)",
        "disjunctive_strict(x, p)",
        "distribute(p, y) = x",
        "x[a] = b",
        "global_cardinality(x, set2array(index_set(p))) = p",
        "global_cardinality (blank)",
        "global_cardinality(y, set2array(index_set(x))) = x",
        "global_cardinality (blank)",
        "global_cardinality_closed(x, set2array(index_set(p))) = p",
        "global_cardinality(x, set2array(index_set(p)), p, q)",
        "global_cardinality_closed(x, set2array(index_set(p)), p, q)",
        "increasing(x)",
        "inverse(x) = p",
        "inverse_in_range(x, p)",
        "knapsack(p, q, x, a, b)",
        "lex_less(x, p)",
        "lex_lesseq(x, p)",
        "lex2(m)",
        "lex2_strict(m)",
        "lex_chain_less(m)",
        "lex_chain_lesseq(m)",
        "max(x) = a",
        "maximum (blank)",
        "member(x, a)",
        "min(x) = a",
        "nvalue(x) = a",
        "seq_precede_chain(x)",
        "sort(x) = p",
        "strictly_decreasing(x)",
        "strictly_increasing(x)",
        "subcircuit(x)",
        "symmetric_all_different(x)",
        "value_precede_chain(p, x)",
        "all_equal(bs)",
        "count (i in index_set(bs)) (bs[i]) = a",
        "count (i in index_set(bs)) (bs[i]) <= a",
        "count_leq (blank)",
        "count (i in index_set(bs)) (bs[i]) < a",
        "count (i in index_set(bs)) (bs[i]) >= a",
        "count (i in index_set(bs)) (bs[i]) > a",
        "decreasing(bs)",
        "bs[a] = t",
        "increasing(bs)",
        "lex_chain_less(bm)",
        "lex_chain_lesseq(bm)",
        "all_different(ss)",
        "all_disjoint(ss)",
        "all_equal(ss)",
        "count (i in index_set(x)) (x[i] in s) = a",
        "count (i in index_set(x)) (x[i] in s) <= a",
        "count (i in index_set(x)) (x[i] in s) < a",
        "count (i in index_set(x)) (x[i] in s) >= a",
        "count (i in index_set(x)) (x[i] in s) > a",
        "at_most1(ss)",
        "int_set_channel(x, ss)",
        "link_set_to_booleans(s, bs)",
        "all_different(o)",
        "cumulative(o, p, q, a)",
        "disjunctive(o, x)",
        "disjunctive_strict(o, x)",
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
			"if ub_array(x) - lb_array(x) <= 100 then [lb_array(x)] ++ [count(x, i) | i in lb_array(x)..ub_array(x)] else [] endif"
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

		let bools = [typed("b", true, 1, Elem::Bool)];
		let mut most = filtered(&bools, &["count_leq"], &[], usize::MAX);
		assert_eq!(most[0].feature().unwrap(), "[sum(b), length(b)]");
		// At most all of them is trivially true, and a missing value fails
		assert!(!most[0].clone().fill(&[vec![1, 3], vec![3, 3]]));
		assert!(!most[0].clone().fill(&[vec![1, 3], vec![]]));
		assert!(most[0].fill(&[vec![0, 3], vec![1, 3]]));
		assert_eq!(most[0].call(), "count (i in index_set(b)) (b[i]) <= 1");
	}
}
