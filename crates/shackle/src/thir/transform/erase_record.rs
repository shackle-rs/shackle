//! Replace records with tuples.
//!
//! Records should already have been sorted, so we can just turn them directly into tuples.

use crate::{
	hir::OptType,
	thir::{
		db::Thir,
		traverse::{fold_domain, fold_expression, Folder, ReplacementMap},
		Domain, DomainData, Expression, ExpressionData, Marker, Model, TupleLiteral,
	},
};

struct RecordEraser<Dst, Src = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
}

impl<Dst: Marker, Src: Marker> Folder<Dst, Src> for RecordEraser<Dst, Src> {
	fn model(&mut self) -> &mut Model<Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src> {
		&mut self.replacement_map
	}

	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		expression: &Expression<Src>,
	) -> Expression<Dst> {
		let origin = expression.origin();
		match &**expression {
			ExpressionData::RecordLiteral(rl) => {
				let fields = rl
					.iter()
					.map(|(_, e)| self.fold_expression(db, model, e))
					.collect();
				let mut e = Expression::new(db, &self.model, origin, TupleLiteral(fields));
				e.annotations_mut().extend(
					expression
						.annotations()
						.iter()
						.map(|ann| self.fold_expression(db, model, ann)),
				);
				e
			}
			_ => fold_expression(self, db, model, expression),
		}
	}

	fn fold_domain(
		&mut self,
		db: &dyn Thir,
		model: &Model<Src>,
		domain: &Domain<Src>,
	) -> Domain<Dst> {
		let origin = domain.origin();
		match &**domain {
			DomainData::Record(items) => {
				let fields = items
					.iter()
					.map(|(_, d)| self.fold_domain(db, model, d))
					.collect::<Vec<_>>();
				Domain::tuple(db, origin, OptType::NonOpt, fields)
			}
			_ => fold_domain(self, db, model, domain),
		}
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_record(db: &dyn Thir, model: &Model) -> Model {
	let mut c = RecordEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
	};
	c.add_model(db, model);
	c.model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check_no_stdlib;

	use super::erase_record;

	#[test]
	fn test_record_type_erasure() {
		check_no_stdlib(
			erase_record,
			r#"
                record(int: foo, float: bar): x = (foo: 1, bar: 2.5);
            "#,
			expect!([r#"
    tuple(int, float): x = (1, 2.5);
    solve satisfy;
"#]),
		);
	}
}
