include "fzn_partition_set.mzn";
include "fzn_partition_set_reif.mzn";

/** @group globals.set
  Constrains the sets in array \a S to partition the \a universe.
*/
predicate partition_set(array [_] of var set of $$E: S, set of $$E: universe) =
	fzn_partition_set(erase_enum(S), erase_enum(universe));
