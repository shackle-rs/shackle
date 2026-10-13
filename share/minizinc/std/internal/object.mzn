% Helpers for object string ids, used by the object lowering. Every potential
% object of a class `C` has an id in `C_ids`; objects given in a data file take
% the id written there. The lookups use MiniZinc's cached `arg_val_weak` and
% `first_duplicate`, which build an index of `C_ids` once.

% The builtins from MiniZinc's arg_val.mzn, declared here rather than included:
% an include in a file that Shackle includes automatically stops the files it
% reaches from being printed for the MiniZinc target.
function opt $$E: arg_val_weak(array [$$E] of string: xs, string: value);
function opt string: first_duplicate(array [$$E] of string: xs);

% The object in `data` whose id in `ids` is `s`. Data can only refer to objects
% that are given in the data, so objects outside `data` are never found. `from`
% names what holds the reference, for the error message.
function $$E: mzn_object_from_id(
	array [$$E] of string: ids,
	set of $$E: data,
	string: s,
	string: cls,
	string: from,
) =
	let {
		opt string: duplicate = first_duplicate(ids);
		any: found = arg_val_weak(ids, s);
	} in assert(
		absent(duplicate),
		"more than one " ++ cls ++ " object has the id \"" ++ deopt(duplicate) ++ "\"",
		assert(
			occurs(found) /\ deopt(found) in data,
			"no " ++ cls ++ " object with id \"" ++ s ++ "\" in the data, referenced by " ++ from,
			deopt(found),
		),
	);

% Whether the ids in `ids` are valid: none is empty, and no two are equal.
% Aborts on the first invalid id.
test mzn_check_object_ids(array [$$E] of string: ids, string: cls) =
	forall (i in index_set(ids)) (
		ids[i] != "" \/ abort("object ids must not be empty, but a " ++ cls ++ " object in the data has the id \"\"")
	) /\ let {
		opt string: duplicate = first_duplicate(ids);
	} in absent(duplicate) \/ abort("more than one " ++ cls ++ " object has the id \"" ++ deopt(duplicate) ++ "\"");
