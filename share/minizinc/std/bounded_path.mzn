include "fzn_bounded_path.mzn";
include "fzn_bounded_path_reif.mzn";
include "fzn_bounded_dpath.mzn";
include "fzn_bounded_dpath_reif.mzn";

/** @group globals.graph
   Constrains the subgraph \a ns and \a es of a given directed graph to be a path from \a s to \a t of weight \a K.

   @param from: the leaving node for each edge
   @param to: the entering node for each edge
   @param w: the weight of each edge
   @param s: the source node (which may be variable)
   @param t: the dest node (which may be variable)
   @param ns: a Boolean for each node whether it is in the subgraph
   @param es: a Boolean for each edge whether it is in the subgraph
   @param K: the cost of the path
*/
predicate bounded_dpath(array[$$E] of $$N: from, array[$$E] of $$N: to, array[$$E] of int: w,
                    var $$N: s, var $$N: t, array[$$N] of var bool: ns, array[$$E] of var bool: es, var int: K) =
   assert(index_set(from) = index_set(to),"bounded_dpath: index set of from and to must be identical") /\
   assert(index_set(from) = index_set(es),"bounded_dpath: index set of from and es must be identical") /\
   assert(index_set(w) = index_set(es),"bounded_dpath: index set of w and es must be identical") /\
   assert(dom_array(from) subset index_set(ns),"bounded_dpath: nodes in from must be in index set of ns") /\
   assert(dom_array(to) subset index_set(ns),"bounded_dpath: nodes in to must be in index set of ns") /\
   fzn_bounded_dpath(erase_enum(from),erase_enum(to),erase_enum(w),erase_enum(s),erase_enum(t),erase_enum(ns),erase_enum(es),K);
   
/** @group globals.graph
   Constrains the subgraph \a ns and \a es of a given undirected graph to be a path from \a s to \a t of weight \a K.

   @param from: the leaving node for each edge
   @param to: the entering node for each edge
   @param w: the weight of each edge
   @param s: the source node (which may be variable)
   @param t: the dest node (which may be variable)
   @param ns: a Boolean for each node whether it is in the subgraph
   @param es: a Boolean for each edge whether it is in the subgraph
   @param K: the cost of the path
*/
predicate bounded_path(array[$$E] of $$N: from, array[$$E] of $$N: to, array[$$E] of int: w,
                    var $$N: s, var $$N: t, array[$$N] of var bool: ns, array[$$E] of var bool: es, var int: K) =
   assert(index_set(from) = index_set(to),"bounded_path: index set of from and to must be identical") /\
   assert(index_set(from) = index_set(es),"bounded_path: index set of from and es must be identical") /\
   assert(index_set(w) = index_set(es),"bounded_path: index set of w and es must be identical") /\
   assert(dom_array(from) subset index_set(ns),"bounded_path: nodes in from must be in index set of ns") /\
   assert(dom_array(to) subset index_set(ns),"bounded_path: nodes in to must be in index set of ns") /\
   fzn_bounded_path(erase_enum(from),erase_enum(to),erase_enum(w),erase_enum(s),erase_enum(t),erase_enum(ns),erase_enum(es),K);
   
%-----------------------------------------------------------------------------%
%-----------------------------------------------------------------------------%
