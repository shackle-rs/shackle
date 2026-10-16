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
/// Options for the solver tests: the initial pass (redundancy and
/// viewpoints) only runs in the tests for it, as it checks every group
fn options(max_group: usize, include: &[&str]) -> GlobalizeOptions {
	GlobalizeOptions {
		max_group,
		include: include.iter().map(|s| s.to_string()).collect(),
		jobs: 2,
		initial_pass: false,
		..Default::default()
	}
}

/// Options with the initial pass
fn initial(max_group: usize, include: &[&str]) -> GlobalizeOptions {
	GlobalizeOptions {
		initial_pass: true,
		..options(max_group, include)
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
	replacements(globalize(src, &[], &options(max_group, include)))
}

fn replacements(result: Globalized) -> Vec<String> {
	result.findings.into_iter().map(|f| f.replacement).collect()
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
fn renames_avoid_capture_of_argument_names() {
	check_parts(
		r#"
		int: i1 = 1;
		array[1..2, 1..3] of var 1..3: m;
		predicate pos(array[int] of var int: r) = forall(i in index_set(r))(r[i] > i1);
		constraint forall(i in 1..2)(pos(m[i, ..]));
		"#,
		expect![[r#"
    forall(i in ((1) .. (2)), i2 in index_set((m[i, ..])))(((((m[i, ..])[i2])) > (i1)))
"#]],
	);
}

#[test]
fn renames_avoid_capture_in_definitions_and_lets() {
	let src = r#"
		int: n = 1;
		array[1..2, 1..3] of var 1..3: m;
		array[1..3] of var 1..3: x;
		predicate pos(array[int] of var int: r) = forall(j in index_set(r))(r[j] > n);
		predicate low(array[int] of var int: r) = forall(k in index_set(r))(r[k] < 3);
		var bool: b = forall(n in 1..2)(pos(m[n, ..]));
		constraint b;
		constraint let { constraint forall(n in 1..2)(pos(m[n, ..])) } in true;
		constraint low([x[k] | k in 1..3]);
	"#;
	let printed = with(src, |g| {
		let decls = g.parts[0].decls(g.model);
		// The definition of `b`, and the parts
		let b = g
			.preamble(&decls)
			.lines()
			.filter(|l| l.contains(" b ="))
			.map(|l| format!("{l}\n"))
			.collect::<String>();
		let parts = g.parts.iter().map(|p| format!("{}\n", g.print_part(p)));
		std::iter::once(b).chain(parts).collect::<String>()
	});
	// The loop variables that would capture `n` are renamed, and `k` in the
	// argument is bound there, so the body keeps its name
	expect![[r#"
    var bool: b = forall([forall([((((m[n1, ..])[j])) > (n)) | j in index_set((m[n1, ..]))]) | n1 in ((1) .. (2))]);
    b
    let {
      constraint forall([forall([((((m[n1, ..])[j])) > (n)) | j in index_set((m[n1, ..]))]) | n1 in ((1) .. (2))]);
    } in true
    forall(k in index_set([(x[k]) | k in ((1) .. (3))]))(((([(x[k]) | k in ((1) .. (3))][k])) < (3)))
"#]].assert_eq(&printed);
}

#[test]
fn renames_avoid_enum_members_and_top_level_names() {
	check_parts(
		r#"
		enum E = {a, n1};
		enum F = {u};
		int: n = 1;
		array[1..2, 1..3] of var 1..3: m;
		array[E] of var 1..3: y;
		predicate pos(array[int] of var int: r) = forall(j in index_set(r))(r[j] > n);
		predicate p(var int: v) = exists(a in 1..3)(v = a);
		constraint forall(n in 1..2)(pos(m[n, ..]) /\ y[n1] > 1);
		constraint p(y[a]);
		constraint forall(u in 1..2)(m[u, 1] > 0);
		"#,
		expect![[r#"
    forall(n2 in ((1) .. (2)), j in index_set((m[n2, ..])))(((((m[n2, ..])[j])) > (n)))
    forall(n2 in ((1) .. (2)))((((y[n1])) > (1)))
    exists([(((y[a])) = (a1)) | a1 in ((1) .. (3))])
    forall(u1 in ((1) .. (2)))((((m[u1, 1])) > (0)))
"#]],
	);
}

#[test]
fn renamed_parameters_stay_distinct() {
	let src = r#"
		int: n = 1;
		function int: f(int: n, int: n1) = if n = 0 then 0 else f(n - 1, n1) endif;
		var 0..3: y;
		constraint y = f(2, 0);
	"#;
	let preamble = with(src, |g| g.preamble(&g.parts[0].decls(g.model)));
	// `n` has a top-level name, so it is renamed, skipping `n1`, which the
	// other parameter has
	assert!(preamble.contains("int: n2, int: n1"), "{preamble}");
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
	let solve = "solve :: seq_search([int_search(array1d(x) ++ [a], input_order, indomain_random), bool_search([occurs(gz_e) | gz_e in array1d(xo)], input_order, indomain_random), int_search(array1d(xo), input_order, indomain_random), bool_search([b], input_order, indomain_random), bool_search([gz_e in gz_s | gz_s in [s], gz_e in ub(gz_s)], input_order, indomain_random)])";
	assert_eq!(
		search,
		[
			format!("{solve} satisfy;\n"),
			format!("{solve} :: restart_luby(1) satisfy;\n"),
		]
	);
}

#[test]
fn replacements_loop_over_every_leader() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(i in 1..n, j in i + 1..n where j > 1)(x[i, j] > 0);
	"#;
	let (kept, replacement) = with(src, |g| {
		let variant = Variant {
			fixed: vec![vec![0, 1]],
		};
		unrolled(g, &[0], &variant, |u| {
			let plan = g.plan(u, Pass::Full).unwrap();
			// The set of `j` depends on `i`, so a candidate that uses `j`
			// loops over both
			let kept = plan.candidates.iter().all(|c| c.leaders.contains(&1));
			let cand = Candidate::raw("all_different", &["x[i, ..]"]);
			(kept, Checker::replacement(&plan, &cand))
		})
	});
	assert!(kept);
	assert_eq!(
		replacement,
		"forall (i in 1..n, j in i + 1..n where j > 1) (all_different(x[i, ..]))"
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
fn location_ends_at_the_last_character() {
	let src = SourceFile::new("m.mzn".into(), "x < yé;\n".to_owned());
	let span = SourceSpan::new(0.into(), "x < yé".len());
	assert_eq!(Location { src, span }.to_string(), "m.mzn|1|1|1|6");
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

/// Every table entry on small arguments of each type: the reported forms
/// compile, and each one implies the form checked on the samples (if that is
/// another expression)
#[test]
fn solve_table_entries_compile_and_agree() {
	use candidates::{Arg, Elem, candidates};
	let decls = r#"
		var 1..3: a;
		var 1..3: b;
		int: k = 2;
		array[1..3] of var 1..3: x;
		array[1..3] of var 1..3: y;
		array[1..3] of int: p = [1, 2, 3];
		array[1..2, 1..3] of var 1..3: m;
		array[1..2, 1..3] of int: t = [|1, 2, 3|3, 2, 1|];
		array[1..2, 1..2, 1..2] of var 0..1: z;
		var opt 1..3: o;
		array[1..3] of var opt 1..3: ox;
		var bool: c;
		array[1..3] of var bool: bs;
		array[1..3] of var bool: cs;
		array[1..2, 1..3] of var bool: bm;
		array[1..2, 1..3] of bool: bt = [|true, false, true|false, false, true|];
		var set of 1..3: s;
		set of int: S = {1, 2};
		array[1..3] of var set of 1..3: ss;
		array[1..3] of var set of 1..3: tt;
	"#;
	let arg = |text: &str, var, dims, elem| Arg {
		text: text.to_owned(),
		var,
		dims,
		elem,
		leaders: Vec::new(),
	};
	let pool = [
		arg("a", true, 0, Elem::Int),
		arg("b", true, 0, Elem::Int),
		arg("k", false, 0, Elem::Int),
		arg("x", true, 1, Elem::Int),
		arg("y", true, 1, Elem::Int),
		arg("p", false, 1, Elem::Int),
		arg("m", true, 2, Elem::Int),
		arg("t", false, 2, Elem::Int),
		arg("z", true, 3, Elem::Int),
		arg("o", true, 0, Elem::OptInt),
		arg("ox", true, 1, Elem::OptInt),
		arg("c", true, 0, Elem::Bool),
		arg("bs", true, 1, Elem::Bool),
		arg("cs", true, 1, Elem::Bool),
		arg("bm", true, 2, Elem::Bool),
		arg("bt", false, 2, Elem::Bool),
		arg("s", true, 0, Elem::Set),
		arg("S", false, 0, Elem::Set),
		arg("ss", true, 1, Elem::Set),
		arg("tt", true, 1, Elem::Set),
	];
	let cands = candidates(&pool, |_, _, _| true, usize::MAX)
		.into_iter()
		.filter(|c| c.blank.is_none())
		.collect::<Vec<_>>();
	let options = GlobalizeOptions::default();
	let runner = Runner::new(&options);
	let preamble = format!("include \"globals.mzn\";{}{decls}", candidates::LIBRARY);
	// All reified forms in one model
	let reified = cands
		.iter()
		.filter(|c| c.guard_and_test().1.is_none())
		.map(|c| c.checked())
		.join(",\n");
	let src =
		format!("{preamble}array[int] of var bool: r :: output = [\n{reified}];\nsolve satisfy;\n");
	let outcome = runner.run(&src, None, 1, false).unwrap();
	assert!(
		outcome.error.is_none() && !outcome.solutions.is_empty(),
		"{outcome:?}"
	);
	// Each form with another checked expression: no solution of the
	// reported form (where the guard holds) violates the checked one
	let checks = cands
		.iter()
		.filter_map(|c| {
			let (guard, test) = c.guard_and_test();
			let test = test?;
			let guard = guard.unwrap_or("true");
			Some((c.call(), format!(
				"{preamble}constraint {guard};\nconstraint {};\nvar bool: v :: output = not ({test});\nsolve maximize bool2int(v);\n",
				c.call()
			)))
		})
		.collect::<Vec<_>>();
	let failed = std::thread::scope(|scope| {
		let chunks = checks.chunks(checks.len().div_ceil(4)).collect::<Vec<_>>();
		chunks
			.into_iter()
			.map(|chunk| {
				scope.spawn(|| {
					chunk
						.iter()
						.filter(|(call, src)| {
							let outcome = runner.run(src, None, 1, false).unwrap();
							let violated = outcome.solutions.last().is_some_and(|s| {
								s.get("v") == Some(&serde_json::Value::Bool(true))
							});
							if outcome.error.is_some() || violated {
								eprintln!("{call}: {outcome:?}");
							}
							outcome.error.is_some() || violated
						})
						.map(|(call, _)| call.clone())
						.collect::<Vec<_>>()
				})
			})
			.collect::<Vec<_>>()
			.into_iter()
			.flat_map(|h| h.join().unwrap())
			.collect::<Vec<_>>()
	});
	assert!(failed.is_empty(), "{failed:?}");
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
	assert_eq!(solve(src, 1, &["all_different"]), ["all_different(x)"]);
}

#[test]
fn solve_all_different_missing_pair() {
	let src = "array[1..4] of var 1..5: x;\nconstraint forall(i, j in 1..4 where i < j /\\ (i, j) != (1, 4))(x[i] != x[j]);";
	assert!(solve(src, 2, &[]).is_empty());
}

#[test]
fn solve_all_different_of_scalars() {
	let src = "var 1..3: a;\nvar 1..3: b;\nvar 1..3: c;\nconstraint a != b;\nconstraint b != c;\nconstraint a != c;";
	let found = solve(src, 3, &["all_different"]);
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
	// equivalent, but not reported (the only test with all globals)
	let src = "int: n = 3;\narray[1..n] of var 1..n: x;\nconstraint forall(i, j in 1..n where i < j)(x[i] != x[j]);";
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
	assert!(solve(src, 1, &["all_different"]).is_empty());
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
	let src = "int: n = 3;\narray[1..n, 1..n] of var 1..n: x;\nconstraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2 /\\ (r, c1, c2) != (n, 1, 2))(x[r, c1] != x[r, c2]);";
	assert!(solve(src, 1, &["all_different"]).is_empty());
}

#[test]
fn solve_leaders_never_give_true() {
	// For i = 1 and j = 8 the diagonal is rarely violated on samples
	let src = "int: n = 8;\narray[1..n] of var 1..n: q;\nconstraint forall(i, j in 1..n where i < j)(q[i] + i != q[j] + j);";
	let found = replacements(globalize(src, &[], &initial(1, &["true"])));
	assert!(!found.contains(&"true".to_owned()));
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
	let solve = |src| replacements(globalize(src, &[], &initial(1, &[])));
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 1;\nconstraint x[2] >= 2;";
	assert_eq!(solve(src), ["true"]);
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 2;";
	assert!(solve(src).is_empty());
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
	assert_eq!(solve(src, 1, &["increasing"]), ["increasing(y)"]);
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
		array[1..3] of var 1..3: x;
		array[1..3] of var 1..3: y;
		constraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);
		constraint forall(i in 1..2)(y[i] <= y[i + 1]);
	"#;
	let found = |jobs| {
		let options = GlobalizeOptions {
			jobs,
			..options(1, &["all_different", "increasing"])
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
	let result = globalize(LATIN01, &[], &initial(1, &["all_different"]));
	// Each viewpoint, with the suggestions that use it
	let found = result
		.viewpoints
		.iter()
		.map(|v| {
			let declaration = v.suggestion.replacement.split(';').next().unwrap();
			let uses = v
				.findings
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>();
			(v.name.as_str(), declaration, uses)
		})
		.collect::<Vec<_>>();
	assert_eq!(
		found,
		[
			(
				"b_3b",
				"array [index_set_1of3(b), index_set_3of3(b)] of var index_set_2of3(b): b_3b",
				vec!["<unnamed file>|4|2|4|64 [ <unnamed file>|5|2|5|64 ] forall (i in 1..n) (all_different(b_3b[i, ..]))".to_owned()],
			),
			(
				"b_3c",
				"array [index_set_1of3(b), index_set_2of3(b)] of var index_set_3of3(b): b_3c",
				vec!["<unnamed file>|5|2|5|64 [ <unnamed file>|4|2|4|64 ] forall (i in 1..n) (all_different(b_3c[i, ..]))".to_owned()],
			),
		]
	);
	assert!(result.findings.is_empty());
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
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 1;\nconstraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);";
	assert_eq!(
		shown(&globalize(src, &[], &initial(1, &["all_different"]))),
		[
			"<unnamed file>|2|1|2|20 [ ] true",
			"<unnamed file>|3|1|3|57 [ ] all_different(x)",
		]
	);
}

// One test for each global in the table

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
	solve_global_cardinality_counts_from_samples,
	"global_cardinality",
	3,
	"global_cardinality_closed(x, [1, 2, 3]) = [1, 1, 2]",
	"array[1..4] of var 1..3: x;\n",
	"constraint sum(i in 1..4)(x[i] = 1) = 1 /\\ sum(i in 1..4)(x[i] = 2) = 1 /\\ sum(i in 1..4)(x[i] = 3) = 2;",
	"constraint sum(i in 1..4)(x[i] = 1) = 1 /\\ sum(i in 1..4)(x[i] = 2) <= 1;",
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
	solve_all_different_of_opt,
	"all_different",
	1,
	"all_different(x)",
	"array[1..3] of var opt 1..3: x;\n",
	"constraint forall(i, j in 1..3 where i < j)(occurs(x[i]) /\\ occurs(x[j]) -> deopt(x[i]) != deopt(x[j]));",
	"constraint forall(i in 1..2)(occurs(x[i]) /\\ occurs(x[i + 1]) -> deopt(x[i]) != deopt(x[i + 1]));",
);

#[test]
fn solve_all_different_with_unused_exceptions() {
	// No value of `x` is in `S`, so this is `all_different(x)`
	let src = "include \"all_different.mzn\";\nset of int: S = {9};\narray[1..3] of var 1..3: x;\nconstraint all_different(x, S);";
	assert_eq!(solve(src, 1, &["all_different"]), ["all_different(x)"]);
}

#[test]
fn solve_other_names_of_the_same_global_are_not_suggested() {
	let except_0 = "include \"all_different.mzn\";\narray[1..4] of var 0..4: x;\nconstraint all_different(x, {0});";
	assert_eq!(solve(except_0, 1, &["all"]), Vec::<String>::new());
	let greater = "include \"lex_greater.mzn\";\narray[1..3] of var 1..3: x;\narray[1..3] of var 1..3: y;\nconstraint lex_greater(x, y);";
	assert_eq!(solve(greater, 1, &["lex"]), Vec::<String>::new());
}
