%-----------------------------------------------------------------------------%
% Requires that the array 'x' is in decreasing order (duplicates are allowed).
%-----------------------------------------------------------------------------%

predicate fzn_decreasing_set_reif(
	array [int] of var set of int: x,
	var bool: b,
) =
	b <->
		 forall (i in index_set(x) diff {min(index_set(x))}) (x[i - 1] >= x[i]);
