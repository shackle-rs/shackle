% Temporary redefinitions which may be baked into the interpreter in the future.

include "arg_val.mzn";

% Conjunction

function bool: '/\'(bool: x, bool: y) :: mzn_inline_call_by_name =
	if x then if y then true else false endif else false endif;

function var bool: '/\'(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let {
			constraint x;
			constraint y;
		} in true
	else
		mzn_conjunction_t(x, y)
	endif;

function var bool: mzn_conjunction_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint array_bool_and([x, y], r);
	} in r;

% Disjunction

function bool: '\/'(bool: x, bool: y) :: mzn_inline_call_by_name =
	if x then true elseif y then true else false endif;

function var bool: '\/'(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_clause([x, y], []) } in true
	else
		mzn_disjunction_t(x, y)
	endif;

function var bool: mzn_disjunction_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_clause([x, y], [r]);
		constraint bool_clause([r], [x]);
		constraint bool_clause([r], [y]);
	} in r;

% Universal quantification

function bool: forall(array [$X] of bool: x) = length([x_i | x_i in x where not x_i]) = 0;
function bool: forall(array [int] of bool: x) = length([x_i | x_i in x where not x_i]) = 0;

function var bool: forall(array [$X] of var bool: x) = mzn_forall(array1d(x));
function var bool: forall(array [int] of var bool: x) = mzn_forall(x);

function var bool: mzn_forall(array [int] of var bool: x) =
	if mzn_in_root_context() then
		let { any: xx = [let { constraint x_i } in true | x_i in x] } in true
	else
		mzn_forall_t(x)
	endif;

function var bool: mzn_forall_t(
	array [int] of var bool: x,
)
	:: promise_total =
	let {
		var bool: r;
		constraint array_bool_and(x, r);
	} in r;

% Existential quantification

function bool: exists(array [$T] of bool: x) = length([x_i | x_i in x where x_i]) > 0;
function bool: exists(array [int] of bool: x) = length([x_i | x_i in x where x_i]) > 0;

function var bool: exists(array [$T] of var bool: x) = mzn_exists(array1d(x));
function var bool: exists(array [int] of var bool: x) = mzn_exists(x);

function var bool: mzn_exists(array [int] of var bool: x) = clause(x, []);

% Boolean clause

function bool: clause(array [$T] of bool: x, array [$U] of bool: y) = exists(x) \/ not forall(y);
function bool: clause(array [$T] of bool: x, array [$T] of bool: y) = exists(x) \/ not forall(y);

function var bool: clause(array [$T] of var bool: x, array [$U] of var bool: y) =
	mzn_clause(array1d(x), array1d(y));
function var bool: clause(array [$T] of var bool: x, array [$T] of var bool: y) =
	mzn_clause(array1d(x), array1d(y));

function var bool: mzn_clause(array [int] of var bool: x, array [int] of var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_clause(x, y) } in true
	else
		mzn_clause_t(x, y)
	endif;

function var bool: mzn_clause_t(array [int] of var bool: x, array [int] of var bool: y)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_clause(x, y ++ [r]);
		any: dcx = [let { constraint bool_clause([r], [x_i]) } in true | x_i in x];
		any: dcy = [let { constraint bool_clause([y_i, r], []) } in true | y_i in y];
	} in r;

% Boolean negation

function bool: 'not'(bool: x) :: mzn_inline_call_by_name = if x then false else true endif;

function var bool: 'not'(var bool: x) = mzn_not(x);

function var bool: mzn_not(
	var bool: x,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_not(x, r);
	} in r;

predicate bool_not(var bool: b) = mzn_not(b);

% If and only if

function bool: '<->'(bool: x, bool: y) = if x then y else not y endif;

function var bool: '<->'(var bool: x, var bool: y) = mzn_iff(x, y);

function var bool: mzn_iff(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_eq(x, y) } in true
	else
		mzn_iff_t(x, y)
	endif;

function var bool: mzn_iff_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_eq_reif(x, y, r);
	} in r;

% Implication

function bool: '->'(bool: x, bool: y) :: mzn_inline_call_by_name = if x then y endif;

function var bool: '->'(var bool: x, var bool: y) = mzn_implies(x, y);

function var bool: mzn_implies(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_clause([y], [x]) } in true
	else
		mzn_implies_t(x, y)
	endif;

function var bool: mzn_implies_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_clause([y], [x, r]);
		constraint bool_clause([r], [y]);
		constraint bool_clause([x, r], []);
	} in r;

function bool: '<-'(bool: x, bool: y) :: mzn_inline_call_by_name = if y then x endif;

function var bool: '<-'(var bool: x, var bool: y) = mzn_implies(y, x);

% Exclusive or

function bool: 'xor'(bool: x, bool: y) = if x then not y else y endif;

function var bool: 'xor'(var bool: x, var bool: y) = mzn_xor(x, y);

function var bool: mzn_xor(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_not(x, y) } in true
	else
		mzn_xor_t(x, y)
	endif;

function var bool: mzn_xor_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_xor(x, y, r);
	} in r;

predicate bool_xor(var bool: x, var bool: y) = mzn_xor(x, y);

% Xorall

function bool: xorall(array [$T] of bool: x) =
	mod_internal(length([x_i | x_i in x where x_i]), 2) = 1;
function bool: iffall(array [$T] of bool: x) = xorall([true] ++ array1d(x));

% Subset

function bool: 'subset'(set of $T: x, set of $T: y) = forall (x_i in x) (x_i in y);
function bool: 'subset'(
	set of int: x,
	set of int: y,
) =
	let {
		any: xr = mzn_set_to_ranges(x);
		any: yr = mzn_set_to_ranges(y);
	} in forall (x_i in xr) (exists (y_j in yr) (x_i.1 >= y_j.1 /\ x_i.2 <= y_j.2));
function bool: 'subset'(
	set of float: x,
	set of float: y,
) =
	let {
		any: xr = mzn_set_to_ranges(x);
		any: yr = mzn_set_to_ranges(y);
	} in forall (x_i in xr) (exists (y_j in yr) (x_i.1 >= y_j.1 /\ x_i.2 <= y_j.2));

function var bool: 'subset'(var set of $$E: x, var set of $$E: y) =
	mzn_subset(enum2int(x), enum2int(y));
function var bool: 'subset'(var set of $$E: x, set of $$E: y) =
	 mzn_subset(enum2int(x), enum2int(y));
function var bool: 'subset'(set of $$E: x, var set of $$E: y) =
	 mzn_subset(enum2int(x), enum2int(y));

function var bool: mzn_subset(var set of int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_subset(x, y) } in true
	else
		mzn_subset_t(x, y)
	endif;

function var bool: mzn_subset_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var bool: r;
		constraint set_subset_reif(x, y, r);
	} in r;

% Superset

test 'superset'(set of $T: x, set of $T: y) = y subset x;
predicate 'superset'(var set of $$E: x, var set of $$E: y) = y subset x;

% Set in for integers

function bool: 'in'($T: x, set of $T: y) = exists (y_i in y) (x = y_i);
function bool: 'in'($$E: x, set of $$E: y) = enum2int(x) in enum2int(y);
function bool: 'in'(int: x, set of int: y) =
	exists([a <= x /\ x <= b | (a, b) in mzn_set_to_ranges(y)]);

function var bool: 'in'($$E: x, var set of $$E: y) = mzn_in_int(enum2int(x), enum2int(y));
function var bool: 'in'(var $$E: x, var set of $$E: y) = mzn_in_int(enum2int(x), enum2int(y));
function var bool: 'in'(var $$E: x, set of $$E: y) = mzn_in_int(enum2int(x), enum2int(y));

function var bool: 'in'(int: x, var set of int: y) = mzn_in_int(x, y);
function var bool: 'in'(var int: x, var set of int: y) = mzn_in_int(x, y);
function var bool: 'in'(var int: x, set of int: y) = mzn_in_int(x, y);

function var bool: mzn_in_int(var int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_in(x, y) } in true
	else
		mzn_in_int_t(x, y)
	endif;

function var bool: mzn_in_int_t(
	var int: x,
	var set of int: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint set_in_reif(x, y, r);
	} in r;

% Set in for floats

function bool: 'in'(float: x, set of float: y) =
	exists([a <= x /\ x <= b | (a, b) in mzn_set_to_ranges(y)]);

function var bool: 'in'(var float: x, set of float: y) =
	exists([a <= x /\ x <= b | (a, b) in mzn_set_to_ranges(y)]);

% Equality for booleans

function var bool: '='(var bool: x, var bool: y) = x <-> y;
function var bool: '='(var bool: x, bool: y) = x <-> y;
function var bool: '='(bool: x, var bool: y) = x <-> y;

% Equality for integers

function var bool: '='(var int: x, var int: y) = mzn_int_eq(x, y);
function var bool: '='(var int: x, int: y) = mzn_int_eq(x, y);
function var bool: '='(int: x, var int: y) = mzn_int_eq(x, y);

function var bool: mzn_int_eq(var int: x, var int: y) =
	if mzn_in_root_context() then
		let { constraint int_eq(x, y) } in true
	else
		mzn_int_eq_t(x, y)
	endif;

function var bool: mzn_int_eq_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint int_eq_reif(x, y, r);
	} in r;

% Equality for floats

function var bool: '='(var float: x, var float: y) = mzn_float_eq(x, y);
function var bool: '='(var float: x, float: y) = mzn_float_eq(x, y);
function var bool: '='(float: x, var float: y) = mzn_float_eq(x, y);

function var bool: mzn_float_eq(var float: x, var float: y) =
	if mzn_in_root_context() then
		let { constraint float_eq(x, y) } in true
	else
		mzn_float_eq_t(x, y)
	endif;

function var bool: mzn_float_eq_t(
	var float: x,
	var float: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint float_eq_reif(x, y, r);
	} in r;

% Equality for sets of integers

function var bool: '='(var set of $$E: x, var set of $$E: y) = mzn_set_eq(enum2int(x), enum2int(y));
function var bool: '='(var set of $$E: x, set of $$E: y) = mzn_set_eq(enum2int(x), enum2int(y));
function var bool: '='(set of $$E: x, var set of $$E: y) = mzn_set_eq(enum2int(x), enum2int(y));

function var bool: '='(var set of int: x, var set of int: y) = mzn_set_eq(x, y);
function var bool: '='(var set of int: x, set of int: y) = mzn_set_eq(x, y);
function var bool: '='(set of int: x, var set of int: y) = mzn_set_eq(x, y);

function var bool: mzn_set_eq(var set of int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_eq(x, y) } in true
	else
		mzn_set_eq_t(x, y)
	endif;

function var bool: mzn_set_eq_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var bool: r;
		constraint set_eq_reif(x, y, r);
	} in r;

% Disequality of par types

function bool: '!='($T: x, $T: y) = not (x = y);

% Disequality for var types

function var bool: '!='(any $T: x, any $T: y) = not (x = y);

% Disequality for booleans

function var bool: '!='(var bool: x, var bool: y) = x xor y;
function var bool: '!='(var bool: x, bool: y) = x xor y;
function var bool: '!='(bool: x, var bool: y) = x xor y;

% Disequality for integers

function var bool: '!='(var $$E: x, var $$E: y) = mzn_int_ne(enum2int(x), enum2int(y));
function var bool: '!='(var $$E: x, $$E: y) = mzn_int_ne(enum2int(x), enum2int(y));
function var bool: '!='($$E: x, var $$E: y) = mzn_int_ne(enum2int(x), enum2int(y));

function var bool: '!='(var int: x, var int: y) = mzn_int_ne(x, y);
function var bool: '!='(var int: x, int: y) = mzn_int_ne(x, y);
function var bool: '!='(int: x, var int: y) = mzn_int_ne(x, y);

function var bool: mzn_int_ne(var int: x, var int: y) =
	if mzn_in_root_context() then
		let { constraint int_ne(x, y) } in true
	else
		mzn_int_ne_t(x, y)
	endif;

function var bool: mzn_int_ne_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint int_ne_reif(x, y, r);
	} in r;

% Disequality for floats

function var bool: '!='(var float: x, var float: y) = mzn_float_ne(x, y);
function var bool: '!='(var float: x, float: y) = mzn_float_ne(x, y);
function var bool: '!='(float: x, var float: y) = mzn_float_ne(x, y);

function var bool: mzn_float_ne(var float: x, var float: y) =
	if mzn_in_root_context() then
		let { constraint float_ne(x, y) } in true
	else
		mzn_float_ne_t(x, y)
	endif;

function var bool: mzn_float_ne_t(
	var float: x,
	var float: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint float_ne_reif(x, y, r);
	} in r;

% Disequality for sets of integers

function var bool: '!='(var set of $$E: x, var set of $$E: y) =
	 mzn_set_ne(enum2int(x), enum2int(y));
function var bool: '!='(var set of $$E: x, set of $$E: y) = mzn_set_ne(enum2int(x), enum2int(y));
function var bool: '!='(set of $$E: x, var set of $$E: y) = mzn_set_ne(enum2int(x), enum2int(y));

function var bool: '!='(var set of int: x, var set of int: y) = mzn_set_ne(x, y);
function var bool: '!='(var set of int: x, set of int: y) = mzn_set_ne(x, y);
function var bool: '!='(set of int: x, var set of int: y) = mzn_set_ne(x, y);

function var bool: mzn_set_ne(var set of int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_ne(x, y) } in true
	else
		mzn_set_ne_t(x, y)
	endif;

function var bool: mzn_set_ne_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var bool: r;
		constraint set_ne_reif(x, y, r);
	} in r;

% Less than for booleans

function var bool: '<'(var bool: x, var bool: y) = mzn_bool_lt(x, y);
function var bool: '<'(var bool: x, bool: y) = if y then not x else false endif;
function var bool: '<'(bool: x, var bool: y) = if x then false else y endif;

function var bool: mzn_bool_lt(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_lt(x, y) } in true
	else
		mzn_bool_lt_t(x, y)
	endif;

function var bool: mzn_bool_lt_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_lt_reif(x, y, r);
	} in r;

% Less than for integers

function var bool: '<'(var $$E: x, var $$E: y) = mzn_int_lt(enum2int(x), enum2int(y));
function var bool: '<'(var $$E: x, $$E: y) = mzn_int_lt(enum2int(x), enum2int(y));
function var bool: '<'($$E: x, var $$E: y) = mzn_int_lt(enum2int(x), enum2int(y));

function var bool: '<'(var int: x, var int: y) = mzn_int_lt(x, y);
function var bool: '<'(var int: x, int: y) = mzn_int_lt(x, y);
function var bool: '<'(int: x, var int: y) = mzn_int_lt(x, y);

function var bool: mzn_int_lt(var int: x, var int: y) =
	if mzn_in_root_context() then
		let { constraint int_lt(x, y) } in true
	else
		mzn_int_lt_t(x, y)
	endif;

function var bool: mzn_int_lt_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint int_lt_reif(x, y, r);
	} in r;

% Less than for floats

function var bool: '<'(var float: x, var float: y) = mzn_float_lt(x, y);
function var bool: '<'(var float: x, float: y) = mzn_float_lt(x, y);
function var bool: '<'(float: x, var float: y) = mzn_float_lt(x, y);

function var bool: mzn_float_lt(var float: x, var float: y) =
	if mzn_in_root_context() then
		let { constraint float_lt(x, y) } in true
	else
		mzn_float_lt_t(x, y)
	endif;

function var bool: mzn_float_lt_t(
	var float: x,
	var float: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint float_lt_reif(x, y, r);
	} in r;

% Less than for sets

function var bool: '<'(var set of $$E: x, var set of $$E: y) = mzn_set_lt(enum2int(x), enum2int(y));
function var bool: '<'(var set of $$E: x, set of $$E: y) = mzn_set_lt(enum2int(x), enum2int(y));
function var bool: '<'(set of $$E: x, var set of $$E: y) = mzn_set_lt(enum2int(x), enum2int(y));

function var bool: '<'(var set of int: x, var set of int: y) = mzn_set_lt(x, y);
function var bool: '<'(var set of int: x, set of int: y) = mzn_set_lt(x, y);
function var bool: '<'(set of int: x, var set of int: y) = mzn_set_lt(x, y);

function var bool: mzn_set_lt(var set of int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_lt(x, y) } in true
	else
		mzn_set_lt_t(x, y)
	endif;

function var bool: mzn_set_lt_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var bool: r;
		constraint set_lt_reif(x, y, r);
	} in r;

% Less than or equal to for par
test '<='($T: x, $T: y) = x < y \/ x = y;

% Less than or equal to for booleans

function var bool: '<='(var bool: x, var bool: y) = mzn_bool_le(x, y);
function var bool: '<='(var bool: x, bool: y) = mzn_bool_le(x, y);
function var bool: '<='(bool: x, var bool: y) = mzn_bool_le(x, y);

function var bool: mzn_bool_le(var bool: x, var bool: y) =
	if mzn_in_root_context() then
		let { constraint bool_le(x, y) } in true
	else
		mzn_bool_le_t(x, y)
	endif;

function var bool: mzn_bool_le_t(
	var bool: x,
	var bool: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_le_reif(x, y, r);
	} in r;

% Less than or equal to for integers

function var bool: '<='(var $$E: x, var $$E: y) = mzn_int_le(enum2int(x), enum2int(y));
function var bool: '<='(var $$E: x, $$E: y) = mzn_int_le(enum2int(x), enum2int(y));
function var bool: '<='($$E: x, var $$E: y) = mzn_int_le(enum2int(x), enum2int(y));

function var bool: '<='(var int: x, var int: y) = mzn_int_le(x, y);
function var bool: '<='(var int: x, int: y) = mzn_int_le(x, y);
function var bool: '<='(int: x, var int: y) = mzn_int_le(x, y);

function var bool: mzn_int_le(var int: x, var int: y) =
	if mzn_in_root_context() then
		let { constraint int_le(x, y) } in true
	else
		mzn_int_le_t(x, y)
	endif;

function var bool: mzn_int_le_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint int_le_reif(x, y, r);
	} in r;

% Less than or equal to for floats

function var bool: '<='(var float: x, var float: y) = mzn_float_le(x, y);
function var bool: '<='(var float: x, float: y) = mzn_float_le(x, y);
function var bool: '<='(float: x, var float: y) = mzn_float_le(x, y);

function var bool: mzn_float_le(var float: x, var float: y) =
	if mzn_in_root_context() then
		let { constraint float_le(x, y) } in true
	else
		mzn_float_le_t(x, y)
	endif;

function var bool: mzn_float_le_t(
	var float: x,
	var float: y,
)
	:: promise_total =
	let {
		var bool: r;
		constraint float_le_reif(x, y, r);
	} in r;

% Less than or equal to for sets

function var bool: '<='(var set of $$E: x, var set of $$E: y) =
	 mzn_set_le(enum2int(x), enum2int(y));
function var bool: '<='(var set of $$E: x, set of $$E: y) = mzn_set_le(enum2int(x), enum2int(y));
function var bool: '<='(set of $$E: x, var set of $$E: y) = mzn_set_le(enum2int(x), enum2int(y));

function var bool: '<='(var set of int: x, var set of int: y) = mzn_set_le(x, y);
function var bool: '<='(var set of int: x, set of int: y) = mzn_set_le(x, y);
function var bool: '<='(set of int: x, var set of int: y) = mzn_set_le(x, y);

function var bool: mzn_set_le(var set of int: x, var set of int: y) =
	if mzn_in_root_context() then
		let { constraint set_le(x, y) } in true
	else
		mzn_set_le_t(x, y)
	endif;

function var bool: mzn_set_le_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var bool: r;
		constraint set_le_reif(x, y, r);
	} in r;

% Weak comparisons on optional values: true iff either side is absent, or
% both occur and the underlying comparison holds.

function bool: '<'(opt $T: x, opt $T: y) = absent(x) \/ absent(y) \/ deopt(x) < deopt(y);
function var bool: '<'(var opt $T: x, var opt $T: y) =
	 absent(x) \/ absent(y) \/ deopt(x) < deopt(y);

function bool: '<='(opt $T: x, opt $T: y) = absent(x) \/ absent(y) \/ deopt(x) <= deopt(y);
function var bool: '<='(var opt $T: x, var opt $T: y) =
	absent(x) \/ absent(y) \/ deopt(x) <= deopt(y);

function bool: '>'(opt $T: x, opt $T: y) = absent(x) \/ absent(y) \/ deopt(x) > deopt(y);
function var bool: '>'(var opt $T: x, var opt $T: y) =
	 absent(x) \/ absent(y) \/ deopt(x) > deopt(y);

function bool: '>='(opt $T: x, opt $T: y) = absent(x) \/ absent(y) \/ deopt(x) >= deopt(y);
function var bool: '>='(var opt $T: x, var opt $T: y) =
	absent(x) \/ absent(y) \/ deopt(x) >= deopt(y);

% Greater than and greater than or equal to are defined in terms of less than and less than or equal to

function var bool: '>'(var $T: x, var $T: y) = y < x;
function var bool: '>'($T: x, var $T: y) = y < x;
function var bool: '>'(var $T: x, $T: y) = y < x;
function bool: '>'($T: x, $T: y) = y < x;

function var bool: '>='(var $T: x, var $T: y) = y <= x;
function var bool: '>='($T: x, var $T: y) = y <= x;
function var bool: '>='(var $T: x, $T: y) = y <= x;
function bool: '>='($T: x, $T: y) = y <= x;

% Addition of integers
function var int: '+'(var int: x, var int: y) :: promise_total = sum([x, y]);

function var int: sum(array [$T] of var int: x) = mzn_sum_t(array1d(x));
function var int: sum(array [int] of var int: x) = mzn_sum_t(array1d(x));

function var int: mzn_sum_t(
	array [int] of var int: x,
)
	:: promise_total =
	let {
		var int: r;
		constraint int_lin_eq([1 | x_i in x] ++ [-1], array1d(x) ++ [r], 0);
	} in r;

% Par sum rewritten using addition

function int: sum(array [$T] of int: x) = sum(array1d(x));

function int: sum(array [int] of int: x) =
	if length(x) = 0 then
		0
	elseif length(x) = 1 then
		mzn_element_internal(array1d(x), 1)
	else
		let {
			any: xx = array1d(x);
		} in sum(
			[mzn_element_internal(xx, 1) + mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

% Addition of floats

function var float: '+'(var float: x, var float: y) :: promise_total = sum([x, y]);

function var float: sum(array [$T] of var float: x) = mzn_sum_float_t(array1d(x));
function var float: sum(array [int] of var float: x) = mzn_sum_float_t(array1d(x));

function var float: mzn_sum_float_t(array [int] of var float: x) :: promise_total =
	let {
		var float: r;
		constraint float_lin_eq([1.0 | x_i in x] ++ [-1.0], array1d(x) ++ [r], 0.0);
	} in r;

function float: sum(array [$T] of float: x) = sum(array1d(x));

function float: sum(array [int] of float: x) =
	if length(x) = 0 then
		0.0
	elseif length(x) = 1 then
		array1d(x)[1]
	else
		let {
			any: xx = array1d(x);
		} in sum(
			[mzn_element_internal(xx, 1) + mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

% Negation of integers

function var int: '-'(var int: x) = mzn_neg_t(x);

function var int: mzn_neg_t(
	var int: x,
)
	:: promise_total =
	let {
		var int: r;
		constraint int_lin_eq([-1, -1], [x, r], 0);
	} in r;

% Negation of floats

function var float: '-'(var float: x) = mzn_neg_float_t(x);

function var float: mzn_neg_float_t(
	var float: x,
)
	:: promise_total =
	let {
		var float: r;
		constraint float_lin_eq([-1.0, -1.0], [x, r], 0.0);
	} in r;

% Subtraction of integers

function var int: '-'(var int: x, var int: y) = mzn_sub_t(x, y);

function var int: mzn_sub_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var int: r;
		constraint int_lin_eq([1, -1, -1], [x, y, r], 0);
	} in r;

% Subtraction of floats

function var float: '-'(var float: x, var float: y) = mzn_sub_float_t(x, y);

function var float: mzn_sub_float_t(var float: x, var float: y) :: promise_total =
	let {
		var float: r;
		constraint float_lin_eq([1.0, -1.0, -1.0], [x, y, r], 0.0);
	} in r;

% Product of integers

function var int: '*'(var int: x, var int: y) = mzn_mul_t(x, y);

function var int: mzn_mul_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var int: r;
		constraint int_times(x, y, r);
	} in r;

function int: product(array [$T] of int: x) = product(array1d(x));

function int: product(array [int] of int: x) =
	if length(x) = 0 then
		1
	elseif length(x) = 1 then
		mzn_element_internal(array1d(x), 1)
	else
		let {
			any: xx = array1d(x);
		} in product(
			[mzn_element_internal(xx, 1) * mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

% Product of floats

function var float: '*'(var float: x, var float: y) = mzn_mul_float_t(x, y);

function var float: mzn_mul_float_t(var float: x, var float: y) :: promise_total =
	let {
		var float: r;
		constraint float_times(x, y, r);
	} in r;

function float: product(array [$T] of float: x) = product(array1d(x));

function float: product(array [int] of float: x) =
	if length(x) = 0 then
		1.0
	elseif length(x) = 1 then
		array1d(x)[1]
	else
		let {
			any: xx = array1d(x);
		} in product(
			[mzn_element_internal(xx, 1) * mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

% Modulus of integers

function int: mod_internal(int: x, int: y) = x - y * div_internal_int(x, y);

% Exponentiation of integers

function var int: '^'(var int: x, var int: y) = mzn_pow_t(x, y);

function var int: mzn_pow_t(
	var int: x,
	var int: y,
)
	:: promise_total =
	let {
		var int: r;
		constraint int_pow(x, y, r);
	} in r;

% Exponentiation of floats

function var float: '^'(var float: x, var float: y) = mzn_pow_float_t(x, y);

function var float: mzn_pow_float_t(var float: x, var float: y) :: promise_total =
	let {
		var float: r;
		constraint float_pow(x, y, r);
	} in r;

% Absolute value

function int: abs(int: x) = if x < 0 then -x else x endif;
function float: abs(float: x) = if x < 0.0 then -x else x endif;

% Set union

function set of $T: 'union'(set of $T: x, set of $T: y) = {v | v in set2array(x) ++ set2array(y)};
function set of int: 'union'(set of int: x, set of int: y) = union_internal_int(x, y);
function set of int: union_internal_int(set of int: x, set of int: y);
function set of float: 'union'(set of float: x, set of float: y) = union_internal_float(x, y);
function set of float: union_internal_float(set of float: x, set of float: y);

function var set of $$E: 'union'(var set of $$E: x, var set of $$E: y) =
	to_enum_internal(enum_of(x), mzn_union_t(enum2int(x), enum2int(y)));

function var set of int: mzn_union_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var set of ub(x) union ub(y): r;
		constraint set_union(x, y, r);
	} in r;

function set of $U: array_union(array [$T] of set of $U: x) =
	let {
		any: xx = array1d(x);
	} in if length(xx) = 0 then
		{}
	elseif length(xx) = 1 then
		mzn_element_internal(xx, 1)
	else
		array_union(
			[mzn_element_internal(xx, 1) union mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

% Set intersection

function set of $T: 'intersect'(set of $T: x, set of $T: y) = {x_i | x_i in x where x_i in y};
function set of int: 'intersect'(set of int: x, set of int: y) = intersect_internal_int(x, y);
function set of int: intersect_internal_int(set of int: x, set of int: y);
function set of float: 'intersect'(set of float: x, set of float: y) =
	intersect_internal_float(x, y);
function set of float: intersect_internal_float(set of float: x, set of float: y);

function var set of $$E: 'intersect'(var set of $$E: x, var set of $$E: y) =
	to_enum_internal(enum_of(x), mzn_intersect_t(enum2int(x), enum2int(y)));

function var set of int: mzn_intersect_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var set of ub(x) intersect ub(y): r;
		constraint set_intersect(x, y, r);
	} in r;

% Set difference

function set of $T: 'diff'(set of $T: x, set of $T: y) = {x_i | x_i in x where not (x_i in y)};
function set of int: 'diff'(set of int: x, set of int: y) = diff_internal_int(x, y);
function set of int: diff_internal_int(set of int: x, set of int: y);
function set of float: 'diff'(set of float: x, set of float: y) = diff_internal_float(x, y);
function set of float: diff_internal_float(set of float: x, set of float: y);

function var set of $$E: 'diff'(var set of $$E: x, var set of $$E: y) =
	to_enum_internal(enum_of(x), mzn_diff_t(enum2int(x), enum2int(y)));

function var set of int: mzn_diff_t(var set of int: x, var set of int: y) :: promise_total =
	let {
		var set of ub(x) diff lb(y): r;
		constraint set_diff(x, y, r);
	} in r;

% Set symmetric difference

function set of $T: 'symdiff'(set of $T: x, set of $T: y) = (x diff y) union (y diff x);

function var set of $$E: 'symdiff'(var set of $$E: x, var set of $$E: y) =
	(x diff y) union (y diff x);

% Coercions

function int: bool2int(bool: x) = if x then 1 else 0 endif;
function var int: bool2int(var bool: x) = bool2int_t(x);

function var int: bool2int_t(
	var bool: x,
)
	:: promise_total =
	let {
		var 0..1: i;
		constraint bool2int(x, i);
	} in i;

function var float: int2float(var int: x) = int2float_t(x);

function var float: int2float_t(
	var int: x,
)
	:: promise_total =
	let {
		var float: f;
		constraint int2float(x, f);
	} in f;

% String join with sep

function string: join(string: d, array [$T] of string: s) =
	if length(s) = 0 then
		""
	else
		concat([
			if mod_internal(i, 2) = 1 then
				mzn_element_internal(xx, div_internal_int(i, 2) + 1)
			else
				d
			endif |
				xx = array1d(s),
				i in 1..length(s) * 2 - 1,
		])
	endif;
function string: concat(array [$T] of string: s) =
	if length(s) = 0 then
		""
	elseif length(s) = 1 then
		mzn_element_internal(array1d(s), 1)
	else
		let {
			any: xx = array1d(s);
		} in concat(
			[mzn_element_internal(xx, 1) ++ mzn_element_internal(xx, 2)] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

function array [int] of any $T: '++'(array [$$X] of any $T: x, array [$$Y] of any $T: y) = [
	if i <= length(x) then
		mzn_element_internal(xx, i)
	else
		mzn_element_internal(yy, i - length(x))
	endif |
		xx = array1d(x),
		yy = array1d(y),
		i in 1..length(x) + length(y),
];

function $$E: arg_min_internal(array [$$E] of $T: xs) = arg_val(xs, min(xs));
function $$E: arg_max_internal(array [$$E] of $T: xs) = arg_val(xs, max(xs));

function $T: min($T: x, $T: y) = if x < y then x else y endif;
function $T: max($T: x, $T: y) = if x < y then y else x endif;

function $$E: min_internal(set of $$E: s) = mzn_element_internal(set2array(s), 1);
function $$E: max_internal(set of $$E: s) = mzn_element_internal(set2array(s), card(s));

function $T: min_internal_array(array [$$E] of $T: x) =
	let {
		any: xx = array1d(x);
	} in if length(xx) = 1 then
		mzn_element_internal(xx, 1)
	elseif length(x) = 2 then
		min(mzn_element_internal(xx, 1), mzn_element_internal(xx, 2))
	else
		min_internal_array(
			[min(mzn_element_internal(xx, 1), mzn_element_internal(xx, 2))] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

function $T: max_internal_array(array [$$E] of $T: x) =
	let {
		any: xx = array1d(x);
	} in if length(xx) = 1 then
		mzn_element_internal(xx, 1)
	elseif length(x) = 2 then
		max(mzn_element_internal(xx, 1), mzn_element_internal(xx, 2))
	else
		max_internal_array(
			[max(mzn_element_internal(xx, 1), mzn_element_internal(xx, 2))] ++
				mzn_slice_internal(xx, [3..length(xx)]),
		)
	endif;

function set of int: compute_div_bounds(var int: x, var int: y) =
	if lb(x) = -infinity \/ ub(x) = infinity \/ lb(y) = -infinity \/ ub(y) = infinity then
		-infinity..infinity
	elseif lb(y) = 0 /\ ub(y) = 0 then
		% y is the singleton {0}; the caller is responsible for excluding it.
		-infinity..infinity
	else
		let {
			set of int: candidates =
				(
					if lb(y) < 0 then
						{
							div_internal_int(lb(x), lb(y)),
							div_internal_int(ub(x), lb(y)),
							div_internal_int(lb(x), min(ub(y), -1)),
							div_internal_int(ub(x), min(ub(y), -1)),
						}
					else
						{}
					endif
				) union
					(
						if ub(y) > 0 then
							{
								div_internal_int(lb(x), max(lb(y), 1)),
								div_internal_int(ub(x), max(lb(y), 1)),
								div_internal_int(lb(x), ub(y)),
								div_internal_int(ub(x), ub(y)),
							}
						else
							{}
						endif
					);
		} in min(candidates)..max(candidates)
	endif;

function set of int: compute_mod_bounds(var int: x, var int: y) =
	if lb(y) = -infinity \/ ub(y) = infinity then
		-infinity..infinity
	elseif lb(x) = 0 /\ ub(x) = 0 then
		{0}
	else
		let {
			int: am = max(-lb(y), ub(y)) - 1;
		} in if lb(x) >= 0 then 0..am elseif ub(x) <= 0 then -am..0 else -am..am endif
	endif;

function set of float: compute_float_div_bounds(var float: x, var float: y) =
	if lb(x) = -infinity \/
		ub(x) = infinity \/
		lb(y) = -infinity \/
		ub(y) = infinity \/
		lb(y) <= 0.0 /\ ub(y) >= 0.0 then
		let {
			float: l = -infinity;
			float: u = infinity;
		} in l..u
	elseif lb(x) = 0.0 /\ ub(x) = 0.0 then
		0.0..0.0
	else
		min(min(lb(x) / lb(y), lb(x) / ub(y)), min(ub(x) / lb(y), ub(x) / ub(y))) ..
			max(max(lb(x) / lb(y), lb(x) / ub(y)), max(ub(x) / lb(y), ub(x) / ub(y)))
	endif;

function set of int: compute_pow_bounds(var int: x, var int: y) =
	if lb(x) = -infinity \/ ub(x) = infinity \/ lb(y) = -infinity \/ ub(y) = infinity then
		-infinity..infinity
	elseif lb(x) = 0 /\ ub(x) = 0 /\ ub(y) < 0 then
		{}
	else
		let {
			set of int: candidates =
				(
					if lb(x) <= 1 /\ ub(x) >= 1 \/ lb(y) <= 0 /\ ub(y) >= 0 then {1} else {} endif
				) union
					(
						if lb(x) <= -1 /\ ub(x) >= -1 then
							if lb(y) != ub(y) then
								{-1, 1}
							elseif mod_internal(lb(y), 2) = 0 then
								{1}
							else
								{-1}
							endif
						else
							{}
						endif
					) union
					(if ub(x) != 0 \/ ub(y) >= 0 then {ub(x) ^ ub(y)} else {} endif) union
					(
						if lb(x) < 0 then
							{lb(x) ^ ub(y)} union
								(if lb(y) != ub(y) then {lb(x) ^ (ub(y) - 1)} else {} endif)
						elseif (lb(x) = 0 /\ ub(y) > 0) \/ (ub(x) > 1 /\ lb(y) < 0) then
							{0}
						elseif lb(y) >= 0 then
							{lb(x) ^ lb(y)}
						else
							{}
						endif
					) union
					(
						if lb(x) < 0 /\ lb(y) = ub(y) /\ ub(x) >= 0 /\ lb(y) > 0 then {0} else {} endif
					);
		} in if candidates = {} then {} else min(candidates)..max(candidates) endif
	endif;

function array [int] of any $T: concat_index_1(
	array [$$X11] of any $T: x,
	array [$$X12] of any $T: y,
) = x ++ y;
function array [int, $$X2] of any $T: concat_index_1(
	array [$$X11, $$X2] of any $T: x,
	array [$$X12, $$X2] of any $T: y,
) = x ++ y;
function array [int, $$X2, $$X3] of any $T: concat_index_1(
	array [$$X11, $$X2, $$X3] of any $T: x,
	array [$$X12, $$X2, $$X3] of any $T: y,
) = x ++ y;
function array [int, $$X2, $$X3, $$X4] of any $T: concat_index_1(
	array [$$X11, $$X2, $$X3, $$X4] of any $T: x,
	array [$$X12, $$X2, $$X3, $$X4] of any $T: y,
) = x ++ y;
function array [int, $$X2, $$X3, $$X4, $$X5] of any $T: concat_index_1(
	array [$$X11, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	array [$$X12, $$X2, $$X3, $$X4, $$X5] of any $T: y,
) = x ++ y;
function array [int, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: concat_index_1(
	array [$$X11, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	array [$$X12, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: y,
) = x ++ y;
function array [$$X1, int] of any $T: concat_index_2(
	array [$$X1, $$X21] of any $T: x,
	array [$$X1, $$X22] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xcols = card(index_set_2of2(xx));
		any: ycols = card(index_set_2of2(y));
	} in assert(
		index_sets_agree(xx[.., 1..0], yy[.., 1..0]),
		"Row index sets must match",
		[
			(i, j): if j <= xcols then xx[enum2int(i), j] else yy[enum2int(i), j - xcols] endif |
				i in index_set_1of2(x),
				j in 1..xcols + ycols,
		],
	);
function array [$$X1, int, $$X3] of any $T: concat_index_2(
	array [$$X1, $$X21, $$X3] of any $T: x,
	array [$$X1, $$X22, $$X3] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_2of3(xx));
		any: ylen = card(index_set_2of3(y));
	} in assert(
		index_sets_agree(xx[.., 1..0, ..], yy[.., 1..0, ..]),
		"Index sets other than the second index set must match",
		[
			(i, j, k): if j <= xlen then
				xx[enum2int(i), j, enum2int(k)]
			else
				yy[enum2int(i), j - xlen, enum2int(k)]
			endif |
				i in index_set_1of3(x),
				j in 1..xlen + ylen,
				k in index_set_3of3(x),
		],
	);
function array [$$X1, int, $$X3, $$X4] of any $T: concat_index_2(
	array [$$X1, $$X21, $$X3, $$X4] of any $T: x,
	array [$$X1, $$X22, $$X3, $$X4] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_2of4(xx));
		any: ylen = card(index_set_2of4(y));
	} in assert(
		index_sets_agree(xx[.., 1..0, .., ..], yy[.., 1..0, .., ..]),
		"Index sets other than the second index set must match",
		[
			(i, j, k, l): if j <= xlen then
				xx[enum2int(i), j, enum2int(k), enum2int(l)]
			else
				yy[enum2int(i), j - xlen, enum2int(k), enum2int(l)]
			endif |
				i in index_set_1of4(x),
				j in 1..xlen + ylen,
				k in index_set_3of4(x),
				l in index_set_4of4(x),
		],
	);
function array [$$X1, int, $$X3, $$X4, $$X5] of any $T: concat_index_2(
	array [$$X1, $$X21, $$X3, $$X4, $$X5] of any $T: x,
	array [$$X1, $$X22, $$X3, $$X4, $$X5] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_2of5(xx));
		any: ylen = card(index_set_2of5(y));
	} in assert(
		index_sets_agree(xx[.., 1..0, .., .., ..], yy[.., 1..0, .., .., ..]),
		"Index sets other than the second index set must match",
		[
			(i, j, k, l, m): if j <= xlen then
				xx[enum2int(i), j, enum2int(k), enum2int(l), enum2int(m)]
			else
				yy[enum2int(i), j - xlen, enum2int(k), enum2int(l), enum2int(m)]
			endif |
				i in index_set_1of5(x),
				j in 1..xlen + ylen,
				k in index_set_3of5(x),
				l in index_set_4of5(x),
				m in index_set_5of5(x),
		],
	);
function array [$$X1, int, $$X3, $$X4, $$X5, $$X6] of any $T: concat_index_2(
	array [$$X1, $$X21, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	array [$$X1, $$X22, $$X3, $$X4, $$X5, $$X6] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_2of6(xx));
		any: ylen = card(index_set_2of6(y));
	} in assert(
		index_sets_agree(xx[.., 1..0, .., .., .., ..], yy[.., 1..0, .., .., .., ..]),
		"Index sets other than the second index set must match",
		[
			(i, j, k, l, m, n): if j <= xlen then
				xx[enum2int(i), j, enum2int(k), enum2int(l), enum2int(m), enum2int(n)]
			else
				yy[enum2int(i), j - xlen, enum2int(k), enum2int(l), enum2int(m), enum2int(n)]
			endif |
				i in index_set_1of6(x),
				j in 1..xlen + ylen,
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, int] of any $T: concat_index_3(
	array [$$X1, $$X2, $$X31] of any $T: x,
	array [$$X1, $$X2, $$X32] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_3of3(xx));
		any: ylen = card(index_set_3of3(yy));
	} in assert(
		index_sets_agree(xx[.., .., 1..0], yy[.., .., 1..0]),
		"Index sets other than the third index set must match",
		[
			(i, j, k): if k <= xlen then
				xx[enum2int(i), enum2int(j), k]
			else
				yy[enum2int(i), enum2int(j), k - xlen]
			endif |
				i in index_set_1of3(x),
				j in index_set_2of3(x),
				k in 1..xlen + ylen,
		],
	);
function array [$$X1, $$X2, int, $$X4] of any $T: concat_index_3(
	array [$$X1, $$X2, $$X31, $$X4] of any $T: x,
	array [$$X1, $$X2, $$X32, $$X4] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_3of4(xx));
		any: ylen = card(index_set_3of4(yy));
	} in assert(
		index_sets_agree(xx[.., .., 1..0, ..], yy[.., .., 1..0, ..]),
		"Index sets other than the third index set must match",
		[
			(i, j, k, l): if k <= xlen then
				xx[enum2int(i), enum2int(j), k, enum2int(l)]
			else
				yy[enum2int(i), enum2int(j), k - xlen, enum2int(l)]
			endif |
				i in index_set_1of4(x),
				j in index_set_2of4(x),
				k in 1..xlen + ylen,
				l in index_set_4of4(x),
		],
	);
function array [$$X1, $$X2, int, $$X4, $$X5] of any $T: concat_index_3(
	array [$$X1, $$X2, $$X31, $$X4, $$X5] of any $T: x,
	array [$$X1, $$X2, $$X32, $$X4, $$X5] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_3of5(xx));
		any: ylen = card(index_set_3of5(yy));
	} in assert(
		index_sets_agree(xx[.., .., 1..0, .., ..], yy[.., .., 1..0, .., ..]),
		"Index sets other than the third index set must match",
		[
			(i, j, k, l, m): if k <= xlen then
				xx[enum2int(i), enum2int(j), k, enum2int(l), enum2int(m)]
			else
				yy[enum2int(i), enum2int(j), k - xlen, enum2int(l), enum2int(m)]
			endif |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in 1..xlen + ylen,
				l in index_set_4of5(x),
				m in index_set_5of5(x),
		],
	);
function array [$$X1, $$X2, int, $$X4, $$X5, $$X6] of any $T: concat_index_3(
	array [$$X1, $$X2, $$X31, $$X4, $$X5, $$X6] of any $T: x,
	array [$$X1, $$X2, $$X32, $$X4, $$X5, $$X6] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_3of6(xx));
		any: ylen = card(index_set_3of6(yy));
	} in assert(
		index_sets_agree(xx[.., .., 1..0, .., .., ..], yy[.., .., 1..0, .., .., ..]),
		"Index sets other than the third index set must match",
		[
			(i, j, k, l, m, n): if k <= xlen then
				xx[enum2int(i), enum2int(j), k, enum2int(l), enum2int(m), enum2int(n)]
			else
				yy[enum2int(i), enum2int(j), k - xlen, enum2int(l), enum2int(m), enum2int(n)]
			endif |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in 1..xlen + ylen,
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, int] of any $T: concat_index_4(
	array [$$X1, $$X2, $$X3, $$X41] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X42] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_4of4(xx));
		any: ylen = card(index_set_4of4(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., 1..0], yy[.., .., .., 1..0]),
		"Index sets other than the fourth index set must match",
		[
			(i, j, k, l): if l <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), l]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), l - xlen]
			endif |
				i in index_set_1of4(x),
				j in index_set_2of4(x),
				k in index_set_3of4(x),
				l in 1..xlen + ylen,
		],
	);
function array [$$X1, $$X2, $$X3, int, $$X5] of any $T: concat_index_4(
	array [$$X1, $$X2, $$X3, $$X41, $$X5] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X42, $$X5] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_4of5(xx));
		any: ylen = card(index_set_4of5(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., 1..0, ..], yy[.., .., .., 1..0, ..]),
		"Index sets other than the fourth index set must match",
		[
			(i, j, k, l, m): if l <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), l, enum2int(m)]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), l - xlen, enum2int(m)]
			endif |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in index_set_3of5(x),
				l in 1..xlen + ylen,
				m in index_set_5of5(x),
		],
	);
function array [$$X1, $$X2, $$X3, int, $$X5, $$X6] of any $T: concat_index_4(
	array [$$X1, $$X2, $$X3, $$X41, $$X5, $$X6] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X42, $$X5, $$X6] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_4of6(xx));
		any: ylen = card(index_set_4of6(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., 1..0, .., ..], yy[.., .., .., 1..0, .., ..]),
		"Index sets other than the fourth index set must match",
		[
			(i, j, k, l, m, n): if l <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), l, enum2int(m), enum2int(n)]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), l - xlen, enum2int(m), enum2int(n)]
			endif |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in 1..xlen + ylen,
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, int] of any $T: concat_index_5(
	array [$$X1, $$X2, $$X3, $$X4, $$X51] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X4, $$X52] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_5of5(xx));
		any: ylen = card(index_set_5of5(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., .., 1..0], yy[.., .., .., .., 1..0]),
		"Index sets other than the fifth index set must match",
		[
			(i, j, k, l, m): if m <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), enum2int(l), m]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), enum2int(l), m - xlen]
			endif |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in index_set_3of5(x),
				l in index_set_4of5(x),
				m in 1..xlen + ylen,
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, int, $$X6] of any $T: concat_index_5(
	array [$$X1, $$X2, $$X3, $$X4, $$X51, $$X6] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X4, $$X52, $$X6] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_5of6(xx));
		any: ylen = card(index_set_5of6(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., .., 1..0, ..], yy[.., .., .., .., 1..0, ..]),
		"Index sets other than the fifth index set must match",
		[
			(i, j, k, l, m, n): if m <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), enum2int(l), m, enum2int(n)]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), enum2int(l), m - xlen, enum2int(n)]
			endif |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in 1..xlen + ylen,
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, $$X5, int] of any $T: concat_index_6(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X61] of any $T: x,
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X62] of any $T: y,
) =
	let {
		any: xx = index2int(x);
		any: yy = index2int(y);
		any: xlen = card(index_set_6of6(xx));
		any: ylen = card(index_set_6of6(yy));
	} in assert(
		index_sets_agree(xx[.., .., .., .., .., 1..0], yy[.., .., .., .., .., 1..0]),
		"Index sets other than the sixth index set must match",
		[
			(i, j, k, l, m, n): if n <= xlen then
				xx[enum2int(i), enum2int(j), enum2int(k), enum2int(l), enum2int(m), n]
			else
				yy[enum2int(i), enum2int(j), enum2int(k), enum2int(l), enum2int(m), n - xlen]
			endif |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in 1..xlen + ylen,
		],
	);
function array [int] of any $T: select_from_index_1(array [$$X1] of any $T: x, set of $$X1: S) =
	let {
		constraint
			mzn_assert_warn(S subset index_set(x), "Selection must be in the first index set");
	} in [x[i] | i in S];
function array [int, $$X2] of any $T: select_from_index_1(
	array [$$X1, $$X2] of any $T: x,
	set of $$X1: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_1of2(x), "Selection must be in the first index set");
	} in array2d(1..card(S), index_set_2of2(x), [x[i, j] | i in S, j in index_set_2of2(x)]);
function array [int, $$X2, $$X3] of any $T: select_from_index_1(
	array [$$X1, $$X2, $$X3] of any $T: x,
	set of $$X1: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_1of3(x), "Selection must be in the first index set");
	} in array3d(
		1..card(S),
		index_set_2of3(x),
		index_set_3of3(x),
		[x[i, j, k] | i in S, j in index_set_2of3(x), k in index_set_3of3(x)],
	);
function array [int, $$X2, $$X3, $$X4] of any $T: select_from_index_1(
	array [$$X1, $$X2, $$X3, $$X4] of any $T: x,
	set of $$X1: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_1of4(x), "Selection must be in the first index set");
	} in array4d(
		1..card(S),
		index_set_2of4(x),
		index_set_3of4(x),
		index_set_4of4(x),
		[
			x[i, j, k, l] |
				i in S,
				j in index_set_2of4(x),
				k in index_set_3of4(x),
				l in index_set_4of4(x),
		],
	);
function array [int, $$X2, $$X3, $$X4, $$X5] of any $T: select_from_index_1(
	array [$$X1, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	set of $$X1: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_1of5(x), "Selection must be in the first index set");
	} in array5d(
		1..card(S),
		index_set_2of5(x),
		index_set_3of5(x),
		index_set_4of5(x),
		index_set_5of5(x),
		[
			x[i, j, k, l, m] |
				i in S,
				j in index_set_2of5(x),
				k in index_set_3of5(x),
				l in index_set_4of5(x),
				m in index_set_5of5(x),
		],
	);
function array [int, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: select_from_index_1(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X1: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_1of6(x), "Selection must be in the first index set");
	} in array6d(
		1..card(S),
		index_set_2of6(x),
		index_set_3of6(x),
		index_set_4of6(x),
		index_set_5of6(x),
		index_set_6of6(x),
		[
			x[i, j, k, l, m, n] |
				i in S,
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, int] of any $T: select_from_index_2(
	array [$$X1, $$X2] of any $T: x,
	set of $$X2: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_2of2(x),
				"Selection must be in the second index set",
			);
	} in array2d(index_set_1of2(x), 1..card(S), [x[i, j] | i in index_set_1of2(x), j in S]);
function array [$$X1, int, $$X3] of any $T: select_from_index_2(
	array [$$X1, $$X2, $$X3] of any $T: x,
	set of $$X2: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_2of3(x),
				"Selection must be in the second index set",
			);
	} in array3d(
		index_set_1of3(x),
		1..card(S),
		index_set_3of3(x),
		[x[i, j, k] | i in index_set_1of3(x), j in S, k in index_set_3of3(x)],
	);
function array [$$X1, int, $$X3, $$X4] of any $T: select_from_index_2(
	array [$$X1, $$X2, $$X3, $$X4] of any $T: x,
	set of $$X2: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_2of4(x),
				"Selection must be in the second index set",
			);
	} in array4d(
		index_set_1of4(x),
		1..card(S),
		index_set_3of4(x),
		index_set_4of4(x),
		[
			x[i, j, k, l] |
				i in index_set_1of4(x),
				j in S,
				k in index_set_3of4(x),
				l in index_set_4of4(x),
		],
	);
function array [$$X1, int, $$X3, $$X4, $$X5] of any $T: select_from_index_2(
	array [$$X1, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	set of $$X2: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_2of5(x),
				"Selection must be in the second index set",
			);
	} in array5d(
		index_set_1of5(x),
		1..card(S),
		index_set_3of5(x),
		index_set_4of5(x),
		index_set_5of5(x),
		[
			x[i, j, k, l, m] |
				i in index_set_1of5(x),
				j in S,
				k in index_set_3of5(x),
				l in index_set_4of5(x),
				m in index_set_5of5(x),
		],
	);
function array [$$X1, int, $$X3, $$X4, $$X5, $$X6] of any $T: select_from_index_2(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X2: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_2of6(x),
				"Selection must be in the second index set",
			);
	} in array6d(
		index_set_1of6(x),
		1..card(S),
		index_set_3of6(x),
		index_set_4of6(x),
		index_set_5of6(x),
		index_set_6of6(x),
		[
			x[i, j, k, l, m, n] |
				i in index_set_1of6(x),
				j in S,
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, int] of any $T: select_from_index_3(
	array [$$X1, $$X2, $$X3] of any $T: x,
	set of $$X3: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_3of3(x), "Selection must be in the third index set");
	} in array3d(
		index_set_1of3(x),
		index_set_2of3(x),
		1..card(S),
		[x[i, j, k] | i in index_set_1of3(x), j in index_set_2of3(x), k in S],
	);
function array [$$X1, $$X2, int, $$X4] of any $T: select_from_index_3(
	array [$$X1, $$X2, $$X3, $$X4] of any $T: x,
	set of $$X3: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_3of4(x), "Selection must be in the third index set");
	} in array4d(
		index_set_1of4(x),
		index_set_2of4(x),
		1..card(S),
		index_set_4of4(x),
		[
			x[i, j, k, l] |
				i in index_set_1of4(x),
				j in index_set_2of4(x),
				k in S,
				l in index_set_4of4(x),
		],
	);
function array [$$X1, $$X2, int, $$X4, $$X5] of any $T: select_from_index_3(
	array [$$X1, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	set of $$X3: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_3of5(x), "Selection must be in the third index set");
	} in array5d(
		index_set_1of5(x),
		index_set_2of5(x),
		1..card(S),
		index_set_4of5(x),
		index_set_5of5(x),
		[
			x[i, j, k, l, m] |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in S,
				l in index_set_4of5(x),
				m in index_set_5of5(x),
		],
	);
function array [$$X1, $$X2, int, $$X4, $$X5, $$X6] of any $T: select_from_index_3(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X3: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_3of6(x), "Selection must be in the third index set");
	} in array6d(
		index_set_1of6(x),
		index_set_2of6(x),
		1..card(S),
		index_set_4of6(x),
		index_set_5of6(x),
		index_set_6of6(x),
		[
			x[i, j, k, l, m, n] |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in S,
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, int] of any $T: select_from_index_4(
	array [$$X1, $$X2, $$X3, $$X4] of any $T: x,
	set of $$X4: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_4of4(x),
				"Selection must be in the fourth index set",
			);
	} in array4d(
		index_set_1of4(x),
		index_set_2of4(x),
		index_set_3of4(x),
		1..card(S),
		[
			x[i, j, k, l] |
				i in index_set_1of4(x),
				j in index_set_2of4(x),
				k in index_set_3of4(x),
				l in S,
		],
	);
function array [$$X1, $$X2, $$X3, int, $$X5] of any $T: select_from_index_4(
	array [$$X1, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	set of $$X4: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_4of5(x),
				"Selection must be in the fourth index set",
			);
	} in array5d(
		index_set_1of5(x),
		index_set_2of5(x),
		index_set_3of5(x),
		1..card(S),
		index_set_5of5(x),
		[
			x[i, j, k, l, m] |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in index_set_3of5(x),
				l in S,
				m in index_set_5of5(x),
		],
	);
function array [$$X1, $$X2, $$X3, int, $$X5, $$X6] of any $T: select_from_index_4(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X4: S,
) =
	let {
		constraint
			mzn_assert_warn(
				S subset index_set_4of6(x),
				"Selection must be in the fourth index set",
			);
	} in array6d(
		index_set_1of6(x),
		index_set_2of6(x),
		index_set_3of6(x),
		1..card(S),
		index_set_5of6(x),
		index_set_6of6(x),
		[
			x[i, j, k, l, m, n] |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in S,
				m in index_set_5of6(x),
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, int] of any $T: select_from_index_5(
	array [$$X1, $$X2, $$X3, $$X4, $$X5] of any $T: x,
	set of $$X5: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_5of5(x), "Selection must be in the fifth index set");
	} in array5d(
		index_set_1of5(x),
		index_set_2of5(x),
		index_set_3of5(x),
		index_set_4of5(x),
		1..card(S),
		[
			x[i, j, k, l, m] |
				i in index_set_1of5(x),
				j in index_set_2of5(x),
				k in index_set_3of5(x),
				l in index_set_4of5(x),
				m in S,
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, int, $$X6] of any $T: select_from_index_5(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X5: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_5of6(x), "Selection must be in the fifth index set");
	} in array6d(
		index_set_1of6(x),
		index_set_2of6(x),
		index_set_3of6(x),
		index_set_4of6(x),
		1..card(S),
		index_set_6of6(x),
		[
			x[i, j, k, l, m, n] |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in S,
				n in index_set_6of6(x),
		],
	);
function array [$$X1, $$X2, $$X3, $$X4, $$X5, int] of any $T: select_from_index_6(
	array [$$X1, $$X2, $$X3, $$X4, $$X5, $$X6] of any $T: x,
	set of $$X6: S,
) =
	let {
		constraint
			mzn_assert_warn(S subset index_set_6of6(x), "Selection must be in the sixth index set");
	} in array6d(
		index_set_1of6(x),
		index_set_2of6(x),
		index_set_3of6(x),
		index_set_4of6(x),
		index_set_5of6(x),
		1..card(S),
		[
			x[i, j, k, l, m, n] |
				i in index_set_1of6(x),
				j in index_set_2of6(x),
				k in index_set_3of6(x),
				l in index_set_4of6(x),
				m in index_set_5of6(x),
				n in S,
		],
	);

function array [$$E] of $$F: inverse_internal(array [$$F] of $$E: f) = [
	f[i]: i |
		i in index_set(f),
];
function set of $U: array_intersect_internal(array [int] of set of $U: x) =
	if length(x) = 1 then x[1] else array_intersect_internal([x[1] intersect x[2]] ++ x[3..]) endif;

function array [int] of $$E: set_to_ranges(set of $$E: S) =
	to_enum_internal(enum_of(S), set_to_ranges_internal_int(enum2int(S)));
function array [int] of int: set_to_ranges_internal_int(set of int: S);

function array [int] of float: set_to_ranges(set of float: S) = set_to_ranges_internal_float(S);
function array [int] of float: set_to_ranges_internal_float(set of float: S);

function array [int] of $$E: set_to_sparse_inverse(set of $$E: S) =
	let { constraint abort("not implemented") } in [];

function array [$$F] of any $T: slice_1d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
) = let { constraint abort("not implemented") } in [];
function array [$$F, $$G] of any $T: slice_2d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
	set of $$G: dims2,
) = let { constraint abort("not implemented") } in [];
function array [$$F, $$G, $$H] of any $T: slice_3d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
	set of $$G: dims2,
	set of $$H: dims3,
) = let { constraint abort("not implemented") } in [];
function array [$$F, $$G, $$H, $$I] of any $T: slice_4d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
	set of $$G: dims2,
	set of $$H: dims3,
	set of $$I: dims4,
) = let { constraint abort("not implemented") } in [];
function array [$$F, $$G, $$H, $$I, $$J] of any $T: slice_5d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
	set of $$G: dims2,
	set of $$H: dims3,
	set of $$I: dims4,
	set of $$J: dims5,
) = let { constraint abort("not implemented") } in [];
function array [$$F, $$G, $$H, $$I, $$J, $$K] of any $T: slice_6d(
	array [$E] of any $T: x,
	array [int] of set of int: s,
	set of $$F: dims1,
	set of $$G: dims2,
	set of $$H: dims3,
	set of $$I: dims4,
	set of $$J: dims5,
	set of $$K: dims6,
) = let { constraint abort("not implemented") } in [];

function var int: lin_exp(array [int] of int, array [int] of var int, int) =
	let { constraint abort("not implemented") } in 0;
function var float: lin_exp(array [int] of float, array [int] of var float, float) =
	let { constraint abort("not implemented") } in 0;

function set of int: anon_enum(array [int] of string: x) =
	let { constraint abort("not implemented") } in {};

function array [int] of tuple($$E, $$E): mzn_set_to_ranges(set of $$E: S) =
	let {
		any: ranges = set_to_ranges(S);
	} in [
		(mzn_element_internal(ranges, i), mzn_element_internal(ranges, i + 1)) |
			i in index_set(ranges) where mod_internal(i, 2) = 1,
	];

function array [int] of tuple(float, float): mzn_set_to_ranges(set of float: S) =
	let {
		any: ranges = set_to_ranges(S);
	} in [
		(mzn_element_internal(ranges, i), mzn_element_internal(ranges, i + 1)) |
			i in index_set(ranges) where mod_internal(i, 2) = 1,
	];
