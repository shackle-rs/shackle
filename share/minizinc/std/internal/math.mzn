function int: 'div'(int: x, int: y) =
	let {
		constraint mzn_assert_warn(y != 0, "division by zero is undefined");
	} in div_internal_int(x, y);
function int: div_internal_int(int: x, int: y);

function int: 'mod'(int: x, int: y) =
	let {
		constraint mzn_assert_warn(y != 0, "modulus by zero is undefined");
	} in mod_internal(x, y);
function int: mod_internal(int: x, int: y);

function float: '/'(float: x, float: y) =
	let {
		constraint mzn_assert_warn(y != 0.0, "division by zero is undefined");
	} in div_internal_float(x, y);
function float: div_internal_float(float: x, float: y);

function $T: min(set of $T: s) =
	let {
		constraint mzn_assert_warn(card(s) > 0, "minimum of an empty set is undefined");
	} in min_internal_set(s);
% TODO: Only needed for compatibility with old minizinc library which only defines min of set of $$E
function $$E: min(set of $$E: s) =
	let {
		constraint mzn_assert_warn(card(s) > 0, "minimum of an empty set is undefined");
	} in min_internal_set(s);
function $T: min_internal_set(set of $T: s);

function $T: min(array [$U] of $T: x) =
	let {
		constraint mzn_assert_warn(length(x) > 0, "minimum of an empty array is undefined");
	} in min_internal_array(x);
function $T: min_internal_array(array [$U] of $T: x);

function $T: max(set of $T: s) =
	let {
		constraint mzn_assert_warn(card(s) > 0, "maximum of an empty set is undefined");
	} in max_internal_set(s);
% TODO: Only needed for compatibility with old minizinc library which only defines max of set of $$E
function $$E: max(set of $$E: s) =
	let {
		constraint mzn_assert_warn(card(s) > 0, "maximum of an empty set is undefined");
	} in max_internal_set(s);
function $T: max_internal_set(set of $T: s);

function $T: max(array [$U] of $T: x) =
	let {
		constraint mzn_assert_warn(length(x) > 0, "maximum of an empty array is undefined");
	} in max_internal_array(array1d(x));
function $T: max_internal_array(array [$U] of $T: x);

function $$E: arg_min(array [$$E] of $T: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_min of an empty array is undefined");
	} in arg_min_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_min(array [$$E] of bool: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_min of an empty array is undefined");
	} in arg_min_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_min(array [$$E] of $$T: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_min of an empty array is undefined");
	} in arg_min_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_min(array [$$E] of float: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_min of an empty array is undefined");
	} in arg_min_internal(xs);
function $$E: arg_min_internal(array [$$E] of $T: xs) :: mzn_internal_representation;

function $$E: arg_max(array [$$E] of $T: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_max of an empty array is undefined");
	} in arg_max_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_max(array [$$E] of bool: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_max of an empty array is undefined");
	} in arg_max_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_max(array [$$E] of $$T: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_max of an empty array is undefined");
	} in arg_max_internal(xs);
% TODO: Only needed for compatibility with old minizinc library
function $$E: arg_max(array [$$E] of float: xs) =
	let {
		constraint mzn_assert_warn(length(xs) > 0, "arg_max of an empty array is undefined");
	} in arg_max_internal(xs);
function $$E: arg_max_internal(array [$$E] of $T: xs);

function int: '^'(int: x, int: y) = pow(x, y);
function float: '^'(float: x, float: y) = pow(x, y);

function int: pow(int: x, int: y) =
	let {
		constraint mzn_assert_warn(x != 0 \/ y >= 0, "negative power of zero is undefined");
	} in pow_internal_int(x, y);
function int: pow_internal_int(int: x, int: y);

function float: pow(float: x, float: y) =
	let {
		constraint mzn_assert_warn(x != 0 \/ y >= 0, "negative power of zero is undefined");
	} in pow_internal_float(x, y);
function float: pow_internal_float(float: x, float: y);

function array [$$E] of $$F: inverse(array [$$F] of $$E: f) =
	let {
		constraint
			mzn_assert_warn(
				length(f) = 0 \/ enum2int(max(f)) - enum2int(min(f)) + 1 = length(f),
				"inverse on non-contiguous set of values is undefined",
			);
	} in inverse_internal(f);
function array [$$E] of $$F: inverse_internal(array [$$F] of $$E: f);

function set of $U: array_intersect(array [$T] of set of $U: x) :: promise_commutative =
	let {
		constraint
			 mzn_assert_warn(length(x) > 0, "array_intersect of an empty array is undefined");
	} in array_intersect_internal(array1d(x));
function set of $U: array_intersect_internal(array [int] of set of $U: x);
