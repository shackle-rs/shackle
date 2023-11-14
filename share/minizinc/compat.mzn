% Definitions to keep verbatim for compatibility with old MiniZinc

function $T: 'default'(opt $T:x, $T: y);
function opt $T: 'default'(opt $T:x, opt $T: y);
function var $T: 'default'(var opt $T:x, var $T: y);
function var opt $T: 'default'(var opt $T:x, var opt $T: y);
function array[$U] of $T: 'default'(array[$U] of $T:x, array[$U] of $T: y);
function array[$U] of opt $T: 'default'(array[$U] of opt $T:x, array[$U] of opt $T: y);
function array[$U] of var $T: 'default'(array[$U] of var $T:x, array[$U] of var $T: y);
function array[$U] of var opt $T: 'default'(array[$U] of var opt $T:x, array[$U] of var opt $T: y);

function set of $$E: enum_of(var opt $$E: x) ::mzn_internal_representation;
function set of $$E: enum_of(array[$T] of var opt $$E: x) ::mzn_internal_representation;

function string: show2d_indexed(array[int] of string: row_hdr,
                                array[int] of string: col_hdr,
                                array[int,int] of string: vs) =
  let {
      int: row_hdr_max = max([0]++[string_length(r)+1 | r in row_hdr]);
      string: row_hdr_offset = if row_hdr_max = 0 then " " else concat([" " | _ in 1..row_hdr_max+2]) endif;
      array[int] of int: col_hdr_max =
        [ j : max([if length(col_hdr)=0 then 0 else string_length(col_hdr[j]) endif]++
              [string_length(vs[i,j]) | i in index_set_1of2(vs)]) | j in index_set_2of2(vs)];
  } in if length(vs) = 0 then "[| |]" else
  if length(col_hdr)=0 then "["
  else 
  "[|"++row_hdr_offset++concat(i in index_set(col_hdr))(format_justify_string(col_hdr_max[i],col_hdr[i])++": ")++"\n "
  endif
  ++concat([
      "| "++
      if length(row_hdr) > 0 then format_justify_string(row_hdr_max-1, row_hdr[i])++": " endif++
      join(", ", [format_justify_string(col_hdr_max[j], vs[i,j]) | j in index_set_2of2(vs)])++"\n "
  | i in index_set_1of2(vs)
  ])++"|]" endif;

function $$E: lb(var opt $$E: x);
function $$E: ub(var opt $$E: x);
function float: lb(var opt float: x);
function float: ub(var opt float: x);
function $$E: lb_array(array[$U] of var opt $$E: x);
function $$E: ub_array(array[$U] of var opt $$E: x);
function float: lb_array(array[$U] of var opt float: x);
function float: ub_array(array[$U] of var opt float: x);
function set of $$E: dom(var opt $$E: x);
function set of $$E: dom_array(array[$T] of var opt $$E: x);
function set of int: dom_bounds_array(array[$T] of var opt int: x);

function $$E: lb_array(array[$U] of var $$E: x) = let {
    array [int] of var opt int: xx = array1d(x);
} in to_enum(enum_of(x), lb_array(xx));

function $$E: ub_array(array[$U] of var $$E: x) = let {
    array [int] of var opt int: xx = array1d(x);
} in to_enum(enum_of(x), ub_array(xx));

function float: lb_array(array[$U] of var float: x) = let {
    array [int] of var opt float: xx = array1d(x);
} in lb_array(xx);

function float: ub_array(array[$U] of var float: x) = let {
    array [int] of var opt float: xx = array1d(x);
} in ub_array(xx);

function opt $T: fix(var opt $T: x);
function array[$U] of opt $T: fix(array[$U] of var opt $T: x);

function $$T: deopt(opt $$T: x);
function $T: deopt(opt $T: x);
test occurs(opt $T: x);

function set of $$E: enum_of_internal(set of $$E: e) = e;

function bool: mzn_add_warning(string: msg) = trace("Warning: " ++ msg ++ "\n");

test mzn_check_only_range_domains() = if mzn_opt_only_range_domains.1 then mzn_opt_only_range_domains.2 else false endif;
test mzn_check_annotate_defines_var() = if mzn_opt_annotate_defines_var.1 then mzn_opt_annotate_defines_var.2 else true endif;
test mzn_check_ignore_symmetry_breaking_constraints() = if mzn_ignore_symmetry_breaking_constraints.1 then mzn_ignore_symmetry_breaking_constraints.2 else false endif;
test mzn_check_ignore_redundant_constraints() = if mzn_ignore_redundant_constraints.1 then mzn_ignore_redundant_constraints.2 else false endif;
test mzn_check_half_reify_clause() = if mzn_half_reify_clause.1 then mzn_half_reify_clause.2 else true endif;

function ann: 'output'(any $T: x :: annotated_expression);
function ann: 'output'(array [$U] of any $T: x :: annotated_expression);

predicate if_then_else(array[int] of var bool: c, array[int] of int: x, var int: y) =
    fzn_if_then_else_int(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of var int: x, var int: y) =
    fzn_if_then_else_var_int(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of bool: x, var bool: y) =
    fzn_if_then_else_bool(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of var bool: x, var bool: y) =
    fzn_if_then_else_var_bool(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of float: x, var float: y) =
    fzn_if_then_else_float(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of var float: x, var float: y) =
    fzn_if_then_else_var_float(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of set of int: x, var set of int: y) =
    fzn_if_then_else_set(c, x, y);
predicate if_then_else(array[int] of var bool: c, array[int] of var set of int: x, var set of int: y) =
    fzn_if_then_else_var_set(c, x, y);

function string: show_index_sets(array[$U] of any $T: x);
