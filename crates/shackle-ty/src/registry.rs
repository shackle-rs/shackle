//! Registry of useful types
//!

use crate::Ty;

macro_rules! type_registry {
	($(#[$meta:meta])* $struct:ident, $db:ident, $($name:ident: $value:expr),+$(,)?) => {
        $(#[$meta])*
		#[derive(Clone, Debug, PartialEq, Eq, salsa::SalsaValue)]
		pub struct $struct<'db> {
			/// All types in the registry
			pub all: Vec<$crate::Ty<'db>>,
			$(
				#[allow(missing_docs, reason = "Fields are self-explanatory")]
				pub $name: $crate::Ty<'db>
			),+
		}

		impl<'db> $struct<'db> {
			/// Create a new type registry
			pub(crate) fn new(db: &'db dyn salsa::Database) -> Self {
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

#[cfg(test)]
pub(crate) use type_registry;

type_registry!(
	/// Common types
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
	array_of_opt_bottom: Ty::array(db, par_int, opt_bottom).unwrap(),
	array_of_int: Ty::array(db, par_int, par_int).unwrap(),
	array_of_tuple_int_set_of_int: Ty::array(db, par_int, Ty::tuple(db, [par_int, set_of_int])).unwrap(),
	mzn_enum: Ty::tuple(db, [par_int, Ty::array(db, par_int, Ty::tuple(db, [string, array_of_tuple_int_set_of_int, par_int])).unwrap()]),
	mzn_enum_definition: Ty::array(db, par_int, Ty::tuple(db, [string, Ty::array(db, par_int, Ty::tuple(db, [par_int, set_of_int])).unwrap()])).unwrap(),
	mzn_enum_param: Ty::tuple(db, [par_int, Ty::array(db, par_int, Ty::tuple(db, [par_int, par_int])).unwrap()])
);

impl<'db> TypeRegistry<'db> {
	/// Get the type registry
	pub fn lookup(db: &'db dyn salsa::Database) -> &'db Self {
		create_type_registry(db)
	}
}

#[salsa::tracked]
fn create_type_registry<'db>(db: &'db dyn salsa::Database) -> TypeRegistry<'db> {
	TypeRegistry::new(db)
}
