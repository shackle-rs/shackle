% Definitions to keep verbatim for compatibility with old MiniZinc

function $T: 'default'(opt $T:x, $T: y);
function opt $T: 'default'(opt $T:x, opt $T: y);
function var $T: 'default'(var opt $T:x, var $T: y);
function var opt $T: 'default'(var opt $T:x, var opt $T: y);
function array[$U] of $T: 'default'(array[$U] of $T:x, array[$U] of $T: y);
function array[$U] of opt $T: 'default'(array[$U] of opt $T:x, array[$U] of opt $T: y);
function array[$U] of var $T: 'default'(array[$U] of var $T:x, array[$U] of var $T: y);
function array[$U] of var opt $T: 'default'(array[$U] of var opt $T:x, array[$U] of var opt $T: y);

function $T: 'default'(tuple(bool, $T): x, $T: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function var $T: 'default'(tuple(var bool, var $T): x, var $T: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function bool: 'default'(tuple(bool, bool): x, bool: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function var bool: 'default'(tuple(var bool, var bool): x, var bool: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function int: 'default'(tuple(bool, int): x, int: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function var int: 'default'(tuple(var bool, var int): x, var int: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function float: 'default'(tuple(bool, float): x, float: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

function var float: 'default'(tuple(var bool, var float): x, var float: y) = let {
    any: z = x default (true, y);
} in if z.1 then z.2 else y endif;

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

function opt $T: fix(var opt $T: x);
function array[$U] of opt $T: fix(array[$U] of var opt $T: x);

function $$T: deopt(opt $$T: x);
function $T: deopt(opt $T: x);
test occurs(opt $T: x);

mzn_absent_zero = (false, false);
mzn_min_version_required = (false, 0);
mzn_opt_only_range_domains = (false, false);
mzn_opt_annotate_defines_var = (false, false);
mzn_ignore_symmetry_breaking_constraints = (false, false);
mzn_ignore_redundant_constraints = (false, false);
