//! A library for working with set of values represented as inclusive ranges.
//!
//! This library provides a [`RangeList`] struct that can be used to represent
//! sets of values as a collection of inclusive ranges. The ranges are stored
//! in a deduplicated sorted order.
//!
//! Additionally, the library provides a [`IntervalIter`] trait that can be
//! implemented by types that can provide an iterator of sorted inclusive
//! ranges. This trait then provides methods to perform set operations on
//! any combination of types that implement it.

use std::{
	fmt::{Debug, Display},
	iter::Map,
	ops::RangeInclusive,
};

use castaway::match_type;

/// Macro that implments `RangeList::card` for integral types.
macro_rules! impl_card {
	( $($type:ty),* $(,)? ) => {
		$(
			impl RangeList<$type> {
				/// Returns the number of elements contained within the RangeList.
				pub fn card(&self) -> usize {
					self.iter().map(|r| (r.end() - r.start() + 1) as usize).sum()
				}
			}
		)*
	};
}
impl_card!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

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
#[derive(Default, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct RangeList<E: PartialOrd> {
	/// Memory representation of the ranges
	ranges: Vec<(E, E)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RangeOrdering {
	/// A compared range is strictly less than another.
	Less = -1,
	/// A compared range overlaps with another.
	Overlap = 0,
	/// A compared range is strictly greater than another.
	Greater = 1,
}

/// A trait that provides operations on iterators of orderdered intervals.
pub trait IntervalIter<E: PartialOrd> {
	/// The type of the interval iterator.
	type Iter: Iterator<Item = RangeInclusive<E>>;
	/// Returns an iterator over the ordered intervals.
	fn intervals(&self) -> Self::Iter;

	/// Returns whether `self` and `other` are disjoint sets
	fn disjoint<O: IntervalIter<E> + ?Sized>(&self, other: &O) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match overlap(l, r) {
				RangeOrdering::Less => {
					// Move to next "self range"
					let _ = lhs.next();
				}
				RangeOrdering::Overlap => return false,
				RangeOrdering::Greater => {
					// Move to next "other range"
					let _ = rhs.next();
				}
			}
		}
		true
	}

	/// Return the set intersection of two interval iterators.
	fn intersect<O, R>(&self, other: &O) -> R
	where
		E: Clone,
		O: IntervalIter<E>,
		R: FromIterator<RangeInclusive<E>>,
	{
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match overlap(l, r) {
				RangeOrdering::Less => {
					let _ = lhs.next();
				}
				RangeOrdering::Greater => {
					let _ = rhs.next();
				}
				RangeOrdering::Overlap => {
					ranges.push(max(l.start(), r.start()).clone()..=min(l.end(), r.end()).clone());
					if l.end() <= r.end() {
						let _ = lhs.next();
					} else {
						let _ = rhs.next();
					}
				}
			}
		}
		ranges.into_iter().collect()
	}

	/// Returns whether `self` is a subset of `other`
	fn subset<O: IntervalIter<E> + ?Sized>(&self, other: &O) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match overlap(l, r) {
				RangeOrdering::Overlap if r.start() <= l.start() && l.end() <= r.end() => {
					// Current "self range" is included in the current other range
					// Move to next "self range" that needs to be covered
					let _ = lhs.next();
				}
				RangeOrdering::Greater => {
					// Move to next "other range"
					let _ = rhs.next();
				}
				_ => {
					// Current "self range" can no longer be covered
					return false;
				}
			}
		}
		lhs.peek().is_none()
	}

	/// Returns whether `self` is a superset of `other`
	fn superset<O: IntervalIter<E> + ?Sized>(&self, other: &O) -> bool {
		other.subset(self)
	}

	/// Return the set union of two interval iterators.
	fn union<O, R>(&self, other: &O) -> R
	where
		E: Clone,
		O: IntervalIter<E>,
		R: FromIterator<RangeInclusive<E>>,
	{
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while lhs.peek().is_some() || rhs.peek().is_some() {
			match (lhs.peek(), rhs.peek()) {
				(Some(l), None) => {
					ranges.push(l.clone());
					let _ = lhs.next();
				}
				(None, Some(r)) => {
					ranges.push(r.clone());
					let _ = rhs.next();
				}
				(Some(l), Some(r)) => match overlap(l, r) {
					RangeOrdering::Less => {
						ranges.push(l.clone());
						let _ = lhs.next();
					}
					RangeOrdering::Greater => {
						ranges.push(r.clone());
						let _ = rhs.next();
					}
					RangeOrdering::Overlap => {
						let mut ext =
							min(l.start(), r.start()).clone()..=max(l.end(), r.end()).clone();
						let _ = lhs.next();
						let _ = rhs.next();
						loop {
							if let Some(l) = lhs.peek() {
								if overlap(&ext, l) == RangeOrdering::Overlap {
									ext = ext.start().clone()..=max(ext.end(), l.end()).clone();
									let _ = lhs.next();
									continue;
								}
							}
							if let Some(r) = rhs.peek() {
								if overlap(&ext, r) == RangeOrdering::Overlap {
									ext = ext.start().clone()..=max(ext.end(), r.end()).clone();
									let _ = rhs.next();
									continue;
								}
							}
							break;
						}
						ranges.push(ext);
					}
				},
				(None, None) => unreachable!(),
			}
		}
		ranges.into_iter().collect()
	}
}

/// Returns the maximum of two values that implement PartialOrd
fn max<E: PartialOrd>(a: E, b: E) -> E {
	if a > b {
		a
	} else {
		b
	}
}

/// Returns the minimum of two values that implement PartialOrd
fn min<E: PartialOrd>(a: E, b: E) -> E {
	if a < b {
		a
	} else {
		b
	}
}

/// Returns whether two Ranges overlap
fn overlap<E: PartialOrd>(r1: &RangeInclusive<E>, r2: &RangeInclusive<E>) -> RangeOrdering {
	if r1.end() < r2.start() {
		RangeOrdering::Less
	} else if r2.end() < r1.start() {
		RangeOrdering::Greater
	} else {
		RangeOrdering::Overlap
	}
}

impl<E: PartialOrd> RangeList<E> {
	/// Returns `true` if the range list contains no items.
	///
	/// # Examples
	///
	/// ```
	/// # use rangelist::RangeList;
	/// assert!(!RangeList::from_iter([3..=4]).is_empty());
	/// assert!(RangeList::<i64>::default().is_empty());
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
	/// # use rangelist::RangeList;
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
	/// # use rangelist::RangeList;
	/// assert_eq!(RangeList::from_iter([1..=4]).lower_bound(), Some(&1));
	/// assert_eq!(RangeList::from_iter([1..=4, 6..=7, -5..=-3]).lower_bound(), Some(&-5));
	///
	/// assert_eq!(RangeList::<i64>::default().lower_bound(), None);
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
	/// # use std::ops::RangeInclusive;
	/// # use rangelist::RangeList;
	/// assert_eq!(RangeList::from_iter([1..=4]).upper_bound(), Some(&4));
	/// assert_eq!(RangeList::from_iter([1..=4, 6..=7, -5..=-3]).upper_bound(), Some(&7));
	///
	/// assert_eq!(RangeList::<i64>::default().upper_bound(), None);
	/// ```
	pub fn upper_bound(&self) -> Option<&E> {
		self.ranges.last().map(|(_, end)| end)
	}
}

impl<E: PartialOrd + Copy> RangeList<E> {
	/// Returns an Copying iterator for the ranges in the set.
	#[allow(clippy::type_complexity)]
	pub fn iter<'a>(
		&'a self,
	) -> Map<
		<&RangeList<E> as IntoIterator>::IntoIter,
		fn(RangeInclusive<&'a E>) -> RangeInclusive<E>,
	> {
		self.into_iter().map(|r| **r.start()..=**r.end())
	}
}

impl<E: PartialOrd + Debug> Debug for RangeList<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.ranges.is_empty() {
			return write!(f, "RangeList::default()");
		}
		if self.ranges.len() == 1 {
			return write!(
				f,
				"RangeList::from({:?}..={:?})",
				self.ranges[0].0, self.ranges[0].1
			);
		}
		write!(f, "RangeList::from_iter([")?;
		let mut first = true;
		for r in self {
			if !first {
				write!(f, ", ")?;
			}
			write!(f, "{:?}", r)?;
			first = false;
		}
		write!(f, "])")
	}
}

impl<E: PartialOrd + Debug> Display for RangeList<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut first = true;
		for r in &self.ranges {
			if !first {
				write!(f, " union ")?;
			}
			write!(f, "{:?}..{:?}", r.0, r.1)?;
			first = false;
		}
		if first {
			write!(f, "1..0")?;
		}
		Ok(())
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

impl<E, R> FromIterator<R> for RangeList<E>
where
	E: PartialOrd + Clone,
	R: Into<RangeInclusive<E>>,
{
	fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
		let mut non_empty: Vec<(E, E)> = iter
			.into_iter()
			.filter_map(|r| {
				let r = r.into();
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
			// Determine distance between the two ranges if integral
			let dist = match_type!((cur.1.clone(),next.0.clone()),
			{
				(i128,i128) as (ub, lb) => (lb - ub) as usize,
				(i64,i64) as (ub, lb) => (lb - ub) as usize,
				(i32,i32) as (ub, lb) => (lb - ub) as usize,
				(i16,i16) as (ub, lb) => (lb - ub) as usize,
				(i8,i8) as (ub, lb) => (lb - ub) as usize,
				(u128,u128) as (ub, lb) => (lb - ub) as usize,
				(u64,u64) as (ub, lb) => (lb - ub) as usize,
				(u32,u32) as (ub, lb) => (lb - ub) as usize,
				(u16,u16) as (ub, lb) => (lb - ub) as usize,
				(u8,u8) as (ub, lb) => (lb - ub) as usize,
				_ => usize::MAX,
			});

			if cur.1 >= next.0 || dist <= 1 {
				cur.1 = next.1;
			} else {
				ranges.push(cur);
				cur = next;
			}
		}
		ranges.push(cur);
		Self { ranges }
	}
}

impl<E: PartialOrd + Clone> IntoIterator for RangeList<E> {
	type IntoIter = Map<std::vec::IntoIter<(E, E)>, fn((E, E)) -> RangeInclusive<E>>;
	type Item = RangeInclusive<E>;

	fn into_iter(self) -> Self::IntoIter {
		self.ranges
			.into_iter()
			.map(|(start, end)| RangeInclusive::new(start, end))
	}
}

impl<'a, E: PartialOrd> IntoIterator for &'a RangeList<E> {
	type IntoIter = Map<std::slice::Iter<'a, (E, E)>, fn(&'a (E, E)) -> RangeInclusive<&'a E>>;
	type Item = RangeInclusive<&'a E>;

	fn into_iter(self) -> Self::IntoIter {
		self.ranges
			.iter()
			.map(|(start, end)| RangeInclusive::new(start, end))
	}
}

impl<E, I, S> IntervalIter<E> for S
where
	E: PartialOrd,
	I: Into<RangeInclusive<E>>,
	S: IntoIterator<Item = I> + Clone,
{
	type Iter = Map<<S as IntoIterator>::IntoIter, fn(I) -> RangeInclusive<E>>;

	fn intervals(&self) -> Self::Iter {
		self.clone().into_iter().map(Into::into)
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
    RangeList::default()
"#]]
		.assert_debug_eq(&empty);
		assert!(empty.is_empty());

		let single_range = RangeList::from_iter([1..=4]);
		expect![[r#"
    RangeList::from(1..=4)
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

	#[test]
	fn test_display_rangelist() {
		let empty: RangeList<i64> = RangeList::default();
		assert_eq!(empty.to_string(), "1..0");

		let single_range = RangeList::from_iter([1..=4]);
		assert_eq!(single_range.to_string(), "1..4");

		let multi_range = RangeList::from_iter([1..=4, 6..=7, -5..=-3]);
		assert_eq!(multi_range.to_string(), "-5..-3 union 1..4 union 6..7");

		let float_range = RangeList::from_iter([0.1..=3.2, 8.1..=50.0]);
		assert_eq!(float_range.to_string(), "0.1..3.2 union 8.1..50.0");
	}

	#[test]
	fn test_set_union() {
		let empty: RangeList<i64> = RangeList::default();
		let inf: RangeList<i64> = RangeList::from_iter([i64::MIN..=i64::MAX]);
		let res: RangeList<_> = empty.union(&empty);
		assert_eq!(res, empty);
		let res: RangeList<_> = inf.union(&inf);
		assert_eq!(res, inf);
		let res: RangeList<_> = empty.union(&inf);
		assert_eq!(res, inf);
		let res: RangeList<_> = inf.union(&empty);
		assert_eq!(res, inf);

		let x = RangeList::from(1..=5);
		let y = RangeList::from(4..=9);
		let z: RangeList<_> = x.union(&y);
		expect!["1..9"].assert_eq(&z.to_string());

		let y = RangeList::from_iter([1..=2, 4..=4]);
		let z: RangeList<_> = x.union(&y);
		expect!["1..5"].assert_eq(&z.to_string());

		let y = RangeList::from_iter([-5..=-1, 6..=9]);
		let z: RangeList<_> = x.union(&y);
		expect!["-5..-1 union 1..9"].assert_eq(&z.to_string());

		let z: RangeList<_> = y.union(&x);
		expect!["-5..-1 union 1..9"].assert_eq(&z.to_string());

		let x = RangeList::from(1..=9);
		let y = RangeList::from_iter([1..=2, 4..=5, 7..=8]);
		let z: RangeList<_> = x.union(&y);
		expect!["1..9"].assert_eq(&z.to_string());
		let z: RangeList<_> = y.union(&x);
		expect!["1..9"].assert_eq(&z.to_string());

		let x = RangeList::from(1.0..=5.0);
		let y = RangeList::from(4.0..=9.0);
		let z: RangeList<_> = x.union(&y);
		expect!["1.0..9.0"].assert_eq(&z.to_string());
	}

	#[test]
	fn test_set_intersect() {
		let empty = RangeList::default();
		let inf = RangeList::from_iter([i64::MIN..=i64::MAX]);
		let res: RangeList<_> = empty.intersect(&empty);
		assert_eq!(res, empty);
		let res: RangeList<_> = inf.intersect(&inf);
		assert_eq!(res, inf);
		let res: RangeList<_> = empty.intersect(&inf);
		assert_eq!(res, empty);
		let res: RangeList<_> = inf.intersect(&empty);
		assert_eq!(res, empty);

		let x = RangeList::from(1..=5);
		let y = RangeList::from(4..=9);
		let z: RangeList<_> = x.intersect(&y);
		expect!["4..5"].assert_eq(&z.to_string());

		let y = RangeList::from_iter([1..=2, 4..=9]);
		let z: RangeList<_> = x.intersect(&y);
		expect!["1..2 union 4..5"].assert_eq(&z.to_string());
		let z: RangeList<_> = y.intersect(&x);
		expect!["1..2 union 4..5"].assert_eq(&z.to_string());

		let y = RangeList::from_iter([-5..=-1, 1..=3]);
		let z: RangeList<_> = x.intersect(&y);
		expect!["1..3"].assert_eq(&z.to_string());
		let z: RangeList<_> = y.intersect(&x);
		expect!["1..3"].assert_eq(&z.to_string());

		let x = RangeList::from(1.0..=5.0);
		let y = RangeList::from(4.0..=9.0);
		let z: RangeList<_> = x.intersect(&y);
		expect!["4.0..5.0"].assert_eq(&z.to_string());
	}

	#[test]
	fn test_set_subset() {
		let empty = RangeList::default();
		let inf = RangeList::from(i64::MIN..=i64::MAX);
		assert!(empty.subset(&inf));
		assert!(!inf.subset(&empty));

		let x = RangeList::from(1..=5);
		let y = RangeList::from(1..=9);
		assert!(x.subset(&x));
		assert!(x.subset(&y));
		assert!(!y.subset(&x));
		assert!(y.subset(&y));

		let x = RangeList::from_iter([1..=2, 4..=9]);
		assert!(x.subset(&x));
		assert!(x.subset(&y));

		let x = RangeList::from(1.0..=5.0);
		let y = RangeList::from(1.0..=9.0);
		assert!(x.subset(&x));
		assert!(x.subset(&y));
		assert!(!y.subset(&x));
		assert!(y.subset(&y));
	}

	#[test]
	fn test_set_disjoint() {
		let empty = RangeList::default();
		let inf = RangeList::from(i64::MIN..=i64::MAX);

		assert!(empty.disjoint(&empty));
		assert!(empty.disjoint(&inf));
		assert!(inf.disjoint(&empty));
		assert!(!inf.disjoint(&inf));

		let x = RangeList::from_iter([1..=2, 4..=6, 8..=9]);
		assert!(empty.disjoint(&x));
		assert!(x.disjoint(&empty));
		assert!(!x.disjoint(&x));
		assert!(!inf.disjoint(&x));
		assert!(!x.disjoint(&inf));

		let x = RangeList::from_iter([1.0..=2.0, 5.0..=6.0]);
		let y = RangeList::from_iter([3.0..=4.0, 7.0..=8.0]);
		assert!(x.disjoint(&y));
		assert!(y.disjoint(&x));
	}

	#[test]
	fn test_set_card() {
		let empty = RangeList::<i64>::default();
		assert_eq!(empty.card(), 0);

		let x = RangeList::<i8>::from(1..=5);
		assert_eq!(x.card(), 5);

		let y = RangeList::<u32>::from_iter([1..=2, 4..=6, 8..=9]);
		assert_eq!(y.card(), 7);
	}
}
