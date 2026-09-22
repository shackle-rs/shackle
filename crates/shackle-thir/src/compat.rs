//! Compatibility layer for printing THIR as MiniZinc compatible with the old compiler.

use std::fmt::Write;

use shackle_hir::{
	Db, Identifier,
	constants::IdentifierRegistry,
	ids::NodeRef,
	input::{resolve_auto_includes, shackle_share_directory},
};
use shackle_utils::{hash::Set, maybe_grow_stack};

use crate::{
	Callable, ConstraintId, DeclarationId, Expression, ExpressionData, FunctionId, Generator,
	ItemId, Marker, Model,
	db::final_thir,
	pretty_print::{
		Printer, print_declaration_id, print_expression, print_function, print_generator,
		print_item,
	},
	transform::{Transform, Transformer},
};

/// Pretty print the final THIR to be compatible with the old MiniZinc compiler
pub struct OldMiniZincPrinter<'db> {
	transforms: &'db [Transform],
	ids: &'db IdentifierRegistry<'db>,
	print_input_files_only: bool,
}

impl<'db> std::fmt::Debug for OldMiniZincPrinter<'db> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("OldMiniZincPrinter").finish()
	}
}

impl<'db> OldMiniZincPrinter<'db> {
	/// Run the printer
	pub fn run(db: &'db dyn Db, print_input_files_only: bool) -> String {
		let transforms = Transformer::get_transforms(db);
		let model = final_thir(db).unwrap();
		Self {
			transforms,
			ids: IdentifierRegistry::lookup(db),
			print_input_files_only,
		}
		.print_model(db, model)
	}
}

impl<'db, T: Marker> Printer<'db, T> for OldMiniZincPrinter<'db> {
	fn print_model(&self, db: &'db dyn Db, model: &Model<'db, T>) -> String {
		let mut buf = String::new();

		let auto_includes = Set::from_iter(resolve_auto_includes(db).iter().copied());
		for item in model.top_level_items() {
			if self.print_input_files_only {
				let excluded = match model.item_origin(item).node() {
					Some(NodeRef::Item(item)) => auto_includes.contains(&item.model_file(db)),
					Some(NodeRef::Entity(entity)) => {
						auto_includes.contains(&entity.item(db).model_file(db))
					}
					Some(NodeRef::Model(m)) => auto_includes.contains(&m),
					None => false,
				};
				if excluded {
					continue;
				}
			}

			match item {
				ItemId::Function(f)
					if model[f].name() == self.ids.functions.default
						|| model[f].name() == self.ids.functions.mzn_element_internal
						|| model[f].name() == self.ids.functions.mzn_slice_internal
						|| model[f].name() == self.ids.functions.mzn_indexed_array =>
				{
					// These signatures aren't accepted by the old compiler, so don't print them
					// Instead alternative implementations are provided in compat.mzn
					continue;
				}
				ItemId::Annotation(a)
					if model[a].name == Some(self.ids.annotations.output)
						|| model[a]
							.origin()
							.node()
							.and_then(|n| n.model_file(db).name(db))
							.map(|n| n.starts_with("stdlib_"))
							.unwrap_or(false) =>
				{
					// These will conflict with stdlib annotations in the old compiler, so don't print them
					continue;
				}
				_ => (),
			}
			writeln!(&mut buf, "{};", print_item(self, db, model, item)).unwrap();
		}
		if model.solve().is_none() {
			writeln!(&mut buf, "solve satisfy;").unwrap();
		}
		if !self.print_input_files_only {
			let compat_path = shackle_share_directory(db)
				.as_ref()
				.expect("Shackle share directory should exist")
				.join("compat.mzn");
			let compat = db
				.file_handler()
				.read_file(&compat_path)
				.unwrap_or_else(|err| panic!("failed to read {}: {err}", compat_path.display()));
			writeln!(&mut buf, "{compat}").unwrap();
		}
		buf
	}

	fn print_constraint(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: ConstraintId<'db, T>,
	) -> String {
		let constraint = &model[idx];
		let mut buf = "constraint ".to_owned();
		for ann in constraint.annotations().iter() {
			if matches!(&**ann, ExpressionData::StringLiteral(_)) {
				// Old compiler only supports single string annotation
				write!(&mut buf, ":: {} ", self.print_expression(db, model, ann)).unwrap();
				break;
			}
		}
		write!(
			&mut buf,
			"{}",
			self.print_expression(db, model, constraint.expression())
		)
		.unwrap();
		buf
	}

	fn print_function(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: FunctionId<'db, T>,
		signature_only: bool,
	) -> String {
		let mut buf = print_function(self, db, model, idx, true);
		if self.transforms.contains(&Transform::Totalise) {
			buf.push_str(" :: promise_total");
		}
		if !signature_only && let Some(body) = model[idx].body() {
			write!(&mut buf, " = {}", self.print_expression(db, model, body)).unwrap();
		}
		buf
	}

	fn print_expression(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		expression: &Expression<'db, T>,
	) -> String {
		if let ExpressionData::Call(c) = &**expression
			&& (c.arguments.len() == 1 || c.arguments.len() == 2)
			&& let Callable::Function(f) = &c.function
			&& (self.print_input_files_only || model[*f].body().is_none())
		{
			let name = model[*f].name().as_identifier(db).lookup(db);
			if c.arguments.len() == 1 {
				if matches!(name, "o.." | "o<.." | "o..<" | "o<..<") {
					return format!(
						"({}({}))",
						&name[1..],
						self.print_expression(db, model, &c.arguments[0])
					);
				}
				if shackle_syntax::is_prefix_operator(name) {
					return format!(
						"({}({}))",
						name,
						self.print_expression(db, model, &c.arguments[0])
					);
				}
				if name.len() > 1 && shackle_syntax::is_postfix_operator(&name[..name.len() - 1]) {
					return format!(
						"(({}){})",
						self.print_expression(db, model, &c.arguments[0]),
						&name[..name.len() - 1],
					);
				};
				if (model[*f].name() == self.ids.functions.forall
					|| model[*f].name() == self.ids.functions.exists)
					&& c.arguments.len() == 1
					&& let ExpressionData::ArrayLiteral(al) = &*c.arguments[0]
					&& !al.is_empty()
				{
					// Sometimes the old compiler doesn't make these short-circuit (e.g. inside flat_cv_exp), so turn into and/or
					return al
						.iter()
						.map(|e| format!("({})", self.print_expression(db, model, e)))
						.collect::<Vec<_>>()
						.join(if model[*f].name() == self.ids.functions.forall {
							" /\\ "
						} else {
							" \\/ "
						});
				}
			} else if shackle_syntax::is_infix_operator(name) {
				return format!(
					"(({}) {} ({}))",
					self.print_expression(db, model, &c.arguments[0]),
					name,
					self.print_expression(db, model, &c.arguments[1])
				);
			}
			if self.print_input_files_only && model[*f].name() == self.ids.functions.array_access {
				if let ExpressionData::TupleLiteral(tl) = &*c.arguments[1] {
					if tl.len() == 1 {
						return format!(
							"({}[{}])",
							self.print_expression(db, model, &c.arguments[0]),
							self.print_expression(db, model, &tl[0])
						);
					}
					return format!(
						"({}[{}])",
						self.print_expression(db, model, &c.arguments[0]),
						tl.iter()
							.map(|e| self.print_expression(db, model, e))
							.collect::<Vec<_>>()
							.join(", ")
					);
				}
				return format!(
					"({}[{}])",
					self.print_expression(db, model, &c.arguments[0]),
					self.print_expression(db, model, &c.arguments[1])
				);
			}
		}

		maybe_grow_stack(|| print_expression(self, db, model, expression))
	}

	fn print_generator(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		g: &Generator<'db, T>,
	) -> String {
		if let Generator::Assignment {
			assignment,
			where_clause,
		} = g
		{
			let decl = &model[*assignment];
			return format!(
				"{} = {}{}",
				self.print_declaration_id(db, model, *assignment),
				self.print_expression(db, model, decl.definition().unwrap()),
				if let Some(where_clause) = where_clause {
					format!(" where {}", self.print_expression(db, model, where_clause))
				} else {
					"".to_owned()
				}
			);
		}
		print_generator(self, db, model, g)
	}

	fn print_declaration_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		d: DeclarationId<'db, T>,
	) -> String {
		if model[d].top_level()
			&& let Some(name) = model[d].name()
			&& (name == self.ids.names.objective
				|| model[d]
					.origin()
					.node()
					.and_then(|n| n.model_file(db).name(db))
					.map(|n| n.starts_with("stdlib_"))
					.unwrap_or(false))
		{
			return Identifier::new(db, format!("shackle_{}", name.lookup(db))).pretty_print(db);
		}
		print_declaration_id(self, db, model, d)
	}

	fn print_function_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		f: FunctionId<'db, T>,
	) -> String {
		let name = if let Some(tys) = model[f].mangled_param_tys()
			&& model[f].body().is_some()
		{
			model[f].name().mangled(db, tys.iter().copied())
		} else {
			model[f].name().as_identifier(db)
		};

		if !self.print_input_files_only && model[f].body().is_some() {
			// Prefix to avoid conflicts with stdlib functions in the old compiler
			return Identifier::new(db, format!("shackle_{}", name.lookup(db))).pretty_print(db);
		}

		name.pretty_print(db)
	}
}
