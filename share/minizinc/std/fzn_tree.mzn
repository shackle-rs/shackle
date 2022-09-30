include "tree.mzn";

predicate fzn_tree(array[int] of int: from, array[int] of int: to,
                        var int: r, array[int] of var bool: ns, array[int] of var bool: es) =
    let {
        set of int: EDGE = min(index_set(es))..max(index_set(es))+length(es);
        array[EDGE] of int: dfrom = from ++ to;
        array[EDGE] of int: dto   = to ++ from;
        array[EDGE] of var bool: des;
    } in
      /* ensure that the directed edges selected agree with undirected edges */
      forall(e in index_set(es))(es[e] <-> (des[e] \/ des[e+length(es)])) /\
      /* duplicate the edges so that the we can use directed graph reachability */
      dtree(dfrom,dto,r,ns,des);
   
%-----------------------------------------------------------------------------%
