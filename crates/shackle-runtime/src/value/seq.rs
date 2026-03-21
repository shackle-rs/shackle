use std::{iter::once, ops::Index};

use bilge::{
	arbitrary_int::{u14, u2},
	bitsize,
	prelude::Number,
	Bitsized, DebugBits, FromBits,
};
use itertools::Itertools;
use once_cell::sync::Lazy;
use varlen::array_init::{FromIterPrefix, MoveFrom};

use super::{DataView, Value};
use crate::value::{value_storage, InitEmpty, ValType};

#[bitsize(2)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, FromBits, Clone, Copy)]
pub(crate) enum InnerViewType {
	// A sequence with added dimensions.
	Dim = 0b00,
	// A sequence with dimensions, where not all underlying values are used
	Slice = 0b01,
	// A reversed sequence
	Transpose = 0b10,
	// Repeat first element
	Compact = 0b11,
}

#[bitsize(32)]
#[derive(DebugBits, PartialEq, Eq, PartialOrd, Ord, FromBits, Clone, Copy)]
pub(crate) struct ViewType {
	/// Type of View
	pub(crate) ty: InnerViewType,
	/// Number of dimensions
	pub(crate) dim: u8,
	/// Number of dimensions resulting from slice
	pub(crate) slice: u8,
	reserved: u14,
}

impl ViewType {
	// TODO: replace once bilge has const feature
	pub(crate) const fn from_len(val: u32) -> ViewType {
		unsafe { std::mem::transmute(val) }
	}

	pub(crate) const fn as_len(&self) -> u32 {
		self.value
	}

	// TODO: replace once bilge has const feature
	pub(crate) const fn const_ty(&self) -> InnerViewType {
		match self.value & 0b11 {
			0b00 => InnerViewType::Dim,
			0b01 => InnerViewType::Slice,
			0b10 => InnerViewType::Transpose,
			0b11 => InnerViewType::Compact,
			_ => unreachable!(),
		}
	}

	// TODO: replace once bilge has const feature
	pub(crate) const fn const_dim(&self) -> usize {
		((self.value >> 2) & 0b11111111) as usize
	}

	// TODO: replace once bilge has const feature
	pub(crate) const fn const_slice(&self) -> usize {
		((self.value >> 10) & 0b11111111) as usize
	}

	pub(crate) const fn int_len(&self) -> usize {
		match self.const_ty() {
			InnerViewType::Dim => self.const_dim() * 2,
			InnerViewType::Slice => (self.const_dim() + self.const_slice()) * 2,
			InnerViewType::Compact => self.const_dim() * 2 + 1,
			InnerViewType::Transpose => self.const_dim(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeqView<'a> {
	Direct(&'a [Value]),
	WithDim {
		dims: Pairs<'a, i64>,
		storage: &'a Value,
	},
	Slice {
		dims: Pairs<'a, i64>,
		slice: Pairs<'a, i64>,
		storage: &'a Value,
	},
	Transposed {
		reloc: &'a [i64],
		storage: &'a Value,
	},
	Compressed {
		dims: Pairs<'a, i64>,
		repeat: i64,
		storage: &'a Value,
	},
}

impl SeqView<'_> {
	pub const fn dims(&self) -> usize {
		match self {
			SeqView::Direct(_) => 1,
			SeqView::WithDim { dims, storage: _ }
			| SeqView::Compressed {
				dims,
				repeat: _,
				storage: _,
			}
			| SeqView::Slice {
				dims,
				slice: _,
				storage: _,
			} => dims.len(),
			SeqView::Transposed { reloc, storage: _ } => reloc.len(),
		}
	}

	pub fn dim(&self, i: usize) -> (i64, i64) {
		debug_assert!(1 <= i && i <= self.dims());
		match self {
			SeqView::Direct(v) => (1, v.len() as i64),
			SeqView::WithDim { dims, storage: _ }
			| SeqView::Compressed {
				dims,
				repeat: _,
				storage: _,
			}
			| SeqView::Slice {
				dims,
				slice: _,
				storage: _,
			} => {
				let pair = &dims[i - 1];
				(pair[0], pair[1])
			}
			SeqView::Transposed { reloc, storage } => {
				let DataView::Seq(v) = storage.deref() else {
					unreachable!("Storage of SeqView must point to another sequence")
				};
				v.dim(reloc[i - 1].unsigned_abs() as usize)
			}
		}
	}

	pub fn len(&self) -> usize {
		match self {
			SeqView::Direct(v) => v.len(),
			SeqView::Slice {
				dims,
				slice: _,
				storage: _,
			} => dims
				.iter()
				.map(|(start, end)| (end - start + 1) as usize)
				.sum(),
			SeqView::Compressed {
				dims: _,
				repeat,
				storage,
			} => {
				let DataView::Seq(v) = storage.deref() else {
					unreachable!("Storage of SeqView must point to another sequence")
				};
				v.len() + *repeat as usize
			}
			SeqView::WithDim { dims: _, storage } | SeqView::Transposed { reloc: _, storage } => {
				let DataView::Seq(v) = storage.deref() else {
					unreachable!("Storage of SeqView must point to another sequence")
				};
				v.len()
			}
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &Value> {
		// TODO: Can this be done more effiencently?
		let indices = (1..=self.dims())
			.map(|i| {
				let dim = self.dim(i);
				dim.0..=dim.1
			})
			.multi_cartesian_product();
		indices.map(|v| self.index(&v))
	}
}

impl<'a> Index<&[i64]> for SeqView<'a> {
	type Output = Value;

	fn index(&self, index: &[i64]) -> &'a Self::Output {
		debug_assert_eq!(self.dims(), index.len());

		const RESOLVE_DIM: fn(&[i64], &Pairs<i64>) -> i64 = |idx, dims| {
			debug_assert_eq!(idx.len(), dims.len());
			let mut real_dim: i64 = dims.iter().map(|(min, max)| max - min + 1).product();

			let mut real_idx = 1;
			for (ix, (min, max)) in idx.iter().zip(dims.iter()) {
				real_dim /= max - min + 1;
				real_idx += (ix - min) * real_dim;
			}
			real_idx
		};

		const RESOLVE_SLICE: fn(i64, &Pairs<i64>, &SeqView) -> Vec<i64> = |idx, slice, seq| {
			let mut cur_idx = idx - 1;
			let mut idxs = vec![0; slice.len()];

			debug_assert_eq!(slice.len(), seq.dims());
			for i in (1..=slice.len()).rev() {
				let &[sl_min, sl_max] = &slice[i - 1] else {
					unreachable!()
				};
				let sl_width = sl_max - sl_min + 1;
				let (d_min, _) = seq.dim(i);
				idxs[i - 1] = (cur_idx % sl_width) + d_min;
				cur_idx /= sl_width;
			}
			idxs
		};

		let mut dv = DataView::Seq(self.clone());
		let mut index: Vec<i64> = Vec::from_iter(index.iter().copied());
		while let DataView::Seq(s) = dv {
			match s {
				SeqView::Direct(v) => {
					debug_assert!(1 <= index[0] && index[0] <= v.len() as i64);
					let i = (index[0] - 1) as usize;
					return &v[i];
				}
				SeqView::WithDim { dims, storage } => {
					index = vec![RESOLVE_DIM(&index, &dims)];
					dv = storage.deref();
				}
				SeqView::Slice {
					dims,
					slice,
					storage,
				} => {
					let idx = RESOLVE_DIM(&index, &dims);
					let DataView::Seq(seq) = storage.deref() else {
						unreachable!()
					};
					index = RESOLVE_SLICE(idx, &slice, &seq);
					dv = storage.deref();
				}
				SeqView::Transposed { reloc, storage } => {
					let DataView::Seq(v) = storage.deref() else {
						unreachable!()
					};
					debug_assert_eq!(index.len(), reloc.len());
					index = reloc
						.iter()
						.map(|&i| {
							debug_assert!(
								-(reloc.len() as i64) <= i && i <= reloc.len() as i64 && i != 0
							);
							let ii = (i.abs() - 1) as usize;
							if i.is_positive() {
								index[ii]
							} else {
								let (_, max) = v.dim(i.unsigned_abs() as usize);
								max - index[ii] + 1
							}
						})
						.collect_vec();
					dv = storage.deref();
				}
				SeqView::Compressed {
					dims,
					repeat,
					storage,
				} => {
					let idx = RESOLVE_DIM(&index, &dims);
					index = vec![if idx <= repeat { 1 } else { idx - repeat + 1 }];
					dv = storage.deref();
				}
			}
		}
		unreachable!()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pairs<'a, T>(&'a [T]);

impl<'a, T> Pairs<'a, T> {
	pub fn new(slice: &'a [T]) -> Self {
		assert_eq!(slice.len() % 2, 0);
		Self(slice)
	}

	pub const fn len(&self) -> usize {
		self.0.len() / 2
	}

	pub fn iter(&self) -> impl Iterator<Item = (&'a T, &'a T)> {
		self.0.iter().tuples()
	}
}

impl<T> Index<usize> for Pairs<'_, T> {
	type Output = [T];

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index * 2..=index * 2 + 1]
	}
}

impl Value {
	/// Create a sequence view with custom index set dimensions
	///
	/// # Warning
	/// This method will panic if it is called on a non-sequence value, or if the
	/// size of the provided dimensions do not equal the number of elements in the
	/// sequence.
	pub fn with_dim<D: IntoIterator<Item = (i64, i64)>>(&self, dims: D) -> Value {
		assert!(matches!(self.deref(), DataView::Seq(_)));
		let DataView::Seq(seq) = self.deref() else {
			panic!("unable to give dimensions to non-sequence value");
		};
		let dims = dims.into_iter().flat_map(|(a, b)| [a, b]).collect_vec();
		assert!(
			seq.len()
				== dims
					.iter()
					.tuples()
					.map(|(a, b)| (b - a + 1) as usize)
					.product::<usize>(),
			"dimensions given do not match the length of the array"
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
				bool_var: InitEmpty,
				int_var: InitEmpty,
				float_var: InitEmpty,
				int_set_var: InitEmpty,
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
				ints: FromIterPrefix(once(repeat).chain(dims)),
				floats: InitEmpty,
				bool_var: InitEmpty,
				int_var: InitEmpty,
				float_var: InitEmpty,
				int_set_var: InitEmpty,
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
				bool_var: InitEmpty,
				int_var: InitEmpty,
				float_var: InitEmpty,
				int_set_var: InitEmpty,
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
				bool_var: InitEmpty,
				int_var: InitEmpty,
				float_var: InitEmpty,
				int_set_var: InitEmpty,
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
			ints: FromIterPrefix(dims.into_iter().chain(slice)),
			floats: InitEmpty,
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
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
			ints: FromIterPrefix(dims.into_iter()),
			floats: InitEmpty,
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
			bytes: InitEmpty,
		})
	}
}

pub static EMPTY_SEQ: Lazy<Value> = Lazy::new(|| {
	Value::new_box(value_storage::Init {
		ty: ValType::Seq,
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
			bool_var: InitEmpty,
			int_var: InitEmpty,
			float_var: InitEmpty,
			int_set_var: InitEmpty,
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
				bool_var: InitEmpty,
				int_var: InitEmpty,
				float_var: InitEmpty,
				int_set_var: InitEmpty,
				bytes: InitEmpty,
			});
		}
		val
	}
}

#[cfg(test)]
mod tests {
	use std::iter::empty;

	use itertools::Itertools;

	use crate::{
		value::{
			seq::{InnerViewType, SeqView, ViewType, EMPTY_SEQ},
			DataView,
		},
		Value,
	};

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

		// Test reverse
		let blist = vlist.transpose([-1]);
		let DataView::Seq(view) = blist.deref() else {
			unreachable!()
		};
		assert!(itertools::equal(view.iter(), list.iter().rev()));

		let list = (1..=100).map(|_| Value::from(1i64)).collect_vec();
		let vlist: Value = list.iter().cloned().collect();
		let DataView::Seq(view) = vlist.deref() else {
			unreachable!()
		};

		assert!(matches!(
			view,
			SeqView::Compressed {
				dims: _,
				repeat: 99,
				storage: _
			}
		));
		assert_eq!(view.len(), list.len());
		itertools::equal(list.iter(), view.iter());

		let vlist: Value = (1..=9).map(Value::from).collect();
		// Test dimensions
		let dim_list = vlist.with_dim([(-1, 1), (5, 7)]);
		let DataView::Seq(view) = dim_list.deref() else {
			unreachable!()
		};
		for i in -1..=1 {
			for j in 5..=7 {
				assert_eq!(view[&[i, j]], ((i + 1) * 3 + j - 4).into());
			}
		}
		// Test slicing
		for i in 1..=3 {
			// Slice row
			let row = dim_list.slice([(i - 2, i - 2), (5, 7)], [(0, 2)]);
			let DataView::Seq(view) = row.deref() else {
				unreachable!()
			};
			assert_eq!(view.len(), 3);
			assert_eq!(view.iter().count(), 3);
			itertools::equal(
				view.iter().cloned(),
				(1..=3).map(|j| Value::from((i - 1) * 3 + j)),
			);
			// Slice column
			let row = dim_list.slice([(-1, 1), (i + 4, i + 4)], [(1, 3)]);
			let DataView::Seq(view) = row.deref() else {
				unreachable!()
			};
			assert_eq!(view.len(), 3);
			assert_eq!(view.iter().count(), 3);
			itertools::equal(
				view.iter().cloned(),
				(1..=3).map(|j| Value::from((j - 1) * 3 + i)),
			);
		}
		// Test transpose
		let t = dim_list.with_dim([(1, 3), (1, 3)]).transpose([2, 1]);
		let DataView::Seq(view) = t.deref() else {
			unreachable!()
		};
		for i in 1..=3 {
			for j in 1..=3 {
				assert_eq!(view[&[i, j]], ((j - 1) * 3 + i).into());
			}
		}
	}

	#[test]
	fn test_view_struct() {
		let ty = [
			InnerViewType::Dim,
			InnerViewType::Slice,
			InnerViewType::Transpose,
		];
		let dim = [0, 1, u8::MAX];
		let slice = [0, 1, u8::MAX];

		for t in ty {
			for d in dim {
				for s in slice {
					let vt = ViewType::new(t, d, s);
					assert_eq!(vt.const_ty(), t);
					assert_eq!(vt.const_dim(), usize::from(d));
					assert_eq!(vt.const_slice(), usize::from(s));
					assert_eq!(ViewType::from_len(vt.value), vt);
				}
			}
		}
	}
}
