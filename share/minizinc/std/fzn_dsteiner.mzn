include "tree.mzn";

predicate fzn_dsteiner(
	array [int] of int: from,
	array [int] of int: to,
	array [int] of int: w,
	var int: r,
	array [int] of var bool: ns,
	array [int] of var bool: es,
	var int: K,
) = dtree(from, to, r, ns, es) /\ K = sum (e in index_set(es)) (es[e] * w[e]);

%-----------------------------------------------------------------------------%
