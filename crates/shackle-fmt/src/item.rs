use shackle_compiler::syntax::ast::{self, AstNode};

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for ast::Item {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let mut elements = Vec::new();
		let node = self.cst_node().as_ref();
		if let Some(p) = node.prev_sibling() {
			if p.end_position().row < node.start_position().row.saturating_sub(1) {
				elements.push(Element::line_break());
			}
		}
		let element = match self {
			ast::Item::Annotation(x) => x.format(formatter),
			ast::Item::Assignment(x) => x.format(formatter),
			ast::Item::Constraint(x) => x.format(formatter),
			ast::Item::Declaration(x) => x.format(formatter),
			ast::Item::Enumeration(x) => x.format(formatter),
			ast::Item::Function(x) => x.format(formatter),
			ast::Item::Include(x) => x.format(formatter),
			ast::Item::Output(x) => x.format(formatter),
			ast::Item::Predicate(x) => x.format(formatter),
			ast::Item::Solve(x) => x.format(formatter),
			ast::Item::TypeAlias(x) => x.format(formatter),
		};
		elements.push(element);
		elements.push(Element::text(";"));
		Element::sequence(vec![
			formatter.attach_comments(self, elements),
			Element::line_break(),
		])
	}
}

impl Format for ast::Annotation {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("annotation "),
			ast::Expression::Identifier(self.id()).format(formatter),
		];
		if let Some(params) = self.parameters() {
			elements.push(formatter.format_list("(", ")", params.iter()));
		}
		Element::sequence(elements)
	}
}

impl Format for ast::Assignment {
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

impl Format for ast::Constraint {
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

impl Format for ast::Declaration {
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

impl Format for ast::Enumeration {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("enum "),
			ast::Expression::Identifier(self.id()).format(formatter),
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

impl Format for ast::EnumerationCase {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let c = match self {
			ast::EnumerationCase::Anonymous(e) => e.format(formatter),
			ast::EnumerationCase::Constructor(c) => c.format(formatter),
			ast::EnumerationCase::Members(m) => m.format(formatter),
		};
		formatter.attach_comments(self, vec![c])
	}
}

impl Format for ast::AnonymousEnumeration {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("_(", ")", self.parameters())
	}
}

impl Format for ast::EnumerationConstructor {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			ast::Expression::Identifier(self.id()).format(formatter),
			formatter.format_list("(", ")", self.parameters()),
		])
	}
}

impl Format for ast::EnumerationMembers {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		formatter.format_list("{", "}", self.members().map(ast::Expression::Identifier))
	}
}

impl Format for ast::Function {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("function "),
			self.return_type().format(formatter),
			Element::text(": "),
			ast::Expression::Identifier(self.id()).format(formatter),
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

impl Format for ast::Predicate {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text(if self.declared_type() == ast::PredicateType::Predicate {
				"predicate "
			} else {
				"test "
			}),
			ast::Expression::Identifier(self.id()).format(formatter),
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

impl Format for ast::Parameter {
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

impl Format for ast::Include {
	fn format(&self, _formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("include "),
			Element::text(self.file().cst_text()),
		])
	}
}

impl Format for ast::Output {
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

impl Format for ast::Solve {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![
			Element::text("solve"),
			formatter.format_annotations(self.annotations()),
		];
		match self.goal() {
			ast::Goal::Maximize(obj) => {
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
			ast::Goal::Minimize(obj) => {
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
			ast::Goal::Satisfy => elements.push(Element::text(" satisfy")),
		}
		Element::sequence(elements)
	}
}

impl Format for ast::TypeAlias {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("type "),
			ast::Expression::Identifier(self.name()).format(formatter),
			Element::text(" = "),
			self.aliased_type().format(formatter),
		])
	}
}
