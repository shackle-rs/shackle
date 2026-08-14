//! Generation of output
//! - Create `::output_only` string declarations for output item sections
//! - Make make default `::output` variables explicit

use rustc_hash::FxHashMap;
use shackle_diagnostics::Result;
use shackle_hir::{Identifier, StringLiteral, constants::IdentifierRegistry};
use shackle_ty::registry::TypeRegistry;

use crate::{
	Db, Declaration, Domain, Expression, ExpressionData, Item, LookupCall, LookupIdentifier, Model,
	source::Origin,
};

/// Generate the output
pub fn generate_output<'db>(db: &'db dyn Db, mut model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Generating output");

	let ids = IdentifierRegistry::lookup(db);
	let tys = TypeRegistry::lookup(db);
	let origin = Origin::Introduced("<generated-output>");
	let mut sections: FxHashMap<StringLiteral, Vec<Expression>> = FxHashMap::default();
	let outputs = model.take_outputs();
	for output in outputs {
		let (_, output) = output.into_inner();
		let (section, expression) = output.into_inner();
		if let Some(s) = section {
			let section = match &*s {
				ExpressionData::StringLiteral(sl) => sl.clone(),
				_ => unreachable!(),
			};
			sections.entry(section).or_default().push(expression)
		} else {
			sections
				.entry(ids.literals.default.into())
				.or_default()
				.push(expression)
		}
	}
	let mut sections = sections
		.into_iter()
		.map(|(k, v)| (k.value(db), v))
		.collect::<Vec<_>>();
	sections.sort_by_key(|(a, _)| *a);
	for (section, expressions) in sections {
		let definition = expressions
			.into_iter()
			.reduce(|acc, e| {
				Expression::new(
					db,
					&model,
					origin,
					LookupCall {
						function: ids.functions.plus_plus.into(),
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
						function: ids.functions.concat.into(),
						arguments: vec![arg],
					},
				)
			})
			.unwrap_or_else(|| {
				Expression::new(
					db,
					&model,
					origin,
					StringLiteral::from(ids.literals.empty_string),
				)
			});
		let mut declaration = Declaration::new(true, Domain::unbounded(db, origin, tys.string));
		declaration.set_name(Identifier::new(db, format!("mzn_output_{}", section)));
		declaration.annotations_mut().push(Expression::new(
			db,
			&model,
			origin,
			ids.annotations.output_only,
		));
		declaration.annotations_mut().push(Expression::new(
			db,
			&model,
			origin,
			ids.annotations.output,
		));
		declaration.set_definition(definition);
		let _ = model.add_declaration(Item::new(declaration, origin));
	}

	// Make output variables explicit
	let implicit_output_vars = model
		.top_level_declarations()
		.filter_map(|(idx, decl)| {
			if decl.definition().is_none()
				&& !decl.ty().known_par(db)
				&& !decl.annotations().has(&model, ids.annotations.no_output)
				&& !decl.annotations().has(&model, ids.annotations.output)
			{
				Some(idx)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
	for idx in implicit_output_vars {
		let output_ann = Expression::new(
			db,
			&model,
			model[idx].origin(),
			LookupIdentifier(ids.annotations.output),
		);
		model[idx].annotations_mut().push(output_ann);
	}

	Ok(model)
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::generate_output;
	use crate::transform::tests::check;

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
    string: mzn_output_default :: (output_only) :: ('output') = concat(["Hello, world"]);
    string: mzn_output_one :: (output_only) :: ('output') = concat('++'(["A"], ["C"]));
    string: mzn_output_two :: (output_only) :: ('output') = concat(["B"]);
"#]),
		);
	}

	#[test]
	fn test_implicit_output_vars() {
		check(
			generate_output,
			r#"
				var 1..3: x;
				var opt 1..3: y;
				array [1..2] of var 1..3: z;
				var 1..3: p :: output;
				var 1..2: q :: no_output;
				1..3: a;
				var 1..3: b = 2;
			"#,
			expect!([r#"
    var '..'(1, 3): x :: ('output');
    var opt '..'(1, 3): y :: ('output');
    array ['..'(1, 2)] of var '..'(1, 3): z :: ('output');
    var '..'(1, 3): p :: ('output');
    var '..'(1, 2): q :: (no_output);
    '..'(1, 3): a;
    var '..'(1, 3): b = 2;
"#]),
		);
	}
}
