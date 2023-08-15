% Definitions to keep verbatim for compatibility with old MiniZinc

function $T: 'default'(opt $T:x, $T: y);
function opt $T: 'default'(opt $T:x, opt $T: y);
function var $T: 'default'(var opt $T:x, var $T: y);
function var opt $T: 'default'(var opt $T:x, var opt $T: y);
function array[$U] of $T: 'default'(array[$U] of $T:x, array[$U] of $T: y);
function array[$U] of opt $T: 'default'(array[$U] of opt $T:x, array[$U] of opt $T: y);
function array[$U] of var $T: 'default'(array[$U] of var $T:x, array[$U] of var $T: y);
function array[$U] of var opt $T: 'default'(array[$U] of var opt $T:x, array[$U] of var opt $T: y);

function set of $$E: enum_of(var opt $$E: x) ::mzn_internal_representation;
function set of $$E: enum_of(array[$T] of var opt $$E: x) ::mzn_internal_representation;

function $$E: lb(var opt $$E: x);
function $$E: ub(var opt $$E: x);
function float: lb(var opt float: x);
function float: ub(var opt float: x);
function $$E: lb_array(array[$U] of var opt $$E: x);
function $$E: ub_array(array[$U] of var opt $$E: x);
function float: lb_array(array[$U] of var opt float: x);
function float: ub_array(array[$U] of var opt float: x);
function set of $$E: dom(var opt $$E: x);
function set of $$E: dom_array(array[$T] of var opt $$E: x);
function set of int: dom_bounds_array(array[$T] of var opt int: x);

function $$E: lb_array(array[$U] of var $$E: x) = let {
    array [int] of var opt int: xx = array1d(x);
} in to_enum(enum_of(x), lb_array(xx));

function $$E: ub_array(array[$U] of var $$E: x) = let {
    array [int] of var opt int: xx = array1d(x);
} in to_enum(enum_of(x), ub_array(xx));

function float: lb_array(array[$U] of var float: x) = let {
    array [int] of var opt float: xx = array1d(x);
} in lb_array(xx);

function float: ub_array(array[$U] of var float: x) = let {
    array [int] of var opt float: xx = array1d(x);
} in ub_array(xx);

function opt $T: fix(var opt $T: x);
function array[$U] of opt $T: fix(array[$U] of var opt $T: x);

function $$T: deopt(opt $$T: x);
function $T: deopt(opt $T: x);
test occurs(opt $T: x);

function set of $$E: enum_of_internal(set of $$E: e) = e;

function bool: mzn_assert_warn(bool: b, string: msg) = b;

test mzn_check_only_range_domains() = if mzn_opt_only_range_domains.1 then mzn_opt_only_range_domains.2 else false endif;
test mzn_check_annotate_defines_var() = if mzn_opt_annotate_defines_var.1 then mzn_opt_annotate_defines_var.2 else true endif;
test mzn_check_ignore_symmetry_breaking_constraints() = if mzn_ignore_symmetry_breaking_constraints.1 then mzn_ignore_symmetry_breaking_constraints.2 else false endif;
test mzn_check_ignore_redundant_constraints() = if mzn_ignore_redundant_constraints.1 then mzn_ignore_redundant_constraints.2 else false endif;
test mzn_check_absent_zero() = if mzn_absent_zero.1 then mzn_absent_zero.2 else true endif;

function ann: 'output'(any $T: x :: annotated_expression);
function ann: 'output'(array [$U] of any $T: x :: annotated_expression);

mzn_absent_zero = (false, false);
mzn_min_version_required = (false, 0);
mzn_opt_only_range_domains = (false, false);
mzn_opt_annotate_defines_var = (false, false);
mzn_ignore_symmetry_breaking_constraints = (false, false);
mzn_ignore_redundant_constraints = (false, false);
