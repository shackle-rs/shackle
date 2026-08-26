% Var less than involving tuples/records
function var bool: '<'(any $T: x, any $T: y) :: mzn_internal_generated = abort("unreachable");
% Par less than
function bool: '<'($T: x, $T: y);

% TODO: Only required since these are defined in the old compiler library
function var bool: '<'(var $T: x, var $T: y) :: mzn_internal_generated = abort("unreachable");
function var bool: '<'($T: x, var $T: y) :: mzn_internal_generated = abort("unreachable");
function var bool: '<'(var $T: x, $T: y) :: mzn_internal_generated = abort("unreachable");

% Var less than or equal involving tuples/records
function var bool: '<='(any $T: x, any $T: y) :: mzn_internal_generated = abort("unreachable");

% TODO: Only required since these are defined in the old compiler library
function var bool: '<='(var $T: x, var $T: y) :: mzn_internal_generated = abort("unreachable");
function var bool: '<='($T: x, var $T: y) :: mzn_internal_generated = abort("unreachable");
function var bool: '<='(var $T: x, $T: y) :: mzn_internal_generated = abort("unreachable");

% Var equality involving tuples/records
function var bool: '='(any $T: x, any $T: y) :: mzn_internal_generated = abort("unreachable");
% Par equality
function bool: '='($T: x, $T: y);

function var bool: '='(array [$U] of any $T: x, array [$U] of any $T: y) :: promise_commutative =
	matching_index_sets(
		x,
		y,
	) /\
		(
			let {
				any: xx = array1d(x);
				any: yy = array1d(y);
			} in forall (i in index_set(xx)) (xx[i] = yy[i])
		);
