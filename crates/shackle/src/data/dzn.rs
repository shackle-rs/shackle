//! # The DataZinc parser
//!
//! This module contains a parser for the DataZinc format (i.e., `.dzn`) files.
//! These files are often used to provide data for MiniZinc models.

use itertools::Itertools;
use tree_sitter::Parser;

use crate::{
	data::ParserVal,
	diagnostics::{ShackleError, TypeMismatch},
	file::SourceFile,
	syntax::{
		ast::{Assignment, AstNode, Children, Expression, RecordLiteralMember},
		cst::Cst,
	},
	value::Polarity,
	Type,
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
		.parse(src.as_bytes(), None)
		.expect("DataZinc Tree Sitter parser did not return tree object");

	let cst = Cst::from_str(tree, src.contents());

	// TODO: Check for syntax errors

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
					if exprs.len() <= i || exprs[i].name().name() != elem_tys[i].0 {
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
					.map(|elem| typecheck_dzn(file, name, &elem, &elem_ty))
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
			if matches!(ty, Type::Float(_)) {
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
				} else if members[0].indices().is_none() {
					// Array literal without any indices
					if dim.len() != 1 {
						return Err(TypeMismatch {
							src: file.clone(),
							msg: format!("The parameter variable '{}' is declared in the model as type '{}', but the assigned array literal is missing indices.", name, ty),
							span: members[0].cst_node().as_ref().byte_range().into(),
						}
						.into());
					}
					let mut elems = Vec::with_capacity(members.len());
					for m in members {
						if m.indices().is_some() {
							todo!()
						}
						elems.push(typecheck_dzn(file, name, &m.value(), &element)?);
					}
					Ok(ParserVal::SimpleArray(
						vec![(
							ParserVal::Integer(1),
							ParserVal::Integer(elems.len() as i64),
						)],
						elems,
					))
				} else if members.len() > 1 && members[1].indices().is_none() {
					// Array literal with indices for only the first element
					todo!()
				} else {
					// Array literal with indices for all element
					todo!()
				}
			}
			_ => type_err("an array literal"),
		},
		Expression::ArrayLiteral2D(_) => todo!(),
		_ => unreachable!(), // Should not be accepted by the parser
	}
}

#[cfg(test)]
mod tests {}
