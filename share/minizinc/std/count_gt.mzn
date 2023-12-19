include "fzn_count_gt_par.mzn";
include "fzn_count_gt.mzn";
include "fzn_count_gt_par_reif.mzn";
include "fzn_count_gt_reif.mzn";

/** @group globals.counting
  Constrains \a c to be strictly greater than the number of occurrences
  of \a y in \a x. 
*/
predicate count_gt(array [$X] of var $$E: x, var $$E: y, var int: c) =
	fzn_count_gt(erase_enum(array1d(x)), erase_enum(y), c);

/** @group globals.counting
  Constrains \a c to be strictly greater than the number of occurrences
  of \a y in \a x. 
*/
predicate count_gt(array [$X] of var $$E: x, $$E: y, int: c) =
	fzn_count_gt_par(erase_enum(array1d(x)), erase_enum(y), c);

%-----------------------------------------------------------------------------%
