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
function $T: deopt(opt $T: x) = let {
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
function array [$U] of $T: deopt(array [$U] of opt $T: x) = arrayXd(x, [deopt(x_i) | x_i in x]);
/** @group stdlib.optiontypes Return value \a x unchanged (since \a x is guaranteed
  to be non-optional). */
function array [$U] of var $T: deopt(array [$U] of var $T: x) = x;
/** @group stdlib.optiontypes Return array of the value of \a x[i] (assumes that \a x[i] is not absent). */
function array [$U] of var $T : deopt(array [$U] of var opt $T : x) = arrayXd(x, [deopt(x_i) | x_i in x]);

/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(var opt int: x) :: cache_result = not mzn_check_absent_zero() \/ (0 in dom(x));
/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(opt int: x) = not mzn_check_absent_zero() \/ (x = 0);
/** @group stdlib.optiontypes.int True if \a x had zero in its original domain.

    Returns true if absent zero representation is disabled or if it is possible
    that \( \text{occurs}(x) \wedge \text{deopt}(x) = 0 \) */
function bool: had_zero(array [int] of var opt int: x) = exists (xi in x) (had_zero(xi));

predicate mzn_absent_zero(var opt int: x) =
  if mzn_check_absent_zero() then
    if had_zero(x) then
      occurs(x) \/ deopt(x) = 0
    else
      occurs(x) <-> deopt(x) != 0
    endif
  endif;

function var opt $T: mzn_construct_opt(tuple(var bool, var $T));
function opt $T: mzn_construct_opt(tuple(bool, $T));
function tuple(var bool, var $T): mzn_destruct_opt(var opt $T);
function tuple(bool, $T): mzn_destruct_opt(opt $T);
