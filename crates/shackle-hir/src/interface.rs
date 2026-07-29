//! Input/output interface

use shackle_ty::Ty;
use shackle_utils::hash::{Map, Set};

use crate::{
	Db, GlobalScope, Identifier, Item, PatternTy, constants::IdentifierRegistry,
	input::resolve_includes,
};

/// Interface for program input
#[derive(Debug, Clone, PartialEq, Eq, salsa::SalsaValue)]
pub struct InputInterface<'db> {
	/// The set of input enums
	pub enums: Set<Identifier<'db>>,
	/// A map from input parameters name to type
	pub params: Map<Identifier<'db>, Ty<'db>>,
}

impl<'db> InputInterface<'db> {
	/// Get the input interface
	pub fn get(db: &'db dyn Db) -> &'db Self {
		get_input_interface(db)
	}

	/// Get the input enums sorted by name
	pub fn enums(&self, db: &'db dyn Db) -> Vec<&'db str> {
		let mut names = self.enums.iter().map(|i| i.lookup(db)).collect::<Vec<_>>();
		names.sort();
		names
	}

	/// Get the input parameters sorted by name
	pub fn params(&self, db: &'db dyn Db) -> Vec<(&'db str, Ty<'db>)> {
		let mut params = self
			.params
			.iter()
			.map(|(i, ty)| (i.lookup(db), *ty))
			.collect::<Vec<_>>();
		params.sort_by_key(|(name, _)| *name);
		params
	}
}

#[salsa::tracked]
fn get_input_interface<'db>(db: &'db dyn Db) -> InputInterface<'db> {
	let mut params = Map::default();
	let mut enums = Set::default();

	// Collect par decls with no RHS
	for (name, pattern) in GlobalScope::variables(db) {
		let item = pattern.item(db);
		match item {
			Item::Declaration(d) => {
				let decl = d.declaration(db);
				if let Some(PatternTy::Variable(ty)) =
					&item.signature(db).patterns.get(&pattern.pattern(db))
					&& decl.definition.is_none()
					&& !ty.contains_error(db)
					&& ty.contains_par(db)
				{
					let _ = params.insert(name, *ty);
				}
			}
			Item::Enumeration(e) if e.enumeration(db).definition.is_none() => {
				let enumeration = e.enumeration(db);
				if let Some(PatternTy::Enum(_)) =
					&item.signature(db).patterns.get(&enumeration.pattern)
				{
					let _ = enums.insert(name);
				}
			}
			_ => (),
		}
	}

	// Remove decls which are later assigned
	for model_file in resolve_includes(db) {
		for item in model_file.hir(db).items(db).iter() {
			match item {
				Item::Assignment(a) => {
					let types = item.signature(db);
					if let Some(p) = types.identifier_resolution.get(&a.assignment(db).assignee) {
						let _ = params.remove(&p.identifier(db).unwrap());
					}
				}
				Item::EnumAssignment(a) => {
					let types = item.signature(db);
					if let Some(p) = types
						.identifier_resolution
						.get(&a.enum_assignment(db).assignee)
					{
						let _ = enums.remove(&p.identifier(db).unwrap());
					}
				}
				_ => (),
			}
		}
	}

	InputInterface { enums, params }
}

/// The output interface of the program
#[derive(Debug, Clone, PartialEq, Eq, salsa::SalsaValue)]
pub struct OutputInterface<'db> {
	/// The variables to be output
	pub variables: Map<Identifier<'db>, Ty<'db>>,
}

impl<'db> OutputInterface<'db> {
	/// Get the output interface
	pub fn get(db: &'db dyn Db) -> &'db Self {
		get_output_interface(db)
	}

	/// Get the output variables sorted by name
	pub fn variables(&self, db: &'db dyn Db) -> Vec<(&'db str, Ty<'db>)> {
		let mut variables = self
			.variables
			.iter()
			.map(|(i, ty)| (i.lookup(db), *ty))
			.collect::<Vec<_>>();
		variables.sort_by_key(|(name, _)| *name);
		variables
	}
}

#[salsa::tracked]
fn get_output_interface<'db>(db: &'db dyn Db) -> OutputInterface<'db> {
	let ids = IdentifierRegistry::lookup(db);
	let mut variables = Map::default();

	for (name, pattern) in GlobalScope::variables(db) {
		let item = pattern.item(db);

		let Item::Declaration(d) = item else {
			continue;
		};

		let decl = d.declaration(db);
		let Some(PatternTy::Variable(ty)) = &item.signature(db).patterns.get(&pattern.pattern(db))
		else {
			continue;
		};

		let mut output =
			decl.definition.is_none() && !ty.contains_error(db) && !ty.contains_par(db);
		for ann in decl.annotations.iter() {
			let Ok(ann_ident) = decl[*ann].try_unwrap_identifier_ref() else {
				continue;
			};
			if *ann_ident == ids.annotations.output {
				output = true;
				break;
			}
			if *ann_ident == ids.annotations.no_output {
				output = false;
				break;
			}
		}
		if output {
			let _ = variables.insert(name, *ty);
		}
	}

	OutputInterface { variables }
}

#[cfg(test)]
mod tests {
	use expect_test::{Expect, expect};
	use salsa::{Setter, attach};
	use shackle_syntax::InputLang;

	use crate::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
		interface::{InputInterface, OutputInterface},
	};

	fn check_input_interface(contents: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, contents.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let input_interface = InputInterface::get(&db);
		attach(&db, || {
			expected.assert_debug_eq(input_interface);
		});
	}

	fn check_output_interface(contents: &str, expected: Expect) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, contents.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let output_interface = OutputInterface::get(&db);
		attach(&db, || {
			expected.assert_debug_eq(output_interface);
		});
	}

	#[test]
	fn test_input_interface() {
		check_input_interface(
			r#"
            var int: x;
            int: y;
            enum Foo;
            enum Bar = {A, B};
        "#,
			expect![[r#"
    InputInterface {
        enums: {
            Identifier(
                "Foo",
            ),
        },
        params: {
            Identifier(
                "y",
            ): int,
        },
    }
"#]],
		);
	}

	#[test]
	fn test_output_interface() {
		check_output_interface(
			r#"
            var int: x;
            var int: y :: no_output;
            var int: z :: output = 3;
            var int: a = 3;
            int: b :: output;
            int: c = 3;
            int: d;
        "#,
			expect![[r#"
    OutputInterface {
        variables: {
            Identifier(
                "b",
            ): int,
            Identifier(
                "x",
            ): var int,
            Identifier(
                "z",
            ): var int,
        },
    }
"#]],
		)
	}
}
