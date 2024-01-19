use std::{fmt::Debug, iter::Map, ops::RangeInclusive};

use serde::{Deserialize, Serialize};

/// A sorted collection of inclusive ranges that can be used to represent
/// non-continuous sets of values.
///
/// # Warning
///
/// Although [`RangeList`] can be constructed for elements that do not implement
/// [`std::cmp::Ord`], but do implement [`std::cmp::PartialOrd`], constructor
/// methods, such as the [`FromIterator`] implementation, will panic if the used
/// boundary values cannot be sorted. This requirement allows the usage of types
/// like [`f64`], as long as the user can guarantee that values that cannot be
/// ordered, like `NaN`, will not appear.
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RangeList<E: PartialOrd> {
	ranges: Vec<(E, E)>,
}

impl<E: PartialOrd + Debug> Debug for RangeList<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "RangeList::from_iter([")?;
		let mut first = true;
		for r in self {
			if !first {
				write!(f, ", ")?
			}
			write!(f, "{:?}", r)?;
			first = false;
		}
		write!(f, "])")
	}
}

impl<E: PartialOrd + Clone> IntoIterator for RangeList<E> {
	type Item = RangeInclusive<E>;
	type IntoIter = Map<std::vec::IntoIter<(E, E)>, fn((E, E)) -> RangeInclusive<E>>;

	fn into_iter(self) -> Self::IntoIter {
		self.ranges
			.into_iter()
			.map(|(start, end)| RangeInclusive::new(start, end))
	}
}
impl<'a, E: PartialOrd> IntoIterator for &'a RangeList<E> {
	type Item = RangeInclusive<&'a E>;
	type IntoIter = Map<std::slice::Iter<'a, (E, E)>, fn(&'a (E, E)) -> RangeInclusive<&'a E>>;

	fn into_iter(self) -> Self::IntoIter {
		self.ranges
			.iter()
			.map(|(start, end)| RangeInclusive::new(start, end))
	}
}
impl<E: PartialOrd + Clone> From<&RangeInclusive<E>> for RangeList<E> {
	fn from(value: &RangeInclusive<E>) -> Self {
		if value.is_empty() {
			RangeList { ranges: Vec::new() }
		} else {
			Self {
				ranges: vec![(value.start().clone(), value.end().clone())],
			}
		}
	}
}
impl<E: PartialOrd + Clone> From<RangeInclusive<E>> for RangeList<E> {
	fn from(value: RangeInclusive<E>) -> Self {
		(&value).into()
	}
}
impl<E: PartialOrd + Clone> FromIterator<RangeInclusive<E>> for RangeList<E> {
	fn from_iter<T: IntoIterator<Item = RangeInclusive<E>>>(iter: T) -> Self {
		let mut non_empty: Vec<(E, E)> = iter
			.into_iter()
			.filter_map(|r| {
				if r.is_empty() {
					None
				} else {
					Some((r.start().clone(), r.end().clone()))
				}
			})
			.collect();
		if non_empty.is_empty() {
			return RangeList { ranges: Vec::new() };
		}
		non_empty.sort_by(|a, b| {
			a.0.partial_cmp(&b.0)
				.expect("the order of the bounds in the RangeList cannot be partial")
		});
		let mut it = non_empty.into_iter();
		let mut ranges = Vec::new();
		let mut cur = it.next().unwrap();
		for next in it {
			if cur.1 >= next.0 {
				cur.1 = next.1
			} else {
				ranges.push(cur);
				cur = next;
			}
		}
		ranges.push(cur);
		Self { ranges }
	}
}

impl<E: PartialOrd> RangeList<E> {
	/// Returns `true` if the range list contains no items.
	///
	/// # Examples
	///
	/// ```
	/// # use flatzinc_serde::RangeList;
	/// assert!(!RangeList::from_iter([3..=4]).is_empty());
	/// assert!(RangeList::<i64>::from_iter([]).is_empty());
	/// assert!(RangeList::from_iter([3..=2]).is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.ranges.is_empty()
	}

	/// Returns `true` if `item` is contained in the range list.
	///
	/// # Examples
	///
	/// ```
	/// # use flatzinc_serde::RangeList;
	/// assert!(RangeList::from_iter([1..=4]).contains(&4));
	/// assert!(!RangeList::from_iter([1..=4]).contains(&0));
	///
	/// assert!(RangeList::from_iter([1..=4, 6..=7, -5..=-3]).contains(&7));
	/// assert!(!RangeList::from_iter([1..=4, 6..=7, -5..=-3]).contains(&0));
	/// ```
	pub fn contains(&self, item: &E) -> bool {
		for r in self {
			if r.contains(&item) {
				return true;
			}
		}
		false
	}

	/// Returns the lower bound of the range list, or `None` if the range list is
	/// empty.
	///
	/// # Examples
	///
	/// ```
	/// # use flatzinc_serde::RangeList;
	/// assert_eq!(RangeList::from_iter([1..=4]).lower_bound(), Some(&1));
	/// assert_eq!(RangeList::from_iter([1..=4, 6..=7, -5..=-3]).lower_bound(), Some(&-5));
	///
	/// assert_eq!(RangeList::<i64>::from_iter([]).lower_bound(), None);
	/// ```
	pub fn lower_bound(&self) -> Option<&E> {
		self.ranges.first().map(|(start, _)| start)
	}

	/// Returns the upper bound of the range list, or `None` if the range list is
	/// empty
	///
	/// # Examples
	///
	/// ```
	/// # use flatzinc_serde::RangeList;
	/// assert_eq!(RangeList::from_iter([1..=4]).upper_bound(), Some(&4));
	/// assert_eq!(RangeList::from_iter([1..=4, 6..=7, -5..=-3]).upper_bound(), Some(&7));
	///
	/// assert_eq!(RangeList::<i64>::from_iter([]).upper_bound(), None);
	/// ```
	pub fn upper_bound(&self) -> Option<&E> {
		self.ranges.last().map(|(_, end)| end)
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use super::*;

	#[test]
	fn test_rangelist() {
		let empty: RangeList<i64> = RangeList::default();
		expect![[r#"
    RangeList::from_iter([])
"#]]
		.assert_debug_eq(&empty);
		assert!(empty.is_empty());

		let single_range = RangeList::from_iter([1..=4]);
		expect![[r#"
    RangeList::from_iter([1..=4])
"#]]
		.assert_debug_eq(&single_range);
		assert!(!single_range.is_empty());
		assert!(single_range.contains(&1));
		assert!(single_range.contains(&2));
		assert!(single_range.contains(&4));
		assert!(!single_range.contains(&0));
		assert!(!single_range.contains(&5));

		let multi_range = RangeList::from_iter([1..=4, 6..=7, -5..=-3]);
		expect![[r#"
    RangeList::from_iter([-5..=-3, 1..=4, 6..=7])
"#]]
		.assert_debug_eq(&multi_range);
		assert!(multi_range.contains(&-5));
		assert!(multi_range.contains(&-3));
		assert!(multi_range.contains(&1));
		assert!(multi_range.contains(&4));
		assert!(multi_range.contains(&6));
		assert!(multi_range.contains(&7));
		assert!(!multi_range.contains(&0));
		assert!(!multi_range.contains(&5));
		assert!(!multi_range.contains(&-6));
		assert!(!multi_range.contains(&8));

		let collapse_range = RangeList::from_iter([1..=2, 2..=3, 10..=12, 11..=15]);
		expect![[r#"
    RangeList::from_iter([1..=3, 10..=15])
"#]]
		.assert_debug_eq(&collapse_range);

		let float_range = RangeList::from_iter([0.1..=3.2, 8.1..=11.2, 10.0..=50.0]);
		expect![[r#"
    RangeList::from_iter([0.1..=3.2, 8.1..=50.0])
"#]]
		.assert_debug_eq(&float_range);
	}
}
