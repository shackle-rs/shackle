//! Pretty-printed THIR snapshots of the object-syntax lowering: each test
//! pins the shape `lower_model` emits for an object model, with the standard
//! library available so builtins resolve.
//!
//! Class-analysis snapshots live in `shackle_hir::class_analysis`, and the
//! HIR-level object diagnostics (unsupported shapes, type errors, and the
//! not-fenced counterparts) live in `shackle_hir::object_validation` and
//! `shackle_hir::typecheck` — this file no longer covers them.

use std::{fs, panic::AssertUnwindSafe, path::PathBuf};

use expect_test::expect_file;
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

/// Absolute path to the object-test fixtures directory, where each test's
/// `<name>.mzn` source and its `<name>.thir` expected snapshot live side by
/// side.
fn fixtures_dir() -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/objects/fixtures")
}

/// Lower `fixtures/<name>.mzn` with the standard library available, pretty
/// print the user model's own top-level items (see [`user_items_pretty`]), and
/// compare against `fixtures/<name>.thir`. Regenerate the expected file with
/// `UPDATE_EXPECT=1`.
fn check_snapshot(name: &str) {
	let dir = fixtures_dir();
	let source = fs::read_to_string(dir.join(format!("{name}.mzn")))
		.unwrap_or_else(|err| panic!("failed to read fixture {name}.mzn: {err}"));
	expect_file![dir.join(format!("{name}.thir"))].assert_eq(&user_items_pretty(&source));
}

/// Like [`check_snapshot`] but lowers with the standard library disabled and
/// pretty prints the whole lowered THIR (no user-item filtering). Pins the raw
/// shape the object lowering emits for the few tests that need no stdlib items.
fn check_snapshot_ignore_stdlib(name: &str) {
	let dir = fixtures_dir();
	let source = fs::read_to_string(dir.join(format!("{name}.mzn")))
		.unwrap_or_else(|err| panic!("failed to read fixture {name}.mzn: {err}"));
	let db = db_for(&source);
	let model = lower_model(&db);
	let pretty = PrettyPrinter::new(&db, model.get().as_ref()).pretty_print();
	expect_file![dir.join(format!("{name}.thir"))].assert_eq(&pretty);
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
	check_snapshot("par_computed_attribute");
}

// A chain of computed attributes (`z = y + 4` depends on `y = x + 1`): the
// reconstruction comprehension emits the aliases in declaration order so each
// computed attribute can reference the previous one.
#[test]
fn object_par_class_computed_attribute_chain_compiles() {
	check_snapshot("par_computed_chain");
}

// A var attribute whose declared domain depends on a computed attribute
// (`var 1..z: s`): the storage record element type carries `var int: s` and the
// per-object bound is minted in the reconstruction via `let { var 1..z: .. }`.
#[test]
fn object_par_class_computed_var_attr_compiles() {
	check_snapshot("par_computed_var_attr");
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
	check_snapshot("par_new_object_computed");
}

// The par SET-root sibling of the test above: `set(1..2) of new A` where each
// input record supplies a different number of children. `children` is minted
// per parent from the prefix-sum ordinal range and `n = card(children)` is
// alias-defined — the lowered model must carry no valueless `n_init`.
#[test]
fn object_par_set_new_object_computed_compiles() {
	check_snapshot("par_set_new_object_computed");
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
	check_snapshot("par_nested_computed_attr");
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
	check_snapshot("var_set_new_nested_computed_attr");
}

// Domain relocation on a NESTED slot: the binding-domain
// binding-domain + total-RHS shape on a var-reached nested class takes the
// relocation encoding — the element record's `z` relaxed to unbounded, the
// declared 3..4 re-imposed as the realised-set invariant
// `forall(this in B)(this.z in 3..4)`, and the alias unguarded.
#[test]
fn object_var_set_new_nested_computed_bounded_domain_compiles() {
	check_snapshot("var_set_new_nested_computed_bounded_domain");
}

// 2.5(a) elision on a NESTED slot: `y = 6 div (x + 1)` is not on the
// totality whitelist, so the nested engine contribution KEEPS the value
// guard — witness decl, pin, `realised = p in B` alias, if-then-else —
// exactly like the root shape.
#[test]
fn object_var_set_new_nested_computed_div_keeps_guard_compiles() {
	check_snapshot("var_set_new_nested_computed_div_keeps_guard");
}

// A singular `var new` root of a subclass whose SUPERCLASS
// declares a computed attribute. The superclass projection
// (`S_T_t_objects`) now reads every field — including the alias-defined
// `c` — from the direct-class objects array instead of fresh-minting from
// the raw inputs, inherits its determined flag, and S's class-body forall
// is dropped.
#[test]
fn object_var_new_inherited_computed_attr_compiles() {
	check_snapshot("var_new_inherited_computed_attr");
}

// A var-reached class (`var new C`) with a computed (RHS) attribute
// (`c = b + 1`). The computed attribute should be *defined* by the storage
// reconstruction alias chain (like par), not left as a free `_storage`
// decision pinned by the class-body forall.
#[test]
fn object_var_class_computed_attribute_compiles() {
	check_snapshot("var_computed_attribute");
}

// A var-reached class with a computed *set* attribute (`var set of int:
// z = {x, 2*x}`). A free `var set of int` decision is illegal, so this field
// must be realised as a reconstruction alias — `z` is excluded from the free
// `_storage` array and *defined* in the comprehension.
#[test]
fn object_var_class_computed_set_attr_compiles() {
	check_snapshot("var_computed_set_attr");
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
	check_snapshot("var_set_new_computed_class_attr");
}

// A NON-monotone computed attribute (`y = 5 - x`) on a `var set of new` root.
// The symmetry-break wave must not pin alias-defined fields: the alias gives
// `y = 5` at the pinned `x = 0` on unrealised slots, while the old pin
// demanded `y = mzn_safe_default(y) = 0`, forcing every potential realised and
// making `card(as) = 0` UNSAT. The lowered model must carry a pin for the free
// `x` but NONE for the defined `y`.
#[test]
fn object_var_set_new_computed_nonmonotone_compiles() {
	check_snapshot("var_set_new_computed_nonmonotone");
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
	check_snapshot("var_set_new_computed_bounded_domain");
}

// A computed attribute whose RHS is NOT on the totality whitelist (`div`) on
// a `var set of new` root: the 2.5(a) elision analysis must NOT fire. The
// value guard REMAINS — witness decl + pin, per-slot `realised` alias, and
// the `y = if realised then ... else <witness> endif` form.
#[test]
fn object_var_set_new_computed_div_keeps_guard_compiles() {
	check_snapshot("var_set_new_computed_div_keeps_guard");
}

// Singular `var opt new` root, one field per elision outcome: `s = 5 - x`
// (total, no domain) must lose its guard and witness entirely, while
// `y = 6 div (x + 1)` keeps its guard — and with it the shared `realised`
// alias.
#[test]
fn object_var_opt_new_computed_elide_and_guard_compiles() {
	check_snapshot("var_opt_new_computed_elide_and_guard");
}

// Total RHS with a binding declared domain on the singular `var opt new`
// arm. The domain relocation applies — unguarded alias, `var int: z`
// (relaxed) in the element record, and the realised-set invariant
// `forall(this in A)(this.z in 3..4)` instead of a value guard.
#[test]
fn object_var_opt_new_computed_bounded_domain_compiles() {
	check_snapshot("var_opt_new_computed_bounded_domain");
}

// A `var opt new` root with a nested exact-cardinality `set(2..2) of new B`
// field. The invariant must iterate the REALISED class set (`this in A`), not
// potential storage — the storage-iterating form constrained the possibly-
// unrealised slot (children defaults to `{}`, card 0 not in 2..2) and made
// `absent(a)` UNSAT.
#[test]
fn object_var_opt_new_nested_card_absent_compiles() {
	check_snapshot("var_opt_new_nested_card_absent");
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
	check_snapshot("var_new_computed_class_attr");
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
	check_snapshot("par_root_deep_nested_var_reached_object_1");
	// (2) Var-reached deep class via all-set edges.
	check_snapshot("par_root_deep_nested_var_reached_object_2");
	// (3) Var-reached deep class whose object field is itself var-existence
	// (`var set of new D`), dropped from the deep par input: minted as a free
	// var subset in the deep contribution.
	check_snapshot("par_root_deep_nested_var_reached_object_3");
	// (4) Var-SET-reached (`var set of new C`) deep class: the deep par
	// contribution `++`s with the free var-set contribution.
	check_snapshot("par_root_deep_nested_var_reached_object_4");
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
	check_snapshot("par_root_deep_nested_var_existence_field_1");
	// Depth-1 SET edge: par root A owns `set of new C`; C owns the var set.
	check_snapshot("par_root_deep_nested_var_existence_field_2");
	// Depth-1 SINGULAR edge: par root A owns `new C`; C owns the var set.
	check_snapshot("par_root_deep_nested_var_existence_field_3");
	// Deep (depth-2): A.bs -> B.cs -> C, C owns the var set two hops down.
	check_snapshot("par_root_deep_nested_var_existence_field_4");
	// VAR OPT field on a par owner one hop below the root.
	check_snapshot("par_root_deep_nested_var_existence_field_5");
	// Deep var opt: depth-2 owner with `var opt new D`.
	check_snapshot("par_root_deep_nested_var_existence_field_6");
}

#[test]
fn object_thir_lowering_reference_equality_snapshot() {
	check_snapshot("reference_equality_1");
	check_snapshot("reference_equality_2");
	check_snapshot("reference_equality_3");
}

#[test]
fn object_thir_lowering_reference_ordering_snapshot() {
	check_snapshot("reference_ordering_1");
	check_snapshot("reference_ordering_2");
	check_snapshot("reference_ordering_3");
}

#[test]
fn object_thir_lowering_generic_enumerable_snapshot() {
	check_snapshot("generic_enumerable");
}

#[test]
fn object_thir_lowering_simple_new_snapshot() {
	check_snapshot("simple_new");
}

#[test]
fn object_thir_lowering_optional_new_snapshot() {
	check_snapshot("optional_new");
}

#[test]
fn object_thir_lowering_simple_class_constraint_snapshot() {
	check_snapshot("simple_class_constraint");
}

#[test]
fn object_thir_lowering_inherited_class_constraint_snapshot() {
	check_snapshot("inherited_class_constraint");
}

#[test]
fn object_thir_lowering_simple_class_reference_snapshot() {
	check_snapshot("simple_class_reference");
}

#[test]
fn object_thir_lowering_inheritance_snapshot() {
	check_snapshot("inheritance");
}

#[test]
fn object_thir_lowering_top_level_set_new_snapshot() {
	check_snapshot_ignore_stdlib("top_level_set_new");
}

#[test]
fn object_thir_lowering_top_level_set_new_mixed_scalar_snapshot() {
	check_snapshot("top_level_set_new_mixed_scalar");
}

#[test]
fn object_thir_lowering_empty_record_set_new_snapshot() {
	check_snapshot_ignore_stdlib("empty_record_set_new");
}

#[test]
fn object_thir_lowering_self_class_constraint_reference_snapshot() {
	check_snapshot("self_class_constraint_reference");
}

#[test]
fn object_thir_lowering_bounded_set_new_snapshot() {
	check_snapshot("bounded_set_new");
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
	check_snapshot("set_cardinality_constraint");
}

#[test]
fn object_thir_lowering_bounded_set_new_symmetry_defaults_snapshot() {
	check_snapshot("bounded_set_new_symmetry_defaults");
}

#[test]
fn object_thir_lowering_bounded_two_sets_new_snapshot() {
	check_snapshot("bounded_two_sets_new");
}

#[test]
fn object_thir_lowering_mixed_par_var_set_new_snapshot() {
	check_snapshot("mixed_par_var_set_new");
}

#[test]
fn object_thir_lowering_inherited_bounded_set_new_snapshot() {
	check_snapshot("inherited_bounded_set_new");
}

#[test]
fn object_thir_lowering_inherited_bounded_set_class_constraint_snapshot() {
	check_snapshot("inherited_bounded_set_superclass_alias");
}

#[test]
fn object_thir_lowering_inherited_bounded_set_superclass_set_alias_snapshot() {
	check_snapshot("inherited_bounded_set_superclass_set_alias");
}

#[test]
fn object_thir_lowering_nested_bounded_set_class_constraint_snapshot() {
	check_snapshot("nested_bounded_set_class_constraint");
}

#[test]
fn object_thir_lowering_nested_bounded_set_alias_snapshot() {
	check_snapshot("nested_bounded_set_alias");
}

#[test]
fn object_thir_lowering_nested_bounded_set_field_access_snapshot() {
	check_snapshot("nested_bounded_set_field_access");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_field_access_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root_field_access");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_cardinality_channeling_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root_cardinality_channeling");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_fields_same_class_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root_two_fields_same_class");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_parent_membership_channeling_snapshot()
 {
	check_snapshot("nested_bounded_set_under_bounded_root_parent_membership_channeling");
}

#[test]
fn object_thir_lowering_iterator_field_set_field_access_snapshot() {
	check_snapshot("iterator_field_set_field_access");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_path_parent_exclusivity_snapshot()
{
	check_snapshot("nested_bounded_set_under_bounded_root_two_path_parent_exclusivity");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_alias_two_path_parent_exclusivity_snapshot()
 {
	check_snapshot("nested_bounded_set_under_bounded_root_alias_two_path_parent_exclusivity");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_filtered_alias_ownership_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root_filtered_alias_ownership");
}

#[test]
fn object_thir_lowering_nested_bounded_set_under_bounded_root_two_filtered_aliases_snapshot() {
	check_snapshot("nested_bounded_set_under_bounded_root_two_filtered_aliases");
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_snapshot() {
	check_snapshot("nested_bounded_set_two_roots");
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_shared_alias_snapshot() {
	check_snapshot("nested_bounded_set_two_roots_shared_alias");
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_shared_alias_field_access_snapshot() {
	check_snapshot("nested_bounded_set_two_roots_shared_alias_field_access");
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_composite_consumers_snapshot() {
	check_snapshot("nested_bounded_set_two_roots_composite_consumers");
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_alias_field_access_snapshot()
 {
	check_snapshot("nested_inherited_bounded_set_two_roots_superclass_alias_field_access");
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_class_constraint_snapshot()
 {
	check_snapshot("nested_inherited_bounded_set_two_roots_superclass_class_constraint");
}

#[test]
fn object_thir_lowering_nested_inherited_bounded_set_two_roots_superclass_composite_consumers_snapshot()
 {
	check_snapshot("nested_inherited_bounded_set_two_roots_superclass_composite_consumers");
}

#[test]
fn object_thir_lowering_nested_bounded_set_two_roots_class_constraint_snapshot() {
	check_snapshot("nested_bounded_set_two_roots_class_constraint");
}

#[test]
fn object_thir_lowering_nested_new_snapshot() {
	check_snapshot("nested_new");
}

#[test]
fn object_thir_lowering_nested_var_new_snapshot() {
	check_snapshot("nested_var_new_no_constraint");
}

#[test]
fn object_thir_lowering_nested_var_opt_new_snapshot() {
	check_snapshot("nested_var_opt_new_no_constraint");
}

#[test]
fn object_thir_lowering_inherited_nested_new_snapshot() {
	check_snapshot("inherited_nested_new");
}

#[test]
fn object_thir_lowering_nested_bounded_par_set_new_snapshot() {
	check_snapshot("nested_bounded_par_set_new");
}

#[test]
fn object_thir_lowering_nested_bounded_par_set_under_var_root_field_access_snapshot() {
	check_snapshot("nested_bounded_par_set_under_var_root_field_access");
}

#[test]
fn object_thir_lowering_nested_par_set_new_snapshot() {
	check_snapshot("nested_par_set_new");
}

#[test]
fn object_thir_lowering_nested_par_set_new_mixed_scalar_snapshot() {
	check_snapshot("nested_par_set_new_mixed_scalar");
}

#[test]
fn object_thir_lowering_nested_par_set_two_roots_snapshot() {
	check_snapshot("nested_par_set_two_roots");
}

#[test]
fn object_thir_lowering_deep_nested_par_set_two_roots_snapshot() {
	check_snapshot("deep_nested_par_set_two_roots");
}

#[test]
fn object_thir_lowering_repeated_nested_par_set_new_snapshot() {
	check_snapshot("repeated_nested_par_set_new");
}

#[test]
fn object_thir_lowering_nested_inherited_par_set_new_snapshot() {
	check_snapshot("nested_inherited_par_set_new");
}

#[test]
fn object_thir_lowering_nested_inherited_par_set_new_mixed_scalar_snapshot() {
	check_snapshot("nested_inherited_par_set_new_mixed_scalar");
}

#[test]
fn object_thir_lowering_nested_inherited_child_par_set_new_snapshot() {
	check_snapshot("nested_inherited_child_par_set_new");
}

#[test]
fn object_thir_lowering_nested_field_access_snapshot() {
	check_snapshot("nested_par_field_access");
}

#[test]
fn object_thir_lowering_inherited_field_access_snapshot() {
	check_snapshot("inherited_par_field_access");
}

#[test]
fn object_thir_lowering_superclass_field_access_snapshot() {
	check_snapshot("superclass_par_field_access");
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
	check_snapshot("reference_cycle_deopt_set_field");
}

// Array-typed attribute read back through a var index (par class): pins the
// per-position fast path (`decompose_array_field_var_access`) — one scalar
// arrayXd column per position j, reassembled with arrayXd over the
// representative element's index set.
#[test]
fn object_thir_lowering_array_field_var_index_par_snapshot() {
	check_snapshot("array_field_var_index_par");
}

#[test]
fn object_thir_lowering_superclass_var_field_access_snapshot() {
	check_snapshot("superclass_var_field_access");
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
		let source = match fs::read_to_string(path) {
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
