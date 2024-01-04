use crate::hir::{db::Hir, Identifier};

macro_rules! id_registry {
	($struct:ident, $($tail:tt)*) => {
		id_registry!(@def $struct ($($tail)*) ());
		id_registry!(@imp $struct db ($($tail)*) ());
	};

	(@def $struct:ident ($($name:ident $(:$value:expr)?)?) ($($rest:tt)*)) => {
		/// Registry for common identifiers
		#[derive(Clone, Debug, PartialEq, Eq)]
		pub struct $struct {
			$($rest)*
			$(
				#[allow(missing_docs)]
				pub $name: Identifier,
			)?
		}
	};
	(@def $struct:ident ($name:ident $(:$value:expr)?, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@def $struct ($($todo)*) (
			$($rest)*
			#[allow(missing_docs)]
			pub $name: Identifier,
		));
	};

	(@imp $struct:ident $db:ident ($($name:ident)?) ($($rest:tt)*)) => {
		impl $struct {
			/// Create a new identifier registry
			pub fn new($db: &dyn Hir) -> Self {
				Self {
					$($rest)*
					$(
						$name: Identifier::new(stringify!($name), $db),
					)?
				}
			}
		}
	};
	(@imp $struct:ident $db:ident ($name:ident, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@imp $struct $db ($($todo)*) (
			$($rest)*
			$name: Identifier::new(stringify!($name), $db),
		));
	};


	(@imp $struct:ident $db:ident ($name:ident: $value:expr) ($($rest:tt)*)) => {
		impl $struct {
			/// Create a new identifier registry
			pub fn new($db: &dyn Hir) -> Self {
				Self {
					$($rest)*
					$name: Identifier::new($value, $db)
				}
			}
		}
	};
	(@imp $struct:ident $db:ident ($name:ident: $value:expr, $($todo:tt)*) ($($rest:tt)*)) => {
		id_registry!(@imp $struct $db ($($todo)*) (
			$($rest)*
			$name: Identifier::new($value, $db),
		));
	};
}

pub(crate) use id_registry;

id_registry!(
	IdentifierRegistry,
	empty_string: "",
	annotated_expression,
	output_only,
	array_nd: "arrayNd",
	array_xd: "arrayXd",
	mzn_array_kd: "mzn_array_kd",
	array2d,
	concat,
	join,
	plus_plus: "++",
	dot_dot: "..",
	array2set,
	objective: "_objective",
	show,
	show_dzn: "showDzn",
	show_json: "showJSON",
	is_fixed,
	fix,
	eq: "=",
	index_set,
	index_sets,
	index_set_1of2,
	index_set_2of2,
	index_set_1of3,
	index_set_2of3,
	index_set_3of3,
	index_set_1of4,
	index_set_2of4,
	index_set_3of4,
	index_set_4of4,
	index_set_1of5,
	index_set_2of5,
	index_set_3of5,
	index_set_4of5,
	index_set_5of5,
	index_set_1of6,
	index_set_2of6,
	index_set_3of6,
	index_set_4of6,
	index_set_5of6,
	index_set_6of6,
	shackle_type,
	empty_annotation,
	minus: "-",
	plus: "+",
	times: "*",
	sum,
	product,
	erase_enum,
	mzn_erase_index_sets,
	forall,
	exists,
	set2array,
	annotate,
	to_enum,
	mzn_to_enum,
	occurs,
	deopt,
	ub,
	set2iter,
	in_: "in",
	conj: "/\\",
	disj: "\\/",
	imp: "->",
	card,
	mzn_get_enum,
	mzn_defining_set,
	mzn_construct_enum,
	mzn_destruct_enum,
	mzn_show_enum,
	default,
	output,
	no_output,
	dzn,
	mzn_construct_opt,
	mzn_destruct_opt,
	mzn_opt_domain,
	mzn_opt_channel,
	mzn_domain_constraint,
	mzn_check_index_set,
	mzn_show_array_access,
	mzn_show_tuple_access,
	mzn_show_record_access,
	return_value: "<return value>",
	mzn_inline,
	mzn_inline_call_by_name,
	mzn_opt_bool,
	array_access: "[]",
	mzn_element_internal,
	mzn_slice_internal,
	mzn_construct_partial,
	mzn_destruct_partial,
	mzn_unreachable,
);
