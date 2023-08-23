//! # The DataZinc parser
//!
//! This module contains a parser for the DataZinc format (i.e., `.dzn`) files.
//! These files are often used to provide data for MiniZinc models.

use itertools::Itertools;
use tree_sitter::Parser;

use crate::{
	data::ParserVal,
	diagnostics::{InvalidArrayLiteral, ShackleError, SyntaxError, TypeMismatch},
	file::SourceFile,
	syntax::{
		ast::{
			Assignment, AstNode, Children, Expression, Identifier, InfixOperator,
			RecordLiteralMember,
		},
		cst::{Cst, CstNode},
	},
	value::Polarity,
	OptType, Type,
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
		Expression::IntegerLiteral(il) => {
			if matches!(ty, Type::Integer(_) | Type::Float(_)) {
				Ok(ParserVal::Integer(il.value()))
			} else {
				type_err("an integer literal")
			}
		}
		Expression::FloatLiteral(fl) => {
			if matches!(ty, Type::Float(_)) {
				Ok(ParserVal::Float(fl.value()))
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
		Expression::BooleanLiteral(b) => {
			if matches!(ty, Type::Boolean(_) | Type::Integer(_) | Type::Float(_)) {
				Ok(ParserVal::Boolean(b.value()))
			} else {
				type_err("a Boolean literal")
			}
		}
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
		Expression::Infinity(_) => {
			if matches!(ty, Type::Integer(_) | Type::Float(_)) {
				Ok(ParserVal::Infinity(Polarity::Pos))
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
					|| (members.len() > 1 && members[1].indices().is_none())
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
						ParserVal::Integer(v + elems.len() as i64)
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
						Expression::IntegerLiteral(v) => Ok(ParserVal::Integer(v.value())),
						Expression::Identifier(_) | Expression::Call(_) => {
							const UNKNOWN_ENUM: Type = Type::Enum(OptType::NonOpt, None);
							typecheck_dzn(file, name, &expr, &UNKNOWN_ENUM)
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

						Ok(ParserVal::Range(Box::new(extract_range(op, ty)?)))
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
mod tests {}
