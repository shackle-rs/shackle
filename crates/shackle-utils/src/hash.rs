//! `FxHashMap` and `FxHashSet` wrappers.

use std::{
	borrow::Borrow,
	fmt::{Debug, Formatter},
	hash::{DefaultHasher, Hash, Hasher},
	ops::{BitAnd, BitOr, BitXor, Deref, DerefMut, Index, Sub},
};

use rustc_hash::{FxHashMap, FxHashSet};

/// `FxHashMap` with deterministic `Debug` output.
///
/// Entries are rendered in ascending order by the `Debug` representation of
/// the key, which makes snapshot tests stable.
///
/// Care must still be taken to not iterate over the map without sorting since this is non-deterministic.
#[derive(Clone, salsa::SalsaValue)]
pub struct Map<K, V>(FxHashMap<K, V>);

impl<K, V> PartialEq for Map<K, V>
where
	K: Eq + Hash,
	V: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<K, V> Eq for Map<K, V>
where
	K: Eq + Hash,
	V: Eq,
{
}

impl<K, V> PartialEq<FxHashMap<K, V>> for Map<K, V>
where
	K: Eq + Hash,
	V: PartialEq,
{
	fn eq(&self, other: &FxHashMap<K, V>) -> bool {
		&self.0 == other
	}
}

impl<K, V> PartialEq<Map<K, V>> for FxHashMap<K, V>
where
	K: Eq + Hash,
	V: PartialEq,
{
	fn eq(&self, other: &Map<K, V>) -> bool {
		self == &other.0
	}
}

impl<K, V> Hash for Map<K, V>
where
	K: Eq + Hash,
	V: Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut entries = self
			.0
			.iter()
			.map(|entry| {
				let mut hasher = DefaultHasher::new();
				entry.hash(&mut hasher);
				hasher.finish()
			})
			.collect::<Vec<_>>();
		entries.sort_unstable();

		self.0.len().hash(state);
		entries.hash(state);
	}
}

impl<K, V> Default for Map<K, V> {
	fn default() -> Self {
		Self(FxHashMap::default())
	}
}

impl<K, V> Map<K, V> {
	/// Create an empty map.
	pub fn new() -> Self {
		Self::default()
	}

	/// Create an iterator visiting all keys in arbitrary order.
	pub fn into_keys(self) -> impl Iterator<Item = K> {
		self.0.into_keys()
	}

	/// Create an iterator visiting all values in arbitrary order.
	pub fn into_values(self) -> impl Iterator<Item = V> {
		self.0.into_values()
	}

	/// Consume the wrapper and return the inner map.
	pub fn into_inner(self) -> FxHashMap<K, V> {
		self.0
	}
}

impl<K, V> From<FxHashMap<K, V>> for Map<K, V> {
	fn from(value: FxHashMap<K, V>) -> Self {
		Self(value)
	}
}

impl<K, V> From<Map<K, V>> for FxHashMap<K, V> {
	fn from(value: Map<K, V>) -> Self {
		value.0
	}
}

impl<K: Eq + Hash, V, const N: usize> From<[(K, V); N]> for Map<K, V> {
	fn from(value: [(K, V); N]) -> Self {
		Self(value.into_iter().collect())
	}
}

impl<K: Eq + Hash, V> FromIterator<(K, V)> for Map<K, V> {
	fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
		Self(FxHashMap::from_iter(iter))
	}
}

impl<K: Eq + Hash, V> Extend<(K, V)> for Map<K, V> {
	fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
		self.0.extend(iter);
	}
}

impl<K, V> IntoIterator for Map<K, V> {
	type IntoIter = <FxHashMap<K, V> as IntoIterator>::IntoIter;
	type Item = (K, V);

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
	type IntoIter = <&'a FxHashMap<K, V> as IntoIterator>::IntoIter;
	type Item = (&'a K, &'a V);

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<'a, K, V> IntoIterator for &'a mut Map<K, V> {
	type IntoIter = <&'a mut FxHashMap<K, V> as IntoIterator>::IntoIter;
	type Item = (&'a K, &'a mut V);

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter_mut()
	}
}

impl<K, V> Deref for Map<K, V> {
	type Target = FxHashMap<K, V>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<K, V> DerefMut for Map<K, V> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<K, V> AsRef<FxHashMap<K, V>> for Map<K, V> {
	fn as_ref(&self) -> &FxHashMap<K, V> {
		&self.0
	}
}

impl<K, V> AsMut<FxHashMap<K, V>> for Map<K, V> {
	fn as_mut(&mut self) -> &mut FxHashMap<K, V> {
		&mut self.0
	}
}

impl<K, Q, V> Index<&Q> for Map<K, V>
where
	K: Eq + Hash + Borrow<Q>,
	Q: Eq + Hash + ?Sized,
{
	type Output = V;

	fn index(&self, index: &Q) -> &Self::Output {
		&self.0[index]
	}
}

impl<K: Debug, V: Debug> Debug for Map<K, V> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut entries = self
			.0
			.iter()
			.map(|(k, v)| (format!("{k:?}"), k, v))
			.collect::<Vec<_>>();
		entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

		let mut map = f.debug_map();
		for (_, key, value) in entries {
			let _ = map.entry(key, value);
		}
		map.finish()
	}
}

/// `FxHashSet` with deterministic `Debug` output.
///
/// Entries are rendered in ascending order by the `Debug` representation of
/// each item, which makes snapshot tests stable.
///
/// Care must still be taken to not iterate over the set without sorting since this is non-deterministic.
#[derive(Clone, salsa::SalsaValue)]
pub struct Set<T>(FxHashSet<T>);

impl<T> PartialEq for Set<T>
where
	T: Eq + Hash,
{
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl<T> Eq for Set<T> where T: Eq + Hash {}

impl<T> PartialEq<FxHashSet<T>> for Set<T>
where
	T: Eq + Hash,
{
	fn eq(&self, other: &FxHashSet<T>) -> bool {
		&self.0 == other
	}
}

impl<T> PartialEq<Set<T>> for FxHashSet<T>
where
	T: Eq + Hash,
{
	fn eq(&self, other: &Set<T>) -> bool {
		self == &other.0
	}
}

impl<T> Hash for Set<T>
where
	T: Eq + Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) {
		let mut entries = self
			.0
			.iter()
			.map(|entry| {
				let mut hasher = DefaultHasher::new();
				entry.hash(&mut hasher);
				hasher.finish()
			})
			.collect::<Vec<_>>();
		entries.sort_unstable();

		self.0.len().hash(state);
		entries.hash(state);
	}
}

impl<T> Default for Set<T> {
	fn default() -> Self {
		Self(FxHashSet::default())
	}
}

impl<T> Set<T> {
	/// Create an empty set.
	pub fn new() -> Self {
		Self::default()
	}

	/// Consume the wrapper and return the inner set.
	pub fn into_inner(self) -> FxHashSet<T> {
		self.0
	}
}

impl<T> From<FxHashSet<T>> for Set<T> {
	fn from(value: FxHashSet<T>) -> Self {
		Self(value)
	}
}

impl<T> From<Set<T>> for FxHashSet<T> {
	fn from(value: Set<T>) -> Self {
		value.0
	}
}

impl<T: Eq + Hash, const N: usize> From<[T; N]> for Set<T> {
	fn from(value: [T; N]) -> Self {
		Self(value.into_iter().collect())
	}
}

impl<T: Eq + Hash> FromIterator<T> for Set<T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		Self(FxHashSet::from_iter(iter))
	}
}

impl<T: Eq + Hash> Extend<T> for Set<T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		self.0.extend(iter);
	}
}

impl<T> IntoIterator for Set<T> {
	type IntoIter = <FxHashSet<T> as IntoIterator>::IntoIter;
	type Item = T;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T> IntoIterator for &'a Set<T> {
	type IntoIter = <&'a FxHashSet<T> as IntoIterator>::IntoIter;
	type Item = &'a T;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<T> Deref for Set<T> {
	type Target = FxHashSet<T>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for Set<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> AsRef<FxHashSet<T>> for Set<T> {
	fn as_ref(&self) -> &FxHashSet<T> {
		&self.0
	}
}

impl<T> AsMut<FxHashSet<T>> for Set<T> {
	fn as_mut(&mut self) -> &mut FxHashSet<T> {
		&mut self.0
	}
}

impl<T> BitOr<&Set<T>> for &Set<T>
where
	T: Eq + Hash + Clone,
{
	type Output = Set<T>;

	fn bitor(self, rhs: &Set<T>) -> Self::Output {
		Set(self.0.union(&rhs.0).cloned().collect())
	}
}

impl<T> BitAnd<&Set<T>> for &Set<T>
where
	T: Eq + Hash + Clone,
{
	type Output = Set<T>;

	fn bitand(self, rhs: &Set<T>) -> Self::Output {
		Set(self.0.intersection(&rhs.0).cloned().collect())
	}
}

impl<T> BitXor<&Set<T>> for &Set<T>
where
	T: Eq + Hash + Clone,
{
	type Output = Set<T>;

	fn bitxor(self, rhs: &Set<T>) -> Self::Output {
		Set(self.0.symmetric_difference(&rhs.0).cloned().collect())
	}
}

impl<T> Sub<&Set<T>> for &Set<T>
where
	T: Eq + Hash + Clone,
{
	type Output = Set<T>;

	fn sub(self, rhs: &Set<T>) -> Self::Output {
		Set(self.0.difference(&rhs.0).cloned().collect())
	}
}

impl<T: Debug> Debug for Set<T> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut entries = self
			.0
			.iter()
			.map(|item| (format!("{item:?}"), item))
			.collect::<Vec<_>>();
		entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

		let mut set = f.debug_set();
		for (_, item) in entries {
			let _ = set.entry(item);
		}
		set.finish()
	}
}

#[cfg(test)]
mod tests {
	use std::hash::{DefaultHasher, Hash, Hasher};

	use super::{Map, Set};

	#[test]
	fn debug_is_sorted_by_key_debug_representation() {
		let mut map = Map::new();
		let _ = map.insert("z", 0);
		let _ = map.insert("a", 1);
		let _ = map.insert("m", 2);

		assert_eq!(format!("{map:?}"), r#"{"a": 1, "m": 2, "z": 0}"#);
	}

	#[test]
	fn debug_set_is_sorted_by_item_debug_representation() {
		let mut set = Set::new();
		let _ = set.insert("z");
		let _ = set.insert("a");
		let _ = set.insert("m");

		assert_eq!(format!("{set:?}"), r#"{"a", "m", "z"}"#);
	}

	#[test]
	fn map_equality_and_hash_are_order_independent() {
		let mut a = Map::new();
		let _ = a.insert("z", 0);
		let _ = a.insert("a", 1);

		let mut b = Map::new();
		let _ = b.insert("a", 1);
		let _ = b.insert("z", 0);

		assert_eq!(a, b);

		let mut ah = DefaultHasher::new();
		a.hash(&mut ah);
		let mut bh = DefaultHasher::new();
		b.hash(&mut bh);
		assert_eq!(ah.finish(), bh.finish());
	}

	#[test]
	fn set_equality_and_hash_are_order_independent() {
		let mut a = Set::new();
		let _ = a.insert("z");
		let _ = a.insert("a");

		let mut b = Set::new();
		let _ = b.insert("a");
		let _ = b.insert("z");

		assert_eq!(a, b);

		let mut ah = DefaultHasher::new();
		a.hash(&mut ah);
		let mut bh = DefaultHasher::new();
		b.hash(&mut bh);
		assert_eq!(ah.finish(), bh.finish());
	}

	#[test]
	fn map_from_array_and_index() {
		let map = Map::from([("a", 1), ("b", 2)]);
		assert_eq!(map["a"], 1);
	}

	#[test]
	fn set_ops_and_from_array() {
		let a = Set::from(["a", "b", "c"]);
		let b = Set::from(["b", "c", "d"]);

		let union = &a | &b;
		let intersection = &a & &b;
		let symmetric_difference = &a ^ &b;
		let difference = &a - &b;

		assert_eq!(union, Set::from(["a", "b", "c", "d"]));
		assert_eq!(intersection, Set::from(["b", "c"]));
		assert_eq!(symmetric_difference, Set::from(["a", "d"]));
		assert_eq!(difference, Set::from(["a"]));
	}
}
