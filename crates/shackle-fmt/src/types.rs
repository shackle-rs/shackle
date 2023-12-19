use shackle_compiler::{
	syntax::ast::{self, AstNode, RecordField},
	ty::{OptType, VarType},
	utils::maybe_grow_stack,
};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for ast::Type {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		maybe_grow_stack(|| {
			let t = match self {
				ast::Type::AnyType(a) => Element::text(a.cst_text()),
				ast::Type::TypeBase(b) => b.format(formatter),
				ast::Type::ArrayType(a) => a.format(formatter),
				ast::Type::SetType(s) => s.format(formatter),
				ast::Type::TupleType(t) => t.format(formatter),
				ast::Type::RecordType(r) => r.format(formatter),
				ast::Type::OperationType(o) => o.format(formatter),
			};
			formatter.attach_comments(self, vec![t])
		})
	}
}

impl Format for ast::TypeBase {
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

impl Format for ast::Domain {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let e = match self {
			ast::Domain::Bounded(b) => b.format(formatter),
			ast::Domain::TypeInstEnumIdentifier(t) => Element::text(t.name()),
			ast::Domain::TypeInstIdentifier(t) => Element::text(t.name()),
			ast::Domain::Unbounded(u) => Element::text(u.cst_text()),
		};
		formatter.attach_comments(self, vec![e])
	}
}

impl Format for ast::ArrayType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			formatter.format_list("array [", "] of ", self.dimensions()),
			self.element_type().format(formatter),
		])
	}
}

impl Format for ast::SetType {
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

impl Format for ast::TupleType {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let VarType::Var = self.var_type() {
			elements.push(Element::text("var "));
		}
		elements.push(formatter.format_list("tuple(", ")", self.fields()));
		Element::sequence(elements)
	}
}

impl Format for ast::RecordType {
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
			ast::Expression::Identifier(self.name()).format(formatter),
		];
		formatter.attach_comments(self, elements)
	}
}

impl Format for ast::OperationType {
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
