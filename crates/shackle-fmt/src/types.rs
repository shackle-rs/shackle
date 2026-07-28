use shackle_syntax::{ast::AstNode, minizinc};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl<'tree> Format for minizinc::Type<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
			let t = match self {
				minizinc::Type::AnyType(a) => Element::text(a.cst_text(formatter.source())),
				minizinc::Type::TypeBase(b) => b.format(formatter),
				minizinc::Type::ArrayType(a) => a.format(formatter),
				minizinc::Type::ListType(l) => l.format(formatter),
				minizinc::Type::SetType(s) => s.format(formatter),
				minizinc::Type::TupleType(t) => t.format(formatter),
				minizinc::Type::RecordType(r) => r.format(formatter),
				minizinc::Type::OperationType(o) => o.format(formatter),
			};
			formatter.attach_comments(self, vec![t])
		})
	}

	fn has_brackets(&self, formatter: &MiniZincFormatter) -> bool {
		if let minizinc::Type::TypeBase(b) = self {
			b.has_brackets(formatter)
		} else {
			false
		}
	}
}

impl<'tree> Format for minizinc::TypeBase<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if self.any_type() {
			elements.push(Element::text("any "));
		}
		if let Some(v) = self.var_type() {
			match v {
				minizinc::VarType::Var => {
					elements.push(Element::text("var "));
				}
				minizinc::VarType::Par => {
					elements.push(Element::text("par "));
				}
			}
		}
		if let Some(minizinc::OptType::Opt) = self.opt_type() {
			elements.push(Element::text("opt "));
		}

		elements.push(self.domain().format(formatter));
		Element::sequence(elements)
	}

	fn has_brackets(&self, formatter: &MiniZincFormatter) -> bool {
		if !self.any_type() && self.var_type().is_none() && self.opt_type().is_none() {
			self.domain().has_brackets(formatter)
		} else {
			false
		}
	}
}

impl<'tree> Format for minizinc::Domain<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let e = match self {
			minizinc::Domain::Bounded(b) => b.format(formatter),
			minizinc::Domain::TypeInstEnumIdentifier(t) => {
				Element::text(t.name(formatter.source()))
			}
			minizinc::Domain::TypeInstIdentifier(t) => Element::text(t.name(formatter.source())),
			minizinc::Domain::NewType(n) => Element::sequence(vec![
				Element::text("new "),
				minizinc::Expression::Identifier(n.name()).format(formatter),
			]),
			minizinc::Domain::Unbounded(u) => Element::text(u.cst_text(formatter.source())),
		};
		formatter.attach_comments(self, vec![e])
	}

	fn has_brackets(&self, formatter: &MiniZincFormatter) -> bool {
		if let minizinc::Domain::Bounded(b) = self {
			b.has_brackets(formatter)
		} else {
			false
		}
	}
}

impl<'tree> Format for minizinc::ArrayType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			formatter.format_list("array [", "] of ", self.dimensions()),
			self.element_type().format(formatter),
		])
	}
}

impl<'tree> Format for minizinc::ArrayDimension<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		if let Some(name) = self.name() {
			Element::sequence([
				Element::text(name.cst_text(formatter.source())),
				Element::text(" in "),
				self.dim_type().format(formatter),
			])
		} else {
			self.dim_type().format(formatter)
		}
	}
}

impl<'tree> Format for minizinc::ListType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("list of "),
			self.element_type().format(formatter),
		])
	}
}

impl<'tree> Format for minizinc::SetType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let minizinc::VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		if let minizinc::OptType::Opt = self.opt_type() {
			elements.push(Element::text("opt "));
		}
		elements.push(Element::text("set"));
		if let Some(c) = self.cardinality() {
			elements.push(Element::text("("));
			elements.push(c.format(formatter));
			elements.push(Element::text(")"));
		}
		elements.push(Element::text(" of "));
		elements.push(self.element_type().format(formatter));
		Element::sequence(elements)
	}
}

impl<'tree> Format for minizinc::TupleType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let minizinc::VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		elements.push(formatter.format_list("tuple(", ")", self.fields()));
		Element::sequence(elements)
	}
}

impl<'tree> Format for minizinc::RecordType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let minizinc::VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		elements.push(formatter.format_list("record(", ")", self.fields()));
		Element::sequence(elements)
	}
}

impl<'tree> Format for minizinc::RecordField<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let elements = vec![
			self.field_type().format(formatter),
			Element::text(": "),
			minizinc::Expression::Identifier(self.name()).format(formatter),
		];
		formatter.attach_comments(self, elements)
	}
}

impl<'tree> Format for minizinc::OperationType<'tree> {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("op("),
			Element::group(vec![
				Element::indent(vec![
					Element::line_break_or_empty(),
					self.return_type().format(formatter),
					Element::text(": "),
					formatter.format_list("(", ")", self.parameter_types()),
				]),
				Element::line_break_or_empty(),
			]),
			Element::text(")"),
		])
	}
}
