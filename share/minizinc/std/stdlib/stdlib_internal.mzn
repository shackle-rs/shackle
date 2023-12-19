%-----------------------------------------------------------------------------%
%
% Internal compiler functions
%
% These functions are used internally by the compiler.
%

% domain constraints
test mzn_domain_constraint(string: name, $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(erase_enum(x), erase_enum(s)),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, opt $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(erase_enum(x), erase_enum(s)),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, set of $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(erase_enum(x), erase_enum(s)),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, float: x, set of float: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(x, s),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, opt float: x, set of float: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(x, s),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, set of float: x, set of float: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		var_dom(x, s),
		"the type-inst of '" ++
			name ++
			"' is \(s), but is assigned the value \(x).",
	);

predicate mzn_domain_constraint(string: name, var $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name = var_dom(erase_enum(x), erase_enum(s));
predicate mzn_domain_constraint(string: name, var opt $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name = var_dom(erase_enum(x), erase_enum(s));
predicate mzn_domain_constraint(string: name, var set of $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name = var_dom(erase_enum(x), erase_enum(s));
predicate mzn_domain_constraint(string: name, var float: x, set of float: s)
	:: mzn_inline_call_by_name = var_dom(x, s);
predicate mzn_domain_constraint(string: name, var opt float: x, set of float: s)
	:: mzn_inline_call_by_name = var_dom(x, s);

test var_dom(int: x, set of int: s) = x in s;
predicate var_dom(var int: x, set of int: s) =
	if has_bounds(x) /\ dom(x) subset s then true else x in s endif;

test var_dom(opt int: x, set of int: s) = absent(x) \/ deopt(x) in s;
predicate var_dom(var opt int: x, set of int: s) =
	var_dom(deopt(x), mzn_opt_domain(s)) /\ mzn_opt_channel(x, s);
test var_dom(set of int: x, set of int: s) = x subset s;
predicate var_dom(var set of int: x, set of int: s) =
	if has_ub_set(x) /\ ub(x) subset s then true else set_subset(x, s) endif;

test var_dom(array [int] of int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));
predicate var_dom(array [int] of var int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));
test var_dom(array [int] of opt int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));
predicate var_dom(array [int] of var opt int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));
test var_dom(array [int] of set of int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));
predicate var_dom(array [int] of var set of int: x, set of int: d) =
	forall (x_i in x) (var_dom(x_i, d));

test var_dom(float: x, float: l, float: u) = x >= l /\ x <= u;
predicate var_dom(var float: x, float: l, float: u) =
	if has_bounds(x) /\ lb(x) >= l /\ ub(x) <= u then
		true
	else
		x >= l /\ x <= u
	endif;

test var_dom(float: x, set of float: s) = x in s;
predicate var_dom(var float: x, set of float: s) = x in s;

test var_dom(opt float: x, set of float: s) = absent(x) \/ deopt(x) in s;
predicate var_dom(var opt float: x, set of float: s) =
	var_dom(deopt(x), mzn_opt_domain(s)) /\ mzn_opt_channel(x, s);

test var_dom(set of float: x, set of float: s) = x subset s;

test var_dom(array [int] of float: x, float: l, float: u) =
	forall (x_i in x) (var_dom(x_i, l, u));
predicate var_dom(array [int] of var float: x, float: l, float: u) =
	forall (x_i in x) (var_dom(x_i, l, u));
test var_dom(array [int] of float: x, set of float: d) =
	forall (x_i in x) (var_dom(x_i, d));
predicate var_dom(array [int] of var float: x, set of float: d) =
	forall (x_i in x) (var_dom(x_i, d));

predicate set_in(array [$T] of var int: X, set of int: s) =
	forall (x in X) (x in s);
predicate int_eq(array [$T] of var int: X, int: s) = forall (x in X) (x = s);
predicate float_eq(array [$T] of var int: X, float: s) =
	 forall (x in X) (x = s);
predicate int_le(array [$T] of var int: X, int: s) = forall (x in X) (x <= s);
predicate int_le(int: s, array [$T] of var int: X) = forall (x in X) (x >= s);
predicate float_le(array [$T] of var float: X, float: s) =
	forall (x in X) (x <= s);
predicate float_le(float: s, array [$T] of var float: X) =
	forall (x in X) (x >= s);

predicate array_var_int_element(var int: x, array [int] of int: y, var int: z) =
	array_int_element(x, y, z);
predicate array_var_bool_element(
	var int: x,
	array [int] of bool: y,
	var bool: z,
) = array_bool_element(x, y, z);
predicate array_var_float_element(
	var int: x,
	array [int] of float: y,
	var float: z,
) = array_float_element(x, y, z);
predicate array_var_set_element(
	var int: x,
	array [int] of set of int: y,
	var set of int: z,
) = array_set_element(x, y, z);

predicate bool_opt_eq(var opt bool: x, var opt bool: y) =
	deopt(x) = deopt(y) /\ occurs(x) = occurs(y);
/* True iff both \a b0 and \a b1 are absent or
  both are present and have the same value. */
predicate bool_eq(var opt bool: b0, var opt bool: b1) =
	absent(b0) /\ absent(b1) \/
		occurs(b0) /\ occurs(b1) /\ deopt(b0) = deopt(b1);
/* True iff \a b0 occurs and is equal to \a b1 */
predicate bool_eq(var opt bool: b0, var bool: b1) =
	 occurs(b0) /\ deopt(b0) = b1;
/* True iff \a b1 occurs and is equal to \a b0 */
predicate bool_eq(var bool: b0, var opt bool: b1) =
	 occurs(b1) /\ deopt(b1) = b0;

predicate bool_xor_reif(var bool: a, var bool: b, var bool: c) =
	bool_xor(a, b, c);

predicate int_opt_eq(var opt int: x, var opt int: y) =
	deopt(x) = deopt(y) /\ occurs(x) = occurs(y);
/* True iff both \a x and \a y are absent or
  both are present and have the same value. */
predicate int_eq(var opt int: x, var opt int: y) =
	absent(x) /\ absent(y) \/
		occurs(x) /\ occurs(y) /\ (deopt(x) = deopt(y)) :: maybe_partial;
/* True iff only one of \a x and \a y is absent or
  both are present and have different values. */
predicate int_ne(var opt int: x, var opt int: y) =
	absent(x) != absent(y) \/
		occurs(x) /\ occurs(y) /\ (deopt(x) != deopt(y)) :: maybe_partial;
/* Constrains \a x \( \in \) \a S */
predicate set_in(var opt int: x, set of int: S) =
	if occurs(x) then deopt(x) in S endif;
/* Constrains \a x \( \in \) \a S */
predicate set_in(var opt int: x, var set of int: S) =
	if occurs(x) then deopt(x) in S endif;

% :NOTE: does it apply to float?

/*
predicate var_dom(var opt float:x, set of float: s) =
  let {
    var float: dx = deopt(x);
    set of float: new_dom = dom(dx) intersect s;
  } in if new_dom = {} then absent(x) else dx in new_dom endif;

predicate var_dom(array[$T] of var opt float: x, set of float: d) =
    let { array[int] of var opt float: xx = array1d(x) }
    in forall (i in index_set(xx)) (var_dom(xx[i],d));
*/
predicate float_dom(var opt float: x, array [int] of float: as) =
	float_dom(deopt(x), as);

predicate float_opt_eq(var opt float: x, var opt float: y) =
	deopt(x) = deopt(y) /\ occurs(x) = occurs(y);
/* True iff both \a x and \a y are absent or
  both are present and have the same value. */
predicate float_eq(var opt float: x, var opt float: y) =
	absent(x) /\ absent(y) \/
		occurs(x) /\ occurs(y) /\ (deopt(x) = deopt(y)) :: maybe_partial;

/* True iff only one of \a x and \a y is absent or
  both are present and have different values. */
predicate float_ne(var opt float: x, var opt float: y) =
	absent(x) != absent(y) \/
		occurs(x) /\ occurs(y) /\ (deopt(x) != deopt(y)) :: maybe_partial;

predicate xorall_reif(array [int] of var bool: b, var bool: c) =
	let {
		var bool: nc :: is_defined_var;
		constraint xorall([nc] ++ b) :: defines_var(nc);
	} in c = not nc;

function var int: lin_exp(array [int] of int, array [int] of var int, int)
	:: mzn_internal_representation;
function var float: lin_exp(
	array [int] of float,
	array [int] of var float,
	float,
) :: mzn_internal_representation;

test mzn_in_root_context(var $T) :: mzn_internal_representation;

test mzn_in_redundant_constraint();

test mzn_in_symmetry_breaking_constraint();

/* Internal function used to optimize over option type objective */
function var float: objective_deopt_(var opt float: x, bool: direction) =
	let {
		float: worst = if direction then lb(x) - 1 else ub(x) + 1 endif;
	} in if occurs(x) then deopt(x) else worst endif;

%-----------------------------------------------------------------------------%
%
% Element constraint implementations
%
% MiniZinc compiles element constraints using a series of intermediate
% functions that test whether the constraint is total and perform array slicing
% for multi-dimensional element constraints.
%

% Unchecked internal array par array access
function any $T: mzn_element_internal(array [$E] of any $T: x, $F: idx);

% To allow var dispatch to par version
function any $T: element(array [$E] of any $T: x, $E: idx) = '[]'(x, idx);
function any $T: element_total(array [$E] of any $T: x, $E: idx) =
	mzn_element_internal(x, idx);

% 1D var array access not known to be total
function var $T: element(array [$$E] of var $T: x, var $$E: idx) =
	if mzn_in_root_context(idx) then
		let {
			constraint mzn_array_access_valid(x, idx);
		} in element_t(mzn_erase_index_sets(x), erase_enum(idx))
	elseif mzn_array_access_known_valid(x, idx) then
		element_t(mzn_erase_index_sets(x), erase_enum(idx))
	else
		let {
			constraint mzn_array_access_valid(x, idx);
		} in element_mt(mzn_erase_index_sets(x), erase_enum(idx))
	endif;
function var $$T: element(array [$$E] of var $$T: x, var $$E: idx) =
	if mzn_in_root_context(idx) then
		let {
			constraint mzn_array_access_valid(x, idx);
		} in mzn_to_enum(enum_of(x), element_t(erase_enum(x), erase_enum(idx)))
	elseif mzn_array_access_known_valid(x, idx) then
		mzn_to_enum(enum_of(x), element_t(erase_enum(x), erase_enum(idx)))
	else
		let {
			constraint mzn_array_access_valid(x, idx);
		} in mzn_to_enum(
			enum_of(x),
			element_mt(erase_enum(x), erase_enum(idx)),
		)
	endif;
function var $T: element(array [$$E] of var $T: x, tuple(var $$E): idx) =
	element(x, idx.1);

% 1D var array access known to be total
function var $T: element_total(array [$$E] of var $T: x, var $$E: idx) =
	element_t(mzn_erase_index_sets(x), erase_enum(idx));
function var $$T: element_total(array [$$E] of var $$T: x, var $$E: idx) =
	mzn_to_enum(enum_of(x), element_t(erase_enum(x), erase_enum(idx)));
function var $T: element_total(array [$$E] of var $T: x, tuple(var $$E): idx) =
	element_total(x, idx.1);

% 2D var array access not known to be total
function var $T: element(
	array [$$E, $$F] of var $T: x,
	tuple(var $$E, var $$F): idx,
) =
	if is_fixed(idx.1) then
		element(x[fix(idx.1), ..], idx.2)
	elseif is_fixed(idx.2) then
		element(x[.., fix(idx.2)], idx.1)
	elseif mzn_in_root_context(idx) then
		let {
			constraint mzn_array_access_valid(x, idx);
		} in element_t(
			mzn_erase_index_sets(x),
			erase_enum(idx.1),
			erase_enum(idx.2),
		)
	else
		let {
			constraint mzn_array_access_valid(x, idx);
		} in element_mt(
			mzn_erase_index_sets(x),
			erase_enum(idx.1),
			erase_enum(idx.2),
		)
	endif;
function var $$T: element(
	array [$$E, $$F] of var $$T: x,
	tuple(var $$E, var $$F): idx,
) =
	if is_fixed(idx.1) then
		element(x[fix(idx.1), ..], idx.2)
	elseif is_fixed(idx.2) then
		element(x[.., fix(idx.2)], idx.1)
	elseif mzn_in_root_context(idx) then
		let {
			constraint mzn_array_access_valid(x, idx);
		} in mzn_to_enum(
			enum_of(x),
			element_t(erase_enum(x), erase_enum(idx.1), erase_enum(idx.2)),
		)
	else
		let {
			constraint mzn_array_access_valid(x, idx);
		} in mzn_to_enum(
			enum_of(x),
			element_mt(erase_enum(x), erase_enum(idx.1), erase_enum(idx.2)),
		)
	endif;

% 2D var array access known to be total
function var $T: element_total(
	array [$$E, $$F] of var $T: x,
	tuple(var $$E, var $$F): idx,
) =
	if is_fixed(idx.1) then
		let {
			any: idx_set = index_sets(x);
		} in element_total(
			array1d(
				idx_set.2,
				mzn_slice_internal(
					x,
					[{erase_enum(fix(idx.1))}, erase_enum(idx_set.2)],
				),
			),
			idx.2,
		)
	elseif is_fixed(idx.2) then
		let {
			any: idx_set = index_sets(x);
		} in element_total(
			array1d(
				idx_set.1,
				mzn_slice_internal(
					x,
					[erase_enum(idx_set.1), {erase_enum(fix(idx.2))}],
				),
			),
			idx.1,
		)
	else
		element_t(mzn_erase_index_sets(x), erase_enum(idx.1), erase_enum(idx.2))
	endif;
function var $$T: element_total(
	array [$$E, $$F] of var $$T: x,
	tuple(var $$E, var $$F): idx,
) =
	if is_fixed(idx.1) then
		let {
			any: idx_set = index_sets(x);
		} in element_total(
			array1d(
				idx_set.2,
				mzn_slice_internal(
					x,
					[{erase_enum(fix(idx.1))}, erase_enum(idx_set.2)],
				),
			),
			idx.2,
		)
	elseif is_fixed(idx.2) then
		let {
			any: idx_set = index_sets(x);
		} in element_total(
			array1d(
				idx_set.1,
				mzn_slice_internal(
					x,
					[erase_enum(idx_set.1), {erase_enum(fix(idx.2))}],
				),
			),
			idx.1,
		)
	else
		mzn_to_enum(
			enum_of(x),
			element_t(erase_enum(x), erase_enum(idx.1), erase_enum(idx.2)),
		)
	endif;

% Multidimensional var array access
function var $T: element(array [$E] of var $T: x, var $F: idx) =
	let {
		any: pairs = mzn_index_pairs(x, idx);
		constraint mzn_fixed_array_access_valid(x, idx);
		any: xx =
			mzn_slice_internal(
				x,
				[
					if is_fixed(i) then {fix(i)} else is endif |
						(i, is) in pairs,
				],
			);
		any: var_pairs = [p | p in pairs where not is_fixed(p.1)];
	} in if length(var_pairs) == 1 then
		let {
			any: (i, is) = mzn_element_internal(var_pairs, 1);
		} in element(xx, i - min(is) + 1)
	elseif length(var_pairs) == 2 then
		let {
			any: (i1, is1) = mzn_element_internal(var_pairs, 1);
			any: (i2, is2) = mzn_element_internal(var_pairs, 2);
		} in element(
			array2d(is1, is2, xx),
			(i1 - min(is1) + 1, i2 - min(is2) + 1),
		)
	else
		let {
			any: index =
				(
					1 +
						sum (
							a in index_set(var_pairs),
							(i, is) = mzn_element_internal(var_pairs, a),
						) (
							(i - min(is)) *
								product (
									b in a + 1..max(index_set(var_pairs)),
								) (card(mzn_element_internal(var_pairs, b).2))
						)
				) :: domain_propagation;
		} in element(xx, index)
	endif;
function var $T: element_total(array [$E] of var $T: x, var $F: idx) =
	let {
		any: pairs = mzn_index_pairs(x, idx);
		any: xx =
			mzn_slice_internal(
				x,
				[
					if is_fixed(i) then {fix(i)} else is endif |
						(i, is) in pairs,
				],
			);
		any: var_pairs = [p | p in pairs where not is_fixed(p.1)];
	} in if length(var_pairs) == 1 then
		let {
			any: (i, is) = mzn_element_internal(var_pairs, 1);
		} in element_total(xx, i - min(is) + 1)
	elseif length(var_pairs) == 2 then
		let {
			any: (i1, is1) = mzn_element_internal(var_pairs, 1);
			any: (i2, is2) = mzn_element_internal(var_pairs, 2);
		} in element_total(
			array2d(is1, is2, xx),
			(i1 - min(is1) + 1, i2 - min(is2) + 1),
		)
	else
		let {
			any: index =
				(
					1 +
						sum (
							a in index_set(var_pairs),
							(i, is) = mzn_element_internal(var_pairs, a),
						) (
							(i - min(is)) *
								product (
									b in a + 1..max(index_set(var_pairs)),
								) (card(mzn_element_internal(var_pairs, b).2))
						)
				) :: domain_propagation;
		} in element_total(xx, index)
	endif;

% Helper to check validity of runtime fixed indices
test mzn_fixed_array_access_valid(array [$E] of any $T: x, var $F: idx)
	:: mzn_unreachable;
test mzn_fixed_array_access_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var $$E, var $$F, var $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
		constraint
			if is_fixed(idx.1) then
				mzn_array_access_valid(fix(idx.1), idx_set.1, 1)
			endif;
		constraint
			if is_fixed(idx.2) then
				mzn_array_access_valid(fix(idx.2), idx_set.2, 2)
			endif;
		constraint
			if is_fixed(idx.3) then
				mzn_array_access_valid(fix(idx.3), idx_set.3, 3)
			endif;
	} in true;
test mzn_fixed_array_access_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H): idx,
	tuple(int, int, int, int): n,
) =
	let {
		any: idx_set = index_sets(x);
		constraint
			if is_fixed(idx.1) then
				mzn_array_access_valid(fix(idx.1), idx_set.1, 1)
			endif;
		constraint
			if is_fixed(idx.2) then
				mzn_array_access_valid(fix(idx.2), idx_set.2, 2)
			endif;
		constraint
			if is_fixed(idx.3) then
				mzn_array_access_valid(fix(idx.3), idx_set.3, 3)
			endif;
		constraint
			if is_fixed(idx.4) then
				mzn_array_access_valid(fix(idx.4), idx_set.4, 4)
			endif;
	} in true;
test mzn_fixed_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I): idx,
	tuple(int, int, int, int, int): n,
) =
	let {
		any: idx_set = index_sets(x);
		constraint
			if is_fixed(idx.1) then
				mzn_array_access_valid(fix(idx.1), idx_set.1, 1)
			endif;
		constraint
			if is_fixed(idx.2) then
				mzn_array_access_valid(fix(idx.2), idx_set.2, 2)
			endif;
		constraint
			if is_fixed(idx.3) then
				mzn_array_access_valid(fix(idx.3), idx_set.3, 3)
			endif;
		constraint
			if is_fixed(idx.4) then
				mzn_array_access_valid(fix(idx.4), idx_set.4, 4)
			endif;
		constraint
			if is_fixed(idx.5) then
				mzn_array_access_valid(fix(idx.5), idx_set.5, 5)
			endif;
	} in true;
test mzn_fixed_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I, var $$J): idx,
	tuple(int, int, int, int, int, int): n,
) =
	let {
		any: idx_set = index_sets(x);
		constraint
			if is_fixed(idx.1) then
				mzn_array_access_valid(fix(idx.1), idx_set.1, 1)
			endif;
		constraint
			if is_fixed(idx.2) then
				mzn_array_access_valid(fix(idx.2), idx_set.2, 2)
			endif;
		constraint
			if is_fixed(idx.3) then
				mzn_array_access_valid(fix(idx.3), idx_set.3, 3)
			endif;
		constraint
			if is_fixed(idx.4) then
				mzn_array_access_valid(fix(idx.4), idx_set.4, 4)
			endif;
		constraint
			if is_fixed(idx.5) then
				mzn_array_access_valid(fix(idx.5), idx_set.5, 5)
			endif;
		constraint
			if is_fixed(idx.6) then
				mzn_array_access_valid(fix(idx.6), idx_set.6, 6)
			endif;
	} in true;

% Helper to turn run fixed indexing into slicing
function array [int] of tuple(var int, set of int): mzn_index_pairs(
	array [$E] of any $T: x,
	var $F: idx,
) :: mzn_unreachable;
function array [int] of tuple(var int, set of int): mzn_index_pairs(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var $$E, var $$F, var $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in [
		(erase_enum(idx.1), erase_enum(idx_set.1)),
		(erase_enum(idx.2), erase_enum(idx_set.2)),
		(erase_enum(idx.3), erase_enum(idx_set.3)),
	];
function array [int] of tuple(var int, set of int): mzn_index_pairs(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in [
		(erase_enum(idx.1), erase_enum(idx_set.1)),
		(erase_enum(idx.2), erase_enum(idx_set.2)),
		(erase_enum(idx.3), erase_enum(idx_set.3)),
		(erase_enum(idx.4), erase_enum(idx_set.4)),
	];
function array [int] of tuple(var int, set of int): mzn_index_pairs(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in [
		(erase_enum(idx.1), erase_enum(idx_set.1)),
		(erase_enum(idx.2), erase_enum(idx_set.2)),
		(erase_enum(idx.3), erase_enum(idx_set.3)),
		(erase_enum(idx.4), erase_enum(idx_set.4)),
		(erase_enum(idx.5), erase_enum(idx_set.5)),
	];
function array [int] of tuple(var int, set of int): mzn_index_pairs(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I, var $$J): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in [
		(erase_enum(idx.1), erase_enum(idx_set.1)),
		(erase_enum(idx.2), erase_enum(idx_set.2)),
		(erase_enum(idx.3), erase_enum(idx_set.3)),
		(erase_enum(idx.4), erase_enum(idx_set.4)),
		(erase_enum(idx.5), erase_enum(idx_set.5)),
		(erase_enum(idx.6), erase_enum(idx_set.6)),
	];

% Internal total element functions
function var $T: element_t(array [int] of var $T: x, var int: idx)
	:: mzn_unreachable;
function var $T: element_t(
	array [int, int] of var $T: x,
	var int: idx1,
	var int: idx2,
) :: mzn_unreachable;
function var $T: element_mt(array [int] of var $T: x, var int: idx)
	:: mzn_unreachable;
function var $T: element_mt(
	array [int, int] of var $T: x,
	var int: idx1,
	var int: idx2,
) :: mzn_unreachable;

% Element on integers
function var int: element_t(array [int] of var int: x, var int: idx)
	:: promise_total =
	let {
		var dom_bounds_array(x): r :: is_defined_var;
		constraint idx in index_set(x);
		constraint
			 array_var_int_element_nonshifted(idx, x, r) :: defines_var(r);
	} in r;

function var int: element_mt(array [int] of var int: x, var int: idx)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		var min(index_set(x))..max(index_set(x)): idx2;
		constraint idx in index_set(x) -> idx2 = idx;
		constraint idx in index_set(x) \/ idx2 = min(index_set(x));
		constraint
			array_var_int_element_nonshifted(idx2, x, r) :: defines_var(r);
	} in r;

function var int: element_t(
	array [int, int] of var int: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var dom_bounds_array(x): r :: is_defined_var;
		constraint idx1 in index_set_1of2(x);
		constraint idx2 in index_set_2of2(x);
		constraint
			array_var_int_element2d_nonshifted(idx1, idx2, x, r)
				:: defines_var(r);
	} in r;

function var int: element_mt(
	array [int, int] of var int: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		var min(index_set_1of2(x))..max(index_set_1of2(x)): idx1_2;
		var min(index_set_2of2(x))..max(index_set_2of2(x)): idx2_2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) ->
				idx1_2 = idx1 /\ idx2_2 = idx2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) \/
				idx1_2 = min(index_set_1of2(x)) /\
					idx2_2 = min(index_set_2of2(x));
		constraint
			array_var_int_element2d_nonshifted(idx1_2, idx2_2, x, r)
				:: defines_var(r);
	} in r;

% Element on floats
function var float: element_t(array [int] of var float: x, var int: idx)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		constraint idx in index_set(x);
		constraint
			array_var_float_element_nonshifted(idx, x, r) :: defines_var(r);
	} in r;

function var float: element_mt(array [int] of var float: x, var int: idx)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		var min(index_set(x))..max(index_set(x)): idx2;
		constraint idx in index_set(x) -> idx2 = idx;
		constraint idx in index_set(x) \/ idx2 = min(index_set(x));
		constraint
			array_var_float_element_nonshifted(idx2, x, r) :: defines_var(r);
	} in r;

function var float: element_t(
	array [int, int] of var float: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		constraint idx1 in index_set_1of2(x);
		constraint idx2 in index_set_2of2(x);
		constraint
			array_var_float_element2d_nonshifted(idx1, idx2, x, r)
				:: defines_var(r);
	} in r;

function var float: element_mt(
	array [int, int] of var float: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var lb_array(x)..ub_array(x): r :: is_defined_var;
		var min(index_set_1of2(x))..max(index_set_1of2(x)): idx1_2;
		var min(index_set_2of2(x))..max(index_set_2of2(x)): idx2_2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) ->
				idx1_2 = idx1 /\ idx2_2 = idx2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) \/
				idx1_2 = min(index_set_1of2(x)) /\
					idx2_2 = min(index_set_2of2(x));
		constraint
			array_var_float_element2d_nonshifted(idx1_2, idx2_2, x, r)
				:: defines_var(r);
	} in r;

% Element on sets
function var set of int: element_t(
	array [int] of var set of int: x,
	var int: idx,
) :: promise_total =
	let {
		var set of min(ub_array(x))..max(ub_array(x)): r :: is_defined_var;
		constraint idx in index_set(x);
		constraint
			 array_var_set_element_nonshifted(idx, x, r) :: defines_var(r);
	} in r;

function var set of int: element_mt(
	array [int] of var set of int: x,
	var int: idx,
) :: promise_total =
	let {
		var set of min(ub_array(x))..max(ub_array(x)): r :: is_defined_var;
		var min(index_set(x))..max(index_set(x)): idx2;
		constraint idx in index_set(x) -> idx2 = idx;
		constraint idx in index_set(x) \/ idx2 = min(index_set(x));
		constraint
			array_var_set_element_nonshifted(idx2, x, r) :: defines_var(r);
	} in r;

function var set of int: element_t(
	array [int, int] of var set of int: x,
	var int: idx1,
	var int: idx2,
) :: promise_total =
	let {
		var set of min(ub_array(x))..max(ub_array(x)): r :: is_defined_var;
		constraint idx1 in index_set_1of2(x);
		constraint idx2 in index_set_2of2(x);
		constraint
			array_var_set_element2d_nonshifted(idx1, idx2, x, r)
				:: defines_var(r);
	} in r;

function var set of int: element_mt(
	array [int, int] of var set of int: x,
	var int: idx1,
	var int: idx2,
) :: promise_total =
	let {
		var set of min(ub_array(x))..max(ub_array(x)): r :: is_defined_var;
		var min(index_set_1of2(x))..max(index_set_1of2(x)): idx1_2;
		var min(index_set_2of2(x))..max(index_set_2of2(x)): idx2_2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) ->
				idx1_2 = idx1 /\ idx2_2 = idx2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) \/
				idx1_2 = min(index_set_1of2(x)) /\
					idx2_2 = min(index_set_2of2(x));
		constraint
			array_var_set_element2d_nonshifted(idx1_2, idx2_2, x, r)
				:: defines_var(r);
	} in r;

% Element on booleans
function var bool: element_t(array [int] of var bool: x, var int: idx)
	:: promise_total =
	let {
		var bool: r :: is_defined_var;
		constraint idx in index_set(x);
		constraint
			array_var_bool_element_nonshifted(idx, x, r) :: defines_var(r);
	} in r;

function var bool: element_mt(array [int] of var bool: x, var int: idx)
	:: promise_total =
	let {
		var bool: r :: is_defined_var;
		var min(index_set(x))..max(index_set(x)): idx2;
		constraint idx in index_set(x) -> idx2 = idx;
		constraint idx in index_set(x) \/ idx2 = min(index_set(x));
		constraint
			array_var_bool_element_nonshifted(idx2, x, r) :: defines_var(r);
	} in r;

function var bool: element_t(
	array [int, int] of var bool: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var bool: r :: is_defined_var;
		constraint idx1 in index_set_1of2(x);
		constraint idx2 in index_set_2of2(x);
		constraint
			array_var_bool_element2d_nonshifted(idx1, idx2, x, r)
				:: defines_var(r);
	} in r;

function var bool: element_mt(
	array [int, int] of var bool: x,
	var int: idx1,
	var int: idx2,
)
	:: promise_total =
	let {
		var bool: r :: is_defined_var;
		var min(index_set_1of2(x))..max(index_set_1of2(x)): idx1_2;
		var min(index_set_2of2(x))..max(index_set_2of2(x)): idx2_2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) ->
				idx1_2 = idx1 /\ idx2_2 = idx2;
		constraint
			idx1 in index_set_1of2(x) /\ idx2 in index_set_2of2(x) \/
				idx1_2 = min(index_set_1of2(x)) /\
					idx2_2 = min(index_set_2of2(x));
		constraint
			array_var_bool_element2d_nonshifted(idx1_2, idx2_2, x, r)
				:: defines_var(r);
	} in r;

% Optional array access
function opt $T: '[]'(array [$$E] of $T: x, opt $$E: idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(array [$$E] of $T: x, tuple(opt $$E): idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx.1);
function opt $T: '[]'(array [$$E, $$F] of $T: x, tuple(opt $$E, opt $$F): idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G] of $T: x,
	tuple(opt $$E, opt $$F, opt $$G): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H] of $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I] of $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);

function opt $T: '[]'(array [$$E] of opt $T: x, opt $$E: idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(array [$$E] of opt $T: x, tuple(opt $$E): idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx.1);
function opt $T: '[]'(
	array [$$E, $$F] of opt $T: x,
	tuple(opt $$E, opt $$F): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G] of opt $T: x,
	tuple(opt $$E, opt $$F, opt $$G): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H] of opt $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I] of opt $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of opt $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);

function var opt $T: '[]'(array [$$E] of var $T: x, opt $$E: idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(array [$$E] of var $T: x, tuple(opt $$E): idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx.1);
function var opt $T: '[]'(
	array [$$E, $$F] of var $T: x,
	tuple(opt $$E, opt $$F): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G] of var $T: x,
	tuple(opt $$E, opt $$F, opt $$G): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H] of var $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I] of var $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of var $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);

function var opt $T: '[]'(array [$$E] of var $T: x, var opt $$E: idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(array [$$E] of var $T: x, tuple(var opt $$E): idx)
	:: mzn_inline_call_by_name = element_opt_dispatch(x, idx.1);
function var opt $T: '[]'(
	array [$$E, $$F] of var $T: x,
	tuple(var opt $$E, var opt $$F): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G] of var $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H] of var $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I] of var $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
	): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);
function var opt $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of var $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): idx,
) :: mzn_inline_call_by_name = element_opt_dispatch(x, idx);

% Helper to test whether indices all occur
test mzn_all_occurs($E) :: mzn_unreachable;
test mzn_all_occurs(opt $$E: i) = occurs(i);
test mzn_all_occurs(tuple(opt $$E): i) = occurs(i.1);
test mzn_all_occurs(tuple(opt $$E, opt $$F): i) = occurs(i.1) /\ occurs(i.2);
test mzn_all_occurs(tuple(opt $$E, opt $$F, opt $$G): i) =
	forall([occurs(i.1), occurs(i.2), occurs(i.3)]);
test mzn_all_occurs(tuple(opt $$E, opt $$F, opt $$G, opt $$H): i) =
	forall([occurs(i.1), occurs(i.2), occurs(i.3), occurs(i.4)]);
test mzn_all_occurs(tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): i) =
	forall([occurs(i.1), occurs(i.2), occurs(i.3), occurs(i.4), occurs(i.5)]);
test mzn_all_occurs(
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): i,
) =
	forall([
		occurs(i.1),
		occurs(i.2),
		occurs(i.3),
		occurs(i.4),
		occurs(i.5),
		occurs(i.6),
	]);
predicate mzn_all_occurs(var $E) :: mzn_unreachable;
predicate mzn_all_occurs(var opt $$E: i) = occurs(i);
predicate mzn_all_occurs(tuple(var opt $$E): i) = occurs(i.1);
predicate mzn_all_occurs(tuple(var opt $$E, var opt $$F): i) =
	occurs(i.1) /\ occurs(i.2);
predicate mzn_all_occurs(tuple(var opt $$E, var opt $$F, var opt $$G): i) =
	forall([occurs(i.1), occurs(i.2), occurs(i.3)]);
predicate mzn_all_occurs(
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): i,
) = forall([occurs(i.1), occurs(i.2), occurs(i.3), occurs(i.4)]);
predicate mzn_all_occurs(
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H, var opt $$I): i,
) = forall([occurs(i.1), occurs(i.2), occurs(i.3), occurs(i.4), occurs(i.5)]);
predicate mzn_all_occurs(
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): i,
) =
	forall([
		occurs(i.1),
		occurs(i.2),
		occurs(i.3),
		occurs(i.4),
		occurs(i.5),
		occurs(i.6),
	]);

% Helper to test whether indices are all absent
test mzn_all_absent($E) :: mzn_unreachable;
test mzn_all_absent(opt $$E: i) = absent(i);
test mzn_all_absent(tuple(opt $$E): i) = absent(i.1);
test mzn_all_absent(tuple(opt $$E, opt $$F): i) = absent(i.1) /\ absent(i.2);
test mzn_all_absent(tuple(opt $$E, opt $$F, opt $$G): i) =
	forall([absent(i.1), absent(i.2), absent(i.3)]);
test mzn_all_absent(tuple(opt $$E, opt $$F, opt $$G, opt $$H): i) =
	forall([absent(i.1), absent(i.2), absent(i.3), absent(i.4)]);
test mzn_all_absent(tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): i) =
	forall([absent(i.1), absent(i.2), absent(i.3), absent(i.4), absent(i.5)]);
test mzn_all_absent(
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): i,
) =
	forall([
		absent(i.1),
		absent(i.2),
		absent(i.3),
		absent(i.4),
		absent(i.5),
		absent(i.6),
	]);
predicate mzn_all_absent(var $E) :: mzn_unreachable;
predicate mzn_all_absent(var opt $$E: i) = absent(i);
predicate mzn_all_absent(tuple(var opt $$E): i) = absent(i.1);
predicate mzn_all_absent(tuple(var opt $$E, var opt $$F): i) =
	absent(i.1) /\ absent(i.2);
predicate mzn_all_absent(tuple(var opt $$E, var opt $$F, var opt $$G): i) =
	forall([absent(i.1), absent(i.2), absent(i.3)]);
predicate mzn_all_absent(
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): i,
) = forall([absent(i.1), absent(i.2), absent(i.3), absent(i.4)]);
predicate mzn_all_absent(
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H, var opt $$I): i,
) = forall([absent(i.1), absent(i.2), absent(i.3), absent(i.4), absent(i.5)]);
predicate mzn_all_absent(
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): i,
) =
	forall([
		absent(i.1),
		absent(i.2),
		absent(i.3),
		absent(i.4),
		absent(i.5),
		absent(i.6),
	]);

% Helper to deopt indices
function $E: mzn_deopt_index(array [$E] of any $T: x, $F: idx)
	:: mzn_unreachable;
function $$E: mzn_deopt_index(array [$$E] of any $T: x, opt $$E: idx) =
	deopt(idx);
function tuple($$E, $$F): mzn_deopt_index(
	array [$$E, $$F] of any $T: x,
	tuple(opt $$E, opt $$F): idx,
) = (deopt(idx.1), deopt(idx.2));
function tuple($$E, $$F, $$G): mzn_deopt_index(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3));
function tuple($$E, $$F, $$G, $$H): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3), deopt(idx.4));
function tuple($$E, $$F, $$G, $$H, $$I): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3), deopt(idx.4), deopt(idx.5));
function tuple($$E, $$F, $$G, $$H, $$I, $$J): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): idx,
) = (
	deopt(idx.1),
	deopt(idx.2),
	deopt(idx.3),
	deopt(idx.4),
	deopt(idx.5),
	deopt(idx.6),
);
function var $E: mzn_deopt_index(array [$E] of any $T: x, var $F: idx)
	:: mzn_unreachable;
function var $$E: mzn_deopt_index(array [$$E] of any $T: x, var opt $$E: idx) =
	deopt(idx);
function tuple(var $$E, var $$F): mzn_deopt_index(
	array [$$E, $$F] of any $T: x,
	tuple(var opt $$E, var opt $$F): idx,
) = (deopt(idx.1), deopt(idx.2));
function tuple(var $$E, var $$F, var $$G): mzn_deopt_index(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3));
function tuple(var $$E, var $$F, var $$G, var $$H): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3), deopt(idx.4));
function tuple(var $$E, var $$F, var $$G, var $$H, var $$I): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H, var opt $$I): idx,
) = (deopt(idx.1), deopt(idx.2), deopt(idx.3), deopt(idx.4), deopt(idx.5));
function tuple(
	var $$E,
	var $$F,
	var $$G,
	var $$H,
	var $$I,
	var $$J,
): mzn_deopt_index(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): idx,
) = (
	deopt(idx.1),
	deopt(idx.2),
	deopt(idx.3),
	deopt(idx.4),
	deopt(idx.5),
	deopt(idx.6),
);

% Par optional array access
function opt $T: element_opt_dispatch(array [$$E] of $T: x, opt $$E: idx)
	:: mzn_inline =
	let { constraint mzn_array_access_valid(x, idx) } in element_opt(x, idx);
function opt $T: element_opt_dispatch(array [$E] of $T: x, $F: idx)
	:: mzn_inline =
	let { constraint mzn_array_access_valid(x, idx) } in element_opt(x, idx);
function opt $T: element_opt_dispatch(array [$$E] of opt $T: x, opt $$E: idx)
	:: mzn_inline =
	let { constraint mzn_array_access_valid(x, idx) } in element_opt(x, idx);
function opt $T: element_opt_dispatch(array [$E] of opt $T: x, $F: idx)
	:: mzn_inline =
	let { constraint mzn_array_access_valid(x, idx) } in element_opt(x, idx);

function opt $T: element_opt(array [$$E] of $T: x, opt $$E: idx) =
	if occurs(idx) then mzn_element_internal(x, deopt(idx)) else <> endif;
function opt $T: element_opt(array [$E] of $T: x, $F: idx) =
	if mzn_all_occurs(idx) then
		mzn_element_internal(x, mzn_deopt_index(x, idx))
	else
		<>
	endif;
function opt $T: element_opt(array [$$E] of opt $T: x, opt $$E: idx) =
	if occurs(idx) then mzn_element_internal(x, deopt(idx)) else <> endif;
function opt $T: element_opt(array [$E] of opt $T: x, $F: idx) =
	if mzn_all_occurs(idx) then
		mzn_element_internal(x, mzn_deopt_index(x, idx))
	else
		<>
	endif;

% Var optional array access
function var opt $T: element_opt_dispatch(
	array [$$E] of var $T: x,
	var opt $$E: idx,
) :: mzn_inline =
	if mzn_array_access_known_valid(x, idx) then
		element_opt_total(x, idx)
	else
		element_opt(x, idx)
	endif;
function var opt $T: element_opt_dispatch(array [$E] of var $T: x, var $F: idx)
	:: mzn_inline =
	if mzn_array_access_known_valid(x, idx) then
		element_opt_total(x, idx)
	else
		element_opt(x, idx)
	endif;
function var opt $T: element_opt_dispatch(
	array [$$E] of var opt $T: x,
	var opt $$E: idx,
) :: mzn_inline =
	if mzn_array_access_known_valid(x, idx) then
		element_opt_total(x, idx)
	else
		element_opt(x, idx)
	endif;
function var opt $T: element_opt_dispatch(
	array [$E] of var opt $T: x,
	var $F: idx,
) :: mzn_inline =
	if mzn_array_access_known_valid(x, idx) then
		element_opt_total(x, idx)
	else
		element_opt(x, idx)
	endif;

function var opt $T: element_opt(array [$$E] of var $T: x, var opt $$E: idx) =
	if length(x) = 0 then
		let { constraint absent(idx) } in <>
	elseif occurs(idx) then
		element(x, deopt(idx))
	else
		<>
	endif;
function var opt $T: element_opt(array [$E] of var $T: x, var $F: idx) =
	if length(x) = 0 then
		let { constraint mzn_all_absent(idx) } in <>
	elseif mzn_all_occurs(idx) then
		element(x, mzn_deopt_index(x, idx))
	else
		<>
	endif;
function var opt $T: element_opt(
	array [$$E] of var opt $T: x,
	var opt $$E: idx,
) =
	if length(x) = 0 then
		let { constraint absent(idx) } in <>
	elseif mzn_can_extend_array_opt(x, idx) then
		mzn_construct_opt(
			element(mzn_destruct_opt(mzn_extend_array_opt(x)), deopt(idx)),
		)
	elseif occurs(idx) then
		mzn_construct_opt(element(mzn_destruct_opt(x), deopt(idx)))
	else
		<>
	endif;
function var opt $T: element_opt(array [$E] of var opt $T: x, var $F: idx) =
	if length(x) = 0 then
		let { constraint mzn_all_absent(idx) } in <>
	elseif mzn_can_extend_array_opt(x, idx) then
		mzn_construct_opt(
			element(
				mzn_destruct_opt(mzn_extend_array_opt(x)),
				mzn_deopt_index(x, idx),
			),
		)
	elseif mzn_all_occurs(idx) then
		mzn_construct_opt(element(mzn_destruct_opt(x), mzn_deopt_index(x, idx)))
	else
		<>
	endif;
function var opt $T: element_opt_total(
	array [$$E] of var $T: x,
	var opt $$E: idx,
) = if occurs(idx) then element_total(x, deopt(idx)) else <> endif;
function var opt $T: element_opt_total(array [$E] of var $T: x, var $F: idx) =
	if mzn_all_occurs(idx) then
		element_total(x, mzn_deopt_index(x, idx))
	else
		<>
	endif;
function var opt $T: element_opt_total(
	array [$$E] of var opt $T: x,
	var opt $$E: idx,
) =
	if mzn_can_extend_array_opt(x, idx) then
		mzn_construct_opt(
			element_total(
				mzn_destruct_opt(mzn_extend_array_opt(x)),
				deopt(idx),
			),
		)
	elseif occurs(idx) then
		mzn_construct_opt(
			element_total(mzn_destruct_opt(x), mzn_deopt_index(x, idx)),
		)
	else
		<>
	endif;
function var opt $T: element_opt_total(
	array [$E] of var opt $T: x,
	var $F: idx,
) =
	if mzn_can_extend_array_opt(x, idx) then
		mzn_construct_opt(
			element_total(
				mzn_destruct_opt(mzn_extend_array_opt(x)),
				mzn_deopt_index(x, idx),
			),
		)
	elseif mzn_all_occurs(idx) then
		mzn_construct_opt(
			element_total(mzn_destruct_opt(x), mzn_deopt_index(x, idx)),
		)
	else
		<>
	endif;

% Helper to test if it is cheap to extend the array to handle absent indices
test mzn_can_extend_array_opt(array [$E] of any $T: x, var $F: idx)
	:: mzn_unreachable;
test mzn_can_extend_array_opt(array [$$E] of any $T: x, var opt $$E: idx) =
	erase_enum(min(index_set(x))) = 1 /\ not had_zero(idx);
test mzn_can_extend_array_opt(
	array [$$E, $$F] of any $T: x,
	tuple(var opt $$E, var opt $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		erase_enum(min(idx_set.1)) = 1,
		erase_enum(min(idx_set.2)) = 1,
		not
		had_zero(idx.1),
		not
		had_zero(idx.2),
	]);
test mzn_can_extend_array_opt(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		erase_enum(min(idx_set.1)) = 1,
		erase_enum(min(idx_set.2)) = 1,
		erase_enum(min(idx_set.3)) = 1,
		not
		had_zero(idx.1),
		not
		had_zero(idx.2),
		not
		had_zero(idx.3),
	]);
test mzn_can_extend_array_opt(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		erase_enum(min(idx_set.1)) = 1,
		erase_enum(min(idx_set.2)) = 1,
		erase_enum(min(idx_set.3)) = 1,
		erase_enum(min(idx_set.4)) = 1,
		not
		had_zero(idx.1),
		not
		had_zero(idx.2),
		not
		had_zero(idx.3),
		not
		had_zero(idx.4),
	]);
test mzn_can_extend_array_opt(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
	): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		erase_enum(min(idx_set.1)) = 1,
		erase_enum(min(idx_set.2)) = 1,
		erase_enum(min(idx_set.3)) = 1,
		erase_enum(min(idx_set.4)) = 1,
		erase_enum(min(idx_set.5)) = 1,
		not
		had_zero(idx.1),
		not
		had_zero(idx.2),
		not
		had_zero(idx.3),
		not
		had_zero(idx.4),
		not
		had_zero(idx.5),
	]);
test mzn_can_extend_array_opt(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		erase_enum(min(idx_set.1)) = 1,
		erase_enum(min(idx_set.2)) = 1,
		erase_enum(min(idx_set.3)) = 1,
		erase_enum(min(idx_set.4)) = 1,
		erase_enum(min(idx_set.5)) = 1,
		erase_enum(min(idx_set.6)) = 1,
		not
		had_zero(idx.1),
		not
		had_zero(idx.2),
		not
		had_zero(idx.3),
		not
		had_zero(idx.4),
		not
		had_zero(idx.5),
		not
		had_zero(idx.6),
	]);

% Helper to extend arrays to include <> at index zero
function array [$E] of any $T: mzn_extend_array_opt(array [$E] of any $T: x)
	:: mzn_unreachable;
function array [$$E] of any $T: mzn_extend_array_opt(array [$$E] of any $T: x) =
	array1d(
		mzn_to_enum(enum_of(index_set(x)), 0)..max(index_set(x)),
		[
			if i = 0 then <> else mzn_element_internal(x, i) endif |
				i in 0..erase_enum(max(index_set(x))),
		],
	);
function array [$$E, $$F] of any $T: mzn_extend_array_opt(
	array [$$E, $$F] of any $T: x,
) =
	let {
		any: idx_set = index_sets(x);
	} in array2d(
		mzn_to_enum(enum_of(idx_set.1), 0)..max(idx_set.1),
		mzn_to_enum(enum_of(idx_set.2), 0)..max(idx_set.2),
		[
			if i = 0 \/ j = 0 then <> else mzn_element_internal(x, (i, j)) endif |
				i in 0..erase_enum(max(idx_set.1)),
				j in 0..erase_enum(max(idx_set.2)),
		],
	);
function array [$$E, $$F, $$G] of any $T: mzn_extend_array_opt(
	array [$$E, $$F, $$G] of any $T: x,
) =
	let {
		any: idx_set = index_sets(x);
	} in array3d(
		mzn_to_enum(enum_of(idx_set.1), 0)..max(idx_set.1),
		mzn_to_enum(enum_of(idx_set.2), 0)..max(idx_set.2),
		mzn_to_enum(enum_of(idx_set.3), 0)..max(idx_set.3),
		[
			if i = 0 \/ j = 0 \/ k = 0 then
				<>
			else
				mzn_element_internal(x, (i, j, k))
			endif |
				i in 0..erase_enum(max(idx_set.1)),
				j in 0..erase_enum(max(idx_set.2)),
				k in 0..erase_enum(max(idx_set.3)),
		],
	);
function array [$$E, $$F, $$G, $$H] of any $T: mzn_extend_array_opt(
	array [$$E, $$F, $$G, $$H] of any $T: x,
) =
	let {
		any: idx_set = index_sets(x);
	} in array4d(
		mzn_to_enum(enum_of(idx_set.1), 0)..max(idx_set.1),
		mzn_to_enum(enum_of(idx_set.2), 0)..max(idx_set.2),
		mzn_to_enum(enum_of(idx_set.3), 0)..max(idx_set.3),
		mzn_to_enum(enum_of(idx_set.4), 0)..max(idx_set.4),
		[
			if i = 0 \/ j = 0 \/ k = 0 \/ l = 0 then
				<>
			else
				mzn_element_internal(x, (i, j, k, l))
			endif |
				i in 0..erase_enum(max(idx_set.1)),
				j in 0..erase_enum(max(idx_set.2)),
				k in 0..erase_enum(max(idx_set.3)),
				l in 0..erase_enum(max(idx_set.4)),
		],
	);
function array [$$E, $$F, $$G, $$H, $$I] of any $T: mzn_extend_array_opt(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
) =
	let {
		any: idx_set = index_sets(x);
	} in array5d(
		mzn_to_enum(enum_of(idx_set.1), 0)..max(idx_set.1),
		mzn_to_enum(enum_of(idx_set.2), 0)..max(idx_set.2),
		mzn_to_enum(enum_of(idx_set.3), 0)..max(idx_set.3),
		mzn_to_enum(enum_of(idx_set.4), 0)..max(idx_set.4),
		mzn_to_enum(enum_of(idx_set.5), 0)..max(idx_set.5),
		[
			if i = 0 \/ j = 0 \/ k = 0 \/ l = 0 \/ m = 0 then
				<>
			else
				mzn_element_internal(x, (i, j, k, l, m))
			endif |
				i in 0..erase_enum(max(idx_set.1)),
				j in 0..erase_enum(max(idx_set.2)),
				k in 0..erase_enum(max(idx_set.3)),
				l in 0..erase_enum(max(idx_set.4)),
				m in 0..erase_enum(max(idx_set.5)),
		],
	);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: mzn_extend_array_opt(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
) =
	let {
		any: idx_set = index_sets(x);
	} in array6d(
		mzn_to_enum(enum_of(idx_set.1), 0)..max(idx_set.1),
		mzn_to_enum(enum_of(idx_set.2), 0)..max(idx_set.2),
		mzn_to_enum(enum_of(idx_set.3), 0)..max(idx_set.3),
		mzn_to_enum(enum_of(idx_set.4), 0)..max(idx_set.4),
		mzn_to_enum(enum_of(idx_set.5), 0)..max(idx_set.5),
		mzn_to_enum(enum_of(idx_set.6), 0)..max(idx_set.6),
		[
			if i = 0 \/ j = 0 \/ k = 0 \/ l = 0 \/ m = 0 \/ n = 0 then
				<>
			else
				mzn_element_internal(x, (i, j, k, l, m, n))
			endif |
				i in 0..erase_enum(max(idx_set.1)),
				j in 0..erase_enum(max(idx_set.2)),
				k in 0..erase_enum(max(idx_set.3)),
				l in 0..erase_enum(max(idx_set.4)),
				m in 0..erase_enum(max(idx_set.5)),
				n in 0..erase_enum(max(idx_set.6)),
		],
	);

% Par index validity check
test mzn_array_access_valid($$E: idx, set of $$E: idx_set, int: n) =
	mzn_assert_warn(
		idx in idx_set,
		if n > 0 then
			"array access out of bounds, index set \(n) is \(
				idx_set
			), but given index is \(idx)"
		else
			"array access out of bounds, index set is \(
				idx_set
			), but given index is \(idx)"
		endif,
	);
test mzn_array_access_valid(array [$E] of any $T: x, $F: idx)
	 :: mzn_unreachable;
test mzn_array_access_valid(array [$$E] of any $T: x, $$E: idx) =
	mzn_array_access_valid(idx, index_set(x), 0);
test mzn_array_access_valid(array [$$E] of any $T: x, tuple($$E): idx) =
	mzn_array_access_valid(x, idx.1);
test mzn_array_access_valid(
	array [$$E, $$F] of any $T: x,
	tuple($$E, $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple($$E, $$F, $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple($$E, $$F, $$G, $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple($$E, $$F, $$G, $$H, $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
		mzn_array_access_valid(idx.5, idx_set.5, 5),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple($$E, $$F, $$G, $$H, $$I, $$J): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
		mzn_array_access_valid(idx.5, idx_set.5, 5),
		mzn_array_access_valid(idx.6, idx_set.6, 6),
	]);

% Par optional index validity check
test mzn_array_access_valid(array [$$E] of any $T: x, opt $$E: idx) =
	absent(idx) \/ mzn_array_access_valid(deopt(idx), index_set(x), 0);
test mzn_array_access_valid(array [$$E] of any $T: x, tuple(opt $$E): idx) =
	mzn_array_access_valid(x, idx.1);
test mzn_array_access_valid(
	array [$$E, $$F] of any $T: x,
	tuple(opt $$E, opt $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		absent(idx.1) \/ mzn_array_access_valid(deopt(idx.1), idx_set.1, 1),
		absent(idx.2) \/ mzn_array_access_valid(deopt(idx.2), idx_set.2, 2),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		absent(idx.1) \/ mzn_array_access_valid(deopt(idx.1), idx_set.1, 1),
		absent(idx.2) \/ mzn_array_access_valid(deopt(idx.2), idx_set.2, 2),
		absent(idx.3) \/ mzn_array_access_valid(deopt(idx.3), idx_set.3, 3),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		absent(idx.1) \/ mzn_array_access_valid(deopt(idx.1), idx_set.1, 1),
		absent(idx.2) \/ mzn_array_access_valid(deopt(idx.2), idx_set.2, 2),
		absent(idx.3) \/ mzn_array_access_valid(deopt(idx.3), idx_set.3, 3),
		absent(idx.4) \/ mzn_array_access_valid(deopt(idx.4), idx_set.4, 4),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		absent(idx.1) \/ mzn_array_access_valid(deopt(idx.1), idx_set.1, 1),
		absent(idx.2) \/ mzn_array_access_valid(deopt(idx.2), idx_set.2, 2),
		absent(idx.3) \/ mzn_array_access_valid(deopt(idx.3), idx_set.3, 3),
		absent(idx.4) \/ mzn_array_access_valid(deopt(idx.4), idx_set.4, 4),
		absent(idx.5) \/ mzn_array_access_valid(deopt(idx.5), idx_set.5, 5),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(opt $$E, opt $$F, opt $$G, opt $$H, opt $$I, opt $$J): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		absent(idx.1) \/ mzn_array_access_valid(deopt(idx.1), idx_set.1, 1),
		absent(idx.2) \/ mzn_array_access_valid(deopt(idx.2), idx_set.2, 2),
		absent(idx.3) \/ mzn_array_access_valid(deopt(idx.3), idx_set.3, 3),
		absent(idx.4) \/ mzn_array_access_valid(deopt(idx.4), idx_set.4, 4),
		absent(idx.5) \/ mzn_array_access_valid(deopt(idx.5), idx_set.5, 5),
		absent(idx.6) \/ mzn_array_access_valid(deopt(idx.6), idx_set.6, 6),
	]);

% Slice validity check
test mzn_array_access_valid(set of $$E: idx, set of $$E: idx_set, int: n) =
	if card(idx) = 1 then
		mzn_array_access_valid(min(idx), idx_set, n)
	else
		mzn_assert_warn(
			idx subset idx_set,
			if n > 0 then
				"array slice out of bounds, index set \(n) is \(
					idx_set
				), but given range is \(idx)"
			else
				"array slice out of bounds, index set is \(
					idx_set
				), but given range is \(idx)"
			endif,
		)
	endif;
test mzn_array_access_valid(array [$$E] of any $T: x, set of $$E: idx) =
	mzn_array_access_valid(idx, index_set(x), 0);
test mzn_array_access_valid(array [$$E] of any $T: x, tuple(set of $$E): idx) =
	mzn_array_access_valid(x, idx.1);
test mzn_array_access_valid(
	array [$$E, $$F] of any $T: x,
	tuple(set of $$E, set of $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G, set of $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
		mzn_array_access_valid(idx.5, idx_set.5, 5),
	]);
test mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(
		set of $$E,
		set of $$F,
		set of $$G,
		set of $$H,
		set of $$I,
		set of $$J,
	): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		mzn_array_access_valid(idx.1, idx_set.1, 1),
		mzn_array_access_valid(idx.2, idx_set.2, 2),
		mzn_array_access_valid(idx.3, idx_set.3, 3),
		mzn_array_access_valid(idx.4, idx_set.4, 4),
		mzn_array_access_valid(idx.5, idx_set.5, 5),
		mzn_array_access_valid(idx.6, idx_set.6, 6),
	]);

% Var index validity check
predicate mzn_array_access_valid(array [$E] of any $T: x, var $E: idx)
	:: mzn_unreachable;
predicate mzn_array_access_valid(array [$$E] of any $T: x, var $$E: idx) =
	idx in index_set(x);
predicate mzn_array_access_valid(
	array [$$E] of any $T: x,
	tuple(var $$E): idx,
) = mzn_array_access_valid(x, idx.1);
predicate mzn_array_access_valid(
	array [$$E, $$F] of any $T: x,
	tuple(var $$E, var $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([idx.1 in idx_set.1, idx.2 in idx_set.2]);
predicate mzn_array_access_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var $$E, var $$F, var $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([idx.1 in idx_set.1, idx.2 in idx_set.2, idx.3 in idx_set.3]);
predicate mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		idx.1 in idx_set.1,
		idx.2 in idx_set.2,
		idx.3 in idx_set.3,
		idx.4 in idx_set.4,
	]);
predicate mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		idx.1 in idx_set.1,
		idx.2 in idx_set.2,
		idx.3 in idx_set.3,
		idx.4 in idx_set.4,
		idx.5 in idx_set.5,
	]);
predicate mzn_array_access_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I, var $$J): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		idx.1 in idx_set.1,
		idx.2 in idx_set.2,
		idx.3 in idx_set.3,
		idx.4 in idx_set.4,
		idx.5 in idx_set.5,
		idx.6 in idx_set.6,
	]);

% Var index bounds optimisation
test mzn_array_access_known_valid(array [$E] of any $T: x, var $F: idx)
	:: mzn_unreachable;
test mzn_array_access_known_valid(array [$$E] of any $T: x, var $$E: idx) =
	has_bounds(idx) /\ lb(idx)..ub(idx) subset index_set(x);
test mzn_array_access_known_valid(
	array [$$E] of any $T: x,
	tuple(var $$E): idx,
) = mzn_array_access_known_valid(x, idx.1);
test mzn_array_access_known_valid(
	array [$$E, $$F] of any $T: x,
	tuple(var $$E, var $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var $$E, var $$F, var $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
		has_bounds(idx.5) /\ lb(idx.5)..ub(idx.5) subset idx_set.5,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(var $$E, var $$F, var $$G, var $$H, var $$I, var $$J): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
		has_bounds(idx.5) /\ lb(idx.5)..ub(idx.5) subset idx_set.5,
		has_bounds(idx.6) /\ lb(idx.6)..ub(idx.6) subset idx_set.6,
	]);

test mzn_array_access_known_valid(array [$$E] of any $T: x, var opt $$E: idx) =
	has_bounds(idx) /\ lb(idx)..ub(idx) subset index_set(x);
test mzn_array_access_known_valid(
	array [$$E] of any $T: x,
	tuple(var opt $$E): idx,
) = mzn_array_access_known_valid(x, idx.1);
test mzn_array_access_known_valid(
	array [$$E, $$F] of any $T: x,
	tuple(var opt $$E, var opt $$F): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		is_fixed(absent(idx.1)) /\ fix(absent(idx.1)) \/
			has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		is_fixed(absent(idx.2)) /\ fix(absent(idx.2)) \/
			has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		is_fixed(absent(idx.1)) /\ fix(absent(idx.1)) \/
			has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		is_fixed(absent(idx.2)) /\ fix(absent(idx.2)) \/
			has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		is_fixed(absent(idx.3)) /\ fix(absent(idx.3)) \/
			has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(var opt $$E, var opt $$F, var opt $$G, var opt $$H): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		is_fixed(absent(idx.1)) /\ fix(absent(idx.1)) \/
			has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		is_fixed(absent(idx.2)) /\ fix(absent(idx.2)) \/
			has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		is_fixed(absent(idx.3)) /\ fix(absent(idx.3)) \/
			has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		is_fixed(absent(idx.4)) /\ fix(absent(idx.4)) \/
			has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
	): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		is_fixed(absent(idx.1)) /\ fix(absent(idx.1)) \/
			has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		is_fixed(absent(idx.2)) /\ fix(absent(idx.2)) \/
			has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		is_fixed(absent(idx.3)) /\ fix(absent(idx.3)) \/
			has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		is_fixed(absent(idx.4)) /\ fix(absent(idx.4)) \/
			has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
		is_fixed(absent(idx.5)) /\ fix(absent(idx.5)) \/
			has_bounds(idx.5) /\ lb(idx.5)..ub(idx.5) subset idx_set.5,
	]);
test mzn_array_access_known_valid(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(
		var opt $$E,
		var opt $$F,
		var opt $$G,
		var opt $$H,
		var opt $$I,
		var opt $$J,
	): idx,
) =
	let {
		any: idx_set = index_sets(x);
	} in forall([
		is_fixed(absent(idx.1)) /\ fix(absent(idx.1)) \/
			has_bounds(idx.1) /\ lb(idx.1)..ub(idx.1) subset idx_set.1,
		is_fixed(absent(idx.2)) /\ fix(absent(idx.2)) \/
			has_bounds(idx.2) /\ lb(idx.2)..ub(idx.2) subset idx_set.2,
		is_fixed(absent(idx.3)) /\ fix(absent(idx.3)) \/
			has_bounds(idx.3) /\ lb(idx.3)..ub(idx.3) subset idx_set.3,
		is_fixed(absent(idx.4)) /\ fix(absent(idx.4)) \/
			has_bounds(idx.4) /\ lb(idx.4)..ub(idx.4) subset idx_set.4,
		is_fixed(absent(idx.5)) /\ fix(absent(idx.5)) \/
			has_bounds(idx.5) /\ lb(idx.5)..ub(idx.5) subset idx_set.5,
		is_fixed(absent(idx.6)) /\ fix(absent(idx.6)) \/
			has_bounds(idx.6) /\ lb(idx.6)..ub(idx.6) subset idx_set.6,
	]);

% Array slicing
function array [int] of any $T: mzn_slice_internal(
	array [$X] of any $T: x,
	array [int] of set of int: i,
);

function array [int] of any $T: '[]'(array [$$E] of any $T: x, set of $$E: i)
	:: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(mzn_erase_index_sets(x), [erase_enum(i)]);
function array [int] of any $T: '[]'(
	array [$$E] of any $T: x,
	tuple(set of $$E): i,
) :: mzn_inline_call_by_name = '[]'(x, i.1);

function array [int] of any $T: '[]'(
	array [$$E, $$F] of any $T: x,
	tuple(set of $$E, set of $$F): i,
) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(
		mzn_erase_index_sets(x),
		[erase_enum(i.1), erase_enum(i.2)],
	);

function array [int] of any $T: '[]'(
	array [$$E, $$F, $$G] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G): i,
) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(
		mzn_erase_index_sets(x),
		[erase_enum(i.1), erase_enum(i.2), erase_enum(i.3)],
	);

function array [int] of any $T: '[]'(
	array [$$E, $$F, $$G, $$H] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G, set of $$H): i,
) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(
		mzn_erase_index_sets(x),
		[erase_enum(i.1), erase_enum(i.2), erase_enum(i.3), erase_enum(i.4)],
	);

function array [int] of any $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I] of any $T: x,
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): i,
) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(
		mzn_erase_index_sets(x),
		[
			erase_enum(i.1),
			erase_enum(i.2),
			erase_enum(i.3),
			erase_enum(i.4),
			erase_enum(i.5),
		],
	);

function array [int] of any $T: '[]'(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x,
	tuple(
		set of $$E,
		set of $$F,
		set of $$G,
		set of $$H,
		set of $$I,
		set of $$J,
	): i,
) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, i);
	} in mzn_slice_internal(
		mzn_erase_index_sets(x),
		[
			erase_enum(i.1),
			erase_enum(i.2),
			erase_enum(i.3),
			erase_enum(i.4),
			erase_enum(i.5),
			erase_enum(i.6),
		],
	);

%-----------------------------------------------------------------------------%
%
% Internal functions for implementing div, mod etc

function set of int: compute_div_bounds(var int: x, var int: y);
function set of int: compute_mod_bounds(var int: x, var int: y);
function set of float: compute_float_div_bounds(var float: x, var float: y);
function set of int: compute_pow_bounds(var int: x, var int: y);

function var int: div_t(var int: x, var int: y) :: promise_total =
	let {
		var compute_div_bounds(x, y): z :: is_defined_var;
		constraint y != 0;
		constraint int_div(x, y, z) :: defines_var(z);
	} in z;

function var int: div_mt(var int: x, var int: y) :: promise_total =
	let {
		var dom(y) diff {0} union {1}: yy = if y = 0 then 1 else y endif;
	} in div_t(x, yy);

function var float: fldiv_t(var float: x, float: y) :: promise_total =
	x * (1.0 / y);

function var float: fldiv_t(var float: x, var float: y) :: promise_total =
	let {
		var compute_float_div_bounds(x, y): z :: is_defined_var;
		constraint if lb(y) <= 0 /\ ub(y) >= 0 then y != 0.0 endif;
		constraint float_div(x, y, z) :: defines_var(z);
	} in z;

function var float: fldiv_mt(var float: x, var float: y) :: promise_total =
	let {
		var lb(y)..ub(y) diff {0.0} union {1.0}: yy =
			if y = 0.0 then 1.0 else y endif;
	} in fldiv_t(x, yy);

function var int: mod_t(var int: x, var int: y) :: promise_total =
	let {
		var compute_mod_bounds(x, y): z;
		constraint y != 0;
		constraint int_mod(x, y, z);
	} in z;

function var int: mod_mt(var int: x, var int: y) :: promise_total =
	let {
		var {1} union dom(y): yy = if y = 0 then 1 else y endif;
	} in mod_t(x, yy);

function var float: sqrt_t(var float: x) :: promise_total =
	let {
		float: sqrt_lb = if lb(x) < 0.0 then 0.0 else sqrt(lb(x)) endif;
		float: sqrt_ub =
			if ub(x) < 0.0 then
				1.0
			elseif ub(x) = infinity then
				infinity
			else
				sqrt(ub(x))
			endif;
		var sqrt_lb..sqrt_ub: r;
		constraint float_sqrt(x, r);
	} in r;

function var float: sqrt_mt(var float: x) :: promise_total =
	let {
		float: sqrt_lb = if lb(x) < 0.0 then 0.0 else sqrt(lb(x)) endif;
		float: sqrt_ub =
			if ub(x) < 0.0 then
				1.0
			elseif ub(x) = infinity then
				infinity
			else
				sqrt(ub(x))
			endif;
		var sqrt_lb..sqrt_ub: r;
		var float: xx;
		constraint x < 0.0 -> xx = 1.0;
		constraint x < 0.0 \/ xx = x;
		constraint float_sqrt(xx, r);
	} in r;

function var int: product_rec(array [int] of var int: x) =
	if length(x) = 0 then
		1
	elseif length(x) = 1 then
		x[min(index_set(x))]
	else
		let {
			array [int] of var int: xx = array1d(x);
			array [index_set(xx)] of var int: y;
			constraint y[1] = xx[1];
			constraint forall (i in 2..length(y)) (y[i] = y[i - 1] * xx[i]);
		} in y[length(y)]
	endif;

function var float: product_rec(array [int] of var float: x) =
	if length(x) = 0 then
		1.0
	elseif length(x) = 1 then
		x[min(index_set(x))]
	else
		let {
			array [int] of var float: xx = array1d(x);
			array [index_set(xx)] of var float: y;
			constraint y[1] = xx[1];
			constraint forall (i in 2..length(y)) (y[i] = y[i - 1] * xx[i]);
		} in y[length(y)]
	endif;

function var int: max_t(array [int] of var int: x) :: promise_total =
	if length(x) = 0 then
		0
	elseif length(x) = 1 then
		x[min(index_set(x))]
	elseif length(x) = 2 then
		max(x[1], x[2])
	else
		let {
			var lb_array(x)..ub_array(x): m :: is_defined_var;
			constraint array_int_maximum(m, x) :: defines_var(m);
		} in m
	endif;

function var int: min_t(array [int] of var int: x) :: promise_total =
	if length(x) = 0 then
		0
	elseif length(x) = 1 then
		x[1]
	elseif length(x) = 2 then
		min(x[1], x[2])
	else
		let {
			var lb_array(x)..ub_array(x): m :: is_defined_var;
			constraint array_int_minimum(m, x) :: defines_var(m);
		} in m
	endif;

function var float: max_t(array [int] of var float: x) :: promise_total =
	if length(x) = 0 then
		0.0
	elseif length(x) = 1 then
		x[min(index_set(x))]
	elseif length(x) = 2 then
		max(x[1], x[2])
	else
		let {
			var lb_array(x)..ub_array(x): m :: is_defined_var;
			constraint array_float_maximum(m, x) :: defines_var(m);
		} in m
	endif;

function var float: min_t(array [int] of var float: x) :: promise_total =
	if length(x) = 0 then
		0.0
	elseif length(x) = 1 then
		x[1]
	elseif length(x) = 2 then
		min(x[1], x[2])
	else
		let {
			var lb_array(x)..ub_array(x): m :: is_defined_var;
			constraint array_float_minimum(m, x) :: defines_var(m);
		} in m
	endif;

function var int: pow_fixed_t(var int: x, int: y) :: promise_total =
	let {
		constraint y < 0 -> x != 0;
		var int: r :: is_defined_var;
		constraint int_pow_fixed(x, y, r) :: defines_var(r);
	} in r;

function var int: pow_fixed_mt(var int: x, int: y) :: promise_total =
	let {
		constraint
			assert(
				y < 0 /\ 0 in dom(x),
				"pow_fixed_mt called on variant guaranteed to be total",
			);
		var dom(x) diff {0}: nx = if x = 0 then 1 else x endif;
	} in pow_fixed_t(nx, y);

function var int: pow_t(var int: x, var int: y) :: promise_total =
	let {
		constraint y < 0 -> x != 0;
		var int: r :: is_defined_var;
		constraint int_pow(x, y, r) :: defines_var(r);
	} in r;

function var int: pow_mt(var int: x, var int: y) :: promise_total =
	let {
		constraint
			assert(
				lb(y) < 0 /\ 0 in dom(x),
				"pow_mt called on variant guaranteed to be total",
			);
		var dom(x): nx = if x = 0 /\ y < 0 then 1 else x endif;
	} in pow_t(nx, y);

/* These predicates are used to intercept symmetry_breaking_constraint and
 * redundant_constraint calls in user models, so that they can be ignored
 * if the corresponding options have been set.
 */

predicate mzn_symmetry_breaking_constraint(var bool: b);
predicate mzn_redundant_constraint(var bool: b);

/* Annotations used for bookkeeping by the compiler */

annotation mzn_was_undefined;

/* Predicate used in the compiler for translating infinite domains into constraints */
predicate mzn_set_in_internal(var int: x, set of int: s) =
	if card(s) = 0 \/ min(s) != -infinity /\ max(s) != infinity then
		set_in(x, s)
	else
		let {
			array [int] of int: ranges_ = set_to_ranges(s);
			array [int, 1..2] of int: ranges =
				array2d(1..length(ranges_) div 2, 1..2, ranges_);
		} in exists (i in index_set_1of2(ranges)) (
			if ranges[i, 1] = -infinity then
				x <= ranges[i, 2]
			elseif ranges[i, 2] = infinity then
				x >= ranges[i, 1]
			elseif ranges[i, 1] = ranges[i, 2] then
				x = ranges[i, 1]
			else
				x in ranges[i, 1]..ranges[i, 2]
			endif
		)
	endif;

/* Predicate used in the compiler for translating infinite domains into constraints */
predicate mzn_set_in_internal(var opt float: x, set of float: s) =
	let {
		array [int] of float: ranges = set_to_ranges(s);
		int: len = length(ranges);
		int: from = if ranges[1] = -infinity then 3 else 1 endif;
		int: to = if ranges[len] = infinity then len - 2 else len endif;
	} in if ranges[1] = -infinity then x <= ranges[2] else false endif \/
		if from..to subset index_set(ranges) then
			float_dom(x, array1d(ranges[from..to]))
		else
			false
		endif \/
		if ranges[len] = infinity then x >= ranges[len - 1] else false endif;

/* Functions used for the type erasure of enumerated types */

test mzn_set_is_contiguous(set of int: X) =
	card(X) = 0 \/ card(X) = max(X) - min(X) + 1;

function int: mzn_min_or_0(set of int: X) =
	if card(X) = 0 then
		0
	elseif mzn_set_is_contiguous(X) then
		min(X)
	else
		1
	endif;

predicate lex_less_std_decomposition(
	array [int] of var $T: x :: promise_ctx_antitone,
	array [int] of var $T: y :: promise_ctx_monotone,
) :: promise_total =
	if length(y) = 0 then
		false
	elseif length(x) = 0 then
		true
	else
		let {
			int: lx = min(index_set(x));
			int: ux = max(index_set(x));
			int: ly = min(index_set(y));
			int: uy = max(index_set(y));
			int: size = min(ux - lx, uy - ly);
			array [0..size + 1] of var bool: b;
			constraint b[size + 1] = (ux - lx < uy - ly);
			constraint
				forall (i in 0..size) (
					b[i] =
						(
							x[lx + i] <= y[ly + i] /\
								(x[lx + i] < y[ly + i] \/ b[i + 1])
						)
				);
		} in b[0]
	endif;

predicate lex_lesseq_std_decomposition(
	array [int] of var $T: x :: promise_ctx_antitone,
	array [int] of var $T: y :: promise_ctx_monotone,
) :: promise_total =
	if length(x) = 0 then
		true
	elseif length(y) = 0 then
		length(x) = 0
	else
		let {
			int: lx = min(index_set(x));
			int: ux = max(index_set(x));
			int: ly = min(index_set(y));
			int: uy = max(index_set(y));
			int: size = min(ux - lx, uy - ly);
			% b[i] is true if the lexicographical order holds from position i on.
			array [0..size + 1] of var bool: b;
			constraint b[size + 1] = (ux - lx <= uy - ly);
			constraint
				forall (i in 0..size) (
					b[i] =
						(
							x[lx + i] <= y[ly + i] /\
								(x[lx + i] < y[ly + i] \/ b[i + 1])
						)
				);
		} in b[0]
	endif;

function set of int: set_min_to_max(set of int: x) =
	if x = {} then {1 | _ in 1..0} else min(x)..max(x) endif;

/* Used to generate fresh values */
function int: mzn_increment_counter(string: key);

function bool: mzn_add_warning(string: msg);
function bool: mzn_assert_warn(bool: b, string: msg)
	:: mzn_inline_call_by_name =
	let {
		bool: condition = b;
		constraint if condition then true else mzn_add_warning(msg) endif;
	} in condition;

function string: mzn_show_tuple_access(string: x, int: i) = x ++ ".\(i)";
function string: mzn_show_record_access(string: x, string: f) = x ++ "." ++ f;

function string: mzn_show_array_access(string: x, $$E: i) =
	x ++ "[" ++ show(i) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E): i) =
	x ++ "[" ++ show(i.1) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F): i) =
	x ++ "[" ++ show(i.1) ++ "," ++ show(i.2) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F, $$G): i) =
	x ++ "[" ++ show(i.1) ++ "," ++ show(i.2) ++ "," ++ show(i.3) ++ "]";
function string: mzn_show_array_access(
	string: x,
	tuple($$E, $$F, $$G, $$H): i,
) =
	x ++
		"[" ++
		show(i.1) ++
		"," ++
		show(i.2) ++
		"," ++
		show(i.3) ++
		"," ++
		show(i.4) ++
		"]";
function string: mzn_show_array_access(
	string: x,
	tuple($$E, $$F, $$G, $$H, $$I): i,
) =
	x ++
		"[" ++
		show(i.1) ++
		"," ++
		show(i.2) ++
		"," ++
		show(i.3) ++
		"," ++
		show(i.4) ++
		"," ++
		show(i.5) ++
		"]";
function string: mzn_show_array_access(
	string: x,
	tuple($$E, $$F, $$G, $$H, $$I, $$J): i,
) =
	x ++
		"[" ++
		show(i.1) ++
		"," ++
		show(i.2) ++
		"," ++
		show(i.3) ++
		"," ++
		show(i.4) ++
		"," ++
		show(i.5) ++
		"," ++
		show(i.6) ++
		"]";

test mzn_check_index_set(string: name, set of $$E: actual, set of $$E: expected)
	:: mzn_inline_call_by_name =
	mzn_check_index_set(name, 1, 1, actual, expected);
test mzn_check_index_set(
	string: name,
	int: d,
	int: dims,
	set of $$E: actual,
	set of $$E: expected,
) :: mzn_inline_call_by_name =
	assert(
		actual = expected,
		"Index set " ++
			if dims > 1 then "\(d) " endif ++
			"of '" ++
			name ++
			"' is \(expected), but is assigned an array with " ++
			if dims > 1 then "the corresponding " endif ++
			"index set \(
				actual
			). You may need to coerce the index sets using the array\(
				dims
			)d function.",
	);

annotation mzn_inline;
annotation mzn_inline_call_by_name;
annotation mzn_unreachable;

function array [int] of any $T: mzn_erase_index_sets(array [_] of any $T: x);
function array [int, int] of any $T: mzn_erase_index_sets(
	array [_, _] of any $T: x,
);
function array [int, int, int] of any $T: mzn_erase_index_sets(
	array [_, _, _] of any $T: x,
);
function array [int, int, int, int] of any $T: mzn_erase_index_sets(
	array [_, _, _, _] of any $T: x,
);
function array [int, int, int, int, int] of any $T: mzn_erase_index_sets(
	array [_, _, _, _, _] of any $T: x,
);
function array [int, int, int, int, int, int] of any $T: mzn_erase_index_sets(
	array [_, _, _, _, _, _] of any $T: x,
);

function var opt $T: mzn_construct_partial(tuple(var bool, var opt $T): x)
	:: mzn_internal_representation;
function var $T: mzn_construct_partial(tuple(var bool, var $T): x)
	:: mzn_internal_representation;
function opt $T: mzn_construct_partial(tuple(bool, opt $T): x)
	:: mzn_internal_representation;
function $T: mzn_construct_partial(tuple(bool, $T): x)
	:: mzn_internal_representation;
function array [$X] of var opt $T: mzn_construct_partial(
	tuple(var bool, array [$X] of var opt $T): x,
) :: mzn_internal_representation;
function array [$X] of var $T: mzn_construct_partial(
	tuple(var bool, array [$X] of var $T): x,
) :: mzn_internal_representation;
function array [$X] of opt $T: mzn_construct_partial(
	tuple(bool, array [$X] of opt $T): x,
) :: mzn_internal_representation;
function array [$X] of $T: mzn_construct_partial(
	tuple(bool, array [$X] of $T): x,
) :: mzn_internal_representation;

function tuple(var bool, var opt $T): mzn_destruct_partial(var opt $T: x)
	:: mzn_internal_representation;
function tuple(var bool, var $T): mzn_destruct_partial(var $T: x)
	:: mzn_internal_representation;
function tuple(bool, opt $T): mzn_destruct_partial(opt $T: x)
	:: mzn_internal_representation;
function tuple(bool, $T): mzn_destruct_partial($T: x)
	:: mzn_internal_representation;
function tuple(var bool, array [$X] of var opt $T): mzn_destruct_partial(
	array [$X] of var opt $T: x,
) :: mzn_internal_representation;
function tuple(var bool, array [$X] of var $T): mzn_destruct_partial(
	array [$X] of var $T: x,
) :: mzn_internal_representation;
function tuple(bool, array [$X] of opt $T): mzn_destruct_partial(
	array [$X] of opt $T: x,
) :: mzn_internal_representation;
function tuple(bool, array [$X] of $T): mzn_destruct_partial(
	array [$X] of $T: x,
) :: mzn_internal_representation;
