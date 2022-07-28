/// Types of 'bodies': everything that isn't part of a signature.
///
/// E.g.
/// - Annotations on items
/// - RHS of variable declarations
/// - Bodies of functions
use rustc_hash::FxHashMap;

use crate::{
	arena::{ArenaIndex, ArenaMap},
	hir::{
		db::Hir,
		ids::{ExpressionRef, ItemRef, LocalItemRef, PatternRef},
		Expression, Pattern, Ty, Type, TypeRegistry,
	},
	Error,
};

use super::{DeclarationType, TypeContext, Typer};

/// Collected types for an item body
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyTypes {
	/// Types of declarations
	pub patterns: ArenaMap<Pattern, DeclarationType>,
	/// Types of expressions
	pub expressions: ArenaMap<Expression, Ty>,
	/// Identifier resolution
	pub identifier_resolution: FxHashMap<ArenaIndex<Expression>, PatternRef>,
	/// Pattern resolution
	pub pattern_resolution: FxHashMap<ArenaIndex<Pattern>, PatternRef>,
}

/// Context for typing an item body
pub struct BodyTypeContext {
	item: ItemRef,
	data: BodyTypes,
	diagnostics: Vec<Error>,
}

impl BodyTypeContext {
	/// Create a new signature type context
	pub fn new(item: ItemRef) -> Self {
		Self {
			item,
			data: BodyTypes {
				patterns: ArenaMap::new(),
				expressions: ArenaMap::new(),
				identifier_resolution: FxHashMap::default(),
				pattern_resolution: FxHashMap::default(),
			},
			diagnostics: Vec::new(),
		}
	}

	/// Compute the type of the body of this item
	pub fn type_item(&mut self, db: &dyn Hir, types: &TypeRegistry) {
		let item = self.item;
		let model = self.item.model(db);
		let it = self.item.local_item_ref(db);
		let data = it.data(&model);
		let mut typer = Typer::new(db, types, self, item, data);
		match it {
			LocalItemRef::Function(f) => {
				let it = &model[f];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}

				if let Some(e) = it.body {
					let signature = db.lookup_item_signature(item);
					match &signature.patterns[&PatternRef::new(item, it.pattern)] {
						DeclarationType::Function(function) => {
							typer.typecheck_expression(e, function.overload.return_type());
						}
						_ => unreachable!(),
					};
				}
			}
			LocalItemRef::Declaration(d) => {
				let it = &model[d];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}
				if let Type::Any = data[it.declared_type] {
					// Already done in signature
				} else if let Some(e) = it.definition {
					let signature = db.lookup_item_signature(item);
					let expected = match &signature.patterns[&PatternRef::new(item, it.pattern)] {
						DeclarationType::Variable(t) => *t,
						_ => unreachable!(),
					};
					typer.typecheck_expression(e, expected);
				}
			}
			LocalItemRef::Output(o) => {
				let it = &model[o];
				if let Some(s) = &it.section {
					typer.typecheck_expression(*s, types.string);
				}
				typer.typecheck_expression(it.expression, types.array_of_string);
			}
			LocalItemRef::Constraint(c) => {
				let it = &model[c];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}
				typer.typecheck_expression(it.expression, types.var_bool);
			}
			LocalItemRef::Solve(s) => {
				let it = &model[s];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}
			}
			LocalItemRef::Assignment(a) => {
				let it = &model[a];
				let expected = typer.collect_expression(it.assignee);
				typer.typecheck_expression(it.definition, expected);
			}
			LocalItemRef::Enumeration(e) => {
				let it = &model[e];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}
			}
			LocalItemRef::TypeAlias(t) => {
				let it = &model[t];
				for ann in it.annotations.iter() {
					typer.typecheck_expression(*ann, types.ann);
				}
			}
		}
	}
	/// Get results of typing
	pub fn finish(mut self) -> (BodyTypes, Vec<Error>) {
		self.data.patterns.shrink_to_fit();
		self.data.expressions.shrink_to_fit();
		self.data.identifier_resolution.shrink_to_fit();
		self.data.pattern_resolution.shrink_to_fit();
		(self.data, self.diagnostics)
	}
}

impl TypeContext for BodyTypeContext {
	fn add_declaration(&mut self, pattern: PatternRef, declaration: DeclarationType) {
		assert_eq!(pattern.item(), self.item);
		self.data.patterns.insert(pattern.pattern(), declaration);
	}
	fn add_expression(&mut self, expression: ExpressionRef, ty: Ty) {
		assert_eq!(expression.item(), self.item);
		self.data.expressions.insert(expression.expression(), ty);
	}
	fn add_identifier_resolution(&mut self, expression: ExpressionRef, resolution: PatternRef) {
		assert_eq!(expression.item(), self.item);
		self.data
			.identifier_resolution
			.insert(expression.expression(), resolution);
	}
	fn add_pattern_resolution(&mut self, pattern: PatternRef, resolution: PatternRef) {
		assert_eq!(pattern.item(), self.item);
		self.data
			.pattern_resolution
			.insert(pattern.pattern(), resolution);
	}
	fn add_diagnostic(&mut self, item: ItemRef, e: impl Into<Error>) {
		let error = e.into();
		assert_eq!(item, self.item, "Got error '{}' for wrong item", error);
		self.diagnostics.push(error);
	}

	fn type_pattern(
		&mut self,
		db: &dyn Hir,
		_types: &TypeRegistry,
		pattern: PatternRef,
	) -> DeclarationType {
		if pattern.item() == self.item {
			if let Some(d) = self.data.patterns.get(pattern.pattern()) {
				return d.clone();
			}
		}
		let signature = db.lookup_item_signature(pattern.item());
		signature.patterns[&pattern].clone()
	}
}
