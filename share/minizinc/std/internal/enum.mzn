function bool: enum2int(bool: x) :: mzn_internal_representation;
function var bool: enum2int(var bool: x) :: mzn_internal_representation;

function opt bool: enum2int(opt bool: x) :: mzn_internal_representation;
function var opt bool: enum2int(var opt bool: x) :: mzn_internal_representation;

function opt set of bool: enum2int(opt set of bool: x) :: mzn_internal_representation;
function set of bool: enum2int(set of bool: x) :: mzn_internal_representation;
function var set of bool: enum2int(var set of bool: x) :: mzn_internal_representation;

function array [$X] of bool: enum2int(array [$X] of bool: x) :: mzn_internal_representation;
function array [$X] of var bool: enum2int(array [$X] of var bool: x) :: mzn_internal_representation;

function array [$X] of opt bool: enum2int(array [$X] of opt bool: x) :: mzn_internal_representation;
function array [$X] of var opt bool: enum2int(array [$X] of var opt bool: x)
	:: mzn_internal_representation;

function array [$X] of opt set of bool: enum2int(array [$X] of opt set of bool: x)
	:: mzn_internal_representation;
function array [$X] of set of bool: enum2int(array [$X] of set of bool: x)
	:: mzn_internal_representation;
function array [$X] of var set of bool: enum2int(array [$X] of var set of bool: x)
	:: mzn_internal_representation;

% (cardinality, [(constrcutor name, [(param enum id, domain)], offset)])
type MznEnum = tuple(int, array [int] of tuple(string, array [int] of tuple(int, set of int), int));

function MznEnum: mzn_get_enum(
	array [int] of tuple(string, array [int] of tuple(int, set of int)): e,
) = (
	sum (
		x in e,
	) (
		product (y in x.2) (card(y.2))
	),
	[
		let {
			any: (name, params) = e[i];
			any: offset = sum (x in e[1..i - 1]) (product (y in x.2) (card(y.2)));
		} in (name, params, offset) |
			i in 1..length(e),
	],
);
function MznEnum: mzn_get_enum(array [int] of string: e) = mzn_get_enum([(e_i, []) | e_i in e]);

function set of int: mzn_defining_set(MznEnum: x) = 1..x.1;

function int: mzn_construct_enum(MznEnum: e, int: i) = e.2[i].3 + 1;

function int: mzn_construct_enum(MznEnum: e, int: i, array [int] of int: args) =
	let {
		any: (name, params, offset) = e.2[i];
		constraint
			forall (a in index_set(args) where params[a].1 == 0) (
				mzn_assert_warn(
					args[a] in params[a].2,
					"The value for argument \(a) of the enum constructor '" ++
						name ++
						"' is outside the declared range of \(params[a].2).",
				)
			);
		any: value =
			sum (a in index_set(args)) (
				let {
					any: p = product (b in a + 1..max(index_set(args))) (card(params[b].2));
					any: v =
						if mzn_set_is_contiguous(params[a].2) then
							args[a] - min(params[a].2)
						else
							count (z in params[a].2) (z < args[a])
						endif;
				} in p * v
			);
	} in offset + value + 1;

function opt int: mzn_construct_enum(MznEnum: e, int: i, array [int] of opt int: args) =
	if forall (a in args) (occurs(a)) then
		mzn_construct_enum(e, i, [deopt(a) | a in args])
	else
		<>
	endif;

function var int: mzn_construct_enum(MznEnum: e, int: i, array [int] of var int: args) =
	let {
		any: (name, params, offset) = e.2[i];
		constraint forall (a in index_set(args) where params[a].1 == 0) (args[a] in params[a].2);
		any: value =
			sum (a in index_set(args)) (
				let {
					any: p = product (b in a + 1..max(index_set(args))) (card(params[b].2));
					any: v =
						if mzn_set_is_contiguous(params[a].2) then
							args[a] - min(params[a].2)
						else
							count (z in params[a].2) (z < args[a])
						endif;
				} in p * v
			);
	} in offset + value + 1;

function var opt int: mzn_construct_enum(MznEnum: e, int: i, array [int] of var opt int: args) =
	if forall (a in args) (occurs(a)) then
		mzn_construct_enum(e, i, [deopt(a) | a in args])
	else
		<>
	endif;

function set of int: mzn_construct_enum(MznEnum: e, int: i, array [int] of set of int: args) = {
	mzn_construct_enum(e, i, v) |
		(v,) in mzn_cartesian_product(args),
};

function var set of int: mzn_construct_enum(
	MznEnum: e,
	int: i,
	array [int] of var set of int: args,
) =
	array2set([
		if forall (idx in index_set(args)) (v[idx] in args[idx]) then
			mzn_construct_enum(e, i, v)
		else
			<>
		endif |
			(v,) in mzn_cartesian_product([ub(a) | a in args]),
	]);

function array [int] of int: mzn_destruct_enum(MznEnum: e, int: i, int: v) =
	let {
		any: (name, params, offset) = e.2[i];
		any: lower = offset + 1;
		any: upper = if i == length(e.2) then e.1 else e.2[i + 1].3 endif;
		constraint
			mzn_assert_warn(
				v in lower..upper,
				"Cannot destructure value using " ++ name ++ "(...).",
			);
	} in [
		let {
			any: d =
				(v - offset - 1) div
					product (j in p + 1..max(index_set(params))) (card(params[j].2)) mod
					card(params[p].2);
			any: r =
				if mzn_set_is_contiguous(params[p].2) then
					min(params[p].2) + d
				else
					set2array(params[p].2)[d + 1]
				endif;
		} in r |
			p in index_set(params),
	];

function array [int] of var int: mzn_destruct_enum(MznEnum: e, int: i, var int: v) =
	let {
		any: (name, params, offset) = e.2[i];
		any: lower = offset + 1;
		any: upper = if i == length(e.2) then e.1 else e.2[i + 1].3 endif;
		constraint v in lower..upper;
	} in [
		let {
			any: d =
				(v - offset - 1) div
					product (j in p + 1..max(index_set(params))) (card(params[j].2)) mod
					card(params[p].2);
			any: r =
				if mzn_set_is_contiguous(params[p].2) then
					min(params[p].2) + d
				else
					set2array(params[p].2)[d + 1]
				endif;
		} in r |
			p in index_set(params),
	];

function array [int] of opt int: mzn_destruct_enum(MznEnum: e, int: i, opt int: v) =
	if occurs(v) then mzn_destruct_enum(e, i, deopt(v)) else [<> | p in e.2[i].2] endif;

function array [int] of var opt int: mzn_destruct_enum(MznEnum: e, int: i, var opt int: v) = [
	if occurs(v) then mzn_destruct_enum(e, i, deopt(v))[p] else <> endif |
		p in index_set(e.2[i].2),
];

function array [int] of set of int: mzn_destruct_enum(MznEnum: e, int: i, set of int: v) = [
	{mzn_destruct_enum(e, i, v_i)[p] | v_i in v} |
		p in index_set(e.2[i].2),
];

function array [int] of var set of int: mzn_destruct_enum(MznEnum: e, int: i, var set of int: v) = [
	{mzn_destruct_enum(e, i, v_i)[p] | v_i in v} |
		p in index_set(e.2[i].2),
];

function tuple(string, array [int] of tuple(int, int)): mzn_unpack_enum(MznEnum: e, int: v) =
	let {
		any: i = arg_max([v <= i.3 | i in e.2] ++ [true]) - 1;
		any: (name, def, offset) = e.2[i];
		any: args =
			if length(def) > 0 then
				let {
					any: d = mzn_destruct_enum(e, i, v);
					any: s = [(def[idx].1, d[idx]) | idx in index_set(d)];
				} in s
			else
				[]
			endif;
	} in (name, args);

function string: mzn_show_enum(array [int] of MznEnum: enums, int: e, int: v) =
	let {
		any: (name, ctors) = mzn_unpack_enum(enums[e], v);
		any: args =
			if length(ctors) > 0 then
				"(" ++
					join(
						", ",
						[
							if ctor.1 == 0 then
								show(ctor.2)
							else
								mzn_show_enum(enums, ctor.1, ctor.2)
							endif |
								ctor in ctors,
						],
					) ++
					")"
			endif;
	} in name ++ args;

function string: mzn_show_json_enum(array [int] of MznEnum: enums, int: e, int: v) =
	let {
		any: (name, ctors) = mzn_unpack_enum(enums[e], v);
		any: args =
			if length(ctors) > 0 then
				"(" ++
					join(
						", ",
						[
							if ctor.1 == 0 then
								show(ctor.2)
							else
								mzn_show_json_enum(enums, ctor.1, ctor.2)
							endif |
								ctor in ctors,
						],
					) ++
					")"
			endif;
	} in if length(ctors) > 1 then
		"{\"c\": " ++
			showJSON(name) ++
			", \"e\": [" ++
			join(
				", ",
				[
					if ctor.1 == 0 then
						show(ctor.2)
					else
						mzn_show_json_enum(enums, ctor.1, ctor.2)
					endif |
						ctor in ctors,
				],
			) ++
			"]}"
	elseif length(ctors) = 1 then
		let {
			any: ctor = ctors[1];
		} in "{\"c\": " ++
			showJSON(name) ++
			", \"e\": " ++
			if ctor.1 == 0 then
				showJSON(ctor.2)
			else
				mzn_show_json_enum(enums, ctor.1, ctor.2)
			endif ++
			"}"
	else
		"{\"e\": " ++ showJSON(name) ++ "}"
	endif;

function array [int] of tuple(array [int] of int): mzn_cartesian_product(
	array [int] of set of int: x,
	int: i,
) =
	if length(x) == 0 then
		[]
	elseif i == max(index_set(x)) then
		[([j],) | j in x[i]]
	else
		[([j] ++ k,) | j in x[i], (k,) in mzn_cartesian_product(x, i + 1)]
	endif;

function array [int] of tuple(array [int] of int): mzn_cartesian_product(
	array [int] of set of int: x,
) = mzn_cartesian_product(x, 1);

function set of int: mzn_infinite_range() = -infinity..infinity;
