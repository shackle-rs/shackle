//! Pretty-printed THIR snapshots of the object-syntax lowering: each test
//! pins the shape `lower_model` emits for an object model, with the standard
//! library available so builtins resolve.
//!
//! Class-analysis snapshots live in `shackle_hir::class_analysis`, and the
//! HIR-level object diagnostics (unsupported shapes, type errors, and the
//! not-fenced counterparts) live in `shackle_hir::object_validation` and
//! `shackle_hir::typecheck` — this file no longer covers them.

const SIMPLE_CLASS_REFERENCE: &str =
	include_str!("../tests/objects/fixtures/simple_class_reference.mzn");
const SIMPLE_NEW: &str = include_str!("../tests/objects/fixtures/simple_new.mzn");
const PAR_COMPUTED_ATTRIBUTE: &str =
	include_str!("../tests/objects/fixtures/par_computed_attribute.mzn");
const PAR_COMPUTED_CHAIN: &str = include_str!("../tests/objects/fixtures/par_computed_chain.mzn");
const PAR_COMPUTED_VAR_ATTR: &str =
	include_str!("../tests/objects/fixtures/par_computed_var_attr.mzn");
const PAR_NEW_OBJECT_COMPUTED: &str =
	include_str!("../tests/objects/fixtures/par_new_object_computed.mzn");
const PAR_SET_NEW_OBJECT_COMPUTED: &str =
	include_str!("../tests/objects/fixtures/par_set_new_object_computed.mzn");
const VAR_COMPUTED_ATTRIBUTE: &str =
	include_str!("../tests/objects/fixtures/var_computed_attribute.mzn");
const VAR_COMPUTED_SET_ATTR: &str =
	include_str!("../tests/objects/fixtures/var_computed_set_attr.mzn");
const VAR_SET_NEW_COMPUTED_CLASS_ATTR: &str =
	include_str!("../tests/objects/fixtures/var_set_new_computed_class_attr.mzn");
const VAR_NEW_COMPUTED_CLASS_ATTR: &str =
	include_str!("../tests/objects/fixtures/var_new_computed_class_attr.mzn");
const VAR_SET_NEW_COMPUTED_NONMONOTONE: &str =
	include_str!("../tests/objects/fixtures/var_set_new_computed_nonmonotone.mzn");
const VAR_SET_NEW_COMPUTED_BOUNDED_DOMAIN: &str =
	include_str!("../tests/objects/fixtures/var_set_new_computed_bounded_domain.mzn");
const VAR_SET_NEW_COMPUTED_DIV_KEEPS_GUARD: &str =
	include_str!("../tests/objects/fixtures/var_set_new_computed_div_keeps_guard.mzn");
const PAR_NESTED_COMPUTED_ATTR: &str =
	include_str!("../tests/objects/fixtures/par_nested_computed_attr.mzn");
const VAR_SET_NEW_NESTED_COMPUTED_ATTR: &str =
	include_str!("../tests/objects/fixtures/var_set_new_nested_computed_attr.mzn");
const VAR_SET_NEW_NESTED_COMPUTED_DIV_KEEPS_GUARD: &str =
	include_str!("../tests/objects/fixtures/var_set_new_nested_computed_div_keeps_guard.mzn");
const VAR_NEW_INHERITED_COMPUTED_ATTR: &str =
	include_str!("../tests/objects/fixtures/var_new_inherited_computed_attr.mzn");
const VAR_SET_NEW_NESTED_COMPUTED_BOUNDED_DOMAIN: &str =
	include_str!("../tests/objects/fixtures/var_set_new_nested_computed_bounded_domain.mzn");
const VAR_OPT_NEW_COMPUTED_ELIDE_AND_GUARD: &str =
	include_str!("../tests/objects/fixtures/var_opt_new_computed_elide_and_guard.mzn");
const VAR_OPT_NEW_COMPUTED_BOUNDED_DOMAIN: &str =
	include_str!("../tests/objects/fixtures/var_opt_new_computed_bounded_domain.mzn");
const VAR_OPT_NEW_NESTED_CARD_ABSENT: &str =
	include_str!("../tests/objects/fixtures/var_opt_new_nested_card_absent.mzn");
const SIMPLE_CLASS_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/simple_class_constraint.mzn");
const SELF_CLASS_CONSTRAINT_REFERENCE: &str =
	include_str!("../tests/objects/fixtures/self_class_constraint_reference.mzn");
const ITERATOR_FIELD_SET_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/iterator_field_set_field_access.mzn");
const TOP_LEVEL_SET_NEW: &str = include_str!("../tests/objects/fixtures/top_level_set_new.mzn");
const TOP_LEVEL_SET_NEW_MIXED_SCALAR: &str =
	include_str!("../tests/objects/fixtures/top_level_set_new_mixed_scalar.mzn");
const EMPTY_RECORD_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/empty_record_set_new.mzn");
const BOUNDED_SET_NEW: &str = include_str!("../tests/objects/fixtures/bounded_set_new.mzn");
const BOUNDED_SET_NEW_SYMMETRY_DEFAULTS: &str =
	include_str!("../tests/objects/fixtures/bounded_set_new_symmetry_defaults.mzn");
const BOUNDED_TWO_SETS_NEW: &str =
	include_str!("../tests/objects/fixtures/bounded_two_sets_new.mzn");
const MIXED_PAR_VAR_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/mixed_par_var_set_new.mzn");
const INHERITED_BOUNDED_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/inherited_bounded_set_new.mzn");
const INHERITED_BOUNDED_SET_SUPERCLASS_ALIAS: &str =
	include_str!("../tests/objects/fixtures/inherited_bounded_set_superclass_alias.mzn");
const INHERITED_BOUNDED_SET_SUPERCLASS_SET_ALIAS: &str =
	include_str!("../tests/objects/fixtures/inherited_bounded_set_superclass_set_alias.mzn");
const OPTIONAL_NEW: &str = include_str!("../tests/objects/fixtures/optional_new.mzn");
const INHERITANCE: &str = include_str!("../tests/objects/fixtures/inheritance.mzn");
const INHERITED_PAR_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/inherited_par_field_access.mzn");
const SUPERCLASS_PAR_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/superclass_par_field_access.mzn");
const SUPERCLASS_VAR_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/superclass_var_field_access.mzn");
const REFERENCE_CYCLE_DEOPT_SET_FIELD: &str =
	include_str!("../tests/objects/fixtures/reference_cycle_deopt_set_field.mzn");
const ARRAY_FIELD_VAR_INDEX_PAR: &str =
	include_str!("../tests/objects/fixtures/array_field_var_index_par.mzn");
const INHERITED_CLASS_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/inherited_class_constraint.mzn");
const NESTED_NEW: &str = include_str!("../tests/objects/fixtures/nested_new.mzn");
const NESTED_VAR_NEW_NO_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/nested_var_new_no_constraint.mzn");
const NESTED_VAR_OPT_NEW_NO_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/nested_var_opt_new_no_constraint.mzn");
const NESTED_PAR_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/nested_par_field_access.mzn");
const INHERITED_NESTED_NEW: &str =
	include_str!("../tests/objects/fixtures/inherited_nested_new.mzn");
const NESTED_BOUNDED_SET_CLASS_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_class_constraint.mzn");
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_under_bounded_root.mzn");
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_FIELD_ACCESS: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_field_access.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_CARDINALITY_CHANNELING: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_cardinality_channeling.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_FIELDS_SAME_CLASS: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_two_fields_same_class.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_PARENT_MEMBERSHIP_CHANNELING: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_parent_membership_channeling.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_PATH_PARENT_EXCLUSIVITY: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_two_path_parent_exclusivity.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_ALIAS_TWO_PATH_PARENT_EXCLUSIVITY: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_alias_two_path_parent_exclusivity.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_FILTERED_ALIAS_OWNERSHIP: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_filtered_alias_ownership.mzn"
);
const NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_FILTERED_ALIASES: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_under_bounded_root_two_filtered_aliases.mzn"
);
const NESTED_BOUNDED_SET_ALIAS: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_alias.mzn");
const NESTED_BOUNDED_SET_FIELD_ACCESS: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_field_access.mzn");
const NESTED_BOUNDED_SET_TWO_ROOTS: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_two_roots.mzn");
const NESTED_BOUNDED_SET_TWO_ROOTS_SHARED_ALIAS: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_two_roots_shared_alias.mzn");
const NESTED_BOUNDED_SET_TWO_ROOTS_SHARED_ALIAS_FIELD_ACCESS: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_set_two_roots_shared_alias_field_access.mzn"
);
const NESTED_BOUNDED_SET_TWO_ROOTS_COMPOSITE_CONSUMERS: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_two_roots_composite_consumers.mzn");
const NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_ALIAS_FIELD_ACCESS: &str = include_str!(
	"../tests/objects/fixtures/nested_inherited_bounded_set_two_roots_superclass_alias_field_access.mzn"
);
const NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_CLASS_CONSTRAINT: &str = include_str!(
	"../tests/objects/fixtures/nested_inherited_bounded_set_two_roots_superclass_class_constraint.mzn"
);
const NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_COMPOSITE_CONSUMERS: &str = include_str!(
	"../tests/objects/fixtures/nested_inherited_bounded_set_two_roots_superclass_composite_consumers.mzn"
);
const NESTED_BOUNDED_SET_TWO_ROOTS_CLASS_CONSTRAINT: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_set_two_roots_class_constraint.mzn");
const NESTED_BOUNDED_PAR_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/nested_bounded_par_set_new.mzn");
const NESTED_BOUNDED_PAR_SET_UNDER_VAR_ROOT_FIELD_ACCESS: &str = include_str!(
	"../tests/objects/fixtures/nested_bounded_par_set_under_var_root_field_access.mzn"
);
const NESTED_PAR_SET_NEW: &str = include_str!("../tests/objects/fixtures/nested_par_set_new.mzn");
const NESTED_PAR_SET_NEW_MIXED_SCALAR: &str =
	include_str!("../tests/objects/fixtures/nested_par_set_new_mixed_scalar.mzn");
const NESTED_PAR_SET_TWO_ROOTS: &str =
	include_str!("../tests/objects/fixtures/nested_par_set_two_roots.mzn");
const DEEP_NESTED_PAR_SET_TWO_ROOTS: &str =
	include_str!("../tests/objects/fixtures/deep_nested_par_set_two_roots.mzn");
const REPEATED_NESTED_PAR_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/repeated_nested_par_set_new.mzn");
const NESTED_INHERITED_PAR_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/nested_inherited_par_set_new.mzn");
const NESTED_INHERITED_PAR_SET_NEW_MIXED_SCALAR: &str =
	include_str!("../tests/objects/fixtures/nested_inherited_par_set_new_mixed_scalar.mzn");
const NESTED_INHERITED_CHILD_PAR_SET_NEW: &str =
	include_str!("../tests/objects/fixtures/nested_inherited_child_par_set_new.mzn");

use std::panic::AssertUnwindSafe;

use expect_test::{Expect, expect};
use salsa::Setter;
use shackle_hir::{
	CompilerDatabase,
	input::{CompilerSettings, InlineModelFile, InputFiles, ModelFile},
	run_hir_phase,
};
use shackle_syntax::InputLang;

use crate::{
	lower::lower_model,
	pretty_print::PrettyPrinter,
	transform::{tests::NameMapper, thir_transforms},
};

/// HIR-phase errors attributable to the test's own inline model.
fn user_hir_errors(db: &CompilerDatabase) -> Vec<&shackle_diagnostics::Error> {
	run_hir_phase(db).errors
}

fn db_for(source: &str) -> CompilerDatabase {
	let mut db = CompilerDatabase::default();
	let _ = CompilerSettings::get(&db)
		.set_ignore_stdlib(&mut db)
		.to(true);
	let file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc);
	let _ = InputFiles::get(&db)
		.set_files(&mut db)
		.to(vec![file.into()]);
	db
}

fn db_for_with_stdlib(source: &str) -> CompilerDatabase {
	let mut db = CompilerDatabase::default();
	let file = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc);
	let _ = InputFiles::get(&db)
		.set_files(&mut db)
		.to(vec![file.into()]);
	db
}

/// Pretty print the raw lowered THIR (no transforms), without the standard
/// library. Pins the shape the object lowering itself emits.
fn check_thir_model(source: &str, expected: Expect) {
	let db = db_for(source);
	let model = lower_model(&db);
	let pretty = PrettyPrinter::new(&db, model.get().as_ref()).pretty_print();
	expected.assert_eq(&pretty);
}

/// Pretty print the THIR after the full transform pipeline, from `anchor`
/// (the first user item) to the end — the preceding standard-library items
/// are elided.
fn check_thir_model_with_stdlib(source: &str, expected: Expect) {
	expected.assert_eq(&user_items_pretty(source));
}

/// Pretty print the user model's own top-level items as produced by the
/// object lowering itself, with NO downstream transforms (no totalisation,
/// enum/record/opt erasure, output generation, etc.). This pins the shape the
/// object lowering emits — the only thing these tests are meant to verify —
/// rather than the end of the whole THIR pipeline. Standard-library items are
/// filtered out by origin, and unnamed introduced declarations are renamed
/// `_DECL_n` so the snapshot stays stable across standard-library changes.
fn user_items_pretty(source: &str) -> String {
	let mut db = CompilerDatabase::default();
	let model_file: ModelFile =
		InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
	let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
	let mut model = lower_model(&db).take();
	let to_print = NameMapper::default().run(&db, model_file, &mut model);
	let printer = PrettyPrinter::new(&db, &model);
	let mut pretty = String::new();
	for item in to_print {
		pretty.push_str(&printer.pretty_print_item(item));
		pretty.push_str(";\n");
	}
	pretty
}

// A par class with a computed (RHS) attribute (`int: y = x + 1`). The computed
// attribute is excluded from the class input record (so `new A: a = (x: 1)`
// type-checks) and *defined* in the storage reconstruction comprehension via a
// generator alias (`y = x + 1`), the only valid form for par storage. The
// residual `forall(this in A)(this.y = x + 1)` is now a redundant (true) par
// constraint.
#[test]
fn object_par_class_computed_attribute_compiles() {
	check_thir_model_with_stdlib(
		PAR_COMPUTED_ATTRIBUTE,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(int: x, int: y): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(int: x): A_a_inputs = [(x: 1)];
    array [int] of record(int: x, int: y): A_a_objects = [(x: x, y: y) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), x = (input).x, y = '+'(x, 1)];
    A_potential: a = A_occ_0(1);
    solve satisfy;
"#]),
	);
}

// A chain of computed attributes (`z = y + 4` depends on `y = x + 1`): the
// reconstruction comprehension emits the aliases in declaration order so each
// computed attribute can reference the previous one.
#[test]
fn object_par_class_computed_attribute_chain_compiles() {
	check_thir_model_with_stdlib(
		PAR_COMPUTED_CHAIN,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: x, int: y, int: z): A_objects = A_a_objects;
    set of A_potential: A = a;
    array [int] of record(int: x): A_a_inputs = [(x: 3), (x: 5)];
    int: A_a_root_start = 1;
    int: A_a_root_end = 3;
    array [int] of record(int: x, int: y, int: z): A_a_objects = [(x: x, y: y, z: z) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), x = (input).x, y = '+'(x, 1), z = '+'(y, 4)];
    set of A_occ_0({1, 2}): a = A_occ_0({1, 2});
    output ["xs=", show([('[]'(A_objects, enum2int(p))).x | p in a]), "\n", "ys=", show([('[]'(A_objects, enum2int(p))).y | p in a]), "\n", "zs=", show([('[]'(A_objects, enum2int(p))).z | p in a]), "\n"];
    solve satisfy;
"#]),
	);
}

// A var attribute whose declared domain depends on a computed attribute
// (`var 1..z: s`): the storage record element type carries `var int: s` and the
// per-object bound is minted in the reconstruction via `let { var 1..z: .. }`.
#[test]
fn object_par_class_computed_var_attr_compiles() {
	check_thir_model_with_stdlib(
		PAR_COMPUTED_VAR_ATTR,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(var int: s, int: x, int: y, int: z): A_objects = A_a_objects;
    set of A_potential: A = a;
    array [int] of record(int: x): A_a_inputs = [(x: 3), (x: 5)];
    int: A_a_root_start = 1;
    int: A_a_root_end = 3;
    array [int] of record(var int: s, int: x, int: y, int: z): A_a_objects = [(s: s, x: x, y: y, z: z) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), x = (input).x, y = '+'(x, 1), z = '+'(y, 4), s = let {
      var '..'(1, z): s_init;
    } in s_init];
    set of A_occ_0({1, 2}): a = A_occ_0({1, 2});
    constraint forall(['>='(('[]'(A_objects, enum2int(p))).s, ('[]'(A_objects, enum2int(p))).x) | p in a]);
    output ["xs=", show([('[]'(A_objects, enum2int(p))).x | p in a]), "\n", "ys=", show([('[]'(A_objects, enum2int(p))).y | p in a]), "\n", "zs=", show([('[]'(A_objects, enum2int(p))).z | p in a]), "\n", "ss=", show([('[]'(A_objects, enum2int(p))).s | p in a]), "\n"];
    solve satisfy;
"#]),
	);
}

// A PAR singular root whose class has an object-typed field (`set of new B:
// children`) plus a computed attribute over it (`n = card(children)`). Before
// the engine fold this shape fresh-minted `n` as a valueless par decl
// (`int: n_init;` — invalid MiniZinc) and encoded `children` as a
// `set2array(..)` int array that mismatched the forall's `set of B_potential`
// alias. The engine identity-mints `children` as a `<B>_potential` ordinal
// range and alias-defines `n = card(children)` against it.
#[test]
fn object_par_new_object_computed_compiles() {
	check_thir_model_with_stdlib(
		PAR_NEW_OBJECT_COMPUTED,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, sum([length((i).children) | i in A_a_inputs])));
    array [int] of record('..'(2, 3): x): B_objects = B_children_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_children_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: children, int: n): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(int: x): children): A_a_inputs = [(children: [(x: 2)])];
    constraint forall(['in'(length((i).children), '..'(1, 1)) | i in A_a_inputs]);
    array [int] of record('..'(2, 3): x): B_children_objects = [k | i in A_a_inputs, k in (i).children];
    int: A_a_children_start = 1;
    int: A_a_children_end = '+'(A_a_children_start, sum([length((i).children) | i in A_a_inputs]));
    array [int] of record(set of B_potential: children, int: n): A_a_objects = [(children: children, n: n) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), children = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).children) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).children) | q in '..'(1, '-'(p, 1))]), length((input).children)))), n = card(children)];
    A_potential: a = A_occ_0(1);
    constraint '='(('[]'(A_objects, enum2int(a))).n, 1);
    output ["n=", show(('[]'(A_objects, enum2int(a))).n), "\n"];
    solve satisfy;
"#]),
	);
}

// The par SET-root sibling of the test above: `set(1..2) of new A` where each
// input record supplies a different number of children. `children` is minted
// per parent from the prefix-sum ordinal range and `n = card(children)` is
// alias-defined — the lowered model must carry no valueless `n_init`.
#[test]
fn object_par_set_new_object_computed_compiles() {
	check_thir_model_with_stdlib(
		PAR_SET_NEW_OBJECT_COMPUTED,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, sum([length((i).children) | i in A_as_inputs])));
    array [int] of record('..'(2, 3): x): B_objects = B_children_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_as_children_end, 1)))]);
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(set of B_potential: children, int: n): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(array [int] of record(int: x): children): A_as_inputs = [(children: [(x: 2)]), (children: [(x: 2), (x: 3)])];
    constraint forall(['in'(length((i).children), '..'(1, 2)) | i in A_as_inputs]);
    array [int] of record('..'(2, 3): x): B_children_objects = [k | i in A_as_inputs, k in (i).children];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    int: A_as_children_start = 1;
    int: A_as_children_end = '+'(A_as_children_start, sum([length((i).children) | i in A_as_inputs]));
    array [int] of record(set of B_potential: children, int: n): A_as_objects = [(children: children, n: n) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), children = B_occ_1('..'('+'(1, sum([length(('[]'(A_as_inputs, q)).children) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_as_inputs, q)).children) | q in '..'(1, '-'(p, 1))]), length((input).children)))), n = card(children)];
    set( '..'(1, 2) ) of  A_occ_0({1, 2}): as = A_occ_0({1, 2});
    output ["ns=", show([('[]'(A_objects, enum2int(a))).n | a in as]), "\n"];
    solve satisfy;
"#]),
	);
}

// A computed attribute on a PAR-NESTED class: B is introduced
// via `A.children`, not at a root. The nested contribution previously took
// the template path and fresh-minted `y` as a valueless par decl
// (`let { int: y_init; } in y_init` — invalid MiniZinc). It now runs the
// engine over the caller's element iteration: `y = x + 1` alias-defined
// against the input-read `x`, and B's class-body forall is dropped (the
// contribution registers determined).
#[test]
fn object_par_nested_computed_attr_compiles() {
	check_thir_model_with_stdlib(
		PAR_NESTED_COMPUTED_ATTR,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, sum([length((i).children) | i in A_a_inputs])));
    array [int] of record('..'(2, 3): x, int: y): B_objects = B_children_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_children_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: children): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(int: x): children): A_a_inputs = [(children: [(x: 2), (x: 3)])];
    constraint forall(['in'(length((i).children), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record('..'(2, 3): x, int: y): B_children_objects = [(x: x, y: y) | i in A_a_inputs, k in (i).children, x = (k).x, y = '+'(x, 1)];
    int: A_a_children_start = 1;
    int: A_a_children_end = '+'(A_a_children_start, sum([length((i).children) | i in A_a_inputs]));
    array [int] of record(set of B_potential: children): A_a_objects = [(children: children) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), children = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).children) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).children) | q in '..'(1, '-'(p, 1))]), length((input).children))))];
    A_potential: a = A_occ_0(1);
    output ["ys=", show([('[]'(B_objects, enum2int(b))).y | b in ('[]'(A_objects, enum2int(a))).children]), "\n"];
    solve satisfy;
"#]),
	);
}

// A computed attribute on a VAR-REACHED NESTED class: B under
// a `var set of new A` root. Previously registered uninitialized FULL-record
// storage (computed `y` a free decision) and panicked during lowering. Now
// the free decisions live in `B_children_storage` (free record, enum-image
// dim) and `B_children_objects` is the engine reconstruction over it with
// the slot-identity realisation test `p in B` — elided for `y = x + 1`
// (total RHS, non-binding domain), so no guard appears.
#[test]
fn object_var_set_new_nested_computed_attr_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_NESTED_COMPUTED_ATTR,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))));
    array [int] of record(var '..'(2, 3): x, var int: y): B_objects = B_children_objects;
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).children else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, 1));
    array [int] of record(var set of B_potential: children): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(2, 2)) | this in A]);
    array ['..'(1, 1)] of record(var set of B_potential: children): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))))] of record(var '..'(2, 3): x): B_children_storage;
    array [int] of record(var '..'(2, 3): x, var int: y): B_children_objects = [(x: x, y: y) | p in B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2))))), input = '[]'(B_children_storage, p), x = (input).x, y = '+'(x, 1)];
    int: A_as_children_start = 1;
    int: A_as_children_end = '+'(A_as_children_start, '*'(card(A_potential), max('..'(2, 2))));
    array [int] of set of B_potential: A_as_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(2, 2)))), '*'(_DECL_2, max('..'(2, 2))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: children): A_as_objects = A_as_storage;
    var set( '..'(0, 1) ) of  A_occ_0('..'(1, 1)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).children), enum2int('[]'(A_as_children_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint '='(card(as), 1);
    output ["ys=", show([fix(('[]'(B_objects, enum2int(b))).y) | a in as, b in ('[]'(A_objects, enum2int(a))).children]), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).children, lb(('[]'(A_objects, enum2int(x))).children))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

// Domain relocation on a NESTED slot: the binding-domain
// binding-domain + total-RHS shape on a var-reached nested class takes the
// relocation encoding — the element record's `z` relaxed to unbounded, the
// declared 3..4 re-imposed as the realised-set invariant
// `forall(this in B)(this.z in 3..4)`, and the alias unguarded.
#[test]
fn object_var_set_new_nested_computed_bounded_domain_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_NESTED_COMPUTED_BOUNDED_DOMAIN,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))));
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): B_objects = B_children_objects;
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).children else {} endif | _DECL_1 in index_set(A_as_objects)]);
    constraint forall(['in'(('[]'(B_objects, enum2int(this))).z, '..'(3, 4)) | this in B]);
    enum A_potential = A_occ_0('..'(1, 1));
    array [int] of record(var set of B_potential: children): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(2, 2)) | this in A]);
    array ['..'(1, 1)] of record(var set of B_potential: children): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))))] of record(var '..'(0, 2): x1, var '..'(0, 2): x2): B_children_storage;
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): B_children_objects = [(x1: x1, x2: x2, z: z) | p in B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2))))), input = '[]'(B_children_storage, p), x1 = (input).x1, x2 = (input).x2, z = '+'(x1, x2)];
    int: A_as_children_start = 1;
    int: A_as_children_end = '+'(A_as_children_start, '*'(card(A_potential), max('..'(2, 2))));
    array [int] of set of B_potential: A_as_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(2, 2)))), '*'(_DECL_2, max('..'(2, 2))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: children): A_as_objects = A_as_storage;
    var set( '..'(0, 1) ) of  A_occ_0('..'(1, 1)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).children), enum2int('[]'(A_as_children_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint '='(card(as), 0);
    output ["card=", show(card(as)), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).children, lb(('[]'(A_objects, enum2int(x))).children))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x1, mzn_safe_default(('[]'(B_objects, enum2int(x))).x1))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x2, mzn_safe_default(('[]'(B_objects, enum2int(x))).x2))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

// 2.5(a) elision on a NESTED slot: `y = 6 div (x + 1)` is not on the
// totality whitelist, so the nested engine contribution KEEPS the value
// guard — witness decl, pin, `realised = p in B` alias, if-then-else —
// exactly like the root shape.
#[test]
fn object_var_set_new_nested_computed_div_keeps_guard_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_NESTED_COMPUTED_DIV_KEEPS_GUARD,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))));
    array [int] of record(var '..'(0, 5): x, var '..'(1, 6): y): B_objects = B_children_objects;
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).children else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, 1));
    array [int] of record(var set of B_potential: children): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(2, 2)) | this in A]);
    array ['..'(1, 1)] of record(var set of B_potential: children): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))))] of record(var '..'(0, 5): x): B_children_storage;
    var '..'(1, 6): B_children_y_unrealised_default;
    constraint '='(B_children_y_unrealised_default, mzn_safe_default(B_children_y_unrealised_default));
    array [int] of record(var '..'(0, 5): x, var '..'(1, 6): y): B_children_objects = [(x: x, y: y) | p in B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2))))), input = '[]'(B_children_storage, p), realised = 'in'(p, B), x = (input).x, y = if realised then 'div'(6, '+'(x, 1)) else B_children_y_unrealised_default endif];
    int: A_as_children_start = 1;
    int: A_as_children_end = '+'(A_as_children_start, '*'(card(A_potential), max('..'(2, 2))));
    array [int] of set of B_potential: A_as_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(2, 2)))), '*'(_DECL_2, max('..'(2, 2))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: children): A_as_objects = A_as_storage;
    var set( '..'(0, 1) ) of  A_occ_0('..'(1, 1)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).children), enum2int('[]'(A_as_children_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint '='(card(as), 0);
    output ["card=", show(card(as)), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).children, lb(('[]'(A_objects, enum2int(x))).children))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

// A singular `var new` root of a subclass whose SUPERCLASS
// declares a computed attribute. The superclass projection
// (`S_T_t_objects`) now reads every field — including the alias-defined
// `c` — from the direct-class objects array instead of fresh-minting from
// the raw inputs, inherits its determined flag, and S's class-body forall
// is dropped.
#[test]
fn object_var_new_inherited_computed_attr_compiles() {
	check_thir_model_with_stdlib(
		VAR_NEW_INHERITED_COMPUTED_ATTR,
		expect!([r#"
    enum T_potential = T_occ_0({1});
    array [int] of record(var '..'(0, 3): b, var int: c, var '..'(0, 2): d): T_objects = T_t_objects;
    set of T_potential: T = T_occ_0({1});
    enum S_potential = S_occ_0({1});
    array [int] of record(var '..'(0, 3): b, var int: c): S_objects = S_T_t_objects;
    set of S_potential: S = array_union([{S_occ_0(_DECL_1)} | _DECL_1 in index_set(T_t_objects)]);
    array [{1}] of record(var '..'(0, 3): b, var '..'(0, 2): d): T_t_storage;
    int: T_t__start = 1;
    int: T_t__end = 2;
    array [int] of record(var '..'(0, 3): b, var int: c, var '..'(0, 2): d): T_t_objects = [(b: b, c: c, d: d) | p in index_set(T_t_storage), input = '[]'(T_t_storage, p), b = (input).b, c = '+'(b, 1), d = (input).d];
    array [int] of record(var '..'(0, 3): b, var int: c): S_T_t_objects = [(b: ('[]'(T_t_objects, p)).b, c: ('[]'(T_t_objects, p)).c) | p in index_set(T_t_objects)];
    T_potential: t = T_occ_0(1);
    constraint '='(('[]'(T_objects, enum2int(t))).b, 2);
    constraint '='(('[]'(T_objects, enum2int(t))).d, 1);
    output ["cs=", show([fix(('[]'(S_objects, enum2int(s))).c) | s in S]), " d=", show(fix(('[]'(T_objects, enum2int(t))).d)), "\n"];
    solve satisfy;
"#]),
	);
}

// A var-reached class (`var new C`) with a computed (RHS) attribute
// (`c = b + 1`). The computed attribute should be *defined* by the storage
// reconstruction alias chain (like par), not left as a free `_storage`
// decision pinned by the class-body forall.
#[test]
fn object_var_class_computed_attribute_compiles() {
	check_thir_model_with_stdlib(
		VAR_COMPUTED_ATTRIBUTE,
		expect!([r#"
    enum C_potential = C_occ_0({1});
    array [int] of record(var '..'(0, 9): b, var '..'(0, 10): c): C_objects = C_obj_objects;
    set of C_potential: C = C_occ_0({1});
    array [{1}] of record(var '..'(0, 9): b): C_obj_storage;
    array [int] of record(var '..'(0, 9): b, var '..'(0, 10): c): C_obj_objects = [(b: b, c: c) | p in index_set(C_obj_storage), input = '[]'(C_obj_storage, p), b = (input).b, c = '+'(b, 1)];
    C_potential: obj = C_occ_0(1);
    constraint '='(('[]'(C_objects, enum2int(obj))).b, 4);
    output ["b=", show(('[]'(C_objects, enum2int(obj))).b), " c=", show(('[]'(C_objects, enum2int(obj))).c), "\n"];
    solve satisfy;
"#]),
	);
}

// A var-reached class with a computed *set* attribute (`var set of int:
// z = {x, 2*x}`). A free `var set of int` decision is illegal, so this field
// must be realised as a reconstruction alias — `z` is excluded from the free
// `_storage` array and *defined* in the comprehension.
#[test]
fn object_var_class_computed_set_attr_compiles() {
	check_thir_model_with_stdlib(
		VAR_COMPUTED_SET_ATTR,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 4): x, var set of int: z): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var '..'(0, 4): x): A_a_storage;
    array [int] of record(var '..'(0, 4): x, var set of int: z): A_a_objects = [(x: x, z: z) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), x = (input).x, z = {x, '*'(2, x)}];
    A_potential: a = A_occ_0(1);
    constraint '='(('[]'(A_objects, enum2int(a))).x, 3);
    output ["x=", show(('[]'(A_objects, enum2int(a))).x), " z=", show(('[]'(A_objects, enum2int(a))).z), "\n"];
    solve satisfy;
"#]),
	);
}

// A `var set of new A` root whose class has a class-typed field (`set of new B:
// children`) plus a computed attribute over it (`int: nChildren =
// card(children)`). The set-of-new path used its free `_storage` array directly
// as `<A>_objects`, but the computed field is excluded from the free array
// (`free_storage_record_ty`), so `<A>_objects` was missing `nChildren` and every
// downstream `.nChildren` access panicked. The set root now reconstructs the
// full storage record (computed field minted + pinned by the class-body forall,
// symmetric with the singular `var new` path) whenever a storage field is
// missing from the free element type. `<A>_as_objects` carries both `children`
// (read from storage) and `nChildren` (fresh `nChildren_init`).
#[test]
fn object_var_set_new_computed_class_attr_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_COMPUTED_CLASS_ATTR,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, 3)))));
    array [int] of record(var '..'(0, 3): x): B_objects = array1d(B_children_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).children else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, 1));
    array [int] of record(var set of B_potential: children, var int: nChildren): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall([let {
      var set of B_potential: children = ('[]'(A_objects, enum2int(this))).children;
    } in '='(card(children), 2) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(0, 3)) | this in A]);
    array ['..'(1, 1)] of record(var set of B_potential: children): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, 3)))))] of record(var '..'(0, 3): x): B_children_objects;
    int: A_as_children_start = 1;
    int: A_as_children_end = '+'(A_as_children_start, '*'(card(A_potential), max('..'(0, 3))));
    array [int] of set of B_potential: A_as_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, 3)))), '*'(_DECL_2, max('..'(0, 3))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: children, var int: nChildren): A_as_objects = [(children: children, nChildren: nChildren) | p in index_set(A_as_storage), input = '[]'(A_as_storage, p), children = (input).children, nChildren = card(children)];
    var set( '..'(1, 1) ) of  A_occ_0('..'(1, 1)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).children), enum2int('[]'(A_as_children_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint '='(card(as), 1);
    output ["ns=", show([fix(('[]'(A_objects, enum2int(a))).nChildren) | a in as]), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).children, lb(('[]'(A_objects, enum2int(x))).children))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

// A NON-monotone computed attribute (`y = 5 - x`) on a `var set of new` root.
// The symmetry-break wave must not pin alias-defined fields: the alias gives
// `y = 5` at the pinned `x = 0` on unrealised slots, while the old pin
// demanded `y = mzn_safe_default(y) = 0`, forcing every potential realised and
// making `card(as) = 0` UNSAT. The lowered model must carry a pin for the free
// `x` but NONE for the defined `y`.
#[test]
fn object_var_set_new_computed_nonmonotone_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_COMPUTED_NONMONOTONE,
		expect!([r#"
    enum A_potential = A_occ_0('..'(1, 2));
    array [int] of record(var '..'(0, 5): x, var int: y): A_objects = A_as_objects;
    var set of A_potential: A = as;
    array ['..'(1, 2)] of record(var '..'(0, 5): x): A_as_storage;
    array [int] of record(var '..'(0, 5): x, var int: y): A_as_objects = [(x: x, y: y) | p in index_set(A_as_storage), input = '[]'(A_as_storage, p), x = (input).x, y = '-'(5, x)];
    var set( '..'(0, 2) ) of  A_occ_0('..'(1, 2)): as;
    constraint '='(card(as), 0);
    output ["card=", show(card(as)), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

// A computed
// attribute with a BINDING declared domain (`var 3..4: z = x1 + x2`) on a
// `var set of new` root. The declared domain is enforced on ALL slots by the
// `<C>_objects` element record type, so without the realisation guard an
// unrealised slot's pinned frees (x1 = x2 = 1 after back-propagation) give
// `z = 2 ∉ 3..4` and `card(as) = 0` was UNSAT. The alias must be guarded:
// `z = if realised then x1 + x2 else <in-domain default> endif`, with one
// hoisted `realised = A_occ_0(p) in A` per slot and a pinned top-level
// witness carrying the declared domain.
#[test]
fn object_var_set_new_computed_bounded_domain_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_COMPUTED_BOUNDED_DOMAIN,
		expect!([r#"
    enum A_potential = A_occ_0('..'(1, 2));
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(('[]'(A_objects, enum2int(this))).z, '..'(3, 4)) | this in A]);
    array ['..'(1, 2)] of record(var '..'(0, 2): x1, var '..'(0, 2): x2): A_as_storage;
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): A_as_objects = [(x1: x1, x2: x2, z: z) | p in index_set(A_as_storage), input = '[]'(A_as_storage, p), x1 = (input).x1, x2 = (input).x2, z = '+'(x1, x2)];
    var set( '..'(0, 2) ) of  A_occ_0('..'(1, 2)): as;
    constraint '='(card(as), 0);
    output ["card=", show(card(as)), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x1, mzn_safe_default(('[]'(A_objects, enum2int(x))).x1))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x2, mzn_safe_default(('[]'(A_objects, enum2int(x))).x2))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

// A computed attribute whose RHS is NOT on the totality whitelist (`div`) on
// a `var set of new` root: the 2.5(a) elision analysis must NOT fire. The
// value guard REMAINS — witness decl + pin, per-slot `realised` alias, and
// the `y = if realised then ... else <witness> endif` form.
#[test]
fn object_var_set_new_computed_div_keeps_guard_compiles() {
	check_thir_model_with_stdlib(
		VAR_SET_NEW_COMPUTED_DIV_KEEPS_GUARD,
		expect!([r#"
    enum A_potential = A_occ_0('..'(1, 2));
    array [int] of record(var '..'(0, 5): x, var '..'(1, 6): y): A_objects = A_as_objects;
    var set of A_potential: A = as;
    array ['..'(1, 2)] of record(var '..'(0, 5): x): A_as_storage;
    var '..'(1, 6): A_as_y_unrealised_default;
    constraint '='(A_as_y_unrealised_default, mzn_safe_default(A_as_y_unrealised_default));
    array [int] of record(var '..'(0, 5): x, var '..'(1, 6): y): A_as_objects = [(x: x, y: y) | p in index_set(A_as_storage), input = '[]'(A_as_storage, p), realised = 'in'(A_occ_0(p), A), x = (input).x, y = if realised then 'div'(6, '+'(x, 1)) else A_as_y_unrealised_default endif];
    var set( '..'(0, 2) ) of  A_occ_0('..'(1, 2)): as;
    constraint '='(card(as), 0);
    output ["card=", show(card(as)), "\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

// Singular `var opt new` root, one field per elision outcome: `s = 5 - x`
// (total, no domain) must lose its guard and witness entirely, while
// `y = 6 div (x + 1)` keeps its guard — and with it the shared `realised`
// alias.
#[test]
fn object_var_opt_new_computed_elide_and_guard_compiles() {
	check_thir_model_with_stdlib(
		VAR_OPT_NEW_COMPUTED_ELIDE_AND_GUARD,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(var int: s, var '..'(0, 5): x, var '..'(1, 6): y): A_objects = A_a_objects;
    var set of A_potential: A;
    array [{1}] of record(var '..'(0, 5): x): A_a_storage;
    var '..'(1, 6): A_a_y_unrealised_default;
    constraint '='(A_a_y_unrealised_default, mzn_safe_default(A_a_y_unrealised_default));
    array [int] of record(var int: s, var '..'(0, 5): x, var '..'(1, 6): y): A_a_objects = [(s: s, x: x, y: y) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), realised = 'in'(A_occ_0(p), A), x = (input).x, s = '-'(5, x), y = if realised then 'div'(6, '+'(x, 1)) else A_a_y_unrealised_default endif];
    var opt A_potential: a = if 'in'(A_occ_0(1), A) then A_occ_0(1) else <> endif;
    constraint absent(a);
    output ["absent ok\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

// Total RHS with a binding declared domain on the singular `var opt new`
// arm. The domain relocation applies — unguarded alias, `var int: z`
// (relaxed) in the element record, and the realised-set invariant
// `forall(this in A)(this.z in 3..4)` instead of a value guard.
#[test]
fn object_var_opt_new_computed_bounded_domain_compiles() {
	check_thir_model_with_stdlib(
		VAR_OPT_NEW_COMPUTED_BOUNDED_DOMAIN,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): A_objects = A_a_objects;
    var set of A_potential: A;
    constraint forall(['in'(('[]'(A_objects, enum2int(this))).z, '..'(3, 4)) | this in A]);
    array [{1}] of record(var '..'(0, 2): x1, var '..'(0, 2): x2): A_a_storage;
    array [int] of record(var '..'(0, 2): x1, var '..'(0, 2): x2, var int: z): A_a_objects = [(x1: x1, x2: x2, z: z) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), x1 = (input).x1, x2 = (input).x2, z = '+'(x1, x2)];
    var opt A_potential: a = if 'in'(A_occ_0(1), A) then A_occ_0(1) else <> endif;
    constraint absent(a);
    output ["absent ok\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x1, mzn_safe_default(('[]'(A_objects, enum2int(x))).x1))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x2, mzn_safe_default(('[]'(A_objects, enum2int(x))).x2))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

// A `var opt new` root with a nested exact-cardinality `set(2..2) of new B`
// field. The invariant must iterate the REALISED class set (`this in A`), not
// potential storage — the storage-iterating form constrained the possibly-
// unrealised slot (children defaults to `{}`, card 0 not in 2..2) and made
// `absent(a)` UNSAT.
#[test]
fn object_var_opt_new_nested_card_absent_compiles() {
	check_thir_model_with_stdlib(
		VAR_OPT_NEW_NESTED_CARD_ABSENT,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))));
    array [int] of record(var '..'(2, 3): x): B_objects = array1d(B_children_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_a_objects, _DECL_1)).children else {} endif | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: children): A_objects = A_a_objects;
    var set of A_potential: A;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(2, 2)) | this in A]);
    array [{1}] of record(var set of B_potential: children): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(2, 2)))))] of record(var '..'(2, 3): x): B_children_objects;
    int: A_a_children_start = 1;
    int: A_a_children_end = '+'(A_a_children_start, '*'(card(A_potential), max('..'(2, 2))));
    array [int] of set of B_potential: A_a_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(2, 2)))), '*'(_DECL_2, max('..'(2, 2))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: children): A_a_objects = [(children: children) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), children = (input).children];
    var opt A_potential: a = if 'in'(A_occ_0(1), A) then A_occ_0(1) else <> endif;
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).children), enum2int('[]'(A_a_children_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    constraint absent(a);
    output ["absent ok\n"];
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).children, lb(('[]'(A_objects, enum2int(x))).children))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

// The singular `var new A` counterpart of the test above: a class-typed field
// (`set of new B: children`) plus a computed attribute (`int: nChildren =
// card(children)`). A's class-typed field is a `<B>_potential` identity held in
// its free `_storage`, so the alias chain reads it straight through and
// *defines* `nChildren = card(children)` in `A_a_objects` — no free
// `nChildren_init` decision. Unifies the singular object-field path with the
// scalar and set-of-new computed paths.
#[test]
fn object_var_new_computed_class_attr_compiles() {
	check_thir_model_with_stdlib(
		VAR_NEW_COMPUTED_CLASS_ATTR,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, 3)))));
    array [int] of record(var '..'(0, 3): x): B_objects = array1d(B_children_objects);
    var set of B_potential: B = array_union([('[]'(A_a_objects, _DECL_1)).children | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: children, var int: nChildren): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall([let {
      var set of B_potential: children = ('[]'(A_objects, enum2int(this))).children;
    } in '='(card(children), 2) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).children), '..'(0, 3)) | this in A]);
    array [{1}] of record(var set of B_potential: children): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, 3)))))] of record(var '..'(0, 3): x): B_children_objects;
    int: A_a_children_start = 1;
    int: A_a_children_end = '+'(A_a_children_start, '*'(card(A_potential), max('..'(0, 3))));
    array [int] of set of B_potential: A_a_children_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, 3)))), '*'(_DECL_2, max('..'(0, 3))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: children, var int: nChildren): A_a_objects = [(children: children, nChildren: nChildren) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), children = (input).children, nChildren = card(children)];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).children), enum2int('[]'(A_a_children_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    output ["n=", show(fix(('[]'(A_objects, enum2int(a))).nChildren)), "\n"];
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

/// A VAR-REACHED object-carrying class introduced two or more
/// `new`-hops below a par root now lowers (the last cross-introduction fence is
/// gone). The class is deep under a par root (so its data-supplied object
/// fields are par-reconstructed as `<GrandChild>_potential` identity ranges by
/// the flat 1-D prefix-sum deep builder) AND var-reached from elsewhere (so its
/// storage is var and its var-existence object fields mint free `var set`/`var
/// opt` decisions). The var-actual-set machinery `++`s the deep par
/// contribution with the var contribution into the class's var storage — the
/// same composition depth-1 var-reached nesting already uses. This used to be
/// fenced (`validate_mixed_introductions`, now removed) out of a stale concern
/// that the deep builder would store inline records where the var identity
/// model is expected; it actually mints identities. The only remaining
/// varification rejection is the unbounded `set of new` attribute of a varified
/// class (`object_negative_var_reached_set_of_new_needs_cardinality`), which is
/// a genuine "no finite storage universe" type error, independent of depth.
#[test]
fn object_thir_lowering_par_root_deep_nested_var_reached_object_snapshot() {
	// (1) Var-reached (`var new C`) deep class via singular edges:
	// A.b (new B) -> B.c (new C), C owns `ds` two hops below the par root.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( set(1..1) of new D: ds; );
    class B ( new C: c; );
    class A ( new B: b; );
    new A: a = (b: (c: (ds: [(v: 2)])));
    var new C: c2;
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_3('..'(1, sum([length((((i).b).c).ds) | i in A_a_inputs]))) ++ D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 1)))));
    array [int] of record(var '..'(2, 3): v): D_objects = '++'(D_b_c_ds_objects, D_ds_objects);
    var set of D_potential: D = array_union('++'([D_occ_3('..'(1, '-'(A_a_b_c_ds_end, 1)))], [('[]'(C_c2_objects, _DECL_1)).ds | _DECL_1 in index_set(C_c2_objects)]));
    enum C_potential = C_occ_2('..'(1, '*'(card(B_potential), 1))) ++ C_occ_4({1});
    array [int] of record(var set of D_potential: ds): C_objects = '++'(C_b_c_objects, C_c2_objects);
    set of C_potential: C = array_union('++'([C_occ_4({1})], [{C_occ_2(_DECL_2)} | _DECL_2 in index_set(B_b_objects)]));
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 1)) | this in C]);
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(C_potential: c): B_objects = B_b_objects;
    set of B_potential: B = array_union([{B_occ_1(_DECL_3)} | _DECL_3 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(B_potential: b): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(record(record(array [int] of record(int: v): ds): c): b): A_a_inputs = [(b: (c: (ds: [(v: 2)])))];
    array [int] of record(C_potential: c): B_b_objects = [(c: c) | p in index_set(A_a_inputs), input = ('[]'(A_a_inputs, p)).b, c = C_occ_2(p)];
    array [int] of record(var set of D_potential: ds): C_b_c_objects = let {
      array [int] of record(array [int] of record(int: v): ds): b_c_flat_inputs = [((i).b).c | i in A_a_inputs];
    } in [(ds: ds) | ci in index_set(b_c_flat_inputs), input = '[]'(b_c_flat_inputs, ci), ds = D_occ_3('..'('+'(1, sum([length(('[]'(b_c_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))])), '+'(sum([length(('[]'(b_c_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))]), length((input).ds))))];
    array [int] of record('..'(2, 3): v): D_b_c_ds_objects = [k | i in A_a_inputs, k in (((i).b).c).ds];
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    int: A_a_b_c_start = 1;
    int: A_a_b_c_end = '+'(A_a_b_c_start, '*'(card(B_potential), 1));
    int: A_a_b_c_ds_start = 1;
    int: A_a_b_c_ds_end = '+'(A_a_b_c_ds_start, sum([length((((i).b).c).ds) | i in A_a_inputs]));
    array [int] of record(B_potential: b): A_a_objects = [(b: b) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), b = B_occ_1(p)];
    A_potential: a = A_occ_0(1);
    array [{1}] of record(var set of D_potential: ds): C_c2_storage;
    array [D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 1)))))] of record(var '..'(2, 3): v): D_ds_objects;
    int: C_c2_ds_start = A_a_b_c_ds_end;
    int: C_c2_ds_end = '+'(C_c2_ds_start, '*'(card(C_potential), max('..'(1, 1))));
    array [int] of set of D_potential: C_c2_ds_potential = [_DECL_4: D_occ_5('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(1, 1)))), '*'(_DECL_4, max('..'(1, 1))))) | _DECL_4 in index_set(C_c2_storage)];
    array [int] of record(var set of D_potential: ds): C_c2_objects = [(ds: ds) | p in index_set(C_c2_storage), input = '[]'(C_c2_storage, p), ds = (input).ds];
    C_potential: c2 = C_occ_4(1);
    constraint forall(['subset'(enum2int(('[]'(C_c2_objects, _DECL_5)).ds), enum2int('[]'(C_c2_ds_potential, _DECL_5))) | _DECL_5 in index_set(C_c2_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// (2) Var-reached deep class via all-set edges.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( set(1..1) of new D: ds; );
    class B ( set(1..1) of new C: cs; );
    class A ( set(1..1) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(v: 2)])])]);
    var new C: c2;
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_3('..'(1, sum([length((j2).ds) | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs]))) ++ D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 1)))));
    array [int] of record(var '..'(2, 3): v): D_objects = '++'(D_bs_cs_ds_objects, D_ds_objects);
    var set of D_potential: D = array_union('++'([D_occ_3('..'(1, '-'(A_a_bs_cs_ds_end, 1)))], [('[]'(C_c2_objects, _DECL_1)).ds | _DECL_1 in index_set(C_c2_objects)]));
    enum C_potential = C_occ_2('..'(1, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]))) ++ C_occ_4({1});
    array [int] of record(var set of D_potential: ds): C_objects = '++'(C_bs_cs_objects, C_c2_objects);
    set of C_potential: C = array_union('++'([C_occ_4({1})], [C_occ_2('..'(1, '-'(A_a_bs_cs_end, 1)))]));
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 1)) | this in C]);
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_a_inputs])));
    array [int] of record(set of C_potential: cs): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_bs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(array [int] of record(array [int] of record(int: v): ds): cs): bs): A_a_inputs = [(bs: [(cs: [(ds: [(v: 2)])])])];
    constraint forall(['in'(length((i).bs), '..'(1, 1)) | i in A_a_inputs]);
    array [int] of record(set of C_potential: cs): B_bs_objects = [(cs: cs) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).bs), input = '[]'(('[]'(A_a_inputs, p)).bs, r), cs = C_occ_2('..'('+'(1, '+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))])), length((input).cs))))];
    constraint forall(['in'(length((j1).cs), '..'(1, 1)) | i in A_a_inputs, j1 in (i).bs]);
    array [int] of record(var set of D_potential: ds): C_bs_cs_objects = let {
      array [int] of record(array [int] of record(int: v): ds): bs_cs_flat_inputs = [j2 | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs];
    } in [(ds: ds) | ci in index_set(bs_cs_flat_inputs), input = '[]'(bs_cs_flat_inputs, ci), ds = D_occ_3('..'('+'(1, sum([length(('[]'(bs_cs_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))])), '+'(sum([length(('[]'(bs_cs_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))]), length((input).ds))))];
    array [int] of record('..'(2, 3): v): D_bs_cs_ds_objects = [k | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs, k in (j2).ds];
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, sum([length((i).bs) | i in A_a_inputs]));
    int: A_a_bs_cs_start = 1;
    int: A_a_bs_cs_end = '+'(A_a_bs_cs_start, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]));
    int: A_a_bs_cs_ds_start = 1;
    int: A_a_bs_cs_ds_end = '+'(A_a_bs_cs_ds_start, sum([length((j2).ds) | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs]));
    array [int] of record(set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    A_potential: a = A_occ_0(1);
    array [{1}] of record(var set of D_potential: ds): C_c2_storage;
    array [D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 1)))))] of record(var '..'(2, 3): v): D_ds_objects;
    int: C_c2_ds_start = A_a_bs_cs_ds_end;
    int: C_c2_ds_end = '+'(C_c2_ds_start, '*'(card(C_potential), max('..'(1, 1))));
    array [int] of set of D_potential: C_c2_ds_potential = [_DECL_2: D_occ_5('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(1, 1)))), '*'(_DECL_2, max('..'(1, 1))))) | _DECL_2 in index_set(C_c2_storage)];
    array [int] of record(var set of D_potential: ds): C_c2_objects = [(ds: ds) | p in index_set(C_c2_storage), input = '[]'(C_c2_storage, p), ds = (input).ds];
    C_potential: c2 = C_occ_4(1);
    constraint forall(['subset'(enum2int(('[]'(C_c2_objects, _DECL_3)).ds), enum2int('[]'(C_c2_ds_potential, _DECL_3))) | _DECL_3 in index_set(C_c2_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// (3) Var-reached deep class whose object field is itself var-existence
	// (`var set of new D`), dropped from the deep par input: minted as a free
	// var subset in the deep contribution.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( var set(1..2) of new D: ds; );
    class B ( new C: c; );
    class A ( new B: b; );
    new A: a = (b: (c: ( )));
    var new C: c2;
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_3('..'(1, '*'(card(C_potential), max('..'(1, 2))))) ++ D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = '++'(D_b_c_ds_objects, D_ds_objects);
    var set of D_potential: D = array_union('++'([('[]'(C_b_c_objects, _DECL_1)).ds | _DECL_1 in index_set(C_b_c_objects)], [('[]'(C_c2_objects, _DECL_2)).ds | _DECL_2 in index_set(C_c2_objects)]));
    enum C_potential = C_occ_2('..'(1, '*'(card(B_potential), 1))) ++ C_occ_4({1});
    array [int] of record(var set of D_potential: ds): C_objects = '++'(C_b_c_objects, C_c2_objects);
    set of C_potential: C = array_union('++'([C_occ_4({1})], [{C_occ_2(_DECL_3)} | _DECL_3 in index_set(B_b_objects)]));
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(C_potential: c): B_objects = B_b_objects;
    set of B_potential: B = array_union([{B_occ_1(_DECL_4)} | _DECL_4 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(B_potential: b): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(record(record(): c): b): A_a_inputs = [(b: (c: ()))];
    array [int] of record(C_potential: c): B_b_objects = [(c: c) | p in index_set(A_a_inputs), input = ('[]'(A_a_inputs, p)).b, c = C_occ_2(p)];
    array [int] of record(var set of D_potential: ds): C_b_c_objects = let {
      array [int] of record(): b_c_flat_inputs = [((i).b).c | i in A_a_inputs];
    } in [(ds: ds) | ci in index_set(b_c_flat_inputs), input = '[]'(b_c_flat_inputs, ci), ds = let {
      var set of D_potential: ds_init;
    } in ds_init];
    array [D_occ_3('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_b_c_ds_objects;
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    int: A_a_b_c_start = 1;
    int: A_a_b_c_end = '+'(A_a_b_c_start, '*'(card(B_potential), 1));
    int: A_a_b_c_ds_start = 1;
    int: A_a_b_c_ds_end = '+'(A_a_b_c_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: A_a_b_c_ds_potential = [_DECL_5: D_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(1, 2)))), '*'(_DECL_5, max('..'(1, 2))))) | _DECL_5 in index_set(C_b_c_objects)];
    array [int] of record(B_potential: b): A_a_objects = [(b: b) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), b = B_occ_1(p)];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(C_b_c_objects, _DECL_6)).ds), enum2int('[]'(A_a_b_c_ds_potential, _DECL_6))) | _DECL_6 in index_set(C_b_c_objects)]);
    array [{1}] of record(var set of D_potential: ds): C_c2_storage;
    array [D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_ds_objects;
    int: C_c2_ds_start = A_a_b_c_ds_end;
    int: C_c2_ds_end = '+'(C_c2_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: C_c2_ds_potential = [_DECL_7: D_occ_5('..'('+'(1, '*'('-'(_DECL_7, 1), max('..'(1, 2)))), '*'(_DECL_7, max('..'(1, 2))))) | _DECL_7 in index_set(C_c2_storage)];
    array [int] of record(var set of D_potential: ds): C_c2_objects = [(ds: ds) | p in index_set(C_c2_storage), input = '[]'(C_c2_storage, p), ds = (input).ds];
    C_potential: c2 = C_occ_4(1);
    constraint forall(['subset'(enum2int(('[]'(C_c2_objects, _DECL_8)).ds), enum2int('[]'(C_c2_ds_potential, _DECL_8))) | _DECL_8 in index_set(C_c2_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// (4) Var-SET-reached (`var set of new C`) deep class: the deep par
	// contribution `++`s with the free var-set contribution.
	check_thir_model_with_stdlib(
		r#"
    int: n = 2;
    class D ( 2..3: v; );
    class C ( set(1..2) of new D: ds; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ds: [(v: 2)])])]);
    var set(0..n) of new C: cs2;
    solve satisfy;
    "#,
		expect![[r#"
    int: n = 2;
    enum D_potential = D_occ_3('..'(1, sum([length((j2).ds) | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs]))) ++ D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = '++'(D_bs_cs_ds_objects, D_ds_objects);
    var set of D_potential: D = array_union('++'([D_occ_3('..'(1, '-'(A_a_bs_cs_ds_end, 1)))], [if 'in'(C_occ_4(_DECL_1), C) then ('[]'(C_cs2_objects, _DECL_1)).ds else {} endif | _DECL_1 in index_set(C_cs2_objects)]));
    enum C_potential = C_occ_2('..'(1, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]))) ++ C_occ_4('..'(1, n));
    array [int] of record(var set of D_potential: ds): C_objects = '++'(C_bs_cs_objects, C_cs2_objects);
    var set of C_potential: C = array_union('++'([cs2], [C_occ_2('..'(1, '-'(A_a_bs_cs_end, 1)))]));
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_a_inputs])));
    array [int] of record(set of C_potential: cs): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_bs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(array [int] of record(array [int] of record(int: v): ds): cs): bs): A_a_inputs = [(bs: [(cs: [(ds: [(v: 2)])])])];
    constraint forall(['in'(length((i).bs), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record(set of C_potential: cs): B_bs_objects = [(cs: cs) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).bs), input = '[]'(('[]'(A_a_inputs, p)).bs, r), cs = C_occ_2('..'('+'(1, '+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))])), length((input).cs))))];
    constraint forall(['in'(length((j1).cs), '..'(1, 2)) | i in A_a_inputs, j1 in (i).bs]);
    array [int] of record(var set of D_potential: ds): C_bs_cs_objects = let {
      array [int] of record(array [int] of record(int: v): ds): bs_cs_flat_inputs = [j2 | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs];
    } in [(ds: ds) | ci in index_set(bs_cs_flat_inputs), input = '[]'(bs_cs_flat_inputs, ci), ds = D_occ_3('..'('+'(1, sum([length(('[]'(bs_cs_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))])), '+'(sum([length(('[]'(bs_cs_flat_inputs, cj)).ds) | cj in '..'(1, '-'(ci, 1))]), length((input).ds))))];
    array [int] of record('..'(2, 3): v): D_bs_cs_ds_objects = [k | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs, k in (j2).ds];
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, sum([length((i).bs) | i in A_a_inputs]));
    int: A_a_bs_cs_start = 1;
    int: A_a_bs_cs_end = '+'(A_a_bs_cs_start, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]));
    int: A_a_bs_cs_ds_start = 1;
    int: A_a_bs_cs_ds_end = '+'(A_a_bs_cs_ds_start, sum([length((j2).ds) | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs]));
    array [int] of record(set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    A_potential: a = A_occ_0(1);
    array ['..'(1, n)] of record(var set of D_potential: ds): C_cs2_storage;
    array [D_occ_5('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_ds_objects;
    int: C_cs2_ds_start = A_a_bs_cs_ds_end;
    int: C_cs2_ds_end = '+'(C_cs2_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: C_cs2_ds_potential = [_DECL_2: D_occ_5('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(1, 2)))), '*'(_DECL_2, max('..'(1, 2))))) | _DECL_2 in index_set(C_cs2_storage)];
    array [int] of record(var set of D_potential: ds): C_cs2_objects = C_cs2_storage;
    var set( '..'(0, n) ) of  C_occ_4('..'(1, n)): cs2;
    constraint forall(['subset'(enum2int(('[]'(C_cs2_objects, _DECL_3)).ds), enum2int('[]'(C_cs2_ds_potential, _DECL_3))) | _DECL_3 in index_set(C_cs2_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    constraint forall(['\/'('in'(x, C), '='(('[]'(C_objects, enum2int(x))).ds, lb(('[]'(C_objects, enum2int(x))).ds))) | x in C_potential]);
    solve satisfy;
"#]],
	);
}

/// A par owner transitively owns a VAR-EXISTENCE object field (`var set
/// of new D` / `var opt new D`). The set/opt existence is a solver decision,
/// dropped from the par input record, so the reconstruction builders mint it
/// as a fresh free `var set of <D>_potential` / `var opt <D>_potential`
/// decision confined to its per-parent block (`var_existence_field_mint`),
/// instead of a par identity range read off `length(input.<field>)` (which
/// panicked reading the dropped field — the shape the fence used to reject at
/// depth ≥ 2 and that panicked unfenced at depth 1). Each shape below fully
/// lowers (no THIR panic); solution-equivalence is pinned by the
/// `par_var_set_new_field` / `par_var_set_new_deep` / `par_var_opt_new_field`
/// pairs. Covers the par-owner root arm, the depth-1 nested set and singular
/// edges, the deep (depth-2) arm, and both var-set and var-opt field kinds.
#[test]
fn object_thir_lowering_par_root_deep_nested_var_existence_field_snapshot() {
	// Root owner: a par singular `new C` whose C owns `var set of new D`.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var set(1..2) of new D: ds; );
    new C: c = (ck: 1);
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_1('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_ds_objects);
    var set of D_potential: D = array_union([('[]'(C_c_objects, _DECL_1)).ds | _DECL_1 in index_set(C_c_objects)]);
    enum C_potential = C_occ_0({1});
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_objects = C_c_objects;
    set of C_potential: C = C_occ_0({1});
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    array [int] of record(int: ck): C_c_inputs = [(ck: 1)];
    array [D_occ_1('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_ds_objects;
    int: C_c_ds_start = 1;
    int: C_c_ds_end = '+'(C_c_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: C_c_ds_potential = [_DECL_2: D_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(1, 2)))), '*'(_DECL_2, max('..'(1, 2))))) | _DECL_2 in index_set(C_c_inputs)];
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_c_objects = [(ck: ck, ds: ds) | p in index_set(C_c_inputs), input = '[]'(C_c_inputs, p), ck = (input).ck, ds = let {
      var set of D_potential: ds_init;
    } in ds_init];
    C_potential: c = C_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(C_c_objects, _DECL_3)).ds), enum2int('[]'(C_c_ds_potential, _DECL_3))) | _DECL_3 in index_set(C_c_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// Depth-1 SET edge: par root A owns `set of new C`; C owns the var set.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var set(1..2) of new D: ds; );
    class A ( set(1..2) of new C: cs; );
    new A: a = (cs: [(ck: 1), (ck: 2)]);
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_2('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_cs_ds_objects);
    var set of D_potential: D = array_union([('[]'(C_cs_objects, _DECL_1)).ds | _DECL_1 in index_set(C_cs_objects)]);
    enum C_potential = C_occ_1('..'(1, sum([length((i).cs) | i in A_a_inputs])));
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_objects = C_cs_objects;
    set of C_potential: C = array_union([C_occ_1('..'(1, '-'(A_a_cs_end, 1)))]);
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of C_potential: cs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(int: ck): cs): A_a_inputs = [(cs: [(ck: 1), (ck: 2)])];
    constraint forall(['in'(length((i).cs), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_cs_objects = [(ck: ck, ds: ds) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).cs), input = '[]'(('[]'(A_a_inputs, p)).cs, r), ck = (input).ck, ds = let {
      var set of D_potential: ds_init;
    } in ds_init];
    array [D_occ_2('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_cs_ds_objects;
    int: A_a_cs_start = 1;
    int: A_a_cs_end = '+'(A_a_cs_start, sum([length((i).cs) | i in A_a_inputs]));
    int: A_a_cs_ds_start = 1;
    int: A_a_cs_ds_end = '+'(A_a_cs_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: A_a_cs_ds_potential = [_DECL_2: D_occ_2('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(1, 2)))), '*'(_DECL_2, max('..'(1, 2))))) | _DECL_2 in index_set(C_cs_objects)];
    array [int] of record(set of C_potential: cs): A_a_objects = [(cs: cs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), cs = C_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).cs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).cs) | q in '..'(1, '-'(p, 1))]), length((input).cs))))];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(C_cs_objects, _DECL_3)).ds), enum2int('[]'(A_a_cs_ds_potential, _DECL_3))) | _DECL_3 in index_set(C_cs_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// Depth-1 SINGULAR edge: par root A owns `new C`; C owns the var set.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var set(1..2) of new D: ds; );
    class A ( new C: c; );
    set of new A: as = [(c: (ck: 1)), (c: (ck: 2))];
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_2('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_c_ds_objects);
    var set of D_potential: D = array_union([('[]'(C_c_objects, _DECL_1)).ds | _DECL_1 in index_set(C_c_objects)]);
    enum C_potential = C_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_objects = C_c_objects;
    set of C_potential: C = array_union([{C_occ_1(_DECL_2)} | _DECL_2 in index_set(A_as_objects)]);
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(C_potential: c): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(record(int: ck): c): A_as_inputs = [(c: (ck: 1)), (c: (ck: 2))];
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_c_objects = [(ck: ck, ds: ds) | p in index_set(A_as_inputs), input = ('[]'(A_as_inputs, p)).c, ck = (input).ck, ds = let {
      var set of D_potential: ds_init;
    } in ds_init];
    array [D_occ_2('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_c_ds_objects;
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    int: A_as_c_start = 1;
    int: A_as_c_end = '+'(A_as_c_start, '*'(card(A_potential), 1));
    int: A_as_c_ds_start = 1;
    int: A_as_c_ds_end = '+'(A_as_c_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: A_as_c_ds_potential = [_DECL_3: D_occ_2('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(1, 2)))), '*'(_DECL_3, max('..'(1, 2))))) | _DECL_3 in index_set(C_c_objects)];
    array [int] of record(C_potential: c): A_as_objects = [(c: c) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), c = C_occ_1(p)];
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    constraint forall(['subset'(enum2int(('[]'(C_c_objects, _DECL_4)).ds), enum2int('[]'(A_as_c_ds_potential, _DECL_4))) | _DECL_4 in index_set(C_c_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// Deep (depth-2): A.bs -> B.cs -> C, C owns the var set two hops down.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var set(1..2) of new D: ds; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ck: 1), (ck: 2)])]);
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_3('..'(1, '*'(card(C_potential), max('..'(1, 2)))));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_bs_cs_ds_objects);
    var set of D_potential: D = array_union([('[]'(C_bs_cs_objects, _DECL_1)).ds | _DECL_1 in index_set(C_bs_cs_objects)]);
    enum C_potential = C_occ_2('..'(1, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs])));
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_objects = C_bs_cs_objects;
    set of C_potential: C = array_union([C_occ_2('..'(1, '-'(A_a_bs_cs_end, 1)))]);
    constraint forall(['in'(card(('[]'(C_objects, enum2int(this))).ds), '..'(1, 2)) | this in C]);
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_a_inputs])));
    array [int] of record(set of C_potential: cs): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_bs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(array [int] of record(int: ck): cs): bs): A_a_inputs = [(bs: [(cs: [(ck: 1), (ck: 2)])])];
    constraint forall(['in'(length((i).bs), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record(set of C_potential: cs): B_bs_objects = [(cs: cs) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).bs), input = '[]'(('[]'(A_a_inputs, p)).bs, r), cs = C_occ_2('..'('+'(1, '+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))])), length((input).cs))))];
    constraint forall(['in'(length((j1).cs), '..'(1, 2)) | i in A_a_inputs, j1 in (i).bs]);
    array [int] of record('..'(1, 3): ck, var set of D_potential: ds): C_bs_cs_objects = let {
      array [int] of record(int: ck): bs_cs_flat_inputs = [j2 | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs];
    } in [(ck: ck, ds: ds) | ci in index_set(bs_cs_flat_inputs), input = '[]'(bs_cs_flat_inputs, ci), ck = (input).ck, ds = let {
      var set of D_potential: ds_init;
    } in ds_init];
    array [D_occ_3('..'(1, '*'(card(C_potential), max('..'(1, 2)))))] of record(var '..'(2, 3): v): D_bs_cs_ds_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, sum([length((i).bs) | i in A_a_inputs]));
    int: A_a_bs_cs_start = 1;
    int: A_a_bs_cs_end = '+'(A_a_bs_cs_start, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]));
    int: A_a_bs_cs_ds_start = 1;
    int: A_a_bs_cs_ds_end = '+'(A_a_bs_cs_ds_start, '*'(card(C_potential), max('..'(1, 2))));
    array [int] of set of D_potential: A_a_bs_cs_ds_potential = [_DECL_2: D_occ_3('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(1, 2)))), '*'(_DECL_2, max('..'(1, 2))))) | _DECL_2 in index_set(C_bs_cs_objects)];
    array [int] of record(set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(C_bs_cs_objects, _DECL_3)).ds), enum2int('[]'(A_a_bs_cs_ds_potential, _DECL_3))) | _DECL_3 in index_set(C_bs_cs_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// VAR OPT field on a par owner one hop below the root.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var opt new D: d; );
    class A ( set(1..2) of new C: cs; );
    new A: a = (cs: [(ck: 1), (ck: 2)]);
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_2('..'(1, '*'(card(C_potential), 1)));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_cs_d_objects);
    var set of D_potential: D = array_union([if occurs(('[]'(C_cs_objects, _DECL_1)).d) then {D_occ_2(_DECL_1)} else {} endif | _DECL_1 in index_set(C_cs_objects)]);
    enum C_potential = C_occ_1('..'(1, sum([length((i).cs) | i in A_a_inputs])));
    array [int] of record('..'(1, 3): ck, var opt D_potential: d): C_objects = C_cs_objects;
    set of C_potential: C = array_union([C_occ_1('..'(1, '-'(A_a_cs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of C_potential: cs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(int: ck): cs): A_a_inputs = [(cs: [(ck: 1), (ck: 2)])];
    constraint forall(['in'(length((i).cs), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record('..'(1, 3): ck, var opt D_potential: d): C_cs_objects = [(ck: ck, d: d) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).cs), input = '[]'(('[]'(A_a_inputs, p)).cs, r), ck = (input).ck, d = let {
      var opt D_potential: d_init;
    } in d_init];
    array [D_occ_2('..'(1, '*'(card(C_potential), 1)))] of record(var '..'(2, 3): v): D_cs_d_objects;
    int: A_a_cs_start = 1;
    int: A_a_cs_end = '+'(A_a_cs_start, sum([length((i).cs) | i in A_a_inputs]));
    int: A_a_cs_d_start = 1;
    int: A_a_cs_d_end = '+'(A_a_cs_d_start, '*'(card(C_potential), 1));
    array [int] of record(set of C_potential: cs): A_a_objects = [(cs: cs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), cs = C_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).cs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).cs) | q in '..'(1, '-'(p, 1))]), length((input).cs))))];
    A_potential: a = A_occ_0(1);
    constraint forall(['->'(occurs(('[]'(C_cs_objects, _DECL_2)).d), '='(('[]'(C_cs_objects, _DECL_2)).d, D_occ_2(_DECL_2))) | _DECL_2 in index_set(C_cs_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
	// Deep var opt: depth-2 owner with `var opt new D`.
	check_thir_model_with_stdlib(
		r#"
    class D ( 2..3: v; );
    class C ( 1..3: ck; var opt new D: d; );
    class B ( set(1..2) of new C: cs; );
    class A ( set(1..2) of new B: bs; );
    new A: a = (bs: [(cs: [(ck: 1), (ck: 2)])]);
    solve satisfy;
    "#,
		expect![[r#"
    enum D_potential = D_occ_3('..'(1, '*'(card(C_potential), 1)));
    array [int] of record(var '..'(2, 3): v): D_objects = array1d(D_bs_cs_d_objects);
    var set of D_potential: D = array_union([if occurs(('[]'(C_bs_cs_objects, _DECL_1)).d) then {D_occ_3(_DECL_1)} else {} endif | _DECL_1 in index_set(C_bs_cs_objects)]);
    enum C_potential = C_occ_2('..'(1, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs])));
    array [int] of record('..'(1, 3): ck, var opt D_potential: d): C_objects = C_bs_cs_objects;
    set of C_potential: C = array_union([C_occ_2('..'(1, '-'(A_a_bs_cs_end, 1)))]);
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_a_inputs])));
    array [int] of record(set of C_potential: cs): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_a_bs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(array [int] of record(array [int] of record(int: ck): cs): bs): A_a_inputs = [(bs: [(cs: [(ck: 1), (ck: 2)])])];
    constraint forall(['in'(length((i).bs), '..'(1, 2)) | i in A_a_inputs]);
    array [int] of record(set of C_potential: cs): B_bs_objects = [(cs: cs) | p in index_set(A_a_inputs), r in index_set(('[]'(A_a_inputs, p)).bs), input = '[]'(('[]'(A_a_inputs, p)).bs, r), cs = C_occ_2('..'('+'(1, '+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).cs) | q in '..'(1, '-'(p, 1)), j in ('[]'(A_a_inputs, q)).bs]), sum([length(('[]'(('[]'(A_a_inputs, p)).bs, s)).cs) | s in '..'(1, '-'(r, 1))])), length((input).cs))))];
    constraint forall(['in'(length((j1).cs), '..'(1, 2)) | i in A_a_inputs, j1 in (i).bs]);
    array [int] of record('..'(1, 3): ck, var opt D_potential: d): C_bs_cs_objects = let {
      array [int] of record(int: ck): bs_cs_flat_inputs = [j2 | i in A_a_inputs, j1 in (i).bs, j2 in (j1).cs];
    } in [(ck: ck, d: d) | ci in index_set(bs_cs_flat_inputs), input = '[]'(bs_cs_flat_inputs, ci), ck = (input).ck, d = let {
      var opt D_potential: d_init;
    } in d_init];
    array [D_occ_3('..'(1, '*'(card(C_potential), 1)))] of record(var '..'(2, 3): v): D_bs_cs_d_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, sum([length((i).bs) | i in A_a_inputs]));
    int: A_a_bs_cs_start = 1;
    int: A_a_bs_cs_end = '+'(A_a_bs_cs_start, sum([length((j1).cs) | i in A_a_inputs, j1 in (i).bs]));
    int: A_a_bs_cs_d_start = 1;
    int: A_a_bs_cs_d_end = '+'(A_a_bs_cs_d_start, '*'(card(C_potential), 1));
    array [int] of record(set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_a_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    A_potential: a = A_occ_0(1);
    constraint forall(['->'(occurs(('[]'(C_bs_cs_objects, _DECL_2)).d), '='(('[]'(C_bs_cs_objects, _DECL_2)).d, D_occ_3(_DECL_2))) | _DECL_2 in index_set(C_bs_cs_objects)]);
    constraint forall(['\/'('in'(x, D), '='(('[]'(D_objects, enum2int(x))).v, mzn_safe_default(('[]'(D_objects, enum2int(x))).v))) | x in D_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_reference_equality_snapshot() {
	check_thir_model_with_stdlib(
		r#"
    class A ();
    A: a;
    A: b;
    constraint a = b;
    constraint a != b;
    solve satisfy;
    "#,
		expect![[r#"
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    A: a;
    A: b;
    constraint '='(a, b);
    constraint '!='(a, b);
    solve satisfy;
"#]],
	);
	check_thir_model_with_stdlib(
		r#"
    class A ();
    opt A: a;
    opt A: b;
    constraint a = b;
    constraint a != b;
    solve satisfy;
    "#,
		expect![[r#"
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    opt A: a;
    opt A: b;
    constraint '='(a, b);
    constraint '!='(a, b);
    solve satisfy;
"#]],
	);
	check_thir_model_with_stdlib(
		r#"
    class A ();
    class B extends A ();
    A: a;
    B: b;
    constraint a = b;
    constraint a != b;
    solve satisfy;
    "#,
		expect![[r#"
    enum B_potential;
    array [int] of record(): B_objects;
    set of B_potential: B;
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    A: a;
    B: b;
    constraint '='(enum2int(a), enum2int(b));
    constraint '!='(enum2int(a), enum2int(b));
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_reference_ordering_snapshot() {
	check_thir_model_with_stdlib(
		r#"
    class A ();
    A: a;
    A: b;
    constraint a < b;
    constraint a <= b;
    constraint a > b;
    constraint a >= b;
    solve satisfy;
    "#,
		expect![[r#"
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    A: a;
    A: b;
    constraint '<'(a, b);
    constraint '<='(a, b);
    constraint '>'(a, b);
    constraint '>='(a, b);
    solve satisfy;
"#]],
	);
	check_thir_model_with_stdlib(
		r#"
    class A ();
    class B extends A ();
    A: a;
    B: b;
    constraint a < b;
    constraint a <= b;
    constraint a > b;
    constraint a >= b;
    solve satisfy;
    "#,
		expect![[r#"
    enum B_potential;
    array [int] of record(): B_objects;
    set of B_potential: B;
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    A: a;
    B: b;
    constraint '<'(enum2int(a), enum2int(b));
    constraint '<='(enum2int(a), enum2int(b));
    constraint '>'(enum2int(a), enum2int(b));
    constraint '>='(enum2int(a), enum2int(b));
    solve satisfy;
"#]],
	);
	check_thir_model_with_stdlib(
		r#"
    class A ();
    opt A: a;
    opt A: b;
    constraint a < b;
    constraint a <= b;
    constraint a > b;
    constraint a >= b;
    solve satisfy;
    "#,
		expect![[r#"
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    opt A: a;
    opt A: b;
    constraint '<'(a, b);
    constraint '<='(a, b);
    constraint '>'(a, b);
    constraint '>='(a, b);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_generic_enumerable_snapshot() {
	check_thir_model_with_stdlib(
		r#"
    class A ();
    A: a;
    A: b;
    opt A: oa;
    set of A: span = a..b;
    constraint a in enum_of(a);
    constraint b in span;
    constraint absent(oa) \/ deopt(oa) in enum_of(oa);
    solve satisfy;
    "#,
		expect![[r#"
    enum A_potential;
    array [int] of record(): A_objects;
    set of A_potential: A;
    A: a;
    A: b;
    opt A: oa;
    set of A_potential: span = '..'(a, b);
    constraint 'in'(a, enum_of(a));
    constraint 'in'(b, span);
    constraint '\/'(absent(oa), 'in'(deopt(oa), enum_of(oa)));
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_simple_new_snapshot() {
	check_thir_model_with_stdlib(
		SIMPLE_NEW,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 2): x): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var '..'(0, 2): x): A_a_storage;
    array [int] of record(var '..'(0, 2): x): A_a_objects = A_a_storage;
    A_potential: a = A_occ_0(1);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_optional_new_snapshot() {
	check_thir_model_with_stdlib(
		OPTIONAL_NEW,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 2): x): A_objects = A_maybe_a_objects;
    var set of A_potential: A;
    array [{1}] of record(var '..'(0, 2): x): A_maybe_a_storage;
    array [int] of record(var '..'(0, 2): x): A_maybe_a_objects = A_maybe_a_storage;
    var opt A_potential: maybe_a = if 'in'(A_occ_0(1), A) then A_occ_0(1) else <> endif;
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_simple_class_constraint_snapshot() {
	check_thir_model_with_stdlib(
		SIMPLE_CLASS_CONSTRAINT,
		expect!([r#"
    enum A_potential = A_occ_0({1});
    array [int] of record(int: x): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall(['>='(('[]'(A_objects, enum2int(this))).x, 0) | this in A]);
    array [int] of record(int: x): A_a_inputs = [(x: 1)];
    array [int] of record(int: x): A_a_objects = A_a_inputs;
    A_potential: a = A_occ_0(1);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_class_constraint_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_CLASS_CONSTRAINT,
		expect!([r#"
    enum B_potential = B_occ_0({1});
    array [int] of record(int: x, int: y): B_objects = B_b_objects;
    set of B_potential: B = B_occ_0({1});
    constraint forall(['<='(('[]'(B_objects, enum2int(this))).x, ('[]'(B_objects, enum2int(this))).y) | this in B]);
    enum A_potential = A_occ_0({1});
    array [int] of record(int: x): A_objects = A_B_b_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_1)} | _DECL_1 in index_set(B_b_objects)]);
    constraint forall(['>='(('[]'(A_objects, enum2int(this))).x, 0) | this in A]);
    array [int] of record(int: x, int: y): B_b_inputs = [(x: 1, y: 2)];
    int: B_b__start = 1;
    int: B_b__end = 2;
    array [int] of record(int: x, int: y): B_b_objects = B_b_inputs;
    array [int] of record(int: x): A_B_b_objects = [(x: ('[]'(B_b_objects, p)).x) | p in index_set(B_b_objects)];
    B_potential: b = B_occ_0(1);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_simple_class_reference_snapshot() {
	check_thir_model_with_stdlib(
		SIMPLE_CLASS_REFERENCE,
		expect!([r#"
    enum A_potential;
    array [int] of record(int: x): A_objects;
    set of A_potential: A;
    constraint forall(['>='(('[]'(A_objects, enum2int(this))).x, 0) | this in A]);
    A: a;
    constraint '>='(('[]'(A_objects, enum2int(a))).x, 0);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inheritance_snapshot() {
	check_thir_model_with_stdlib(
		INHERITANCE,
		expect!([r#"
    enum B_potential = B_occ_0({1});
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = B_b_objects;
    set of B_potential: B = B_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 4): x): A_objects = A_B_b_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_1)} | _DECL_1 in index_set(B_b_objects)]);
    array [{1}] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_b_storage;
    int: B_b__start = 1;
    int: B_b__end = 2;
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_b_objects = B_b_storage;
    array [int] of record(var '..'(0, 4): x): A_B_b_objects = [(x: ('[]'(B_b_objects, p)).x) | p in index_set(B_b_objects)];
    B_potential: b = B_occ_0(1);
    constraint '<='(('[]'(B_objects, enum2int(b))).x, ('[]'(B_objects, enum2int(b))).y);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_top_level_set_new_snapshot() {
	check_thir_model(
		TOP_LEVEL_SET_NEW,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: c): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: c): A_as_inputs = [(c: 1), (c: 2)];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    array [int] of record(int: c): A_as_objects = A_as_inputs;
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_top_level_set_new_mixed_scalar_snapshot() {
	check_thir_model_with_stdlib(
		TOP_LEVEL_SET_NEW_MIXED_SCALAR,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: x, var '..'(0, 2): y): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: x): A_as_inputs = [(x: 1), (x: 2)];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    array [int] of record(int: x, var '..'(0, 2): y): A_as_objects = [(x: x, y: y) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), x = (input).x, y = let {
      var '..'(0, 2): y_init;
    } in y_init];
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_empty_record_set_new_snapshot() {
	check_thir_model(
		EMPTY_RECORD_SET_NEW,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(): A_as_inputs = [(), ()];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    array [int] of record(): A_as_objects = A_as_inputs;
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_self_class_constraint_reference_snapshot() {
	check_thir_model_with_stdlib(
		SELF_CLASS_CONSTRAINT_REFERENCE,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: x): A_objects = A_as_objects;
    set of A_potential: A = as;
    constraint forall(['in'(this, A) | this in A]);
    array [int] of record(int: x): A_as_inputs = [(x: 1), (x: 2)];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    array [int] of record(int: x): A_as_objects = A_as_inputs;
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_bounded_set_new_snapshot() {
	check_thir_model_with_stdlib(
		BOUNDED_SET_NEW,
		expect!([r#"
    int: n;
    enum A_potential = A_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 2): x): A_objects = A_as_objects;
    var set of A_potential: A = as;
    array ['..'(1, n)] of record(var '..'(0, 2): x): A_as_storage;
    array [int] of record(var '..'(0, 2): x): A_as_objects = A_as_storage;
    var set( '..'(0, n) ) of  A_occ_0('..'(1, n)): as;
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

/// Focused regression for `set(<card>) of new` cardinality lowering.
/// The declared cardinality bound must surface as an explicit
/// `card(<root>) in <card>` constraint: the `<C>_potential` universe is
/// sized to the *upper* bound (so `card <= ub` holds structurally), but
/// nothing else constrains the lower bound. Literal bounds here so the
/// constraint reads `card(as) in 2..3` rather than a parameter range.
/// See the direct-emission shim in `thir/lower.rs::collect_declaration`.
#[test]
fn object_thir_lowering_set_cardinality_constraint_snapshot() {
	check_thir_model_with_stdlib(
		r#"
class A (
  int: x;
);

var set(2..3) of new A: as;
solve satisfy;
"#,
		expect!([r#"
    enum A_potential = A_occ_0('..'(1, 3));
    array [int] of record(var int: x): A_objects = A_as_objects;
    var set of A_potential: A = as;
    array ['..'(1, 3)] of record(var int: x): A_as_storage;
    array [int] of record(var int: x): A_as_objects = A_as_storage;
    var set( '..'(2, 3) ) of  A_occ_0('..'(1, 3)): as;
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_bounded_set_new_symmetry_defaults_snapshot() {
	check_thir_model_with_stdlib(
		BOUNDED_SET_NEW_SYMMETRY_DEFAULTS,
		expect!([r#"
    int: n;
    enum C_potential = C_occ_0('..'(1, n));
    array [int] of record(var bool: b, var '..'(5, 10): i, var opt int: o, var set of '..'(0, 3): s): C_objects = C_cs_objects;
    var set of C_potential: C = cs;
    array ['..'(1, n)] of record(var bool: b, var '..'(5, 10): i, var opt int: o, var set of '..'(0, 3): s): C_cs_storage;
    array [int] of record(var bool: b, var '..'(5, 10): i, var opt int: o, var set of '..'(0, 3): s): C_cs_objects = [(b: b, i: i, o: o, s: s) | p in index_set(C_cs_storage), input = '[]'(C_cs_storage, p), i = (input).i, b = (input).b, o = (input).o, s = (input).s];
    var set( '..'(0, n) ) of  C_occ_0('..'(1, n)): cs;
    constraint forall(['\/'('in'(x, C), '='(('[]'(C_objects, enum2int(x))).b, lb(('[]'(C_objects, enum2int(x))).b))) | x in C_potential]);
    constraint forall(['\/'('in'(x, C), '='(('[]'(C_objects, enum2int(x))).i, mzn_safe_default(('[]'(C_objects, enum2int(x))).i))) | x in C_potential]);
    constraint forall(['\/'('in'(x, C), '='(('[]'(C_objects, enum2int(x))).o, <>)) | x in C_potential]);
    constraint forall(['\/'('in'(x, C), '='(('[]'(C_objects, enum2int(x))).s, lb(('[]'(C_objects, enum2int(x))).s))) | x in C_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_bounded_two_sets_new_snapshot() {
	check_thir_model_with_stdlib(
		BOUNDED_TWO_SETS_NEW,
		expect!([r#"
    enum A_potential = A_occ_0('..'(1, n)) ++ A_occ_1('..'(1, m));
    array [int] of record(var '..'(0, 2): c): A_objects = '++'(A_as1_objects, A_as2_objects);
    var set of A_potential: A = array_union([as1, as2]);
    int: n;
    array ['..'(1, n)] of record(var '..'(0, 2): c): A_as1_storage;
    array [int] of record(var '..'(0, 2): c): A_as1_objects = A_as1_storage;
    var set( '..'(1, n) ) of  A_occ_0('..'(1, n)): as1;
    int: m;
    array ['..'(1, m)] of record(var '..'(0, 2): c): A_as2_storage;
    array [int] of record(var '..'(0, 2): c): A_as2_objects = A_as2_storage;
    var set( '..'(1, m) ) of  A_occ_1('..'(1, m)): as2;
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).c, mzn_safe_default(('[]'(A_objects, enum2int(x))).c))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_mixed_par_var_set_new_snapshot() {
	check_thir_model_with_stdlib(
		MIXED_PAR_VAR_SET_NEW,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2}) ++ A_occ_1('..'(1, 4));
    array [int] of record(var '..'(0, 4): x): A_objects = '++'(A_a1_objects, A_a2_objects);
    var set of A_potential: A = array_union([a1, a2]);
    array [int] of record(int: x): A_a1_inputs = [(x: 3), (x: 4)];
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 3;
    array [int] of record('..'(0, 4): x): A_a1_objects = A_a1_inputs;
    set of A_occ_0({1, 2}): a1 = A_occ_0({1, 2});
    array ['..'(1, 4)] of record(var '..'(0, 4): x): A_a2_storage;
    array [int] of record(var '..'(0, 4): x): A_a2_objects = A_a2_storage;
    var set( '..'(1, 4) ) of  A_occ_1('..'(1, 4)): a2;
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_bounded_set_new_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_BOUNDED_SET_NEW,
		expect!([r#"
    enum B_potential = B_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_objects = B_bs_objects;
    var set of B_potential: B = bs;
    enum A_potential = A_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 2): x): A_objects = A_B_bs_objects;
    var set of A_potential: A = array_union([if 'in'(B_occ_0(_DECL_1), B) then {A_occ_0(_DECL_1)} else {} endif | _DECL_1 in index_set(B_bs_objects)]);
    int: n;
    array ['..'(1, n)] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_bs_storage;
    int: B_bs__start = 1;
    int: B_bs__end = 2;
    array [int] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_bs_objects = B_bs_storage;
    array [int] of record(var '..'(0, 2): x): A_B_bs_objects = [(x: ('[]'(B_bs_objects, p)).x) | p in index_set(B_bs_objects)];
    var set( '..'(0, n) ) of  B_occ_0('..'(1, n)): bs;
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_bounded_set_class_constraint_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_BOUNDED_SET_SUPERCLASS_ALIAS,
		expect!([r#"
    enum B_potential = B_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = B_bs_objects;
    var set of B_potential: B = bs;
    constraint forall(['<='(('[]'(B_objects, enum2int(this))).x, ('[]'(B_objects, enum2int(this))).y) | this in B]);
    enum A_potential = A_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 4): x): A_objects = A_B_bs_objects;
    var set of A_potential: A = array_union([if 'in'(B_occ_0(_DECL_1), B) then {A_occ_0(_DECL_1)} else {} endif | _DECL_1 in index_set(B_bs_objects)]);
    constraint forall(['>='(('[]'(A_objects, enum2int(this))).x, 0) | this in A]);
    int: n;
    array ['..'(1, n)] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_storage;
    int: B_bs__start = 1;
    int: B_bs__end = 2;
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_objects = B_bs_storage;
    array [int] of record(var '..'(0, 4): x): A_B_bs_objects = [(x: ('[]'(B_bs_objects, p)).x) | p in index_set(B_bs_objects)];
    var set( '..'(0, n) ) of  B_occ_0('..'(1, n)): bs;
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_bounded_set_superclass_set_alias_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_BOUNDED_SET_SUPERCLASS_SET_ALIAS,
		expect!([r#"
    enum B_potential = B_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = B_bs_objects;
    var set of B_potential: B = bs;
    enum A_potential = A_occ_0('..'(1, n));
    array [int] of record(var '..'(0, 4): x): A_objects = A_B_bs_objects;
    var set of A_potential: A = array_union([if 'in'(B_occ_0(_DECL_1), B) then {A_occ_0(_DECL_1)} else {} endif | _DECL_1 in index_set(B_bs_objects)]);
    int: n;
    array ['..'(1, n)] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_storage;
    int: B_bs__start = 1;
    int: B_bs__end = 2;
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_objects = B_bs_storage;
    array [int] of record(var '..'(0, 4): x): A_B_bs_objects = [(x: ('[]'(B_bs_objects, p)).x) | p in index_set(B_bs_objects)];
    var set( '..'(0, n) ) of  B_occ_0('..'(1, n)): bs;
    var set of A_potential: as = A_occ_0(enum2int(bs));
    constraint forall(['>='(('[]'(A_objects, enum2int(a))).x, 0) | a in as]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_class_constraint_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_CLASS_CONSTRAINT,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([('[]'(A_a_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a_objects)]);
    constraint forall(['>='(('[]'(B_objects, enum2int(this))).y, 0) | this in B]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), bs = (input).bs];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).bs), enum2int('[]'(A_a_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_alias_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_ALIAS,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([('[]'(A_a_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), bs = (input).bs];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).bs), enum2int('[]'(A_a_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    var set of B_potential: bs = ('[]'(A_objects, enum2int(a))).bs;
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_FIELD_ACCESS,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([('[]'(A_a_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), bs = (input).bs];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).bs), enum2int('[]'(A_a_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in ('[]'(A_objects, enum2int(a))).bs]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).bs), enum2int('[]'(A_as_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_FIELD_ACCESS,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).bs), enum2int('[]'(A_as_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | a in as, b in ('[]'(A_objects, enum2int(a))).bs]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_cardinality_channeling_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_CARDINALITY_CHANNELING,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).bs), enum2int('[]'(A_as_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint '='(card(B), sum([card(('[]'(A_objects, enum2int(a))).bs) | a in as]));
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_fields_same_class_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_FIELDS_SAME_CLASS,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_cs_objects);
    var set of B_potential: B = array_union('++'([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)], [if 'in'(A_occ_0(_DECL_2), A) then ('[]'(A_as_objects, _DECL_2)).cs else {} endif | _DECL_2 in index_set(A_as_objects)]));
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).cs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    array [B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_cs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_as_storage)];
    int: A_as_cs_start = A_as_bs_end;
    int: A_as_cs_end = '+'(A_as_cs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_cs_potential = [_DECL_4: B_occ_2('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(0, n)))), '*'(_DECL_4, max('..'(0, n))))) | _DECL_4 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_5)).bs), enum2int('[]'(A_as_bs_potential, _DECL_5))) | _DECL_5 in index_set(A_as_objects)]);
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_6)).cs), enum2int('[]'(A_as_cs_potential, _DECL_6))) | _DECL_6 in index_set(A_as_objects)]);
    constraint '='(card(B), sum(['+'(card(('[]'(A_objects, enum2int(a))).bs), card(('[]'(A_objects, enum2int(a))).cs)) | a in as]));
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | a in as, b in ('[]'(A_objects, enum2int(a))).bs]);
    constraint forall(['>='(('[]'(B_objects, enum2int(c))).y, 0) | a in as, c in ('[]'(A_objects, enum2int(a))).cs]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).cs, lb(('[]'(A_objects, enum2int(x))).cs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_parent_membership_channeling_snapshot()
 {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_PARENT_MEMBERSHIP_CHANNELING,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)]);
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_3)).bs), enum2int('[]'(A_as_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_as_objects)]);
    constraint forall(['='(sum([bool2int('in'(b, ('[]'(A_objects, enum2int(a))).bs)) | a in as]), 1) | b in B]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_iterator_field_set_field_access_snapshot() {
	check_thir_model_with_stdlib(
		ITERATOR_FIELD_SET_FIELD_ACCESS,
		expect!([r#"
    int: maxSeats = 2;
    enum Seat_potential;
    array [int] of record(Wagon_potential: wagon): Seat_objects;
    set of Seat_potential: Seat;
    enum Wagon_potential;
    array [int] of record(set of Seat_potential: seats): Wagon_objects;
    set of Wagon_potential: Wagon;
    constraint forall([forall(['in'(s, ('[]'(Wagon_objects, enum2int(('[]'(Seat_objects, enum2int(s))).wagon))).seats) | s in ('[]'(Wagon_objects, enum2int(w))).seats]) | w in Wagon]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_path_parent_exclusivity_snapshot()
{
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_PATH_PARENT_EXCLUSIVITY,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = '++'(B_bs_objects, B_cs_objects);
    var set of B_potential: B = array_union('++'([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)], [if 'in'(A_occ_0(_DECL_2), A) then ('[]'(A_as_objects, _DECL_2)).cs else {} endif | _DECL_2 in index_set(A_as_objects)]));
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).cs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    array [B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_cs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_as_storage)];
    int: A_as_cs_start = A_as_bs_end;
    int: A_as_cs_end = '+'(A_as_cs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_cs_potential = [_DECL_4: B_occ_2('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(0, n)))), '*'(_DECL_4, max('..'(0, n))))) | _DECL_4 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_5)).bs), enum2int('[]'(A_as_bs_potential, _DECL_5))) | _DECL_5 in index_set(A_as_objects)]);
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_6)).cs), enum2int('[]'(A_as_cs_potential, _DECL_6))) | _DECL_6 in index_set(A_as_objects)]);
    constraint forall(['='(sum(['+'(bool2int('in'(b, ('[]'(A_objects, enum2int(a))).bs)), bool2int('in'(b, ('[]'(A_objects, enum2int(a))).cs))) | a in as]), 1) | b in B]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).cs, lb(('[]'(A_objects, enum2int(x))).cs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_alias_two_path_parent_exclusivity_snapshot()
 {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_ALIAS_TWO_PATH_PARENT_EXCLUSIVITY,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = '++'(B_bs_objects, B_cs_objects);
    var set of B_potential: B = array_union('++'([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)], [if 'in'(A_occ_0(_DECL_2), A) then ('[]'(A_as_objects, _DECL_2)).cs else {} endif | _DECL_2 in index_set(A_as_objects)]));
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).cs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    array [B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_cs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_as_storage)];
    int: A_as_cs_start = A_as_bs_end;
    int: A_as_cs_end = '+'(A_as_cs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_cs_potential = [_DECL_4: B_occ_2('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(0, n)))), '*'(_DECL_4, max('..'(0, n))))) | _DECL_4 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_5)).bs), enum2int('[]'(A_as_bs_potential, _DECL_5))) | _DECL_5 in index_set(A_as_objects)]);
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_6)).cs), enum2int('[]'(A_as_cs_potential, _DECL_6))) | _DECL_6 in index_set(A_as_objects)]);
    constraint forall(['='(sum([let {
      var set of B_potential: owned = 'union'(('[]'(A_objects, enum2int(a))).bs, ('[]'(A_objects, enum2int(a))).cs);
    } in bool2int('in'(b, owned)) | a in as]), 1) | b in B]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).cs, lb(('[]'(A_objects, enum2int(x))).cs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_filtered_alias_ownership_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_FILTERED_ALIAS_OWNERSHIP,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = '++'(B_bs_objects, B_cs_objects);
    var set of B_potential: B = array_union('++'([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)], [if 'in'(A_occ_0(_DECL_2), A) then ('[]'(A_as_objects, _DECL_2)).cs else {} endif | _DECL_2 in index_set(A_as_objects)]));
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).cs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    array [B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_cs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_as_storage)];
    int: A_as_cs_start = A_as_bs_end;
    int: A_as_cs_end = '+'(A_as_cs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_cs_potential = [_DECL_4: B_occ_2('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(0, n)))), '*'(_DECL_4, max('..'(0, n))))) | _DECL_4 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_5)).bs), enum2int('[]'(A_as_bs_potential, _DECL_5))) | _DECL_5 in index_set(A_as_objects)]);
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_6)).cs), enum2int('[]'(A_as_cs_potential, _DECL_6))) | _DECL_6 in index_set(A_as_objects)]);
    constraint forall(['='(sum([let {
      var set of B_potential: owned = {x | x in 'union'(('[]'(A_objects, enum2int(a))).bs, ('[]'(A_objects, enum2int(a))).cs) where 'in'(x, ('[]'(A_objects, enum2int(a))).bs)};
    } in bool2int('in'(b, owned)) | a in as]), 1) | b in B]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).cs, lb(('[]'(A_objects, enum2int(x))).cs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_filtered_aliases_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_UNDER_BOUNDED_ROOT_TWO_FILTERED_ALIASES,
		expect!([r#"
    int: m;
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = '++'(B_bs_objects, B_cs_objects);
    var set of B_potential: B = array_union('++'([if 'in'(A_occ_0(_DECL_1), A) then ('[]'(A_as_objects, _DECL_1)).bs else {} endif | _DECL_1 in index_set(A_as_objects)], [if 'in'(A_occ_0(_DECL_2), A) then ('[]'(A_as_objects, _DECL_2)).cs else {} endif | _DECL_2 in index_set(A_as_objects)]));
    enum A_potential = A_occ_0('..'(1, m));
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_objects = A_as_objects;
    var set of A_potential: A = as;
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).cs), '..'(0, n)) | this in A]);
    array ['..'(1, m)] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    array [B_occ_2('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_cs_objects;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_as_storage)];
    int: A_as_cs_start = A_as_bs_end;
    int: A_as_cs_end = '+'(A_as_cs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_as_cs_potential = [_DECL_4: B_occ_2('..'('+'(1, '*'('-'(_DECL_4, 1), max('..'(0, n)))), '*'(_DECL_4, max('..'(0, n))))) | _DECL_4 in index_set(A_as_storage)];
    array [int] of record(var set of B_potential: bs, var set of B_potential: cs): A_as_objects = A_as_storage;
    var set( '..'(0, m) ) of  A_occ_0('..'(1, m)): as;
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_5)).bs), enum2int('[]'(A_as_bs_potential, _DECL_5))) | _DECL_5 in index_set(A_as_objects)]);
    constraint forall(['subset'(enum2int(('[]'(A_as_objects, _DECL_6)).cs), enum2int('[]'(A_as_cs_potential, _DECL_6))) | _DECL_6 in index_set(A_as_objects)]);
    constraint forall([let {
      var set of B_potential: owned = 'union'(('[]'(A_objects, enum2int(a))).bs, ('[]'(A_objects, enum2int(a))).cs);
      var set of B_potential: owned_bs = {x | x in owned where 'in'(x, ('[]'(A_objects, enum2int(a))).bs)};
      var set of B_potential: owned_cs = {x | x in owned where 'in'(x, ('[]'(A_objects, enum2int(a))).cs)};
    } in '='('+'(card(owned_bs), card(owned_cs)), '+'(card(('[]'(A_objects, enum2int(a))).bs), card(('[]'(A_objects, enum2int(a))).cs))) | a in as]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).bs, lb(('[]'(A_objects, enum2int(x))).bs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).cs, lb(('[]'(A_objects, enum2int(x))).cs))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_TWO_ROOTS,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 2): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(A_a1_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a1_objects)], [('[]'(A_a2_objects, _DECL_2)).bs | _DECL_2 in index_set(A_a2_objects)]));
    enum A_potential = A_occ_0({1}) ++ A_occ_2({1});
    array [int] of record(var set of B_potential: bs): A_objects = '++'(A_a1_objects, A_a2_objects);
    set of A_potential: A = array_union([A_occ_0({1}), A_occ_2({1})]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a1_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_objects;
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 2;
    int: A_a1_bs_start = 1;
    int: A_a1_bs_end = '+'(A_a1_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a1_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_a1_storage)];
    array [int] of record(var set of B_potential: bs): A_a1_objects = [(bs: bs) | p in index_set(A_a1_storage), input = '[]'(A_a1_storage, p), bs = (input).bs];
    A_potential: a1 = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a1_objects, _DECL_4)).bs), enum2int('[]'(A_a1_bs_potential, _DECL_4))) | _DECL_4 in index_set(A_a1_objects)]);
    array [{1}] of record(var set of B_potential: bs): A_a2_storage;
    array [B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 2): y): B_bs_occ_1_objects;
    int: A_a2_bs_start = A_a1_bs_end;
    int: A_a2_bs_end = '+'(A_a2_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a2_bs_potential = [_DECL_5: B_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(A_a2_storage)];
    array [int] of record(var set of B_potential: bs): A_a2_objects = [(bs: bs) | p in index_set(A_a2_storage), input = '[]'(A_a2_storage, p), bs = (input).bs];
    A_potential: a2 = A_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(A_a2_objects, _DECL_6)).bs), enum2int('[]'(A_a2_bs_potential, _DECL_6))) | _DECL_6 in index_set(A_a2_objects)]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_shared_alias_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_TWO_ROOTS_SHARED_ALIAS,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(A_a1_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a1_objects)], [('[]'(A_a2_objects, _DECL_2)).bs | _DECL_2 in index_set(A_a2_objects)]));
    enum A_potential = A_occ_0({1}) ++ A_occ_2({1});
    array [int] of record(var set of B_potential: bs): A_objects = '++'(A_a1_objects, A_a2_objects);
    set of A_potential: A = array_union([A_occ_0({1}), A_occ_2({1})]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a1_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 2;
    int: A_a1_bs_start = 1;
    int: A_a1_bs_end = '+'(A_a1_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a1_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_a1_storage)];
    array [int] of record(var set of B_potential: bs): A_a1_objects = [(bs: bs) | p in index_set(A_a1_storage), input = '[]'(A_a1_storage, p), bs = (input).bs];
    A_potential: a1 = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a1_objects, _DECL_4)).bs), enum2int('[]'(A_a1_bs_potential, _DECL_4))) | _DECL_4 in index_set(A_a1_objects)]);
    array [{1}] of record(var set of B_potential: bs): A_a2_storage;
    array [B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_occ_1_objects;
    int: A_a2_bs_start = A_a1_bs_end;
    int: A_a2_bs_end = '+'(A_a2_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a2_bs_potential = [_DECL_5: B_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(A_a2_storage)];
    array [int] of record(var set of B_potential: bs): A_a2_objects = [(bs: bs) | p in index_set(A_a2_storage), input = '[]'(A_a2_storage, p), bs = (input).bs];
    A_potential: a2 = A_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(A_a2_objects, _DECL_6)).bs), enum2int('[]'(A_a2_bs_potential, _DECL_6))) | _DECL_6 in index_set(A_a2_objects)]);
    var set of B_potential: bs = B;
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_shared_alias_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_TWO_ROOTS_SHARED_ALIAS_FIELD_ACCESS,
		expect![[r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(A_a1_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a1_objects)], [('[]'(A_a2_objects, _DECL_2)).bs | _DECL_2 in index_set(A_a2_objects)]));
    enum A_potential = A_occ_0({1}) ++ A_occ_2({1});
    array [int] of record(var set of B_potential: bs): A_objects = '++'(A_a1_objects, A_a2_objects);
    set of A_potential: A = array_union([A_occ_0({1}), A_occ_2({1})]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a1_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 2;
    int: A_a1_bs_start = 1;
    int: A_a1_bs_end = '+'(A_a1_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a1_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_a1_storage)];
    array [int] of record(var set of B_potential: bs): A_a1_objects = [(bs: bs) | p in index_set(A_a1_storage), input = '[]'(A_a1_storage, p), bs = (input).bs];
    A_potential: a1 = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a1_objects, _DECL_4)).bs), enum2int('[]'(A_a1_bs_potential, _DECL_4))) | _DECL_4 in index_set(A_a1_objects)]);
    array [{1}] of record(var set of B_potential: bs): A_a2_storage;
    array [B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_occ_1_objects;
    int: A_a2_bs_start = A_a1_bs_end;
    int: A_a2_bs_end = '+'(A_a2_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a2_bs_potential = [_DECL_5: B_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(A_a2_storage)];
    array [int] of record(var set of B_potential: bs): A_a2_objects = [(bs: bs) | p in index_set(A_a2_storage), input = '[]'(A_a2_storage, p), bs = (input).bs];
    A_potential: a2 = A_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(A_a2_objects, _DECL_6)).bs), enum2int('[]'(A_a2_bs_potential, _DECL_6))) | _DECL_6 in index_set(A_a2_objects)]);
    var set of B_potential: bs = B;
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in bs]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_composite_consumers_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_TWO_ROOTS_COMPOSITE_CONSUMERS,
		expect![[r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(A_a1_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a1_objects)], [('[]'(A_a2_objects, _DECL_2)).bs | _DECL_2 in index_set(A_a2_objects)]));
    enum A_potential = A_occ_0({1}) ++ A_occ_2({1});
    array [int] of record(var set of B_potential: bs): A_objects = '++'(A_a1_objects, A_a2_objects);
    set of A_potential: A = array_union([A_occ_0({1}), A_occ_2({1})]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a1_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 2;
    int: A_a1_bs_start = 1;
    int: A_a1_bs_end = '+'(A_a1_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a1_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_a1_storage)];
    array [int] of record(var set of B_potential: bs): A_a1_objects = [(bs: bs) | p in index_set(A_a1_storage), input = '[]'(A_a1_storage, p), bs = (input).bs];
    A_potential: a1 = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a1_objects, _DECL_4)).bs), enum2int('[]'(A_a1_bs_potential, _DECL_4))) | _DECL_4 in index_set(A_a1_objects)]);
    array [{1}] of record(var set of B_potential: bs): A_a2_storage;
    array [B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_occ_1_objects;
    int: A_a2_bs_start = A_a1_bs_end;
    int: A_a2_bs_end = '+'(A_a2_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a2_bs_potential = [_DECL_5: B_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(A_a2_storage)];
    array [int] of record(var set of B_potential: bs): A_a2_objects = [(bs: bs) | p in index_set(A_a2_storage), input = '[]'(A_a2_storage, p), bs = (input).bs];
    A_potential: a2 = A_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(A_a2_objects, _DECL_6)).bs), enum2int('[]'(A_a2_bs_potential, _DECL_6))) | _DECL_6 in index_set(A_a2_objects)]);
    var set of B_potential: bs1 = ('[]'(A_objects, enum2int(a1))).bs;
    var set of B_potential: bs2 = ('[]'(A_objects, enum2int(a2))).bs;
    var set of B_potential: all_bs = B;
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in bs1]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in bs2]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in all_bs]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_alias_field_access_snapshot()
 {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_ALIAS_FIELD_ACCESS,
		expect![[r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(Root_r1_objects, _DECL_1)).bs | _DECL_1 in index_set(Root_r1_objects)], [('[]'(Root_r2_objects, _DECL_2)).bs | _DECL_2 in index_set(Root_r2_objects)]));
    enum A_potential = A_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ A_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x): A_objects = '++'(A_bs_objects, A_bs_occ_1_objects);
    var set of A_potential: A = array_union('++'([if 'in'(_DECL_3, B) then {A_occ_1(enum2int(_DECL_3))} else {} endif | _DECL_3 in index_set(B_bs_objects)], [if 'in'(_DECL_4, B) then {A_occ_3('+'(1, '-'(enum2int(_DECL_4), Root_r1_bs_end)))} else {} endif | _DECL_4 in index_set(B_bs_occ_1_objects)]));
    enum Root_potential = Root_occ_0({1}) ++ Root_occ_2({1});
    array [int] of record(var set of B_potential: bs): Root_objects = '++'(Root_r1_objects, Root_r2_objects);
    set of Root_potential: Root = array_union([Root_occ_0({1}), Root_occ_2({1})]);
    constraint forall(['in'(card(('[]'(Root_objects, enum2int(this))).bs), '..'(0, n)) | this in Root]);
    array [{1}] of record(var set of B_potential: bs): Root_r1_storage;
    array [B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_objects = [(x: (proj).x) | proj in B_bs_objects];
    int: Root_r1_bs_A_start = 1;
    int: Root_r1_bs_A_end = 2;
    int: Root_r1_bs_start = 1;
    int: Root_r1_bs_end = '+'(Root_r1_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r1_bs_potential = [_DECL_5: B_occ_1('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(Root_r1_storage)];
    int: Root_r1_root_start = 1;
    int: Root_r1_root_end = 2;
    array [int] of record(var set of B_potential: bs): Root_r1_objects = [(bs: bs) | p in index_set(Root_r1_storage), input = '[]'(Root_r1_storage, p), bs = (input).bs];
    Root_potential: r1 = Root_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r1_objects, _DECL_6)).bs), enum2int('[]'(Root_r1_bs_potential, _DECL_6))) | _DECL_6 in index_set(Root_r1_objects)]);
    array [{1}] of record(var set of B_potential: bs): Root_r2_storage;
    array [B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_occ_1_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_occ_1_objects = [(x: (proj).x) | proj in B_bs_occ_1_objects];
    int: Root_r2_bs_A_start = Root_r1_bs_A_end;
    int: Root_r2_bs_A_end = '+'(Root_r2_bs_A_start, 1);
    int: Root_r2_bs_start = Root_r1_bs_end;
    int: Root_r2_bs_end = '+'(Root_r2_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r2_bs_potential = [_DECL_7: B_occ_3('..'('+'(1, '*'('-'(_DECL_7, 1), max('..'(0, n)))), '*'(_DECL_7, max('..'(0, n))))) | _DECL_7 in index_set(Root_r2_storage)];
    array [int] of record(var set of B_potential: bs): Root_r2_objects = [(bs: bs) | p in index_set(Root_r2_storage), input = '[]'(Root_r2_storage, p), bs = (input).bs];
    Root_potential: r2 = Root_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r2_objects, _DECL_8)).bs), enum2int('[]'(Root_r2_bs_potential, _DECL_8))) | _DECL_8 in index_set(Root_r2_objects)]);
    var set of A_potential: as = A;
    constraint forall(['>='(('[]'(A_objects, enum2int(a))).x, 0) | a in as]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_class_constraint_snapshot()
 {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_CLASS_CONSTRAINT,
		expect![[r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(Root_r1_objects, _DECL_1)).bs | _DECL_1 in index_set(Root_r1_objects)], [('[]'(Root_r2_objects, _DECL_2)).bs | _DECL_2 in index_set(Root_r2_objects)]));
    enum A_potential = A_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ A_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x): A_objects = '++'(A_bs_objects, A_bs_occ_1_objects);
    var set of A_potential: A = array_union('++'([if 'in'(_DECL_3, B) then {A_occ_1(enum2int(_DECL_3))} else {} endif | _DECL_3 in index_set(B_bs_objects)], [if 'in'(_DECL_4, B) then {A_occ_3('+'(1, '-'(enum2int(_DECL_4), Root_r1_bs_end)))} else {} endif | _DECL_4 in index_set(B_bs_occ_1_objects)]));
    constraint forall(['>='(('[]'(A_objects, enum2int(this))).x, 0) | this in A]);
    enum Root_potential = Root_occ_0({1}) ++ Root_occ_2({1});
    array [int] of record(var set of B_potential: bs): Root_objects = '++'(Root_r1_objects, Root_r2_objects);
    set of Root_potential: Root = array_union([Root_occ_0({1}), Root_occ_2({1})]);
    constraint forall(['in'(card(('[]'(Root_objects, enum2int(this))).bs), '..'(0, n)) | this in Root]);
    array [{1}] of record(var set of B_potential: bs): Root_r1_storage;
    array [B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_objects = [(x: (proj).x) | proj in B_bs_objects];
    int: Root_r1_bs_A_start = 1;
    int: Root_r1_bs_A_end = 2;
    int: Root_r1_bs_start = 1;
    int: Root_r1_bs_end = '+'(Root_r1_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r1_bs_potential = [_DECL_5: B_occ_1('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(Root_r1_storage)];
    int: Root_r1_root_start = 1;
    int: Root_r1_root_end = 2;
    array [int] of record(var set of B_potential: bs): Root_r1_objects = [(bs: bs) | p in index_set(Root_r1_storage), input = '[]'(Root_r1_storage, p), bs = (input).bs];
    Root_potential: r1 = Root_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r1_objects, _DECL_6)).bs), enum2int('[]'(Root_r1_bs_potential, _DECL_6))) | _DECL_6 in index_set(Root_r1_objects)]);
    array [{1}] of record(var set of B_potential: bs): Root_r2_storage;
    array [B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_occ_1_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_occ_1_objects = [(x: (proj).x) | proj in B_bs_occ_1_objects];
    int: Root_r2_bs_A_start = Root_r1_bs_A_end;
    int: Root_r2_bs_A_end = '+'(Root_r2_bs_A_start, 1);
    int: Root_r2_bs_start = Root_r1_bs_end;
    int: Root_r2_bs_end = '+'(Root_r2_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r2_bs_potential = [_DECL_7: B_occ_3('..'('+'(1, '*'('-'(_DECL_7, 1), max('..'(0, n)))), '*'(_DECL_7, max('..'(0, n))))) | _DECL_7 in index_set(Root_r2_storage)];
    array [int] of record(var set of B_potential: bs): Root_r2_objects = [(bs: bs) | p in index_set(Root_r2_storage), input = '[]'(Root_r2_storage, p), bs = (input).bs];
    Root_potential: r2 = Root_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r2_objects, _DECL_8)).bs), enum2int('[]'(Root_r2_bs_potential, _DECL_8))) | _DECL_8 in index_set(Root_r2_objects)]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_composite_consumers_snapshot()
 {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_BOUNDED_SET_TWO_ROOTS_SUPERCLASS_COMPOSITE_CONSUMERS,
		expect![[r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(Root_r1_objects, _DECL_1)).bs | _DECL_1 in index_set(Root_r1_objects)], [('[]'(Root_r2_objects, _DECL_2)).bs | _DECL_2 in index_set(Root_r2_objects)]));
    enum A_potential = A_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n))))) ++ A_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): x): A_objects = '++'(A_bs_objects, A_bs_occ_1_objects);
    var set of A_potential: A = array_union('++'([if 'in'(_DECL_3, B) then {A_occ_1(enum2int(_DECL_3))} else {} endif | _DECL_3 in index_set(B_bs_objects)], [if 'in'(_DECL_4, B) then {A_occ_3('+'(1, '-'(enum2int(_DECL_4), Root_r1_bs_end)))} else {} endif | _DECL_4 in index_set(B_bs_occ_1_objects)]));
    enum Root_potential = Root_occ_0({1}) ++ Root_occ_2({1});
    array [int] of record(var set of B_potential: bs): Root_objects = '++'(Root_r1_objects, Root_r2_objects);
    set of Root_potential: Root = array_union([Root_occ_0({1}), Root_occ_2({1})]);
    constraint forall(['in'(card(('[]'(Root_objects, enum2int(this))).bs), '..'(0, n)) | this in Root]);
    array [{1}] of record(var set of B_potential: bs): Root_r1_storage;
    array [B_occ_1('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_objects = [(x: (proj).x) | proj in B_bs_objects];
    int: Root_r1_bs_A_start = 1;
    int: Root_r1_bs_A_end = 2;
    int: Root_r1_bs_start = 1;
    int: Root_r1_bs_end = '+'(Root_r1_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r1_bs_potential = [_DECL_5: B_occ_1('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(Root_r1_storage)];
    int: Root_r1_root_start = 1;
    int: Root_r1_root_end = 2;
    array [int] of record(var set of B_potential: bs): Root_r1_objects = [(bs: bs) | p in index_set(Root_r1_storage), input = '[]'(Root_r1_storage, p), bs = (input).bs];
    Root_potential: r1 = Root_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r1_objects, _DECL_6)).bs), enum2int('[]'(Root_r1_bs_potential, _DECL_6))) | _DECL_6 in index_set(Root_r1_objects)]);
    array [{1}] of record(var set of B_potential: bs): Root_r2_storage;
    array [B_occ_3('..'(1, '*'(card(Root_potential), max('..'(0, n)))))] of record(var '..'(0, 4): x, var '..'(0, 4): y): B_bs_occ_1_objects;
    array [int] of record(var '..'(0, 4): x): A_bs_occ_1_objects = [(x: (proj).x) | proj in B_bs_occ_1_objects];
    int: Root_r2_bs_A_start = Root_r1_bs_A_end;
    int: Root_r2_bs_A_end = '+'(Root_r2_bs_A_start, 1);
    int: Root_r2_bs_start = Root_r1_bs_end;
    int: Root_r2_bs_end = '+'(Root_r2_bs_start, '*'(card(Root_potential), max('..'(0, n))));
    array [int] of set of B_potential: Root_r2_bs_potential = [_DECL_7: B_occ_3('..'('+'(1, '*'('-'(_DECL_7, 1), max('..'(0, n)))), '*'(_DECL_7, max('..'(0, n))))) | _DECL_7 in index_set(Root_r2_storage)];
    array [int] of record(var set of B_potential: bs): Root_r2_objects = [(bs: bs) | p in index_set(Root_r2_storage), input = '[]'(Root_r2_storage, p), bs = (input).bs];
    Root_potential: r2 = Root_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(Root_r2_objects, _DECL_8)).bs), enum2int('[]'(Root_r2_bs_potential, _DECL_8))) | _DECL_8 in index_set(Root_r2_objects)]);
    var set of B_potential: bs1 = ('[]'(Root_objects, enum2int(r1))).bs;
    var set of B_potential: bs2 = ('[]'(Root_objects, enum2int(r2))).bs;
    var set of A_potential: as = A;
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in bs1]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in bs2]);
    constraint forall(['>='(('[]'(A_objects, enum2int(a))).x, 0) | a in as]);
    constraint forall(['\/'('in'(x, A), '='(('[]'(A_objects, enum2int(x))).x, mzn_safe_default(('[]'(A_objects, enum2int(x))).x))) | x in A_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).x, mzn_safe_default(('[]'(B_objects, enum2int(x))).x))) | x in B_potential]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]],
	);
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_class_constraint_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_SET_TWO_ROOTS_CLASS_CONSTRAINT,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n))))) ++ B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = '++'(B_bs_objects, B_bs_occ_1_objects);
    var set of B_potential: B = array_union('++'([('[]'(A_a1_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a1_objects)], [('[]'(A_a2_objects, _DECL_2)).bs | _DECL_2 in index_set(A_a2_objects)]));
    constraint forall(['>='(('[]'(B_objects, enum2int(this))).y, 0) | this in B]);
    enum A_potential = A_occ_0({1}) ++ A_occ_2({1});
    array [int] of record(var set of B_potential: bs): A_objects = '++'(A_a1_objects, A_a2_objects);
    set of A_potential: A = array_union([A_occ_0({1}), A_occ_2({1})]);
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a1_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a1_root_start = 1;
    int: A_a1_root_end = 2;
    int: A_a1_bs_start = 1;
    int: A_a1_bs_end = '+'(A_a1_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a1_bs_potential = [_DECL_3: B_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(0, n)))), '*'(_DECL_3, max('..'(0, n))))) | _DECL_3 in index_set(A_a1_storage)];
    array [int] of record(var set of B_potential: bs): A_a1_objects = [(bs: bs) | p in index_set(A_a1_storage), input = '[]'(A_a1_storage, p), bs = (input).bs];
    A_potential: a1 = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a1_objects, _DECL_4)).bs), enum2int('[]'(A_a1_bs_potential, _DECL_4))) | _DECL_4 in index_set(A_a1_objects)]);
    array [{1}] of record(var set of B_potential: bs): A_a2_storage;
    array [B_occ_3('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_occ_1_objects;
    int: A_a2_bs_start = A_a1_bs_end;
    int: A_a2_bs_end = '+'(A_a2_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a2_bs_potential = [_DECL_5: B_occ_3('..'('+'(1, '*'('-'(_DECL_5, 1), max('..'(0, n)))), '*'(_DECL_5, max('..'(0, n))))) | _DECL_5 in index_set(A_a2_storage)];
    array [int] of record(var set of B_potential: bs): A_a2_objects = [(bs: bs) | p in index_set(A_a2_storage), input = '[]'(A_a2_storage, p), bs = (input).bs];
    A_potential: a2 = A_occ_2(1);
    constraint forall(['subset'(enum2int(('[]'(A_a2_objects, _DECL_6)).bs), enum2int('[]'(A_a2_bs_potential, _DECL_6))) | _DECL_6 in index_set(A_a2_objects)]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_var_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_NEW,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_b_objects);
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var B_potential: b, var '..'(0, 4): x): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var B_potential: b, var '..'(0, 4): x): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 4): y): B_b_objects;
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    array [int] of record(var B_potential: b, var '..'(0, 4): x): A_a_objects = [(b: b, x: x) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), b = (input).b, x = (input).x];
    A_potential: a = A_occ_0(1);
    constraint '>='(('[]'(B_objects, enum2int(('[]'(A_objects, enum2int(a))).b))).y, 0);
    constraint forall(['='(('[]'(A_a_objects, _DECL_2)).b, B_occ_1(_DECL_2)) | _DECL_2 in index_set(A_a_objects)]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_NEW,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_b_objects);
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var B_potential: b, var '..'(0, 4): x): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var B_potential: b, var '..'(0, 4): x): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 4): y): B_b_objects;
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    array [int] of record(var B_potential: b, var '..'(0, 4): x): A_a_objects = [(b: b, x: x) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), b = (input).b, x = (input).x];
    A_potential: a = A_occ_0(1);
    constraint '>='(('[]'(B_objects, enum2int(('[]'(A_objects, enum2int(a))).b))).y, 0);
    constraint forall(['='(('[]'(A_a_objects, _DECL_2)).b, B_occ_1(_DECL_2)) | _DECL_2 in index_set(A_a_objects)]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_var_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_VAR_NEW_NO_CONSTRAINT,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_b_objects);
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var B_potential: b): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var B_potential: b): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 2): y): B_b_objects;
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    array [int] of record(var B_potential: b): A_a_objects = [(b: b) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), b = (input).b];
    A_potential: a = A_occ_0(1);
    constraint forall(['='(('[]'(A_a_objects, _DECL_2)).b, B_occ_1(_DECL_2)) | _DECL_2 in index_set(A_a_objects)]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_var_opt_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_VAR_OPT_NEW_NO_CONSTRAINT,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 2): y): B_objects = array1d(B_b_objects);
    var set of B_potential: B = array_union([if occurs(('[]'(A_a_objects, _DECL_1)).b) then {B_occ_1(_DECL_1)} else {} endif | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var opt B_potential: b): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [{1}] of record(var opt B_potential: b): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 2): y): B_b_objects;
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    array [int] of record(var opt B_potential: b): A_a_objects = [(b: b) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), b = (input).b];
    A_potential: a = A_occ_0(1);
    constraint forall(['->'(occurs(('[]'(A_a_objects, _DECL_2)).b), '='(('[]'(A_a_objects, _DECL_2)).b, B_occ_1(_DECL_2))) | _DECL_2 in index_set(A_a_objects)]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_nested_var_field_access_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_NESTED_NEW,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_b_objects);
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_C_c_objects)]);
    enum C_potential = C_occ_0({1});
    array [int] of record(var B_potential: b, var '..'(0, 4): z): C_objects = C_c_objects;
    set of C_potential: C = C_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(var B_potential: b): A_objects = A_C_c_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_2)} | _DECL_2 in index_set(C_c_objects)]);
    array [{1}] of record(var B_potential: b, var '..'(0, 4): z): C_c_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 4): y): B_b_objects;
    int: C_c__start = 1;
    int: C_c__end = 2;
    int: C_c_b_start = 1;
    int: C_c_b_end = '+'(C_c_b_start, '*'(card(A_potential), 1));
    array [int] of record(var B_potential: b, var '..'(0, 4): z): C_c_objects = [(b: b, z: z) | p in index_set(C_c_storage), input = '[]'(C_c_storage, p), b = (input).b, z = (input).z];
    array [int] of record(var B_potential: b): A_C_c_objects = [(b: ('[]'(C_c_objects, p)).b) | p in index_set(C_c_objects)];
    C_potential: c = C_occ_0(1);
    constraint '<='(('[]'(B_objects, enum2int(('[]'(C_objects, enum2int(c))).b))).y, ('[]'(C_objects, enum2int(c))).z);
    constraint forall(['='(('[]'(A_C_c_objects, _DECL_3)).b, B_occ_1(_DECL_3)) | _DECL_3 in index_set(A_C_c_objects)]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_nested_new_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_NESTED_NEW,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_b_objects);
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_C_c_objects)]);
    enum C_potential = C_occ_0({1});
    array [int] of record(var B_potential: b, var '..'(0, 4): z): C_objects = C_c_objects;
    set of C_potential: C = C_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(var B_potential: b): A_objects = A_C_c_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_2)} | _DECL_2 in index_set(C_c_objects)]);
    array [{1}] of record(var B_potential: b, var '..'(0, 4): z): C_c_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), 1)))] of record(var '..'(0, 4): y): B_b_objects;
    int: C_c__start = 1;
    int: C_c__end = 2;
    int: C_c_b_start = 1;
    int: C_c_b_end = '+'(C_c_b_start, '*'(card(A_potential), 1));
    array [int] of record(var B_potential: b, var '..'(0, 4): z): C_c_objects = [(b: b, z: z) | p in index_set(C_c_storage), input = '[]'(C_c_storage, p), b = (input).b, z = (input).z];
    array [int] of record(var B_potential: b): A_C_c_objects = [(b: ('[]'(C_c_objects, p)).b) | p in index_set(C_c_objects)];
    C_potential: c = C_occ_0(1);
    constraint '<='(('[]'(B_objects, enum2int(('[]'(C_objects, enum2int(c))).b))).y, ('[]'(C_objects, enum2int(c))).z);
    constraint forall(['='(('[]'(A_C_c_objects, _DECL_3)).b, B_occ_1(_DECL_3)) | _DECL_3 in index_set(A_C_c_objects)]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_par_set_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_PAR_SET_NEW,
		expect!([r#"
    int: n = 2;
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_as_inputs])));
    array [int] of record(int: b): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_as_bs_end, 1)))]);
    enum A_potential = A_occ_0({1});
    array [int] of record(int: a, set of B_potential: bs): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: a, array [int] of record(int: b): bs): A_as_inputs = [(a: 1, bs: [(b: 1), (b: 2)])];
    constraint forall(['in'(length((i).bs), '..'(0, n)) | i in A_as_inputs]);
    array [int] of record(int: b): B_bs_objects = [k | i in A_as_inputs, k in (i).bs];
    int: A_as_root_start = 1;
    int: A_as_root_end = 2;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, sum([length((i).bs) | i in A_as_inputs]));
    array [int] of record(int: a, set of B_potential: bs): A_as_objects = [(a: a, bs: bs) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), a = (input).a, bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of A_occ_0({1}): as = A_occ_0({1});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_bounded_par_set_under_var_root_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_BOUNDED_PAR_SET_UNDER_VAR_ROOT_FIELD_ACCESS,
		expect!([r#"
    int: n;
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))));
    array [int] of record(var '..'(0, 4): y): B_objects = array1d(B_bs_objects);
    var set of B_potential: B = array_union([('[]'(A_a_objects, _DECL_1)).bs | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(var set of B_potential: bs): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    constraint forall(['in'(card(('[]'(A_objects, enum2int(this))).bs), '..'(0, n)) | this in A]);
    array [{1}] of record(var set of B_potential: bs): A_a_storage;
    array [B_occ_1('..'(1, '*'(card(A_potential), max('..'(0, n)))))] of record(var '..'(0, 4): y): B_bs_objects;
    int: A_a_bs_start = 1;
    int: A_a_bs_end = '+'(A_a_bs_start, '*'(card(A_potential), max('..'(0, n))));
    array [int] of set of B_potential: A_a_bs_potential = [_DECL_2: B_occ_1('..'('+'(1, '*'('-'(_DECL_2, 1), max('..'(0, n)))), '*'(_DECL_2, max('..'(0, n))))) | _DECL_2 in index_set(A_a_storage)];
    array [int] of record(var set of B_potential: bs): A_a_objects = [(bs: bs) | p in index_set(A_a_storage), input = '[]'(A_a_storage, p), bs = (input).bs];
    A_potential: a = A_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(A_a_objects, _DECL_3)).bs), enum2int('[]'(A_a_bs_potential, _DECL_3))) | _DECL_3 in index_set(A_a_objects)]);
    constraint forall(['>='(('[]'(B_objects, enum2int(b))).y, 0) | b in ('[]'(A_objects, enum2int(a))).bs]);
    constraint forall(['\/'('in'(x, B), '='(('[]'(B_objects, enum2int(x))).y, mzn_safe_default(('[]'(B_objects, enum2int(x))).y))) | x in B_potential]);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_par_set_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_PAR_SET_NEW,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_as_inputs])));
    array [int] of record(int: b): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_as_bs_end, 1)))]);
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: a, set of B_potential: bs): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: a, array [int] of record(int: b): bs): A_as_inputs = [(a: 1, bs: [(b: 1), (b: 2)]), (a: 2, bs: [(b: 3)])];
    array [int] of record(int: b): B_bs_objects = [k | i in A_as_inputs, k in (i).bs];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, sum([length((i).bs) | i in A_as_inputs]));
    array [int] of record(int: a, set of B_potential: bs): A_as_objects = [(a: a, bs: bs) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), a = (input).a, bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_par_set_new_mixed_scalar_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_PAR_SET_NEW_MIXED_SCALAR,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in A_as_inputs])));
    array [int] of record(int: b): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(A_as_bs_end, 1)))]);
    enum A_potential = A_occ_0({1, 2});
    array [int] of record(int: a, set of B_potential: bs, var '..'(0, 2): x): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: a, array [int] of record(int: b): bs): A_as_inputs = [(a: 1, bs: [(b: 1), (b: 2)]), (a: 2, bs: [(b: 3)])];
    array [int] of record(int: b): B_bs_objects = [k | i in A_as_inputs, k in (i).bs];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    int: A_as_bs_start = 1;
    int: A_as_bs_end = '+'(A_as_bs_start, sum([length((i).bs) | i in A_as_inputs]));
    array [int] of record(int: a, set of B_potential: bs, var '..'(0, 2): x): A_as_objects = [(a: a, bs: bs, x: x) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), a = (input).a, x = let {
      var '..'(0, 2): x_init;
    } in x_init, bs = B_occ_1('..'('+'(1, sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(A_as_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_par_set_two_roots_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_PAR_SET_TWO_ROOTS,
		expect!([r#"
    enum Leaf_potential = Leaf_occ_1('..'(1, sum([length((i).leaves) | i in Node_left_inputs]))) ++ Leaf_occ_3('..'(1, sum([length((i).leaves) | i in Node_right_inputs])));
    array [int] of record(int: value): Leaf_objects = '++'(Leaf_leaves_objects, Leaf_leaves_occ_1_objects);
    set of Leaf_potential: Leaf = array_union('++'([Leaf_occ_1('..'(1, '-'(Node_left_leaves_end, 1)))], [Leaf_occ_3('..'(1, '-'(Node_right_leaves_end, Node_left_leaves_end)))]));
    enum Node_potential = Node_occ_0({1, 2}) ++ Node_occ_2('..'(Node_left_root_end, '-'(Node_right_root_end, 1)));
    array [int] of record(set of Leaf_potential: leaves): Node_objects = '++'(Node_left_objects, Node_right_objects);
    set of Node_potential: Node = array_union([left, right]);
    array [int] of record(array [int] of record(int: value): leaves): Node_left_inputs = [(leaves: [(value: 1), (value: 2)]), (leaves: [(value: 3)])];
    array [int] of record(int: value): Leaf_leaves_objects = [k | i in Node_left_inputs, k in (i).leaves];
    int: Node_left_leaves_start = 1;
    int: Node_left_leaves_end = '+'(Node_left_leaves_start, sum([length((i).leaves) | i in Node_left_inputs]));
    int: Node_left_root_start = 1;
    int: Node_left_root_end = 3;
    array [int] of record(set of Leaf_potential: leaves): Node_left_objects = [(leaves: leaves) | p in index_set(Node_left_inputs), input = '[]'(Node_left_inputs, p), leaves = Leaf_occ_1('..'('+'(1, sum([length(('[]'(Node_left_inputs, q)).leaves) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Node_left_inputs, q)).leaves) | q in '..'(1, '-'(p, 1))]), length((input).leaves))))];
    set of Node_occ_0({1, 2}): left = Node_occ_0({1, 2});
    array [int] of record(array [int] of record(int: value): leaves): Node_right_inputs = [(leaves: [(value: 4)]), (leaves: [(value: 5), (value: 6)])];
    array [int] of record(int: value): Leaf_leaves_occ_1_objects = [k | i in Node_right_inputs, k in (i).leaves];
    int: Node_right_leaves_start = Node_left_leaves_end;
    int: Node_right_leaves_end = '+'(Node_right_leaves_start, sum([length((i).leaves) | i in Node_right_inputs]));
    int: Node_right_root_start = Node_left_root_end;
    int: Node_right_root_end = '+'(Node_right_root_start, 2);
    array [int] of record(set of Leaf_potential: leaves): Node_right_objects = [(leaves: leaves) | p in index_set(Node_right_inputs), input = '[]'(Node_right_inputs, p), leaves = Leaf_occ_3('..'('+'(1, sum([length(('[]'(Node_right_inputs, q)).leaves) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Node_right_inputs, q)).leaves) | q in '..'(1, '-'(p, 1))]), length((input).leaves))))];
    set of Node_occ_2('..'(Node_left_root_end, '-'(Node_right_root_end, 1))): right = Node_occ_2('..'(Node_left_root_end, '-'(Node_right_root_end, 1)));
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_deep_nested_par_set_two_roots_snapshot() {
	check_thir_model_with_stdlib(
		DEEP_NESTED_PAR_SET_TWO_ROOTS,
		expect!([r#"
    enum A_potential = A_occ_2('..'(1, sum([length((j1).a1) | i in C_c1_inputs, j1 in (i).b1]))) ++ A_occ_3('..'(1, sum([length((j1).a2) | i in C_c1_inputs, j1 in (i).b1]))) ++ A_occ_5('..'(1, sum([length((j1).a1) | i in C_c1_inputs, j1 in (i).b2]))) ++ A_occ_6('..'(1, sum([length((j1).a2) | i in C_c1_inputs, j1 in (i).b2]))) ++ A_occ_9('..'(1, sum([length((j1).a1) | i in C_c2_inputs, j1 in (i).b1]))) ++ A_occ_10('..'(1, sum([length((j1).a2) | i in C_c2_inputs, j1 in (i).b1]))) ++ A_occ_12('..'(1, sum([length((j1).a1) | i in C_c2_inputs, j1 in (i).b2]))) ++ A_occ_13('..'(1, sum([length((j1).a2) | i in C_c2_inputs, j1 in (i).b2])));
    array [int] of record(int: x): A_objects = '++'('++'('++'('++'('++'('++'('++'(A_b1_a1_objects, A_b1_a2_objects), A_b2_a1_objects), A_b2_a2_objects), A_b1_a1_occ_4_objects), A_b1_a2_occ_5_objects), A_b2_a1_occ_6_objects), A_b2_a2_occ_7_objects);
    set of A_potential: A = array_union('++'('++'('++'('++'('++'('++'('++'([A_occ_2('..'(1, '-'(C_c1_b1_a1_end, 1)))], [A_occ_3('..'(1, '-'(C_c1_b1_a2_end, C_c1_b1_a1_end)))]), [A_occ_5('..'(1, '-'(C_c1_b2_a1_end, C_c1_b1_a2_end)))]), [A_occ_6('..'(1, '-'(C_c1_b2_a2_end, C_c1_b2_a1_end)))]), [A_occ_9('..'(1, '-'(C_c2_b1_a1_end, C_c1_b2_a2_end)))]), [A_occ_10('..'(1, '-'(C_c2_b1_a2_end, C_c2_b1_a1_end)))]), [A_occ_12('..'(1, '-'(C_c2_b2_a1_end, C_c2_b1_a2_end)))]), [A_occ_13('..'(1, '-'(C_c2_b2_a2_end, C_c2_b2_a1_end)))]));
    enum B_potential = B_occ_1('..'(1, sum([length((i).b1) | i in C_c1_inputs]))) ++ B_occ_4('..'(1, sum([length((i).b2) | i in C_c1_inputs]))) ++ B_occ_8('..'(1, sum([length((i).b1) | i in C_c2_inputs]))) ++ B_occ_11('..'(1, sum([length((i).b2) | i in C_c2_inputs])));
    array [int] of record(set of A_potential: a1, set of A_potential: a2): B_objects = '++'('++'('++'(B_b1_objects, B_b2_objects), B_b1_occ_2_objects), B_b2_occ_3_objects);
    set of B_potential: B = array_union('++'('++'('++'([B_occ_1('..'(1, '-'(C_c1_b1_end, 1)))], [B_occ_4('..'(1, '-'(C_c1_b2_end, C_c1_b1_end)))]), [B_occ_8('..'(1, '-'(C_c2_b1_end, C_c1_b2_end)))]), [B_occ_11('..'(1, '-'(C_c2_b2_end, C_c2_b1_end)))]));
    enum C_potential = C_occ_0({1}) ++ C_occ_7('..'(C_c1_root_end, '-'(C_c2_root_end, 1)));
    array [int] of record(set of B_potential: b1, set of B_potential: b2): C_objects = '++'(C_c1_objects, C_c2_objects);
    set of C_potential: C = array_union([c1, c2]);
    array [int] of record(array [int] of record(array [int] of record(int: x): a1, array [int] of record(int: x): a2): b1, array [int] of record(array [int] of record(int: x): a1, array [int] of record(int: x): a2): b2): C_c1_inputs = [(b1: [(a1: [(x: 1)], a2: [(x: 2), (x: 3)])], b2: [(a1: [(x: 4), (x: 5)], a2: [])])];
    array [int] of record(set of A_potential: a1, set of A_potential: a2): B_b1_objects = [(a1: a1, a2: a2) | p in index_set(C_c1_inputs), r in index_set(('[]'(C_c1_inputs, p)).b1), input = '[]'(('[]'(C_c1_inputs, p)).b1, r), a1 = A_occ_2('..'('+'(1, '+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b1, s)).a1) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b1, s)).a1) | s in '..'(1, '-'(r, 1))])), length((input).a1)))), a2 = A_occ_3('..'('+'(1, '+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b1, s)).a2) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b1, s)).a2) | s in '..'(1, '-'(r, 1))])), length((input).a2))))];
    array [int] of record(set of A_potential: a1, set of A_potential: a2): B_b2_objects = [(a1: a1, a2: a2) | p in index_set(C_c1_inputs), r in index_set(('[]'(C_c1_inputs, p)).b2), input = '[]'(('[]'(C_c1_inputs, p)).b2, r), a1 = A_occ_5('..'('+'(1, '+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b2, s)).a1) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b2, s)).a1) | s in '..'(1, '-'(r, 1))])), length((input).a1)))), a2 = A_occ_6('..'('+'(1, '+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b2, s)).a2) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c1_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c1_inputs, p)).b2, s)).a2) | s in '..'(1, '-'(r, 1))])), length((input).a2))))];
    array [int] of record(int: x): A_b1_a1_objects = [k | i in C_c1_inputs, j1 in (i).b1, k in (j1).a1];
    array [int] of record(int: x): A_b1_a2_objects = [k | i in C_c1_inputs, j1 in (i).b1, k in (j1).a2];
    array [int] of record(int: x): A_b2_a1_objects = [k | i in C_c1_inputs, j1 in (i).b2, k in (j1).a1];
    array [int] of record(int: x): A_b2_a2_objects = [k | i in C_c1_inputs, j1 in (i).b2, k in (j1).a2];
    int: C_c1_b1_a1_start = 1;
    int: C_c1_b1_a1_end = '+'(C_c1_b1_a1_start, sum([length((j1).a1) | i in C_c1_inputs, j1 in (i).b1]));
    int: C_c1_b1_a2_start = C_c1_b1_a1_end;
    int: C_c1_b1_a2_end = '+'(C_c1_b1_a2_start, sum([length((j1).a2) | i in C_c1_inputs, j1 in (i).b1]));
    int: C_c1_b2_a1_start = C_c1_b1_a2_end;
    int: C_c1_b2_a1_end = '+'(C_c1_b2_a1_start, sum([length((j1).a1) | i in C_c1_inputs, j1 in (i).b2]));
    int: C_c1_b2_a2_start = C_c1_b2_a1_end;
    int: C_c1_b2_a2_end = '+'(C_c1_b2_a2_start, sum([length((j1).a2) | i in C_c1_inputs, j1 in (i).b2]));
    int: C_c1_b1_start = 1;
    int: C_c1_b1_end = '+'(C_c1_b1_start, sum([length((i).b1) | i in C_c1_inputs]));
    int: C_c1_b2_start = C_c1_b1_end;
    int: C_c1_b2_end = '+'(C_c1_b2_start, sum([length((i).b2) | i in C_c1_inputs]));
    int: C_c1_root_start = 1;
    int: C_c1_root_end = 2;
    array [int] of record(set of B_potential: b1, set of B_potential: b2): C_c1_objects = [(b1: b1, b2: b2) | p in index_set(C_c1_inputs), input = '[]'(C_c1_inputs, p), b1 = B_occ_1('..'('+'(1, sum([length(('[]'(C_c1_inputs, q)).b1) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(C_c1_inputs, q)).b1) | q in '..'(1, '-'(p, 1))]), length((input).b1)))), b2 = B_occ_4('..'('+'(1, sum([length(('[]'(C_c1_inputs, q)).b2) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(C_c1_inputs, q)).b2) | q in '..'(1, '-'(p, 1))]), length((input).b2))))];
    set of C_occ_0({1}): c1 = C_occ_0({1});
    array [int] of record(array [int] of record(array [int] of record(int: x): a1, array [int] of record(int: x): a2): b1, array [int] of record(array [int] of record(int: x): a1, array [int] of record(int: x): a2): b2): C_c2_inputs = [(b1: [(a1: [], a2: [(x: 6)])], b2: [(a1: [(x: 7)], a2: [(x: 8)]), (a1: [(x: 9)], a2: [(x: 10), (x: 11)])])];
    array [int] of record(set of A_potential: a1, set of A_potential: a2): B_b1_occ_2_objects = [(a1: a1, a2: a2) | p in index_set(C_c2_inputs), r in index_set(('[]'(C_c2_inputs, p)).b1), input = '[]'(('[]'(C_c2_inputs, p)).b1, r), a1 = A_occ_9('..'('+'(1, '+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b1, s)).a1) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b1, s)).a1) | s in '..'(1, '-'(r, 1))])), length((input).a1)))), a2 = A_occ_10('..'('+'(1, '+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b1, s)).a2) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b1]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b1, s)).a2) | s in '..'(1, '-'(r, 1))])), length((input).a2))))];
    array [int] of record(set of A_potential: a1, set of A_potential: a2): B_b2_occ_3_objects = [(a1: a1, a2: a2) | p in index_set(C_c2_inputs), r in index_set(('[]'(C_c2_inputs, p)).b2), input = '[]'(('[]'(C_c2_inputs, p)).b2, r), a1 = A_occ_12('..'('+'(1, '+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b2, s)).a1) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a1) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b2, s)).a1) | s in '..'(1, '-'(r, 1))])), length((input).a1)))), a2 = A_occ_13('..'('+'(1, '+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b2, s)).a2) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).a2) | q in '..'(1, '-'(p, 1)), j in ('[]'(C_c2_inputs, q)).b2]), sum([length(('[]'(('[]'(C_c2_inputs, p)).b2, s)).a2) | s in '..'(1, '-'(r, 1))])), length((input).a2))))];
    array [int] of record(int: x): A_b1_a1_occ_4_objects = [k | i in C_c2_inputs, j1 in (i).b1, k in (j1).a1];
    array [int] of record(int: x): A_b1_a2_occ_5_objects = [k | i in C_c2_inputs, j1 in (i).b1, k in (j1).a2];
    array [int] of record(int: x): A_b2_a1_occ_6_objects = [k | i in C_c2_inputs, j1 in (i).b2, k in (j1).a1];
    array [int] of record(int: x): A_b2_a2_occ_7_objects = [k | i in C_c2_inputs, j1 in (i).b2, k in (j1).a2];
    int: C_c2_b1_a1_start = C_c1_b2_a2_end;
    int: C_c2_b1_a1_end = '+'(C_c2_b1_a1_start, sum([length((j1).a1) | i in C_c2_inputs, j1 in (i).b1]));
    int: C_c2_b1_a2_start = C_c2_b1_a1_end;
    int: C_c2_b1_a2_end = '+'(C_c2_b1_a2_start, sum([length((j1).a2) | i in C_c2_inputs, j1 in (i).b1]));
    int: C_c2_b2_a1_start = C_c2_b1_a2_end;
    int: C_c2_b2_a1_end = '+'(C_c2_b2_a1_start, sum([length((j1).a1) | i in C_c2_inputs, j1 in (i).b2]));
    int: C_c2_b2_a2_start = C_c2_b2_a1_end;
    int: C_c2_b2_a2_end = '+'(C_c2_b2_a2_start, sum([length((j1).a2) | i in C_c2_inputs, j1 in (i).b2]));
    int: C_c2_b1_start = C_c1_b2_end;
    int: C_c2_b1_end = '+'(C_c2_b1_start, sum([length((i).b1) | i in C_c2_inputs]));
    int: C_c2_b2_start = C_c2_b1_end;
    int: C_c2_b2_end = '+'(C_c2_b2_start, sum([length((i).b2) | i in C_c2_inputs]));
    int: C_c2_root_start = C_c1_root_end;
    int: C_c2_root_end = '+'(C_c2_root_start, 1);
    array [int] of record(set of B_potential: b1, set of B_potential: b2): C_c2_objects = [(b1: b1, b2: b2) | p in index_set(C_c2_inputs), input = '[]'(C_c2_inputs, p), b1 = B_occ_8('..'('+'(1, sum([length(('[]'(C_c2_inputs, q)).b1) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(C_c2_inputs, q)).b1) | q in '..'(1, '-'(p, 1))]), length((input).b1)))), b2 = B_occ_11('..'('+'(1, sum([length(('[]'(C_c2_inputs, q)).b2) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(C_c2_inputs, q)).b2) | q in '..'(1, '-'(p, 1))]), length((input).b2))))];
    set of C_occ_7('..'(C_c1_root_end, '-'(C_c2_root_end, 1))): c2 = C_occ_7('..'(C_c1_root_end, '-'(C_c2_root_end, 1)));
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_repeated_nested_par_set_new_snapshot() {
	check_thir_model_with_stdlib(
		REPEATED_NESTED_PAR_SET_NEW,
		expect!([r#"
    enum Leaf_potential = Leaf_occ_1('..'(1, sum([length((i).left) | i in Node_roots_inputs]))) ++ Leaf_occ_2('..'(1, sum([length((i).right) | i in Node_roots_inputs])));
    array [int] of record(int: value): Leaf_objects = '++'(Leaf_left_objects, Leaf_right_objects);
    set of Leaf_potential: Leaf = array_union('++'([Leaf_occ_1('..'(1, '-'(Node_roots_left_end, 1)))], [Leaf_occ_2('..'(1, '-'(Node_roots_right_end, Node_roots_left_end)))]));
    enum Node_potential = Node_occ_0({1, 2});
    array [int] of record(set of Leaf_potential: left, set of Leaf_potential: right): Node_objects = Node_roots_objects;
    set of Node_potential: Node = roots;
    array [int] of record(array [int] of record(int: value): left, array [int] of record(int: value): right): Node_roots_inputs = [(left: [(value: 1), (value: 2)], right: [(value: 3)]), (left: [(value: 4)], right: [(value: 5), (value: 6)])];
    array [int] of record(int: value): Leaf_left_objects = [k | i in Node_roots_inputs, k in (i).left];
    array [int] of record(int: value): Leaf_right_objects = [k | i in Node_roots_inputs, k in (i).right];
    int: Node_roots_left_start = 1;
    int: Node_roots_left_end = '+'(Node_roots_left_start, sum([length((i).left) | i in Node_roots_inputs]));
    int: Node_roots_right_start = Node_roots_left_end;
    int: Node_roots_right_end = '+'(Node_roots_right_start, sum([length((i).right) | i in Node_roots_inputs]));
    int: Node_roots_root_start = 1;
    int: Node_roots_root_end = 3;
    array [int] of record(set of Leaf_potential: left, set of Leaf_potential: right): Node_roots_objects = [(left: left, right: right) | p in index_set(Node_roots_inputs), input = '[]'(Node_roots_inputs, p), left = Leaf_occ_1('..'('+'(1, sum([length(('[]'(Node_roots_inputs, q)).left) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Node_roots_inputs, q)).left) | q in '..'(1, '-'(p, 1))]), length((input).left)))), right = Leaf_occ_2('..'('+'(1, sum([length(('[]'(Node_roots_inputs, q)).right) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Node_roots_inputs, q)).right) | q in '..'(1, '-'(p, 1))]), length((input).right))))];
    set of Node_occ_0({1, 2}): roots = Node_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_inherited_par_set_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_PAR_SET_NEW,
		expect!([r#"
    enum B_potential = B_occ_2('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(int: x, int: y): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_2('..'(1, '-'(Root_roots_bs_end, 1)))]);
    enum A_potential = A_occ_1('..'(1, sum([length((i).as) | i in Root_roots_inputs]))) ++ A_occ_2('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(int: x): A_objects = '++'(A_as_objects, A_bs_objects);
    set of A_potential: A = array_union('++'([A_occ_1('..'(1, '-'(Root_roots_as_end, 1)))], [{A_occ_2(_DECL_1)} | _DECL_1 in index_set(B_bs_objects)]));
    enum Root_potential = Root_occ_0({1, 2});
    array [int] of record(set of A_potential: as, set of B_potential: bs): Root_objects = Root_roots_objects;
    set of Root_potential: Root = roots;
    array [int] of record(array [int] of record(int: x): as, array [int] of record(int: x, int: y): bs): Root_roots_inputs = [(as: [(x: 1)], bs: [(x: 2, y: 3), (x: 4, y: 5)]), (as: [(x: 6), (x: 7)], bs: [(x: 8, y: 9)])];
    array [int] of record(int: x): A_as_objects = [k | i in Root_roots_inputs, k in (i).as];
    array [int] of record(int: x, int: y): B_bs_objects = [k | i in Root_roots_inputs, k in (i).bs];
    array [int] of record(int: x): A_bs_objects = [(x: (k).x) | i in Root_roots_inputs, k in (i).bs];
    int: Root_roots_as_start = 1;
    int: Root_roots_as_end = '+'(Root_roots_as_start, sum([length((i).as) | i in Root_roots_inputs]));
    int: Root_roots_bs_A_start = Root_roots_as_end;
    int: Root_roots_bs_A_end = '+'(Root_roots_bs_A_start, 1);
    int: Root_roots_bs_start = 1;
    int: Root_roots_bs_end = '+'(Root_roots_bs_start, sum([length((i).bs) | i in Root_roots_inputs]));
    int: Root_roots_root_start = 1;
    int: Root_roots_root_end = 3;
    array [int] of record(set of A_potential: as, set of B_potential: bs): Root_roots_objects = [(as: as, bs: bs) | p in index_set(Root_roots_inputs), input = '[]'(Root_roots_inputs, p), as = A_occ_1('..'('+'(1, sum([length(('[]'(Root_roots_inputs, q)).as) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Root_roots_inputs, q)).as) | q in '..'(1, '-'(p, 1))]), length((input).as)))), bs = B_occ_2('..'('+'(1, sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of Root_occ_0({1, 2}): roots = Root_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_inherited_par_set_new_mixed_scalar_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_PAR_SET_NEW_MIXED_SCALAR,
		expect!([r#"
    enum B_potential = B_occ_2('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(int: x, int: y): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_2('..'(1, '-'(Root_roots_bs_end, 1)))]);
    enum A_potential = A_occ_1('..'(1, sum([length((i).as) | i in Root_roots_inputs]))) ++ A_occ_2('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(int: x): A_objects = '++'(A_as_objects, A_bs_objects);
    set of A_potential: A = array_union('++'([A_occ_1('..'(1, '-'(Root_roots_as_end, 1)))], [{A_occ_2(_DECL_1)} | _DECL_1 in index_set(B_bs_objects)]));
    enum Root_potential = Root_occ_0({1, 2});
    array [int] of record(set of A_potential: as, set of B_potential: bs, var '..'(0, 2): z): Root_objects = Root_roots_objects;
    set of Root_potential: Root = roots;
    array [int] of record(array [int] of record(int: x): as, array [int] of record(int: x, int: y): bs): Root_roots_inputs = [(as: [(x: 1)], bs: [(x: 2, y: 3), (x: 4, y: 5)]), (as: [(x: 6), (x: 7)], bs: [(x: 8, y: 9)])];
    array [int] of record(int: x): A_as_objects = [k | i in Root_roots_inputs, k in (i).as];
    array [int] of record(int: x, int: y): B_bs_objects = [k | i in Root_roots_inputs, k in (i).bs];
    array [int] of record(int: x): A_bs_objects = [(x: (k).x) | i in Root_roots_inputs, k in (i).bs];
    int: Root_roots_as_start = 1;
    int: Root_roots_as_end = '+'(Root_roots_as_start, sum([length((i).as) | i in Root_roots_inputs]));
    int: Root_roots_bs_A_start = Root_roots_as_end;
    int: Root_roots_bs_A_end = '+'(Root_roots_bs_A_start, 1);
    int: Root_roots_bs_start = 1;
    int: Root_roots_bs_end = '+'(Root_roots_bs_start, sum([length((i).bs) | i in Root_roots_inputs]));
    int: Root_roots_root_start = 1;
    int: Root_roots_root_end = 3;
    array [int] of record(set of A_potential: as, set of B_potential: bs, var '..'(0, 2): z): Root_roots_objects = [(as: as, bs: bs, z: z) | p in index_set(Root_roots_inputs), input = '[]'(Root_roots_inputs, p), z = let {
      var '..'(0, 2): z_init;
    } in z_init, as = A_occ_1('..'('+'(1, sum([length(('[]'(Root_roots_inputs, q)).as) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Root_roots_inputs, q)).as) | q in '..'(1, '-'(p, 1))]), length((input).as)))), bs = B_occ_2('..'('+'(1, sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of Root_occ_0({1, 2}): roots = Root_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_inherited_child_par_set_new_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_INHERITED_CHILD_PAR_SET_NEW,
		expect!([r#"
    enum Leaf_potential = Leaf_occ_2('..'(1, sum([length((j1).leaves) | i in Root_roots_inputs, j1 in (i).bs])));
    array [int] of record(int: value): Leaf_objects = Leaf_bs_leaves_objects;
    set of Leaf_potential: Leaf = array_union([Leaf_occ_2('..'(1, '-'(Root_roots_bs_leaves_end, 1)))]);
    enum B_potential = B_occ_1('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(set of Leaf_potential: leaves, int: x, int: y): B_objects = B_bs_objects;
    set of B_potential: B = array_union([B_occ_1('..'(1, '-'(Root_roots_bs_end, 1)))]);
    enum A_potential = A_occ_1('..'(1, sum([length((i).bs) | i in Root_roots_inputs])));
    array [int] of record(int: x): A_objects = A_bs_objects;
    set of A_potential: A = array_union([{A_occ_1(_DECL_1)} | _DECL_1 in index_set(B_bs_objects)]);
    enum Root_potential = Root_occ_0({1, 2});
    array [int] of record(set of B_potential: bs): Root_objects = Root_roots_objects;
    set of Root_potential: Root = roots;
    array [int] of record(array [int] of record(array [int] of record(int: value): leaves, int: x, int: y): bs): Root_roots_inputs = [(bs: [(x: 1, y: 2, leaves: [(value: 3), (value: 4)])]), (bs: [(x: 5, y: 6, leaves: [(value: 7)])])];
    array [int] of record(set of Leaf_potential: leaves, int: x, int: y): B_bs_objects = [(leaves: leaves, x: x, y: y) | p in index_set(Root_roots_inputs), r in index_set(('[]'(Root_roots_inputs, p)).bs), input = '[]'(('[]'(Root_roots_inputs, p)).bs, r), x = (input).x, y = (input).y, leaves = Leaf_occ_2('..'('+'(1, '+'(sum([length((j).leaves) | q in '..'(1, '-'(p, 1)), j in ('[]'(Root_roots_inputs, q)).bs]), sum([length(('[]'(('[]'(Root_roots_inputs, p)).bs, s)).leaves) | s in '..'(1, '-'(r, 1))]))), '+'('+'(sum([length((j).leaves) | q in '..'(1, '-'(p, 1)), j in ('[]'(Root_roots_inputs, q)).bs]), sum([length(('[]'(('[]'(Root_roots_inputs, p)).bs, s)).leaves) | s in '..'(1, '-'(r, 1))])), length((input).leaves))))];
    array [int] of record(int: x): A_bs_objects = [(x: (k).x) | i in Root_roots_inputs, k in (i).bs];
    array [int] of record(int: value): Leaf_bs_leaves_objects = [k | i in Root_roots_inputs, j1 in (i).bs, k in (j1).leaves];
    int: Root_roots_bs_A_start = 1;
    int: Root_roots_bs_A_end = 2;
    int: Root_roots_bs_start = 1;
    int: Root_roots_bs_end = '+'(Root_roots_bs_start, sum([length((i).bs) | i in Root_roots_inputs]));
    int: Root_roots_bs_leaves_start = 1;
    int: Root_roots_bs_leaves_end = '+'(Root_roots_bs_leaves_start, sum([length((j1).leaves) | i in Root_roots_inputs, j1 in (i).bs]));
    int: Root_roots_root_start = 1;
    int: Root_roots_root_end = 3;
    array [int] of record(set of B_potential: bs): Root_roots_objects = [(bs: bs) | p in index_set(Root_roots_inputs), input = '[]'(Root_roots_inputs, p), bs = B_occ_1('..'('+'(1, sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))])), '+'(sum([length(('[]'(Root_roots_inputs, q)).bs) | q in '..'(1, '-'(p, 1))]), length((input).bs))))];
    set of Root_occ_0({1, 2}): roots = Root_occ_0({1, 2});
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_nested_field_access_snapshot() {
	check_thir_model_with_stdlib(
		NESTED_PAR_FIELD_ACCESS,
		expect!([r#"
    enum B_potential = B_occ_1('..'(1, '*'(card(A_potential), 1)));
    array [int] of record(int: y): B_objects = B_b_objects;
    set of B_potential: B = array_union([{B_occ_1(_DECL_1)} | _DECL_1 in index_set(A_a_objects)]);
    enum A_potential = A_occ_0({1});
    array [int] of record(B_potential: b, int: x): A_objects = A_a_objects;
    set of A_potential: A = A_occ_0({1});
    array [int] of record(record(int: y): b, int: x): A_a_inputs = [(b: (y: 1), x: 2)];
    array [int] of record(int: y): B_b_objects = [(i).b | i in A_a_inputs];
    int: A_a_b_start = 1;
    int: A_a_b_end = '+'(A_a_b_start, '*'(card(A_potential), 1));
    array [int] of record(B_potential: b, int: x): A_a_objects = [(b: b, x: x) | p in index_set(A_a_inputs), input = '[]'(A_a_inputs, p), b = B_occ_1(p), x = (input).x];
    A_potential: a = A_occ_0(1);
    constraint '>='(('[]'(B_objects, enum2int(('[]'(A_objects, enum2int(a))).b))).y, 0);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_inherited_field_access_snapshot() {
	check_thir_model_with_stdlib(
		INHERITED_PAR_FIELD_ACCESS,
		expect!([r#"
    enum B_potential = B_occ_0({1});
    array [int] of record(int: x, int: y): B_objects = B_b_objects;
    set of B_potential: B = B_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(int: x): A_objects = A_B_b_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_1)} | _DECL_1 in index_set(B_b_objects)]);
    array [int] of record(int: x, int: y): B_b_inputs = [(x: 1, y: 2)];
    int: B_b__start = 1;
    int: B_b__end = 2;
    array [int] of record(int: x, int: y): B_b_objects = B_b_inputs;
    array [int] of record(int: x): A_B_b_objects = [(x: ('[]'(B_b_objects, p)).x) | p in index_set(B_b_objects)];
    B_potential: b = B_occ_0(1);
    constraint '<='(('[]'(B_objects, enum2int(b))).x, ('[]'(B_objects, enum2int(b))).y);
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_superclass_field_access_snapshot() {
	check_thir_model_with_stdlib(
		SUPERCLASS_PAR_FIELD_ACCESS,
		expect!([r#"
    enum B_potential = B_occ_0({1});
    array [int] of record(int: x, int: y): B_objects = B_b_objects;
    set of B_potential: B = B_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(int: x): A_objects = A_B_b_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_1)} | _DECL_1 in index_set(B_b_objects)]);
    array [int] of record(int: x, int: y): B_b_inputs = [(x: 1, y: 2)];
    int: B_b__start = 1;
    int: B_b__end = 2;
    array [int] of record(int: x, int: y): B_b_objects = B_b_inputs;
    array [int] of record(int: x): A_B_b_objects = [(x: ('[]'(B_b_objects, p)).x) | p in index_set(B_b_objects)];
    B_potential: b = B_occ_0(1);
    A: a = A_occ_0(enum2int(b));
    constraint '<='(('[]'(A_objects, enum2int(a))).x, 2);
    solve satisfy;
"#]),
	);
}

// Reference cycle between two var-reached classes (Seat references Handrail,
// Handrail's reference-set field contains Seats). No topological predeclare
// order exists for a cycle, so `<C>_objects` storage domains must be repaired
// after every class is registered
// (`repair_predeclared_class_objects_domains`) — this snapshot pins the
// substituted `Seat_potential` / `Handrail_potential` storage fields.
// Regression test for a reference-cycle panic ("Type by construction (var set of
// Seat) disagrees with typechecker (set of Seat)").
#[test]
fn object_thir_lowering_reference_cycle_deopt_set_field_snapshot() {
	check_thir_model_with_stdlib(
		REFERENCE_CYCLE_DEOPT_SET_FIELD,
		expect!([r#"
    enum Handrail_potential = Handrail_occ_2('..'(1, '*'(card(Wagon_potential), 1)));
    array [int] of record(var set of Seat_potential: attached): Handrail_objects = array1d(Handrail_handrail_objects);
    var set of Handrail_potential: Handrail = array_union([if occurs(('[]'(Wagon_w_objects, _DECL_1)).handrail) then {Handrail_occ_2(_DECL_1)} else {} endif | _DECL_1 in index_set(Wagon_w_objects)]);
    constraint forall(['in'(card(('[]'(Handrail_objects, enum2int(this))).attached), '..'(1, 1)) | this in Handrail]);
    enum Seat_potential = Seat_occ_1('..'(1, '*'(card(Wagon_potential), max('..'(1, 1)))));
    array [int] of record(var '..'(1, 2): comfort, var opt Handrail_potential: handrail): Seat_objects = array1d(Seat_seats_objects);
    var set of Seat_potential: Seat = array_union([('[]'(Wagon_w_objects, _DECL_2)).seats | _DECL_2 in index_set(Wagon_w_objects)]);
    enum Wagon_potential = Wagon_occ_0({1});
    array [int] of record(var opt Handrail_potential: handrail, var set of Seat_potential: seats): Wagon_objects = Wagon_w_objects;
    set of Wagon_potential: Wagon = Wagon_occ_0({1});
    constraint forall(['in'(card(('[]'(Wagon_objects, enum2int(this))).seats), '..'(1, 1)) | this in Wagon]);
    constraint forall([occurs(('[]'(Wagon_objects, enum2int(w))).handrail) | w in Wagon]);
    constraint forall([forall(['/\'('in'(s, ('[]'(Wagon_objects, enum2int(w))).seats), '='(('[]'(Seat_objects, enum2int(s))).comfort, 2)) | s in ('[]'(Handrail_objects, enum2int(deopt(('[]'(Wagon_objects, enum2int(w))).handrail)))).attached]) | w in Wagon where occurs(('[]'(Wagon_objects, enum2int(w))).handrail)]);
    array [{1}] of record(var opt Handrail_potential: handrail, var set of Seat_potential: seats): Wagon_w_storage;
    array [Seat_occ_1('..'(1, '*'(card(Wagon_potential), max('..'(1, 1)))))] of record(var '..'(1, 2): comfort, var opt Handrail_potential: handrail): Seat_seats_objects;
    array [Handrail_occ_2('..'(1, '*'(card(Wagon_potential), 1)))] of record(var set of Seat_potential: attached): Handrail_handrail_objects;
    int: Wagon_w_handrail_start = 1;
    int: Wagon_w_handrail_end = '+'(Wagon_w_handrail_start, '*'(card(Wagon_potential), 1));
    int: Wagon_w_seats_start = 1;
    int: Wagon_w_seats_end = '+'(Wagon_w_seats_start, '*'(card(Wagon_potential), max('..'(1, 1))));
    array [int] of set of Seat_potential: Wagon_w_seats_potential = [_DECL_3: Seat_occ_1('..'('+'(1, '*'('-'(_DECL_3, 1), max('..'(1, 1)))), '*'(_DECL_3, max('..'(1, 1))))) | _DECL_3 in index_set(Wagon_w_storage)];
    array [int] of record(var opt Handrail_potential: handrail, var set of Seat_potential: seats): Wagon_w_objects = [(handrail: handrail, seats: seats) | p in index_set(Wagon_w_storage), input = '[]'(Wagon_w_storage, p), seats = (input).seats, handrail = (input).handrail];
    Wagon_potential: w = Wagon_occ_0(1);
    constraint forall(['subset'(enum2int(('[]'(Wagon_w_objects, _DECL_4)).seats), enum2int('[]'(Wagon_w_seats_potential, _DECL_4))) | _DECL_4 in index_set(Wagon_w_objects)]);
    output ["comforts=", show([('[]'(Seat_objects, enum2int(s))).comfort | s in ('[]'(Wagon_objects, enum2int(w))).seats])];
    constraint forall(['->'(occurs(('[]'(Wagon_w_objects, _DECL_5)).handrail), '='(('[]'(Wagon_w_objects, _DECL_5)).handrail, Handrail_occ_2(_DECL_5))) | _DECL_5 in index_set(Wagon_w_objects)]);
    constraint forall(['\/'('in'(x, Seat), '='(('[]'(Seat_objects, enum2int(x))).comfort, mzn_safe_default(('[]'(Seat_objects, enum2int(x))).comfort))) | x in Seat_potential]);
    constraint forall(['\/'('in'(x, Seat), '='(('[]'(Seat_objects, enum2int(x))).handrail, <>)) | x in Seat_potential]);
    constraint forall(['\/'('in'(x, Handrail), '='(('[]'(Handrail_objects, enum2int(x))).attached, lb(('[]'(Handrail_objects, enum2int(x))).attached))) | x in Handrail_potential]);
    solve satisfy;
"#]),
	);
}

// Array-typed attribute read back through a var index (par class): pins the
// per-position fast path (`decompose_array_field_var_access`) — one scalar
// arrayXd column per position j, reassembled with arrayXd over the
// representative element's index set.
#[test]
fn object_thir_lowering_array_field_var_index_par_snapshot() {
	check_thir_model_with_stdlib(
		ARRAY_FIELD_VAR_INDEX_PAR,
		expect!([r#"
    enum A_potential = A_occ_0({1, 2});
    array [int] of record('..'(1, 2): id, array ['..'(1, 3)] of var '..'(0, 2): xs): A_objects = A_as_objects;
    set of A_potential: A = as;
    array [int] of record(int: id): A_as_inputs = [(id: 1), (id: 2)];
    int: A_as_root_start = 1;
    int: A_as_root_end = 3;
    array [int] of record('..'(1, 2): id, array ['..'(1, 3)] of var '..'(0, 2): xs): A_as_objects = [(id: id, xs: xs) | p in index_set(A_as_inputs), input = '[]'(A_as_inputs, p), id = (input).id, xs = let {
      array ['..'(1, 3)] of var '..'(0, 2): xs_init;
    } in xs_init];
    set of A_occ_0({1, 2}): as = A_occ_0({1, 2});
    var A: chosen;
    var '..'(1, 2): cid;
    constraint '='(sum([sum(('[]'(A_objects, enum2int(a))).xs) | a in as]), 1);
    constraint forall(['->'('='(('[]'(A_objects, enum2int(a))).id, 2), '='(sum(('[]'(A_objects, enum2int(a))).xs), 0)) | a in as]);
    constraint '='(sum(('[]'(A_objects, enum2int(chosen))).xs), 1);
    constraint '='(cid, ('[]'(A_objects, enum2int(chosen))).id);
    output ["cid=", show(cid), " sums=", show([sum(('[]'(A_objects, enum2int(a))).xs) | a in as])];
    solve satisfy;
"#]),
	);
}

#[test]
fn object_thir_lowering_superclass_var_field_access_snapshot() {
	check_thir_model_with_stdlib(
		SUPERCLASS_VAR_FIELD_ACCESS,
		expect!([r#"
    enum B_potential = B_occ_0({1});
    array [int] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_objects = B_b_objects;
    set of B_potential: B = B_occ_0({1});
    enum A_potential = A_occ_0({1});
    array [int] of record(var '..'(0, 2): x): A_objects = A_B_b_objects;
    set of A_potential: A = array_union([{A_occ_0(_DECL_1)} | _DECL_1 in index_set(B_b_objects)]);
    array [{1}] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_b_storage;
    int: B_b__start = 1;
    int: B_b__end = 2;
    array [int] of record(var '..'(0, 2): x, var '..'(0, 2): y): B_b_objects = B_b_storage;
    array [int] of record(var '..'(0, 2): x): A_B_b_objects = [(x: ('[]'(B_b_objects, p)).x) | p in index_set(B_b_objects)];
    B_potential: b = B_occ_0(1);
    var A_potential: a = A_occ_0(enum2int(b));
    constraint '<='(('[]'(A_objects, enum2int(a))).x, 2);
    solve satisfy;
"#]),
	);
}

/// Parse, type-check and THIR-lower `source` through the full transform
/// pipeline. Returns `Ok(())` on a clean run, or `Err(<diagnostic text>)`
/// for HIR diagnostics, a THIR transform error, or a panic caught via
/// `catch_unwind`.
fn lowering_gate(source: &str) -> Result<(), String> {
	let db = db_for_with_stdlib(source);
	let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
		let errors = user_hir_errors(&db);
		if !errors.is_empty() {
			return Err(errors
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>()
				.join("\n"));
		}
		match thir_transforms()(&db, lower_model(&db).take()) {
			Ok(_) => Ok(()),
			Err(err) => Err(format!("THIR transform error: {err}")),
		}
	}));
	match outcome {
		Ok(inner) => inner,
		Err(panic) => {
			let msg = panic
				.downcast_ref::<String>()
				.cloned()
				.or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
				.unwrap_or_else(|| "<non-string panic payload>".to_owned());
			Err(format!("THIR lowering panicked: {msg}"))
		}
	}
}

/// Cheap, solver-free correctness gate for externally produced object
/// models: does the model parse, type-check, and run the full THIR
/// pipeline without a diagnostic or a panic? Catching that class of
/// mistake without standing up MiniZinc keeps a convert-and-check loop
/// fast.
///
/// Driven by an env var (mirroring how the MiniZinc harnesses gate on
/// `SHACKLE_MINIZINC`) because it needs the `cfg(test)` crate-internal
/// harness helpers:
///
/// ```text
/// SHACKLE_GATE_MODEL=/abs/path/to/object.mzn \
///   cargo test -p shackle-thir object_lowering_gate -- --nocapture
/// ```
///
/// `SHACKLE_GATE_MODEL` may be a comma-separated list; every entry is
/// gated and a per-file `OK` / `FAIL` line is printed. The test fails
/// (panics) iff at least one entry failed, with the collected
/// diagnostics in the panic message. When the env var is unset the
/// test is a clean no-op, so an ordinary `cargo test` run stays green.
#[test]
fn object_lowering_gate() {
	let Ok(raw) = std::env::var("SHACKLE_GATE_MODEL") else {
		eprintln!(
			"object_lowering_gate: SHACKLE_GATE_MODEL not set — nothing to gate (no-op).\n  \
			 Usage: SHACKLE_GATE_MODEL=/abs/path/to/object.mzn \
			 cargo test -p shackle-thir object_lowering_gate -- --nocapture"
		);
		return;
	};
	let paths: Vec<&str> = raw
		.split(',')
		.map(str::trim)
		.filter(|p| !p.is_empty())
		.collect();
	assert!(!paths.is_empty(), "SHACKLE_GATE_MODEL is set but empty");

	let mut failures = Vec::new();
	for path in paths {
		let source = match std::fs::read_to_string(path) {
			Ok(s) => s,
			Err(err) => {
				eprintln!("FAIL  {path}  (could not read file: {err})");
				failures.push(format!("{path}: could not read file: {err}"));
				continue;
			}
		};
		match lowering_gate(&source) {
			Ok(()) => eprintln!("OK    {path}"),
			Err(diag) => {
				eprintln!("FAIL  {path}\n{diag}\n");
				failures.push(format!("--- {path} ---\n{diag}"));
			}
		}
	}

	assert!(
		failures.is_empty(),
		"object lowering gate failed for {} model(s):\n\n{}",
		failures.len(),
		failures.join("\n\n")
	);
}
