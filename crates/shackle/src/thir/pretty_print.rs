//! Pretty printing of THIR as MiniZinc
//!

use crate::arena::ArenaIndex;

use super::db::Thir;
use super::{
	AnnotationId, ConstraintId, DeclarationId, Domain, DomainData, EnumerationId, Expression,
	ExpressionAllocator, ExpressionData, FunctionId, Generator, Goal, ItemId, LetItem, Model,
	OutputId, Pattern, ResolvedIdentifier,
};
use std::fmt::Write;

/// Pretty prints THIR as MiniZinc
pub struct PrettyPrinter<'a> {
	db: &'a dyn Thir,
	model: &'a Model,
	/// Whether to output a model compatible with old MiniZinc (default `true`)
	pub old_compat: bool,
	/// Whether to output `shackle_type("...")` annotations for sanity checking
	pub debug_types: bool,
}

impl<'a> PrettyPrinter<'a> {
	/// Create a new pretty printer
	pub fn new(db: &'a dyn Thir, model: &'a Model) -> Self {
		Self {
			db,
			model,
			old_compat: true,
			debug_types: false,
		}
	}
	/// Pretty print the model
	pub fn pretty_print(&self) -> String {
		let mut buf = String::new();
		for item in self.model.top_level_items() {
			writeln!(&mut buf, "{};", self.pretty_print_item(item)).unwrap();
		}
		if self.model.solve().is_none() {
			writeln!(&mut buf, "solve satisfy;").unwrap();
		}
		buf
	}

	/// Pretty print an item from a model
	pub fn pretty_print_item(&self, item: ItemId) -> String {
		match item {
			ItemId::Annotation(i) => self.pretty_print_annotation(i),
			ItemId::Constraint(i) => self.pretty_print_constraint(i),
			ItemId::Declaration(i) => self.pretty_print_declaration(i, false),
			ItemId::Enumeration(i) => self.pretty_print_enumeration(i),
			ItemId::Function(i) => self.pretty_print_function(i),
			ItemId::Output(i) => self.pretty_print_output(i),
			ItemId::Solve => self.pretty_print_solve(),
		}
	}

	fn pretty_print_annotation(&self, idx: AnnotationId) -> String {
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
					.map(|p| self.pretty_print_declaration(*p, false))
					.collect::<Vec<_>>()
					.join(", ")
			)
			.unwrap();
		}
		buf
	}

	fn pretty_print_constraint(&self, idx: ConstraintId) -> String {
		let constraint = &self.model[idx];
		let mut buf = "constraint ".to_owned();
		for ann in constraint.annotations() {
			write!(
				&mut buf,
				":: ({}) ",
				self.pretty_print_expression(ann, constraint.expressions())
			)
			.unwrap();
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(constraint.expression(), constraint.expressions())
		)
		.unwrap();
		buf
	}

	fn pretty_print_declaration(&self, idx: DeclarationId, is_let_item: bool) -> String {
		let declaration = &self.model[idx];
		let ty = declaration.ty();
		let mut buf = if is_let_item
			&& ty.contains_type_inst_var(self.db.upcast())
			&& declaration.definition().is_some()
		{
			// Workaround since let items can't use TiIDs in MiniZinc
			"any".to_owned()
		} else {
			self.pretty_print_domain(declaration.domain(), declaration.expressions())
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
		for ann in declaration.annotations() {
			write!(
				&mut buf,
				" :: ({})",
				self.pretty_print_expression(ann, declaration.expressions())
			)
			.unwrap();
		}
		if let Some(def) = declaration.definition() {
			write!(
				&mut buf,
				" = {}",
				self.pretty_print_expression(def, declaration.expressions())
			)
			.unwrap();
		}
		buf
	}

	fn pretty_print_enumeration(&self, idx: EnumerationId) -> String {
		let enumeration = &self.model[idx];
		let enum_name = enumeration.enum_type().pretty_print(self.db.upcast());
		let mut buf = format!("enum {}", enum_name);
		for ann in enumeration.annotations() {
			write!(
				&mut buf,
				" :: ({})",
				self.pretty_print_expression(ann, enumeration.expressions())
			)
			.unwrap();
		}
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
									.map(|d| {
										self.pretty_print_domain(
											self.model[*d].domain(),
											self.model[*d].expressions(),
										)
									})
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
	fn pretty_print_function(&self, idx: FunctionId) -> String {
		let function = &self.model[idx];
		let mut buf = String::new();
		write!(
			&mut buf,
			"function {}: {}({})",
			self.pretty_print_domain(function.domain(), function.expressions()),
			function.name().pretty_print(self.db.upcast()),
			function
				.parameters()
				.iter()
				.map(|p| self.pretty_print_declaration(*p, false))
				.collect::<Vec<_>>()
				.join(", ")
		)
		.unwrap();
		for ann in function.annotations() {
			write!(
				&mut buf,
				" :: ({})",
				self.pretty_print_expression(ann, function.expressions())
			)
			.unwrap();
		}
		if let Some(body) = function.body() {
			if self.old_compat
				&& function.name().lookup(self.db.upcast()) == "deopt"
				&& !function.type_inst_vars().is_empty()
				&& function.parameters().len() == 1
				&& {
					let ty = self.model[function.parameter(0)].ty();
					!ty.known_par(self.db.upcast()) && !ty.known_occurs(self.db.upcast())
				} {
				// For compatibility with old minizinc, we can just directly coerce
				match &*function[body] {
					ExpressionData::Call {
						function: f,
						arguments: args,
					} => match &*function[*f] {
						ExpressionData::Identifier(ResolvedIdentifier::Function(idx)) => {
							assert_eq!(self.model[*idx].name().lookup(self.db.upcast()), "to_enum");
							assert_eq!(args.len(), 2);
							write!(
								&mut buf,
								" = {}",
								self.pretty_print_expression(args[1], function.expressions())
							)
							.unwrap();
						}
						_ => unreachable!(),
					},
					_ => unreachable!(),
				}
			} else {
				write!(
					&mut buf,
					" = {}",
					self.pretty_print_expression(body, function.expressions())
				)
				.unwrap();
			}
		} else if self.old_compat && function.name().lookup(self.db.upcast()) == "erase_enum" {
			// For compatibility with old minizinc, we can just directly coerce
			let d = function.parameter(0);
			let ident = self.model[d]
				.name()
				.map(|n| n.pretty_print(self.db.upcast()))
				.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(d)));
			write!(&mut buf, " = {}", ident).unwrap();
		}
		buf
	}

	fn pretty_print_output(&self, idx: OutputId) -> String {
		let output = &self.model[idx];
		let mut buf = "output ".to_owned();
		if let Some(s) = output.section() {
			write!(
				&mut buf,
				":: {} ",
				self.pretty_print_expression(s, output.expressions())
			)
			.unwrap();
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(output.expression(), output.expressions())
		)
		.unwrap();
		buf
	}

	fn pretty_print_solve(&self) -> String {
		let solve = self.model.solve().unwrap();
		let mut buf = "solve ".to_owned();
		for ann in solve.annotations() {
			write!(
				&mut buf,
				":: ({}) ",
				self.pretty_print_expression(ann, solve.expressions())
			)
			.unwrap();
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
	pub fn pretty_print_domain(&self, domain: &Domain, data: &ExpressionAllocator) -> String {
		let ty = domain.ty();
		match &**domain {
			DomainData::Array(dim, el) => {
				let dims = match &***dim {
					DomainData::Tuple(ds) => ds
						.iter()
						.map(|d| self.pretty_print_domain(d, data))
						.collect::<Vec<_>>()
						.join(", "),
					_ => self.pretty_print_domain(dim, data),
				};
				ty.opt(self.db.upcast())
					.into_iter()
					.flat_map(|o| o.pretty_print())
					.chain([format!(
						"array [{}] of {}",
						dims,
						self.pretty_print_domain(el, data)
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
				.chain([self.pretty_print_expression(*e, data)])
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
				.chain([self.pretty_print_domain(s, data)])
				.collect::<Vec<_>>()
				.join(" "),
			DomainData::Tuple(ds) => {
				let doms = ds
					.iter()
					.map(|d| self.pretty_print_domain(d, data))
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
							self.pretty_print_domain(d, data),
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
	pub fn pretty_print_expression(
		&self,
		idx: ArenaIndex<Expression>,
		data: &ExpressionAllocator,
	) -> String {
		let mut out = match &*data[idx] {
			ExpressionData::Absent => "<>".to_owned(),
			ExpressionData::ArrayAccess {
				collection,
				indices,
			} => format!(
				"({})[{}]",
				self.pretty_print_expression(*collection, data),
				match &*data[*indices] {
					ExpressionData::TupleLiteral(es) => es
						.iter()
						.map(|e| self.pretty_print_expression(*e, data))
						.collect::<Vec<_>>()
						.join(", "),
					_ => self.pretty_print_expression(*indices, data),
				}
			),
			ExpressionData::ArrayComprehension {
				template,
				indices,
				generators,
			} => {
				let mut buf = String::new();
				write!(&mut buf, "[").unwrap();
				if let Some(i) = indices {
					write!(&mut buf, "{}: ", self.pretty_print_expression(*i, data)).unwrap();
				}
				let t = self.pretty_print_expression(*template, data);
				let gs = generators
					.iter()
					.map(|g| self.pretty_print_generator(g, data))
					.collect::<Vec<_>>()
					.join(", ");
				write!(&mut buf, "{} | {}]", t, gs).unwrap();
				buf
			}
			ExpressionData::ArrayLiteral(al) => {
				format!(
					"[{}]",
					al.iter()
						.map(|e| self.pretty_print_expression(*e, data))
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
			ExpressionData::Call {
				function,
				arguments,
			} => {
				let f = self.pretty_print_expression(*function, data);
				let args = arguments
					.iter()
					.map(|a| self.pretty_print_expression(*a, data))
					.collect::<Vec<_>>()
					.join(", ");
				if self.old_compat {
					format!("{}({})", f, args)
				} else {
					format!("({})({})", f, args)
				}
			}
			ExpressionData::Case {
				scrutinee,
				branches,
			} => {
				let branches = branches
					.iter()
					.map(|b| {
						format!(
							"{} => {}",
							self.pretty_print_pattern(&b.pattern, data),
							self.pretty_print_expression(b.result, data)
						)
					})
					.collect::<Vec<_>>();
				format!(
					"case {} of {} endcase",
					self.pretty_print_expression(*scrutinee, data),
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
				ResolvedIdentifier::AnnotationDestructure(a) => self.model[*a]
					.name
					.map(|n| n.inversed(self.db.upcast()).pretty_print(self.db.upcast()))
					.unwrap_or_else(|| format!("_ANN_{}⁻¹", Into::<u32>::into(*a))),
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
				ResolvedIdentifier::EnumerationDestructure(m) => self.model[*m]
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
				ResolvedIdentifier::Function(f) => {
					self.model[*f].name().pretty_print(self.db.upcast())
				}
				ResolvedIdentifier::TyVarRef(t) => {
					self.model[*t].ty_var.pretty_print(self.db.upcast())
				}
			},
			ExpressionData::IfThenElse {
				branches,
				else_result,
			} => {
				let mut buf = String::new();
				let mut bs = branches.iter();
				let first = bs.next().expect("No branches in if-then-else");
				write!(
					&mut buf,
					"if {} then {} ",
					self.pretty_print_expression(first.condition, data),
					self.pretty_print_expression(first.result, data)
				)
				.unwrap();
				for branch in bs {
					write!(
						&mut buf,
						"elseif {} then {} ",
						self.pretty_print_expression(branch.condition, data),
						self.pretty_print_expression(branch.result, data)
					)
					.unwrap();
				}
				write!(
					&mut buf,
					"else {} endif",
					self.pretty_print_expression(*else_result, data)
				)
				.unwrap();
				buf
			}
			ExpressionData::Infinity => "infinity".to_owned(),
			ExpressionData::IntegerLiteral(i) => format!("{}", i.0),
			ExpressionData::Lambda {
				domain,
				parameters,
				body,
			} => format!(
				"lambda {}: ({}) => {}",
				self.pretty_print_domain(domain, data),
				parameters
					.iter()
					.map(|p| self.pretty_print_declaration(*p, false))
					.collect::<Vec<_>>()
					.join(", "),
				self.pretty_print_expression(*body, data)
			),
			ExpressionData::Let {
				items,
				in_expression,
			} => {
				let mut buf = String::new();
				writeln!(&mut buf, "let {{").unwrap();
				for item in items.iter() {
					match item {
						LetItem::Constraint(c) => {
							writeln!(&mut buf, "  {};", self.pretty_print_constraint(*c)).unwrap()
						}
						LetItem::Declaration(d) => {
							writeln!(&mut buf, "  {};", self.pretty_print_declaration(*d, true))
								.unwrap()
						}
					}
				}
				write!(
					&mut buf,
					"}} in {}",
					self.pretty_print_expression(*in_expression, data)
				)
				.unwrap();
				buf
			}
			ExpressionData::RecordAccess { record, field } => {
				format!(
					"{}.{}",
					self.pretty_print_expression(*record, data),
					field.pretty_print(self.db.upcast())
				)
			}
			ExpressionData::RecordLiteral(fs) => {
				let pairs = fs
					.iter()
					.map(|(i, e)| {
						format!(
							"{}: {}",
							i.pretty_print(self.db.upcast()),
							self.pretty_print_expression(*e, data)
						)
					})
					.collect::<Vec<_>>()
					.join(", ");
				format!("({})", pairs)
			}
			ExpressionData::SetComprehension {
				template,
				generators,
			} => {
				let mut buf = String::new();
				write!(&mut buf, "{{").unwrap();
				let t = self.pretty_print_expression(*template, data);
				let gs = generators
					.iter()
					.map(|g| self.pretty_print_generator(g, data))
					.collect::<Vec<_>>()
					.join(", ");
				write!(&mut buf, "{} | {}}}", t, gs).unwrap();
				buf
			}
			ExpressionData::SetLiteral(sl) => {
				format!(
					"{{{}}}",
					sl.iter()
						.map(|e| self.pretty_print_expression(*e, data))
						.collect::<Vec<_>>()
						.join(", ")
				)
			}
			ExpressionData::StringLiteral(s) => format!("{:?}", s.value(self.db.upcast())),
			ExpressionData::TupleAccess { tuple, field } => {
				format!("{}.{}", self.pretty_print_expression(*tuple, data), field.0)
			}
			ExpressionData::TupleLiteral(fs) => {
				let fields = fs
					.iter()
					.map(|f| self.pretty_print_expression(*f, data))
					.collect::<Vec<_>>()
					.join(", ");
				format!("({})", fields)
			}
		};
		for ann in data.expression_annotations(idx) {
			write!(
				&mut out,
				" :: ({})",
				self.pretty_print_expression(ann, data)
			)
			.unwrap();
		}
		if self.debug_types {
			write!(
				&mut out,
				":: shackle_type({:?})",
				&*data[idx].ty().pretty_print(self.db.upcast())
			)
			.unwrap();
		}
		out
	}

	fn pretty_print_generator(&self, g: &Generator, data: &ExpressionAllocator) -> String {
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
					format!(
						"{} in {}",
						decls,
						self.pretty_print_expression(*collection, data)
					),
					*where_clause,
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
						self.pretty_print_expression(
							decl.definition().unwrap(),
							decl.expressions()
						)
					),
					*where_clause,
				)
			}
		};
		if let Some(where_clause) = w {
			write!(
				&mut gen,
				" where {}",
				self.pretty_print_expression(where_clause, data)
			)
			.unwrap();
		}
		gen
	}

	fn pretty_print_pattern(&self, pat: &Pattern, data: &ExpressionAllocator) -> String {
		match pat {
			Pattern::Anonymous(_) => "_".to_owned(),
			Pattern::Expression(e) => self.pretty_print_expression(*e, data),
			Pattern::Tuple(fs) => format!(
				"({})",
				fs.iter()
					.map(|p| self.pretty_print_pattern(p, data))
					.collect::<Vec<_>>()
					.join(", ")
			),
			Pattern::Record(fs) => format!(
				"({})",
				fs.iter()
					.map(|(i, p)| format!(
						"{}: {}",
						i.pretty_print(self.db.upcast()),
						self.pretty_print_pattern(p, data)
					))
					.collect::<Vec<_>>()
					.join(", ")
			),
			Pattern::EnumConstructor { member, args, .. } => {
				let ctor = self.model[*member]
					.name
					.unwrap()
					.pretty_print(self.db.upcast());
				let ps = args
					.iter()
					.map(|p| self.pretty_print_pattern(p, data))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", ctor, ps)
			}
			Pattern::AnnotationConstructor { item, args } => {
				let ctor = self.model[*item]
					.name
					.unwrap()
					.pretty_print(self.db.upcast());
				let ps = args
					.iter()
					.map(|p| self.pretty_print_pattern(p, data))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", ctor, ps)
			}
		}
	}
}
