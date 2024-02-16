use std::{
	fmt::Debug,
	pin::Pin,
	ptr::NonNull,
	str::{from_utf8, from_utf8_unchecked},
};

use bilge::{
	bitsize,
	prelude::{u10, u2, u51, u61, u62, Number},
	Bitsized, FromBits, TryFromBits,
};
use varlen::{
	define_varlen,
	prelude::{FillWithDefault, FromIterPrefix},
	Initializer, Layout, VarLen,
};

#[bitsize(2)]
#[derive(Debug, PartialEq, Eq, FromBits)]
enum RefType {
	Nil,
	Int,
	Float,
	Boxed,
}

#[bitsize(64)]
pub struct Value {
	ty: ValType,
	inner: u62,
}

impl Default for Value {
	fn default() -> Self {
		Self::new(ValType::Nil, 0u8.into())
	}
}

impl Clone for Value {
	fn clone(&self) -> Self {
		if matches!(self.ref_ty(), RefType::Boxed) {
			let mut x = self.get_box();
			debug_assert!(*x.refs().ref_count > 0);
			*x.as_mut().muts().ref_count += 1;
		}
		Self::new(self.ty(), self.inner())
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		if matches!(self.ref_ty(), RefType::Boxed) {
			let mut x = self.get_box();
			debug_assert!(*x.refs().ref_count > 0);
			*x.as_mut().muts().ref_count -= 1;
			if *x.refs().ref_count == 0 {
				// TODO: Deinit?
				if *x.refs().weak_count == 0 {
					unsafe { self.drop_box() }
				}
			}
		}
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		// Check whether bitwise identical
		if self.value == other.value {
			return true;
		}
		match self.ty() {
			ValType::Float => todo!(),
			ValType::Boxed => {
				let a = self.get_box();
				let b = other.get_box();
				if a.ty != b.ty {
					return false;
				}
				match a.ty {
					BoxedType::Seq => todo!(),
					BoxedType::Str => self.get_str() == other.get_str(),
					BoxedType::View => todo!(),
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
			ValType::Nil => write!(f, "Value::Nil"),
			ValType::Int => {
				write!(
					f,
					"Value::Int {{ {} ({:#X}) }}",
					self.clone()
						.try_into()
						.map(|i: i64| i.to_string())
						.unwrap_or_default(),
					self.inner().value()
				)
			}
			ValType::Float => todo!(),
			ValType::Boxed => todo!(),
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
	const SIGN_BIT: u64 = 1 << 61;

	fn new_box(init: impl Initializer<BoxedValue>) -> Value {
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
			// TODO(reinerp): Compare directly on Layout type? Or too much generated code?
			let p = p as usize;
			Value::new(ValType::Boxed, u62::new(p as u64))
		}
	}

	pub fn new_tuple<I: ExactSizeIterator<Item = Value>, C: IntoIterator<IntoIter = I>>(
		members: C,
	) -> Self {
		let iter = members.into_iter();
		if iter.len() == 0 {
			return Value::new(ValType::Nil, 0u8.into());
		}
		Self::new_box(boxed_value::Init {
			ty: ValType::Seq,
			ref_count: 1,
			weak_count: 0,
			len: iter.len() as u32,
			values: FromIterPrefix(iter),
			raw: FillWithDefault,
		})
	}

	pub fn new_str<I: ExactSizeIterator<Item = u8>, S: IntoIterator<IntoIter = I>>(s: S) -> Value {
		let s = s.into_iter();
		if s.len() == 0 {
			Self::new(ValType::Nil, 0u8.into());
		}
		let v = Self::new_box(boxed_value::Init {
			ty: ValType::Str,
			len: s.len() as u32,
			ref_count: 1,
			weak_count: 0,
			values: FillWithDefault,
			raw: FromIterPrefix(s),
		});
		debug_assert!(!from_utf8(v.get_box().refs().raw).is_err());
		v
	}

	pub fn get_str(&self) -> &str {
		match self.ty() {
			ValType::Nil => "",
			ValType::Boxed => unsafe {
				let b = &*(self.inner().value() as *mut BoxedValue);
				from_utf8_unchecked(b.refs().raw)
			},
			_ => unreachable!(),
		}
	}

	pub fn get_slice(&self) -> &[Value] {
		match self.ty() {
			ValType::Nil => &[],
			ValType::Boxed => unsafe {
				let b = &*(self.inner().value() as *mut BoxedValue);
				b.refs().values
			},
			_ => unreachable!(),
		}
	}

	pub fn len(&self) -> u32 {
		match self.ty() {
			ValType::Nil => 0,
			ValType::Boxed => self.get_box().len,
			_ => unreachable!(),
		}
	}

	fn get_box(&self) -> Pin<&mut BoxedValue> {
		assert_eq!(self.ty(), ValType::Boxed);
		let ptr = self.inner().value() as *mut BoxedValue;
		unsafe { Pin::new_unchecked(&mut (*ptr)) }
	}

	unsafe fn drop_box(&self) {
		debug_assert_eq!(self.ref_ty(), RefType::Boxed);
		let ptr = u64::from(self.inner()) as *mut BoxedValue;
		let b = self.get_box();
		unsafe {
			let layout = BoxedValue::calculate_layout(&b);
			let alloc_layout =
				std::alloc::Layout::from_size_align_unchecked(layout.size(), BoxedValue::ALIGN);
			BoxedValue::vdrop(self.get_box(), layout);
			std::alloc::dealloc(ptr as *mut u8, alloc_layout);
		}
	}
}

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Value::from(if value { 1i32 } else { 0i32 })
	}
}
impl TryInto<bool> for Value {
	type Error = ();

	fn try_into(self) -> Result<bool, Self::Error> {
		if !matches!(self.ty(), ValType::Int)
			|| (self.inner() != 0u8.into() && self.inner() != 1u8.into())
		{
			todo!()
		}
		return Ok(self.inner() == 1u8.into());
	}
}

impl TryFrom<i64> for Value {
	type Error = ();

	fn try_from(value: i64) -> Result<Self, Self::Error> {
		if value < Self::MIN_INT || value > Self::MAX_INT {
			todo!()
		}
		let mut x = value.abs() as u64;
		if value.is_negative() {
			x |= Self::SIGN_BIT;
		}
		Ok(Value::new(ValType::Int, u62::new(x)))
	}
}
impl TryInto<i64> for Value {
	type Error = ();

	fn try_into(self) -> Result<i64, Self::Error> {
		if !matches!(self.ty(), ValType::Int) {
			todo!()
		}
		let pos = (self.inner().value() & Self::SIGN_BIT) == 0;
		let val = self.inner().value();
		let val = if pos {
			val as i64
		} else if -(val as i64) == Self::MIN_INT {
			Self::MIN_INT
		} else {
			-((val & (Self::SIGN_BIT - 1)) as i64)
		};
		debug_assert!(val >= Self::MIN_INT && val <= Self::MAX_INT);
		Ok(val)
	}
}
impl From<i32> for Value {
	fn from(value: i32) -> Self {
		let mut x = value.abs() as u64;
		if value.is_negative() {
			x |= Self::SIGN_BIT;
		}
		Value::new(ValType::Int, u62::new(x))
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
		Value::new(ValType::Int, u62::new(value.into()))
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

impl TryFrom<f64> for Value {
	type Error = ();

	fn try_from(value: f64) -> Result<Self, Self::Error> {
		const EXPONENT_MASK: u64 = 0x7FF << 52;
		let value = value.to_bits();
		let mut exponent = (value & EXPONENT_MASK) >> 52;
		if exponent != 0 {
			if exponent < 513 || exponent > 1534 {
				// Exponent doesn't fit in 10 bits
				todo!()
			}
			exponent -= 512; // Make exponent fit in 10 bits, with bias 511
		}
		debug_assert!(exponent <= <u10 as Number>::MAX.value().into());
		let sign = (value & (1 << 63)) != 0;

		const FRACTION_MASK: u64 = 0xFFFFFFFFFFFFF;
		let fraction = (value & FRACTION_MASK) >> 1; // Remove one bit of precision
		debug_assert!(fraction <= <u51 as Number>::MAX.value().into());
		let mut value = fraction | (exponent << 51);
		if sign {
			value |= Self::SIGN_BIT;
		}
		Ok(Value::new(ValType::Float, u62::new(value)))
	}
}
impl TryInto<f64> for Value {
	type Error = ();

	fn try_into(self) -> Result<f64, Self::Error> {
		if !matches!(self.ty(), ValType::Float) {
			todo!()
		}
		let value = self.inner().value();
		let pos = (value & Self::SIGN_BIT) == 0;
		const EXPONENT_MASK: u64 = 0x3FF << 51;
		let mut exponent = (value & EXPONENT_MASK) >> 51;
		if exponent != 0 {
			exponent += 512; // reconstruct original bias of 1023
		}
		const FRACTION_MASK: u64 = 0x7FFFFFFFFFFFF;
		let fraction = value & FRACTION_MASK;
		let mut value = (fraction << 1) | (exponent << 52);
		if !pos {
			value |= 1u64 << 63;
		}
		Ok(f64::from_bits(value))
	}
}

impl From<&str> for Value {
	fn from(value: &str) -> Self {
		Self::new_str(value.as_bytes().iter().copied())
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
	ref_count: u16,
	// Number of weak references (e.g., CSE)
	weak_count: u8,

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
		let zero: i64 = Value::try_from(0i64).unwrap().try_into().unwrap();
		assert_eq!(zero, 0i64);

		let one: i64 = Value::try_from(1i64).unwrap().try_into().unwrap();
		assert_eq!(one, 1i64);
		let minus_one: i64 = Value::try_from(-1i64).unwrap().try_into().unwrap();
		assert_eq!(minus_one, -1i64);

		let minimum: i64 = Value::try_from(Value::MIN_INT).unwrap().try_into().unwrap();
		assert_eq!(minimum, Value::MIN_INT);
		let maximum: i64 = Value::try_from(Value::MAX_INT).unwrap().try_into().unwrap();
		assert_eq!(maximum, Value::MAX_INT);
	}

	#[test]
	fn test_float_value() {
		let zero: f64 = Value::try_from(0.0f64).unwrap().try_into().unwrap();
		assert_eq!(zero, 0.0);
		let one: f64 = Value::try_from(1.0f64).unwrap().try_into().unwrap();
		assert_eq!(one, 1.0);
		let minus_one: f64 = Value::try_from(-1.0f64).unwrap().try_into().unwrap();
		assert_eq!(minus_one, -1.0);

		let twodottwo: f64 = Value::try_from(2.2f64).unwrap().try_into().unwrap();
		assert_eq!(twodottwo, 2.2);
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
