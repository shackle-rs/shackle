function bool: mzn_add_warning(string: msg);
function bool: mzn_assert_warn(bool: b, string: msg) :: mzn_inline_call_by_name =
	let {
		bool: condition = b;
		constraint if condition then true else mzn_add_warning(msg) endif;
	} in condition;

test assert(bool: b, string: msg) :: mzn_inline_call_by_name = if b then true else abort(msg) endif;
function any $T: assert(bool: b, string: msg, any $T: x) :: mzn_inline_call_by_name =
	let { constraint assert(b, msg) } in x;
% TODO: Only needed for compatibility with old minizinc library
function array [$U] of any $T: assert(bool: b, string: msg, array [$U] of any $T: x)
	:: mzn_inline_call_by_name = let { constraint assert(b, msg) } in x;
