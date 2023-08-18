//! Generation of output
//! - Create ::output_only string declarations for output item sections
//!

use rustc_hash::FxHashMap;

use crate::{
	hir::{Identifier, StringLiteral},
	thir::{
		db::Thir, source::Origin, Declaration, Domain, Expression, ExpressionData, Item,
		LookupCall, Model,
	},
};

/// Generate the output
pub fn generate_output(db: &dyn Thir, model: &Model) -> Model {
	let ids = db.identifier_registry();
	let tys = db.type_registry();
	let origin = Origin::Introduced("<generated-output>");
	let mut model = model.clone();
	let mut sections: FxHashMap<StringLiteral, Vec<Expression>> = FxHashMap::default();
	let outputs = model.take_outputs();
	for output in outputs {
		let (_, output) = output.into_inner();
		let (section, expression) = output.into_inner();
		if let Some(s) = section {
			let section = match s.into_inner().1 {
				ExpressionData::StringLiteral(sl) => sl,
				_ => unreachable!(),
			};
			sections.entry(section).or_default().push(expression)
		} else {
			sections
				.entry(ids.default.into())
				.or_default()
				.push(expression)
		}
	}
	let mut sections = sections
		.into_iter()
		.map(|(k, v)| (k.value(db.upcast()), v))
		.collect::<Vec<_>>();
	sections.sort_by(|(a, _), (b, _)| a.cmp(b));
	for (section, expressions) in sections {
		let definition = expressions
			.into_iter()
			.reduce(|acc, e| {
				Expression::new(
					db,
					&model,
					origin,
					LookupCall {
						function: ids.plus_plus.into(),
						arguments: vec![acc, e],
					},
				)
			})
			.map(|arg| {
				Expression::new(
					db,
					&model,
					origin,
					LookupCall {
						function: ids.concat.into(),
						arguments: vec![arg],
					},
				)
			})
			.unwrap_or_else(|| {
				Expression::new(db, &model, origin, StringLiteral::from(ids.empty_string))
			});
		let mut declaration = Declaration::new(true, Domain::unbounded(db, origin, tys.string));
		declaration.set_name(Identifier::new(
			format!("mzn_output_{}", section),
			db.upcast(),
		));
		declaration
			.annotations_mut()
			.push(Expression::new(db, &model, origin, ids.output_only));
		declaration.set_definition(definition);
		model.add_declaration(Item::new(declaration, origin));
	}
	model
}

#[cfg(test)]
mod test {
	use expect_test::expect;

	use crate::thir::transform::test::check;

	use super::generate_output;

	#[test]
	fn test_output_generation() {
		check(
			generate_output,
			r#"
				output ["Hello, world"];
				output :: "one" ["A"];
				output :: "two" ["B"];
				output :: "one" ["C"];
            "#,
			expect!([r#"
    string: mzn_output_default :: (output_only) = concat(["Hello, world"]);
    string: mzn_output_one :: (output_only) = concat('++'(["A"], ["C"]));
    string: mzn_output_two :: (output_only) = concat(["B"]);
"#]),
		);
	}
}
