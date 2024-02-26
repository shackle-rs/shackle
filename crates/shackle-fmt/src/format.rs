use rustc_hash::FxHashMap;
use shackle_compiler::syntax::{
	ast::AstNode,
	minizinc::{Expression, MznModel},
};
use tree_sitter::{Node, Query, QueryCursor};
use tree_sitter_minizinc::Precedence;

use crate::{ir::Element, MiniZincFormatOptions};

/// Trait for formatting nodes
pub trait Format {
	/// Format this node
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element;

	/// Whether this node already has brackets around it
	fn has_brackets(&self, _formatter: &MiniZincFormatter) -> bool {
		false
	}
}

impl Format for MznModel {
	fn format(&self, formatter: &mut MiniZincFormatter) -> Element {
		let elements = self
			.items()
			.map(|item| item.format(formatter))
			.collect::<Vec<_>>();
		formatter.attach_model_comments(elements)
	}
}

/// Formatter for MiniZinc
pub struct MiniZincFormatter<'a> {
	model: &'a MznModel,
	options: &'a MiniZincFormatOptions,
	comments: CommentMap,
}

impl<'a> MiniZincFormatter<'a> {
	/// Create a new formatter
	pub fn new(model: &'a MznModel, options: &'a MiniZincFormatOptions) -> Self {
		Self {
			model,
			options,
			comments: CommentMap::new(model),
		}
	}

	/// Run the formatter
	pub fn format(&mut self) -> String {
		let element = self.model.format(self);
		assert!(
			self.comments.map.is_empty(),
			"Did not attach all comments {:?}",
			self.comments.map
		);
		element.format(&self.options.core)
	}

	/// Get the formatting options
	pub fn options(&'a self) -> &'a MiniZincFormatOptions {
		self.options
	}

	/// Attach model comments to these elements
	pub fn attach_model_comments(
		&mut self,
		elements: impl IntoIterator<Item = Element>,
	) -> Element {
		if let Some(c) = self.comments.map.remove(&self.model.cst().root_node().id()) {
			vec![
				Element::sequence(c.before),
				Element::sequence(elements),
				Element::sequence(c.after),
			]
			.into()
		} else {
			Element::sequence(elements)
		}
	}

	/// Attach comments to these elements
	pub fn attach_comments(
		&mut self,
		node: &impl AstNode,
		elements: impl IntoIterator<Item = Element>,
	) -> Element {
		if let Some(c) = self.comments.map.remove(&node.cst_node().as_ref().id()) {
			log::debug!("Attached comments to {}", &node.cst_node().as_ref().id());
			vec![
				Element::sequence(c.before),
				Element::sequence(elements),
				Element::sequence(c.after),
			]
			.into()
		} else {
			Element::sequence(elements)
		}
	}

	/// Take the comments for this node
	pub fn take_comments(&mut self, node: &impl AstNode) -> Option<Comments> {
		self.comments.map.remove(&node.cst_node().as_ref().id())
	}

	/// Format items as a list
	pub fn format_list(
		&mut self,
		open: &str,
		close: &str,
		items: impl Iterator<Item = impl Format>,
	) -> Element {
		let (mut elements, brackets) = items
			.into_iter()
			.map(|item| (item.format(self), item.has_brackets(self)))
			.unzip::<_, _, Vec<_>, Vec<_>>();
		if brackets.is_empty() {
			return vec![Element::text(open), Element::text(close)].into();
		}
		if brackets.len() == 1 && brackets[0] {
			return vec![
				Element::text(open),
				elements.pop().unwrap(),
				Element::text(close),
			]
			.into();
		}
		Element::group(vec![
			Element::text(open),
			Element::indent(vec![
				Element::line_break_or_empty(),
				Element::join(
					elements,
					vec![Element::text(","), Element::line_break_or_space()],
				),
				Element::if_broken(Element::text(",")),
			]),
			Element::line_break_or_empty(),
			Element::text(close),
		])
	}

	/// Format annotations
	pub fn format_annotations(
		&mut self,
		annotations: impl Iterator<Item = impl Format>,
	) -> Element {
		let anns = annotations
			.flat_map(|ann| {
				vec![
					Element::line_break_or_space(),
					Element::text(":: "),
					ann.format(self),
				]
			})
			.collect::<Vec<_>>();
		if anns.is_empty() {
			vec![].into()
		} else {
			Element::group(Element::indent(anns))
		}
	}

	/// Parenthesise a node
	pub fn parenthesise(&mut self, node: impl Format) -> Element {
		if node.has_brackets(self) {
			return vec![Element::text("("), node.format(self), Element::text(")")].into();
		}
		Element::group(vec![
			Element::text("("),
			Element::indent(vec![Element::line_break_or_empty(), node.format(self)]),
			Element::line_break_or_empty(),
			Element::text(")"),
		])
	}

	/// Get precedence for the given expression
	pub fn precedence(&self, expression: &Expression) -> Precedence {
		match expression {
			Expression::Call(_) => Precedence::call(),
			Expression::GeneratorCall(_) => Precedence::generator_call(),
			Expression::ArrayAccess(_) => Precedence::indexed_access(),
			Expression::TupleAccess(_) => Precedence::tuple_access(),
			Expression::RecordAccess(_) => Precedence::record_access(),
			Expression::AnnotatedExpression(_) => Precedence::annotated_expression(),
			Expression::InfixOperator(o) => Precedence::infix_operator(o.operator().name()),
			Expression::PrefixOperator(o) => Precedence::prefix_operator(o.operator().name()),
			Expression::PostfixOperator(o) => Precedence::postfix_operator(o.operator().name()),
			Expression::Lambda(_) | Expression::Let(_) => Precedence::Prec(0),
			_ => Precedence::Prec(i64::MAX),
		}
	}
}

/// Comments to attach to a node
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Comments {
	/// Comments to place before the node
	pub before: Vec<Element>,
	/// Comments to place after the node
	pub after: Vec<Element>,
}

/// Keeps track of where to attach comments
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentMap {
	map: FxHashMap<usize, Comments>,
}

impl CommentMap {
	/// Create a comment map from the given model
	pub fn new(model: &MznModel) -> Self {
		let mut map: FxHashMap<usize, Comments> = FxHashMap::default();

		let query = Query::new(
			&tree_sitter_minizinc::language(),
			tree_sitter_minizinc::COMMENTS_QUERY,
		)
		.expect("Failed to create query");
		let text = model.cst().text().as_bytes();
		let mut cursor = QueryCursor::new();
		let captures = cursor.captures(&query, model.cst().root_node(), text);

		for (c, _) in captures {
			let node = c.captures[0].node;
			let contents = node.utf8_text(text).unwrap();
			let is_line = node.kind() == "line_comment";
			let mut prev = Some(node);
			while let Some(n) = prev {
				if !n.is_extra() {
					break;
				}
				prev = n.prev_named_sibling();
			}
			let mut next_non_extra = Some(node);
			while let Some(n) = next_non_extra {
				if !n.is_extra() {
					break;
				}
				next_non_extra = n.next_sibling();
			}
			let blank_line_before =
				node.prev_sibling()
					.map(|n| n.end_position().row < node.start_position().row.saturating_sub(1))
					.unwrap_or_default() && node
					.parent()
					.map(|n| n.kind() == "source_file")
					.unwrap_or_default();
			let is_suffix =
				prev.map(|p| p.end_position().row == node.start_position().row)
					.unwrap_or_default() && (is_line
					|| next_non_extra
						.map(|n| n.start_position().row > node.end_position().row || !n.is_named())
						.unwrap_or(true));
			if is_suffix {
				let attach_to = ensure_valid_node(prev.unwrap());
				log::debug!(
					"Attaching comment {:?} after {} ({})",
					contents,
					attach_to.kind(),
					attach_to.id(),
				);
				let comments = map.entry(attach_to.id()).or_default();
				comments.after.push(Element::break_parent());
				comments
					.after
					.push(Element::line_suffix(format!(" {}", contents.trim_end())));
			} else {
				let mut next = Some(node);
				while let Some(n) = next {
					if !n.is_extra() {
						break;
					}
					next = n.next_named_sibling();
				}
				let (mut attach_to, before) = if let Some(n) = next {
					// Have a named sibling to attach to
					(n, true)
				} else if let Some(n) = prev {
					// Nothing after the comment to attach to, so attach to end of previous
					(n, false)
				} else {
					// Nothing at this level to attach to, so attach to parent instead
					let mut n = node.parent().unwrap();
					while !n.is_named() {
						n = n.parent().unwrap();
					}
					(n, true)
				};
				attach_to = ensure_valid_node(attach_to);
				let comments = map.entry(attach_to.id()).or_default();
				if before {
					log::debug!(
						"Attaching comment {:?} before {} ({})",
						contents,
						attach_to.kind(),
						attach_to.id(),
					);
					if blank_line_before {
						comments.before.push(Element::line_break());
					}
					if is_line {
						comments.before.push(Element::text(contents));
						comments.before.push(Element::line_break());
					} else {
						comments.before.push(Element::text(contents));
						if node
							.next_sibling()
							.map(|n| node.end_position().row == n.start_position().row)
							.unwrap_or_default()
						{
							comments.before.push(Element::line_break_or_space());
						} else {
							comments.before.push(Element::line_break());
						}
					}
				} else {
					log::debug!(
						"Attaching comment {:?} after {} ({})",
						contents,
						attach_to.kind(),
						attach_to.id(),
					);
					if blank_line_before {
						comments.after.push(Element::line_break());
					}
					comments.after.push(Element::line_break());
					comments.after.push(Element::text(contents));
				}
			}
		}
		CommentMap { map }
	}
}

fn ensure_valid_node(mut node: Node<'_>) -> Node<'_> {
	while matches!(node.kind(), "strategy") {
		node = node.parent().unwrap()
	}
	while node.kind() == "parenthesised_expression" {
		node = node.child_by_field_name("expression").unwrap();
	}
	node
}
