include "fzn_subgraph.mzn";
include "fzn_subgraph_reif.mzn";

/** @group globals.graph
    Constrains that \a ns and \a es is a subgraph of a given directed graph.

    @param from: the leaving node for each edge
    @param to: the entering node for each edge
    @param ns: a Boolean for each node whether it is in the subgraph
    @param es: a Boolean for each edge whether it is in the subgraph
*/
predicate subgraph(
	array [$$E] of $$N: from,
	array [$$E] of $$N: to,
	array [$$N] of var bool: ns,
	array [$$E] of var bool: es,
) =
	assert(
		index_set(from) = index_set(to),
		"subgraph: index set of from and to must be identical",
	) /\
		assert(
			index_set(from) = index_set(es),
			"subgraph: index set of from and es must be identical",
		) /\
		assert(
			dom_array(from) subset index_set(ns),
			"subgraph: elements in from must be in index set of ns",
		) /\
		assert(
			dom_array(to) subset index_set(ns),
			"subgraph: elements in to must be in index set of ns",
		) /\
		fzn_subgraph(
			erase_enum(from),
			erase_enum(to),
			erase_enum(ns),
			erase_enum(es),
		);

%-----------------------------------------------------------------------------%
