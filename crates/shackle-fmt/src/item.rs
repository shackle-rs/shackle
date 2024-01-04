use shackle_compiler::syntax::{ast::AstNode, minizinc};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for minizinc::Item {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let mut elements = Vec::new();
		let node = self.cst_node().as_ref();
		if let Some(p) = node.prev_sibling() {
			if p.end_position().row < node.start_position().row.saturating_sub(1) {
				elements.push(Element::line_break());
			}
		}
		let element = match self {
			minizinc::Item::Annotation(x) => x.format(formatter),
			minizinc::Item::Assignment(x) => x.format(formatter),
			minizinc::Item::Constraint(x) => x.format(formatter),
			minizinc::Item::Declaration(x) => x.format(formatter),
			minizinc::Item::Enumeration(x) => x.format(formatter),
			minizinc::Item::Function(x) => x.format(formatter),
			minizinc::Item::Include(x) => x.format(formatter),
			minizinc::Item::Output(x) => x.format(formatter),
			minizinc::Item::Predicate(x) => x.format(formatter),
			minizinc::Item::Solve(x) => x.format(formatter),
			minizinc::Item::TypeAlias(x) => x.format(formatter),
		};
		elements.push(element);
		elements.push(Element::text(";"));
		Element::sequence(vec![
			formatter.attach_comments(self, elements),
			Element::line_break(),
		])
	}
}

impl Format for minizinc::Annotation {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("annotation "),
			minizinc::Expression::Identifier(self.id()).format(formatter),
		];
		if let Some(params) = self.parameters() {
			elements.push(formatter.format_list("(", ")", params.iter()));
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::Assignment {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			self.assignee().format(formatter),
			Element::text(" ="),
			if self.definition().has_brackets(formatter) {
				Element::sequence(vec![
					Element::text(" "),
					self.definition().format(formatter),
				])
			} else {
				Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					self.definition().format(formatter),
				])])
			},
		])
	}
}

impl Format for minizinc::Constraint {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("constraint"),
			formatter.format_annotations(self.annotations()),
			if self.expression().has_brackets(formatter) {
				Element::sequence(vec![
					Element::text(" "),
					self.expression().format(formatter),
				])
			} else {
				Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					self.expression().format(formatter),
				])])
			},
		])
	}
}

impl Format for minizinc::Declaration {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			self.declared_type().format(formatter),
			Element::text(": "),
			self.pattern().format(formatter),
			formatter.format_annotations(self.annotations()),
		];
		if let Some(def) = self.definition() {
			elements.push(Element::text(" ="));
			if def.has_brackets(formatter) {
				elements.push(Element::text(" "));
				elements.push(def.format(formatter));
			} else {
				elements.push(Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					def.format(formatter),
				])]))
			}
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::Enumeration {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("enum "),
			minizinc::Expression::Identifier(self.id()).format(formatter),
			formatter.format_annotations(self.annotations()),
		];
		let cases = self
			.cases()
			.map(|case| case.format(formatter))
			.collect::<Vec<_>>();
		if !cases.is_empty() {
			elements.push(Element::text(" ="));
			elements.push(Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				Element::join(
					cases,
					vec![Element::text(" ++"), Element::line_break_or_space()],
				),
			])]));
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::EnumerationCase {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let c = match self {
			minizinc::EnumerationCase::Anonymous(e) => e.format(formatter),
			minizinc::EnumerationCase::Constructor(c) => c.format(formatter),
			minizinc::EnumerationCase::Members(m) => m.format(formatter),
		};
		formatter.attach_comments(self, vec![c])
	}
}

impl Format for minizinc::AnonymousEnumeration {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("_(", ")", self.parameters())
	}
}

impl Format for minizinc::EnumerationConstructor {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			minizinc::Expression::Identifier(self.id()).format(formatter),
			formatter.format_list("(", ")", self.parameters()),
		])
	}
}

impl Format for minizinc::EnumerationMembers {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list(
			"{",
			"}",
			self.members().map(minizinc::Expression::Identifier),
		)
	}
}

impl Format for minizinc::Function {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("function "),
			self.return_type().format(formatter),
			Element::text(": "),
			minizinc::Expression::Identifier(self.id()).format(formatter),
			formatter.format_list("(", ")", self.parameters()),
			formatter.format_annotations(self.annotations()),
		];
		if let Some(body) = self.body() {
			elements.push(Element::text(" ="));
			if body.has_brackets(formatter) {
				elements.push(Element::text(" "));
				elements.push(body.format(formatter));
			} else {
				elements.push(Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					body.format(formatter),
				])]))
			}
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::Predicate {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text(
				if self.declared_type() == minizinc::PredicateType::Predicate {
					"predicate "
				} else {
					"test "
				},
			),
			minizinc::Expression::Identifier(self.id()).format(formatter),
			formatter.format_list("(", ")", self.parameters()),
			formatter.format_annotations(self.annotations()),
		];
		if let Some(body) = self.body() {
			elements.push(Element::text(" ="));
			if body.has_brackets(formatter) {
				elements.push(Element::text(" "));
				elements.push(body.format(formatter));
			} else {
				elements.push(Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					body.format(formatter),
				])]))
			}
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::Parameter {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![self.declared_type().format(formatter)];
		if let Some(p) = self.pattern() {
			elements.push(Element::text(": "));
			elements.push(p.format(formatter));
			elements.push(formatter.format_annotations(self.annotations()));
		}
		formatter.attach_comments(self, elements)
	}
}

impl Format for minizinc::Include {
	fn format(&self, _formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("include "),
			Element::text(self.file().cst_text()),
		])
	}
}

impl Format for minizinc::Output {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![Element::text("output")];
		if let Some(section) = self.section() {
			elements.push(Element::text(" :: "));
			elements.push(Element::text(section.cst_text()));
		}
		if self.expression().has_brackets(formatter) {
			elements.push(Element::text(" "));
			elements.push(self.expression().format(formatter));
		} else {
			elements.push(Element::group(vec![Element::indent(vec![
				Element::line_break_or_space(),
				self.expression().format(formatter),
			])]));
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::Solve {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("solve"),
			formatter.format_annotations(self.annotations()),
		];
		match self.goal() {
			minizinc::Goal::Maximize(obj) => {
				elements.push(Element::text(" maximize"));
				if obj.has_brackets(formatter) {
					elements.push(Element::text(" "));
					elements.push(obj.format(formatter));
				} else {
					elements.push(Element::group(vec![Element::indent(vec![
						Element::line_break_or_space(),
						obj.format(formatter),
					])]));
				}
			}
			minizinc::Goal::Minimize(obj) => {
				elements.push(Element::text(" minimize"));
				if obj.has_brackets(formatter) {
					elements.push(Element::text(" "));
					elements.push(obj.format(formatter));
				} else {
					elements.push(Element::group(vec![Element::indent(vec![
						Element::line_break_or_space(),
						obj.format(formatter),
					])]));
				}
			}
			minizinc::Goal::Satisfy => elements.push(Element::text(" satisfy")),
		}
		Element::sequence(elements)
	}
}

impl Format for minizinc::TypeAlias {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("type "),
			minizinc::Expression::Identifier(self.name()).format(formatter),
			Element::text(" = "),
			self.aliased_type().format(formatter),
		])
	}
}
