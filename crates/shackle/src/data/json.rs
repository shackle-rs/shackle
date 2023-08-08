use crate::{
	db::CompilerDatabase,
	diagnostics::InternalError,
	ty::{Ty, TyData},
	value::{Array, Index, Record, Set, Value},
};

fn flatten_array(
	db: &CompilerDatabase,
	content: &mut Vec<Value>,
	arr: serde_json::Value,
	ndim: usize,
	elem_ty: Ty,
) -> Result<(), InternalError> {
	if let serde_json::Value::Array(vec) = arr {
		if ndim > 1 {
			for sub_arr in vec {
				flatten_array(db, content, sub_arr, ndim - 1, elem_ty)?;
			}
		} else {
			for elem in vec {
				content.push(deserialize_legacy_value(db, elem_ty, elem)?);
			}
		}
		Ok(())
	} else {
		Err(InternalError::new(
			"value from legacy interpreter does not have the expected number of dimensions",
		))
	}
}

pub fn deserialize_legacy_value(
	db: &CompilerDatabase,
	ty: Ty,
	val: serde_json::Value,
) -> Result<Value, InternalError> {
	match val {
		serde_json::Value::Null => {
			if ty.known_occurs(db) {
				Err(InternalError::new(
					format!("legacy interpreter returned an absent value for variable with a type `{}',  known to occur", ty.pretty_print(db)),
				))
			} else {
				Ok(Value::Absent)
			}
		}
		serde_json::Value::Bool(b) => {
			if let TyData::Boolean(_, _) = ty.lookup(db) {
				Ok(Value::Boolean(b))
			} else {
				Err(InternalError::new(format!(
					"legacy interpreter returned a Boolean value for variable of type `{}'",
					ty.pretty_print(db)
				)))
			}
		}
		serde_json::Value::Number(v) => {
			if v.is_f64() {
				if let TyData::Float(_, _) = ty.lookup(db) {
					Ok(Value::Float(v.as_f64().unwrap()))
				} else {
					Err(InternalError::new(format!(
						"legacy interpreter returned a floating point value for variable of type `{}'",
						ty.pretty_print(db)
					)))
				}
			} else {
				assert!(v.is_i64());
				match ty.lookup(db) {
					TyData::Integer(_, _) => Ok(Value::Integer(v.as_i64().unwrap())),
					TyData::Float(_, _) => Ok(Value::Float(v.as_i64().unwrap() as f64)),
					_ => Err(InternalError::new(format!(
						"legacy interpreter returned a integer value for variable of type `{}'",
						ty.pretty_print(db)
					))),
				}
			}
		}
		serde_json::Value::String(v) => {
			if let TyData::String(_) = ty.lookup(db) {
				Ok(Value::String(v.into()))
			} else {
				Err(InternalError::new(format!(
					"legacy interpreter returned a String value for variable of type `{}'",
					ty.pretty_print(db)
				)))
			}
		}
		serde_json::Value::Array(v) => match ty.lookup(db) {
			TyData::Array {
				opt: _,
				dim,
				element,
			} => {
				if let TyData::Tuple(_, tt) = dim.lookup(db) {
					// Determine the index sets of the array
					// FIXME should be returned from the model
					let mut ranges = Vec::with_capacity(tt.len());
					let arr = serde_json::Value::Array(v);
					let mut ii = &arr;
					while let serde_json::Value::Array(v) = ii {
						ranges.push(Index::Integer(1..=v.len() as i64));
						if let Some(fst) = v.last() {
							ii = fst;
						} else {
							#[allow(clippy::reversed_empty_ranges)]
							ranges.push(Index::Integer(1..=0i64));
						}
					}
					// Flatten content
					let mut content = Vec::new();
					flatten_array(db, &mut content, arr, tt.len(), element)?;

					Ok(Value::Array(Array::new(ranges, content)))
				} else {
					let range = Index::Integer(1..=v.len() as i64);
					let content = v
						.into_iter()
						.map(|val| deserialize_legacy_value(db, element, val))
						.collect::<Result<Vec<_>, _>>()?;
					Ok(Value::Array(Array::new(vec![range], content)))
				}
			}
			TyData::Tuple(_, types) => {
				assert_eq!(types.len(), v.len());
				v.into_iter()
					.zip(types.iter())
					.map(|(val, ty)| deserialize_legacy_value(db, *ty, val))
					.collect::<Result<Vec<_>, _>>()
					.map(Value::Tuple)
			}
			_ => Err(InternalError::new(format!(
				"legacy interpreter returned a Array value for variable of type `{}'",
				ty.pretty_print(db)
			))),
		},
		serde_json::Value::Object(mut obj) => match ty.lookup(db) {
			TyData::Enum(_, _, _) => Ok(Value::Enum(if obj.contains_key("c") {
				let _e = if let Some(_s) = obj["e"].as_str() {
					todo!()
				// Value::Enum(EnumValue::new_ident_member(s.to_string()))
				} else if let Some(x) = obj["e"].as_i64() {
					Value::Integer(x)
				} else if let Value::Enum(s) = deserialize_legacy_value(db, ty, obj["e"].clone())? {
					Value::Enum(s)
				} else {
					return Err(InternalError::new(format!(
						"lagacy interpreter returned an invalid enum value `{:?}'",
						obj
					)));
				};
				todo!()
			// EnumValue::new_constructor_member(obj["c"].as_str().unwrap().to_string(), e)
			} else if obj.contains_key("i") {
				assert_eq!(obj.len(), 2);
				todo!()
			// EnumValue::new_anon_member(
			// 	obj["e"].as_str().unwrap().to_string(),
			// 	obj["i"].as_u64().unwrap() as usize,
			// )
			} else {
				assert_eq!(obj.len(), 1);
				todo!()
				// EnumValue::new_ident_member(obj["e"].as_str().unwrap().to_string())
			})),
			TyData::Set(_, _, elem) => {
				let set = obj.remove("set").unwrap();
				if let serde_json::Value::Array(set) = set {
					match elem.lookup(db) {
						TyData::Integer(_, _) if matches!(set[0], serde_json::Value::Array(_)) => {
							let mut content = Vec::with_capacity(set.len());
							for mem in set {
								if let serde_json::Value::Array(x) = mem {
									assert_eq!(x.len(), 2);
									content.push(x[0].as_i64().unwrap()..=x[1].as_i64().unwrap());
								} else {
									return Err(InternalError::new(format!(
										"legacy interpreter invalid range in set members `{}'",
										mem
									)));
								}
							}
							Ok(Value::Set(Set::IntRangeList(content)))
						}
						_ => {
							let content = set
								.into_iter()
								.map(|v| deserialize_legacy_value(db, elem, v))
								.collect::<Result<Vec<_>, _>>()?;
							Ok(Value::Set(Set::SetList(content)))
						}
					}
				} else {
					Err(InternalError::new(format!(
						"legacy interpreter returned invalid set members `{}'",
						set
					)))
				}
			}
			TyData::Record(_, types) => {
				assert_eq!(types.len(), obj.len());
				let rec = types
					.iter()
					.map(|(name, tt)| {
						let name = name.value(db);
						match deserialize_legacy_value(db, *tt, obj.remove(&name).unwrap()) {
							Ok(val) => Ok((name.into(), val)),
							Err(err) => Err(err),
						}
					})
					.collect::<Result<Record, _>>()?;
				Ok(Value::Record(rec))
			}
			_ => Err(InternalError::new(format!(
				"legacy interpreter returned a Object value for variable of type `{}'",
				ty.pretty_print(db)
			))),
		},
	}
}
