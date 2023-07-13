use crate::hir::db::Hir;
use crate::hir::Identifier;

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
	annotated_expression,
	output_only,
	array_nd: "arrayNd",
	array_xd: "arrayXd",
	array2d,
	concat,
	join,
	plus_plus: "++",
	dot_dot: "..",
	objective: "_objective",
	show,
	show_dzn: "showDzn",
	show_json: "showJSON",
	is_fixed,
	fix,
	eq: "=",
	index_set,
	shackle_type,
	empty_annotation,
	minus: "-",
	plus: "+",
	times: "*",
	sum,
	product,
	erase_enum,
	forall,
	exists,
	set2array,
	annotate,
	to_enum,
	occurs,
	deopt,
	ub,
	in_: "in",
	conj: "/\\",
	disj: "\\/",
	imp: "->",
	if_then_else,
	card,
	mzn_enum_constructor,
	mzn_enum_destructor,
);
