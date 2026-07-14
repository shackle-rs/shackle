% Domain constraints
test mzn_domain_constraint(string: name, $$E: x, set of $$E: s) :: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(enum2int(x), enum2int(s)),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, opt $$E: x, set of $$E: s) :: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(enum2int(x), enum2int(s)),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, set of $$E: x, set of $$E: s) :: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(enum2int(x), enum2int(s)),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, float: x, set of float: s) :: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(x, s),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, opt float: x, set of float: s) :: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(x, s),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);
test mzn_domain_constraint(string: name, set of float: x, set of float: s)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		mzn_var_dom(x, s),
		"the type-inst of '" ++ name ++ "' is \(s), but is assigned the value \(x).",
	);

test mzn_card_constraint(string: name, set of $T: x, set of int: cardinality)
	:: mzn_inline_call_by_name =
	mzn_assert_warn(
		card(x) in cardinality,
		"the cardinality of '" ++
			name ++
			"' is \(card(x)), but the type-inst constrains the cardinality to \(cardinality).",
	);
predicate mzn_card_constraint(string: name, var set of $$T: x, set of int: cardinality)
	:: mzn_inline_call_by_name = card(x) in cardinality;

% Symmetry-break default for a scalar var field. The object lowering pins
% the fields of unused potential objects to a canonical in-domain value;
% `lb`/`ub` of an unbounded var are -infinity/infinity, which are not valid
% values, so fall back to the first finite bound, then 0.
function int: mzn_safe_default(var int: x) =
	if lb(x) != -infinity then lb(x) elseif ub(x) != infinity then ub(x) else 0 endif;
function float: mzn_safe_default(var float: x) =
	if lb(x) != -infinity then lb(x) elseif ub(x) != infinity then ub(x) else 0.0 endif;

predicate mzn_domain_constraint(string: name, var $$E: x, set of $$E: s)
	 :: mzn_inline_call_by_name = mzn_var_dom(enum2int(x), enum2int(s));
predicate mzn_domain_constraint(string: name, var opt $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name = mzn_var_dom(enum2int(x), enum2int(s));
predicate mzn_domain_constraint(string: name, var set of $$E: x, set of $$E: s)
	:: mzn_inline_call_by_name = mzn_var_dom(enum2int(x), enum2int(s));
predicate mzn_domain_constraint(string: name, var float: x, set of float: s)
	:: mzn_inline_call_by_name = mzn_var_dom(x, s);
predicate mzn_domain_constraint(string: name, var opt float: x, set of float: s)
	:: mzn_inline_call_by_name = mzn_var_dom(x, s);

test mzn_var_dom(int: x, set of int: s) = x in s;
predicate mzn_var_dom(var int: x, set of int: s) =
	if has_bounds(x) /\ dom(x) subset s then true else x in s endif;

test mzn_var_dom(opt int: x, set of int: s) = absent(x) \/ deopt(x) in s;
predicate mzn_var_dom(var opt int: x, set of int: s) =
	mzn_var_dom(deopt(x), mzn_opt_domain(s)) /\ mzn_opt_channel(x, s);
test mzn_var_dom(set of int: x, set of int: s) = x subset s;
predicate mzn_var_dom(var set of int: x, set of int: s) =
	if has_ub_set(x) /\ ub(x) subset s then true else x subset s endif;

test mzn_var_dom(array [int] of int: x, set of int: d) = forall (x_i in x) (mzn_var_dom(x_i, d));
predicate mzn_var_dom(array [int] of var int: x, set of int: d) =
	forall (x_i in x) (mzn_var_dom(x_i, d));
test mzn_var_dom(array [int] of opt int: x, set of int: d) =
	 forall (x_i in x) (mzn_var_dom(x_i, d));
predicate mzn_var_dom(array [int] of var opt int: x, set of int: d) =
	forall (x_i in x) (mzn_var_dom(x_i, d));
test mzn_var_dom(array [int] of set of int: x, set of int: d) =
	forall (x_i in x) (mzn_var_dom(x_i, d));
predicate mzn_var_dom(array [int] of var set of int: x, set of int: d) =
	forall (x_i in x) (mzn_var_dom(x_i, d));

test mzn_var_dom(float: x, float: l, float: u) = x >= l /\ x <= u;
predicate mzn_var_dom(var float: x, float: l, float: u) =
	if has_bounds(x) /\ lb(x) >= l /\ ub(x) <= u then true else x >= l /\ x <= u endif;

test mzn_var_dom(float: x, set of float: s) = x in s;
predicate mzn_var_dom(var float: x, set of float: s) = x in s;

test mzn_var_dom(opt float: x, set of float: s) = absent(x) \/ deopt(x) in s;
predicate mzn_var_dom(var opt float: x, set of float: s) =
	mzn_var_dom(deopt(x), mzn_opt_domain(s)) /\ mzn_opt_channel(x, s);

test mzn_var_dom(set of float: x, set of float: s) = x subset s;

test mzn_var_dom(array [int] of float: x, float: l, float: u) =
	forall (x_i in x) (mzn_var_dom(x_i, l, u));
predicate mzn_var_dom(array [int] of var float: x, float: l, float: u) =
	forall (x_i in x) (mzn_var_dom(x_i, l, u));
test mzn_var_dom(array [int] of float: x, set of float: d) =
	 forall (x_i in x) (mzn_var_dom(x_i, d));
predicate mzn_var_dom(array [int] of var float: x, set of float: d) =
	forall (x_i in x) (mzn_var_dom(x_i, d));

test mzn_check_index_set(string: name, set of $$E: actual, set of $$E: expected)
	:: mzn_inline_call_by_name = mzn_check_index_set(name, 1, 1, actual, expected);
test mzn_check_index_set(string: name, int: d, int: dims, set of $$E: actual, set of $$E: expected)
	:: mzn_inline_call_by_name =
	assert(
		actual = expected,
		"Index set " ++
			if dims > 1 then "\(d) " endif ++
			"of '" ++
			name ++
			"' is \(expected), but is assigned an array with " ++
			if dims > 1 then "the corresponding " endif ++
			"index set \(actual). You may need to coerce the index sets using the array\(
				dims
			)d function.",
	);

test has_bounds(var opt $$E: x) = has_bounds(mzn_destruct_opt(x).2);
test has_bounds(var opt float: x) = has_bounds(mzn_destruct_opt(x).2);

function array [int] of var bool: mzn_unwrap_bool_tuple(array [int] of tuple(var bool): x) = x.1;
