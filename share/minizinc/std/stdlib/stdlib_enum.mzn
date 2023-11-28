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
function $$E: to_enum(set of $$E: X, int: x) = let {
  constraint mzn_assert_warn(x in erase_enum(X), "The value \(x) is outside the enum range.");
} in mzn_to_enum(X, x);

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

type MznEnum = tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int));

function MznEnum: mzn_get_enum(array [int] of tuple(string, array [int] of tuple(int, set of int)): e) = (
  sum (x in e) (product (y in x.2) (card(y.2))),
  [
    let {
      any: (name, params) = e[i];
      any: offset = sum (x in e[1..i - 1]) (product (y in x.2) (card(y.2)));
    } in (name, params, offset)
    | i in 1..length(e)
  ]
);
function set of int: mzn_defining_set(MznEnum: x) = 1..x.1;

function int: mzn_construct_enum(MznEnum: e, int: i) = e.2[i].3 + 1;

function int: mzn_construct_enum(MznEnum: e, int: i, array [int] of int: args) = let {
  any: (name, params, offset) = e.2[i];
  constraint forall (a in index_set(args) where params[a].1 == 0) (
    mzn_assert_warn(args[a] in params[a].2, "The value for argument \(a) of the enum constructor '" ++ name ++ "' is outside the declared range of \(params[a].2).")
  );
  any: value = sum (a in index_set(args)) (let {
      any: p = product (b in a + 1..max(index_set(args))) (card(params[b].2));
      any: v =
        if mzn_set_is_contiguous(params[a].2) then
          args[a] - min(params[a].2)
        else
          count (z in params[a].2) (z < args[a])
        endif
  } in p * v);
} in offset + value + 1;

function opt int: mzn_construct_enum(MznEnum: e, int: i, array [int] of opt int: args) =
  if forall (a in args) (occurs(a)) then
    mzn_construct_enum(e, i, [deopt(a) | a in args])
  else
    <>
  endif;

function var int: mzn_construct_enum(MznEnum: e, int: i, array [int] of var int: args) = let {
  any: (name, params, offset) = e.2[i];
  constraint forall (a in index_set(args) where params[a].1 == 0) (args[a] in params[a].2);
  any: value = sum (a in index_set(args)) (let {
      any: p = product (b in a + 1..max(index_set(args))) (card(params[b].2));
      any: v =
        if mzn_set_is_contiguous(params[a].2) then
          args[a] - min(params[a].2)
        else
          count (z in params[a].2) (z < args[a])
        endif
  } in p * v);
} in offset + value + 1;

function var opt int: mzn_construct_enum(MznEnum: e, int: i, array [int] of var opt int: args) =
  if forall (a in args) (occurs(a)) then
    mzn_construct_enum(e, i, [deopt(a) | a in args])
  else
    <>
  endif;

function set of int: mzn_construct_enum(MznEnum: e, int: i, array [int] of set of int: args) =
  { mzn_construct_enum(e, i, v) | (v,) in mzn_cartesian_product(args) };

function var set of int: mzn_construct_enum(MznEnum: e, int: i, array [int] of var set of int: args) =
  array2set([
    if forall (idx in index_set(args)) (v[idx] in args[idx]) then
      mzn_construct_enum(e, i, v)
    else
      <>
    endif
    | (v,) in mzn_cartesian_product([ub(a) | a in args])
  ]);

function array [int] of int: mzn_destruct_enum(MznEnum: e, int: i, int: v) = let {
  any: (name, params, offset) = e.2[i];
  any: lower = offset + 1;
  any: upper = if i == length(e.2) then e.1 else e.2[i + 1].3 endif;
  constraint mzn_assert_warn(v in lower..upper, "Cannot destructure value using " ++ name ++ "(...).");
} in [
  let {
    any: d = (v - offset - 1) div product (j in p + 1..max(index_set(params))) (card(params[j].2)) mod card(params[p].2);
    any: r = 
      if mzn_set_is_contiguous(params[p].2) then
        min(params[p].2) + d
      else
        set2array(params[p].2)[d + 1]
      endif;
  } in r
  | p in index_set(params)
];

function array [int] of var int: mzn_destruct_enum(MznEnum: e, int: i, var int: v) = let {
  any: (name, params, offset) = e.2[i];
  any: lower = offset + 1;
  any: upper = if i == length(e.2) then e.1 else e.2[i + 1].3 endif;
  constraint v in lower..upper;
} in [
  let {
    any: d = (v - offset - 1) div product (j in p + 1..max(index_set(params))) (card(params[j].2)) mod card(params[p].2);
    any: r = 
      if mzn_set_is_contiguous(params[p].2) then
        min(params[p].2) + d
      else
        set2array(params[p].2)[d + 1]
      endif;
  } in r
  | p in index_set(params)
];

function array [int] of opt int: mzn_destruct_enum(MznEnum: e, int: i, opt int: v) =
  if occurs(v) then
    mzn_destruct_enum(e, i, deopt(v))
  else
    [<> | p in e.2[i].2]
  endif;

function array [int] of var opt int: mzn_destruct_enum(MznEnum: e, int: i, var opt int: v) = [
  if occurs(v) then
    mzn_destruct_enum(e, i, deopt(v))[p]
  else
    <>
  endif
  | p in index_set(e.2[i].2)
];

function array [int] of set of int: mzn_destruct_enum(MznEnum: e, int: i, set of int: v) = [
  { mzn_destruct_enum(e, i, v_i)[p] | v_i in v }
  | p in index_set(e.2[i].2)
];

function array [int] of var set of int: mzn_destruct_enum(MznEnum: e, int: i, var set of int: v) = [
  { mzn_destruct_enum(e, i, v_i)[p] | v_i in v }
  | p in index_set(e.2[i].2)
];

function tuple(string, array [int] of tuple(int, int)): mzn_unpack_enum(MznEnum: e, int: v) = let {
  any: i = arg_max([v <= i.3 | i in e.2] ++ [true]) - 1;
  any: (name, def, offset) = e.2[i];
  any: args = if length(def) > 0 then let {
    any: d = mzn_destruct_enum(e, i, v);
    any: s = [(def[idx].1, d[idx]) | idx in index_set(d)];
  } in s else [] endif;
} in (name, args);

function string: mzn_show_enum(array [int] of MznEnum: enums, int: e, int: v) = let {
  any: (name, ctors) = mzn_unpack_enum(enums[e], v);
  any: args = if length(ctors) > 0 then "(" ++ join(", ", [
    if ctor.1 == 0 then
      show(ctor.2)
    else
      mzn_show_enum(enums, ctor.1, ctor.2)
    endif
    | ctor in ctors
  ]) ++ ")" endif;
} in name ++ args;

function array [int] of tuple(array [int] of int): mzn_cartesian_product(array [int] of set of int: x, int: i) =
  if length(x) == 0 then
    []
  elseif i == max(index_set(x)) then
    [([j],) | j in x[i]]
  else
    [([j] ++ k,) | j in x[i], (k,) in mzn_cartesian_product(x, i + 1)]
  endif;

function array [int] of tuple(array [int] of int): mzn_cartesian_product(array [int] of set of int: x) = mzn_cartesian_product(x, 1);

function $$E: mzn_to_enum(set of $$E: X, int: x);
function var $$E: mzn_to_enum(set of $$E: X, var int: x);
function set of $$E: mzn_to_enum(set of $$E: X, set of int: x);
function var set of $$E: mzn_to_enum(set of $$E: X, var set of int: x);
