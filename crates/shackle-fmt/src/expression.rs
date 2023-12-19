use shackle_compiler::{
	syntax::ast::{self, AstNode},
	utils::pretty_print_identifier,
};
use tree_sitter_minizinc::Precedence;

use crate::{
	format::{Format, MiniZincFormatter},
	ir::Element,
};

impl Format for ast::Expression {
	fn format(&self, formatter: &mut MiniZincFormatter) -> crate::ir::Element {
		let e = match self {
			ast::Expression::IntegerLiteral(i) => Element::text(i.cst_text()),
			ast::Expression::FloatLiteral(f) => Element::text(f.cst_text()),
			ast::Expression::TupleLiteral(t) => t.format(formatter),
			ast::Expression::RecordLiteral(r) => r.format(formatter),
			ast::Expression::SetLiteral(s) => s.format(formatter),
			ast::Expression::BooleanLiteral(b) => {
				Element::text(if b.value() { "true" } else { "false" })
			}
			ast::Expression::StringLiteral(s) => Element::text(s.cst_text()),
			ast::Expression::Identifier(i) => Element::text(pretty_print_identifier(&i.name())),
			ast::Expression::Absent(a) => Element::text(a.cst_text()),
			ast::Expression::Infinity(i) => Element::text(i.cst_text()),
			ast::Expression::Anonymous(a) => Element::text(a.cst_text()),
			ast::Expression::ArrayLiteral(a) => a.format(formatter),
			ast::Expression::ArrayLiteral2D(a) => a.format(formatter),
			ast::Expression::ArrayAccess(a) => a.format(formatter),
			ast::Expression::ArrayComprehension(c) => c.format(formatter),
			ast::Expression::SetComprehension(c) => c.format(formatter),
			ast::Expression::IfThenElse(i) => i.format(formatter),
			ast::Expression::Call(c) => c.format(formatter),
			ast::Expression::PrefixOperator(o) => o.format(formatter),
			ast::Expression::InfixOperator(o) => o.format(formatter),
			ast::Expression::PostfixOperator(o) => o.format(formatter),
			ast::Expression::GeneratorCall(c) => c.format(formatter),
			ast::Expression::StringInterpolation(s) => s.format(formatter),
			ast::Expression::Case(c) => c.format(formatter),
			ast::Expression::Let(l) => l.format(formatter),
			ast::Expression::TupleAccess(t) => t.format(formatter),
			ast::Expression::RecordAccess(r) => r.format(formatter),
			ast::Expression::Lambda(l) => l.format(formatter),
			ast::Expression::AnnotatedExpression(e) => e.format(formatter),
		};
		let result = formatter.attach_comments(self, vec![e]);
		if formatter.options().keep_parentheses && self.is_parenthesised() {
			Element::sequence(vec![
				Element::text("("),
				Element::group(vec![
					Element::indent(vec![Element::line_break_or_empty(), result]),
					Element::line_break_or_empty(),
				]),
				Element::text(")"),
			])
		} else {
			result
		}
	}

	fn has_brackets(&self, formatter: &MiniZincFormatter) -> bool {
		matches!(
			self,
			ast::Expression::ArrayLiteral(_)
				| ast::Expression::ArrayLiteral2D(_)
				| ast::Expression::SetLiteral(_)
				| ast::Expression::TupleLiteral(_)
				| ast::Expression::RecordLiteral(_)
				| ast::Expression::ArrayComprehension(_)
				| ast::Expression::SetComprehension(_)
		) || formatter.options().keep_parentheses && self.is_parenthesised()
	}
}

impl Format for ast::AnnotatedExpression {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let prec = Precedence::annotated_expression();
		let mut needs_parentheses = false;
		if !formatter.options().keep_parentheses {
			match (formatter.precedence(&self.expression()), &prec) {
				(Precedence::Left(i), Precedence::Left(j)) if i == *j => (),
				(a, b) if a.get() > b.get() => (),
				_ => {
					needs_parentheses = true;
				}
			}
		}
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.expression())
			} else {
				self.expression().format(formatter)
			},
			formatter.format_annotations(self.annotations()),
		])
	}
}

impl Format for ast::Call {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::call().get() > formatter.precedence(&self.function()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.function())
			} else {
				self.function().format(formatter)
			},
			formatter.format_list("(", ")", self.arguments()),
		])
	}
}

impl Format for ast::GeneratorCall {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::generator_call().get() > formatter.precedence(&self.function()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.function())
			} else {
				self.function().format(formatter)
			},
			formatter.format_list(" (", ") ", self.generators()),
			formatter.parenthesise(self.template()),
		])
	}
}

impl Format for ast::PrefixOperator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::prefix_operator(self.operator().name()).get()
				> formatter.precedence(&self.operand()).get();
		Element::sequence(vec![
			if self.operator().name() == "not" {
				Element::sequence(vec![
					Element::text(self.operator().name()),
					Element::line_break_or_space(),
				])
			} else {
				Element::text(self.operator().name())
			},
			if needs_parentheses {
				formatter.parenthesise(self.operand())
			} else {
				self.operand().format(formatter)
			},
		])
	}
}

enum InfixOperatorPart {
	Left(ast::Expression),
	Operator(ast::Operator),
	Right(ast::Expression),
	Comments(Vec<Element>),
}

impl Format for ast::InfixOperator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let prec = Precedence::infix_operator(self.operator().name());
		let mut todo = vec![
			InfixOperatorPart::Right(self.right()),
			InfixOperatorPart::Operator(self.operator()),
			InfixOperatorPart::Left(self.left()),
		];
		let mut elements = Vec::new();
		while let Some(p) = todo.pop() {
			match p {
				InfixOperatorPart::Left(e) => {
					if formatter.options().keep_parentheses && e.is_parenthesised() {
						elements.push(e.format(formatter));
					} else {
						match (formatter.precedence(&e), &prec) {
							(Precedence::Left(i), Precedence::Left(j)) if i == *j => {
								if let Some(op) = e.cast_ref::<ast::InfixOperator>() {
									if let Some(comments) = formatter.take_comments(op) {
										todo.push(InfixOperatorPart::Comments(comments.after));
										todo.push(InfixOperatorPart::Right(op.right()));
										todo.push(InfixOperatorPart::Operator(op.operator()));
										todo.push(InfixOperatorPart::Left(op.left()));
										todo.push(InfixOperatorPart::Comments(comments.before));
									} else {
										todo.push(InfixOperatorPart::Right(op.right()));
										todo.push(InfixOperatorPart::Operator(op.operator()));
										todo.push(InfixOperatorPart::Left(op.left()));
									}
								} else {
									elements.push(e.format(formatter));
								}
							}
							(a, b) if a.get() > b.get() => elements.push(e.format(formatter)),
							_ => elements.push(formatter.parenthesise(e)),
						}
					}
				}
				InfixOperatorPart::Operator(op) => {
					if matches!(op.name(), ".." | "<.." | "<..<" | "..<") {
						elements.push(Element::if_broken(vec![Element::text(" ")]));
						elements.push(Element::text(op.name()));
						elements.push(Element::line_break_or_empty());
					} else {
						elements.push(Element::text(" "));
						elements.push(Element::text(op.name()));
						elements.push(Element::line_break_or_space());
					}
				}
				InfixOperatorPart::Right(e) => {
					if formatter.options().keep_parentheses && e.is_parenthesised() {
						elements.push(e.format(formatter));
					} else {
						match (formatter.precedence(&e), &prec) {
							(Precedence::Right(i), Precedence::Right(j)) if i == *j => {
								if let Some(op) = e.cast_ref::<ast::InfixOperator>() {
									if let Some(comments) = formatter.take_comments(op) {
										todo.push(InfixOperatorPart::Comments(comments.after));
										todo.push(InfixOperatorPart::Right(op.right()));
										todo.push(InfixOperatorPart::Operator(op.operator()));
										todo.push(InfixOperatorPart::Left(op.left()));
										todo.push(InfixOperatorPart::Comments(comments.before));
									} else {
										todo.push(InfixOperatorPart::Right(op.right()));
										todo.push(InfixOperatorPart::Operator(op.operator()));
										todo.push(InfixOperatorPart::Left(op.left()));
									}
								} else {
									elements.push(e.format(formatter));
								}
							}
							(a, b) if a.get() > b.get() => elements.push(e.format(formatter)),
							_ => elements.push(formatter.parenthesise(e)),
						}
					}
				}
				InfixOperatorPart::Comments(comments) => elements.extend(comments),
			}
		}
		let mut iter = elements.into_iter();
		let first = iter.next().unwrap();
		Element::group(vec![first, Element::indent(iter)])
	}
}

impl Format for ast::PostfixOperator {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let needs_parentheses = !formatter.options().keep_parentheses
			&& Precedence::postfix_operator(self.operator().name()).get()
				> formatter.precedence(&self.operand()).get();
		Element::sequence(vec![
			if needs_parentheses {
				formatter.parenthesise(self.operand())
			} else {
				self.operand().format(formatter)
			},
			Element::text(self.operator().name()),
		])
	}
}

impl Format for ast::IfThenElse {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = Vec::new();
		for (i, b) in self.branches().enumerate() {
			if i == 0 {
				elements.push(Element::text("if "));
			} else {
				elements.push(Element::line_break_or_space());
				elements.push(Element::text("elseif "));
			}
			elements.push(b.condition.format(formatter));
			elements.push(Element::text(" then"));
			elements.push(Element::indent(vec![
				Element::line_break_or_space(),
				b.result.format(formatter),
			]));
		}
		if let Some(e) = self.else_result() {
			elements.push(Element::line_break_or_space());
			elements.push(Element::text("else"));
			elements.push(Element::indent(vec![
				Element::line_break_or_space(),
				e.format(formatter),
			]));
		}
		elements.push(Element::line_break_or_space());
		elements.push(Element::text("endif"));
		Element::group(elements)
	}
}

impl Format for ast::Case {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::text("case "),
			self.expression().format(formatter),
			Element::text(" of"),
			Element::indent(self.cases().map(|c| c.format(formatter))),
			Element::line_break(),
			Element::text("endcase"),
		])
	}
}

impl Format for ast::CaseItem {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		Element::sequence(vec![
			Element::line_break(),
			self.pattern().format(formatter),
			Element::text(" =>"),
			if self.value().has_brackets(formatter) {
				Element::sequence(vec![Element::text(" "), self.value().format(formatter)])
			} else {
				Element::group(vec![Element::indent(vec![
					Element::line_break_or_space(),
					self.value().format(formatter),
				])])
			},
			Element::text(","),
		])
	}
}

impl Format for ast::Let {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let it = self.items().collect::<Vec<_>>();
		if it.is_empty() {
			return self.in_expression().format(formatter);
		}
		let items = if it.len() == 1 {
			let item = it.first().unwrap();
			let i = match item {
				ast::LetItem::Constraint(c) => c.format(formatter),
				ast::LetItem::Declaration(d) => d.format(formatter),
			};
			formatter.attach_comments(item, vec![i])
		} else {
			Element::join(
				self.items().map(|item| {
					let i = match &item {
						ast::LetItem::Constraint(c) => c.format(formatter),
						ast::LetItem::Declaration(d) => d.format(formatter),
					};
					formatter.attach_comments(&item, vec![i])
				}),
				vec![Element::text(";"), Element::line_break()],
			)
		};
		Element::group(vec![
			Element::text("let {"),
			Element::indent(vec![
				Element::line_break_or_space(),
				items,
				Element::if_broken(vec![Element::text(";")]),
			]),
			Element::line_break_or_space(),
			Element::text("} in "),
			self.in_expression().format(formatter),
		])
	}
}

impl Format for ast::StringInterpolation {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![Element::text("\"")];
		for p in self.contents() {
			if let Some(e) = p.expression() {
				elements.push(Element::text("\\("));
				elements.push(Element::group(vec![
					Element::indent(vec![Element::line_break_or_empty(), e.format(formatter)]),
					Element::line_break_or_empty(),
				]));
				elements.push(Element::text(")"));
			} else {
				let c = format!("{:?}", p.string().unwrap());
				elements.push(Element::text(&c[1..c.len() - 1]));
			}
		}
		elements.push(Element::text("\""));
		Element::sequence(elements)
	}
}

impl Format for ast::Lambda {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let mut elements = vec![Element::text("lambda ")];
		if let Some(r) = self.return_type() {
			elements.push(r.format(formatter));
			elements.push(Element::text(": "));
		}
		elements.push(formatter.format_list("(", ") =>", self.parameters()));
		elements.push(Element::indent(vec![
			Element::line_break_or_space(),
			self.body().format(formatter),
		]));
		Element::sequence(elements)
	}
}
