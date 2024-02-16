use std::{fmt::Display, iter::once};

use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetView<'a, T> {
	/// Whether the set has a defined lower bound
	pub(crate) has_lb: bool,
	/// Whether the set has a defined upper bound
	pub(crate) has_ub: bool,
	/// Raw storage of bounds plus gaps, assumed to be in the order [lb, ub,
	/// gaps[0].0, gaps[0].1,gaps[1].0,gaps[1].1, etc.]
	///
	/// Note that when has_lb or has_ub is false, these elements are not present
	/// in ranges
	pub(crate) ranges: &'a [T],
}

impl<'a, T: Clone> SetView<'a, T> {
	pub fn lb(&self) -> Option<T> {
		if self.has_lb {
			Some(self.ranges[0].clone())
		} else {
			None
		}
	}
	pub fn ub(&self) -> Option<T> {
		if self.has_ub {
			Some(self.ranges[if self.has_lb { 1 } else { 0 }].clone())
		} else {
			None
		}
	}
	pub fn gaps(&self) -> impl Iterator<Item = (T, T)> + 'a {
		let offset = self.has_lb as usize + self.has_ub as usize;
		self.ranges[offset..].iter().cloned().tuples()
	}
}

impl<'a, T: Display + PartialOrd + Clone> Display for SetView<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.lb(), self.ub()) {
			(Some(lb), Some(ub)) if lb > ub => write!(f, "∅"),
			(None, None) if self.ranges.is_empty() => write!(f, "-∞..+∞"),
			_ => write!(
				f,
				"{}{}..{}",
				self.lb().map(|b| b.to_string()).unwrap_or_default(),
				self.gaps()
					.format_with("", |(lb, ub), f| f(&format_args!("..{}  ∪ {}", lb, ub))),
				self.ub().map(|b| b.to_string()).unwrap_or_default(),
			),
		}
	}
}

impl<'a> SetView<'a, i64> {
	pub fn values(&self) -> Option<impl Iterator<Item = i64> + 'a> {
		if let (Some(lb), Some(ub)) = (self.lb(), self.ub()) {
			Some(
				once(lb)
					.chain(self.ranges[2..].iter().copied())
					.chain(once(ub))
					.tuples()
					.flat_map(|(lb, ub)| lb..=ub),
			)
		} else {
			None
		}
	}
}
