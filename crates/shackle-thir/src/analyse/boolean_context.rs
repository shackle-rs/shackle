//! Boolean (partiality) context analysis
//!
//! Determines if an expression is in root context or not, allowing us to then
//! totalise any partial expressions in non-root context in a later phase.

use std::ops::Not;

use rustc_hash::FxHashMap;
use shackle_hir::{BooleanLiteral, constants::IdentifierRegistry};
use shackle_ty::registry::TypeRegistry;
use shackle_utils::{maybe_grow_stack, refmap::RefMap};

use crate::{
	Call, Callable, Case, Db, DeclarationId, DomainData, Expression, ExpressionData, Generator,
	IfThenElse, ItemId, Let, LetItem, Marker, Model, ResolvedIdentifier, follow::follow_expression,
	traverse::Visitor,
};
/// The boolean context of an expression
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
	/// Boolean is fixed
	Root {
		/// True if root, false if root negative
		result: bool,
	},
	/// Promise total
	PromiseTotal,
	/// Positive, negative or mixed context
	NonRoot,
}

impl Not for Mode {
	type Output = Mode;

	fn not(self) -> Self::Output {
		match self {
			Mode::Root { result } => Mode::Root { result: !result },
			_ => self,
		}
	}
}

impl Mode {
	/// Create a new root mode
	pub fn root() -> Self {
		Mode::Root { result: true }
	}

	/// Whether this is root context
	pub fn is_root(&self) -> bool {
		matches!(self, Mode::Root { result: true, .. })
	}

	/// Whether this is root negative context
	pub fn is_root_neg(&self) -> bool {
		matches!(self, Mode::Root { result: false, .. })
	}

	/// Whether this is the promise total context
	pub fn is_promise_total(&self) -> bool {
		matches!(self, Mode::PromiseTotal)
	}

	/// Update this mode
	pub fn update(self, other: Mode) -> Mode {
		if let (Mode::NonRoot | Mode::PromiseTotal, Mode::Root { result }) = (self, other) {
			return Mode::Root { result };
		}
		self
	}

	/// Get as a string which can be used as an annotation name for debugging
	pub fn as_str(&self) -> &'static str {
		if self.is_root() {
			"ctx_root"
		} else if self.is_root_neg() {
			"ctx_root_neg"
		} else {
			"ctx_non_root"
		}
	}
}

/// Boolean context analysis
///
/// Determines whether expression are in root or non-root context.
/// Also determines the context of subexpressions of function bodies assuming
/// that the function is in root context.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModeAnalysis<'a, 'db, T: Marker = ()> {
	map: RefMap<'a, Expression<'db, T>, Mode>,
	root_map: RefMap<'a, Expression<'db, T>, Mode>,
}

impl<'a, 'db, T: Marker> ModeAnalysis<'a, 'db, T> {
	/// Run context analysis
	pub fn analyse(db: &'db dyn Db, model: &'a Model<'db, T>) -> ModeAnalysis<'a, 'db, T> {
		ModeAnalyser::run(db, model)
	}

	/// Get the context of the given expression
	pub fn get(&self, e: &'a Expression<'db, T>) -> Mode {
		self.map[e]
	}

	/// Get the context of the given expression assuming the containing function is root
	pub fn get_in_root_fn(&self, e: &'a Expression<'db, T>) -> Mode {
		self.root_map.get(e).copied().unwrap_or_else(|| self.map[e])
	}

	/// Update the mode for an expression and return the new mode if it was updated
	fn update(&mut self, e: &'a Expression<'db, T>, mode: Mode) -> Option<Mode> {
		log::debug!(
			"Updating mode of expression at {:?} to {:?}",
			e.origin(),
			mode
		);
		let mut inserted = false;
		let mut updated = false;
		let result = self.map.update_or_insert(
			e,
			|m| {
				let new_mode = m.update(mode);
				updated = *m != new_mode;
				*m = new_mode;
			},
			|| {
				inserted = true;
				mode
			},
		);
		if inserted || updated {
			Some(*result)
		} else {
			None
		}
	}

	/// Update the mode for an expression in the root fn map and return the new mode if it was updated
	fn update_in_root_fn(&mut self, e: &'a Expression<'db, T>, mode: Mode) -> Option<Mode> {
		let orig_mode = self.get_in_root_fn(e);
		let new_mode = orig_mode.update(mode);
		let changed = orig_mode != new_mode;
		if changed {
			let _ = self.root_map.insert(e, new_mode);
			Some(new_mode)
		} else {
			None
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct TodoItem<'a, 'db, T: Marker> {
	/// The expression
	expression: &'a Expression<'db, T>,
	/// The mode to be joined for this expression
	mode: Mode,
	/// True if this is an array where we need to propagate the context to the contained boolean values
	///
	/// E.g. for the argument of forall in root, or exists in root negative
	update_bools: bool,
}

/// Boolean context analyser
struct ModeAnalyser<'a, 'db, T: Marker> {
	model: &'a Model<'db, T>,
	todo: Vec<TodoItem<'a, 'db, T>>,
	result: ModeAnalysis<'a, 'db, T>,
	ids: &'db IdentifierRegistry<'db>,
	tys: &'db TypeRegistry<'db>,
	booleans: FxHashMap<DeclarationId<'db, T>, bool>,
	in_root_mode: bool,
}

impl<'a, 'db, T: Marker> ModeAnalyser<'a, 'db, T> {
	fn run(db: &'db dyn Db, model: &'a Model<'db, T>) -> ModeAnalysis<'a, 'db, T> {
		let mut analyser = ModeAnalyser {
			model,
			todo: Vec::new(),
			result: ModeAnalysis::default(),
			ids: IdentifierRegistry::lookup(db),
			tys: TypeRegistry::lookup(db),
			booleans: FxHashMap::default(),
			in_root_mode: false,
		};
		let root = Mode::root();
		// Analyse model items
		for item in model.top_level_items() {
			match item {
				ItemId::Annotation(_) => (),
				ItemId::Constraint(c) => {
					for ann in model[c].annotations().iter() {
						analyser.update(ann, root, false);
					}
					analyser.update(model[c].expression(), root, false);
				}
				ItemId::Declaration(d) => {
					for ann in model[d].annotations().iter() {
						analyser.update(ann, root, false);
					}
					for dom in model[d].domain().walk() {
						if let DomainData::Bounded(b) = &**dom {
							analyser.update(b, root, false);
						}
					}
					if let Some(def) = model[d].definition() {
						if def.ty().is_bool(db) {
							analyser.update(def, Mode::NonRoot, false);
						} else {
							analyser.update(def, root, false);
						}
					}
				}
				ItemId::Function(f) => {
					for ann in model[f].annotations().iter() {
						analyser.update(ann, root, false);
					}
					for p in model[f].parameters() {
						for ann in model[*p].annotations().iter() {
							analyser.update(ann, root, false);
						}
					}
					if let Some(b) = model[f].body() {
						analyser.update(
							b,
							if model[f].return_type() == analyser.tys.ann {
								root
							} else if model[f]
								.annotations()
								.has(model, analyser.ids.annotations.promise_total)
							{
								if model[f].return_type() == analyser.tys.var_bool {
									Mode::PromiseTotal
								} else {
									root
								}
							} else {
								Mode::NonRoot
							},
							false,
						);
					}
				}
				ItemId::Solve => {
					let solve = model.solve().unwrap();
					for ann in solve.annotations().iter() {
						analyser.update(ann, root, false);
					}
				}
				ItemId::Enumeration(_) | ItemId::Output(_) => unreachable!(),
			}
		}

		while let Some(it) = analyser.todo.pop() {
			analyser.run_iteration(db, &it);
		}

		// Analysis as if all functions are root and store separately
		analyser.in_root_mode = true;
		for (_, f) in model.top_level_functions() {
			if let Some(b) = f.body() {
				analyser.update(b, root, false);
			}
		}

		while let Some(it) = analyser.todo.pop() {
			analyser.run_iteration(db, &it);
		}

		analyser.result
	}

	fn run_iteration(&mut self, db: &'db dyn Db, it: &TodoItem<'a, 'db, T>) {
		assert!(
			!it.update_bools
				|| it
					.expression
					.ty()
					.elem_ty(db)
					.unwrap_or(self.tys.error)
					.is_bool(db),
			"Set update bools for non array of bool expression"
		);
		for ann in it.expression.annotations().iter() {
			self.update(ann, it.mode, false);
		}

		match &**it.expression {
			ExpressionData::Absent
			| ExpressionData::BooleanLiteral(_)
			| ExpressionData::IntegerLiteral(_)
			| ExpressionData::FloatLiteral(_)
			| ExpressionData::StringLiteral(_)
			| ExpressionData::Infinity => (), // No children to update
			ExpressionData::Identifier(i) => {
				if let ResolvedIdentifier::Declaration(d) = i
					&& let Some(def) = self.model[*d].definition()
				{
					self.update(def, it.mode, it.update_bools);
				}
			}
			ExpressionData::ArrayLiteral(al) => {
				if it.expression.ty().elem_ty(db).unwrap().is_bool(db) {
					if it.update_bools {
						// Argument to a logical predicate
						for e in al.iter() {
							self.update(e, it.mode, false);
						}
					} else {
						for e in al.iter() {
							self.update(e, Mode::NonRoot, false);
						}
					}
				} else {
					for e in al.iter() {
						self.update(e, it.mode, false);
					}
				}
			}
			ExpressionData::SetLiteral(sl) => {
				if it.expression.ty().elem_ty(db).unwrap().is_bool(db) {
					for e in sl.iter() {
						self.update(e, Mode::NonRoot, false);
					}
				} else {
					for e in sl.iter() {
						self.update(e, it.mode, false);
					}
				}
			}
			ExpressionData::TupleLiteral(tl) => {
				for e in tl.iter() {
					if e.ty().is_bool(db) {
						self.update(e, Mode::NonRoot, false);
					} else {
						self.update(e, it.mode, false);
					}
				}
			}
			ExpressionData::ArrayComprehension(c) => {
				for g in c.generators.iter() {
					match g {
						Generator::Iterator { collection, .. } => {
							self.update(collection, it.mode, false);
						}
						Generator::Assignment { assignment, .. } => {
							let mode = if self.model[*assignment].ty().is_bool(db) {
								Mode::NonRoot
							} else {
								it.mode
							};
							self.update(self.model[*assignment].definition().unwrap(), mode, false);
						}
					}
					if let Some(w) = g.where_clause() {
						self.update(w, Mode::NonRoot, false);
					}
				}
				if let Some(e) = &c.indices {
					self.update(e, it.mode, false);
				}
				if it.expression.ty().elem_ty(db).unwrap().is_bool(db) {
					if it.update_bools {
						self.update(&c.template, Mode::NonRoot, false);
					} else {
						self.update(&c.template, it.mode, false);
					}
				} else {
					self.update(&c.template, it.mode, false);
				}
			}
			ExpressionData::TupleAccess(ta) => {
				for e in follow_expression(self.model, it.expression) {
					self.update(e, it.mode, false);
				}
				self.update(&ta.tuple, it.mode, false);
			}
			ExpressionData::IfThenElse(ite) => {
				let mut mode = it.mode;
				for b in ite.branches.iter() {
					self.update(&b.condition, Mode::NonRoot, false);
					if b.var_condition(db) {
						mode = Mode::NonRoot;
					}
					self.update(&b.result, mode, it.update_bools);
				}
				self.update(&ite.else_result, mode, it.update_bools);
			}
			ExpressionData::Case(_) => todo!(),
			ExpressionData::Call(c) => (|| {
				match &c.function {
					Callable::Function(f) => {
						let name = self.model[*f].name();
						let is_builtin = self.model[*f].body().is_none();

						if name == self.ids.functions.not && c.arguments.len() == 1 && is_builtin {
							self.update(&c.arguments[0], !it.mode, false);
							return;
						} else if (name == self.ids.functions.and && it.mode.is_root()
							|| name == self.ids.functions.or && it.mode.is_root_neg())
							&& c.arguments.len() == 2
							&& is_builtin
						{
							self.update(&c.arguments[0], it.mode, false);
							self.update(&c.arguments[1], it.mode, false);
						} else if (name == self.ids.functions.forall && it.mode.is_root()
							|| name == self.ids.functions.exists && it.mode.is_root_neg())
							&& c.arguments.len() == 1
							&& is_builtin
						{
							self.update(&c.arguments[0], it.mode, true);
							return;
						} else if name == self.ids.functions.clause
							&& c.arguments.len() == 2
							&& it.mode.is_root_neg()
							&& is_builtin
						{
							self.update(&c.arguments[0], it.mode, true);
							self.update(&c.arguments[1], !it.mode, true);
							return;
						} else if (name == self.ids.functions.symmetry_breaking_constraint
							|| name == self.ids.functions.redundant_constraint)
							&& c.arguments.len() == 1
						{
							self.update(&c.arguments[0], it.mode, false);
							return;
						} else if name == self.ids.functions.mzn_default_partial
							&& c.arguments.len() == 2
						{
							self.update(&c.arguments[0], Mode::NonRoot, it.update_bools);
							self.update(&c.arguments[1], Mode::NonRoot, it.update_bools);
							return;
						}
					}
					Callable::Expression(e) => {
						self.update(e, it.mode, false);
					}
					Callable::Annotation(_) | Callable::AnnotationDestructure(_) => (),
					Callable::EnumConstructor(_) | Callable::EnumDestructor(_) => {
						unreachable!()
					}
				}
				for e in c.arguments.iter() {
					if e.ty().is_bool(db) {
						self.update(e, Mode::NonRoot, false);
					} else {
						self.update(e, it.mode, false);
					}
				}
			})(),
			ExpressionData::Let(l) => {
				for item in l.items.iter() {
					match item {
						LetItem::Constraint(c) => {
							for ann in self.model[*c].annotations().iter() {
								self.update(ann, it.mode, false);
							}

							let in_expression = self.model[*c].expression();
							let mode = if it.mode.is_promise_total()
								|| BooleanVisitor::is_true(
									db,
									self.model,
									&mut self.booleans,
									in_expression,
								) {
								Mode::root()
							} else {
								it.mode
							};

							self.update(in_expression, mode, false);
						}
						LetItem::Declaration(d) => {
							for ann in self.model[*d].annotations().iter() {
								self.update(ann, it.mode, false);
							}
							for dom in self.model[*d].domain().walk() {
								if let DomainData::Bounded(b) = &**dom {
									self.update(b, it.mode, false);
								}
							}
							if let Some(def) = self.model[*d].definition() {
								if self.model[*d].ty().is_bool(db) {
									self.update(def, Mode::NonRoot, false);
								} else {
									self.update(def, it.mode, false);
								}
							}
						}
					}
				}
				self.update(&l.in_expression, it.mode, false);
			}
			ExpressionData::Lambda(_) => todo!(),
			ExpressionData::RecordAccess(_)
			| ExpressionData::SetComprehension(_)
			| ExpressionData::RecordLiteral(_) => {
				unreachable!()
			}
		}
	}

	/// Update the context of an expression.
	fn update(&mut self, expression: &'a Expression<'db, T>, mode: Mode, update_bools: bool) {
		let new_mode = if self.in_root_mode {
			self.result.update_in_root_fn(expression, mode)
		} else {
			self.result.update(expression, mode)
		};

		// After update, subexpressions may need updating
		if let Some(m) = new_mode {
			self.todo.push(TodoItem {
				expression,
				mode: m,
				update_bools,
			});
		}
	}
}

struct BooleanVisitor<'a, 'db, T: Marker> {
	db: &'db dyn Db,
	ids: &'db IdentifierRegistry<'db>,
	decls: &'a mut FxHashMap<DeclarationId<'db, T>, bool>,
	is_true: bool,
}

impl<'a, 'db, T: Marker> BooleanVisitor<'a, 'db, T> {
	/// Whether or not the given expression is statically known to be true
	fn is_true(
		db: &'db dyn Db,
		model: &'a Model<'db, T>,
		decls: &'a mut FxHashMap<DeclarationId<'db, T>, bool>,
		expression: &'a Expression<'db, T>,
	) -> bool {
		let mut visitor = Self {
			db,
			decls,
			ids: IdentifierRegistry::lookup(db),
			is_true: true,
		};
		visitor.visit_expression(model, expression);
		visitor.is_true
	}
}

impl<'a, 'db, T: Marker> Visitor<'a, 'db, T> for BooleanVisitor<'a, 'db, T> {
	fn visit_expression(&mut self, model: &'a Model<'db, T>, expression: &'a Expression<'db, T>) {
		maybe_grow_stack(|| {
			match &**expression {
				ExpressionData::BooleanLiteral(b) => self.visit_boolean(model, b),
				ExpressionData::Identifier(i) => self.visit_identifier(model, i),
				ExpressionData::IfThenElse(ite) => self.visit_if_then_else(model, ite),
				ExpressionData::TupleAccess(_) => {
					if let Some(e) = follow_expression(model, expression).next() {
						self.visit_expression(model, e);
					} else {
						self.is_true = false
					}
				}
				ExpressionData::Case(c) => self.visit_case(model, c),
				ExpressionData::Call(c) => self.visit_call(model, c),
				ExpressionData::Let(l) => self.visit_let(model, l),
				_ => self.is_true = false,
			};
		});
	}

	fn visit_boolean(&mut self, _model: &'a Model<'db, T>, b: &'a BooleanLiteral) {
		self.is_true &= b.0;
	}

	fn visit_identifier(
		&mut self,
		model: &'a Model<'db, T>,
		identifier: &'a ResolvedIdentifier<'db, T>,
	) {
		if let ResolvedIdentifier::Declaration(d) = identifier {
			if let Some(t) = self.decls.get(d) {
				self.is_true &= *t;
			} else if let Some(def) = model[*d].definition() {
				// Cache result in case identifier used multiple times
				let is_true = BooleanVisitor::is_true(self.db, model, self.decls, def);
				let _ = self.decls.insert(*d, is_true);
				self.is_true &= is_true;
			} else {
				// No RHS, so unknown
				let _ = self.decls.insert(*d, false);
				self.is_true = false;
			}
		}
	}

	fn visit_call(&mut self, model: &'a Model<'db, T>, call: &'a Call<'db, T>) {
		if call.matches_builtin(model, self.ids.functions.abort) {
			return;
		}
		self.is_true = false;
	}

	fn visit_let(&mut self, model: &'a Model<'db, T>, l: &'a Let<'db, T>) {
		for item in l.items.iter() {
			match item {
				LetItem::Declaration(_) => {
					// Do not know if this expression is fixed to true
					self.is_true = false;
					return;
				}
				LetItem::Constraint(c) => {
					self.visit_expression(model, model[*c].expression());
					if !self.is_true {
						return;
					}
				}
			}
		}
		self.visit_expression(model, &l.in_expression);
	}

	fn visit_if_then_else(&mut self, model: &'a Model<'db, T>, ite: &'a IfThenElse<'db, T>) {
		for branch in ite.branches.iter() {
			self.visit_expression(model, &branch.result);
			if !self.is_true {
				return;
			}
		}
		self.visit_expression(model, &ite.else_result);
	}

	fn visit_case(&mut self, model: &'a Model<'db, T>, c: &'a Case<'db, T>) {
		for branch in c.branches.iter() {
			self.visit_expression(model, &branch.result);
			if !self.is_true {
				return;
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
		ids::NodeRef,
		input::{CompilerSettings, InlineModelFile, InputFiles},
	};
	use shackle_syntax::InputLang;

	use super::ModeAnalysis;
	use crate::{
		lower::lower_model, pretty_print::PrettyPrinter, transform::inlining::inline_functions,
	};

	fn check_bool_ctx(program: &str, expected: Expect, fn_root: bool) {
		let mut db = CompilerDatabase::default();
		let _ = CompilerSettings::get(&db)
			.set_ignore_stdlib(&mut db)
			.to(true);
		let model_file = InlineModelFile::new(&db, program.to_owned(), InputLang::MiniZinc).into();
		let _ = InputFiles::get(&db).set_files(&mut db).to(vec![model_file]);
		let mut model = lower_model(&db).take();
		model = inline_functions(&db, model).unwrap();
		let result = ModeAnalysis::analyse(&db, &model);
		let mut printer = PrettyPrinter::new(&db, &model);
		printer.expression_annotator = Some(Box::new(|e| {
			if fn_root {
				Some(result.get_in_root_fn(e).as_str().to_owned())
			} else {
				Some(result.get(e).as_str().to_owned())
			}
		}));
		let to_print = model
			.top_level_items()
			.filter(|it| match model.item_origin(*it).node() {
				Some(NodeRef::Item(item)) => item.model_file(&db) == model_file,
				Some(NodeRef::Entity(entity)) => entity.item(&db).model_file(&db) == model_file,
				Some(NodeRef::Model(m)) => m == model_file,
				None => true,
			});
		let mut pretty = String::new();
		for item in to_print {
			pretty.push_str(&printer.pretty_print_item(item));
			pretty.push_str(";\n");
		}
		expected.assert_eq(&pretty);
	}

	#[test]
	fn test_bool_ctx() {
		check_bool_ctx(
			r#"
			predicate forall(array [int] of var bool: b);
			predicate '\/'(var bool: x, var bool: y);
			predicate 'not'(var bool: x);
			var bool: a;
			var bool: b;
			constraint forall([a, b]);
			var bool: c;
			var int: d;
			function var bool: foo(var bool: c, var int: d);
			constraint foo(c, d);
			var bool: e;
			var bool: f;
			constraint not (e \/ f);
			var int: g = let {
				var int: h = 1;
				var bool: p = true;
				var bool: q = true;
				constraint q;
			} in h;
			"#,
			expect![[r#"
    predicate forall(array [int] of var bool: b);
    function var bool: '\/'(var bool: x, var bool: y);
    function var bool: 'not'(var bool: x);
    var bool: a;
    var bool: b;
    constraint forall([a:: ctx_root, b:: ctx_root]:: ctx_root):: ctx_root;
    var bool: c;
    var int: d;
    predicate foo(var bool: c, var int: d);
    constraint foo(c:: ctx_non_root, d:: ctx_root):: ctx_root;
    var bool: e;
    var bool: f;
    constraint 'not'('\/'(e:: ctx_root_neg, f:: ctx_root_neg):: ctx_root_neg):: ctx_root;
    var int: g = let {
      var int: h = 1:: ctx_root;
      var bool: p = true:: ctx_non_root;
      var bool: q = true:: ctx_root;
      constraint q:: ctx_root;
    } in h:: ctx_root:: ctx_root;
"#]],
			false,
		)
	}

	#[test]
	fn test_fn_ctx() {
		let program = r#"
			predicate '>'(var int: x, var int: y);
			function var int: '+'(var int: x, var int: y);
			function var int: foo(var int: x, var int: y) = let {
				constraint x > y;
			} in x + y
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    function var bool: '>'(var int: x, var int: y);
    function var int: '+'(var int: x, var int: y);
    function var int: foo(var int: x, var int: y) = let {
      constraint '>'(x:: ctx_non_root, y:: ctx_non_root):: ctx_non_root;
    } in '+'(x:: ctx_non_root, y:: ctx_non_root):: ctx_non_root:: ctx_non_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    function var bool: '>'(var int: x, var int: y);
    function var int: '+'(var int: x, var int: y);
    function var int: foo(var int: x, var int: y) = let {
      constraint '>'(x:: ctx_root, y:: ctx_root):: ctx_root;
    } in '+'(x:: ctx_root, y:: ctx_root):: ctx_root:: ctx_root;
"#]],
			true,
		);
	}

	#[test]
	fn test_bool_ctx_let() {
		let program = r#"
            function var set of int: foo() = let {
				var bool: b;
				constraint b;
			} in {1, 3, 5};
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    function var set of int: foo() = let {
      var bool: b;
      constraint b:: ctx_non_root;
    } in {1:: ctx_non_root, 3:: ctx_non_root, 5:: ctx_non_root}:: ctx_non_root:: ctx_non_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    function var set of int: foo() = let {
      var bool: b;
      constraint b:: ctx_root;
    } in {1:: ctx_root, 3:: ctx_root, 5:: ctx_root}:: ctx_root:: ctx_root;
"#]],
			true,
		);
	}

	#[test]
	fn test_bool_ctx_abort() {
		let program = r#"
			test abort(string: msg);
			test bar(int: x);
			function int: foo(int: x) = let {
				constraint if bar(x) then abort("foo") endif;
			} in x;
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    function bool: abort(string: msg);
    function bool: bar(int: x);
    function int: foo(int: x) = let {
      constraint if bar(x:: ctx_non_root):: ctx_non_root then abort("foo":: ctx_root):: ctx_root else true:: ctx_root endif:: ctx_root;
    } in x:: ctx_non_root:: ctx_non_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    function bool: abort(string: msg);
    function bool: bar(int: x);
    function int: foo(int: x) = let {
      constraint if bar(x:: ctx_non_root):: ctx_non_root then abort("foo":: ctx_root):: ctx_root else true:: ctx_root endif:: ctx_root;
    } in x:: ctx_root:: ctx_root;
"#]],
			true,
		);
	}

	#[test]
	fn test_bool_ctx_default() {
		let program = r#"
			function var int: mzn_default_partial(var int, var int);
			function var int: foo() = mzn_default_partial(1, 2);
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    function var int: mzn_default_partial(var int: _DECL_1, var int: _DECL_2);
    function var int: foo() = mzn_default_partial(1:: ctx_non_root, 2:: ctx_non_root):: ctx_non_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    function var int: mzn_default_partial(var int: _DECL_1, var int: _DECL_2);
    function var int: foo() = mzn_default_partial(1:: ctx_non_root, 2:: ctx_non_root):: ctx_root;
"#]],
			true,
		);
	}

	#[test]
	fn test_bool_ctx_promise_total_bool() {
		let program = r#"
			annotation promise_total;
			predicate bar(var int: x, var bool: r);
			function var int: foo(var int: x) :: promise_total =
				let {
					var bool: r;
					constraint bar(x, r);
				} in r;
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    annotation promise_total;
    predicate bar(var int: x, var bool: r);
    function var int: foo(var int: x) :: (promise_total:: ctx_root) = let {
      var bool: r;
      constraint bar(x:: ctx_root, r:: ctx_non_root):: ctx_root;
    } in r:: ctx_root:: ctx_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    annotation promise_total;
    predicate bar(var int: x, var bool: r);
    function var int: foo(var int: x) :: (promise_total:: ctx_root) = let {
      var bool: r;
      constraint bar(x:: ctx_root, r:: ctx_non_root):: ctx_root;
    } in r:: ctx_root:: ctx_root;
"#]],
			true,
		);
	}

	#[test]
	fn test_bool_ctx_promise_total_non_bool() {
		let program = r#"
			annotation promise_total;
			predicate bar(var int: x, var int: r);
			function var int: foo(var int: x) :: promise_total =
				let {
					var int: r;
					constraint bar(x, r);
				} in r;
		"#;
		check_bool_ctx(
			program,
			expect![[r#"
    annotation promise_total;
    predicate bar(var int: x, var int: r);
    function var int: foo(var int: x) :: (promise_total:: ctx_root) = let {
      var int: r;
      constraint bar(x:: ctx_root, r:: ctx_root):: ctx_root;
    } in r:: ctx_root:: ctx_root;
"#]],
			false,
		);
		check_bool_ctx(
			program,
			expect![[r#"
    annotation promise_total;
    predicate bar(var int: x, var int: r);
    function var int: foo(var int: x) :: (promise_total:: ctx_root) = let {
      var int: r;
      constraint bar(x:: ctx_root, r:: ctx_root):: ctx_root;
    } in r:: ctx_root:: ctx_root;
"#]],
			true,
		);
	}
}
