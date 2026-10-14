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
fn normalise_keeps_multi_variable_generators() {
	check_parts(
		r#"
		array[1..3] of var 1..3: x;
		constraint forall(i, j in 1..3 where i < j)(x[i] != x[j]);
		"#,
		expect![[r#"
            forall(i, j in ((1) .. (3)) where ((i) < (j)))((((x[i])) != ((x[j]))))
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
			.flat_map(|p| part_decls(g.model, &g.parts[*p]))
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
	let args = with(src, |g| {
		let decls = part_decls(g.model, &g.parts[0]);
		g.arguments(&decls)
	});
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
	let preamble = with(src, |g| g.preamble(&part_decls(g.model, &g.parts[0])));
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
		let decls = part_decls(g.model, &g.parts[0]);
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
fn solve_unsatisfiable_group_is_skipped() {
	let src = r#"
		array[1..4] of var 1..3: x;
		constraint forall(i, j in 1..4 where i < j)(x[i] != x[j]);
	"#;
	assert!(solve(src, &[], group_of(1)).is_empty());
}
