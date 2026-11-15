//! Collection types.
//!
//! Re-exports the collections provided by `alloc` and adds a `HashMap`
//! implementation so that code written for `std::collections` can also be
//! compiled against `axstd`.

pub use alloc::collections::{
    binary_heap, btree_map, btree_set, linked_list, vec_deque, BTreeMap, BTreeSet, BinaryHeap,
    LinkedList, VecDeque,
};
use core::borrow::Borrow;

/// A map collection backed by a `BTreeMap`.
///
/// The interface mirrors the commonly used subset of `std::collections::HashMap`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HashMap<K, V> {
    inner: BTreeMap<K, V>,
}

impl<K, V> HashMap<K, V> {
    /// Creates an empty `HashMap`.
    pub const fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Creates an empty `HashMap` (capacity is ignored).
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears the map, removing all elements.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<K: Ord, V> HashMap<K, V> {
    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.inner.get_mut(key)
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.inner.contains_key(key)
    }

    /// Removes a key from the map, returning the value at the key if it existed.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.inner.remove(key)
    }

    /// An iterator visiting all key-value pairs in order.
    pub fn iter(&self) -> alloc::collections::btree_map::Iter<'_, K, V> {
        self.inner.iter()
    }

    /// An iterator visiting all key-value pairs in order, with mutable values.
    pub fn iter_mut(&mut self) -> alloc::collections::btree_map::IterMut<'_, K, V> {
        self.inner.iter_mut()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for HashMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<K: Ord, V> Extend<(K, V)> for HashMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = alloc::collections::btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}
