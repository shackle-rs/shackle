use std::{
	collections::{HashMap, VecDeque},
	fmt::Debug,
	iter::once,
	mem::MaybeUninit,
	num::NonZeroU64,
	ops::RangeInclusive,
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
	array_init::MoveFrom,
	define_varlen,
	prelude::{ArrayInitializer, FromIterPrefix},
	Initializer, Layout, VarLen,
};

use self::{
	num::{FloatVal, IntVal},
	seq::{InnerViewType, Pairs, SeqView, ViewType},
	set::{FloatSetView, IntSetView},
};

mod num;
mod seq;
mod set;

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
					ValType::BoolVar => todo!(),
					ValType::IntVar => todo!(),
					ValType::FloatVar => todo!(),
					ValType::IntSetVar => todo!(),
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

	fn new_boxed_int(i: IntVal) -> Value {
		let (b, i) = match i {
			IntVal::InfPos => (true, 1),
			IntVal::InfNeg => (true, -1),
			IntVal::Int(i) => (false, i),
		};
		Self::new_box(value_storage::Init {
			ty: ValType::Int,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: FromIterPrefix([i].into_iter()),
			floats: InitEmpty,
			bytes: FromIterPrefix([b as u8].into_iter()),
		})
	}

	fn new_boxed_float(i: FloatVal) -> Value {
		Self::new_box(value_storage::Init {
			ty: ValType::Float,
			len: 0,
			ref_count: 1.into(),
			weak_count: 0.into(),
			values: InitEmpty,
			ints: InitEmpty,
			floats: FromIterPrefix([i].into_iter()),
			bytes: InitEmpty,
		})
	}

	pub fn new_seq_with_dim<V: IntoIterator<Item = Value>, D: IntoIterator<Item = (i64, i64)>>(
		values: V,
		dims: D,
	) -> Value {
		let values = values.into_iter().collect_vec();
		if values.is_empty() {
			return EMPTY_SEQ.clone();
		}
		let dims = dims.into_iter().collect_vec();
		assert!(!dims.is_empty());
		if dims.len() == 1 {
			assert_eq!(dims[0].1 as usize, values.len());
			return values.into_iter().collect();
		}

		let mut eq_count: i64 = 0;
		if values.len() >= 4 {
			let _ = values.iter().skip(1).take_while(|&x| {
				if x == &values[0] {
					eq_count += 1;
					true
				} else {
					false
				}
			});
		}
		let iter = values
			.into_iter()
			.skip(if eq_count >= 3 { eq_count as usize } else { 0 });

		let len = iter.len() as u32;
		let val = Self::new_box(value_storage::Init {
			ty: ValType::Seq,
			ref_count: 1.into(),
			weak_count: 0.into(),
			len,
			values: FromIterPrefix(iter),
			ints: InitEmpty,
			floats: InitEmpty,
			bytes: InitEmpty,
		});

		assert_eq!(
			dims.iter()
				.map(|(min, max)| (max - min + 1) as u32)
				.sum::<u32>(),
			len
		);
		match seq {
			SeqView::Slice {
				dims: _,
				slice,
				storage: _,
			} => Self::new_box(value_storage::Init {
				ty: ValType::View,
				ref_count: 1.into(),
				weak_count: 0.into(),
				len: ViewType::new(
					InnerViewType::Slice,
					(dims.len() / 2) as u8,
					slice.len() as u8,
				)
				.as_len(),
				values: MoveFrom([self.clone()]),
				ints: FromIterPrefix(
					dims.into_iter()
						.chain(slice.iter().flat_map(|(&min, &max)| [min, max])),
				),
				floats: InitEmpty,
				bytes: InitEmpty,
			}),
			SeqView::Compressed {
				dims: _,
				repeat,
				storage: _,
			} => Self::new_box(value_storage::Init {
				ty: ValType::View,
				ref_count: 1.into(),
				weak_count: 0.into(),
				len: ViewType::new(InnerViewType::Compact, (dims.len() / 2) as u8, 0).as_len(),
				values: MoveFrom([self.clone()]),
				ints: FromIterPrefix(once(repeat).chain(dims.into_iter())),
				floats: InitEmpty,
				bytes: InitEmpty,
			}),
			SeqView::WithDim { dims: _, storage } => Self::new_box(value_storage::Init {
				ty: ValType::View,
				ref_count: 1.into(),
				weak_count: 0.into(),
				len: ViewType::new(InnerViewType::Dim, (dims.len() / 2) as u8, 0).as_len(),
				values: MoveFrom([storage.clone()]),
				ints: FromIterPrefix(dims.into_iter()),
				floats: InitEmpty,
				bytes: InitEmpty,
			}),
			_ => Self::new_box(value_storage::Init {
				ty: ValType::View,
				ref_count: 1.into(),
				weak_count: 0.into(),
				len: ViewType::new(InnerViewType::Dim, (dims.len() / 2) as u8, 0).as_len(),
				values: MoveFrom([self.clone()]),
				ints: FromIterPrefix(dims.into_iter()),
				floats: InitEmpty,
				bytes: InitEmpty,
			}),
		}
	}

	/// Create a slice view of a sequence
	///
	/// This creates a view that occludes part of the underlying sequence, and
	/// optionally gives the view new dimensions.
	///
	/// # Warning
	/// This method will panic if the underlying value is a non-sequence value, if
	///  the sequence is sliced outside its underlying index set(s), or if the
	///  number of non-occluded elements does not equal the size of the provided
	///  dimensions
	pub fn slice<
		It1: ExactSizeIterator<Item = (i64, i64)>,
		It2: ExactSizeIterator<Item = (i64, i64)>,
		I: IntoIterator<IntoIter = It1>,
		J: IntoIterator<IntoIter = It2>,
	>(
		&self,
		select_idxs: J,
		view_dims: I,
	) -> Value {
		let DataView::Seq(seq) = self.deref() else {
			panic!("unable to give dimensions to non-sequence value");
		};
		let slice: Vec<i64> = select_idxs
			.into_iter()
			.flat_map(|(start, end)| [start, end])
			.collect();
		assert_eq!(
			seq.dims(),
			slice.len() / 2,
			"unable to slice a sequence with {} dimensions, using {} sets",
			seq.dims(),
			slice.len() / 2
		);

		assert!(
			slice
				.iter()
				.tuples()
				.zip(1..=seq.dims())
				.all(|((start, end), d)| {
					let (d_start, d_end) = seq.dim(d);
					d_start <= *start && *end <= d_end
				}),
			"slicing index out of bounds"
		);

		let dims: Vec<i64> = view_dims
			.into_iter()
			.flat_map(|(start, end)| [start, end])
			.collect();
		assert_eq!(
			dims.iter().tuples().map(|(start, end)| end - start + 1).product::<i64>(),
			slice.iter().tuples().map(|(start, end)| end - start + 1).product::<i64>(),
			"size of the dimensions provided for the slice does not match the number of elements in sliced sequence"
		);
		// TODO: See what underlying view could be incorporated in the slice. (WithDim and Slice?)
		Self::new_box(value_storage::Init {
			ty: ValType::View,
			ref_count: 1.into(),
			weak_count: 0.into(),
			len: ViewType::new(
				InnerViewType::Slice,
				(dims.len() / 2) as u8,
				(slice.len() / 2) as u8,
			)
			.as_len(),
			values: MoveFrom([self.clone()]),
			ints: FromIterPrefix(dims.into_iter().chain(slice.into_iter())),
			floats: InitEmpty,
			bytes: InitEmpty,
		})
	}

	/// Create a sequence view transposing an existing view
	///
	/// The arguments of this method are the number of the index sets to which the
	/// n-th index will be translated. Negative numbers can be used to reverse a
	/// dimension.
	///
	/// # Warning
	/// This method will panic if it is called on a non-sequence value, or if it
	/// is provided with a number for an index set that is beyond the possible
	/// number of index sets.
	pub fn transpose<D: IntoIterator<Item = i64>>(&self, dims: D) -> Value {
		let DataView::Seq(seq) = self.deref() else {
			panic!("unable to give dimensions to non-sequence value");
		};
		let dims = dims.into_iter().collect_vec();
		assert!(
			dims.iter()
				.all(|i| i.unsigned_abs() as usize <= seq.dims() && *i != 0),
			"invalid index set reference"
		);
		Self::new_box(value_storage::Init {
			ty: ValType::View,
			ref_count: 1.into(),
			weak_count: 0.into(),
			len: ViewType::new(InnerViewType::Transpose, dims.len() as u8, 0).as_len(),
			values: MoveFrom([self.clone()]),
			ints: FromIterPrefix(
				dims.flat_map(|(min, max)| [min, max])
					.chain(slice.flat_map(|(min, max)| [min, max])),
			),
			floats: InitEmpty,
			bytes: InitEmpty,
		})
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
		bytes: InitEmpty,
	})
});

pub static EMPTY_SEQ: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::Seq,
		len: 0u32,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: InitEmpty,
		floats: InitEmpty,
		bytes: InitEmpty,
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

static FLOAT_MAP: Lazy<Mutex<HashMap<FloatVal, Value>>> = Lazy::new(|| HashMap::new().into());
#[allow(dead_code)] // TODO!
pub static FLOAT_INF_POS: Lazy<Value> = Lazy::new(|| FloatVal::INFINITY.into());
#[allow(dead_code)] // TODO!
pub static FLOAT_INF_NEG: Lazy<Value> = Lazy::new(|| FloatVal::NEG_INFINITY.into());

pub static INT_SET_EMPTY: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::IntSet,
		len: 0b11,
		ref_count: 1.into(),
		weak_count: 0.into(),
		values: InitEmpty,
		ints: FromIterPrefix([1i64, 0].into_iter()),
		floats: InitEmpty,
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
		bytes: InitEmpty,
	})
});

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Value::from(if value { 1 } else { 0 })
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
		if let DataView::Int(i) = self.deref() {
			Ok(i)
		} else {
			todo!()
		}
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

impl From<FloatVal> for Value {
	fn from(value: FloatVal) -> Self {
		let f: f64 = value.into();
		const EXPONENT_MASK: u64 = 0x7FF << 52;
		let bits = f.to_bits();
		let mut exponent = (bits & EXPONENT_MASK) >> 52;
		if exponent != 0 {
			if !(513..=1534).contains(&exponent) {
				// Exponent doesn't fit in 10 bits
				let mut map = FLOAT_MAP.lock().unwrap();
				let v = map
					.entry(value)
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
impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Value::from(FloatVal::from(value))
	}
}
impl TryInto<FloatVal> for &Value {
	type Error = ();
	fn try_into(self) -> Result<FloatVal, Self::Error> {
		if let DataView::Float(f) = self.deref() {
			Ok(f)
		} else {
			todo!()
		}
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
		Ok(fv.into())
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
		if let DataView::Str(s) = self.deref() {
			Ok(s)
		} else {
			todo!()
		}
	}
}

impl FromIterator<Value> for Value {
	fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
		let v = iter.into_iter().collect_vec();
		if v.is_empty() {
			return EMPTY_SEQ.clone();
		}
		let mut eq_count = 0;
		if v.len() >= 4 {
			eq_count = v.iter().skip(1).take_while(|&x| x == &v[0]).count();
		}
		let iter = v.into_iter().skip(if eq_count >= 3 { eq_count } else { 0 });

		let len = iter.len() as u32;
		let mut val = Self::new_box(value_storage::Init {
			ty: ValType::Seq,
			ref_count: 1.into(),
			weak_count: 0.into(),
			len,
			values: FromIterPrefix(iter),
			ints: InitEmpty,
			floats: InitEmpty,
			bytes: InitEmpty,
		});

		if eq_count >= 3 {
			val = Self::new_box(value_storage::Init {
				ty: ValType::View,
				ref_count: 1.into(),
				weak_count: 0.into(),
				len: ViewType::new(InnerViewType::Compact, 1, 0).as_len(),
				values: MoveFrom([val]),
				ints: MoveFrom([eq_count as i64, 1, len as i64]),
				floats: InitEmpty,
				bytes: InitEmpty,
			});
		}
		val
	}
}
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
			bytes: InitEmpty,
		})
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataView<'a> {
	Int(IntVal),
	Float(FloatVal),
	Seq(SeqView<'a>),
	Str(&'a str),
	IntSet(IntSetView<'a>),
	FloatSet(FloatSetView<'a>),
}

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

	#[allow(dead_code)] // accessed through [`.refs()`]
	#[varlen_array]
	values: [Value; match *ty {
		ValType::Seq => *len as usize,
		ValType::View => 1,
		_ => 0,
	}],
	#[allow(dead_code)] // accessed through [`.refs()`]
	#[varlen_array]
	ints: [i64; match *ty {
		ValType::Int => 1,
		ValType::IntSet => ValueStorage::int_set_len(*len),
		ValType::View => ViewType::from_len(*len).int_len(),
		_ => 0,
	}],
	#[allow(dead_code)] // accessed through [`.refs()`]
	#[varlen_array]
	floats: [FloatVal; match *ty {
		ValType::Float => 1,
		ValType::FloatSet => *len as usize * 2,
		_ => 0,
	}],
	#[allow(dead_code)] // accessed through [`.refs()`]
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
	use std::iter::empty;

	use expect_test::expect;
	use itertools::Itertools;

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
		let pos_inf: f64 = Value::from(f64::INFINITY).try_into().unwrap();
		assert_eq!(pos_inf, f64::INFINITY);
		let neg_inf: f64 = Value::from(f64::NEG_INFINITY).try_into().unwrap();
		assert_eq!(neg_inf, f64::NEG_INFINITY);
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

	#[test]
	fn test_sequence() {
		let empty: Value = empty::<Value>().collect();
		assert_eq!(empty.deref(), DataView::Seq(SeqView::Direct(&[])));
		assert!(empty.is_constant(&EMPTY_SEQ));

		let single: Value = [Value::from(1)].into_iter().collect();
		assert_eq!(single.deref(), DataView::Seq(SeqView::Direct(&[1.into()])));

		let tup2: Value = [Value::from(1), 2.2.into()].into_iter().collect();
		assert_eq!(
			tup2.deref(),
			DataView::Seq(SeqView::Direct(&[1.into(), 2.2.into()]))
		);

		let list = (1..=2000).map(Value::from).collect_vec();
		let vlist: Value = list.iter().cloned().collect();
		assert_eq!(vlist.deref(), DataView::Seq(SeqView::Direct(&list)));
	}

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
}
