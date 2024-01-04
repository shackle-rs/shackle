use shackle_compiler::{
	syntax::{
		ast::AstNode,
		minizinc::{self, PatternNumericLiteral},
	},
	utils::pretty_print_identifier,
};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for minizinc::Pattern {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let e = match self {
			minizinc::Pattern::Absent(a) => Element::text(a.cst_text()),
			minizinc::Pattern::Anonymous(a) => Element::text(a.cst_text()),
			minizinc::Pattern::BooleanLiteral(b) => {
				Element::text(if b.value() { "true" } else { "false" })
			}
			minizinc::Pattern::Call(c) => c.format(formatter),
			minizinc::Pattern::Identifier(i) => Element::text(pretty_print_identifier(&i.name())),
			minizinc::Pattern::PatternNumericLiteral(n) => n.format(formatter),
			minizinc::Pattern::StringLiteral(s) => Element::text(s.cst_text()),
			minizinc::Pattern::Tuple(t) => t.format(formatter),
			minizinc::Pattern::Record(r) => formatter.format_list("(", ")", r.fields()),
		};
		formatter.attach_comments(self, vec![e])
	}

	fn has_brackets(&self, _formatter: &MiniZincFormatter) -> bool {
		matches!(
			self,
			minizinc::Pattern::Tuple(_) | minizinc::Pattern::Record(_)
		)
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

impl Format for minizinc::PatternCall {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			minizinc::Expression::Identifier(self.identifier()).format(formatter),
			formatter.format_list("(", ")", self.arguments()),
		])
	}
}

impl Format for minizinc::PatternTuple {
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

impl Format for minizinc::PatternRecordField {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			minizinc::Expression::Identifier(self.name()).format(formatter),
			Element::text(": "),
			self.value().format(formatter),
		])
	}
}
