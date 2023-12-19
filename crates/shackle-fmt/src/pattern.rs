use shackle_compiler::{
	syntax::ast::{self, AstNode, PatternNumericLiteral},
	utils::pretty_print_identifier,
};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for ast::Pattern {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let e = match self {
			ast::Pattern::Absent(a) => Element::text(a.cst_text()),
			ast::Pattern::Anonymous(a) => Element::text(a.cst_text()),
			ast::Pattern::BooleanLiteral(b) => {
				Element::text(if b.value() { "true" } else { "false" })
			}
			ast::Pattern::Call(c) => c.format(formatter),
			ast::Pattern::Identifier(i) => Element::text(pretty_print_identifier(&i.name())),
			ast::Pattern::PatternNumericLiteral(n) => n.format(formatter),
			ast::Pattern::StringLiteral(s) => Element::text(s.cst_text()),
			ast::Pattern::Tuple(t) => t.format(formatter),
			ast::Pattern::Record(r) => formatter.format_list("(", ")", r.fields()),
		};
		formatter.attach_comments(self, vec![e])
	}

	fn has_brackets(&self, _formatter: &MiniZincFormatter) -> bool {
		matches!(self, ast::Pattern::Tuple(_) | ast::Pattern::Record(_))
	}
}

impl Format for PatternNumericLiteral {
	fn format(&self, _formatter: &mut MiniZincFormatter) -> Element {
		if self.negated() {
			Element::sequence(vec![
				Element::text("-"),
				Element::text(self.value().cst_text()),
			])
		} else {
			Element::text(self.value().cst_text())
		}
	}
}

impl Format for ast::PatternCall {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			ast::Expression::Identifier(self.identifier()).format(formatter),
			formatter.format_list("(", ")", self.arguments()),
		])
	}
}

impl Format for ast::PatternTuple {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let fields = self.fields().collect::<Vec<_>>();
		if fields.is_empty() {
			return Element::text("(,)");
		}
		if fields.len() == 1 {
			return Element::group(vec![
				Element::text("("),
				Element::indent(vec![
					Element::line_break_or_empty(),
					fields[0].format(formatter),
				]),
				Element::line_break_or_empty(),
				Element::text(",)"),
			]);
		}
		formatter.format_list("(", ")", fields.into_iter())
	}
}

impl Format for ast::PatternRecordField {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			ast::Expression::Identifier(self.name()).format(formatter),
			Element::text(": "),
			self.value().format(formatter),
		])
	}
}
