use shackle_compiler::{
	syntax::{
		ast::AstNode,
		minizinc::{self, RecordField},
	},
	ty::{OptType, VarType},
	utils::maybe_grow_stack,
};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for minizinc::Type {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		maybe_grow_stack(|| {
			let t = match self {
				minizinc::Type::AnyType(a) => Element::text(a.cst_text()),
				minizinc::Type::TypeBase(b) => b.format(formatter),
				minizinc::Type::ArrayType(a) => a.format(formatter),
				minizinc::Type::SetType(s) => s.format(formatter),
				minizinc::Type::TupleType(t) => t.format(formatter),
				minizinc::Type::RecordType(r) => r.format(formatter),
				minizinc::Type::OperationType(o) => o.format(formatter),
			};
			formatter.attach_comments(self, vec![t])
		})
	}
}

impl Format for minizinc::TypeBase {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if self.any_type() {
			elements.push(Element::text("any "));
		}
		if let Some(v) = self.var_type() {
			match v {
				VarType::Var => {
					elements.push(Element::text("var "));
				}
				VarType::Par => {
					elements.push(Element::text("par "));
				}
			}
		}
		if let Some(OptType::Opt) = self.opt_type() {
			elements.push(Element::text("opt "));
		}

		elements.push(self.domain().format(formatter));
		Element::sequence(elements)
	}
}

impl Format for minizinc::Domain {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let e = match self {
			minizinc::Domain::Bounded(b) => b.format(formatter),
			minizinc::Domain::TypeInstEnumIdentifier(t) => Element::text(t.name()),
			minizinc::Domain::TypeInstIdentifier(t) => Element::text(t.name()),
			minizinc::Domain::Unbounded(u) => Element::text(u.cst_text()),
		};
		formatter.attach_comments(self, vec![e])
	}
}

impl Format for minizinc::ArrayType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			formatter.format_list("array [", "] of ", self.dimensions()),
			self.element_type().format(formatter),
		])
	}
}

impl Format for minizinc::SetType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		if let OptType::Opt = self.opt_type() {
			elements.push(Element::text("opt "));
		}
		elements.push(Element::text("set of "));
		elements.push(self.element_type().format(formatter));
		Element::sequence(elements)
	}
}

impl Format for minizinc::TupleType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		elements.push(formatter.format_list("tuple(", ")", self.fields()));
		Element::sequence(elements)
	}
}

impl Format for minizinc::RecordType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		elements.push(formatter.format_list("record(", ")", self.fields()));
		Element::sequence(elements)
	}
}

impl Format for RecordField {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let elements = vec![
			self.field_type().format(formatter),
			Element::text(": "),
			minizinc::Expression::Identifier(self.name()).format(formatter),
		];
		formatter.attach_comments(self, elements)
	}
}

impl Format for minizinc::OperationType {
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
