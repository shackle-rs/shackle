% Definitions to keep verbatim for compatibility with old MiniZinc

function $T: 'default'(opt $T:x, $T: y);
function opt $T: 'default'(opt $T:x, opt $T: y);
function var $T: 'default'(var opt $T:x, var $T: y);
function var opt $T: 'default'(var opt $T:x, var opt $T: y);
function array[$U] of $T: 'default'(array[$U] of $T:x, array[$U] of $T: y);
function array[$U] of opt $T: 'default'(array[$U] of opt $T:x, array[$U] of opt $T: y);
function array[$U] of var $T: 'default'(array[$U] of var $T:x, array[$U] of var $T: y);
function array[$U] of var opt $T: 'default'(array[$U] of var opt $T:x, array[$U] of var opt $T: y);

function $$E: to_enum(set of $$E: X, int: x);

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
    
function tuple(set of $$E): index_sets(array[$$E] of any $U: x) = (index_set(x),);
function tuple(set of $$E, set of $$F): index_sets(array[$$E, $$F] of any $U: x) =
  (index_set_1of2(x), index_set_2of2(x));
function tuple(set of $$E, set of $$F, set of $$G): index_sets(array[$$E, $$F, $$G] of any $U: x) =
  (index_set_1of3(x), index_set_2of3(x), index_set_3of3(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H): index_sets(array[$$E, $$F, $$G, $$H] of any $U: x) =
  (index_set_1of4(x), index_set_2of4(x), index_set_3of4(x), index_set_4of4(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I): index_sets(array[$$E, $$F, $$G, $$H, $$I] of any $U: x) =
  (index_set_1of5(x), index_set_2of5(x), index_set_3of5(x), index_set_4of5(x), index_set_5of5(x));
function tuple(set of $$E, set of $$F, set of $$G, set of $$H, set of $$I, set of $$J): index_sets(array[$$E, $$F, $$G, $$H, $$I, $$J] of any $U: x) =
  (index_set_1of6(x), index_set_2of6(x), index_set_3of6(x), index_set_4of6(x), index_set_5of6(x), index_set_6of6(x));

function any $T: mzn_element_internal(array [int] of any $T: x, int: index) = x[index];
function any $T: mzn_element_internal(array [int] of any $T: x, tuple(int): index) = x[index.1];
function any $T: mzn_element_internal(array [int, int] of any $T: x, tuple(int, int): indices) = x[indices.1, indices.2];
function any $T: mzn_element_internal(array [int, int, int] of any $T: x, tuple(int, int, int): indices) = x[indices.1, indices.2, indices.3];
function any $T: mzn_element_internal(array [int, int, int, int] of any $T: x, tuple(int, int, int, int): indices) = x[indices.1, indices.2, indices.3, indices.4];
function any $T: mzn_element_internal(array [int, int, int, int, int] of any $T: x, tuple(int, int, int, int, int): indices) = x[indices.1, indices.2, indices.3, indices.4, indices.5];
function any $T: mzn_element_internal(array [int, int, int, int, int, int] of any $T: x, tuple(int, int, int, int, int, int): indices) = x[indices.1, indices.2, indices.3, indices.4, indices.5, indices.6];

function var int: erase_enum(var $$E) ::mzn_internal_representation;
function int: erase_enum($$E) ::mzn_internal_representation;
function var opt int: erase_enum(var opt $$E) ::mzn_internal_representation;
function opt int: erase_enum(opt $$E) ::mzn_internal_representation;
function var set of int: erase_enum(var set of $$E: x) ::mzn_internal_representation;

function set of $$E: index_set(array[$$E] of any $U: x);
function set of $$E: index_set_1of2(array[$$E,_] of any $U: x);
function set of $$E: index_set_2of2(array[_,$$E] of any $U: x);
function set of $$E: index_set_1of3(array[$$E,_,_] of any $U: x);
function set of $$E: index_set_2of3(array[_,$$E,_] of any $U: x);
function set of $$E: index_set_3of3(array[_,_,$$E] of any $U: x);
function set of $$E: index_set_1of4(array[$$E,_,_,_] of any $U: x);
function set of $$E: index_set_2of4(array[_,$$E,_,_] of any $U: x);
function set of $$E: index_set_3of4(array[_,_,$$E,_] of any $U: x);
function set of $$E: index_set_4of4(array[_,_,_,$$E] of any $U: x);
function set of $$E: index_set_1of5(array[$$E,_,_,_,_] of any $U: x);
function set of $$E: index_set_2of5(array[_,$$E,_,_,_] of any $U: x);
function set of $$E: index_set_3of5(array[_,_,$$E,_,_] of any $U: x);
function set of $$E: index_set_4of5(array[_,_,_,$$E,_] of any $U: x);
function set of $$E: index_set_5of5(array[_,_,_,_,$$E] of any $U: x);
function set of $$E: index_set_1of6(array[$$E,_,_,_,_,_] of any $U: x);
function set of $$E: index_set_2of6(array[_,$$E,_,_,_,_] of any $U: x);
function set of $$E: index_set_3of6(array[_,_,$$E,_,_,_] of any $U: x);
function set of $$E: index_set_4of6(array[_,_,_,$$E,_,_] of any $U: x);
function set of $$E: index_set_5of6(array[_,_,_,_,$$E,_] of any $U: x);
function set of $$E: index_set_6of6(array[_,_,_,_,_,$$E] of any $U: x);


function array[$$F] of any $T: slice_1d(array[$E] of any $T: x, array[int] of set of int: s, set of $$F: dims1);
function array[$$F,$$G] of any $T: slice_2d(array[$E] of any $T: x, array[int] of set of int: s, set of $$F: dims1, set of $$G: dims2);
function array[$$F,$$G,$$H] of any $T: slice_3d(array[$E] of any $T: x, array[int] of set of int: s,
                                                set of $$F: dims1, set of $$G: dims2, set of $$H: dims3);
function array[$$F,$$G,$$H,$$I] of any $T: slice_4d(array[$E] of any $T: x, array[int] of set of int: s,
  set of $$F: dims1, set of $$G: dims2, set of $$H: dims3, set of $$I: dims4);
function array[$$F,$$G,$$H,$$I,$$J] of any $T: slice_5d(array[$E] of any $T: x, array[int] of set of int: s,
  set of $$F: dims1, set of $$G: dims2, set of $$H: dims3, set of $$I: dims4, set of $$J: dims5);
function array[$$F,$$G,$$H,$$I,$$J,$$K] of any $T: slice_6d(array[$E] of any $T: x, array[int] of set of int: s,
  set of $$F: dims1, set of $$G: dims2, set of $$H: dims3, set of $$I: dims4, set of $$J: dims5, set of $$K: dims6);

function var opt $T: mzn_construct_partial(tuple(var bool, var opt $T): x) = let { constraint x.1 } in x.2;
function var $T: mzn_construct_partial(tuple(var bool, var $T): x) = let { constraint x.1 } in x.2;
function opt $T: mzn_construct_partial(tuple(bool, opt $T): x) = let { constraint x.1 } in x.2;
function $T: mzn_construct_partial(tuple(bool, $T): x) = let { constraint x.1 } in x.2;
function array [$X] of var opt $T: mzn_construct_partial(tuple(var bool, array [$X] of var opt $T): x) = let { constraint x.1 } in x.2;
function array [$X] of var $T: mzn_construct_partial(tuple(var bool, array [$X] of var $T): x) = let { constraint x.1 } in x.2;
function array [$X] of opt $T: mzn_construct_partial(tuple(bool, array [$X] of opt $T): x) = let { constraint x.1 } in x.2;
function array [$X] of $T: mzn_construct_partial(tuple(bool, array [$X] of $T): x) = let { constraint x.1 } in x.2;

function var $T: element_t(array [int] of var $T: x, var int: idx) = abort("Called internal function");
function var $T: element_t(array [int, int] of var $T: x, var int: idx1, var int: idx2) = abort("Called internal function");
function var $T: element_mt(array [int] of var $T: x, var int: idx) = abort("Called internal function");
function var $T: element_mt(array [int, int] of var $T: x, var int: idx1, var int: idx2) = abort("Called internal function");
