mod bool_var;
mod float_var;
mod int_set_var;
mod int_var;
mod num;
mod seq;
mod set;

use std::{
	fmt::Debug,
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
	prelude::{u2, u61, Number},
	Bitsized, TryFromBits,
};
use once_cell::sync::Lazy;
use varlen::{
	define_varlen,
	prelude::{ArrayInitializer, FromIterPrefix},
	Initializer, Layout, VarLen,
};

use crate::value::{
	bool_var::{BoolVar, BoolVarRef},
	float_var::{FloatVar, FloatVarRef},
	int_set_var::{IntSetVar, IntSetVarRef},
	int_var::{IntVar, IntVarRef},
	num::{FloatVal, IntVal},
	seq::{InnerViewType, Pairs, SeqView, ViewType},
	set::{FloatSetView, IntSetView},
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
			let mut x = self.get_pin();
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
			let mut slf = self.get_pin();
			debug_assert!(slf.refs().ref_count.load(Relaxed) > 0);
			if slf.as_mut().muts().ref_count.fetch_sub(1, Release) == 1 {
				atomic::fence(Acquire);
				unsafe { ValueStorage::deinit(self.get_pin()) };
				if slf.refs().weak_count.load(Relaxed) == 0 {
					unsafe { ValueStorage::drop(slf) }
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
		// Interned or boxed data would have been bitwise identical
		if !matches!(self.ref_ty(), RefType::Boxed)
			|| matches!(self.get_pin().ty, ValType::Int | ValType::Float)
		{
			// TODO: is this okay for Float??
			return false;
		}
		// Compare
		self.deref().eq(&self.deref())
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.deref().fmt(f)
	}
}

#[inline(never)]
#[cold]
fn allocation_overflow() -> ! {
	panic!("Allocation size overflow")
}

impl Value {
	const FLOAT_SIGN_BIT: u64 = 0b1 << 63;
	const FLOAT_TAG: u64 = 0b1;
	const INT_SIGN_BIT: u64 = 0b100;
	const INT_TAG: u64 = 0b10;
	pub const MAX_INT: i64 = <u61 as Number>::MAX.value() as i64;
	pub const MIN_INT: i64 = -(Self::MAX_INT + 1);

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

	pub fn deref(&self) -> DataView<'_> {
		match self.ref_ty() {
			RefType::Float => {
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
				DataView::Float(f64::from_bits(value).into())
			}
			RefType::Int => {
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
				DataView::Int(IntVal::Int(val))
			}
			RefType::Boxed => unsafe {
				let v = &*self.get_ptr();
				match v.ty {
					ValType::Int => {
						let inf = v.refs().bytes[0] != 0;
						let val = v.refs().ints[0];
						DataView::Int(if inf {
							if val >= 0 {
								IntVal::InfPos
							} else {
								IntVal::InfNeg
							}
						} else {
							IntVal::Int(val)
						})
					}
					ValType::Float => {
						let val = v.refs().floats[0];
						DataView::Float(val)
					}
					ValType::Seq => DataView::Seq(SeqView::Direct(v.refs().values)),
					ValType::Str => DataView::Str(from_utf8_unchecked(v.refs().bytes)),
					ValType::IntSet => DataView::IntSet(IntSetView {
						has_lb: v.len & 0b01 != 0,
						has_ub: v.len & 0b10 != 0,
						intervals: v.refs().ints,
					}),
					ValType::FloatSet => DataView::FloatSet(FloatSetView {
						intervals: v.refs().floats,
					}),
					ValType::View => {
						let vty = ViewType::from_len(v.len);
						DataView::Seq(match vty.ty() {
							InnerViewType::Dim => SeqView::WithDim {
								dims: Pairs::new(v.refs().ints),
								storage: &v.refs().values[0],
							},
							InnerViewType::Slice => {
								let slice_offset = vty.slice() as usize;
								SeqView::Slice {
									dims: Pairs::new(&v.refs().ints[..slice_offset]),
									slice: Pairs::new(&v.refs().ints[slice_offset..]),
									storage: &v.refs().values[0],
								}
							}
							InnerViewType::Transpose => SeqView::Transposed {
								reloc: v.refs().ints,
								storage: &v.refs().values[0],
							},
							InnerViewType::Compact => SeqView::Compressed {
								dims: Pairs::new(&v.refs().ints[1..]),
								repeat: v.refs().ints[0],
								storage: &v.refs().values[0],
							},
						})
					}
					ValType::BoolVar => {
						DataView::BoolVar(v.refs().bool_var[0].lock().unwrap().into())
					}
					ValType::IntVar => DataView::IntVar(v.refs().int_var[0].lock().unwrap().into()),
					ValType::FloatVar => {
						DataView::FloatVar(v.refs().float_var[0].lock().unwrap().into())
					}
					ValType::IntSetVar => {
						DataView::IntSetVar(v.refs().int_set_var[0].lock().unwrap().into())
					}
				}
			},
		}
	}

	pub fn is_constant(&self, c: &'static Value) -> bool {
		self.raw == c.raw
	}

	fn new_box(init: impl Initializer<ValueStorage>) -> Value {
		// Note: This code is inspired by the initialisation code of varlen::VBox::new
		let layout = init
			.calculate_layout_cautious()
			.unwrap_or_else(|| allocation_overflow());
		let alloc_layout = std::alloc::Layout::from_size_align(layout.size(), ValueStorage::ALIGN)
			.unwrap_or_else(|_| allocation_overflow());
		unsafe {
			let p = std::alloc::alloc(alloc_layout) as *mut ValueStorage;
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

	pub fn new_str<I: ExactSizeIterator<Item = u8>, S: IntoIterator<IntoIter = I>>(s: S) -> Value {
		let s = s.into_iter();
		if s.len() == 0 {
			return EMPTY_STRING.clone();
		}
		let v = Self::new_box(value_storage::Init {
			ty: ValType::Str,
			len: s.len() as u32,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: InitEmpty,
			floats: InitEmpty,
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: FromIterPrefix(s),
		});
		debug_assert!(from_utf8(v.get_pin().refs().bytes).is_ok());
		v
	}

	unsafe fn get_ptr(&self) -> *mut ValueStorage {
		debug_assert_eq!(self.ref_ty(), RefType::Boxed);
		(self.raw.get() >> 2) as *mut ValueStorage
	}

	fn get_pin(&self) -> Pin<&mut ValueStorage> {
		assert_eq!(self.ref_ty(), RefType::Boxed);
		unsafe { Pin::new_unchecked(&mut (*self.get_ptr())) }
	}
}

pub static EMPTY_STRING: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::Str,
		len: 0u32,
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

impl From<&str> for Value {
	fn from(value: &str) -> Self {
		Self::new_str(value.as_bytes().iter().copied())
	}
}
impl<'a> TryInto<&'a str> for &'a Value {
	type Error = ();

	fn try_into(self) -> Result<&'a str, Self::Error> {
		if let DataView::Str(s) = self.deref() {
			Ok(s)
		} else {
			todo!()
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum DataView<'a> {
	Int(IntVal),
	Float(FloatVal),
	Seq(SeqView<'a>),
	Str(&'a str),
	IntSet(IntSetView<'a>),
	FloatSet(FloatSetView<'a>),
	BoolVar(BoolVarRef<'a>),
	IntVar(IntVarRef<'a>),
	FloatVar(FloatVarRef<'a>),
	IntSetVar(IntSetVarRef<'a>),
}

macro_rules! define_var_ref {
	($v:ident, $r:ident) => {
		#[derive(Debug)]
		pub struct $r<'a> {
			guard: std::sync::MutexGuard<'a, $v>,
		}

		impl PartialEq for $r<'_> {
			fn eq(&self, other: &Self) -> bool {
				*self.guard == *other.guard
			}
		}
		impl Eq for $r<'_> {}
		impl std::ops::Deref for $r<'_> {
			type Target = $v;

			fn deref(&self) -> &Self::Target {
				&self.guard
			}
		}
		impl std::ops::DerefMut for $r<'_> {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.guard
			}
		}
		impl<'a> From<std::sync::MutexGuard<'a, $v>> for $r<'a> {
			fn from(guard: std::sync::MutexGuard<'a, $v>) -> Self {
				Self { guard }
			}
		}
	};
}
pub(crate) use define_var_ref;

#[bitsize(8)]
#[derive(Clone, PartialEq, Eq, TryFromBits)]
enum ValType {
	/// Boxed integers (cannot fit in 62 bits)
	Int,
	/// Boxed floats (cannot fit in 63 bits)
	Float,
	/// Types that can be represented as simple sequences of values: Tuples, 1-to-n arrays
	Seq,
	/// Character strings
	Str,
	/// View into an array
	View,
	/// Int Set (Range List)
	IntSet,
	/// Float Set (Range List)
	FloatSet,
	/// Boolean decision variable
	BoolVar,
	/// Integer decision variable
	IntVar,
	/// Floating point decision variable
	FloatVar,
	/// Set of integers decision variable
	IntSetVar,
}

#[allow(dead_code)] // attributes accessed through [`.refs()`]
#[define_varlen]
struct ValueStorage {
	/// Type of the value
	#[controls_layout]
	ty: ValType,
	/// Length of the value (if relevant)
	/// Seq -> number of values
	/// Str -> number of bytes
	/// IntSet -> lowest two bit whether value has lb/ub, rest is number of gaps
	/// FloatSet -> Number of intervals
	#[controls_layout]
	len: u32,
	// Number of values referencing this value
	ref_count: AtomicU16,
	// Number of weak references (e.g., CSE)
	weak_count: AtomicU8,

	#[varlen_array]
	values: [Value; match *ty {
		ValType::Seq => *len as usize,
		ValType::View => 1,
		_ => 0,
	}],
	#[varlen_array]
	ints: [i64; match *ty {
		ValType::Int => 1,
		ValType::IntSet => ValueStorage::int_set_len(*len),
		ValType::View => ViewType::from_len(*len).int_len(),
		_ => 0,
	}],
	#[varlen_array]
	floats: [FloatVal; match *ty {
		ValType::Float => 1,
		ValType::FloatSet => *len as usize * 2,
		_ => 0,
	}],
	#[varlen_array]
	bool_var: [Mutex<BoolVar>; match *ty {
		ValType::BoolVar => 1,
		_ => 0,
	}],
	#[varlen_array]
	int_var: [Mutex<IntVar>; match *ty {
		ValType::IntVar => 1,
		_ => 0,
	}],
	#[varlen_array]
	float_var: [Mutex<FloatVar>; match *ty {
		ValType::IntVar => 1,
		_ => 0,
	}],
	#[varlen_array]
	int_set_var: [Mutex<IntSetVar>; match *ty {
		ValType::IntSetVar => 1,
		_ => 0,
	}],
	#[varlen_array]
	bytes: [u8; match *ty {
		// Fully stored as bytes
		ValType::Str => *len as usize,
		// Stored infinity tag
		ValType::Int => 1,
		_ => 0,
	}],
}

impl ValueStorage {
	const fn int_set_len(len: u32) -> usize {
		let has_lb = (len & 0b01 != 0) as usize;
		let has_ub = (len & 0b10 != 0) as usize;
		((len >> 2) as usize * 2) + has_lb + has_ub
	}

	unsafe fn deinit(slf: Pin<&mut ValueStorage>) {
		match slf.ty {
			ValType::Seq => {
				// Replace all values in the sequence with non-reference counted objects
				for v in slf.muts().values {
					*v = 0.into();
				}
			}
			ValType::View => slf.muts().values[0] = 0.into(),
			ValType::IntVar => todo!(),
			ValType::FloatVar => todo!(),
			ValType::IntSetVar => todo!(),
			_ => {}
		}
	}

	unsafe fn drop(slf: Pin<&mut ValueStorage>) {
		let ptr: *mut ValueStorage = Pin::get_unchecked_mut(slf);
		let slf = Pin::new_unchecked(&mut *ptr);
		let layout = ValueStorage::calculate_layout(&slf);
		let alloc_layout =
			std::alloc::Layout::from_size_align_unchecked(layout.size(), ValueStorage::ALIGN);
		ValueStorage::vdrop(slf, layout);
		std::alloc::dealloc(ptr as *mut u8, alloc_layout);
	}
}

struct InitEmpty;
unsafe impl<T> ArrayInitializer<T> for InitEmpty {
	fn initialize(self, dst: &mut [MaybeUninit<T>]) {
		assert_eq!(dst.len(), 0);
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn memory_guarantees() {
		assert_eq!(std::mem::size_of::<Value>(), 8);
		assert_eq!(ValueStorage::ALIGN, 8);

		const BOX_BASE_BYTES: usize = std::mem::size_of::<u64>();

		const S: &str = "123";
		let val_str = Value::from(S);
		assert_eq!(
			val_str.get_pin().calculate_layout().size(),
			BOX_BASE_BYTES + std::mem::size_of_val(S.as_bytes())
		);
		let t: &[Value] = &[1.into(), 2.2.into()];
		let tup2: Value = t.iter().cloned().collect();
		assert_eq!(
			tup2.get_pin().calculate_layout().size(),
			BOX_BASE_BYTES + std::mem::size_of_val(t)
		);
	}

	#[test]
	fn test_string_value() {
		let empty = Value::from("");
		assert_eq!(empty.deref(), DataView::Str(""));
		assert!(empty.is_constant(&EMPTY_STRING));

		let single = Value::from("1");
		assert_eq!(single.deref(), DataView::Str("1"));
		let double = Value::from("12");
		let double_str: &str = (&double).try_into().unwrap();
		assert_eq!(double_str, "12");

		let lorem = r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
		Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
		Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
		Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."#;
		let vlorem = Value::from(lorem);
		assert_eq!(vlorem.deref(), DataView::Str(lorem));
	}
}
