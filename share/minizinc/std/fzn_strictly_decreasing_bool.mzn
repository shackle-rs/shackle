%-----------------------------------------------------------------------------%
% Requires that the array 'x' is in strict decreasing order
%-----------------------------------------------------------------------------%

predicate fzn_strictly_decreasing_bool(array [int] of var bool: x) =
	forall (i in index_set(x) diff {min(index_set(x))}) (x[i - 1] > x[i]);
