/***
  @groupdef stdlib.optiontypes Option type support

  These functions and predicates implement the standard library for working
  with option types. Note that option type support is still incomplete.
*/

/** @group stdlib.optiontypes Test if \a x is not absent (always returns true) */
function bool: occurs($T: x) = true;
/** @group stdlib.optiontypes Test if \a x is not absent (always returns true) */
function bool: occurs(var $T: x) = true;

/** @group stdlib.optiontypes.bool True iff \a x is not absent */
function bool: occurs(opt $T: x) = mzn_destruct_opt(x).1;
/** @group stdlib.optiontypes.bool True iff \a x is not absent */
function var bool: occurs(var opt $T: x) = mzn_destruct_opt(x).1;

/** @group stdlib.optiontypes Test if \a x is absent (always returns false) */
function bool: absent($T: x) = false;
/** @group stdlib.optiontypes Test if \a x is absent (always returns false) */
function bool: absent(var $T: x) = false;

/** @group stdlib.optiontypes True iff \a x is absent */
function bool: absent(opt $T: x) = not occurs(x);
/** @group stdlib.optiontypes True iff \a x is absent */
function var bool: absent(var opt $T: x) = not occurs(x);

/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function $T: deopt($T: x) = x;
/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function var $T: deopt(var $T: x) = x;

/** @group stdlib.optiontypes Return value of \a x if \a x is not absent. Returns 
  undefined when evaluated on absent value. */
function $T: deopt(
	opt $T: x,
) =
	let {
		any: (bx, dx) = mzn_destruct_opt(x);
		constraint bx;
	} in dx;
/** @group stdlib.optiontypes Return value of \a x if \a x is not absent. Returns 
  undefined when evaluated on absent value. */
function var $T: deopt(
	var opt $T: x,
) =
	let {
		any: (bx, dx) = mzn_destruct_opt(x);
		constraint bx;
	} in dx;

/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(var opt $$E: x) = mzn_had_zero(enum2int(x));

function bool: mzn_had_zero(var opt int: x) :: cache_result =
	not mzn_check_absent_zero() \/ (0 in dom(x));

/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(opt $$E: x) = not mzn_check_absent_zero() \/ enum2int(x) = 0;
/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(array [int] of var opt $$E: x) = exists (xi in x) (had_zero(xi));

function tuple(var bool, var float): opt_internal_float(var opt float: x) = mzn_destruct_opt(x);
function var opt $T: reverse_map_var_opt(var bool: b, var $T: dx) :: output_only =
	mzn_construct_opt((b, dx));

/* Construct a var opt set that occurs iff \a b, with value \a s if it occurs. The value of \a s
  must be a subset of \a u. */
function var opt set of $$E: mzn_opt_set(var bool: b, var set of $$E: s, set of $$E: u)
	:: promise_total =
	let {
		var opt set of u: r;
		constraint occurs(r) = b;
		constraint b -> deopt(r) = s;
	} in r;
