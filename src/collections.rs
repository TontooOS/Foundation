//! Collections – Array, Dictionary, Set with Foundation-like API

use crate::error::Result;
use std::collections::HashMap;

/// NSArray equivalent – ordered collection
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Array<T> {
    items: Vec<T>,
}

impl<T> Array<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<T>) -> Self {
        Self { items }
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }

    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    pub fn remove_last(&mut self) -> Option<T> {
        self.items.pop()
    }

    pub fn insert(&mut self, index: usize, item: T) {
        if index <= self.items.len() {
            self.items.insert(index, item);
        }
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.items.contains(item)
    }

    pub fn index_of(&self, item: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.items.iter().position(|x| x == item)
    }

    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool,
        T: Clone,
    {
        Self { items: self.items.iter().filter(|x| predicate(x)).cloned().collect() }
    }

    pub fn map<F, U>(&self, f: F) -> Array<U>
    where
        F: Fn(&T) -> U,
    {
        Array { items: self.items.iter().map(f).collect() }
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> std::cmp::Ordering,
    {
        self.items.sort_by(compare);
    }

    pub fn reversed(&self) -> Self
    where
        T: Clone,
    {
        let mut items = self.items.clone();
        items.reverse();
        Self { items }
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    pub fn to_vec(&self) -> &Vec<T> {
        &self.items
    }
}

impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T> From<Vec<T>> for Array<T> {
    fn from(v: Vec<T>) -> Self {
        Self::from_vec(v)
    }
}

/// NSDictionary equivalent – key-value collection
#[derive(Debug, Clone, Default)]
pub struct Dictionary<K, V> {
    map: HashMap<K, V>,
}

impl<K: std::hash::Hash + Eq, V> Dictionary<K, V> {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn from_map(map: HashMap<K, V>) -> Self {
        Self { map }
    }

    pub fn count(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.map.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn keys(&self) -> Vec<&K> {
        self.map.keys().collect()
    }

    pub fn values(&self) -> Vec<&V> {
        self.map.values().collect()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<K, V> {
        self.map.iter()
    }

    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&K, &V) -> bool,
        K: Clone,
        V: Clone,
    {
        Self {
            map: self.map.iter()
                .filter(|(k, v)| predicate(k, v))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

impl<K: std::hash::Hash + Eq, V> From<HashMap<K, V>> for Dictionary<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Self::from_map(map)
    }
}

/// NSSet equivalent – unordered unique collection
#[derive(Debug, Clone, Default)]
pub struct Set<T> {
    items: std::collections::HashSet<T>,
}

impl<T: std::hash::Hash + Eq> Set<T> {
    pub fn new() -> Self {
        Self { items: std::collections::HashSet::new() }
    }

    pub fn from_set(items: std::collections::HashSet<T>) -> Self {
        Self { items }
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn insert(&mut self, item: T) -> bool {
        self.items.insert(item)
    }

    pub fn remove(&mut self, item: &T) -> bool {
        self.items.remove(item)
    }

    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    pub fn union(&self, other: &Self) -> Self
    where
        T: Clone,
    {
        Self { items: self.items.union(&other.items).cloned().collect() }
    }

    pub fn intersection(&self, other: &Self) -> Self
    where
        T: Clone,
    {
        Self { items: self.items.intersection(&other.items).cloned().collect() }
    }

    pub fn difference(&self, other: &Self) -> Self
    where
        T: Clone,
    {
        Self { items: self.items.difference(&other.items).cloned().collect() }
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.items.is_subset(&other.items)
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        self.items.is_superset(&other.items)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<T> {
        self.items.iter()
    }
}

impl<T: std::hash::Hash + Eq> From<std::collections::HashSet<T>> for Set<T> {
    fn from(s: std::collections::HashSet<T>) -> Self {
        Self::from_set(s)
    }
}
