% Helpers for object string ids, used by the object lowering. Every potential
% object of a class `C` has an id in `C_ids`; objects given in a data file take
% the id written there. A builtin may replace these linear scans later.

% The object in `data` whose id in `ids` is `s`. Data can only refer to objects
% that are given in the data, so objects outside `data` are never found.
function $$E: mzn_object_from_id(
	array [$$E] of string: ids,
	set of $$E: data,
	string: s,
	string: cls,
) =
	let {
		any: found = [o | o in data where ids[o] = s];
	} in assert(
		length(found) > 0,
		"no " ++ cls ++ " object with id \"" ++ s ++ "\" in the data",
		assert(
			length(found) = 1,
			"more than one " ++ cls ++ " object has the id \"" ++ s ++ "\"",
			found[1],
		),
	);

% Whether all ids in `ids` are distinct. Aborts on the first repeated id.
test mzn_object_ids_distinct(array [$$E] of string: ids, string: cls) =
	forall (i, j in index_set(ids) where i < j) (
		ids[i] != ids[j] \/ abort("two " ++ cls ++ " objects have the id \"" ++ ids[i] ++ "\"")
	);
