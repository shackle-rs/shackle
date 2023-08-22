use std::ops::Deref;

use rustc_hash::FxHashSet;

use super::*;
use crate::{constants::type_registry, db::CompilerDatabase};

type_registry!(
	TypeRegistry,
	db,
	par_bool: Ty::par_bool(db),
	var_bool: par_bool.make_var(db).unwrap(),
	par_opt_bool: par_bool.make_opt(db),
	var_opt_bool: var_bool.make_opt(db),
	par_int: Ty::par_int(db),
	var_int: par_int.make_var(db).unwrap(),
	par_opt_int: par_int.make_opt(db),
	var_opt_int: var_int.make_opt(db),
	par_float: Ty::par_float(db),
	var_float: par_float.make_var(db).unwrap(),
	par_opt_float: par_float.make_opt(db),
	var_opt_float: var_float.make_opt(db),
	string: Ty::string(db),
	opt_string: string.make_opt(db),
	ann: Ty::ann(db),
	opt_ann: ann.make_opt(db),
	bottom: Ty::bottom(db),
	opt_bottom: bottom.make_opt(db),
	array_int_of_par_int: Ty::array(db, par_int, par_int).unwrap(),
	array_int_of_var_int: Ty::array(db, par_int, var_int).unwrap(),
	set_of_int: Ty::par_set(db, par_int).unwrap(),
	set_of_float: Ty::par_set(db, par_float).unwrap(),
	tuple_of_int: Ty::tuple(db, vec![par_int, par_int]),
	tuple_of_float: Ty::tuple(db, vec![par_float, par_float]),
	record: Ty::record(db, vec![(InternedString::new("a", db), var_int)]),
	sub_record: Ty::record(db, vec![(InternedString::new("a", db), par_int), (InternedString::new("b", db), par_bool)]),
	par_enum: Ty::par_enum(db, EnumRef::introduce(db, InternedString::new("Foo", db))),
	var_enum: par_enum.make_var(db).unwrap(),
	par_opt_enum: par_enum.make_opt(db),
	var_opt_enum: var_enum.make_opt(db),
	function: Ty::function(db, FunctionType {
		params: Box::new([par_int, var_bool]),
		return_type: var_int
	}),
	sub_function: Ty::function(db, FunctionType {
		params: Box::new([par_float, var_int]),
		return_type: var_bool
	}),
	ty_var: Ty::type_inst_var(db, TyVar {
		ty_var: TyVarRef::introduce(db, InternedString::new("$T", db)),
		enumerable: false,
		indexable: false,
		varifiable: false
	}),
);

struct Types {
	db: CompilerDatabase,
	registry: TypeRegistry,
}

impl Types {
	fn new() -> Self {
		let db = CompilerDatabase::default();
		let registry = TypeRegistry::new(&db);
		Self { db, registry }
	}
}

impl Deref for Types {
	type Target = TypeRegistry;

	fn deref(&self) -> &Self::Target {
		&self.registry
	}
}

/// Given a set of coercions, asserts that they all hold, and that the given source types do not
/// coerce to any other types.
fn check_coercions(types: &Types, coercions: impl IntoIterator<Item = (Ty, Ty)>) {
	let db = &types.db;
	let mut cs: FxHashMap<Ty, FxHashSet<Ty>> = FxHashMap::default();
	for (src, dst) in coercions {
		cs.entry(src).or_default().insert(dst);
	}
	for (src, dsts) in cs {
		for dst in types.all.iter() {
			if dsts.contains(dst) {
				assert!(
					src.is_subtype_of(&types.db, *dst),
					"Expected coercion from {} to {}",
					src.pretty_print(db),
					dst.pretty_print(db)
				);
				let supertype = Ty::most_specific_supertype(db, [src, *dst]);
				assert_eq!(
					supertype,
					Some(*dst),
					"Expected most specific supertype of {} and {} to be {}, but got {}",
					src.pretty_print(db),
					dst.pretty_print(db),
					dst.pretty_print(db),
					supertype
						.map(|s| s.pretty_print(db))
						.unwrap_or("none".to_owned())
				);
				let subtype = Ty::most_general_subtype(db, [src, *dst]);
				assert_eq!(
					subtype,
					Some(src),
					"Expected most general subtype of {} and {} to be {}, but got {}",
					src.pretty_print(db),
					dst.pretty_print(db),
					src.pretty_print(db),
					subtype
						.map(|s| s.pretty_print(db))
						.unwrap_or("none".to_owned())
				);
			} else {
				assert!(
					!src.is_subtype_of(&types.db, *dst),
					"Unexpected coercion from {} to {}",
					src.pretty_print(db),
					dst.pretty_print(db)
				);
			}
		}
	}
}

#[test]
fn test_bool() {
	let types = Types::new();
	let db = &types.db;
	assert_eq!(
		types.par_bool.lookup(db),
		TyData::Boolean(VarType::Par, OptType::NonOpt)
	);
	assert!(types.par_bool.known_par(db));
	assert!(types.par_bool.known_varifiable(db));
	assert!(types.par_bool.known_enumerable(db));
	assert_eq!(types.par_bool.pretty_print(db), "bool");

	assert_eq!(
		types.var_bool.lookup(db),
		TyData::Boolean(VarType::Var, OptType::NonOpt)
	);
	assert!(!types.var_bool.known_par(db));
	assert!(types.var_bool.known_varifiable(db));
	assert!(types.var_bool.known_enumerable(db));
	assert_eq!(types.var_bool.pretty_print(db), "var bool");

	assert_eq!(
		types.par_opt_bool.lookup(db),
		TyData::Boolean(VarType::Par, OptType::Opt)
	);
	assert!(types.par_opt_bool.known_par(db));
	assert!(types.par_opt_bool.known_varifiable(db));
	assert!(types.par_opt_bool.known_enumerable(db));
	assert_eq!(types.par_opt_bool.pretty_print(db), "opt bool");

	assert_eq!(
		types.var_opt_bool.lookup(db),
		TyData::Boolean(VarType::Var, OptType::Opt)
	);
	assert!(!types.var_opt_bool.known_par(db));
	assert!(types.var_opt_bool.known_varifiable(db));
	assert!(types.var_opt_bool.known_enumerable(db));
	assert_eq!(types.var_opt_bool.pretty_print(db), "var opt bool");
}

#[test]
fn test_bool_coercion() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			// bool to bool
			(types.par_bool, types.par_bool),
			(types.par_bool, types.var_bool),
			(types.par_bool, types.par_opt_bool),
			(types.par_bool, types.var_opt_bool),
			(types.var_bool, types.var_bool),
			(types.var_bool, types.var_opt_bool),
			(types.par_opt_bool, types.par_opt_bool),
			(types.par_opt_bool, types.var_opt_bool),
			(types.var_opt_bool, types.var_opt_bool),
			// bool to int
			(types.par_bool, types.par_int),
			(types.par_bool, types.var_int),
			(types.par_bool, types.par_opt_int),
			(types.par_bool, types.var_opt_int),
			(types.var_bool, types.var_int),
			(types.var_bool, types.var_opt_int),
			(types.par_opt_bool, types.par_opt_int),
			(types.par_opt_bool, types.var_opt_int),
			(types.var_opt_bool, types.var_opt_int),
			// bool to float
			(types.par_bool, types.par_float),
			(types.par_bool, types.var_float),
			(types.par_bool, types.par_opt_float),
			(types.par_bool, types.var_opt_float),
			(types.var_bool, types.var_float),
			(types.var_bool, types.var_opt_float),
			(types.par_opt_bool, types.par_opt_float),
			(types.par_opt_bool, types.var_opt_float),
			(types.var_opt_bool, types.var_opt_float),
		],
	);
}

#[test]
fn test_int() {
	let types = Types::new();
	let db = &types.db;
	assert_eq!(
		types.par_int.lookup(db),
		TyData::Integer(VarType::Par, OptType::NonOpt)
	);
	assert!(types.par_int.known_par(db));
	assert!(types.par_int.known_varifiable(db));
	assert!(types.par_int.known_enumerable(db));
	assert_eq!(types.par_int.pretty_print(db), "int");

	assert_eq!(
		types.var_int.lookup(db),
		TyData::Integer(VarType::Var, OptType::NonOpt)
	);
	assert!(!types.var_int.known_par(db));
	assert!(types.var_int.known_varifiable(db));
	assert!(types.var_int.known_enumerable(db));
	assert_eq!(types.var_int.pretty_print(db), "var int");

	assert_eq!(
		types.par_opt_int.lookup(db),
		TyData::Integer(VarType::Par, OptType::Opt)
	);
	assert!(types.par_opt_int.known_par(db));
	assert!(types.par_opt_int.known_varifiable(db));
	assert!(types.par_opt_int.known_enumerable(db));
	assert_eq!(types.par_opt_int.pretty_print(db), "opt int");

	assert_eq!(
		types.var_opt_int.lookup(db),
		TyData::Integer(VarType::Var, OptType::Opt)
	);
	assert!(!types.var_opt_int.known_par(db));
	assert!(types.var_opt_int.known_varifiable(db));
	assert!(types.var_opt_int.known_enumerable(db));
	assert_eq!(types.var_opt_int.pretty_print(db), "var opt int");
}

#[test]
fn test_int_coercion() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			// int to int
			(types.par_int, types.par_int),
			(types.par_int, types.var_int),
			(types.par_int, types.par_opt_int),
			(types.par_int, types.var_opt_int),
			(types.var_int, types.var_int),
			(types.var_int, types.var_opt_int),
			(types.par_opt_int, types.par_opt_int),
			(types.par_opt_int, types.var_opt_int),
			(types.var_opt_int, types.var_opt_int),
			// int to float
			(types.par_int, types.par_float),
			(types.par_int, types.var_float),
			(types.par_int, types.par_opt_float),
			(types.par_int, types.var_opt_float),
			(types.var_int, types.var_float),
			(types.var_int, types.var_opt_float),
			(types.par_opt_int, types.par_opt_float),
			(types.par_opt_int, types.var_opt_float),
			(types.var_opt_int, types.var_opt_float),
		],
	);
}

#[test]
fn test_float() {
	let types = Types::new();
	let db = &types.db;
	assert_eq!(
		types.par_float.lookup(db),
		TyData::Float(VarType::Par, OptType::NonOpt)
	);
	assert!(types.par_float.known_par(db));
	assert!(types.par_float.known_varifiable(db));
	assert!(!types.par_float.known_enumerable(db));
	assert_eq!(types.par_float.pretty_print(db), "float");

	assert_eq!(
		types.var_float.lookup(db),
		TyData::Float(VarType::Var, OptType::NonOpt)
	);
	assert!(!types.var_float.known_par(db));
	assert!(types.var_float.known_varifiable(db));
	assert!(!types.var_float.known_enumerable(db));
	assert_eq!(types.var_float.pretty_print(db), "var float");

	assert_eq!(
		types.par_opt_float.lookup(db),
		TyData::Float(VarType::Par, OptType::Opt)
	);
	assert!(types.par_opt_float.known_par(db));
	assert!(types.par_opt_float.known_varifiable(db));
	assert!(!types.par_opt_float.known_enumerable(db));
	assert_eq!(types.par_opt_float.pretty_print(db), "opt float");

	assert_eq!(
		types.var_opt_float.lookup(db),
		TyData::Float(VarType::Var, OptType::Opt)
	);
	assert!(!types.var_opt_float.known_par(db));
	assert!(types.var_opt_float.known_varifiable(db));
	assert!(!types.var_opt_float.known_enumerable(db));
	assert_eq!(types.var_opt_float.pretty_print(db), "var opt float");
}

#[test]
fn test_float_coercion() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			// float to float
			(types.par_float, types.par_float),
			(types.par_float, types.var_float),
			(types.par_float, types.par_opt_float),
			(types.par_float, types.var_opt_float),
			(types.var_float, types.var_float),
			(types.var_float, types.var_opt_float),
			(types.par_opt_float, types.par_opt_float),
			(types.par_opt_float, types.var_opt_float),
			(types.var_opt_float, types.var_opt_float),
		],
	);
}

#[test]
fn test_string() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(types.string.lookup(db), TyData::String(OptType::NonOpt));
	assert!(types.string.known_par(db));
	assert!(!types.string.known_varifiable(db));
	assert!(!types.string.known_enumerable(db));
	assert_eq!(types.string.pretty_print(db), "string");

	assert_eq!(types.opt_string.lookup(db), TyData::String(OptType::Opt));
	assert!(types.opt_string.known_par(db));
	assert!(!types.opt_string.known_varifiable(db));
	assert!(!types.opt_string.known_enumerable(db));
	assert_eq!(types.opt_string.pretty_print(db), "opt string");
}

#[test]
fn test_string_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.string, types.string),
			(types.string, types.opt_string),
			(types.opt_string, types.opt_string),
		],
	);
}

#[test]
fn test_ann() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(types.ann.lookup(db), TyData::Annotation(OptType::NonOpt));
	assert!(types.ann.known_par(db));
	assert!(!types.ann.known_varifiable(db));
	assert!(!types.ann.known_enumerable(db));
	assert_eq!(types.ann.pretty_print(db), "ann");

	assert_eq!(types.opt_ann.lookup(db), TyData::Annotation(OptType::Opt));
	assert!(types.opt_ann.known_par(db));
	assert!(!types.opt_ann.known_varifiable(db));
	assert!(!types.opt_ann.known_enumerable(db));
	assert_eq!(types.opt_ann.pretty_print(db), "opt ann");
}

#[test]
fn test_ann_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.ann, types.ann),
			(types.ann, types.opt_ann),
			(types.opt_ann, types.opt_ann),
		],
	);
}

#[test]
fn test_bottom() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(types.bottom.lookup(db), TyData::Bottom(OptType::NonOpt));
	assert!(types.bottom.known_par(db));
	assert!(types.bottom.known_varifiable(db));
	assert!(types.bottom.known_enumerable(db));
	assert_eq!(types.bottom.pretty_print(db), "..");

	assert_eq!(types.opt_bottom.lookup(db), TyData::Bottom(OptType::Opt));
	assert!(types.opt_bottom.known_par(db));
	assert!(types.opt_bottom.known_varifiable(db));
	assert!(types.opt_bottom.known_enumerable(db));
	assert_eq!(types.opt_bottom.pretty_print(db), "opt ..");
}

#[test]
fn test_bottom_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			// Bottom to bottom
			(types.bottom, types.bottom),
			(types.bottom, types.opt_bottom),
			(types.opt_bottom, types.opt_bottom),
			// Bottom to bool
			(types.bottom, types.par_bool),
			(types.bottom, types.var_bool),
			(types.bottom, types.par_opt_bool),
			(types.bottom, types.var_opt_bool),
			(types.opt_bottom, types.par_opt_bool),
			(types.opt_bottom, types.var_opt_bool),
			// Bottom to int
			(types.bottom, types.par_int),
			(types.bottom, types.var_int),
			(types.bottom, types.par_opt_int),
			(types.bottom, types.var_opt_int),
			(types.opt_bottom, types.par_opt_int),
			(types.opt_bottom, types.var_opt_int),
			// Bottom to float
			(types.bottom, types.par_float),
			(types.bottom, types.var_float),
			(types.bottom, types.par_opt_float),
			(types.bottom, types.var_opt_float),
			(types.opt_bottom, types.par_opt_float),
			(types.opt_bottom, types.var_opt_float),
			// // Bottom to enum
			(types.bottom, types.par_enum),
			(types.bottom, types.var_enum),
			(types.bottom, types.par_opt_enum),
			(types.bottom, types.var_opt_enum),
			(types.opt_bottom, types.par_opt_enum),
			(types.opt_bottom, types.var_opt_enum),
			// Bottom to string
			(types.bottom, types.string),
			(types.bottom, types.opt_string),
			(types.opt_bottom, types.opt_string),
			// Bottom to ann
			(types.bottom, types.ann),
			(types.bottom, types.opt_ann),
			(types.opt_bottom, types.opt_ann),
			// Bottom to array
			(types.bottom, types.array_int_of_par_int),
			(types.bottom, types.array_int_of_var_int),
			// Bottom to set
			(types.bottom, types.set_of_int),
			(types.bottom, types.set_of_float),
			// Bottom to tuple
			(types.bottom, types.tuple_of_int),
			(types.bottom, types.tuple_of_float),
			// Bottom to record
			(types.bottom, types.record),
			(types.bottom, types.sub_record),
			// Bottom to function
			(types.bottom, types.function),
			(types.bottom, types.sub_function),
			// Bottom to tyvar
			(types.bottom, types.ty_var),
		],
	);
}

#[test]
fn test_array() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(
		types.array_int_of_par_int.lookup(db),
		TyData::Array {
			opt: OptType::NonOpt,
			dim: types.par_int,
			element: types.par_int
		}
	);
	assert!(types.array_int_of_par_int.known_par(db));
	assert!(!types.array_int_of_par_int.known_varifiable(db));
	assert!(!types.array_int_of_par_int.known_enumerable(db));

	assert_eq!(
		types.array_int_of_var_int.lookup(db),
		TyData::Array {
			opt: OptType::NonOpt,
			dim: types.par_int,
			element: types.var_int
		}
	);
	assert!(!types.array_int_of_var_int.known_par(db));
	assert!(!types.array_int_of_var_int.known_varifiable(db));
	assert!(!types.array_int_of_var_int.known_enumerable(db));
}

#[test]
fn test_array_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.array_int_of_par_int, types.array_int_of_par_int),
			(types.array_int_of_var_int, types.array_int_of_var_int),
			(types.array_int_of_par_int, types.array_int_of_var_int),
		],
	);
}

#[test]
fn test_set() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(
		types.set_of_int.lookup(db),
		TyData::Set(VarType::Par, OptType::NonOpt, types.par_int)
	);
	assert!(types.set_of_int.known_par(db));
	assert!(types.set_of_int.known_varifiable(db));
	assert!(!types.set_of_int.known_enumerable(db));

	assert_eq!(
		types.set_of_float.lookup(db),
		TyData::Set(VarType::Par, OptType::NonOpt, types.par_float)
	);
	assert!(types.set_of_float.known_par(db));
	assert!(!types.set_of_float.known_varifiable(db));
	assert!(!types.set_of_float.known_enumerable(db));
}

#[test]
fn test_set_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.set_of_int, types.set_of_int),
			(types.set_of_float, types.set_of_float),
			(types.set_of_int, types.set_of_float),
		],
	);
}

#[test]
fn test_tuple() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(
		types.tuple_of_int.lookup(db),
		TyData::Tuple(OptType::NonOpt, Box::new([types.par_int, types.par_int]))
	);
	assert!(types.tuple_of_int.known_par(db));
	assert!(types.tuple_of_int.known_varifiable(db));
	assert!(!types.tuple_of_int.known_enumerable(db));

	assert_eq!(
		types.tuple_of_float.lookup(db),
		TyData::Tuple(
			OptType::NonOpt,
			Box::new([types.par_float, types.par_float])
		)
	);
	assert!(types.tuple_of_float.known_par(db));
	assert!(types.tuple_of_float.known_varifiable(db));
	assert!(!types.tuple_of_float.known_enumerable(db));
}

#[test]
fn test_tuple_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.tuple_of_int, types.tuple_of_int),
			(types.tuple_of_float, types.tuple_of_float),
			(types.tuple_of_int, types.tuple_of_float),
		],
	);
}

#[test]
fn test_record() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(
		types.record.lookup(db),
		TyData::Record(
			OptType::NonOpt,
			Box::new([(InternedString::new("a", db), types.var_int),])
		)
	);
	assert!(!types.record.known_par(db));
	assert!(types.record.known_varifiable(db));
	assert!(!types.record.known_enumerable(db));

	assert_eq!(
		types.sub_record.lookup(db),
		TyData::Record(
			OptType::NonOpt,
			Box::new([
				(InternedString::new("a", db), types.par_int),
				(InternedString::new("b", db), types.par_bool)
			])
		)
	);
	assert!(types.sub_record.known_par(db));
	assert!(types.sub_record.known_varifiable(db));
	assert!(!types.sub_record.known_enumerable(db));
}

#[test]
fn test_record_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.record, types.record),
			(types.sub_record, types.sub_record),
		],
	);
}

#[test]
fn test_enum() {
	let types = Types::new();
	let db = &types.db;
	assert!(matches!(
		types.par_enum.lookup(db),
		TyData::Enum(VarType::Par, OptType::NonOpt, _)
	));
	assert!(types.par_enum.known_par(db));
	assert!(types.par_enum.known_varifiable(db));
	assert!(types.par_enum.known_enumerable(db));
	assert_eq!(types.par_enum.pretty_print(db), "Foo");

	assert!(matches!(
		types.var_enum.lookup(db),
		TyData::Enum(VarType::Var, OptType::NonOpt, _)
	));
	assert!(!types.var_enum.known_par(db));
	assert!(types.var_enum.known_varifiable(db));
	assert!(types.var_enum.known_enumerable(db));
	assert_eq!(types.var_enum.pretty_print(db), "var Foo");

	assert!(matches!(
		types.par_opt_enum.lookup(db),
		TyData::Enum(VarType::Par, OptType::Opt, _)
	));
	assert!(types.par_opt_enum.known_par(db));
	assert!(types.par_opt_enum.known_varifiable(db));
	assert!(types.par_opt_enum.known_enumerable(db));
	assert_eq!(types.par_opt_enum.pretty_print(db), "opt Foo");

	assert!(matches!(
		types.var_opt_enum.lookup(db),
		TyData::Enum(VarType::Var, OptType::Opt, _)
	));
	assert!(!types.var_opt_enum.known_par(db));
	assert!(types.var_opt_enum.known_varifiable(db));
	assert!(types.var_opt_enum.known_enumerable(db));
	assert_eq!(types.var_opt_enum.pretty_print(db), "var opt Foo");
}

#[test]
fn test_enum_coercion() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.par_enum, types.par_enum),
			(types.par_enum, types.var_enum),
			(types.par_enum, types.par_opt_enum),
			(types.par_enum, types.var_opt_enum),
			(types.var_enum, types.var_enum),
			(types.var_enum, types.var_opt_enum),
			(types.par_opt_enum, types.par_opt_enum),
			(types.par_opt_enum, types.var_opt_enum),
			(types.var_opt_enum, types.var_opt_enum),
		],
	);
}

#[test]
fn test_function() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(
		types.function.lookup(db),
		TyData::Function(
			OptType::NonOpt,
			FunctionType {
				params: Box::new([types.par_int, types.var_bool]),
				return_type: types.var_int
			}
		)
	);
	assert!(types.function.known_par(db));
	assert!(!types.function.known_varifiable(db));
	assert!(!types.function.known_enumerable(db));

	assert_eq!(
		types.sub_function.lookup(db),
		TyData::Function(
			OptType::NonOpt,
			FunctionType {
				params: Box::new([types.par_float, types.var_int]),
				return_type: types.var_bool
			}
		)
	);
	assert!(types.sub_function.known_par(db));
	assert!(!types.sub_function.known_varifiable(db));
	assert!(!types.sub_function.known_enumerable(db));
}

#[test]
fn test_function_coercions() {
	let types = Types::new();
	check_coercions(
		&types,
		[
			(types.function, types.function),
			(types.sub_function, types.sub_function),
			(types.sub_function, types.function),
		],
	);
}

#[test]
fn test_ty_var() {
	let types = Types::new();
	let db = &types.db;

	assert!(matches!(
		types.ty_var.lookup(db),
		TyData::TyVar(None, None, _)
	));
	assert!(!types.ty_var.known_par(db));
	assert!(!types.ty_var.known_varifiable(db));
	assert!(!types.ty_var.known_enumerable(db));
}

#[test]
fn test_most_specific_supertype() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(Ty::most_specific_supertype(db, []), None);
	assert_eq!(
		Ty::most_specific_supertype(db, [types.var_int]),
		Some(types.var_int)
	);
	assert_eq!(
		Ty::most_specific_supertype(db, [types.var_int, types.par_float]),
		Some(types.var_float)
	);
	assert_eq!(
		Ty::most_specific_supertype(db, [types.var_opt_bool, types.var_int, types.par_opt_float]),
		Some(types.var_opt_float)
	);
}

#[test]
fn test_most_general_subtype() {
	let types = Types::new();
	let db = &types.db;

	assert_eq!(Ty::most_general_subtype(db, []), None);
	assert_eq!(
		Ty::most_general_subtype(db, [types.var_int]),
		Some(types.var_int)
	);
	assert_eq!(
		Ty::most_general_subtype(db, [types.var_int, types.par_float]),
		Some(types.par_int)
	);
	assert_eq!(
		Ty::most_general_subtype(db, [types.var_opt_bool, types.var_int, types.par_opt_float]),
		Some(types.par_bool)
	);
}
