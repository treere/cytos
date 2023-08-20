//! Map structs.
//!
//! This module contains maps that should be optimized for small data:
//!
//! *[`VecMap`] is a map that use a vector to store data

use std::ops::{Index, IndexMut};

/// A Map using a Vector to data.
#[derive(Debug)]
pub struct VecMap<K, V> {
    data: Vec<(K, V)>,
}

impl<K, V> VecMap<K, V> {
    /// Creates a new map.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a new map from an iterator.
    pub fn from_iterator(it: impl Iterator<Item = (K, V)>) -> Self {
        Self { data: it.collect() }
    }

    /// Inserts a key-value pair into that map
    pub fn insert(&mut self, k: K, v: V) {
        self.data.push((k, v))
    }
}

impl<K, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq, V> VecMap<K, V> {
    /// Get a value from the map
    pub fn get(&self, k: &K) -> Option<&V> {
        self.data.iter().find(|(o, _)| o == k).map(|(_, v)| v)
    }

    /// Get a mutable value from a map
    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.data.iter_mut().find(|(o, _)| o == k).map(|(_, v)| v)
    }
}

impl<K: PartialEq, V> Index<K> for VecMap<K, V> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<K: PartialEq, V> IndexMut<K> for VecMap<K, V> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}
