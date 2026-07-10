//! Utility for following expressions

use std::iter::from_fn;

use shackle_utils::maybe_grow_stack;

use super::{Expression, ExpressionData, Marker, Model, ResolvedIdentifier};

/// Follow this expression through identifiers and tuple/record access
///
/// Does not evaluate values
pub fn follow_expression<'a, 'db, T: Marker>(
	model: &'a Model<'db, T>,
	expression: &'a Expression<'db, T>,
) -> impl Iterator<Item = &'a Expression<'db, T>> {
	let mut current = follow_expression_inner(model, expression);
	from_fn(move || {
		let result = current;
		if let Some(e) = current {
			current = follow_expression_inner(model, e);
		}
		result
	})
}

fn follow_expression_inner<'a, 'db, T: Marker>(
	model: &'a Model<'db, T>,
	expression: &'a Expression<'db, T>,
) -> Option<&'a Expression<'db, T>> {
	match &**expression {
		ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) => model[*d].definition(),
		ExpressionData::TupleAccess(ta) => {
			let mut current: Option<&Expression<T>> = Some(&ta.tuple);
			loop {
				let tuple = current?;
				if let ExpressionData::TupleLiteral(tl) = &**tuple {
					return Some(&tl[(ta.field.0 - 1) as usize]);
				}
				maybe_grow_stack(|| {
					current = follow_expression_inner(model, tuple);
				});
			}
		}
		ExpressionData::RecordAccess(ra) => {
			let mut current: Option<&Expression<T>> = Some(&ra.record);
			loop {
				let record = current?;
				if let ExpressionData::RecordLiteral(rl) = &**record {
					return Some(rl.as_hash_map()[&ra.field]);
				}
				maybe_grow_stack(|| {
					current = follow_expression_inner(model, record);
				});
			}
		}
		ExpressionData::Let(l) => Some(&l.in_expression),
		ExpressionData::Identifier(ResolvedIdentifier::Annotation(_))
		| ExpressionData::Identifier(ResolvedIdentifier::Enumeration(_))
		| ExpressionData::Identifier(ResolvedIdentifier::EnumerationMember(_))
		| ExpressionData::IfThenElse(_)
		| ExpressionData::Case(_)
		| ExpressionData::Call(_)
		| ExpressionData::Absent
		| ExpressionData::BooleanLiteral(_)
		| ExpressionData::IntegerLiteral(_)
		| ExpressionData::FloatLiteral(_)
		| ExpressionData::StringLiteral(_)
		| ExpressionData::Infinity
		| ExpressionData::ArrayLiteral(_)
		| ExpressionData::SetLiteral(_)
		| ExpressionData::TupleLiteral(_)
		| ExpressionData::RecordLiteral(_)
		| ExpressionData::ArrayComprehension(_)
		| ExpressionData::SetComprehension(_)
		| ExpressionData::Lambda(_) => None,
	}
}
