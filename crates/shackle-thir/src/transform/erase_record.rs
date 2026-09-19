//! Replace records with tuples.
//!
//! Records should already have been sorted, so we can just turn them directly into tuples.
use shackle_diagnostics::Result;
use shackle_hir::{IntegerLiteral, OptType};
use shackle_utils::maybe_grow_stack;

use crate::{
	Db, Domain, DomainData, Expression, ExpressionData, Marker, Model, TupleAccess, TupleLiteral,
	traverse::{Folder, ReplacementMap, fold_domain, fold_expression},
};

struct RecordEraser<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for RecordEraser<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		expression: &Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			let origin = expression.origin();
			match &**expression {
				ExpressionData::RecordLiteral(rl) => {
					let mut pairs = rl
						.iter()
						.map(|(i, e)| (*i, self.fold_expression(db, model, e)))
						.collect::<Vec<_>>();
					pairs.sort_by_key(|(i, _)| i.lookup(db));
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
					let field_tys = ra.record.ty().record_fields(db).unwrap();
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
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		domain: &Domain<'db, Src>,
	) -> Domain<'db, Dst> {
		maybe_grow_stack(|| {
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
		})
	}
}

/// Erase types which are not present in MicroZinc
pub fn erase_record<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Erasing record types");
	let mut c = RecordEraser {
		model: Model::with_capacities(&model.item_counts()),
		replacement_map: ReplacementMap::default(),
	};
	c.add_model(db, &model);
	Ok(c.model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check_no_stdlib};

	#[test]
	fn test_record_type_erasure() {
		check_no_stdlib(
			Transform::EraseRecord,
			r#"
                record(int: foo, float: bar): x = (foo: 1, bar: 2.5);
				int: y = x.foo;
				float: z = x.bar;
            "#,
			expect!([r#"
    tuple(float, int): x = (2.5, 1);
    int: y = (x).2;
    float: z = (x).1;
    solve satisfy;
"#]),
		);
	}

	#[test]
	fn test_record_type_erasure_sorting() {
		check_no_stdlib(
			Transform::EraseRecord,
			r#"
                record(int: foo, float: bar): x = (bar: 2.5, foo: 1);
				int: y = x.foo;
				float: z = x.bar;
            "#,
			expect!([r#"
    tuple(float, int): x = (2.5, 1);
    int: y = (x).2;
    float: z = (x).1;
    solve satisfy;
"#]),
		);
	}
}
