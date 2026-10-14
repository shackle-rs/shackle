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
            "any: r = min(((1) .. (n)));\nany: c1 = min(((1) .. (n)));\nany: c2 = min(((1) .. (n)));\nany: s = r;\n",
            "any: r = max(((1) .. (n)));\nany: c1 = max(((1) .. (n)));\nany: c2 = max(((1) .. (n)));\nany: s = r;\n",
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
					g.replacement(
						u,
						&Candidate {
							name: "all_different",
							args: vec!["x[i, ..]".to_owned()],
							extra: None,
							leaders,
						},
					)
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
		replacement: "all_different(x)".to_owned(),
	};
	assert_eq!(
		finding.to_string(),
		"m.mzn|1|1|1|5;m.mzn|2|1|2|5 all_different(x)"
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
	assert_eq!(solve(src, &["n = 4;"], group_of(1)), ["all_different(x)"]);
}

#[test]
fn solve_rows_with_leaders() {
	let src = r#"
		int: n = 4;
		array[1..n, 1..n] of var 1..n: x;
		constraint forall(r in 1..n, c1, c2 in 1..n where c1 < c2)(x[r, c1] != x[r, c2]);
	"#;
	assert_eq!(
		solve(src, &[], group_of(1)),
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
