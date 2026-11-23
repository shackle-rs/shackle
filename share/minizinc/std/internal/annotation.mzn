annotation mzn_inline;
annotation mzn_inline_call_by_name;
annotation mzn_unreachable;
annotation mzn_builtin;

annotation 'output';
annotation mzn_var_where_clause;
annotation mzn_fresh_var;

annotation mzn_internal_generated;

% An output array of (key, value) pairs, shown as an array indexed by the keys
annotation mzn_output_keyed;
% Forward the annotations to the next function call.
annotation mzn_forward_annotations;

% TODO: Deal with these
function ann: 'output'(any $T: x :: annotated_expression) = empty_annotation;
function ann: 'output'(array [$U] of any $T: x :: annotated_expression) = empty_annotation;
function any $T: mzn_deprecate(string: name, string: version, string: msg, any $T: x) = x;
function array [$U] of any $T: mzn_deprecate(
	string: name,
	string: version,
	string: msg,
	array [$U] of any $T: x,
) = x;

annotation output_array(array [int] of set of int: a);
function ann: output_array(array [$U] of set of int: a) = output_array(array1d(a));

function ann: defines_var(any $T: c) = empty_annotation;
function ann: defines_var(array [$U] of any $T: arr) = empty_annotation;

annotation bool_search_internal(array [int] of var bool: x, ann: select, ann: choice, ann: explore);
annotation int_search_internal(array [int] of var int: x, ann: select, ann: choice, ann: explore);
annotation float_search_internal(
	array [int] of var float: x,
	ann: select,
	ann: choice,
	ann: explore,
);
annotation set_search_internal(
	array [int] of var set of int: x,
	ann: select,
	ann: choice,
	ann: explore,
);

function ann: int_search(array [$X] of var $$E: x, ann: select, ann: choice, ann: explore) =
	int_search_internal(array1d(enum2int(x)), select, choice, explore);
function ann: bool_search(array [$X] of var bool: x, ann: select, ann: choice, ann: explore) =
	bool_search_internal(array1d(x), select, choice, explore);
function ann: float_search(
	array [$X] of var float: x,
	float: prec,
	ann: select,
	ann: choice,
	ann: explore,
) = float_search_internal(array1d(x), select, choice, explore);

function ann: set_search(array [$X] of var set of $$E: x, ann: select, ann: choice, ann: explore) =
	set_search_internal(array1d(enum2int(x)), select, choice, explore);

annotation warm_start_bool(array [int] of var bool: x, array [int] of bool: v);
annotation warm_start_int(array [int] of var int: x, array [int] of int: v);
annotation warm_start_float(array [int] of var float: x, array [int] of float: v);
annotation warm_start_set_of_int(array [int] of var set of int: x, array [int] of set of int: v);

function ann: warm_start(array [$X] of var bool: x, array [$X] of bool: v) =
	warm_start_bool(array1d(x), array1d(v));
function ann: warm_start(array [int] of var bool: x, array [int] of bool: v) =
	 warm_start_bool(x, v);
function ann: warm_start(array [$X] of var $$E: x, array [$X] of $$E: v) =
	warm_start_int(array1d(enum2int(x)), array1d(enum2int(v)));
function ann: warm_start(array [int] of var $$E: x, array [int] of $$E: v) =
	warm_start_int(enum2int(x), enum2int(v));
function ann: warm_start(array [$X] of var float: x, array [$X] of float: v) =
	warm_start_float(array1d(x), array1d(v));
function ann: warm_start(array [int] of var float: x, array [int] of float: v) =
	warm_start_float(x, v);
function ann: warm_start(array [$X] of var set of $$E: x, array [$X] of set of $$E: v) =
	warm_start_set_of_int(array1d(enum2int(x)), array1d(enum2int(v)));
function ann: warm_start(array [int] of var set of int: x, array [int] of set of int: v) =
	warm_start_set_of_int(x, v);

function ann: mzn_path(int: i) = empty_annotation;

annotation mzn_no_remove;
