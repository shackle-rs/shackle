/***
  @groupdef stdlib.builtins.enum Functions for enums
*/

%TODO: Document these
function set of int: anon_enum(int: n) = 1..n;

function set of int: anon_enum_set(set of int: S) = 1..card(S);

function set of int: anon_enum(array[int] of string: x) ::mzn_internal_representation;

/** @group stdlib.builtins.enum Return enum set of \a x */
function set of $$E: enum_of(var opt $$E: x) ::mzn_internal_representation;
/** @group stdlib.builtins.enum Return enum set of \a x */
function set of $$E: enum_of(var set of $$E: x) ::mzn_internal_representation;
/** @group stdlib.builtins.enum Return enum set of \a x */
function set of $$E: enum_of(array[$T] of var opt $$E: x) ::mzn_internal_representation;
/** @group stdlib.builtins.enum Return enum set of \a x */
function set of $$E: enum_of(array[$T] of var set of $$E: x) ::mzn_internal_representation;

function set of $$E: enum_of_internal(set of $$E: e) = e;

/** @group stdlib.builtins.enum Return next greater enum value of \a x */
function $$E: enum_next($$E: x) = enum_next(enum_of(x), x);
/** @group stdlib.builtins.enum Return next greater enum value of \a x */
function opt $$E: enum_next(opt $$E: x) = enum_next(enum_of(x), x);
/** @group stdlib.builtins.enum Return next greater enum value of \a x */
function var $$E: enum_next(var $$E: x) = enum_next(enum_of(x), x);
/** @group stdlib.builtins.enum Return next greater enum value of \a x */
function var opt $$E: enum_next(var opt $$E: x) = enum_next(enum_of(x), x);

/** @group stdlib.builtins.enum Return next greater enum value of \a x in enum \a e */
function $$E: enum_next(set of $$E: e, $$E: x);
/** @group stdlib.builtins.enum Return next greater enum value of \a x in enum \a e */
function opt $$E: enum_next(set of $$E: e, opt $$E: x) =
  if occurs(x) then enum_next(e,deopt(x)) else <> endif;
/** @group stdlib.builtins.enum Return next greater enum value of \a x in enum \a e */
function var $$E: enum_next(set of $$E: e, var $$E: x) =
  let { constraint erase_enum(x) < max(erase_enum(e)) } in to_enum(e, erase_enum(x)+1);
/** @group stdlib.builtins.enum Return next greater enum value of \a x in enum \a e */
function var opt $$E: enum_next(set of $$E: e, var opt $$E: x) =
  if occurs(x) then enum_next(e,deopt(x)) else <> endif;

/** @group stdlib.builtins.enum Return next smaller enum value of \a x */
function $$E: enum_prev($$E: x) = enum_prev(enum_of(x), x);
/** @group stdlib.builtins.enum Return next smaller enum value of \a x */
function opt $$E: enum_prev(opt $$E: x) = enum_prev(enum_of(x), x);
/** @group stdlib.builtins.enum Return next smaller enum value of \a x */
function var $$E: enum_prev(var $$E: x) = enum_prev(enum_of(x), x);
/** @group stdlib.builtins.enum Return next smaller enum value of \a x */
function var opt $$E: enum_prev(var opt $$E: x) = enum_prev(enum_of(x), x);

/** @group stdlib.builtins.enum Return next smaller enum value of \a x in enum \a e */
function $$E: enum_prev(set of $$E: e, $$E: x);
/** @group stdlib.builtins.enum Return next smaller enum value of \a x in enum \a e */
function opt $$E: enum_prev(set of $$E: e, opt $$E: x) =
  if occurs(x) then enum_prev(e,deopt(x)) else <> endif;
/** @group stdlib.builtins.enum Return next smaller enum value of \a x in enum \a e */
function var $$E: enum_prev(set of $$E: e, var $$E: x) =
  let { constraint erase_enum(x) > min(erase_enum(e)) } in to_enum(e, erase_enum(x)-1);
/** @group stdlib.builtins.enum Return next smaller enum value of \a x in enum \a e */
function var opt $$E: enum_prev(set of $$E: e, var opt $$E: x) =
  if occurs(x) then enum_prev(e,deopt(x)) else <> endif;

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function $$E: to_enum(set of $$E: X, int: x);
/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function opt $$E: to_enum(set of $$E: X, opt int: x) =
  if occurs(x) then to_enum(X, deopt(x)) else <> endif;
/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function var $$E: to_enum(set of $$E: X, var int: x) =
  let { var X: xx; constraint erase_enum(xx) = x; } in xx;
/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function var opt $$E: to_enum(set of $$E: X, var opt int: x) =
  if occurs(x) then to_enum(X, deopt(x)) else <> endif;
  
/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of $$E: to_enum(set of $$E: X, array[$U] of int: x) =
  let { array[int] of int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X,xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of opt $$E: to_enum(set of $$E: X, array[$U] of opt int: x) =
  let { array[int] of opt int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X,xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of var $$E: to_enum(set of $$E: X, array[$U] of var int: x) =
  let { array[int] of var int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X,xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of var opt $$E: to_enum(set of $$E: X, array[$U] of var opt int: x) =
  let { array[int] of var opt int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X,xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function set of $$E: to_enum(set of $$E: X, set of int: x) = { to_enum(X,i) | i in x };

%/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function var set of $$E: to_enum(set of $$E: X, var set of int: x) = 
  let { var set of X: y;
        constraint x subset erase_enum(X);
        constraint forall (i in X) (erase_enum(i) in x <-> i in y);
      } in y;

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of set of $$E: to_enum(set of $$E: X, array[$U] of set of int: x) =
  let { array[int] of set of int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X, xx[i]) | i in index_set(xx)]);

/** @group stdlib.builtins.enum Convert \a x to enum type \a X */
function array[$U] of var set of $$E: to_enum(set of $$E: X, array[$U] of var set of int: x) =
  let { array[int] of var set of int: xx = array1d(x) } in
  arrayXd(x, [ to_enum(X, xx[i]) | i in index_set(xx)]);

function int: mzn_enum_constructor(array [int] of int: prev_card, array [int] of tuple(set of int, int): x) =
  sum(prev_card) + sum (i in index_set(x)) (
    let {
      int: p = product (j in i + 1..max(index_set(x))) (card(x[j].1));
      int: v =
        if mzn_set_is_contiguous(x[i].1) then
          x[i].2 - min(x[i].1)
        else
          count (e in x[i].1) (e <= x[i].2)
        endif
    } in p * v
  ) + 1;

function var int: mzn_enum_constructor(array [int] of int: prev_card, array [int] of tuple(set of int, var int): x) =
  sum(prev_card) + sum (i in index_set(x)) (
    let {
      int: p = product (j in i + 1..max(index_set(x))) (card(x[j].1));
      var int: v =
        if mzn_set_is_contiguous(x[i].1) then
          x[i].2 - min(x[i].1)
        else
          count (e in x[i].1) (e <= x[i].2)
        endif
    } in p * v
  ) + 1;

function opt int: mzn_enum_constructor(array [int] of int: prev_card, array [int] of tuple(set of int, opt int): x) =
  if forall (x_i in x) (occurs(x_i.2)) then
    mzn_enum_constructor(prev_card, [(x_i.1, deopt(x_i.2)) | x_i in x])
  else
    <>
  endif;

function var opt int: mzn_enum_constructor(array [int] of int: prev_card, array [int] of tuple(set of int, var opt int): x) =
  if forall (x_i in x) (occurs(x_i.2)) then
    mzn_enum_constructor(prev_card, [(x_i.1, deopt(x_i.2)) | x_i in x])
  else
    <>
  endif;

function array [int] of int: mzn_enum_destructor(array [int] of int: prev_card, array [int] of set of int: parts, int: value) = [
  let {
    int: d = (value - sum(prev_card) - 1) div product (j in i + 1..max(index_set(parts))) (card(parts[j])) mod card(parts[i]);
    int: v = 
      if card(parts[i]) = max(parts[i]) - min(parts[i]) + 1 then
        min(parts[i]) + d
      else
        set2array(parts[i])[d + 1]
      endif;
  } in v
  | i in index_set(parts)
];

function array [int] of var int: mzn_enum_destructor(array [int] of int: prev_card, array [int] of set of int: parts, var int: value) = [
  let {
    var int: d = (value - sum(prev_card) - 1) div product (j in i + 1..max(index_set(parts))) (card(parts[j])) mod card(parts[i]);
    var int: v = 
      if card(parts[i]) = max(parts[i]) - min(parts[i]) + 1 then
        min(parts[i]) + d
      else
        set2array(parts[i])[d + 1]
      endif;
  } in v
  | i in index_set(parts)
];

function array [int] of opt int: mzn_enum_destructor(array [int] of int: prev_card, array [int] of set of int: parts, opt int: value) =
  if occurs(value) then
    mzn_enum_destructor(prev_card, parts, deopt(value))
  else
    [<> | i in index_set(parts)]
  endif;

function array [int] of var opt int: mzn_enum_destructor(array [int] of int: prev_card, array [int] of set of int: parts, var opt int: value) = [
  if occurs(value) then
    mzn_enum_destructor(prev_card, parts, deopt(value))[i]
  else
    <>
  endif
  | i in index_set(parts)
];
