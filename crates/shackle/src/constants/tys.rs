use crate::db::Interner;
use crate::hir::{OptType, VarType};
use crate::ty::Ty;

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
	var_bool: par_bool.with_inst(db, VarType::Var).unwrap(),
	par_int: Ty::par_int(db),
	var_int: par_int.with_inst(db, VarType::Var).unwrap(),
	par_float: Ty::par_float(db),
	var_float: par_float.with_inst(db, VarType::Var).unwrap(),
	string: Ty::string(db),
	ann: Ty::ann(db),
	bottom: Ty::bottom(db),
	opt_bottom: bottom.with_opt(db, OptType::Opt),
	set_of_bottom: Ty::par_set(db, bottom).unwrap(),
	array_of_string: Ty::array(db, par_int, string).unwrap(),
	array_of_bottom: Ty::array(db, bottom, bottom).unwrap(),
);
