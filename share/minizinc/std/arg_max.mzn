include "fzn_arg_max_int.mzn";
include "fzn_arg_max_int_opt.mzn";
include "fzn_arg_max_bool.mzn";
include "fzn_arg_max_bool_opt.mzn";
include "fzn_arg_max_float.mzn";
include "fzn_arg_max_float_opt.mzn";

/** @group globals.math
	Returns the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var $$V: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var bool: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var float: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var opt $$V: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
	Returns absent when all elements are absent.
*/
function var opt $$E: arg_max_weak(array [$$E] of var opt $$V: x)
	:: promise_total =
	if length(x) = 0 then
		<>
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_int_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, erase_enum(x) ++ [erase_enum(lb_array(x))]);
			var idx: i;
			var opt index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					<>
				endif;
			constraint fzn_maximum_arg_int_opt(xx, i);
		} in ii
	endif;

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var opt bool: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
	Returns absent when all elements are absent.
*/
function var opt $$E: arg_max_weak(array [$$E] of var opt bool: x)
	:: promise_total =
	if length(x) = 0 then
		<>
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_bool_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, x ++ [false]);
			var idx: i;
			var opt index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					<>
				endif;
			constraint fzn_maximum_arg_bool_opt(xx, i);
		} in ii
	endif;

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
*/
function var $$E: arg_max(array [$$E] of var opt float: x) =
	let { constraint length(x) > 0 } in arg_max_total(x);

/** @group globals.math
	Returns the index of the maximum non-absent value in the array \a x.
	When breaking ties the least index is returned.
	Returns absent when all elements are absent.
*/
function var opt $$E: arg_max_weak(array [$$E] of var opt float: x)
	:: promise_total =
	if length(x) = 0 then
		<>
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_float_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, x ++ [lb_array(x)]);
			var idx: i;
			var opt index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					<>
				endif;
			constraint fzn_maximum_arg_float_opt(xx, i);
		} in ii
	endif;

function var $$E: arg_max_total(array [$$E] of var $$V: x) :: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	else
		let {
			var min(index_set(x))..max(index_set(x)): i;
			constraint fzn_maximum_arg_int(erase_enum(x), erase_enum(i));
		} in i
	endif;

function var $$E: arg_max_total(array [$$E] of var bool: x) :: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	else
		let {
			var min(index_set(x))..max(index_set(x)): i;
			constraint fzn_maximum_arg_bool(erase_enum(x), erase_enum(i));
		} in i
	endif;

function var $$E: arg_max_total(array [$$E] of var float: x) :: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	else
		let {
			var min(index_set(x))..max(index_set(x)): i;
			constraint fzn_maximum_arg_float(erase_enum(x), erase_enum(i));
		} in i
	endif;

function var $$E: arg_max_total(array [$$E] of var opt $$V: x)
	 :: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_int_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, erase_enum(x) ++ [erase_enum(lb_array(x))]);
			var idx: i;
			var index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					min(index_set(x))
				endif;
			constraint fzn_maximum_arg_int_opt(xx, i);
		} in ii
	endif;

function var $$E: arg_max_total(array [$$E] of var opt bool: x)
	 :: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_bool_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, x ++ [false]);
			var idx: i;
			var index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					min(index_set(x))
				endif;
			constraint fzn_maximum_arg_bool_opt(xx, i);
		} in ii
	endif;

function var $$E: arg_max_total(array [$$E] of var opt float: x)
	:: promise_total =
	if length(x) = 0 then
		mzn_to_enum(index_set(x), 0)
	elseif exists (xi in x) (is_fixed(occurs(xi)) /\ fix(occurs(xi))) then
		let {
			var index_set(x): i;
			constraint fzn_maximum_arg_float_opt(erase_enum(x), erase_enum(i));
		} in i
	else
		let {
			set of int: idx =
				min(erase_enum(index_set(x))) ..
					max(erase_enum(index_set(x))) + 1;
			any: xx = array1d(idx, x ++ [lb_array(x)]);
			var idx: i;
			var index_set(x): ii =
				if i in erase_enum(index_set(x)) then
					mzn_to_enum(index_set(x), i)
				else
					min(index_set(x))
				endif;
			constraint fzn_maximum_arg_float_opt(xx, i);
		} in ii
	endif;

/** @group globals.math
	Constrain \a i to be the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.

	Assumption: |\a x| > 0
*/
predicate maximum_arg(array [$$E] of var $$V: x, var $$E: i) =
	fzn_maximum_arg_int(erase_enum(x), erase_enum(i));

/** @group globals.math
	Constrain \a i to be the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.

	Assumption: |\a x| > 0
*/
predicate maximum_arg(array [$$E] of var bool: x, var $$E: i) =
	fzn_maximum_arg_bool(erase_enum(x), erase_enum(i));

/** @group globals.math
	Constrain \a i to be the index of the maximum value in the array \a x.
	When breaking ties the least index is returned.
    
	Assumption: |\a x| > 0
*/
predicate maximum_arg(array [$$E] of var float: x, var $$E: i) =
	fzn_maximum_arg_float(erase_enum(x), erase_enum(i));
