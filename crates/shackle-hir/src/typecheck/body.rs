use shackle_diagnostics::{Error, Warning};
use shackle_ty::{Ty, registry::TypeRegistry};
use shackle_utils::arena::ArenaMap;
/// Types of 'bodies': everything that isn't part of a signature.
///
/// E.g.
/// - Annotations on items
/// - RHS of variable declarations
/// - Bodies of functions
use shackle_utils::hash::Map;

use crate::{
	Db, Expression, ExpressionId, Item, Pattern, PatternId, PatternTy, Type, TypeContext, TypeId,
	Typer, constants::IdentifierRegistry, diagnostics::Diagnostics, ids::PatternRef,
};

/// Collected types for an item body
#[derive(Clone, Debug, PartialEq, Eq, salsa::Update, Default)]
pub struct BodyTypes<'db> {
	/// Types of declarations
	pub patterns: ArenaMap<Pattern<'db>, PatternTy<'db>>,
	/// Types of expressions
	pub expressions: ArenaMap<Expression<'db>, Ty<'db>>,
	/// Identifier resolution
	pub identifier_resolution: Map<ExpressionId<'db>, PatternRef<'db>>,
	/// Pattern resolution
	pub pattern_resolution: Map<PatternId<'db>, PatternRef<'db>>,
	/// Types computed for declared types
	pub types: ArenaMap<Type<'db>, Ty<'db>>,
}

#[derive(Clone, Debug, PartialEq, Eq, salsa::Update)]
struct BodyTypesResult<'db> {
	/// The body types for this item
	body_types: BodyTypes<'db>,
	/// Errors produced during body typechecking.
	///
	/// We can't directly accumulate these since the type checker uses possibly cyclic queries
	errors: Diagnostics,
}

impl<'db> Item<'db> {
	/// Get the body types for this item
	pub fn body_types(&self, db: &'db dyn Db) -> &'db BodyTypes<'db> {
		&item_body_types(db, *self).body_types
	}
}

/// Get body types without accumulating errors
#[salsa::tracked(returns(ref))]
fn item_body_types<'db>(db: &'db dyn Db, item: Item<'db>) -> BodyTypesResult<'db> {
	let mut ctx = BodyTypeContext::new(item);
	ctx.type_item(db);
	ctx.finish()
}

/// Accumulate body typechecking diagnostics for this item
#[salsa::tracked]
pub(super) fn accumulate_item_body_diagnostics<'db>(db: &'db dyn Db, item: Item<'db>) {
	item_body_types(db, item).errors.accumulate(db);
}

/// Context for typing an item body
#[derive(Debug)]
pub struct BodyTypeContext<'db> {
	item: Item<'db>,
	data: BodyTypes<'db>,
	diagnostics: Diagnostics,
}

impl<'db> BodyTypeContext<'db> {
	/// Create a new signature type context
	pub fn new(item: impl Into<Item<'db>>) -> Self {
		let item = item.into();
		Self {
			item,
			data: BodyTypes::default(),
			diagnostics: Diagnostics::default(),
		}
	}

	/// Create a new signature type context with the given capacity for patterns/expressions
	pub fn with_capacity(item: impl Into<Item<'db>>, patterns: u32, expressions: u32) -> Self {
		let item = item.into();
		Self {
			item,
			data: BodyTypes {
				patterns: ArenaMap::with_capacity(patterns + 1),
				expressions: ArenaMap::with_capacity(expressions + 1),
				identifier_resolution: Map::default(),
				pattern_resolution: Map::default(),
				types: ArenaMap::default(),
			},
			diagnostics: Diagnostics::default(),
		}
	}

	/// Compute the type of the body of this item
	pub fn type_item(&mut self, db: &'db dyn Db) {
		let item = self.item;
		let data = item.data(db);
		let mut typer = Typer::new(db, self, item, data);
		let types = TypeRegistry::lookup(db);
		match item {
			Item::Annotation(_) => {}
			Item::Function(f) => {
				let it = f.function(db);
				let signature = item.signature(db);
				for ann in it.annotations.iter() {
					let _ = typer.typecheck_expression(*ann, types.ann);
				}
				for param in it.parameters.iter() {
					if let Some(p) = param.pattern {
						let param_ty = match &signature.patterns[&p] {
							PatternTy::Argument(t) | PatternTy::Destructuring(t) => *t,
							_ => unreachable!(),
						};
						for ann in param.annotations.iter() {
							typer.typecheck_declaration_annotation(*ann, param_ty);
						}
					}
				}

				if let Some(e) = it.body {
					match &signature.patterns[&it.pattern] {
						PatternTy::Function(function) => {
							let _ = typer.typecheck_expression(e, function.overload.return_type());
						}
						_ => unreachable!(),
					};
				}
			}
			Item::Declaration(d) => {
				let it = d.declaration(db);
				let signature = item.signature(db);
				let lhs_ty = match &signature.patterns[&it.pattern] {
					PatternTy::Variable(t) | PatternTy::Destructuring(t) => *t,
					_ => unreachable!(),
				};
				// An object introduction is defined by the record its objects are
				// constructed from, not by the class type itself
				let expected = if data[it.declared_type].is_new(data) {
					typer
						.class_type_to_input_record_type(it.declared_type)
						.unwrap_or(lhs_ty)
				} else {
					lhs_ty
				};
				// Declarations with incomplete types would have been done during signature typing
				if data[it.declared_type].is_complete(data) {
					if let Some(e) = it.definition {
						let ids = IdentifierRegistry::lookup(db);
						let output_only = it.annotations.iter().any(|ann| match &data[*ann] {
							Expression::Identifier(i) => *i == ids.annotations.output_only,
							_ => false,
						});
						if output_only {
							typer.typecheck_output(e, expected);
						} else {
							let _ = typer.typecheck_expression(e, expected);
						}
					}
					for ann in it.annotations.iter() {
						typer.typecheck_declaration_annotation(*ann, expected);
					}
				}
			}
			Item::Output(o) => {
				let it = o.output(db);
				if let Some(s) = &it.section {
					let _ = typer.typecheck_expression(*s, types.string);
				}
				typer.typecheck_output(it.expression, types.array_of_string);
			}
			Item::Constraint(c) => {
				let it = c.constraint(db);
				let _ = typer.typecheck_expression(it.expression, types.var_bool);
				for ann in it.annotations.iter() {
					let _ = typer.typecheck_expression(*ann, types.ann);
				}
			}
			Item::Solve(s) => {
				let it = s.solve(db);
				for ann in it.annotations.iter() {
					let _ = typer.typecheck_expression(*ann, types.ann);
				}
			}
			Item::Assignment(a) => {
				let it = a.assignment(db);
				let expected = typer.collect_expression(it.assignee);
				let _ = typer.typecheck_expression(it.definition, expected);
			}
			Item::Enumeration(e) => {
				let it = e.enumeration(db);
				let signature = item.signature(db);
				let ty = match &signature.patterns[&it.pattern] {
					PatternTy::Enum(t) => *t,
					_ => unreachable!(),
				};
				for ann in it.annotations.iter() {
					typer.typecheck_declaration_annotation(*ann, ty);
				}
			}
			Item::EnumAssignment(e) => {
				let it = e.enum_assignment(db);
				let _ = typer.collect_expression(it.assignee);
			}
			Item::TypeAlias(t) => {
				let it = t.type_alias(db);
				for ann in it.annotations.iter() {
					let _ = typer.typecheck_expression(*ann, types.ann);
				}
			}
			// A class has no body: its attributes, constraints and annotations are
			// all typed as part of its signature.
			Item::Class(_) => {}
		}
	}

	/// Get results of typing
	fn finish(mut self) -> BodyTypesResult<'db> {
		self.data.patterns.shrink_to_fit();
		self.data.expressions.shrink_to_fit();
		self.data.identifier_resolution.shrink_to_fit();
		self.data.pattern_resolution.shrink_to_fit();
		BodyTypesResult {
			body_types: self.data,
			errors: self.diagnostics,
		}
	}
}

impl<'db> TypeContext<'db> for BodyTypeContext<'db> {
	fn add_declaration(
		&mut self,
		_db: &'db dyn Db,
		pattern: PatternId<'db>,
		declaration: PatternTy<'db>,
	) {
		assert!(
			matches!(
				self.data.patterns.get(pattern),
				None | Some(PatternTy::Computing)
			),
			"Tried to add declaration for {:?} twice",
			pattern
		);
		self.data.patterns.insert(pattern, declaration);
	}

	fn add_expression(&mut self, _db: &'db dyn Db, expression: ExpressionId<'db>, ty: Ty<'db>) {
		assert!(
			self.data.expressions.get(expression).is_none(),
			"Tried to add type for expression {:?} twice",
			expression
		);
		self.data.expressions.insert(expression, ty);
	}

	fn add_identifier_resolution(
		&mut self,
		_db: &'db dyn Db,
		expression: ExpressionId<'db>,
		resolution: PatternRef<'db>,
	) {
		let old = self
			.data
			.identifier_resolution
			.insert(expression, resolution);
		assert!(
			old.is_none(),
			"Tried to add identifier resolution for {:?} twice",
			expression
		);
	}

	fn add_pattern_resolution(
		&mut self,
		_db: &'db dyn Db,
		pattern: PatternId<'db>,
		resolution: PatternRef<'db>,
	) {
		let old = self.data.pattern_resolution.insert(pattern, resolution);
		assert!(
			old.is_none(),
			"Tried to add pattern resolution for {:?} twice",
			pattern
		);
	}

	fn add_diagnostic(&mut self, _db: &'db dyn Db, item: Item<'db>, e: impl Into<Error>) {
		let error = e.into();
		assert_eq!(item, self.item, "Got error '{}' for wrong item", error);
		self.diagnostics.add_error(error);
	}

	fn add_warning(&mut self, _db: &'db dyn Db, item: Item<'db>, e: impl Into<Warning>) {
		let warning = e.into();
		assert_eq!(item, self.item, "Got warning '{}' for wrong item", warning);
		self.diagnostics.add_warning(warning);
	}

	fn add_type(&mut self, _db: &'db dyn Db, declared_type: TypeId<'db>, ty: Ty<'db>) {
		self.data.types.insert(declared_type, ty);
	}

	fn get_type(&self, db: &'db dyn Db, declared_type: TypeId<'db>) -> Ty<'db> {
		if let Some(ty) = self.data.types.get(declared_type) {
			return *ty;
		}
		// A declared type is usually completed while typing this item's signature,
		// not its body. A type which failed to complete is never recorded; an error
		// was already reported for it, so report the error type rather than panicking.
		self.item
			.possibly_cyclic_signature(db)
			.and_then(|signature| signature.types.get(&declared_type).copied())
			.unwrap_or_else(|| TypeRegistry::lookup(db).error)
	}

	fn type_pattern(&mut self, db: &'db dyn Db, pattern: PatternRef<'db>) -> PatternTy<'db> {
		let item = pattern.item(db);
		if item == self.item
			&& let Some(d) = self.data.patterns.get(pattern.pattern(db))
		{
			return d.clone();
		}
		let Some(signature) = item.possibly_cyclic_signature(db) else {
			return PatternTy::Computing;
		};
		signature.patterns[&pattern.pattern(db)].clone()
	}
}
