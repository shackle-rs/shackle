//! Validation of object-syntax constructs.
//!
//! Object syntax (class declarations and `new` object introductions) is
//! type-checked, but is not lowered to THIR yet. Until it is, any use of it is
//! rejected here, at the end of the HIR phase, so that a model using objects
//! reports an error rather than reaching a lowering step which cannot handle it.
//!
//! When object lowering lands, this module becomes the place which rejects only
//! the object shapes that lowering does not support, instead of all of them.

use shackle_diagnostics::UnsupportedObjectFeature;

use crate::{
	Db, Item,
	diagnostics::Errors,
	ids::{NodeRef, TypeRef},
	lower::lower_models,
};

/// Report every use of object syntax as unsupported.
pub fn validate_object_lowering(db: &dyn Db) {
	for model in lower_models(db).iter() {
		for item in model.items(db).iter() {
			match item {
				Item::Class(_) => {
					let (src, span) = NodeRef::from(*item).source_span(db);
					Errors::add(
						db,
						UnsupportedObjectFeature {
							src,
							span,
							msg: "class declarations are not supported yet".to_owned(),
						},
					);
				}
				Item::Declaration(d) => {
					let declaration = d.declaration(db);
					let data = declaration.data();
					if data[declaration.declared_type].is_new(data) {
						let (src, span) =
							TypeRef::new(db, *item, declaration.declared_type).source_span(db);
						Errors::add(
							db,
							UnsupportedObjectFeature {
								src,
								span,
								msg: "object introductions are not supported yet".to_owned(),
							},
						);
					}
				}
				_ => (),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_syntax::InputLang;

	use crate::{
		db::CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
		run_hir_phase,
	};

	fn check_object_errors(model: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model = InlineModelFile::new(&db, model.to_owned(), InputLang::MiniZinc);
		let _ = InputFiles::get(&db)
			.set_files(&mut db)
			.to(vec![model.into()]);
		let errors = run_hir_phase(&db)
			.errors
			.iter()
			.map(|e| e.to_string())
			.collect::<Vec<_>>()
			.join("\n");
		expected.assert_eq(&errors);
	}

	#[test]
	fn test_object_syntax_is_rejected() {
		// Object syntax typechecks but cannot be lowered yet, so it must be
		// rejected rather than reaching THIR
		check_object_errors(
			r#"
			class A (int: x);
			new A: a = (x: 3);
			"#,
			expect![[r#"
                Unsupported object feature
                Unsupported object feature"#]],
		);
		// A model which does not use objects is unaffected
		check_object_errors(
			r#"
			var int: x;
			"#,
			expect![""],
		);
	}
}
