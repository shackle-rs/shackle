//! HIR common identifiers
use crate::{Db, Identifier};

macro_rules! id_registry {
	($struct:ident, $($tail:tt)*) => {
		id_registry!(@def $struct all ($($tail)*) ());
		id_registry!(@imp $struct db all ($($tail)*) ());
	};

	(@def $struct:ident $all:ident ($($name:ident $(:$value:expr)?)?) ($($rest:tt)*)) => {
		/// Registry for common identifiers
		#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
		pub struct $struct<'db> {
			/// All identifiers
			pub $all: Vec<Identifier<'db>>,
			$($rest)*
			$(
				#[allow(missing_docs, reason = "Fields are self-explanatory")]
				pub $name: Identifier<'db>,
			)?
		}
	};
	(@def $struct:ident $all:ident ($name:ident $(:$value:expr)?, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@def $struct $all ($($todo)*) (
			$($rest)*
			#[allow(missing_docs, reason = "Fields are self-explanatory")]
			pub $name: Identifier<'db>,
		));
	};

	(@imp $struct:ident $db:ident  $all:ident () ($($rest:tt)*)) => {
		impl<'db> $struct<'db> {
			/// Create a new identifier registry
			pub fn new($db: &'db dyn Db) -> Self {
				let mut $all = Vec::new();
				Self {
					$($rest)*
					$all
				}
			}
		}
	};
	(@imp $struct:ident $db:ident $all:ident ($name:ident, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@imp $struct $db $all ($($todo)*) (
			$($rest)*
			$name: { let ident = Identifier::new($db, stringify!($name)); $all.push(ident); ident },
		));
	};

	(@imp $struct:ident $db:ident $all:ident ($name:ident: $value:expr, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@imp $struct $db $all ($($todo)*) (
			$($rest)*
			$name: { let ident = Identifier::new($db, $value); $all.push(ident); ident },
		));
	};
}

id_registry!(
	Annotations,
	annotated_expression,
	output_only,
	shackle_type,
	empty_annotation,
	mzn_var_where_clause,
	mzn_fresh_var,
	promise_total,
	promise_commutative,
	promise_ctx_monotone,
	promise_ctx_antitone,
	output,
	no_output,
	mzn_inline,
	mzn_inline_call_by_name,
	mzn_unreachable,
	mzn_opt_bool,
	mzn_builtin,
	mzn_internal_representation,
	mzn_internal_generated,
);

id_registry!(
	Literals,
	empty_string: "",
	return_value: "<return value>",
	default,
);

id_registry!(
	Names,
	objective: "_objective",
	this: "this",
);

id_registry!(
	Functions,
	abort,
	abs,
	acos,
	acosh,
	and: "/\\",
	annotate,
	arg_max,
	arg_max_internal,
	arg_min,
	arg_min_internal,
	array1d,
	array_access: "[]",
	array_union,
	array_xd: "arrayXd",
	asin,
	asinh,
	assert_: "assert",
	atan,
	atanh,
	bernoulli,
	binomial,
	bool2float,
	bool2int,
	card,
	cauchy,
	ceil,
	chisquared,
	clause,
	compute_div_bounds,
	compute_float_div_bounds,
	compute_mod_bounds,
	compute_pow_bounds,
	concat,
	cos,
	cosh,
	default,
	deopt,
	diff,
	discrete_distribution,
	div,
	div_float: "/",
	div_internal_float,
	div_internal_int,
	dom,
	dom_array,
	dom_bounds_array,
	dot_dot: "..",
	enum2int,
	enum_of,
	eq: "=",
	exists,
	exp,
	exponential,
	fdistribution,
	fix,
	floor,
	forall,
	format,
	format_justify_string,
	gamma,
	ge: ">=",
	gt: ">",
	has_ann,
	has_bounds,
	has_ub_set,
	if_then_else,
	iff: "<->",
	iffall,
	implies: "->",
	in_: "in",
	index2int,
	index_set,
	index_sets,
	index_sets_agree,
	int2float,
	intersect,
	is_fixed,
	is_same,
	join,
	lb,
	lb_array,
	le: "<=",
	length,
	ln,
	log10,
	lognormal,
	logstream_to_string,
	lt: "<",
	max,
	max_internal_array,
	max_internal_set,
	min,
	min_internal_array,
	min_internal_set,
	minus: "-",
	modulo: "mod",
	mzn_add_warning,
	mzn_array2set,
	mzn_array_2d_literal,
	mzn_array_access_known_valid,
	mzn_array_access_valid,
	mzn_array_kd,
	mzn_card_constraint,
	mzn_check_index_set,
	mzn_compiler_version,
	mzn_construct_enum,
	mzn_construct_opt,
	mzn_default_partial,
	mzn_defining_set,
	mzn_destruct_enum,
	mzn_destruct_opt,
	mzn_domain_constraint,
	mzn_element_internal,
	mzn_get_enum,
	mzn_get_parameter,
	mzn_in_root_context,
	mzn_indexed_array,
	mzn_infinite_range,
	mzn_internal_check_debug_mode,
	mzn_trace_to_section,
	mzn_opt_channel,
	mzn_opt_domain,
	mzn_parse_enum,
	mzn_safe_default,
	mzn_show_array_access,
	mzn_show_enum,
	mzn_show_record_access,
	mzn_show_tuple_access,
	mzn_slice,
	mzn_slice_internal,
	mzn_start_indexed_array,
	mzn_unwrap_bool_tuple,
	ne: "!=",
	normal,
	not,
	occurs,
	or: "\\/",
	output_to_json_section,
	output_to_section,
	plus: "+",
	plus_plus: "++",
	poisson,
	pow,
	pow_internal_float,
	pow_internal_int,
	product,
	range_internal_float,
	range_internal_int,
	redundant_constraint,
	rev_imp: "<-",
	round,
	set2array,
	set2iter,
	set_in: "in",
	set_to_ranges,
	set_to_ranges_internal_float,
	set_to_ranges_internal_int,
	show,
	show_checker_output: "showCheckerOutput",
	show_dzn: "showDzn",
	show_dzn_id: "showDznId",
	show_json: "showJSON",
	show_internal,
	sin,
	sinh,
	sort,
	sort_by,
	sqrt,
	subset,
	sum,
	string_length,
	superset,
	symdiff,
	symmetry_breaking_constraint,
	tan,
	tanh,
	tdistribution,
	times: "*",
	to_enum,
	to_enum_internal,
	trace_exp,
	trace_to_section,
	ub,
	ub_array,
	uniform_internal_float,
	uniform_internal_int,
	union,
	val2opt,
	weibull,
	xor,
	xorall,
);

/// Registry for common identifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::SalsaValue)]
pub struct IdentifierRegistry<'db> {
	/// Annotations
	pub annotations: Annotations<'db>,
	/// Literals for strings
	pub literals: Literals<'db>,
	/// Names of variables
	pub names: Names<'db>,
	/// Non-builtin functions (or compiler erased functions)
	pub functions: Functions<'db>,
}

impl<'db> IdentifierRegistry<'db> {
	/// Get the identifier registry
	pub fn lookup(db: &'db dyn Db) -> &'db Self {
		create_id_registry(db)
	}
}

#[salsa::tracked]
fn create_id_registry<'db>(db: &'db dyn Db) -> IdentifierRegistry<'db> {
	IdentifierRegistry {
		annotations: Annotations::new(db),
		literals: Literals::new(db),
		names: Names::new(db),
		functions: Functions::new(db),
	}
}
