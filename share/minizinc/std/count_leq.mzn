include "fzn_count_leq_par.mzn";
include "fzn_count_leq.mzn";
include "fzn_count_leq_par_reif.mzn";
include "fzn_count_leq_reif.mzn";

/** @group globals.counting
  Constrains \a c to be less than or equal to the number of occurrences
  of \a y in \a x. 
*/
predicate count_leq(array [$X] of var $$E: x, var $$E: y, var int: c) =
	fzn_count_leq(erase_enum(array1d(x)), erase_enum(y), c);

/** @group globals.counting
  Constrains \a c to be less than or equal to the number of occurrences
  of \a y in \a x. 
*/
predicate count_leq(array [$X] of var $$E: x, $$E: y, int: c) =
	fzn_count_leq_par(erase_enum(array1d(x)), erase_enum(y), c);

%-----------------------------------------------------------------------------%
