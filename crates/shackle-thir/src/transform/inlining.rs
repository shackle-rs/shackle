//! Performs inlining of `:: mzn_inline_call_by_name` functions.
//! - Replaces calls with the bodies of functions
//! - Replaces references to parameters with the corresponding call argument
//!
//! [`inline_user_functions`] instead inlines a given set of functions by
//! name (e.g. the user's predicates, for analysis).

use rustc_hash::{FxHashMap, FxHashSet};
use shackle_diagnostics::Result;
use shackle_hir::constants::IdentifierRegistry;
use shackle_utils::maybe_grow_stack;

use crate::{
	AnnotationId, Callable, Db, Declaration, DeclarationId, Domain, EnumerationId, Expression,
	ExpressionData, FunctionId, Identifier, Item, Let, LetItem, Marker, Model, ResolvedIdentifier,
	pretty_print::PrettyPrinter,
	traverse::{
		Folder, ReplacementMap, Visitor, add_function, fold_declaration, fold_expression,
		fold_function_body, fold_function_id, visit_annotation, visit_declaration,
		visit_enumeration,
	},
};

struct Inliner<'db, Dst: Marker, Src: Marker = ()> {
	model: Model<'db, Dst>,
	replacement_map: ReplacementMap<'db, Dst, Src>,
	ids: &'db IdentifierRegistry<'db>,
	map: FxHashMap<DeclarationId<'db, Src>, Expression<'db, Dst>>,
	depth: u16,
	mode: Mode<'db, Src>,
	/// Functions being inlined (to leave recursive calls alone)
	inlining: Vec<FunctionId<'db, Src>>,
	/// Names that a local would capture, so it is renamed: the top-level,
	/// enum and annotation names, and the free names in the arguments of the
	/// calls being inlined by name
	captured: Vec<Identifier<'db>>,
	/// Names that a renamed local must not take: all names outside the
	/// library functions. Locals with the same name get the same new name,
	/// so they shadow each other in the same way.
	used: FxHashSet<Identifier<'db>>,
	/// Whether a library function (not one of the inlined functions) is being
	/// folded: its parameters and locals keep their names
	in_library: bool,
}

/// The declarations that a part of a model declares and refers to, and the
/// names of the enums, enum members and annotations it defines or refers to
/// (function items are skipped)
struct Names<'db, T: Marker> {
	declared: Vec<DeclarationId<'db, T>>,
	referenced: Vec<DeclarationId<'db, T>>,
	/// Enums, enum members and annotations that are defined
	defined: Vec<Identifier<'db>>,
	/// Enums, enum members and annotations that are referred to
	atoms: Vec<Identifier<'db>>,
}

impl<'db, T: Marker> Names<'db, T> {
	fn new() -> Self {
		Names {
			declared: Vec::new(),
			referenced: Vec::new(),
			defined: Vec::new(),
			atoms: Vec::new(),
		}
	}

	/// All these names
	fn all(&self, model: &Model<'db, T>) -> impl Iterator<Item = Identifier<'db>> {
		self.declared
			.iter()
			.chain(&self.referenced)
			.filter_map(|d| model[*d].name())
			.chain(self.defined.iter().copied())
			.chain(self.atoms.iter().copied())
	}

	/// The names that are not declared here
	fn free(&self, model: &Model<'db, T>) -> Vec<Identifier<'db>> {
		self.referenced
			.iter()
			.filter(|d| !self.declared.contains(d))
			.filter_map(|d| model[*d].name())
			.chain(self.atoms.iter().copied())
			.collect()
	}
}

impl<'a, 'db, T: Marker> Visitor<'a, 'db, T> for Names<'db, T> {
	fn visit_function(&mut self, _model: &'a Model<'db, T>, _function: FunctionId<'db, T>) {}

	fn visit_enumeration(&mut self, model: &'a Model<'db, T>, enumeration: EnumerationId<'db, T>) {
		let e = &model[enumeration];
		self.defined.push(Identifier(e.enum_type().name()));
		self.defined
			.extend(e.definition().into_iter().flatten().filter_map(|c| c.name));
		visit_enumeration(self, model, enumeration);
	}

	fn visit_annotation(&mut self, model: &'a Model<'db, T>, annotation: AnnotationId<'db, T>) {
		self.defined.extend(model[annotation].name);
		visit_annotation(self, model, annotation);
	}

	fn visit_declaration(&mut self, model: &'a Model<'db, T>, declaration: DeclarationId<'db, T>) {
		self.declared.push(declaration);
		visit_declaration(self, model, declaration);
	}

	fn visit_identifier(
		&mut self,
		model: &'a Model<'db, T>,
		identifier: &'a ResolvedIdentifier<'db, T>,
	) {
		match identifier {
			ResolvedIdentifier::Declaration(d) => self.referenced.push(*d),
			ResolvedIdentifier::Annotation(a) => self.atoms.extend(model[*a].name),
			ResolvedIdentifier::EnumerationMember(m) => self.atoms.extend(model[*m].name),
			ResolvedIdentifier::Enumeration(e) => {
				self.atoms.push(Identifier(model[*e].enum_type().name()));
			}
		}
	}
}

/// Which functions are inlined
enum Mode<'db, Src: Marker> {
	/// Functions annotated with `mzn_inline_call_by_name` (by name), and with
	/// `mzn_inline` or a macro body (by value). Their items are removed.
	Annotated,
	/// Only these functions (by name). Their items are kept.
	Only(FxHashSet<FunctionId<'db, Src>>),
}

impl<'db, Dst: Marker, Src: Marker> Inliner<'db, Dst, Src> {
	fn new(db: &'db dyn Db, model: &Model<'db, Src>, mode: Mode<'db, Src>) -> Self {
		// Inlining of annotated functions removes the names of locals instead
		let (used, captured) = match &mode {
			Mode::Only(functions) => {
				let mut names = Names::new();
				for f in functions {
					for p in model[*f].parameters() {
						names.visit_declaration(model, *p);
					}
					if let Some(body) = model[*f].body() {
						names.visit_expression(model, body);
					}
				}
				names.visit_model(model);
				let captured = model
					.top_level_declarations()
					.filter_map(|(_, d)| d.name())
					.chain(names.defined.iter().copied())
					.collect();
				(names.all(model).collect(), captured)
			}
			Mode::Annotated => Default::default(),
		};
		Inliner {
			replacement_map: ReplacementMap::default(),
			model: Model::with_capacities(&model.item_counts()),
			ids: IdentifierRegistry::lookup(db),
			map: FxHashMap::default(),
			depth: 0,
			mode,
			inlining: Vec::new(),
			captured,
			used,
			in_library: false,
		}
	}

	fn annotated(
		&self,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
		ann: Identifier<'db>,
	) -> bool {
		model[f].annotations().has(model, ann)
	}

	/// Whether the function item is removed because all calls are inlined
	fn removed(&self, model: &Model<'db, Src>, f: FunctionId<'db, Src>) -> bool {
		matches!(self.mode, Mode::Annotated)
			&& (self.annotated(model, f, self.ids.annotations.mzn_inline_call_by_name)
				|| self.annotated(model, f, self.ids.annotations.mzn_inline))
	}

	/// Whether calls to `f` (with the given body) are inlined by value
	fn by_value(
		&self,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
		body: &Expression<'db, Src>,
	) -> bool {
		matches!(self.mode, Mode::Annotated)
			&& (self.annotated(model, f, self.ids.annotations.mzn_inline) || is_macro_call(body))
	}

	/// Run `fold` on (a part of) the function `f`, keeping the names of its
	/// locals if it is a library function
	fn in_function(&mut self, f: FunctionId<'db, Src>, fold: impl FnOnce(&mut Self)) {
		let library = matches!(&self.mode, Mode::Only(functions) if !functions.contains(&f));
		let outer = std::mem::replace(&mut self.in_library, library);
		fold(self);
		self.in_library = outer;
	}

	/// Whether calls to `f` are inlined by name
	fn by_name(&self, model: &Model<'db, Src>, f: FunctionId<'db, Src>) -> bool {
		match &self.mode {
			Mode::Annotated => {
				self.annotated(model, f, self.ids.annotations.mzn_inline_call_by_name)
			}
			Mode::Only(only) => only.contains(&f) && !self.inlining.contains(&f),
		}
	}
}

impl<'db, Dst: Marker, Src: Marker> Folder<'_, 'db, Dst, Src> for Inliner<'db, Dst, Src> {
	fn model(&mut self) -> &mut Model<'db, Dst> {
		&mut self.model
	}

	fn replacement_map(&mut self) -> &mut ReplacementMap<'db, Dst, Src> {
		&mut self.replacement_map
	}

	fn add_function(&mut self, db: &'db dyn Db, model: &Model<'db, Src>, f: FunctionId<'db, Src>) {
		if self.removed(model, f) {
			// Remove inlined function
			return;
		}
		// The parameters of a library function keep their names too
		self.in_function(f, |inliner| {
			if model[f].top_level() {
				let _ = fold_function_id(inliner, db, model, f);
			} else {
				let _ = add_function(inliner, db, model, f);
			}
		});
	}

	fn fold_function_body(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		f: FunctionId<'db, Src>,
	) {
		if self.removed(model, f) {
			// Remove inlined function
			return;
		}
		self.in_function(f, |inliner| fold_function_body(inliner, db, model, f));
	}

	fn fold_declaration(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		d: &Declaration<'db, Src>,
	) -> Declaration<'db, Dst> {
		let mut folded = fold_declaration(self, db, model, d);
		if !self.map.is_empty() && matches!(self.mode, Mode::Annotated) {
			// Alpha rename for safety when inlining
			folded.remove_name();
		}
		// Rename locals that would capture or clash with a name (see `captured`)
		if let Some(name) = folded.name()
			&& !folded.top_level()
			&& !self.in_library
			&& self.captured.contains(&name)
		{
			let fresh = (1..)
				.map(|n| Identifier::new(db, format!("{}{n}", name.lookup(db))))
				.find(|n| !self.captured.contains(n) && !self.used.contains(n))
				.unwrap();
			folded.set_name(fresh);
		}
		folded
	}

	fn fold_expression(
		&mut self,
		db: &'db dyn Db,
		model: &Model<'db, Src>,
		expression: &Expression<'db, Src>,
	) -> Expression<'db, Dst> {
		maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::Identifier(ResolvedIdentifier::Declaration(d)) => {
					if let Some(e) = self.map.get(d) {
						return e.clone();
					}
				}
				ExpressionData::Call(c) => {
					if let Callable::Function(f) = &c.function
						&& let Some(body) = model[*f].body()
					{
						if self.by_value(model, *f, body) {
							log::debug!(
								"Inlining {} at {} using call by value semantics",
								PrettyPrinter::new(db, model).pretty_print_signature((*f).into()),
								expression.origin().pretty_print(db)
							);

							let old_depth = self.depth;
							self.depth += 1;

							assert!(
								self.depth < 100,
								"Exceeded maximum inlining depth, likely due to infinite recursion"
							);

							let mut restore = Vec::with_capacity(c.arguments.len());
							let items = c
								.arguments
								.iter()
								.zip(model[*f].parameters().iter())
								.filter_map(|(arg, param)| {
									let origin = arg.origin();
									let folded = self.fold_expression(db, model, arg);
									let param_ty = model[*param].ty();
									if matches!(&*folded, ExpressionData::Identifier(_))
										&& folded.ty() == param_ty
									{
										restore.push(self.map.insert(*param, folded));
										None
									} else {
										let mut decl = Declaration::new(
											false,
											Domain::unbounded(db, origin, param_ty),
										);
										decl.set_definition(folded);
										let idx =
											self.model.add_declaration(Item::new(decl, origin));
										let ident = Expression::new(db, &self.model, origin, idx);
										restore.push(self.map.insert(*param, ident));
										Some(LetItem::Declaration(idx))
									}
								})
								.collect::<Vec<_>>();
							let inlined = self.fold_expression(db, model, body);
							for (param, prev) in model[*f].parameters().iter().zip(restore) {
								if let Some(prev) = prev {
									let _ = self.map.insert(*param, prev);
								} else {
									let _ = self.map.remove(param);
								}
							}

							self.depth = old_depth;
							return if items.is_empty() {
								inlined
							} else {
								Expression::new(
									db,
									&self.model,
									expression.origin(),
									Let {
										items,
										in_expression: Box::new(inlined),
									},
								)
							};
						}
						if self.by_name(model, *f) {
							log::debug!(
								"Inlining {} at {} using call by name semantics",
								PrettyPrinter::new(db, model).pretty_print_signature((*f).into()),
								expression.origin().pretty_print(db)
							);

							let captured = self.captured.len();
							let mut restore = Vec::with_capacity(c.arguments.len());
							for (param, arg) in
								model[*f].parameters().iter().zip(c.arguments.iter())
							{
								let e = self.fold_expression(db, model, arg);
								// Annotated inlining removes the names of locals
								if matches!(self.mode, Mode::Only(_)) {
									let mut names = Names::new();
									names.visit_expression(&self.model, &e);
									self.captured.extend(names.free(&self.model));
								}
								restore.push(self.map.insert(*param, e));
							}
							// After the arguments, so `f(f(x))` inlines both calls
							self.inlining.push(*f);
							let old_depth = self.depth;
							self.depth += 1;

							assert!(
								self.depth < 100,
								"Exceeded maximum inlining depth, likely due to infinite recursion"
							);
							let mut inlined = self.fold_expression(db, model, body);
							inlined.set_origin(expression.origin());
							self.captured.truncate(captured);
							let _ = self.inlining.pop();
							for (param, prev) in model[*f].parameters().iter().zip(restore) {
								if let Some(prev) = prev {
									let _ = self.map.insert(*param, prev);
								} else {
									let _ = self.map.remove(param);
								}
							}

							self.depth = old_depth;

							return inlined;
						}
					}
				}
				_ => (),
			}
			fold_expression(self, db, model, expression)
		})
	}
}

fn is_macro_call<T: Marker>(expression: &Expression<T>) -> bool {
	let ExpressionData::Call(c) = &**expression else {
		return false;
	};
	c.arguments
		.iter()
		.all(|arg| matches!(&**arg, ExpressionData::Identifier(_)))
}

/// Perform inlining to implement call-by-name semantics for functions annotated with
/// `:: mzn_inline_call_by_name`.
pub fn inline_functions<'db>(db: &'db dyn Db, model: Model<'db>) -> Result<Model<'db>> {
	log::info!("Inlining functions");
	let mut inliner = Inliner::new(db, &model, Mode::Annotated);
	inliner.add_model(db, &model);
	Ok(inliner.model)
}

/// Inline all calls to `functions` (which must have bodies) using call by
/// name semantics. Recursive calls are left as calls, and the function items
/// are kept. Locals outside the library functions that have a top-level, enum
/// or annotation name, or that would capture a name in an inlined argument,
/// are renamed, so the model still means the same when it is printed.
pub fn inline_user_functions<'db>(
	db: &'db dyn Db,
	model: Model<'db>,
	functions: FxHashSet<FunctionId<'db>>,
) -> Model<'db> {
	let mut inliner = Inliner::new(db, &model, Mode::Only(functions));
	inliner.add_model(db, &model);
	inliner.model
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::transform::{Transform, tests::check_no_stdlib};

	#[test]
	fn test_inline_call_by_name() {
		check_no_stdlib(
			Transform::InlineFunctions,
			r#"
				annotation mzn_inline_call_by_name;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline_call_by_name =
                    if a then b else c endif;
                any: x = foo(true, 1, 2);
			"#,
			expect!([r#"
    annotation mzn_inline_call_by_name;
    int: x = if true then 1 else 2 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_inline_alpha_rename() {
		check_no_stdlib(
			Transform::InlineFunctions,
			r#"
				annotation mzn_inline_call_by_name;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline_call_by_name =
                    let {
                        float: p = 2.5;
                    } in if a then b else c endif;
                int: p = 1;
                any: x = foo(true, p, 2);
			"#,
			expect!([r#"
    annotation mzn_inline_call_by_name;
    int: p = 1;
    int: x = let {
      float: _DECL_2 = 2.5;
    } in if true then p else 2 endif;
    solve satisfy;
"#]),
		)
	}

	#[test]
	fn test_inline_call_by_value() {
		check_no_stdlib(
			Transform::InlineFunctions,
			r#"
				annotation mzn_inline;
                function int: foo(bool: a, int: b, int: c) :: mzn_inline =
                    if a then bar(b, b) else bar(b, c) endif;
				function int: bar(int: x, int: y);
                any: x = foo(true, 1, 2);
			"#,
			expect!([r#"
    annotation mzn_inline;
    function int: bar(int: x, int: y);
    int: x = let {
      bool: _DECL_3 = true;
      int: _DECL_4 = 1;
      int: _DECL_5 = 2;
    } in if _DECL_3 then bar(_DECL_4, _DECL_4) else bar(_DECL_4, _DECL_5) endif;
    solve satisfy;
"#]),
		)
	}
}
