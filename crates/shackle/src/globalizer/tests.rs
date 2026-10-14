//! Globalizer tests. The unit tests need `MZN_STDLIB_DIR`, the `solve_*`
//! tests also need `minizinc` with Gecode on `PATH`.

use std::io::Write as _;

use expect_test::{Expect, expect};
use shackle_hir::run_hir_phase;
use shackle_syntax::InputLang;

use super::*;
use crate::Model;

fn with<R>(src: &str, f: impl FnOnce(&Globalizer<'_, '_>) -> R) -> R {
	let model = Model::from_string(src.to_owned(), InputLang::MiniZinc);
	let errors = run_hir_phase(&model.db).errors;
	assert!(errors.is_empty(), "{errors:?}");
	with_globalizer(&model.db, &[], &GlobalizeOptions::default(), f)
}

/// Run `f` on the group without unrolling
fn no_unrolling<R>(
	g: &Globalizer<'_, '_>,
	group: &[usize],
	f: impl FnOnce(&Unrolled<'_, '_, '_>) -> R,
) -> R {
	unrolled(
		g,
		group,
		&Variant {
			fixed: vec![Vec::new(); group.len()],
		},
		f,
	)
}

fn unrolled<R>(
	g: &Globalizer<'_, '_>,
	group: &[usize],
	variant: &Variant,
	f: impl FnOnce(&Unrolled<'_, '_, '_>) -> R,
) -> R {
	let mut decls = Vec::new();
	for d in group.iter().flat_map(|p| g.parts[*p].decls(g.model)) {
		if !decls.contains(&d) {
			decls.push(d);
		}
	}
	f(&Unrolled {
		parts: group.iter().map(|p| &g.parts[*p]).collect(),
		variant,
		context: None,
		decls,
	})
}

fn printed_parts(g: &Globalizer<'_, '_>) -> String {
	g.parts
		.iter()
		.map(|p| format!("{}\n", g.print_part(p)))
		.collect()
}

fn check_parts(src: &str, expected: Expect) {
	with(src, |g| expected.assert_eq(&printed_parts(g)));
}

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
		let all = (0..g.parts.len()).collect::<Vec<_>>();
		let decls = all
			.iter()
			.flat_map(|p| g.parts[*p].decls(g.model))
			.collect::<Vec<_>>();
		let mut text = g.preamble(&decls);
		for p in &g.parts {
			writeln!(text, "constraint {};", g.print_part(p)).unwrap();
		}
		text
	});
	let again = Model::from_string(printed.clone(), InputLang::MiniZinc);
	let errors = run_hir_phase(&again.db).errors;
	assert!(errors.is_empty(), "{printed}\n{errors:?}");
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
					let names = f
						.iter()
						.map(|l| g.name(g.parts[*p].loops[*l].decl))
						.collect::<Vec<_>>();
					format!("[{}]", names.join(" "))
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
fn variants_match_loops_of_other_parts_by_set() {
	let src = r#"
		int: n = 3;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n)(x[r, 1] > 0);
		constraint forall(a in 1..n, b in 1..n)(x[a, b] != 2);
		constraint forall(t in 1..2)(x[t, 2] > 0);
	"#;
	let variants = with(src, |g| {
		(printed_variants(g, &[0, 1]), printed_variants(g, &[0, 2]))
	});
	assert_eq!(variants.0, ["[][]", "[r][a]", "[r][b]"]);
	// `1..2` is not the same set as `1..n`
	assert_eq!(variants.1, ["[][]"]);
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
				let variant = Variant { fixed };
				unrolled(g, &[0, 1], &variant, |u| {
					let all = Globalizer::instantiations(u);
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
		unrolled(g, &[0], &variant, |u| {
			g.arguments(u)
				.iter()
				.map(|a| format!("{} var={} {:?} {:?}", a.text, a.var, a.ty, a.leaders))
				.collect::<Vec<_>>()
		})
	});
	// `c` is still a loop, so it is never an index
	assert_eq!(
		args,
		[
			"0 var=false Int(0) []",
			"n var=false Int(0) []",
			"x var=true Int(2) []",
			"k var=false Int(0) []",
			"w var=false Int(1) []",
			"x[r, ..] var=true Int(1) [0]",
			"x[k, ..] var=true Int(1) []",
			"w[r] var=false Int(0) [0]",
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
			[vec![], vec![0], vec![1]]
				.into_iter()
				.map(|leaders| {
					let mut cand = Candidate::raw("all_different", &["x[i, ..]"]);
					cand.leaders = leaders;
					g.replacement(u, &cand)
				})
				.collect::<Vec<_>>()
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

#[test]
fn arguments_come_from_the_group() {
	let src = r#"
		int: n = 3;
		array[1..n] of var 1..3: x;
		array[1..n] of int: p = [1, 2, 3];
		array[1..2, 1..2] of var 1..3: m;
		array[1..n] of var float: f;
		var 1..3: a;
		var 1..3: b;
		var 1..3: c;
		var opt 1..3: o;
		constraint x[1] + p[1] + m[1, 1] + a + b + c > f[1] + 1.0;
		constraint o != 1;
	"#;
	let args = with(src, |g| no_unrolling(g, &[0], |u| g.arguments(u)));
	let printed = args
		.iter()
		.map(|a| format!("{} var={} {:?}", a.text, a.var, a.ty))
		.collect::<Vec<_>>();
	assert_eq!(
		printed,
		[
			"0 var=false Int(0)",
			"x var=true Int(1)",
			"p var=false Int(1)",
			"m var=true Int(2)",
			"a var=true Int(0)",
			"b var=true Int(0)",
			"c var=true Int(0)",
			"[a, b, c] var=true Int(1)",
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
    int: n = 3;
    array [((1) .. (n))] of var ((1) .. (3)): x;
    var int: s = sum(x);
    function int: double(int: v) = ((2) * (v));
"#]]
	.assert_eq(&preamble);
}

#[test]
fn search_annotation_uses_group_int_variables() {
	let src = r#"
		array[1..2] of var 1..3: x;
		var 1..3: a;
		var bool: b;
		constraint b -> x[1] != a;
	"#;
	let search = with(src, |g| {
		let decls = g.parts[0].decls(g.model);
		(g.search(&decls, true), g.search(&decls, false))
	});
	assert_eq!(
		search.0,
		"solve :: int_search(array1d(x) ++ [a], input_order, indomain_random) :: restart_luby(1) satisfy;\n"
	);
	assert_eq!(
		search.1,
		"solve :: int_search(array1d(x) ++ [a], input_order, indomain_random) satisfy;\n"
	);
}

#[test]
fn locations_are_one_based_and_inclusive() {
	let src = "var 1..3: a;\nvar 1..3: b;\nconstraint a != b /\\ b > 1;\nconstraint forall(i in 1..2)(\n  a > i);\n";
	let locations = with(src, |g| {
		g.parts
			.iter()
			.map(|p| location(g.db, p.origin))
			.collect::<Vec<_>>()
	});
	assert_eq!(
		locations,
		[
			"<unnamed file>|3|12|3|17",
			"<unnamed file>|3|22|3|26",
			"<unnamed file>|4|1|5|8",
		]
	);
}

#[test]
fn finding_display_joins_locations() {
	let finding = Finding {
		locations: vec!["m.mzn|1|1|1|5".to_owned(), "m.mzn|2|1|2|5".to_owned()],
		context: None,
		replacement: "all_different(x)".to_owned(),
	};
	assert_eq!(
		finding.to_string(),
		"m.mzn|1|1|1|5;m.mzn|2|1|2|5 [ ] all_different(x)"
	);
}

// Solver integration tests

fn solve(src: &str, data: &[&str], options: GlobalizeOptions) -> Vec<String> {
	let dir = tempfile::tempdir().unwrap();
	let data = data
		.iter()
		.enumerate()
		.map(|(i, d)| {
			let path = dir.path().join(format!("data{i}.dzn"));
			std::fs::File::create(&path)
				.unwrap()
				.write_all(d.as_bytes())
				.unwrap();
			path
		})
		.collect::<Vec<_>>();
	let model = Model::from_string(src.to_owned(), InputLang::MiniZinc);
	model
		.globalize(&data, &options)
		.unwrap()
		.findings
		.into_iter()
		.map(|f| f.replacement)
		.collect()
}

fn group_of(max_group: usize) -> GlobalizeOptions {
	GlobalizeOptions {
		max_group,
		..Default::default()
	}
}

#[test]
fn solve_all_different_pairwise() {
	let src = r#"
		array[1..4] of var 1..5: x;
		constraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);
	"#;
	assert_eq!(solve(src, &[], group_of(1)), ["all_different(x)"]);
}

#[test]
fn solve_all_different_missing_pair() {
	let src = r#"
		array[1..4] of var 1..5: x;
		constraint forall(i, j in 1..4 where i < j /\ (i, j) != (1, 4))(x[i] != x[j]);
	"#;
	assert!(solve(src, &[], group_of(2)).is_empty());
}

#[test]
fn solve_all_different_of_scalars() {
	let src = r#"
		var 1..3: a;
		var 1..3: b;
		var 1..3: c;
		constraint a != b;
		constraint b != c;
		constraint a != c;
	"#;
	let found = solve(src, &[], group_of(3));
	assert!(
		found.contains(&"all_different([a, b, c])".to_owned()),
		"{found:?}"
	);
	// Two of the three disequalities are not enough
	assert_eq!(
		found
			.iter()
			.filter(|f| f.as_str() == "all_different([a, b, c])")
			.count(),
		1
	);
}

#[test]
fn solve_existing_global_is_vacuous() {
	let src = r#"
		include "all_different.mzn";
		array[1..4] of var 1..5: x;
		constraint all_different(x);
	"#;
	assert!(solve(src, &[], group_of(1)).is_empty());
}

#[test]
fn solve_uses_data_files() {
	let src = r#"
		int: n;
		array[1..n] of var 1..n: x;
		constraint forall(i, j in 1..n where i < j)(x[i] != x[j]);
	"#;
	let options = GlobalizeOptions {
		max_group: 1,
		include: vec!["all_different".to_owned()],
		..Default::default()
	};
	assert_eq!(solve(src, &["n = 4;"], options), ["all_different(x)"]);
}

#[test]
fn solve_rows_with_leaders() {
	let src = r#"
		int: n = 4;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);
	"#;
	assert_eq!(
		solve_only(src, "all_different", 1),
		["forall (r in 1..n) (all_different(x[r, ..]))"]
	);
}

#[test]
fn solve_rows_missing_a_pair_in_the_last_row() {
	// Only the last row misses a pair, so only the last leader value fails
	let src = r#"
		int: n = 4;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2 /\ (r, c1, c2) != (n, 1, 2))(x[r, c1] != x[r, c2]);
	"#;
	assert!(solve(src, &[], group_of(1)).is_empty());
}

#[test]
fn solve_increasing() {
	let src = r#"
		array[1..5] of var 1..9: x;
		constraint forall(i in 1..4)(x[i] <= x[i + 1]);
	"#;
	assert_eq!(solve(src, &[], group_of(1)), ["increasing(x)"]);
}

#[test]
fn solve_increasing_broken_chain() {
	let src = r#"
		array[1..5] of var 1..9: x;
		constraint forall(i in 1..4 where i != 2)(x[i] <= x[i + 1]);
	"#;
	assert!(solve(src, &[], group_of(1)).is_empty());
}

#[test]
fn solve_decreasing() {
	let src = r#"
		array[1..5] of var 1..9: x;
		constraint forall(i in 1..4)(x[i] >= x[i + 1]);
	"#;
	assert_eq!(solve(src, &[], group_of(1)), ["decreasing(x)"]);
}

#[test]
fn solve_unsatisfiable_group_is_skipped() {
	let src = r#"
		array[1..4] of var 1..3: x;
		constraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);
	"#;
	assert!(solve(src, &[], group_of(1)).is_empty());
}

// M3: the candidate table, contexts and filters

/// Findings (replacements) with only the globals whose name contains `filter`
fn solve_only(src: &str, filter: &str, max_group: usize) -> Vec<String> {
	solve(
		src,
		&[],
		GlobalizeOptions {
			max_group,
			include: vec![filter.to_owned()],
			..Default::default()
		},
	)
}

fn assert_found(found: &[String], expected: &str) {
	assert!(
		found.iter().any(|f| f == expected),
		"{expected} not in {found:?}"
	);
}

#[test]
fn shadowed_findings_are_hidden() {
	let found: Vec<RawFinding> = vec![
		(vec![0], None, "a".to_owned()),
		(vec![0, 1], None, "a".to_owned()),
		(vec![0], Some(1), "a".to_owned()),
		(vec![0, 1], None, "b".to_owned()),
		(vec![1], Some(0), "b".to_owned()),
		(vec![2], Some(0), "c".to_owned()),
		(vec![2], Some(1), "c".to_owned()),
	];
	let kept = shadow(&found);
	// 4 is hidden by 3: its context is part of the group of 3
	assert_eq!(kept, [&found[0], &found[3], &found[5], &found[6]]);
}

#[test]
fn finding_display_shows_context() {
	let mut finding = Finding {
		locations: vec!["m.mzn|1|1|1|5".to_owned()],
		context: None,
		replacement: "true".to_owned(),
	};
	assert_eq!(finding.to_string(), "m.mzn|1|1|1|5 [ ] true");
	finding.context = Some("m.mzn|2|1|2|5".to_owned());
	assert_eq!(finding.to_string(), "m.mzn|1|1|1|5 [ m.mzn|2|1|2|5 ] true");
}

#[test]
fn solve_bad_candidate_is_split_off() {
	let src = r#"
		array[1..4] of var 1..5: x;
		constraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);
	"#;
	let kept = with(src, |g| {
		no_unrolling(g, &[0], |u| {
			let mut base = g.preamble(&u.decls);
			writeln!(base, "constraint {};", g.print_part(&g.parts[0])).unwrap();
			let cands = vec![
				Candidate::raw("all_different", &["x"]),
				Candidate::raw("no_such_global", &["x"]),
				Candidate::raw("increasing", &["x"]),
				Candidate::raw("all_equal", &["x"]),
			];
			g.check_candidates(&base, u, None, cands).unwrap().unwrap()
		})
	});
	assert_eq!(kept, [Candidate::raw("all_different", &["x"])]);
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
			let base = g.preamble(&u.decls);
			let [a, b] = &g.print_group(u)[..] else {
				unreachable!()
			};
			(
				g.implies(&base, u, None, a, b).unwrap(),
				g.implies(&base, u, None, b, a).unwrap(),
			)
		})
	});
	assert!(forward);
	assert!(!backward);
}

#[test]
fn solve_context_is_reported() {
	let src = r#"
		array[1..3] of var 1..3: x;
		constraint x[1] <= x[2];
		constraint x[2] <= x[3];
	"#;
	let model = Model::from_string(src.to_owned(), InputLang::MiniZinc);
	let found = model
		.globalize(
			&[],
			&GlobalizeOptions {
				max_group: 1,
				include: vec!["increasing".to_owned()],
				..Default::default()
			},
		)
		.unwrap()
		.findings;
	let shown = found.iter().map(ToString::to_string).collect::<Vec<_>>();
	assert_eq!(
		shown,
		[
			"<unnamed file>|3|3|3|25 [ <unnamed file>|4|3|4|25 ] increasing(x)",
			"<unnamed file>|4|3|4|25 [ <unnamed file>|3|3|3|25 ] increasing(x)",
		]
	);
}

#[test]
fn solve_redundant_group_is_true() {
	let src = r#"
		array[1..3] of var 1..3: x;
		constraint x[1] >= 1;
		constraint x[2] >= 2;
	"#;
	let found = solve(src, &[], group_of(1));
	assert_eq!(found, ["true"]);
}

/// The test templates must accept exactly the solutions of the global
#[test]
fn solve_test_templates_match_the_globals() {
	let pairs = [
		("circuit(x)", "gz_circuit(x)"),
		("subcircuit(x)", "gz_subcircuit(x)"),
		(
			"value_precede(1, 2, x)",
			"forall (i in index_set(x)) (x[i] = 2 -> exists (j in index_set(x) where j < i) (x[j] = 1))",
		),
		("maximum(m, x)", "m = max(x)"),
		("minimum(m, x)", "m = min(x)"),
		(
			"sliding_sum(1, 5, 2, x)",
			"forall (i in index_set(x) where i + 2 - 1 <= max(index_set(x))) (sum (j in i..i + 2 - 1) (x[j]) in 1..5)",
		),
	];
	for (global, test) in pairs {
		let all = |c: &str| {
			let src = format!(
				"include \"globals.mzn\";{}\narray[1..4] of var 1..4: x;\nvar 1..4: m;\nconstraint {c};\nsolve satisfy;\n",
				super::candidates::LIBRARY
			);
			let dir = tempfile::tempdir().unwrap();
			let path = dir.path().join("m.mzn");
			std::fs::write(&path, src).unwrap();
			let out = std::process::Command::new("minizinc")
				.args(["--solver", "gecode", "-a"])
				.arg(&path)
				.output()
				.unwrap();
			assert!(
				out.status.success(),
				"{}",
				String::from_utf8_lossy(&out.stderr)
			);
			let mut sols = String::from_utf8(out.stdout)
				.unwrap()
				.split("----------")
				.map(str::trim)
				.filter(|s| !s.is_empty() && !s.starts_with('='))
				.map(str::to_owned)
				.collect::<Vec<_>>();
			sols.sort();
			sols
		};
		let expected = all(global);
		assert!(!expected.is_empty(), "{global}");
		assert_eq!(all(test), expected, "{test} differs from {global}");
	}
}

#[test]
fn solve_all_equal() {
	let src = "array[1..4] of var 1..3: x;\nconstraint forall(i in 1..3)(x[i] = x[i + 1]);";
	assert_found(&solve_only(src, "all_equal", 1), "all_equal(x)");
	let src = "array[1..4] of var 1..3: x;\nconstraint forall(i in 1..3)(x[i] <= x[i + 1]);";
	assert!(solve_only(src, "all_equal", 1).is_empty());
}

#[test]
fn solve_alldifferent_except_0() {
	let src = "array[1..4] of var 0..4: x;\nconstraint forall(i, j in 1..4 where i < j)(x[i] = 0 \\/ x[j] = 0 \\/ x[i] != x[j]);";
	assert_found(&solve_only(src, "except_0", 1), "alldifferent_except_0(x)");
	let src = "array[1..4] of var 0..4: x;\nconstraint forall(i, j in 1..4 where i < j /\\ (i, j) != (1, 4))(x[i] = 0 \\/ x[j] = 0 \\/ x[i] != x[j]);";
	assert!(solve_only(src, "except_0", 1).is_empty());
}

#[test]
fn solve_bin_packing_load() {
	let model = |items: &str| {
		format!(
			"array[1..3] of int: w = [2, 3, 4];\narray[1..3] of var 1..2: bin;\narray[1..2] of var 0..9: load;\nconstraint forall(b in 1..2)(load[b] = sum(i in {items})(w[i] * (bin[i] = b)));"
		)
	};
	assert_found(
		&solve_only(&model("1..3"), "bin_packing_load", 1),
		"bin_packing_load(load, bin, w)",
	);
	assert!(solve_only(&model("1..2"), "bin_packing_load", 1).is_empty());
}

#[test]
fn solve_bin_packing_capa() {
	let model = |cap: &str| {
		format!(
			"array[1..3] of int: w = [2, 3, 4];\narray[1..2] of int: c = [5, 7];\narray[1..3] of var 1..2: bin;\nconstraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= {cap});"
		)
	};
	assert_found(
		&solve_only(&model("c[b]"), "bin_packing_capa", 1),
		"bin_packing_capa(c, bin, w)",
	);
	assert!(solve_only(&model("c[b] + 1"), "bin_packing_capa", 1).is_empty());
}

#[test]
fn solve_bin_packing() {
	let model = |cap: &str| {
		format!(
			"array[1..3] of int: w = [2, 3, 4];\nint: c = 7;\narray[1..3] of var 1..2: bin;\nconstraint forall(b in 1..2)(sum(i in 1..3)(w[i] * (bin[i] = b)) <= {cap});"
		)
	};
	assert_found(
		&solve_only(&model("c"), "bin_packing", 1),
		"bin_packing(c, bin, w)",
	);
	assert!(
		!solve_only(&model("c - 1"), "bin_packing", 1)
			.contains(&"bin_packing(c, bin, w)".to_owned())
	);
}

#[test]
fn solve_count() {
	let model = |op: &str| {
		format!(
			"int: v = 2;\nint: c = 2;\narray[1..4] of var 1..3: x;\nconstraint sum(i in 1..4)(x[i] = v) {op} c;"
		)
	};
	assert_found(&solve_only(&model("="), "count", 1), "count(x, v, c)");
	assert!(solve_only(&model("<="), "count", 1).is_empty());
}

#[test]
fn solve_global_cardinality() {
	let model = |op: &str, c: &str| {
		format!(
			"array[1..3] of int: c = {c};\narray[1..4] of var 1..3: x;\nconstraint forall(v in 1..3)(sum(i in 1..4)(x[i] = v) {op} c[v]);"
		)
	};
	assert_found(
		&solve_only(&model("=", "[1, 1, 2]"), "global_cardinality", 1),
		"global_cardinality(x, set2array(index_set(c)), c)",
	);
	// Counts of at most 2 can vary
	assert!(solve_only(&model("<=", "[2, 2, 2]"), "global_cardinality", 1).is_empty());
}

#[test]
fn solve_global_cardinality_counts_from_samples() {
	let src = "array[1..4] of var 1..3: x;\nconstraint sum(i in 1..4)(x[i] = 1) = 1 /\\ sum(i in 1..4)(x[i] = 2) = 1 /\\ sum(i in 1..4)(x[i] = 3) = 2;";
	assert_found(
		&solve_only(src, "global_cardinality", 3),
		"global_cardinality(x, set2array(index_set([1, 1, 2])), [1, 1, 2])",
	);
}

#[test]
fn solve_cumulative() {
	let model = |b: &str| {
		format!(
			"array[1..3] of int: d = [2, 2, 2];\narray[1..3] of int: r = [1, 1, 1];\nint: b = 2;\narray[1..3] of var 0..4: s;\nconstraint forall(t in 0..6)(sum(i in 1..3)(r[i] * (s[i] <= t /\\ t < s[i] + d[i])) <= {b});"
		)
	};
	assert_found(
		&solve_only(&model("b"), "cumulative", 1),
		"cumulative(s, d, r, b)",
	);
	assert!(solve_only(&model("b + 1"), "cumulative", 1).is_empty());
}

#[test]
fn solve_disjunctive() {
	let model = |extra: &str| {
		format!(
			"array[1..3] of int: d = [2, 1, 2];\narray[1..3] of var 0..5: s;\nconstraint forall(i, j in 1..3 where i < j {extra})(s[i] + d[i] <= s[j] \\/ s[j] + d[j] <= s[i]);"
		)
	};
	assert_found(
		&solve_only(&model(""), "disjunctive", 1),
		"disjunctive(s, d)",
	);
	assert!(solve_only(&model("/\\ j != 3"), "disjunctive", 1).is_empty());
}

#[test]
fn solve_diffn() {
	let model = |cond: &str| {
		format!(
			"array[1..3] of int: w = [1, 2, 1];\narray[1..3] of int: h = [2, 1, 1];\narray[1..3] of var 0..3: x;\narray[1..3] of var 0..3: y;\nconstraint forall(i, j in 1..3 where i < j)({cond});"
		)
	};
	let full = "x[i] + w[i] <= x[j] \\/ x[j] + w[j] <= x[i] \\/ y[i] + h[i] <= y[j] \\/ y[j] + h[j] <= y[i]";
	assert_found(&solve_only(&model(full), "diffn", 1), "diffn(x, y, w, h)");
	let partial = "x[i] + w[i] <= x[j] \\/ x[j] + w[j] <= x[i] \\/ y[i] + h[i] <= y[j]";
	assert!(!solve_only(&model(partial), "diffn", 1).contains(&"diffn(x, y, w, h)".to_owned()));
}

#[test]
fn solve_distribute() {
	let model = |n: &str| {
		format!(
			"array[1..2] of int: value = [1, 3];\narray[1..4] of var 1..3: base;\narray[1..2] of var 0..4: card;\nconstraint forall(i in 1..2)(card[i] = sum(j in 1..{n})(base[j] = value[i]));"
		)
	};
	assert_found(
		&solve_only(&model("4"), "distribute", 1),
		"distribute(card, value, base)",
	);
	assert!(solve_only(&model("3"), "distribute", 1).is_empty());
}

#[test]
fn solve_inverse() {
	let model = |cond: &str| {
		format!(
			"array[1..3] of var 1..3: f;\narray[1..3] of var 1..3: g;\nconstraint forall(i, j in 1..3 where {cond})(f[i] = j <-> g[j] = i);"
		)
	};
	assert_found(&solve_only(&model("true"), "inverse", 1), "inverse(f, g)");
	assert!(solve_only(&model("i != 3"), "inverse", 1).is_empty());
}

#[test]
fn solve_lex_lesseq_and_lex_less() {
	let model = |last: &str| {
		format!(
			"array[1..3] of var 1..2: x;\narray[1..3] of var 1..2: y;\nconstraint x[1] < y[1] \\/ (x[1] = y[1] /\\ (x[2] < y[2] \\/ (x[2] = y[2] /\\ x[3] {last} y[3])));"
		)
	};
	assert_eq!(solve_only(&model("<="), "lex_", 1), ["lex_lesseq(x, y)"]);
	assert_eq!(solve_only(&model("<"), "lex_", 1), ["lex_less(x, y)"]);
	assert!(solve_only(&model(">"), "lex_", 1).is_empty());
}

#[test]
fn solve_lex2() {
	let model = |cols: &str| {
		format!(
			"array[1..3, 1..3] of var 0..1: m;\nconstraint forall(i in 1..2)(lex_lesseq(m[i, ..], m[i + 1, ..]));\nconstraint forall(j in {cols})(lex_lesseq(m[.., j], m[.., j + 1]));"
		)
	};
	assert_found(&solve_only(&model("1..2"), "lex2", 2), "lex2(m)");
	assert!(!solve_only(&model("1..1"), "lex2", 2).contains(&"lex2(m)".to_owned()));
}

#[test]
fn solve_maximum_and_minimum() {
	let model = |op: &str, f: &str| {
		format!(
			"array[1..3] of var 1..4: x;\nvar 1..4: m;\nconstraint forall(i in 1..3)(m {op} x[i]);\nconstraint {f}(i in 1..3)(m = x[i]);"
		)
	};
	assert_found(
		&solve_only(&model(">=", "exists"), "maximum", 2),
		"maximum(m, x)",
	);
	assert_found(
		&solve_only(&model("<=", "exists"), "minimum", 2),
		"minimum(m, x)",
	);
	// Without the second part, `m` can be larger than all of `x`
	let src = "array[1..3] of var 1..4: x;\nvar 1..4: m;\nconstraint forall(i in 1..3)(m >= x[i]);";
	assert!(solve_only(src, "maximum", 2).is_empty());
}

#[test]
fn solve_maximum_from_samples() {
	let src = "array[1..3] of var 1..9: x;\nconstraint exists(i in 1..3)(x[i] = 7) /\\ forall(i in 1..3)(x[i] <= 7);";
	assert_found(&solve_only(src, "maximum", 2), "maximum(7, x)");
}

#[test]
fn solve_nvalue() {
	let model = |vals: &str| {
		format!(
			"array[1..4] of var 1..3: x;\nvar 0..4: n;\nconstraint n = sum(v in {vals})(exists(i in 1..4)(x[i] = v));"
		)
	};
	assert_found(&solve_only(&model("1..3"), "nvalue", 1), "nvalue(n, x)");
	assert!(solve_only(&model("1..2"), "nvalue", 1).is_empty());
}

#[test]
fn solve_sliding_sum() {
	let model = |last: &str| {
		format!(
			"int: u = 3;\nint: w = 2;\narray[1..5] of var 0..3: x;\nconstraint forall(i in 1..{last})(sum(j in i..i + w - 1)(x[j]) <= u);"
		)
	};
	assert_found(
		&solve_only(&model("4"), "sliding_sum", 1),
		"sliding_sum(0, u, w, x)",
	);
	assert!(solve_only(&model("3"), "sliding_sum", 1).is_empty());
}

#[test]
fn solve_sort() {
	let model = |perm: &str| {
		format!(
			"include \"globals.mzn\";\narray[1..3] of var 1..3: x;\narray[1..3] of var 1..3: y;\nconstraint increasing(y);\nconstraint {perm};"
		)
	};
	assert_found(
		&solve_only(
			&model("forall(v in 1..3)(count(x, v) = count(y, v))"),
			"sort",
			2,
		),
		"sort(x, y)",
	);
	assert!(solve_only(&model("y[1] <= x[1]"), "sort", 2).is_empty());
}

#[test]
fn solve_value_precede() {
	let model = |s: &str, t: &str| {
		format!(
			"int: s = {s};\nint: t = {t};\narray[1..4] of var 1..3: x;\nconstraint forall(i in 1..4)(x[i] = t -> exists(j in 1..i - 1)(x[j] = s));"
		)
	};
	assert_found(
		&solve_only(&model("1", "2"), "value_precede", 1),
		"value_precede(s, t, x)",
	);
	assert!(
		!solve_only(&model("1", "2"), "value_precede", 1)
			.contains(&"value_precede(t, s, x)".to_owned())
	);
}

#[test]
fn solve_true() {
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 1;";
	assert_eq!(solve_only(src, "true", 1), ["true"]);
	let src = "array[1..3] of var 1..3: x;\nconstraint x[1] >= 2;";
	assert!(solve_only(src, "true", 1).is_empty());
}

#[test]
fn solve_all_different_dominates_its_equivalent_forms() {
	// Without the rule, `nvalue(n, x)` and `global_cardinality` with all
	// counts 1 are reported too
	let src = "int: n = 4;\narray[1..n] of var 1..n: x;\nconstraint forall(i, j in 1..n where i < j)(x[i] != x[j]);";
	assert_eq!(solve(src, &[], group_of(1)), ["all_different(x)"]);
}

#[test]
fn solve_leaders_never_give_true() {
	// For i = 1 and j = 8 the diagonal is rarely violated on samples
	let src = "int: n = 8;\narray[1..n] of var 1..n: q;\nconstraint forall(i, j in 1..n where i < j)(q[i] + i != q[j] + j);";
	assert!(!solve_only(src, "true", 1).contains(&"true".to_owned()));
}
