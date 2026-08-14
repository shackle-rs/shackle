% Definitions to keep verbatim for compatibility with old MiniZinc

shackle_mzn_absent_zero =
	if occurs(mzn_absent_zero) then (true, deopt(mzn_absent_zero)) else (false, false) endif;
shackle_mzn_min_version_required =
	if occurs(mzn_min_version_required) then
		(true, deopt(mzn_min_version_required))
	else
		(false, 0)
	endif;
shackle_mzn_max_version_required =
	if occurs(mzn_max_version_required) then
		(true, deopt(mzn_max_version_required))
	else
		(false, 0)
	endif;
shackle_mzn_opt_only_range_domains =
	if occurs(mzn_opt_only_range_domains) then
		(true, deopt(mzn_opt_only_range_domains))
	else
		(false, false)
	endif;
shackle_mzn_opt_annotate_defines_var =
	if occurs(mzn_opt_annotate_defines_var) then
		(true, deopt(mzn_opt_annotate_defines_var))
	else
		(false, false)
	endif;
shackle_mzn_opt_annotate_computed_domains =
	if occurs(mzn_opt_annotate_computed_domains) then
		(true, deopt(mzn_opt_annotate_computed_domains))
	else
		(false, false)
	endif;
shackle_mzn_ignore_symmetry_breaking_constraints =
	if occurs(mzn_ignore_symmetry_breaking_constraints) then
		(true, deopt(mzn_ignore_symmetry_breaking_constraints))
	else
		(false, false)
	endif;
shackle_mzn_ignore_redundant_constraints =
	if occurs(mzn_ignore_redundant_constraints) then
		(true, deopt(mzn_ignore_redundant_constraints))
	else
		(false, false)
	endif;
shackle_mzn_half_reify_clause =
	if occurs(mzn_half_reify_clause) then
		(true, deopt(mzn_half_reify_clause))
	else
		(false, false)
	endif;

function any $T: mzn_element_internal(array [int] of any $T: x, int: index) = x[index];
function any $T: mzn_element_internal(array [int] of any $T: x, tuple(int): index) = x[index.1];
function any $T: mzn_element_internal(array [int, int] of any $T: x, tuple(int, int): indices) =
	x[indices.1, indices.2];
function any $T: mzn_element_internal(
	array [int, int, int] of any $T: x,
	tuple(int, int, int): indices,
) = x[indices.1, indices.2, indices.3];
function any $T: mzn_element_internal(
	array [int, int, int, int] of any $T: x,
	tuple(int, int, int, int): indices,
) = x[indices.1, indices.2, indices.3, indices.4];
function any $T: mzn_element_internal(
	array [int, int, int, int, int] of any $T: x,
	tuple(int, int, int, int, int): indices,
) = x[indices.1, indices.2, indices.3, indices.4, indices.5];
function any $T: mzn_element_internal(
	array [int, int, int, int, int, int] of any $T: x,
	tuple(int, int, int, int, int, int): indices,
) = x[indices.1, indices.2, indices.3, indices.4, indices.5, indices.6];

function array [int] of any $T: mzn_slice_internal(
	array [$X] of any $T: x,
	array [int] of set of int: i,
) = slice_1d(x, i, 1..product (s in i) (card(s)));

function tuple(set of $$E): index_sets(array [$$E] of any $U: x) = (index_set(x),);
function tuple(set of $$E, set of $$F): index_sets(array [$$E, $$F] of any $U: x) = (
	index_set_1of2(x),
	index_set_2of2(x),
);
function tuple(set of $$E, set of $$F, set of $$G): index_sets(
	array [$$E, $$F, $$G] of any $U: x,
) = (index_set_1of3(x), index_set_2of3(x), index_set_3of3(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H): index_sets(
	array [$$E, $$F, $$G, $$H] of any $U: x,
) = (index_set_1of4(x), index_set_2of4(x), index_set_3of4(x), index_set_4of4(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): index_sets(
	array [$$E, $$F, $$G, $$H, $$I] of any $U: x,
) = (index_set_1of5(x), index_set_2of5(x), index_set_3of5(x), index_set_4of5(x), index_set_5of5(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I, set of $$J): index_sets(
	array [$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x,
) = (
	index_set_1of6(x),
	index_set_2of6(x),
	index_set_3of6(x),
	index_set_4of6(x),
	index_set_5of6(x),
	index_set_6of6(x),
);

function array [$$E] of any $T: mzn_array_kd(tuple(set of $$E): idx, array [$U] of any $T: x) =
	array1d(idx.1, x);
function array [$$E, $$F] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F): idx,
	array [$U] of any $T: x,
) = array2d(idx.1, idx.2, x);
function array [$$E, $$F, $$G] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G): idx,
	array [$U] of any $T: x,
) = array3d(idx.1, idx.2, idx.3, x);
function array [$$E, $$F, $$G, $$H] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H): idx,
	array [$U] of any $T: x,
) = array4d(idx.1, idx.2, idx.3, idx.4, x);
function array [$$E, $$F, $$G, $$H, $$I] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): idx,
	array [$U] of any $T: x,
) = array5d(idx.1, idx.2, idx.3, idx.4, idx.5, x);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: mzn_array_kd(
	tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I, set of $$J): idx,
	array [$U] of any $T: x,
) = array6d(idx.1, idx.2, idx.3, idx.4, idx.5, idx.6, x);

function array [$$E] of any $T: mzn_indexed_array(array [int] of tuple($$E, any $T): x) =
	arrayNd(x.1, x.2);
function array [$$E, $$F] of any $T: mzn_indexed_array(
	array [int] of tuple(tuple($$E, $$F), any $T): x,
) = arrayNd([x_i.1.1 | x_i in x], [x_i.1.2 | x_i in x], [x_i.2 | x_i in x]);
function array [$$E, $$F, $$G] of any $T: mzn_indexed_array(
	array [int] of tuple(tuple($$E, $$F, $$G), any $T): x,
) = arrayNd([x_i.1.1 | x_i in x], [x_i.1.2 | x_i in x], [x_i.1.3 | x_i in x], [x_i.2 | x_i in x]);
function array [$$E, $$F, $$G, $$H] of any $T: mzn_indexed_array(
	array [int] of tuple(tuple($$E, $$F, $$G, $$H), any $T): x,
) =
	arrayNd(
		[x_i.1.1 | x_i in x],
		[x_i.1.2 | x_i in x],
		[x_i.1.3 | x_i in x],
		[x_i.1.4 | x_i in x],
		[x_i.2 | x_i in x],
	);
function array [$$E, $$F, $$G, $$H, $$I] of any $T: mzn_indexed_array(
	array [int] of tuple(tuple($$E, $$F, $$G, $$H, $$I), any $T): x,
) =
	arrayNd(
		[x_i.1.1 | x_i in x],
		[x_i.1.2 | x_i in x],
		[x_i.1.3 | x_i in x],
		[x_i.1.4 | x_i in x],
		[x_i.1.5 | x_i in x],
		[x_i.2 | x_i in x],
	);
function array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: mzn_indexed_array(
	array [int] of tuple(tuple($$E, $$F, $$G, $$H, $$I, $$J), any $T): x,
) =
	arrayNd(
		[x_i.1.1 | x_i in x],
		[x_i.1.2 | x_i in x],
		[x_i.1.3 | x_i in x],
		[x_i.1.4 | x_i in x],
		[x_i.1.5 | x_i in x],
		[x_i.1.6 | x_i in x],
		[x_i.2 | x_i in x],
	);

function set of int: mzn_array2set(array [int] of int: x) = array2set(x);
function set of float: mzn_array2set(array [int] of float: x) = array2set(x);

function bool: mzn_add_warning(string: msg) = trace("WARNING: " ++ msg);

function int: shackle_div(int: x, int: y) = x div y;
% Parameter names must match the shackle stdlib's public wrappers
% (std/internal/math.mzn): the old_compat prefixing renames those wrappers to
% these same `shackle_*` names, and old MiniZinc rejects same-signature
% overloads that differ only in parameter names as ambiguous.
function $$E: shackle_min(set of $$E: s) = min(s);
function $$E: shackle_max(set of $$E: s) = max(s);
function $$E: shackle_arg_min(array [$$E] of bool: xs) = arg_min(xs);
function $$E: shackle_arg_min(array [$$E] of $$T: xs) = arg_min(xs);
function $$E: shackle_arg_min(array [$$E] of float: xs) = arg_min(xs);
function $$E: shackle_arg_max(array [$$E] of bool: xs) = arg_max(xs);
function $$E: shackle_arg_max(array [$$E] of $$T: xs) = arg_max(xs);
function $$E: shackle_arg_max(array [$$E] of float: xs) = arg_max(xs);
function int: shackle_pow(int: x, int: y) = pow(x, y);
function array [$$E] of $$F: inverse(array [$$F] of $$E: f);
function array [$$E] of $$F: shackle_inverse(array [$$F] of $$E: f) = inverse(f);
function set of $U: shackle_array_intersect(array [$T] of set of $U: x) = array_intersect(x);
