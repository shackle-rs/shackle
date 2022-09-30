/** @group globals.math Constrains \a m to be the minimum of the values in \a x.

  Assumptions: |\a x| > 0.
*/
predicate minimum(var float: m, array[_] of var float: x) =
    array_float_minimum(m, erase_enum(x));

/** @group globals.math Constrains \a m to be the minimum of the values in \a x.

  Assumptions: |\a x| > 0.
*/
predicate minimum(var $$E: m, array[_] of var $$E: x) =
    array_int_minimum(erase_enum(m), erase_enum(x));
