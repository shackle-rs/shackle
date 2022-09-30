include "fzn_path.mzn";
include "fzn_path_reif.mzn";
include "fzn_dpath.mzn";
include "fzn_dpath_reif.mzn";

/** @group globals.graph
   Constrains the subgraph \a ns and \a es of a given directed graph to be a path from \a s to \a t.

   @param from: the leaving node for each edge
   @param to: the entering node for each edge
   @param s: the source node (which may be variable)
   @param t: the dest node (which may be variable)
   @param ns: a Boolean for each node whether it is in the subgraph
   @param es: a Boolean for each edge whether it is in the subgraph
*/
predicate dpath(array[$$E] of $$N: from, array[$$E] of $$N: to,
                    var $$N: s, var $$N: t, array[$$N] of var bool: ns, array[$$E] of var bool: es) =
   assert(index_set(from) = index_set(to),"dpath: index set of from and to must be identical") /\
   assert(index_set(from) = index_set(es),"dpath: index set of from and es must be identical") /\
   assert(dom_array(from) subset index_set(ns),"dpath: nodes in from must be in index set of ns") /\
   assert(dom_array(to) subset index_set(ns),"dpath: nodes in to must be in index set of ns") /\
   fzn_dpath(erase_enum(from),erase_enum(to),erase_enum(s),erase_enum(t),erase_enum(ns),erase_enum(es));

%-----------------------------------------------------------------------------%

/** @group globals.graph
   Constrains the subgraph \a ns and \a es of a given undirected graph to be a path from \a s to \a t.

   @param from: the leaving node for each edge
   @param to: the entering node for each edge
   @param s: the source node (which may be variable)
   @param t: the dest node (which may be variable)
   @param ns: a Boolean for each node whether it is in the subgraph
   @param es: a Boolean for each edge whether it is in the subgraph
*/
predicate path(array[$$E] of $$N: from, array[$$E] of $$N: to,
                    var $$N: s, var $$N: t, array[$$N] of var bool: ns, array[$$E] of var bool: es) =
   assert(index_set(from) = index_set(to),"path: index set of from and to must be identical") /\
   assert(index_set(from) = index_set(es),"path: index set of from and es must be identical") /\
   assert(dom_array(from) subset index_set(ns),"path: nodes in from must be in index set of ns") /\
   assert(dom_array(to) subset index_set(ns),"path: nodes in to must be in index set of ns") /\
   fzn_path(erase_enum(from),erase_enum(to),erase_enum(s),erase_enum(t),erase_enum(ns),erase_enum(es));
