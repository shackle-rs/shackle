include "fzn_reachable.mzn";
include "fzn_reachable_reif.mzn";
include "fzn_dreachable.mzn";
include "fzn_dreachable_reif.mzn";

/** @group globals.graph
    Constrains the subgraph \a ns and \a es of a given directed graph to be reachable from \a r.

    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param r: the root node (which may be variable)
    @param ns: a Boolean for each node whether it is in the subgraph
    @param es: a Boolean for each edge whether it is in the subgraph
*/
predicate dreachable(
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	var $$N: r,
	array [$$N] of var bool: ns,
	array [$$E] of var bool: es,
) =
	assert(
		index_set(from) = index_set(to),
		"dreachable: index set of from and to must be identical",
	) /\
		assert(
			index_set(from) = index_set(es),
			"dreachable: index set of from and es must be identical",
		) /\
		assert(
			dom_array(from) subset index_set(ns),
			"dreachable: nodes in from must be in index set of ns",
		) /\
		assert(
			dom_array(to) subset index_set(ns),
			"dreachable: nodes in to must be in index set of ns",
		) /\
		fzn_dreachable(
			erase_enum(from),
			erase_enum(to),
			erase_enum(r),
			erase_enum(ns),
			erase_enum(es),
		);

%-----------------------------------------------------------------------------%

/** @group globals.graph
    Constrains the subgraph \a ns and \a es of a given undirected graph to be reachable from \a r.

    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param r: the root node (which may be variable)
    @param ns: a Boolean for each node whether it is in the subgraph
    @param es: a Boolean for each edge whether it is in the subgraph
*/
predicate reachable(
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	var $$N: r,
	array [$$N] of var bool: ns,
	array [$$E] of var bool: es,
) =
	assert(
		index_set(from) = index_set(to),
		"reachable: index set of from and to must be identical",
	) /\
		assert(
			index_set(from) = index_set(es),
			"reachable: index set of from and es must be identical",
		) /\
		assert(
			dom_array(from) subset index_set(ns),
			"reachable: nodes in from must be in index set of ns",
		) /\
		assert(
			dom_array(to) subset index_set(ns),
			"reachable: nodes in to must be in index set of ns",
		) /\
		fzn_reachable(
			erase_enum(from),
			erase_enum(to),
			erase_enum(r),
			erase_enum(ns),
			erase_enum(es),
		);
