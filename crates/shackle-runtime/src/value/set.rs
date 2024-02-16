use std::{
	cmp::{max, min},
	fmt::Display,
	iter::{once, Chain, Copied, FilterMap, Map, Once},
	ops::RangeInclusive,
};

use itertools::{Itertools, MapInto, Tuples};

use super::num::FloatVal;
use crate::value::num::IntVal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RangeOrdering {
	/// A compared range is strictly less than another.
	Less = -1,
	/// A compared range overlaps with another.
	Overlap = 0,
	/// A compared range is strictly greater than another.
	Greater = 1,
}

pub trait SetView<T: Clone + Ord> {
	type Iter: Iterator<Item = RangeInclusive<T>>;
	/// Gives the lower bound of the set, or None if there is no lower bound
	fn lb(&self) -> T;
	/// Gives the upper bound of the set, or None if there is no lower bound
	fn ub(&self) -> T;
	/// Returns an iterator over the intervals of values contained in the set.
	///
	/// The value `None` is used to signal that the interval is unbounded on that
	/// side. It can be assumed that only the first left value and the
	/// last right value can be None.
	fn intervals(&self) -> Self::Iter;
	/// Returns the cardinality of the set
	fn card(&self) -> IntVal;
	/// Returns whether two Ranges overlap
	fn overlap(r1: &RangeInclusive<T>, r2: &RangeInclusive<T>) -> RangeOrdering {
		if r1.end() < r2.start() {
			RangeOrdering::Less
		} else if r2.end() < r1.start() {
			RangeOrdering::Greater
		} else {
			RangeOrdering::Overlap
		}
	}

	fn intersect<Storage: FromIterator<RangeInclusive<T>>>(&self, other: &Self) -> Storage {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match Self::overlap(l, r) {
				RangeOrdering::Less => {
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
		ranges.into_iter().collect()
	}

	fn union<Storage: FromIterator<RangeInclusive<T>>>(&self, other: &Self) -> Storage {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		let mut ranges = Vec::new();
		while lhs.peek().is_some() || rhs.peek().is_some() {
			match (lhs.peek(), rhs.peek()) {
				(Some(l), None) => {
					ranges.push(l.clone());
					lhs.next();
				}
				(None, Some(r)) => {
					ranges.push(r.clone());
					rhs.next();
				}
				(Some(l), Some(r)) => match Self::overlap(l, r) {
					RangeOrdering::Less => {
						ranges.push(l.clone());
						lhs.next();
					}
					RangeOrdering::Greater => {
						ranges.push(r.clone());
						rhs.next();
					}
					RangeOrdering::Overlap => {
						let mut ext =
							min(l.start(), r.start()).clone()..=max(l.end(), r.end()).clone();
						lhs.next();
						rhs.next();
						loop {
							if let Some(l) = lhs.peek() {
								if Self::overlap(&ext, l) == RangeOrdering::Overlap {
									ext = ext.start().clone()..=max(ext.end(), l.end()).clone();
									lhs.next();
									continue;
								}
							}
							if let Some(r) = rhs.peek() {
								if Self::overlap(&ext, r) == RangeOrdering::Overlap {
									ext = ext.start().clone()..=max(ext.end(), r.end()).clone();
									rhs.next();
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

	fn contains(&self, val: &T) -> bool {
		// Check whether `val` falls within the set bounds
		if !(self.lb()..=self.ub()).contains(val) {
			return false;
		}
		self.intervals().any(|r| r.contains(val))
	}

	/// Returns whether `self` is a subset of `other`
	fn subset(&self, other: &Self) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match Self::overlap(l, r) {
				RangeOrdering::Overlap if r.start() <= l.start() && l.end() <= r.end() => {
					// Current "self range" is included in the current other range
					// Move to next "self range" that needs to be covered
					lhs.next();
				}
				RangeOrdering::Greater => {
					// Move to next "other range"
					rhs.next();
				}
				_ => {
					// Current "self range" can no longer be covered
					return false;
				}
			}
		}
		lhs.peek().is_none()
	}

	/// Returns whether `self` and `other` are disjoint sets
	fn disjoint(&self, other: &Self) -> bool {
		let mut lhs = self.intervals().peekable();
		let mut rhs = other.intervals().peekable();
		while let (Some(l), Some(r)) = (lhs.peek(), rhs.peek()) {
			match Self::overlap(l, r) {
				RangeOrdering::Less => {
					// Move to next "self range"
					lhs.next();
				}
				RangeOrdering::Overlap => return false,
				RangeOrdering::Greater => {
					// Move to next "other range"
					rhs.next();
				}
			}
		}
		true
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntSetView<'a> {
	/// Whether the set has a defined lower bound
	pub(crate) has_lb: bool,
	/// Whether the set has a defined upper bound
	pub(crate) has_ub: bool,
	/// Raw storage of bounds plus gaps, assumed to be in the order [lb, ub,
	/// gaps[0].0, gaps[0].1,gaps[1].0,gaps[1].1, etc.]
	///
	/// Note that when has_lb or has_ub is false, these elements are not present
	/// in ranges
	pub(crate) intervals: &'a [i64],
}

impl<'a> SetView<IntVal> for IntSetView<'a> {
	type Iter = FilterMap<
		Tuples<
			Chain<
				Chain<Once<IntVal>, MapInto<Copied<std::slice::Iter<'a, i64>>, IntVal>>,
				Once<IntVal>,
			>,
			(IntVal, IntVal),
		>,
		fn((IntVal, IntVal)) -> Option<RangeInclusive<IntVal>>,
	>;

	fn lb(&self) -> IntVal {
		if self.has_lb {
			IntVal::Int(self.intervals[0])
		} else {
			IntVal::InfNeg
		}
	}

	fn ub(&self) -> IntVal {
		if self.has_ub {
			IntVal::Int(self.intervals[self.intervals.len() - 1])
		} else {
			IntVal::InfPos
		}
	}

	fn card(&self) -> IntVal {
		if let (IntVal::Int(lb), IntVal::Int(ub)) = (self.lb(), self.ub()) {
			if lb > ub {
				0.into()
			} else {
				self.intervals
					.iter()
					.tuples()
					.map(|(lb, ub)| ub - lb + 1)
					.sum::<i64>()
					.into()
			}
		} else {
			IntVal::InfPos
		}
	}

	fn intervals(&self) -> Self::Iter {
		once(self.lb())
			.chain(
				self.intervals[self.has_lb as usize..]
					.iter()
					.copied()
					.map_into(),
			)
			.chain(once(self.ub()))
			.tuples()
			.filter_map(|(s, e)| if s <= e { Some(s..=e) } else { None })
	}
}

impl<'a> IntSetView<'a> {
	/// Returns an iterator over the values contained in the set if the set
	/// contains a finite number of elements
	pub fn values(&self) -> impl Iterator<Item = IntVal> + '_ {
		if !(self.lb().is_finite() && self.ub().is_finite()) {
			panic!("unable to iterate over an infinite set")
		}
		self.intervals()
			.flat_map(|r| {
				let IntVal::Int(a) = *r.start() else {
					unreachable!()
				};
				let IntVal::Int(b) = *r.end() else {
					unreachable!()
				};
				a..=b
			})
			.map(IntVal::Int)
	}
}

impl<'a> Display for IntSetView<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.lb(), self.ub()) {
			(IntVal::Int(lb), IntVal::Int(ub)) if lb > ub => write!(f, "∅"),
			(IntVal::InfNeg, IntVal::InfPos) if self.intervals.is_empty() => write!(f, "int"),
			_ => write!(
				f,
				"{}",
				self.intervals().format_with(" ∪ ", |r, f| f(&format_args!(
					"{}..{}",
					if r.start().is_finite() {
						r.start().to_string()
					} else {
						"".to_string()
					},
					if r.end().is_finite() {
						r.end().to_string()
					} else {
						"".to_string()
					},
				))),
			),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatSetView<'a> {
	pub(crate) intervals: &'a [FloatVal],
}

impl<'a> SetView<FloatVal> for FloatSetView<'a> {
	type Iter = Map<
		Tuples<Copied<std::slice::Iter<'a, FloatVal>>, (FloatVal, FloatVal)>,
		fn((FloatVal, FloatVal)) -> RangeInclusive<FloatVal>,
	>;

	fn lb(&self) -> FloatVal {
		if self.intervals.is_empty() {
			1.0.into()
		} else {
			self.intervals[0]
		}
	}

	fn ub(&self) -> FloatVal {
		if self.intervals.is_empty() {
			0.0.into()
		} else {
			self.intervals[self.intervals.len() - 1]
		}
	}

	fn intervals(&self) -> Self::Iter {
		self.intervals.iter().copied().tuples().map(|(a, b)| a..=b)
	}

	fn card(&self) -> IntVal {
		let mut card = 0;
		for r in self.intervals() {
			if !r.start().is_finite() || !r.end().is_finite() || r.start() < r.end() {
				return IntVal::InfPos;
			}
			// Do not count if b < a
			if r.start() == r.end() {
				card += 1;
			}
		}
		IntVal::Int(card)
	}
}
impl<'a> Display for FloatSetView<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.lb() > self.ub() {
			write!(f, "∅")
		} else if self.intervals.len() == 2
			&& self.lb() == f64::NEG_INFINITY.into()
			&& self.ub() == f64::INFINITY.into()
		{
			write!(f, "float")
		} else {
			write!(
				f,
				"{}",
				self.intervals().format_with(" ∪ ", |r, f| f(&format_args!(
					"{}..{}",
					r.start(),
					r.end()
				))),
			)
		}
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;

	use crate::{
		value::{
			num::{FloatVal, IntVal},
			set::SetView,
			DataView, FLOAT_SET_EMPTY, FLOAT_SET_INF, INT_SET_EMPTY, INT_SET_INF,
		},
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
		assert!(empty.union::<Value>(&empty).is_constant(&INT_SET_EMPTY));
		assert!(inf.union::<Value>(&inf).is_constant(&INT_SET_INF));
		assert!(empty.union::<Value>(&inf).is_constant(&INT_SET_INF));
		assert!(inf.union::<Value>(&empty).is_constant(&INT_SET_INF));

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..9"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(1)..=2.into(), 4.into()..=4.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(-5)..=(-1).into(), 6.into()..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["-5..-1 ∪ 1..9"].assert_eq(&z.to_string());

		let binding = y.union::<Value>(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["-5..-1 ∪ 1..9"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(1)..=9.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([
			IntVal::Int(1)..=2.into(),
			4.into()..=5.into(),
			7.into()..=8.into(),
		]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = y.union::<Value>(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..9"].assert_eq(&z.to_string());
		let binding = x.union::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..9"].assert_eq(&z.to_string());

		let binding = Value::from_iter([FloatVal::from(1.0)..=(5.0).into()]);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([FloatVal::from(4.0)..=(9.0).into()]);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.union::<Value>(&y);
		let DataView::FloatSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1.0..9.0"].assert_eq(&z.to_string());
	}

	#[test]
	fn test_set_intersect() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(empty.intersect::<Value>(&empty).is_constant(&INT_SET_EMPTY));
		assert!(inf.intersect::<Value>(&inf).is_constant(&INT_SET_INF));
		assert!(empty.intersect::<Value>(&inf).is_constant(&INT_SET_EMPTY));
		assert!(inf.intersect::<Value>(&empty).is_constant(&INT_SET_EMPTY));

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["4..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(1)..=2.into(), IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..2 ∪ 4..5"].assert_eq(&z.to_string());
		let binding = y.intersect::<Value>(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..2 ∪ 4..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(-5)..=(-1).into(), IntVal::Int(1)..=3.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect::<Value>(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..3"].assert_eq(&z.to_string());
		let binding = y.intersect::<Value>(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..3"].assert_eq(&z.to_string());

		let binding = Value::from_iter([FloatVal::from(1.0)..=(5.0).into()]);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([FloatVal::from(4.0)..=(9.0).into()]);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding = x.intersect::<Value>(&y);
		let DataView::FloatSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["4.0..5.0"].assert_eq(&z.to_string());
	}

	#[test]
	fn test_set_contains() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(!empty.contains(&0.into()));
		assert!(inf.contains(&0.into()));

		let binding = Value::from_iter([
			IntVal::Int(1)..=2.into(),
			4.into()..=6.into(),
			8.into()..=9.into(),
		]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(y.contains(&1.into()));
		assert!(y.contains(&9.into()));
		assert!(y.contains(&4.into()));
		assert!(y.contains(&5.into()));
		assert!(y.contains(&6.into()));
		assert!(!y.contains(&3.into()));
		assert!(!y.contains(&7.into()));

		let binding = Value::from_iter([FloatVal::from(1.0)..=(9.0).into()]);
		let DataView::FloatSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(!y.contains(&0.999.into()));
		assert!(y.contains(&1.0.into()));
		assert!(y.contains(&1.1.into()));
		assert!(y.contains(&9.0.into()));
		assert!(!y.contains(&9.001.into()));
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

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([IntVal::Int(1)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		assert!(x.subset(&x));
		assert!(x.subset(&y));
		assert!(!y.subset(&x));
		assert!(y.subset(&y));

		let binding = Value::from_iter([IntVal::Int(1)..=2.into(), 4.into()..=9.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		assert!(x.subset(&x));
		assert!(x.subset(&y));

		let binding = Value::from_iter([FloatVal::from(1.0)..=(5.0).into()]);
		let DataView::FloatSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([FloatVal::from(1.0)..=(9.0).into()]);
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

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		assert_eq!(x.card(), 5.into());

		let binding = Value::from_iter([
			IntVal::Int(1)..=2.into(),
			4.into()..=6.into(),
			8.into()..=9.into(),
		]);
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
