function opt $T: mzn_construct_opt(tuple(bool, $T): x);
function var opt $T: mzn_construct_opt(tuple(var bool, var $T): x);

function array [$U] of opt $T: mzn_construct_array_opt(array [$U] of tuple(bool, $T): x) =
	arrayXd(x, [mzn_construct_opt(x_i) | x_i in x]);
function array [$U] of var opt $T: mzn_construct_array_opt(
	array [$U] of tuple(var bool, var $T): x,
) = arrayXd(x, [mzn_construct_opt(x_i) | x_i in x]);

function tuple(bool, $T): mzn_destruct_opt(opt $T: x);
function tuple(var bool, var $T): mzn_destruct_opt(var opt $T: x);

function array [$U] of tuple(bool, $T): mzn_destruct_array_opt(array [$U] of opt $T: x) =
	arrayXd(x, [mzn_destruct_opt(x_i) | x_i in x]);
function array [$U] of tuple(var bool, var $T): mzn_destruct_array_opt(
	array [$U] of var opt $T: x,
) = arrayXd(x, [mzn_destruct_opt(x_i) | x_i in x]);

predicate mzn_opt_channel(var opt int: x, set of int: s) =
	let {
		any: (bx, dx) = mzn_destruct_opt(x);
	} in if 0 in s then not bx -> dx = 0 else not bx <-> dx = 0 endif;
predicate mzn_opt_channel(var opt float: x, set of float: s) =
	let {
		any: (bx, dx) = mzn_destruct_opt(x);
	} in if 0.0 in s then not bx -> dx = 0.0 else not bx <-> dx = 0.0 endif;
function set of int: mzn_opt_domain(set of int: s) = s union {0};
function set of float: mzn_opt_domain(set of float: s) = s union {0.0};

function var bool: '='(
	var opt $T: x,
	var opt $T: y,
) =
	let {
		any: (bx, dx) = mzn_destruct_opt(x);
		any: (by, dy) = mzn_destruct_opt(y);
	} in bx = by /\ dx = dy;

function $T: mzn_default_opt(opt $T: x, $T: y) :: mzn_inline_call_by_name =
	let {
		any: yy = y;
	} in mzn_default_partial(
		let { any: xx = x } in (if occurs(xx) then deopt(xx) else yy endif),
		yy,
	);
function var $T: mzn_default_opt(var opt $T: x, var $T: y) :: mzn_inline_call_by_name =
	let {
		any: yy = y;
	} in mzn_default_partial(
		let { any: xx = x } in (if occurs(xx) then deopt(xx) else yy endif),
		yy,
	);

function opt $T: mzn_default_opt(opt $T: x, opt $T: y) :: mzn_inline_call_by_name =
	let {
		any: yy = y;
	} in mzn_default_partial(
		let { any: xx = x } in (if occurs(xx) then deopt(xx) else yy endif),
		yy,
	);
function var opt $T: mzn_default_opt(var opt $T: x, var opt $T: y) :: mzn_inline_call_by_name =
	let {
		any: yy = y;
	} in mzn_default_partial(
		let { any: xx = x } in (if occurs(xx) then deopt(xx) else yy endif),
		yy,
	);

function opt $T: val2opt($T: x);
function var opt $T: val2opt(var $T: x);
