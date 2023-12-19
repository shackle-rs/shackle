include "fzn_steiner.mzn";
include "fzn_steiner_reif.mzn";
include "fzn_dsteiner.mzn";
include "fzn_dsteiner_reif.mzn";
include "weighted_spanning_tree.mzn";

/** @group globals.graph
    Constrains the subgraph \a  ns and \a es of a given directed graph to be a weighted spanning tree rooted at \a r of weight \a W.

    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param w: the weight of each edge
    @param r: the root node (which may be variable)
    @param ns: a Boolean for each node whether it is in the subgraph
    @param es: a Boolean for each edge whether it is in the subgraph
    @param K: the weight of the tree
*/
predicate dsteiner(
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	array [$$E] of int: w,
	var $$N: r,
	array [$$N] of var bool: ns,
	array [$$E] of var bool: es,
	var int: K,
) =
	assert(
		index_set(from) = index_set(to),
		"dsteiner: index set of `from` and `to` must be identical",
	) /\
		assert(
			index_set(to) = index_set(w),
			"dsteiner: index set of `to` and `w` must be identical",
		) /\
		assert(
			index_set(w) = index_set(es),
			"dsteiner: index set of `w` and `es` must be identical",
		) /\
		if forall (n in index_set(ns)) (is_fixed(ns[n]) /\ fix(ns[n])) then
			d_weighted_spanning_tree(index_set(ns), from, to, w, r, es, K)
		else
			fzn_dsteiner(
				erase_enum(from),
				erase_enum(to),
				erase_enum(w),
				erase_enum(r),
				erase_enum(ns),
				erase_enum(es),
				K,
			)
		endif;

/** @group globals.graph
    Constrains the set of edges \a es of a given undirected graph to be a weighted spanning tree of weight \a W.

    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param w: the weight of each edge
    @param ns: a Boolean for each node whether it is in the subgraph
    @param es: a Boolean for each edge whether it is in the subgraph
    @param K: the weight of the tree
**/
predicate steiner(
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	array [$$E] of int: w,
	array [$$N] of var bool: ns,
	array [$$E] of var bool: es,
	var int: K,
) =
	assert(
		index_set(from) = index_set(to),
		"steiner: index set of `from` and `to` must be identical",
	) /\
		assert(
			index_set(to) = index_set(w),
			"steiner: index set of `to` and `w` must be identical",
		) /\
		assert(
			index_set(w) = index_set(es),
			"steiner: index set of `w` and `es` must be identical",
		) /\
		if forall (n in index_set(ns)) (is_fixed(ns[n]) /\ fix(ns[n])) then
			weighted_spanning_tree(index_set(ns), from, to, w, es, K)
		else
			fzn_steiner(
				erase_enum(from),
				erase_enum(to),
				erase_enum(w),
				erase_enum(ns),
				erase_enum(es),
				K,
			)
		endif;

%-----------------------------------------------------------------------------%
