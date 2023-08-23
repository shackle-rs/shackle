//! Values types used for input and output for Programs

use std::{
	cmp::max,
	fmt::{self, Display},
	iter::FusedIterator,
	ops::RangeInclusive,
	rc::Rc,
	sync::Arc,
};

use itertools::Itertools;

/// Value types that can be part of a Solution
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	/// Absence of an optional value
	Absent,
	/// Infinity (+∞ or -∞)
	Infinity(Polarity),
	/// Boolean
	Boolean(bool),
	/// Signed integer
	Integer(i64),
	/// Floating point
	Float(f64),
	/// String
	String(Rc<str>),
	/// Identifier of a value of an enumerated type
	Enum(EnumValue),
	/// Annotation
	Ann(Rc<str>, Vec<Value>),
	/// An array of values
	/// All values are of the same type
	Array(Array),
	/// A set of values
	/// All values are of the same type and only occur once
	Set(Set),
	/// A tuple of values
	Tuple(Vec<Value>),
	/// A record of values
	Record(Record),
}

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Self::Boolean(value)
	}
}
impl From<i64> for Value {
	fn from(value: i64) -> Self {
		Self::Integer(value)
	}
}
impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Self::Float(value)
	}
}
impl From<Array> for Value {
	fn from(value: Array) -> Self {
		Self::Array(value)
	}
}
impl From<Set> for Value {
	fn from(value: Set) -> Self {
		Self::Set(value)
	}
}
impl From<Record> for Value {
	fn from(value: Record) -> Self {
		Self::Record(value)
	}
}

/// Whether an value is negative or positive
///
/// For example, used for the constant infinity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Polarity {
	/// Positive
	Pos,
	/// Negative
	Neg,
}

impl Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Absent => write!(f, "<>"),
			Value::Infinity(p) => {
				if p == &Polarity::Neg {
					write!(f, "-")?;
				};
				write!(f, "∞")
			}
			Value::Boolean(v) => write!(f, "{v}"),
			Value::Integer(v) => write!(f, "{v}"),
			Value::Float(v) => write!(f, "{v}"),
			Value::String(v) => write!(f, "{:?}", v),
			Value::Enum(v) => write!(f, "{v}"),
			Value::Ann(ann, args) => {
				if args.is_empty() {
					write!(f, "{ann}")
				} else {
					write!(f, "{ann}({})", args.iter().format(", "))
				}
			}
			Value::Array(arr) => {
				write!(f, "{arr}")
			}
			Value::Set(v) => {
				write!(f, "{v}")
			}
			Value::Tuple(v) => {
				write!(f, "({})", v.iter().format(", "))
			}
			Value::Record(rec) => {
				write!(f, "{rec}")
			}
		}
	}
}

/// Representation of an (multidimensional) indexed array
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array {
	pub(crate) indices: Box<[Index]>,
	pub(crate) members: Box<[Value]>,
}

impl Array {
	/// Create a new array that contains the values in `elements` indexes by the given index sets
	pub fn new(indexes: Vec<Index>, elements: Vec<Value>) -> Self {
		assert_eq!(
			indexes.iter().map(|i| i.len()).product::<usize>(),
			elements.len(),
			"the size suggested by the index sets {} does not match the number of elements {}",
			indexes.iter().map(|i| i.len()).product::<usize>(),
			elements.len()
		);
		Self {
			indices: indexes.into_boxed_slice(),
			members: elements.into_boxed_slice(),
		}
	}

	/// Returns whether the array contains any members
	pub fn is_empty(&self) -> bool {
		self.members.is_empty()
	}

	/// Returns the number of dimensions used to index the Array
	pub fn dim(&self) -> u8 {
		self.indices.len() as u8
	}

	/// Returns an iterator over the array and its indices.
	///
	/// The iterator yields all items from start to end.
	pub fn iter(&self) -> impl Iterator<Item = (Vec<Value>, &Value)> {
		self.indices
			.iter()
			.map(|ii| ii.iter())
			.multi_cartesian_product()
			.zip_eq(self.members.iter())
	}
}

impl std::ops::Index<&[Value]> for Array {
	type Output = Value;
	fn index(&self, index: &[Value]) -> &Self::Output {
		let mut idx = 0;
		let mut mult = 1;
		for (ii, ctx) in index.iter().zip_eq(self.indices.iter()) {
			idx *= mult;
			match ctx {
				Index::Integer(r) => {
					if let Value::Integer(ii) = ii {
						assert!(
							r.contains(ii),
							"index out of bounds: the index set is {}..={} but the index is {ii}",
							r.start(),
							r.end()
						);
						idx += (ii - r.start()) as usize;
					} else {
						panic!("incorrect index type: using {ii} for an integer index")
					}
				}
				Index::Enum(e) => {
					if let Value::Enum(val) = ii {
						if e.set == val.set {
							idx += val.val
						} else {
							panic!("incorrect index type: using value of type {} for an index of type {}", 
							if let Some(name) = &e.set.name {name.as_str()} else{"anonymous enum"},
							if let Some(name) = &val.set.name {name.as_str()} else{"anonymous enum"},)
						}
					} else {
						panic!(
							"incorrect index type: using {ii} for an index of type {}",
							if let Some(name) = &e.set.name {
								name.as_str()
							} else {
								"anonymous enum"
							}
						)
					}
				}
			}
			mult *= ctx.len();
		}
		&self.members[idx]
	}
}

impl Display for Array {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.is_empty() {
			return write!(f, "[]");
		}
		if let [Index::Integer(ii)] = &(*self.indices) {
			return write!(f, "[{}: {}]", ii.start(), self.members.iter().format(", "));
		}
		write!(
			f,
			"[{}]",
			self.iter()
				.map(|(ii, x)| {
					let ii_str = match &ii[..] {
						[i] => format!("{i}"),
						ii => format!("({})", ii.iter().format(", ")),
					};
					format!("{ii_str}: {x}")
				})
				.format(", ")
		)
	}
}

/// Representation of Array indexes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
	/// Closed integer range index
	Integer(RangeInclusive<i64>),
	/// Enumerated type used as an index
	Enum(EnumRangeInclusive),
}

impl Index {
	/// Returns the cardinality of the index set
	pub fn len(&self) -> usize {
		match self {
			Index::Integer(r) => {
				if r.is_empty() {
					0
				} else {
					(r.end() - r.start()) as usize + 1
				}
			}
			Index::Enum(e) => e.len(),
		}
	}

	/// Returns the lower bound of the index range (inclusive).
	pub fn start(&self) -> Value {
		match self {
			Index::Integer(it) => Value::Integer(*it.start()),
			Index::Enum(it) => Value::Enum(it.start()),
		}
	}

	/// Returns the upper bound of the index range (inclusive).
	pub fn end(&self) -> Value {
		match self {
			Index::Integer(it) => Value::Integer(*it.end()),
			Index::Enum(it) => Value::Enum(it.end()),
		}
	}

	/// Returns whether the index set contains any members
	pub fn is_empty(&self) -> bool {
		match &self {
			Index::Integer(r) => r.is_empty(),
			Index::Enum(e) => e.is_empty(),
		}
	}

	fn iter(&self) -> IndexIter {
		match self {
			Index::Integer(x) => IndexIter::Integer(x.clone()),
			Index::Enum(e) => IndexIter::Enum(e.clone()),
		}
	}
}

#[derive(Debug, Clone)]
enum IndexIter {
	Integer(RangeInclusive<i64>),
	Enum(EnumRangeInclusive),
}

impl Iterator for IndexIter {
	type Item = Value;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			IndexIter::Integer(it) => it.next().map(Value::Integer),
			IndexIter::Enum(it) => it.next().map(Value::Enum),
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, None)
	}

	fn count(self) -> usize {
		match self {
			IndexIter::Integer(it) => it.count(),
			IndexIter::Enum(it) => it.count(),
		}
	}

	fn last(self) -> Option<Self::Item> {
		match self {
			IndexIter::Integer(it) => it.last().map(Value::Integer),
			IndexIter::Enum(it) => it.last().map(Value::Enum),
		}
	}
}

/// Member declaration of an enumerated type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
	name: Option<String>,
	constructors: Vec<(String, Option<Index>)>,
}

impl Enum {
	/// Returns the number of members of the enumerated type
	pub fn len(&self) -> usize {
		self.constructors
			.iter()
			.map(|(_, i)| if let Some(i) = i { i.len() } else { 1 })
			.sum()
	}

	/// Returns whether the enumerated type has any members
	pub fn is_empty(&self) -> bool {
		self.constructors.is_empty()
	}
}

/// Member declaration of an enumerated type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValue {
	set: Arc<Enum>,
	val: usize,
}

impl EnumValue {
	/// Internal function used to find the constructor definition in the
	/// enumerated type and the position the value has within this constructor
	pub(crate) fn constructor_and_pos(&self) -> (&String, &Option<Index>, usize) {
		let mut i = self.val;
		let c = self
			.set
			.constructors
			.iter()
			.skip_while(|c| {
				let len = if let Some(ii) = &c.1 { ii.len() } else { 1 };
				if i > len {
					i -= len;
					true
				} else {
					false
				}
			})
			.take(1)
			.next()
			.unwrap();
		(&c.0, &c.1, i)
	}

	/// Returns the enumerated type to which this enumerated value belongs
	///
	/// ## Warning
	/// On parsed data the enumerated type might be a placeholder with only
	/// information required to fit the data to a `Program`.
	pub fn enum_type(&self) -> Arc<Enum> {
		self.set.clone()
	}

	/// Returns the name used to construct the value of the enumerated type
	///
	/// The method returns [`None`] if the enumerated type is anonymous
	pub fn constructor(&self) -> Option<&str> {
		let (c, _, _) = self.constructor_and_pos();
		if c == "_" {
			None
		} else {
			Some(c.as_str())
		}
	}

	/// Returns the argument used to construct the value of the enumerated type
	///
	/// This method resturns [`None`] if no argument was used to construct the
	/// value
	pub fn arg(&self) -> Option<Value> {
		let (_, index, i) = self.constructor_and_pos();
		match index {
			Some(Index::Enum(idx)) => Some(Value::Enum(EnumValue {
				set: idx.set.clone(),
				val: i,
			})),
			Some(Index::Integer(idx)) => Some(Value::Integer(idx.start() + i as i64 - 1)),
			None => {
				debug_assert!(i == 1);
				None
			}
		}
	}

	/// Returns the integer value that is internally used to represent the value
	/// of the enumerated types after enumerated types have been type erased.
	pub(crate) fn int_val(&self) -> usize {
		self.val
	}
}

impl Display for EnumValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.constructor() {
			Some(constructor) => match self.arg() {
				Some(arg) => write!(f, "{constructor}({arg})"),
				None => write!(f, "{}", constructor),
			},
			None => write!(
				f,
				"to_enum({}, {})",
				if let Some(name) = &self.set.name {
					name.as_str()
				} else {
					"_"
				},
				self.arg().unwrap()
			),
		}
	}
}

/// A range of values of a single enumerated type bounded inclusively below and above
///
/// The `EnumRangeInclusive::new(start, end)` contains all values with `x >= start`
/// and `x <= end`. It is empty unless `start <= end`.
///
/// This iterator is [fused], but the specific values of `start` and `end` after
/// iteration has finished are **unspecified** other than that [`.is_empty()`]
/// will return `true` once no more values will be produced.
///
/// [fused]: crate::iter::FusedIterator
/// [`.is_empty()`]: EnumRangeInclusive::is_empty
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumRangeInclusive {
	set: Arc<Enum>,
	start: usize,
	end: usize,
}

impl EnumRangeInclusive {
	/// Create a new EnumRangeInclusive
	///
	/// ## Warning
	/// This function will panic if the arguments contained are of two different Enum types
	pub fn new(start: EnumValue, end: EnumValue) -> Self {
		if start.set != end.set {
			panic!("creating EnumRangeInclusive using two different enum types")
		}
		EnumRangeInclusive {
			set: start.set,
			start: start.val,
			end: end.val,
		}
	}

	pub(crate) fn from_internal_values(set: Arc<Enum>, start: usize, end: usize) -> Self {
		EnumRangeInclusive { set, start, end }
	}

	/// Returns `true` if item is contained in the range.
	pub fn contains(&self, item: EnumValue) -> bool {
		if item.set != self.set {
			false
		} else {
			item.val >= self.start && item.val <= self.end
		}
	}

	/// Returns `true' if the iterator is empty.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the lower bound of the EnumRangeInclusive
	///
	/// When using an inclusive range for iteration, the values of `start()` and
	/// [`end()`] are unspecified after the iteration ended. To determine
	/// whether the inclusive range is empty, use the [`is_empty()`] method
	/// instead of comparing `start() > end()`.
	///
	/// Note: the value returned by this method is unspecified after the range
	/// has been iterated to exhaustion.
	///
	/// [`end()`]: EnumRangeInclusive::end
	/// [`is_empty()`]: EnumRangeInclusive::is_empty
	pub fn start(&self) -> EnumValue {
		EnumValue {
			set: self.set.clone(),
			val: self.start,
		}
	}
	/// Returns the upper bound of the EnumRangeInclusive
	///
	/// When using an inclusive range for iteration, the values of [`start()`]
	/// and `end()` are unspecified after the iteration ended. To determine
	/// whether the inclusive range is empty, use the [`is_empty()`] method
	/// instead of comparing `start() > end()`.
	///
	/// Note: the value returned by this method is unspecified after the range
	/// has been iterated to exhaustion.
	///
	/// [`start()`]: EnumRangeInclusive::start
	/// [`is_empty()`]: EnumRangeInclusive::is_empty
	pub fn end(&self) -> EnumValue {
		EnumValue {
			set: self.set.clone(),
			val: self.end,
		}
	}
}

impl From<(EnumValue, EnumValue)> for EnumRangeInclusive {
	fn from(value: (EnumValue, EnumValue)) -> Self {
		EnumRangeInclusive::from((&value.0, &value.1))
	}
}
impl From<(&EnumValue, &EnumValue)> for EnumRangeInclusive {
	fn from(value: (&EnumValue, &EnumValue)) -> Self {
		assert_eq!(
			value.0.set, value.1.set,
			"EnumRangeInclusive must be of a single enumerated type"
		);
		Self {
			set: value.0.set.clone(),
			start: value.0.val,
			end: value.1.val,
		}
	}
}

impl Display for EnumRangeInclusive {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}..{}",
			EnumValue {
				set: self.set.clone(),
				val: self.start,
			},
			EnumValue {
				set: self.set.clone(),
				val: self.end,
			},
		)
	}
}

impl Iterator for EnumRangeInclusive {
	type Item = EnumValue;

	fn next(&mut self) -> Option<Self::Item> {
		if self.start > self.end {
			None
		} else {
			let val = EnumValue {
				set: self.set.clone(),
				val: self.start,
			};
			self.start += 1;
			Some(val)
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = max(self.end - self.start + 1, 0);
		(len, Some(len))
	}

	fn count(self) -> usize {
		self.len()
	}

	fn last(self) -> Option<Self::Item> {
		if self.is_empty() {
			None
		} else {
			Some(EnumValue {
				set: self.set,
				val: self.end,
			})
		}
	}
}
impl DoubleEndedIterator for EnumRangeInclusive {
	fn next_back(&mut self) -> Option<Self::Item> {
		if self.start > self.end {
			None
		} else {
			let val = EnumValue {
				set: self.set.clone(),
				val: self.end,
			};
			self.end -= 1;
			Some(val)
		}
	}
}
impl ExactSizeIterator for EnumRangeInclusive {
	fn len(&self) -> usize {
		self.end - self.start + 1
	}
}
impl FusedIterator for EnumRangeInclusive {}

/// Different representations used to represent sets in [`Value`]
#[derive(Debug, Clone, PartialEq)]
pub enum Set {
	/// Set that spans all members of an enumerated type
	EnumRangeList(Vec<EnumRangeInclusive>),
	/// Sorted list of non-overlapping inclusive floating point ranges
	FloatRangeList(Vec<RangeInclusive<f64>>),
	/// Sorted list of non-overlapping inclusive integer ranges
	IntRangeList(Vec<RangeInclusive<i64>>),
}

impl From<EnumRangeInclusive> for Set {
	fn from(value: EnumRangeInclusive) -> Self {
		Self::EnumRangeList(vec![value])
	}
}
impl FromIterator<EnumRangeInclusive> for Set {
	fn from_iter<T: IntoIterator<Item = EnumRangeInclusive>>(iter: T) -> Self {
		// Eliminate empty ranges & sort ranges by starting value
		let mut iter = iter
			.into_iter()
			.filter(|r| r.start <= r.end)
			.sorted_by_key(|r| r.start);
		if let Some(r) = iter.next() {
			let mut ranges = vec![r];
			// Combine overlapping ranges
			while let Some(r) = iter.next() {
				let last = ranges.last().unwrap();
				if last.end >= r.start {
					(*ranges.last_mut().unwrap()).end = r.end
				} else {
					ranges.push(r)
				}
			}
			Self::EnumRangeList(ranges)
		} else {
			Self::EnumRangeList(Vec::new())
		}
	}
}

impl From<RangeInclusive<f64>> for Set {
	fn from(value: RangeInclusive<f64>) -> Self {
		Self::FloatRangeList(vec![value])
	}
}
impl FromIterator<RangeInclusive<f64>> for Set {
	fn from_iter<T: IntoIterator<Item = RangeInclusive<f64>>>(iter: T) -> Self {
		// Eliminate empty ranges & sort ranges by starting value
		let mut iter = iter
			.into_iter()
			.filter(|r| r.start() <= r.end())
			.sorted_by(|a, b| a.start().partial_cmp(b.start()).unwrap());
		if let Some(r) = iter.next() {
			let mut ranges = vec![r];
			// Combine overlapping ranges
			while let Some(r) = iter.next() {
				let last = ranges.last().unwrap();
				if last.end() >= r.start() {
					*ranges.last_mut().unwrap() = *last.start()..=*r.end()
				} else {
					ranges.push(r)
				}
			}
			Self::FloatRangeList(ranges)
		} else {
			Self::FloatRangeList(Vec::new())
		}
	}
}

impl From<RangeInclusive<i64>> for Set {
	fn from(value: RangeInclusive<i64>) -> Self {
		Self::IntRangeList(vec![value])
	}
}
impl FromIterator<RangeInclusive<i64>> for Set {
	fn from_iter<T: IntoIterator<Item = RangeInclusive<i64>>>(iter: T) -> Self {
		// Eliminate empty ranges & sort ranges by starting value
		let mut iter = iter
			.into_iter()
			.filter(|r| r.start() <= r.end())
			.sorted_by_key(|r| *r.start());
		if let Some(r) = iter.next() {
			let mut ranges = vec![r];
			// Combine overlapping ranges
			while let Some(r) = iter.next() {
				let last = ranges.last().unwrap();
				if last.end() >= r.start() {
					*ranges.last_mut().unwrap() = *last.start()..=*r.end()
				} else {
					ranges.push(r)
				}
			}
			Self::IntRangeList(ranges)
		} else {
			Self::IntRangeList(Vec::new())
		}
	}
}

impl Display for Set {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Set::EnumRangeList(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(f, "{}", ranges.iter().format(" union "))
			}
			Set::IntRangeList(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(
					f,
					"{}",
					ranges
						.iter()
						.format_with(" union ", |range, f| f(&format_args!(
							"{}..{}",
							range.start(),
							range.end()
						)))
				)
			}
			Set::FloatRangeList(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(
					f,
					"{}",
					ranges
						.iter()
						.format_with(" union ", |range, f| f(&format_args!(
							"{}..{}",
							range.start(),
							range.end()
						)))
				)
			}
		}
	}
}

/// A value of a record type
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Record {
	// fields are hidden to possibly replace inner implementation in the future
	fields: Vec<(Arc<str>, Value)>,
}

impl Record {
	/// Returns an iterator over the array and its indices.
	///
	/// The iterator yields all items from start to end.
	pub fn iter(&self) -> impl Iterator<Item = (Arc<str>, &Value)> {
		self.fields.iter().map(|(k, v)| (k.clone(), v))
	}

	/// Returns the number of fields of the record literal
	pub fn len(&self) -> usize {
		self.fields.len()
	}

	/// Returns whether the record literal contains any fields
	pub fn is_empty(&self) -> bool {
		false
	}
}

impl FromIterator<(Arc<str>, Value)> for Record {
	fn from_iter<T: IntoIterator<Item = (Arc<str>, Value)>>(iter: T) -> Self {
		let mut fields: Vec<(Arc<str>, Value)> = iter.into_iter().collect();
		fields.sort_by(|(k1, _), (k2, _)| k1.as_ref().cmp(k2.as_ref()));
		assert!(!fields.is_empty(), "empty record literals are not allowed");
		Self { fields }
	}
}

impl std::ops::Index<&str> for Record {
	type Output = Value;

	fn index(&self, index: &str) -> &Self::Output {
		for (k, v) in &self.fields {
			if k.as_ref() == index {
				return v;
			}
		}
		panic!("no entry found for key");
	}
}

impl Display for Record {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"({})",
			&self
				.fields
				.iter()
				.map(|(k, v)| format!("{}: {}", *k, v))
				.format(", ")
		)
	}
}
