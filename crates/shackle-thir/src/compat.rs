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

	/// Return the length and, when explicitly specified, the printed members of
	/// an index array produced while lowering a 2-D literal.
	fn print_2d_literal_dimension<T: Marker>(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		expression: &Expression<'db, T>,
	) -> Option<(usize, Option<Vec<String>>)> {
		if let ExpressionData::ArrayLiteral(items) = &**expression {
			return Some((
				items.len(),
				Some(
					items
						.iter()
						.map(|item| self.print_expression(db, model, item))
						.collect(),
				),
			));
		}

		// A non-indexed dimension is lowered to `set2array(1..n)` (or
		// `set2array({})`).  It has no index prefix in the 2-D literal syntax.
		let ExpressionData::Call(set2array) = &**expression else {
			return None;
		};
		let Callable::Function(set2array_fn) = &set2array.function else {
			return None;
		};
		if model[*set2array_fn].name() != self.ids.functions.set2array
			|| set2array.arguments.len() != 1
		{
			return None;
		}
		match &*set2array.arguments[0] {
			ExpressionData::SetLiteral(items) if items.is_empty() => Some((0, None)),
			ExpressionData::Call(range) => {
				let Callable::Function(range_fn) = &range.function else {
					return None;
				};
				if model[*range_fn].name() != self.ids.functions.dot_dot
					|| range.arguments.len() != 2
				{
					return None;
				}
				let ExpressionData::IntegerLiteral(start) = &*range.arguments[0] else {
					return None;
				};
				let ExpressionData::IntegerLiteral(end) = &*range.arguments[1] else {
					return None;
				};
				(start.0 == 1 && end.0 >= 1).then_some((end.0 as usize, None))
			}
			_ => None,
		}
	}

	/// Reconstruct MiniZinc syntax for calls synthesized by HIR-to-THIR
	/// lowering.  This is only used when printing input files without the
	/// Shackle compatibility library.
	fn print_input_only_synthesized_call<T: Marker>(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		call: &crate::Call<'db, T>,
	) -> Option<String> {
		let Callable::Function(function) = &call.function else {
			return None;
		};
		let name = model[*function].name();

		if name == self.ids.functions.mzn_array_2d_literal && call.arguments.len() == 3 {
			let (row_count, row_indices) =
				self.print_2d_literal_dimension(db, model, &call.arguments[0])?;
			let (column_count, column_indices) =
				self.print_2d_literal_dimension(db, model, &call.arguments[1])?;
			let ExpressionData::ArrayLiteral(values) = &*call.arguments[2] else {
				return None;
			};
			if values.len() != row_count.checked_mul(column_count)? {
				return None;
			}
			if column_count == 0 {
				return (row_count == 0).then_some("[||]".to_owned());
			}

			let mut rows = Vec::with_capacity(row_count + usize::from(column_indices.is_some()));
			if let Some(indices) = column_indices {
				rows.push(format!("{}:", indices.join(": ")));
			}
			for (row, values) in values.chunks(column_count).enumerate() {
				let prefix = row_indices
					.as_ref()
					.map(|indices| format!("{}: ", indices[row]))
					.unwrap_or_default();
				rows.push(format!(
					"{}{}",
					prefix,
					values
						.iter()
						.map(|value| self.print_expression(db, model, value))
						.collect::<Vec<_>>()
						.join(", ")
				));
			}
			return Some(format!("[| {} |]", rows.join(" | ")));
		}

		if name == self.ids.functions.mzn_start_indexed_array && call.arguments.len() == 2 {
			let ExpressionData::ArrayLiteral(values) = &*call.arguments[1] else {
				return None;
			};
			if values.is_empty() {
				return None;
			}
			return Some(format!(
				"[{}: {}]",
				self.print_expression(db, model, &call.arguments[0]),
				values
					.iter()
					.map(|value| self.print_expression(db, model, value))
					.collect::<Vec<_>>()
					.join(", ")
			));
		}

		if name == self.ids.functions.mzn_indexed_array && call.arguments.len() == 1 {
			let ExpressionData::ArrayLiteral(members) = &*call.arguments[0] else {
				return None;
			};
			let members = members
				.iter()
				.map(|member| {
					let ExpressionData::TupleLiteral(member) = &**member else {
						return None;
					};
					(member.len() == 2).then(|| {
						format!(
							"{}: {}",
							self.print_expression(db, model, &member[0]),
							self.print_expression(db, model, &member[1]),
						)
					})
				})
				.collect::<Option<Vec<_>>>()?;
			return Some(format!("[{}]", members.join(", ")));
		}

		// Slice lowering wraps mzn_slice in array<N>d to restore the output
		// dimensions.  The original access syntax already has those dimensions,
		// so print the entire pattern as one access.
		let function_name = name.as_identifier(db).lookup(db);
		if function_name
			.strip_prefix("array")
			.and_then(|rank| rank.strip_suffix('d'))
			.and_then(|rank| rank.parse::<usize>().ok())
			.is_some_and(|rank| rank == call.arguments.len() - 1)
			&& let Some(slice) = call.arguments.last()
			&& let ExpressionData::Call(slice) = &**slice
			&& let Callable::Function(slice_fn) = &slice.function
			&& model[*slice_fn].name() == self.ids.functions.mzn_slice
			&& slice.arguments.len() == 2
			&& let ExpressionData::TupleLiteral(indices) = &*slice.arguments[1]
		{
			let restored_indices = indices
				.iter()
				.map(|index| {
					let was_slice = call.arguments[..call.arguments.len() - 1]
						.iter()
						.any(|argument| argument == index);
					if was_slice {
						Some(self.print_expression(db, model, index))
					} else if let ExpressionData::SetLiteral(items) = &**index
						&& items.len() == 1
					{
						Some(self.print_expression(db, model, &items[0]))
					} else {
						None
					}
				})
				.collect::<Option<Vec<_>>>()?;
			return Some(format!(
				"({}[{}])",
				self.print_expression(db, model, &slice.arguments[0]),
				restored_indices.join(", ")
			));
		}

		if name == self.ids.functions.mzn_slice
			&& call.arguments.len() == 2
			&& let ExpressionData::TupleLiteral(indices) = &*call.arguments[1]
		{
			return Some(format!(
				"({}[{}])",
				self.print_expression(db, model, &call.arguments[0]),
				indices
					.iter()
					.map(|index| self.print_expression(db, model, index))
					.collect::<Vec<_>>()
					.join(", ")
			));
		}

		None
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
		if self.print_input_files_only
			&& let ExpressionData::Call(call) = &**expression
			&& let Some(printed) = self.print_input_only_synthesized_call(db, model, call)
		{
			return printed;
		}
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

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use salsa::Setter;
	use shackle_hir::{
		CompilerDatabase,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use super::OldMiniZincPrinter;
	use crate::transform::Transformer;

	#[test]
	fn print_input_files_reconstructs_lowered_array_syntax() {
		let mut db = CompilerDatabase::default();
		let shackle_stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("../..")
			.join("share/minizinc");
		let _ = CompilerSettings::get(&db)
			.set_stdlib_directory(&mut db)
			.to(Some(shackle_stdlib));
		Transformer::set_transforms(&mut db, []);
		let source = r#"
            array [int, int] of int: matrix = [| 1, 2 | 3, 4 |];
            array [int] of int: offset = [3: 10, 20];
            array [int] of int: sparse = [1: 10, 3: 20];
			array [int, int] of int: sliced = matrix[1..2, 2..2];
        "#;
		let input = InlineModelFile::new(&db, source.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![input]);

		let printed = OldMiniZincPrinter::run(&db, true);

		assert!(printed.contains("[| 1, 2 | 3, 4 |]"), "{printed}");
		assert!(printed.contains("[3: 10, 20]"), "{printed}");
		assert!(printed.contains("[1: 10, 3: 20]"), "{printed}");
		assert!(printed.contains("matrix["), "{printed}");
		for helper in [
			"mzn_array_2d_literal",
			"mzn_start_indexed_array",
			"mzn_indexed_array",
			"mzn_slice",
		] {
			assert!(
				!printed.contains(helper),
				"{helper} remained in:\n{printed}"
			);
		}
	}
}
