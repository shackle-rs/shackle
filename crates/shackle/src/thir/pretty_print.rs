//! Pretty printing of THIR as MiniZinc
//!

use super::db::Thir;
use super::{
	AnnotationId, Callable, ConstraintId, DeclarationId, Domain, DomainData, EnumerationId,
	Expression, ExpressionData, FunctionId, Generator, Goal, ItemId, LetItem, Marker, Model,
	OutputId, Pattern, PatternData, ResolvedIdentifier,
};
use std::fmt::Write;

static MINIZINC_COMPAT: &str = include_str!("../../../../share/minizinc/compat.mzn");

/// Pretty prints THIR as MiniZinc
pub struct PrettyPrinter<'a, T = ()> {
	db: &'a dyn Thir,
	model: &'a Model<T>,
	/// Whether to output a model compatible with old MiniZinc (default `true`)
	pub old_compat: bool,
	/// Whether to output `shackle_type("...")` annotations for sanity checking
	pub debug_types: bool,
}

impl<'a, T: Marker> PrettyPrinter<'a, T> {
	/// Create a new pretty printer
	pub fn new(db: &'a dyn Thir, model: &'a Model<T>) -> Self {
		Self {
			db,
			model,
			old_compat: false,
			debug_types: false,
		}
	}

	/// Create a new pretty printer which prints output compatible with old MiniZinc
	pub fn new_compat(db: &'a dyn Thir, model: &'a Model<T>) -> Self {
		let mut printer = Self::new(db, model);
		printer.old_compat = true;
		printer
	}

	/// Pretty print the model
	pub fn pretty_print(&self) -> String {
		let ids = self.db.identifier_registry();
		let mut buf = String::new();
		for item in self.model.top_level_items() {
			if self.old_compat {
				match item {
					ItemId::Function(f) if self.model[f].name() == ids.default => {
						continue;
					}
					ItemId::Annotation(a) if self.model[a].name == Some(ids.output) => {
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
	pub fn pretty_print_signature(&self, item: ItemId<T>) -> String {
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
	pub fn pretty_print_item(&self, item: ItemId<T>) -> String {
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

	fn pretty_print_annotation(&self, idx: AnnotationId<T>) -> String {
		let annotation = &self.model[idx];
		let name = annotation
			.name
			.map(|i| i.pretty_print(self.db.upcast()))
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

	fn pretty_print_constraint(&self, idx: ConstraintId<T>) -> String {
		let constraint = &self.model[idx];
		let mut buf = "constraint ".to_owned();
		for ann in constraint.annotations().iter() {
			write!(&mut buf, ":: ({}) ", self.pretty_print_expression(ann)).unwrap();
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
		idx: DeclarationId<T>,
		is_let_item: bool,
		signature_only: bool,
	) -> String {
		let declaration = &self.model[idx];
		let ty = declaration.ty();
		let mut buf = if is_let_item
			&& ty.contains_type_inst_var(self.db.upcast())
			&& declaration.definition().is_some()
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
				.map(|name| if name.lookup(self.db.upcast()) == "_objective" {
					"_DECL_OBJ".to_owned()
				} else {
					name.pretty_print(self.db.upcast())
				})
				.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(idx)))
		)
		.unwrap();
		for ann in declaration.annotations().iter() {
			write!(&mut buf, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if !signature_only {
			if let Some(def) = declaration.definition() {
				write!(&mut buf, " = {}", self.pretty_print_expression(def)).unwrap();
			}
		}
		buf
	}

	fn pretty_print_enumeration(&self, idx: EnumerationId<T>, signature_only: bool) -> String {
		let enumeration = &self.model[idx];
		let enum_name = enumeration.enum_type().pretty_print(self.db.upcast());
		let mut buf = format!("enum {}", enum_name);
		for ann in enumeration.annotations().iter() {
			write!(&mut buf, " :: ({})", self.pretty_print_expression(ann)).unwrap();
		}
		if !signature_only {
			if let Some(cases) = enumeration.definition() {
				write!(
					&mut buf,
					" = {}",
					cases
						.iter()
						.enumerate()
						.map(|(i, c)| {
							let name = c
								.name
								.map(|n| n.pretty_print(self.db.upcast()))
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
		}
		buf
	}
	fn pretty_print_function(&self, idx: FunctionId<T>, signature_only: bool) -> String {
		let function = &self.model[idx];
		let mut buf = String::new();
		write!(
			&mut buf,
			"function {}: {}({})",
			self.pretty_print_domain(function.domain()),
			function.name().pretty_print(self.db),
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
		if !signature_only {
			if let Some(body) = function.body() {
				if self.old_compat
					&& function.name() == self.db.identifier_registry().deopt
					&& !function.type_inst_vars().is_empty()
					&& function.parameters().len() == 1
					&& {
						let ty = self.model[function.parameter(0)].ty();
						!ty.known_par(self.db.upcast()) && !ty.known_occurs(self.db.upcast())
					} {
					// For compatibility with old minizinc, we can just directly coerce
					match &**body {
						ExpressionData::Call(c) => match &c.function {
							Callable::Function(idx) => {
								assert_eq!(
									self.model[*idx].name(),
									self.db.identifier_registry().to_enum
								);
								assert_eq!(c.arguments.len(), 2);
								write!(
									&mut buf,
									" = {}",
									self.pretty_print_expression(&c.arguments[1])
								)
								.unwrap();
							}
							_ => unreachable!(),
						},
						_ => unreachable!(),
					}
				} else {
					write!(&mut buf, " = {}", self.pretty_print_expression(body)).unwrap();
				}
			} else if self.old_compat && function.name() == self.db.identifier_registry().erase_enum
			{
				// For compatibility with old minizinc, we can just directly coerce
				let d = function.parameter(0);
				let ident = self.model[d]
					.name()
					.map(|n| n.pretty_print(self.db.upcast()))
					.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(d)));
				write!(&mut buf, " = {}", ident).unwrap();
			}
		}
		buf
	}

	fn pretty_print_output(&self, idx: OutputId<T>) -> String {
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
	pub fn pretty_print_domain(&self, domain: &Domain<T>) -> String {
		let ty = domain.ty();
		match &**domain {
			DomainData::Array(dim, el) => {
				let dims = match &***dim {
					DomainData::Tuple(ds) => ds
						.iter()
						.map(|d| self.pretty_print_domain(d))
						.collect::<Vec<_>>()
						.join(", "),
					DomainData::Unbounded => ty
						.dim_ty(self.db.upcast())
						.unwrap()
						.pretty_print_as_dims(self.db.upcast()),
					_ => self.pretty_print_domain(dim),
				};
				ty.opt(self.db.upcast())
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
				.inst(self.db.upcast())
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(
					ty.opt(self.db.upcast())
						.into_iter()
						.flat_map(|o| o.pretty_print()),
				)
				.chain([self.pretty_print_expression(e)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Set(s) => ty
				.inst(self.db.upcast())
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(
					ty.opt(self.db.upcast())
						.into_iter()
						.flat_map(|o| o.pretty_print()),
				)
				.chain(["set of".to_owned()])
				.chain([self.pretty_print_domain(s)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Tuple(ds) => {
				let doms = ds
					.iter()
					.map(|d| self.pretty_print_domain(d))
					.collect::<Vec<_>>()
					.join(", ");
				ty.inst(self.db.upcast())
					.into_iter()
					.flat_map(|i| i.pretty_print())
					.chain(
						ty.opt(self.db.upcast())
							.into_iter()
							.flat_map(|o| o.pretty_print()),
					)
					.chain([format!("tuple({})", doms)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			DomainData::Record(ds) => {
				let doms = ds
					.iter()
					.map(|(i, d)| {
						format!(
							"{}: {}",
							self.pretty_print_domain(d),
							i.pretty_print(self.db.upcast())
						)
					})
					.collect::<Vec<_>>()
					.join(", ");
				ty.inst(self.db.upcast())
					.into_iter()
					.flat_map(|i| i.pretty_print())
					.chain(
						ty.opt(self.db.upcast())
							.into_iter()
							.flat_map(|o| o.pretty_print()),
					)
					.chain([format!("record({})", doms)])
					.collect::<Vec<_>>()
					.join(" ")
			}
			DomainData::Unbounded => ty.pretty_print(self.db.upcast()),
		}
	}

	/// Pretty print an expression
	pub fn pretty_print_expression(&self, expression: &Expression<T>) -> String {
		let mut out = match &**expression {
			ExpressionData::Absent => "<>".to_owned(),
			ExpressionData::ArrayAccess(aa) => format!(
				"({})[{}]",
				self.pretty_print_expression(&aa.collection),
				match &**aa.indices {
					ExpressionData::TupleLiteral(es) => es
						.iter()
						.map(|e| self.pretty_print_expression(e))
						.collect::<Vec<_>>()
						.join(", "),
					_ => self.pretty_print_expression(&aa.indices),
				}
			),
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
				let f = match &c.function {
					Callable::Annotation(a) => self.model[*a]
						.name
						.map(|n| n.pretty_print(self.db.upcast()))
						.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(*a))),
					Callable::AnnotationDestructure(a) => self.model[*a]
						.name
						.map(|n| n.inversed(self.db.upcast()).pretty_print(self.db.upcast()))
						.unwrap_or_else(|| format!("_ANN_{}⁻¹", Into::<u32>::into(*a))),
					Callable::EnumConstructor(m) => self.model[*m]
						.name
						.map(|n| n.pretty_print(self.db.upcast()))
						.unwrap_or_else(|| {
							format!(
								"_EM_{}_{}",
								self.model[m.enumeration_id()]
									.enum_type()
									.pretty_print(self.db.upcast()),
								m.member_index()
							)
						}),
					Callable::EnumDestructor(m) => self.model[*m]
						.name
						.map(|n| n.inversed(self.db.upcast()).pretty_print(self.db.upcast()))
						.unwrap_or_else(|| {
							format!(
								"_EM_{}_{}⁻¹",
								self.model[m.enumeration_id()]
									.enum_type()
									.pretty_print(self.db.upcast()),
								m.member_index()
							)
						}),
					Callable::Function(f) => self.model[*f].name().pretty_print(self.db),
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
					.map(|n| n.pretty_print(self.db.upcast()))
					.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(*a))),

				ResolvedIdentifier::Declaration(d) => self.model[*d]
					.name()
					.map(|n| n.pretty_print(self.db.upcast()))
					.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*d))),
				ResolvedIdentifier::Enumeration(e) => {
					self.model[*e].enum_type().pretty_print(self.db.upcast())
				}
				ResolvedIdentifier::EnumerationMember(m) => self.model[*m]
					.name
					.map(|n| n.pretty_print(self.db.upcast()))
					.unwrap_or_else(|| {
						format!(
							"_EM_{}_{}",
							self.model[m.enumeration_id()]
								.enum_type()
								.pretty_print(self.db.upcast()),
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
					"{}.{}",
					self.pretty_print_expression(&ra.record),
					ra.field.pretty_print(self.db.upcast())
				)
			}
			ExpressionData::RecordLiteral(fs) => {
				let pairs = fs
					.iter()
					.map(|(i, e)| {
						format!(
							"{}: {}",
							i.pretty_print(self.db.upcast()),
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
			ExpressionData::StringLiteral(s) => format!("{:?}", s.value(self.db.upcast())),
			ExpressionData::TupleAccess(ta) => {
				format!("{}.{}", self.pretty_print_expression(&ta.tuple), ta.field.0)
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
		if self.debug_types {
			write!(
				&mut out,
				":: shackle_type({:?})",
				expression.ty().pretty_print(self.db.upcast())
			)
			.unwrap();
		}
		out
	}

	fn pretty_print_generator(&self, g: &Generator<T>) -> String {
		let (mut gen, w) = match g {
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
							.map(|n| n.pretty_print(self.db.upcast()))
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
						"{} = {}",
						decl.name()
							.map(|n| n.pretty_print(self.db.upcast()))
							.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*assignment))),
						self.pretty_print_expression(decl.definition().unwrap())
					),
					where_clause,
				)
			}
		};
		if let Some(where_clause) = w {
			write!(
				&mut gen,
				" where {}",
				self.pretty_print_expression(where_clause)
			)
			.unwrap();
		}
		gen
	}

	fn pretty_print_pattern(&self, pat: &Pattern<T>) -> String {
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
			PatternData::Record(fs) => format!(
				"({})",
				fs.iter()
					.map(|(i, p)| format!(
						"{}: {}",
						i.pretty_print(self.db.upcast()),
						self.pretty_print_pattern(p)
					))
					.collect::<Vec<_>>()
					.join(", ")
			),
			PatternData::EnumConstructor { member, args, .. } => {
				let ctor = self.model[*member]
					.name
					.unwrap()
					.pretty_print(self.db.upcast());
				let ps = args
					.iter()
					.map(|p| self.pretty_print_pattern(p))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", ctor, ps)
			}
			PatternData::AnnotationConstructor { item, args } => {
				let ctor = self.model[*item]
					.name
					.unwrap()
					.pretty_print(self.db.upcast());
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
