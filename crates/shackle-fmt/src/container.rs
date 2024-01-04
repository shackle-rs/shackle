use shackle_compiler::syntax::minizinc;
use tree_sitter_minizinc::Precedence;

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for minizinc::ArrayLiteral {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("[", "]", self.members())
	}
}

impl Format for minizinc::ArrayLiteralMember {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		if let Some(idx) = self.indices() {
			Element::sequence(vec![
				idx.format(formatter),
				Element::text(": "),
				self.value().format(formatter),
			])
		} else {
			self.value().format(formatter)
		}
	}
}

impl Format for minizinc::ArrayLiteral2D {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		let indices = self.column_indices().collect::<Vec<_>>();
		let rows = self.rows().collect::<Vec<_>>();
		if !indices.is_empty() {
			elements.push(Element::group(vec![Element::join(
				self.column_indices()
					.map(|i| Element::sequence(vec![i.format(formatter), Element::text(":")])),
				vec![Element::line_break_or_space()],
			)]));
		}
		elements.extend(rows.iter().map(|r| r.format(formatter)));

		Element::sequence(vec![
			Element::text("[|"),
			Element::group(vec![
				Element::indent(vec![
					Element::line_break_or_space(),
					Element::join(elements, vec![Element::text(" |"), Element::line_break()]),
				]),
				Element::line_break_or_space(),
			]),
			Element::text("|]"),
		])
	}
}

impl Format for minizinc::ArrayLiteral2DRow {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		if let Some(idx) = self.index() {
			elements.push(idx.format(formatter));
			elements.push(Element::text(": "));
		}
		elements.push(Element::join(
			self.members().map(|e| e.format(formatter)),
			vec![Element::text(","), Element::line_break_or_space()],
		));
		Element::group(elements)
	}
}

impl Format for minizinc::ArrayAccess {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::indexed_access().get() > formatter.precedence(&self.collection()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.collection())
			} else {
				self.collection().format(formatter)
			},
			formatter.format_list("[", "]", self.indices()),
		])
	}
}

impl Format for minizinc::ArrayIndex {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		match self {
			minizinc::ArrayIndex::IndexSlice(x) => {
				formatter.attach_comments(self, vec![Element::text(x.operator())])
			}
			minizinc::ArrayIndex::Expression(e) => e.format(formatter),
		}
	}
}

impl Format for minizinc::ArrayComprehension {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("["),
			Element::group(vec![
				Element::indent(vec![
					Element::line_break_or_empty(),
					Element::sequence(if let Some(indices) = self.indices() {
						vec![indices.format(formatter), Element::text(": ")]
					} else {
						vec![]
					}),
					self.template().format(formatter),
					Element::text(" |"),
					Element::indent(vec![
						Element::line_break_or_space(),
						Element::join(
							self.generators().map(|g| g.format(formatter)),
							vec![Element::text(","), Element::line_break_or_space()],
						),
						Element::if_broken(vec![Element::text(",")]),
					]),
				]),
				Element::line_break_or_empty(),
			]),
			Element::text("]"),
		])
	}
}

impl Format for minizinc::SetLiteral {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("{", "}", self.members())
	}
}

impl Format for minizinc::SetComprehension {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("{"),
			Element::group(vec![
				Element::indent(vec![
					Element::line_break_or_empty(),
					self.template().format(formatter),
					Element::text(" |"),
					Element::indent(vec![
						Element::line_break_or_space(),
						Element::join(
							self.generators().map(|g| g.format(formatter)),
							vec![Element::text(","), Element::line_break_or_space()],
						),
						Element::if_broken(vec![Element::text(",")]),
					]),
				]),
				Element::line_break_or_empty(),
			]),
			Element::text("}"),
		])
	}
}

impl Format for minizinc::Generator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let e = match self {
			minizinc::Generator::AssignmentGenerator(a) => a.format(formatter),
			minizinc::Generator::IteratorGenerator(i) => i.format(formatter),
		};
		formatter.attach_comments(self, vec![e])
	}
}

impl Format for minizinc::IteratorGenerator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let mut elements = vec![
			Element::join(
				self.patterns().map(|p| p.format(formatter)),
				vec![Element::text(", ")],
			),
			Element::text(" in"),
			Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				self.collection().format(formatter),
			])]),
		];
		if let Some(w) = self.where_clause() {
			elements.push(Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				Element::text("where "),
				w.format(formatter),
			])]));
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::AssignmentGenerator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let mut elements = vec![
			self.pattern().format(formatter),
			Element::text(" ="),
			Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				self.value().format(formatter),
			])]),
		];
		if let Some(w) = self.where_clause() {
			elements.push(Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				Element::text("where "),
				w.format(formatter),
			])]));
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::TupleLiteral {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let members = self.members().collect::<Vec<_>>();
		if members.is_empty() {
			return Element::text("(,)");
		}
		if members.len() == 1 {
			return Element::group(vec![
				Element::text("("),
				Element::indent(vec![
					Element::line_break_or_empty(),
					members[0].format(formatter),
				]),
				Element::line_break_or_empty(),
				Element::text(",)"),
			]);
		}
		formatter.format_list("(", ")", members.into_iter())
	}
}

impl Format for minizinc::TupleAccess {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::tuple_access().get() > formatter.precedence(&self.tuple()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.tuple())
			} else {
				self.tuple().format(formatter)
			},
			Element::text("."),
			minizinc::Expression::IntegerLiteral(self.field()).format(formatter),
		])
	}
}

impl Format for minizinc::RecordLiteral {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("(", ")", self.members())
	}
}

impl Format for minizinc::RecordLiteralMember {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			minizinc::Expression::Identifier(self.name()).format(formatter),
			Element::text(": "),
			self.value().format(formatter),
		])
	}
}

impl Format for minizinc::RecordAccess {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::record_access().get() > formatter.precedence(&self.record()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.record())
			} else {
				self.record().format(formatter)
			},
			Element::text("."),
			minizinc::Expression::Identifier(self.field()).format(formatter),
		])
	}
}
