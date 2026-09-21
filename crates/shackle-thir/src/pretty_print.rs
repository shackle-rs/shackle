//! Pretty printing of THIR as MiniZinc
//!

use std::fmt::Write;

use shackle_ty::registry::TypeRegistry;
use shackle_utils::maybe_grow_stack;

use crate::{
	AnnotationId, Callable, ConstraintId, Db, DeclarationId, Domain, DomainData, EnumMemberId,
	EnumerationId, Expression, ExpressionData, FunctionId, Generator, Goal, ItemId, LetItem,
	Marker, Model, OutputId, Pattern, PatternData, ResolvedIdentifier,
};

/// Pretty prints THIR as MiniZinc
pub struct PrettyPrinter<'db, T: Marker> {
	db: &'db dyn Db,
	model: &'db Model<'db, T>,
}

impl<'db, T: Marker> std::fmt::Debug for PrettyPrinter<'db, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("PrettyPrinter").finish()
	}
}

impl<'db, T: Marker> PrettyPrinter<'db, T> {
	/// Create a new pretty printer
	pub fn new(db: &'db dyn Db, model: &'db Model<'db, T>) -> Self {
		Self { db, model }
	}

	/// Pretty print the model
	pub fn pretty_print(&self) -> String {
		self.print_model(self.db, self.model)
	}

	/// Pretty print an item from a model
	pub fn pretty_print_signature(&self, item: ItemId<'db, T>) -> String {
		print_signature(self, self.db, self.model, item)
	}

	/// Pretty print an item from a model
	pub fn pretty_print_item(&self, item: ItemId<'db, T>) -> String {
		print_item(self, self.db, self.model, item)
	}

	/// Pretty print a domain
	pub fn pretty_print_domain(&self, domain: &Domain<'db, T>) -> String {
		self.print_domain(self.db, self.model, domain)
	}

	/// Pretty print an expression
	pub fn pretty_print_expression(&self, expression: &Expression<'db, T>) -> String {
		self.print_expression(self.db, self.model, expression)
	}
}

impl<'db, T: Marker> Printer<'db, T> for PrettyPrinter<'db, T> {}

/// Trait for implementing a pretty printer for THIR
pub trait Printer<'db, T: Marker> {
	/// Pretty print the model
	fn print_model(&self, db: &'db dyn Db, model: &Model<'db, T>) -> String {
		print_model(self, db, model)
	}
	/// Pretty print an annotation
	fn print_annotation(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: AnnotationId<'db, T>,
	) -> String {
		print_annotation(self, db, model, idx)
	}
	/// Pretty print a constraint
	fn print_constraint(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: ConstraintId<'db, T>,
	) -> String {
		print_constraint(self, db, model, idx)
	}
	/// Pretty print a declaration
	fn print_declaration(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: DeclarationId<'db, T>,
		is_let_item: bool,
		signature_only: bool,
	) -> String {
		print_declaration(self, db, model, idx, is_let_item, signature_only)
	}
	/// Pretty print an enumeration
	fn print_enumeration(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: EnumerationId<'db, T>,
		signature_only: bool,
	) -> String {
		print_enumeration(self, db, model, idx, signature_only)
	}
	/// Pretty print a function
	fn print_function(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: FunctionId<'db, T>,
		signature_only: bool,
	) -> String {
		print_function(self, db, model, idx, signature_only)
	}
	/// Pretty print an output item
	fn print_output(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		idx: OutputId<'db, T>,
	) -> String {
		print_output(self, db, model, idx)
	}
	/// Pretty print the solve item
	fn print_solve(&self, db: &'db dyn Db, model: &Model<'db, T>) -> String {
		print_solve(self, db, model)
	}
	/// Pretty print an expression
	fn print_expression(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		expression: &Expression<'db, T>,
	) -> String {
		maybe_grow_stack(|| print_expression(self, db, model, expression))
	}
	/// Pretty print a domain
	fn print_domain(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		domain: &Domain<'db, T>,
	) -> String {
		maybe_grow_stack(|| print_domain(self, db, model, domain))
	}
	/// Pretty print a generator
	fn print_generator(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		g: &Generator<'db, T>,
	) -> String {
		print_generator(self, db, model, g)
	}
	/// Pretty print a pattern
	fn print_pattern(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		pat: &Pattern<'db, T>,
	) -> String {
		print_pattern(self, db, model, pat)
	}
	/// Pretty print the name of an annotation
	fn print_annotation_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		a: AnnotationId<'db, T>,
	) -> String {
		print_annotation_id(self, db, model, a)
	}
	/// Pretty print the name of an inversed annotation with a ^-1 suffix
	fn print_inversed_annotation_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		a: AnnotationId<'db, T>,
	) -> String {
		print_inversed_annotation_id(self, db, model, a)
	}
	/// Pretty print the name of a declaration
	fn print_declaration_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		d: DeclarationId<'db, T>,
	) -> String {
		print_declaration_id(self, db, model, d)
	}
	/// Pretty print the name of an enumeration
	fn print_function_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		f: FunctionId<'db, T>,
	) -> String {
		print_function_id(self, db, model, f)
	}
	/// Pretty print the name of an enumeration
	fn print_enumeration_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		e: EnumerationId<'db, T>,
	) -> String {
		print_enumeration_id(self, db, model, e)
	}
	/// Pretty print the name of an enumeration member
	fn print_enumeration_member_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		m: EnumMemberId<'db, T>,
	) -> String {
		print_enumeration_member_id(self, db, model, m)
	}
	/// Pretty print the name of an enumeration member with a ^-1 suffix
	fn print_inversed_enumeration_member_id(
		&self,
		db: &'db dyn Db,
		model: &Model<'db, T>,
		m: EnumMemberId<'db, T>,
	) -> String {
		print_inversed_enumeration_member_id(self, db, model, m)
	}
}

/// Default implementation for printing a model
pub fn print_model<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
) -> String {
	let mut buf = String::new();
	for item in model.top_level_items() {
		writeln!(&mut buf, "{};", print_item(printer, db, model, item)).unwrap();
	}
	if model.solve().is_none() {
		writeln!(&mut buf, "solve satisfy;").unwrap();
	}
	buf
}

/// Default implementation for printing a signature of an item
pub fn print_signature<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	item: ItemId<'db, T>,
) -> String {
	match item {
		ItemId::Annotation(i) => printer.print_annotation(db, model, i),
		ItemId::Constraint(i) => printer.print_constraint(db, model, i),
		ItemId::Declaration(i) => printer.print_declaration(db, model, i, false, true),
		ItemId::Enumeration(i) => printer.print_enumeration(db, model, i, true),
		ItemId::Function(i) => printer.print_function(db, model, i, true),
		ItemId::Output(i) => printer.print_output(db, model, i),
		ItemId::Solve => printer.print_solve(db, model),
	}
}
/// Default implementation for printing an item
pub fn print_item<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	item: ItemId<'db, T>,
) -> String {
	match item {
		ItemId::Annotation(i) => printer.print_annotation(db, model, i),
		ItemId::Constraint(i) => printer.print_constraint(db, model, i),
		ItemId::Declaration(i) => printer.print_declaration(db, model, i, false, false),
		ItemId::Enumeration(i) => printer.print_enumeration(db, model, i, false),
		ItemId::Function(i) => printer.print_function(db, model, i, false),
		ItemId::Output(i) => printer.print_output(db, model, i),
		ItemId::Solve => printer.print_solve(db, model),
	}
}
/// Default implementation for printing an annotation
pub fn print_annotation<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: AnnotationId<'db, T>,
) -> String {
	let annotation = &model[idx];
	let mut buf = format!("annotation {}", printer.print_annotation_id(db, model, idx));
	if let Some(params) = &annotation.parameters {
		write!(
			&mut buf,
			"({})",
			params
				.iter()
				.map(|p| printer.print_declaration(db, model, *p, false, true))
				.collect::<Vec<_>>()
				.join(", ")
		)
		.unwrap();
	}
	buf
}
/// Default implementation for printing a constraint
pub fn print_constraint<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: ConstraintId<'db, T>,
) -> String {
	let constraint = &model[idx];
	let mut buf = "constraint ".to_owned();
	for ann in constraint.annotations().iter() {
		write!(
			&mut buf,
			":: ({}) ",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	write!(
		&mut buf,
		"{}",
		printer.print_expression(db, model, constraint.expression())
	)
	.unwrap();
	buf
}
/// Default implementation for printing a declaration
pub fn print_declaration<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: DeclarationId<'db, T>,
	is_let_item: bool,
	signature_only: bool,
) -> String {
	let declaration = &model[idx];
	let ty = declaration.ty();
	let mut buf =
		if is_let_item && ty.contains_type_inst_var(db) && declaration.definition().is_some()
			|| ty == TypeRegistry::lookup(db).bottom
		{
			"any".to_owned()
		} else {
			printer.print_domain(db, model, declaration.domain())
		};
	write!(
		&mut buf,
		": {}",
		printer.print_declaration_id(db, model, idx)
	)
	.unwrap();
	for ann in declaration.annotations().iter() {
		write!(
			&mut buf,
			" :: ({})",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	if !signature_only && let Some(def) = declaration.definition() {
		write!(&mut buf, " = {}", printer.print_expression(db, model, def)).unwrap();
	}
	buf
}
/// Default implementation for printing an enumeration
pub fn print_enumeration<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: EnumerationId<'db, T>,
	signature_only: bool,
) -> String {
	let enumeration = &model[idx];
	let enum_name = printer.print_enumeration_id(db, model, idx);
	let mut buf = format!("enum {}", enum_name);
	for ann in enumeration.annotations().iter() {
		write!(
			&mut buf,
			" :: ({})",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	if !signature_only && let Some(cases) = enumeration.definition() {
		write!(
			&mut buf,
			" = {}",
			cases
				.iter()
				.enumerate()
				.map(|(i, c)| {
					let name = printer.print_enumeration_member_id(
						db,
						model,
						EnumMemberId::new(idx, i as u32),
					);
					match &c.parameters {
						Some(ps) => format!(
							"{}({})",
							name,
							ps.iter()
								.map(|d| printer.print_domain(db, model, model[*d].domain()))
								.collect::<Vec<_>>()
								.join(", ")
						),
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
/// Default implementation for printing a function
pub fn print_function<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: FunctionId<'db, T>,
	signature_only: bool,
) -> String {
	let function = &model[idx];
	let name = printer.print_function_id(db, model, idx);
	let mut buf = String::new();
	if function.body().is_none()
		&& function.return_type() == TypeRegistry::lookup(db).var_bool
		&& !name.starts_with('\'')
	{
		write!(&mut buf, "predicate").unwrap();
	} else {
		write!(
			&mut buf,
			"function {}:",
			printer.print_domain(db, model, function.domain())
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
			.map(|p| printer.print_declaration(db, model, *p, false, signature_only))
			.collect::<Vec<_>>()
			.join(", ")
	)
	.unwrap();
	for ann in function.annotations().iter() {
		write!(
			&mut buf,
			" :: ({})",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	if !signature_only && let Some(body) = function.body() {
		write!(&mut buf, " = {}", printer.print_expression(db, model, body)).unwrap();
	}
	buf
}
/// Default implementation for printing an output item
pub fn print_output<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	idx: OutputId<'db, T>,
) -> String {
	let output = &model[idx];
	let mut buf = "output ".to_owned();
	if let Some(s) = output.section() {
		write!(&mut buf, ":: {} ", printer.print_expression(db, model, s)).unwrap();
	}
	write!(
		&mut buf,
		"{}",
		printer.print_expression(db, model, output.expression())
	)
	.unwrap();
	buf
}
/// Default implementation for printing the solve item
pub fn print_solve<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
) -> String {
	let solve = model.solve().unwrap();
	let mut buf = "solve ".to_owned();
	for ann in solve.annotations().iter() {
		write!(
			&mut buf,
			":: ({}) ",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	match solve.goal() {
		Goal::Satisfy => write!(&mut buf, "satisfy").unwrap(),
		Goal::Maximize { objective } => write!(
			&mut buf,
			"maximize {}",
			printer.print_declaration_id(db, model, *objective)
		)
		.unwrap(),
		Goal::Minimize { objective } => write!(
			&mut buf,
			"minimize {}",
			printer.print_declaration_id(db, model, *objective)
		)
		.unwrap(),
	};
	buf
}
/// Default implementation for printing a domain
pub fn print_domain<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	domain: &Domain<'db, T>,
) -> String {
	let ty = domain.ty();
	match &**domain {
		DomainData::Array(dim, el) => {
			let dims = match &***dim {
				DomainData::Tuple(ds) => ds
					.iter()
					.map(|d| printer.print_domain(db, model, d))
					.collect::<Vec<_>>()
					.join(", "),
				DomainData::Unbounded => ty.dim_ty(db).unwrap().pretty_print_as_dims(db),
				_ => printer.print_domain(db, model, dim),
			};
			ty.opt(db)
				.into_iter()
				.flat_map(|o| o.pretty_print())
				.chain([format!(
					"array [{}] of {}",
					dims,
					printer.print_domain(db, model, el)
				)])
				.collect::<Vec<_>>()
				.join(" ")
		}
		DomainData::Bounded(e) => ty
			.inst(db)
			.into_iter()
			.flat_map(|i| i.pretty_print())
			.chain(ty.opt(db).into_iter().flat_map(|o| o.pretty_print()))
			.chain([printer.print_expression(db, model, e)])
			.collect::<Vec<_>>()
			.join(" "),
		DomainData::Set(s, None) => ty
			.inst(db)
			.into_iter()
			.flat_map(|i| i.pretty_print())
			.chain(ty.opt(db).into_iter().flat_map(|o| o.pretty_print()))
			.chain(["set of".to_owned()])
			.chain([printer.print_domain(db, model, s)])
			.collect::<Vec<_>>()
			.join(" "),
		DomainData::Set(s, Some(c)) => ty
			.inst(db)
			.into_iter()
			.flat_map(|i| i.pretty_print())
			.chain(ty.opt(db).into_iter().flat_map(|o| o.pretty_print()))
			.chain(["set(".to_owned()])
			.chain([printer.print_expression(db, model, c)])
			.chain([") of ".to_owned()])
			.chain([printer.print_domain(db, model, s)])
			.collect::<Vec<_>>()
			.join(" "),
		DomainData::Tuple(ds) => {
			let doms = ds
				.iter()
				.map(|d| printer.print_domain(db, model, d))
				.collect::<Vec<_>>()
				.join(", ");
			ty.inst(db)
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(ty.opt(db).into_iter().flat_map(|o| o.pretty_print()))
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
						printer.print_domain(db, model, d),
						i.pretty_print(db)
					)
				})
				.collect::<Vec<_>>()
				.join(", ");
			ty.inst(db)
				.into_iter()
				.flat_map(|i| i.pretty_print())
				.chain(ty.opt(db).into_iter().flat_map(|o| o.pretty_print()))
				.chain([format!("record({})", doms)])
				.collect::<Vec<_>>()
				.join(" ")
		}
		DomainData::Unbounded => ty.pretty_print(db),
	}
}
/// Default implementation for printing an expression
pub fn print_expression<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	expression: &Expression<'db, T>,
) -> String {
	let mut out = match &**expression {
		ExpressionData::Absent => "<>".to_owned(),
		ExpressionData::ArrayComprehension(c) => {
			let mut buf = String::new();
			write!(&mut buf, "[").unwrap();
			if let Some(i) = &c.indices {
				write!(&mut buf, "{}: ", printer.print_expression(db, model, i)).unwrap();
			}
			write!(
				&mut buf,
				"{} | {}]",
				printer.print_expression(db, model, &c.template),
				c.generators
					.iter()
					.map(|g| printer.print_generator(db, model, g))
					.collect::<Vec<_>>()
					.join(", ")
			)
			.unwrap();
			buf
		}
		ExpressionData::ArrayLiteral(al) => format!(
			"[{}]",
			al.iter()
				.map(|e| printer.print_expression(db, model, e))
				.collect::<Vec<_>>()
				.join(", ")
		),
		ExpressionData::BooleanLiteral(b) => {
			if b.0 {
				"true".to_owned()
			} else {
				"false".to_owned()
			}
		}
		ExpressionData::Call(c) => {
			let f = match &c.function {
				Callable::Annotation(a) => printer.print_annotation_id(db, model, *a),
				Callable::AnnotationDestructure(a) => {
					printer.print_inversed_annotation_id(db, model, *a)
				}
				Callable::EnumConstructor(m) => printer.print_enumeration_member_id(db, model, *m),
				Callable::EnumDestructor(m) => {
					printer.print_inversed_enumeration_member_id(db, model, *m)
				}
				Callable::Function(f) => printer.print_function_id(db, model, *f),
				Callable::Expression(e) => format!("({})", printer.print_expression(db, model, e)),
			};
			format!(
				"{}({})",
				f,
				c.arguments
					.iter()
					.map(|a| printer.print_expression(db, model, a))
					.collect::<Vec<_>>()
					.join(", ")
			)
		}
		ExpressionData::Case(c) => format!(
			"case {} of {} endcase",
			printer.print_expression(db, model, &c.scrutinee),
			c.branches
				.iter()
				.map(|b| format!(
					"{} => {}",
					printer.print_pattern(db, model, &b.pattern),
					printer.print_expression(db, model, &b.result)
				))
				.collect::<Vec<_>>()
				.join(", ")
		),
		ExpressionData::FloatLiteral(f) => {
			let value = f.value();
			if value.fract() == 0.0 {
				format!("{}.0", value)
			} else {
				format!("{}", value)
			}
		}
		ExpressionData::Identifier(i) => match i {
			ResolvedIdentifier::Annotation(a) => printer.print_annotation_id(db, model, *a),
			ResolvedIdentifier::Declaration(d) => printer.print_declaration_id(db, model, *d),
			ResolvedIdentifier::Enumeration(e) => printer.print_enumeration_id(db, model, *e),
			ResolvedIdentifier::EnumerationMember(m) => {
				printer.print_enumeration_member_id(db, model, *m)
			}
		},
		ExpressionData::IfThenElse(ite) => {
			let mut buf = String::new();
			let mut bs = ite.branches.iter();
			let first = bs.next().expect("No branches in if-then-else");
			write!(
				&mut buf,
				"if {} then {} ",
				printer.print_expression(db, model, &first.condition),
				printer.print_expression(db, model, &first.result)
			)
			.unwrap();
			for branch in bs {
				write!(
					&mut buf,
					"elseif {} then {} ",
					printer.print_expression(db, model, &branch.condition),
					printer.print_expression(db, model, &branch.result)
				)
				.unwrap();
			}
			write!(
				&mut buf,
				"else {} endif",
				printer.print_expression(db, model, &ite.else_result)
			)
			.unwrap();
			buf
		}
		ExpressionData::Infinity => "infinity".to_owned(),
		ExpressionData::IntegerLiteral(i) => format!("{}", i.0),
		ExpressionData::Lambda(l) => format!(
			"lambda {}: ({}) => {}",
			printer.print_domain(db, model, model[**l].domain()),
			model[**l]
				.parameters()
				.iter()
				.map(|p| printer.print_declaration(db, model, *p, false, true))
				.collect::<Vec<_>>()
				.join(", "),
			printer.print_expression(db, model, model[**l].body().unwrap())
		),
		ExpressionData::Let(l) => {
			let mut buf = String::new();
			writeln!(&mut buf, "let {{").unwrap();
			for item in l.items.iter() {
				match item {
					LetItem::Constraint(c) => {
						writeln!(&mut buf, "  {};", printer.print_constraint(db, model, *c))
							.unwrap()
					}
					LetItem::Declaration(d) => writeln!(
						&mut buf,
						"  {};",
						printer.print_declaration(db, model, *d, true, false)
					)
					.unwrap(),
				}
			}
			write!(
				&mut buf,
				"}} in {}",
				printer.print_expression(db, model, &l.in_expression)
			)
			.unwrap();
			buf
		}
		ExpressionData::RecordAccess(ra) => format!(
			"({}).{}",
			printer.print_expression(db, model, &ra.record),
			ra.field.pretty_print(db)
		),
		ExpressionData::RecordLiteral(fs) => format!(
			"({})",
			fs.iter()
				.map(|(i, e)| format!(
					"{}: {}",
					i.pretty_print(db),
					printer.print_expression(db, model, e)
				))
				.collect::<Vec<_>>()
				.join(", ")
		),
		ExpressionData::SetComprehension(c) => format!(
			"{{{} | {}}}",
			printer.print_expression(db, model, &c.template),
			c.generators
				.iter()
				.map(|g| printer.print_generator(db, model, g))
				.collect::<Vec<_>>()
				.join(", ")
		),
		ExpressionData::SetLiteral(sl) => format!(
			"{{{}}}",
			sl.iter()
				.map(|e| printer.print_expression(db, model, e))
				.collect::<Vec<_>>()
				.join(", ")
		),
		ExpressionData::StringLiteral(s) => format!("{:?}", s.value(db)),
		ExpressionData::TupleAccess(ta) => format!(
			"({}).{}",
			printer.print_expression(db, model, &ta.tuple),
			ta.field.0
		),
		ExpressionData::TupleLiteral(fs) => {
			let fields = fs
				.iter()
				.map(|f| printer.print_expression(db, model, f))
				.collect::<Vec<_>>()
				.join(", ");
			let end = if fs.len() <= 1 { "," } else { "" };
			format!("({}{})", fields, end)
		}
	};
	for ann in expression.annotations().iter() {
		write!(
			&mut out,
			" :: ({})",
			printer.print_expression(db, model, ann)
		)
		.unwrap();
	}
	out
}
/// Default implementation for printing a generator
pub fn print_generator<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	g: &Generator<'db, T>,
) -> String {
	let (mut gtor, where_clause) = match g {
		Generator::Iterator {
			declarations,
			collection,
			where_clause,
		} => (
			format!(
				"{} in {}",
				declarations
					.iter()
					.map(|d| printer.print_declaration_id(db, model, *d))
					.collect::<Vec<_>>()
					.join(", "),
				printer.print_expression(db, model, collection)
			),
			where_clause,
		),
		Generator::Assignment {
			assignment,
			where_clause,
		} => {
			let decl = &model[*assignment];
			(
				format!(
					"{}{} = {}",
					printer.print_declaration_id(db, model, *assignment),
					decl.annotations()
						.iter()
						.map(|ann| format!(" :: ({})", printer.print_expression(db, model, ann)))
						.collect::<Vec<_>>()
						.join(""),
					printer.print_expression(db, model, decl.definition().unwrap()),
				),
				where_clause,
			)
		}
	};
	if let Some(where_clause) = where_clause {
		write!(
			&mut gtor,
			" where {}",
			printer.print_expression(db, model, where_clause)
		)
		.unwrap();
	}
	gtor
}
/// Default implementation for printing a pattern
pub fn print_pattern<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	pat: &Pattern<'db, T>,
) -> String {
	match &**pat {
		PatternData::Anonymous(_) => "_".to_owned(),
		PatternData::Expression(e) => printer.print_expression(db, model, e),
		PatternData::Tuple(fs) => format!(
			"({})",
			fs.iter()
				.map(|p| printer.print_pattern(db, model, p))
				.collect::<Vec<_>>()
				.join(", ")
		),
		PatternData::Record(fs) => format!(
			"({})",
			fs.iter()
				.map(|(i, p)| format!(
					"{}: {}",
					i.pretty_print(db),
					printer.print_pattern(db, model, p)
				))
				.collect::<Vec<_>>()
				.join(", ")
		),
		PatternData::EnumConstructor { member, args, .. } => format!(
			"{}({})",
			printer.print_enumeration_member_id(db, model, *member),
			args.iter()
				.map(|p| printer.print_pattern(db, model, p))
				.collect::<Vec<_>>()
				.join(", ")
		),
		PatternData::AnnotationConstructor { item, args } => format!(
			"{}({})",
			printer.print_annotation_id(db, model, *item),
			args.iter()
				.map(|p| printer.print_pattern(db, model, p))
				.collect::<Vec<_>>()
				.join(", ")
		),
	}
}
/// Default implementation for printing the name of an annotation
pub fn print_annotation_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	a: AnnotationId<'db, T>,
) -> String {
	model[a]
		.name
		.map(|n| n.pretty_print(db))
		.unwrap_or_else(|| format!("_ANN_{}", Into::<u32>::into(a)))
}
/// Default implementation for printing the name of an inversed annotation
pub fn print_inversed_annotation_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	a: AnnotationId<'db, T>,
) -> String {
	model[a]
		.name
		.map(|n| n.inversed(db).pretty_print(db))
		.unwrap_or_else(|| format!("_ANN_{}⁻¹", Into::<u32>::into(a)))
}
/// Default implementation for printing the name of a declaration
pub fn print_declaration_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	d: DeclarationId<'db, T>,
) -> String {
	model[d]
		.name()
		.map(|n| n.pretty_print(db))
		.unwrap_or_else(|| format!("_DECL_{}", Into::<u32>::into(d)))
}
/// Default implementation for printing the name of a function
pub fn print_function_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	f: FunctionId<'db, T>,
) -> String {
	let name = if let Some(tys) = model[f].mangled_param_tys() {
		model[f].name().mangled(db, tys.iter().copied())
	} else {
		model[f].name().as_identifier(db)
	};
	name.pretty_print(db)
}
/// Default implementation for printing the name of an enumeration
pub fn print_enumeration_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	e: EnumerationId<'db, T>,
) -> String {
	model[e].enum_type().pretty_print(db)
}
/// Default implementation for printing the name of an enumeration member
pub fn print_enumeration_member_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	m: EnumMemberId<'db, T>,
) -> String {
	model[m]
		.name
		.map(|n| n.pretty_print(db))
		.unwrap_or_else(|| {
			format!(
				"_EM_{}_{}",
				model[m.enumeration_id()].enum_type().pretty_print(db),
				m.member_index()
			)
		})
}
/// Default implementation for printing the name of an inversed enumeration member
pub fn print_inversed_enumeration_member_id<'db, T: Marker, P: Printer<'db, T> + ?Sized>(
	_printer: &P,
	db: &'db dyn Db,
	model: &Model<'db, T>,
	m: EnumMemberId<'db, T>,
) -> String {
	model[m]
		.name
		.map(|n| n.pretty_print(db))
		.unwrap_or_else(|| {
			format!(
				"_EM_{}_{}⁻¹",
				model[m.enumeration_id()].enum_type().pretty_print(db),
				m.member_index()
			)
		})
}
