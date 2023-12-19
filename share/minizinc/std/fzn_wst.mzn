include "tree.mzn";

predicate fzn_wst(
	set of int: NODE,
	array [int] of int: from,
	array [int] of int: to,
	array [int] of int: w,
	array [int] of var bool: es,
	var int: K,
) =
	let {
		var NODE: r; /* root of tree */
		array [NODE] of bool: ns = [true | n in NODE];
	} in tree(from, to, r, ns, es) /\
		K = sum (e in index_set(es)) (es[e] * w[e]);

%-----------------------------------------------------------------------------%
