//! Determine which top-level variables which need to be accessible from anywhere in the program.
//!
//! This includes
//! - Input parameters of the model
//! - Variables marked as output
//! - Global variables referred to from within function bodies
//! - Global variables referred to from output_only expressions

use shackle_hir::{Db, constants::IdentifierRegistry};
use shackle_utils::hash::Set;

use crate::{DeclarationId, Marker, Model, ResolvedIdentifier, traverse::Visitor};

/// Global variables in a THIR model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalVariables<'db, T: Marker> {
	/// Input parameters of the model
	pub input_parameters: Vec<DeclarationId<'db, T>>,
	/// Global variables referred to from within function bodies (or that need to be output)
	pub referred_variables: Vec<DeclarationId<'db, T>>,
}

impl<'db, T: Marker> GlobalVariables<'db, T> {
	/// Find the global variables
	pub fn find(db: &'db dyn Db, model: &Model<'db, T>) -> Self {
		let mut visitor = GlobalVarVisitor {
			referred_variables: Vec::new(),
			seen: Set::new(),
		};
		let mut input_parameters = Vec::new();
		for (declaration, decl) in model.top_level_declarations() {
			if decl.definition().is_none() && decl.ty().contains_par(db) {
				input_parameters.push(declaration);
				let _ = visitor.seen.insert(declaration);
			}
		}
		for (function, _) in model.all_functions() {
			visitor.visit_function(model, function);
		}
		let ids = IdentifierRegistry::lookup(db);
		for (idx, declaration) in model.top_level_declarations() {
			if declaration.annotations().has(model, ids.annotations.output) {
				visitor.visit_identifier(model, &ResolvedIdentifier::Declaration(idx));
			}
			if declaration
				.annotations()
				.has(model, ids.annotations.output_only)
			{
				visitor.visit_expression(model, declaration.definition().unwrap());
			}
		}
		Self {
			input_parameters,
			referred_variables: visitor.referred_variables,
		}
	}
}

struct GlobalVarVisitor<'db, T: Marker> {
	referred_variables: Vec<DeclarationId<'db, T>>,
	seen: Set<DeclarationId<'db, T>>,
}

impl<'db, T: Marker> Visitor<'_, 'db, T> for GlobalVarVisitor<'db, T> {
	fn visit_identifier(&mut self, model: &Model<'db, T>, identifier: &ResolvedIdentifier<'db, T>) {
		if let ResolvedIdentifier::Declaration(d) = identifier {
			let decl = &model[*d];
			if decl.top_level() && self.seen.insert(*d) {
				self.referred_variables.push(*d);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use super::*;
	use crate::lower::lower_model;

	#[derive(Debug)]
	struct GlobalVariablesDebug {
		#[allow(dead_code, reason = "Used by debug impl")]
		input_parameters: Vec<String>,
		#[allow(dead_code, reason = "Used by debug impl")]
		referred_variables: Vec<String>,
	}

	fn check(program: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, program.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let model = lower_model(&db).take();
		let global_vars = GlobalVariables::find(&db, &model);
		let input_parameters = global_vars
			.input_parameters
			.iter()
			.map(|d| model[*d].name().unwrap().pretty_print(&db))
			.collect();
		let referred_variables = global_vars
			.referred_variables
			.iter()
			.map(|d| model[*d].name().unwrap().pretty_print(&db))
			.collect();
		let actual = GlobalVariablesDebug {
			input_parameters,
			referred_variables,
		};
		expected.assert_debug_eq(&actual);
	}

	#[test]
	fn test_find_global_variables() {
		check(
			r#"
			function int: fix(var int: x);
			annotation 'output';
			annotation output_only;
            predicate foo() = let {
                any: v = x;
				any: another = x;
				any: data = y;
            } in true;
			predicate bar() = let {
				any: v = w;
			} in true;
            var int: x;
            var int: z;
            int: y;
            int: w = 3;
			var int: p :: output;
			var int: q;
			int: r :: output :: output_only = q;
        "#,
			expect![[r#"
    GlobalVariablesDebug {
        input_parameters: [
            "y",
        ],
        referred_variables: [
            "x",
            "w",
            "p",
            "r",
            "q",
        ],
    }
"#]],
		)
	}
}
