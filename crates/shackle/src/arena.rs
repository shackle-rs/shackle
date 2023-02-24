//! A simple vector-based arena for allocating expressions

use std::{
	any::type_name,
	fmt::Debug,
	hash::Hash,
	marker::PhantomData,
	num::NonZeroU32,
	ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo},
};

/// Index of a member in an Arena
pub struct ArenaIndex<T> {
	index: NonZeroU32,
	phantom: PhantomData<T>,
}

impl<T> Copy for ArenaIndex<T> {}

impl<T> Clone for ArenaIndex<T> {
	fn clone(&self) -> Self {
		Self {
			index: self.index,
			phantom: PhantomData,
		}
	}
}

impl<T> Debug for ArenaIndex<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let t = type_name::<T>()
			.split('<')
			.map(|p| {
				if let Some(i) = p.rfind(':') {
					&p[i + 1..]
				} else {
					p
				}
			})
			.collect::<Vec<_>>()
			.join("<");
		write!(f, "<{}::{}>", t, self.index.get())
	}
}

impl<T> Hash for ArenaIndex<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.index.hash(state)
	}
}

impl<T> PartialEq for ArenaIndex<T> {
	fn eq(&self, other: &Self) -> bool {
		self.index.eq(&other.index)
	}
}

impl<T> Eq for ArenaIndex<T> {}

impl<T> PartialOrd for ArenaIndex<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.index.partial_cmp(&other.index)
	}
}

impl<T> Ord for ArenaIndex<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.index.cmp(&other.index)
	}
}

impl<T> ArenaIndex<T> {
	fn new(raw: u32) -> Self {
		Self {
			index: NonZeroU32::new(raw).expect("Expected non-zero index"),
			phantom: PhantomData,
		}
	}
}

impl<T> From<ArenaIndex<T>> for u32 {
	fn from(i: ArenaIndex<T>) -> Self {
		i.index.get()
	}
}

/// A vector-based single-type arena
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Arena<T> {
	items: Vec<T>,
	forbidden: Option<ArenaIndex<T>>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Arena<T> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		fmt.debug_struct("Arena")
			.field("len", &self.len())
			.field("data", &self.items)
			.finish()
	}
}

impl<T> std::default::Default for Arena<T> {
	fn default() -> Self {
		Self {
			items: Vec::new(),
			forbidden: None,
		}
	}
}

impl<T> Arena<T> {
	/// Create a new arena.
	pub fn new() -> Arena<T> {
		Self::default()
	}

	/// Clear the arena.
	pub fn clear(&mut self) {
		self.items.clear();
	}

	/// Get the length of the arena.
	pub fn len(&self) -> u32 {
		self.items.len() as u32
	}

	/// Return whether the arena is empty.
	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	/// Allocate a value in the arena.
	pub fn insert<V: Into<T>>(&mut self, value: V) -> ArenaIndex<T> {
		self.items.push(value.into());
		ArenaIndex::new(self.len())
	}

	/// Get a reference to a value in the arena by its index if it exists.
	pub fn get(&self, idx: ArenaIndex<T>) -> Option<&T> {
		self.items.get((idx.index.get() - 1) as usize)
	}

	/// Get a mutable reference to a value in the arena by its index if it exists.
	pub fn get_mut(&mut self, idx: ArenaIndex<T>) -> Option<&mut T> {
		self.items.get_mut((idx.index.get() - 1) as usize)
	}

	/// Get an iterator over the keys in this arena.
	pub fn keys(&self) -> impl Iterator<Item = ArenaIndex<T>> {
		(1..=self.len()).map(ArenaIndex::new)
	}

	/// Get an iterator over the values allocated in this arena.
	pub fn values(&self) -> impl Iterator<Item = &T> {
		self.items.iter()
	}

	/// Get an iterator over the mutable values allocated in this arena.
	pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
		self.items.iter_mut()
	}

	/// Get an iterator of pairs of `ArenaIndex` and values allocated in this arena.
	pub fn iter(&self) -> impl Iterator<Item = (ArenaIndex<T>, &T)> {
		self.items
			.iter()
			.enumerate()
			.map(|(idx, o)| (ArenaIndex::new((idx + 1) as u32), o))
	}

	/// Get an iterator of pairs of `ArenaIndex` and values allocated in this arena.
	pub fn iter_mut(&mut self) -> impl Iterator<Item = (ArenaIndex<T>, &mut T)> {
		self.items
			.iter_mut()
			.enumerate()
			.map(|(idx, o)| (ArenaIndex::new((idx + 1) as u32), o))
	}

	/// Shrink arena so capacity is minimized.
	pub fn shrink_to_fit(&mut self) {
		self.items.shrink_to_fit();
	}
}

impl<T> Index<ArenaIndex<T>> for Arena<T> {
	type Output = T;
	fn index(&self, idx: ArenaIndex<T>) -> &T {
		&self.items[(idx.index.get() - 1) as usize]
	}
}

impl<T> IndexMut<ArenaIndex<T>> for Arena<T> {
	fn index_mut(&mut self, idx: ArenaIndex<T>) -> &mut T {
		&mut self.items[(idx.index.get() - 1) as usize]
	}
}

impl<T> Index<Range<ArenaIndex<T>>> for Arena<T> {
	type Output = [T];
	fn index(&self, idx: Range<ArenaIndex<T>>) -> &[T] {
		let start = (idx.start.index.get() - 1) as usize;
		let end = (idx.end.index.get() - 1) as usize;
		&self.items[start..end]
	}
}

impl<T> IndexMut<Range<ArenaIndex<T>>> for Arena<T> {
	fn index_mut(&mut self, idx: Range<ArenaIndex<T>>) -> &mut [T] {
		let start = (idx.start.index.get() - 1) as usize;
		let end = (idx.end.index.get() - 1) as usize;
		&mut self.items[start..end]
	}
}

impl<T> Index<RangeFrom<ArenaIndex<T>>> for Arena<T> {
	type Output = [T];
	fn index(&self, idx: RangeFrom<ArenaIndex<T>>) -> &[T] {
		let start = (idx.start.index.get() - 1) as usize;
		&self.items[start..]
	}
}

impl<T> IndexMut<RangeFrom<ArenaIndex<T>>> for Arena<T> {
	fn index_mut(&mut self, idx: RangeFrom<ArenaIndex<T>>) -> &mut [T] {
		let start = (idx.start.index.get() - 1) as usize;
		&mut self.items[start..]
	}
}

impl<T> Index<RangeFull> for Arena<T> {
	type Output = [T];
	fn index(&self, idx: RangeFull) -> &[T] {
		&self.items[idx]
	}
}

impl<T> IndexMut<RangeFull> for Arena<T> {
	fn index_mut(&mut self, idx: RangeFull) -> &mut [T] {
		&mut self.items[idx]
	}
}

impl<T> Index<RangeInclusive<ArenaIndex<T>>> for Arena<T> {
	type Output = [T];
	fn index(&self, idx: RangeInclusive<ArenaIndex<T>>) -> &[T] {
		let start = (idx.start().index.get() - 1) as usize;
		let end = (idx.end().index.get() - 1) as usize;
		&self.items[start..=end]
	}
}

impl<T> IndexMut<RangeInclusive<ArenaIndex<T>>> for Arena<T> {
	fn index_mut(&mut self, idx: RangeInclusive<ArenaIndex<T>>) -> &mut [T] {
		let start = (idx.start().index.get() - 1) as usize;
		let end = (idx.end().index.get() - 1) as usize;
		&mut self.items[start..=end]
	}
}

impl<T> Index<RangeTo<ArenaIndex<T>>> for Arena<T> {
	type Output = [T];
	fn index(&self, idx: RangeTo<ArenaIndex<T>>) -> &[T] {
		let end = (idx.end.index.get() - 1) as usize;
		&self.items[..end]
	}
}

impl<T> IndexMut<RangeTo<ArenaIndex<T>>> for Arena<T> {
	fn index_mut(&mut self, idx: RangeTo<ArenaIndex<T>>) -> &mut [T] {
		let end = (idx.end.index.get() - 1) as usize;
		&mut self.items[..end]
	}
}

/// A mapping between `ArenaIndex`es and values
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ArenaMap<K, V> {
	items: Vec<Option<V>>,
	phantom: PhantomData<K>,
}

impl<K, V: std::fmt::Debug> std::fmt::Debug for ArenaMap<K, V> {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut dbg = fmt.debug_map();
		let items = self.items.iter().enumerate().filter_map(|(i, v)| {
			v.as_ref()
				.map(|x| (ArenaIndex::<K>::new((i + 1) as u32), x))
		});
		for (k, v) in items {
			dbg.entry(&k, &v);
		}
		dbg.finish()
	}
}

impl<K, V> std::default::Default for ArenaMap<K, V> {
	fn default() -> Self {
		Self {
			items: Vec::new(),
			phantom: PhantomData,
		}
	}
}

impl<K, V> ArenaMap<K, V> {
	/// Create a new arena map.
	pub fn new() -> ArenaMap<K, V> {
		Self::default()
	}

	/// Clear the arena map.
	pub fn clear(&mut self) {
		self.items.clear();
	}

	/// Return whether the arena map is empty.
	pub fn is_empty(&self) -> bool {
		self.items.is_empty() || self.items.iter().all(|v| v.is_none())
	}

	/// Insert a key-value pair into the map
	pub fn insert(&mut self, k: ArenaIndex<K>, v: V) {
		let i = (k.index.get() - 1) as usize;
		self.items
			.resize_with((i + 1).max(self.items.len()), || None);
		self.items[i] = Some(v);
	}

	/// Get a reference to a value in the arena map by its key if it exists.
	pub fn get(&self, idx: ArenaIndex<K>) -> Option<&V> {
		match self.items.get((idx.index.get() - 1) as usize) {
			Some(v) => v.as_ref(),
			None => None,
		}
	}

	/// Get a mutable reference to a value in the arena map by its key if it exists.
	pub fn get_mut(&mut self, idx: ArenaIndex<K>) -> Option<&mut V> {
		match self.items.get_mut((idx.index.get() - 1) as usize) {
			Some(v) => v.as_mut(),
			None => None,
		}
	}

	/// Get a mutable reference to a value in the arena map, inserting the default
	/// value if it is not present.
	pub fn get_mut_or_default(&mut self, idx: ArenaIndex<K>) -> &mut V
	where
		V: Default,
	{
		if self.items.get((idx.index.get() - 1) as usize).is_none() {
			self.insert(idx, V::default());
		}
		self.get_mut(idx).unwrap()
	}

	/// Get an iterator over the values in the map.
	pub fn values(&self) -> impl Iterator<Item = &V> {
		self.items.iter().filter_map(|v| v.as_ref())
	}

	/// Get an iterator over the mutable values in the map.
	pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
		self.items.iter_mut().filter_map(|v| v.as_mut())
	}

	/// Get an iterator of pairs of `ArenaIndex` and values allocated in this arena.
	pub fn iter(&self) -> impl Iterator<Item = (ArenaIndex<K>, &V)> {
		self.items
			.iter()
			.enumerate()
			.filter_map(|(idx, o)| o.as_ref().map(|v| (ArenaIndex::new((idx + 1) as u32), v)))
	}

	/// Consume and iterate
	#[allow(clippy::should_implement_trait)] // TODO: How can we implement this as IntoIter? Somehow the resulting type is almost impossible to express.
	pub fn into_iter(self) -> impl Iterator<Item = (ArenaIndex<K>, V)> {
		self.items
			.into_iter()
			.enumerate()
			.filter_map(|(idx, o)| o.map(|v| (ArenaIndex::new((idx + 1) as u32), v)))
	}

	/// Shrink arena map so capacity is minimized.
	pub fn shrink_to_fit(&mut self) {
		self.items.shrink_to_fit();
	}
}

impl<K, V> Index<ArenaIndex<K>> for ArenaMap<K, V> {
	type Output = V;
	fn index(&self, idx: ArenaIndex<K>) -> &V {
		self.get(idx)
			.unwrap_or_else(|| panic!("No entry for {:?} in ArenaMap", idx))
	}
}

impl<K, V> IndexMut<ArenaIndex<K>> for ArenaMap<K, V> {
	fn index_mut(&mut self, idx: ArenaIndex<K>) -> &mut V {
		self.get_mut(idx)
			.unwrap_or_else(|| panic!("No entry for {:?} in ArenaMap", idx))
	}
}

impl<K, V> FromIterator<(ArenaIndex<K>, V)> for ArenaMap<K, V> {
	fn from_iter<T: IntoIterator<Item = (ArenaIndex<K>, V)>>(iter: T) -> Self {
		let mut result = Self::new();
		for (k, v) in iter {
			result.insert(k, v);
		}
		result
	}
}
