# Collections

Collection types providing Apple Foundation-like ordered, keyed, and unique data structures for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `Array<T>` | Ordered collection (NSArray equivalent) |
| `Dictionary<K, V>` | Key-value collection (NSDictionary equivalent) |
| `Set<T>` | Unordered unique collection (NSSet equivalent) |

## Array

Ordered, indexable collection wrapping `Vec<T>`.

```rust
pub struct Array<T> {
    items: Vec<T>,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn from_vec(items: Vec<T>) -> Self
```

### Accessors

```rust
pub fn count(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn get(&self, index: usize) -> Option<&T>
pub fn first(&self) -> Option<&T>
pub fn last(&self) -> Option<&T>
pub fn iter(&self) -> std::slice::Iter<T>
pub fn to_vec(&self) -> &Vec<T>
```

### Mutators

```rust
pub fn add(&mut self, item: T)
pub fn remove(&mut self, index: usize) -> Option<T>
pub fn remove_last(&mut self) -> Option<T>
pub fn insert(&mut self, index: usize, item: T)
```

### Query

```rust
pub fn contains(&self, item: &T) -> bool
pub fn index_of(&self, item: &T) -> Option<usize>
pub fn filter<F>(&self, predicate: F) -> Self
pub fn map<F, U>(&self, f: F) -> Array<U>
pub fn sort_by<F>(&mut self, compare: F)
pub fn reversed(&self) -> Self
```

All query methods require `T: PartialEq` or `T: Clone` as appropriate.

## Dictionary

Key-value collection wrapping `HashMap<K, V>`.

```rust
pub struct Dictionary<K, V> {
    map: HashMap<K, V>,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn from_map(map: HashMap<K, V>) -> Self
```

### Accessors

```rust
pub fn count(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn get(&self, key: &K) -> Option<&V>
pub fn get_mut(&mut self, key: &K) -> Option<&mut V>
pub fn keys(&self) -> Vec<&K>
pub fn values(&self) -> Vec<&V>
pub fn iter(&self) -> std::collections::hash_map::Iter<K, V>
```

### Mutators

```rust
pub fn set(&mut self, key: K, value: V)
pub fn remove(&mut self, key: &K) -> Option<V>
pub fn contains_key(&self, key: &K) -> bool
pub fn filter<F>(&self, predicate: F) -> Self
```

Requires `K: std::hash::Hash + Eq`.

## Set

Unordered unique collection wrapping `HashSet<T>`.

```rust
pub struct Set<T> {
    items: std::collections::HashSet<T>,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn from_set(items: std::collections::HashSet<T>) -> Self
```

### Accessors

```rust
pub fn count(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn contains(&self, item: &T) -> bool
pub fn iter(&self) -> std::collections::hash_set::Iter<T>
```

### Mutators

```rust
pub fn insert(&mut self, item: T) -> bool
pub fn remove(&mut self, item: &T) -> bool
```

### Set Operations

```rust
pub fn union(&self, other: &Self) -> Self
pub fn intersection(&self, other: &Self) -> Self
pub fn difference(&self, other: &Self) -> Self
pub fn is_subset(&self, other: &Self) -> bool
pub fn is_superset(&self, other: &Self) -> bool
```

All set operations require `T: Clone`.

## Usage

```rust
use tontoo_foundation::prelude::*;

// Array
let mut arr = Array::new();
arr.add(1);
arr.add(2);
arr.add(3);
assert_eq!(arr.count(), 3);
let doubled = arr.map(|x| x * 2);
assert_eq!(doubled.get(0), Some(&2));

// Dictionary
let mut dict: Dictionary<String, i32> = Dictionary::new();
dict.set("one".to_string(), 1);
assert_eq!(dict.get(&"one".to_string()), Some(&1));

// Set
let mut set = Set::new();
set.insert(1);
set.insert(2);
set.insert(1); // duplicate ignored
assert_eq!(set.count(), 2);
```

## Cross References

- [String.md](String.md) - TString for string-based keys
- [Predicate.md](Predicate.md) - Sorting and filtering collections
