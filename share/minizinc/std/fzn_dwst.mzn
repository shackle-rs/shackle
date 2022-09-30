include "tree.mzn";

predicate fzn_dwst(set of int: NODE, array[int] of int: from, array[int] of int: to, array[int] of int: w,
                   var int: r, array[int] of var bool: es, var int: K) =
   let { array[NODE] of bool: ns = [true | n in NODE]; } in
      dtree(from,to,r,ns,es) /\
      K = sum(e in index_set(es))(es[e]*w[e]);
   
%-----------------------------------------------------------------------------%
