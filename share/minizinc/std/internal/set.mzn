% Set operations on enum-typed sets

function var set of $$E: to_enum_internal(set of $$E: X, var set of int: x)
	:: mzn_internal_representation;
function set of $$E: to_enum_internal(set of $$E: X, set of int: x) :: mzn_internal_representation;

/** @group stdlib.builtins.set Return the set \(\{\a a,\ldots,\a b\}\) */
function set of $$E: '..'($$E: a, $$E: b) =
	to_enum_internal(enum_of(a), range_internal_int(enum2int(a), enum2int(b)));
/** @group stdlib.builtins.set Return the set \(\{\a a,\ldots,\a b\}\) */
function set of float: '..'(float: a, float: b) = range_internal_float(a, b);

function set of int: range_internal_int(int: a, int: b);
function set of float: range_internal_float(float: a, float: b);
