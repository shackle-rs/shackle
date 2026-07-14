//! Pretty printing of THIR as MiniZinc
//!

use std::fmt::Write;

use shackle_hir::constants::IdentifierRegistry;
use shackle_ty::registry::TypeRegistry;
use shackle_utils::maybe_grow_stack;

use crate::{
	AnnotationId, Callable, ConstraintId, Db, DeclarationId, Domain, DomainData, EnumerationId,
	Expression, ExpressionData, FunctionId, Generator, Goal, ItemId, LetItem, Marker, Model,
	OutputId, Pattern, PatternData, ResolvedIdentifier,
};

static MINIZINC_COMPAT: &str = include_str!("../../../share/minizinc/compat.mzn");

/// Callback which adds an annotation to a pretty-printed expression.
pub type ExpressionAnnotator<'db, T> = dyn Fn(&Expression<'db, T>) -> Option<String> + 'db;

/// Pretty prints THIR as MiniZinc
pub struct PrettyPrinter<'db, T: Marker = ()> {
	db: &'db dyn Db,
	model: &'db Model<'db, T>,
	ids: &'db IdentifierRegistry<'db>,
	/// Whether to output a model compatible with old MiniZinc (default `false`)
	pub old_compat: bool,
	/// Add an extra annotation on each expression using the given function
	pub expression_annotator: Option<Box<ExpressionAnnotator<'db, T>>>,
}

impl<'db, T: Marker> std::fmt::Debug for PrettyPrinter<'db, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("PrettyPrinter")
			.field("old_compat", &self.old_compat)
			.finish()
	}
}

impl<'db, T: Marker> PrettyPrinter<'db, T> {
	/// Create a new pretty printer
	pub fn new(db: &'db dyn Db, model: &'db Model<'db, T>) -> Self {
		let ids = IdentifierRegistry::lookup(db);
		Self {
			db,
			model,
			ids,
			old_compat: false,
			expression_annotator: None,
		}
	}

	/// Create a new pretty printer which prints output compatible with old MiniZinc
	pub fn new_compat(db: &'db dyn Db, model: &'db Model<'db, T>) -> Self {
		let mut printer = Self::new(db, model);
		printer.old_compat = true;
		printer
	}

	/// Pretty print the model
	pub fn pretty_print(&self) -> String {
		let mut buf = String::new();
		for item in self.model.top_level_items() {
			if self.old_compat {
				match item {
					ItemId::Function(f)
						if self.model[f].name() == self.ids.functions.default
							|| self.model[f].name() == self.ids.builtins.mzn_element_internal
							|| self.model[f].name() == self.ids.builtins.mzn_slice_internal =>
					{
						continue;
					}
					ItemId::Annotation(a)
						if self.model[a].name == Some(self.ids.annotations.output) =>
					{
						continue;
					}
					_ => (),
				}
			}
			writeln!(&mut buf, "{};", self.pretty_print_item(item)).unwrap();
		}
		if self.model.solve().is_none() {
			writeln!(&mut buf, "solve satisfy;").unwrap();
		}
		if self.old_compat {
			writeln!(&mut buf, "{}", MINIZINC_COMPAT).unwrap();
		}
		buf
	}

	/// Pretty print an item from a model
	pub fn pretty_print_signature(&self, item: ItemId<'db, T>) -> String {
		match item {
			ItemId::Annotation(i) => self.pretty_print_annotation(i),
			ItemId::Constraint(i) => self.pretty_print_constraint(i),
			ItemId::Declaration(i) => self.pretty_print_declaration(i, false, true),
			ItemId::Enumeration(i) => self.pretty_print_enumeration(i, true),
			ItemId::Function(i) => self.pretty_print_function(i, true),
			ItemId::Output(i) => self.pretty_print_output(i),
			ItemId::Solve => self.pretty_print_solve(),
		}
	}

	/// Pretty print an item from a model
	pub fn pretty_print_item(&self, item: ItemId<'db, T>) -> String {
		match item {
			ItemId::Annotation(i) => self.pretty_print_annotation(i),
			ItemId::Constraint(i) => self.pretty_print_constraint(i),
			ItemId::Declaration(i) => self.pretty_print_declaration(i, false, false),
			ItemId::Enumeration(i) => self.pretty_print_enumeration(i, false),
			ItemId::Function(i) => self.pretty_print_function(i, false),
			ItemId::Output(i) => self.pretty_print_output(i),
			ItemId::Solve => self.pretty_print_solve(),
		}
	}

	fn pretty_print_annotation(&self, idx: AnnotationId<'db, T>) -> String {
		let annotation = &self.model[idx];
		let name = annotation
			.name
			.map(|i| i.pretty_print(self.db))
			.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(idx)));
		let mut buf = format!("annotation {}", name);
		if let Some(params) = &annotation.parameters {
			write!(
				&mut buf,
				"({})",
				params
					.iter()
					.map(|p| self.pretty_print_declaration(*p, false, true))
					.collect::<Vec<_>>()
					.join(", ")
			)
			.unwrap();
		}
		buf
	}

	fn pretty_print_constraint(&self, idx: ConstraintId<'db, T>) -> String {
		let constraint = &self.model[idx];
		let mut buf = "constraint ".to_owned();
		if self.old_compat {
			for ann in constraint.annotations().iter() {
				if matches!(&**ann, ExpressionData::StringLiteral(_)) {
					// Old compiler only supports single string annotation
					write!(&mut buf, ":: ({}) ", self.pretty_print_expression(ann)).unwrap();
					break;
				}
			}
		} else {
			for ann in constraint.annotations().iter() {
				write!(&mut buf, ":: ({}) ", self.pretty_print_expression(ann)).unwrap();
			}
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(constraint.expression())
		)
		.unwrap();
		buf
	}

	fn pretty_print_declaration(
		&self,
		idx: DeclarationId<'db, T>,
		is_let_item: bool,
		signature_only: bool,
	) -> String {
		let declaration = &self.model[idx];
		let ty = declaration.ty();
		let mut buf = if is_let_item
			&& ty.contains_type_inst_var(self.db)
			&& declaration.definition().is_some()
			|| ty == TypeRegistry::lookup(self.db).bottom
		{
			// Workaround since let items can't use TiIDs in MiniZinc
			"any".to_owned()
		} else {
			self.pretty_print_domain(declaration.domain())
		};
		write!(
			&mut buf,
			": {}",
			declaration
				.name()
				.map(|name| if name.lookup(self.db) == "_objective" {
					"_DECL_OBJ".to_owned()
				} else {
					name.pretty_print(self.db)
				})
				.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(idx)))
		)
		.unwrap();
		for ann in declaration.annotations().iter() {
			write!(&mut buf, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if !signature_only && let Some(def) = declaration.definition() {
			write!(&mut buf, " = {}", self.pretty_print_expression(def)).unwrap();
		}
		buf
	}

	fn pretty_print_enumeration(&self, idx: EnumerationId<'db, T>, signature_only: bool) -> String {
		let enumeration = &self.model[idx];
		let enum_name = enumeration.enum_type().pretty_print(self.db);
		let mut buf = format!("enum {}", enum_name);
		for ann in enumeration.annotations().iter() {
			write!(&mut buf, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if !signature_only && let Some(cases) = enumeration.definition() {
			write!(
				&mut buf,
				" = {}",
				cases
					.iter()
					.enumerate()
					.map(|(i, c)| {
						let name = c
							.name
							.map(|n| n.pretty_print(self.db))
							.unwrap_or_else(|| format!("_EM_{}_{}", enum_name, i));

						match &c.parameters {
							Some(ps) => {
								let params = ps
									.iter()
									.map(|d| self.pretty_print_domain(self.model[*d].domain()))
									.collect::<Vec<_>>()
									.join(", ");
								format!("{}({})", name, params)
							}
							None => format!("{{ {} }}", name),
						}
					})
					.collect::<Vec<_>>()
					.join(" ++ ")
			)
			.unwrap();
		}
		buf
	}

	fn pretty_print_function(&self, idx: FunctionId<'db, T>, signature_only: bool) -> String {
		let function = &self.model[idx];
		let name = (|| {
			if let Some(tys) = function.mangled_param_tys()
				&& (!self.old_compat || function.body().is_some())
			{
				return function
					.name()
					.mangled(self.db, tys.iter().copied())
					.pretty_print(self.db);
			}
			function.name().pretty_print(self.db)
		})();
		let mut buf = String::new();
		if function.body().is_none()
			&& function.return_type() == TypeRegistry::lookup(self.db).var_bool
			&& !name.starts_with('\'')
		{
			write!(&mut buf, "predicate",).unwrap();
		} else {
			write!(
				&mut buf,
				"function {}:",
				self.pretty_print_domain(function.domain()),
			)
			.unwrap();
		}
		write!(
			&mut buf,
			" {}({})",
			name,
			function
				.parameters()
				.iter()
				.map(|p| self.pretty_print_declaration(*p, false, signature_only))
				.collect::<Vec<_>>()
				.join(", ")
		)
		.unwrap();
		for ann in function.annotations().iter() {
			write!(&mut buf, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if self.old_compat {
			write!(&mut buf, " :: promise_total").unwrap();
			if let Some(body) = function.body() {
				if function.name() == self.ids.functions.deopt
					&& !function.type_inst_vars().is_empty()
					&& function.parameters().len() == 1
					&& {
						let ty = self.model[function.parameter(0)].ty();
						!ty.known_par(self.db) && !ty.known_occurs(self.db)
					} {
					// For compatibility with old minizinc, we can just directly coerce
					if let ExpressionData::Call(c) = &**body
						&& let Callable::Function(idx) = &c.function
						&& self.model[*idx].name() == self.ids.functions.to_enum
						&& c.arguments.len() == 2
					{
						write!(
							&mut buf,
							" = {}",
							self.pretty_print_expression(&c.arguments[1])
						)
						.unwrap();
					}
				}
			} else if function.name() == self.ids.functions.enum2int {
				// For compatibility with old minizinc, we can just directly coerce
				let d = function.parameter(0);
				let ident = self.model[d]
					.name()
					.map(|n| n.pretty_print(self.db))
					.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(d)));
				write!(&mut buf, " = {}", ident).unwrap();
			}
		}

		if !signature_only && let Some(body) = function.body() {
			write!(&mut buf, " = {}", self.pretty_print_expression(body)).unwrap();
		}

		buf
	}

	fn pretty_print_output(&self, idx: OutputId<'db, T>) -> String {
		let output = &self.model[idx];
		let mut buf = "output ".to_owned();
		if let Some(s) = output.section() {
			write!(&mut buf, ":: {} ", self.pretty_print_expression(s)).unwrap();
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(output.expression())
		)
		.unwrap();
		buf
	}

	fn pretty_print_solve(&self) -> String {
		let solve = self.model.solve().unwrap();
		let mut buf = "solve ".to_owned();
		for ann in solve.annotations().iter() {
			write!(&mut buf, ":: ({}) ", self.pretty_print_expression(ann)).unwrap();
		}
		let s = match solve.goal() {
			Goal::Satisfy => "satisfy",
			Goal::Maximize { .. } => "maximize _DECL_OBJ",
			Goal::Minimize { .. } => "minimize _DECL_OBJ",
		};
		write!(&mut buf, "{}", s).unwrap();
		buf
	}

	/// Pretty print a domain
	pub fn pretty_print_domain(&self, domain: &Domain<'db, T>) -> String {
		maybe_grow_stack(|| self.pretty_print_domain_inner(domain))
	}

	fn pretty_print_domain_inner(&self, domain: &Domain<'db, T>) -> String {
		let ty = domain.ty();
		match &**domain {
			DomainData::Array(dim, el) => {
				let dims = match &***dim {
					DomainData::Tuple(ds) => ds
						.iter()
						.map(|d| self.pretty_print_domain(d))
						.collect::<Vec<_>>()
						.join(", "),
					DomainData::Unbounded => {
						ty.dim_ty(self.db).unwrap().pretty_print_as_dims(self.db)
					}
					_ => self.pretty_print_domain(dim),
				};
				ty.opt(self.db)
					.into_iter()
					.flat_map(|o| o.pretty_print())
					.chain([format!(
						"array [{}] of {}",
						dims,
						self.pretty_print_domain(el)
					)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			DomainData::Bounded(e) => ty
				.inst(self.db)
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
				.chain([self.pretty_print_expression(e)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Set(s, None) => ty
				.inst(self.db)
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
				.chain(["set of".to_owned()])
				.chain([self.pretty_print_domain(s)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Set(s, Some(c)) => ty
				.inst(self.db)
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
				.chain(["set(".to_owned()])
				.chain([self.pretty_print_expression(c)])
				.chain([") of ".to_owned()])
				.chain([self.pretty_print_domain(s)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Tuple(ds) => {
				let doms = ds
					.iter()
					.map(|d| self.pretty_print_domain(d))
					.collect::<Vec<_>>()
					.join(", ");
				ty.inst(self.db)
					.into_iter()
					.flat_map(|i| i.pretty_print())
					.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
					.chain([format!("tuple({})", doms)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			DomainData::Record(ds) => {
				if self.old_compat && ds.is_empty() {
					return ty
						.inst(self.db)
						.into_iter()
						.flat_map(|i| i.pretty_print())
						.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
						.chain(["bool".to_owned()])
						.collect::<Vec<_>>()
						.join(" ");
				}
				let doms = ds
					.iter()
					.map(|(i, d)| {
						format!(
							"{}: {}",
							self.pretty_print_domain(d),
							i.pretty_print(self.db)
						)
					})
					.collect::<Vec<_>>()
					.join(", ");
				ty.inst(self.db)
					.into_iter()
					.flat_map(|i| i.pretty_print())
					.chain(ty.opt(self.db).into_iter().flat_map(|o| o.pretty_print()))
					.chain([format!("record({})", doms)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			DomainData::Unbounded => ty.pretty_print(self.db),
		}
	}

	/// Pretty print an expression
	pub fn pretty_print_expression(&self, expression: &Expression<'db, T>) -> String {
		maybe_grow_stack(|| self.pretty_print_expression_inner(expression))
	}

	fn pretty_print_expression_inner(&self, expression: &Expression<'db, T>) -> String {
		let mut out = match &**expression {
			ExpressionData::Absent => "<>".to_owned(),
			ExpressionData::ArrayComprehension(c) => {
				let mut buf = String::new();
				write!(&mut buf, "[").unwrap();
				if let Some(i) = &c.indices {
					write!(&mut buf, "{}: ", self.pretty_print_expression(i)).unwrap();
				}
				let t = self.pretty_print_expression(&c.template);
				let gs = c
					.generators
					.iter()
					.map(|g| self.pretty_print_generator(g))
					.collect::<Vec<_>>()
					.join(", ");
				write!(&mut buf, "{} | {}]", t, gs).unwrap();
				buf
			}
			ExpressionData::ArrayLiteral(al) => {
				format!(
					"[{}]",
					al.iter()
						.map(|e| self.pretty_print_expression(e))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			ExpressionData::BooleanLiteral(b) => {
				if b.0 {
					"true".to_owned()
				} else {
					"false".to_owned()
				}
			}
			ExpressionData::Call(c) => {
				if self.old_compat
					&& (c.arguments.len() == 1 || c.arguments.len() == 2)
					&& let Callable::Function(f) = &c.function
					&& self.model[*f].body().is_none()
				{
					let name = self.model[*f].name().as_identifier(self.db).lookup(self.db);
					if c.arguments.len() == 1 {
						if matches!(name, "o.." | "o<.." | "o..<" | "o<..<") {
							return format!(
								"({}({}))",
								&name[1..],
								self.pretty_print_expression(&c.arguments[0])
							);
						}
						if shackle_syntax::is_prefix_operator(name) {
							return format!(
								"({}({}))",
								name,
								self.pretty_print_expression(&c.arguments[0])
							);
						}
						if name.len() > 1
							&& shackle_syntax::is_postfix_operator(&name[..name.len() - 1])
						{
							return format!(
								"(({}){})",
								self.pretty_print_expression(&c.arguments[0]),
								&name[..name.len() - 1],
							);
						};
					} else if shackle_syntax::is_infix_operator(name) {
						return format!(
							"(({}) {} ({}))",
							self.pretty_print_expression(&c.arguments[0]),
							name,
							self.pretty_print_expression(&c.arguments[1])
						);
					}
				}

				let f = match &c.function {
					Callable::Annotation(a) => self.model[*a]
						.name
						.map(|n| n.pretty_print(self.db))
						.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(*a))),
					Callable::AnnotationDestructure(a) => self.model[*a]
						.name
						.map(|n| n.inversed(self.db).pretty_print(self.db))
						.unwrap_or_else(|| format!("_ANN_{}⁻¹", Into::<u32>::into(*a))),
					Callable::EnumConstructor(m) => self.model[*m]
						.name
						.map(|n| n.pretty_print(self.db))
						.unwrap_or_else(|| {
							format!(
								"_EM_{}_{}",
								self.model[m.enumeration_id()]
									.enum_type()
									.pretty_print(self.db),
								m.member_index()
							)
						}),
					Callable::EnumDestructor(m) => self.model[*m]
						.name
						.map(|n| n.inversed(self.db).pretty_print(self.db))
						.unwrap_or_else(|| {
							format!(
								"_EM_{}_{}⁻¹",
								self.model[m.enumeration_id()]
									.enum_type()
									.pretty_print(self.db),
								m.member_index()
							)
						}),
					Callable::Function(f) => {
						let name = self.model[*f].name();
						if self.old_compat
							&& (name == self.ids.functions.forall
								|| name == self.ids.functions.exists)
							&& c.arguments.len() == 1
							&& let ExpressionData::ArrayLiteral(al) = &*c.arguments[0]
							&& !al.is_empty()
						{
							// Sometimes the old compiler doesn't make these short-circuit (e.g. inside flat_cv_exp), so turn into and/or
							return al
								.iter()
								.map(|e| format!("({})", self.pretty_print_expression(e)))
								.collect::<Vec<_>>()
								.join(if name == self.ids.functions.forall {
									" /\\ "
								} else {
									" \\/ "
								});
						}
						(|| {
							if let Some(tys) = self.model[*f].mangled_param_tys()
								&& (!self.old_compat || self.model[*f].body().is_some())
							{
								return self.model[*f]
									.name()
									.mangled(self.db, tys.iter().copied())
									.pretty_print(self.db);
							}
							name.pretty_print(self.db)
						})()
					}
					Callable::Expression(e) => format!("({})", self.pretty_print_expression(e)),
				};
				let args = c
					.arguments
					.iter()
					.map(|a| self.pretty_print_expression(a))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", f, args)
			}
			ExpressionData::Case(c) => {
				let branches = c
					.branches
					.iter()
					.map(|b| {
						format!(
							"{} => {}",
							self.pretty_print_pattern(&b.pattern),
							self.pretty_print_expression(&b.result)
						)
					})
					.collect::<Vec<_>>();
				format!(
					"case {} of {} endcase",
					self.pretty_print_expression(&c.scrutinee),
					branches.join(", ")
				)
			}
			ExpressionData::FloatLiteral(f) => {
				let value = f.value();
				if value.fract() == 0.0 {
					// Ensure this is is printed as a float literal and not an integer
					format!("{}.0", value)
				} else {
					format!("{}", value)
				}
			}
			ExpressionData::Identifier(i) => match i {
				ResolvedIdentifier::Annotation(a) => self.model[*a]
					.name
					.map(|n| match &n.pretty_print(self.db)[..] {
						"shackle_mzn_internal_representation" if self.old_compat => {
							"mzn_internal_representation".to_owned()
						}
						"shackle_output_only" if self.old_compat => "output_only".to_owned(),
						"shackle_promise_total" if self.old_compat => "promise_total".to_owned(),
						"shackle_promise_commutative" if self.old_compat => {
							"promise_commutative".to_owned()
						}
						"shackle_promise_ctx_monotone" if self.old_compat => {
							"promise_ctx_monotone".to_owned()
						}
						"shackle_promise_ctx_antitone" if self.old_compat => {
							"promise_ctx_antitone".to_owned()
						}
						"shackle_maybe_partial" if self.old_compat => "maybe_partial".to_owned(),
						"shackle_output" if self.old_compat => "output".to_owned(),
						n => n.to_owned(),
					})
					.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(*a))),

				ResolvedIdentifier::Declaration(d) => self.model[*d]
					.name()
					.map(|n| n.pretty_print(self.db))
					.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*d))),
				ResolvedIdentifier::Enumeration(e) => {
					self.model[*e].enum_type().pretty_print(self.db)
				}
				ResolvedIdentifier::EnumerationMember(m) => self.model[*m]
					.name
					.map(|n| n.pretty_print(self.db))
					.unwrap_or_else(|| {
						format!(
							"_EM_{}_{}",
							self.model[m.enumeration_id()]
								.enum_type()
								.pretty_print(self.db),
							m.member_index()
						)
					}),
			},
			ExpressionData::IfThenElse(ite) => {
				let mut buf = String::new();
				let mut bs = ite.branches.iter();
				let first = bs.next().expect("No branches in if-then-else");
				write!(
					&mut buf,
					"if {} then {} ",
					self.pretty_print_expression(&first.condition),
					self.pretty_print_expression(&first.result)
				)
				.unwrap();
				for branch in bs {
					write!(
						&mut buf,
						"elseif {} then {} ",
						self.pretty_print_expression(&branch.condition),
						self.pretty_print_expression(&branch.result)
					)
					.unwrap();
				}
				write!(
					&mut buf,
					"else {} endif",
					self.pretty_print_expression(&ite.else_result)
				)
				.unwrap();
				buf
			}
			ExpressionData::Infinity => "infinity".to_owned(),
			ExpressionData::IntegerLiteral(i) => format!("{}", i.0),
			ExpressionData::Lambda(l) => format!(
				"lambda {}: ({}) => {}",
				self.pretty_print_domain(self.model[**l].domain()),
				self.model[**l]
					.parameters()
					.iter()
					.map(|p| self.pretty_print_declaration(*p, false, true))
					.collect::<Vec<_>>()
					.join(", "),
				self.pretty_print_expression(self.model[**l].body().unwrap())
			),
			ExpressionData::Let(l) => {
				let mut buf = String::new();
				writeln!(&mut buf, "let {{").unwrap();
				for item in l.items.iter() {
					match item {
						LetItem::Constraint(c) => {
							writeln!(&mut buf, "  {};", self.pretty_print_constraint(*c)).unwrap()
						}
						LetItem::Declaration(d) => writeln!(
							&mut buf,
							"  {};",
							self.pretty_print_declaration(*d, true, false)
						)
						.unwrap(),
					}
				}
				write!(
					&mut buf,
					"}} in {}",
					self.pretty_print_expression(&l.in_expression)
				)
				.unwrap();
				buf
			}
			ExpressionData::RecordAccess(ra) => {
				format!(
					"({}).{}",
					self.pretty_print_expression(&ra.record),
					ra.field.pretty_print(self.db)
				)
			}
			ExpressionData::RecordLiteral(fs) => {
				if self.old_compat && fs.is_empty() {
					return "true".to_owned();
				}
				let pairs = fs
					.iter()
					.map(|(i, e)| {
						format!(
							"{}: {}",
							i.pretty_print(self.db),
							self.pretty_print_expression(e)
						)
					})
					.collect::<Vec<_>>()
					.join(", ");
				format!("({})", pairs)
			}
			ExpressionData::SetComprehension(c) => {
				let mut buf = String::new();
				write!(&mut buf, "{{").unwrap();
				let t = self.pretty_print_expression(&c.template);
				let gs = c
					.generators
					.iter()
					.map(|g| self.pretty_print_generator(g))
					.collect::<Vec<_>>()
					.join(", ");
				write!(&mut buf, "{} | {}}}", t, gs).unwrap();
				buf
			}
			ExpressionData::SetLiteral(sl) => {
				format!(
					"{{{}}}",
					sl.iter()
						.map(|e| self.pretty_print_expression(e))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			ExpressionData::StringLiteral(s) => format!("{:?}", s.value(self.db)),
			ExpressionData::TupleAccess(ta) => {
				format!(
					"({}).{}",
					self.pretty_print_expression(&ta.tuple),
					ta.field.0
				)
			}
			ExpressionData::TupleLiteral(fs) => {
				let fields = fs
					.iter()
					.map(|f| self.pretty_print_expression(f))
					.collect::<Vec<_>>()
					.join(", ");
				let end = if fs.len() <= 1 { "," } else { "" };
				format!("({}{})", fields, end)
			}
		};
		for ann in expression.annotations().iter() {
			write!(&mut out, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if let Some(f) = &self.expression_annotator
			&& let Some(v) = f(expression)
		{
			write!(&mut out, ":: {}", v).unwrap();
		}
		out
	}

	fn pretty_print_generator(&self, g: &Generator<'db, T>) -> String {
		let (mut gtor, w) = match g {
			Generator::Iterator {
				declarations,
				collection,
				where_clause,
			} => {
				let decls = declarations
					.iter()
					.map(|d| {
						self.model[*d]
							.name()
							.map(|n| n.pretty_print(self.db))
							.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*d)))
					})
					.collect::<Vec<_>>()
					.join(", ");
				(
					format!("{} in {}", decls, self.pretty_print_expression(collection)),
					where_clause,
				)
			}
			Generator::Assignment {
				assignment,
				where_clause,
			} => {
				let decl = &self.model[*assignment];
				(
					format!(
						"{}{} = {}",
						decl.name()
							.map(|n| n.pretty_print(self.db))
							.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*assignment))),
						if !self.old_compat {
							decl.annotations()
								.iter()
								.map(|ann| format!(" :: ({})", self.pretty_print_expression(ann)))
								.collect::<Vec<_>>()
								.join("")
						} else {
							"".to_owned()
						},
						self.pretty_print_expression(decl.definition().unwrap())
					),
					where_clause,
				)
			}
		};
		if let Some(where_clause) = w {
			write!(
				&mut gtor,
				" where {}",
				self.pretty_print_expression(where_clause)
			)
			.unwrap();
		}
		gtor
	}

	fn pretty_print_pattern(&self, pat: &Pattern<'db, T>) -> String {
		match &**pat {
			PatternData::Anonymous(_) => "_".to_owned(),
			PatternData::Expression(e) => self.pretty_print_expression(e),
			PatternData::Tuple(fs) => format!(
				"({})",
				fs.iter()
					.map(|p| self.pretty_print_pattern(p))
					.collect::<Vec<_>>()
					.join(", ")
			),
			PatternData::Record(fs) => {
				if self.old_compat && fs.is_empty() {
					"_".to_owned()
				} else {
					format!(
						"({})",
						fs.iter()
							.map(|(i, p)| format!(
								"{}: {}",
								i.pretty_print(self.db),
								self.pretty_print_pattern(p)
							))
							.collect::<Vec<_>>()
							.join(", ")
					)
				}
			}
			PatternData::EnumConstructor { member, args, .. } => {
				let ctor = self.model[*member].name.unwrap().pretty_print(self.db);
				let ps = args
					.iter()
					.map(|p| self.pretty_print_pattern(p))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", ctor, ps)
			}
			PatternData::AnnotationConstructor { item, args } => {
				let ctor = self.model[*item].name.unwrap().pretty_print(self.db);
				let ps = args
					.iter()
					.map(|p| self.pretty_print_pattern(p))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", ctor, ps)
			}
		}
	}
}
