% Temporary redefinitions which may be baked into the interpreter in the future.

% Conjunction

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

function var bool: exists(array [$T] of var bool: x) = mzn_exists(array1d(x));
function var bool: exists(array [int] of var bool: x) = mzn_exists(x);

function var bool: mzn_exists(array [int] of var bool: x) = clause(x, []);

% Boolean clause

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

function var bool: 'not'(var bool: x) = mzn_not(x);

function var bool: mzn_not(
	var bool: x,
)
	:: promise_total =
	let {
		var bool: r;
		constraint bool_not(x, r);
	} in r;

% If and only if

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

function var bool: '<-'(var bool: x, var bool: y) = mzn_implies(y, x);

% Exclusive or

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

% Subset

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
	} in true;

% Set in for integers

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

function var bool: 'in'(
	var float: x,
	set of float: y,
) =
	let {
		any: yr = set_to_ranges(y);
		any: yrt = [(yr[i], yr[i + 1]) | i in index_set(yr) where i mod 2 = 1];
	} in exists([a <= x /\ x <= b | (a, b) in yrt]);

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
function var bool: '<'(var bool: x, bool: y) = mzn_bool_lt(x, y);
function var bool: '<'(bool: x, var bool: y) = mzn_bool_lt(x, y);

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

function var bool: '>='(var $T: x, var $T: y) = y <= x;
function var bool: '>='($T: x, var $T: y) = y <= x;
function var bool: '>='(var $T: x, $T: y) = y <= x;
