//! Pretty printing of THIR as MiniZinc
//!

use crate::arena::ArenaIndex;

use super::db::Thir;
use super::{
	Annotation, Constraint, Declaration, Domain, Enumeration, Expression, ExpressionData, Function,
	Goal, Item, ItemData, ItemId, LetItem, Model, Output, ResolvedIdentifier,
};
use std::fmt::Write;

/// Pretty prints THIR as MiniZinc
pub struct PrettyPrinter<'a> {
	db: &'a dyn Thir,
	model: &'a Model,
}

impl<'a> PrettyPrinter<'a> {
	/// Create a new pretty printer
	pub fn new(db: &'a dyn Thir, model: &'a Model) -> Self {
		Self { db, model }
	}

	/// Pretty print the model
	pub fn pretty_print(&self) -> String {
		let mut buf = String::new();
		for item in self.model.top_level.iter() {
			writeln!(&mut buf, "{};", self.pretty_print_item(*item)).unwrap();
		}
		writeln!(&mut buf, "{};", self.pretty_print_solve()).unwrap();
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
		}
	}

	fn pretty_print_annotation(&self, idx: ArenaIndex<Item<Annotation>>) -> String {
		let annotation = &self.model[idx];
		let name = annotation
			.name
			.expect("Annotation has no name")
			.pretty_print(self.db.upcast());
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

	fn pretty_print_constraint(&self, idx: ArenaIndex<Item<Constraint>>) -> String {
		let constraint = &self.model[idx];
		let mut buf = "constraint ".to_owned();
		for ann in constraint.annotations.iter() {
			write!(
				&mut buf,
				":: {} ",
				self.pretty_print_expression(*ann, &constraint.data)
			)
			.unwrap();
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(constraint.expression, &constraint.data)
		)
		.unwrap();
		buf
	}

	fn pretty_print_declaration(
		&self,
		idx: ArenaIndex<Item<Declaration>>,
		is_let_item: bool,
	) -> String {
		let declaration = &self.model[idx];
		let ty = declaration.domain.ty();
		let mut buf = if is_let_item
			&& ty.contains_type_inst_var(self.db.upcast())
			&& declaration.definition.is_some()
		{
			// Workaround since let items can't use TiIDs in MiniZinc
			"any".to_owned()
		} else {
			self.pretty_print_domain(&declaration.domain, &declaration.data)
		};
		write!(
			&mut buf,
			": {}",
			declaration
				.name
				.map(|name| if name.lookup(self.db.upcast()) == "_objective" {
					"_DECL_OBJ".to_owned()
				} else {
					name.pretty_print(self.db.upcast())
				})
				.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(idx)))
		)
		.unwrap();
		for ann in declaration.annotations.iter() {
			write!(
				&mut buf,
				" :: {}",
				self.pretty_print_expression(*ann, &declaration.data)
			)
			.unwrap();
		}
		if let Some(def) = declaration.definition {
			write!(
				&mut buf,
				" = {}",
				self.pretty_print_expression(def, &declaration.data)
			)
			.unwrap();
		}
		buf
	}

	fn pretty_print_enumeration(&self, idx: ArenaIndex<Item<Enumeration>>) -> String {
		let enumeration = &self.model[idx];
		let mut buf = format!(
			"enum {}",
			enumeration.enum_type.pretty_print(self.db.upcast())
		);
		for ann in enumeration.annotations.iter() {
			write!(
				&mut buf,
				" :: {}",
				self.pretty_print_expression(*ann, &enumeration.data)
			)
			.unwrap();
		}
		if let Some(cases) = &enumeration.definition {
			write!(
				&mut buf,
				" = {}",
				cases
					.iter()
					.map(|c| {
						let name = c
							.name
							.map(|n| n.pretty_print(self.db.upcast()))
							.unwrap_or_else(|| "_".to_owned());

						match &c.parameters {
							Some(ps) => {
								let params = ps
									.iter()
									.map(|d| {
										self.pretty_print_domain(
											&self.model[*d].domain,
											&self.model[*d].data,
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
	fn pretty_print_function(&self, idx: ArenaIndex<Item<Function>>) -> String {
		let function = &self.model[idx];
		let mut buf = String::new();
		write!(
			&mut buf,
			"function {}: {}({})",
			self.pretty_print_domain(&function.domain, &function.data),
			function.name.pretty_print(self.db.upcast()),
			function
				.parameters
				.iter()
				.map(|p| self.pretty_print_declaration(*p, false))
				.collect::<Vec<_>>()
				.join(", ")
		)
		.unwrap();
		for ann in function.annotations.iter() {
			write!(
				&mut buf,
				" :: {}",
				self.pretty_print_expression(*ann, &function.data)
			)
			.unwrap();
		}
		if let Some(body) = function.body {
			if function.name.lookup(self.db.upcast()) == "deopt"
				&& !function.type_inst_vars.is_empty()
				&& function.parameters.len() == 1
				&& {
					let ty = self.model[function.parameters[0]].domain.ty();
					!ty.known_par(self.db.upcast()) && !ty.known_occurs(self.db.upcast())
				} {
				// For compatibility with old minizinc, we can just directly coerce
				match &*function.data.expressions[body] {
					ExpressionData::Call {
						function: f,
						arguments: args,
					} => match &*function.data.expressions[*f] {
						ExpressionData::Identifier(ResolvedIdentifier::Function(idx)) => {
							assert_eq!(self.model[*idx].name.lookup(self.db.upcast()), "to_enum");
							assert_eq!(args.len(), 2);
							write!(
								&mut buf,
								" = {}",
								self.pretty_print_expression(args[1], &function.data)
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
					self.pretty_print_expression(body, &function.data)
				)
				.unwrap();
			}
		} else if function.name.lookup(self.db.upcast()) == "erase_enum" {
			// For compatibility with old minizinc, we can just directly coerce
			let d = function.parameters[0];
			let ident = self.model[d]
				.name
				.map(|n| n.pretty_print(self.db.upcast()))
				.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(d)));
			write!(&mut buf, " = {}", ident).unwrap();
		}
		buf
	}

	fn pretty_print_output(&self, idx: ArenaIndex<Item<Output>>) -> String {
		let output = &self.model[idx];
		let mut buf = "output ".to_owned();
		if let Some(s) = output.section {
			write!(
				&mut buf,
				":: {} ",
				self.pretty_print_expression(s, &output.data)
			)
			.unwrap();
		}
		write!(
			&mut buf,
			"{}",
			self.pretty_print_expression(output.expression, &output.data)
		)
		.unwrap();
		buf
	}

	fn pretty_print_solve(&self) -> String {
		let solve = self.model.solve();
		let mut buf = "solve ".to_owned();
		for ann in solve.annotations.iter() {
			write!(
				&mut buf,
				":: {} ",
				self.pretty_print_expression(*ann, &solve.data)
			)
			.unwrap();
		}
		let s = match solve.goal {
			Goal::Satisfy => "satisfy",
			Goal::Maximize { .. } => "maximize _DECL_OBJ",
			Goal::Minimize { .. } => "minimize _DECL_OBJ",
		};
		write!(&mut buf, "{}", s).unwrap();
		buf
	}

	/// Pretty print a domain
	pub fn pretty_print_domain(&self, domain: &Domain, data: &ItemData) -> String {
		match domain {
			Domain::Array(ty, dim, el) => {
				let dims = match &**dim {
					Domain::Tuple(_, ds) => ds
						.iter()
						.map(|d| self.pretty_print_domain(d, data))
						.collect::<Vec<_>>()
						.join(", "),
					dom => self.pretty_print_domain(dom, data),
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
			Domain::Set(ty, el) => ty
				.inst(self.db.upcast())
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(
					ty.opt(self.db.upcast())
						.into_iter()
						.flat_map(|o| o.pretty_print()),
				)
				.chain(["set of".to_owned(), self.pretty_print_domain(el, data)])
				.collect::<Vec<_>>()
				.join(" "),
			Domain::Tuple(ty, ds) => {
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
			Domain::Record(ty, ds) => {
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
			Domain::Bounded(ty, e) => ty
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
			Domain::Unbounded(ty) => ty.pretty_print(self.db.upcast()),
		}
	}

	/// Pretty print an expression
	pub fn pretty_print_expression(&self, idx: ArenaIndex<Expression>, data: &ItemData) -> String {
		let mut out = match &*data.expressions[idx] {
			ExpressionData::Absent => "<>".to_owned(),
			ExpressionData::ArrayAccess {
				collection,
				indices,
			} => format!(
				"{}[{}]",
				self.pretty_print_expression(*collection, data),
				match &*data.expressions[*indices] {
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
				let model = self.db.model_thir();
				let mut buf = String::new();
				write!(&mut buf, "[").unwrap();
				if let Some(i) = indices {
					write!(&mut buf, "{}: ", self.pretty_print_expression(*i, data)).unwrap();
				}
				let t = self.pretty_print_expression(*template, data);
				let gs = generators
					.iter()
					.map(|g| {
						let decls = g
							.declarations
							.iter()
							.map(|d| {
								model[*d]
									.name
									.map(|n| n.pretty_print(self.db.upcast()))
									.unwrap_or_else(|| "_".to_owned())
							})
							.collect::<Vec<_>>()
							.join(", ");
						let mut gen = format!(
							"{} in {}",
							decls,
							self.pretty_print_expression(g.collection, data)
						);
						if let Some(where_clause) = g.where_clause {
							write!(
								&mut gen,
								" where {}",
								self.pretty_print_expression(where_clause, data)
							)
							.unwrap();
						}
						gen
					})
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
				if let ExpressionData::Identifier(ResolvedIdentifier::Function(func)) =
					&*data.expressions[*function]
				{
					// Special case to output valid slicing syntax
					let f = &self.model[*func];
					let name = f.name.lookup(self.db.upcast());
					if arguments.is_empty() {
						match name.as_str() {
							".." | "<..<" | "..<" | "<.." => return name,
							_ => (),
						}
					}
				}
				let f = self.pretty_print_expression(*function, data);
				let args = arguments
					.iter()
					.map(|a| self.pretty_print_expression(*a, data))
					.collect::<Vec<_>>()
					.join(", ");
				format!("{}({})", f, args)
			}
			ExpressionData::FloatLiteral(f) => format!("{}", f.value()),
			ExpressionData::Identifier(i) => {
				let model = self.db.model_thir();
				match i {
					ResolvedIdentifier::Annotation(a) => model[*a]
						.name
						.map(|n| n.pretty_print(self.db.upcast()))
						.expect("Identifier refers to annotation without name"),
					ResolvedIdentifier::Declaration(d) => model[*d]
						.name
						.map(|n| n.pretty_print(self.db.upcast()))
						.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(*d))),
					ResolvedIdentifier::Enumeration(e) => {
						model[*e].enum_type.pretty_print(self.db.upcast())
					}
					ResolvedIdentifier::EnumerationMember(e, i) => model[*e]
						.definition
						.as_ref()
						.expect("Identifier refers to non-existent enum member")[*i]
						.name
						.map(|n| n.pretty_print(self.db.upcast()))
						.unwrap_or_else(|| "_".to_owned()),
					ResolvedIdentifier::Function(f) => {
						model[*f].name.pretty_print(self.db.upcast())
					}
					ResolvedIdentifier::TyVarRef(t) => t.pretty_print(self.db.upcast()),
				}
			}
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
				let model = self.db.model_thir();
				let mut buf = String::new();
				write!(&mut buf, "{{").unwrap();
				let t = self.pretty_print_expression(*template, data);
				let gs = generators
					.iter()
					.map(|g| {
						let decls = g
							.declarations
							.iter()
							.map(|d| {
								model[*d]
									.name
									.map(|n| n.pretty_print(self.db.upcast()))
									.unwrap_or_else(|| "_".to_owned())
							})
							.collect::<Vec<_>>()
							.join(", ");
						let mut gen = format!(
							"{} in {}",
							decls,
							self.pretty_print_expression(g.collection, data)
						);
						if let Some(where_clause) = g.where_clause {
							write!(
								&mut gen,
								" where {}",
								self.pretty_print_expression(where_clause, data)
							)
							.unwrap();
						}
						gen
					})
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
		for ann in data.expressions[idx].annotations() {
			write!(&mut out, " :: {}", self.pretty_print_expression(*ann, data)).unwrap();
		}
		out
	}
}
