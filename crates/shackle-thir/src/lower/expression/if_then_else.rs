//! Lowering of if-then-else expressions

use shackle_ty::{Ty, TyData};

use crate::{
	lower::expression::{ExpressionCollector, alloc_expression},
	source::Origin,
	*,
};

impl<'db, 'a, 'b, 'c> ExpressionCollector<'db, 'a, 'b, 'c> {
	/// Collect if-then-else expression
	pub(super) fn collect_if_then_else(
		&mut self,
		ite: &shackle_hir::IfThenElse<'db>,
		ty: Ty<'db>,
		origin: impl Into<Origin<'db>>,
	) -> Expression<'db> {
		let origin = origin.into();
		alloc_expression(
			IfThenElse {
				branches: ite
					.branches
					.iter()
					.map(|b| {
						Branch::new(
							self.collect_expression(b.condition),
							self.collect_expression(b.result),
						)
					})
					.collect(),
				else_result: Box::new(
					ite.else_result
						.map(|e| self.collect_expression(e))
						.unwrap_or_else(|| self.collect_default_else(ty, origin)),
				),
			},
			self,
			origin,
		)
	}

	/// Create a default else expression for an if-then-else, based on the expected type.
	pub(super) fn collect_default_else(
		&mut self,
		ty: Ty<'db>,
		origin: Origin<'db>,
	) -> Expression<'db> {
		let db = self.parent.db;
		match ty.lookup(db) {
			TyData::Boolean(_, OptType::Opt)
			| TyData::Integer(_, OptType::Opt)
			| TyData::Float(_, OptType::Opt)
			| TyData::Enum(_, OptType::Opt, _)
			| TyData::Bottom(OptType::Opt)
			| TyData::Array {
				opt: OptType::Opt, ..
			}
			| TyData::Set(_, OptType::Opt, _)
			| TyData::Tuple(OptType::Opt, _)
			| TyData::Record(OptType::Opt, _)
			| TyData::Function(OptType::Opt, _)
			| TyData::TyVar(_, Some(OptType::Opt), _) => alloc_expression(Absent, self, origin),
			TyData::Boolean(_, _) => alloc_expression(BooleanLiteral(true), self, origin),
			TyData::String(_) => {
				alloc_expression(StringLiteral::new(self.parent.db, ""), self, origin)
			}
			TyData::Annotation(_) => {
				alloc_expression(self.parent.ids.annotations.empty_annotation, self, origin)
			}
			TyData::Array { .. } => alloc_expression(ArrayLiteral::default(), self, origin),
			TyData::Set(_, _, _) => alloc_expression(SetLiteral::default(), self, origin),
			TyData::Tuple(_, fs) => alloc_expression(
				TupleLiteral(
					fs.iter()
						.map(|f| self.collect_default_else(*f, origin))
						.collect(),
				),
				self,
				origin,
			),
			TyData::Record(_, fs) => alloc_expression(
				RecordLiteral(
					fs.iter()
						.map(|(i, t)| (Identifier(*i), self.collect_default_else(*t, origin)))
						.collect(),
				),
				self,
				origin,
			),
			_ => unreachable!("No default value for this type"),
		}
	}
}
