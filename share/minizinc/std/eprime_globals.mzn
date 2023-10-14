% Synonym Constraints & Functions
include "all_different.mzn";
include "global_cardinality.mzn";
include "lex_less.mzn";
include "lex_lesseq.mzn";
include "lex_greater.mzn";
include "lex_greatereq.mzn";
include "all_different_except.mzn";
include "count_eq.mzn";
% include "fzn_global_cardinality_low_up.mzn"; Not Needed as it is already included in the global_cardinality.mzn file

%  EPrime's atleast/atmost takes an array of cover and counts and as such functions more like gcc
predicate atleast(array[int] of var int: x, array [int] of int: c, array[int] of int: vals) = 
    fzn_global_cardinality_low_up(x, vals, c, array1d(index_set(c), [length(x) | i in 1..length(c)]));

predicate atmost(array[int] of var int: x, array [int] of int: c, array[int] of int: vals) = 
    fzn_global_cardinality_low_up(x, vals, array1d(index_set(c), [0 | i in 1..length(c)]), c);

predicate alldifferent_except(array[$X] of var $$E: X, int: v) = 
    all_different_except(X, {v});

% Indexing array based on 0..n-1.
function array[int] of $T: indexing_0(array[$U] of $T: x) = let {any: y = x} in array1d(0..length(y) - 1, x);
function array[int] of var $T: indexing_0(array[$U] of var $T: x) = let {any: y = x} in array1d(0..length(y) - 1, x);