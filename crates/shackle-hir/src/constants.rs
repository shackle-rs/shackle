//! HIR common identifiers
use crate::{Db, Identifier};

macro_rules! id_registry {
	($struct:ident, $($tail:tt)*) => {
		id_registry!(@def $struct all ($($tail)*) ());
		id_registry!(@imp $struct db all ($($tail)*) ());
	};

	(@def $struct:ident $all:ident ($($name:ident $(:$value:expr)?)?) ($($rest:tt)*)) => {
		/// Registry for common identifiers
		#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
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
	Builtins,
	mzn_get_parameter,
	forall,
	exists,
	mzn_indexed_array,
	mzn_element_internal,
	mzn_slice_internal,
	mzn_array2set,
	plus_plus: "++",
	length,
	index_sets_agree,
	index_sets,
	array_xd: "arrayXd",
	mzn_array_kd,
	compute_div_bounds,
	compute_mod_bounds,
	compute_float_div_bounds,
	compute_pow_bounds,
	normal,
	uniform_float,
	uniform_int,
	poisson,
	gamma,
	weibull,
	exponential,
	lognormal,
	chisquared,
	cauchy,
	fdistribution,
	tdistribution,
	discrete_distribution,
	bernoulli,
	binomial,
	mzn_add_warning,
	trace_exp,
	trace_to_section,
	logstream_to_string,
	abort,
	mzn_internal_check_debug_mode,
	lb,
	ub,
	lb_array,
	ub_array,
	dom,
	dom_array,
	dom_bounds_array,
	has_bounds,
	has_ub_set,
	is_fixed,
	fix,
	has_ann,
	annotate,
	is_same,
	mzn_compiler_version,
	concat,
	join,
	lt: "<",
	le: "<=",
	ne: "!=",
	eq: "==",
	and: "/\\",
	or: "\\/",
	implies: "->",
	xor,
	not,
	xorall,
	iffall,
	clause,
	sort,
	sort_by,
	show,
	show_dzn,
	show_dzn_id,
	show_checker_output,
	show_json,
	format,
	format_justify_string,
	output_to_section,
	output_to_json_section,
	dot_dot: "..",
	set_in: "in",
	subset,
	superset,
	union,
	intersect,
	diff,
	symdiff,
	set_to_ranges,
	ceil,
	floor,
	round,
	set2array,
	plus: "+",
	minus: "-",
	times: "*",
	pow,
	div,
	modulo: "mod",
	div_float: "/",
	sum,
	product,
	min,
	max,
	arg_min,
	arg_max,
	abs,
	sqrt,
	exp,
	ln,
	log10,
	sin,
	cos,
	tan,
	asin,
	acos,
	atan,
	sinh,
	cosh,
	tanh,
	asinh,
	acosh,
	atanh,
	mzn_default_partial,
	mzn_in_root_context,
);

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
	forall,
	exists,
	sum,
	show,
	show_json: "showJSON",
	show_dzn: "showDzn",
	join,
	plus_plus: "++",
	concat,
	and: "/\\",
	or: "\\/",
	not,
	times: "*",
	lb,
	ub,
	implies: "->",
	in_: "in",
	array_xd: "arrayXd",
	mzn_start_indexed_array,
	array_access: "[]",
	mzn_slice,
	index_set,
	mzn_infinite_range,
	set2iter,
	symmetry_breaking_constraint,
	redundant_constraint,
	enum2int,
	index2int,
	enum_of,
	to_enum,
	to_enum_internal,
	occurs,
	deopt,
	minus: "-",
	eq: "=",
	set2array,
	dot_dot: "..",
	fix,
	is_fixed,
	annotate,
	index_sets,
	mzn_get_enum,
	mzn_defining_set,
	mzn_construct_enum,
	mzn_destruct_enum,
	mzn_parse_enum,
	mzn_show_enum,
	default,
	mzn_construct_opt,
	mzn_destruct_opt,
	mzn_opt_domain,
	mzn_opt_channel,
	mzn_domain_constraint,
	mzn_check_index_set,
	mzn_show_array_access,
	mzn_show_tuple_access,
	mzn_show_record_access,
	mzn_array_access_valid,
	mzn_array_access_known_valid,
	mzn_array_2d_literal,
	if_then_else,
	val2opt,
	bool2int,
	int2float,
	bool2float,
	mzn_unwrap_bool_tuple,
);

/// Registry for common identifiers
#[derive(Clone, Debug, PartialEq, Eq, Hash, salsa::Update)]
pub struct IdentifierRegistry<'db> {
	/// Interpreter builtins
	pub builtins: Builtins<'db>,
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

#[salsa::tracked(returns(ref))]
fn create_id_registry<'db>(db: &'db dyn Db) -> IdentifierRegistry<'db> {
	IdentifierRegistry {
		builtins: Builtins::new(db),
		annotations: Annotations::new(db),
		literals: Literals::new(db),
		names: Names::new(db),
		functions: Functions::new(db),
	}
}
