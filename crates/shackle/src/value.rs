//! Values types used for input and output for Programs

use std::{
	cmp::max,
	fmt::{self, Display},
	iter::FusedIterator,
	ops::{Deref, RangeInclusive},
	rc::Rc,
	sync::{Arc, Mutex, MutexGuard},
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
impl From<EnumValue> for Value {
	fn from(value: EnumValue) -> Self {
		Self::Enum(value)
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
			Value::String(v) => write!(f, "{v:?}"),
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
				write!(
					f,
					"({}{})",
					v.iter().format(", "),
					if v.len() == 1 { "," } else { "" }
				)
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

	/// Create a new empty array
	pub fn empty() -> Self {
		Self {
			indices: [].into(),
			members: [].into(),
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
							"index out of bounds: the index set is {}..{} but the index is {ii}",
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
						if e.ty == val.ty {
							idx += val.pos
						} else {
							panic!("incorrect index type: using value of type {} for an index of type {}", 
							e.ty.name,
							val.ty.name)
						}
					} else {
						panic!(
							"incorrect index type: using {ii} for an index of type {}",
							e.ty.name
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
			if *ii.start() == 1 {
				return write!(f, "[{}]", self.members.iter().format(", "));
			} else {
				return write!(f, "[{}: {}]", ii.start(), self.members.iter().format(", "));
			}
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

impl Display for Index {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Index::Integer(v) => write!(f, "{}..{}", v.start(), v.end()),
			Index::Enum(v) => write!(f, "{v}"),
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
#[derive(Debug)]
pub struct Enum {
	name: Arc<str>,
	pub(crate) state: Mutex<EnumInner>,
}

impl Enum {
	pub(crate) fn from_data(name: Arc<str>) -> Self {
		Self {
			name,
			state: EnumInner::NoDefinition.into(),
		}
	}

	pub(crate) fn model_defined<I: IntoIterator<Item = Arc<str>>>(name: Arc<str>, deps: I) -> Self {
		Self {
			name,
			state: EnumInner::AwaitData(Vec::from_iter(deps).into_boxed_slice()).into(),
		}
	}

	/// Returns the number of members of the enumerated type
	///
	/// ## Warning
	/// This function will panic if Enum type is uninitialized
	pub fn len(&self) -> usize {
		self.lock().iter().map(|(_, _, len)| len).sum()
	}

	/// Returns the name of the enumerated type
	pub fn name(&self) -> &Arc<str> {
		&self.name
	}

	/// Returns whether the enumerated type has any members
	///
	/// ## Warning
	/// This function will panic if Enum type is uninitialized
	pub fn is_empty(&self) -> bool {
		self.lock().iter().next().is_none()
	}

	pub(crate) fn lock(&self) -> CtorLock {
		CtorLock {
			lock: self.state.lock().unwrap(),
		}
	}

	pub(crate) fn get(&self, name: &str) -> Option<(usize, Box<[Index]>)> {
		let mut offset = 1;
		for ctor in self.lock().iter() {
			if &*ctor.0 == name {
				return Some((offset, ctor.1.clone()));
			}
			offset += ctor.2;
		}
		None
	}
}

impl PartialEq for Enum {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.state.lock().unwrap().deref() == other.state.lock().unwrap().deref()
	}
}
impl Eq for Enum {}
impl Display for Enum {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.is_empty() {
			write!(f, "{} = {{}}", self.name)
		} else {
			write!(
				f,
				"{} = {}",
				self.name,
				self.lock().iter().format_with(" ++ ", |ctor, f| {
					if ctor.1.is_empty() {
						f(&format_args!("{{{}}}", ctor.0)) // TODO: repeated constructors with no arguments should be grouped together
					} else {
						f(&format_args!("{}({})", ctor.0, ctor.1.iter().format(",")))
					}
				})
			)
		}
	}
}

pub(crate) struct CtorLock<'a> {
	lock: MutexGuard<'a, EnumInner>,
}

impl<'a> CtorLock<'a> {
	/// Returns the list of the constructors of the enumerated type
	///
	/// ## Warning
	/// This function will panic if Enum type is uninitialized
	pub fn iter(&self) -> impl Iterator<Item = &Constructor> {
		let EnumInner::Constructors(ref cons) = self.lock.deref() else {
			panic!("cannot access constructors of an uninitialized enumerated type")
		};
		cons.iter().filter(|ctor| ctor.2 > 0)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnumInner {
	NoDefinition,
	AwaitData(Box<[Arc<str>]>),
	Constructors(Box<[Constructor]>),
}

pub(crate) type Constructor = (Arc<str>, Box<[Index]>, usize);

/// Member declaration of an enumerated type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValue {
	ty: Arc<Enum>,
	pos: usize,
}

impl EnumValue {
	pub(crate) fn from_enum_and_pos(ty: Arc<Enum>, pos: usize) -> Self {
		debug_assert!(pos >= 1 && pos <= ty.len());
		Self { ty, pos }
	}

	/// Internal function used to find the constructor definition in the
	/// enumerated type and the arguments to the constructor to create the value
	pub(crate) fn constructor_and_args(&self) -> (Arc<str>, Vec<Value>) {
		let mut val = self.pos - 1;
		let lock = self.ty.lock();
		let (name, idx, _) = lock
			.iter()
			.skip_while(|(_, _, len)| {
				if val >= *len {
					val -= len;
					true
				} else {
					false
				}
			})
			.take(1)
			.next()
			.unwrap();
		let mut args = vec![Value::Absent; idx.len()];
		for i in (0..idx.len()).rev() {
			let offset = val % idx[i].len();
			args[i] = match &idx[i] {
				Index::Integer(ii) => Value::Integer(ii.start() + offset as i64),
				Index::Enum(ii) => {
					EnumValue::from_enum_and_pos(ii.enum_type(), ii.start + offset).into()
				}
			};
			val /= idx[i].len();
		}
		(name.clone(), args)
	}

	/// Returns the enumerated type to which this enumerated value belongs
	pub fn enum_type(&self) -> Arc<Enum> {
		self.ty.clone()
	}

	/// Returns the name used to construct the value of the enumerated type
	///
	/// The method returns [`None`] if the enumerated type is anonymous
	pub fn constructor(&self) -> Arc<str> {
		self.constructor_and_args().0
	}

	/// Returns the argument used to construct the value of the enumerated type
	///
	/// This method resturns [`None`] if no argument was used to construct the
	/// value
	pub fn args(&self) -> Vec<Value> {
		let (_, args) = self.constructor_and_args();
		args
	}

	/// Returns the integer value that is internally used to represent the value
	/// of the enumerated types after enumerated types have been type erased.
	pub(crate) fn int_val(&self) -> usize {
		self.pos
	}
}

impl Display for EnumValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (c, a) = self.constructor_and_args();
		write!(f, "{c}")?;
		if !a.is_empty() {
			write!(f, "({})", a.iter().format(","))
		} else {
			Ok(())
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
	ty: Arc<Enum>,
	start: usize,
	end: usize,
}

impl EnumRangeInclusive {
	/// Create a new EnumRangeInclusive
	///
	/// ## Warning
	/// This function will panic if the arguments contained are of two different Enum types
	pub fn new(start: EnumValue, end: EnumValue) -> Self {
		if start.ty != end.ty {
			panic!("creating EnumRangeInclusive using two different enum types")
		}
		EnumRangeInclusive {
			ty: start.ty,
			start: start.pos,
			end: end.pos,
		}
	}

	pub(crate) fn from_enum_and_positions(set: Arc<Enum>, start: usize, end: usize) -> Self {
		EnumRangeInclusive {
			ty: set,
			start,
			end,
		}
	}

	/// Returns the enumerated type to which this enumerated value belongs
	pub fn enum_type(&self) -> Arc<Enum> {
		self.ty.clone()
	}

	/// Returns `true` if item is contained in the range.
	pub fn contains(&self, item: &EnumValue) -> bool {
		if item.ty != self.ty {
			false
		} else {
			item.pos >= self.start && item.pos <= self.end
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
			ty: self.ty.clone(),
			pos: self.start,
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
			ty: self.ty.clone(),
			pos: self.end,
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
			value.0.ty, value.1.ty,
			"EnumRangeInclusive must be of a single enumerated type"
		);
		Self {
			ty: value.0.ty.clone(),
			start: value.0.pos,
			end: value.1.pos,
		}
	}
}

impl Display for EnumRangeInclusive {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}..{}",
			EnumValue {
				ty: self.ty.clone(),
				pos: self.start,
			},
			EnumValue {
				ty: self.ty.clone(),
				pos: self.end,
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
				ty: self.ty.clone(),
				pos: self.start,
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
				ty: self.ty,
				pos: self.end,
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
				ty: self.ty.clone(),
				pos: self.end,
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
	Enum(Vec<EnumRangeInclusive>),
	/// Sorted list of non-overlapping inclusive floating point ranges
	Float(Vec<RangeInclusive<f64>>),
	/// Sorted list of non-overlapping inclusive integer ranges
	Int(Vec<RangeInclusive<i64>>),
}

impl From<EnumRangeInclusive> for Set {
	fn from(value: EnumRangeInclusive) -> Self {
		Self::Enum(vec![value])
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
			for r in iter {
				let last = ranges.last().unwrap();
				if last.end >= r.start {
					ranges.last_mut().unwrap().end = r.end
				} else {
					ranges.push(r)
				}
			}
			Self::Enum(ranges)
		} else {
			Self::Enum(Vec::new())
		}
	}
}

impl From<RangeInclusive<f64>> for Set {
	fn from(value: RangeInclusive<f64>) -> Self {
		Self::Float(vec![value])
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
			for r in iter {
				let last = ranges.last().unwrap();
				if last.end() >= r.start() {
					*ranges.last_mut().unwrap() = *last.start()..=*r.end()
				} else {
					ranges.push(r)
				}
			}
			Self::Float(ranges)
		} else {
			Self::Float(Vec::new())
		}
	}
}

impl From<RangeInclusive<i64>> for Set {
	fn from(value: RangeInclusive<i64>) -> Self {
		Self::Int(vec![value])
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
			for r in iter {
				let last = ranges.last().unwrap();
				if last.end() >= r.start() {
					*ranges.last_mut().unwrap() = *last.start()..=*r.end()
				} else {
					ranges.push(r)
				}
			}
			Self::Int(ranges)
		} else {
			Self::Int(Vec::new())
		}
	}
}

impl Display for Set {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Set::Enum(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(f, "{}", ranges.iter().format(" ∪ "))
			}
			Set::Int(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(
					f,
					"{}",
					ranges.iter().format_with(" ∪ ", |range, f| f(&format_args!(
						"{}..{}",
						range.start(),
						range.end()
					)))
				)
			}
			Set::Float(ranges) => {
				if ranges.is_empty() || (ranges.len() == 1 && ranges.last().unwrap().is_empty()) {
					return write!(f, "∅");
				}
				write!(
					f,
					"{}",
					ranges.iter().format_with(" ∪ ", |range, f| f(&format_args!(
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

#[cfg(test)]
mod tests {
	use itertools::Itertools;

	use crate::value::Array;

	#[test]
	fn test_array_iter() {
		assert_eq!(Array::empty().iter().collect_vec(), Vec::new());
	}
}
