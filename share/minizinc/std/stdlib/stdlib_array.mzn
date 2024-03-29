/***
  @groupdef stdlib.builtins.array Array operations

  These functions implement the basic operations on arrays.
*/

/** @group stdlib.builtins.array Return \a x[\a idx] */
function any $T: '[]'(array [$E] of any $T: x, $E: idx) :: mzn_inline =
	let {
		constraint mzn_array_access_valid(x, idx);
	} in mzn_element_internal(x, idx);
/** @group stdlib.builtins.array Return \a x[\a idx] */
function var $T: '[]'(array [$E] of var $T: x, var $E: idx) :: mzn_inline =
	if mzn_array_access_known_valid(x, idx) then
		element_total(x, idx)
	else
		element(x, idx)
	endif;
/** @group stdlib.builtins.array Return \a x[\a idx] */
function var opt $T: '[]'(array [$E] of var opt $T: x, var $E: idx)
	:: mzn_inline = mzn_construct_opt('[]'(mzn_destruct_opt(x), idx));

/** @group stdlib.builtins.array Return the concatenation of arrays \a x and \a y */
function array [int] of any $T: '++'(
	array [$$E] of any $T: x,
	array [$$F] of any $T: y,
) :: mzn_internal_representation;

/** @group stdlib.builtins.array Return the length of array \a x

  Note that the length is defined as the number of elements in the
  array, regardless of its dimensionality.
*/
function int: length(array [$T] of any $U: x);

/** @group stdlib.builtins.array Return the array \a x in reverse order

  The resulting array has the same index set as \a x.
*/
function array [$$E] of any $T: reverse(array [$$E] of any $T: x) =
	if length(x) = 0 then
		[]
	else
		let {
			array [int] of any $T: xx = array1d(x);
			int: l = length(x) + 1;
		} in array1d(index_set(x), [xx[l - i] | i in index_set(xx)])
	endif;

/** @group stdlib.builtins.array Test if \a x and \a y have the same index sets */
test index_sets_agree(array [$T] of any $U: x, array [$T] of any $W: y);

/** @group stdlib.builtins.array Return index set of one-dimensional array \a x */
function set of $$E: index_set(array [$$E] of any $U: x) = index_sets(x).1;
/** @group stdlib.builtins.array Return index set of first dimension of two-dimensional array \a x */
function set of $$E: index_set_1of2(array [$$E, _] of any $U: x) =
	index_sets(x).1;
/** @group stdlib.builtins.array Return index set of second dimension of two-dimensional array \a x */
function set of $$E: index_set_2of2(array [_, $$E] of any $U: x) =
	index_sets(x).2;
/** @group stdlib.builtins.array Return index set of first dimension of 3-dimensional array \a x */
function set of $$E: index_set_1of3(array [$$E, _, _] of any $U: x) =
	index_sets(x).1;
/** @group stdlib.builtins.array Return index set of second dimension of 3-dimensional array \a x */
function set of $$E: index_set_2of3(array [_, $$E, _] of any $U: x) =
	index_sets(x).2;
/** @group stdlib.builtins.array Return index set of third dimension of 3-dimensional array \a x */
function set of $$E: index_set_3of3(array [_, _, $$E] of any $U: x) =
	index_sets(x).3;
/** @group stdlib.builtins.array Return index set of first dimension of 4-dimensional array \a x */
function set of $$E: index_set_1of4(array [$$E, _, _, _] of any $U: x) =
	index_sets(x).1;
/** @group stdlib.builtins.array Return index set of second dimension of 4-dimensional array \a x */
function set of $$E: index_set_2of4(array [_, $$E, _, _] of any $U: x) =
	index_sets(x).2;
/** @group stdlib.builtins.array Return index set of third dimension of 4-dimensional array \a x */
function set of $$E: index_set_3of4(array [_, _, $$E, _] of any $U: x) =
	index_sets(x).3;
/** @group stdlib.builtins.array Return index set of fourth dimension of 4-dimensional array \a x */
function set of $$E: index_set_4of4(array [_, _, _, $$E] of any $U: x) =
	index_sets(x).4;
/** @group stdlib.builtins.array Return index set of first dimension of 5-dimensional array \a x */
function set of $$E: index_set_1of5(array [$$E, _, _, _, _] of any $U: x) =
	index_sets(x).1;
/** @group stdlib.builtins.array Return index set of second dimension of 5-dimensional array \a x */
function set of $$E: index_set_2of5(array [_, $$E, _, _, _] of any $U: x) =
	index_sets(x).2;
/** @group stdlib.builtins.array Return index set of third dimension of 5-dimensional array \a x */
function set of $$E: index_set_3of5(array [_, _, $$E, _, _] of any $U: x) =
	index_sets(x).3;
/** @group stdlib.builtins.array Return index set of fourth dimension of 5-dimensional array \a x */
function set of $$E: index_set_4of5(array [_, _, _, $$E, _] of any $U: x) =
	index_sets(x).4;
/** @group stdlib.builtins.array Return index set of fifth dimension of 5-dimensional array \a x */
function set of $$E: index_set_5of5(array [_, _, _, _, $$E] of any $U: x) =
	index_sets(x).5;
/** @group stdlib.builtins.array Return index set of first dimension of 6-dimensional array \a x */
function set of $$E: index_set_1of6(array [$$E, _, _, _, _, _] of any $U: x) =
	index_sets(x).1;
/** @group stdlib.builtins.array Return index set of second dimension of 6-dimensional array \a x */
function set of $$E: index_set_2of6(array [_, $$E, _, _, _, _] of any $U: x) =
	index_sets(x).2;
/** @group stdlib.builtins.array Return index set of third dimension of 6-dimensional array \a x */
function set of $$E: index_set_3of6(array [_, _, $$E, _, _, _] of any $U: x) =
	index_sets(x).3;
/** @group stdlib.builtins.array Return index set of fourth dimension of 6-dimensional array \a x */
function set of $$E: index_set_4of6(array [_, _, _, $$E, _, _] of any $U: x) =
	index_sets(x).4;
/** @group stdlib.builtins.array Return index set of fifth dimension of 6-dimensional array \a x */
function set of $$E: index_set_5of6(array [_, _, _, _, $$E, _] of any $U: x) =
	index_sets(x).5;
/** @group stdlib.builtins.array Return index set of sixth dimension of 6-dimensional array \a x */
function set of $$E: index_set_6of6(array [_, _, _, _, _, $$E] of any $U: x) =
	index_sets(x).6;

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E): index_sets(array [$$E] of any $U: x);
/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F): index_sets(
	array [$$E, $$F] of any $U: x,
);
/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G): index_sets(
	array [$$E, $$F, $$G] of any $U: x,
);
/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G, set of $$H): index_sets(
	array [$$E, $$F, $$G, $$H] of any $U: x,
);
/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(
	set of $$E,
	set of $$F,
	set of $$G,
	set of $$H,
	set of $$I,
): index_sets(array [$$E, $$F, $$G, $$H, $$I] of any $U: x);
/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(
	set of $$E,
	set of $$F,
	set of $$G,
	set of $$H,
	set of $$I,
	set of $$J,
): index_sets(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x);

/** @group stdlib.builtins.array Return array \a x coerced to index set 1..length(\a x).
  Coercions are performed by considering the array \a x in row-major order. */
function array [int] of any $V: array1d(array [$U] of any $V: x);

/** @group stdlib.builtins.array Return array \a x coerced to one-dimensional array with index set \a S.
  Coercions are performed by considering the array \a x in row-major order.  */
function array [$$E] of any $V: array1d(set of $$E: S, array [$U] of any $V: x);

/** @group stdlib.builtins.array Return array \a x coerced to two-dimensional array with index sets \a S1 and \a S2.
  Coercions are performed by considering the array \a x in row-major order.  */
function array [$$E, $$F] of any $V: array2d(
	set of $$E: S1,
	set of $$F: S2,
	array [$U] of any $V: x,
);

function array [$$E, $$F] of any $V: array2d(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of any $V: x,
) =
	let {
		int: d = length(S2);
	} in [
		(S1[(i - 1) div d + 1], S2[(i - 1) mod d + 1]): x[i] |
			i in index_set(x),
	];

function array [$$E] of any $V: arrayNd($$E: S1, array [int] of any $V: x) =
	array1d(S1..to_enum(enum_of(S1), erase_enum(S1) + length(x) - 1), x);
function array [$$E] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of any $V: x,
) = [S1[i]: x[i] | i in index_set(x)];
function array [$$E, $$F] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of any $V: x,
) = [(S1[i], S2[i]): x[i] | i in index_set(x)];
function array [$$E, $$F, $$G] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of $$G: S3,
	array [int] of any $V: x,
) = [(S1[i], S2[i], S3[i]): x[i] | i in index_set(x)];
function array [$$E, $$F, $$G, $$H] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of $$G: S3,
	array [int] of $$H: S4,
	array [int] of any $V: x,
) = [(S1[i], S2[i], S3[i], S4[i]): x[i] | i in index_set(x)];
function array [$$E, $$F, $$G, $$H, $$I] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of $$G: S3,
	array [int] of $$H: S4,
	array [int] of $$I: S5,
	array [int] of any $V: x,
) = [(S1[i], S2[i], S3[i], S4[i], S5[i]): x[i] | i in index_set(x)];
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $V: arrayNd(
	array [int] of $$E: S1,
	array [int] of $$F: S2,
	array [int] of $$G: S3,
	array [int] of $$H: S4,
	array [int] of $$I: S5,
	array [int] of $$J: S6,
	array [int] of any $V: x,
) = [(S1[i], S2[i], S3[i], S4[i], S5[i], S6[i]): x[i] | i in index_set(x)];

/** @group stdlib.builtins.array Return array \a x coerced to three-dimensional array with
  index sets \a S1, \a S2 and \a S3.
  Coercions are performed by considering the array \a x in row-major order. 
*/
function array [$$E, $$F, $$G] of any $V: array3d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	array [$U] of any $V: x,
);

/** @group stdlib.builtins.array Return array \a x coerced to 4-dimensional array with
  index sets \a S1, \a S2, \a S3 and \a S4.
  Coercions are performed by considering the array \a x in row-major order. 
*/
function array [$$E, $$F, $$G, $$H] of any $V: array4d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	array [$U] of any $V: x,
);

/** @group stdlib.builtins.array Return array \a x coerced to 5-dimensional array with
  index sets \a S1, \a S2, \a S3, \a S4 and \a S5.
  Coercions are performed by considering the array \a x in row-major order. 
*/
function array [$$E, $$F, $$G, $$H, $$I] of any $V: array5d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	set of $$I: S5,
	array [$U] of any $V: x,
);

/** @group stdlib.builtins.array Return array \a x coerced to 6-dimensional array with
  index sets \a S1, \a S2, \a S3, \a S4, \a S5 and \a S6.
  Coercions are performed by considering the array \a x in row-major order. 
*/
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $V: array6d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	set of $$I: S5,
	set of $$J: S6,
	array [$U] of any $V: x,
);

/** @group stdlib.builtins.array Forces a arrayNd call to throw an error if the new index sets are offsets of the current index sets.
*/
annotation array_check_form;

/** @group stdlib.builtins.array Return array \a y coerced to array with same number of
  dimensions and same index sets as array \a x.
  Coercions are performed by considering the array \a y in row-major order. 
*/
function array [$T] of any $V: arrayXd(
	array [$T] of any $X: x,
	array [$U] of any $V: y,
);

/** @group stdlib.builtins.array Return row \a r of array \a x */
function array [$$E] of any $T: row(array [int, $$E] of any $T: x, int: r) =
	x[r, ..];

/** @group stdlib.builtins.array Return column \a c of array \a x */
function array [$$E] of any $T: col(array [$$E, int] of any $T: x, int: c) =
	x[.., c];

/** @group stdlib.builtins.array Test if \a i is in the index set of \a x */
test has_index($$E: i, array [$$E] of any $T: x) = i in index_set(x);
/** @group stdlib.builtins.array Test if \a e is an element of array \a x */
test has_element($T: e, array [$X] of $T: x) = exists (x_i in x) (x_i = e);
/** @group stdlib.builtins.array Test if \a e is an element of array \a x */
test has_element($T: e, array [$X] of opt $T: x) = exists (x_i in x) (x_i = e);
/** @group stdlib.builtins.array Test if \a e is an element of array \a x */
predicate has_element($T: e, array [$X] of any $T: x) =
	exists (x_i in x) (x_i = e);

/** @group stdlib.builtins.array Return the set containing the elements of \a x */
function var set of $$T: array2set(array [int] of var $$T: x) :: promise_total =
	mzn_to_enum(enum_of(x), array2set(erase_enum(x)));
function var set of int: array2set(array [int] of var int: x) :: promise_total =
	if length(x) = 0 then
		{}
	else
		let {
			set of int: D = dom_array(x);
			constraint
				assert(
					min(D) > -infinity /\ max(D) < infinity,
					"array2set needs finite bounds on argument array",
				);
			var set of D: y;
			constraint fzn_array_int_union(x, y);
		} in y
	endif;

function var set of $$T: array2set(array [int] of var opt $$T: x)
	:: promise_total = mzn_to_enum(enum_of(x), array2set(erase_enum(x)));
function var set of int: array2set(array [int] of var opt int: x)
	:: promise_total =
	if length(x) = 0 then
		{}
	else
		let {
			set of int: D = dom_array_occurring(x);
			constraint
				assert(
					min(D) > -infinity /\ max(D) < infinity,
					"array2set needs finite bounds on argument array",
				);
			var set of D: y;
			constraint fzn_array_opt_int_union(x, y);
		} in y
	endif;

/** @group stdlib.builtins.array Return the set containing the elements of \a x */
function set of $$T: array2set(array [int] of $$T: x) = {
	x[i] |
		i in index_set(x),
};
function set of int: array2set(array [int] of int: x) = {
	x[i] |
		i in index_set(x),
};

/** @group stdlib.builtins.array Return the set containing the elements of \a x */
function set of $$T: array2set(array [int] of opt $$T: x) = {
	deopt(x[i]) |
		i in index_set(x) where occurs(x[i]),
};
function set of int: array2set(array [int] of opt int: x) = {
	deopt(x[i]) |
		i in index_set(x) where occurs(x[i]),
};

/** @group stdlib.builtins.array Return the set containing the elements of \a x */
function set of bool: array2set(array [int] of bool: x) =
	let {
		bool: f = exists (b in x) (not b);
		bool: t = exists (b in x) (b);
	} in if f /\ t then
		{false, true}
	elseif f then
		{false}
	elseif t then
		{true}
	else
		{}
	endif;

/** @group stdlib.builtins.array Return the set containing the elements of \a x */
function set of float: array2set(array [int] of float: x) = {
	x[i] |
		i in index_set(x),
};

/** @group stdlib.builtins.array Return if \a y contains \a x */
function bool: 'in'($X: x, array [$T] of $X: y) =
	exists (z in array1d(y)) (z = x);

/** @group stdlib.builtins.array Return if \a y contains \a x */
function bool: 'in'(opt $X: x, array [$T] of opt $X: y) =
	exists (z in array1d(y)) (z = x);

/** @group stdlib.builtins.array Return if \a y contains \a x */
function var bool: 'in'(var $X: x, array [$T] of var $X: y) =
	exists (z in array1d(y)) (z = x);

/** @group stdlib.builtins.array Return if \a y contains \a x */
function var bool: 'in'(var opt $X: x, array [$T] of var opt $X: y) =
	exists (z in array1d(y)) (z = x);

/** @group stdlib.builtins.array Return if \a y contains \a x */
function bool: 'in'(set of $X: x, array [$T] of set of $X: y) =
	exists (z in array1d(y)) (z = x);

/** @group stdlib.builtins.array Return if \a y contains \a x */
function var bool: 'in'(var set of $$X: x, array [$T] of var set of $$X: y) =
	exists (z in array1d(y)) (z = x);

function array [$$E] of any $V: mzn_array_kd(
	set of $$E: i,
	array [$U] of any $V: x,
) = array1d(i, x);
function array [$$E] of any $V: mzn_array_kd(
	tuple(set of $$E): i,
	array [$U] of any $V: x,
) = mzn_array_kd(i.1, x);
function array [$$E, $$F] of any $V: mzn_array_kd(
	tuple(set of $$E, set of $$F): i,
	array [$U] of any $V: x,
) = array2d(i.1, i.2, x);
function array [$$E, $$F, $$G] of any $V: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G): i,
	array [$U] of any $V: x,
) = array3d(i.1, i.2, i.3, x);
function array [$$E, $$F, $$G, $$H] of any $V: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H): i,
	array [$U] of any $V: x,
) = array4d(i.1, i.2, i.3, i.4, x);
function array [$$E, $$F, $$G, $$H, $$I] of any $V: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): i,
	array [$U] of any $V: x,
) = array5d(i.1, i.2, i.3, i.4, i.5, x);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $V: mzn_array_kd(
	tuple(
		set of $$E,
		set of $$F,
		set of $$G,
		set of $$H,
		set of $$I,
		set of $$J,
	): i,
	array [$U] of any $V: x,
) = array6d(i.1, i.2, i.3, i.4, i.5, i.6, x);
