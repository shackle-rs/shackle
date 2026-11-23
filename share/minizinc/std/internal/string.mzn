function string: mzn_show_tuple_access(string: x, int: i) = x ++ ".\(i)";
function string: mzn_show_record_access(string: x, string: f) = x ++ "." ++ f;

function string: mzn_show_array_access(string: x, $$E: i) = x ++ "[" ++ show(i) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E): i) = x ++ "[" ++ show(i.1) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F): i) =
	x ++ "[" ++ show(i.1) ++ "," ++ show(i.2) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F, $$G): i) =
	x ++ "[" ++ show(i.1) ++ "," ++ show(i.2) ++ "," ++ show(i.3) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F, $$G, $$H): i) =
	x ++ "[" ++ show(i.1) ++ "," ++ show(i.2) ++ "," ++ show(i.3) ++ "," ++ show(i.4) ++ "]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F, $$G, $$H, $$I): i) =
	x ++
		"[" ++
		show(i.1) ++
		"," ++
		show(i.2) ++
		"," ++
		show(i.3) ++
		"," ++
		show(i.4) ++
		"," ++
		show(i.5) ++
		"]";
function string: mzn_show_array_access(string: x, tuple($$E, $$F, $$G, $$H, $$I, $$J): i) =
	x ++
		"[" ++
		show(i.1) ++
		"," ++
		show(i.2) ++
		"," ++
		show(i.3) ++
		"," ++
		show(i.4) ++
		"," ++
		show(i.5) ++
		"," ++
		show(i.6) ++
		"]";

% TODO: Remove these overloads which will no longer be needed
function string: show(any $T: x) = "unreachable";
function string: show(array [$U] of any $T: x) = "unreachable";
function string: showJSON(any $T: x) = "unreachable";
function string: showJSON(array [$U] of any $T: x) = "unreachable";
function string: show_gantt(
	array [$$T] of var int: start,
	array [$$T] of var int: dur,
	array [$$T] of string: name,
) :: output_only = "unreachable";
function string: show_gantt(array [$$T] of var int: start, array [$$T] of var int: dur)
	:: output_only = "unreachable";

function string: show($T: x) :: mzn_internal_generated = show_internal(x);

function string: show(opt $T: x) = if occurs(x) then show(deopt(x)) else "<>" endif;

function string: show(array [$X] of $T: x) = "[" ++ join(", ", [show(x_i) | x_i in x]) ++ "]";
function string: show(array [$X] of opt $T: x) = "[" ++ join(", ", [show(x_i) | x_i in x]) ++ "]";

function string: show(set of $T: x) = "{" ++ join(", ", [show(x_i) | x_i in x]) ++ "}";
function string: show(set of int: x) =
	let {
		any: ranges = set_to_ranges(x);
	} in if length(ranges) = 2 /\ ranges[1] != ranges[2] then
		show(ranges[1]) ++ ".." ++ show(ranges[2])
	else
		"{" ++ join(", ", [show(x_i) | x_i in x]) ++ "}"
	endif;
function string: show(set of float: x) =
	let {
		any: ranges = set_to_ranges(x);
	} in if forall (i in index_set(ranges) where i mod 2 = 1) (ranges[i] = ranges[i + 1]) then
		"{" ++ join(", ", [show(ranges[i]) | i in index_set(ranges) where i mod 2 = 1]) ++ "}"
	else
		join(
			" union ",
			[
				show(ranges[i]) ++ ".." ++ show(ranges[i + 1]) |
					i in index_set(ranges) where i mod 2 = 1,
			],
		)
	endif;

function string: show(tuple($T): x) = "(" ++ show(x.1) ++ ",)";

function string: show_internal($T: x);

function string: showJSON($T: x) :: mzn_internal_generated = show_json_internal(x);

function string: showJSON(opt $T: x) = if occurs(x) then showJSON(deopt(x)) else "null" endif;

function string: showJSON(array [$X] of $T: x) =
	"[" ++ join(", ", [showJSON(x_i) | x_i in x]) ++ "]";
function string: showJSON(array [$X] of opt $T: x) =
	"[" ++ join(", ", [showJSON(x_i) | x_i in x]) ++ "]";

function string: showJSON(set of $T: x) =
	"{\"set\": [" ++ join(", ", [showJSON(x_i) | x_i in x]) ++ "]}";
function string: showJSON(set of int: x) =
	let {
		any: ranges = set_to_ranges(x);
	} in "{\"set\": [" ++
		join(
			", ",
			[
				if l = u then showJSON(l) else showJSON([l, u]) endif |
					i in index_set(ranges) where i mod 2 = 1,
					l = ranges[i],
					u = ranges[i + 1],
			],
		) ++
		"]}";
function string: showJSON(set of float: x) =
	let {
		any: ranges = set_to_ranges(x);
	} in "{\"set\": [" ++
		join(
			", ",
			[
				if l = u then showJSON(l) else showJSON([l, u]) endif |
					i in index_set(ranges) where i mod 2 = 1,
					l = ranges[i],
					u = ranges[i + 1],
			],
		) ++
		"]}";

function string: show_json_internal($T: x);

% TODO: Remove this when no longer be needed
function string: show_index_sets(array [$U] of any $T: x) = "unreachable";

function string: show_index_sets(array [$$E] of any $T: x) =
	let { any: idxs = index_sets(x) } in "[" ++ mzn_show_index_set(idxs.1) ++ "]";
function string: show_index_sets(array [$$E, $$F] of any $T: x) =
	let {
		any: idxs = index_sets(x);
	} in "[" ++ join(", ", [mzn_show_index_set(idxs.1), mzn_show_index_set(idxs.2)]) ++ "]";
function string: show_index_sets(array [$$E, $$F, $$G] of any $T: x) =
	let {
		any: idxs = index_sets(x);
	} in "[" ++
		join(
			", ",
			[mzn_show_index_set(idxs.1), mzn_show_index_set(idxs.2), mzn_show_index_set(idxs.3)],
		) ++
		"]";
function string: show_index_sets(array [$$E, $$F, $$G, $$H] of any $T: x) =
	let {
		any: idxs = index_sets(x);
	} in "[" ++
		join(
			", ",
			[
				mzn_show_index_set(idxs.1),
				mzn_show_index_set(idxs.2),
				mzn_show_index_set(idxs.3),
				mzn_show_index_set(idxs.4),
			],
		) ++
		"]";
function string: show_index_sets(array [$$E, $$F, $$G, $$H, $$I] of any $T: x) =
	let {
		any: idxs = index_sets(x);
	} in "[" ++
		join(
			", ",
			[
				mzn_show_index_set(idxs.1),
				mzn_show_index_set(idxs.2),
				mzn_show_index_set(idxs.3),
				mzn_show_index_set(idxs.4),
				mzn_show_index_set(idxs.5),
			],
		) ++
		"]";
function string: show_index_sets(array [$$E, $$F, $$G, $$H, $$I, $$J] of any $T: x) =
	let {
		any: idxs = index_sets(x);
	} in "[" ++
		join(
			", ",
			[
				mzn_show_index_set(idxs.1),
				mzn_show_index_set(idxs.2),
				mzn_show_index_set(idxs.3),
				mzn_show_index_set(idxs.4),
				mzn_show_index_set(idxs.5),
				mzn_show_index_set(idxs.6),
			],
		) ++
		"]";

function string: mzn_show_index_set(set of $$E: idx) =
	if enum_of(idx) == idx then enum_name_internal(enum_of(idx)) else show(idx) endif;
