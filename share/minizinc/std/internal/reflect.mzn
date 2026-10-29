function $$E: lb(var opt $$E: x) = lb(mzn_destruct_opt(x).2);
function $$E: ub(var opt $$E: x) = ub(mzn_destruct_opt(x).2);
function float: lb(var opt float: x) = lb(mzn_destruct_opt(x).2);
function float: ub(var opt float: x) = ub(mzn_destruct_opt(x).2);

function $$E: lb_array(array [$U] of var opt $$E: x) =
	lb_array([mzn_destruct_opt(x_i).2 | x_i in x]);
function $$E: ub_array(array [$U] of var opt $$E: x) =
	ub_array([mzn_destruct_opt(x_i).2 | x_i in x]);
function float: lb_array(array [$U] of var opt float: x) =
	lb_array([mzn_destruct_opt(x_i).2 | x_i in x]);
function float: ub_array(array [$U] of var opt float: x) =
	ub_array([mzn_destruct_opt(x_i).2 | x_i in x]);

function set of $$E: dom(var opt $$E: x) = dom(mzn_destruct_opt(x).2);

function set of $$E: dom_array(array [$T] of var opt $$E: x) =
	dom_array([mzn_destruct_opt(xi).2 | xi in x]);
function set of $$E: dom_bounds_array(array [$T] of var opt $$E: x) =
	dom_bounds_array([mzn_destruct_opt(xi).2 | xi in x]);

function opt $T: fix(var opt $T: x) = mzn_construct_opt(fix(mzn_destruct_opt(x)));
% Fixed element by element: MiniZinc's `fix` of a whole array of tuples keeps the
% elements var
function array [$U] of opt $T: fix(array [$U] of var opt $T: x) =
	arrayXd(x, [fix(x_i) | x_i in x]);
