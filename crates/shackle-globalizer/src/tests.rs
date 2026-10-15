//! Globalizer tests. All tests need `MZN_STDLIB_DIR`; the `solve_*` tests
//! also need `minizinc` with Gecode on `PATH`.

use std::fmt::Write as _;

use expect_test::{Expect, expect};
use shackle::Model;
use shackle_hir::run_hir_phase;
use shackle_syntax::InputLang;

use super::{
	candidates::Candidate,
	groups::{Unrolled, Variant},
	*,
};

// Helpers

fn with<R>(src: &str, f: impl FnOnce(&Globalizer<'_, '_>) -> R) -> R {
	let model = Model::from_string(src.to_owned(), InputLang::MiniZinc);
	let errors = run_hir_phase(model.db()).errors;
	assert!(errors.is_empty(), "{errors:?}");
	with_globalizer(model.db(), &[], &GlobalizeOptions::default(), |g| f(g))
}

/// Run `f` on the group with the given variant
fn unrolled<R>(
	g: &Globalizer<'_, '_>,
	group: &[usize],
	variant: &Variant,
	f: impl FnOnce(&Unrolled<'_, '_, '_>) -> R,
) -> R {
	let decls = group
		.iter()
		.flat_map(|p| g.parts[*p].decls(g.model))
		.unique()
		.collect::<Vec<_>>();
	f(&Unrolled {
		parts: group.iter().map(|p| &g.parts[*p]).collect(),
		variant,
		decls: &decls,
	})
}

/// Run `f` on the group without unrolling
fn no_unrolling<R>(
	g: &Globalizer<'_, '_>,
	group: &[usize],
	f: impl FnOnce(&Unrolled<'_, '_, '_>) -> R,
) -> R {
	let variant = Variant {
		fixed: vec![Vec::new(); group.len()],
	};
	unrolled(g, group, &variant, f)
}

/// The preamble and the solve items of the group
fn submodel(g: &Globalizer<'_, '_>, u: &Unrolled<'_, '_, '_>) -> (String, [String; 2]) {
	(g.preamble(u.decls), g.search(u.decls))
}

/// The locations of the parts of `src`
fn locations(src: &str) -> Vec<String> {
	with(src, |g| {
		g.parts
			.iter()
			.map(|p| Location::new(g.db, p.origin).to_string())
			.collect()
	})
}

fn check_parts(src: &str, expected: Expect) {
	let printed = with(src, |g| {
		g.parts
			.iter()
			.map(|p| format!("{}\n", g.print_part(p)))
			.collect::<String>()
	});
	expected.assert_eq(&printed);
}

/// Options with two worker threads
fn options(max_group: usize, include: &[&str]) -> GlobalizeOptions {
	GlobalizeOptions {
		max_group,
		include: include.iter().map(|s| s.to_string()).collect(),
		jobs: 2,
		..Default::default()
	}
}

/// Globalize `src` with the given data files (as text)
fn globalize(src: &str, data: &[&str], options: &GlobalizeOptions) -> Globalized {
	let dir = tempfile::tempdir().unwrap();
	let data = data
		.iter()
		.enumerate()
		.map(|(i, d)| {
			let path = dir.path().join(format!("data{i}.dzn"));
			std::fs::write(&path, d).unwrap();
			path
		})
		.collect::<Vec<_>>();
	crate::globalize(
		&Model::from_string(src.to_owned(), InputLang::MiniZinc),
		&data,
		options,
	)
	.unwrap()
}

/// The findings as printed
fn shown(result: &Globalized) -> Vec<String> {
	result.findings.iter().map(ToString::to_string).collect()
}

/// The replacements found without data, with only the globals whose names
/// contain one of `include` (all if empty)
fn solve(src: &str, max_group: usize, include: &[&str]) -> Vec<String> {
	globalize(src, &[], &options(max_group, include))
		.findings
		.into_iter()
		.map(|f| f.replacement)
		.collect()
}

fn assert_found(found: &[String], expected: &str) {
	assert!(
		found.iter().any(|f| f == expected),
		"{expected} not in {found:?}"
	);
}

/// `expected` is found for the model `prelude` + `good`, but not for the
/// similar model `prelude` + `bad`, with only the globals whose name contains
/// `filter`
macro_rules! global_test {
	(
		$name:ident,
		$filter:literal,
		$max_group:literal,
		$expected:literal,
		$prelude:literal,
		$good:literal,
		$bad:literal $(,)?
	) => {
		#[test]
		fn $name() {
			let good = solve(concat!($prelude, $good), $max_group, &[$filter]);
			assert_found(&good, $expected);
			let bad = solve(concat!($prelude, $bad), $max_group, &[$filter]);
			assert!(!bad.contains(&$expected.to_owned()), "{bad:?}");
		}
	};
}

// Normalisation

#[test]
fn normalise_splits_conjunctions_and_foralls() {
	check_parts(
		r#"
		array[1..3] of var 1..3: x;
		var bool: b;
		constraint true;
		constraint forall(i in 1..3)(forall(j in 1..3 where i < j)(x[i] != x[j] /\ x[i] > 0));
		constraint x[1] = 1 /\ (x[2] = 2 /\ x[3] = 3);
		constraint forall([x[1] < 3, x[2] < 3]);
		"#,
		expect![[r#"
            forall(i in ((1) .. (3)), j in ((1) .. (3)) where ((i) < (j)))((((x[i])) != ((x[j]))))
            forall(i in ((1) .. (3)), j in ((1) .. (3)) where ((i) < (j)))((((x[i])) > (0)))
            (((x[1])) = (1))
            (((x[2])) = (2))
            (((x[3])) = (3))
            (((x[1])) < (3))
            (((x[2])) < (3))
        "#]],
	);
}

#[test]
fn normalise_keeps_disjunctions_and_exists() {
	check_parts(
		r#"
		array[1..3] of var 1..3: x;
		var bool: b;
		constraint exists(i in 1..3)(x[i] = 2);
		constraint b \/ x[1] > 1;
		constraint b -> x[2] > 1;
		"#,
		expect![[r#"
            exists([(((x[i])) = (2)) | i in ((1) .. (3))])
            ((b) \/ ((((x[1])) > (1))))
            ((b) -> ((((x[2])) > (1))))
        "#]],
	);
}

#[test]
fn normalise_splits_multi_variable_generators() {
	check_parts(
		r#"
		array[1..3] of var 1..3: x;
		constraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);
		"#,
		expect![[r#"
    forall(i in ((1) .. (3)), j in ((1) .. (3)) where ((i) < (j)))((((x[i])) != ((x[j]))))
"#]],
	);
}

#[test]
fn normalise_inlines_user_predicates() {
	check_parts(
		r#"
		include "all_different.mzn";
		array[1..3] of var 1..3: x;
		predicate pairwise(array[int] of var int: a) =
			forall(i, j in index_set(a) where i < j)(a[i] != a[j]);
		predicate both(var int: p, var int: q) = p > 0 /\ q > 0;
		predicate opaque(var int: p);
		predicate rec(int: k, var int: p) = if k = 0 then p > 0 else rec(k - 1, p) endif;
		constraint pairwise(x);
		constraint both(x[1], x[2]);
		constraint opaque(x[3]);
		constraint rec(2, x[3]);
		constraint all_different(x);
		"#,
		expect![[r#"
    forall(i in index_set(x), j in index_set(x) where ((i) < (j)))((((x[i])) != ((x[j]))))
    (((x[1])) > (0))
    (((x[2])) > (0))
    opaque((x[3]))
    if ((2) = (0)) then (((x[3])) > (0)) else rec(((2) - (1)), (x[3])) endif
    all_different(x, {})
"#]],
	);
}

/// The printed submodel of every part must be accepted by shackle again
#[test]
fn submodels_of_modern_models_parse_again() {
	let src = r#"
		enum Colour = {Red, Green, Blue};
		array[Colour] of var 1..3: x;
		var opt 1..3: o;
		var tuple(int, bool): t;
		record(var 1..3: a, var 1..3: b): r;
		constraint forall(c, d in Colour where c < d)(x[c] != x[d]);
		constraint t.1 > 0 /\ t.2;
		constraint r.a != r.b;
		constraint o != 2;
		constraint let { var 1..3: y = x[Red] + 1 } in y > 2;
		constraint case x[Green] = 1 of true => r.a = 1, _ => true endcase;
	"#;
	let printed = with(src, |g| {
		let decls = g
			.parts
			.iter()
			.flat_map(|p| p.decls(g.model))
			.collect::<Vec<_>>();
		let mut text = g.preamble(&decls);
		for p in &g.parts {
			writeln!(text, "constraint {};", g.print_part(p)).unwrap();
		}
		text
	});
	let again = Model::from_string(printed.clone(), InputLang::MiniZinc);
	let errors = run_hir_phase(again.db()).errors;
	assert!(errors.is_empty(), "{printed}\n{errors:?}");
}

#[test]
fn locations_are_one_based_and_inclusive() {
	let src = "var 1..3: a;\nvar 1..3: b;\nconstraint a != b /\\ b > 1;\nconstraint forall(i in 1..2)(\n  a > i);\n";
	assert_eq!(
		locations(src),
		[
			"<unnamed file>|3|12|3|17",
			"<unnamed file>|3|22|3|26",
			"<unnamed file>|4|1|5|8",
		]
	);
}

#[test]
fn inlined_parts_keep_the_call_location() {
	let src = "array[1..3] of var 1..3: x;\npredicate p(var int: a, var int: b) = a != b /\\ a < 9;\nconstraint p(x[1], x[2]);\nconstraint p(x[2], x[3]);\n";
	assert_eq!(
		locations(src),
		[
			"<unnamed file>|3|1|3|24",
			"<unnamed file>|3|1|3|24",
			"<unnamed file>|4|1|4|24",
			"<unnamed file>|4|1|4|24",
		]
	);
}

// Groups and variants

#[test]
fn groups_need_shared_variables() {
	// Parts 0-1 share `a`, parts 1-2 share `b`, part 3 is separate, part 4
	// has no variables
	let vars = vec![vec!["a"], vec!["a", "b"], vec!["b"], vec!["c"], vec![]];
	assert_eq!(groups(&vars, 1), [vec![0], vec![1], vec![2], vec![3]]);
	assert_eq!(
		groups(&vars, 2),
		[vec![0], vec![1], vec![2], vec![3], vec![0, 1], vec![1, 2]]
	);
	// {0, 2} is connected through 1
	assert!(groups(&vars, 3).contains(&vec![0, 1, 2]));
	assert!(!groups(&vars, 3).contains(&vec![0, 2, 3]));
	assert_eq!(groups(&vars, 3).len(), 7);
	assert_eq!(groups::<&str>(&[], 2), Vec::<Vec<usize>>::new());
}

/// Print the variants of a group as the fixed loop names of each part
fn printed_variants(g: &Globalizer<'_, '_>, group: &[usize]) -> Vec<String> {
	g.variants(group)
		.iter()
		.map(|v| {
			v.fixed
				.iter()
				.zip(group)
				.map(|(f, p)| {
					let names = f.iter().map(|l| g.name(g.parts[*p].loops[*l].decl));
					format!("[{}]", names.collect::<Vec<_>>().join(" "))
				})
				.collect::<String>()
		})
		.collect()
}

#[test]
fn variants_fix_subsets_of_set_loops() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		array[1..n] of int: p = [1, 2, 3];
		constraint forall(r in 1..n, c in 1..n)(x[r, c] > 0);
		constraint forall(i in 1..n, j in i + 1..n)(x[i, j] > 0);
		constraint forall(v in p)(x[v, v] > 0);
		constraint forall(i in 1..n, k = i + 1 where k <= n)(x[i, k] > 0);
	"#;
	let variants = with(src, |g| {
		(0..4)
			.map(|p| printed_variants(g, &[p]))
			.collect::<Vec<_>>()
	});
	assert_eq!(variants[0], ["[]", "[r]", "[c]", "[r c]"]);
	// `j` cannot be fixed while `i` is still a loop
	assert_eq!(variants[1], ["[]", "[i]", "[i j]"]);
	// Arrays and assignments are not sets
	assert_eq!(variants[2], ["[]"]);
	assert_eq!(variants[3], ["[]", "[i]"]);
}

#[test]
fn variants_match_loops_of_other_parts() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n)(x[r, 1] > 0);
		constraint forall(a in 1..n, b in 1..n)(x[a, b] != 2);
		constraint forall(t in 1..2)(x[t, 2] > 0);
		constraint forall(i in 1..n, j in 1..n)(x[i, j] != 1);
		constraint forall(j in 1..n, i in 1..n)(x[i, j] != 3);
		constraint forall(r in 1..n where r > 1)(x[r, 3] > 0);
	"#;
	let variants = with(src, |g| {
		[[0, 1], [0, 2], [3, 4], [0, 5]].map(|group| printed_variants(g, &group))
	});
	assert_eq!(variants[0], ["[][]", "[r][a]", "[r][b]"]);
	// `1..2` is not the same set as `1..n`
	assert_eq!(variants[1], ["[][]"]);
	// With both loops fixed, `i` cannot match `j` of the second part: the
	// other leader already has that name (no `[i j][j i]`)
	assert_eq!(
		variants[2],
		["[][]", "[i][j]", "[i][i]", "[j][j]", "[j][i]", "[i j][i j]"]
	);
	// The `where` clauses differ
	assert_eq!(variants[3], ["[][]"]);
}

#[test]
fn unrolled_parts_and_leaders() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);
		constraint forall(s in 1..n)(x[s, 1] > 0);
	"#;
	let printed = with(src, |g| {
		// Fixing `c2` alone is invalid: its `where` clause uses `c1`
		assert!(!g.parts[0].can_fix(g.db, g.model, &[2]));
		[vec![vec![0], vec![0]], vec![vec![0, 1, 2], vec![0]]]
			.into_iter()
			.map(|fixed| {
				unrolled(g, &[0, 1], &Variant { fixed }, |u| {
					let all = u.instantiations();
					(
						g.print_group(u),
						g.print_leaders(u, &all[0]),
						g.print_leaders(u, all.last().unwrap()),
						all.len(),
					)
				})
			})
			.collect::<Vec<_>>()
	});
	expect![[r#"
    [
        (
            [
                "forall(c1 in ((1) .. (n)), c2 in ((1) .. (n)) where ((c1) < (c2)))((((x[r, c1])) != ((x[r, c2]))))",
                "(((x[s, 1])) > (0))",
            ],
            "any: r = min(((1) .. (n)));\nany: s = r;\n",
            "any: r = max(((1) .. (n)));\nany: s = r;\n",
            2,
        ),
        (
            [
                "((((c1) < (c2)))) -> ((((x[r, c1])) != ((x[r, c2]))))",
                "(((x[s, 1])) > (0))",
            ],
            "any: r = min(((1) .. (n)));\nany: c1 = min(((1) .. (n)));\nany: c2 = min(((1) .. (n)));\nany: s = r;\nconstraint ((c1) < (c2));\n",
            "any: r = max(((1) .. (n)));\nany: c1 = max(((1) .. (n)));\nany: c2 = max(((1) .. (n)));\nany: s = r;\nconstraint ((c1) < (c2));\n",
            8,
        ),
    ]
"#]].assert_debug_eq(&printed);
}

// Arguments and submodels

fn printed_args(args: &[candidates::Arg]) -> Vec<String> {
	args.iter()
		.map(|a| format!("{} var={} dims={} {:?}", a.text, a.var, a.dims, a.leaders))
		.collect()
}

#[test]
fn arguments_come_from_the_group() {
	let src = r#"
		int: n = 3;
		array[1..n] of var 1..3: x;
		array[1..n] of int: p = [1, 2, 3];
		array[1..2, 1..2] of var 1..3: m;
		array[1..n] of var float: f;
		array[1..n] of var opt 1..3: xo;
		var 1..3: a;
		var 1..3: b;
		var 1..3: c;
		var opt 1..3: o;
		constraint x[1] + p[1] + m[1, 1] + a + b + c + xo[1] > f[1] + 1.0;
		constraint o != 1;
	"#;
	let args = with(src, |g| no_unrolling(g, &[0], |u| g.arguments(u)));
	assert_eq!(
		printed_args(&args),
		[
			"0 var=false dims=0 []",
			"x var=true dims=1 []",
			"p var=false dims=1 []",
			"m var=true dims=2 []",
			"a var=true dims=0 []",
			"b var=true dims=0 []",
			"c var=true dims=0 []",
			"xo var=true dims=1 []",
			"[a, b, c] var=true dims=1 []",
		]
	);
}

#[test]
fn arguments_use_leaders_and_par_indices() {
	let src = r#"
		int: n = 3;
		int: k = 2;
		array[1..n, 1..n] of var 1..n: x;
		array[1..n] of int: w = [1, 2, 3];
		constraint forall(r in 1..n, c in 1..n)(x[r, c] != x[k, c] + w[r]);
	"#;
	let args = with(src, |g| {
		let variant = Variant {
			fixed: vec![vec![0]],
		};
		unrolled(g, &[0], &variant, |u| g.arguments(u))
	});
	// `c` is still a loop, so it is never an index
	assert_eq!(
		printed_args(&args),
		[
			"0 var=false dims=0 []",
			"n var=false dims=0 []",
			"x var=true dims=2 []",
			"k var=false dims=0 []",
			"w var=false dims=1 []",
			"x[r, ..] var=true dims=1 [0]",
			"x[k, ..] var=true dims=1 []",
			"w[r] var=false dims=0 [0]",
		]
	);
}

#[test]
fn preamble_declares_only_needed_variables() {
	let src = r#"
		int: n = 3;
		array[1..n] of var 1..3: x;
		var int: s = sum(x);
		var 1..3: unrelated;
		function int: double(int: v) = 2 * v;
		constraint s < double(n);
		constraint unrelated > 1;
	"#;
	let preamble = with(src, |g| g.preamble(&g.parts[0].decls(g.model)));
	expect![[r#"
    include "globals.mzn";
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
    int: n = 3;
    array [((1) .. (n))] of var ((1) .. (3)): x;
    var int: s = sum(x);
"#]]
	.assert_eq(&preamble);
}

#[test]
fn preamble_declares_variables_of_par_items() {
	// `k` is printed in every submodel, so `y` must be too
	let src = r#"
		array[1..3] of var 1..3: y;
		int: k = length(y);
		var 1..3: a;
		constraint a > 1;
		constraint y[1] > 1;
	"#;
	let preamble = with(src, |g| g.preamble(&g.parts[0].decls(g.model)));
	assert!(preamble.contains("var ((1) .. (3)): y;"), "{preamble}");
}

#[test]
fn local_includes_are_user_files() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(
		dir.path().join("helper.mzn"),
		"array[1..3] of var 1..3: x;\npredicate pairwise(array[int] of var int: a) = forall(i, j in index_set(a) where i < j)(a[i] != a[j]);\n",
	)
	.unwrap();
	let main = dir.path().join("main.mzn");
	std::fs::write(&main, "include \"helper.mzn\";\nconstraint pairwise(x);\n").unwrap();
	let model = Model::from_file(main);
	let preamble = with_globalizer(model.db(), &[], &GlobalizeOptions::default(), |g| {
		g.preamble(&g.parts[0].decls(g.model))
	});
	assert!(preamble.contains(": x;"), "{preamble}");
}

#[test]
fn search_annotation_covers_every_argument_type() {
	let src = r#"
		array[1..2] of var 1..3: x;
		array[1..2] of var opt 1..3: xo;
		var 1..3: a;
		var bool: b;
		var set of 1..3: s;
		constraint b -> x[1] != a /\ xo[1] != 1 /\ 1 in s;
	"#;
	let search = with(src, |g| {
		let decls = g.parts[0].decls(g.model);
		g.search(&decls)
	});
	let solve = "solve :: seq_search([int_search(array1d(x) ++ [a], input_order, indomain_random), int_search(array1d(xo), input_order, indomain_random), bool_search([b], input_order, indomain_random), bool_search([gz_e in gz_s | gz_s in [s], gz_e in ub(gz_s)], input_order, indomain_random)])";
	assert_eq!(
		search,
		[
			format!("{solve} satisfy;\n"),
			format!("{solve} :: restart_luby(1) satisfy;\n"),
		]
	);
}

#[test]
fn replacement_wraps_used_leaders_and_their_dependencies() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(i in 1..n, j in i + 1..n where j > 1)(x[i, j] > 0);
	"#;
	let replacements = with(src, |g| {
		let variant = Variant {
			fixed: vec![vec![0, 1]],
		};
		unrolled(g, &[0], &variant, |u| {
			let plan = g.plan(u, Pass::Full).unwrap();
			[vec![], vec![0], vec![1]].map(|leaders| {
				let mut cand = Candidate::raw("all_different", &["x[i, ..]"]);
				cand.leaders = leaders;
				Checker::replacement(&plan, &cand)
			})
		})
	});
	assert_eq!(
		replacements,
		[
			"all_different(x[i, ..])",
			"forall (i in 1..n) (all_different(x[i, ..]))",
			"forall (i in 1..n, j in i + 1..n where j > 1) (all_different(x[i, ..]))",
		]
	);
}

#[test]
fn tidy_removes_parentheses() {
	assert_eq!(
		tidy("forall(r in ((1) .. (n)))(all_different(x[(r), ..]))".to_owned()),
		"forall (r in 1..n) (all_different(x[r, ..]))"
	);
	// Not MiniZinc: returned unchanged
	assert_eq!(tidy("((".to_owned()), "((");
}

#[test]
fn passes_split_the_candidates() {
	assert!(Pass::Initial.checks(Kind::Redundant));
	assert!(Pass::Initial.checks(Kind::Viewpoint(1)));
	assert!(!Pass::Initial.checks(Kind::Normal));
	assert!(Pass::Full.checks(Kind::Normal));
	assert!(!Pass::Full.checks(Kind::Redundant));
	assert!(!Pass::Full.checks(Kind::Viewpoint(0)));
}

#[test]
fn viewpoint_texts() {
	let src = "array[1..2, 1..3, 1..4] of var 0..1: b;\nconstraint b[1, 1, 1] = 1;";
	let texts = with(src, |g| {
		let b = g.parts[0].decls(g.model)[0];
		(0..3)
			.map(|axis| {
				let vp = Viewpoint::new(b, "b", axis);
				format!(
					"{}\n{}\n{}\n{}",
					vp.name, vp.text, vp.definition, vp.channel
				)
			})
			.collect::<Vec<_>>()
	});
	expect![[r#"
    [
        "b_3a\narray [index_set_2of3(b), index_set_3of3(b)] of var index_set_1of3(b): b_3a; constraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3a[j, k] = i);\narray [index_set_2of3(b), index_set_3of3(b)] of var int: b_3a = array2d(index_set_2of3(b), index_set_3of3(b), [sum (i in index_set_1of3(b)) (i * b[i, j, k]) | j in index_set_2of3(b), k in index_set_3of3(b)]);\nconstraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3a[j, k] = i);\nconstraint forall (v in array1d(b_3a)) (v in index_set_1of3(b));",
        "b_3b\narray [index_set_1of3(b), index_set_3of3(b)] of var index_set_2of3(b): b_3b; constraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3b[i, k] = j);\narray [index_set_1of3(b), index_set_3of3(b)] of var int: b_3b = array2d(index_set_1of3(b), index_set_3of3(b), [sum (j in index_set_2of3(b)) (j * b[i, j, k]) | i in index_set_1of3(b), k in index_set_3of3(b)]);\nconstraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3b[i, k] = j);\nconstraint forall (v in array1d(b_3b)) (v in index_set_2of3(b));",
        "b_3c\narray [index_set_1of3(b), index_set_2of3(b)] of var index_set_3of3(b): b_3c; constraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3c[i, j] = k);\narray [index_set_1of3(b), index_set_2of3(b)] of var int: b_3c = array2d(index_set_1of3(b), index_set_2of3(b), [sum (k in index_set_3of3(b)) (k * b[i, j, k]) | i in index_set_1of3(b), j in index_set_2of3(b)]);\nconstraint forall (i in index_set_1of3(b), j in index_set_2of3(b), k in index_set_3of3(b)) (b[i, j, k] = 1 <-> b_3c[i, j] = k);\nconstraint forall (v in array1d(b_3c)) (v in index_set_3of3(b));",
    ]
"#]].assert_debug_eq(&texts);
}

// Findings

#[test]
fn shadowed_findings_are_hidden() {
	let raw = |group: &[usize], context, replacement: &str| RawFinding {
		group: group.to_vec(),
		context,
		replacement: replacement.to_owned(),
		candidate: Candidate::raw("true", &[]),
	};
	let found = [
		raw(&[0], None, "a"),
		raw(&[0, 1], None, "a"),
		raw(&[0], Some(1), "a"),
		raw(&[0, 1], None, "b"),
		raw(&[1], Some(0), "b"),
		raw(&[2], Some(0), "c"),
		raw(&[2], Some(1), "c"),
	];
	// 4 is hidden by 3: its context is part of the group of 3
	assert_eq!(
		shadow(&found),
		[true, false, false, true, false, true, true]
	);
}

#[test]
fn finding_display() {
	let src = SourceFile::new("m.mzn".into(), "aaaaa\nbbbbb\nccccc\n".to_owned());
	let line = |l: usize| Location {
		src: src.clone(),
		span: SourceSpan::new((6 * l).into(), 5),
	};
	let mut finding = Finding {
		locations: vec![line(0), line(1)],
		context: None,
		replacement: "all_different(x)".to_owned(),
	};
	assert_eq!(
		finding.to_string(),
		"m.mzn|1|1|1|5;m.mzn|2|1|2|5 [ ] all_different(x)"
	);
	finding.context = Some(line(2));
	assert_eq!(
		finding.to_string(),
		"m.mzn|1|1|1|5;m.mzn|2|1|2|5 [ m.mzn|3|1|3|5 ] all_different(x)"
	);
}

// Checks with the solver

#[test]
fn solve_bad_candidate_is_split_off() {
	let src = r#"
		array[1..4] of var 1..5: x;
		constraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);
	"#;
	let kept = with(src, |g| {
		no_unrolling(g, &[0], |u| {
			let (preamble, search) = submodel(g, u);
			let base = format!("{preamble}constraint {};\n", g.print_part(&g.parts[0]));
			let cands = vec![
				Candidate::raw("all_different", &["x"]),
				Candidate::raw("no_such_global", &["x"]),
				Candidate::raw("increasing", &["x"]),
				Candidate::raw("all_equal", &["x"]),
			];
			g.checker
				.check_candidates(&base, &search, &None, cands)
				.unwrap()
				.unwrap()
		})
	});
	assert_eq!(kept, [Candidate::raw("all_different", &["x"])]);
}

#[test]
fn solve_broken_base_has_no_samples() {
	let src = "array[1..4] of var 1..5: x;\nconstraint x[1] != x[2];";
	let (kept, calls) = with(src, |g| {
		no_unrolling(g, &[0], |u| {
			let (preamble, search) = submodel(g, u);
			let base = format!("{preamble}constraint no_such_predicate(x);\n");
			let cands = (0..8)
				.map(|_| Candidate::raw("all_different", &["x"]))
				.collect::<Vec<_>>();
			let kept = g.checker.check_candidates(&base, &search, &None, cands);
			(
				kept.unwrap(),
				g.checker.runner.calls.load(Ordering::Relaxed),
			)
		})
	});
	assert_eq!(kept, None);
	// The full run and the base, not a split down to single candidates
	assert_eq!(calls, 2);
}

#[test]
fn solve_implies_check() {
	let src = r#"
		array[1..3] of var 1..3: x;
		constraint x[1] < x[2];
		constraint x[1] != x[2];
	"#;
	let (forward, backward) = with(src, |g| {
		no_unrolling(g, &[0, 1], |u| {
			let (preamble, search) = submodel(g, u);
			let [a, b] = &g.print_group(u)[..] else {
				unreachable!()
			};
			let checker = &g.checker;
			(
				checker.implies(&preamble, &search, &None, a, b).unwrap(),
				checker.implies(&preamble, &search, &None, b, a).unwrap(),
			)
		})
	});
	assert!(forward);
	assert!(!backward);
}

/// The test templates must accept exactly the solutions of the global
#[test]
fn solve_test_templates_match_the_globals() {
	let pairs = [
		("circuit(x)", "gz_circuit(x)"),
		("subcircuit(x)", "gz_subcircuit(x)"),
		("value_precede(1, 2, x)", "gz_value_precede(1, 2, x)"),
		(
			"value_precede_chain([1, 2, 3], x)",
			"forall (i in index_set([1, 2, 3]) where i < max(index_set([1, 2, 3]))) (gz_value_precede([1, 2, 3][i], [1, 2, 3][i + 1], x))",
		),
		(
			"seq_precede_chain(x)",
			"forall (i in 1..ub_array(x) - 1) (gz_value_precede(i, i + 1, x))",
		),
		(
			"sliding_among(1, 2, 2, x, {1, 2})",
			"gz_sliding_among(1, 2, 2, x, {1, 2})",
		),
	];
	let options = GlobalizeOptions::default();
	let runner = Runner::new(&options);
	let all = |c: &str| {
		let src = format!(
			"include \"globals.mzn\";{}\narray[1..4] of var 1..4: x;\nconstraint {c};\nsolve satisfy;\n",
			candidates::LIBRARY
		);
		let outcome = runner.run(&src, None, 1000, false).unwrap();
		assert!(
			outcome.complete && outcome.error.is_none(),
			"{c}: {outcome:?}"
		);
		outcome
			.solutions
			.iter()
			.map(|s| serde_json::to_string(s).unwrap())
			.sorted()
			.collect::<Vec<_>>()
	};
	for (global, test) in pairs {
		let expected = all(global);
		assert!(!expected.is_empty(), "{global}");
		assert_eq!(all(test), expected, "{test} differs from {global}");
	}
}

#[test]
fn solve_all_different_pairwise() {
	let src =
		"array[1..4] of var 1..5: x;\nconstraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);";
	assert_eq!(solve(src, 1, &[]), ["all_different(x)"]);
}

#[test]
fn solve_all_different_missing_pair() {
	let src = "array[1..4] of var 1..5: x;\nconstraint forall(i, j in 1..4 where i < j /\\ (i, j) != (1, 4))(x[i] != x[j]);";
	assert!(solve(src, 2, &[]).is_empty());
}

#[test]
fn solve_all_different_of_scalars() {
	let src = "var 1..3: a;\nvar 1..3: b;\nvar 1..3: c;\nconstraint a != b;\nconstraint b != c;\nconstraint a != c;";
	let found = solve(src, 3, &[]);
	// Found once: two of the three disequalities are not enough
	let count = found
		.iter()
		.filter(|f| *f == "all_different([a, b, c])")
		.count();
	assert_eq!(count, 1, "{found:?}");
}

#[test]
fn solve_all_different_dominates_its_equivalent_forms() {
	// `nvalue(n, x)` and `global_cardinality` with all counts 1 are
	// equivalent, but not reported
	let src = "int: n = 4;\narray[1..n] of var 1..n: x;\nconstraint forall(i, j in 1..n where i < j)(x[i] != x[j]);";
	assert_eq!(solve(src, 1, &[]), ["all_different(x)"]);
}

#[test]
fn solve_existing_global_is_vacuous() {
	let src =
		"include \"all_different.mzn\";\narray[1..4] of var 1..5: x;\nconstraint all_different(x);";
	assert!(solve(src, 1, &[]).is_empty());
}

#[test]
fn solve_unsatisfiable_group_is_skipped() {
	let src =
		"array[1..4] of var 1..3: x;\nconstraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);";
	assert!(solve(src, 1, &[]).is_empty());
}

#[test]
fn solve_rows_with_leaders() {
	let src = "int: n = 4;\narray[1..n, 1..n] of var 1..n: x;\nconstraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);";
	assert_eq!(
		solve(src, 1, &["all_different"]),
		["forall (r in 1..n) (all_different(x[r, ..]))"]
	);
}

#[test]
fn solve_rows_missing_a_pair_in_the_last_row() {
	// Only the last row misses a pair, so only the last leader value fails
	let src = "int: n = 4;\narray[1..n, 1..n] of var 1..n: x;\nconstraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2 /\\ (r, c1, c2) != (n, 1, 2))(x[r, c1] != x[r, c2]);";
	assert!(solve(src, 1, &[]).is_empty());
}

#[test]
fn solve_leaders_never_give_true() {
	// For i = 1 and j = 8 the diagonal is rarely violated on samples
	let src = "int: n = 8;\narray[1..n] of var 1..n: q;\nconstraint forall(i, j in 1..n where i < j)(q[i] + i != q[j] + j);";
	assert!(!solve(src, 1, &["true"]).contains(&"true".to_owned()));
}

#[test]
fn solve_context_is_reported() {
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] <= x[2];\nconstraint x[2] <= x[3];";
	assert_eq!(
		shown(&globalize(src, &[], &options(1, &["increasing"]))),
		[
			"<unnamed file>|2|1|2|23 [ <unnamed file>|3|1|3|23 ] increasing(x)",
			"<unnamed file>|3|1|3|23 [ <unnamed file>|2|1|2|23 ] increasing(x)",
		]
	);
}

#[test]
fn solve_redundant_group_is_true() {
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 1;\nconstraint x[2] >= 2;";
	assert_eq!(solve(src, 1, &[]), ["true"]);
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 2;";
	assert!(solve(src, 1, &[]).is_empty());
}

#[test]
fn solve_user_predicate_is_inlined() {
	let src = r#"
		int: n = 4;
		array[1..n] of var 0..9: y;
		predicate sorted(array[int] of var int: a) =
			forall(i in 1..length(a) - 1)(a[i] <= a[i + 1]) /\ a[1] >= 0;
		constraint sorted(y);
	"#;
	// The initial pass also finds that `a[1] >= 0` is redundant
	assert_eq!(solve(src, 1, &["increasing"]), ["increasing(y)", "true"]);
}

#[test]
fn solve_uses_data_files() {
	let src = "int: n;\narray[1..n] of var 1..n: x;\nconstraint forall(i, j in 1..n where i < j)(x[i] != x[j]);";
	let result = globalize(src, &["n = 4;"], &options(1, &["all_different"]));
	assert_eq!(result.findings[0].replacement, "all_different(x)");
}

#[test]
fn solve_global_must_hold_for_every_data_file() {
	let src = "int: n;\narray[1..n] of var 1..n: x;\nconstraint forall(i in 1..n - 1)(x[i] <= x[i + 1] \\/ n > 4);";
	let options = options(1, &["increasing"]);
	assert_eq!(globalize(src, &["n = 4;"], &options).findings.len(), 1);
	assert!(
		globalize(src, &["n = 4;", "n = 5;"], &options)
			.findings
			.is_empty()
	);
}

#[test]
fn solve_same_findings_for_any_number_of_workers() {
	let src = r#"
		int: n = 4;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);
		constraint forall(c in 1..n, r1, r2 in 1..n where r1 < r2)(x[r1, c] != x[r2, c]);
	"#;
	let found = |jobs| {
		let options = GlobalizeOptions {
			jobs,
			..options(1, &["all_"])
		};
		globalize(src, &[], &options).findings
	};
	let one = found(1);
	assert_eq!(one.len(), 2, "{one:?}");
	assert_eq!(found(4), one);
}

const LATIN01: &str = r#"
	int: n = 3;
	array[1..n, 1..n, 1..n] of var 0..1: b;
	constraint forall(i, j in 1..n)(sum(k in 1..n)(b[i, j, k]) = 1);
	constraint forall(i, k in 1..n)(sum(j in 1..n)(b[i, j, k]) = 1);
"#;

#[test]
fn solve_initial_pass_finds_viewpoints() {
	let result = globalize(LATIN01, &[], &options(1, &["all_different"]));
	let declarations = result
		.viewpoints
		.iter()
		.map(|v| v.replacement.split(';').next().unwrap())
		.collect::<Vec<_>>();
	assert_eq!(
		declarations,
		[
			"array [index_set_1of3(b), index_set_3of3(b)] of var index_set_2of3(b): b_3b",
			"array [index_set_1of3(b), index_set_2of3(b)] of var index_set_3of3(b): b_3c",
		]
	);
	assert_eq!(
		shown(&result),
		[
			"<unnamed file>|4|2|4|64 [ <unnamed file>|5|2|5|64 ] forall (i in 1..n) (all_different(b_3b[i, ..]))",
			"<unnamed file>|5|2|5|64 [ <unnamed file>|4|2|4|64 ] forall (i in 1..n) (all_different(b_3c[i, ..]))",
		]
	);
}

#[test]
fn solve_without_initial_pass() {
	let options = |initial_pass| GlobalizeOptions {
		initial_pass,
		..options(1, &["all_different"])
	};
	// No viewpoints: nothing to find on the 0/1 array
	let result = globalize(LATIN01, &[], &options(false));
	assert!(result.viewpoints.is_empty());
	assert!(result.findings.is_empty());
	// No redundant groups either
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 1;";
	assert!(globalize(src, &[], &options(false)).findings.is_empty());
	assert_eq!(globalize(src, &[], &options(true)).findings.len(), 1);
}

#[test]
fn solve_redundant_group_is_left_out_of_the_full_pass() {
	// `x[1] >= 1` is redundant, so it is not grouped alone in the full pass
	let src = "array[1..4] of var 1..4: x;\nconstraint x[1] >= 1;\nconstraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);";
	assert_eq!(
		shown(&globalize(src, &[], &options(1, &[]))),
		[
			"<unnamed file>|2|1|2|20 [ ] true",
			"<unnamed file>|3|1|3|57 [ ] all_different(x)",
		]
	);
}

// One test for each global in the table

global_test!(
	solve_all_equal,
	"all_equal",
	1,
	"all_equal(x)",
	"array[1..4] of var 1..3: x;\n",
	"constraint forall(i in 1..3)(x[i] = x[i + 1]);",
	"constraint forall(i in 1..3)(x[i] <= x[i + 1]);",
);

global_test!(
	solve_alldifferent_except_0,
	"except_0",
	1,
	"alldifferent_except_0(x)",
	"array[1..4] of var 0..4: x;\n",
	"constraint forall(i, j in 1..4 where i < j)(x[i] = 0 \\/ x[j] = 0 \\/ x[i] != x[j]);",
	"constraint forall(i, j in 1..4 where i < j /\\ (i, j) != (1, 4))(x[i] = 0 \\/ x[j] = 0 \\/ x[i] != x[j]);",
);

global_test!(
	solve_bin_packing,
	"bin_packing",
	1,
	"bin_packing(c, bin, w)",
	"array[1..3] of int: w = [2, 3, 4];\nint: c = 7;\narray[1..3] of var 1..2: bin;\n",
	"constraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= c);",
	"constraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= c - 1);",
);

global_test!(
	solve_bin_packing_capa,
	"bin_packing_capa",
	1,
	"bin_packing_capa(c, bin, w)",
	"array[1..3] of int: w = [2, 3, 4];\narray[1..2] of int: c = [5, 7];\narray[1..3] of var 1..2: bin;\n",
	"constraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= c[b]);",
	"constraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= c[b] + 1);",
);

global_test!(
	solve_bin_packing_load,
	"bin_packing_load",
	1,
	"bin_packing_load(bin, w) = load",
	"array[1..3] of int: w = [2, 3, 4];\narray[1..3] of var 1..2: bin;\narray[1..2] of var 0..9: load;\n",
	"constraint forall(b in 1..2)(load[b] = sum(i in 1..3)(w[i] * (bin[i] = b)));",
	"constraint forall(b in 1..2)(load[b] = sum(i in 1..2)(w[i] * (bin[i] = b)));",
);

global_test!(
	solve_count,
	"count",
	1,
	"count (i in index_set(x)) (x[i] = v) = c",
	"int: v = 2;\nint: c = 2;\narray[1..4] of var 1..3: x;\n",
	"constraint sum(i in 1..4)(x[i] = v) = c;",
	"constraint sum(i in 1..4)(x[i] = v) <= c;",
);

global_test!(
	solve_count_leq,
	"count",
	1,
	"count (i in index_set(x)) (x[i] = v) <= c",
	"int: v = 2;\nint: c = 2;\narray[1..4] of var 1..3: x;\n",
	"constraint sum(i in 1..4)(x[i] = v) <= c;",
	"constraint sum(i in 1..3)(x[i] = v) <= c;",
);

global_test!(
	solve_count_geq,
	"count",
	1,
	"count (i in index_set(x)) (x[i] = v) >= c",
	"int: v = 2;\nint: c = 2;\narray[1..4] of var 1..3: x;\n",
	"constraint sum(i in 1..4)(x[i] = v) >= c;",
	"constraint sum(i in 1..3)(x[i] = v) >= c;",
);

global_test!(
	solve_cumulative,
	"cumulative",
	1,
	"cumulative(s, d, r, b)",
	"array[1..3] of int: d = [2, 2, 2];\narray[1..3] of int: r = [1, 1, 1];\nint: b = 2;\narray[1..3] of var 0..4: s;\n",
	"constraint forall(t in 0..6)(sum(i in 1..3)(r[i] * (s[i] <= t /\\ t < s[i] + d[i])) <= b);",
	"constraint forall(t in 0..6)(sum(i in 1..3)(r[i] * (s[i] <= t /\\ t < s[i] + d[i])) <= b - 1);",
);

global_test!(
	solve_decreasing,
	"decreasing",
	1,
	"decreasing(x)",
	"array[1..5] of var 1..9: x;\n",
	"constraint forall(i in 1..4)(x[i] >= x[i + 1]);",
	"constraint forall(i in 1..4 where i != 2)(x[i] >= x[i + 1]);",
);

global_test!(
	solve_diffn,
	"diffn",
	1,
	"diffn(x, y, w, h)",
	"array[1..3] of int: w = [1, 2, 1];\narray[1..3] of int: h = [2, 1, 1];\narray[1..3] of var 0..3: x;\narray[1..3] of var 0..3: y;\n",
	"constraint forall(i, j in 1..3 where i < j)(x[i] + w[i] <= x[j] \\/ x[j] + w[j] <= x[i] \\/ y[i] + h[i] <= y[j] \\/ y[j] + h[j] <= y[i]);",
	"constraint forall(i, j in 1..3 where i < j)(x[i] + w[i] <= x[j] \\/ x[j] + w[j] <= x[i] \\/ y[i] + h[i] <= y[j]);",
);

global_test!(
	solve_disjunctive,
	"disjunctive",
	1,
	"disjunctive(s, d)",
	"array[1..3] of int: d = [2, 1, 2];\narray[1..3] of var 0..5: s;\n",
	"constraint forall(i, j in 1..3 where i < j)(s[i] + d[i] <= s[j] \\/ s[j] + d[j] <= s[i]);",
	"constraint forall(i, j in 1..3 where i < j /\\ j != 3)(s[i] + d[i] <= s[j] \\/ s[j] + d[j] <= s[i]);",
);

global_test!(
	solve_distribute,
	"distribute",
	1,
	"distribute(value, base) = card",
	"array[1..2] of int: value = [1, 3];\narray[1..4] of var 1..3: base;\narray[1..2] of var 0..4: card;\n",
	"constraint forall(i in 1..2)(card[i] = sum(j in 1..4)(base[j] = value[i]));",
	"constraint forall(i in 1..2)(card[i] = sum(j in 1..3)(base[j] = value[i]));",
);

global_test!(
	solve_global_cardinality,
	"global_cardinality",
	1,
	"global_cardinality_closed(x, set2array(index_set(c))) = c",
	"",
	"array[1..3] of int: c = [1, 1, 2];\narray[1..4] of var 1..3: x;\nconstraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) = c[v]);",
	// Counts of at most 2 can vary
	"array[1..3] of int: c = [2, 2, 2];\narray[1..4] of var 1..3: x;\nconstraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) <= c[v]);",
);

// The value 4 is not counted
global_test!(
	solve_global_cardinality_open,
	"global_cardinality",
	1,
	"global_cardinality(x, set2array(index_set(c))) = c",
	"array[1..3] of int: c = [1, 1, 1];\narray[1..4] of var 1..4: x;\n",
	"constraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) = c[v]);",
	"constraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) <= c[v]);",
);

global_test!(
	solve_global_cardinality_counts_from_samples,
	"global_cardinality",
	3,
	"global_cardinality_closed(x, [1, 2, 3]) = [1, 1, 2]",
	"array[1..4] of var 1..3: x;\n",
	"constraint sum(i in 1..4)(x[i] = 1) = 1 /\\ sum(i in 1..4)(x[i] = 2) = 1 /\\ sum(i in 1..4)(x[i] = 3) = 2;",
	"constraint sum(i in 1..4)(x[i] = 1) = 1 /\\ sum(i in 1..4)(x[i] = 2) <= 1;",
);

global_test!(
	solve_increasing,
	"increasing",
	1,
	"increasing(x)",
	"array[1..5] of var 1..9: x;\n",
	"constraint forall(i in 1..4)(x[i] <= x[i + 1]);",
	"constraint forall(i in 1..4 where i != 2)(x[i] <= x[i + 1]);",
);

global_test!(
	solve_inverse,
	"inverse",
	1,
	"inverse(f) = g",
	"array[1..3] of var 1..3: f;\narray[1..3] of var 1..3: g;\n",
	"constraint forall(i, j in 1..3)(f[i] = j <-> g[j] = i);",
	"constraint forall(i, j in 1..3 where i != 3)(f[i] = j <-> g[j] = i);",
);

global_test!(
	solve_lex_lesseq,
	"lex_",
	1,
	"lex_lesseq(x, y)",
	"array[1..3] of var 1..2: x;\narray[1..3] of var 1..2: y;\n",
	"constraint x[1] < y[1] \\/ (x[1] = y[1] /\\ (x[2] < y[2] \\/ (x[2] = y[2] /\\ x[3] <= y[3])));",
	"constraint x[1] < y[1] \\/ (x[1] = y[1] /\\ (x[2] < y[2] \\/ (x[2] = y[2] /\\ x[3] > y[3])));",
);

global_test!(
	solve_lex_less,
	"lex_",
	1,
	"lex_less(x, y)",
	"array[1..3] of var 1..2: x;\narray[1..3] of var 1..2: y;\n",
	"constraint x[1] < y[1] \\/ (x[1] = y[1] /\\ (x[2] < y[2] \\/ (x[2] = y[2] /\\ x[3] < y[3])));",
	"constraint x[1] < y[1] \\/ (x[1] = y[1] /\\ (x[2] < y[2] \\/ (x[2] = y[2] /\\ x[3] <= y[3])));",
);

global_test!(
	solve_lex2,
	"lex2",
	2,
	"lex2(m)",
	"array[1..3, 1..3] of var 0..1: m;\nconstraint forall(i in 1..2)(lex_lesseq(m[i, ..], m[i + 1, ..]));\n",
	"constraint forall(j in 1..2)(lex_lesseq(m[.., j], m[.., j + 1]));",
	"constraint forall(j in 1..1)(lex_lesseq(m[.., j], m[.., j + 1]));",
);

global_test!(
	solve_maximum,
	"maximum",
	2,
	"max(x) = m",
	"array[1..3] of var 1..4: x;\nvar 1..4: m;\n",
	"constraint forall(i in 1..3)(m >= x[i]);\nconstraint exists(i in 1..3)(m = x[i]);",
	// `m` can be larger than all of `x`
	"constraint forall(i in 1..3)(m >= x[i]);",
);

global_test!(
	solve_maximum_from_samples,
	"maximum",
	2,
	"max(x) = 7",
	"array[1..3] of var 1..9: x;\n",
	"constraint exists(i in 1..3)(x[i] = 7) /\\ forall(i in 1..3)(x[i] <= 7);",
	"constraint forall(i in 1..3)(x[i] <= 7);",
);

global_test!(
	solve_minimum,
	"minimum",
	2,
	"min(x) = m",
	"array[1..3] of var 1..4: x;\nvar 1..4: m;\n",
	"constraint forall(i in 1..3)(m <= x[i]);\nconstraint exists(i in 1..3)(m = x[i]);",
	"constraint forall(i in 1..3)(m <= x[i]);",
);

global_test!(
	solve_nvalue,
	"nvalue",
	1,
	"nvalue(x) = n",
	"array[1..4] of var 1..3: x;\nvar 0..4: n;\n",
	"constraint n = sum(v in 1..3)(exists(i in 1..4)(x[i] = v));",
	"constraint n = sum(v in 1..2)(exists(i in 1..4)(x[i] = v));",
);

global_test!(
	solve_sliding_sum,
	"sliding_sum",
	1,
	"sliding_sum(0, u, w, x)",
	"int: u = 3;\nint: w = 2;\narray[1..5] of var 0..3: x;\n",
	"constraint forall(i in 1..4)(sum(j in i..i + w - 1)(x[j]) <= u);",
	"constraint forall(i in 1..3)(sum(j in i..i + w - 1)(x[j]) <= u);",
);

global_test!(
	solve_sort,
	"sort",
	2,
	"sort(x) = y",
	"include \"globals.mzn\";\narray[1..3] of var 1..3: x;\narray[1..3] of var 1..3: y;\nconstraint increasing(y);\n",
	"constraint forall(v in 1..3)(count(x, v) = count(y, v));",
	"constraint y[1] <= x[1];",
);

global_test!(
	solve_value_precede,
	"value_precede",
	1,
	"value_precede(s, t, x)",
	"int: s = 1;\nint: t = 2;\narray[1..4] of var 1..3: x;\n",
	"constraint forall(i in 1..4)(x[i] = t -> exists(j in 1..i - 1)(x[j] = s));",
	"constraint forall(i in 1..4)(x[i] = s -> exists(j in 1..i - 1)(x[j] = t));",
);

global_test!(
	solve_arg_max,
	"arg_max",
	2,
	"arg_max(x) = i",
	"array[1..3] of var 1..4: x;\nvar 1..3: i;\n",
	"constraint forall(j in 1..3)(x[i] >= x[j]);\nconstraint forall(j in 1..3)(j < i -> x[j] < x[i]);",
	// Ties do not pick the first index
	"constraint forall(j in 1..3)(x[i] >= x[j]);",
);

global_test!(
	solve_element,
	"element",
	1,
	"x[i] = y",
	"array[1..3] of var 1..4: x;\nvar 1..3: i;\nvar 1..4: y;\n",
	"constraint forall(j in 1..3)(i = j -> y = x[j]);",
	"constraint forall(j in 1..2)(i = j -> y = x[j]);",
);

global_test!(
	solve_lex_chain_less,
	"lex_chain",
	1,
	"lex_chain_less(m)",
	"array[1..2, 1..3] of var 0..1: m;\n",
	"constraint forall(j in 1..2)(lex_less(m[.., j], m[.., j + 1]));",
	"constraint forall(j in 1..2)(lex_lesseq(m[.., j], m[.., j + 1]));",
);

global_test!(
	solve_member,
	"member",
	1,
	"member(x, y)",
	"array[1..3] of var 1..4: x;\nvar 1..4: y;\n",
	"constraint exists(i in 1..3)(x[i] = y);",
	"constraint exists(i in 1..2)(x[i] = y);",
);

global_test!(
	solve_strictly_increasing,
	"increasing",
	1,
	"strictly_increasing(x)",
	"array[1..4] of var 1..6: x;\n",
	"constraint forall(i in 1..3)(x[i] < x[i + 1]);",
	"constraint forall(i in 1..3)(x[i] <= x[i + 1]);",
);

global_test!(
	solve_symmetric_all_different,
	"symmetric",
	1,
	"symmetric_all_different(x)",
	"array[1..4] of var 1..4: x;\n",
	"constraint forall(i, j in 1..4)(x[i] = j <-> x[j] = i);",
	"constraint forall(i, j in 1..4 where i < j)(x[i] = j -> x[j] = i);",
);

global_test!(
	solve_all_different_of_enums,
	"all_different",
	1,
	"all_different(x)",
	"enum C = {R, G, B};\narray[1..3] of var C: x;\n",
	"constraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);",
	"constraint forall(i in 1..2)(x[i] != x[i + 1]);",
);

#[test]
fn solve_forms_the_group_already_uses_are_not_suggested() {
	let max = "array[1..3] of var 1..4: x;\nvar 1..4: m;\nconstraint m = max(x);";
	assert_eq!(solve(max, 1, &["maximum"]), Vec::<String>::new());
	let element = "array[1..3] of var 1..4: x;\nvar 1..3: i;\nvar 1..4: y;\nconstraint y = x[i];";
	assert_eq!(solve(element, 1, &["element"]), Vec::<String>::new());
}

#[test]
fn solve_subcircuit_of_three_values_is_all_different() {
	let found = solve(
		"array[1..3] of var 1..3: x;\nconstraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);",
		1,
		&["circuit", "all_different"],
	);
	assert_eq!(found, ["all_different(x)"]);
}

global_test!(
	solve_at_most_one_of_bools,
	"count",
	1,
	"count (i in index_set(b)) (b[i]) <= 1",
	"array[1..4] of var bool: b;\n",
	"constraint forall(i, j in 1..4 where i < j)(not b[i] \\/ not b[j]);",
	"constraint forall(i in 1..3)(not b[i] \\/ not b[i + 1]);",
);

global_test!(
	solve_all_disjoint_sets,
	"disjoint",
	1,
	"all_disjoint(s)",
	"array[1..3] of var set of 1..4: s;\n",
	"constraint forall(i, j in 1..3 where i < j)(s[i] intersect s[j] = {});",
	"constraint forall(i in 1..2)(s[i] intersect s[i + 1] = {});",
);

global_test!(
	solve_link_set_to_booleans,
	"link",
	1,
	"link_set_to_booleans(s, b)",
	"var set of 1..3: s;\narray[1..3] of var bool: b;\n",
	"constraint forall(i in 1..3)(b[i] <-> i in s);",
	"constraint forall(i in 1..2)(b[i] <-> i in s);",
);

global_test!(
	solve_int_set_channel,
	"channel",
	1,
	"int_set_channel(x, s)",
	"array[1..3] of var 1..2: x;\narray[1..2] of var set of 1..3: s;\n",
	"constraint forall(i in 1..3, j in 1..2)(x[i] = j <-> i in s[j]);",
	"constraint forall(i in 1..3, j in 1..2)(x[i] = j -> i in s[j]);",
);

global_test!(
	solve_among,
	"among",
	1,
	"count (i in index_set(x)) (x[i] in S) = k",
	"set of int: S = {1, 2};\nint: k = 2;\narray[1..4] of var 1..4: x;\n",
	"constraint sum(i in 1..4)(x[i] in S) = k;",
	"constraint sum(i in 1..3)(x[i] in S) = k;",
);

global_test!(
	solve_all_different_except,
	"except",
	1,
	"all_different_except(x, S)",
	"set of int: S = {0, 1};\narray[1..4] of var 0..4: x;\n",
	"constraint forall(i, j in 1..4 where i < j)(x[i] in S \\/ x[i] != x[j]);",
	"constraint forall(i, j in 1..4 where i < j)(x[i] = 0 \\/ x[i] != x[j]);",
);

global_test!(
	solve_all_different_of_opt,
	"all_different",
	1,
	"all_different(x)",
	"array[1..3] of var opt 1..3: x;\n",
	"constraint forall(i, j in 1..3 where i < j)(occurs(x[i]) /\\ occurs(x[j]) -> deopt(x[i]) != deopt(x[j]));",
	"constraint forall(i in 1..2)(occurs(x[i]) /\\ occurs(x[i + 1]) -> deopt(x[i]) != deopt(x[i + 1]));",
);

global_test!(
	solve_arg_sort,
	"arg_sort",
	2,
	"arg_sort(x) = p",
	"include \"all_different.mzn\";\narray[1..3] of var 1..4: x;\narray[1..3] of var 1..3: p;\n",
	"constraint all_different(p);\nconstraint forall(i in 1..2)(x[p[i]] < x[p[i + 1]] \\/ x[p[i]] = x[p[i + 1]] /\\ p[i] < p[i + 1]);",
	// Ties in any order
	"constraint all_different(p);\nconstraint forall(i in 1..2)(x[p[i]] <= x[p[i + 1]]);",
);

global_test!(
	solve_table,
	"table",
	1,
	"table(x, t)",
	"array[1..2, 1..3] of int: t = [|1, 2, 3|3, 2, 1|];\narray[1..3] of var 1..3: x;\n",
	"constraint exists(r in 1..2)(forall(j in 1..3)(x[j] = t[r, j]));",
	"constraint exists(r in 1..2)(forall(j in 1..2)(x[j] = t[r, j]));",
);

global_test!(
	solve_knapsack,
	"knapsack",
	2,
	"knapsack(w, p, x, tw, tp)",
	"array[1..3] of int: w = [2, 3, 4];\narray[1..3] of int: p = [1, 5, 2];\narray[1..3] of var 0..2: x;\nvar 0..20: tw;\nvar 0..20: tp;\n",
	"constraint tw = sum(i in 1..3)(w[i] * x[i]);\nconstraint tp = sum(i in 1..3)(p[i] * x[i]);",
	"constraint tw = sum(i in 1..3)(w[i] * x[i]);\nconstraint tp >= sum(i in 1..3)(p[i] * x[i]);",
);

global_test!(
	solve_value_precede_chain,
	"precede_chain",
	1,
	"value_precede_chain(c, x)",
	"array[1..3] of int: c = [1, 2, 3];\narray[1..4] of var 1..3: x;\n",
	"constraint forall(k in 1..2, i in 1..4)(x[i] = c[k + 1] -> exists(j in 1..i - 1)(x[j] = c[k]));",
	"constraint forall(k in 1..1, i in 1..4)(x[i] = c[k + 1] -> exists(j in 1..i - 1)(x[j] = c[k]));",
);

global_test!(
	solve_seq_precede_chain,
	"seq_precede",
	1,
	"seq_precede_chain(x)",
	"array[1..4] of var 1..3: x;\n",
	"constraint forall(k in 1..2, i in 1..4)(x[i] = k + 1 -> exists(j in 1..i - 1)(x[j] = k));",
	"constraint forall(k in 1..1, i in 1..4)(x[i] = k + 1 -> exists(j in 1..i - 1)(x[j] = k));",
);

global_test!(
	solve_sliding_among,
	"sliding_among",
	1,
	"sliding_among(lo, hi, n, x, S)",
	"int: lo = 1;\nint: hi = 1;\nint: n = 2;\nset of int: S = {1};\narray[1..5] of var 1..3: x;\n",
	"constraint forall(i in 1..5 - n + 1)(sum(j in i..i + n - 1)(x[j] in S) in lo..hi);",
	"constraint forall(i in 1..5 - n)(sum(j in i..i + n - 1)(x[j] in S) in lo..hi);",
);

global_test!(
	solve_disjunctive_strict,
	"disjunctive",
	1,
	"disjunctive_strict(s, d)",
	"array[1..3] of int: d = [0, 2, 1];\narray[1..3] of var 0..4: s;\n",
	"constraint forall(i, j in 1..3 where i < j)(s[i] + d[i] <= s[j] \\/ s[j] + d[j] <= s[i]);",
	"constraint forall(i, j in 1..3 where i < j)(d[i] = 0 \\/ d[j] = 0 \\/ s[i] + d[i] <= s[j] \\/ s[j] + d[j] <= s[i]);",
);

global_test!(
	solve_global_cardinality_between_bounds,
	"global_cardinality",
	1,
	"global_cardinality(x, set2array(index_set(lb)), lb, ub)",
	"array[1..3] of int: lb = [1, 0, 1];\narray[1..3] of int: ub = [2, 1, 2];\narray[1..4] of var 1..4: x;\n",
	"constraint forall(v in 1..3)(lb[v] <= sum(i in 1..4)(x[i] = v) /\\ sum(i in 1..4)(x[i] = v) <= ub[v]);",
	"constraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) <= ub[v]);",
);

global_test!(
	solve_all_equal_bools,
	"all_equal",
	1,
	"all_equal(b)",
	"array[1..4] of var bool: b;\n",
	"constraint forall(i in 1..3)(b[i] = b[i + 1]);",
	"constraint forall(i in 1..2)(b[i] = b[i + 1]);",
);

global_test!(
	solve_arg_val,
	"arg_val",
	2,
	"arg_val(x, v) = i",
	"array[1..3] of var 1..3: x;\nvar 1..3: v;\nvar 1..3: i;\n",
	"constraint x[i] = v /\\ forall(j in 1..3)(j < i -> x[j] != v);",
	"constraint x[i] = v;",
);

global_test!(
	solve_arg_val_weak,
	"arg_val",
	1,
	"arg_val_weak(x, v) = o",
	"array[1..3] of var 1..3: x;\nvar 1..3: v;\nvar opt 1..3: o;\n",
	"constraint if exists(j in 1..3)(x[j] = v) then occurs(o) /\\ x[deopt(o)] = v /\\ forall(j in 1..3)(j < deopt(o) -> x[j] != v) else absent(o) endif;",
	"constraint occurs(o) -> x[deopt(o)] = v;",
);
