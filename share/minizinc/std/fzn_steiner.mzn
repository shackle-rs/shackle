include "tree.mzn";

predicate fzn_steiner(array[int] of int: from, array[int] of int: to, array[int] of int: w,
                      array[int] of var bool: ns, array[int] of var bool: es, var int: K) =
    let { var index_set(ns): r; } in
    tree(from,to,r,ns,es) /\
    K = sum(e in index_set(es))(es[e]*w[e]);
   
%-----------------------------------------------------------------------------%
