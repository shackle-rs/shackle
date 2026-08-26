/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E): index_sets(array [$$E] of any $U: x);

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F): index_sets(array [$$E, $$F] of any $U: x);

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G): index_sets(array [$$E, $$F, $$G] of any $U: x);

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G, set of $$H): index_sets(
	array [$$E, $$F, $$G, $$H] of any $U: x,
);

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): index_sets(
	array [$$E, $$F, $$G, $$H, $$I] of any $U: x,
);

/** @group stdlib.builtins.array Return index sets of the array \a x */
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I, set of $$J): index_sets(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x,
);

function set of $$E: index_set(array [$$E] of any $U: x) = index_sets(x).1;
function set of $$E: index_set_1of2(array [$$E, $$F] of any $U: x) = index_sets(x).1;
function set of $$E: index_set_1of3(array [$$E, $$F, $$G] of any $U: x) = index_sets(x).1;
function set of $$E: index_set_1of4(array [$$E, $$F, $$G, $$H] of any $U: x) = index_sets(x).1;
function set of $$E: index_set_1of5(array [$$E, $$F, $$G, $$H, $$I] of any $U: x) = index_sets(x).1;
function set of $$E: index_set_1of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).1;
function set of $$F: index_set_2of2(array [$$E, $$F] of any $U: x) = index_sets(x).2;
function set of $$F: index_set_2of3(array [$$E, $$F, $$G] of any $U: x) = index_sets(x).2;
function set of $$F: index_set_2of4(array [$$E, $$F, $$G, $$H] of any $U: x) = index_sets(x).2;
function set of $$F: index_set_2of5(array [$$E, $$F, $$G, $$H, $$I] of any $U: x) = index_sets(x).2;
function set of $$F: index_set_2of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).2;
function set of $$G: index_set_3of3(array [$$E, $$F, $$G] of any $U: x) = index_sets(x).3;
function set of $$G: index_set_3of4(array [$$E, $$F, $$G, $$H] of any $U: x) = index_sets(x).3;
function set of $$G: index_set_3of5(array [$$E, $$F, $$G, $$H, $$I] of any $U: x) = index_sets(x).3;
function set of $$G: index_set_3of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).3;
function set of $$H: index_set_4of4(array [$$E, $$F, $$G, $$H] of any $U: x) = index_sets(x).4;
function set of $$H: index_set_4of5(array [$$E, $$F, $$G, $$H, $$I] of any $U: x) = index_sets(x).4;
function set of $$H: index_set_4of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).4;
function set of $$I: index_set_5of5(array [$$E, $$F, $$G, $$H, $$I] of any $U: x) = index_sets(x).5;
function set of $$I: index_set_5of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).5;
function set of $$J: index_set_6of6(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
	index_sets(x).6;

test index_sets_agree(array [$T] of any $U: x, array [$T] of any $W: y) :: mzn_unreachable =
	abort("unreachable");
test index_sets_agree(array [$$E] of any $U: x, array [$$E] of any $W: y) =
	index_sets(x) = index_sets(y);
test index_sets_agree(array [$$E, $$F] of any $U: x, array [$$E, $$F] of any $W: y) =
	index_sets(x) = index_sets(y);
test index_sets_agree(array [$$E, $$F, $$G] of any $U: x, array [$$E, $$F, $$G] of any $W: y) =
	index_sets(x) = index_sets(y);
test index_sets_agree(
	array [$$E, $$F, $$G, $$H] of any $U: x,
	array [$$E, $$F, $$G, $$H] of any $W: y,
) = index_sets(x) = index_sets(y);
test index_sets_agree(
	array [$$E, $$F, $$G, $$H, $$I] of any $U: x,
	array [$$E, $$F, $$G, $$H, $$I] of any $W: y,
) = index_sets(x) = index_sets(y);
test index_sets_agree(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x,
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $W: y,
) = index_sets(x) = index_sets(y);

function array [$$E] of any $T: mzn_array_kd(tuple(set of $$E): idx, array [$U] of any $T: x);
function array [$$E, $$F] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F): idx,
	array [$U] of any $T: x,
);
function array [$$E, $$F, $$G] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G): idx,
	array [$U] of any $T: x,
);
function array [$$E, $$F, $$G, $$H] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H): idx,
	array [$U] of any $T: x,
);
function array [$$E, $$F, $$G, $$H, $$I] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): idx,
	array [$U] of any $T: x,
);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I, set of $$J): idx,
	array [$U] of any $T: x,
);

function array [int] of any $V: array1d(array [$U] of any $V: x) = mzn_array_kd((1..length(x),), x);
function array [$$E] of any $V: array1d(set of $$E: S, array [$U] of any $V: x) =
	let {
		any: expected = card(S);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index set.",
		mzn_array_kd((S,), x),
	);
function array [$$E, $$F] of any $V: array2d(
	set of $$E: S1,
	set of $$F: S2,
	array [$U] of any $V: x,
) =
	let {
		any: expected = card(S1) * card(S2);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index sets.",
		mzn_array_kd((S1, S2), x),
	);
function array [$$E, $$F, $$G] of any $V: array3d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	array [$U] of any $V: x,
) =
	let {
		any: expected = card(S1) * card(S2) * card(S3);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index sets.",
		mzn_array_kd((S1, S2, S3), x),
	);
function array [$$E, $$F, $$G, $$H] of any $V: array4d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	array [$U] of any $V: x,
) =
	let {
		any: expected = card(S1) * card(S2) * card(S3) * card(S4);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index sets.",
		mzn_array_kd((S1, S2, S3, S4), x),
	);
function array [$$E, $$F, $$G, $$H, $$I] of any $V: array5d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	set of $$I: S5,
	array [$U] of any $V: x,
) =
	let {
		any: expected = card(S1) * card(S2) * card(S3) * card(S4) * card(S5);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index sets.",
		mzn_array_kd((S1, S2, S3, S4, S5), x),
	);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $V: array6d(
	set of $$E: S1,
	set of $$F: S2,
	set of $$G: S3,
	set of $$H: S4,
	set of $$I: S5,
	set of $$J: S6,
	array [$U] of any $V: x,
) =
	let {
		any: expected = card(S1) * card(S2) * card(S3) * card(S4) * card(S5) * card(S6);
		any: actual = length(x);
	} in assert(
		expected = actual,
		"mismatch in array dimensions: the array contains \(actual) elements, but \(
			expected
		) elements were expected according to the given index sets.",
		mzn_array_kd((S1, S2, S3, S4, S5, S6), x),
	);

function array [$$E, $$F] of any $T: mzn_array_2d_literal(
	array [int] of $$E: idx1,
	array [int] of $$F: idx2,
	array [int] of any $T: x,
) = [
	(mzn_element_internal(idx1, i), mzn_element_internal(idx2, j)): mzn_element_internal(
		x,
		(i - 1) * length(idx2) + j,
	) |
		i in index_set(idx1),
		j in index_set(idx2),
];

function array [$E] of any $T: mzn_indexed_array(array [int] of tuple($E, any $T): x);
function array [$$E] of any $T: mzn_start_indexed_array($$E: start, array [int] of any $T: x) =
	array1d(start..to_enum(enum_of(start), enum2int(start) + length(x) - 1), x);

function var set of int: mzn_array2set(array [int] of var int: x) :: promise_total =
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

function var set of int: mzn_array2set(array [int] of var opt int: x) :: promise_total =
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

function set of int: mzn_array2set(array [int] of int: x);
function set of int: mzn_array2set(array [int] of opt int: x) = {
	deopt(x[i]) |
		i in index_set(x) where occurs(x[i]),
};

function set of bool: mzn_array2set(array [int] of bool: x) =
	let {
		bool: f = exists (b in x) (not b);
		bool: t = exists (b in x) (b);
	} in if f /\ t then {false, true} elseif f then {false} elseif t then {true} else {} endif;

function set of float: mzn_array2set(array [int] of float: x);

function set of $$E: mzn_array2set(array [int] of $$E: x) :: mzn_unreachable;
