//! # The DataZinc parser
//!
//! This module contains a parser for the DataZinc format (i.e., `.dzn`) files.
//! These files are often used to provide data for MiniZinc models.

use std::sync::Arc;

use itertools::Itertools;
use tree_sitter::Parser;

use crate::{
	data::ParserVal,
	diagnostics::{
		InvalidArrayLiteral, InvalidNumericLiteral, ShackleError, SyntaxError, TypeMismatch,
	},
	file::SourceFile,
	syntax::{
		ast::{
			Assignment, AstNode, Children, Expression, Identifier, InfixOperator,
			RecordLiteralMember,
		},
		cst::{Cst, CstNode},
	},
	value::Polarity,
	Enum, OptType, Type,
};

/// Parses a DataZinc file, returning a mapping of the name of the left hand
/// side of the assignment items to the values on the right hand side.
///
/// An optional filename can be given that will be used to indicate the location
/// if an error occurs
pub(crate) fn parse_dzn(src: &SourceFile) -> Result<Vec<Assignment>, ShackleError> {
	let mut parser = Parser::new();
	parser
		.set_language(tree_sitter_datazinc::language())
		.expect("Failed to set Tree Sitter parser language");
	let tree = parser
		.parse(src.contents().as_bytes(), None)
		.expect("DataZinc Tree Sitter parser did not return tree object");

	let cst = Cst::from_str(tree, src.contents());
	cst.error(|_| src.clone())?; // Check for any syntax errors

	let root = cst.node(cst.root_node());
	let it = Children::from_cst(&root, "item");

	Ok(it.collect())
}

/// Check that the assignment parsed matches the determined type in the model
pub(crate) fn typecheck_dzn(
	file: &SourceFile,
	name: &str,
	val: &Expression,
	ty: &Type,
) -> Result<ParserVal, ShackleError> {
	let type_err = |val_kind| {
		Err(TypeMismatch {
			src: file.clone(),
			msg: format!("The parameter variable '{}' is declared in the model as type '{}', but is assigned {}", name, ty, val_kind),
			span: val.cst_node().as_ref().byte_range().into(),
		}.into())
	};

	match val {
		Expression::IntegerLiteral(v) => {
			let v = v.value().map_err(|e| InvalidNumericLiteral {
				src: file.clone(),
				span: v.cst_node().as_ref().byte_range().into(),
				msg: e.to_string(),
			})?;
			match ty {
				Type::Integer(_) => Ok(ParserVal::Integer(v)),
				Type::Float(_) => Ok(ParserVal::Float(v as f64)),
				_ => type_err("an integer literal"),
			}
		}
		Expression::FloatLiteral(v) => {
			if matches!(ty, Type::Float(_)) {
				Ok(ParserVal::Float(v.value().map_err(|e| {
					InvalidNumericLiteral {
						src: file.clone(),
						span: v.cst_node().as_ref().byte_range().into(),
						msg: e.to_string(),
					}
				})?))
			} else {
				type_err("a floating point literal")
			}
		}
		Expression::TupleLiteral(tup) => {
			if let Type::Tuple(_, members) = ty {
				let len = tup.members().count();
				if len != members.len() {
					type_err(&format!("tuple literal of length {len}"))
				} else {
					let v: Vec<ParserVal> = tup
						.members()
						.zip(members.iter())
						.map(|(expr, ty)| typecheck_dzn(file, name, &expr, ty))
						.collect::<Result<Vec<_>, _>>()?;
					Ok(ParserVal::Tuple(v))
				}
			} else {
				type_err("a tuple literal")
			}
		}
		Expression::RecordLiteral(r) => match ty {
			Type::Record(_, elem_tys) => {
				// Assume that the list of types is already sorted based on the identifiers
				debug_assert!(elem_tys.iter().tuple_windows().all(|(a, b)| a.0 <= b.0));
				// Sort the AST record literal based on the identifiers
				let exprs: Vec<RecordLiteralMember> = r
					.members()
					.sorted_by_key(|x| x.name().name().to_string())
					.collect();
				// Now walk the types and expressions together, if there are less expressions or if
				// the names do not match, then one of the keys is missing from the data
				let mut vals = Vec::with_capacity(elem_tys.len());
				for i in 0..elem_tys.len() {
					if exprs.len() <= i || exprs[i].name().name().as_ref() != elem_tys[i].0.as_ref()
					{
						return Err(TypeMismatch {
							src: file.clone(),
							msg: format!("The parameter variable '{}' is declared in the model as type '{}', but the assigned record literal does not contain the key '{}'", name, ty, elem_tys[i].0),
							span: val.cst_node().as_ref().byte_range().into(),
						}.into());
					}
					vals.push((
						elem_tys[i].0.clone(),
						typecheck_dzn(file, name, &exprs[i].value(), &elem_tys[i].1)?,
					))
				}
				// Check whether there are any additional remaining keys
				if exprs.len() > elem_tys.len() {
					let additional = &exprs[elem_tys.len()..exprs.len()];
					return Err(TypeMismatch {
						src: file.clone(),
						msg: format!("The parameter variable '{}' is declared in the model as type '{}', but the assigned record literal constains the addition key{} {}", name, ty, if additional.len() > 1 { "s" } else {""}, additional.iter().format_with(", ", |key, f| {
							f(&format_args!("'{}'", key.name().name()))
						})),
						span: val.cst_node().as_ref().byte_range().into(),
					}.into());
				}
				Ok(ParserVal::Record(vals))
			}
			_ => type_err("a record literal"),
		},
		Expression::SetLiteral(sl) => match ty {
			Type::Set(_, elem_ty) => {
				let c = sl
					.members()
					.map(|elem| typecheck_dzn(file, name, &elem, elem_ty))
					.collect::<Result<_, _>>()?;
				Ok(ParserVal::SetList(c))
			}
			_ => type_err("a set literal"),
		},
		Expression::BooleanLiteral(b) => match ty {
			Type::Boolean(_) => Ok(ParserVal::Boolean(b.value())),
			Type::Integer(_) => Ok(ParserVal::Integer(b.value() as i64)),
			Type::Float(_) => Ok(ParserVal::Float(b.value() as i64 as f64)),
			_ => type_err("a Boolean literal"),
		},
		Expression::StringLiteral(s) => {
			if matches!(ty, Type::String(_)) {
				Ok(ParserVal::String(s.value()))
			} else {
				type_err("a string literal")
			}
		}
		Expression::Identifier(ident) => match ty {
			Type::Enum(_, _) => Ok(ParserVal::Enum(ident.name().to_string(), Vec::new())),
			Type::Annotation(_) => Ok(ParserVal::Ann(ident.name().to_string(), Vec::new())),
			_ => type_err("an identifier"),
		},
		Expression::Absent(_) => {
			if ty.is_opt() {
				Ok(ParserVal::Absent)
			} else {
				type_err("absent")
			}
		}
		Expression::Infinity(v) => {
			if matches!(ty, Type::Integer(_) | Type::Float(_)) {
				debug_assert_eq!(v.cst_text().trim_start(), v.cst_text());
				Ok(ParserVal::Infinity(if v.cst_text().starts_with("-") {
					Polarity::Neg
				} else {
					Polarity::Pos
				}))
			} else {
				type_err("infinity")
			}
		}
		Expression::ArrayLiteral(al) => match ty {
			Type::Array {
				opt: _,
				dim,
				element,
			} => {
				let members: Vec<_> = al.members().collect();
				if members.is_empty() {
					// Empty array literal
					Ok(ParserVal::SimpleArray(Vec::new(), Vec::new()))
				} else if members[0].indices().is_none()
					|| members.len() <= 1
					|| members[1].indices().is_none()
				{
					// Array literal without any indices or a single index
					if dim.len() != 1 {
						return Err(TypeMismatch {
							src: file.clone(),
							msg: format!("Indexed array literal with {} dimensions must be fully indexes using tuples", dim.len()),
							span: al.cst_node().as_ref().byte_range().into(),
						}
						.into());
					}
					let mut elems = Vec::with_capacity(members.len());
					let mut iter = members.iter();
					let first = iter.next().unwrap();
					let start = if let Some(idx) = first.indices() {
						typecheck_dzn(file, name, &idx, &dim[0])?
					} else {
						ParserVal::Integer(1)
					};
					elems.push(typecheck_dzn(file, name, &first.value(), element)?);
					for m in iter {
						if m.indices().is_some() {
							return Err(InvalidArrayLiteral {
								src: file.clone(),
								msg: "Indexed array literal must be fully indexed, or only have an index for the first element".to_string(),
								span: al.cst_node().as_ref().byte_range().into(),
							}
							.into());
						}
						elems.push(typecheck_dzn(file, name, &m.value(), element)?);
					}
					let end = if let ParserVal::Integer(v) = &start {
						ParserVal::Integer(v - 1 + elems.len() as i64)
					} else {
						ParserVal::Infinity(Polarity::Pos)
					};
					Ok(ParserVal::SimpleArray(vec![(start, end)], elems))
				} else {
					// Array literal with indices for all element
					let mut elems = Vec::with_capacity(members.len() * (dim.len() + 1));
					for m in members {
						match m.indices() {
							Some(indices) => {
								match indices {
									Expression::TupleLiteral(v) => {
										let mut i = 0;
										for (idx, idx_ty) in v.members().zip_eq(dim.iter()) {
											elems.push(typecheck_dzn(file, name, &idx, &idx_ty)?);
											i += 1;
										}
										if i != dim.len() {
											return Err(TypeMismatch {
												src: file.clone(),
												msg: format!("Indexed array literal with {} dimensions cannot be indexed with tuples of size {i}", dim.len()),
												span: m.cst_node().as_ref().byte_range().into(),
											}
											.into());
										}
									},
									v @ (Expression::IntegerLiteral(_) | Expression::Identifier(_) | Expression::Call(_)) => {
										if dim.len() != 1 {
											return Err(InvalidArrayLiteral {
												src: file.clone(),
												msg: "Indexed array literal with multiple dimensions must be fully indexes using tuples".into(),
												span: al.cst_node().as_ref().byte_range().into(),
											}
											.into());
										}
										elems.push(typecheck_dzn(file, name, &v, &dim[0])?);
									},
									_ => unreachable!(),
								}
							},
							None => return Err(InvalidArrayLiteral {
								src: file.clone(),
								msg: "Indexed array literal must be fully indexed, or only have an index for the first element".to_string(),
								span: al.cst_node().as_ref().byte_range().into(),
							}
							.into()),
						}
						elems.push(typecheck_dzn(file, name, &m.value(), element)?);
					}
					debug_assert!(elems.len() % (dim.len() + 1) == 0);
					Ok(ParserVal::IndexedArray(dim.len(), elems))
				}
			}
			_ => type_err("an array literal"),
		},
		Expression::ArrayLiteral2D(al) => {
			if let Type::Array {
				opt: _,
				dim,
				element,
			} = ty
			{
				if dim.len() != 2 {
					return type_err("a 2d array literal");
				}
				let col_indices = al
					.column_indices()
					.map(|i| typecheck_dzn(file, name, &i, &dim[1]))
					.collect::<Result<Vec<_>, _>>()?;
				let mut first = true;
				let mut col_count = 0;
				let mut row_indices = Vec::new();
				let mut row_count = 0;
				let mut values = Vec::new();
				for row in al.rows() {
					let members = row
						.members()
						.map(|m| typecheck_dzn(file, name, &m, &element))
						.collect::<Result<Vec<_>, _>>()?;
					let index = row.index();
					if let Some(ref i) = index {
						row_indices.push(typecheck_dzn(file, name, i, &dim[0])?);
					}

					if first {
						col_count = members.len();
						first = false;

						if !col_indices.is_empty() && col_count != col_indices.len() {
							return Err(InvalidArrayLiteral {
								src: file.clone(),
								span: al.cst_node().as_ref().byte_range().into(),
								msg: "2D array literal has different row length to index row"
									.to_string(),
							}
							.into());
						}
					} else if members.len() != col_count {
						return Err(InvalidArrayLiteral {
							src: file.clone(),
							span: al.cst_node().as_ref().byte_range().into(),
							msg: "Non-uniform 2D array literal row length".to_string(),
						}
						.into());
					}

					if index.is_none() != row_indices.is_empty() {
						return Err(InvalidArrayLiteral {
							src: file.clone(),
							span: al.cst_node().as_ref().byte_range().into(),
							msg: "Mixing indexed and non-indexed rows not allowed".to_string(),
						}
						.into());
					}

					values.extend(members);
					row_count += 1;
				}

				Ok(if row_indices.is_empty() && col_indices.is_empty() {
					ParserVal::SimpleArray(
						vec![
							(ParserVal::Integer(1), ParserVal::Integer(row_count)),
							(ParserVal::Integer(1), ParserVal::Integer(col_count as i64)),
						],
						values,
					)
				} else {
					if row_indices.is_empty() {
						row_indices.extend((1..=row_count).map(ParserVal::Integer))
					};
					let mut col_indices = col_indices;
					if col_indices.is_empty() {
						col_indices.extend((1..=col_count as i64).map(ParserVal::Integer))
					};

					let mut indexed_values = Vec::with_capacity(values.len() * 3);
					for ((row, col), v) in row_indices
						.into_iter()
						.cartesian_product(col_indices)
						.zip_eq(values)
					{
						indexed_values.extend_from_slice(&[row, col, v])
					}
					ParserVal::IndexedArray(2, indexed_values)
				})
			} else {
				type_err("a 2d array literal")
			}
		}
		Expression::Call(c) => match ty {
			Type::Enum(_, _) => {
				let ident: Identifier = c.function().cast().unwrap();
				let args = c.arguments().map( |expr |
					match expr {
						Expression::IntegerLiteral(v) => Ok(ParserVal::Integer(v.value().map_err(|e| InvalidNumericLiteral {
							src: file.clone(),
							span: v.cst_node().as_ref().byte_range().into(),
							msg: e.to_string(),
						})?)),
						Expression::Identifier(_) | Expression::Call(_) => {
							let unknown_enum: Type = Type::Enum(OptType::NonOpt, Arc::new(Enum::from_data("".into())));
							typecheck_dzn(file, name, &expr, &unknown_enum)
						},
						_ => Err(TypeMismatch {
							src: file.clone(),
							msg: format!("Constructor arguments for an enumerated type must be integers or values of enumerated types"),
							span: expr.cst_node().as_ref().byte_range().into(),
						}.into())
					}
				).collect::<Result<Vec<_>, _>>()?;
				Ok(ParserVal::Enum(ident.name().to_string(), args))
			}
			Type::Annotation(_) => todo!(),
			Type::Array {
				opt: _,
				dim: _,
				element: _,
			} => todo!(),
			_ => type_err("a call"),
		},
		Expression::InfixOperator(op) => {
			let extract_range = |op: &InfixOperator, ty| {
				let left = typecheck_dzn(file, name, &op.left(), ty)?;
				let right = typecheck_dzn(file, name, &op.right(), ty)?;
				Ok::<_, ShackleError>((left, right))
			};
			match op.operator().name() {
				".." => {
					if let Type::Set(_, ty) = ty {
						Ok(ParserVal::Range(Box::new(extract_range(op, ty)?)))
					} else {
						type_err("a range literal")
					}
				}
				"union" | "∪" => {
					if let Type::Set(_, ty) = ty {
						let non_range_err = |e: &CstNode| {
							Err(SyntaxError {
								src: file.clone(),
								span: e.as_ref().byte_range().into(),
								msg: "non range expression found in datazinc union operation"
									.to_string(),
								other: Vec::new(),
							}
							.into())
						};

						let mut ranges = Vec::with_capacity(2);
						let mut stack = vec![op.right(), op.left()];
						while let Some(e) = stack.pop() {
							if let Expression::InfixOperator(op) = e {
								match op.operator().name() {
									".." => ranges.push(op),
									"union" | "∪" => {
										stack.push(op.right());
										stack.push(op.left())
									}
									_ => return non_range_err(op.cst_node()),
								}
							} else {
								return non_range_err(e.cst_node());
							}
						}

						Ok(ParserVal::SetRangeList(
							ranges
								.iter()
								.map(|op| extract_range(op, ty))
								.collect::<Result<_, _>>()?,
						))
					} else {
						type_err("a union of ranges")
					}
				}
				"++" => todo!(),
				_ => unreachable!("other infix operators are not supported in DZN"),
			}
		}
		_ => unreachable!(), // Should not be accepted by the parser
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use expect_test::{expect, Expect};

	use super::parse_dzn;
	use crate::{data::dzn::typecheck_dzn, file::SourceFile, OptType, Type};

	fn check_serialization(input: &str, ty: &Type, expected: Expect) {
		let src = SourceFile::from(Arc::new(format!("x = {input};")));
		let assignments = parse_dzn(&src).expect("unexpected syntax error");
		assert_eq!(assignments.len(), 1);

		let val = typecheck_dzn(&src, "x", &assignments[0].definition(), ty)
			.expect("unexpected type error");
		let val = val.resolve_value(ty).expect("unexpected resolve error");
		let s = val.to_string();
		expected.assert_eq(&s);

		// Serialize as DZN and then deserialize again ensuring it is equal
		let src = SourceFile::from(Arc::new(format!("x = {val};")));
		let assignments = parse_dzn(&src).expect("unexpected syntax error");
		assert_eq!(assignments.len(), 1);
		let val = typecheck_dzn(&src, "x", &assignments[0].definition(), ty)
			.expect("unexpected type error");
		let val = val.resolve_value(ty).expect("unexpected resolve error");
		assert_eq!(s, val.to_string());
	}

	#[test]
	fn test_parse_absent() {
		check_serialization("<>", &Type::Integer(OptType::Opt), expect!("<>"));
	}

	#[test]
	fn test_parse_inf() {
		check_serialization("infinity", &Type::Integer(OptType::NonOpt), expect!("∞"));
		check_serialization("-infinity", &Type::Float(OptType::NonOpt), expect!("-∞"));
		check_serialization("∞", &Type::Float(OptType::NonOpt), expect!("∞"));
		check_serialization("-∞", &Type::Integer(OptType::NonOpt), expect!("-∞"));
	}

	#[test]
	fn test_parse_boolean() {
		check_serialization("true", &Type::Boolean(OptType::NonOpt), expect!("true"));
		check_serialization("false", &Type::Boolean(OptType::NonOpt), expect!("false"));
	}

	#[test]
	fn test_parse_integer() {
		check_serialization("0", &Type::Integer(OptType::NonOpt), expect!("0"));
		check_serialization("1", &Type::Integer(OptType::NonOpt), expect!("1"));
		check_serialization("99", &Type::Integer(OptType::NonOpt), expect!("99"));
		check_serialization("-1", &Type::Integer(OptType::NonOpt), expect!("-1"));
		check_serialization("0b1010", &Type::Integer(OptType::NonOpt), expect!("10"));
		check_serialization("0o70", &Type::Integer(OptType::NonOpt), expect!("56"));
		check_serialization("0xFF", &Type::Integer(OptType::NonOpt), expect!("255"));
	}

	#[test]
	fn test_parse_float() {
		check_serialization("0.1", &Type::Float(OptType::NonOpt), expect!("0.1"));
		check_serialization("3.65", &Type::Float(OptType::NonOpt), expect!("3.65"));
		check_serialization("-3.65", &Type::Float(OptType::NonOpt), expect!("-3.65"));
		check_serialization(
			"4.5e10",
			&Type::Float(OptType::NonOpt),
			expect!("45000000000"),
		);
		check_serialization(
			"5E-10",
			&Type::Float(OptType::NonOpt),
			expect!("0.0000000005"),
		);
	}

	#[test]
	fn test_parse_string() {
		check_serialization("\"\"", &Type::String(OptType::NonOpt), expect!([r#""""#]));
		check_serialization(
			"\"test\"",
			&Type::String(OptType::NonOpt),
			expect!([r#""test""#]),
		);
		check_serialization(
			"\"    Another test    \"",
			&Type::String(OptType::NonOpt),
			expect!([r#""    Another test    ""#]),
		);
		check_serialization(
			"\"\\t\\n\"",
			&Type::String(OptType::NonOpt),
			expect!([r#""\t\n""#]),
		);
	}

	#[test]
	fn test_parse_tuple() {
		check_serialization(
			"(1,)",
			&Type::Tuple(OptType::NonOpt, Arc::new([Type::Integer(OptType::NonOpt)])),
			expect!("(1,)"),
		);
		check_serialization(
			"(1, \"foo\")",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Integer(OptType::NonOpt),
					Type::String(OptType::NonOpt),
				]),
			),
			expect!([r#"(1, "foo")"#]),
		);
		check_serialization(
			"(2.5, true, <>,)",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Float(OptType::NonOpt),
					Type::Boolean(OptType::NonOpt),
					Type::Boolean(OptType::Opt),
				]),
			),
			expect!("(2.5, true, <>)"),
		);
		check_serialization(
			"([1, 2], {3, 4}, 5)",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Array {
						opt: OptType::NonOpt,
						dim: Box::new([Type::Integer(OptType::NonOpt)]),
						element: Box::new(Type::Integer(OptType::NonOpt)),
					},
					Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
					Type::Integer(OptType::NonOpt),
				]),
			),
			expect!("([1, 2], 3..3 ∪ 4..4, 5)"),
		);
		check_serialization(
			"(1, (2, (4, 5)), 6)",
			&Type::Tuple(
				OptType::NonOpt,
				Arc::new([
					Type::Integer(OptType::NonOpt),
					Type::Tuple(
						OptType::NonOpt,
						Arc::new([
							Type::Integer(OptType::NonOpt),
							Type::Tuple(
								OptType::NonOpt,
								Arc::new([
									Type::Integer(OptType::NonOpt),
									Type::Integer(OptType::NonOpt),
								]),
							),
						]),
					),
					Type::Integer(OptType::NonOpt),
				]),
			),
			expect!("(1, (2, (4, 5)), 6)"),
		);
	}

	#[test]
	fn test_parse_set() {
		check_serialization(
			"{ }",
			&Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
			expect!("∅"),
		);
		check_serialization(
			"∅",
			&Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
			expect!("∅"),
		);
		check_serialization(
			"{1.0}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..1"),
		);
		check_serialization(
			"{1,2.2}",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..1 ∪ 2.2..2.2"),
		);
		check_serialization(
			"1..3",
			&Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
			expect!("1..3"),
		);
		check_serialization(
			"1.0..3.3",
			&Type::Set(OptType::NonOpt, Box::new(Type::Float(OptType::NonOpt))),
			expect!("1..3.3"),
		);
	}

	#[test]
	fn test_parse_record() {
		let a: Arc<str> = "a".into();
		let b: Arc<str> = "b".into();
		let c: Arc<str> = "c".into();
		let d = "d".into();
		let e = "e".into();
		let f = "f".into();

		check_serialization(
			"(a: 1, b: 2.5)",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(a.clone(), Type::Integer(OptType::NonOpt)),
					(b.clone(), Type::Float(OptType::NonOpt)),
				]),
			),
			expect!("(a: 1, b: 2.5)"),
		);
		check_serialization(
			"( b: (3.5, true), a: {1, 2}, c: [<>])",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(
						a.clone(),
						Type::Set(OptType::NonOpt, Box::new(Type::Integer(OptType::NonOpt))),
					),
					(
						b.clone(),
						Type::Tuple(
							OptType::NonOpt,
							Arc::new([
								Type::Float(OptType::NonOpt),
								Type::Boolean(OptType::NonOpt),
							]),
						),
					),
					(
						c.clone(),
						Type::Array {
							opt: OptType::NonOpt,
							dim: [Type::Integer(OptType::NonOpt)].into(),
							element: Type::Integer(OptType::Opt).into(),
						},
					),
				]),
			),
			expect!("(a: 1..1 ∪ 2..2, b: (3.5, true), c: [<>])"),
		);

		check_serialization(
			"(b: (d: (e: 3, f: 4), c: 2), a: 1)",
			&Type::Record(
				OptType::NonOpt,
				Arc::new([
					(a, Type::Integer(OptType::NonOpt)),
					(
						b,
						Type::Record(
							OptType::NonOpt,
							Arc::new([
								(c, Type::Integer(OptType::NonOpt)),
								(
									d,
									Type::Record(
										OptType::NonOpt,
										Arc::new([
											(e, Type::Integer(OptType::NonOpt)),
											(f, Type::Integer(OptType::NonOpt)),
										]),
									),
								),
							]),
						),
					),
				]),
			),
			expect!("(a: 1, b: (c: 2, d: (e: 3, f: 4)))"),
		);
	}

	#[test]
	fn test_parse_simple_array() {
		check_serialization(
			"[]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Integer(OptType::NonOpt).into(),
			},
			expect!("[]"),
		);
		check_serialization(
			"[1.0]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Float(OptType::NonOpt).into(),
			},
			expect!("[1]"),
		);
		check_serialization(
			"[1, 2.2]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Float(OptType::NonOpt).into(),
			},
			expect!("[1, 2.2]"),
		);
		check_serialization(
			"[<>, <>, 1, <>,]",
			&Type::Array {
				opt: OptType::NonOpt,
				dim: [Type::Integer(OptType::NonOpt)].into(),
				element: Type::Integer(OptType::Opt).into(),
			},
			expect!("[<>, <>, 1, <>]"),
		);
	}

	// #[test]
	// fn test_parse_ident() {
	// 	let (_, out) = identifier(span("Albus")).unwrap();
	// 	assert_eq!(out, "Albus");
	// 	assert!(identifier(span("1")).is_err());
	// }

	// #[test]
	// fn test_parse_enum_val() {
	// 	// Simple identifier representing an enum value
	// 	let (_, out) = value_or_enum(span("A")).unwrap();
	// 	assert_eq!(out, ParsedVal::EnumVal(EnumVal::Ident("A".to_owned())));

	// 	// Enum value with integer argument
	// 	let (_, out) = value_or_enum(span("A(1)")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumVal(EnumVal::IntArg(("A".to_owned(), 1)))
	// 	);

	// 	// Enum value with another enum value as argument
	// 	let (_, out) = value_or_enum(span("A(B)")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumVal(EnumVal::EnumArg((
	// 			"A".to_owned(),
	// 			Box::new(EnumVal::Ident("B".to_owned()))
	// 		)))
	// 	);

	// 	// Complex chain of enum constructors to make value
	// 	let (_, out) = value_or_enum(span("A(B(C(D(-60))))")).unwrap();
	// 	assert_eq!(out.to_string(), "A(B(C(D(-60))))");
	// }

	// #[test]
	// fn test_parse_enum_members() {
	// 	let (_, out) = value_or_enum(span("{ A }")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumSetList(vec![EnumVal::Ident("A".to_owned())])
	// 	);

	// 	let (_, out) = value_or_enum(span("{ A, B, C }")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumSetList(vec![
	// 			EnumVal::Ident("A".to_owned()),
	// 			EnumVal::Ident("B".to_owned()),
	// 			EnumVal::Ident("C".to_owned())
	// 		])
	// 	);

	// 	let (_, out) = value_or_enum(span("X(1..6)")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumCtor(EnumCtor::SetArg((
	// 			"X".to_owned(),
	// 			Set::IntRangeList(vec![1..=6])
	// 		)))
	// 	);

	// 	let (_, out) = value_or_enum(span("{ A } ++ Z(-1..1) ++ X(Y)")).unwrap();
	// 	assert_eq!(
	// 		out,
	// 		ParsedVal::EnumCtor(EnumCtor::Concat(vec![
	// 			EnumCtor::ValueList(vec!["A".to_owned()]),
	// 			EnumCtor::SetArg(("Z".to_owned(), Set::IntRangeList(vec![-1..=1]))),
	// 			EnumCtor::NameArg(("X".to_owned(), "Y".to_owned()))
	// 		]))
	// 	)
	// }
}
