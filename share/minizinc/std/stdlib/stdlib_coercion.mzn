/***
  @groupdef stdlib.builtins.coercion Coercions

  These functions implement coercions, or channeling, between different types.
*/

/** @group stdlib.builtins.coercion Return \( \lceil{ \a x} \rceil \) */
function int: ceil(float: x);
/** @group stdlib.builtins.coercion Return \( \lfloor{ \a x} \rfloor \) */
function int: floor(float: x);
/** @group stdlib.builtins.coercion Return \a x rounded to nearest integer */
function int: round(float: x);

/** @group stdlib.builtins.coercion Return Boolean \a b coerced to an integer  */
function int: bool2int(bool: b);

/** @group stdlib.builtins.coercion Return Boolean \a b coerced to an integer  */
function opt int: bool2int(opt bool: b) =
  if occurs(b) then bool2int(deopt(b)) else <> endif;

/** @group stdlib.builtins.coercion Return optional 0/1 integer that is absent iff \a x
  is absent, and 1 iff \a x occurs and is true. */
function var opt int: bool2int(var opt bool: x) ::promise_total =
  let {
    var opt 0..1: xi;
    constraint absent(xi)=absent(x);
    constraint deopt(xi)=bool2int(deopt(x));
  } in xi;


/** @group stdlib.builtins.coercion Return Boolean \a b coerced to a float  */
function float: bool2float(bool: b) = if b then 1.0 else 0.0 endif;
/** @group stdlib.builtins.coercion Return Boolean \a b coerced to a float  */
function opt float: bool2float(opt bool: b) =
  if occurs(b) then bool2float(deopt(b)) else <> endif;

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of floats  */
function array[$T] of float: bool2float(array[$T] of bool: x) ::promise_total =
  let {
    array[int] of bool: xx = array1d(x)
  } in arrayXd(x,[bool2float(xx[i]) | i in index_set(xx)]);
/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of floats  */
function array[$T] of var float: bool2float(array[$T] of var bool: x) ::promise_total =
  let {
    array[int] of var bool: xx = array1d(x)
  } in arrayXd(x,[bool2float(xx[i]) | i in index_set(xx)]);
/** @group stdlib.builtins.coercion Return Boolean \a b coerced to an integer  */
function var int: bool2int(var bool: b) ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Return array of Booleans \a b coerced to an array of integers  */
function array[$T] of var int: bool2int(array[$T] of var bool: b);
/** @group stdlib.builtins.coercion Return Boolean \a b coerced to a float  */
function var float: bool2float(var bool: b) = int2float(bool2int(b));
/** @group stdlib.builtins.coercion Return integer \a x coerced to a float  */
function float: int2float(int: x);
/** @group stdlib.builtins.coercion Return integer \a x coerced to a float  */
function opt float: int2float(opt int: x) =
  if occurs(x) then int2float(deopt(x)) else <> endif;
/** @group stdlib.builtins.coercion Return integer \a x coerced to a float  */
function var float: int2float(var int: x) ::promise_total ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Return integer \a x coerced to a float  */
function set of float: int2float(set of int: x) = {int2float(x_i) | x_i in x};

/** @group stdlib.builtins.coercion Return optional 0/1 float that is absent iff \a x
  is absent, and 1 iff \a x occurs and is true. */
function var opt float: bool2float(var opt bool: x) ::promise_total =
  let {
    var opt 0.0..1.0: xi;
    constraint absent(xi)=absent(x);
    constraint deopt(xi)=bool2float(deopt(x));
  } in xi;
/** @group stdlib.builtins.coercion Return optional 0/1 integer that is absent iff \a x
  is absent, and 1 iff \a x occurs and is true. */
function var opt float: int2float(var opt int: x) ::promise_total =
  let {
    var opt int2float(lb(x))..int2float(ub(x)): xi;
    constraint absent(xi)=absent(x);
    constraint deopt(xi)=int2float(deopt(x));
  } in xi;

function set of int: bool2int(set of bool: b) =
  if b={false,true} then {0,1}
  elseif b={false} then {0}
  elseif b={true} then {1}
  else {} endif;

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of integers  */
function array[$T] of int: bool2int(array[$T] of bool: x) ::promise_total =
  let {
    array[int] of bool: xx = array1d(x)
  } in arrayXd(x,[bool2int(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of sets of Booleans \a x coerced to an array of sets of integers  */
function array[$T] of set of int: bool2int(array[$T] of set of bool: x) ::promise_total =
  let {
    array[int] of set of bool: xx = array1d(x)
  } in arrayXd(x,[bool2int(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of integers  */
function array[$T] of var int: bool2int(array[$T] of var bool: x) ::promise_total =
  let {
    array[int] of var bool: xx = array1d(x)
  } in arrayXd(x,[bool2int(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of integers  */
function array[$T] of opt int: bool2int(array[$T] of opt bool: x) =
  let {
    array[int] of opt bool: xx = array1d(x)
  } in arrayXd(x,[bool2int(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of integers  */
function array[$T] of var opt int: bool2int(array[$T] of var opt bool: x) ::promise_total =
  let {
    array[int] of var opt bool: xx = array1d(x)
  } in arrayXd(x,[bool2int(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of floats  */
function array[$T] of opt float: bool2float(array[$T] of opt bool: x) =
  let {
    array[int] of opt bool: xx = array1d(x)
  } in arrayXd(x, [bool2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of Booleans \a x coerced to an array of floats  */
function array[$T] of var opt float: bool2float(array[$T] of var opt bool: x) ::promise_total =
  let {
    array[int] of var opt bool: xx = array1d(x)
  } in arrayXd(x,[bool2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of integers \a x coerced to an array of floats  */
function array[$T] of float: int2float(array[$T] of int: x) ::promise_total =
  let {
    array[int] of int: xx = array1d(x)
  } in arrayXd(x,[int2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of integers \a x coerced to an array of floats  */
function array[$T] of opt float: int2float(array[$T] of opt int: x) ::promise_total =
  let {
    array[int] of opt int: xx = array1d(x)
  } in arrayXd(x,[int2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of integers \a x coerced to an array of floats  */
function array[$T] of var float: int2float(array[$T] of var int: x) ::promise_total =
  let {
    array[int] of var int: xx = array1d(x)
  } in arrayXd(x,[int2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Return array of optional integers \a x coerced to an array of optional floats  */
function array[$T] of var opt float: int2float(array[$T] of var opt int: x) ::promise_total =
  let {
    array[int] of var opt int: xx = array1d(x)
  } in arrayXd(x,[int2float(xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an integer  */
function var int: erase_enum(var $$E) ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an integer  */
function int: erase_enum($$E) ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an integer  */
function var opt int: erase_enum(var opt $$E) ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an integer  */
function opt int: erase_enum(opt $$E) ::mzn_internal_representation;

/** @group stdlib.builtins.coercion Erase enum information by converting \x to be a set of integers  */
function var set of int: erase_enum(var set of $$E: x) ::mzn_internal_representation;
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be a set of integers  */
function set of int: erase_enum(set of $$E: x) = {erase_enum(i) | i in x};

/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of bool: erase_enum(array [_] of bool: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of bool: erase_enum(array [_, _] of bool: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of opt bool: erase_enum(array [_] of opt bool: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of opt bool: erase_enum(array [_, _] of opt bool: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of var bool: erase_enum(array [_] of var bool: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of var bool: erase_enum(array [_, _] of var bool: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of var opt bool: erase_enum(array [_] of var opt bool: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of var opt bool: erase_enum(array [_, _] of var opt bool: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];

/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of var int: erase_enum(array [_] of var $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of var int: erase_enum(array [_, _] of var $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of int: erase_enum(array [_] of $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of int: erase_enum(array [_, _] of $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of var opt int: erase_enum(array [_] of var opt $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of var opt int: erase_enum(array [_, _] of var opt $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of opt int: erase_enum(array [_] of opt $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of opt int: erase_enum(array [_, _] of opt $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];

/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of float: erase_enum(array [_] of float: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of float: erase_enum(array [_, _] of float: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of opt float: erase_enum(array [_] of opt float: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of opt float: erase_enum(array [_, _] of opt float: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of var float: erase_enum(array [_] of var float: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of var float: erase_enum(array [_, _] of var float: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int] of var opt float: erase_enum(array [_] of var opt float: x) = [erase_enum(i): x[i] | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum index set information  */
function array [int, int] of var opt float: erase_enum(array [_, _] of var opt float: x) = [(erase_enum(i), erase_enum(j)): x[i, j] | i in index_set_1of2(x), j in index_set_2of2(x)];

/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of var set of int: erase_enum(array [_] of var set of $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of var set of int: erase_enum(array [_, _] of var set of $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int] of set of int: erase_enum(array [_] of set of $$E: x) = [erase_enum(i): erase_enum(x[i]) | i in index_set(x)];
/** @group stdlib.builtins.coercion Erase enum information by converting \x to be an array of integers with integer index sets  */
function array [int, int] of set of int: erase_enum(array [_, _] of set of $$E: x) = [(erase_enum(i), erase_enum(j)): erase_enum(x[i, j]) | i in index_set_1of2(x), j in index_set_2of2(x)];

/** @group stdlib.builtins.coercion Return a set of integers \a x coerced to an array of integers  */
function array[int] of $$E: set2array(set of $$E: x);

/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function $T: 'default'($T: x, $T: y) = let {
  bool: defined = let { any: xx = x } in true;
} in if defined then x else y endif;
/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function var $T: 'default'(var $T: x, var $T: y) = let {
  var bool: defined = let { any: xx = x } in true;
} in if defined then x else y endif;
/** @group stdlib.debug If \a x is defined and not absent, return \a x, otherwise return \a y. */
function $T: 'default'(opt $T: x, $T: y) = let {
  bool: defined = let { any: xx = x } in true;
} in if defined /\ occurs(x) then deopt(x) else y endif;
/** @group stdlib.debug If \a x is defined and not absent, return \a x, otherwise return \a y. */
function opt $T: 'default'(opt $T: x, opt $T: y) = let {
  bool: defined = let { any: xx = x } in true;
} in if defined /\ occurs(x) then x else y endif;
/** @group stdlib.debug If \a x is defined and not absent, return \a x, otherwise return \a y. */
function var $T: 'default'(var opt $T: x, var $T: y) = let {
  var bool: defined = let { any: xx = x } in true;
} in if defined /\ occurs(x) then deopt(x) else y endif;
/** @group stdlib.debug If \a x is defined and not absent, return \a x, otherwise return \a y. */
function var opt $T: 'default'(var opt $T: x, var opt $T: y) = let {
  var bool: defined = let { any: xx = x } in true;
} in if defined /\ occurs(x) then x else y endif;

/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function array[$U] of $T: 'default'(array[$U] of $T: x, array[$U] of $T: y) = let {
  bool: defined = let { any: xx = x } in true;
} in if defined then x else y endif;
/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function array[$U] of var $T: 'default'(array[$U] of var $T: x, array[$U] of var $T: y) = let {
  var bool: defined = let { any: xx = x } in true;
  any: yy = array1d(y);
} in arrayXd(y, [if defined then array1d(x)[i] else yy[i] endif | i in index_set(yy)]);
/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function array[$U] of opt $T: 'default'(array[$U] of opt $T: x, array[$U] of opt $T: y) = let {
  bool: defined = let { any: xx = x } in true;
} in if defined then x else y endif;
/** @group stdlib.debug If \a x is defined, return \a x, otherwise return \a y. */
function array[$U] of var opt $T: 'default'(array[$U] of var opt $T: x, array[$U] of var opt $T: y) = let {
  var bool: defined = let { any: xx = x } in true;
  any: yy = array1d(y);
} in arrayXd(y, [if defined then array1d(x)[i] else yy[i] endif | i in index_set(yy)]);
