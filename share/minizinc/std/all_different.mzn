include "fzn_all_different_int.mzn";
include "fzn_all_different_int_opt.mzn";
include "fzn_all_different_int_reif.mzn";
include "fzn_all_different_set.mzn";
include "fzn_all_different_set_reif.mzn";
include "analyse_all_different.mzn";

/** @group globals.alldifferent
  Constrain the elements in the array \a x to be pairwise different.
*/
predicate all_different(array [$X] of var $$E: x) =
	analyse_all_different(erase_enum(array1d(x))) /\
		fzn_all_different_int(erase_enum(array1d(x)));

/**  @group globals.alldifferent
  Constrain the elements in the array \a x to be pairwise different.
*/
predicate all_different(array [$X] of var set of $$E: x) =
	fzn_all_different_set(erase_enum(array1d(x)));

/**  @group globals.alldifferent
  Constrain the non-absent elements in the array \a x to be pairwise different.
  The absent value <> is allowed to occur multiple times.
*/
predicate all_different(array [$X] of var opt $$E: x) =
	analyse_all_different(erase_enum(array1d(x))) /\
		fzn_all_different_int_opt(erase_enum(array1d(x)));

% Synonyms for the above.
predicate alldifferent(array [$X] of var $$E: x) = all_different(x);

predicate alldifferent(array [$X] of var set of $$E: x) = all_different(x);

predicate alldifferent(array [$X] of var opt $$E: x) = all_different(x);
