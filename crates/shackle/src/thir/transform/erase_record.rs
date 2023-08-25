//! Replace records with tuples.
//!
//! Records should already have been sorted, so we can just turn them directly into tuples.

use crate::{
	hir::{IntegerLiteral, OptType},
	thir::{
		db::Thir,
		traverse::{fold_domain, fold_expression, Folder, ReplacementMap},
		Domain, DomainData, Expression, ExpressionData, Marker, Model, TupleAccess, TupleLiteral,
	},
	utils::maybe_grow_stack,
};

struct RecordEraser<Dst: Marker, Src: Marker = ()> {
	model: Model<Dst>,
	replacement_map: ReplacementMap<Dst, Src>,
}

impl<Dst: Marker, Src: Marker> Folder<'_, Dst, Src> for RecordEraser<Dst, Src> {
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
		maybe_grow_stack(|| {
			let origin = expression.origin();
			match &**expression {
				ExpressionData::RecordLiteral(rl) => {
					let mut pairs = rl
						.iter()
						.map(|(i, e)| (*i, self.fold_expression(db, model, e)))
						.collect::<Vec<_>>();
					pairs.sort_by_key(|(i, _)| *i);
					let fields = pairs.into_iter().map(|(_, e)| e).collect();
					let mut e = Expression::new(db, &self.model, origin, TupleLiteral(fields));
					e.annotations_mut().extend(
						expression
							.annotations()
							.iter()
							.map(|ann| self.fold_expression(db, model, ann)),
					);
					e
				}
				ExpressionData::RecordAccess(ra) => {
					let field_tys = ra.record.ty().record_fields(db.upcast()).unwrap();
					let tuple = self.fold_expression(db, model, &ra.record);
					Expression::new(
						db,
						&self.model,
						origin,
						TupleAccess {
							tuple: Box::new(tuple),
							field: field_tys
								.iter()
								.enumerate()
								.find_map(|(n, (i, _))| {
									if *i == ra.field.0 {
										Some(IntegerLiteral(n as i64 + 1))
									} else {
										None
									}
								})
								.unwrap(),
						},
					)
				}
				_ => fold_expression(self, db, model, expression),
			}
		})
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
pub fn erase_record(db: &dyn Thir, model: Model) -> Model {
	log::info!("Erasing record types");
	let mut c = RecordEraser {
		model: Model::default(),
		replacement_map: ReplacementMap::default(),
	};
	c.add_model(db, &model);
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
				int: y = x.foo;
				float: z = x.bar;
            "#,
			expect!([r#"
    tuple(int, float): x = (1, 2.5);
    int: y = x.1;
    float: z = x.2;
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_record_type_erasure_sorting() {
		check_no_stdlib(
			erase_record,
			r#"
                record(int: foo, float: bar): x = (bar: 2.5, foo: 1);
				int: y = x.foo;
				float: z = x.bar;
            "#,
			expect!([r#"
    tuple(int, float): x = (1, 2.5);
    int: y = x.1;
    float: z = x.2;
    solve satisfy;
"#]),
		);
	}
}
