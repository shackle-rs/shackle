/***
  @groupdef stdlib.optiontypes Option type support

  These functions and predicates implement the standard library for working
  with option types. Note that option type support is still incomplete.
*/

/** @group stdlib.optiontypes.bool True iff \a x is not absent */
predicate occurs(var opt $T: x) = mzn_destruct_opt(x).1;
/** @group stdlib.optiontypes.bool True iff \a x is not absent */
test occurs(opt $T: x) = mzn_destruct_opt(x).1;
/** @group stdlib.optiontypes Test if \a x is not absent (always returns true) */
test occurs(var $T: x) = true;
/** @group stdlib.optiontypes Test if \a x is not absent (always returns true) */
test occurs($T: x) = true;

/** @group stdlib.optiontypes Test if \a x is absent */
predicate absent(var opt $T: x) = not occurs(x);
/** @group stdlib.optiontypes Test if \a x is absent */
test absent(opt $T: x) = not occurs(x);
/** @group stdlib.optiontypes Test if \a x is absent (always returns false) */
test absent(var $T: x) = false;
/** @group stdlib.optiontypes Test if \a x is absent (always returns false) */
test absent($T: x) = false;

/** @group stdlib.optiontypes.bool Return value of \a x (assumes that \a x is not absent) */
function var $T: deopt(var opt $T: x) = mzn_destruct_opt(x).2;
/** @group stdlib.optiontypes Return value of \a x if \a x is not absent. Returns
  undefined when evaluated on absent value. */
function $T: deopt(
	opt $T: x,
) =
	let {
		any: (ox, dx) = mzn_destruct_opt(x);
		constraint mzn_assert_warn(ox, "Deopt on absent value is undefined");
	} in dx;

/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function var $T: deopt(var $T: x) = x;
/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function $T: deopt($T: x) = x;

/** @group stdlib.optiontypes Return array of the value of \a x[i] where \a x[i] is not absent.
  Returns undefined when evaluated with an absent element. */
function array [$U] of $T: deopt(array [$U] of opt $T: x) =
	arrayXd(x, [deopt(x_i) | x_i in x]);
/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function array [$U] of var $T: deopt(array [$U] of var $T: x) = x;
/** @group stdlib.optiontypes Return array of the value of \a x[i] (assumes that \a x[i] is not absent). */
function array [$U] of var $T: deopt(array [$U] of var opt $T: x) =
	arrayXd(x, [deopt(x_i) | x_i in x]);

/** @group stdlib.optiontypes.int True if \a x had zero in its original domain. */
function bool: had_zero(var opt $$E: x) :: cache_result =
	not
	(
		is_fixed(mzn_opt_absent_iff_zero(erase_enum(x))) /\
			fix(mzn_opt_absent_iff_zero(erase_enum(x)))
	);
/** @group stdlib.optiontypes.int True if \a x had zero in its original domain. */
function bool: had_zero(opt $$E: x) = erase_enum(x) = 0;
/** @group stdlib.optiontypes.int True if \a x had zero in its original domain. */
function bool: had_zero(array [$X] of var opt $$E: x) =
	exists (xi in x) (had_zero(xi));

function var opt $T: mzn_construct_opt(tuple(var bool, var $T));
function opt $T: mzn_construct_opt(tuple(bool, $T));
function tuple(var bool, var $T): mzn_destruct_opt(var opt $T);
function tuple(bool, $T): mzn_destruct_opt(opt $T);

function array [$X] of var opt $T: mzn_construct_opt(
	array [$X] of tuple(var bool, var $T),
);
function array [$X] of opt $T: mzn_construct_opt(array [$X] of tuple(bool, $T));
function array [$X] of tuple(var bool, var $T): mzn_destruct_opt(
	array [$X] of var opt $T,
);
function array [$X] of tuple(bool, $T): mzn_destruct_opt(array [$X] of opt $T);

function set of int: mzn_opt_domain(set of int: x) = x union {0};
function set of float: mzn_opt_domain(set of float: x) = x union {0.0};

predicate mzn_opt_channel(var opt int: x, set of int: s) =
	if 0 in s then
		mzn_opt_absent_then_zero(x)
	else
		mzn_opt_absent_iff_zero(x)
	endif;

predicate mzn_opt_channel(var opt float: x, set of float: s) =
	if 0.0 in s then
		mzn_opt_absent_then_zero(x)
	else
		mzn_opt_absent_iff_zero(x)
	endif;

predicate mzn_opt_absent_iff_zero(var opt int: x) = absent(x) <-> deopt(x) = 0;
predicate mzn_opt_absent_then_zero(var opt int: x) = absent(x) -> deopt(x) = 0;

predicate mzn_opt_absent_iff_zero(var opt float: x) =
	absent(x) <-> deopt(x) = 0.0;
predicate mzn_opt_absent_then_zero(var opt float: x) =
	absent(x) -> deopt(x) = 0.0;

function tuple(bool, bool): mzn_opt_bool(tuple(bool, bool): x)
	:: mzn_inline_call_by_name =
	let {
		any: xx = mzn_destruct_partial(x);
	} in if xx.1 then xx.2 else (true, false) endif;

function tuple(var bool, var bool): mzn_opt_bool(tuple(var bool, var bool): x)
	:: mzn_inline_call_by_name =
	let {
		any: xx = mzn_destruct_partial(x);
	} in (not xx.1 \/ xx.2.1, xx.1 /\ xx.2.2);
