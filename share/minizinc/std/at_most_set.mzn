include "fzn_at_most_set.mzn";
include "fzn_at_most_set_reif.mzn";

%-----------------------------------------------------------------------------%
% Requires at most 'n' variables in 'x' to take the value 'v'.
%-----------------------------------------------------------------------------%

predicate at_most_set(int: n, array [$X] of var set of $$E: x, set of $$E: v) =
	fzn_at_most_set(n, erase_enum(array1d(x)), erase_enum(v));
