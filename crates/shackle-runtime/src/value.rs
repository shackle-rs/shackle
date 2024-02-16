use std::{
	collections::HashMap,
	fmt::Debug,
	hash::Hash,
	mem::MaybeUninit,
	num::NonZeroU64,
	pin::Pin,
	ptr::NonNull,
	str::{from_utf8, from_utf8_unchecked},
	sync::{
		atomic::{
			self, AtomicU16, AtomicU8,
			Ordering::{Acquire, Relaxed, Release},
		},
		Mutex,
	},
};

use bilge::{
	bitsize,
	prelude::{u10, u2, u52, u61, Number},
	Bitsized, TryFromBits,
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use varlen::{
	define_varlen,
	prelude::{ArrayInitializer, FillWithDefault, FromIterPrefix},
	Initializer, Layout, VarLen,
};

#[bitsize(2)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, TryFromBits)]
enum RefType {
	Boxed = 0b00,
	Float = 0b1,
	Int = 0b10,
}

pub struct Value {
	raw: NonZeroU64,
}

impl Clone for Value {
	fn clone(&self) -> Self {
		if matches!(self.ref_ty(), RefType::Boxed) {
			let mut x = self.get_box();
			debug_assert!(x.refs().ref_count.load(Relaxed) > 0);
			// Using a relaxed ordering is alright here, as knowledge of the
			// original reference prevents other threads from erroneously deleting
			// the object.
			//
			// As explained in the [Boost documentation][1], Increasing the
			// reference counter can always be done with memory_order_relaxed: New
			// references to an object can only be formed from an existing
			// reference, and passing an existing reference from one thread to
			// another must already provide any required synchronization.
			//
			// [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
			x.as_mut().muts().ref_count.fetch_add(1, Relaxed);
		}
		Self { raw: self.raw }
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		if matches!(self.ref_ty(), RefType::Boxed) {
			let mut x = self.get_box();
			debug_assert!(x.refs().ref_count.load(Relaxed) > 0);
			if x.as_mut().muts().ref_count.fetch_sub(1, Release) == 1 {
				atomic::fence(Acquire);
				self.deinit();
				if x.refs().weak_count.load(Relaxed) == 0 {
					unsafe { self.drop_box() }
				}
			}
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		// Check whether bitwise identical
		if self.raw == other.raw {
			return true;
		}
		match self.ty() {
			ValType::Float => {
				if other.ty() != ValType::Float {
					return false;
				}
				let a: f64 = self.try_into().unwrap();
				let b: f64 = other.try_into().unwrap();
				a.eq(&b)
			}
			ValType::Boxed => {
				let a = self.get_box();
				let b = other.get_box();
				if a.ty != b.ty {
					return false;
				}
				match a.ty {
					BoxedType::Seq => self.get_slice().iter().eq(other.get_slice().iter()),
					BoxedType::Str => self.get_str() == other.get_str(),
					BoxedType::View => todo!(),
					BoxedType::Int => {
						let a: IntVal = self.try_into().unwrap();
						let b: IntVal = other.try_into().unwrap();
						a == b
					}
					BoxedType::Float => {
						let a: FloatVal = self.try_into().unwrap();
						let b: FloatVal = other.try_into().unwrap();
						a == b
					}
					BoxedType::BoolVar => todo!(),
					BoxedType::IntVar => todo!(),
					BoxedType::FloatVar => todo!(),
					BoxedType::IntSetVar => todo!(),
				}
			}
			_ => false, // Would have been bitwise identical
		}
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.ty() {
			ValType::Int => {
				write!(
					f,
					"Value::Int {{ {:?} }}",
					self.try_into()
						.map(|i: i64| i.to_string())
						.unwrap_or_default()
				)
			}
			ValType::Float => write!(
				f,
				"Value::Float {{ {:?} }}",
				self.try_into()
					.map(|f: f64| f.to_string())
					.unwrap_or_default()
			),
			ValType::Boxed => {
				let b = self.get_box();
				match b.ty {
					BoxedType::Seq => write!(
						f,
						"Value::Seq {{ ⟨{}⟩ }}",
						self.get_slice()
							.iter()
							.format_with(",", |v, f| f(&format_args!("{v:?}")))
					),
					BoxedType::Str => write!(f, "Value::Str {{ {:?} }}", self.get_str()),
					BoxedType::Int => write!(f, "Value::Str {{ {:?} }}", self.get_str()),
					BoxedType::Float => todo!(),
					BoxedType::View => todo!(),
					BoxedType::BoolVar => todo!(),
					BoxedType::IntVar => todo!(),
					BoxedType::FloatVar => todo!(),
					BoxedType::IntSetVar => todo!(),
				}
			}
		}
	}
}

#[inline(never)]
#[cold]
fn allocation_overflow() -> ! {
	panic!("Allocation size overflow")
}

impl Value {
	pub const MAX_INT: i64 = <u61 as Number>::MAX.value() as i64;
	pub const MIN_INT: i64 = -(Self::MAX_INT + 1);
	const FLOAT_SIGN_BIT: u64 = 0b1 << 63;
	const FLOAT_TAG: u64 = 0b1;
	const INT_SIGN_BIT: u64 = 0b100;
	const INT_TAG: u64 = 0b10;

	/// Determine the type of the reference based on bit-tags
	fn ref_ty(&self) -> RefType {
		// Note that the order in which the tags is checked is imporant. INT_TAG can
		// trigger on a unboxed float
		if self.raw.get() & Self::FLOAT_TAG == Self::FLOAT_TAG {
			RefType::Float // Unboxed Float
		} else if self.raw.get() & Self::INT_TAG == Self::INT_TAG {
			RefType::Int // Unboxed Int
		} else {
			RefType::Boxed // Boxed Value
		}
	}

	fn new_box(init: impl Initializer<BoxedValue>) -> Value {
		// Note: This code is inspired by the initialisation code of varlen::VBox::new
		let layout = init
			.calculate_layout_cautious()
			.unwrap_or_else(|| allocation_overflow());
		let alloc_layout = std::alloc::Layout::from_size_align(layout.size(), BoxedValue::ALIGN)
			.unwrap_or_else(|_| allocation_overflow());
		unsafe {
			let p = std::alloc::alloc(alloc_layout) as *mut BoxedValue;
			let layout_size = layout.size();
			init.initialize(NonNull::new_unchecked(p), layout);
			debug_assert_eq!((*p).calculate_layout().size(), layout_size);
			let p = p as u64;
			debug_assert!(p & (0b11 << 62) == 0 && p != 0);
			Value {
				raw: NonZeroU64::new_unchecked(p << 2),
			}
		}
	}

	fn new_boxed_int(i: IntVal) -> Value {
		let (b, i) = match i {
			IntVal::InfPos => (true, 1),
			IntVal::InfNeg => (true, -1),
			IntVal::Int(i) => (false, i),
		};
		let it = [b as u8].into_iter().chain(i.to_le_bytes());
		Self::new_box(boxed_value::Init {
			ty: ValType::Int,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			raw: FromIterPrefix(it),
		})
	}

	fn new_boxed_float(i: FloatVal) -> Value {
		let (b, f) = match i {
			FloatVal::InfPos => (true, 1.0),
			FloatVal::InfNeg => (true, -1.0),
			FloatVal::Float(f) => (false, f),
		};
		let it = [b as u8].into_iter().chain(f.to_le_bytes());
		Self::new_box(boxed_value::Init {
			ty: ValType::Float,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			raw: FromIterPrefix(it),
		})
	}

	pub fn new_tuple<I: ExactSizeIterator<Item = Value>, C: IntoIterator<IntoIter = I>>(
		members: C,
	) -> Self {
		let iter = members.into_iter();
		if iter.len() == 0 {
			return EMPTY_SEQ.clone();
		}
		Self::new_box(boxed_value::Init {
			ty: ValType::Seq,
			ref_count: 1.into(),
			weak_count: 0.into(),
			len: iter.len() as u32,
			values: FromIterPrefix(iter),
			raw: FillWithDefault,
		})
	}

	pub fn new_str<I: ExactSizeIterator<Item = u8>, S: IntoIterator<IntoIter = I>>(s: S) -> Value {
		let s = s.into_iter();
		if s.len() == 0 {
			return EMPTY_STRING.clone();
		}
		let v = Self::new_box(boxed_value::Init {
			ty: ValType::Str,
			len: s.len() as u32,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			raw: FromIterPrefix(s),
		});
		debug_assert!(from_utf8(v.get_box().refs().raw).is_ok());
		v
	}

	pub fn get_str(&self) -> &str {
		assert_eq!(self.ty(), ValType::Boxed);
		unsafe {
			let v = &*self.get_ptr();
			from_utf8_unchecked(v.refs().raw)
		}
	}

	pub fn get_slice(&self) -> &[Value] {
		assert_eq!(self.ty(), ValType::Boxed);
		unsafe {
			let v = &*self.get_ptr();
			v.refs().values
		}
	}

	pub fn len(&self) -> u32 {
		match self.ty() {
			ValType::Boxed => self.get_box().len,
			_ => unreachable!(),
		}
	}

	unsafe fn get_ptr(&self) -> *mut BoxedValue {
		debug_assert_eq!(self.ty(), ValType::Boxed);
		(self.raw.get() >> 2) as *mut BoxedValue
	}

	fn get_box(&self) -> Pin<&mut BoxedValue> {
		assert_eq!(self.ty(), ValType::Boxed);
		unsafe { Pin::new_unchecked(&mut (*self.get_ptr())) }
	}

	fn deinit(&self) {
		debug_assert_eq!(self.ty(), ValType::Boxed);
		let b = self.get_box();
		match b.ty {
			BoxedType::Seq => {
				// Replace all values in the sequence with non-reference counted objects
				for v in b.muts().values {
					*v = 0.into();
				}
			}
			BoxedType::View => todo!(),
			BoxedType::BoolVar => todo!(),
			BoxedType::IntVar => todo!(),
			BoxedType::FloatVar => todo!(),
			BoxedType::IntSetVar => todo!(),
			_ => {}
		}
	}

	unsafe fn drop_box(&self) {
		debug_assert_eq!(self.ref_ty(), RefType::Boxed);
		let ptr = self.get_ptr();
		let b = self.get_box();
		let layout = BoxedValue::calculate_layout(&b);
		let alloc_layout =
			std::alloc::Layout::from_size_align_unchecked(layout.size(), BoxedValue::ALIGN);
		BoxedValue::vdrop(self.get_box(), layout);
		std::alloc::dealloc(ptr as *mut u8, alloc_layout);
	}
}

pub static EMPTY_STRING: Lazy<Value> = Lazy::new(|| {
	Value::new_box(boxed_value::Init {
		ty: ValType::Str,
		len: 0u32,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		raw: InitEmpty,
	})
});

pub static EMPTY_SEQ: Lazy<Value> = Lazy::new(|| {
	Value::new_box(boxed_value::Init {
		ty: ValType::Seq,
		len: 0u32,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		raw: InitEmpty,
	})
});

static INT_MAP: Lazy<Mutex<HashMap<IntVal, Value>>> = Lazy::new(|| HashMap::new().into());
pub static INT_INF_POS: Lazy<Value> = Lazy::new(|| {
	let mut map = INT_MAP.lock().unwrap();
	let inf = map
		.entry(IntVal::InfPos)
		.or_insert_with(|| Value::new_boxed_int(IntVal::InfPos));
	inf.clone()
});
pub static INT_INF_NEG: Lazy<Value> = Lazy::new(|| {
	let mut map = INT_MAP.lock().unwrap();
	let inf = map
		.entry(IntVal::InfNeg)
		.or_insert_with(|| Value::new_boxed_int(IntVal::InfNeg));
	inf.clone()
});
static FLOAT_MAP: Lazy<Mutex<HashMap<FloatValWrap, Value>>> = Lazy::new(|| HashMap::new().into());
pub static FLOAT_INF_POS: Lazy<Value> = Lazy::new(|| {
	let mut map = FLOAT_MAP.lock().unwrap();
	let inf = map
		.entry(FloatValWrap(FloatVal::InfPos))
		.or_insert_with(|| Value::new_boxed_float(FloatVal::InfPos));
	inf.clone()
});
pub static FLOAT_INF_NEG: Lazy<Value> = Lazy::new(|| {
	let mut map = FLOAT_MAP.lock().unwrap();
	let inf = map
		.entry(FloatValWrap(FloatVal::InfNeg))
		.or_insert_with(|| Value::new_boxed_float(FloatVal::InfNeg));
	inf.clone()
});
struct InitEmpty;

unsafe impl<T> ArrayInitializer<T> for InitEmpty {
	fn initialize(self, dst: &mut [MaybeUninit<T>]) {
		assert_eq!(dst.len(), 0);
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntVal {
	InfPos,
	InfNeg,
	Int(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FloatVal {
	InfPos,
	InfNeg,
	Float(f64),
}

#[derive(Debug, Clone, Copy)]
struct FloatValWrap(FloatVal);

impl Hash for FloatValWrap {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		core::mem::discriminant(&self.0).hash(state);
		if let FloatVal::Float(f) = self.0 {
			state.write_u64(f.to_bits());
		}
	}
}
impl PartialEq for FloatValWrap {
	fn eq(&self, other: &Self) -> bool {
		if let (FloatVal::Float(a), FloatVal::Float(b)) = (self.0, other.0) {
			a.to_bits() == b.to_bits()
		} else {
			self.0 == other.0
		}
	}
}
impl Eq for FloatValWrap {}

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Value::from(if value { 1i32 } else { 0i32 })
	}
}
impl TryInto<bool> for &Value {
	type Error = ();

	fn try_into(self) -> Result<bool, Self::Error> {
		let val: i64 = self.try_into()?;
		if val != 0 && val != 1 {
			todo!()
		}
		Ok(val == 1)
	}
}
impl TryInto<bool> for Value {
	type Error = ();
	fn try_into(self) -> Result<bool, Self::Error> {
		(&self).try_into()
	}
}

impl From<IntVal> for Value {
	fn from(value: IntVal) -> Self {
		match value {
			IntVal::InfPos => INT_INF_POS.clone(),
			IntVal::InfNeg => INT_INF_NEG.clone(),
			IntVal::Int(i) if (Self::MIN_INT..=Self::MAX_INT).contains(&i) => {
				// Can box integer (fits in 62 bits)
				let mut x = i.unsigned_abs() << 3;
				if i.is_negative() {
					x |= Self::INT_SIGN_BIT;
				}
				x |= Self::INT_TAG;
				Self {
					raw: NonZeroU64::new(x).unwrap(),
				}
			}
			iv => {
				// Try and find integer in map or allocate new integer
				let mut map = INT_MAP.lock().unwrap();
				let v = map.entry(iv).or_insert_with(|| Value::new_boxed_int(iv));
				v.clone()
			}
		}
	}
}
impl TryInto<IntVal> for &Value {
	type Error = ();

	fn try_into(self) -> Result<IntVal, Self::Error> {
		if matches!(self.ty(), ValType::Boxed) {
			let b = self.get_box();
			if !matches!(b.ty, BoxedType::Int) {
				todo!()
			}
			debug_assert_eq!(b.refs().raw.len(), 1 + std::mem::size_of::<i64>());
			let inf = b.refs().raw[0] != 0;
			let val = i64::from_le_bytes(b.refs().raw[1..].try_into().unwrap());
			return Ok(if inf {
				if val >= 0 {
					IntVal::InfPos
				} else {
					IntVal::InfNeg
				}
			} else {
				IntVal::Int(val)
			});
		}
		if !matches!(self.ty(), ValType::Int) {
			todo!()
		}
		let pos = (self.raw.get() & Value::INT_SIGN_BIT) == 0;
		let val = (self.raw.get() >> 3) as i64;
		let val = if pos {
			val
		} else if val == 0 {
			Value::MIN_INT
		} else {
			-val
		};
		debug_assert!((Value::MIN_INT..=Value::MAX_INT).contains(&val));
		Ok(IntVal::Int(val))
	}
}
impl TryInto<IntVal> for Value {
	type Error = ();
	fn try_into(self) -> Result<IntVal, Self::Error> {
		(&self).try_into()
	}
}
impl TryInto<i64> for &Value {
	type Error = ();
	fn try_into(self) -> Result<i64, Self::Error> {
		if let IntVal::Int(i) = self.try_into()? {
			Ok(i)
		} else {
			Err(())
		}
	}
}
impl TryInto<i64> for Value {
	type Error = ();
	fn try_into(self) -> Result<i64, Self::Error> {
		(&self).try_into()
	}
}
impl From<i64> for Value {
	fn from(value: i64) -> Self {
		IntVal::Int(value).into()
	}
}
impl From<i32> for Value {
	fn from(value: i32) -> Self {
		let mut raw = (value.unsigned_abs() as u64) << 3;
		if value.is_negative() {
			raw |= Self::INT_SIGN_BIT;
		}
		raw |= Self::INT_TAG;
		Self {
			raw: NonZeroU64::new(raw).unwrap(),
		}
	}
}
impl From<i16> for Value {
	fn from(value: i16) -> Self {
		Value::from(value as i32)
	}
}
impl From<i8> for Value {
	fn from(value: i8) -> Self {
		Value::from(value as i32)
	}
}
impl From<u32> for Value {
	fn from(value: u32) -> Self {
		let raw = (value << 3) as u64 | Self::INT_TAG;
		Value {
			raw: NonZeroU64::new(raw).unwrap(),
		}
	}
}
impl From<u16> for Value {
	fn from(value: u16) -> Self {
		Value::from(value as u32)
	}
}
impl From<u8> for Value {
	fn from(value: u8) -> Self {
		Value::from(value as u32)
	}
}

impl From<FloatVal> for Value {
	fn from(value: FloatVal) -> Self {
		match value {
			FloatVal::InfPos => FLOAT_INF_POS.clone(),
			FloatVal::InfNeg => FLOAT_INF_NEG.clone(),
			FloatVal::Float(f) => {
				const EXPONENT_MASK: u64 = 0x7FF << 52;
				let bits = f.to_bits();
				let mut exponent = (bits & EXPONENT_MASK) >> 52;
				if exponent != 0 {
					if !(513..=1534).contains(&exponent) {
						// Exponent doesn't fit in 10 bits
						let mut map = FLOAT_MAP.lock().unwrap();
						let v = map
							.entry(FloatValWrap(value))
							.or_insert_with(|| Value::new_boxed_float(value));
						return v.clone();
					}
					exponent -= 512; // Make exponent fit in 10 bits, with bias 511
				}
				debug_assert!(exponent <= <u10 as Number>::MAX.value().into());
				let sign = (bits & (1 << 63)) != 0;

				const FRACTION_MASK: u64 = 0xFFFFFFFFFFFFF;
				let fraction = bits & FRACTION_MASK; // Remove one bit of precision
				debug_assert!(fraction <= <u52 as Number>::MAX.value());
				let mut raw = (fraction << 1) | (exponent << 53) | Self::FLOAT_TAG;
				if sign {
					raw |= Self::FLOAT_SIGN_BIT;
				}
				Value {
					raw: NonZeroU64::new(raw).unwrap(),
				}
			}
		}
	}
}
impl From<f64> for Value {
	fn from(value: f64) -> Self {
		FloatVal::Float(value).into()
	}
}
impl TryInto<FloatVal> for &Value {
	type Error = ();

	fn try_into(self) -> Result<FloatVal, Self::Error> {
		if matches!(self.ty(), ValType::Boxed) {
			let b = self.get_box();
			if !matches!(b.ty, BoxedType::Float) {
				todo!()
			}
			let inf = b.refs().raw[0] != 0;
			let val = f64::from_le_bytes(b.refs().raw[1..].try_into().unwrap());
			return Ok(if inf {
				if val >= 0.0 {
					FloatVal::InfPos
				} else {
					FloatVal::InfNeg
				}
			} else {
				FloatVal::Float(val)
			});
		}
		if !matches!(self.ty(), ValType::Float) {
			todo!()
		}
		let pos = (self.raw.get() & Value::FLOAT_SIGN_BIT) == 0;
		let value = self.raw.get() >> 1;
		const EXPONENT_MASK: u64 = 0x3FF << 52;
		let mut exponent = (value & EXPONENT_MASK) >> 52;
		if exponent != 0 {
			exponent += 512; // reconstruct original bias of 1023
		}
		const FRACTION_MASK: u64 = 0xFFFFFFFFFFFFF;
		let fraction = value & FRACTION_MASK;
		let mut value = fraction | (exponent << 52);
		if !pos {
			value |= Value::FLOAT_SIGN_BIT;
		}
		Ok(FloatVal::Float(f64::from_bits(value)))
	}
}
impl TryInto<FloatVal> for Value {
	type Error = ();
	fn try_into(self) -> Result<FloatVal, Self::Error> {
		(&self).try_into()
	}
}
impl TryInto<f64> for &Value {
	type Error = ();

	fn try_into(self) -> Result<f64, Self::Error> {
		let fv: FloatVal = self.try_into()?;
		match fv {
			FloatVal::Float(f) => Ok(f),
			_ => Err(()),
		}
	}
}
impl TryInto<f64> for Value {
	type Error = ();
	fn try_into(self) -> Result<f64, Self::Error> {
		(&self).try_into()
	}
}

impl From<&str> for Value {
	fn from(value: &str) -> Self {
		Self::new_str(value.as_bytes().iter().copied())
	}
}
impl<'a> TryInto<&'a str> for &'a Value {
	type Error = ();

	fn try_into(self) -> Result<&'a str, Self::Error> {
		if self.ty() != ValType::Boxed || self.get_box().ty != BoxedType::Str {
			todo!()
		}
		Ok(self.get_str())
	}
}

#[bitsize(8)]
#[derive(Clone, PartialEq, Eq, TryFromBits)]
enum ValType {
	/// Types that can be represented as simple sequences of values: Tuples, Sets/Range lists, 1-to-n arrays
	Seq,
	/// Character strings
	Str,
	/// View into an array
	View,
	/// Boxed integers (cannot fit in 62 bits)
	Int,
	/// Boxed floats (cannot fit in 63 bits)
	Float,
	/// Boolean decision variable
	BoolVar,
	/// Integer decision variable
	IntVar,
	/// Floating point decision variable
	FloatVar,
	/// Set of integers decision variable
	IntSetVar,
}

#[define_varlen]
struct BoxedValue {
	/// Type of the value
	#[controls_layout]
	ty: ValType,
	/// Length of the value (if relevant)
	#[controls_layout]
	len: u32,
	// Number of values referencing this value
	ref_count: AtomicU16,
	// Number of weak references (e.g., CSE)
	weak_count: AtomicU8,

	#[allow(dead_code)] // accessed through [`.refs()`]
	#[varlen_array]
	values: [Value; match *ty {
		ValType::Seq => *len as usize,
		_ => 0,
	}],
	#[allow(dead_code)] // accessed through [`.refs()`]
	#[varlen_array]
	raw: [u8; match *ty {
		ValType::Str => *len as usize,
		BoxedType::Int => 1 + std::mem::size_of::<i64>(),
		BoxedType::Float => 1 + std::mem::size_of::<f64>(),
		_ => 0,
	}],
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn memory_guarantees() {
		assert_eq!(std::mem::size_of::<Value>(), 8);
		const BOX_BASE_BYTES: usize = 8;
		let val_str = Value::from("12");
		assert_eq!(
			val_str.get_box().calculate_layout().size(),
			BOX_BASE_BYTES + val_str.len() as usize * std::mem::size_of::<u8>()
		);
		let tup2 = Value::new_tuple([1.try_into().unwrap(), 2.2.try_into().unwrap()]);
		assert_eq!(
			tup2.get_box().calculate_layout().size(),
			BOX_BASE_BYTES + tup2.len() as usize * std::mem::size_of::<Value>()
		);
	}

	#[test]
	fn test_integer_value() {
		let zero: i64 = Value::from(0i64).try_into().unwrap();
		assert_eq!(zero, 0i64);

		let one: i64 = Value::from(1i64).try_into().unwrap();
		assert_eq!(one, 1i64);
		let minus_one: i64 = Value::from(-1i64).try_into().unwrap();
		assert_eq!(minus_one, -1i64);

		// Unboxed min and max
		let minimum: i64 = Value::from(Value::MIN_INT).try_into().unwrap();
		assert_eq!(minimum, Value::MIN_INT);
		let maximum: i64 = Value::from(Value::MAX_INT).try_into().unwrap();
		assert_eq!(maximum, Value::MAX_INT);

		// Positive and Negative Infinity
		let pos_inf: IntVal = Value::from(IntVal::InfPos).try_into().unwrap();
		assert_eq!(pos_inf, IntVal::InfPos);
		let neg_inf: IntVal = Value::from(IntVal::InfNeg).try_into().unwrap();
		assert_eq!(neg_inf, IntVal::InfNeg);

		// i64 min and max
		let minimum: i64 = Value::from(i64::MAX).try_into().unwrap();
		assert_eq!(minimum, i64::MAX);
		let maximum: i64 = Value::from(i64::MIN).try_into().unwrap();
		assert_eq!(maximum, i64::MIN);
	}

	#[test]
	fn test_float_value() {
		let zero: f64 = Value::from(0.0f64).try_into().unwrap();
		assert_eq!(zero, 0.0);
		let one: f64 = Value::from(1.0f64).try_into().unwrap();
		assert_eq!(one, 1.0);
		let minus_one: f64 = Value::from(-1.0f64).try_into().unwrap();
		assert_eq!(minus_one, -1.0);

		let twodottwo: f64 = Value::from(2.2f64).try_into().unwrap();
		assert_eq!(twodottwo, 2.2);

		// Positive and Negative Infinity
		let pos_inf: FloatVal = Value::from(FloatVal::InfPos).try_into().unwrap();
		assert_eq!(pos_inf, FloatVal::InfPos);
		let neg_inf: FloatVal = Value::from(FloatVal::InfNeg).try_into().unwrap();
		assert_eq!(neg_inf, FloatVal::InfNeg);
		// f64 min and max
		let minimum: f64 = Value::from(f64::MAX).try_into().unwrap();
		assert_eq!(minimum, f64::MAX);
		let maximum: f64 = Value::from(f64::MIN).try_into().unwrap();
		assert_eq!(maximum, f64::MIN);

		assert_eq!(FLOAT_INF_NEG.ref_ty(), RefType::Boxed);
		assert_eq!(FLOAT_INF_POS.ref_ty(), RefType::Boxed);
	}

	#[test]
	fn test_string_value() {
		let empty = Value::from("");
		assert_eq!(empty.get_str(), "");

		let single = Value::from("1");
		assert_eq!(single.get_str(), "1");
		let double = Value::from("12");
		assert_eq!(double.get_str(), "12");

		let lorem = r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
		Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
		Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
		Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."#;
		let vlorem = Value::from(lorem);
		assert_eq!(vlorem.get_str(), lorem);
	}

	#[test]
	fn test_sequence() {
		let empty = Value::new_tuple([]);
		assert_eq!(empty.get_slice(), &[]);

		let single = Value::new_tuple([1.try_into().unwrap()]);
		assert_eq!(single.get_slice(), &[1.try_into().unwrap()]);

		let tup2 = Value::new_tuple([1.try_into().unwrap(), 2.2.try_into().unwrap()]);
		assert_eq!(
			tup2.get_slice(),
			&[1.try_into().unwrap(), 2.2.try_into().unwrap()]
		);

		let list = (1..=2000).map(Value::from).collect_vec();
		let vlist = Value::new_tuple(list.clone());
		assert_eq!(vlist.get_slice(), &list)
	}
}
