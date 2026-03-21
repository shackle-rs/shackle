use std::{
	collections::VecDeque,
	fmt::Display,
	iter::{once, Chain, Copied, FilterMap, Map, Once},
	ops::RangeInclusive,
};

use itertools::{Itertools, MapInto, Tuples};
use once_cell::sync::Lazy;
use rangelist::IntervalIterator;
use varlen::array_init::FromIterPrefix;

use crate::{
	value::{
		num::{FloatVal, IntVal},
		value_storage, InitEmpty, ValType, ValueStorage,
	},
	Value,
};

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

impl<'a> IntervalIterator<IntVal> for IntSetView<'a> {
	type IntervalIter = FilterMap<
		Tuples<
			Chain<
				Chain<Once<IntVal>, MapInto<Copied<std::slice::Iter<'a, i64>>, IntVal>>,
				Once<IntVal>,
			>,
			(IntVal, IntVal),
		>,
		fn((IntVal, IntVal)) -> Option<RangeInclusive<IntVal>>,
	>;

	fn intervals(&self) -> Self::IntervalIter {
		once(self.lower_bound())
			.chain(
				self.intervals[self.has_lb as usize..]
					.iter()
					.copied()
					.map_into(),
			)
			.chain(once(self.upper_bound()))
			.tuples()
			.filter_map(|(s, e)| if s <= e { Some(s..=e) } else { None })
	}
}

impl IntSetView<'_> {
	pub fn card(&self) -> IntVal {
		// Extract finite bounds or return infinity
		let (IntVal::Int(lb), IntVal::Int(ub)) = (self.lower_bound(), self.upper_bound()) else {
			return IntVal::InfPos;
		};
		// Check whether the set is empty
		if lb > ub {
			return 0.into();
		}
		// Otherwise, compute the number of elements
		self.intervals
			.iter()
			.tuples()
			.map(|(lb, ub)| ub - lb + 1)
			.sum::<i64>()
			.into()
	}

	/// Returns whether the set constains a finite number of elements.
	pub fn is_finite(&self) -> bool {
		self.lower_bound().is_finite() && self.upper_bound().is_finite()
	}

	/// Returns an iterator over the values contained in the set if the set
	/// contains a finite number of elements
	pub fn values(&self) -> impl Iterator<Item = IntVal> + '_ {
		assert!(
			self.is_finite(),
			"unable to iterate over the values of an infinite set"
		);
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

	pub fn lower_bound(&self) -> IntVal {
		if self.has_lb {
			IntVal::Int(self.intervals[0])
		} else {
			IntVal::InfNeg
		}
	}

	pub fn upper_bound(&self) -> IntVal {
		if self.has_ub {
			IntVal::Int(self.intervals[self.intervals.len() - 1])
		} else {
			IntVal::InfPos
		}
	}
}

impl Display for IntSetView<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match (self.lower_bound(), self.upper_bound()) {
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

impl<'a> IntervalIterator<FloatVal> for FloatSetView<'a> {
	type IntervalIter = Map<
		Tuples<Copied<std::slice::Iter<'a, FloatVal>>, (FloatVal, FloatVal)>,
		fn((FloatVal, FloatVal)) -> RangeInclusive<FloatVal>,
	>;

	fn intervals(&self) -> Self::IntervalIter {
		self.intervals.iter().copied().tuples().map(|(a, b)| a..=b)
	}
}

impl FloatSetView<'_> {
	// Returns the cardinality of the set
	pub fn card(&self) -> IntVal {
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
		card.into()
	}

	pub fn lower_bound(&self) -> FloatVal {
		self.intervals.first().copied().unwrap_or(1.0.into())
	}

	pub fn upper_bound(&self) -> FloatVal {
		self.intervals.last().copied().unwrap_or(0.0.into())
	}
}

impl Display for FloatSetView<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.lower_bound() > self.upper_bound() {
			write!(f, "∅")
		} else if self.intervals.len() == 2
			&& self.lower_bound() == f64::NEG_INFINITY.into()
			&& self.upper_bound() == f64::INFINITY.into()
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

pub static INT_SET_EMPTY: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::IntSet,
		len: 0b11,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: FromIterPrefix([1i64, 0].into_iter()),
		floats: InitEmpty,
		bool_var: InitEmpty,
		int_var: InitEmpty,
		float_var: InitEmpty,
		int_set_var: InitEmpty,
		bytes: InitEmpty,
	})
});
pub static INT_SET_INF: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::IntSet,
		len: 0b00,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: InitEmpty,
		floats: InitEmpty,
		bool_var: InitEmpty,
		int_var: InitEmpty,
		float_var: InitEmpty,
		int_set_var: InitEmpty,
		bytes: InitEmpty,
	})
});

pub static FLOAT_SET_EMPTY: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::FloatSet,
		len: 0,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: InitEmpty,
		floats: InitEmpty,
		bool_var: InitEmpty,
		int_var: InitEmpty,
		float_var: InitEmpty,
		int_set_var: InitEmpty,
		bytes: InitEmpty,
	})
});
pub static FLOAT_SET_INF: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::FloatSet,
		len: 1,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: InitEmpty,
		floats: FromIterPrefix([FloatVal::NEG_INFINITY, FloatVal::INFINITY].into_iter()),
		bool_var: InitEmpty,
		int_var: InitEmpty,
		float_var: InitEmpty,
		int_set_var: InitEmpty,
		bytes: InitEmpty,
	})
});

impl FromIterator<RangeInclusive<IntVal>> for Value {
	fn from_iter<T: IntoIterator<Item = RangeInclusive<IntVal>>>(iter: T) -> Self {
		let mut values: VecDeque<_> = iter
			.into_iter()
			.filter(|r| r.start() <= r.end())
			.coalesce(|r1, r2| match (r1.end(), r2.start()) {
				(IntVal::Int(i), IntVal::Int(j)) if i + 1 >= *j => Ok(*r1.start()..=*r2.end()),
				(a, b) if a >= b => Ok(*r1.start()..=*r2.end()),
				_ => Err((r1, r2)),
			})
			.flat_map(|r| [*r.start(), *r.end()].into_iter())
			.collect();

		// Only create a single empty / infinity set
		if values.is_empty() {
			return INT_SET_EMPTY.clone();
		} else if matches!(
			&values.as_slices(),
			(&[IntVal::InfNeg, IntVal::InfPos], &[])
		) {
			return INT_SET_INF.clone();
		}

		// Number of counted intervals (for storage lb/ub are counted using flags)
		let len = values.len() / 2 - 1;
		assert!(len < 2_usize.pow(31));
		let mut len = (len << 2) as u32;
		if matches!(values.front().unwrap(), IntVal::Int(_)) {
			len |= 0b01;
		} else {
			values.pop_front();
		}
		if matches!(values.back().unwrap(), IntVal::Int(_)) {
			len |= 0b10;
		} else {
			values.pop_back();
		}

		debug_assert_eq!(ValueStorage::int_set_len(len), values.len());

		Self::new_box(value_storage::Init {
			ty: ValType::IntSet,
			len,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: FromIterPrefix(values.into_iter().map(|i| {
				let IntVal::Int(i) = i else { unreachable!() };
				i
			})),
			floats: InitEmpty,
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: InitEmpty,
		})
	}
}

impl FromIterator<RangeInclusive<FloatVal>> for Value {
	fn from_iter<T: IntoIterator<Item = RangeInclusive<FloatVal>>>(iter: T) -> Self {
		let values: Vec<FloatVal> = iter
			.into_iter()
			.filter(|r| r.start() <= r.end())
			.coalesce(|r1, r2| {
				if r1.end() >= r2.start() {
					Ok(*r1.start()..=*r2.end())
				} else {
					Err((r1, r2))
				}
			})
			.flat_map(|r| [*r.start(), *r.end()].into_iter())
			.collect();

		// Use only single empty and (full) infinity set
		if values.is_empty() {
			return FLOAT_SET_EMPTY.clone();
		} else if values.len() == 2
			&& values[0] == FloatVal::NEG_INFINITY
			&& values[1] == FloatVal::INFINITY
		{
			return FLOAT_SET_INF.clone();
		}

		Self::new_box(value_storage::Init {
			ty: ValType::FloatSet,
			len: (values.len() / 2) as u32,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: InitEmpty,
			floats: FromIterPrefix(values.into_iter()),
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: InitEmpty,
		})
	}
}

#[cfg(test)]
mod tests {
	use expect_test::expect;
	use itertools::Itertools;
	use rangelist::IntervalIterator;

	use crate::{
		value::{
			num::{FloatVal, IntVal},
			set::{FLOAT_SET_EMPTY, FLOAT_SET_INF, INT_SET_EMPTY, INT_SET_INF},
			DataView,
		},
		Value,
	};

	#[test]
	fn test_set() {
		let isv_empty = INT_SET_EMPTY.clone();
		let DataView::IntSet(sv) = isv_empty.deref() else {
			unreachable!()
		};
		expect!["∅"].assert_eq(&sv.to_string());

		let isv_inf = INT_SET_INF.clone();
		let DataView::IntSet(sv) = isv_inf.deref() else {
			unreachable!()
		};
		expect!["int"].assert_eq(&sv.to_string());

		let isv_simple = Value::from_iter([IntVal::Int(-3)..=5.into()]);
		let DataView::IntSet(sv) = isv_simple.deref() else {
			unreachable!()
		};
		expect!["-3..5"].assert_eq(&sv.to_string());
		assert!(itertools::equal(sv.values(), (-3..=5).map_into()));

		let isv_open_left = Value::from_iter([IntVal::InfNeg..=5.into()]);
		let DataView::IntSet(sv) = isv_open_left.deref() else {
			unreachable!()
		};
		expect!["..5"].assert_eq(&sv.to_string());

		let isv_open_right = Value::from_iter([0.into()..=IntVal::InfPos]);
		let DataView::IntSet(sv) = isv_open_right.deref() else {
			unreachable!()
		};
		expect!["0.."].assert_eq(&sv.to_string());

		let isv_gaps = Value::from_iter([
			IntVal::Int(-3)..=(-3).into(),
			0.into()..=0.into(),
			3.into()..=5.into(),
		]);
		let DataView::IntSet(sv) = isv_gaps.deref() else {
			unreachable!()
		};
		expect!["-3..-3 ∪ 0..0 ∪ 3..5"].assert_eq(&sv.to_string());
		assert!(itertools::equal(
			sv.values(),
			[-3i64, 0, 3, 4, 5].into_iter().map_into()
		));

		let fsv_empty = FLOAT_SET_EMPTY.clone();
		let DataView::FloatSet(sv) = fsv_empty.deref() else {
			unreachable!()
		};
		expect!["∅"].assert_eq(&sv.to_string());

		let fsv_inf = FLOAT_SET_INF.clone();
		let DataView::FloatSet(sv) = fsv_inf.deref() else {
			unreachable!()
		};
		expect!["float"].assert_eq(&sv.to_string());

		let fsv_simple = Value::from_iter([FloatVal::from(-2.3)..=4.3.into()]);
		let DataView::FloatSet(sv) = fsv_simple.deref() else {
			unreachable!()
		};
		expect!["-2.3..4.3"].assert_eq(&sv.to_string());
	}

	#[test]
	fn test_set_union() {
		let DataView::IntSet(empty) = INT_SET_EMPTY.deref() else {
			unreachable!()
		};
		let DataView::IntSet(inf) = INT_SET_INF.deref() else {
			unreachable!()
		};
		assert!(empty.union::<_, Value>(&empty).is_constant(&INT_SET_EMPTY));
		assert!(inf.union::<_, Value>(&inf).is_constant(&INT_SET_INF));
		assert!(empty.union::<_, Value>(&inf).is_constant(&INT_SET_INF));
		assert!(inf.union::<_, Value>(&empty).is_constant(&INT_SET_INF));

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..9"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(1)..=2.into(), 4.into()..=4.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(-5)..=(-1).into(), 6.into()..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.union(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["-5..-1 ∪ 1..9"].assert_eq(&z.to_string());

		let binding: Value = y.union(&x);
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
		let binding: Value = y.union(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..9"].assert_eq(&z.to_string());
		let binding: Value = x.union(&y);
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
		let binding: Value = x.union(&y);
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
		assert!(empty
			.intersect::<_, Value>(&empty)
			.is_constant(&INT_SET_EMPTY));
		assert!(inf.intersect::<_, Value>(&inf).is_constant(&INT_SET_INF));
		assert!(empty
			.intersect::<_, Value>(&inf)
			.is_constant(&INT_SET_EMPTY));
		assert!(inf
			.intersect::<_, Value>(&empty)
			.is_constant(&INT_SET_EMPTY));

		let binding = Value::from_iter([IntVal::Int(1)..=5.into()]);
		let DataView::IntSet(x) = binding.deref() else {
			unreachable!()
		};
		let binding = Value::from_iter([IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.intersect(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["4..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(1)..=2.into(), IntVal::Int(4)..=9.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.intersect(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..2 ∪ 4..5"].assert_eq(&z.to_string());
		let binding: Value = y.intersect(&x);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..2 ∪ 4..5"].assert_eq(&z.to_string());

		let binding = Value::from_iter([IntVal::Int(-5)..=(-1).into(), IntVal::Int(1)..=3.into()]);
		let DataView::IntSet(y) = binding.deref() else {
			unreachable!()
		};
		let binding: Value = x.intersect(&y);
		let DataView::IntSet(z) = binding.deref() else {
			unreachable!()
		};
		expect!["1..3"].assert_eq(&z.to_string());
		let binding: Value = y.intersect(&x);
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
		let binding: Value = x.intersect(&y);
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
	#[should_panic(expected = "unable to iterate over the values of an infinite set")]
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
