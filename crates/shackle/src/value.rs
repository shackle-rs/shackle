use std::{
	fmt::{self, Display},
	ops::RangeInclusive,
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
	String(String),
	/// Identifier of a value of an enumerated type
	Enum(EnumValue),
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
	indexes: Box<[Index]>,
	members: Box<[Value]>,
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
			indexes: indexes.into_boxed_slice(),
			members: elements.into_boxed_slice(),
		}
	}
}

impl std::ops::Index<&[Value]> for Array {
	type Output = Value;
	fn index(&self, index: &[Value]) -> &Self::Output {
		let mut idx = 0;
		let mut mult = 1;
		for (ii, ctx) in index.iter().zip_eq(self.indexes.iter()) {
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
						if e == &val.set {
							idx += val.val
						} else {
							panic!("incorrect index type: using value of type {} for an index of type {}", 
							if let Some(name) = &e.name {name.as_str()} else{"anonymous enum"},
							if let Some(name) = &val.set.name {name.as_str()} else{"anonymous enum"},)
						}
					} else {
						panic!(
							"incorrect index type: using {ii} for an index of type {}",
							if let Some(name) = &e.name {
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
		let it = self
			.indexes
			.iter()
			.map(|ii| ii.iter())
			.multi_cartesian_product()
			.zip_eq(self.members.iter());

		write!(
			f,
			"[{}]",
			it.map(|(ii, x)| {
				let ii_str = match &ii[..] {
					[i] => format!("{i}"),
					ii => format!("({})", ii.iter().format(", ")),
				};
				format!("{ii_str}: {x}")
			})
			.format(", ")
		)
		// let mut first = true;
		// write!(f, "[")?;
		// for (ii, x) in it {
		// 	if !first {
		// 		write!(f, ", ")?;
		// 	}
		// 	match &ii[..] {
		// 		[i] => write!(f, "{i}: "),
		// 		ii => {
		// 			write!(f, "(")?;
		// 			let mut tup_first = true;
		// 			for i in ii {
		// 				if !tup_first {
		// 					write!(f, ",")?;
		// 				}
		// 				write!(f, "{i}")?;
		// 				tup_first = false;
		// 			}
		// 			write!(f, "): ")
		// 		}
		// 	}?;
		// 	write!(f, "{x}")?;
		// 	first = false;
		// }
		// write!(f, "]")
	}
}

/// Representation of Array indexes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Index {
	/// Closed integer range index
	Integer(RangeInclusive<i64>),
	/// Enumerated type used as an index
	Enum(Arc<Enum>),
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
			Index::Enum(e) => IndexIter::Enum(e.clone(), 1..=e.len()),
		}
	}
}

#[derive(Debug, Clone)]
enum IndexIter {
	Integer(RangeInclusive<i64>),
	Enum(Arc<Enum>, RangeInclusive<usize>),
}

impl Iterator for IndexIter {
	type Item = Value;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			IndexIter::Integer(r) => r.next().map(Value::Integer),
			IndexIter::Enum(e, r) => r.next().map(|v| {
				Value::Enum(EnumValue {
					set: e.clone(),
					val: v,
				})
			}),
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
			Some(Index::Enum(e)) => Some(Value::Enum(EnumValue {
				set: e.clone(),
				val: i,
			})),
			Some(Index::Integer(range)) => Some(Value::Integer(range.start() + i as i64 - 1)),
			None => {
				debug_assert!(i == 1);
				None
			}
		}
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

/// Different representations used to represent sets in [`Value`]
#[derive(Debug, Clone, PartialEq)]
pub enum Set {
	/// List of (unique) Value elements
	SetList(Vec<Value>),
	/// Set that spans all members of an enumerated type
	EnumRangeList(Vec<(EnumValue, EnumValue)>),
	/// Sorted list of non-overlapping inclusive integer ranges
	IntRangeList(Vec<RangeInclusive<i64>>),
	/// Sorted list of non-overlapping inclusive floating point ranges
	FloatRangeList(Vec<RangeInclusive<f64>>),
}

impl Display for Set {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Set::SetList(v) => {
				if v.is_empty() {
					return write!(f, "∅");
				}
				write!(f, "{{{}}}", v.iter().format(", "))
			}
			Set::EnumRangeList(ranges) => {
				if ranges.is_empty()
					|| (ranges.len() == 1
						&& ranges.last().unwrap().0.val > ranges.last().unwrap().1.val)
				{
					return write!(f, "∅");
				}
				write!(
					f,
					"{}",
					ranges
						.iter()
						.map(|range| format!("{}..{}", range.0, range.1))
						.format(" union ")
				)
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
						.map(|range| format!("{}..{}", range.start(), range.end()))
						.format(" union ")
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
						.map(|range| format!("{}..{}", range.start(), range.end()))
						.format(" union ")
				)
			}
		}
	}
}

/// A value of a record type
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Record {
	// fields are hidden to possibly replace inner implementation in the future
	fields: Vec<(Arc<String>, Value)>,
}

impl FromIterator<(Arc<String>, Value)> for Record {
	fn from_iter<T: IntoIterator<Item = (Arc<String>, Value)>>(iter: T) -> Self {
		let mut fields: Vec<(Arc<String>, Value)> = iter.into_iter().collect();
		fields.sort_by(|(k1, _), (k2, _)| k1.as_str().cmp(k2.as_str()));
		Self { fields }
	}
}
impl<'a> IntoIterator for &'a Record {
	type Item = &'a (Arc<String>, Value);
	type IntoIter = std::slice::Iter<'a, (Arc<String>, Value)>;

	#[inline]
	fn into_iter(self) -> Self::IntoIter {
		self.fields.iter()
	}
}
impl std::ops::Index<&str> for Record {
	type Output = Value;

	fn index(&self, index: &str) -> &Self::Output {
		for (k, v) in &self.fields {
			if k.as_str() == index {
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
