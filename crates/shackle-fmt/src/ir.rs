//! A minimal intermediate representation for producing formatted code.
//!
//! API inspired by [prettier](https://github.com/prettier/prettier/blob/main/commands.md) and
//! [Wadler's paper](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf).
//!
//! Algorithm based on [Pugh and Sinofsky's paper](https://hdl.handle.net/1813/6648).

use std::{collections::VecDeque, fmt::Debug};

/// Formatting options
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormatOptions {
	/// Target maximum line length
	pub line_width: usize,
	/// Whether to indent using tabs
	pub use_tabs: bool,
	/// Size of indent
	pub indent_size: usize,
}

impl Default for FormatOptions {
	fn default() -> Self {
		Self {
			line_width: 80,
			use_tabs: true,
			indent_size: 4,
		}
	}
}

impl<T: IntoIterator<Item = Element>> From<T> for Element {
	fn from(value: T) -> Self {
		Element::sequence(value)
	}
}

/// An element to be formatted.
///
/// The constructor functions are used to produce an IR indicating how to format the code.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Element(ElementData);

impl Element {
	/// A raw string to be printed.
	pub fn text(text: impl ToString) -> Self {
		let string = text.to_string();
		let mut elements = Vec::with_capacity(1);
		for (i, s) in string.split('\n').enumerate() {
			if i > 0 {
				elements.push(Self(ElementData::LineBreak { literal: true }));
			}
			elements.push(Self(ElementData::Text(s.to_string())));
		}
		Self::sequence(elements)
	}

	/// A sequence of elements.
	///
	/// Does not affect breaking (i.e. whether the elements are broken is determined by whether the containing group is broken).
	pub fn sequence(elements: impl IntoIterator<Item = Element>) -> Self {
		let mut elements = elements.into_iter().collect::<Vec<_>>();
		if elements.len() == 1 {
			Self(elements.pop().unwrap().0)
		} else {
			Self(ElementData::Sequence(elements))
		}
	}

	/// A group to try to fit on one line, or else break apart.
	///
	/// This is different from a `sequence` in that a group always tries to fit its contents on one line,
	/// even if its contained inside another group that is broken.
	///
	/// When a group is broken apart, we simply render the children in broken mode - no line breaks are automatically
	/// inserted. The `if_broken` and `line_break_or_space` functions (and similar) are used to control where line
	/// breaks can appear.
	pub fn group(element: impl Into<Element>) -> Self {
		Self(ElementData::Group(Box::new(element.into())))
	}

	/// A group which should be broken apart
	#[allow(dead_code)]
	pub fn broken_group(element: impl Into<Element>) -> Self {
		Self::group(vec![Self::break_parent(), element.into()])
	}

	/// Increments the indentation level for the given element.
	///
	/// Indentation is added when a line break occurs.
	pub fn indent(element: impl Into<Element>) -> Self {
		Self(ElementData::Indent(Box::new(element.into())))
	}

	/// Join the elements together with the given separator.
	pub fn join(
		elements: impl IntoIterator<Item = Element>,
		separator: impl Into<Element>,
	) -> Self {
		let elems = elements.into_iter().collect::<Vec<Element>>();
		if elems.len() <= 1 {
			return Self::sequence(elems);
		}
		let sep = separator.into();
		let mut items = Vec::with_capacity(2 * elems.len() - 1);
		let mut iter = elems.into_iter();
		items.push(iter.next().unwrap());
		for i in iter {
			items.push(sep.clone());
			items.push(i);
		}
		Self::sequence(items)
	}

	/// Chooses between the elements based on if the current group is broken.
	pub fn if_broken_else(broken: impl Into<Element>, unbroken: impl Into<Element>) -> Self {
		Self::sequence([Self::if_broken(broken), Self::if_unbroken(unbroken)])
	}

	/// Emits the element if the current group is broken.
	pub fn if_broken(broken: impl Into<Element>) -> Self {
		Self(ElementData::IfBroken(Box::new(broken.into())))
	}

	/// Emits the element if the current group is unbroken.
	pub fn if_unbroken(unbroken: impl Into<Element>) -> Self {
		Self(ElementData::IfUnbroken(Box::new(unbroken.into())))
	}

	/// Text to place at the end of the current line.
	pub fn line_suffix(suffix: impl ToString) -> Self {
		Self(ElementData::LineSuffix(suffix.to_string()))
	}

	/// A line break, which forces parent groups to break.
	pub fn line_break() -> Self {
		Self(ElementData::LineBreak { literal: false })
	}

	/// Empty if the group is not broken, otherwise breaks here.
	pub fn line_break_or_empty() -> Self {
		Self::if_broken(Self::line_break())
	}

	/// A single space if the group is not broken, otherwise breaks here.
	pub fn line_break_or_space() -> Self {
		Self::if_broken_else(Self::line_break(), Self::text(" "))
	}

	/// A line break which doesn't indent afterwards.
	#[allow(dead_code)]
	pub fn literal_line_break() -> Self {
		Self(ElementData::LineBreak { literal: true })
	}

	/// Force all parent groups to break
	pub fn break_parent() -> Self {
		Self(ElementData::BreakParent)
	}

	/// Format this element
	pub fn format(&self, options: &FormatOptions) -> String {
		Formatter::default().pretty_print(self, options)
	}
}

impl Debug for Element {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.0 {
			ElementData::Text(t) => f.debug_tuple("Element::text").field(t).finish(),
			ElementData::Sequence(c) => f.debug_tuple("Element::sequence").field(c).finish(),
			ElementData::Group(e) => f.debug_tuple("Element::group").field(&**e).finish(),
			ElementData::Indent(e) => f.debug_tuple("Element::indent").field(&**e).finish(),
			ElementData::IfBroken(e) => f.debug_tuple("Element::if_broken").field(&**e).finish(),
			ElementData::IfUnbroken(e) => {
				f.debug_tuple("Element::if_unbroken").field(&**e).finish()
			}
			ElementData::LineSuffix(s) => f.debug_tuple("Element::line_suffix").field(s).finish(),
			ElementData::LineBreak { literal } => f.write_str(if *literal {
				"Element::literal_line_break()"
			} else {
				"Element::line_break()"
			}),
			ElementData::BreakParent => f.write_str("Element::break_parent()"),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ElementData {
	Text(String),
	Sequence(Vec<Element>),
	Group(Box<Element>),
	Indent(Box<Element>),
	IfBroken(Box<Element>),
	IfUnbroken(Box<Element>),
	LineSuffix(String),
	LineBreak { literal: bool },
	BreakParent,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct BufferItem {
	condition: BufferItemCondition,
	data: BufferItemData,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum BufferItemCondition {
	None,
	Broken { level: isize, group: usize },
	Unbroken { level: isize, group: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum BufferItemData {
	Text(String),
	LineBreak { literal: bool },
	Indent,
	Outdent,
	LineSuffix(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct BreakItem {
	items_enqueued: usize,
	group_level: isize,
	group: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Formatter {
	total_items_enqueued: usize,
	total_items_flushed: usize,
	column: usize,
	enqueued_chars: usize,
	indent_level: usize,
	current_level: isize,
	break_level: isize,
	current_group: usize,
	break_group: usize,
	buffer: VecDeque<BufferItem>,
	breaks: VecDeque<BreakItem>,
	line_suffix: Vec<String>,
}

impl Default for Formatter {
	fn default() -> Self {
		Self {
			total_items_enqueued: 0,
			total_items_flushed: 0,
			column: 0,
			enqueued_chars: 0,
			indent_level: 0,
			current_level: 0,
			break_level: -1,
			current_group: 0,
			break_group: 0,
			buffer: VecDeque::new(),
			breaks: VecDeque::new(),
			line_suffix: Vec::new(),
		}
	}
}

impl Formatter {
	fn pretty_print(&mut self, element: &Element, options: &FormatOptions) -> String {
		let mut output = String::new();
		let mut todo = vec![(false, element)];
		let mut current_condition = BufferItemCondition::None;
		let mut conditions = vec![];
		let mut group_count = 0;
		let mut groups = vec![];
		while let Some((done, elem @ Element(data))) = todo.pop() {
			match (done, data) {
				(_, ElementData::Text(text)) => {
					log::debug!("Got {:?} ({:?})", elem, current_condition);
					let immediate_write = match current_condition {
						BufferItemCondition::None => true,
						BufferItemCondition::Broken { level, group } => {
							self.break_level > level || group == self.break_group
						}
						BufferItemCondition::Unbroken { level, group } => {
							if self.break_level > level || group == self.break_group {
								continue;
							}
							false
						}
					};
					if immediate_write {
						while self.column + self.enqueued_chars + text.len() > options.line_width {
							log::debug!("Not enough space, need line break");
							if let Some(temp) = self.breaks.pop_back() {
								log::debug!("Took line break {:?}", temp);
								self.break_level = temp.group_level;
								self.break_group = temp.group;
								self.print_buffer(temp.items_enqueued, &mut output, options);
								self.break_level = self.break_level.min(self.current_level);
								self.break_group = self.break_group.min(self.current_group);
							} else {
								break;
							}
						}
						self.buffer.push_back(BufferItem {
							condition: BufferItemCondition::None,
							data: BufferItemData::Text(text.clone()),
						});
						self.enqueued_chars += text.len();
					} else {
						self.buffer.push_back(BufferItem {
							condition: current_condition,
							data: BufferItemData::Text(text.clone()),
						});
						if matches!(current_condition, BufferItemCondition::Unbroken { .. }) {
							self.enqueued_chars += text.len();
						}
					}
					self.total_items_enqueued += 1;
				}
				(_, ElementData::Sequence(children)) => {
					todo.extend(children.iter().rev().map(|child| (false, child)));
				}
				(false, ElementData::Group(child)) => {
					log::debug!("Start {:?} ({:?})", elem, current_condition);
					groups.push(self.current_group);
					group_count += 1;
					self.current_group = group_count;
					todo.push((true, elem));
					todo.push((false, child));
					self.current_level += 1;
				}
				(true, ElementData::Group(_)) => {
					log::debug!("End {:?})", elem);
					if self.break_level > self.current_level
						|| self.break_group == self.current_group
					{
						self.breaks.clear();
						self.print_buffer(self.total_items_enqueued, &mut output, options);
					}
					self.current_level -= 1;
					self.current_group = groups.pop().unwrap();
				}
				(false, ElementData::Indent(child)) => {
					log::debug!("Start {:?} ({:?})", elem, current_condition);
					todo.push((true, elem));
					todo.push((false, child));
					self.buffer.push_back(BufferItem {
						condition: current_condition,
						data: BufferItemData::Indent,
					});
					self.total_items_enqueued += 1;
				}
				(true, ElementData::Indent(_)) => {
					log::debug!("End {:?}", elem);
					self.buffer.push_back(BufferItem {
						condition: current_condition,
						data: BufferItemData::Outdent,
					});
					self.total_items_enqueued += 1;
				}
				(false, ElementData::IfBroken(child)) => {
					if matches!(
						current_condition,
						BufferItemCondition::None | BufferItemCondition::Broken { .. }
					) {
						log::debug!("Start condition {:?}", elem);
						conditions.push(current_condition);
						current_condition = BufferItemCondition::Broken {
							level: self.current_level,
							group: self.current_group,
						};
						todo.push((true, elem));
						todo.push((false, child));
					} else {
						log::debug!("Discarding unreachable condition {:?}", elem);
					}
				}
				(false, ElementData::IfUnbroken(child)) => {
					if matches!(
						current_condition,
						BufferItemCondition::None | BufferItemCondition::Unbroken { .. }
					) {
						log::debug!("Start condition {:?}", elem);
						conditions.push(current_condition);
						current_condition = BufferItemCondition::Unbroken {
							level: self.current_level,
							group: self.current_group,
						};
						todo.push((true, elem));
						todo.push((false, child));
					} else {
						log::debug!("Discarding unreachable condition {:?}", elem);
					}
				}
				(true, ElementData::IfBroken(_)) | (true, ElementData::IfUnbroken(_)) => {
					log::debug!("End condition {:?}", elem);
					current_condition = conditions.pop().unwrap();
				}
				(_, ElementData::LineSuffix(child)) => {
					self.buffer.push_back(BufferItem {
						condition: current_condition,
						data: BufferItemData::LineSuffix(child.clone()),
					});
					self.total_items_enqueued += 1;
				}
				(_, ElementData::LineBreak { literal }) => {
					log::debug!("Got {:?}", elem);
					let immediate_break = match current_condition {
						BufferItemCondition::None => true,
						BufferItemCondition::Broken { level, group } => {
							self.break_level > level || group == self.break_group
						}
						_ => false,
					};
					let data = BufferItemData::LineBreak { literal: *literal };
					if immediate_break {
						self.buffer.push_back(BufferItem {
							condition: BufferItemCondition::None,
							data,
						});
						self.total_items_enqueued += 1;
						self.breaks.clear();
						self.break_level = self.current_level;
						self.break_group = self.current_group;
						self.print_buffer(self.total_items_enqueued, &mut output, options);
					} else {
						self.buffer.push_back(BufferItem {
							condition: current_condition,
							data,
						});
						self.total_items_enqueued += 1;
						while self
							.breaks
							.front()
							.map(|item| item.group_level >= self.current_level)
							.unwrap_or_default()
						{
							self.breaks.pop_front();
						}
						let line_break = BreakItem {
							items_enqueued: self.total_items_enqueued,
							group_level: self.current_level,
							group: self.current_group,
						};
						self.breaks.push_front(line_break);
					}
				}
				(_, ElementData::BreakParent) => {
					log::debug!("Got {:?} ({:?})", elem, current_condition);
					self.break_level = self.current_level;
					self.break_group = self.current_group;
					self.print_buffer(self.total_items_enqueued, &mut output, options);
				}
			}
		}
		self.print_buffer(self.total_items_enqueued, &mut output, options);
		assert!(self.buffer.is_empty());
		output
	}

	fn print_buffer(&mut self, until: usize, output: &mut String, options: &FormatOptions) {
		while self.total_items_flushed < until {
			log::debug!(
				"break level: {}, break group: {}",
				self.break_level,
				self.break_group
			);
			self.total_items_flushed += 1;
			let item = self.buffer.pop_front().unwrap();
			let print = match item.condition {
				BufferItemCondition::Broken { level, group } => {
					self.break_level > level || group == self.break_group
				}
				BufferItemCondition::Unbroken { level, group } => {
					self.break_level <= level && group != self.break_group
				}
				BufferItemCondition::None => true,
			};
			if !print {
				if let BufferItem {
					condition: BufferItemCondition::Unbroken { .. },
					data: BufferItemData::Text(text),
				} = &item
				{
					self.enqueued_chars -= text.len();
				}
				log::debug!("Skipping {:?}", item);
				continue;
			}
			log::debug!("Printing {:?}", item);
			match item.data {
				BufferItemData::Text(text) => {
					output.push_str(&text);
					if !matches!(item.condition, BufferItemCondition::Broken { .. }) {
						self.enqueued_chars -= text.len();
					}
					self.column += text.len();
				}
				BufferItemData::LineBreak { literal } => {
					for suffix in self.line_suffix.drain(..) {
						output.push_str(&suffix);
					}
					output.push('\n');
					if literal {
						self.column = 0;
					} else {
						if options.use_tabs {
							output.extend(std::iter::repeat('\t').take(self.indent_level));
						} else {
							output.extend(
								std::iter::repeat(' ')
									.take(options.indent_size * self.indent_level),
							);
						}
						self.column = options.indent_size * self.indent_level;
					}
				}
				BufferItemData::Indent => {
					self.indent_level += 1;
				}
				BufferItemData::Outdent => {
					self.indent_level -= 1;
				}
				BufferItemData::LineSuffix(text) => {
					self.line_suffix.push(text);
				}
			}
		}
		log::debug!(
			"Done printing ({}/{} items, {} on current line)",
			self.total_items_flushed,
			self.total_items_enqueued,
			self.column + self.enqueued_chars
		);
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::*;

	fn js_document() -> Element {
		Element::sequence(vec![
			Element::text("function "),
			Element::text("foo"),
			Element::group(vec![
				Element::text("("),
				Element::indent(vec![
					Element::line_break_or_empty(),
					Element::text("abacus"),
					Element::text(","),
					Element::line_break_or_space(),
					Element::text("banana"),
				]),
				Element::if_broken(vec![Element::text(",")]),
				Element::line_break_or_empty(),
				Element::text(")"),
			]),
			Element::text(" "),
			Element::text("{"),
			Element::indent(vec![
				Element::line_break(),
				Element::group(vec![
					Element::text("if ("),
					Element::group(vec![
						Element::indent(vec![
							Element::line_break_or_empty(),
							Element::group(vec![
								Element::text("abacus"),
								Element::text("."),
								Element::text("beads"),
							]),
							Element::text(" "),
							Element::group(vec![
								Element::text("<"),
								Element::line_break_or_space(),
								Element::text("banana"),
							]),
						]),
						Element::line_break_or_empty(),
					]),
					Element::text(")"),
					Element::text(" "),
					Element::text("{"),
					Element::indent(vec![
						Element::line_break(),
						Element::text("throw"),
						Element::text(" "),
						Element::text("new "),
						Element::text("Error"),
						Element::group(vec![
							Element::text("("),
							Element::indent(vec![
								Element::line_break_or_empty(),
								Element::text("\"Not enough beads\""),
							]),
							Element::if_broken(vec![Element::text(",")]),
							Element::line_break_or_empty(),
							Element::text(")"),
						]),
						Element::text(";"),
					]),
					Element::line_break(),
					Element::text("}"),
				]),
				Element::line_break(),
				Element::group(vec![
					Element::group(vec![
						Element::text("abacus"),
						Element::text("."),
						Element::text("beads"),
					]),
					Element::text(" "),
					Element::text("-="),
					Element::indent(vec![
						Element::line_break_or_space(),
						Element::text("banana"),
					]),
				]),
				Element::text(";"),
			]),
			Element::line_break(),
			Element::text("}"),
			Element::line_break(),
		])
	}

	#[test]
	fn test_format_js_80() {
		let document = js_document();
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    function foo(abacus, banana) {
    	if (abacus.beads < banana) {
    		throw new Error("Not enough beads");
    	}
    	abacus.beads -= banana;
    }
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_js_30() {
		let document = js_document();
		let formatted = document.format(&FormatOptions {
			line_width: 30,
			..Default::default()
		});
		let expected = expect![[r#"
    function foo(abacus, banana) {
    	if (
    		abacus.beads < banana
    	) {
    		throw new Error(
    			"Not enough beads",
    		);
    	}
    	abacus.beads -= banana;
    }
"#]];
		expected.assert_eq(&formatted);
	}

	fn json_document() -> Element {
		Element::sequence(vec![
			Element::group([
				Element::text("{"),
				Element::indent([
					Element::line_break_or_space(),
					Element::group(vec![
						Element::text("\"foo\""),
						Element::text(":"),
						Element::text(" "),
						Element::text("1"),
					]),
					Element::text(","),
					Element::line_break_or_space(),
					Element::group(vec![
						Element::text("\"bar\""),
						Element::text(":"),
						Element::text(" "),
						Element::group(vec![
							Element::text("["),
							Element::indent(vec![
								Element::line_break_or_empty(),
								Element::text("\"here are some long\""),
								Element::text(","),
								Element::line_break_or_space(),
								Element::text("\"strings in a list\""),
							]),
							Element::line_break_or_empty(),
							Element::text("]"),
						]),
					]),
					Element::text(","),
					Element::line_break_or_space(),
					Element::group(vec![
						Element::text("\"qux\""),
						Element::text(":"),
						Element::text(" "),
						Element::group(vec![
							Element::text("{"),
							Element::indent(vec![
								Element::line_break_or_space(),
								Element::group(vec![
									Element::text("\"a\""),
									Element::text(":"),
									Element::text(" "),
									Element::text("true"),
								]),
								Element::text(","),
								Element::line_break_or_space(),
								Element::group([
									Element::text("\"b\""),
									Element::text(":"),
									Element::text(" "),
									Element::text("false"),
								]),
							]),
							Element::line_break_or_space(),
							Element::text("}"),
						]),
					]),
				]),
				Element::line_break_or_space(),
				Element::text("}"),
			]),
			Element::line_break(),
		])
	}

	#[test]
	fn test_format_json_120() {
		let document = json_document();
		let formatted = document.format(&FormatOptions {
			line_width: 120,
			..Default::default()
		});
		let expected = expect![[r#"
            { "foo": 1, "bar": ["here are some long", "strings in a list"], "qux": { "a": true, "b": false } }
        "#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_json_80() {
		let document = json_document();
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    {
    	"foo": 1,
    	"bar": ["here are some long", "strings in a list"],
    	"qux": { "a": true, "b": false }
    }
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_json_30() {
		let document = json_document();
		let formatted = document.format(&FormatOptions {
			line_width: 30,
			..Default::default()
		});
		let expected = expect![[r#"
    {
    	"foo": 1,
    	"bar": [
    		"here are some long",
    		"strings in a list"
    	],
    	"qux": {
    		"a": true,
    		"b": false
    	}
    }
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_broken_group() {
		let document = Element::broken_group(vec![Element::broken_group(vec![
			Element::text("a"),
			Element::line_break_or_space(),
			Element::text("b"),
		])]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    a
    b"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_literal_line() {
		let document = Element::broken_group(vec![Element::indent(vec![
			Element::line_break(),
			Element::text("a"),
			Element::literal_line_break(),
			Element::text("b"),
			Element::line_break(),
			Element::text("c"),
		])]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"

    	a
    b
    	c"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_conditional() {
		let document = Element::group(vec![
			Element::broken_group(Element::if_broken_else(
				Element::text("A"),
				Element::text("B"),
			)),
			Element::group(Element::if_broken_else(
				Element::text("C"),
				Element::text("D"),
			)),
			Element::line_break(),
		]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    AD
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_conditional_2() {
		let document = Element::group(vec![
			Element::text("foo"),
			Element::group(vec![
				Element::line_break_or_space(),
				Element::text("some long text"),
			]),
			Element::line_break(),
		]);
		let formatted = document.format(&FormatOptions {
			line_width: 10,
			..Default::default()
		});
		let expected = expect![[r#"
    foo
    some long text
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_conditional_3() {
		let document = Element::sequence([
			Element::text("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
			Element::group(Element::indent(Element::sequence(vec![
				Element::if_broken(Element::line_break()),
				Element::if_unbroken(Element::text(" ")),
				Element::text("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
			]))),
			Element::group(Element::indent(Element::sequence(vec![
				Element::if_broken(Element::line_break()),
				Element::if_unbroken(Element::text(" ")),
				Element::text("cccccccccccccccccccccccccccccccccccccccccccccccccc"),
			]))),
			Element::line_break(),
		]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    	bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    	cccccccccccccccccccccccccccccccccccccccccccccccccc
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_format_conditional_4() {
		let document = Element::sequence([
			Element::text("predicate foo() ="),
			Element::group(Element::indent(Element::sequence([
				Element::if_broken(Element::line_break()),
				Element::if_unbroken(Element::text(" ")),
				Element::group(Element::sequence([
					Element::text("if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa then"),
					Element::indent(Element::sequence([
						Element::if_broken(Element::line_break()),
						Element::if_unbroken(Element::text(" ")),
						Element::text("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
					])),
					Element::if_broken(Element::line_break()),
					Element::if_unbroken(Element::text(" ")),
					Element::text("endif"),
				])),
			]))),
			Element::text(";"),
			Element::line_break(),
		]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    predicate foo() =
    	if aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa then
    		bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    	endif;
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_groups_1() {
		let document = Element::sequence([
			Element::text("test mzn_can_extend_array_opt"),
			Element::group(Element::sequence([
				Element::text("("),
				Element::indent(Element::sequence([
					Element::if_broken(Element::line_break()),
					Element::group(Element::sequence([
						Element::text("array ["),
						Element::indent(Element::sequence([
							Element::if_broken(Element::line_break()),
							Element::text("$E"),
							Element::if_broken(Element::text(",")),
						])),
						Element::if_broken(Element::line_break()),
						Element::text("] of "),
					])),
					Element::text("any $T: x,"),
					Element::if_broken(Element::line_break()),
					Element::if_unbroken(Element::text(" ")),
					Element::text("var $F: idx"),
					Element::if_broken(Element::text(",")),
				])),
				Element::if_broken(Element::line_break()),
				Element::text(")"),
			])),
			Element::group(Element::indent(Element::sequence([
				Element::if_broken(Element::line_break()),
				Element::if_unbroken(Element::text(" ")),
				Element::text(":: mzn_unreachable"),
			]))),
			Element::text(";"),
			Element::line_break(),
		]);
		let formatted = document.format(&Default::default());
		let expected = expect![[r#"
    test mzn_can_extend_array_opt(array [$E] of any $T: x, var $F: idx)
    	:: mzn_unreachable;
"#]];
		expected.assert_eq(&formatted);
	}

	#[test]
	fn test_ir_1() {
		let document = Element::sequence(vec![Element::group(vec![Element::indent(vec![
			Element::line_break(),
			Element::group(vec![Element::text("foo")]),
		])])]);
		let expected = expect![[r#"
    Element::group(
        Element::indent(
            Element::sequence(
                [
                    Element::line_break(),
                    Element::group(
                        Element::text(
                            "foo",
                        ),
                    ),
                ],
            ),
        ),
    )
"#]];
		expected.assert_debug_eq(&document);
	}

	#[test]
	fn test_ir_2() {
		let document = Element::sequence(vec![
			Element::group(vec![Element::if_broken_else(
				vec![Element::text("impossible else")],
				vec![Element::line_break()],
			)]),
			Element::group(vec![Element::if_broken_else(
				vec![Element::line_break()],
				vec![Element::text("no break")],
			)]),
		]);
		let expected = expect![[r#"
    Element::sequence(
        [
            Element::group(
                Element::sequence(
                    [
                        Element::if_broken(
                            Element::text(
                                "impossible else",
                            ),
                        ),
                        Element::if_unbroken(
                            Element::line_break(),
                        ),
                    ],
                ),
            ),
            Element::group(
                Element::sequence(
                    [
                        Element::if_broken(
                            Element::line_break(),
                        ),
                        Element::if_unbroken(
                            Element::text(
                                "no break",
                            ),
                        ),
                    ],
                ),
            ),
        ],
    )
"#]];
		expected.assert_debug_eq(&document);
	}

	#[test]
	fn test_ir_3() {
		let document = Element::sequence(vec![
			Element::sequence(vec![
				Element::sequence(vec![Element::text("a")]),
				Element::text("b"),
			]),
			Element::sequence(vec![
				Element::text("c"),
				Element::sequence(vec![Element::text("d")]),
			]),
			Element::sequence(vec![Element::sequence(vec![
				Element::text("e"),
				Element::text("f"),
			])]),
		]);
		let expected = expect![[r#"
    Element::sequence(
        [
            Element::sequence(
                [
                    Element::text(
                        "a",
                    ),
                    Element::text(
                        "b",
                    ),
                ],
            ),
            Element::sequence(
                [
                    Element::text(
                        "c",
                    ),
                    Element::text(
                        "d",
                    ),
                ],
            ),
            Element::sequence(
                [
                    Element::text(
                        "e",
                    ),
                    Element::text(
                        "f",
                    ),
                ],
            ),
        ],
    )
"#]];
		expected.assert_debug_eq(&document);
	}
}
