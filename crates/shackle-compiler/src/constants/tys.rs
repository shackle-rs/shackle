use crate::{db::Interner, ty::Ty};

macro_rules! type_registry {
	($struct:ident, $db:ident, $($name:ident: $value:expr),+$(,)?) => {
		/// Registry for common types
		#[derive(Clone, Debug, PartialEq, Eq)]
		pub struct $struct {
			#[allow(missing_docs)]
			pub all: Vec<$crate::ty::Ty>,
			$(
				#[allow(missing_docs)]
				pub $name: Ty
			),+
		}

		impl $struct {
			/// Create a new type registry
			pub fn new(db: &dyn Interner) -> Self {
				let $db = db;
				let mut all = Vec::new();
				$(let $name = $value; all.push($name);)+
				Self {
					all,
					$($name),+
				}
			}
		}
	};
}

pub(crate) use type_registry;

type_registry!(
	TypeRegistry,
	db,
	error: Ty::error(db),
	par_bool: Ty::par_bool(db),
	var_bool: par_bool.make_var(db).unwrap(),
	par_opt_bool: par_bool.make_opt(db),
	var_opt_bool: var_bool.make_opt(db),
	par_int: Ty::par_int(db),
	var_int: par_int.make_var(db).unwrap(),
	par_float: Ty::par_float(db),
	var_float: par_float.make_var(db).unwrap(),
	string: Ty::string(db),
	ann: Ty::ann(db),
	bottom: Ty::bottom(db),
	opt_bottom: bottom.make_opt(db),
	set_of_bottom: Ty::par_set(db, bottom).unwrap(),
	set_of_int: Ty::par_set(db, par_int).unwrap(),
	array_of_string: Ty::array(db, par_int, string).unwrap(),
	array_of_bottom: Ty::array(db, bottom, bottom).unwrap(),
	array_of_int: Ty::array(db, par_int, par_int).unwrap(),
	array_of_tuple_int_set_of_int: Ty::array(db, par_int, Ty::tuple(db, [par_int, set_of_int])).unwrap(),
	mzn_enum: Ty::tuple(db, [par_int, Ty::array(db, par_int, Ty::tuple(db, [string, array_of_tuple_int_set_of_int, par_int])).unwrap()]),
	mzn_enum_definition: Ty::array(db, par_int, Ty::tuple(db, [string, Ty::array(db, par_int, Ty::tuple(db, [par_int, set_of_int])).unwrap()])).unwrap(),
);
