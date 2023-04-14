//! `HashMap` using pointers as keys

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::ptr;

use rustc_hash::FxHashMap;

/// `HashMap` using pointers as keys
pub struct RefMap<'a, K, V> {
	map: FxHashMap<KeyRef<'a, K>, V>,
}

impl<'a, K, V> Default for RefMap<'a, K, V> {
	fn default() -> Self {
		Self {
			map: FxHashMap::default(),
		}
	}
}

impl<'a, K, V> RefMap<'a, K, V> {
	/// Create a new `RefMap`
	pub fn new() -> Self {
		Self {
			map: HashMap::default(),
		}
	}

	/// The number of entries in the map
	pub fn len(&self) -> usize {
		self.map.len()
	}

	/// Whether this map is empty
	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	/// Whether this map contains the given key
	pub fn contains_key(&self, key: &'a K) -> bool {
		self.map.contains_key(&KeyRef(key))
	}

	/// Insert a key-value pair into the map
	pub fn insert(&mut self, key: &'a K, value: V) {
		self.map.insert(KeyRef(key), value);
	}

	/// Get the value for this key
	pub fn get(&self, key: &'a K) -> Option<&V> {
		self.map.get(&KeyRef(key))
	}

	/// Get an iterator over key-value pairs
	pub fn iter(&self) -> impl Iterator<Item = (&'a K, &V)> {
		self.map.iter().map(|(k, v)| (k.0, v))
	}

	/// Get an iterator over key-value pairs
	pub fn iter_mut(&mut self) -> impl Iterator<Item = (&'a K, &mut V)> {
		self.map.iter_mut().map(|(k, v)| (k.0, v))
	}

	/// Get the keys in the map
	pub fn keys(&self) -> impl Iterator<Item = &'a K> + '_ {
		self.map.keys().map(move |k| k.0)
	}

	/// Get an iterator of values
	pub fn values(&self) -> impl Iterator<Item = &V> {
		self.map.values()
	}

	/// Get an iterator of mutable values
	pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
		self.map.values_mut()
	}
}

impl<'a, K, V> Extend<(&'a K, V)> for RefMap<'a, K, V> {
	fn extend<T: IntoIterator<Item = (&'a K, V)>>(&mut self, iter: T) {
		self.map
			.extend(iter.into_iter().map(|(k, v)| (KeyRef(k), v)))
	}
}

impl<'a, K, V> Index<&'a K> for RefMap<'a, K, V> {
	type Output = V;
	fn index(&self, index: &'a K) -> &Self::Output {
		&self.map[&KeyRef(index)]
	}
}

struct KeyRef<'a, T>(&'a T);

impl<'a, T> PartialEq for KeyRef<'a, T> {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self.0, other.0)
	}
}

impl<'a, T> Hash for KeyRef<'a, T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		ptr::hash(self.0, state)
	}
}

impl<'a, T> Eq for KeyRef<'a, T> {}
