function $T: 'default'(opt $T: x, $T: y) :: mzn_inline_call_by_name = mzn_default_opt(x, y);
function opt $T: 'default'(opt $T: x, opt $T: y) :: mzn_inline_call_by_name = mzn_default_opt(x, y);
function var $T: 'default'(var opt $T: x, var $T: y) :: mzn_inline_call_by_name =
	mzn_default_opt(x, y);
function var opt $T: 'default'(var opt $T: x, var opt $T: y) :: mzn_inline_call_by_name =
	mzn_default_opt(x, y);

function array [$U] of $T: 'default'(array [$U] of $T: x, array [$U] of $T: y)
	:: mzn_inline_call_by_name = mzn_default_partial(x, y);
function array [$U] of opt $T: 'default'(array [$U] of opt $T: x, array [$U] of opt $T: y)
	:: mzn_inline_call_by_name = mzn_default_partial(x, y);
function array [$U] of var $T: 'default'(array [$U] of var $T: x, array [$U] of var $T: y)
	:: mzn_inline_call_by_name = mzn_default_partial(x, y);
function array [$U] of var opt $T: 'default'(
	array [$U] of var opt $T: x,
	array [$U] of var opt $T: y,
) :: mzn_inline_call_by_name = mzn_default_partial(x, y);

function set of $$E: 'default'(set of $$E: x, set of $$E: y) :: mzn_inline_call_by_name =
	mzn_default_partial(x, y);
function var set of $$E: 'default'(var set of $$E: x, var set of $$E: y)
	 :: mzn_inline_call_by_name = mzn_default_partial(x, y);
function set of $T: 'default'(opt set of $T: x, set of $T: y) :: mzn_inline_call_by_name =
	mzn_default_opt(x, y);
function opt set of $T: 'default'(opt set of $T: x, opt set of $T: y) :: mzn_inline_call_by_name =
	mzn_default_opt(x, y);

function array [int] of $T: set2array(set of $T: x) = [v | v in x];
