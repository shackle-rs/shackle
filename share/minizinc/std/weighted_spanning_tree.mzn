include "fzn_wst.mzn";
include "fzn_wst_reif.mzn";
include "fzn_dwst.mzn";
include "fzn_dwst_reif.mzn";

/** @group globals.graph
    Constrains the set of edges \a es of a given directed graph to be a weighted spanning tree rooted at \a r of weight \a W.

    @param nodes: the nodes in the graph
    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param w: the weight of each edge
    @param r: the root node (which may be variable)
    @param es: a Boolean for each edge whether it is in the subgraph
    @param K: the weight of the tree
*/
predicate d_weighted_spanning_tree(
	set of $$N: nodes,
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	array [$$E] of int: w,
	var $$N: r,
	array [$$E] of var bool: es,
	var int: K,
) =
	assert(
		index_set(from) = index_set(to),
		"d_weighted_spanning_tree: index set of `from` and `to` must be identical",
	) /\
		assert(
			index_set(to) = index_set(w),
			"d_weighted_spanning_tree: index set of `to` and `w` must be identical",
		) /\
		assert(
			index_set(w) = index_set(es),
			"d_weighted_spanning_tree: index set of `w` and `es` must be identical",
		) /\
		fzn_dwst(
			erase_enum(nodes),
			erase_enum(from),
			erase_enum(to),
			erase_enum(w),
			erase_enum(r),
			erase_enum(es),
			K,
		);

/** @group globals.graph
    Constrains the set of edges \a es of a given undirected graph to be a weighted spanning tree of weight \a W.

    @param nodes: the nodes in the graph
    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param w: the weight of each edge
    @param es: a Boolean for each edge whether it is in the subgraph
    @param K: the weight of the tree
*/
predicate weighted_spanning_tree(
	set of $$N: nodes,
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	array [$$E] of int: w,
	array [$$E] of var bool: es,
	var int: K,
) =
	assert(
		index_set(from) = index_set(to),
		"weighted_spanning_tree: index set of `from` and `to` must be identical",
	) /\
		assert(
			index_set(to) = index_set(w),
			"weighted_spanning_tree: index set of `to` and `w` must be identical",
		) /\
		assert(
			index_set(w) = index_set(es),
			"weighted_spanning_tree: index set of `w` and `es` must be identical",
		) /\
		fzn_wst(
			erase_enum(nodes),
			erase_enum(from),
			erase_enum(to),
			erase_enum(w),
			erase_enum(es),
			K,
		);

%-----------------------------------------------------------------------------%
