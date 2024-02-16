use std::{fmt::Display, iter::once};

use itertools::Itertools;

use super::{num::IntVal, FLOAT_SET_EMPTY, FLOAT_SET_INF, INT_SET_EMPTY, INT_SET_INF};
use crate::Value;

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
	pub(crate) intervals: &'a [T],
}

impl<'a, T: Clone> SetView<'a, T> {
	/// Gives the lower bound of the set, or None if there is no lower bound
	pub fn lb(&self) -> Option<T> {
		if self.has_lb {
			Some(self.intervals[0].clone())
		} else {
			None
		}
	}
	/// Gives the upper bound of the set, or None if there is no lower bound
	pub fn ub(&self) -> Option<T> {
		if self.has_ub {
			Some(self.intervals[if self.has_lb { 1 } else { 0 }].clone())
		} else {
			None
		}
	}
	/// Returns an iterator over the gaps in the set between the lower and upper
	/// bound.
	///
	/// Note that the gaps are open intervals. The values in the tuples are
	/// contained in the set.
	pub fn gaps(&self) -> impl Iterator<Item = (T, T)> + 'a {
		let offset = self.has_lb as usize + self.has_ub as usize;
		self.intervals[offset..].iter().cloned().tuples()
	}
	/// Returns an iterator over the intervals of values contained in the set.
	///
	/// The value `None` is used to signal that the interval is unbounded on that
	/// side. It can be assumed that only the first left value and the
	/// last right value can be None.
	pub fn intervals(&self) -> impl Iterator<Item = (Option<T>, Option<T>)> + 'a {
		let offset = self.has_lb as usize + self.has_ub as usize;
		once(if self.has_lb {
			Some(self.intervals[0].clone())
		} else {
			None
		})
		.chain(self.intervals[offset..].iter().cloned().map_into())
		.chain(once(if self.has_ub {
			Some(self.intervals[self.has_lb as usize].clone())
		} else {
			None
		}))
		.tuples()
	}
}

impl<'a, T: Clone + PartialOrd + SetInit> SetView<'a, T> {
	fn max<const NONE_HIGH: bool>(x: &Option<T>, y: &Option<T>) -> Option<T> {
		match (x, y) {
			(Some(x), Some(y)) => Some(if x >= y { x.clone() } else { y.clone() }),
			(Some(x), None) | (None, Some(x)) => {
				if NONE_HIGH {
					None
				} else {
					Some(x.clone())
				}
			}
			(None, None) => None,
		}
	}
	fn min<const NONE_HIGH: bool>(x: &Option<T>, y: &Option<T>) -> Option<T> {
		match (x, y) {
			(Some(x), Some(y)) => Some(if x <= y { x.clone() } else { y.clone() }),
			(Some(x), None) | (None, Some(x)) => {
				if !NONE_HIGH {
					None
				} else {
					Some(x.clone())
				}
			}
			(None, None) => None,
		}
	}
	fn overlaps(max: &Option<T>, min: &Option<T>) -> bool {
		let Some(max) = max else { return true };
		let Some(min) = min else { return true };
		max >= min
	}

	pub fn intersect(&self, other: &Self) -> Value {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while let (Some((l1, l2)), Some((r1, r2))) = (lhs.peek(), rhs.peek()) {
			if !Self::overlaps(l2, r1) {
				lhs.next();
			} else if !Self::overlaps(r2, l1) {
				rhs.next();
			} else {
				ranges.push(Self::max::<false>(l1, r1));
				ranges.push(Self::min::<true>(l2, r2));
				if r2.is_none() || (l2.is_some() && l2.as_ref().unwrap() <= r2.as_ref().unwrap()) {
					lhs.next();
				}
				RangeOrdering::Greater => {
					rhs.next();
				}
				RangeOrdering::Overlap => {
					ranges.push(max(l.start(), r.start()).clone()..=min(l.end(), r.end()).clone());
					if l.end() <= r.end() {
						lhs.next();
					} else {
						rhs.next();
					}
				}
			}
		}
		T::init_from_ranges(ranges)
	}

	pub fn union(&self, other: &Self) -> Value {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while lhs.peek().is_some() || rhs.peek().is_some() {
			match (lhs.peek(), rhs.peek()) {
				(Some(r), None) => {
					ranges.push(r.0.clone());
					ranges.push(r.1.clone());
					lhs.next();
				}
				(None, Some(r)) => {
					ranges.push(r.0.clone());
					ranges.push(r.1.clone());
					rhs.next();
				}
				(Some((lmin, lmax)), Some((rmin, _))) if !Self::overlaps(lmax, rmin) => {
					ranges.push(lmin.clone());
					ranges.push(lmax.clone());
					lhs.next();
				}
				(Some((lmin, _)), Some((rmin, rmax))) if !Self::overlaps(rmax, lmin) => {
					ranges.push(rmin.clone());
					ranges.push(rmax.clone());
					rhs.next();
				}
				(Some(l), Some(r)) => {
					let start = Self::min::<false>(&l.0, &r.0);
					let mut end = Self::max::<true>(&l.1, &r.1);
					lhs.next();
					rhs.next();
					loop {
						if let Some((lmin, lmax)) = lhs.peek() {
							if Self::overlaps(&end, lmin) {
								end = Self::max::<true>(&end, lmax);
								lhs.next();
								continue;
							}
							if let Some(r) = rhs.peek() {
						if let Some((rmin, rmax)) = rhs.peek() {
							if Self::overlaps(&end, rmin) {
								end = Self::max::<true>(&end, rmax);
									continue;
								}
							}
							break;
						}
						ranges.push(ext);
					}
					ranges.push(start);
					ranges.push(end);
				}
				(None, None) => unreachable!(),
			}
		}

		T::init_from_ranges(ranges)
	}

	pub fn contains(&self, val: T) -> bool {
		// Check whether `val` falls within the set bounds
		if let Some(lb) = self.lb() {
			if val < lb {
				return false;
			}
		}
		if let Some(ub) = self.ub() {
			if val > ub {
				return false;
			}
		}
		// Check whether `val` falls in any of the gaps (binary search)
		let offset = self.has_lb as usize + self.has_ub as usize;
		let mut size = (self.intervals.len() - offset) / 2;
		let mut left = 0;
		let mut right = size;
		while left < right {
			let mid = left + size / 2;
			let range_low = &self.intervals[offset + (mid * 2)];
			let range_high = &self.intervals[offset + (mid * 2) + 1];

			if &val >= range_high {
				left = mid + 1;
			} else if &val <= range_low {
				right = mid;
			} else {
				return false;
			}
			size = right - left;
		}
		// val does not fall in a gap
		true
	}

	/// Returns whether `self` is a subset of `other`
	pub fn subset(&self, other: &Self) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some((lmin, lmax)), Some((rmin, rmax))) = (lhs.peek(), rhs.peek()) {
			// Current "other range" has smaller elements than current "self range"
			if !Self::overlaps(rmax, lmin) {
				// Move to next "other range"
				rhs.next();
			} else if Self::min::<false>(lmin, rmin) == *rmin
				&& Self::max::<true>(lmax, rmax) == *rmax
			// Current "self range" is included in the current other range
			{
				// Move to next "self range" that needs to be covered
				lhs.next();
			} else {
				// Current "self range" can no longer be covered
				return false;
			}
		}
		lhs.peek().is_none()
	}

	/// Returns whether `self` and `other` are disjoint sets
	pub fn disjoint(&self, other: &Self) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some((lmin, lmax)), Some((rmin, rmax))) = (lhs.peek(), rhs.peek()) {
			if !Self::overlaps(rmax, lmin) {
				// Move to next "other range"
				rhs.next();
			} else if !Self::overlaps(lmax, rmin) {
				lhs.next();
			} else {
				// At least one of the ranges overlap
				return false;
			}
		}
		true
	}
}

impl<'a, T: Display + PartialOrd + Clone> Display for SetView<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.lb(), self.ub()) {
			(Some(lb), Some(ub)) if lb > ub => write!(f, "∅"),
			(None, None) if self.intervals.is_empty() => write!(f, "-∞..+∞"),
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
	/// Returns an iterator over the values contained in the set if the set
	/// contains a finite number of elements
	pub fn values(&self) -> Option<impl Iterator<Item = i64> + 'a> {
		if let (Some(lb), Some(ub)) = (self.lb(), self.ub()) {
			Some(
				once(lb)
					.chain(self.intervals[2..].iter().copied())
					.chain(once(ub))
					.tuples()
					.flat_map(|(lb, ub)| lb..=ub),
			)
		} else {
			None
		}
	}

	/// Returns the cardinality of the set
	pub fn card(&self) -> IntVal {
		if let (Some(lb), Some(ub)) = (self.lb(), self.ub()) {
			if lb > ub {
				0.into()
			} else {
				once(lb)
					.chain(self.intervals[2..].iter().copied())
					.chain(once(ub))
					.tuples()
					.map(|(lb, ub)| ub - lb + 1)
					.sum::<i64>()
					.into()
			}
		} else {
			IntVal::InfPos
		}
	}
}

pub trait SetInit: Sized {
	fn init_set_val<I: ExactSizeIterator<Item = (Self, Self)>, S: IntoIterator<IntoIter = I>>(
		lb: Option<Self>,
		ub: Option<Self>,
		gaps: S,
	) -> Value;

	fn init_from_ranges<
		I: ExactSizeIterator<Item = Option<Self>> + DoubleEndedIterator,
		S: IntoIterator<IntoIter = I>,
	>(
		ranges: S,
	) -> Value;
}
impl SetInit for i64 {
	fn init_set_val<I: ExactSizeIterator<Item = (Self, Self)>, S: IntoIterator<IntoIter = I>>(
		lb: Option<Self>,
		ub: Option<Self>,
		gaps: S,
	) -> Value {
		match (lb, ub) {
			(Some(lb), Some(ub)) if lb >= ub => INT_SET_EMPTY.clone(),
			(None, None) => INT_SET_INF.clone(),
			_ => Value::new_int_set(lb, ub, gaps),
		}
	}

	fn init_from_ranges<
		I: ExactSizeIterator<Item = Option<Self>> + DoubleEndedIterator,
		S: IntoIterator<IntoIter = I>,
	>(
		ranges: S,
	) -> Value {
		let mut it = ranges.into_iter();
		assert!(it.len() % 2 == 0);
		if it.len() == 0 {
			return INT_SET_EMPTY.clone();
		}
		let lb = it.next().unwrap();
		let ub = it.next_back().unwrap();
		let gaps = it.map(Option::unwrap).tuples().collect_vec();
		Self::init_set_val(lb, ub, gaps)
	}
}
impl SetInit for f64 {
	fn init_set_val<I: ExactSizeIterator<Item = (Self, Self)>, S: IntoIterator<IntoIter = I>>(
		lb: Option<Self>,
		ub: Option<Self>,
		gaps: S,
	) -> Value {
		match (lb, ub) {
			(Some(lb), Some(ub)) if lb >= ub => FLOAT_SET_EMPTY.clone(),
			(None, None) => FLOAT_SET_INF.clone(),
			_ => Value::new_float_set(lb, ub, gaps),
		}
	}

	fn init_from_ranges<
		I: ExactSizeIterator<Item = Option<Self>> + DoubleEndedIterator,
		S: IntoIterator<IntoIter = I>,
	>(
		ranges: S,
	) -> Value {
		let mut it = ranges.into_iter();
		assert!(it.len() % 2 == 0);
		if it.len() == 0 {
			return FLOAT_SET_EMPTY.clone();
		}
		let lb = it.next().unwrap();
		let ub = it.next_back().unwrap();
		let gaps = it.map(Option::unwrap).tuples().collect_vec();
		Self::init_set_val(lb, ub, gaps)
	}
}

#[cfg(test)]
mod tests {
	use itertools::Itertools;

	use crate::{
		value::{num::IntVal, DataView, INT_SET_EMPTY, INT_SET_INF},
		Value,
	};

	#[test]
	fn test_set_union() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(empty.union(&empty).is_constant(&INT_SET_EMPTY));
		assert!(inf.union(&inf).is_constant(&INT_SET_INF));
		assert!(empty.union(&inf).is_constant(&INT_SET_INF));

		let binding = Value::new_int_set(Some(1), Some(5), []);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_int_set(Some(4), Some(9), []);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 1..=9);
		assert_eq!(z.gaps().count(), 0);

		let binding = Value::new_int_set(Some(1), Some(9), [(2, 4)]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 1..=9);
		assert_eq!(z.gaps().count(), 0);

		let binding = Value::new_int_set(Some(-5), Some(9), [(-1, 6)]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), (-5)..=9);
		assert_eq!(z.gaps().collect_vec(), vec![(-1, 1), (5, 6)]);

		let binding = Value::new_float_set(Some(1.0), Some(5.0), []);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_float_set(Some(4.0), Some(9.0), []);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union(&y);
		let DataView::FloatSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 1.0..=9.0);
		assert_eq!(z.gaps().count(), 0);
	}

	#[test]
	fn test_set_intersect() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(empty.intersect(&empty).is_constant(&INT_SET_EMPTY));
		assert!(inf.intersect(&inf).is_constant(&INT_SET_INF));
		assert!(empty.intersect(&inf).is_constant(&INT_SET_EMPTY));

		let binding = Value::new_int_set(Some(1), Some(5), []);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_int_set(Some(4), Some(9), []);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 4..=5);
		assert_eq!(z.gaps().count(), 0);

		let binding = Value::new_int_set(Some(1), Some(9), [(2, 4)]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 1..=5);
		assert_eq!(z.gaps().collect_vec(), vec![(2, 4)]);

		let binding = Value::new_float_set(Some(1.0), Some(5.0), []);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_float_set(Some(4.0), Some(9.0), []);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect(&y);
		let DataView::FloatSet(z) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(z.lb().unwrap()..=z.ub().unwrap(), 4.0..=5.0);
		assert_eq!(z.gaps().count(), 0);
	}

	#[test]
	fn test_set_contains() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(!empty.contains(0));
		assert!(inf.contains(0));

		let binding = Value::new_int_set(Some(1), Some(9), [(2, 4), (6, 8)]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(y.contains(1));
		assert!(y.contains(9));
		assert!(y.contains(4));
		assert!(y.contains(5));
		assert!(y.contains(6));
		assert!(!y.contains(3));
		assert!(!y.contains(7));

		let binding = Value::new_float_set(Some(1.0), Some(9.0), []);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};

		assert!(!y.contains(0.999));
		assert!(y.contains(1.0));
		assert!(y.contains(1.1));
		assert!(y.contains(9.0));
		assert!(!y.contains(9.001));
	}

	#[test]
	fn test_set_subset() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(empty.subset(&inf));
		assert!(!inf.subset(&empty));

		let binding = Value::new_int_set(Some(1), Some(5), []);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_int_set(Some(1), Some(9), []);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(x.subset(&x));
		assert!(x.subset(&y));
		assert!(!y.subset(&x));
		assert!(y.subset(&y));

		let binding = Value::new_int_set(Some(1), Some(9), [(2, 4)]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		assert!(x.subset(&x));
		assert!(x.subset(&y));

		let binding = Value::new_float_set(Some(1.0), Some(5.0), []);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::new_float_set(Some(1.0), Some(9.0), []);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(x.subset(&x));
		assert!(x.subset(&y));
		assert!(!y.subset(&x));
		assert!(y.subset(&y));
	}

	#[test]
	fn test_set_disjoint() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};

		assert!(empty.disjoint(&empty));
		assert!(empty.disjoint(&inf));
		assert!(inf.disjoint(&empty));
		assert!(!inf.disjoint(&inf));

		let binding = Value::from_iter([
			IntVal::Int(1)..=2.into(),
			4.into()..=6.into(),
			8.into()..=9.into(),
		]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		assert!(empty.disjoint(&x));
		assert!(x.disjoint(&empty));
		assert!(!x.disjoint(&x));
		assert!(!inf.disjoint(&x));
		assert!(!x.disjoint(&inf));

		let binding = Value::from_iter([FloatVal::from(1.0)..=2.0.into(), 5.0.into()..=6.0.into()]);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([FloatVal::from(3.0)..=4.0.into(), 7.0.into()..=8.0.into()]);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(x.disjoint(&y));
		assert!(y.disjoint(&x));
	}

	#[test]
	#[should_panic(expected = "unable to iterate over an infinite set")]
	fn test_int_set_values() {
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		for _ in inf.values() {}
	}

	#[test]
	fn test_set_card() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert_eq!(empty.card(), 0.into());
		assert_eq!(inf.card(), IntVal::InfPos);

		let binding = Value::new_int_set(Some(1), Some(5), []);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(x.card(), 5.into());

		let binding = Value::new_int_set(Some(1), Some(9), [(2, 4), (6, 8)]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(y.card(), 7.into());

		let DataView::FloatSet(inf) = FLOAT_SET_INF.deref() else {
			unreachable!();
		};
		assert_eq!(inf.card(), IntVal::InfPos);
		let DataView::FloatSet(inf) = FLOAT_SET_EMPTY.deref() else {
			unreachable!();
		};
		assert_eq!(inf.card(), IntVal::Int(0));

		let binding = Value::from_iter([
			FloatVal::from(0.0)..=FloatVal::from(0.0),
			FloatVal::from(1.0)..=FloatVal::from(1.0),
		]);
		let DataView::FloatSet(fs) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(fs.card(), IntVal::Int(2));
	}
}
