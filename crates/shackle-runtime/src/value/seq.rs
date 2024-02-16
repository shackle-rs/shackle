use std::ops::Index;

use bilge::{
	arbitrary_int::{u14, u2},
	bitsize,
	prelude::Number,
	Bitsized, DebugBits, FromBits,
};
use itertools::Itertools;

use super::{DataView, Value};

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

impl<'a> SeqView<'a> {
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

impl<'a, T> Index<usize> for Pairs<'a, T> {
	type Output = [T];

	fn index(&self, index: usize) -> &Self::Output {
		&self.0[index * 2..=index * 2 + 1]
	}
}

#[cfg(test)]
mod tests {
	use crate::value::seq::{InnerViewType, ViewType};

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
