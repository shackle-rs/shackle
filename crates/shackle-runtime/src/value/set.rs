use std::{fmt::Display, iter::once};

use itertools::Itertools;

use super::{FLOAT_SET_EMPTY, INT_SET_EMPTY};
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
	pub fn ranges(&self) -> impl Iterator<Item = (Option<T>, Option<T>)> + 'a {
		let offset = self.has_lb as usize + self.has_ub as usize;
		once(if self.has_lb {
			Some(self.ranges[0].clone())
		} else {
			None
		})
		.chain(self.ranges[offset..].iter().cloned().map_into())
		.chain(once(if self.has_ub {
			Some(self.ranges[self.has_lb as usize].clone())
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
		let mut lhs = self.ranges().peekable();
		let mut rhs = other.ranges().peekable();
		let mut ranges = Vec::new();
		while lhs.peek().is_some() && rhs.peek().is_some() {
			let (l1, l2) = lhs.peek().unwrap();
			let (r1, r2) = rhs.peek().unwrap();
			if l2.is_some() && r1.is_some() && l2 < r1 {
				lhs.next();
			} else if r2.is_some() && l1.is_some() && r2 < l1 {
				rhs.next();
			} else {
				ranges.push(Self::max::<false>(l1, r2));
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
		let mut lhs = self.ranges().peekable();
		let mut rhs = other.ranges().peekable();
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
		if let (Some(lb), Some(ub)) = (lb, ub) {
			if lb >= ub {
				return INT_SET_EMPTY.clone();
			}
		}
		Value::new_int_set(lb, ub, gaps)
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
		if let (Some(lb), Some(ub)) = (lb, ub) {
			if lb >= ub {
				return FLOAT_SET_EMPTY.clone();
			}
		}
		Value::new_float_set(lb, ub, gaps)
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
