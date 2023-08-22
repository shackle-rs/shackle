//! Handling of errors and warnings during compilation

mod error;
mod warning;

use std::sync::Arc;

pub use error::*;
pub use warning::*;

/// Helper for collecting diagnostics of type `T`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Diagnostics<T> {
	children: Vec<DiagnosticItems<T>>,
}

impl<T> Diagnostics<T> {
	/// Add the given diagnostics vector
	pub fn extend(&mut self, items: Arc<Vec<T>>) {
		self.children.push(DiagnosticItems::Multiple(items));
	}

	/// Add the given diagnostic
	pub fn push(&mut self, item: T) {
		self.children.push(DiagnosticItems::Single(Box::new(item)));
	}
}

impl<T> Default for Diagnostics<T> {
	fn default() -> Self {
		Self {
			children: Vec::new(),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum DiagnosticItems<T> {
	Single(Box<T>),
	Multiple(Arc<Vec<T>>),
}

impl<T> Diagnostics<T> {
	/// True if there are no diagnostics
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Get the number of diagnostics
	pub fn len(&self) -> usize {
		self.children.iter().fold(0, |acc, i| {
			acc + match i {
				DiagnosticItems::Single(_) => 1,
				DiagnosticItems::Multiple(items) => items.len(),
			}
		})
	}

	/// Get an iterator over the diagnostics
	pub fn iter(&self) -> impl '_ + Iterator<Item = &T> {
		let mut iter = self.children.iter();
		let mut todo = Vec::new();
		std::iter::from_fn(move || loop {
			if let Some(d) = todo.pop() {
				return Some(d);
			}
			if let Some(it) = iter.next() {
				match it {
					DiagnosticItems::Multiple(items) => {
						todo.extend(items.iter());
					}
					DiagnosticItems::Single(it) => todo.push(it),
				}
			} else {
				return None;
			}
		})
	}
}
